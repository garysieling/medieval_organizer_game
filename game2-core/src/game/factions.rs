//! Factions: asymmetric powers, ruler personality variation.
//! From backlog: rulers differ - authoritarian vs permissive, narcissist, adhd.

use crate::engine::ecs::{Entity, World};

/// A faction (castle, village, guild, etc.)
#[derive(Debug)]
pub struct Faction {
    pub name: String,
    pub faction_type: FactionType,
    pub ruler: Option<Entity>,
    pub territory: Vec<String>,   // regions/settlements controlled
    pub power: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FactionType {
    Castle,
    Village,
    Guild,
    Religion,
    Empire,
}

/// Ruler personality affects governance
#[derive(Debug)]
pub struct Ruler {
    pub name: String,
    pub style: GovernanceStyle,
    pub personality: RulerTrait,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GovernanceStyle {
    Authoritarian,   // tight control, no delegation
    Permissive,      // delegates control freely
    Bureaucratic,    // process-driven
    Personal,        // rules through relationships
}

/// Ruler personality traits (affects behavior)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RulerTrait {
    Narcissist,
    ADHD,
    Calculating,
    Compassionate,
    Impulsive,
}

/// Spawn a faction with a ruler
pub fn spawn_faction(world: &mut World, name: &str, ftype: FactionType, ruler_name: &str, style: GovernanceStyle, trait_: RulerTrait) -> Entity {
    let r = world.create_entity();
    world.add_component(r, Ruler { name: ruler_name.into(), style, personality: trait_ });
    let f = world.create_entity();
    world.add_component(f, Faction {
        name: name.into(), faction_type: ftype, ruler: Some(r),
        territory: vec![], power: 50.0,
    });
    f
}
