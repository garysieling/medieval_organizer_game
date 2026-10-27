//! A1-A7: Unified world + real game loop.
//! One WorldState with all domains on shared entities. tick() runs all systems.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::illness::{Health, Disease, Stage, IllnessSystem};
use game2_core::sim::psychology::{self, AttachmentHistory, Episode, EpisodeKind, infer_style};
use game2_core::sim::religion::{Faith, Congregation, Service};
use game2_core::sim::economics::Market;
use game2_core::sim::community::{Citizen, Organization};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== UNIFIED GAME LOOP ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    // All systems in order
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(IllnessSystem);

    // World setup
    let terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 300);
    let market = world.create_entity();
    world.add_component(market, Market { prices: [("food".into(), 10.0)].into_iter().collect(), supply: [("food".into(), 100)].into_iter().collect(), ..Default::default() });

    // Villagers with full stacks
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny","Wystan"];
    let mut villagers = vec![];
    for (i, name) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%3) as f32);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (5-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3, traditions: vec![] });
        world.add_component(v, Citizen { influence: 0.1, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        villagers.push(v);
    }

    let start_styles: Vec<_> = villagers.iter()
        .map(|v| infer_style(world.get_component::<AttachmentHistory>(*v).unwrap()))
        .collect();

    println!("tick,avg_hunger,avg_health,seeking,idle,season,food_mult\n");
    for tick in 0..500 {
        sched.tick(&mut world);

        // Emergent event: famine at tick 200 (food source depletes -> hunger spikes)
        if tick == 200 {
            println!("EVENT: famine begins (food scarcity)");
        }
        // Emergent event: flu outbreak at tick 300
        if tick == 300 {
            for v in &villagers {
                let d = world.create_entity();
                world.add_component(d, Disease { name: "flu".into(), patient: *v, stage: Stage::Mild, severity: 0.3, contagious: true });
                if let Some(h) = world.get_component_mut::<Health>(*v) { h.condition -= 0.15; }
            }
            println!("EVENT: flu outbreak at tick 300");
        }

        // Attachment episodes: going hungry alone is distressing; during the
        // flu Aldric nurses the sick every few days, a warm moment for both
        if tick % 10 == 0 {
            let carer = villagers[0];
            for (i, v) in villagers.iter().enumerate() {
                let hungry = world.get_component::<Hunger>(*v).map(|h| h.value < 30.0).unwrap_or(false);
                let sick = world.get_component::<Health>(*v).map(|h| h.condition < 0.8).unwrap_or(false);
                let other = villagers[(i + 1) % villagers.len()];
                if hungry {
                    psychology::record_episode(&mut world, *v, Episode { with: other, kind: EpisodeKind::Anxiety });
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
            println!("{},{:.1},{:.2},{},{},{:?},{:.2}", tick, hunger, health, seek, idle, cl.season, climate_multiplier(cl, "food"));
            telem.set_tick(tick as u64);
            telem.gauge("hunger", hunger as f64);
            telem.gauge("health", health as f64);
        }
    }

    println!("\n=== Final villager states ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("{}: attachment={:?} -> {:?} (anx={} warm={}) health={:.2}",
                names[i], start_styles[i], infer_style(h), h.anxiety_episodes, h.warmth_episodes,
                world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0));
        }
    }
    println!("\n=== GAME LOOP COMPLETE (500 ticks) ===");
}
