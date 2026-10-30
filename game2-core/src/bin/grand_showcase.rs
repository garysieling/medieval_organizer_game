//! Grand showcase: maximum systems in one run.
//! Villagers + pets + market + climate + religion + guild + organizing + player + tech + lifecycle.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::game::tech_tree::{extended_printmaking_tree, research};
use game2_core::game::lifecycle::{Age, Vitality, LifeStage};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::psychology::AttachmentHistory;
use game2_core::sim::religion::Faith;
use game2_core::sim::community::{Citizen, Organization};
use game2_core::sim::economics::Market;
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== GRAND SHOWCASE: the complete game ===\n");

    let mut world = World::new();
    let _telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);

    let _terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 400);
    let market = world.create_entity();
    world.add_component(market, Market { prices: [("food".into(),10.0)].into_iter().collect(), supply: [("food".into(),100)].into_iter().collect(), ..Default::default() });

    // Villagers with full stacks
    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%3) as f32);
        world.add_component(v, Hunger { value: 90.0 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3+(i as f32*0.1), traditions: vec![] });
        world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        world.add_component(v, Age { years: 20.0+(i as f32*15.0), stage: LifeStage::Adult });
        world.add_component(v, Vitality { health: 0.9, max_age: 75.0+(i as f32*10.0), cause_of_death: None });
        villagers.push(v);
    }

    // Player
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 150.0, standing: 0.0 });
    let org = world.create_entity();
    world.add_component(org, Organization { name: "Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });

    // Merchants + pet
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant { name: format!("Merchant{}",i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
    }
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    let mut techs = extended_printmaking_tree();
    println!("tick,season,hunger,org_members,guilds,dog_bond,player_infl,techs\n");
    for tick in 0..400 {
        sched.tick(&mut world);

        // Player actions
        if tick % 15 == 0 && tick > 0 {
            let target = villagers[(tick/15) as usize % 5];
            match tick % 45 {
                0 => { act_talk(&mut world, player, target); }
                15 => { let _r = act_recruit(&mut world, player, org, target); }
                30 => { act_invest(&mut world, player, 10.0); }
                _ => {}
            }
        }

        // Tech + lifecycle
        if tick == 80 { research(&mut techs, "Papyrus", 10.0); }
        if tick == 150 { research(&mut techs, "Parchment", 15.0); }
        if tick == 220 { research(&mut techs, "Paper", 20.0); }
        for v in &villagers {
            if let Some(a) = world.get_component_mut::<Age>(*v) { a.years += 0.1; }
        }

        if tick % 40 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 5.0;
            let members = world.get_component::<Organization>(org).unwrap().members.len();
            let guilds = world.query::<Guild>().len();
            let bond = world.get_component::<Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let infl = world.get_component::<PlayerCharacter>(player).unwrap().influence;
            let techs_done = techs.iter().filter(|t| t.discovered).count();
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            println!("{},{:?},{:.1},{},{},{:.2},{:.2},{}", tick, cl.season, hunger, members, guilds, bond, infl, techs_done);
        }
    }

    println!("\n=== Summary ===");
    println!("org: {} members", world.get_component::<Organization>(org).unwrap().members.len());
    println!("guilds: {}", world.query::<Guild>().len());
    println!("player influence: {:.2}", world.get_component::<PlayerCharacter>(player).unwrap().influence);
    println!("techs: {}", techs.iter().filter(|t| t.discovered).count());
    println!("=== GRAND SHOWCASE COMPLETE ===");
}
