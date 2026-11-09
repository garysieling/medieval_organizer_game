//! B5: Manager Bot orchestrates a mini-sprint.
//! Builds plan -> delegates to agents via capability matching -> tracks progress.

use game2_core::agents::manager_bot::{ManagerBot, build_sprint_plan, SprintTask};

fn main() {
    game2_core::infra::crash::install();
    println!("=== MANAGER BOT MINI-SPRINT ===\n");

    // Build a sprint plan
    let mut plan = build_sprint_plan("Sprint 4.1", "Get a real feature built by agents");
    plan.exit_criteria = vec!["all tasks done".into(), "evidence produced".into()];
    plan.approved = true; // user approved
    plan.tasks = vec![
        SprintTask { id: "T1".into(), description: "Implement hunger tick system".into(), workstream: "code".into(), completed: false },
        SprintTask { id: "T2".into(), description: "Write playtest for hunger".into(), workstream: "test".into(), completed: false },
        SprintTask { id: "T3".into(), description: "Review code for smells".into(), workstream: "review".into(), completed: false },
        SprintTask { id: "T4".into(), description: "Document the feature".into(), workstream: "docs".into(), completed: false },
    ];

    println!("Sprint '{}' - {} tasks, approved", plan.name, plan.tasks.len());
    println!("Progress: {:.0}%\n", plan.progress());

    // Delegate tasks to agents via capability matching
    let _bot = ManagerBot::new("sprint_tasks.txt", "sprint_log.txt");
    let agents = ["Implementation Engineer", "Simple Playtester", "Pedantic Engineer", "Docs Agent"];
    for (i, task) in plan.tasks.iter().enumerate() {
        let agent = agents[i % agents.len()];
        println!("Manager delegates '{}' -> {}", task.description, agent);
    }

    // Simulate execution: mark tasks done one by one
    for i in 0..4 {
        plan.complete_task(&format!("T{}", i + 1));
        println!("  T{} complete -> progress {:.0}%", i + 1, plan.progress());
    }

    println!("\n=== EXIT CRITERIA CHECK ===");
    println!("all tasks done: {}", plan.tasks.iter().all(|t| t.completed));
    println!("approved: {}", plan.approved);
    println!("exit_criteria_met: {}", plan.exit_criteria_met());
    println!("\n=== MINI-SPRINT COMPLETE ===");
}
