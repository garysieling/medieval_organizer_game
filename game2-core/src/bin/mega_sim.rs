//! Mega sim: many systems running together on one world.
//! Villagers (hunger/health/attachment) + pets + crops + climate + religion + illness,
//! over a full year so every season comes round.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::tech_tree::{printmaking_tree, research, next_researchable};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::illness::{Health, Disease, DiseaseKind, IllnessSystem};
use game2_core::sim::psychology::{self, AttachmentHistory, Episode, EpisodeKind, infer_style};
use game2_core::sim::religion::{self, Faith, Congregation, Service, DevotionSystem};
use game2_core::infra::telemetry::Telemetry;

/// Servings each crop field holds when full
const FIELD_CAPACITY: u32 = 2;
/// Servings each field ripens per tick in a normal season
const FIELD_GROWTH: f32 = 0.028;
/// Research points each fed, healthy villager contributes per tick
const RESEARCH_PER_VILLAGER: f32 = 0.1;

fn main() {
    println!("=== MEGA SIM: All Systems Together ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(villager::NourishmentSystem); // starving hurts; eating well mends
    sched.add_system(villager::CropGrowthSystem); // fields regrow with the seasons
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(IllnessSystem); // diseases progress, spread to neighbours, and heal
    sched.add_system(DevotionSystem); // faith fades without practice

    // Villagers
    let mut villagers = vec![];
    for i in 0..5 {
        let v = villager::spawn_villager(&mut world, (i * 3) as f32, 0.0);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        world.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.3, traditions: vec![] });
        villagers.push(v);
    }
    // Three small crop fields along the row: they feed the village in spring
    // and summer, but winter runs lean
    for x in [2.0, 6.0, 10.0] {
        villager::spawn_crop_field(&mut world, x, 4.0, FIELD_CAPACITY, FIELD_GROWTH);
    }

    // Pets
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);
    let cat = pets::spawn_pet(&mut world, "Whiskers", Species::Cat, PetRole::Companion);
    pets::adopt(&mut world, cat, villagers[1]);

    let climate = climate::spawn_climate(&mut world);

    // Religion: the whole village belongs to the chapel
    let church = world.create_entity();
    world.add_component(church, Congregation {
        name: "Village Chapel".into(),
        members: villagers.clone(),
        leader: Some(villagers[0]),
        orthodoxy: 0.6,
        services: vec![],
    });

    // Tech tree
    let mut techs = printmaking_tree();

    let start_styles: Vec<_> = villagers.iter()
        .map(|v| infer_style(world.get_component::<AttachmentHistory>(*v).unwrap()))
        .collect();

    println!("tick,hunger,health,infected,devotion,dog_bond,cat_bond,season,food_mult,food_stock,techs_done\n");
    for tick in 0..360 {
        sched.tick(&mut world);

        // Research comes from the village: every villager who's fed and well
        // puts a little thought into the next tech, so hunger and la grippe
        // slow progress
        let thinkers = villagers.iter().filter(|v| {
            let fed = world.get_component::<Hunger>(**v).map(|h| h.value >= 30.0).unwrap_or(false);
            let well = world.get_component::<Health>(**v).map(|h| h.condition >= 0.8).unwrap_or(false);
            fed && well
        }).count();
        if let Some(next) = next_researchable(&techs).map(str::to_string) {
            if research(&mut techs, &next, thinkers as f32 * RESEARCH_PER_VILLAGER) {
                println!("EVENT: tick {}: the village works out {}", tick, next);
            }
        }

        // A traveller brings la grippe; it's contagious, so neighbours may catch it
        if tick == 100 {
            let d = world.create_entity();
            world.add_component(d, Disease::new(DiseaseKind::Influenza, villagers[2], 0.5));
            println!("EVENT: villager2 comes down with la grippe");
        }

        // Weekly Mass for everyone fed and well enough to attend
        if tick % 7 == 0 {
            let participants: Vec<_> = villagers.iter().copied().filter(|v| {
                let fed = world.get_component::<Hunger>(*v).map(|h| h.value >= 30.0).unwrap_or(true);
                let well = world.get_component::<Health>(*v).map(|h| h.condition >= 0.8).unwrap_or(true);
                fed && well
            }).collect();
            religion::perform_service(&mut world, church, Service {
                name: "Mass".into(), frequency: "weekly".into(),
                participants, cohesion_boost: 0.15, charity_output: 0.1,
            });
        }

        // Going hungry is distressing; a well-bonded pet is a comfort through it
        if tick % 10 == 0 {
            for (i, v) in villagers.iter().enumerate() {
                let hungry = world.get_component::<Hunger>(*v).map(|h| h.value < 30.0).unwrap_or(false);
                if !hungry { continue; }
                let other = villagers[(i + 1) % villagers.len()];
                psychology::record_episode(&mut world, *v, Episode { with: other, kind: EpisodeKind::Anxiety });
                if let Some(pet) = pets::comforting_pet(&world, *v) {
                    psychology::record_episode(&mut world, *v, Episode { with: pet, kind: EpisodeKind::Warmth });
                }
            }
        }

        if tick % 30 == 0 {
            let avg = |f: &dyn Fn(game2_core::engine::ecs::Entity) -> f32| villagers.iter().map(|v| f(*v)).sum::<f32>() / 5.0;
            let hunger = avg(&|v| world.get_component::<Hunger>(v).map(|h| h.value).unwrap_or(0.0));
            let health = avg(&|v| world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0));
            let devotion = avg(&|v| world.get_component::<Faith>(v).map(|f| f.devotion).unwrap_or(0.0));
            let infected = world.query::<Disease>().len();
            let bond = |p| world.get_component::<pets::Pet>(p).map(|p| p.bond_strength).unwrap_or(0.0);
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let techs_done = techs.iter().filter(|t| t.discovered).count();
            println!("{},{:.1},{:.2},{},{:.2},{:.2},{:.2},{:?},{:.2},{},{}",
                tick, hunger, health, infected, devotion, bond(dog), bond(cat),
                cl.season, climate_multiplier(cl, "food"), villager::servings_left(&world), techs_done);
            telem.set_tick(tick as u64);
            telem.gauge("hunger", hunger as f64);
            telem.gauge("health", health as f64);
        }
    }

    println!("\n=== Final States ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("villager{}: attachment={:?} -> {:?} (anx={} warm={}) health={:.2} devotion={:.2}",
                i, start_styles[i], infer_style(h), h.anxiety_episodes, h.warmth_episodes,
                world.get_component::<Health>(*v).map(|h| h.condition).unwrap_or(0.0),
                world.get_component::<Faith>(*v).map(|f| f.devotion).unwrap_or(0.0));
        }
    }
    println!("dog bond: {:.2}", world.get_component::<pets::Pet>(dog).map(|p| p.bond_strength).unwrap_or(0.0));
    println!("cat bond: {:.2}", world.get_component::<pets::Pet>(cat).map(|p| p.bond_strength).unwrap_or(0.0));
    println!("techs discovered: {}", techs.iter().filter(|t| t.discovered).count());
    println!("\n=== MEGA SIM COMPLETE ===");
}
