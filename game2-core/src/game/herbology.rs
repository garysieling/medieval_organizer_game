//! Herbology: witchy apothecary medicine tree.
//! From backlog: "simple witchy apothecary style medicine tree"
//! Plants/natural stuff that is pseudo-medical.

use crate::engine::ecs::{Entity, World};

/// A medicinal herb/plant
#[derive(Debug)]
pub struct Herb {
    pub name: String,
    pub potency: f32,       // 0-1 strength
    pub effect: HerbEffect,
    pub preparation: PrepMethod,
}

#[derive(Debug)]
pub enum HerbEffect {
    Healing,
    Calming,
    Energizing,
    PainRelief,
    Purifying,
    Vision,
}

#[derive(Debug)]
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
