//! Psychology: attachment emergence via anxiety/warmth episodes.
//! Based on Dean Petters' attachment simulation research.
//! Attachment styles EMERGE from learning, not assigned.

use crate::engine::ecs::{Entity, World};

/// Attachment bond between two entities
#[derive(Debug)]
pub struct Bond {
    pub target: Entity,
    pub security: f32,  // 0-1, grows from positive experiences
}

/// Attachment style emerges from episodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachmentStyle {
    Secure,
    Anxious,
    Avoidant,
    Disorganized,
}

/// Tracks anxiety/warmth episodes that shape attachment
#[derive(Debug)]
pub struct AttachmentHistory {
    pub anxiety_episodes: u32,
    pub warmth_episodes: u32,
    pub episodes: Vec<Episode>,
}

#[derive(Debug)]
pub struct Episode {
    pub with: Entity,
    pub kind: EpisodeKind,
}

#[derive(Debug)]
pub enum EpisodeKind {
    Anxiety,   // distress met with comfort or absence
    Warmth,    // positive bonding moment
    Rejection, // bid for comfort rebuffed
    Loss,      // attachment figure absent/removed
}

/// Personality traits (simplified Big5-inspired)
#[derive(Debug)]
pub struct Personality {
    pub openness: f32,
    pub neuroticism: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub conscientiousness: f32,
}

/// Emergent attachment style from history
pub fn infer_style(hist: &AttachmentHistory) -> AttachmentStyle {
    let anxious = hist.anxiety_episodes as f32;
    let warm = hist.warmth_episodes as f32;
    let total = (anxious + warm).max(1.0);
    let security = warm / total;

    if security > 0.6 { AttachmentStyle::Secure }
    else if security > 0.3 {
        if anxious > warm { AttachmentStyle::Anxious }
        else { AttachmentStyle::Avoidant }
    } else { AttachmentStyle::Disorganized }
}

/// Record an episode and update bond security
pub fn record_episode(world: &mut World, agent: Entity, episode: Episode) {
    let bond_update = match episode.kind {
        EpisodeKind::Warmth => 0.15,
        EpisodeKind::Anxiety => -0.1,
        EpisodeKind::Rejection => -0.2,
        EpisodeKind::Loss => -0.3,
    };
    if let Some(hist) = world.get_component_mut::<AttachmentHistory>(agent) {
        hist.episodes.push(episode);
    }
    // Update bond with episode target
    // (simplified: would find Bond component pointing at target)
    let _ = bond_update;
}

/// System: attachment episodes accumulate; styles emerge
pub struct AttachmentSystem;
impl crate::engine::ecs::System for AttachmentSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<AttachmentHistory>() {
            if let Some(hist) = world.get_component::<AttachmentHistory>(e) {
                let style = infer_style(hist);
                let _ = style; // would store as component
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_style_from_warmth() {
        let hist = AttachmentHistory {
            anxiety_episodes: 1, warmth_episodes: 9, episodes: vec![],
        };
        assert_eq!(infer_style(&hist), AttachmentStyle::Secure);
    }

    #[test]
    fn test_anxious_from_anxiety_episodes() {
        let hist = AttachmentHistory {
            anxiety_episodes: 8, warmth_episodes: 2, episodes: vec![],
        };
        assert_eq!(infer_style(&hist), AttachmentStyle::Anxious);
    }
}
