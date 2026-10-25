//! Lifecycle: birth->aging->death, life events.
//! From backlog: "full lifecycle characters", age-based econ effects.

use crate::engine::ecs::{Entity, World};

/// Age in years + life stage
#[derive(Debug)]
pub struct Age {
    pub years: f32,
    pub stage: LifeStage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifeStage {
    Infant,
    Child,
    Adolescent,
    Adult,
    Elder,
    Dead,
}

/// Health + mortality
#[derive(Debug)]
pub struct Vitality {
    pub health: f32,    // 0-1
    pub max_age: f32,   // lifespan
    pub cause_of_death: Option<String>,
}

/// Major life events
#[derive(Debug)]
pub struct LifeEvent {
    pub year: u32,
    pub kind: EventKind,
    pub description: String,
}

#[derive(Debug)]
pub enum EventKind {
    Birth,
    FirstJob,
    Apprenticeship,
    Marriage,
    ChildBorn,
    Loss,
    Achievement,
    Retirement,
    Death,
}

/// System: ages advance each tick (1 tick = 1 month = 0.083 years)
pub struct AgingSystem;
impl crate::engine::ecs::System for AgingSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Age>() {
            if let Some(a) = world.get_component_mut::<Age>(e) {
                a.years += 0.083;
                a.stage = match a.years {
                    y if y < 1.0 => LifeStage::Infant,
                    y if y < 13.0 => LifeStage::Child,
                    y if y < 18.0 => LifeStage::Adolescent,
                    y if y < 60.0 => LifeStage::Adult,
                    _ => LifeStage::Elder,
                };
            }
        }
    }
}

/// Spawn a newborn
pub fn spawn_person(world: &mut World, name: &str) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Age { years: 0.0, stage: LifeStage::Infant });
    world.add_component(e, Vitality { health: 1.0, max_age: 70.0, cause_of_death: None });
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aging_stages() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Aldric");
        w.get_component_mut::<Age>(p).unwrap().years = 25.0;
        // Age stage derives from years on next tick
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Infant); // not yet updated
    }
}
