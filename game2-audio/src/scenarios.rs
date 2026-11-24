//! The scenario library (#1493): named, scripted villages an agent
//! renders by name. This file is the layering test's one allowlist -
//! a scenario is a test rig, so it may name `Game` and the harness
//! verbs (`stage`, `force_event`, `force_road_march`); the audio path
//! itself still sees only `UiModel`.
use game2_core::session::{Command, Game};
use game2_core::session::events::EventId;
use game2_core::session::stage::Stage;

/// What a scenario does to its game on one frame: the same verbs
/// `record_video`'s scripts use (walk, camera, force_event, a road
/// march, talk goes through `Cmd`).
pub enum Op {
    /// A player-seat command (walk, act, talk picks)
    Cmd(Command),
    /// Move the listener's camera to a map tile
    Look(i32, i32),
    /// Centre the listener on the market cross, wherever the seed put it
    LookMarket,
    /// Force a rare event now
    ForceEvent(EventId),
    /// Send the cause's column to site `si`
    RoadMarch(usize),
    /// Put the village in a state (fever, famine, a strong org...)
    Stage(Stage),
}

/// A script line: the frame it applies on, then what it does.
pub struct ScriptOp {
    pub frame: u64,
    pub op: Op,
}

/// A named render: how to build the village from a seed, and the
/// script that plays over it.
pub struct Scenario {
    pub name: &'static str,
    pub build: fn(u64) -> Game,
    pub script: Vec<ScriptOp>,
}

fn quiet_village(seed: u64) -> Game { Game::new_seeded_cast(seed) }

/// Every scenario by name. `quiet_village` is S3's; the later
/// soundscape tickets add theirs here.
pub fn registry() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "quiet_village",
            build: quiet_village,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
    ]
}

/// One scenario by name.
pub fn by_name(name: &str) -> Option<Scenario> {
    registry().into_iter().find(|s| s.name == name)
}
