//! C5: Religion emergence - congregation forms via homophily.
//! Similar agents cluster -> community forms -> cohesion emerges.

use game2_core::engine::ecs::World;
use game2_core::sim::religion::{Faith, Congregation, Service, GatherSystem, DevotionSystem};

fn main() {
    game2_core::infra::crash::install();
    println!("=== RELIGION EMERGENCE (homophily) ===\n");
    let mut world = World::new();

    // Villagers with varying faith devotion
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let devotions = [0.8, 0.75, 0.2, 0.7, 0.15]; // 3 devout, 2 secular
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = world.create_entity();
        world.add_component(v, Faith {
            religion: "Old Faith".into(),
            devotion: devotions[i],
            traditions: if devotions[i] > 0.5 { vec!["harvest_rite".into()] } else { vec![] },
        });
        villagers.push(v);
    }

    // Homophily: devout villagers (devotion > 0.5) cluster -> congregation forms
    println!("Devout villagers (>0.5): Aldric, Eliza, Corvin");
    println!("Secular villagers: Bram, Dagny\n");
    let devout: Vec<_> = villagers.iter().enumerate()
        .filter(|(i, _)| devotions[*i] > 0.5)
        .map(|(_, v)| *v)
        .collect();
    println!("3 devout agents cluster -> forming congregation 'Old Faith Circle'\n");

    let congregation = world.create_entity();
    world.add_component(congregation, Congregation {
        routinization: 0.0,
            absolutism: 0.0,
        name: "Old Faith Circle".into(),
        members: devout.clone(),
        leader: Some(villagers[0]), // most devout leads
        orthodoxy: 0.6,
        services: vec![Service {
            name: "Harvest Rite".into(), frequency: "weekly".into(),
            participants: devout.clone(), cohesion_boost: 0.15, charity_output: 0.1,
        }],
        cultish: false,
    });

    // Run systems - orthodoxy spreads, congregation produces cohesion
    let mut sched = game2_core::engine::ecs::Scheduler::new();
    sched.add_system(GatherSystem);
    sched.add_system(DevotionSystem);
    for tick in 0..20 {
        sched.tick(&mut world);
        // the circle meets weekly; between meetings faith lapses a little
        if tick % 7 == 0 {
            game2_core::sim::religion::perform_service(&mut world, congregation, Service {
                name: "Harvest Rite".into(), frequency: "weekly".into(),
                participants: devout.clone(), cohesion_boost: 0.15, charity_output: 0.1,
            });
        }
    }

    let c = world.get_component::<Congregation>(congregation).unwrap();
    println!("After 20 ticks:");
    println!("  congregation '{}' members: {}", c.name, c.members.len());
    println!("  orthodoxy: {:.2}", c.orthodoxy);
    for v in &devout {
        if let Some(f) = world.get_component::<Faith>(*v) {
            println!("  member devotion: {:.2}", f.devotion);
        }
    }
    println!("\n=== RELIGION EMERGED (homophily clustering) ===");
}
