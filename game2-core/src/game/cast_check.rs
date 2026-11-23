//! Cast distinctiveness checks (E0-1, #1400): the metrics the cast
//! report binary prints and the E1-5 validator (#1413) rejects on.
//! One source of truth so the report can never disagree with the
//! validation pass.

use crate::engine::det::{HashMap, HashSet};
use crate::game::persona::{Belief, Persona};

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
}
