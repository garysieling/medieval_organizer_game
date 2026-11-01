//! Hard winter: a village goes into a failing winter on thin stores, its
//! old leader already gravely ill. Sickness and starvation wear health down
//! until some villagers die, and the
//! mortality chain plays out on its own: the dead stop eating, those who
//! loved them grieve, a dead owner's dog finds a new keeper, and the guild
//! takes its dead off the rolls and passes on leadership.

use game2_core::engine::ecs::{Entity, Scheduler, World};
use game2_core::game::organizing::living_leader;
use game2_core::game::pets::{self, Pet, PetRole, Species};
use game2_core::game::relationships::{self, Relationship, RelKind};
use game2_core::game::villager::{self, Hunger};
use game2_core::sim::climate::{self, Climate, Season, SeasonSystem};
use game2_core::sim::community::{Citizen, InfluenceSystem, Organization};
use game2_core::sim::illness::{Disease, DiseaseKind, Health, IllnessSystem};
use game2_core::sim::psychology::{AttachmentHistory, Bond, GriefSystem};

const TICKS: u64 = 400;

fn main() {
    println!("=== HARD WINTER ===\n");
    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(villager::CropGrowthSystem);
    sched.add_system(villager::NourishmentSystem);
    sched.add_system(IllnessSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GriefSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(InfluenceSystem);

    // Deep winter in a harsh, warming climate
    let cl = climate::spawn_climate(&mut world);
    if let Some(c) = world.get_component_mut::<Climate>(cl) {
        c.season = Season::Winter;
        c.day = 270;
        c.severity = 0.6;
    }
    // One meagre field for the whole village
    villager::spawn_crop_field(&mut world, 6.0, 4.0, 1, FIELD_GROWTH);

    let names = ["Aldric", "Eliza", "Bram", "Corvin", "Dagny"];
    let villagers: Vec<Entity> = names.iter().enumerate().map(|(i, _)| {
        let x = (i * 3) as f32;
        let v = villager::spawn_villager(&mut world, x, 0.0);
        world.get_component_mut::<Hunger>(v).unwrap().value = 30.0 + i as f32 * 10.0;
        world.add_component(v, Health { condition: 0.6, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 2, episodes: vec![] });
        world.add_component(v, Citizen { influence: 0.1 + i as f32 * 0.1, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        v
    }).collect();

    // Eliza's attachment figure is Aldric; Bram and Dagny are close friends
    world.add_component(villagers[1], Bond { target: villagers[0], security: 0.8 });
    for (a, b, strength) in [(0, 2, 0.7), (2, 4, 0.6), (3, 4, 0.4)] {
        relationships::add_relationship(&mut world, Relationship { from: villagers[a], other: villagers[b], kind: RelKind::Friend, strength, interactions: 8 });
    }
    // Aldric, the old guild leader, goes into the winter with a wasting
    // sickness his body can't fight
    if let Some(h) = world.get_component_mut::<Health>(villagers[0]) { h.immunity = 0.0; }
    let sickness = world.create_entity();
    world.add_component(sickness, Disease::new(DiseaseKind::Consumption, villagers[0], 0.8));

    // Aldric keeps a dog and leads the village guild
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);
    let guild = world.create_entity();
    world.add_component(guild, Organization {
        name: "Harvesters".into(), members: villagers[1..].to_vec(), leader: villagers[0],
        influence_pool: 0.0, resources: 0.0, charter: "Mutual aid".into(),
    });

    let name = |e: Entity| villagers.iter().position(|v| *v == e).map(|i| names[i]).unwrap_or("nobody");
    let health = |w: &World, e: Entity| w.get_component::<Health>(e).map(|h| h.condition).unwrap_or(0.0);
    let losses = |w: &World, e: Entity| w.get_component::<AttachmentHistory>(e).map(|h| h.anxiety_episodes).unwrap_or(0);

    println!("tick,alive,avg_hunger,avg_health,food_left\n");
    let mut reported: Vec<Entity> = vec![];
    for tick in 0..TICKS {
        let grief_before: Vec<u32> = villagers.iter().map(|v| losses(&world, *v)).collect();
        sched.tick(&mut world);

        for (i, &v) in villagers.iter().enumerate() {
            if health(&world, v) > 0.0 || reported.contains(&v) { continue; }
            reported.push(v);
            let cause = if world.get_component::<Hunger>(v).map(|h| h.value <= 0.0).unwrap_or(false) { "hunger" } else { "sickness" };
            println!("EVENT tick {}: {} dies of {}", tick, names[i], cause);
            let mourners: Vec<&str> = villagers.iter().enumerate()
                .filter(|(j, m)| losses(&world, **m) > grief_before[*j])
                .map(|(j, _)| names[j]).collect();
            if !mourners.is_empty() { println!("  grieved by {}", mourners.join(", ")); }
            // The systems that ran this tick after the death have already
            // rehomed the dog; leadership passes on when it's next needed
            if world.get_component::<Pet>(dog).map(|p| p.owner != Some(v)).unwrap_or(false) && v == villagers[0] {
                let keeper = world.get_component::<Pet>(dog).and_then(|p| p.owner);
                println!("  Rex is taken in by {}", keeper.map(name).unwrap_or("no one - a stray now"));
            }
            if world.get_component::<Organization>(guild).map(|o| o.leader == v).unwrap_or(false) {
                match living_leader(&mut world, guild) {
                    Some(l) => println!("  {} now leads the Harvesters", name(l)),
                    None => println!("  the Harvesters have no one left to lead them"),
                }
            }
        }

        if tick % 50 == 0 || tick == TICKS - 1 {
            let alive: Vec<&Entity> = villagers.iter().filter(|v| health(&world, **v) > 0.0).collect();
            let n = alive.len().max(1) as f32;
            let hunger: f32 = alive.iter().map(|v| world.get_component::<Hunger>(**v).map(|h| h.value).unwrap_or(0.0)).sum::<f32>() / n;
            let hp: f32 = alive.iter().map(|v| health(&world, **v)).sum::<f32>() / n;
            println!("{},{},{:.1},{:.2},{}", tick, alive.len(), hunger, hp, villager::servings_left(&world));
        }
    }

    let members = world.get_component::<Organization>(guild).map(|o| o.members.len()).unwrap_or(0);
    println!("\nsurvivors: {} of {}; guild members on the rolls: {}", villagers.len() - reported.len(), villagers.len(), members);
    println!("\n=== HARD WINTER COMPLETE ===");
}

/// Servings the village's one field ripens per tick in a normal season
const FIELD_GROWTH: f32 = 0.03;
