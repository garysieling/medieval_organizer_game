//! Jobs: villagers specialize and produce goods.
//! Farmer produces wheat, Miller makes flour, Baker makes bread, etc.
//! Work drives the economy visibly.

use crate::engine::ecs::{Entity, World};
use super::villager::Villager;

/// A job a villager can hold
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Job {
    Farmer,     // produces wheat
    Miller,     // wheat -> flour
    Baker,      // flour -> bread
    Miner,      // produces ore
    Carpenter,  // produces lumber/furniture
    Soldier,    // village defense
    Priest,     // faith services
    Merchant,   // trade
    Laborer,    // general work
    Idle,       // no job
}

/// A villager's job + work progress
#[derive(Debug)]
pub struct JobHolder {
    pub job: Job,
    pub skill: f32,        // 0-1, improves with work
    pub output_rate: f32,  // goods per tick
    pub workplace: Option<Entity>,
}

/// Goods a job produces
pub fn produces(job: Job) -> &'static str {
    match job {
        Job::Farmer => "wheat",
        Job::Miller => "flour",
        Job::Baker => "bread",
        Job::Miner => "ore",
        Job::Carpenter => "lumber",
        Job::Soldier => "security",
        Job::Priest => "faith",
        Job::Merchant => "trade",
        Job::Laborer => "labor",
        Job::Idle => "none",
    }
}

/// Work a tick - produces goods, skill improves
pub fn work_tick(holder: &mut JobHolder) -> (&'static str, f32) {
    holder.skill = (holder.skill + 0.005).min(1.0);
    (produces(holder.job), holder.output_rate * holder.skill)
}

/// System: villagers with jobs produce each tick
pub struct JobSystem;
impl crate::engine::ecs::System for JobSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<JobHolder>() {
            if let Some(j) = world.get_component_mut::<JobHolder>(e) {
                j.skill = (j.skill + 0.005).min(1.0);
            }
        }
    }
}

/// Assign a job to a villager
pub fn assign_job(world: &mut World, villager: Entity, job: Job) {
    world.add_component(villager, JobHolder { job, skill: 0.2, output_rate: 1.0, workplace: None });
}
