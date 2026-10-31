//! Full session: the whole game in one run.
//! Villagers live, seasons pass, events fire, player acts, org forms, illness hits.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::tech_tree::{printmaking_tree, research};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::weather::{Weather, WeatherSystem, spawn_weather};
use game2_core::sim::illness::Health;
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::religion::Faith;
use game2_core::sim::community::{Citizen, Organization};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== FULL SESSION: the game in one run ===\n");

    let mut world = World::new();
    let _telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(WeatherSystem);
    sched.add_system(GuildFormationSystem);

    // World
    let _terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 400);

    // Villagers
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%3) as f32);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3 + (i as f32 * 0.1), traditions: vec![] });
        world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        villagers.push(v);
    }

    // Player: outsider
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter {
        name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 150.0, standing: 0.0,
    });
    let org = world.create_entity();
    world.add_component(org, Organization {
        name: "Harvesters Guild".into(), members: vec![], leader: player,
        influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into(),
    });

    // Merchants (for guild formation)
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant { name: format!("Merchant{}", i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
    }

    // Pet
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    println!("tick,season,hunger,org_members,guilds,dog_bond,player_infl\n");
    let mut techs = printmaking_tree();
    for tick in 0..300 {
        sched.tick(&mut world);

        // Weather events
        if tick == 60 { spawn_weather(&mut world, Weather::Storm, 0.7, 20); println!("EVENT: storm"); }
        if tick == 120 { spawn_weather(&mut world, Weather::Drought, 0.8, 30); println!("EVENT: drought"); }
        if tick == 200 { spawn_weather(&mut world, Weather::Frost, 0.6, 15); println!("EVENT: frost"); }

        // the weather bites: frost and heat take it out of them
        let bite: f32 = world.query::<game2_core::sim::weather::CurrentWeather>().iter()
            .filter_map(|&e| world.get_component::<game2_core::sim::weather::CurrentWeather>(e))
            .map(game2_core::sim::weather::health_effect).sum();
        if bite != 0.0 {
            for v in world.query::<game2_core::sim::illness::Health>() {
                if let Some(h) = world.get_component_mut::<game2_core::sim::illness::Health>(v) {
                    // the weather wears them down but never finishes the job -
                    // illness and hunger are for that
                    h.condition = (h.condition + bite).clamp(0.1, 1.0);
                }
            }
        }

        // Player actions each 10 ticks
        if tick % 10 == 0 && tick > 0 {
            let target = villagers[(tick / 10) as usize % 5];
            match tick % 30 {
                0 => { let _r = act_talk(&mut world, player, target); }
                10 => { let r = act_recruit(&mut world, player, org, target); if r.success { println!("  {} recruited to guild", names[(tick/10) as usize % 5]); } }
                20 => { let _r = act_invest(&mut world, player, 10.0); }
                _ => {}
            }
        }

        // Tech progress
        if tick == 100 { research(&mut techs, "Quill & Ink", 50.0); }
        if tick == 180 { research(&mut techs, "Papyrus", 50.0); }

        if tick % 30 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 5.0;
            let members = world.get_component::<Organization>(org).unwrap().members.len();
            let guilds = world.query::<Guild>().len();
            let bond = world.get_component::<Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let infl = world.get_component::<PlayerCharacter>(player).unwrap().influence;
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            println!("{},{:?},{:.1},{},{},{:.2},{:.2}", tick, cl.season, hunger, members, guilds, bond, infl);
        }
    }

    println!("\n=== Session Summary ===");
    println!("org members: {}", world.get_component::<Organization>(org).unwrap().members.len());
    println!("guilds formed: {}", world.query::<Guild>().len());
    println!("player influence: {:.2}", world.get_component::<PlayerCharacter>(player).unwrap().influence);
    println!("dog bond: {:.2}", world.get_component::<Pet>(dog).unwrap().bond_strength);
    println!("techs discovered: {}", techs.iter().filter(|t| t.discovered).count());
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("{}: attachment={:?}", names[i], infer_style(h));
        }
    }
    println!("\n=== FULL SESSION COMPLETE ===");
}
