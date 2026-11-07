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

/// A roll for this driver's own decisions (the sim's own `Rng` isn't
/// public outside the crate) - splitmix64 off whatever varies call to
/// call, so the same ask doesn't always land or always fail.
fn driver_roll(seed: u64) -> f32 {
    let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    ((z ^ (z >> 31)) % 1000) as f32 / 1000.0
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

/// A house meeting is earned, not summoned: it needs a lent venue and
/// someone who's said they'll come. With two or more trusted villagers in
/// reach, work toward that plan one ask a day - a venue first, then an
/// attendee - and only call the meeting once it's actually ready. Returns
/// whether it took an action (asked, or called the meeting). `last_ask_day`
/// caps a failed ask to one try a day - drive_organizer runs several times
/// a day, and propose() chills the bond on a no, so retrying the same
/// rejected ask before the day turns over would just grind trust down for
/// nothing.
fn call_meeting_if_earned(game: &mut Game, seat: usize, ready: &[usize], last_ask_day: &mut u64) -> bool {
    use game2_core::game::desires::Known;
    use game2_core::game::organizing::{propose, MeetingPlan, PropKind};
    let oe = game.seats[seat].org;
    if game.world.get_component::<MeetingPlan>(oe).map(|p| p.ready()).unwrap_or(false) {
        game.call_meeting_at(seat);
        return true;
    }
    if *last_ask_day == game.tick { return false; }
    // only someone you've heard something from will hear an ask, same
    // as the Talk panel's own asks list (organizing::props_on_offer)
    let heard = |g: &Game, v: game2_core::engine::ecs::Entity| g.world.get_component::<Known>(v)
        .map(|k| !k.revealed.is_empty()).unwrap_or(false);
    let venue_set = game.world.get_component::<MeetingPlan>(oe).map(|p| p.venue.is_some()).unwrap_or(false);
    let target = ready.iter().find(|&&i| {
        let v = game.villagers[i];
        if !heard(game, v) { return false; }
        if !venue_set { return game2_core::game::household::home_of(&game.world, v).is_some(); }
        !game.world.get_component::<MeetingPlan>(oe)
            .map(|p| p.coming.contains(&v) || p.spreaders.contains(&v)).unwrap_or(false)
    }).copied();
    let Some(i) = target else { return false };
    *last_ask_day = game.tick;
    let v = game.villagers[i];
    let p = if venue_set { PropKind::ComeAlong } else { PropKind::LendVenue };
    let (pe, tick) = (game.seats[seat].player, game.tick);
    let roll = driver_roll(tick.wrapping_mul(0x1000_0001) ^ v as u64);
    let name = game2_core::session::vill_name(i);
    let (msg, _) = propose(&mut game.world, pe, oe, v, p, name, roll, tick);
    game.log.push(format!("t{}: {}", tick, msg));
    true
}

fn drive_organizer(game: &mut Game, seat: usize, last_campaign_day: &mut u64, last_meeting_ask_day: &mut u64) {
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
        // rapport comes from fresh conversation - work the room, don't
        // chat up your best friend forever
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
    if ready.len() >= 2 && game.meeting.is_none() && call_meeting_if_earned(game, seat, &ready, last_meeting_ask_day) {
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
    let mut last_meeting_ask_day = u64::MAX;
    // a generous frame budget: ~ticks_per_frame per day plus slack for the
    // day-boundary work the game does around each tick
    let budget = (days + 50) * game.ticks_per_frame() as u64 * 2;
    for _ in 0..budget {
        if game.tick >= days { break; }
        game.update();
        if game.frame_n % driver::DRIVE_EVERY as u32 == 0 { drive_organizer(&mut game, 0, &mut last_campaign_day, &mut last_meeting_ask_day); }
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
