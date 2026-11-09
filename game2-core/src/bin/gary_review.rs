//! Gary Agent: vision-alignment review. Produces nudges from project state.
use game2_core::agents::gary_agent::review;

fn main() {
    game2_core::infra::crash::install();
    println!("=== GARY AGENT REVIEW ===\n");
    println!("Checking project against Gary's vision...\n");
    for nudge in review() {
        let icon = match nudge.status {
            "on_track" => "[OK]",
            "drifting" => "[~~]",
            "missing" => "[!!]",
            _ => "[?]",
        };
        println!("{} {}: {}", icon, nudge.area, nudge.suggestion);
    }
    println!("\n=== REVIEW COMPLETE ===");
}
