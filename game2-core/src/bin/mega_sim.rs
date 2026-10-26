//! Mega sim: many systems running together on one world.
//! Villagers (hunger/health/attachment) + pets + market + climate + religion + illness.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::tech_tree::{printmaking_tree, research, can_research};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::illness::{Health, Disease, Stage};
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::religion::{Faith, Congregation, Service};
use game2_core::sim::economics::Market;
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== MEGA SIM: All Systems Together ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);

    // Villagers
    let mut villagers = vec![];
    for i in 0..5 {
        let v = villager::spawn_villager(&mut world, (i * 3) as f32, 0.0);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        villagers.push(v);
    }

    // Pets
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);
    let cat = pets::spawn_pet(&mut world, "Whiskers", Species::Cat, PetRole::Companion);
    pets::adopt(&mut world, cat, villagers[1]);

    // Market + climate
    let market = world.create_entity();
    world.add_component(market, Market { prices: [("food".into(), 10.0)].into_iter().collect(), supply: [("food".into(), 100)].into_iter().collect() });
    let climate = climate::spawn_climate(&mut world);

    // Religion: a congregation
    let church = world.create_entity();
    world.add_component(church, Congregation {
        name: "Village Chapel".into(),
        members: villagers.clone(),
        leader: Some(villagers[0]),
        orthodoxy: 0.6,
        services: vec![Service { name: "Mass".into(), frequency: "weekly".into(), participants: vec![], cohesion_boost: 0.15, charity_output: 0.1 }],
    });

    // Tech tree
    let mut techs = printmaking_tree();

    println!("tick,hunger,health,pet_bonds,season,food_mult,techs_done\n");
    for tick in 0..150 {
        sched.tick(&mut world);

        // Progress tech over time
        if tick == 40 { research(&mut techs, "Quill & Ink", 50.0); }
        if tick == 80 { research(&mut techs, "Papyrus", 50.0); }
        if tick == 120 { research(&mut techs, "Vellum", 60.0); }

        if tick % 15 == 0 {
            let hunger: f32 = villagers.iter().map(|v| world.get_component::<Hunger>(*v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / 5.0;
            let health: f32 = villagers.iter().map(|v| world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0)).sum::<f32>() / 5.0;
            let dog_bond = world.get_component::<pets::Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0);
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let techs_done = techs.iter().filter(|t| t.discovered).count();
            println!("{},{:.1},{:.2},{:.2},{:?},{:.2},{}", tick, hunger, health, dog_bond, cl.season, climate_multiplier(cl, "food"), techs_done);
            telem.set_tick(tick as u64);
            telem.gauge("hunger", hunger as f64);
        }
    }

    println!("\n=== Final States ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("villager{}: attachment={:?}", i, infer_style(h));
        }
    }
    println!("dog bond: {:.2}", world.get_component::<pets::Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0));
    println!("cat bond: {:.2}", world.get_component::<pets::Pet>(cat).map(|p| p.bond_strength).unwrap_or(0.0));
    println!("techs discovered: {}", techs.iter().filter(|t| t.discovered).count());
    println!("\n=== MEGA SIM COMPLETE ===");
}
