//! Final showcase: the complete game rendered.
//! World + villagers + relationships + tech + character + metrics in one output.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger};
use game2_core::game::tech_tree::{extended_printmaking_tree, research};
use game2_core::ui::isometric::{render_terrain, render_entity};
use game2_core::ui::graph::{Person, Bond, render_graph};
use game2_core::ui::tech_view::render_tech_tree;
use game2_core::ui::character_sheet::{Character, render_sheet};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::psychology::{AttachmentHistory, infer_style, AttachmentStyle};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== FINAL SHOWCASE: the complete game ===\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);

    let terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 200);

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny","Wystan"];
    let mut villagers = vec![];
    for (i, n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%4) as f32);
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (5-i) as u32, episodes: vec![] });
        villagers.push(v);
    }

    println!("Running 150 ticks...");
    for tick in 0..150 {
        sched.tick(&mut world);
        if tick % 50 == 0 {
            println!("  tick {}", tick);
        }
    }

    // 1. World render
    let mut board = render_terrain(&terrain, 500, 350);
    let colors = [(255,80,80),(80,255,80),(80,80,255),(255,255,80),(255,80,255),(80,255,255)];
    for (i, v) in villagers.iter().enumerate() {
        if let Some(p) = world.get_component::<game2_core::game::villager::Pos>(*v) {
            let c = colors[i];
            render_entity(&mut board, p.x, p.y, c.0, c.1, c.2);
        }
    }
    board.save_bmp("evidence/final-world.bmp").unwrap();
    println!("\nsaved evidence/final-world.bmp");

    // 2. Relationship graph
    let mut people = vec![];
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            let style = infer_style(h);
            let col = match style {
                AttachmentStyle::Secure => (100,200,100),
                AttachmentStyle::Anxious => (220,180,60),
                AttachmentStyle::Avoidant => (80,140,200),
                AttachmentStyle::Disorganized => (200,80,80),
            };
            people.push(Person { name: names[i].into(), x: 60.0+(i as f32%3.0)*120.0, y: 60.0+(i as f32/3.0)*100.0, style_color: col });
        }
    }
    let bonds = vec![Bond{from:0,to:1,strength:0.8},Bond{from:1,to:2,strength:0.5},Bond{from:2,to:3,strength:-0.4},Bond{from:3,to:4,strength:0.6},Bond{from:4,to:5,strength:0.7}];
    let gbuf = render_graph(&people, &bonds, 400, 250);
    gbuf.save_bmp("evidence/final-relationships.bmp").unwrap();
    println!("saved evidence/final-relationships.bmp");

    // 3. Tech tree (extended printmaking)
    let mut techs = extended_printmaking_tree();
    research(&mut techs, "Papyrus", 10.0);
    research(&mut techs, "Parchment", 15.0);
    research(&mut techs, "Paper", 20.0);
    research(&mut techs, "Carving", 10.0);
    let positions = vec![
        ("Papyrus".to_string(),40.0,140.0),("Parchment".to_string(),120.0,140.0),("Paper".to_string(),200.0,140.0),
        ("Paper Mill".to_string(),280.0,110.0),("Carving".to_string(),60.0,90.0),("Woodblock".to_string(),150.0,90.0),
        ("Woodcut Prints".to_string(),230.0,80.0),("Metalworking".to_string(),80.0,60.0),("Engraving".to_string(),180.0,60.0),
        ("Etching".to_string(),280.0,50.0),("Clay Type".to_string(),240.0,100.0),("Metal Type".to_string(),320.0,80.0),
        ("Type Foundry".to_string(),350.0,60.0),("Printing Press".to_string(),320.0,40.0),("Chemistry".to_string(),260.0,30.0),
        ("Lithography".to_string(),360.0,20.0),("Playing Cards".to_string(),140.0,50.0),("Devotional Images".to_string(),220.0,30.0),
        ("Printed Books".to_string(),300.0,70.0),("Scientific Illustration".to_string(),340.0,50.0),
    ];
    let tbuf = render_tech_tree(&techs, &positions, 400, 160);
    tbuf.save_bmp("evidence/final-tech.bmp").unwrap();
    println!("saved evidence/final-tech.bmp (extended printmaking)");

    // 4. Character sheet
    let sheet = render_sheet(&Character {
        name: "Aldric the Miller".into(), age: 34.0, health: 0.82,
        skills: vec![("Farming".into(),0.7),("Trade".into(),0.4),("Craft".into(),0.5)],
        traits: vec!["Hardworking".into(), "Anxious".into()],
        relationships: vec![("Eliza".into(),0.8),("Bram".into(),0.6)],
    }, 300, 200);
    sheet.save_bmp("evidence/final-char.bmp").unwrap();
    println!("saved evidence/final-char.bmp");

    // 5. Metrics
    telem.gauge("villagers", villagers.len() as f64);
    telem.gauge("ticks", 150.0);
    for line in telem.export_metrics() { println!("{}", line); }

    println!("\n=== FINAL SHOWCASE COMPLETE - 4 renders ===");
}
