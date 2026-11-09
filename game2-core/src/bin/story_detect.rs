//! Story detection: emergent narratives surface from sim events.
//! Illness->recovery->bonding = Survival story. Recruitment->achievement = Rise.

use game2_core::sim::story::{StoryEvent, EventKind, detect_story, surface_story};

fn main() {
    game2_core::infra::crash::install();
    println!("=== STORY RECOGNITION ===\n");

    // Story 1: Survival - illness -> recovery -> bonding
    let events1 = vec![
        StoryEvent { tick: 10, kind: EventKind::Illness, description: "Aldric falls ill".into() },
        StoryEvent { tick: 25, kind: EventKind::Recovery, description: "Apothecary treats him".into() },
        StoryEvent { tick: 40, kind: EventKind::Bonding, description: "Aldric and Eliza grow close".into() },
    ];
    if let Some(story) = detect_story(&events1) {
        println!("Detected story:");
        println!("{}", surface_story(&story));
    }

    // Story 2: Rise - recruitment -> achievement
    let events2 = vec![
        StoryEvent { tick: 50, kind: EventKind::Recruitment, description: "Bram joins the guild".into() },
        StoryEvent { tick: 75, kind: EventKind::Achievement, description: "Guild wins contract".into() },
        StoryEvent { tick: 90, kind: EventKind::Achievement, description: "Bram becomes leader".into() },
    ];
    if let Some(story) = detect_story(&events2) {
        println!("Detected story:");
        println!("{}", surface_story(&story));
    }

    // Story 3: Fall - betrayal
    let events3 = vec![
        StoryEvent { tick: 100, kind: EventKind::Alliance, description: "Corvin allies with Aldric".into() },
        StoryEvent { tick: 120, kind: EventKind::Betrayal, description: "Corvin betrays the guild".into() },
        StoryEvent { tick: 140, kind: EventKind::Conflict, description: "Guild splits in two".into() },
    ];
    if let Some(story) = detect_story(&events3) {
        println!("Detected story:");
        println!("{}", surface_story(&story));
    }

    println!("=== STORY RECOGNITION COMPLETE ===");
}
