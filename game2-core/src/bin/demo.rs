//! Evidence demo - exercises each system and prints real output.
//! Run with: cargo run --bin demo
//! Output gets captured to evidence/ as proof the software works.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Activity, Hunger};
use game2_core::sim::economics::Market;
use game2_core::sim::psychology::{infer_style, AttachmentHistory, AttachmentStyle};

fn main() {
    println!("=== EVIDENCE DEMO ===\n");

    println!("--- C2: Terrain Generation (seed=7) ---");
    let t = Terrain::generate(33, 0.6, 7);
    println!("{}", t.ascii());

    println!("--- villager: Hunger Cycle ---");
    let mut world = World::new();
    let v = villager::spawn_villager(&mut world, 0.0, 0.0);
    villager::spawn_food_source(&mut world, 4.0, 0.0, 5);
    world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);

    for tick in 0..=6 {
        let h = world.get_component::<Hunger>(v).unwrap().value;
        let a = world.get_component::<villager::Villager>(v).unwrap().activity;
        println!("tick {}: hunger={:.1} activity={:?}", tick, h, a);
        sched.tick(&mut world);
    }

    println!("\n--- B1: Economics (price formation) ---");
    let mut m = Market { prices: Default::default(), supply: Default::default() };
    m.prices.insert("food".to_string(), 10.0);
    m.add_supply("food", 10);
    for demand in [10, 20, 40, 80] {
        m.update_price("food", demand);
        println!("demand={:3} supply={:3} price={:.2}", demand, m.supply["food"], m.prices["food"]);
    }

    println!("\n--- B2: Psychology (attachment emergence) ---");
    for (anx, warm) in [(1u32, 9u32), (8, 2), (2, 3), (5, 1)] {
        let h = AttachmentHistory { anxiety_episodes: anx, warmth_episodes: warm, episodes: vec![] };
        println!("anx={} warm={} -> {:?}", anx, warm, infer_style(&h));
    }

    println!("\n--- B6: Climate (seasons + drift) ---");
    let mut cw = World::new();
    let c = game2_core::sim::climate::spawn_climate(&mut cw);
    let mut csched = Scheduler::new();
    csched.add_system(game2_core::sim::climate::SeasonSystem);
    for _ in 0..5 {
        let cl = cw.get_component::<game2_core::sim::climate::Climate>(c).unwrap();
        println!("year={} season={:?} severity={:.2} food_mult={:.2}",
            cl.year, cl.season, cl.severity,
            game2_core::sim::climate::climate_multiplier(cl, "food"));
        csched.tick(&mut cw);
    }

    println!("\n=== DEMO COMPLETE ===");
}
