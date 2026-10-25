//! G5: Retrospective agent analyzes sprint outputs and files findings.

use game2_core::agents::retrospective_agent::RetrospectiveAgent;

fn main() {
    println!("=== RETROSPECTIVE AGENT ===\n");

    let agent = RetrospectiveAgent::new();

    // Analyze the sprint log
    let sprint_log = "Sprint 4: unified game loop, agent fleet live, player layer.
        Issues: exe blocks, test debt, sim wiring incomplete.
        Wins: evidence pipeline, real Ollama, emergent behavior.";

    let findings = agent.analyze(sprint_log);
    println!("Findings:");
    for f in &findings {
        println!("  [{}] {}: {}", f.category, f.description, f.evidence);
    }

    // Propose actions
    let actions = vec![
        agent.propose_action("Implementation Engineer", "Fix filesystem tests with temp dirs", "exe blocks recurring"),
        agent.propose_action("Manager Bot", "Schedule agent fleet tasks next sprint", "fleet not yet doing real work"),
        agent.propose_action("Fun Inspector", "Run rubric on full session", "playability not yet scored"),
    ];
    println!("\nAction items proposed:");
    for a in &actions {
        println!("  -> {}: {} ({})", a.target_agent, a.proposal, a.motivation);
    }
    println!("\n=== RETROSPECTIVE COMPLETE ===");
}
