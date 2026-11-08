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
    /// a feud put to rest - not a sickness healed
    Peace,
    Bonding,
    Betrayal,
    Recruitment,
    Rejection,
    Alliance,
    Conflict,
    Discovery,
    Achievement,
    /// the village's belly is empty - famine's first domino
    Hardship,
    /// the grumbling turns to anger - grievance's pressure on the street
    Unrest,
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
    /// the village itself is the protagonist: famine, pestilence, the
    /// dying - the cascade the commons lived through
    Calamity,
}

/// Detect a story from a sequence of events
pub fn detect_story(events: &[StoryEvent]) -> Option<Story> {
    if events.len() < 3 { return None; }
    // Pattern: illness -> recovery -> bonding = survival story
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();
    let theme = if kinds.contains(&EventKind::Illness) && kinds.contains(&EventKind::Recovery) {
        StoryTheme::Survival
    } else if {
        let last_fight = kinds.iter().rposition(|k| *k == EventKind::Conflict);
        let last_peace = kinds.iter().rposition(|k| *k == EventKind::Recovery);
        // redemption needs the peace to come after the fighting -
        // a feud reignited since is still a tale of bad blood
        matches!((last_fight, last_peace), (Some(c), Some(r)) if r > c)
    } {
        StoryTheme::Redemption
    } else if kinds.iter().filter(|k| **k == EventKind::Conflict).count() >= 2 {
        StoryTheme::Conflict
    } else if kinds.contains(&EventKind::Death) || kinds.contains(&EventKind::Betrayal) {
        StoryTheme::Fall
    } else if kinds.contains(&EventKind::Recruitment) && kinds.contains(&EventKind::Achievement) {
        StoryTheme::Rise
    } else if kinds.contains(&EventKind::Bonding) {
        StoryTheme::Romance
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

/// Detect a village-scale calamity: the dominos of a crisis falling in
/// order - hardship first, sickness riding it, then the dying or the
/// anger. The events are the census's crossings, so a bare winter that
/// never sickens nobody and a fever with full granaries are no tale.
/// The story is named for what led it: hunger's year, the pestilence,
/// or the great dying when the dead mount.
pub fn detect_cascade(events: &[StoryEvent]) -> Option<Story> {
    if events.len() < 3 { return None; }
    let at = |k: EventKind| events.iter().position(|e| e.kind == k);
    let (h, i) = (at(EventKind::Hardship), at(EventKind::Illness));
    // the want must come before the fever - or the fever stand alone;
    // a want with no sickness can still close on the dead or the anger
    let ordered = match (h, i) {
        (Some(h), Some(i)) => h < i,
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    };
    if !ordered { return None; }
    let marks = [h, i].into_iter().flatten();
    let (first, last) = (marks.clone().min().unwrap(), marks.max().unwrap());
    // and it must end in loss or anger after the dominos have fallen
    let closes = events.iter().enumerate().skip(last + 1)
        .any(|(_, e)| matches!(e.kind, EventKind::Death | EventKind::Unrest));
    if !closes { return None; }
    let window = &events[first..];
    let deaths = window.iter().filter(|e| e.kind == EventKind::Death).count();
    let title = if deaths >= 2 { "the great dying" }
        else if h.is_some() { "the hunger year" }
        else { "the pestilence" };
    Some(Story {
        title: title.into(),
        events: events.to_vec(),
        theme: StoryTheme::Calamity,
        participants: vec![],
        intensity: window.len() as f32 / 10.0,
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
