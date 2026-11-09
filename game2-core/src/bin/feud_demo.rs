//! Feud demo: vengeance cycle - offense -> escalation -> peacemaking/burning.
//! From sagas: "can't get even without going one better."

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::feuds::{spawn_feud, Feud, FeudStatus, VengeanceKind, vengeance_escalation, peacemaking, FeudSystem};

fn main() {
    game2_core::infra::crash::install();
    println!("=== FEUD: vengeance cycle ===\n");
    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(FeudSystem);

    let aldric = world.create_entity();
    let corvin = world.create_entity();
    let arbitrator = world.create_entity();

    let feud = spawn_feud(&mut world, corvin, aldric, "insult at the tavern");
    println!("Feud: Corvin offends Aldric (insult)\n");

    let mut current = VengeanceKind::Slander;
    for tick in 0..20 {
        sched.tick(&mut world);
        if let Some(f) = world.get_component::<Feud>(feud) {
            let status = match f.status {
                FeudStatus::Active => "escalating",
                FeudStatus::Burning => "BURNING",
                FeudStatus::Resolved => "resolved",
                FeudStatus::Peacemaking => "peacemaking",
            };
            if tick % 5 == 0 {
                println!("tick {}: intensity={:.2} status={}", tick, f.intensity, status);
            }
            if f.status == FeudStatus::Burning && f.intensity > 0.75 {
                println!("  vengeance escalates: {:?} -> {:?} (going one better)", current, vengeance_escalation(&current));
                current = vengeance_escalation(&current);
            }
        }
    }

    println!("\nAttempting peacemaking (arbitration)...");
    let resolved = peacemaking(&mut world, feud, arbitrator);
    println!("resolved: {}", resolved);
    if let Some(f) = world.get_component::<Feud>(feud) {
        println!("final status: {:?} intensity={:.2}", f.status, f.intensity);
    }
    println!("\n=== FEUD COMPLETE ===");
}
