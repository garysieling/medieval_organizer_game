//! Illness simulation: QSP/ADME-inspired compartment models.
//! Diseases progress through stages; treatment modifies rates.
//! From backlog: "rimworld-style stat effects", "specific thing well".

use crate::engine::ecs::{Entity, World};

/// Health state of an agent
#[derive(Debug)]
pub struct Health {
    pub condition: f32, // 0-1
    pub immunity: f32,
}

/// A disease progressing through compartments
#[derive(Debug)]
pub struct Disease {
    pub name: String,
    pub stage: Stage,
    pub severity: f32,
    pub contagious: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stage {
    Latent,
    Mild,
    Moderate,
    Severe,
    Critical,
    Recovering,
}

/// ADME-style pharmacokinetic model for a treatment
#[derive(Debug)]
pub struct Treatment {
    pub name: String,
    pub dose: f32,
    pub absorption: f32,  // rate into bloodstream
    pub clearance: f32,   // rate out
    pub efficacy: f32,    // how much it helps
}

/// System: diseases progress, health declines/recovers
pub struct IllnessSystem;
impl crate::engine::ecs::System for IllnessSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Health>() {
            let mut condition = world.get_component::<Health>(e).unwrap().condition;
            let mut new_stage = None;
            for d in world.query::<Disease>() {
                if let Some(dis) = world.get_component::<Disease>(d) {
                    condition -= dis.severity * 0.05;
                    new_stage = Some(advance_stage(dis));
                }
            }
            if let Some(_s) = new_stage {
                if let Some(h) = world.get_component_mut::<Health>(e) {
                    h.condition = condition.max(0.0);
                }
            }
        }
    }
}

fn advance_stage(d: &Disease) -> Stage {
    match d.stage {
        Stage::Latent => Stage::Mild,
        Stage::Mild => Stage::Moderate,
        Stage::Moderate => Stage::Severe,
        Stage::Severe => Stage::Critical,
        Stage::Critical => Stage::Critical,
        Stage::Recovering => Stage::Recovering,
    }
}

/// Apply treatment - modifies disease severity based on efficacy
pub fn apply_treatment(world: &mut World, _patient: Entity, treatment: &Treatment) {
    for d in world.query::<Disease>() {
        if let Some(dis) = world.get_component_mut::<Disease>(d) {
            dis.severity *= 1.0 - treatment.efficacy * treatment.dose;
            if dis.severity < 0.1 {
                dis.stage = Stage::Recovering;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_progression() {
        let d = Disease { name: "flu".into(), stage: Stage::Latent, severity: 0.5, contagious: false };
        assert_eq!(advance_stage(&d), Stage::Mild);
    }

    #[test]
    fn test_treatment_reduces_severity() {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, Health { condition: 0.5, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), stage: Stage::Mild, severity: 0.5, contagious: false });
        let t = Treatment { name: "herb".into(), dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy: 0.8 };
        apply_treatment(&mut w, p, &t);
        assert!(w.get_component::<Disease>(d).unwrap().severity < 0.2);
    }
}
