//! Cast distinctiveness checks (E0-1, #1400): the metrics the cast
//! report binary prints and the E1-5 validator (#1413) rejects on.
//! One source of truth so the report can never disagree with the
//! validation pass.

use crate::engine::det::{HashMap, HashSet};
use crate::game::persona::{draw, Belief, Persona, SIGNATURES};

/// Shared thresholds the report prints and the validator rejects on -
/// tuned off the #1400 baseline: the nearest-pair floor sits just under
/// what a natural roll reaches (0.5 was unreachable: the mean nearest
/// pair over 100 seeds was 0.37)
pub const MIN_ARCHETYPES: usize = 6;
pub const MAX_ARCHETYPE_SHARE: f32 = 0.20;
pub const MAX_WOUND_HOLDERS: usize = 2;
pub const NEAREST_PAIR_FLOOR: f32 = 0.25;
/// Reroll rounds a failing cast gets before the validator accepts the
/// best effort (grilled: "~20 attempts per villager")
pub const MAX_ATTEMPTS: u32 = 24;

/// Per-cast distinctiveness snapshot: one row per seed in the report,
/// one verdict per check in the validator
#[derive(Debug, Clone)]
pub struct CastCheck {
    /// Villagers measured (the founding cast)
    pub cast: usize,
    /// Archetype -> holder count
    pub archetypes: Vec<(crate::game::persona::Archetype, usize)>,
    /// Villagers whose signature another villager shares (target 0)
    pub shared_signatures: usize,
    /// Most holders of one ghost wound (Belief), shared event or not
    /// (target <= 2 - E2-4 keeps this tight)
    pub max_wound_holders: usize,
    /// How many distinct Belief wounds the cast carries
    pub distinct_wounds: usize,
    /// Smallest temperament+values distance between any two villagers
    /// (the "nearest pair" - flagged when under the tuned floor)
    pub nearest_pair: f32,
    /// The two slots of the nearest pair
    pub nearest_slots: (usize, usize),
    /// Distinct trait sets across the cast (the old "26 in 31" metric)
    pub distinct_trait_sets: usize,
    /// Deliberate contradictions drawn into personas (target ~1 in 6)
    pub contradictions: usize,
}

/// Euclidean distance over temperament (5 axes) + values (10): the
/// "how alike are two villagers" number the nearest-pair check uses
pub fn pair_distance(a: &Persona, b: &Persona) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..5 {
        let d = a.temperament[i] - b.temperament[i];
        sum += d * d;
    }
    for i in 0..10 {
        let d = a.values[i] - b.values[i];
        sum += d * d;
    }
    sum.sqrt()
}

/// Measure one cast's personas (the founding villagers, in slot order)
pub fn measure(personas: &[&Persona]) -> CastCheck {
    let mut archetypes: HashMap<usize, usize> = HashMap::default();
    let mut signatures: HashMap<&'static str, usize> = HashMap::default();
    let mut wounds: HashMap<usize, usize> = HashMap::default();
    let mut trait_sets: HashSet<String> = HashSet::default();
    let mut shared_signatures = 0usize;
    let mut contradictions = 0usize;
    let mut nearest = (f32::MAX, 0usize, 0usize);
    for (i, p) in personas.iter().enumerate() {
        *archetypes.entry(p.archetype as usize).or_insert(0) += 1;
        let count = signatures.entry(p.signature.text()).or_insert(0);
        *count += 1;
        if *count > 1 {
            shared_signatures += 1;
        }
        for g in &p.ghosts {
            *wounds.entry(g.lie as usize).or_insert(0) += 1;
        }
        let (_, traits) = p.personalities();
        let mut names: Vec<String> = traits.traits.iter().map(|t| format!("{t:?}")).collect();
        names.sort();
        trait_sets.insert(names.join(","));
        contradictions += p.contradiction.is_some() as usize;
        for (j, other) in personas.iter().enumerate().take(i) {
            let d = pair_distance(p, other);
            if d < nearest.0 {
                nearest = (d, j, i);
            }
        }
    }
    let mut archetypes: Vec<(crate::game::persona::Archetype, usize)> = archetypes
        .into_iter()
        .map(|(idx, n)| (crate::game::persona::ARCHETYPES[idx], n))
        .collect();
    archetypes.sort_by_key(|&(a, _)| a as usize);
    CastCheck {
        cast: personas.len(),
        archetypes,
        shared_signatures,
        max_wound_holders: wounds.values().copied().max().unwrap_or(0),
        distinct_wounds: wounds.len(),
        nearest_pair: if personas.len() > 1 { nearest.0 } else { 0.0 },
        nearest_slots: (nearest.1, nearest.2),
        distinct_trait_sets: trait_sets.len(),
        contradictions,
    }
}

/// Wound-type histogram for the report detail lines
pub fn wound_holders(personas: &[&Persona]) -> Vec<(Belief, usize)> {
    let mut wounds: HashMap<usize, usize> = HashMap::default();
    for p in personas {
        for g in &p.ghosts {
            *wounds.entry(g.lie as usize).or_insert(0) += 1;
        }
    }
    let mut rows: Vec<(Belief, usize)> = wounds
        .into_iter()
        .map(|(idx, n)| (crate::game::persona::BELIEFS[idx], n))
        .collect();
    rows.sort_by_key(|&(b, _)| b as usize);
    rows
}

/// The slots a failed check points at for a reroll: the nearer half of
/// an under-floor pair, the surplus holders of an over-shared
/// archetype, and (when the spread is thin) a holder of the biggest
/// group hoping the salt draws a missing one. Signature dupes aren't
/// here - the deal below removes them without a reroll
fn failing_slots(check: &CastCheck, personas: &[&Persona]) -> Vec<usize> {
    let mut slots: Vec<usize> = vec![];
    if personas.len() > 1 && check.nearest_pair < NEAREST_PAIR_FLOOR {
        slots.push(check.nearest_slots.1);
    }
    let allowed = ((personas.len() as f32 * MAX_ARCHETYPE_SHARE) as usize).max(1);
    for &(a, n) in &check.archetypes {
        if n > allowed {
            let holders: Vec<usize> = personas.iter().enumerate()
                .filter(|(_, p)| p.archetype == a).map(|(i, _)| i).collect();
            slots.extend(holders.iter().skip(allowed).copied());
        }
    }
    if check.archetypes.len() < MIN_ARCHETYPES && !personas.is_empty() {
        let biggest = check.archetypes.iter().max_by_key(|&&(_, n)| n).map(|&(a, _)| a);
        if let Some(a) = biggest {
            if let Some((s, _)) = personas.iter().enumerate()
                .filter(|(_, p)| p.archetype == a)
                .find(|(s, _)| !slots.contains(s)) {
                slots.push(s);
            }
        }
    }
    slots
}

/// E1-5 (#1413): reroll passes 3-6 for the villagers a check rejects,
/// up to MAX_ATTEMPTS rounds - prehistory, households and bodies stay
/// fixed (the salt only moves the later passes). The ghost caps re-run
/// after every round so a regen can't reopen a pile-up. Best-effort:
/// a cast that still fails after MAX_ATTEMPTS ships anyway. Returns
/// the rounds spent
pub fn validate(
    cast: &mut Vec<Option<Persona>>,
    regen: &mut impl FnMut(usize, u32) -> Persona,
) -> u32 {
    for attempt in 1..=MAX_ATTEMPTS {
        let slots = {
            let ps: Vec<&Persona> = cast.iter().filter_map(|p| p.as_ref()).collect();
            failing_slots(&measure(&ps), &ps)
        };
        if slots.is_empty() {
            return attempt - 1;
        }
        for s in slots {
            cast[s] = Some(regen(s, attempt));
        }
        crate::game::persona::cap_cast_ghosts(cast);
    }
    MAX_ATTEMPTS
}

/// Pass 6's "a landmark no neighbor shares" guaranteed: signatures
/// dealt off the pool without replacement in a seeded shuffle, so the
/// deal is deterministic like every other draw (E1-5 - the pool is
/// wider than the founding cast)
pub fn deal_signatures(cast: &mut [Option<Persona>], seed: u64) {
    let n = cast.iter().filter(|p| p.is_some()).count().min(SIGNATURES.len());
    let mut order: Vec<usize> = (0..SIGNATURES.len()).collect();
    for i in (1..order.len()).rev() {
        let j = (draw(seed, 900 + i as u64) * (i + 1) as f32) as usize % (i + 1);
        order.swap(i, j);
    }
    for (slot, &idx) in cast.iter_mut().filter_map(|p| p.as_mut()).zip(&order).take(n) {
        slot.signature = SIGNATURES[idx];
    }
}

/// Pass 6 extended to speech (E4-7, #1416): a pet phrase and a habit
/// per villager, dealt without replacement in a seeded shuffle like
/// the signatures. A persona whose top value a habit speaks for
/// (caution's proverbs to the safety-minded, mockery's to the free)
/// gets it first; the rest take what the shuffle leaves
pub fn deal_idiolects(cast: &mut [Option<Persona>], seed: u64) {
    use crate::game::idiolect::{Idiolect, HABITS, PHRASES};
    let mut horder: Vec<usize> = (0..HABITS.len()).collect();
    for i in (1..horder.len()).rev() {
        let j = (draw(seed, 950 + i as u64) * (i + 1) as f32) as usize % (i + 1);
        horder.swap(i, j);
    }
    let mut porder: Vec<usize> = (0..PHRASES.len()).collect();
    for i in (1..porder.len()).rev() {
        let j = (draw(seed, 1050 + i as u64) * (i + 1) as f32) as usize % (i + 1);
        porder.swap(i, j);
    }
    let n = cast.iter().filter(|p| p.is_some()).count()
        .min(HABITS.len()).min(PHRASES.len());
    let mut used = HashSet::default();
    for (n, slot) in cast.iter_mut().filter_map(|p| p.as_mut()).enumerate().take(n) {
        let top = slot.values.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);
        let pick = horder.iter().copied()
            .find(|&h| !used.contains(&h) && HABITS[h].prefers() == top)
            .or_else(|| horder.iter().copied().find(|&h| !used.contains(&h)))
            .unwrap_or(0);
        used.insert(pick);
        slot.idiolect = Idiolect {
            phrase: PHRASES[porder[n % porder.len()]],
            habit: HABITS[pick],
        };
    }
}

/// A villager is identical to another when it shares the other's
/// signature or its full trait set - the report's "identical-line" side
/// of the baseline ("26 trait sets in 31 villagers")
pub fn identical_line_rate(personas: &[&Persona]) -> f32 {
    if personas.is_empty() {
        return 0.0;
    }
    1.0 - measure(personas).distinct_trait_sets as f32 / personas.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::persona::{generate, Past};

    #[test]
    fn the_check_reads_one_cast() {
        let personas: Vec<Persona> = (0..8).map(|s| generate(7, s, &Past::default())).collect();
        let refs: Vec<&Persona> = personas.iter().collect();
        let c = measure(&refs);
        assert_eq!(c.cast, 8);
        assert_eq!(c.distinct_wounds, 0);
        assert!(c.nearest_pair > 0.0);
        assert!(c.distinct_trait_sets >= 1);
    }

    #[test]
    fn the_deal_shares_no_signature() {
        let mut cast: Vec<Option<Persona>> = (0..31)
            .map(|s| Some(generate(9, s, &Past::default())))
            .collect();
        deal_signatures(&mut cast, 9);
        let mut seen = HashSet::default();
        for p in cast.iter().flatten() {
            assert!(seen.insert(p.signature.text()),
                "{} dealt twice", p.signature.text());
        }
        // deterministic: same seed, same deal
        let mut again: Vec<Option<Persona>> = (0..31)
            .map(|s| Some(generate(9, s, &Past::default())))
            .collect();
        deal_signatures(&mut again, 9);
        for (a, b) in cast.iter().zip(&again) {
            assert_eq!(a.as_ref().unwrap().signature.text(),
                b.as_ref().unwrap().signature.text());
        }
    }

    #[test]
    fn the_validator_rerolls_only_the_later_passes() {
        // two villagers with the same slot roll the same person: the
        // pair floor rejects them, and the salt splits them while the
        // past and body stay put
        let mut cast: Vec<Option<Persona>> = vec![
            Some(generate(9, 0, &Past::default())),
            Some(generate(9, 0, &Past::default())),
            Some(generate(9, 2, &Past::default())),
        ];
        // slot 1 rerolls as if it were slot 1 - give it slot 1's roll
        cast[1] = Some(generate(9, 1, &Past::default()));
        let before = cast[1].clone().unwrap();
        validate(&mut cast, &mut |slot, attempt| {
            crate::game::persona::generate_attempt(9, slot, &Past::default(), attempt)
        });
        let after = cast[1].clone().unwrap();
        assert_eq!(before.sex, after.sex);           // body stays
        assert_eq!(before.story, after.story);       // prehistory stays
        // the reroll salt moved the later passes
        assert_ne!(before.temperament, after.temperament);
    }

    #[test]
    fn the_validator_leaves_a_passing_cast_alone() {
        let mut cast: Vec<Option<Persona>> = (0..31)
            .map(|s| Some(generate(3, s, &Past::default())))
            .collect();
        crate::game::persona::cap_cast_ghosts(&mut cast);
        let rounds = validate(&mut cast, &mut |slot, attempt| {
            crate::game::persona::generate_attempt(3, slot, &Past::default(), attempt)
        });
        let ps: Vec<&Persona> = cast.iter().flatten().collect();
        let c = measure(&ps);
        // whatever the seed drew, no wound exceeds the cap and the
        // archetype share respects the ceiling
        assert!(c.max_wound_holders <= MAX_WOUND_HOLDERS);
        let biggest = c.archetypes.iter().map(|&(_, n)| n).max().unwrap();
        assert!(biggest <= ((31f32 * MAX_ARCHETYPE_SHARE) as usize).max(1)
            || rounds >= MAX_ATTEMPTS);
    }
}
