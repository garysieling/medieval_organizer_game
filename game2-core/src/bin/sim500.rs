//! F5: 500-turn simulation - villager + food + economy + climate + telemetry.
//! Produces real metrics output as evidence of a working sim.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger, Villager, Activity};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::infra::telemetry::Telemetry;

/// Servings each field holds when full
const FIELD_CAPACITY: u32 = 2;
/// Servings each field ripens per tick in a normal season
const FIELD_GROWTH: f32 = 0.028;

fn main() {
    println!("=== 500-TURN SIMULATION ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(villager::CropGrowthSystem);

    // Spawn a village: 5 villagers, 3 crop fields that regrow with the
    // seasons - enough in spring and summer, lean in winter
    let mut villagers = vec![];
    for i in 0..5 {
        villagers.push(villager::spawn_villager(&mut world, (i * 2) as f32, 0.0));
    }
    for i in 0..3 {
        villager::spawn_crop_field(&mut world, (i * 5) as f32, 8.0, FIELD_CAPACITY, FIELD_GROWTH);
    }
    let climate = climate::spawn_climate(&mut world);

    // eating is instant (a villager refills and goes straight back to Idle),
    // so count meals as servings consumed since the last sample instead of
    // looking for villagers caught mid-meal
    let mut last_servings = villager::servings_left(&world);
    let mut last_grown = villager::servings_grown(&world);
    // villager-ticks spent going hungry since the last sample - shortages
    // between samples would otherwise be invisible
    let mut hungry_ticks = 0;
    println!("tick,avg_hunger,seeking,meals,idle,season,food_mult,food_stock,hungry_ticks");
    for tick in 0..500 {
        let s = telem.span("tick");
        sched.tick(&mut world);
        telem.end_span(s);
        hungry_ticks += villagers.iter()
            .filter(|v| world.get_component::<Hunger>(**v).map(|h| h.value < 30.0).unwrap_or(false))
            .count();

        if tick % 50 == 0 {
            // Gather metrics
            let mut hunger_sum = 0.0;
            let (mut seeking, mut idle) = (0, 0);
            for v in &villagers {
                if let Some(h) = world.get_component::<Hunger>(*v) { hunger_sum += h.value; }
                if let Some(vil) = world.get_component::<Villager>(*v) {
                    match vil.activity {
                        Activity::SeekingFood => seeking += 1,
                        _ => idle += 1,
                    }
                }
            }
            let (servings, grown) = (villager::servings_left(&world), villager::servings_grown(&world));
            let meals = (last_servings + grown - last_grown).saturating_sub(servings);
            (last_servings, last_grown) = (servings, grown);
            let avg = hunger_sum / villagers.len() as f32;
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let mult = climate_multiplier(cl, "food");
            println!("{},{:.1},{},{},{},{:?},{:.2},{},{}", tick, avg, seeking, meals, idle, cl.season, mult, servings, hungry_ticks);
            hungry_ticks = 0;
            telem.set_tick(tick as u64);
            telem.gauge("avg_hunger", avg as f64);
            telem.gauge("food_mult", mult as f64);
            telem.gauge("meals", meals as f64);
        }
    }

    println!("\n=== METRICS EXPORT ===");
    for line in telem.export_metrics() {
        println!("{}", line);
    }
    println!("\n=== 500-TURN SIM COMPLETE ===");
}
