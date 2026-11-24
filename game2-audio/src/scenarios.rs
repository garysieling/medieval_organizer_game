//! The scenario library (#1493): named, scripted villages an agent
//! renders by name. This file is the layering test's one allowlist -
//! a scenario is a test rig, so it may name `Game` and the harness
//! verbs (`stage`, `force_event`, `force_road_march`); the audio path
//! itself still sees only `UiModel`.
use game2_core::session::{Command, Game, UiModel};
use game2_core::session::events::EventId;
use game2_core::session::stage::{self, Stage};
use game2_core::sim::climate::{Climate, Season};

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
    /// Centre the listener on the apiary's skeps (A4)
    LookSkeps,
    /// Centre the listener on the church (A4)
    LookChurch,
    /// Centre the listener on the wall's first gate (A4)
    LookGate,
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

/// Where a `Look` op puts the ear, in tile space: the fixed tiles are
/// literal, the named spots resolve against the frame's model (the
/// market's cross, the apiary's first skep, the church, the wall's
/// first gate). Non-look ops resolve to nothing.
pub fn look_spot(op: &Op, m: &UiModel) -> Option<(f32, f32)> {
    use game2_core::game::tilemap::WallPiece;
    use game2_core::session::BSprite;
    match op {
        Op::Look(x, y) => Some((*x as f32 + 0.5, *y as f32 + 0.5)),
        Op::LookMarket => m.market
            .or_else(|| m.stalls.first().map(|s| (s.0, s.1))),
        Op::LookSkeps => m.skeps.first().copied(),
        Op::LookChurch => m.buildings.iter()
            .find(|b| b.sprite == BSprite::Church).map(|b| (b.x, b.y)),
        Op::LookGate => m.map.walls.iter()
            .find(|(_, p)| matches!(p, WallPiece::GateAlongX | WallPiece::GateAlongY))
            .map(|(t, _)| (t.0 as f32 + 0.5, t.1 as f32 + 0.5)),
        _ => None,
    }
}

/// Run the sim until `cond` holds (bounded, so a dead schedule can't
/// hang the harness) - the sanctioned `stage::run_days` mover.
fn run_until(g: &mut Game, cond: impl Fn(&Game) -> bool, max_days: u64) {
    for _ in 0..max_days {
        if cond(g) { return; }
        game2_core::session::stage::run_days(g, 1);
    }
}

/// The apiary in spring: bees and birdsong carry the scene, so the
/// build only needs the season a new game opens in.
fn apiary_spring(seed: u64) -> Game { quiet_village(seed) }

/// Holy day: run to the next holy morning and put the ear at the
/// church - the bell, the chant's room, a hushed murmur.
fn holy_day_church(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    run_until(&mut g, |g| game2_core::session::routine::is_holy(g.tick), 14);
    g
}

/// Market day at the cross: the crowd bed over the murmur.
fn market_square(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    run_until(&mut g, |g| game2_core::session::routine::is_market_day(g.tick), 40);
    g
}

/// Winter at the gate: the world muffled by snow, the gate's creak as
/// trains and folk pass. The climate component goes straight to
/// winter (the harness's licence, like `stage`'s own edits) and the
/// build waits for a day the precipitation system actually shows snow.
fn winter_gate(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    if let Some(c) = g.world.get_component_mut::<Climate>(g.climate) {
        c.season = Season::Winter;
        c.day = c.day / 360 * 360 + 300; // mid-winter in the 360-day year
    }
    g.harsh_winter = true;
    run_until(&mut g, |g| {
        g.ui_model().precip.is_some_and(|p| p.is_snow())
            || g.ui_model().season != Season::Winter
    }, 40);
    g
}

/// The build's arc (M5): a strong org whose campaign publishes its
/// next event, staged close enough to it that the phrases climb. The
/// `drama.next` feed comes from the campaign's own cadence - the same
/// `PlannedV` an announced assembly produces.
fn assembly_build(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    stage::stage(&mut g, Stage::StrongOrg);
    // the campaign publishes its next event, then we wait until it's
    // ~3.5 days out - inside the build's 4-phrase window (~80 s) at the
    // dump's start, so the score climbs the whole run
    run_until(&mut g, |g| g.ui_model().panel.drama.next.is_some(), 40);
    run_until(&mut g, |g| {
        g.ui_model().panel.drama.next.as_ref()
            .is_some_and(|n| n.in_days <= 3.5)
    }, 40);
    g
}

/// A march on the castle, then the answer (M5): the column sets out
/// early in the run so the Peak has somewhere to resolve - a crackdown
/// drops the mode two rungs when the powers reply.
fn march_crackdown(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    stage::stage(&mut g, Stage::StrongOrg);
    g
}

/// A feast at the cross (M5): estampie in triple time, the feast motif.
fn feast_day(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    run_until(&mut g, |g| g.ui_model().feast, 45);
    g
}

/// The lament (M5): scarlet fever took beds the seat knew - grief
/// slows the score and holds it in Aeolian.
fn fever_lament(seed: u64) -> Game {
    let mut g = quiet_village(seed);
    stage::stage(&mut g, Stage::Bereft);
    g
}

/// Every scenario by name. `quiet_village` is S3's; A4's four play to
/// the place-based soundscape's strengths; M5's four play the score's
/// arcs (build, march, feast, lament).
pub fn registry() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "quiet_village",
            build: quiet_village,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
        Scenario {
            name: "apiary_spring",
            build: apiary_spring,
            script: vec![ScriptOp { frame: 0, op: Op::LookSkeps }],
        },
        Scenario {
            name: "holy_day_church",
            build: holy_day_church,
            script: vec![ScriptOp { frame: 0, op: Op::LookChurch }],
        },
        Scenario {
            name: "market_square",
            build: market_square,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
        Scenario {
            name: "winter_gate",
            build: winter_gate,
            script: vec![ScriptOp { frame: 0, op: Op::LookGate }],
        },
        Scenario {
            name: "assembly_build",
            build: assembly_build,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
        Scenario {
            name: "march_crackdown",
            build: march_crackdown,
            script: vec![
                ScriptOp { frame: 60, op: Op::RoadMarch(0) },
            ],
        },
        Scenario {
            name: "feast_day",
            build: feast_day,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
        Scenario {
            name: "fever_lament",
            build: fever_lament,
            script: vec![ScriptOp { frame: 0, op: Op::LookMarket }],
        },
    ]
}

/// One scenario by name.
pub fn by_name(name: &str) -> Option<Scenario> {
    registry().into_iter().find(|s| s.name == name)
}
