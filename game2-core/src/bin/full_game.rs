//! Unified world state: all sim domains on shared entities.
//! Villagers have hunger+health+attachment+faith+pet+trader+relationships.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::illness::{Health, Disease, Stage};
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::religion::{Faith, Congregation, Service};
use game2_core::sim::economics::{Market, Trader};
use game2_core::sim::community::{Citizen, Organization, InfluenceSystem, recruit, exert_influence};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== UNIFIED WORLD STATE ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(InfluenceSystem); // pools member influence into the org each tick

    let climate = climate::spawn_climate(&mut world);

    // Villagers with full component stacks
    let names = ["Aldric", "Eliza", "Bram", "Corvin"];
    let mut villagers = vec![];
    for (i, name) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i * 4) as f32, 0.0);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (3-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3 + (i as f32 * 0.1), traditions: vec!["harvest_rite".into()] });
        world.add_component(v, Citizen { influence: 0.2 + (i as f32 * 0.1), authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        world.add_component(v, Trader { gold: 50.0, inventory: Default::default() });
        villagers.push(v);
    }

    // Pets
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    // Organization the player is building
    let org = world.create_entity();
    world.add_component(org, Organization {
        name: "Harvesters Guild".into(), members: vec![], leader: villagers[0],
        influence_pool: 0.0, resources: 100.0, charter: "Mutual aid".into(),
    });

    // Recruit villagers into org (needs trust > 0.5)
    for (i, v) in villagers.iter().enumerate() {
        if recruit(&mut world, org, *v, 0.4 + (i as f32 * 0.2)) {
            println!("recruited {} into Harvesters Guild", names[i]);
        } else {
            println!("{} not recruited (trust too low)", names[i]);
        }
    }

    // Market
    let market = world.create_entity();
    world.add_component(market, Market {
        prices: [("food".to_string(), 10.0)].into_iter().collect(),
        supply: [("food".to_string(), 80)].into_iter().collect(),
    });

    println!("\nRunning 80 ticks...\n");
    println!("tick,hunger,health,org_influence,dog_bond,season\n");
    for tick in 0..80 {
        sched.tick(&mut world);

        if tick % 20 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 4.0;
            let health: f32 = villagers.iter().map(|v| world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0)).sum::<f32>() / 4.0;
            let infl = exert_influence(&mut world, org, "village");
            let bond = world.get_component::<Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            println!("{},{:.1},{:.2},{:.2},{:.2},{:?}", tick, hunger, health, infl, bond, cl.season);
        }
    }

    println!("\n=== Villager States ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            let style = infer_style(h);
            println!("{}: attachment={:?} faith_devotion={:.1}",
                names[i], style,
                world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0));
        }
    }
    println!("\n=== COMPLETE ===");
}
