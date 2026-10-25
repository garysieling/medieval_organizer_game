//! Music evolution: techno layers change with world state.
//! Calm village -> growing -> tense (conflict) -> triumphant.

use game2_core::engine::ecs::World;
use game2_core::game::music::{spawn_music, evolve, describe, WorldMetrics, MusicState};

fn main() {
    println!("=== MUSIC EVOLUTION (techno layers) ===\n");
    let mut world = World::new();
    let music = spawn_music(&mut world);

    // World states evolve: calm -> growing -> tense -> triumphant
    let states = vec![
        ("Calm village", WorldMetrics { growth: 0.1, conflict: 0.0, loss: 0.0, discovery: 0.0, prosperity: 0.2 }),
        ("Growing settlement", WorldMetrics { growth: 0.7, conflict: 0.1, loss: 0.0, discovery: 0.0, prosperity: 0.4 }),
        ("Conflict approaching", WorldMetrics { growth: 0.2, conflict: 0.8, loss: 0.2, discovery: 0.0, prosperity: 0.3 }),
        ("Victory achieved", WorldMetrics { growth: 0.5, conflict: 0.2, loss: 0.1, discovery: 0.3, prosperity: 0.8 }),
    ];

    for (name, metrics) in &states {
        println!("\n--- {} ---", name);
        for _ in 0..10 { // let layers evolve
            if let Some(m) = world.get_component_mut::<MusicState>(music) {
                evolve(m, metrics);
            }
        }
        if let Some(m) = world.get_component::<MusicState>(music) {
            println!("{}", describe(m));
            println!("spotify: {}", game2_core::game::music::recommend_mood(m.mood));
        }
    }
    println!("\n=== MUSIC EVOLUTION COMPLETE ===");
}
