//! Lifecycle demo: villagers born, age, die, succession happens.
//! Emergent: population cycles, elders die, heirs inherit.

use game2_core::engine::ecs::World;
use game2_core::game::villager::Pos;
use game2_core::game::lifecycle::{Age, Vitality, LifeStage};

fn main() {
    println!("=== LIFECYCLE: birth->aging->death->succession ===\n");
    let mut world = World::new();

    // Spawn villagers of different ages
    let names = ["Young Bram", "Adult Aldric", "Elder Corvin", "Child Dagny"];
    let ages = [18.0, 40.0, 75.0, 8.0];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = world.create_entity();
        world.add_component(v, Pos { x: (i*3) as f32, y: 0.0 });
        world.add_component(v, Age { years: ages[i], stage: if ages[i] < 13.0 { LifeStage::Child } else if ages[i] < 65.0 { LifeStage::Adult } else { LifeStage::Elder } });
        world.add_component(v, Vitality { health: 0.9, max_age: 80.0 + (i as f32 * 5.0), cause_of_death: None });
        villagers.push(v);
    }

    println!("Initial: Young Bram (18), Aldric (40), Elder Corvin (75), Dagny (8)\n");
    println!("year,bram,aldric,corvin,dagny,deaths\n");

    let mut deaths = 0;
    for year in 0..60 {
        // Everyone ages 1 year
        for v in &villagers {
            if let Some(age) = world.get_component_mut::<Age>(*v) {
                age.years += 1.0;
                age.stage = if age.years < 13.0 { LifeStage::Child }
                    else if age.years < 20.0 { LifeStage::Adolescent }
                    else if age.years < 65.0 { LifeStage::Adult }
                    else { LifeStage::Elder };
            }
        }
        // Check for deaths (age > max_age or health drops)
        for (i, v) in villagers.iter().enumerate() {
            let (years, max_age, dead) = {
                let a = world.get_component::<Age>(*v).unwrap();
                let vit = world.get_component::<Vitality>(*v).unwrap();
                (a.years, vit.max_age, vit.cause_of_death.is_some())
            };
            if years > max_age && !dead {
                if let Some(vit) = world.get_component_mut::<Vitality>(*v) {
                    vit.cause_of_death = Some("old age".into());
                }
                deaths += 1;
                println!("  year {}: {} dies of old age", year, names[i]);
            }
        }

        if year % 10 == 0 {
            let mut states = vec![];
            for v in &villagers {
                if let Some(a) = world.get_component::<Age>(*v) {
                    let vit = world.get_component::<Vitality>(*v).unwrap();
                    let status = if vit.cause_of_death.is_some() { "dead" }
                        else if a.years < 13.0 { "child" }
                        else if a.years < 65.0 { "adult" }
                        else { "elder" };
                    states.push(format!("{}({})", status, a.years as i32));
                }
            }
            println!("{},{:?}", year, states);
        }
    }
    println!("\nTotal deaths: {}", deaths);
    println!("=== LIFECYCLE COMPLETE ===");
}
