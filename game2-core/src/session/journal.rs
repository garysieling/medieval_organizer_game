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
    /// A seat's player takes the `idx`-th card of the seed's roster
    PickCharacter { seat: usize, idx: usize },
    /// A seat's player keeps the default wanderer instead of picking
    DeclinePick { seat: usize },
}

/// What an input gave back: the seat it made, or an agent command's answer
#[derive(Debug, Clone, PartialEq)]
pub enum Applied {
    Seat(usize),
    Agent(Result<(), String>),
    /// A seat's roster choice landed (or was refused): a pick or a kept
    /// wanderer - the caller acks it so a leaving client knows it's safe
    Chose(bool),
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
            Input::PickCharacter { seat, idx } => Applied::Chose(self.pick_character_at(*seat, *idx)),
            Input::DeclinePick { seat } => Applied::Chose(self.decline_pick_at(*seat)),
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
        serde_json::to_string(&self.tech).unwrap_or_default().hash(&mut h);
        (pos, kpis, &self.log).hash(&mut h);
        h.finish()
    }
}

/// The journal format a binary reads. Bumped when an input's meaning
/// changes - a journal from before the bump would replay it differently
/// (a dropped button's Act(n) would silently no-op; a TalkPick after a
/// landed come-along now answers the invite's when/where, not the
/// replies it meant; the floor's slate carries more motions now, so a
/// Decide(n) can land on a different row (#1337); the dawn's job offers
/// draw different streams now that ghosts shape them (#1438), so an old
/// journal's return-trip replay would land on different work; and the
/// asks' odds themselves moved to the act model (#1442), so an old
/// journal's rolls land differently still; campaign outcomes now test
/// ghost lies (#1444), so an old journal's writs, parades, pleas and
/// resolves land on different ghost strengths; and the cause's ctx
/// verbs left the wire enum, so an old journal's Ctx(n) names a
/// different act (#1337); and the day's pulls now strain a tension
/// ledger that flips souls in and out of the cause (#1454), so an
/// old journal's orgs keep different members; and the music_tick's
/// "tune turns" lines left the log (#1504), so an old journal's digest
/// holds lines a new replay no longer writes; and the ctx band lost its
/// buttons (#1594-#1598), so an old journal's Ctx(n) clicks land on
/// different acts - or on a press there's no longer a button for; and
/// the meeting's floor now holds for the organizer's steer (#1611), so
/// an old journal's Decide(n) lands mid-pause on a steer row, not the
/// founding slate it was recorded against; and the call's steps gained
/// a form-a-team row (#1614), so an old journal's Decide(n) can land on
/// the street when it meant the team; and the standing asks - postings,
/// stall, mystic, charter, the eyes' hold, parley, the unsworn oath -
/// answer on their own cards now (#1663), so an old journal's Decide(n)
/// resolves the box it was recorded against, not the standing state
/// that used to eat it; and a thin room's meet-again now opens a
/// turnout count card ahead of the day picker (#1713), so an old
/// journal's Decide(n) for "meet again" can land on the count instead
/// of the day it was recorded against)
pub const JOURNAL_VERSION: u32 = 18;

/// A world's seed and every input it took
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    pub seed: u64,
    /// (frame, input), in the order they were applied
    pub entries: Vec<(u64, Input)>,
    /// Journals written before versioning carry no field - they read as 0
    #[serde(default)]
    pub version: u32,
    /// Whether the world was a seeded cast (#1411): replay must rebuild it
    /// the same way, or the digest check rightly rejects the restore.
    /// Journals written before the flag read as a fixed cast.
    #[serde(default)]
    pub cast_seeded: bool,
}

impl Journal {
    pub fn new(seed: u64) -> Journal { Journal::new_cast(seed, false) }
    /// A journal stamped with the cast mode its world runs
    pub fn new_cast(seed: u64, cast_seeded: bool) -> Journal {
        Journal { seed, entries: vec![], version: JOURNAL_VERSION, cast_seeded }
    }

    /// Note an input taken at `frame` (frames never go backwards)
    pub fn record(&mut self, frame: u64, input: Input) {
        debug_assert!(self.entries.last().is_none_or(|(f, _)| *f <= frame), "the journal runs forwards");
        self.entries.push((frame, input));
    }
}

/// Rebuild the world a journal describes, `frames` world-loop frames in
pub fn replay(j: &Journal, frames: u64) -> Game {
    let mut g = if j.cast_seeded { Game::new_seeded_cast(j.seed) } else { Game::new_seeded(j.seed) };
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
