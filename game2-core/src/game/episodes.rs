//! Pair memory (E6-2, #1440): the episodes a villager keeps about each
//! seat's player - day, kind, concept tags, an authored importance -
//! and the Generative-Agents-style retrieval score (recency +
//! importance + concept overlap) that surfaces the top one or two at
//! talk time as `mem:*` facts and a callback line. Callback truth:
//! every `mem:*` fact traces to an episode that actually filed, so
//! nothing is remembered that didn't happen.

use crate::engine::ecs::{Entity, World};

/// The shapes of episode the design names (Part 6): a promise kept, a
/// meeting missed or stood at, an ask that came too soon, a meal shared
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemKind {
    KeptPromise,
    MissedMeeting,
    StoodWithUs,
    AskedTooSoon,
    SharedMeal,
    /// A safety test answered true (E5-4, #1436): their confidence
    /// stayed confidence - the record E5-5's ledger reads back
    KeptConfidence,
    /// Their test leaked to the reeve (E5-4): the sharpest cut a
    /// confidence takes
    LeakedConfidence,
    /// A risky laugh shared (E5-6, #1437): a barb at power taken up
    /// where it could have been reported - who laughs is aligned
    LaughedWithUs,
}

/// One remembered beat between a villager and a player
#[derive(Debug, Clone)]
pub struct PairEp {
    pub day: u64,
    pub kind: MemKind,
    /// The tags the retrieval overlap reads (`C_e` in the design)
    pub concepts: Vec<&'static str>,
}

/// A villager's memory of a player, seat-scoped: the component keys on
/// the seat's player entity, so seat A's history can't leak to seat B
/// (villager↔villager memory stays `SharedBits`' territory)
#[derive(Debug, Default)]
pub struct PairMemory {
    /// The seat's player this memory is of
    pub pe: Entity,
    pub eps: Vec<PairEp>,
}

/// Authored importance per kind (`importance(e)` in the design): a kept
/// promise weighs more than a shared meal; one table to tune
pub fn importance(kind: MemKind) -> f32 {
    match kind {
        MemKind::KeptPromise => 0.9,
        MemKind::StoodWithUs => 0.8,
        MemKind::MissedMeeting => 0.7,
        MemKind::AskedTooSoon => 0.6,
        MemKind::SharedMeal => 0.4,
        MemKind::KeptConfidence => 0.8,
        MemKind::LeakedConfidence => 0.85,
        MemKind::LaughedWithUs => 0.5,
    }
}

/// The `mem:*` fact a remembered episode sets on the villager's speaker
pub fn fact(kind: MemKind) -> &'static str {
    match kind {
        MemKind::KeptPromise => "mem:you_kept_promise",
        MemKind::MissedMeeting => "mem:you_missed_meeting",
        MemKind::StoodWithUs => "mem:you_stood_with_us",
        MemKind::AskedTooSoon => "mem:you_asked_too_soon",
        MemKind::SharedMeal => "mem:you_shared_meal",
        MemKind::KeptConfidence => "mem:you_kept_confidence",
        MemKind::LeakedConfidence => "mem:you_leaked_confidence",
        MemKind::LaughedWithUs => "mem:you_laughed_with_us",
    }
}

/// The callback line a surfaced memory earns at talk open
pub fn callback(kind: MemKind) -> &'static str {
    match kind {
        MemKind::KeptPromise => "You kept your word to me. That stays.",
        MemKind::MissedMeeting => "We looked for you at the meeting. You never came.",
        MemKind::StoodWithUs => "You stood with us when it counted. I don't forget that.",
        MemKind::AskedTooSoon => "You ask deep, too soon. I've learned to watch it.",
        MemKind::SharedMeal => "You shared your table with me once. Small things keep.",
        MemKind::KeptConfidence => "You kept a thing to yourself for me. I've not forgotten.",
        MemKind::LeakedConfidence => "I told you a thing and it traveled. I've learned what you're worth.",
        MemKind::LaughedWithUs => "We laughed where it wasn't safe to. I remember that.",
    }
}

/// File an episode on the villager's memory of this seat's player.
/// Cheap, capped - the retrieval score reads recency, so the tail can
/// go once the memory outgrows it
pub fn file(world: &mut World, villager: Entity, pe: Entity, kind: MemKind, day: u64, concepts: &[&'static str]) {
    let mut m = world.get_component::<PairMemory>(villager)
        .filter(|m| m.pe == pe)
        .map(|m| PairMemory { pe: m.pe, eps: m.eps.clone() })
        .unwrap_or(PairMemory { pe, eps: vec![] });
    m.eps.push(PairEp { day, kind, concepts: concepts.to_vec() });
    if m.eps.len() > 20 { m.eps.remove(0); }
    world.add_component(villager, m);
}

/// The retrieval score: `0.99^Δd + importance/10 + |C_e ∩ C_t|/|C_e|`.
/// `talk_tags` is the talk's concept tags so far - empty at the open,
/// so recency and importance carry it until the talk's own tags refine
/// it (#1424's areas, when they land)
pub fn score(ep: &PairEp, now: u64, talk_tags: &[&str]) -> f32 {
    let age = now.saturating_sub(ep.day) as f32;
    let recency = 0.99f32.powf(age);
    let overlap = if ep.concepts.is_empty() { 0.0 } else {
        ep.concepts.iter().filter(|c| talk_tags.contains(c)).count() as f32
            / ep.concepts.len() as f32
    };
    recency + importance(ep.kind) / 10.0 + overlap
}

/// The top `n` episodes by retrieval score - the ones the talk's open
/// turns into `mem:*` facts and a callback
pub fn top<'a>(eps: &'a [PairEp], now: u64, talk_tags: &[&str], n: usize) -> Vec<&'a PairEp> {
    let mut scored: Vec<(&PairEp, f32)> = eps.iter()
        .map(|e| (e, score(e, now, talk_tags))).collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(n).map(|(e, _)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(day: u64, kind: MemKind, concepts: &[&'static str]) -> PairEp {
        PairEp { day, kind, concepts: concepts.to_vec() }
    }

    #[test]
    fn recency_and_importance_rank_the_memories() {
        // a day-old promise (0.99+0.09) outranks a fresh meal (1+0.04):
        // importance outweighs a small recency gap
        let eps = vec![
            ep(90, MemKind::SharedMeal, &["meal"]),
            ep(89, MemKind::KeptPromise, &["errand"]),
        ];
        let picked = top(&eps, 90, &[], 1);
        assert_eq!(picked[0].kind, MemKind::KeptPromise);
        // but a twenty-day-old promise (0.82+0.09) fades under it
        let eps = vec![
            ep(90, MemKind::SharedMeal, &["meal"]),
            ep(70, MemKind::KeptPromise, &["errand"]),
        ];
        let picked = top(&eps, 90, &[], 1);
        assert_eq!(picked[0].kind, MemKind::SharedMeal);
    }

    #[test]
    fn concept_overlap_lifts_the_matching_episode() {
        let eps = vec![
            ep(88, MemKind::KeptPromise, &["errand", "promise"]),
            ep(90, MemKind::StoodWithUs, &["meeting"]),
        ];
        // a meeting-tagged talk surfaces the meeting memory over the
        // slightly older promise
        let picked = top(&eps, 90, &["meeting"], 1);
        assert_eq!(picked[0].kind, MemKind::StoodWithUs);
    }

    #[test]
    fn every_fact_names_a_kind() {
        for k in [MemKind::KeptPromise, MemKind::MissedMeeting, MemKind::StoodWithUs,
                  MemKind::AskedTooSoon, MemKind::SharedMeal] {
            assert!(fact(k).starts_with("mem:you_"));
            assert!(!callback(k).is_empty());
        }
    }
}
