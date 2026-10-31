//! Mycology: the wood's caps - what feeds, what dreams, what kills.
//! From wiki/literature-reviews/sprint-7-mushrooms.md: the five families
//! (edible, psilocybin, muscimol, amatoxin, ergot), the lookalike pairs,
//! and the amatoxin course - nothing, then the belly, a false mend, then
//! the liver.
//!
//! SAFETY: cap names and trait lines here are story flavour, never a
//! real-world foraging guide. `FORAGING_DISCLAIMER` says so wherever
//! traits are surfaced.

use crate::engine::ecs::Entity;

/// In-game cap traits are flavour, not advice: never take them as a
/// real-world foraging guide.
pub const FORAGING_DISCLAIMER: &str =
    "The caps' names and traits are story, not a foraging guide - \
     never eat a wild mushroom on a game's say-so.";

/// One of the wood's kinds: the family a patch belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MushKind {
    /// The good orange cap - food, with a lookalike that turns the gut
    Chanterelle,
    /// The white meadow cap - food, with the deadliest lookalike of all
    FieldMushroom,
    /// The orange trap: not death, just a sour night
    JackOLantern,
    /// The little nipple-tipped cap of wet grassland: visions, mostly
    LibertyCap,
    /// The little brown mushroom beside it: the lookalike that kills
    Galerina,
    /// Red with white flecks, under the birch: the dream, rarely death
    FlyAgaric,
    /// The pale cap: nothing, then the belly, a false mend, the liver
    DeathCap,
    /// What grows in the under - the cave's own, food and a deeper dream
    UnderCap,
}

/// What eating a cap does to the eater
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MushEffect {
    /// Fills the belly
    Meal,
    /// A sour gut for a night; it passes (jack-o'-lantern)
    TurnsStomach,
    /// The field speaks; now and then a dark turn (liberty cap, under-cap)
    Vision,
    /// The red cap's dream - wild, rarely fatal (fly agaric)
    Delirium,
    /// Nothing, then the belly, a false mend, then the liver (galerina, death-cap)
    Amatoxin,
}

/// What a patch of `kind` means: what the eye reads, what the belly gets
pub struct MushSpec {
    /// What a skilled eye calls it
    pub name: &'static str,
    pub effect: MushEffect,
    /// Hunger a full meal of it returns
    pub food: f32,
    /// The kind a poor read confuses it for (its lookalike partner)
    pub lookalike: MushKind,
    /// What the skilled eye says on a true read
    pub trait_line: &'static str,
    /// What the picker says on taking it
    pub eat_line: &'static str,
}

impl MushKind {
    pub fn spec(self) -> MushSpec {
        match self {
            MushKind::Chanterelle => MushSpec {
                name: "chanterelle", effect: MushEffect::Meal, food: 25.0,
                lookalike: MushKind::JackOLantern,
                trait_line: "Golden and forked-gilled. Good pickings.",
                eat_line: "Chanterelles. Good pickings.",
            },
            MushKind::FieldMushroom => MushSpec {
                name: "field mushroom", effect: MushEffect::Meal, food: 20.0,
                lookalike: MushKind::DeathCap,
                trait_line: "Pink gills going brown - the good white cap.",
                eat_line: "White caps - a panful tonight.",
            },
            MushKind::JackOLantern => MushSpec {
                name: "jack-o'-lantern", effect: MushEffect::TurnsStomach, food: 15.0,
                lookalike: MushKind::Chanterelle,
                trait_line: "Gills run down the stem - a gut-turner, that one.",
                eat_line: "Green, after. Something's off.",
            },
            MushKind::LibertyCap => MushSpec {
                name: "liberty cap", effect: MushEffect::Vision, food: 10.0,
                lookalike: MushKind::Galerina,
                trait_line: "A little nipple-tipped cap - the shepherd's secret.",
                eat_line: "The field spoke. It's still speaking.",
            },
            MushKind::Galerina => MushSpec {
                name: "galerina", effect: MushEffect::Amatoxin, food: 10.0,
                lookalike: MushKind::LibertyCap,
                trait_line: "Brown and ringed - the little cap that's death.",
                eat_line: "Earthy. Good enough.",
            },
            MushKind::FlyAgaric => MushSpec {
                name: "fly agaric", effect: MushEffect::Delirium, food: 10.0,
                lookalike: MushKind::FlyAgaric,
                trait_line: "Red, white-flecked - the fly-killer's cap.",
                eat_line: "The flies are singing in his head.",
            },
            MushKind::DeathCap => MushSpec {
                name: "death-cap", effect: MushEffect::Amatoxin, food: 20.0,
                lookalike: MushKind::FieldMushroom,
                trait_line: "White gills, a cup at the foot - death, that.",
                eat_line: "Earthy. Good enough.",
            },
            MushKind::UnderCap => MushSpec {
                name: "under-cap", effect: MushEffect::Vision, food: 30.0,
                lookalike: MushKind::UnderCap,
                trait_line: "Pale as the dark that grew it - the under's own.",
                eat_line: "The under feeds, and shows.",
            },
        }
    }

    /// The lookalike problem in a word: this one's death
    pub fn is_deadly(self) -> bool {
        self.spec().effect == MushEffect::Amatoxin
    }

    /// Whether a forager who believes a patch is `self` takes it - folk
    /// eat what they think is a meal or a dream, and leave what they
    /// think is a gut-turn or death
    pub fn believed_edible(self) -> bool {
        matches!(self.spec().effect, MushEffect::Meal | MushEffect::Vision | MushEffect::Delirium)
    }
}

/// A misread's odds per mille for a forager of `skill`: the trained eye
/// reads the cap, the hungry one reads what it wants to see
pub fn read_chance(skill: f32) -> u64 {
    (150.0 + skill * 750.0) as u64
}

/// A pending ailment's phase: the amatoxin course keeps its own calendar,
/// and Saint Anthony's brothers make their rounds after an outbreak
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AilPhase {
    /// The belly lands - a day on, when the meal's half forgotten. Carries
    /// the patch index so the poisoned (and the village) can learn it
    ShroomBelly(usize),
    /// The false mend: they seem to turn the corner - they haven't
    ShroomRemission,
    /// The liver gives out, days after the meal
    ShroomLiver,
    /// The brothers of Saint Anthony tend the fire's victims: a dose of
    /// care per pending entry, walked down the church's list
    AntonineCare,
}

/// A waiting ailment: (who, the day it lands, what lands)
#[derive(Debug, Clone, Copy)]
pub struct PendingAil {
    pub who: Entity,
    pub due: u64,
    pub phase: AilPhase,
}
