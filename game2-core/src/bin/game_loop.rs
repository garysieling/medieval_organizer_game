//! A1-A7: Unified world + real game loop.
//! One WorldState with all domains on shared entities. tick() runs all systems.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::jobs::{self, Job, JobHolder, JobSystem};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::illness::{Health, Disease, DiseaseKind, IllnessSystem};
use game2_core::sim::psychology::{self, AttachmentHistory, Episode, EpisodeKind, infer_style};
use game2_core::sim::religion::{self, Faith, Congregation, Service, DevotionSystem};
use game2_core::sim::economics::Market;
use game2_core::sim::community::Citizen;
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== UNIFIED GAME LOOP ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    // All systems in order
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(villager::NourishmentSystem); // starving hurts; eating well mends
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(IllnessSystem);
    sched.add_system(DevotionSystem); // faith fades a little each day without practice
    sched.add_system(JobSystem); // workers stock the market: wheat -> flour -> bread
    sched.add_system(villager::StallSystem); // the market's bread goes on sale as meals

    // World setup
    let _terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 300);
    let market = world.create_entity();
    world.add_component(market, Market { prices: [("food".into(), 10.0)].into_iter().collect(), supply: [("food".into(), 100)].into_iter().collect(), ..Default::default() });
    let bakery = villager::spawn_stall(&mut world, 12.0, 4.0, market, "bread");

    // Villagers with full stacks
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny","Wystan"];
    let roles = [Job::Farmer, Job::Farmer, Job::Miller, Job::Baker, Job::Priest, Job::Carpenter];
    let mut villagers = vec![];
    for (i, _name) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%3) as f32);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (5-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3, traditions: vec![] });
        world.add_component(v, Citizen { influence: 0.1, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        jobs::assign_job(&mut world, v, roles[i]);
        villagers.push(v);
    }

    // The whole village belongs to the Old Faith's congregation
    let church = world.create_entity();
    world.add_component(church, Congregation {
        routinization: 0.0,
            absolutism: 0.0,
        name: "Old Faith".into(), members: villagers.clone(), leader: None, orthodoxy: 0.5, services: vec![], cultish: false 
    });

    // Dagny takes in a dog; if they look after it, it'll see them through hard times
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[4]);

    let start_styles: Vec<_> = villagers.iter()
        .map(|v| infer_style(world.get_component::<AttachmentHistory>(*v).unwrap()))
        .collect();

    println!("tick,avg_hunger,avg_health,seeking,idle,season,food_mult,avg_devotion,at_service,dog_bond,wheat,flour,bread,lumber
");
    let mut last_attendance = 0;
    let mut famine_stores = vec![]; // food set aside during the famine
    for tick in 0..500 {
        sched.tick(&mut world);

        // Emergent event: famine at tick 200 (food source depletes -> hunger spikes).
        // It's a crop failure too: blight spoils the grain in the market and
        // the farms yield nothing, so the bakery can't bake through it
        if tick == 200 {
            println!("EVENT: famine begins (food scarcity)");
            for v in &villagers {
                if let Some(j) = world.get_component_mut::<JobHolder>(*v) {
                    if j.job == Job::Farmer { j.output_rate = 0.0; }
                }
            }
            if let Some(m) = world.get_component_mut::<Market>(market) {
                for g in ["wheat", "flour"] { m.supply.remove(g); }
            }
            for f in world.query::<villager::FoodSource>() {
                if let Some(fs) = world.get_component_mut::<villager::FoodSource>(f) {
                    famine_stores.push((f, fs.servings));
                    fs.servings = 0;
                }
            }
        }
        // ...until the harvest comes in and the stores are refilled
        if tick == 250 {
            for (f, servings) in famine_stores.drain(..) {
                if let Some(fs) = world.get_component_mut::<villager::FoodSource>(f) {
                    fs.servings += servings;
                }
            }
            for v in &villagers {
                if let Some(j) = world.get_component_mut::<JobHolder>(*v) {
                    if j.job == Job::Farmer { j.output_rate = 1.0; }
                }
            }
            println!("EVENT: harvest ends the famine");
        }
        // Emergent event: flu outbreak at tick 300
        if tick == 300 {
            for v in &villagers {
                let d = world.create_entity();
                world.add_component(d, Disease::new(DiseaseKind::Influenza, *v, 0.3));
                if let Some(h) = world.get_component_mut::<Health>(*v) { h.condition -= 0.15; }
            }
            println!("EVENT: flu outbreak at tick 300");
        }

        // Weekly service: whoever isn't starving or sick in bed attends,
        // deepening their faith and their bonds with each other
        if tick % 7 == 0 {
            let participants: Vec<_> = villagers.iter().copied().filter(|v| {
                let fed = world.get_component::<Hunger>(*v).map(|h| h.value >= 30.0).unwrap_or(true);
                let well = world.get_component::<Health>(*v).map(|h| h.condition >= 0.8).unwrap_or(true);
                fed && well
            }).collect();
            last_attendance = religion::perform_service(&mut world, church, Service {
                name: "Harvest Rite".into(), frequency: "weekly".into(),
                participants, cohesion_boost: 0.1, charity_output: 0.1, // the fed share with the hungry
            });
        }

        // Attachment episodes: going hungry alone is distressing, though a
        // well-bonded pet is a comfort through it; during la grippe Aldric
        // nurses the sick every few days, a warm moment for both
        if tick % 10 == 0 {
            let carer = villagers[0];
            for (i, v) in villagers.iter().enumerate() {
                let hungry = world.get_component::<Hunger>(*v).map(|h| h.value < 30.0).unwrap_or(false);
                let sick = world.get_component::<Health>(*v).map(|h| h.condition < 0.8).unwrap_or(false);
                let other = villagers[(i + 1) % villagers.len()];
                if hungry {
                    psychology::record_episode(&mut world, *v, Episode { with: other, kind: EpisodeKind::Anxiety });
                    if let Some(pet) = pets::comforting_pet(&world, *v) {
                        psychology::record_episode(&mut world, *v, Episode { with: pet, kind: EpisodeKind::Warmth });
                    }
                }
                if sick && (300..400).contains(&tick) && tick % 25 == 0 && *v != carer {
                    psychology::record_episode(&mut world, *v, Episode { with: carer, kind: EpisodeKind::Warmth });
                    psychology::record_episode(&mut world, carer, Episode { with: *v, kind: EpisodeKind::Warmth });
                }
            }
        }

        if tick % 50 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 6.0;
            let health: f32 = villagers.iter().map(|v| world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0)).sum::<f32>() / 6.0;
            let (mut seek, mut idle) = (0, 0);
            for v in &villagers {
                match world.get_component::<Villager>(*v).unwrap().activity {
                    Activity::SeekingFood => seek += 1,
                    _ => idle += 1,
                }
            }
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let devotion: f32 = villagers.iter().map(|v| world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0)).sum::<f32>() / 6.0;
            let bond = world.get_component::<Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let mkt = world.get_component::<Market>(market).unwrap();
            let stock = |g: &str| mkt.supply.get(g).copied().unwrap_or(0);
            let bread = world.get_component::<villager::FoodSource>(bakery).map(|f| f.servings).unwrap_or(0);
            println!("{},{:.1},{:.2},{},{},{:?},{:.2},{:.2},{},{:.2},{},{},{},{}", tick, hunger, health, seek, idle, cl.season, climate_multiplier(cl, "food"), devotion, last_attendance, bond,
                stock("wheat"), stock("flour"), bread, stock("lumber"));
            telem.set_tick(tick as u64);
            telem.gauge("hunger", hunger as f64);
            telem.gauge("health", health as f64);
            telem.gauge("bread", bread as f64);
        }
    }

    println!("\n=== Final villager states ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("{}: attachment={:?} -> {:?} (anx={} warm={}) health={:.2} devotion={:.2}",
                names[i], start_styles[i], infer_style(h), h.anxiety_episodes, h.warmth_episodes,
                world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0),
                world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0));
        }
    }
    println!("\n=== GAME LOOP COMPLETE (500 ticks) ===");
}
