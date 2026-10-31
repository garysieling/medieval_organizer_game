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

/// The period diseases a village can catch. Each kind is a Victorian-era
/// disease with its own profile: what the village calls it, how long it
/// brews before it shows, whether it passes between neighbours, and what
/// beating it leaves in the body.
/// (wiki/literature-reviews/victorian-diseases.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiseaseKind {
    /// "la grippe" - influenza; a traveller's gift, weak lasting immunity
    Influenza,
    /// "scarlatina" - scarlet fever; the childhood killer, strong immunity
    ScarletFever,
    /// "saint anthony's fire" - ergotism; it's in the rye
    Ergotism,
    /// "the gripes" - poisoning (death-caps, bad food)
    Gripes,
    /// "consumption" - tuberculosis; the long wasting
    Consumption,
    /// "the sweats" - sweating sickness
    Sweats,
}

impl DiseaseKind {
    /// What the village calls it
    pub fn name(self) -> &'static str {
        match self {
            DiseaseKind::Influenza => "la grippe",
            DiseaseKind::ScarletFever => "scarlatina",
            DiseaseKind::Ergotism => "saint anthony's fire",
            DiseaseKind::Gripes => "the gripes",
            DiseaseKind::Consumption => "consumption",
            DiseaseKind::Sweats => "the sweats",
        }
    }

    /// Days it brews before it shows - and before it can be shed. Poisons
    /// strike as they're taken; infections incubate.
    pub fn incubation(self) -> u32 {
        match self {
            DiseaseKind::Influenza => 2,   // 1-3 days
            DiseaseKind::ScarletFever => 3, // 2-5 days
            DiseaseKind::Ergotism => 0,    // poisoning, not infection
            DiseaseKind::Gripes => 1,      // the death-cap waits a day to tell
            DiseaseKind::Consumption => 0, // the slow kind - no acute onset
            DiseaseKind::Sweats => 2,
        }
    }

    /// Whether it passes villager to villager. The poisonings never do.
    pub fn contagious(self) -> bool {
        matches!(self, DiseaseKind::Influenza | DiseaseKind::ScarletFever | DiseaseKind::Sweats)
    }

    /// Immunity beating it teaches the body. Poisons teach nothing; the
    /// grippe fades; scarlatina protects for life.
    pub fn immunity_gain(self) -> f32 {
        match self {
            DiseaseKind::Influenza => 0.15,
            DiseaseKind::ScarletFever => 0.9,
            DiseaseKind::Ergotism => 0.0,
            DiseaseKind::Gripes => 0.0,
            DiseaseKind::Consumption => 0.1,
            DiseaseKind::Sweats => 0.4,
        }
    }
}

/// A disease progressing through compartments
#[derive(Debug)]
pub struct Disease {
    pub kind: DiseaseKind,
    pub name: String,
    pub patient: Entity, // who is sick with it
    pub stage: Stage,
    pub severity: f32,
    pub contagious: bool,
    /// Days left brewing before it shows - and before it can spread
    pub incubation: u32,
}

impl Disease {
    /// A fresh case of `kind` at full incubation, stage from severity
    pub fn new(kind: DiseaseKind, patient: Entity, severity: f32) -> Self {
        let incubation = kind.incubation();
        Disease {
            kind,
            name: kind.name().into(),
            patient,
            stage: if incubation > 0 { Stage::Latent } else { stage_for(severity) },
            severity,
            contagious: kind.contagious(),
            incubation,
        }
    }
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
            let (patient, stage, severity, kind) = match world.get_component::<Disease>(d) {
                Some(dis) => (dis.patient, dis.stage, dis.severity, dis.kind),
                None => continue,
            };
            let Some(h) = world.get_component_mut::<Health>(patient) else {
                world.destroy_entity(d); // patient is gone
                continue;
            };
            if h.condition <= 0.0 {
                // the dead don't recover or keep fighting off disease
                world.destroy_entity(d);
                continue;
            }
            if stage == Stage::Latent {
                // brewing quietly: no harm, nothing shed - the healthy-
                // looking carrier nobody knows to fear
                if let Some(dis) = world.get_component_mut::<Disease>(d) {
                    dis.incubation = dis.incubation.saturating_sub(1);
                    if dis.incubation == 0 { dis.stage = stage_for(dis.severity); }
                }
                continue;
            }
            if stage == Stage::Recovering {
                h.condition = (h.condition + RECOVERY_RATE).min(1.0);
                if h.condition >= RECOVERED {
                    // beating it leaves them harder to reinfect - how much
                    // harder depends on the disease (poisons teach nothing)
                    h.immunity = (h.immunity + kind.immunity_gain()).min(1.0);
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

/// How much contagion a healthy agent has soaked up from the sick around them
#[derive(Debug, Default)]
pub struct Exposure(pub f32);

/// Contagious diseases spread by proximity: each tick, healthy agents within
/// CONTACT_RANGE of an actively sick one build up exposure in proportion to
/// that patient's severity (fading when they're away from the sick); once
/// exposure outweighs their immunity they catch a slightly milder case.
/// The latent don't shed - a brewing sickness keeps its secret.
fn spread_contagion(world: &mut World) {
    use crate::game::villager::Pos;
    let mut sources = vec![]; // (position, severity, kind)
    let mut sick = vec![];
    for d in world.query::<Disease>() {
        let Some(dis) = world.get_component::<Disease>(d) else { continue };
        sick.push(dis.patient);
        if !dis.contagious || dis.stage == Stage::Recovering || dis.stage == Stage::Latent { continue; }
        if let Some(p) = world.get_component::<Pos>(dis.patient) {
            sources.push(((p.x, p.y), dis.severity, dis.kind));
        }
    }
    for e in world.query::<Health>() {
        if sick.contains(&e) { continue; }
        if crate::game::villager::is_dead(world, e) { continue; } // the dead don't catch it
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
        // catch the worst case they were exposed to, a little weakened -
        // and brewing its own incubation from scratch
        let (_, sev, kind) = near.iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .copied()
            .cloned()
            .unwrap_or(((0.0, 0.0), 0.0, DiseaseKind::Influenza));
        let severity = sev * TRANSMITTED_SEVERITY;
        if severity < 0.1 { continue; } // too weak to take hold
        let d = world.create_entity();
        world.add_component(d, Disease::new(kind, e, severity));
    }
}

fn stage_for(severity: f32) -> Stage {
    if severity < 0.1 { Stage::Recovering }
    else if severity < 0.3 { Stage::Mild }
    else if severity < 0.6 { Stage::Moderate }
    else if severity < 0.85 { Stage::Severe }
    else { Stage::Critical }
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
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: sick, stage: Stage::Mild, severity: 0.4, contagious: false, incubation: 0 });
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
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: p, stage: Stage::Mild, severity: 0.4, contagious: false, incubation: 0 });
        let mut lowest = 1.0f32;
        for _ in 0..300 {
            IllnessSystem.run(&mut w);
            lowest = lowest.min(w.get_component::<Health>(p).unwrap().condition);
        }
        assert!(lowest < 0.7, "the disease should do some harm first");
        assert!(w.get_component::<Health>(p).unwrap().condition >= RECOVERED, "patient should get better");
        assert!(w.get_component::<Disease>(d).is_none(), "a beaten disease should be cleared");
    }

    #[test]
    fn test_a_dead_patient_does_not_revive_while_recovering() {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, Health { condition: 0.0, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: p, stage: Stage::Recovering, severity: 0.05, contagious: false, incubation: 0 });
        IllnessSystem.run(&mut w);
        assert_eq!(w.get_component::<Health>(p).unwrap().condition, 0.0, "the dead should stay dead");
        assert!(w.get_component::<Disease>(d).is_none(), "the dead don't keep fighting off disease");
    }

    fn villager_at(w: &mut World, x: f32, immunity: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Health { condition: 0.9, immunity });
        w.add_component(e, crate::game::villager::Pos { x, y: 0.0 });
        e
    }

    fn infect(w: &mut World, patient: Entity, contagious: bool) {
        let d = w.create_entity();
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient, stage: Stage::Moderate, severity: 0.5, contagious, incubation: 0 });
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
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: p, stage: Stage::Mild, severity: 0.5, contagious: false, incubation: 0 });
        let q = w.create_entity();
        w.add_component(q, Health { condition: 0.5, immunity: 0.5 });
        let other = w.create_entity();
        w.add_component(other, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: q, stage: Stage::Mild, severity: 0.5, contagious: false, incubation: 0 });
        let t = Treatment { name: "herb".into(), dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy: 0.8 };
        let before = w.get_component::<Health>(p).unwrap().condition;
        treat_health(w.get_component_mut::<Health>(p).unwrap(), &t);
        assert!(w.get_component::<Health>(p).unwrap().condition > before, "the dose does something");
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
        w.add_component(d, Disease { kind: DiseaseKind::Sweats, name: "plague".into(), patient: p, stage: Stage::Critical, severity: 1.0, contagious: true, incubation: 0 });
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
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: sick, stage: Stage::Severe, severity: 0.8, contagious: true, incubation: 0 });
        let corpse = w.create_entity();
        w.add_component(corpse, Health { condition: 0.0, immunity: 0.0 });
        w.add_component(corpse, crate::game::villager::Pos { x: 1.0, y: 0.0 });
        for _ in 0..50 { IllnessSystem.run(&mut w); }
        assert!(w.query::<Disease>().iter().all(|d| w.get_component::<Disease>(*d).unwrap().patient != corpse));
    }

    #[test]
    fn test_a_brewing_disease_shows_nothing_then_strikes() {
        let mut w = World::new();
        let p = villager_at(&mut w, 0.0, 0.5);
        let neighbour = villager_at(&mut w, 1.0, 0.0);
        let d = w.create_entity();
        w.add_component(d, Disease::new(DiseaseKind::Influenza, p, 0.6));
        // brewing quietly: no harm yet, and nothing shed to the neighbour
        IllnessSystem.run(&mut w);
        assert_eq!(w.get_component::<Health>(p).unwrap().condition, 0.9);
        assert!(!is_sick(&w, neighbour), "the latent shed nothing");
        // once the incubation's run, it bites
        for _ in 0..DiseaseKind::Influenza.incubation() { IllnessSystem.run(&mut w); }
        assert!(w.get_component::<Health>(p).unwrap().condition < 0.9, "the fever breaks cover");
        assert!(is_sick(&w, neighbour), "and now it spreads");
    }

    #[test]
    fn test_beating_scarlatina_protects_longer_than_la_grippe() {
        for (kind, want) in [(DiseaseKind::ScarletFever, 1.0f32), (DiseaseKind::Influenza, 0.65f32)] {
            let mut w = World::new();
            let p = villager_at(&mut w, 0.0, 0.5);
            let d = w.create_entity();
            let mut dis = Disease::new(kind, p, 0.4);
            dis.incubation = 0;
            dis.stage = Stage::Mild;
            w.add_component(d, dis);
            for _ in 0..400 { IllnessSystem.run(&mut w); }
            let got = w.get_component::<Health>(p).unwrap().immunity;
            assert!((got - want).abs() < 0.01, "{:?} should leave immunity {}, got {}", kind, want, got);
        }
    }
}
