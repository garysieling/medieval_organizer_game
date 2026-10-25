//! Music: techno-style layer evolution from world state.
//! From backlog: "music system that evolves as the world changes... slowly
//! adding/dropping layers over time" like techno. Initial: recommend a mood
//! to look for on Spotify; eventually procedural.

use crate::engine::ecs::{Entity, World};

/// Current musical state - which layers are active
#[derive(Debug)]
pub struct MusicState {
    pub layers: Vec<Layer>,
    pub mood: Mood,
    pub intensity: f32,  // 0-1 overall energy
}

/// A music layer (like a techno track element)
#[derive(Debug)]
pub struct Layer {
    pub name: String,       // kick, bass, synth, pad, percussion
    pub active: bool,
    pub intensity: f32,     // 0-1 volume/presence
    pub added_at: u64,      // tick when added
}

/// Moods that map to world states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Calm,        // peaceful village
    Growing,     // expanding, productive
    Tense,       // danger, conflict approaching
    Triumphant,  // victory, achievement
    Somber,      // loss, hardship
    Mysterious,  // discovery, magic, unknown
}

/// Map world state to a mood (simple heuristics now, richer later)
pub fn infer_mood(world_metrics: &WorldMetrics) -> Mood {
    if world_metrics.conflict > 0.5 { Mood::Tense }
    else if world_metrics.growth > 0.6 { Mood::Growing }
    else if world_metrics.loss > 0.4 { Mood::Somber }
    else if world_metrics.discovery > 0.5 { Mood::Mysterious }
    else if world_metrics.prosperity > 0.7 { Mood::Triumphant }
    else { Mood::Calm }
}

/// Simple metrics that drive music
#[derive(Debug)]
pub struct WorldMetrics {
    pub growth: f32,
    pub conflict: f32,
    pub loss: f32,
    pub discovery: f32,
    pub prosperity: f32,
}

/// Recommend a Spotify mood (initial version - "be silent or recommend a mood")
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

/// System: evolve layers based on world state (techno-style add/drop)
pub struct MusicEvolutionSystem;
impl crate::engine::ecs::System for MusicEvolutionSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<MusicState>() {
            if let Some(m) = world.get_component_mut::<MusicState>(e) {
                // Slowly adjust layer intensities toward target
                // (real impl would compare to world metrics)
                let _ = m;
            }
        }
    }
}

/// Spawn music state
pub fn spawn_music(world: &mut World) -> Entity {
    let e = world.create_entity();
    world.add_component(e, MusicState {
        layers: vec![
            Layer { name: "kick".into(), active: true, intensity: 0.5, added_at: 0 },
            Layer { name: "bass".into(), active: false, intensity: 0.0, added_at: 0 },
            Layer { name: "pad".into(), active: true, intensity: 0.3, added_at: 0 },
        ],
        mood: Mood::Calm,
        intensity: 0.3,
    });
    e
}
