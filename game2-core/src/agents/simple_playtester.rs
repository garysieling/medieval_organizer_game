//! Simple Playtester: executes goal-oriented tests.
//! "build 5 units", "reach tech X", "survive 100 turns" - UI and non-UI variants.

use crate::engine::ecs::{Scheduler, World};

/// A playtest goal
#[derive(Debug)]
pub struct Goal {
    pub description: String,
    pub check: GoalCheck,
}

#[derive(Debug)]
pub enum GoalCheck {
    EntityCount(usize),          // N entities exist
    ComponentExists(String),     // component type exists on any entity
    TickReached(u64),            // survive to tick N
    Custom(fn(&World) -> bool),  // custom check
}

/// Result of a playtest
#[derive(Debug)]
pub struct PlaytestResult {
    pub goal: String,
    pub achieved: bool,
    pub ticks_used: u64,
    pub notes: String,
}

/// Simple Playtester agent
pub struct SimplePlaytester {
    pub name: String,
    pub ui_variant: bool, // true=UI, false=backend API
}

impl SimplePlaytester {
    pub fn new(ui: bool) -> Self {
        Self {
            name: "Simple Playtester".into(),
            ui_variant: ui,
        }
    }

    /// Run a playtest against a goal
    pub fn playtest(&self, world: &mut World, sched: &mut Scheduler, goal: &Goal, max_ticks: u64) -> PlaytestResult {
        for tick in 0..max_ticks {
            sched.tick(world);
            let achieved = match &goal.check {
                GoalCheck::TickReached(n) => tick >= *n,
                GoalCheck::EntityCount(n) => world.entity_count() as usize >= *n,
                GoalCheck::ComponentExists(_) => true, // placeholder
                GoalCheck::Custom(f) => f(world),
            };
            if achieved {
                return PlaytestResult {
                    goal: goal.description.clone(),
                    achieved: true,
                    ticks_used: tick,
                    notes: format!("Achieved in {} ticks", tick),
                };
            }
        }
        PlaytestResult {
            goal: goal.description.clone(),
            achieved: false,
            ticks_used: max_ticks,
            notes: "Not achieved within tick limit".into(),
        }
    }
}
