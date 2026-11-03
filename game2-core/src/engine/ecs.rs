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
type ComponentStorage = HashMap<Entity, Box<dyn Any + Send>>;

/// The world holds entities and all component storages
pub struct World {
    next_entity: Entity,
    storages: HashMap<TypeId, ComponentStorage>,
    /// Singleton data keyed by type rather than by entity: caches and
    /// indices the systems share (e.g. the relationship pair index).
    resources: HashMap<TypeId, Box<dyn Any + Send>>,
    /// Per-type structural revision: bumped whenever a component of that
    /// type is added or removed. An index that stored a revision behind
    /// the current one no longer provably covers that storage.
    mutation_rev: HashMap<TypeId, u64>,
}

impl World {
    /// Create an empty world
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            storages: HashMap::new(),
            resources: HashMap::new(),
            mutation_rev: HashMap::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let e = self.next_entity;
        self.next_entity += 1;
        e
    }

    /// Add a component to an entity
    pub fn add_component<T: Any + Send>(&mut self, entity: Entity, component: T) {
        self.storages
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(entity, Box::new(component));
        *self.mutation_rev.entry(TypeId::of::<T>()).or_default() += 1;
    }

    /// Get a component for an entity
    pub fn get_component<T: Any + Send>(&self, entity: Entity) -> Option<&T> {
        self.storages
            .get(&TypeId::of::<T>())?
            .get(&entity)?
            .downcast_ref::<T>()
    }

    /// Get a mutable component for an entity
    pub fn get_component_mut<T: Any + Send>(&mut self, entity: Entity) -> Option<&mut T> {
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .get_mut(&entity)?
            .downcast_mut::<T>()
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Any + Send>(&mut self, entity: Entity) -> bool {
        if let Some(storage) = self.storages.get_mut(&TypeId::of::<T>()) {
            if storage.remove(&entity).is_some() {
                *self.mutation_rev.entry(TypeId::of::<T>()).or_default() += 1;
                return true;
            }
        }
        false
    }

    /// Destroy an entity and all its components
    pub fn destroy_entity(&mut self, entity: Entity) {
        for (tid, storage) in self.storages.iter_mut() {
            if storage.remove(&entity).is_some() {
                *self.mutation_rev.entry(*tid).or_default() += 1;
            }
        }
    }

    /// All entities that have component T, in creation (id) order. The
    /// storage is a HashMap whose iteration order is randomized per process,
    /// so sorting keeps systems - and every sim and playtest - reproducible.
    pub fn query<T: Any + Send>(&self) -> Vec<Entity> {
        match self.storages.get(&TypeId::of::<T>()) {
            Some(storage) => {
                let mut entities: Vec<Entity> = storage.keys().copied().collect();
                entities.sort_unstable();
                entities
            }
            None => Vec::new(),
        }
    }

    /// Call `f` once per (entity, component) pair of type T, in arbitrary
    /// order. Unlike query() nothing is collected or sorted - for passes
    /// where order doesn't matter and the caller accumulates.
    pub fn iter<T: Any + Send>(&self, mut f: impl FnMut(Entity, &T)) {
        if let Some(storage) = self.storages.get(&TypeId::of::<T>()) {
            for (e, c) in storage.iter() {
                if let Some(c) = c.downcast_ref::<T>() { f(*e, c); }
            }
        }
    }

    /// The first entity whose component T satisfies `pred`, in arbitrary
    /// order. Unlike query() this allocates nothing and skips the sort -
    /// for hot lookups where any match will do.
    pub fn find<T: Any + Send>(&self, mut pred: impl FnMut(Entity, &T) -> bool) -> Option<Entity> {
        self.storages.get(&TypeId::of::<T>())?.iter()
            .find(|(e, c)| c.downcast_ref::<T>().map(|c| pred(**e, c)).unwrap_or(false))
            .map(|(e, _)| *e)
    }

    /// Live components of type T. O(1) - the storage's own length.
    pub fn component_count<T: Any + Send>(&self) -> usize {
        self.storages.get(&TypeId::of::<T>()).map(|s| s.len()).unwrap_or(0)
    }

    /// How many times type T's storage has structurally changed (add,
    /// remove, destroy) - an index that folded at an older revision
    /// can't claim to cover what's live now.
    pub fn mutation_rev<T: Any + Send>(&self) -> u64 {
        self.mutation_rev.get(&TypeId::of::<T>()).copied().unwrap_or(0)
    }

    /// Store world-level singleton data, keyed by type.
    pub fn insert_resource<T: Any + Send>(&mut self, res: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(res));
    }

    /// Read the resource of type T, if one's stored.
    pub fn resource<T: Any + Send>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref()
    }

    /// The resource of type T, created from Default if absent.
    pub fn resource_or_default<T: Any + Send + Default>(&mut self) -> &mut T {
        self.resources
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }

    /// Total number of live entities (approx: highest id issued)
    pub fn entity_count(&self) -> u32 {
        self.next_entity
    }
}

/// A system processes world state each tick
pub trait System: Send {
    /// Run the system once against the world
    fn run(&mut self, world: &mut World);
    /// What perf telemetry calls this system (its type path)
    fn name(&self) -> &'static str { std::any::type_name::<Self>() }
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
            let _s = crate::infra::perf::scope(system.name());
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

    #[test]
    fn test_query_returns_entities_in_creation_order() {
        let mut w = World::new();
        let ids: Vec<Entity> = (0..200).map(|i| {
            let e = w.create_entity();
            w.add_component(e, i as u32);
            e
        }).collect();
        // A HashMap's key order is effectively random over 200 entries
        assert_eq!(w.query::<u32>(), ids);
    }
}
