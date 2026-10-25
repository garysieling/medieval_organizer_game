//! Basic Entity Component System (ECS) foundation
//!
//! Entities are u32 ids. Components are plain data structs stored per-type
//! in a hash map keyed by entity. Systems iterate component sets and apply
//! logic. Kept intentionally small - this is a foundation to build on.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A unique entity identifier
pub type Entity = u32;

/// Storage for one component type: entity -> component
type ComponentStorage = HashMap<Entity, Box<dyn Any>>;

/// The world holds entities and all component storages
pub struct World {
    next_entity: Entity,
    storages: HashMap<TypeId, ComponentStorage>,
}

impl World {
    /// Create an empty world
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            storages: HashMap::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let e = self.next_entity;
        self.next_entity += 1;
        e
    }

    /// Add a component to an entity
    pub fn add_component<T: Any>(&mut self, entity: Entity, component: T) {
        self.storages
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(entity, Box::new(component));
    }

    /// Get a component for an entity
    pub fn get_component<T: Any>(&self, entity: Entity) -> Option<&T> {
        self.storages
            .get(&TypeId::of::<T>())?
            .get(&entity)?
            .downcast_ref::<T>()
    }

    /// Get a mutable component for an entity
    pub fn get_component_mut<T: Any>(&mut self, entity: Entity) -> Option<&mut T> {
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .get_mut(&entity)?
            .downcast_mut::<T>()
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> bool {
        if let Some(storage) = self.storages.get_mut(&TypeId::of::<T>()) {
            return storage.remove(&entity).is_some();
        }
        false
    }

    /// Destroy an entity and all its components
    pub fn destroy_entity(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.remove(&entity);
        }
    }

    /// Iterate over all entities that have component T
    pub fn query<T: Any>(&self) -> Vec<Entity> {
        match self.storages.get(&TypeId::of::<T>()) {
            Some(storage) => storage.keys().copied().collect(),
            None => Vec::new(),
        }
    }

    /// Total number of live entities (approx: highest id issued)
    pub fn entity_count(&self) -> u32 {
        self.next_entity
    }
}

/// A system processes world state each tick
pub trait System {
    /// Run the system once against the world
    fn run(&mut self, world: &mut World);
}

/// Scheduler runs registered systems in order each tick
pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { systems: Vec::new() }
    }

    /// Register a system to run each tick
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Run all systems once (one tick)
    pub fn tick(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position { x: f32, y: f32 }
    struct Velocity { dx: f32, dy: f32 }

    struct MovementSystem;
    impl System for MovementSystem {
        fn run(&mut self, world: &mut World) {
            for e in world.query::<Velocity>() {
                let (vx, vy) = {
                    let v = world.get_component::<Velocity>(e).unwrap();
                    (v.dx, v.dy)
                };
                if let Some(pos) = world.get_component_mut::<Position>(e) {
                    pos.x += vx;
                    pos.y += vy;
                }
            }
        }
    }

    #[test]
    fn test_entity_component() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        assert!(world.get_component::<Position>(e).is_some());
    }

    #[test]
    fn test_movement_system() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        world.add_component(e, Velocity { dx: 1.0, dy: 2.0 });

        let mut sched = Scheduler::new();
        sched.add_system(MovementSystem);
        sched.tick(&mut world);

        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_destroy_entity() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        world.destroy_entity(e);
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e2, Position { x: 1.0, y: 1.0 });

        assert_eq!(world.query::<Position>().len(), 2);
        assert_eq!(world.query::<Velocity>().len(), 0);
    }
}
