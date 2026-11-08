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
    /// "the cholera" - Asiatic cholera; foul water, kills in days
    Cholera,
    /// "the slow fever" - typhoid; foul water, a long hot month
    Typhoid,
    /// "the flux" - dysentery; foul water, the summer sickness
    Dysentery,
    /// "the ague" - fenland malaria; the marsh's autumn visitor, always
    /// coming back
    Ague,
    /// "the pestilence" - the plague; it walks the roads between
    /// settlements on the carriers' backs, not on any season's calendar
    Plague,
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
            DiseaseKind::Cholera => "the cholera",
            DiseaseKind::Typhoid => "the slow fever",
            DiseaseKind::Dysentery => "the flux",
            DiseaseKind::Ague => "the ague",
            DiseaseKind::Plague => "the pestilence",
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
            DiseaseKind::Cholera => 1,     // hours to a few days - terrifyingly fast
            DiseaseKind::Typhoid => 10,    // a week or three before it shows
            DiseaseKind::Dysentery => 2,
            DiseaseKind::Ague => 10,       // a fortnight's brewing, tertian
            DiseaseKind::Plague => 4,      // a few days' silent walk before the bubo tells
        }
    }

    /// The part of the year it rides in on, if it has one (day-of-year
    /// window on the 365-day cycle). A window that wraps the year -
    /// start past end - reads winter into the new year.
    pub fn season(self) -> Option<(u32, u32)> {
        match self {
            DiseaseKind::Ague => Some((240, 310)),       // the autumn marsh fever
            DiseaseKind::Consumption => Some((300, 90)), // winter's long cough
            DiseaseKind::ScarletFever => Some((330, 120)),// winter into spring
            DiseaseKind::Sweats => Some((210, 300)),     // late summer's heat
            DiseaseKind::Cholera => Some((180, 240)),    // summer's foul water
            DiseaseKind::Typhoid => Some((240, 300)),    // autumn's slow fever
            DiseaseKind::Dysentery => Some((180, 240)),  // summer's flux
            DiseaseKind::Influenza => Some((300, 60)),   // winter's grippe
            // the pestilence keeps no season - it comes when the roads carry it
            _ => None,                                 // Ergotism/Gripes - no season
        }
    }

    /// Whether a day-of-year falls inside the kind's season - a window
    /// that wraps the year (start past end) covers winter into spring
    pub fn in_season(self, day: u32) -> bool {
        match self.season() {
            Some((s0, s1)) if s0 <= s1 => (s0..=s1).contains(&day),
            Some((s0, s1)) => day >= s0 || day <= s1,
            None => true, // no season keeps it year-round
        }
    }

    /// How it travels. Snow's lesson: the waterborne kinds ride the well,
    /// not the neighbour - they can't be caught by standing too close.
    pub fn route(self) -> Route {
        match self {
            DiseaseKind::Influenza | DiseaseKind::ScarletFever
            | DiseaseKind::Sweats | DiseaseKind::Consumption => Route::Contact,
            DiseaseKind::Cholera | DiseaseKind::Typhoid | DiseaseKind::Dysentery => Route::Water,
            DiseaseKind::Ergotism | DiseaseKind::Gripes => Route::Ingested,
            DiseaseKind::Ague => Route::Vector,
            DiseaseKind::Plague => Route::Contact,
        }
    }

    /// Whether it passes villager to villager by proximity. The waterborne
    /// kinds spread through the reach instead; the poisonings never spread.
    pub fn contagious(self) -> bool {
        self.route() == Route::Contact
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
            DiseaseKind::Cholera => 0.4,
            DiseaseKind::Typhoid => 0.8,
            DiseaseKind::Dysentery => 0.1,
            DiseaseKind::Ague => 0.05, // the marsh always sends it back
            DiseaseKind::Plague => 0.05, // and the pestilence leaves the body nearly as it found it
        }
    }

    /// Whether beating it truly ends it: the relapsing kinds leave a
    /// dormant seed - (days dormant, severity it wakes at). The ague
    /// sleeps through winter and shakes again; the wasting wakes years on.
    pub fn relapse(self) -> Option<(u32, f32)> {
        match self {
            DiseaseKind::Ague => Some((300, 0.3)),
            DiseaseKind::Consumption => Some((400, 0.35)),
            _ => None, // the pestilence doesn't come back - it burns through and is gone
        }
    }
}

/// How a disease travels between the sick and the well
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Breathed, coughed, touched - passes neighbour to neighbour
    Contact,
    /// Carried in foul water - the reach does the spreading
    Water,
    /// Swallowed - something eaten, not caught
    Ingested,
    /// Carried by what bites - the marsh air, not the neighbour
    Vector,
}

/// A beaten disease lying dormant. The relapsing kinds leave one of
/// these behind on the patient; when its days are counted out it
/// blooms again on its own - no fresh exposure needed.
#[derive(Debug)]
pub struct Relapse {
    pub kind: DiseaseKind,
    pub due_in: u32,
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
            // the half-starved fight it worse: hunger halves what their
            // immune system can clear - malnutrition is a symptom too
            let hungry = world.get_component::<crate::game::villager::Hunger>(patient)
                .map(|hh| hh.value < 40.0).unwrap_or(false);
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
                    if let Some((due_in, _)) = kind.relapse() {
                        // it goes quiet, not gone - the seed stays in them
                        world.add_component(patient, Relapse { kind, due_in });
                    }
                    world.destroy_entity(d);
                }
                continue;
            }
            h.condition = (h.condition - severity * DAMAGE_RATE).max(0.0);
            if h.condition <= 0.0 {
                world.destroy_entity(d); // the disease killed them
                continue;
            }
            let severity = (severity - h.immunity * IMMUNE_RATE * if hungry { 0.5 } else { 1.0 }).max(0.0);
            if let Some(dis) = world.get_component_mut::<Disease>(d) {
                dis.severity = severity;
                dis.stage = stage_for(severity);
            }
        }
        // the dormant wake: a beaten-but-not-gone disease fires again
        let mut wake = vec![];
        for e in world.query::<Relapse>() {
            let alive = world.get_component::<Health>(e)
                .map(|h| h.condition > 0.0).unwrap_or(false);
            if !alive {
                world.remove_component::<Relapse>(e); // the dead stay dead
                continue;
            }
            // holding a live case already? the seed waits it out
            let carrying = world.query::<Disease>().iter().any(|&d|
                world.get_component::<Disease>(d).map(|dd| dd.patient == e).unwrap_or(false));
            if carrying { continue; }
            let Some(r) = world.get_component_mut::<Relapse>(e) else { continue };
            r.due_in = r.due_in.saturating_sub(1);
            if r.due_in == 0 { wake.push((e, r.kind)); }
        }
        for (e, kind) in wake {
            world.remove_component::<Relapse>(e);
            let severity = kind.relapse().map(|(_, s)| s).unwrap_or(0.3);
            let d = world.create_entity();
            world.add_component(d, Disease::new(kind, e, severity));
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
    let mut sources = vec![]; // (position, severity, kind, patient)
    let mut sick = vec![];
    for d in world.query::<Disease>() {
        let Some(dis) = world.get_component::<Disease>(d) else { continue };
        sick.push(dis.patient);
        if !dis.contagious || dis.stage == Stage::Recovering || dis.stage == Stage::Latent { continue; }
        if let Some(p) = world.get_component::<Pos>(dis.patient) {
            sources.push(((p.x, p.y), dis.severity, dis.kind, dis.patient));
        }
    }
    for e in world.query::<Health>() {
        if sick.contains(&e) { continue; }
        if crate::game::villager::is_dead(world, e) { continue; } // the dead don't catch it
        let Some(p) = world.get_component::<Pos>(e).map(|p| (p.x, p.y)) else { continue };
        let home = crate::game::household::home_of(world, e);
        let near: Vec<_> = sources.iter()
            .filter(|((x, y), _, _, _)| (x - p.0).abs() + (y - p.1).abs() <= CONTACT_RANGE)
            .collect();
        // under one roof the dose doubles - the fever respects the rent
        let dose: f32 = near.iter().map(|(_, sev, _, src)| {
            let crowding = if home.is_some() && home == crate::game::household::home_of(world, *src) { 2.0 } else { 1.0 };
            sev * EXPOSURE_RATE * crowding
        }).sum();
        if dose == 0.0 && world.get_component::<Exposure>(e).is_none() { continue; }
        if world.get_component::<Exposure>(e).is_none() {
            world.add_component(e, Exposure::default());
        }
        // hunger halves what their body can resist
        let hungry = world.get_component::<crate::game::villager::Hunger>(e)
            .map(|hh| hh.value < 40.0).unwrap_or(false);
        let immunity = world.get_component::<Health>(e).map(|h| h.immunity).unwrap_or(0.0)
            * if hungry { 0.5 } else { 1.0 };
        let exposure = world.get_component_mut::<Exposure>(e).unwrap();
        exposure.0 = exposure.0 * EXPOSURE_RETAINED + dose;
        if exposure.0 <= immunity { continue; }
        exposure.0 = 0.0;
        // catch the worst case they were exposed to, a little weakened -
        // and brewing its own incubation from scratch
        let (_, sev, kind, _) = near.iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .copied()
            .cloned()
            .unwrap_or(((0.0, 0.0), 0.0, DiseaseKind::Influenza, 0));
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

    #[test]
    fn test_the_hungry_catch_it_first() {
        use crate::game::villager::Hunger;
        let mut w = World::new();
        let zero = villager_at(&mut w, 0.0, 0.6);
        let fed = villager_at(&mut w, 1.0, 0.6);
        let starved = villager_at(&mut w, -1.0, 0.6);
        w.add_component(starved, Hunger { value: 10.0 });
        w.add_component(fed, Hunger { value: 90.0 });
        infect(&mut w, zero, true);
        for _ in 0..10 { IllnessSystem.run(&mut w); }
        assert!(is_sick(&w, starved), "hunger halves resistance - the starved fall first");
        assert!(!is_sick(&w, fed), "the fed hold out through brief contact");
    }

    #[test]
    fn test_the_same_roof_doubles_the_dose() {
        use crate::game::household::{Building, BuildingKind};
        use crate::engine::det::HashMap;
        let mut w = World::new();
        // two houses: the sick one shares hers, the neighbour's alone
        let sick_own = villager_at(&mut w, 0.0, 0.9);
        let family = villager_at(&mut w, 0.0, 0.9);
        let lodger = villager_at(&mut w, 3.0, 0.9);
        let h1 = w.create_entity();
        w.add_component(h1, Building { kind: BuildingKind::Home, name: "home".into(),
            occupants: vec![sick_own, family], worker: None, overseer: None,
            stock: HashMap::default(), capacity: 0, flyer: None });
        let h2 = w.create_entity();
        w.add_component(h2, Building { kind: BuildingKind::Home, name: "home".into(),
            occupants: vec![lodger], worker: None, overseer: None,
            stock: HashMap::default(), capacity: 0, flyer: None });
        // sanity: the fabricated home registers for both occupants
        assert_eq!(crate::game::household::home_of(&w, family), Some(h1));
        assert_eq!(crate::game::household::home_of(&w, sick_own), Some(h1));
        assert_eq!(crate::game::household::home_of(&w, lodger), Some(h2));
        infect(&mut w, sick_own, true);
        // family soaks the doubled household dose (~16 ticks to the
        // threshold); the lodger at the range's edge needs ~30
        for _ in 0..18 { IllnessSystem.run(&mut w); }
        assert!(is_sick(&w, family), "sharing a roof with the sick doubles the dose");
        assert!(!is_sick(&w, lodger), "the lodger at the range's edge holds out");
    }

    #[test]
    fn test_the_ague_comes_back_when_its_time_is_up() {
        let mut w = World::new();
        let p = villager_at(&mut w, 0.0, 0.5);
        let d = w.create_entity();
        // a case already on the mend - it clears on the first tick
        w.add_component(d, Disease { kind: DiseaseKind::Ague, name: "the ague".into(),
            patient: p, stage: Stage::Recovering, severity: 0.05, contagious: false, incubation: 0 });
        IllnessSystem.run(&mut w);
        assert!(!is_sick(&w, p));
        // but it left its seed: quiet for the dormant stretch, then it blooms
        for _ in 0..295 { IllnessSystem.run(&mut w); }
        assert!(w.query::<Disease>().iter().all(|&dd|
            w.get_component::<Disease>(dd).map(|x| x.patient != p).unwrap_or(true)),
            "the seed lies quiet through its dormancy");
        for _ in 0..10 { IllnessSystem.run(&mut w); }
        assert!(w.query::<Disease>().iter().any(|&dd|
            w.get_component::<Disease>(dd).map(|x| x.patient == p && x.kind == DiseaseKind::Ague).unwrap_or(false)),
            "the ague comes back when its time is up");
    }

    #[test]
    fn test_la_grippe_leaves_no_seed_behind() {
        let mut w = World::new();
        let p = villager_at(&mut w, 0.0, 0.5);
        let d = w.create_entity();
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "la grippe".into(),
            patient: p, stage: Stage::Recovering, severity: 0.05, contagious: false, incubation: 0 });
        IllnessSystem.run(&mut w);
        assert!(w.query::<Relapse>().is_empty(), "la grippe is gone for good");
    }
}
