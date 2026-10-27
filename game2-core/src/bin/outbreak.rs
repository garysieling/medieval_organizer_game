//! C6: Illness outbreak - disease spreads through village.
//! Emergent: contagion spreads villager to villager, treatments affect outcome.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Pos};
use game2_core::sim::illness::{Health, Disease, Stage, treat_health};
use game2_core::game::herbology::{self, Apothecary, HerbEffect, PrepMethod};

fn main() {
    println!("=== ILLNESS OUTBREAK ===\n");
    let mut world = World::new();
    let mut sched = Scheduler::new();

    // 6 villagers in close quarters
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny","Wystan"];
    let mut villagers = vec![];
    for (i, n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i % 3) as f32, (i / 3) as f32);
        world.add_component(v, Health { condition: 1.0, immunity: 0.4 + (i as f32 * 0.1) });
        villagers.push(v);
    }

    // The village apothecary knows willow bark and yarrow
    let apothecary = world.create_entity();
    world.add_component(apothecary, Apothecary {
        known_herbs: vec!["willow_bark".into(), "yarrow".into()], remedies: vec![], skill: 0.7,
    });
    let willow = herbology::spawn_herb(&mut world, "willow_bark", 0.6, HerbEffect::PainRelief, PrepMethod::Tea);
    let yarrow = herbology::spawn_herb(&mut world, "yarrow", 0.5, HerbEffect::Healing, PrepMethod::Poultice);

    // Patient zero gets flu at tick 0
    let flu = world.create_entity();
    world.add_component(flu, Disease { name: "flu".into(), patient: villagers[0], stage: Stage::Mild, severity: 0.3, contagious: true });
    world.get_component_mut::<Health>(villagers[0]).unwrap().condition -= 0.2;

    println!("Patient zero ({}) infected. Watching spread...\n", names[0]);
    println!("tick,infected,avg_health,severely_ill\n");

    let mut infected = vec![villagers[0]];
    for tick in 0..40 {
        // Disease spreads to nearby villagers each few ticks
        if tick % 5 == 0 && infected.len() < villagers.len() {
            // Spread to next villager (proximity-based in real impl)
            let next = villagers[infected.len()];
            world.get_component_mut::<Health>(next).unwrap().condition -= 0.25;
            infected.push(next);
            println!("  tick {}: {} now sick (spread)", tick, names[infected.len()-1]);
        }

        // Disease severity progresses
        for v in &infected {
            if let Some(h) = world.get_component_mut::<Health>(*v) {
                h.condition = (h.condition - 0.01).max(0.0);
            }
        }

        // Herbalist treats sick villagers at tick 20+
        if tick == 20 {
            println!("\n  Apothecary starts treating the sick (willow bark + yarrow)\n");
            for (i, v) in infected.iter().enumerate() {
                // Alternate remedies per patient; practice sharpens the apothecary regardless
                let (herb, method) = if i % 2 == 0 {
                    (willow, PrepMethod::Tea)
                } else {
                    (yarrow, PrepMethod::Poultice)
                };
                let Some(remedy) = herbology::prepare_remedy(&mut world, apothecary, herb, method) else { continue };
                if let Some(h) = world.get_component_mut::<Health>(*v) {
                    treat_health(h, &remedy);
                }
            }
        }

        if tick % 10 == 0 {
            let avg: f32 = villagers.iter().map(|v| world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0)).sum::<f32>() / 6.0;
            let severe = villagers.iter().filter(|v| world.get_component::<Health>(**v).map(|h| h.condition < 0.5).unwrap_or(false)).count();
            println!("{},{},{:.2},{}", tick, infected.len(), avg, severe);
        }
    }

    println!("\n=== Final states ===");
    for (i, v) in villagers.iter().enumerate() {
        let h = world.get_component::<Health>(*v).unwrap().condition;
        let status = if h > 0.7 { "healthy" } else if h > 0.4 { "recovering" } else { "severely ill" };
        println!("{}: health={:.2} ({})", names[i], h, status);
    }
    println!("\n=== OUTBREAK COMPLETE ===");
}
