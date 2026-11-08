//! Unified world state: all sim domains on shared entities.
//! Villagers have hunger+health+attachment+faith+pet+trader+relationships.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::illness::Health;
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::religion::{self, Faith, Congregation, Service, DevotionSystem};
use game2_core::sim::economics::{Market, Trader};
use game2_core::sim::community::{Citizen, Organization, InfluenceSystem, recruit, influence};
use game2_core::sim::organized_power::{OrganizedPower, PowerAtrophySystem};
use game2_core::game::organizing::house_meeting;
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== UNIFIED WORLD STATE ===\n");

    let mut world = World::new();
    let _telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(InfluenceSystem); // pools member influence into the org each tick
    sched.add_system(PowerAtrophySystem); // organized power fades when the guild sits idle
    sched.add_system(DevotionSystem); // faith lapses a little each day without practice

    let climate = climate::spawn_climate(&mut world);
    // shared food store in the middle of the villager row, so hunger cycles
    // instead of the village quietly starving
    villager::spawn_food_source(&mut world, 6.0, 4.0, 200);

    // Villagers with full component stacks
    let names = ["Aldric", "Eliza", "Bram", "Corvin"];
    let mut villagers = vec![];
    for (i, _name) in names.iter().enumerate() {
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
    world.add_component(org, OrganizedPower::new());

    // Recruit villagers into org (needs trust > 0.5)
    for (i, v) in villagers.iter().enumerate() {
        if recruit(&mut world, org, *v, 0.4 + (i as f32 * 0.2)) {
            println!("recruited {} into Harvesters Guild", names[i]);
        } else {
            println!("{} not recruited (trust too low)", names[i]);
        }
    }

    // The whole village keeps the Old Faith's harvest rite together
    let church = world.create_entity();
    world.add_component(church, Congregation {
        name: "Old Faith".into(), members: villagers.clone(), leader: None, orthodoxy: 0.5, services: vec![], cultish: false 
    });

    // Market
    let market = world.create_entity();
    world.add_component(market, Market {
        prices: [("food".to_string(), 10.0)].into_iter().collect(),
        supply: [("food".to_string(), 80)].into_iter().collect(),
        ..Default::default()
    });

    println!("\nRunning 200 ticks: the guild organizes until tick 100, then goes quiet...\n");
    println!("tick,hunger,health,org_influence,org_power,dog_bond,season,devotion\n");
    for tick in 0..200 {
        sched.tick(&mut world);

        // The guild organizes: the leader holds a house meeting with the
        // members every few days, building organized power; after tick 100
        // it stops, and the power it built starts to atrophy
        if tick < 100 && tick % 5 == 0 {
            let attendees = world.get_component::<Organization>(org).unwrap().members.clone();
            house_meeting(&mut world, org, &attendees);
        }
        if tick == 100 {
            println!("EVENT: the guild stops meeting");
        }

        // Weekly harvest rite: whoever isn't going hungry attends, deepening
        // their faith (which otherwise lapses) and their bonds with each other
        if tick % 7 == 0 {
            let participants: Vec<_> = villagers.iter().copied()
                .filter(|v| world.get_component::<Hunger>(*v).map(|h| h.value >= 30.0).unwrap_or(true))
                .collect();
            religion::perform_service(&mut world, church, Service {
                name: "harvest_rite".into(), frequency: "weekly".into(),
                participants, cohesion_boost: 0.1, charity_output: 0.0,
            });
        }

        if tick % 20 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 4.0;
            let health: f32 = villagers.iter().map(|v| world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0)).sum::<f32>() / 4.0;
            let infl = influence(&world, org);
            let power = world.get_component::<OrganizedPower>(org).map(|p| p.capacity).unwrap_or(0.0);
            let bond = world.get_component::<Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let devotion: f32 = villagers.iter().map(|v| world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0)).sum::<f32>() / 4.0;
            println!("{},{:.1},{:.2},{:.2},{:.2},{:.2},{:?},{:.2}", tick, hunger, health, infl, power, bond, cl.season, devotion);
        }
    }

    println!("\n=== Villager States ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            let style = infer_style(h);
            println!("{}: attachment={:?} faith_devotion={:.2}",
                names[i], style,
                world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0));
        }
    }
    println!("\n=== COMPLETE ===");
}
