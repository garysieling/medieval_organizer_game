//! Virtue: what a villager's choices slowly make of them (#950). Hidden
//! scores over the Aquinas seven, grown by the moral acts they choose
//! and witness - never a meter the player reads, only weight behind
//! what folk do. The traits they're born with bend the start; the life
//! does the rest.

use crate::engine::ecs::{Entity, World};
use crate::game::traits::{Personality, Trait};

/// The Aquinas seven: the cardinal four and the three that reach past
/// them - indexed into `Virtues::scores`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Virtue {
    Courage = 0,
    Justice,
    Temperance,
    Prudence,
    Faith,
    Hope,
    Charity,
}

/// A villager's settled character: how the choices they've made and
/// seen have shaped them. Every score starts near the common measure
/// and drifts by small nudges - the probe reads them, the village only
/// ever feels them
#[derive(Debug, Clone)]
pub struct Virtues {
    pub scores: [f32; 7],
}

impl Virtues {
    /// Character at birth: every score at the common measure, bent a
    /// little by the traits they carry - a kind soul starts kinder, a
    /// hot head less temperate
    pub fn seeded(p: &Personality) -> Self {
        let mut v = Virtues::unmarked();
        for t in &p.traits {
            let (k, by) = match t {
                Trait::Kind => (Virtue::Charity, 0.12),
                Trait::Stingy => (Virtue::Charity, -0.10),
                Trait::Pious => (Virtue::Faith, 0.12),
                Trait::Hotheaded => (Virtue::Temperance, -0.10),
                Trait::Proud => (Virtue::Courage, 0.05),
                Trait::Gregarious => (Virtue::Hope, 0.05),
                Trait::Ambitious => (Virtue::Prudence, 0.08),
                Trait::Lazy => (Virtue::Temperance, -0.05),
            };
            v.nudge(k, by);
        }
        v
    }

    /// An unremarkable soul: the common measure, nothing carried yet
    pub fn unmarked() -> Self {
        Virtues { scores: [0.5; 7] }
    }

    /// Read a score - the fork's roll and the probe both do
    pub fn get(&self, v: Virtue) -> f32 { self.scores[v as usize] }

    fn nudge(&mut self, v: Virtue, by: f32) {
        let i = v as usize;
        self.scores[i] = (self.scores[i] + by).clamp(0.0, 1.0);
    }
}

/// The moral acts the sim sees: what folk choose when it costs them
/// something, and what they carry for having seen it done
#[derive(Debug, Clone, Copy)]
pub enum MoralAct {
    /// Shared from their own table with a hungrier neighbor
    SharedAid,
    /// Kept their stores while a bonded neighbor hungered
    RefusedAid,
    /// Stood by their word when breaking it was easier
    KeptWord,
    /// Sat with the grieving
    Comforted,
    /// A dilemma answered so the innocent take it light
    SparedTheInnocent,
    /// A dilemma answered so the victim bears the weight
    LaidTheWeight,
    /// Watched a neighbor share their table (the co-witness's small share)
    WitnessedKindness,
}

/// What each act feeds: the virtues it grows (or wears), at weight 1.0
fn deltas(act: MoralAct) -> [(Virtue, f32); 3] {
    use Virtue::*;
    match act {
        MoralAct::SharedAid => [(Charity, 0.06), (Justice, 0.03), (Hope, 0.02)],
        MoralAct::RefusedAid => [(Charity, -0.05), (Prudence, 0.03), (Temperance, 0.01)],
        MoralAct::KeptWord => [(Justice, 0.04), (Faith, 0.04), (Courage, 0.02)],
        MoralAct::Comforted => [(Charity, 0.03), (Hope, 0.03), (Faith, 0.02)],
        MoralAct::SparedTheInnocent => [(Charity, 0.06), (Justice, 0.04), (Courage, 0.02)],
        MoralAct::LaidTheWeight => [(Charity, -0.04), (Prudence, 0.05), (Justice, -0.02)],
        MoralAct::WitnessedKindness => [(Hope, 0.02), (Charity, 0.01), (Faith, 0.01)],
    }
}

/// An act lands on the soul that chose it (or saw it): the scores drift
/// a little each time, so a life's shape comes from a life's choices,
/// not one dramatic moment. `weight` scales the nudge - a debt that
/// lands heavy bends more than a passing kindness. Entities without a
/// soul to shape (the country folk the sim didn't seed) pass quiet
pub fn witness(world: &mut World, e: Entity, act: MoralAct, weight: f32) {
    let Some(v) = world.get_component_mut::<Virtues>(e) else { return };
    for (k, d) in deltas(act) { v.nudge(k, d * weight); }
}
