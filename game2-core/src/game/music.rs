//! Music: techno-style layer evolution from world state.
//! From backlog: layers add/drop over time like techno. Initial: recommend a mood.

use crate::engine::ecs::{Entity, World};

/// A music layer (like a techno track element)
#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub active: bool,
    pub intensity: f32,
    pub added_at: u64,
}

/// Moods mapped to world states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Calm, Growing, Tense, Triumphant, Somber, Mysterious,
}

/// Musical state - which layers are active
#[derive(Debug)]
pub struct MusicState {
    pub layers: Vec<Layer>,
    pub mood: Mood,
    pub intensity: f32,
    pub tick: u64,
    /// The mood the world is currently pointing toward, and since which
    /// tick; it becomes `mood` only once it has held for MOOD_HOLD_TICKS
    pub pending_mood: Mood,
    pub pending_since: u64,
}

/// Ticks a new mood must persist before the music commits to it - a techno
/// build before the drop, so metrics hovering at a threshold don't flip
/// the track back and forth every tick
pub const MOOD_HOLD_TICKS: u64 = 8;

/// Metrics that drive music
#[derive(Debug)]
pub struct WorldMetrics {
    pub growth: f32,
    pub conflict: f32,
    pub loss: f32,
    pub discovery: f32,
    pub prosperity: f32,
}

/// Infer mood from world metrics
pub fn infer_mood(m: &WorldMetrics) -> Mood {
    if m.conflict > 0.5 { Mood::Tense }
    else if m.growth > 0.6 { Mood::Growing }
    else if m.loss > 0.4 { Mood::Somber }
    else if m.discovery > 0.5 { Mood::Mysterious }
    else if m.prosperity > 0.7 { Mood::Triumphant }
    else { Mood::Calm }
}

/// Spotify mood recommendation (initial version)
pub fn recommend_mood(mood: Mood) -> &'static str {
    match mood {
        Mood::Calm => "ambient folk / medieval chill",
        Mood::Growing => "uplifting instrumental / building",
        Mood::Tense => "dark ambient / suspense",
        Mood::Triumphant => "epic orchestral / heroic",
        Mood::Somber => "melancholic strings / mournful",
        Mood::Mysterious => "ethereal / mystical ambience",
    }
}

/// Target layers for each mood (which techno elements play)
fn target_layers(mood: Mood) -> Vec<(&'static str, f32)> {
    match mood {
        Mood::Calm => vec![("pad", 0.4), ("bass", 0.2)],
        Mood::Growing => vec![("pad", 0.5), ("bass", 0.4), ("kick", 0.3), ("percussion", 0.2)],
        Mood::Tense => vec![("kick", 0.6), ("bass", 0.7), ("synth", 0.4), ("percussion", 0.5)],
        Mood::Triumphant => vec![("kick", 0.5), ("bass", 0.5), ("synth", 0.7), ("pad", 0.6), ("percussion", 0.4)],
        Mood::Somber => vec![("pad", 0.3), ("synth", 0.2)],
        Mood::Mysterious => vec![("pad", 0.4), ("synth", 0.5), ("percussion", 0.3)],
    }
}

/// Evolve layers toward target (techno-style: layers fade in/out over ticks)
pub fn evolve(state: &mut MusicState, metrics: &WorldMetrics) {
    let target_mood = infer_mood(metrics);
    if target_mood != state.pending_mood {
        state.pending_mood = target_mood;
        state.pending_since = state.tick;
    }
    if state.pending_mood != state.mood && state.tick - state.pending_since + 1 >= MOOD_HOLD_TICKS {
        state.mood = state.pending_mood;
    }
    let targets = target_layers(state.mood);

    // For each layer, move intensity toward target (techno-style gradual change)
    for layer in state.layers.iter_mut() {
        let target = targets.iter().find(|(n, _)| n == &layer.name.as_str()).map(|(_, i)| *i).unwrap_or(0.0);
        let diff = target - layer.intensity;
        layer.intensity = (layer.intensity + diff * 0.1).clamp(0.0, 1.0);
        let was_active = layer.active;
        layer.active = layer.intensity > 0.05;
        if layer.active && !was_active { layer.added_at = state.tick; }
    }

    // Overall intensity from prosperity+conflict
    state.intensity = ((metrics.prosperity + metrics.conflict) / 2.0).clamp(0.0, 1.0);
    state.tick += 1;
}

/// Describe current music state
pub fn describe(state: &MusicState) -> String {
    let active: Vec<String> = state.layers.iter().filter(|l| l.active).map(|l| format!("{}({:.1})", l.name, l.intensity)).collect();
    format!("mood={:?} intensity={:.2} layers={}", state.mood, state.intensity, active.join(","))
}

/// Spawn music state
pub fn spawn_music(world: &mut World) -> Entity {
    let e = world.create_entity();
    world.add_component(e, MusicState {
        layers: vec![
            Layer { name: "kick".into(), active: false, intensity: 0.0, added_at: 0 },
            Layer { name: "bass".into(), active: false, intensity: 0.0, added_at: 0 },
            Layer { name: "synth".into(), active: false, intensity: 0.0, added_at: 0 },
            Layer { name: "pad".into(), active: true, intensity: 0.3, added_at: 0 },
            Layer { name: "percussion".into(), active: false, intensity: 0.0, added_at: 0 },
        ],
        mood: Mood::Calm,
        intensity: 0.3,
        tick: 0,
        pending_mood: Mood::Calm,
        pending_since: 0,
    });
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(conflict: f32) -> WorldMetrics {
        WorldMetrics { growth: 0.0, conflict, loss: 0.0, discovery: 0.0, prosperity: 0.0 }
    }

    fn music() -> (World, Entity) {
        let mut w = World::new();
        let m = spawn_music(&mut w);
        (w, m)
    }

    #[test]
    fn test_mood_commits_after_holding() {
        let (mut w, m) = music();
        let st = w.get_component_mut::<MusicState>(m).unwrap();
        for _ in 0..MOOD_HOLD_TICKS - 1 { evolve(st, &metrics(0.8)); }
        assert_eq!(st.mood, Mood::Calm, "still building");
        evolve(st, &metrics(0.8));
        assert_eq!(st.mood, Mood::Tense);
    }

    #[test]
    fn test_flickering_metrics_do_not_flip_the_mood() {
        let (mut w, m) = music();
        let st = w.get_component_mut::<MusicState>(m).unwrap();
        for i in 0..40 { evolve(st, &metrics(if i % 2 == 0 { 0.51 } else { 0.49 })); }
        assert_eq!(st.mood, Mood::Calm, "conflict hovering at the threshold shouldn't flip the track");
    }

    #[test]
    fn test_layers_record_when_they_come_in() {
        let (mut w, m) = music();
        let st = w.get_component_mut::<MusicState>(m).unwrap();
        for _ in 0..20 { evolve(st, &metrics(0.8)); }
        let kick = st.layers.iter().find(|l| l.name == "kick").unwrap();
        assert!(kick.active);
        assert!(kick.added_at >= MOOD_HOLD_TICKS - 1, "kick came in after the drop, at {}", kick.added_at);
    }
}
