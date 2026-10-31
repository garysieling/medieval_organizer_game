//! Heuristic drivers: simple goals for characters nobody's steering.
//!
//! `drive_player` runs a wanderer seat whose person went idle - keep them
//! useful: eat, work, talk, invest. `drive_agent` is the same instinct for
//! a bot's villager. Both bias toward things that keep the village running.

use super::{AgentCmd, AgentObs, Command, Game, ObsPerson};
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
    // influence (and recruits) happen later
    let near = game.villagers_in_reach_at(seat);
    if !near.is_empty() {
        game.command_at(seat, Command::Act(0)); // Talk to the reach target
        return;
    }
    if gold > 40.0 {
        game.command_at(seat, Command::Act(2)); // Invest: gold into standing
        return;
    }
    // otherwise amble a little, so they don't stand stock still
    let t = game.tick as f32;
    game.command_at(seat, Command::Walk(t.cos() * 0.5, t.sin() * 0.5));
}

/// What a bot's villager does next, from its observation: nothing while
/// held or mid-errand, eat when hungry, let a running errand run, talk
/// to whoever's near - the ties are what the village runs on - else put
/// in a day's work. The obs carries no craft inputs or sellers' goods,
/// so bench provisioning stays a decision only a real bot can make.
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
    // influence and recruits later
    if let Some(p) = obs.near.iter().filter(|p| !p.dead)
        .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(std::cmp::Ordering::Equal)) {
        return AgentCmd::TalkTo { e: p.e };
    }
    // default: a day's work - village first
    AgentCmd::WorkDay
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs() -> AgentObs {
        AgentObs {
            tick: 0, you: 1, pos: (0.0, 0.0), gold: 10.0, hunger: 80.0,
            busy: Some("Idle".into()), near: vec![], near_buildings: vec![],
            satchel: vec![], walking: false, held: false,
            orgs: vec![], campaigns: vec![], burning: vec![],
        }
    }

    #[test]
    fn test_the_driver_talks_to_whos_near_and_stands_down_when_held() {
        let mut o = obs();
        // company nearby and idle: a word before a day's work
        o.near.push(ObsPerson { e: 9, name: "Watt".into(), dist: 3.0, dead: false });
        assert!(matches!(drive_agent(&o), AgentCmd::TalkTo { e: 9 }));
        // nobody near: the bench gets the day
        o.near.clear();
        assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
        // the dead don't make company - the bench still gets the day
        o.near.push(ObsPerson { e: 9, name: "Watt".into(), dist: 3.0, dead: true });
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
}
