//! G1: PedanticEngineer reviews codebase, finds smells, proposes refactors.

use game2_core::agents::pedantic_senior_engineer::{PedanticEngineer, Severity};

fn main() {
    println!("=== PEDANTIC ENGINEER CODE REVIEW ===\n");

    let mut eng = PedanticEngineer::new();
    eng.review_cadence = 1; // review now

    // Simulate reviewing the codebase files
    let files = vec![
        ("engine/ecs.rs", 180, 0.4),
        ("agents/manager_bot.rs", 250, 0.6),
        ("sim/psychology.rs", 120, 0.3),
        ("game/economy.rs", 200, 0.5),
        ("ui/isometric.rs", 150, 0.4),
        ("infra/task_queue.rs", 280, 0.5),
    ];

    println!("Reviewing {} files...\n", files.len());
    let mut all_findings = vec![];
    for (file, lines, complexity) in &files {
        let findings = eng.review(file, *lines, *complexity);
        for f in &findings {
            let sev = match f.severity { Severity::Critical => "[!!]", Severity::Should => "[!]", Severity::Nitpick => "[.]" };
            println!("{} {}: {}", sev, f.location, f.description);
            println!("    -> {}", f.suggestion);
        }
        all_findings.extend(findings);
    }

    println!("\n{} total findings", all_findings.len());

    // Propose refactors
    println!("\n=== REFACTOR PROPOSALS ===");
    let refactor = eng.propose(
        "extract-sim-systems",
        "Sim domains built but not wired to shared world",
        "each domain -> self-contained module",
        8,
        Some("Facade/Adapter"),
    );
    println!("'{}': {} -> {} ({}h)", refactor.name, refactor.motivation, refactor.seam, refactor.estimated_effort);

    let refactor2 = eng.propose(
        "unify-component-access",
        "Component access is scattered, inconsistent",
        "World::get<T>() -> centralized accessor",
        4,
        Some("Repository"),
    );
    println!("'{}': {} -> {} ({}h)", refactor2.name, refactor2.motivation, refactor2.seam, refactor2.estimated_effort);

    println!("\n=== REVIEW COMPLETE ===");
}
