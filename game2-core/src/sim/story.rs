//! Story recognition: detect emergent narratives from simulation events.
//! From emergent narrative lit review: the system must recognize stories, not just produce events.

use crate::engine::ecs::Entity;

/// A detected emergent story
#[derive(Debug)]
pub struct Story {
    pub title: String,
    pub events: Vec<StoryEvent>,
    pub theme: StoryTheme,
    pub participants: Vec<Entity>,
    pub intensity: f32,   // narrative weight
}

#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub tick: u64,
    pub kind: EventKind,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    Birth,
    Death,
    Illness,
    Recovery,
    Bonding,
    Betrayal,
    Recruitment,
    Rejection,
    Alliance,
    Conflict,
    Discovery,
    Achievement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StoryTheme {
    Rise,        // someone gains influence/power
    Fall,        // someone loses everything
    Romance,     // bond forms
    Redemption,  // bad -> good
    Conflict,    // faction war
    Survival,    // endures hardship
    Community,   // organizing works
}

/// Detect a story from a sequence of events
pub fn detect_story(events: &[StoryEvent]) -> Option<Story> {
    if events.len() < 3 { return None; }
    // Pattern: illness -> recovery -> bonding = survival story
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();
    let theme = if kinds.contains(&EventKind::Illness) && kinds.contains(&EventKind::Recovery) {
        StoryTheme::Survival
    } else if kinds.contains(&EventKind::Recruitment) && kinds.contains(&EventKind::Achievement) {
        StoryTheme::Rise
    } else if kinds.contains(&EventKind::Bonding) {
        StoryTheme::Romance
    } else if kinds.contains(&EventKind::Betrayal) {
        StoryTheme::Fall
    } else {
        StoryTheme::Community
    };
    Some(Story {
        title: format!("A {:?} Story", theme),
        events: events.to_vec(),
        theme,
        participants: vec![],
        intensity: events.len() as f32 / 10.0,
    })
}

/// Surface an emergent story for the player
pub fn surface_story(story: &Story) -> String {
    let mut s = format!("**{}**\n", story.title);
    for e in &story.events {
        s += &format!("  t{}: {}\n", e.tick, e.description);
    }
    s += &format!("  theme: {:?}\n", story.theme);
    s
}
