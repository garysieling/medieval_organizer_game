//! Heuristic drivers: simple goals for characters nobody's steering.
//!
//! `drive_player` runs a wanderer seat whose person went idle - keep them
//! useful: eat, work, talk, invest. `drive_agent` is the same instinct for
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

/// What a bot's villager does next, from its observation: eat when hungry,
/// work when it can, buy food when the store has it, gather wood when idle,
/// talk for ties. Heuristic, biased to keeping the village running.
pub fn drive_agent(obs: &AgentObs) -> AgentCmd {
    // mid-errand: let it finish (nothing happens until the walking is done)
    if obs.walking { return AgentCmd::Idle; }
    // hungry: eat before everything
    if obs.hunger < 40.0 { return AgentCmd::Eat; }
    // carry nothing and broke? put in a day at the bench
    if obs.busy.as_deref() != Some("Idle") && obs.busy.is_some() { return AgentCmd::Idle; }
    // default: a day's work - village first
    AgentCmd::WorkDay
}
