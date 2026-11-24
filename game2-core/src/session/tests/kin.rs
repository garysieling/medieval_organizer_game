//! E2-2 (#1420): households and kin placed from the prehistory - the
//! roof gives everyone their people, the children start from their
//! parents' temperaments, and a family to a roof beats the old index
//! scatter

use crate::session::*;

/// The done-when: siblings sit closer in temperament than strangers,
/// on average - pooled across seeds so a lone pair doesn't carry it
#[test]
fn siblings_sit_closer_than_strangers() {
    let mut sib_d = 0.0;
    let mut sib_n = 0usize;
    let mut str_d = 0.0;
    let mut str_n = 0usize;
    for seed in [7, 42, 77, 1234, 99_999] {
        let g = Game::new_seeded_cast(seed);
        let temps: Vec<[f32; 5]> = g.villagers.iter().map(|&v| g.world
            .get_component::<crate::game::persona::Persona>(v).unwrap().temperament)
            .collect();
        let dist = |a: [f32; 5], b: [f32; 5]|
            a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum::<f32>();
        let mut sibling_pairs = vec![];
        for &v in &g.villagers {
            if let Some(k) = g.world.get_component::<villager::Kin>(v) {
                for &s in &k.siblings {
                    if !sibling_pairs.contains(&(s, v)) { sibling_pairs.push((v, s)); }
                }
            }
        }
        for (a, b) in &sibling_pairs {
            let (ia, ib) = (g.villagers.iter().position(|&x| x == *a).unwrap(),
                g.villagers.iter().position(|&x| x == *b).unwrap());
            sib_d += dist(temps[ia], temps[ib]);
            sib_n += 1;
        }
        for i in 0..g.villagers.len() {
            for j in (i + 1)..g.villagers.len() {
                if !sibling_pairs.contains(&(g.villagers[i], g.villagers[j]))
                    && !sibling_pairs.contains(&(g.villagers[j], g.villagers[i]))
                {
                    str_d += dist(temps[i], temps[j]);
                    str_n += 1;
                }
            }
        }
    }
    assert!(sib_n > 0, "no sibling pairs across the seeds");
    let (sib_avg, str_avg) = (sib_d / sib_n as f32, str_d / str_n as f32);
    assert!(sib_avg < str_avg,
        "siblings {:.3} strangers {:.3}", sib_avg, str_avg);
}

/// A child's axes start within noise of the parents' mean - the pull
/// the pass promises
#[test]
fn a_childs_axes_start_at_the_parents_mean() {
    let g = Game::new_seeded_cast(42);
    for &i in &KID_IDXS {
        let v = g.villagers[i];
        let k = g.world.get_component::<villager::Kin>(v)
            .unwrap_or_else(|| panic!("kid slot {i} has no kin"));
        assert!(!k.parents.is_empty(), "kid slot {i} has no parents");
        let mean = |a: usize| k.parents.iter().map(|&p| g.world
            .get_component::<crate::game::persona::Persona>(p).unwrap()
            .temperament[a]).sum::<f32>() / k.parents.len() as f32;
        let t = g.world.get_component::<crate::game::persona::Persona>(v)
            .unwrap().temperament;
        for a in 0..5 {
            assert!((t[a] - mean(a)).abs() <= 0.151,
                "kid {i} axis {a}: {:.3} vs parents' mean {:.3}", t[a], mean(a));
        }
    }
}

/// The roof's people: every villager carries Kin, everyone under one
/// roof shares a Home building and a Family bond
#[test]
fn a_roof_gives_its_people_to_each_other() {
    let g = Game::new_seeded_cast(77);
    let ph = g.world.resource::<crate::session::prehistory::Prehistory>().unwrap();
    // the record covers the founding 31 - country folk join after it
    for (i, &v) in g.villagers.iter().enumerate().take(ph.households.len()) {
        assert!(g.world.get_component::<villager::Kin>(v).is_some(),
            "slot {i} carries no Kin");
        if i == HERB_IDX { continue; } // her hut is hers alone
        for (j, &u) in g.villagers.iter().enumerate().take(ph.households.len()) {
            if i == j || ph.households[i] != ph.households[j] || j == HERB_IDX { continue; }
            assert_eq!(crate::game::household::home_of(&g.world, v),
                crate::game::household::home_of(&g.world, u),
                "household {}: slots {} and {} sleep apart", ph.households[i], i, j);
            assert_eq!(g.rel_kind(v, u), Some(crate::game::relationships::RelKind::Family),
                "slots {} and {} share a roof but no Family bond", i, j);
        }
    }
}

/// A child born in play joins the roof's kin and reads the house's
/// temperament
#[test]
fn a_born_child_joins_the_roof() {
    let mut g = Game::new_seeded_cast(42);
    // a real roof: the first home that holds two grown persona'd folk
    let home = g.world.query::<crate::game::household::Building>().into_iter()
        .find(|&b| {
            let bb = g.world.get_component::<crate::game::household::Building>(b).unwrap();
            bb.kind == crate::game::household::BuildingKind::Home
                && bb.occupants.iter().filter(|&&o| g.world
                    .get_component::<crate::game::persona::Persona>(o).is_some()
                    && g.world.get_component::<crate::game::vocation::Kid>(o).is_none())
                .count() >= 2
        }).expect("no two-elder home to bear a child");
    let before = g.villagers.len();
    g.spawn_kid(home, 1);
    let v = *g.villagers.last().unwrap();
    assert!(g.villagers.len() > before);
    let k = g.world.get_component::<villager::Kin>(v)
        .expect("the newborn has no kin");
    assert_eq!(k.parents.len(), 2);
    for &p in &k.parents {
        assert_eq!(g.rel_kind(v, p),
            Some(crate::game::relationships::RelKind::Family));
    }
    // and their axes sit within the pull's noise of the parents' mean
    let t = g.world.get_component::<crate::game::persona::Persona>(v).unwrap().temperament;
    for a in 0..5 {
        let mean = k.parents.iter().map(|&p| g.world
            .get_component::<crate::game::persona::Persona>(p).unwrap()
            .temperament[a]).sum::<f32>() / k.parents.len() as f32;
        assert!((t[a] - mean).abs() <= 0.151,
            "newborn axis {a}: {:.3} vs mean {:.3}", t[a], mean);
    }
}
