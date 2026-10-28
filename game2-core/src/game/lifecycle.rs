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

/// System: ages advance each tick (1 tick = 1 month = 0.083 years).
/// A person dies once they outlive their `max_age` or their health gives
/// out; the dead stop aging and stay Dead.
pub struct AgingSystem;
impl crate::engine::ecs::System for AgingSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Age>() {
            // Someone already killed by illness or hunger (Health 0, or marked
            // Dead by the windowed game) is dead here too: mark them so, and
            // they stop aging
            let health_gone = crate::game::villager::is_dead(world, e);
            if health_gone {
                if let Some(a) = world.get_component_mut::<Age>(e) {
                    if a.stage == LifeStage::Dead { continue; }
                    a.stage = LifeStage::Dead;
                }
                if let Some(v) = world.get_component_mut::<Vitality>(e) {
                    v.health = 0.0;
                    v.cause_of_death.get_or_insert_with(|| "illness or hunger".to_string());
                }
                continue;
            }
            let years = match world.get_component_mut::<Age>(e) {
                Some(a) if a.stage != LifeStage::Dead => {
                    a.years += 0.083;
                    a.stage = stage_for(a.years);
                    a.years
                }
                _ => continue,
            };
            let died = match world.get_component_mut::<Vitality>(e) {
                Some(v) => {
                    let cause = if years >= v.max_age { Some("old age") }
                        else if v.health <= 0.0 { Some("poor health") }
                        else { None };
                    if let Some(c) = cause {
                        v.health = 0.0;
                        v.cause_of_death = Some(c.to_string());
                    }
                    cause.is_some()
                }
                None => false,
            };
            if died {
                world.get_component_mut::<Age>(e).unwrap().stage = LifeStage::Dead;
                // The rest of the sim reads death through villager::is_dead
                // (hunger, food, services, grief, pets, orgs), so make it
                // agree: Health to 0, or the Dead marker if there's no Health
                if let Some(h) = world.get_component_mut::<crate::sim::illness::Health>(e) {
                    h.condition = 0.0;
                } else {
                    world.add_component(e, crate::game::villager::Dead);
                }
            }
        }
    }
}

/// Life stage of a living person of the given age
fn stage_for(years: f32) -> LifeStage {
    match years {
        y if y < 1.0 => LifeStage::Infant,
        y if y < 13.0 => LifeStage::Child,
        y if y < 18.0 => LifeStage::Adolescent,
        y if y < 60.0 => LifeStage::Adult,
        _ => LifeStage::Elder,
    }
}

/// Spawn a newborn
pub fn spawn_person(world: &mut World, _name: &str) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Age { years: 0.0, stage: LifeStage::Infant });
    world.add_component(e, Vitality { health: 1.0, max_age: 70.0, cause_of_death: None });
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    #[test]
    fn test_aging_stages() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Aldric");
        w.get_component_mut::<Age>(p).unwrap().years = 25.0;
        // Age stage derives from years on next tick
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Infant);
        AgingSystem.run(&mut w);
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Adult);
    }

    #[test]
    fn test_dies_of_old_age_and_stays_dead() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Wystan");
        w.get_component_mut::<Age>(p).unwrap().years = 69.95;
        AgingSystem.run(&mut w);
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Dead);
        let v = w.get_component::<Vitality>(p).unwrap();
        assert_eq!(v.cause_of_death.as_deref(), Some("old age"));
        assert_eq!(v.health, 0.0);
        let years = w.get_component::<Age>(p).unwrap().years;
        for _ in 0..12 { AgingSystem.run(&mut w); }
        let a = w.get_component::<Age>(p).unwrap();
        assert_eq!(a.stage, LifeStage::Dead, "the dead must not age back into Elder");
        assert_eq!(a.years, years, "the dead stop aging");
    }

    #[test]
    fn test_dies_when_health_runs_out() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Bram");
        w.get_component_mut::<Age>(p).unwrap().years = 30.0;
        w.get_component_mut::<Vitality>(p).unwrap().health = 0.0;
        AgingSystem.run(&mut w);
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Dead);
        assert_eq!(w.get_component::<Vitality>(p).unwrap().cause_of_death.as_deref(), Some("poor health"));
    }

    #[test]
    fn test_healthy_adult_lives() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Eliza");
        w.get_component_mut::<Age>(p).unwrap().years = 30.0;
        for _ in 0..120 { AgingSystem.run(&mut w); }
        assert_eq!(w.get_component::<Age>(p).unwrap().stage, LifeStage::Adult);
        assert!(w.get_component::<Vitality>(p).unwrap().cause_of_death.is_none());
    }

    #[test]
    fn test_death_from_illness_or_hunger_ends_aging() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let p = spawn_person(&mut w, "Bram");
        w.get_component_mut::<Age>(p).unwrap().years = 30.0;
        w.add_component(p, Health { condition: 0.0, immunity: 0.5 });
        for _ in 0..12 { AgingSystem.run(&mut w); }
        let a = w.get_component::<Age>(p).unwrap();
        assert_eq!(a.stage, LifeStage::Dead);
        assert_eq!(a.years, 30.0, "the dead stop aging");
        assert_eq!(w.get_component::<Vitality>(p).unwrap().cause_of_death.as_deref(), Some("illness or hunger"));
    }

    #[test]
    fn test_old_age_death_is_seen_by_the_rest_of_the_sim() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let p = spawn_person(&mut w, "Wystan");
        w.add_component(p, Health { condition: 0.9, immunity: 0.5 });
        w.get_component_mut::<Age>(p).unwrap().years = 69.95;
        AgingSystem.run(&mut w);
        assert_eq!(w.get_component::<Health>(p).unwrap().condition, 0.0);
    }
}

#[cfg(test)]
mod dead_marker_tests {
    use super::*;
    use crate::engine::ecs::System;
    use crate::game::villager::{is_dead, Dead};

    #[test]
    fn test_old_age_without_health_is_still_seen_as_dead() {
        let mut w = World::new();
        let p = spawn_person(&mut w, "Wystan");
        w.get_component_mut::<Age>(p).unwrap().years = 69.95;
        AgingSystem.run(&mut w);
        assert!(is_dead(&w, p), "the rest of the sim should see an old-age death");
    }

    #[test]
    fn test_marked_dead_stops_aging() {
        // the windowed game removes Health on death and marks the villager Dead
        let mut w = World::new();
        let p = spawn_person(&mut w, "Bram");
        w.get_component_mut::<Age>(p).unwrap().years = 30.0;
        w.add_component(p, Dead);
        for _ in 0..12 { AgingSystem.run(&mut w); }
        let a = w.get_component::<Age>(p).unwrap();
        assert_eq!(a.stage, LifeStage::Dead);
        assert_eq!(a.years, 30.0);
    }
}
