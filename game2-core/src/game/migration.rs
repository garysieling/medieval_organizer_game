//! Migration: new people arrive, bringing skills, ideas, conflicts.
//! Emergent: population grows, culture shifts, tension if resources scarce.

use crate::engine::ecs::{Entity, World};

/// A group of migrants
#[derive(Debug)]
pub struct MigrantGroup {
    pub origin: String,
    pub size: usize,
    pub skills: Vec<String>,
    pub wealth: f32,
    pub acceptance: f32,    // 0-1, how well they integrate
    pub ideology: String,
}

/// Migration event: new people arrive in the village
#[derive(Debug)]
pub struct MigrationEvent {
    pub migrants: MigrantGroup,
    pub trigger: MigrationTrigger,
    pub tick: u64,
}

#[derive(Debug)]
pub enum MigrationTrigger {
    Economic,      // seeking work/prosperity
    Fleeing,       // escaping conflict/disaster
    Invited,       // recruited by the village
    Seasonal,      // regular movement (trade/work)
}

/// Migrants arrive -> integrate (or conflict if resources scarce)
pub fn integrate(migrants: &MigrantGroup, resources_available: f32) -> f32 {
    // Acceptance based on resources + skills fit
    let fit = if resources_available > migrants.size as f32 * 10.0 { 0.7 } else { 0.3 };
    migrants.acceptance * fit + (1.0 - migrants.acceptance) * 0.5
}

/// Migration affects the village
pub fn migration_effects(migrants: &MigrantGroup) -> Vec<&'static str> {
    let mut effects = vec!["population grows"];
    if migrants.skills.contains(&"craft".to_string()) { effects.push("new craft knowledge"); }
    if migrants.skills.contains(&"trade".to_string()) { effects.push("trade connections"); }
    if migrants.wealth > 100.0 { effects.push("capital influx"); }
    if migrants.acceptance < 0.4 { effects.push("integration tension"); }
    effects
}

/// Spawn a migrant group
pub fn spawn_migrants(world: &mut World, origin: &str, size: usize, skills: Vec<&str>) -> Entity {
    let e = world.create_entity();
    world.add_component(e, MigrantGroup {
        origin: origin.into(), size,
        skills: skills.iter().map(|s| s.to_string()).collect(),
        wealth: 50.0, acceptance: 0.6,
        ideology: "worker".into(),
    });
    e
}
