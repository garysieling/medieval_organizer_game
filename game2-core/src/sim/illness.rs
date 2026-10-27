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
    pub patient: Entity, // who is sick with it
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

/// Health lost per tick per unit of severity while a disease is active
const DAMAGE_RATE: f32 = 0.01;
/// Severity the immune system clears per tick per unit of immunity
const IMMUNE_RATE: f32 = 0.01;
/// Health regained per tick once a disease is beaten
const RECOVERY_RATE: f32 = 0.005;
/// Health at which a recovering patient is cleared of the disease
const RECOVERED: f32 = 0.9;

/// System: each disease wears down its own patient while their immune
/// system fights it off; once severity drops low enough they recover, and
/// the disease is cleared when their health is back. A patient whose health
/// reaches 0 has died: the disease ends with them, and they neither recover
/// nor infect anyone.
pub struct IllnessSystem;
impl crate::engine::ecs::System for IllnessSystem {
    fn run(&mut self, world: &mut World) {
        for d in world.query::<Disease>() {
            let (patient, stage, severity) = match world.get_component::<Disease>(d) {
                Some(dis) => (dis.patient, dis.stage, dis.severity),
                None => continue,
            };
            let Some(h) = world.get_component_mut::<Health>(patient) else {
                world.destroy_entity(d); // patient is gone
                continue;
            };
            if h.condition <= 0.0 {
                world.destroy_entity(d); // the patient has died
                continue;
            }
            if stage == Stage::Recovering {
                h.condition = (h.condition + RECOVERY_RATE).min(1.0);
                if h.condition >= RECOVERED {
                    // beating it leaves them harder to reinfect
                    h.immunity = (h.immunity + ACQUIRED_IMMUNITY).min(1.0);
                    world.destroy_entity(d);
                }
                continue;
            }
            h.condition = (h.condition - severity * DAMAGE_RATE).max(0.0);
            if h.condition <= 0.0 {
                world.destroy_entity(d); // the disease killed them
                continue;
            }
            let severity = (severity - h.immunity * IMMUNE_RATE).max(0.0);
            if let Some(dis) = world.get_component_mut::<Disease>(d) {
                dis.severity = severity;
                dis.stage = stage_for(severity);
            }
        }
        spread_contagion(world);
    }
}

/// Manhattan distance within which a sick villager exposes others
pub const CONTACT_RANGE: f32 = 3.0;
/// Exposure gained per tick per unit of a nearby source's severity
const EXPOSURE_RATE: f32 = 0.08;
/// Fraction of built-up exposure kept each tick (it fades away from the sick)
const EXPOSURE_RETAINED: f32 = 0.98;
/// Share of the source's severity a newly caught case starts with
const TRANSMITTED_SEVERITY: f32 = 0.8;
/// Immunity gained from beating a disease
const ACQUIRED_IMMUNITY: f32 = 0.3;

/// How much contagion a healthy agent has soaked up from the sick around them
#[derive(Debug, Default)]
pub struct Exposure(pub f32);

/// Contagious diseases spread by proximity: each tick, healthy agents within
/// CONTACT_RANGE of an actively sick one build up exposure in proportion to
/// that patient's severity (fading when they're away from the sick); once
/// exposure outweighs their immunity they catch a slightly milder case.
fn spread_contagion(world: &mut World) {
    use crate::game::villager::Pos;
    let mut sources = vec![]; // (position, severity, name)
    let mut sick = vec![];
    for d in world.query::<Disease>() {
        let Some(dis) = world.get_component::<Disease>(d) else { continue };
        sick.push(dis.patient);
        if !dis.contagious || dis.stage == Stage::Recovering { continue; }
        if let Some(p) = world.get_component::<Pos>(dis.patient) {
            sources.push(((p.x, p.y), dis.severity, dis.name.clone()));
        }
    }
    for e in world.query::<Health>() {
        if sick.contains(&e) { continue; }
        if world.get_component::<Health>(e).map(|h| h.condition <= 0.0).unwrap_or(false) { continue; } // the dead don't catch it
        let Some(p) = world.get_component::<Pos>(e).map(|p| (p.x, p.y)) else { continue };
        let near: Vec<_> = sources.iter()
            .filter(|((x, y), _, _)| (x - p.0).abs() + (y - p.1).abs() <= CONTACT_RANGE)
            .collect();
        let dose: f32 = near.iter().map(|(_, sev, _)| sev * EXPOSURE_RATE).sum();
        if dose == 0.0 && world.get_component::<Exposure>(e).is_none() { continue; }
        if world.get_component::<Exposure>(e).is_none() {
            world.add_component(e, Exposure::default());
        }
        let immunity = world.get_component::<Health>(e).map(|h| h.immunity).unwrap_or(0.0);
        let exposure = world.get_component_mut::<Exposure>(e).unwrap();
        exposure.0 = exposure.0 * EXPOSURE_RETAINED + dose;
        if exposure.0 <= immunity { continue; }
        exposure.0 = 0.0;
        // catch the worst case they were exposed to, a little weakened
        let (_, sev, name) = near.iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .copied()
            .cloned()
            .unwrap_or(((0.0, 0.0), 0.0, String::new()));
        let severity = sev * TRANSMITTED_SEVERITY;
        if severity < 0.1 { continue; } // too weak to take hold
        let d = world.create_entity();
        world.add_component(d, Disease { name, patient: e, stage: stage_for(severity), severity, contagious: true });
    }
}

fn stage_for(severity: f32) -> Stage {
    if severity < 0.1 { Stage::Recovering }
    else if severity < 0.3 { Stage::Mild }
    else if severity < 0.6 { Stage::Moderate }
    else if severity < 0.85 { Stage::Severe }
    else { Stage::Critical }
}

/// Apply treatment - modifies the patient's disease severity based on efficacy
pub fn apply_treatment(world: &mut World, patient: Entity, treatment: &Treatment) {
    for d in world.query::<Disease>() {
        if let Some(dis) = world.get_component_mut::<Disease>(d) {
            if dis.patient != patient { continue; }
            dis.severity *= 1.0 - treatment.efficacy * treatment.dose;
            if dis.severity < 0.1 {
                dis.stage = Stage::Recovering;
            }
        }
    }
}

/// Apply a treatment directly to a patient's health, for callers that track
/// condition without a per-patient Disease component (e.g. outbreak scenarios
/// where severity is modeled as a direct health decrement). The healing
/// scales with the treatment's efficacy and dose, so a weak remedy helps
/// less than a strong one.
pub fn treat_health(health: &mut Health, treatment: &Treatment) {
    let boost = treatment.efficacy * treatment.dose * 0.4;
    health.condition = (health.condition + boost).min(1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engine::ecs::System;

    #[test]
    fn test_stage_follows_severity() {
        assert_eq!(stage_for(0.9), Stage::Critical);
        assert_eq!(stage_for(0.4), Stage::Moderate);
        assert_eq!(stage_for(0.05), Stage::Recovering);
    }

    #[test]
    fn test_disease_only_harms_its_patient() {
        let mut w = World::new();
        let sick = w.create_entity();
        let well = w.create_entity();
        w.add_component(sick, Health { condition: 0.9, immunity: 0.5 });
        w.add_component(well, Health { condition: 0.9, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), patient: sick, stage: Stage::Mild, severity: 0.4, contagious: false });
        IllnessSystem.run(&mut w);
        assert!(w.get_component::<Health>(sick).unwrap().condition < 0.9);
        assert_eq!(w.get_component::<Health>(well).unwrap().condition, 0.9);
    }

    #[test]
    fn test_patient_recovers_and_disease_clears() {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, Health { condition: 0.7, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), patient: p, stage: Stage::Mild, severity: 0.4, contagious: false });
        let mut lowest = 1.0f32;
        for _ in 0..300 {
            IllnessSystem.run(&mut w);
            lowest = lowest.min(w.get_component::<Health>(p).unwrap().condition);
        }
        assert!(lowest < 0.7, "the disease should do some harm first");
        assert!(w.get_component::<Health>(p).unwrap().condition >= RECOVERED, "patient should get better");
        assert!(w.get_component::<Disease>(d).is_none(), "a beaten disease should be cleared");
    }

    fn villager_at(w: &mut World, x: f32, immunity: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Health { condition: 0.9, immunity });
        w.add_component(e, crate::game::villager::Pos { x, y: 0.0 });
        e
    }

    fn infect(w: &mut World, patient: Entity, contagious: bool) {
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), patient, stage: Stage::Moderate, severity: 0.5, contagious });
    }

    fn is_sick(w: &World, e: Entity) -> bool {
        w.query::<Disease>().iter().any(|d| w.get_component::<Disease>(*d).unwrap().patient == e)
    }

    #[test]
    fn test_contagion_spreads_to_close_contacts_only() {
        let mut w = World::new();
        let zero = villager_at(&mut w, 0.0, 0.5);
        let neighbour = villager_at(&mut w, 2.0, 0.5);
        let far = villager_at(&mut w, 20.0, 0.5);
        infect(&mut w, zero, true);
        for _ in 0..40 { IllnessSystem.run(&mut w); }
        assert!(is_sick(&w, neighbour), "sustained close contact should pass the flu on");
        assert!(!is_sick(&w, far), "someone far away shouldn't catch it");
    }

    #[test]
    fn test_non_contagious_disease_does_not_spread() {
        let mut w = World::new();
        let zero = villager_at(&mut w, 0.0, 0.5);
        let neighbour = villager_at(&mut w, 1.0, 0.5);
        infect(&mut w, zero, false);
        for _ in 0..40 { IllnessSystem.run(&mut w); }
        assert!(!is_sick(&w, neighbour));
    }

    #[test]
    fn test_strong_immunity_resists_brief_contact() {
        let mut w = World::new();
        let zero = villager_at(&mut w, 0.0, 0.5);
        let weak = villager_at(&mut w, 1.0, 0.1);
        let strong = villager_at(&mut w, -1.0, 0.95);
        infect(&mut w, zero, true);
        for _ in 0..10 { IllnessSystem.run(&mut w); }
        assert!(is_sick(&w, weak), "low immunity should catch it quickly");
        assert!(!is_sick(&w, strong), "high immunity should hold out through brief contact");
    }

    #[test]
    fn test_beating_a_disease_builds_immunity() {
        let mut w = World::new();
        let p = villager_at(&mut w, 0.0, 0.5);
        infect(&mut w, p, false);
        for _ in 0..300 { IllnessSystem.run(&mut w); }
        assert!(!is_sick(&w, p));
        assert!(w.get_component::<Health>(p).unwrap().immunity > 0.5, "recovering should leave acquired immunity");
    }

    #[test]
    fn test_treatment_reduces_severity() {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, Health { condition: 0.5, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), patient: p, stage: Stage::Mild, severity: 0.5, contagious: false });
        let q = w.create_entity();
        w.add_component(q, Health { condition: 0.5, immunity: 0.5 });
        let other = w.create_entity();
        w.add_component(other, Disease { name: "flu".into(), patient: q, stage: Stage::Mild, severity: 0.5, contagious: false });
        let t = Treatment { name: "herb".into(), dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy: 0.8 };
        apply_treatment(&mut w, p, &t);
        assert!(w.get_component::<Disease>(d).unwrap().severity < 0.2);
        assert_eq!(w.get_component::<Disease>(other).unwrap().severity, 0.5, "only the treated patient's disease eases");
    }

    #[test]
    fn test_treat_health_scales_with_efficacy() {
        let mut weak = Health { condition: 0.3, immunity: 0.5 };
        let mut strong = Health { condition: 0.3, immunity: 0.5 };
        let weak_remedy = Treatment { name: "weak".into(), dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy: 0.1 };
        let strong_remedy = Treatment { name: "strong".into(), dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy: 0.9 };
        treat_health(&mut weak, &weak_remedy);
        treat_health(&mut strong, &strong_remedy);
        assert!(strong.condition > weak.condition, "a more effective remedy should heal more");
    }

    #[test]
    fn test_treat_health_caps_at_full() {
        let mut h = Health { condition: 0.95, immunity: 0.5 };
        let t = Treatment { name: "strong".into(), dose: 2.0, absorption: 1.0, clearance: 0.1, efficacy: 1.0 };
        treat_health(&mut h, &t);
        assert!(h.condition <= 1.0);
    }

    #[test]
    fn test_a_patient_killed_by_disease_stays_dead() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, Health { condition: 0.02, immunity: 0.9 });
        w.add_component(p, crate::game::villager::Pos { x: 0.0, y: 0.0 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "plague".into(), patient: p, stage: Stage::Critical, severity: 1.0, contagious: true });
        let neighbour = w.create_entity();
        w.add_component(neighbour, Health { condition: 1.0, immunity: 0.0 });
        w.add_component(neighbour, crate::game::villager::Pos { x: 1.0, y: 0.0 });

        for _ in 0..3 { IllnessSystem.run(&mut w); }
        assert_eq!(w.get_component::<Health>(p).unwrap().condition, 0.0);
        assert!(w.query::<Disease>().iter().all(|d| w.get_component::<Disease>(*d).unwrap().patient != p),
            "the disease ends with its patient");
        for _ in 0..500 { IllnessSystem.run(&mut w); }
        assert_eq!(w.get_component::<Health>(p).unwrap().condition, 0.0, "the dead don't recover");
    }

    #[test]
    fn test_the_dead_do_not_catch_disease() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let sick = w.create_entity();
        w.add_component(sick, Health { condition: 0.9, immunity: 0.0 });
        w.add_component(sick, crate::game::villager::Pos { x: 0.0, y: 0.0 });
        let d = w.create_entity();
        w.add_component(d, Disease { name: "flu".into(), patient: sick, stage: Stage::Severe, severity: 0.8, contagious: true });
        let corpse = w.create_entity();
        w.add_component(corpse, Health { condition: 0.0, immunity: 0.0 });
        w.add_component(corpse, crate::game::villager::Pos { x: 1.0, y: 0.0 });
        for _ in 0..50 { IllnessSystem.run(&mut w); }
        assert!(w.query::<Disease>().iter().all(|d| w.get_component::<Disease>(*d).unwrap().patient != corpse));
    }
}
