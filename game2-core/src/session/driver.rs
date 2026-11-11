//! Heuristic drivers: simple goals for characters nobody's steering.
//!
//! `drive_player` runs a wanderer seat whose person went idle - keep them
//! useful: eat, work, talk, build. `drive_agent` is the same instinct for
//! a bot's villager. Both bias toward things that keep the village running.

use super::{AgentCmd, AgentObs, Command, Game};
use crate::game::vocation::Craft;
use crate::game::villager::Pos;
use crate::game::player::PlayerCharacter;
use crate::game::livelihood::Purse;

/// How often the fallback driver thinks (in update()s). Cheap: an idle
/// character only needs nudging every couple of seconds.
pub const DRIVE_EVERY: u64 = 40;

/// Server-side fallback for a wanderer seat gone quiet. Picks one command
/// per call - the caller paces these (every DRIVE_EVERY updates).
pub fn drive_player(game: &mut Game, seat: usize) {
    let who = game.seats[seat].player;
    let Some(p) = game.world.get_component::<Pos>(who).map(|p| (p.x, p.y)) else { return };
    let gold = game.world.get_component::<PlayerCharacter>(who).map(|p| p.resources)
        .or_else(|| game.world.get_component::<Purse>(who).map(|p| p.gold)).unwrap_or(0.0);
    let has_work = game.world.get_component::<Craft>(who).map(|c| c.workplace.is_some()).unwrap_or(false);
    // standing alone? stroll to the market where the village gathers
    let mp = game.world.get_component::<Pos>(game.market_food).map(|m| (m.x, m.y)).unwrap_or((10.0, 10.0));
    let d_market = ((mp.0 - p.0).powi(2) + (mp.1 - p.1).powi(2)).sqrt();
    if has_work && gold < 30.0 {
        // poor and employable: put in a day
        game.command_at(seat, Command::CtxAct(0)); // CtxAct::Work, when offered
        return;
    }
    if d_market > 6.0 {
        let (dx, dy) = (mp.0 - p.0, mp.1 - p.1);
        game.command_at(seat, Command::Walk(dx / d_market, dy / d_market));
        return;
    }
    // at the market: talk to whoever's in reach; the ties it builds are how
    // trust (and recruits) happen later
    let near = game.villagers_in_reach_at(seat);
    if !near.is_empty() {
        game.command_at(seat, Command::Act(0)); // Talk to the reach target
        return;
    }
    // flush with gold: a house beside them is worth more than coin idle
    if gold > 40.0 && game.action_blocker_at(seat, 2).is_none() {
        game.command_at(seat, Command::Act(2)); // Build
        return;
    }
    // otherwise amble a little, so they don't stand stock still
    let t = game.tick as f32;
    game.command_at(seat, Command::Walk(t.cos() * 0.5, t.sin() * 0.5));
}

/// What a bot's villager does next, from its observation: nothing while
/// held or mid-errand, eat when hungry, let a running errand run, talk
/// to whoever's near - the ties are what the village runs on - fetch what
/// the bench is short of, else put in a day's work.
pub fn drive_agent(obs: &AgentObs) -> AgentCmd {
    // the cell takes no errands
    if obs.held { return AgentCmd::Idle; }
    // mid-errand: let it finish (nothing happens until the walking is done)
    if obs.walking { return AgentCmd::Idle; }
    // hungry: eat before everything
    if obs.hunger < 40.0 { return AgentCmd::Eat; }
    // already on something that isn't idling: let it run
    if obs.busy.as_deref() != Some("Idle") && obs.busy.is_some() { return AgentCmd::Idle; }
    // someone's beside you and breathing: a word builds the ties that make
    // trust and recruits later - but it's a word, not a life's work, so
    // it answers on its own cadence (a talk day out of every few work days),
    // and only when they're close enough to actually hear it
    let near = obs.near.iter().filter(|p| !p.dead && p.dist <= 8.0)
        .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(std::cmp::Ordering::Equal));
    if obs.tick % 4 == 0 && near.is_some() {
        return AgentCmd::TalkTo { e: near.unwrap().e };
    }
    // the bench is short of its raw good: go and get it (two days in
    // three - a fetch that keeps failing mustn't keep them from the bench)
    if obs.me.needs.is_some() && obs.tick % 3 != 2 { return AgentCmd::FetchInput; }
    // the surplus's own market: a full satchel goes to the market - the
    // villager's own sell, the goods they carry past the day's own need
    if obs.satchel.len() >= 3 {
        return AgentCmd::Context { action: crate::session::AgentAction::Errand, target: None };
    }
    // the cause's own pull: an org's own cause on the village - the
    // villager who sees it joins on the ties' own pull (rare - the
    // cause's own answer when the bench's own work's done)
    if !obs.orgs.is_empty() && obs.tick % 20 == 0 {
        return AgentCmd::JoinOrg;
    }
    // the rest day: a tired villager rests at home (a day in eight -
    // the work's own rhythm)
    if obs.tick % 8 == 7 { return AgentCmd::Idle; }
    // default: a day's work - village first
    AgentCmd::WorkDay
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::ObsPerson;

    fn obs() -> AgentObs {
        AgentObs {
            tick: 0, you: 1, pos: (0.0, 0.0), gold: 10.0, hunger: 80.0,
            busy: Some("Idle".into()), near: vec![], near_animals: vec![], near_buildings: vec![],
            satchel: vec![], walking: false, held: false,
            orgs: vec![], campaigns: vec![], burning: vec![], me: Default::default(),
            notices: vec![], board: None,
        }
    }

    #[test]
    fn test_the_driver_talks_to_whos_near_and_stands_down_when_held() {
        let mut o = obs();
        // company nearby and idle on a talk day: a word before the work
        o.near.push(ObsPerson { e: 9, name: "Watt".into(), dist: 3.0, dead: false, job_offer: None });
        assert!(matches!(drive_agent(&o), AgentCmd::TalkTo { e: 9 }));
        // but not every day - the bench gets its days too
        o.tick = 1;
        assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
        o.tick = 0;
        // nobody near: the bench gets the day
        o.near.clear();
        assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
        // too far to talk across: the bench still gets the day
        o.near.push(ObsPerson { e: 9, name: "Watt".into(), dist: 10.0, dead: false, job_offer: None });
        assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
        // the dead don't make company
        o.near[0].dead = true;
        assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
        // the cell takes no errands, whoever's outside
        o.near.clear();
        o.held = true;
        assert!(matches!(drive_agent(&o), AgentCmd::Idle));
        // and hunger still comes first
        o.held = false;
        o.hunger = 20.0;
        assert!(matches!(drive_agent(&o), AgentCmd::Eat));
    }

    #[test]
    fn test_the_driver_sells_the_surplus_joins_the_cause_and_rests() {
        let mut o = obs();
        // a full satchel goes to market - the errand's own sell
        o.tick = 1;
        o.satchel = vec![("shoes".into(), 2), ("cloth".into(), 1), ("bread".into(), 3)];
        assert!(matches!(drive_agent(&o), AgentCmd::Context { action: crate::session::AgentAction::Errand, .. }));
        // the cause's own pull on a quiet day - the org seen joins
        o.satchel.clear();
        o.tick = 20;
        o.orgs = vec!["the cause".into()];
        assert!(matches!(drive_agent(&o), AgentCmd::JoinOrg));
        // and the rest day: a day in eight, the tired stand down
        o.orgs.clear();
        o.tick = 7;
        assert!(matches!(drive_agent(&o), AgentCmd::Idle));
    }
}
