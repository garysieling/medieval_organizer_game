//! Signature scenes (#1227): short scripted exchanges for characterized
//! pairs, spliced over generated chat the way meeting beats splice into
//! meetings. A pair the table knows plays their scene instead of making
//! small talk - and the take it plays moves with their bond, so the feud
//! warms and cools over weeks rather than running the same lines.
//!
//! Each `Scene` names its pair by a `PairKind` matcher and holds a few
//! `SceneVariant`s gated on the bond strength between the two; the first
//! variant whose band holds the bond plays. Lines alternate speakers by
//! index: 0 is the entity the caller named first, 1 the second - for
//! asymmetric pairs the matcher fixes the order (see `pair_order`).

use crate::engine::ecs::{Entity, World};
use crate::game::desires::{problems_of, Problem};
use crate::game::feuds::{Feud, FeudStatus};
use crate::game::relationships::{find_relationship, Relationship};
use crate::game::vocation::{Craft, TownRole, Vocation};

/// Which characterized pair a scene fits
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PairKind {
    /// Two folk bound in a live feud
    Feuding,
    /// A grieving villager and the priest who looks after them
    WidowPriest,
    /// The market pedlar and the watch who walks by his stall
    PedlarGuard,
}

/// One take of a scene, played when the pair's bond reads inside the band
pub struct SceneVariant {
    /// Played when lo <= bond strength < hi (bands tile the whole line,
    /// first match wins)
    pub lo: f32,
    pub hi: f32,
    /// Alternating lines; the u8 is the speaker index (0 = first entity
    /// the caller named, 1 = the second)
    pub lines: &'static [(u8, &'static str)],
}

/// A signature scene: who it fits, and the takes it can play
pub struct Scene {
    pub id: &'static str,
    pub pair: PairKind,
    pub variants: &'static [SceneVariant],
}

/// The table: the village's small set of stories (#1227). Depth where a
/// pair has one; everywhere else the chat's generated as it ever was
pub const SCENES: &[Scene] = &[
    Scene { id: "feud_neighbors", pair: PairKind::Feuding, variants: &[
        SceneVariant { lo: -2.0, hi: -0.3, lines: &[
            (0, "{2:poss} fence still leans on my strip."),
            (1, "And {2:poss} goose still fouls my stoop. We're even."),
            (0, "Even. When the hedge is righted, we'll talk even."),
            (1, "I'll right it when the crows take the corn."),
        ] },
        SceneVariant { lo: -0.3, hi: 0.35, lines: &[
            (0, "{2:poss} boy ran my fence line yesterday."),
            (1, "He's six. The fence'll keep."),
            (0, "...It's not the fence I'm minded of."),
            (1, "No. It never was."),
        ] },
        SceneVariant { lo: 0.35, hi: 2.0, lines: &[
            (0, "Cold coming in early this year."),
            (1, "Aye. {2:poss} apple took it hard, I saw."),
            (0, "Half went to the press. I'll spare {2:obj} a jar."),
            (1, "{2:subj} {2:be} not so bad, whatever I said."),
        ] },
    ] },
    Scene { id: "widow_priest", pair: PairKind::WidowPriest, variants: &[
        SceneVariant { lo: -2.0, hi: 0.3, lines: &[
            (1, "I saw {2:obj} at the grave again, child."),
            (0, "The earth's fresh. That's all."),
            (1, "Fresh a year now. Grief keeps no calendar."),
            (0, "Mine does. It's always that day."),
        ] },
        SceneVariant { lo: 0.3, hi: 0.6, lines: &[
            (1, "He'd not want {2:obj} keeping cold supper over a mound."),
            (0, "No. He'd want his supper."),
            (1, "And {2:obj}, at his table, laughing. Come to the hall Sunday."),
            (0, "I'll sit at the back."),
        ] },
        SceneVariant { lo: 0.6, hi: 2.0, lines: &[
            (0, "Father, I dreamed him young again, cutting the hedge."),
            (1, "A mercy, that. How did he seem?"),
            (0, "Content. I woke easy for once."),
            (1, "Then let him rest - and {2:obj}, a little, too."),
        ] },
    ] },
    Scene { id: "pedlar_guard", pair: PairKind::PedlarGuard, variants: &[
        SceneVariant { lo: -2.0, hi: 0.2, lines: &[
            (0, "Honest goods, watch - same as the last three times."),
            (1, "What's in the packs today, pedlar?"),
            (0, "Ribbons, pins, a little tin. Weigh them {2:self}, I'll wait."),
            (1, "Honest goods have a way of going short weight."),
        ] },
        SceneVariant { lo: 0.2, hi: 0.5, lines: &[
            (1, "Quiet round the stalls today."),
            (0, "Quiet's bad for the till, watch. Bring me a thief or two."),
            (1, "The village breeds no thieves worth the rope."),
            (0, "Then I'll settle for customers with heavier purses."),
        ] },
        SceneVariant { lo: 0.5, hi: 2.0, lines: &[
            (1, "{2:poss} ribbons were the talk of the feast, pedlar."),
            (0, "Sold out by noon, watch. Kept one back for {2:poss} wife, though."),
            (1, "She'll have {2:poss} hide for the price."),
            (0, "For {2:obj}? No price. For the missus - a fair one."),
        ] },
    ] },
];

/// How the pair's bond reads for the variant bands
fn bond_strength(world: &World, a: Entity, b: Entity) -> f32 {
    find_relationship(world, a, b)
        .and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0)
}

/// Does this pair fit the scene's two chairs, and in which seat order?
/// Speaker index 0 is the entity returned first, 1 the second.
fn pair_order(world: &World, pair: PairKind, a: Entity, b: Entity, tick: u64) -> Option<(Entity, Entity)> {
    match pair {
        PairKind::Feuding => world.query::<Feud>().iter().copied().find_map(|e| {
            let f = world.get_component::<Feud>(e)?;
            let live = matches!(f.status, FeudStatus::Active | FeudStatus::Burning
                | FeudStatus::Peacemaking);
            if !live { return None; }
            if f.offender == a && f.aggrieved == b { Some((a, b)) }
            else if f.offender == b && f.aggrieved == a { Some((b, a)) }
            else { None }
        }),
        // the bereaved speaks first, the priest answers
        PairKind::WidowPriest => {
            let priest = |e| world.get_component::<Craft>(e)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false);
            let bereaved = |e| problems_of(world, e, tick).contains(&Problem::Bereaved);
            if bereaved(a) && priest(b) { Some((a, b)) }
            else if bereaved(b) && priest(a) { Some((b, a)) }
            else { None }
        }
        // the pedlar speaks first, the watch answers
        PairKind::PedlarGuard => {
            let trader = |e| world.get_component::<TownRole>(e)
                .map(|r| *r == TownRole::Trader).unwrap_or(false);
            let watch = |e| world.get_component::<Craft>(e)
                .map(|c| c.vocation == Vocation::Knight).unwrap_or(false);
            if trader(a) && watch(b) { Some((a, b)) }
            else if trader(b) && watch(a) { Some((b, a)) }
            else { None }
        }
    }
}

/// The scene a pair would play now, and the take it would play:
/// (scene, variant, the pair in speaker order). None when the village
/// has no story for these two - generation takes them instead.
pub fn scene_for<'a>(world: &World, a: Entity, b: Entity, tick: u64)
    -> Option<(&'a Scene, &'a SceneVariant, (Entity, Entity))> {
    for scene in SCENES {
        let Some(order) = pair_order(world, scene.pair, a, b, tick) else { continue };
        let s = bond_strength(world, order.0, order.1);
        let variant = scene.variants.iter()
            .find(|v| v.lo <= s && s < v.hi).unwrap_or(&scene.variants[0]);
        return Some((scene, variant, order));
    }
    None
}
