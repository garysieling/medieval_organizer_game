use crate::session::*;
use crate::game::virtue::{MoralAct, Virtue, Virtues};

#[test]
fn test_villagers_carry_a_soul_their_traits_bent() {
    let g = Game::new();
    // everyone carries the seven, near the common measure, and a kind
    // soul starts kinder than a stingy one
    let mut shaped = 0;
    for &v in &g.villagers {
        let vs = g.world.get_component::<Virtues>(v).expect("no soul seeded");
        for s in vs.scores { assert!((0.0..=1.0).contains(&s)); }
        if vs.scores.iter().any(|&s| (s - 0.5).abs() > 1e-6) { shaped += 1; }
        let has_kind = g.world.get_component::<crate::game::traits::Personality>(v)
            .map(|p| p.traits.contains(&crate::game::traits::Trait::Kind)).unwrap_or(false);
        if has_kind { assert!(vs.get(Virtue::Charity) > 0.5, "a kind start reads in the soul"); }
    }
    assert!(shaped > 0, "nobody's traits bent their soul");
}

#[test]
fn test_an_act_bends_the_soul_that_chose_it() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let before = g.world.get_component::<Virtues>(v).unwrap().get(Virtue::Charity);
    crate::game::virtue::witness(&mut g.world, v, MoralAct::SharedAid, 1.0);
    let vs = g.world.get_component::<Virtues>(v).unwrap();
    assert!(vs.get(Virtue::Charity) > before, "sharing grows the merciful");
    crate::game::virtue::witness(&mut g.world, v, MoralAct::RefusedAid, 1.0);
    crate::game::virtue::witness(&mut g.world, v, MoralAct::RefusedAid, 1.0);
    let vs = g.world.get_component::<Virtues>(v).unwrap();
    assert!(vs.get(Virtue::Charity) < before + 0.06, "keeping wears it back");
}

#[test]
fn test_the_knot_weighs_on_who_answered() {
    let mut g = Game::new();
    let (player, victim) = (g.seats[0].player, g.villagers[0]);
    let before = g.world.get_component::<Virtues>(player).unwrap().scores;
    // a heavy debt landing: the pick put the weight on the victim
    g.dilemma_debts.push(crate::session::events::DilemmaDebt {
        branch: 0, due: 5, witness: player, victim, weight: 0.8 });
    g.dilemma_tick(5);
    let after = g.world.get_component::<Virtues>(player).unwrap();
    assert!(after.get(Virtue::Prudence) > before[Virtue::Prudence as usize],
        "laying the weight on teaches a colder wisdom");
    let after_charity = after.get(Virtue::Charity);
    assert!(after_charity < before[Virtue::Charity as usize],
        "and costs the merciful part of them");

    // a light one: sparing the innocent grows them instead
    g.dilemma_debts.push(crate::session::events::DilemmaDebt {
        branch: 1, due: 7, witness: player, victim, weight: 0.2 });
    g.dilemma_tick(7);
    let after2 = g.world.get_component::<Virtues>(player).unwrap();
    assert!(after2.get(Virtue::Charity) > after_charity,
        "sparing the innocent grows the soul back");
}

#[test]
fn test_a_kind_neighbor_shares_their_table() {
    let mut g = Game::new();
    // a roof that isn't hungry itself, and a face to feed
    let a = g.villagers.iter().copied().find(|&v|
        !desires::problems_of(&g.world, v, 0).contains(&desires::Problem::Hungry))
        .expect("nobody with a full pot");
    let b = g.villagers.iter().copied().find(|&v| v != a).unwrap();
    // a bond worth standing on, and b's pot is bare all season
    let r = g.world.create_entity();
    g.world.add_component(r, crate::game::relationships::Relationship {
        from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
        strength: 0.6, interactions: 5 });
    // the fork needs a face: cut every other bond's strength so a's only
    // hungry neighbor is b and nobody else's fork can fire
    for e in g.world.query::<crate::game::relationships::Relationship>() {
        if e == r { continue; }
        if let Some(rel) = g.world.get_component_mut::<crate::game::relationships::Relationship>(e) {
            rel.strength = 0.0;
        }
    }
    desires::add_grievance(&mut g.world, b, desires::Problem::Hungry, 10_000);
    // a is a kind, open-handed soul - the fork can only go one way
    g.world.get_component_mut::<crate::game::traits::Personality>(a).unwrap()
        .traits = vec![crate::game::traits::Trait::Kind];
    g.world.get_component_mut::<Virtues>(a).unwrap().scores =
        [0.5, 0.5, 0.5, 0.1, 0.5, 0.5, 0.9]; // prudent little, charitable much
    // run the days until a's hard moment comes - the gate is per
    // (villager, day), so one of these ticks opens the fork
    let before = g.world.get_component::<Virtues>(a).unwrap().get(Virtue::Charity);
    let mut shared = false;
    for t in 0..600 {
        g.virtue_tick(t);
        let vs = g.world.get_component::<Virtues>(a).unwrap();
        if vs.get(Virtue::Charity) > before { shared = true; break; }
    }
    assert!(shared, "the kind neighbor never opened their table");
    // and b carries the warm memory of it
    let hist = g.world.get_component::<crate::sim::psychology::AttachmentHistory>(b).unwrap();
    assert!(hist.episodes.iter().any(|e| e.with == a
        && matches!(e.kind, crate::sim::psychology::EpisodeKind::Warmth)),
        "the fed neighbor remembers who fed them");
}
