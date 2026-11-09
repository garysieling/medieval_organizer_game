//! B2: SimplePlaytester runs goal-oriented tests.
//! "survive 100 ticks", "feed 5 villagers", "reach tech X" - UI and backend.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, FoodSource};
use game2_core::agents::simple_playtester::{SimplePlaytester, Goal, GoalCheck};

fn main() {
    game2_core::infra::crash::install();
    println!("=== SIMPLE PLAYTESTER ===\n");

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 300);

    let playtester = SimplePlaytester::new(false); // backend API variant

    // Goal 1: survive 100 ticks
    let goal1 = Goal {
        description: "Survive 100 ticks".into(),
        check: GoalCheck::TickReached(100),
    };
    let r1 = playtester.playtest(&mut world, &mut sched, &goal1, 150);
    println!("Goal: {}", r1.goal);
    println!("  achieved: {} ({} ticks)", r1.achieved, r1.ticks_used);
    println!("  notes: {}\n", r1.notes);

    // Goal 2: feed villagers (entity count check)
    for i in 0..5 {
        villager::spawn_villager(&mut world, (i*3) as f32, 0.0);
    }
    let goal2 = Goal {
        description: "5 villagers exist".into(),
        check: GoalCheck::EntityCount(6), // 5 villagers + food source
    };
    let r2 = playtester.playtest(&mut world, &mut sched, &goal2, 50);
    println!("Goal: {}", r2.goal);
    println!("  achieved: {} ({} ticks)", r2.achieved, r2.ticks_used);
    println!("  notes: {}\n", r2.notes);

    // Goal 3: villagers eat (hunger resets - custom check)
    let goal3 = Goal {
        description: "Villagers eat (hunger resets)".into(),
        check: GoalCheck::Custom(|w| {
            w.query::<FoodSource>().iter().any(|e| {
                w.get_component::<FoodSource>(*e).map(|f| f.servings < 300).unwrap_or(false)
            })
        }),
    };
    let r3 = playtester.playtest(&mut world, &mut sched, &goal3, 200);
    println!("Goal: {}", r3.goal);
    println!("  achieved: {} ({} ticks)", r3.achieved, r3.ticks_used);
    println!("  notes: {}\n", r3.notes);

    println!("=== PLAYTESTER COMPLETE - {} goals tested ===", 3);
}
