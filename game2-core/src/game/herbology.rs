//! Herbology: witchy apothecary medicine tree.
//! From backlog: "simple witchy apothecary style medicine tree"
//! Plants/natural stuff that is pseudo-medical.

use crate::engine::ecs::{Entity, World};
use crate::sim::illness::Treatment;

/// A medicinal herb/plant
#[derive(Debug)]
pub struct Herb {
    pub name: String,
    pub potency: f32,       // 0-1 strength
    pub effect: HerbEffect,
    pub preparation: PrepMethod,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HerbEffect {
    Healing,
    Calming,
    Energizing,
    PainRelief,
    Purifying,
    Vision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrepMethod {
    Tea,        // steeped in water
    Poultice,   // crushed, applied
    Tincture,   // alcohol extract
    Salve,      // infused in fat
    Smoke,      // burned, inhaled
}

/// Herbal remedy = herb + preparation
#[derive(Debug)]
pub struct Remedy {
    pub herb: Entity,
    pub method: PrepMethod,
    pub efficacy: f32,
}

/// Apothecary knowledge (learned herbs/recipes)
#[derive(Debug)]
pub struct Apothecary {
    pub known_herbs: Vec<String>,
    pub remedies: Vec<Remedy>,
    pub skill: f32,
}

/// Herb tree recipes (witchy medicine progression)
pub fn herb_tree() -> Vec<(String, HerbEffect, PrepMethod, f32)> {
    vec![
        ("chamomile".into(), HerbEffect::Calming, PrepMethod::Tea, 0.4),
        ("willow_bark".into(), HerbEffect::PainRelief, PrepMethod::Tea, 0.6),
        ("yarrow".into(), HerbEffect::Healing, PrepMethod::Poultice, 0.5),
        ("lavender".into(), HerbEffect::Calming, PrepMethod::Salve, 0.5),
        ("peppermint".into(), HerbEffect::Energizing, PrepMethod::Tea, 0.4),
        ("mugwort".into(), HerbEffect::Vision, PrepMethod::Smoke, 0.3),
        ("sage".into(), HerbEffect::Purifying, PrepMethod::Smoke, 0.6),
    ]
}

/// Spawn an herb
pub fn spawn_herb(world: &mut World, name: &str, potency: f32, effect: HerbEffect, method: PrepMethod) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Herb {
        name: name.into(),
        potency,
        effect,
        preparation: method,
    });
    e
}

/// Skill an apothecary gains each time they prepare a remedy
pub const PRACTICE_GAIN: f32 = 0.02;

/// Whether an herb's effect treats illness (vs. mood/energy/visions)
pub fn treats_illness(effect: HerbEffect) -> bool {
    matches!(effect, HerbEffect::Healing | HerbEffect::PainRelief | HerbEffect::Purifying)
}

/// An apothecary prepares `herb` by `method` into a treatment for the sick.
/// They must know the herb, and it must be medicinal. Efficacy is the
/// herb's potency scaled by the apothecary's skill, halved when it isn't
/// prepared the traditional way. Each preparation is recorded and is
/// practice that improves the apothecary's skill.
pub fn prepare_remedy(world: &mut World, apothecary: Entity, herb: Entity, method: PrepMethod) -> Option<Treatment> {
    let (name, potency, effect, traditional) = world.get_component::<Herb>(herb)
        .map(|h| (h.name.clone(), h.potency, h.effect, h.preparation))?;
    if !treats_illness(effect) { return None; }
    let ap = world.get_component_mut::<Apothecary>(apothecary)?;
    if !ap.known_herbs.contains(&name) { return None; }
    let fit = if method == traditional { 1.0 } else { 0.5 };
    let efficacy = potency * ap.skill * fit;
    ap.remedies.push(Remedy { herb, method, efficacy });
    ap.skill = (ap.skill + PRACTICE_GAIN).min(1.0);
    Some(Treatment {
        name: format!("{} {:?}", name, method).to_lowercase(),
        dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(w: &mut World, known: &[&str], skill: f32) -> Entity {
        let a = w.create_entity();
        w.add_component(a, Apothecary { known_herbs: known.iter().map(|s| s.to_string()).collect(), remedies: vec![], skill });
        a
    }

    #[test]
    fn test_known_herb_makes_remedy_and_builds_skill() {
        let mut w = World::new();
        let ap = setup(&mut w, &["willow_bark"], 0.5);
        let willow = spawn_herb(&mut w, "willow_bark", 0.6, HerbEffect::PainRelief, PrepMethod::Tea);
        let t = prepare_remedy(&mut w, ap, willow, PrepMethod::Tea).unwrap();
        assert!((t.efficacy - 0.3).abs() < 1e-6);
        let a = w.get_component::<Apothecary>(ap).unwrap();
        assert_eq!(a.remedies.len(), 1);
        assert!(a.skill > 0.5);
    }

    #[test]
    fn test_wrong_preparation_is_weaker() {
        let mut w = World::new();
        let ap = setup(&mut w, &["yarrow"], 1.0);
        let yarrow = spawn_herb(&mut w, "yarrow", 0.5, HerbEffect::Healing, PrepMethod::Poultice);
        let right = prepare_remedy(&mut w, ap, yarrow, PrepMethod::Poultice).unwrap().efficacy;
        let wrong = prepare_remedy(&mut w, ap, yarrow, PrepMethod::Smoke).unwrap().efficacy;
        assert!(wrong < right);
    }

    #[test]
    fn test_unknown_or_non_medicinal_herb_gives_nothing() {
        let mut w = World::new();
        let ap = setup(&mut w, &["chamomile"], 1.0);
        let willow = spawn_herb(&mut w, "willow_bark", 0.6, HerbEffect::PainRelief, PrepMethod::Tea);
        let chamomile = spawn_herb(&mut w, "chamomile", 0.4, HerbEffect::Calming, PrepMethod::Tea);
        assert!(prepare_remedy(&mut w, ap, willow, PrepMethod::Tea).is_none(), "unknown herb");
        assert!(prepare_remedy(&mut w, ap, chamomile, PrepMethod::Tea).is_none(), "calming isn't medicine");
    }
}
