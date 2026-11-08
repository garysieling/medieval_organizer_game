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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EpisodeKind {
    Anxiety,   // distress met with comfort or absence
    Warmth,    // positive bonding moment
    Rejection, // bid for comfort rebuffed
    Loss,      // attachment figure absent/removed
    Remembrance, // the dead brought to mind - a cairn does it, a keepsake
}

/// A story two folk carry between them: they were both there, and the
/// telling of it warms the pair a little - if they ever get round to it
#[derive(Debug, Clone)]
pub struct SharedBit {
    /// The other half of the story
    pub partner: Entity,
    /// What they both saw - the episode's subject and its kind
    pub subject: Entity,
    pub kind: EpisodeKind,
    /// Whether the pair have told it to each other yet
    pub told: bool,
    /// The day it formed
    pub born: u64,
}

/// The bits a villager carries: shared memories waiting on a telling
#[derive(Debug, Default)]
pub struct SharedBits {
    pub bits: Vec<SharedBit>,
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

/// A seeded Big5 profile: five draws off the entity's index so the
/// same villager reads the same temperament every run
pub fn personality_from_seed(seed: u64) -> Personality {
    let draw = |n: u64| {
        let mut x = seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^= x >> 31;
        (x % 1000) as f32 / 1000.0
    };
    Personality { openness: draw(1), neuroticism: draw(2),
        extraversion: draw(3), agreeableness: draw(4), conscientiousness: draw(5) }
}

/// The strongest read a Big5 profile gives: its dominant pole, in a
/// word the village would use
pub fn disposition_word(p: &Personality) -> &'static str {
    let (mut best, mut word) = (0.0f32, "even-tempered");
    for (v, w) in [(p.openness, "of a curious bent"), (p.neuroticism, "easily ruffled"),
        (p.extraversion, "full of talk"), (p.agreeableness, "soft-hearted"),
        (p.conscientiousness, "keeps good order")] {
        if v > best { best = v; word = w; }
    }
    word
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

/// Record an episode: count it toward the emerging style and update the
/// agent's bond with whoever the episode was with.
pub fn record_episode(world: &mut World, agent: Entity, episode: Episode) {
    let bond_update = match episode.kind {
        EpisodeKind::Warmth => 0.15,
        EpisodeKind::Anxiety => -0.1,
        EpisodeKind::Rejection => -0.2,
        EpisodeKind::Loss => -0.3,
        EpisodeKind::Remembrance => -0.2,
    };
    // the style they've grown reads each beat: a secure heart warms easy,
    // an anxious one takes every blow harder, an avoidant one warms slow
    let style = world.get_component::<AttachmentHistory>(agent).map(|h| infer_style(h));
    let bond_update = match style {
        Some(AttachmentStyle::Secure) if bond_update > 0.0 => bond_update * 1.3,
        Some(AttachmentStyle::Anxious) if bond_update < 0.0 => bond_update * 1.4,
        Some(AttachmentStyle::Avoidant) if bond_update > 0.0 => bond_update * 0.6,
        _ => bond_update,
    };
    let target = episode.with;
    let is_loss = matches!(episode.kind, EpisodeKind::Loss);
    if let Some(hist) = world.get_component_mut::<AttachmentHistory>(agent) {
        match episode.kind {
            EpisodeKind::Warmth => hist.warmth_episodes += 1,
            _ => hist.anxiety_episodes += 1,
        }
        hist.episodes.push(episode);
    }
    // One primary attachment figure per agent: shift it if this episode was
    // with them, or form a first bond if the agent has none yet.
    match world.get_component_mut::<Bond>(agent) {
        Some(b) if b.target == target => {
            b.security = (b.security + bond_update).clamp(0.0, 1.0);
        }
        Some(b) => {
            // a heart can move: weigh the episodes each tie has earned;
            // the first passerby holds it only until another clearly
            // earns it more
            let cur = b.target;
            let _ = b;
            let score = |t: Entity| -> f32 {
                // the heart's style weighs each beat as it did when the
                // bond was growing, or a switched bond lands wrong
                let weight = |k: &EpisodeKind| {
                    let w = match k {
                        EpisodeKind::Warmth => 0.15, EpisodeKind::Anxiety => -0.1,
                        EpisodeKind::Rejection => -0.2, EpisodeKind::Loss => -0.3,
                        EpisodeKind::Remembrance => -0.2 };
                    match style {
                        Some(AttachmentStyle::Secure) if w > 0.0 => w * 1.3,
                        Some(AttachmentStyle::Anxious) if w < 0.0 => w * 1.4,
                        Some(AttachmentStyle::Avoidant) if w > 0.0 => w * 0.6,
                        _ => w,
                    }
                };
                world.get_component::<AttachmentHistory>(agent).map(|h| h.episodes.iter()
                    .filter(|e| e.with == t).map(|e| weight(&e.kind)).sum()).unwrap_or(0.0)
            };
            let (new_s, old_s) = (score(target), score(cur));
            if new_s > old_s + 0.3 {
                if let Some(b) = world.get_component_mut::<Bond>(agent) {
                    b.target = target;
                    b.security = (0.5 + new_s).clamp(0.0, 1.0);
                }
            }
        }
        None if is_loss => {} // losing someone doesn't form a first bond with them
        None => {
            world.add_component(agent, Bond { target, security: (0.5 + bond_update).clamp(0.0, 1.0) });
        }
    }
}

/// Marks someone whose death has already been grieved
#[derive(Debug)]
pub struct Mourned;

/// Relationship strength at which someone grieves a death
pub const GRIEF_TIE: f32 = 0.3;

/// Demo-only system: when someone dies (villager::is_dead - health 0, or marked Dead
/// as the windowed game does after removing Health), everyone still alive
/// who was attached to them - their primary bond, or a relationship of at
/// least GRIEF_TIE - records a Loss episode, shaking their attachment
/// security. Each death is grieved once.
pub struct GriefSystem;
impl crate::engine::ecs::System for GriefSystem {
    fn run(&mut self, world: &mut World) {
        use crate::game::relationships::{find_relationship, Relationship};
        use crate::game::villager::{is_dead, Dead};
        use crate::sim::illness::Health;
        let dead_now = |w: &World, e: Entity| is_dead(w, e);
        let mut candidates = world.query::<Health>();
        candidates.extend(world.query::<Dead>());
        candidates.sort_unstable();
        candidates.dedup();
        let newly_dead: Vec<Entity> = candidates.into_iter()
            .filter(|&e| dead_now(world, e) && world.get_component::<Mourned>(e).is_none())
            .collect();
        for dead in newly_dead {
            world.add_component(dead, Mourned);
            let mourners: Vec<Entity> = world.query::<AttachmentHistory>().into_iter()
                .filter(|&m| m != dead && !dead_now(world, m))
                .filter(|&m| {
                    let bonded = world.get_component::<Bond>(m).map(|b| b.target == dead).unwrap_or(false);
                    let close = find_relationship(world, m, dead)
                        .and_then(|r| world.get_component::<Relationship>(r))
                        .map(|r| r.strength >= GRIEF_TIE).unwrap_or(false);
                    bonded || close
                })
                .collect();
            for m in mourners {
                record_episode(world, m, Episode { with: dead, kind: EpisodeKind::Loss });
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
    fn test_record_episode_shifts_style() {
        let mut w = World::new();
        let child = w.create_entity();
        let carer = w.create_entity();
        w.add_component(child, AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
        for _ in 0..5 {
            record_episode(&mut w, child, Episode { with: carer, kind: EpisodeKind::Rejection });
        }
        let h = w.get_component::<AttachmentHistory>(child).unwrap();
        assert_eq!(h.anxiety_episodes, 5);
        assert_eq!(infer_style(h), AttachmentStyle::Disorganized);
        for _ in 0..20 {
            record_episode(&mut w, child, Episode { with: carer, kind: EpisodeKind::Warmth });
        }
        let h = w.get_component::<AttachmentHistory>(child).unwrap();
        assert_eq!(h.warmth_episodes, 20);
        assert_eq!(infer_style(h), AttachmentStyle::Secure);
    }

    #[test]
    fn test_record_episode_updates_bond() {
        let mut w = World::new();
        let child = w.create_entity();
        let carer = w.create_entity();
        record_episode(&mut w, child, Episode { with: carer, kind: EpisodeKind::Warmth });
        let first = w.get_component::<Bond>(child).unwrap().security;
        record_episode(&mut w, child, Episode { with: carer, kind: EpisodeKind::Warmth });
        let b = w.get_component::<Bond>(child).unwrap();
        assert_eq!(b.target, carer);
        assert!(b.security > first);
    }

    #[test]
    fn test_anxious_from_anxiety_episodes() {
        // security 4/10 = 0.4 (>0.3) and anxious > warm -> Anxious
        let hist = AttachmentHistory {
            anxiety_episodes: 6, warmth_episodes: 4, episodes: vec![],
        };
        assert_eq!(infer_style(&hist), AttachmentStyle::Anxious);
    }

    #[test]
    fn test_death_is_grieved_once_by_those_attached() {
        use crate::engine::ecs::System;
        use crate::game::relationships::{Relationship, RelKind};
        use crate::sim::illness::Health;
        let mut w = World::new();
        let hist = || AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] };
        let elder = w.create_entity();
        w.add_component(elder, Health { condition: 0.5, immunity: 0.5 });
        let child = w.create_entity(); // primary bond with the elder
        w.add_component(child, hist());
        w.add_component(child, Bond { target: elder, security: 0.8 });
        let friend = w.create_entity(); // close relationship, bonded elsewhere
        w.add_component(friend, hist());
        let r = w.create_entity();
        w.add_component(r, Relationship { from: friend, other: elder, kind: RelKind::Friend, strength: 0.6, interactions: 9 });
        let stranger = w.create_entity();
        w.add_component(stranger, hist());

        GriefSystem.run(&mut w);
        assert_eq!(w.get_component::<AttachmentHistory>(child).unwrap().anxiety_episodes, 0, "no one has died yet");

        w.get_component_mut::<Health>(elder).unwrap().condition = 0.0;
        GriefSystem.run(&mut w);
        GriefSystem.run(&mut w); // grieved once, not every tick
        assert_eq!(w.get_component::<AttachmentHistory>(child).unwrap().anxiety_episodes, 1);
        assert!(w.get_component::<Bond>(child).unwrap().security < 0.8, "losing their attachment figure shakes them");
        assert_eq!(w.get_component::<AttachmentHistory>(friend).unwrap().anxiety_episodes, 1);
        assert!(w.get_component::<Bond>(friend).is_none(), "grief doesn't bond the friend to the dead");
        assert_eq!(w.get_component::<AttachmentHistory>(stranger).unwrap().anxiety_episodes, 0);
    }

    #[test]
    fn test_death_marked_without_health_is_grieved() {
        // the windowed game removes Health on death and marks the villager Dead
        use crate::engine::ecs::System;
        let mut w = World::new();
        let elder = w.create_entity();
        let child = w.create_entity();
        w.add_component(child, AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
        w.add_component(child, Bond { target: elder, security: 0.8 });
        w.add_component(elder, crate::game::villager::Dead);
        GriefSystem.run(&mut w);
        assert_eq!(w.get_component::<AttachmentHistory>(child).unwrap().anxiety_episodes, 1);
    }
}
