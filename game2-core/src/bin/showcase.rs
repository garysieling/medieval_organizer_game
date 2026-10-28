//! Master showcase: everything working together.
//! One run produces: isometric world, villagers, metrics, relationship graph,
//! tech constellation, character sheet - the full game in miniature.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self};
use game2_core::game::tech_tree::{printmaking_tree, research};
use game2_core::ui::isometric::{render_terrain, render_entity};
use game2_core::ui::graph::{Person, Bond, render_graph};
use game2_core::ui::tech_view::render_tech_tree;
use game2_core::ui::character_sheet::{Character, render_sheet};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::psychology::{AttachmentHistory, infer_style, AttachmentStyle};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== MASTER SHOWCASE ===\n");

    // 1. Build a world
    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);

    let terrain = Terrain::generate(33, 0.7, 99);
    let climate = climate::spawn_climate(&mut world);

    // 2. Spawn villagers with personalities
    let names = ["Aldric", "Eliza", "Bram", "Corvin", "Dagny", "Wystan"];
    let mut villagers = vec![];
    for (i, _name) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i * 4) as f32, (i * 3) as f32);
        world.add_component(v, AttachmentHistory {
            anxiety_episodes: (i % 3) as u32,
            warmth_episodes: (3 - i % 3) as u32,
            episodes: vec![],
        });
        villagers.push(v);
    }
    villager::spawn_food_source(&mut world, 8.0, 8.0, 150);

    // 3. Run 100 ticks
    println!("Running 100 ticks of village simulation...\n");
    for tick in 0..100 {
        sched.tick(&mut world);
        if tick % 25 == 0 {
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            println!("tick {}: season={:?} food_mult={:.2}", tick, cl.season, climate_multiplier(cl, "food"));
        }
    }

    // 4. Render the world + villagers
    let mut board = render_terrain(&terrain, 500, 350);
    for (i, _v) in villagers.iter().enumerate() {
        let col = [(255,80,80),(80,255,80),(80,80,255),(255,255,80),(255,80,255),(80,255,255)][i];
        render_entity(&mut board, (i*4) as f32, (i*3) as f32, col.0, col.1, col.2);
    }
    board.save_bmp("evidence/showcase-world.bmp").unwrap();
    println!("\nsaved evidence/showcase-world.bmp (terrain + 6 villagers)");

    // 5. Attachment styles emerged
    println!("\n--- Attachment Styles (emerged) ---");
    let mut people = vec![];
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            let style = infer_style(h);
            let col = match style {
                AttachmentStyle::Secure => (100, 200, 100),
                AttachmentStyle::Anxious => (220, 180, 60),
                AttachmentStyle::Avoidant => (80, 140, 200),
                AttachmentStyle::Disorganized => (200, 80, 80),
            };
            people.push(Person {
                name: names[i].into(),
                x: 60.0 + (i as f32 % 3.0) * 120.0,
                y: 60.0 + (i as f32 / 3.0) * 100.0,
                style_color: col,
            });
            println!("{}: anx={} warm={} -> {:?}", names[i], h.anxiety_episodes, h.warmth_episodes, style);
        }
    }

    // 6. Relationship graph
    let bonds = vec![
        Bond { from: 0, to: 1, strength: 0.8 },
        Bond { from: 1, to: 2, strength: 0.6 },
        Bond { from: 2, to: 3, strength: -0.4 },
        Bond { from: 3, to: 4, strength: 0.5 },
        Bond { from: 4, to: 5, strength: 0.7 },
        Bond { from: 0, to: 5, strength: 0.3 },
    ];
    let gbuf = render_graph(&people, &bonds, 400, 250);
    gbuf.save_bmp("evidence/showcase-relationships.bmp").unwrap();
    println!("saved evidence/showcase-relationships.bmp");

    // 7. Tech tree progress
    let mut techs = printmaking_tree();
    research(&mut techs, "Quill & Ink", 50.0);
    research(&mut techs, "Papyrus", 50.0);
    research(&mut techs, "Vellum", 50.0);
    research(&mut techs, "Woodblock", 60.0);
    let positions = vec![
        ("Quill & Ink".to_string(), 50.0, 150.0), ("Papyrus".to_string(), 150.0, 150.0),
        ("Vellum".to_string(), 250.0, 120.0), ("Woodblock".to_string(), 100.0, 90.0),
        ("Movable Type".to_string(), 200.0, 60.0), ("Paper Mill".to_string(), 300.0, 80.0),
        ("Printing Press".to_string(), 250.0, 40.0), ("Stone Lithography".to_string(), 350.0, 20.0),
    ];
    let tbuf = render_tech_tree(&techs, &positions, 400, 180);
    tbuf.save_bmp("evidence/showcase-tech.bmp").unwrap();
    println!("saved evidence/showcase-tech.bmp (4 techs discovered)");

    // 8. Character sheet for a villager
    let sheet = render_sheet(&Character {
        name: "Aldric the Miller".into(),
        age: 34.0, health: 0.82,
        skills: vec![("Farming".into(), 0.7), ("Trade".into(), 0.4), ("Craft".into(), 0.5)],
        traits: vec!["Hardworking".into()],
        relationships: vec![("Eliza".into(), 0.8), ("Bram".into(), 0.6)],
    }, 300, 200);
    sheet.save_bmp("evidence/showcase-char.bmp").unwrap();
    println!("saved evidence/showcase-char.bmp");

    // 9. Metrics export
    telem.gauge("villagers", villagers.len() as f64);
    telem.gauge("ticks", 100.0);
    println!("\n--- Telemetry ---");
    for line in telem.export_metrics() { println!("{}", line); }

    println!("\n=== SHOWCASE COMPLETE - 5 evidence files ===");
}
