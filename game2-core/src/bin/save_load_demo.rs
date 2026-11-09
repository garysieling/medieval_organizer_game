//! Save/load demo: Datomic-style history with checkpoints + replay.
//! Shows world state saved at ticks, can replay from any point.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger};
use game2_core::infra::save_load::History;

fn main() {
    game2_core::infra::crash::install();
    println!("=== Datomic Save/Load Demo ===\n");

    let mut world = World::new();
    let mut history = History::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);

    let v = villager::spawn_villager(&mut world, 0.0, 0.0);

    // Run sim, checkpoint every 5 ticks
    for tick in 0..20 {
        sched.tick(&mut world);
        if tick % 5 == 0 {
            let h = world.get_component::<Hunger>(v).unwrap().value;
            history.current_tick = tick;
            history.checkpoint(&world, &format!("tick_{}", tick));
            println!("checkpoint at tick {}: hunger={:.1}", tick, h);
        }
    }

    println!("\n=== History ===");
    for cp in &history.checkpoints {
        println!("{}: tick={} data={:?}", cp.id, cp.tick, cp.data);
    }

    println!("\n=== Replay from tick_10 ===");
    if let Some(cp) = history.replay_from("cp_tick_10") {
        println!("replaying from checkpoint {} at tick {}", cp.id, cp.tick);
        println!("would restore world state: {:?}", cp.data);
    }

    println!("\n=== Latest checkpoint ===");
    if let Some(cp) = history.latest() {
        println!("{}: tick={}", cp.id, cp.tick);
    }
    println!("\n=== COMPLETE ===");
}
