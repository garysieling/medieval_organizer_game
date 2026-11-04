//! Balance test mode: run the real game headless and mark it.
//!
//! `cargo run --release --bin balance_test [days] [seed]`
//!
//! The Wanderer's seat is driven by a light organizing policy - talk to
//! whoever's in reach, recruit once the bond's strong enough, call a
//! meeting when a few are ready - so org-growth and reward KPIs measure
//! an actual player, not just the village running on its own. Each day's
//! gauges and log lines feed the BalanceTester's KPI table (see
//! wiki/literature-reviews/ for where the bands come from).
//!
//! Exits 1 when any KPI red-flags, so it can gate a schedule like the
//! playtest gate does.

use game2_core::session::{driver, Command, Game};
use game2_core::agents::balance_tester::{BalanceTester, DaySample, RunRecord};
use game2_core::sim::community::{Organization, RECRUIT_MIN_TRUST};
use game2_core::game::relationships::{self, Relationship};
use game2_core::game::villager::{self, Hunger};
use game2_core::game::livelihood::Purse;

/// The seat's bond with a villager, as the sim reads it.
fn bond(game: &Game, seat: usize, v: game2_core::engine::ecs::Entity) -> f32 {
    relationships::find_relationship(&game.world, game.seats[seat].player, v)
        .and_then(|r| game.world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0)
}

/// A light organizing policy for the Wanderer seat: keep body and gold
/// together via the idle driver, and spend the rest of the attention on
/// the work the game is about - talk, recruit, the odd meeting.
/// Press a ctx button by name - ctx_act_at wants its index in the list
fn act_ctx(game: &mut Game, seat: usize, act: game2_core::session::CtxAct) {
    if let Some(i) = game.seats[seat].ctx_actions.iter().position(|a| *a == act) {
        game.ctx_act_at(seat, i);
    }
}

fn drive_organizer(game: &mut Game, seat: usize, last_campaign_day: &mut u64) {
    // eating comes first, same as any player: if the purse is thin put in
    // a day of work, and let the sim's own hunger logic do the walking
    let who = game.seats[seat].player;
    let gold = game.world.get_component::<Purse>(who).map(|p| p.gold).unwrap_or(0.0);
    let hungry = game.world.get_component::<Hunger>(who).map(|h| h.value < 30.0).unwrap_or(false);
    if hungry || gold < 10.0 { driver::drive_player(game, seat); return; }

    // the campaign loop: once the cause stands, work it - a due meeting,
    // a seat for a member, letters for anyone inside, a fortnight's
    // march to keep the pressure on. One a day, not every frame - the
    // village isn't a button to mash
    if *last_campaign_day < game.tick {
        use game2_core::session::CtxAct;
        let ctxs = game.seats[seat].ctx_actions.clone();
        let acted = if ctxs.contains(&CtxAct::Meet) && game.ctx_blocker_at(seat, CtxAct::Meet).is_none() {
            act_ctx(game, seat, CtxAct::Meet); true
        } else if ctxs.contains(&CtxAct::Delegate) {
            // an open seat goes to someone without one - re-picking the
            // seated member eats the day's slot without delegating
            let seated: Vec<game2_core::engine::ecs::Entity> = game2_core::game::campaign::campaign_of(&game.world, game.seats[seat].org)
                .and_then(|c| game.world.get_component::<game2_core::game::campaign::Campaign>(c))
                .map(|c| c.seats.iter().map(|s| s.holder).collect()).unwrap_or_default();
            let member_i = game.world.get_component::<Organization>(game.seats[seat].org)
                .and_then(|o| o.members.iter().find(|m| !seated.contains(m)).copied())
                .and_then(|m| game.villagers.iter().position(|x| *x == m));
            match member_i {
                Some(i) => {
                    game.seats[seat].selected_target = i;
                    game.seats[seat].picked = true;
                    act_ctx(game, seat, CtxAct::Delegate); true
                }
                None => false,
            }
        } else if let Some(i) = game.villagers.iter().enumerate().find_map(|(i, v)|
            game.world.get_component::<game2_core::game::dungeon::Imprisoned>(*v).is_some().then_some(i)) {
            // the button only shows once someone's inside is picked:
            // pick them, refresh, then press the letters
            game.seats[seat].selected_target = i;
            game.seats[seat].picked = true;
            game.update_ctx_actions_at(seat);
            if game.seats[seat].ctx_actions.contains(&CtxAct::Petition) {
                act_ctx(game, seat, CtxAct::Petition); true
            } else { false }
        } else if ctxs.contains(&CtxAct::March) && game.tick % 14 == 0 {
            act_ctx(game, seat, CtxAct::March); true
        } else { false };
        if acted { *last_campaign_day = game.tick; return; }
    }


    let org = game.seats[seat].org;
    let members: Vec<_> = game.world.get_component::<Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default();
    // the best-prospected non-member in reach: deepest bond first
    let mut cands: Vec<(usize, f32)> = game.villagers_in_reach_at(seat).iter()
        .filter(|v| !members.contains(v) && !villager::is_dead(&game.world, **v))
        .filter_map(|v| game.villagers.iter().position(|x| *x == *v)
            .map(|i| (i, bond(game, seat, *v))))
        .collect();
    cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // only press buttons that aren't greyed out - a blocked action just
    // writes "Can't X" to the log, spam that isn't the village talking
    let can_recruit = game.action_blocker_at(seat, 1).is_none();
    if !can_recruit {
        // influence comes from fresh conversation (+0.05 while rapport is
        // new) - work the room, don't chat up your best friend forever
        cands.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(&(i, _)) = cands.first() {
            game.seats[seat].selected_target = i;
            game.seats[seat].picked = true;
            game.command_at(seat, Command::Act(0));
            return;
        }
        driver::drive_player(game, seat);
        return;
    }
    let ready: Vec<usize> = cands.iter().filter(|(_, s)| *s >= RECRUIT_MIN_TRUST).map(|(i, _)| *i).collect();
    if ready.len() >= 2 && game.meeting.is_none() {
        // a house meeting joins everyone who already trusts you - better
        // value than recruiting them one by one
        game.call_meeting_at(seat);
        return;
    }
    if let Some(&i) = ready.first() {
        game.seats[seat].selected_target = i;
        game.seats[seat].picked = true;
        game.command_at(seat, Command::Act(1));
        return;
    }
    if let Some(&(i, _)) = cands.first() {
        game.seats[seat].selected_target = i;
        game.seats[seat].picked = true;
        game.command_at(seat, Command::Act(0));
        return;
    }
    // nobody near to work on: drift to where the village gathers
    driver::drive_player(game, seat);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(200);
    let seed: u64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(42);

    let mut game = Game::new_seeded(seed);
    game.speed = 4; // turbo: days tick ~4x faster headless, same sim

    let mut record = RunRecord { seed, days, samples: vec![] };
    let mut log_read = 0usize; // cursor over every line ever pushed: log_dropped + index into game.log - the trim can't strand it
    let mut last_tick = game.tick;
    let mut last_campaign_day = 0u64;
    // a generous frame budget: ~ticks_per_frame per day plus slack for the
    // day-boundary work the game does around each tick
    let budget = (days + 50) * game.ticks_per_frame() as u64 * 2;
    for _ in 0..budget {
        if game.tick >= days { break; }
        game.update();
        if game.frame_n % driver::DRIVE_EVERY as u32 == 0 { drive_organizer(&mut game, 0, &mut last_campaign_day); }
        if game.tick != last_tick {
            last_tick = game.tick;
            // the log trims its front in blocks; the absolute cursor still
            // lands on the first line we haven't taken (or the window's
            // start if a trim outran it - only the trimmed lines are lost)
            let off = log_read.saturating_sub(game.log_dropped).min(game.log.len());
            let events: Vec<String> = game.log[off..]
                .iter().map(|l| l.splitn(2, ": ").nth(1).unwrap_or(l).to_string()).collect();
            log_read = game.log_dropped + game.log.len();
            record.samples.push(DaySample { tick: game.tick, gauges: game.secret_kpis().iter()
                .map(|(k, v)| (k.to_string(), *v)).collect(), events });
        }
    }
    record.days = record.samples.len() as u64;

    let tester = BalanceTester::new();
    let report = tester.score(&record);
    print!("{}", tester.render(&report));

    let last_log = game.log.len().saturating_sub(15);
    println!("-- last log lines --");
    for l in &game.log[last_log..] { println!("  {}", l); }

    if report.flags.is_empty() { std::process::exit(0); } else { std::process::exit(1); }
}
