//! First game mechanic: villager hunger cycle
//!
//! Villagers have Hunger that rises each tick. When hungry they seek
//! a FoodSource, move toward it, and eat when they arrive.
//! Demonstrates emergent behavior from simple rules on the ECS.

use crate::engine::ecs::{Entity, System, World};

/// Position on the 2.5D map
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

/// Villager hunger - 0 = starving, 100 = full
#[derive(Debug, Clone, Copy)]
pub struct Hunger {
    pub value: f32,
}

/// Marks an entity as a food source with limited servings
#[derive(Debug)]
pub struct FoodSource {
    pub servings: u32,
}

/// Current activity state of a villager
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Activity {
    Idle,
    SeekingFood,
    Eating,
}

/// Villager marker + state
#[derive(Debug)]
pub struct Villager {
    pub activity: Activity,
    pub target_food: Option<Entity>,
}

/// Hunger rises each tick
pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Hunger>() {
            if let Some(h) = world.get_component_mut::<Hunger>(e) {
                h.value = (h.value - 1.0).max(0.0);
            }
        }
    }
}

/// Hungry villagers seek and eat food
pub struct SeekFoodSystem;
impl System for SeekFoodSystem {
    fn run(&mut self, world: &mut World) {
        let hunger_threshold = 30.0;
        let eat_distance = 1.0;

        // Find villagers and their state
        let villagers: Vec<Entity> = world.query::<Villager>();

        for v in villagers {
            let hunger = match world.get_component::<Hunger>(v) {
                Some(h) => h.value,
                None => continue,
            };
            let pos = *world.get_component::<Pos>(v).unwrap_or(&Pos { x: 0.0, y: 0.0 });

            // Decide activity
            let mut activity = world.get_component::<Villager>(v).unwrap().activity;
            let mut target = world.get_component::<Villager>(v).unwrap().target_food;

            if hunger < hunger_threshold {
                // Find nearest food source
                if target.is_none() {
                    let mut best: Option<(Entity, f32)> = None;
                    for f in world.query::<FoodSource>() {
                        if let Some(fp) = world.get_component::<Pos>(f) {
                            let d = (fp.x - pos.x).abs() + (fp.y - pos.y).abs();
                            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                                best = Some((f, d));
                            }
                        }
                    }
                    target = best.map(|(f, _)| f);
                    activity = Activity::SeekingFood;
                }

                // Move toward or eat at target
                if let Some(f) = target {
                    if let Some(fp) = world.get_component::<Pos>(f) {
                        let dx = fp.x - pos.x;
                        let dy = fp.y - pos.y;
                        let dist = dx.abs() + dy.abs();

                        if dist <= eat_distance {
                            // Eat!
                            activity = Activity::Eating;
                            if let Some(fs) = world.get_component_mut::<FoodSource>(f) {
                                if fs.servings > 0 {
                                    fs.servings -= 1;
                                    if let Some(h) = world.get_component_mut::<Hunger>(v) {
                                        h.value = 100.0;
                                    }
                                    activity = Activity::Idle;
                                    target = None;
                                }
                            }
                        } else {
                            // Step toward food (speed 1 per tick)
                            let step = 1.0f32.min(dist);
                            let nx = pos.x + dx.signum() * step.min(dx.abs());
                            let ny = pos.y + dy.signum() * (step - step.min(dx.abs()));
                            if let Some(p) = world.get_component_mut::<Pos>(v) {
                                p.x = nx;
                                p.y = ny;
                            }
                            activity = Activity::SeekingFood;
                        }
                    } else {
                        target = None; // food source gone
                    }
                }
            } else {
                activity = Activity::Idle;
                target = None;
                // wander when idle: drift a little so they don't pile up
                let seed = (v as u64).wrapping_add((pos.x * 7.0 + pos.y * 13.0) as u64);
                let dx = ((seed % 7) as f32 - 3.0) * 0.06;
                let dy = (((seed / 7) % 7) as f32 - 3.0) * 0.06;
                if let Some(p) = world.get_component_mut::<Pos>(v) {
                    p.x = (p.x + dx).clamp(0.0, 31.0);
                    p.y = (p.y + dy).clamp(0.0, 31.0);
                }
            }

            if let Some(vill) = world.get_component_mut::<Villager>(v) {
                vill.activity = activity;
                vill.target_food = target;
            }
        }
    }
}

/// Spawn a villager at a position
pub fn spawn_villager(world: &mut World, x: f32, y: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, Hunger { value: 100.0 });
    world.add_component(e, Villager {
        activity: Activity::Idle,
        target_food: None,
    });
    e
}

/// Spawn a food source at a position
pub fn spawn_food_source(world: &mut World, x: f32, y: f32, servings: u32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, FoodSource { servings });
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::Scheduler;

    #[test]
    fn test_villager_eats_when_hungry() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);
        spawn_food_source(&mut world, 3.0, 0.0, 5);

        // Make villager hungry
        world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

        let mut sched = Scheduler::new();
        sched.add_system(SeekFoodSystem);

        // Run ticks until villager reaches food
        for _ in 0..10 {
            sched.tick(&mut world);
            if world.get_component::<Hunger>(v).unwrap().value > 50.0 {
                break;
            }
        }

        // Villager should have eaten
        let hunger = world.get_component::<Hunger>(v).unwrap().value;
        assert!(hunger > 50.0, "villager should have eaten, hunger={}", hunger);
    }

    #[test]
    fn test_hunger_rises() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);

        let mut sched = Scheduler::new();
        sched.add_system(HungerSystem);
        sched.tick(&mut world);

        assert_eq!(world.get_component::<Hunger>(v).unwrap().value, 99.0);
    }
}
