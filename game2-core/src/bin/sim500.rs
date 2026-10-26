//! F5: 500-turn simulation - villager + food + economy + climate + telemetry.
//! Produces real metrics output as evidence of a working sim.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger, Villager, Activity};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== 500-TURN SIMULATION ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);

    // Spawn a village: 5 villagers, 3 food sources
    let mut villagers = vec![];
    for i in 0..5 {
        villagers.push(villager::spawn_villager(&mut world, (i * 2) as f32, 0.0));
    }
    for i in 0..3 {
        villager::spawn_food_source(&mut world, (i * 5) as f32, 8.0, 100);
    }
    let climate = climate::spawn_climate(&mut world);

    println!("tick,avg_hunger,seeking,eating,idle,season,food_mult");
    for tick in 0..500 {
        let s = telem.span("tick");
        sched.tick(&mut world);
        telem.end_span(s);

        if tick % 50 == 0 {
            // Gather metrics
            let mut hunger_sum = 0.0;
            let (mut seeking, mut eating, mut idle) = (0, 0, 0);
            for v in &villagers {
                if let Some(h) = world.get_component::<Hunger>(*v) { hunger_sum += h.value; }
                if let Some(vil) = world.get_component::<Villager>(*v) {
                    match vil.activity {
                        Activity::SeekingFood => seeking += 1,
                        Activity::Eating => eating += 1,
                        Activity::Idle => idle += 1,
                    }
                }
            }
            let avg = hunger_sum / villagers.len() as f32;
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let mult = climate_multiplier(cl, "food");
            println!("{},{:.1},{},{},{},{:?},{:.2}", tick, avg, seeking, eating, idle, cl.season, mult);
            telem.set_tick(tick as u64);
            telem.gauge("avg_hunger", avg as f64);
            telem.gauge("food_mult", mult as f64);
        }
    }

    println!("\n=== METRICS EXPORT ===");
    for line in telem.export_metrics().iter().take(10) {
        println!("{}", line);
    }
    println!("\n=== 500-TURN SIM COMPLETE ===");
}
