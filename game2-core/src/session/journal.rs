//! The journal: every input the world took, with the frame it took it in.
//!
//! The sim is deterministic per seed (docs/architecture.md, step 3b), so a
//! seed and its journal rebuild a world exactly: `replay`. That's how the
//! server keeps a village across restarts (`ServerConfig::save`), and how
//! a multiplayer session can be replayed to chase a bug.
//!
//! Every input goes through `Game::apply_input`, live or replayed, so the
//! two can't drift apart. Frames count like the server's world loop: inputs
//! tagged `f` happened after the f-th `update()` and before the next.
//!
//! The cost: a restore replays every frame since the world began (~25s per
//! hour of play at 20 frames a second). Snapshots can bound that later.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use super::{driver, AgentCmd, Command, Game};

/// One input the world took
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    /// A new wanderer seat (a human's Hello)
    AddSeat,
    /// A new agent seat on this villager (an agent's Hello)
    AddAgentSeat { villager: Entity },
    /// A player's input on their seat
    Command { seat: usize, cmd: Command },
    /// An agent's steering on their seat
    AgentCmd { seat: usize, cmd: AgentCmd },
    /// The fallback driver nudging an idle wanderer seat
    DriveIdle { seat: usize },
}

/// What an input gave back: the seat it made, or an agent command's answer
#[derive(Debug, Clone, PartialEq)]
pub enum Applied {
    Seat(usize),
    Agent(Result<(), String>),
    Done,
}

impl Game {
    /// Apply one input - the only way the server changes the world, so a
    /// replay applies exactly what play did
    pub fn apply_input(&mut self, input: &Input) -> Applied {
        match input {
            Input::AddSeat => Applied::Seat(self.add_seat()),
            Input::AddAgentSeat { villager } => Applied::Seat(self.add_agent_seat(*villager)),
            Input::Command { seat, cmd } => { self.command_at(*seat, cmd.clone()); Applied::Done }
            Input::AgentCmd { seat, cmd } => Applied::Agent(self.agent_command(*seat, cmd.clone())),
            Input::DriveIdle { seat } => { driver::drive_player(self, *seat); Applied::Done }
        }
    }

    /// A fingerprint of the world's whole state - the omniscient probe, the
    /// secret gauges, where everyone stands, the log - to check a replay
    /// rebuilt the world it was meant to
    pub fn state_digest(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let pos: Vec<(u32, u32)> = self.villagers.iter().chain(self.seats.iter().map(|s| &s.player))
            .map(|&v| self.world.get_component::<crate::game::villager::Pos>(v)
                .map(|p| (p.x.to_bits(), p.y.to_bits())).unwrap_or((0, 0)))
            .collect();
        let kpis: Vec<(&str, u64)> = self.secret_kpis().iter().map(|(k, v)| (*k, v.to_bits())).collect();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (self.tick, self.frame_n, self.seats.len()).hash(&mut h);
        serde_json::to_string(&self.probe()).unwrap_or_default().hash(&mut h);
        (pos, kpis, &self.log).hash(&mut h);
        h.finish()
    }
}

/// A world's seed and every input it took
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    pub seed: u64,
    /// (frame, input), in the order they were applied
    pub entries: Vec<(u64, Input)>,
}

impl Journal {
    pub fn new(seed: u64) -> Journal { Journal { seed, entries: vec![] } }

    /// Note an input taken at `frame` (frames never go backwards)
    pub fn record(&mut self, frame: u64, input: Input) {
        debug_assert!(self.entries.last().is_none_or(|(f, _)| *f <= frame), "the journal runs forwards");
        self.entries.push((frame, input));
    }
}

/// Rebuild the world a journal describes, `frames` world-loop frames in
pub fn replay(j: &Journal, frames: u64) -> Game {
    let mut g = Game::new_seeded(j.seed);
    let mut next = j.entries.iter().peekable();
    let mut take = |g: &mut Game, f: u64| {
        while let Some((_, input)) = next.next_if(|(ef, _)| *ef == f) { g.apply_input(input); }
    };
    take(&mut g, 0);
    for f in 1..=frames {
        g.update();
        take(&mut g, f);
    }
    g
}
