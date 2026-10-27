//! Jobs: villagers specialize and produce goods.
//! Farmer produces wheat, Miller makes flour, Baker makes bread, etc.
//! Work drives the economy visibly.

use crate::engine::ecs::{Entity, World};
use crate::sim::economics::Market;

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
    pub pending: f32,      // output made but not yet a whole unit for market
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

/// Raw material a job consumes from the market per unit made, if any
pub fn consumes(job: Job) -> Option<&'static str> {
    match job {
        Job::Miller => Some("wheat"),
        Job::Baker => Some("flour"),
        _ => None,
    }
}

/// Whether a job's output is a good sold at market (vs. a service)
pub fn sells_goods(job: Job) -> bool {
    matches!(job, Job::Farmer | Job::Miller | Job::Baker | Job::Miner | Job::Carpenter)
}

/// Work a tick - produces goods, skill improves
pub fn work_tick(holder: &mut JobHolder) -> (&'static str, f32) {
    holder.skill = (holder.skill + 0.005).min(1.0);
    (produces(holder.job), holder.output_rate * holder.skill)
}

/// System: villagers with jobs work each tick. Goods-producing jobs put
/// each whole unit they make onto the market, drawing their raw material
/// (wheat for millers, flour for bakers) from it; with no input in stock
/// they can't make more. Idle villagers neither produce nor gain skill.
pub struct JobSystem;
impl crate::engine::ecs::System for JobSystem {
    fn run(&mut self, world: &mut World) {
        let market = world.query::<Market>().first().copied();
        for e in world.query::<JobHolder>() {
            let (good, input, units) = match world.get_component_mut::<JobHolder>(e) {
                Some(j) if j.job != Job::Idle => {
                    let (good, amount) = work_tick(j);
                    if !sells_goods(j.job) { continue; }
                    j.pending += amount;
                    (good, consumes(j.job), j.pending.floor() as u32)
                }
                _ => continue,
            };
            let made = match market.and_then(|m| world.get_component_mut::<Market>(m)) {
                Some(mkt) => {
                    let made = match input {
                        Some(raw) => {
                            let stock = mkt.supply.entry(raw.to_string()).or_insert(0);
                            let n = units.min(*stock);
                            *stock -= n;
                            n
                        }
                        None => units,
                    };
                    if made > 0 { mkt.add_supply(good, made); }
                    made
                }
                None => 0,
            };
            if let Some(j) = world.get_component_mut::<JobHolder>(e) {
                // Unsold output waits, but a blocked worker doesn't stockpile
                // more than one unit's worth of half-finished work
                j.pending = (j.pending - made as f32).min(1.0);
            }
        }
    }
}

/// Assign a job to a villager
pub fn assign_job(world: &mut World, villager: Entity, job: Job) {
    world.add_component(villager, JobHolder { job, skill: 0.2, output_rate: 1.0, workplace: None, pending: 0.0 });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;
    use crate::sim::economics::spawn_market;

    fn supply(w: &World, m: Entity, good: &str) -> u32 {
        w.get_component::<Market>(m).unwrap().supply.get(good).copied().unwrap_or(0)
    }

    #[test]
    fn test_farmer_stocks_market_with_wheat() {
        let mut w = World::new();
        let m = spawn_market(&mut w);
        let f = w.create_entity();
        assign_job(&mut w, f, Job::Farmer);
        for _ in 0..20 { JobSystem.run(&mut w); }
        let wheat = supply(&w, m, "wheat");
        assert!(wheat >= 4, "a farmer should sell wheat, got {}", wheat);
        assert!(w.get_component::<JobHolder>(f).unwrap().skill > 0.2);
    }

    #[test]
    fn test_production_chain_needs_inputs() {
        let mut w = World::new();
        let m = spawn_market(&mut w);
        let miller = w.create_entity();
        assign_job(&mut w, miller, Job::Miller);
        for _ in 0..20 { JobSystem.run(&mut w); }
        assert_eq!(supply(&w, m, "flour"), 0, "no wheat, no flour");
        // Bring in a farmer: wheat flows to the miller, flour to the market
        let farmer = w.create_entity();
        assign_job(&mut w, farmer, Job::Farmer);
        let baker = w.create_entity();
        assign_job(&mut w, baker, Job::Baker);
        for _ in 0..60 { JobSystem.run(&mut w); }
        let bread = supply(&w, m, "bread");
        assert!(bread > 0, "wheat -> flour -> bread should reach the market");
    }

    #[test]
    fn test_idle_and_services_dont_stock_market() {
        let mut w = World::new();
        let m = spawn_market(&mut w);
        let idle = w.create_entity();
        assign_job(&mut w, idle, Job::Idle);
        let priest = w.create_entity();
        assign_job(&mut w, priest, Job::Priest);
        for _ in 0..20 { JobSystem.run(&mut w); }
        assert_eq!(w.get_component::<JobHolder>(idle).unwrap().skill, 0.2);
        assert_eq!(supply(&w, m, "faith") + supply(&w, m, "none"), 0);
    }
}
