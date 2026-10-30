//! Evidence demo - exercises each system and prints real output.
//! Run with: cargo run --bin demo
//! Output gets captured to evidence/ as proof the software works.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger};
use game2_core::sim::economics::Market;
use game2_core::sim::psychology::{infer_style, AttachmentHistory};

fn main() {
    println!("=== EVIDENCE DEMO ===\n");

    println!("--- C2: Terrain Generation (seed=7) ---");
    let t = Terrain::generate(33, 0.6, 7);
    println!("{}", t.ascii());

    println!("--- villager: Hunger Cycle ---");
    let mut world = World::new();
    let v = villager::spawn_villager(&mut world, 0.0, 0.0);
    villager::spawn_food_source(&mut world, 4.0, 0.0, 5);
    world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);

    for tick in 0..=6 {
        let h = world.get_component::<Hunger>(v).unwrap().value;
        let a = world.get_component::<villager::Villager>(v).unwrap().activity;
        println!("tick {}: hunger={:.1} activity={:?}", tick, h, a);
        sched.tick(&mut world);
    }

    println!("\n--- B1: Economics (price formation) ---");
    let mut m = Market::default();
    m.prices.insert("food".to_string(), 10.0);
    m.add_supply("food", 10);
    for demand in [10, 20, 40, 80] {
        m.update_price("food", demand);
        println!("demand={:3} supply={:3} price={:.2}", demand, m.supply["food"], m.prices["food"]);
    }

    println!("\n--- B2: Psychology (attachment emergence) ---");
    for (anx, warm) in [(1u32, 9u32), (8, 2), (2, 3), (5, 1)] {
        let h = AttachmentHistory { anxiety_episodes: anx, warmth_episodes: warm, episodes: vec![] };
        println!("anx={} warm={} -> {:?}", anx, warm, infer_style(&h));
    }

    println!("\n--- D1: Isometric Board Render (headless) ---");
    let mut board = game2_core::ui::isometric::render_terrain(&t, 400, 300);
    // Render villagers as entities on the board
    game2_core::ui::isometric::render_entity(&mut board, 5.0, 5.0, 255, 100, 100); // villager 1 (red)
    game2_core::ui::isometric::render_entity(&mut board, 8.0, 7.0, 100, 255, 100); // villager 2 (green)
    game2_core::ui::isometric::render_entity(&mut board, 3.0, 10.0, 100, 100, 255); // villager 3 (blue)
    game2_core::ui::isometric::render_entity(&mut board, 12.0, 6.0, 255, 200, 0);  // food source (gold)
    board.save_bmp("evidence/sprint-3-isometric-board.bmp").unwrap();
    println!("saved evidence/sprint-3-isometric-board.bmp ({}x{}) - terrain + entities", board.width, board.height);

    println!("\n--- D2: Minimap with Fog-of-War Fade ---");
    let mut fog = game2_core::ui::minimap::FogOfWar::new(20, 20);
    fog.see(10, 10, 5); // player at center sees radius 5
    let heights: Vec<Vec<f32>> = (0..20).map(|_| (0..20).map(|x| x as f32 / 20.0).collect()).collect();
    let mm = fog.render(&heights);
    mm.save_bmp("evidence/sprint-3-minimap.bmp").unwrap();
    println!("saved evidence/sprint-3-minimap.bmp ({}x{})", mm.width, mm.height);
    // Simulate fade over time
    for _ in 0..10 { fog.decay(0.08); }
    let mm2 = fog.render(&heights);
    mm2.save_bmp("evidence/sprint-3-minimap-faded.bmp").unwrap();
    println!("saved evidence/sprint-3-minimap-faded.bmp (after 10 ticks fade)");

    println!("\n--- B6: Climate (seasons + drift) ---");
    let mut cw = World::new();
    let c = game2_core::sim::climate::spawn_climate(&mut cw);
    let mut csched = Scheduler::new();
    csched.add_system(game2_core::sim::climate::SeasonSystem);
    for _ in 0..5 {
        let cl = cw.get_component::<game2_core::sim::climate::Climate>(c).unwrap();
        println!("year={} season={:?} severity={:.2} food_mult={:.2}",
            cl.year, cl.season, cl.severity,
            game2_core::sim::climate::climate_multiplier(cl, "food"));
        csched.tick(&mut cw);
    }

    println!("\n--- F7: Emergent Effects (CA grass/forest spread) ---");
    let mut ca = game2_core::sim::emergence::Automata::new(20, 20);
    ca.seed();
    println!("tick 0:\n{}", ca.ascii());
    for _ in 0..5 { ca.step(); }
    println!("tick 5 (trees grew from grass):\n{}", ca.ascii());
    // Start a fire
    ca.cells[10][10] = game2_core::sim::emergence::Cell::Burning;
    for _ in 0..5 { ca.step(); }
    println!("tick 10 (fire spread):\n{}", ca.ascii());


    println!("
--- UI: Relationship Graph ---");
    let people = vec![
        game2_core::ui::graph::Person { name: "Aldric".into(), x: 60.0, y: 80.0, style_color: (100, 200, 100) },   // Secure
        game2_core::ui::graph::Person { name: "Eliza".into(), x: 180.0, y: 60.0, style_color: (100, 200, 100) },   // Secure
        game2_core::ui::graph::Person { name: "Bram".into(), x: 120.0, y: 140.0, style_color: (220, 180, 60) },    // Anxious
        game2_core::ui::graph::Person { name: "Corvin".into(), x: 240.0, y: 120.0, style_color: (200, 80, 80) },   // Disorganized
    ];
    let bonds = vec![
        game2_core::ui::graph::Bond { from: 0, to: 1, strength: 0.8 },
        game2_core::ui::graph::Bond { from: 0, to: 2, strength: 0.5 },
        game2_core::ui::graph::Bond { from: 1, to: 3, strength: -0.4 },
        game2_core::ui::graph::Bond { from: 2, to: 3, strength: 0.3 },
    ];
    let gbuf = game2_core::ui::graph::render_graph(&people, &bonds, 300, 200);
    gbuf.save_bmp("evidence/relationship-graph.bmp").unwrap();
    println!("saved evidence/relationship-graph.bmp");

    println!("\n--- UI: Tech Tree Constellation (printmaking) ---");
    let mut techs = game2_core::game::tech_tree::printmaking_tree();
    game2_core::game::tech_tree::research(&mut techs, "Quill & Ink", 20.0);
    game2_core::game::tech_tree::research(&mut techs, "Papyrus", 20.0);
    game2_core::game::tech_tree::research(&mut techs, "Vellum", 30.0);
    let positions = vec![
        ("Quill & Ink".to_string(), 50.0, 150.0),
        ("Papyrus".to_string(), 150.0, 150.0),
        ("Vellum".to_string(), 250.0, 120.0),
        ("Woodblock".to_string(), 100.0, 90.0),
        ("Movable Type".to_string(), 200.0, 60.0),
        ("Paper Mill".to_string(), 300.0, 80.0),
        ("Printing Press".to_string(), 250.0, 40.0),
        ("Stone Lithography".to_string(), 350.0, 20.0),
    ];
    let tbuf = game2_core::ui::tech_view::render_tech_tree(&techs, &positions, 400, 180);
    tbuf.save_bmp("evidence/tech-tree.bmp").unwrap();
    println!("saved evidence/tech-tree.bmp");

    render_character_demo();
    println!("\n=== DEMO COMPLETE ===");
}

// Render a character sheet with real entity data
fn render_character_demo() {
    use game2_core::ui::character_sheet::{Character, render_sheet};
    let c = Character {
        name: "Aldric the Miller".into(),
        age: 34.0,
        health: 0.82,
        skills: vec![
            ("Farming".into(), 0.7),
            ("Trade".into(), 0.4),
            ("Combat".into(), 0.2),
            ("Craft".into(), 0.5),
        ],
        traits: vec!["Hardworking".into(), "Anxious".into()],
        relationships: vec![
            ("Eliza".into(), 0.8),
            ("Bram".into(), 0.6),
            ("Lord".into(), -0.3),
        ],
    };
    let buf = render_sheet(&c, 300, 200);
    buf.save_bmp("evidence/character-sheet.bmp").unwrap();
    println!("saved evidence/character-sheet.bmp");
}
