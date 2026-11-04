//! Deterministic hash collections for sim state.
//!
//! `std::collections::HashMap` seeds its hasher randomly per map, so two runs
//! of one seed walk the same map in different orders. Anything that iterates
//! a map (fire spreading, trails, the probe's lists, the order tiles are
//! drawn) then diverges between runs, which breaks replays, lockstep
//! multiplayer and golden-frame tests. These aliases use std's SipHash with
//! fixed keys: same inserts, same order, every run.
//!
//! Use these in game/, sim/, session/ and engine/. Build them with
//! `HashMap::default()` (there's no `new()` for a custom hasher).

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

/// A fixed-key hasher: `DefaultHasher::new()` always starts from the same keys
pub type DetState = BuildHasherDefault<DefaultHasher>;

pub type HashMap<K, V> = std::collections::HashMap<K, V, DetState>;
pub type HashSet<T> = std::collections::HashSet<T, DetState>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_maps_with_the_same_inserts_iterate_alike() {
        let fill = || {
            let mut m: HashMap<(i32, i32), u32> = HashMap::default();
            for i in 0..500 { m.insert((i * 7 % 31, i), i as u32); }
            m.keys().copied().collect::<Vec<_>>()
        };
        assert_eq!(fill(), fill());
    }
}
