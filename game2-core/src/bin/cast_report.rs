//! Cast report (E0-1, #1400): deals the cast over N seeds and reports
//! the distinctiveness numbers the validator in E1-5 (#1413) checks
//! on - archetype spread, shared signatures, ghost pile-ups, the
//! nearest temperament+values pair, trait-set diversity and
//! contradictions. The numbers come off `cast_check::measure`, the
//! same function the validator calls, so the report cannot disagree
//! with it.
//!
//! Usage: cast_report [N] [mode]   (default 100 seeds, mode `seeded`;
//! `fixed` measures the authored cast's trait sets and hook pile-ups
//! for the pre-persona baseline)

use game2_core::engine::det::HashMap;
use game2_core::game::cast_check::{measure, CastCheck};
use game2_core::game::hooks::{Hooks, HOOKS};
use game2_core::game::persona::Persona;
use game2_core::game::traits::Personality as Traits;
use game2_core::session::{Game, NAMES};

/// Target floors from the design's cast-checks table (starting
/// values, tuned against this report)
const MAX_WOUND_HOLDERS: usize = 2;
const MIN_ARCHETYPES: usize = 6;
const MAX_ARCHETYPE_SHARE: f32 = 0.20;
const NEAREST_PAIR_FLOOR: f32 = 0.5;
const CONTRADICTION_RATE: f32 = 1.0 / 6.0;
const CONTRADICTION_TOLERANCE: f32 = 0.10;

fn pct(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        0.0
    } else {
        part as f32 / whole as f32
    }
}

fn seeded_row(seed: u64) -> CastCheck {
    let g = Game::new_seeded_cast(seed);
    let personas: Vec<&Persona> = g
        .villagers
        .iter()
        .take(NAMES.len())
        .map(|&v| {
            g.world
                .get_component::<Persona>(v)
                .expect("seeded villager carries a Persona")
        })
        .collect();
    measure(&personas)
}

fn fixed_row(seed: u64) -> (usize, usize) {
    // the authored cast has no personas: report its trait-set
    // diversity and its biggest hook pile-up instead
    let g = Game::new_seeded(seed);
    let mut trait_sets = game2_core::engine::det::HashSet::default();
    let mut hooks: HashMap<&'static str, usize> = HashMap::default();
    for &v in g.villagers.iter().take(NAMES.len()) {
        if let Some(t) = g.world.get_component::<Traits>(v) {
            let mut names: Vec<String> = t.traits.iter().map(|t| format!("{t:?}")).collect();
            names.sort();
            trait_sets.insert(names.join(","));
        }
        if let Some(h) = g.world.get_component::<Hooks>(v) {
            for c in &h.carried {
                *hooks.entry(HOOKS[c.id].subject).or_insert(0) += 1;
            }
        }
    }
    (trait_sets.len(), hooks.values().copied().max().unwrap_or(0))
}

fn main() {
    game2_core::infra::crash::install();
    let mut args = std::env::args().skip(1);
    let n: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(100);
    let mode = args.next().unwrap_or_else(|| "seeded".to_string());

    if mode == "fixed" || mode == "both" {
        let mut worst_sets = (usize::MAX, 0u64);
        let mut worst_hook = (0usize, 0u64);
        for seed in 0..n {
            let (sets, hook_max) = fixed_row(seed);
            if sets < worst_sets.0 {
                worst_sets = (sets, seed);
            }
            if hook_max > worst_hook.0 {
                worst_hook = (hook_max, seed);
            }
        }
        println!("== fixed cast (authored, {} seeds) ==", n);
        println!(
            "  fewest distinct trait sets: {} in {} villagers (seed {})",
            worst_sets.0,
            NAMES.len(),
            worst_sets.1
        );
        println!(
            "  biggest hook pile-up: {} holders (seed {})",
            worst_hook.0, worst_hook.1
        );
        if mode == "fixed" {
            return;
        }
        println!();
    }

    let mut archetype_totals: HashMap<usize, (usize, usize, usize)> = HashMap::default(); // (sum, min, max)
    let mut fail_sig = 0usize;
    let mut fail_wound = 0usize;
    let mut fail_archetype = 0usize;
    let mut fail_pair = 0usize;
    let mut fail_contra = 0usize;
    let mut pair_min = f32::MAX;
    let mut pair_max = 0.0f32;
    let mut pair_sum = 0.0f32;
    let mut sets_min = usize::MAX;
    let mut sets_sum = 0usize;
    let mut contra_sum = 0usize;
    let mut wounds_sum = 0usize;
    let mut worst_pair = (0u64, (0usize, 0usize), f32::MAX);

    for seed in 0..n {
        let c = seeded_row(seed);
        let max_share = c.archetypes.iter().map(|&(_, k)| k).max().unwrap_or(0);
        fail_sig += (c.shared_signatures > 0) as usize;
        fail_wound += (c.max_wound_holders > MAX_WOUND_HOLDERS) as usize;
        fail_archetype += (c.archetypes.len() < MIN_ARCHETYPES
            || pct(max_share, c.cast) > MAX_ARCHETYPE_SHARE) as usize;
        fail_pair += (c.nearest_pair < NEAREST_PAIR_FLOOR) as usize;
        let rate = pct(c.contradictions, c.cast);
        fail_contra += ((rate - CONTRADICTION_RATE).abs() > CONTRADICTION_TOLERANCE) as usize;
        pair_min = pair_min.min(c.nearest_pair);
        pair_max = pair_max.max(c.nearest_pair);
        pair_sum += c.nearest_pair;
        if c.nearest_pair < worst_pair.2 {
            worst_pair = (seed, c.nearest_slots, c.nearest_pair);
        }
        sets_min = sets_min.min(c.distinct_trait_sets);
        sets_sum += c.distinct_trait_sets;
        contra_sum += c.contradictions;
        wounds_sum += c.distinct_wounds;
        for &(a, k) in &c.archetypes {
            let e = archetype_totals
                .entry(a as usize)
                .or_insert((0, usize::MAX, 0));
            e.0 += k;
            e.1 = e.1.min(k);
            e.2 = e.2.max(k);
        }
    }

    // ghost detail needs personas again for wound counts - recollect
    // cheaply from the worst rows is overkill; wound_max was tracked
    // per cast above via CastCheck::max_wound_holders
    println!(
        "== seeded cast ({} seeds, {} villagers each) ==",
        n,
        NAMES.len()
    );
    println!();
    println!("check                              target            seeds failing");
    println!(
        "  shared signatures                  0                 {}",
        fail_sig
    );
    println!(
        "  most holders of one wound          <= {}             {}",
        MAX_WOUND_HOLDERS, fail_wound
    );
    println!(
        "  archetypes present / biggest share >= {}/<= 20%      {}",
        MIN_ARCHETYPES, fail_archetype
    );
    println!(
        "  nearest pair distance              >= {:.2}           {}",
        NEAREST_PAIR_FLOOR, fail_pair
    );
    println!(
        "  contradictions per cast            ~1 in 6 +/-10%    {}",
        fail_contra
    );
    println!();
    println!(
        "nearest pair: min {:.3}  mean {:.3}  max {:.3}  (worst seed {}, slots {} and {})",
        pair_min,
        pair_sum / n as f32,
        pair_max,
        worst_pair.0,
        worst_pair.1 .0,
        worst_pair.1 .1
    );
    println!(
        "trait sets:  fewest {} in {}  mean {:.1}",
        sets_min,
        NAMES.len(),
        sets_sum as f32 / n as f32
    );
    println!(
        "contradictions per cast: mean {:.1}",
        contra_sum as f32 / n as f32
    );
    println!(
        "distinct wounds per cast: mean {:.1}",
        wounds_sum as f32 / n as f32
    );
    println!();
    println!("archetype holders per cast (mean min-max):");
    let mut rows: Vec<(usize, (usize, usize, usize))> = archetype_totals.into_iter().collect();
    rows.sort_by_key(|&(idx, _)| idx);
    for (idx, (sum, lo, hi)) in rows {
        println!(
            "  {:<16} {:.1}  ({}..{})",
            format!("{:?}", game2_core::game::persona::ARCHETYPES[idx]),
            sum as f32 / n as f32,
            lo,
            hi
        );
    }
    let failing = fail_sig
        .max(fail_wound)
        .max(fail_archetype)
        .max(fail_pair)
        .max(fail_contra);
    println!();
    println!(
        "distinctive in {:.0}% of seeds (worst single check: {} failures)",
        100.0 * (n as usize - failing) as f32 / n as f32,
        failing
    );
}
