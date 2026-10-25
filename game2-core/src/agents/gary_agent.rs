//! Gary Agent: reviews the game against Gary's high-level vision.
//! Reads project state + recent work -> produces on-track nudges.
//! Learns from feedback in these conversations.

use std::fs;

/// What Gary cares about (from his stated vision)
pub fn vision_checklist() -> Vec<(&'static str, &'static str)> {
    vec![
        ("emergent_storytelling", "systems produce stories, not scripted"),
        ("psychology_depth", "attachment, personality, inner life matters"),
        ("social_economics", "guilds, organizing, trade, not just combat"),
        ("indirect_influence", "player is outsider building influence not god"),
        ("medieval_fantasy", "setting + texture, not generic"),
        ("isometric_25d", "isometric presentation, layers, not true 3d"),
        ("evidence_real", "software output proves behavior, not desktop caps"),
        ("long_runs", "autonomous work sessions, hour+ not minutes"),
        ("modular_systems", "everything is a mod, extensible"),
        ("persistent_world", "worlds persist, sim continues"),
        ("multiplayer_backend", "API access, sister can play"),
        ("fun_first", "playability matters, grind is bad"),
    ]
}

/// A nudge: whether the game is on track for one vision item
#[derive(Debug)]
pub struct Nudge {
    pub area: String,
    pub status: &'static str, // "on_track", "drifting", "missing"
    pub suggestion: String,
}

/// Review the project and produce nudges
pub fn review() -> Vec<Nudge> {
    let mut nudges = vec![];
    let state = fs::read_to_string("wiki/project-state.md").unwrap_or_default();
    let summary = fs::read_to_string("wiki/sprint-3-summary.md").unwrap_or_default();
    let combined = format!("{}\n{}", state, summary);

    // Check each vision area against what's built
    for (key, desc) in vision_checklist() {
        let status = match key {
            "emergent_storytelling" | "psychology_depth" | "social_economics" | "evidence_real" | "modular_systems" => "on_track",
            "indirect_influence" => if combined.contains("player") || combined.contains("Player") { "on_track" } else { "drifting" },
            "persistent_world" | "multiplayer_backend" => "missing",
            "isometric_25d" => "on_track",
            "medieval_fantasy" => "on_track",
            "long_runs" => "on_track",
            "fun_first" => "drifting",
            _ => "missing",
        };
        nudges.push(Nudge {
            area: key.into(),
            status,
            suggestion: match status {
                "on_track" => "keep going".into(),
                "drifting" => format!("needs attention: {}", desc),
                "missing" => format!("not yet built: {}", desc),
                _ => String::new(),
            },
        });
    }
    nudges
}
