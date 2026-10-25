//! B3: FunInspector scores playtest output against the fun rubric.

use game2_core::agents::fun_inspector::{FunInspector, FunScores};

fn main() {
    println!("=== FUN INSPECTOR SCORING ===\n");

    let inspector = FunInspector::new();

    // Score the full session playtest
    let scores = FunScores {
        tension: 3,     // moderate - hunger cycles, weather events
        progress: 4,    // org grows, tech discovered, influence climbs
        surprise: 3,    // weather + guild + illness events
        care: 4,        // villagers have names, attachment styles, pets
        decisions: 4,   // player talk/recruit/invest choices matter
        retention: 3,   // decent loop but needs win condition
    };

    let assessment = inspector.assess("full_session_300t", scores,
        vec![
            "Recruiting felt rewarding (trust gate)".into(),
            "Dog bond growing felt caring".into(),
            "Guild forming emergently was surprising".into(),
        ],
        vec![
            "Hunger cycle feels grindy (always seeking)".into(),
            "No clear win state yet".into(),
        ]);

    println!("Session: {}", assessment.session_id);
    println!("Scores: tension={} progress={} surprise={} care={} decisions={} retention={}",
        assessment.scores.tension, assessment.scores.progress, assessment.scores.surprise,
        assessment.scores.care, assessment.scores.decisions, assessment.scores.retention);
    println!("Overall: {:.1}/5", assessment.overall);
    println!("Verdict: {}", assessment.verdict);
    println!("\nFun moments:");
    for m in &assessment.fun_moments { println!("  + {}", m); }
    println!("Grind moments:");
    for m in &assessment.grind_moments { println!("  - {}", m); }

    // Check for red flags
    let sim_log: Vec<(u64, f32, bool)> = (0..20).map(|i| (i, 50.0, i % 3 == 0)).collect();
    let flags = inspector.detect_red_flags(&sim_log);
    if !flags.is_empty() {
        println!("\nRed flags: {:?}", flags);
    }
    println!("\n=== SCORING COMPLETE ===");
}
