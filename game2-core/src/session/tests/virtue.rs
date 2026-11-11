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

#[test]
fn test_a_contract_kept_is_a_word_kept() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    let before = g.world.get_component::<Virtues>(a).unwrap().get(Virtue::Justice);
    let c = g.world.create_entity();
    g.world.add_component(c, crate::game::contracts::Contract {
        parties: vec![a, b],
        terms: vec![crate::game::contracts::Term {
            description: "haul the stone".into(), obligated: a, fulfilled: true }],
        status: crate::game::contracts::ContractStatus::Active,
        breach_penalty: 0.0, aid_pact: false });
    crate::engine::ecs::System::run(&mut crate::game::contracts::ContractSystem, &mut g.world);
    let after = g.world.get_component::<Virtues>(a).unwrap().get(Virtue::Justice);
    assert!(after > before, "the kept contract didn't grow the just: {before} -> {after}");
}

#[test]
fn test_the_surety_paid_is_a_word_kept() {
    let mut g = Game::new();
    let (voucher, debtor) = (g.villagers[0], g.villagers[1]);
    if let Some(p) = g.world.get_component_mut::<Purse>(debtor) { p.gold += 50.0; }
    let before = g.world.get_component::<Virtues>(debtor).unwrap().get(Virtue::Justice);
    g.vouches.push(crate::game::community_law::Vouch {
        voucher, debtor, bond: 4.0, owed: 0.001, made_on: 0 });
    g.pledge_tick(3);
    let after = g.world.get_component::<Virtues>(debtor).unwrap().get(Virtue::Justice);
    assert!(after > before, "making good didn't grow the just: {before} -> {after}");
    assert!(g.vouches.is_empty(), "the word's still on the books");
}

#[test]
fn test_the_visit_is_comfort_given() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
    let home = household::home_of(&g.world, v).unwrap();
    let checker = g.world.get_component::<Building>(home).unwrap()
        .occupants.iter().copied().find(|&o| o != v);
    let Some(checker) = checker else { return }; // no kin to visit this seed
    let before = g.world.get_component::<Virtues>(checker).unwrap().get(Virtue::Charity);
    g.care_tick(2);
    let after = g.world.get_component::<Virtues>(checker).unwrap().get(Virtue::Charity);
    assert!(after > before, "the morning's visit gave no comfort to the soul: {before} -> {after}");
}

#[test]
fn test_a_pledge_kept_grows_and_broken_wears_the_leader() {
    let mut g = Game::new();
    // a taxed cause to stand an assembly behind
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    let org = g.org;
    let leader = crate::game::organizing::living_leader(&mut g.world, org).unwrap();
    for &v in &g.villagers[1..3] {
        crate::game::relationships::record_interaction(&mut g.world, v,
            crate::game::relationships::Interaction {
                other: leader,
                kind: crate::game::relationships::InteractionKind::Aid,
                intensity: 1.0 });
    }
    crate::game::campaign::announce_assembly(&mut g.world, c, g.tick);
    let held = g.tick + crate::game::campaign::PUBLISH_GAP;
    crate::game::campaign::hold_assembly(&mut g.world, c, held).unwrap();
    // every pledge a yes on a check that can't move: all broke
    {
        let a = g.world.get_component_mut::<crate::game::campaign::Assembly>(c).unwrap();
        for p in &mut a.pledges {
            p.check = crate::game::campaign::PledgeCheck::Action { from: u32::MAX };
            p.answer = true;
            p.deadline = held;
        }
    }
    let justice_before = g.world.get_component::<Virtues>(leader).unwrap().get(Virtue::Justice);
    crate::game::campaign::resolve_pledges(&mut g.world, c, held);
    let justice_after = g.world.get_component::<Virtues>(leader).unwrap().get(Virtue::Justice);
    assert!(justice_after < justice_before,
        "the broken word didn't wear the just: {justice_before} -> {justice_after}");
    // the kept word instead: no's on checks that can't hold stand
    {
        let a = g.world.get_component_mut::<crate::game::campaign::Assembly>(c).unwrap();
        for (i, p) in a.pledges.iter_mut().enumerate() {
            let _ = i;
            p.kept = None;
            p.answer = false;
            p.deadline = held + 1;
        }
    }
    crate::game::campaign::resolve_pledges(&mut g.world, c, held + 1);
    let justice_last = g.world.get_component::<Virtues>(leader).unwrap().get(Virtue::Justice);
    assert!(justice_last > justice_after,
        "the kept word didn't grow it back: {justice_after} -> {justice_last}");
}

#[test]
fn test_the_marked_soul_carries_a_name() {
    let mut g = Game::new();
    let v = g.villagers[0];
    {
        let vs = g.world.get_component_mut::<Virtues>(v).unwrap();
        vs.scores[Virtue::Charity as usize] = 0.7;
    }
    let s = crate::session::dialogue::speaker_of(&g.world, v, "Eliza", 0);
    assert!(s.status.contains(&"kind_soul"), "the marked life carried no name: {:?}", s.status);
    {
        let vs = g.world.get_component_mut::<Virtues>(v).unwrap();
        vs.scores[Virtue::Charity as usize] = 0.2;
    }
    let s = crate::session::dialogue::speaker_of(&g.world, v, "Eliza", 0);
    assert!(s.status.contains(&"hard"), "the closed hand carried no name: {:?}", s.status);
    assert!(!s.status.contains(&"kind_soul"), "hard and kind don't sit together");
}

#[test]
fn test_the_good_name_greets_ahead_of_smalltalk() {
    use crate::game::conversation::{converse, Relation, Speaker};
    let a = Speaker::new("Anna");
    let mut b = Speaker::new("Berta");
    b.status.push("kind_soul");
    let lib = crate::game::dialogue_trees::village_library();
    let tr = converse(&a, &b, Relation::Acquaintance, &lib, &mut crate::game::conversation::Rng::new(7));
    assert_eq!(tr.trees_walked.first().copied(), Some("the_good_name"),
        "the name didn't greet first: {:?}", tr.trees_walked);
    // and the talk doesn't die on the greeting - the rejoin runs
    assert!(tr.trees_walked.len() > 1 || tr.turns.len() > 2,
        "the greeting was the whole conversation");
}

#[test]
fn test_the_cold_one_gets_the_wary_greeting() {
    use crate::game::conversation::{converse, Relation, Speaker};
    let a = Speaker::new("Anna");
    let mut b = Speaker::new("Berta");
    b.status.push("hard");
    let lib = crate::game::dialogue_trees::village_library();
    let tr = converse(&a, &b, Relation::Acquaintance, &lib, &mut crate::game::conversation::Rng::new(3));
    assert_eq!(tr.trees_walked.first().copied(), Some("the_cold_one"),
        "the wary greeting never came: {:?}", tr.trees_walked);
}

#[test]
fn test_a_temperate_soul_feuds_slower() {
    let mut g = Game::new();
    let (hot, cool, victim) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    g.world.get_component_mut::<Virtues>(hot).unwrap()
        .scores[Virtue::Temperance as usize] = 0.1;
    g.world.get_component_mut::<Virtues>(cool).unwrap()
        .scores[Virtue::Temperance as usize] = 0.9;
    let f1 = crate::game::feuds::spawn_feud(&mut g.world, hot, victim, "a slight at the well");
    let f2 = crate::game::feuds::spawn_feud(&mut g.world, cool, victim, "a slight at the well");
    crate::engine::ecs::System::run(&mut crate::game::feuds::FeudSystem, &mut g.world);
    let hot_i = g.world.get_component::<crate::game::feuds::Feud>(f1).unwrap().intensity;
    let cool_i = g.world.get_component::<crate::game::feuds::Feud>(f2).unwrap().intensity;
    assert!(hot_i > cool_i,
        "temper didn't bend the burn: hot {hot_i} vs cool {cool_i}");
}


#[test]
fn test_the_devout_hear_the_call_the_cold_let_pass() {
    let mut g = Game::new();
    // an ordinary pair: no piety, no reform, no old ways
    let pick = |g: &Game, taken: &[Entity]| g.villagers.iter().copied().find(|&v| {
        if taken.contains(&v) || villager::is_dead(&g.world, v)
            || g.world.get_component::<Kid>(v).is_some() { return false; }
        let pious = g.world.get_component::<crate::game::traits::Personality>(v)
            .map(|p| p.traits.contains(&crate::game::traits::Trait::Pious)).unwrap_or(false);
        !pious && !crate::session::routine::keeps_old_ways(&g.world, v)
            && g.world.get_component::<crate::sim::religion::Reformed>(v).is_none()
    }).expect("no plain soul to pick");
    let devout = pick(&g, &[]);
    let cold = pick(&g, &[devout]);
    g.world.get_component_mut::<Virtues>(devout).unwrap().scores[Virtue::Faith as usize] = 1.0;
    g.world.get_component_mut::<Virtues>(cold).unwrap().scores[Virtue::Faith as usize] = 0.0;
    let mut devout_heard = false;
    let mut cold_heard = false;
    for holy in [7u64, 14, 21, 28, 35, 42] {
        g.livelihood_tick(holy);
        for (v, flag) in [(devout, &mut devout_heard), (cold, &mut cold_heard)] {
            if g.world.get_component::<routine::Routine>(v)
                .map(|r| r.mode == routine::Mode::Worshipping && r.day == holy).unwrap_or(false) {
                *flag = true;
            }
        }
    }
    assert!(devout_heard, "the devout soul never made it to prayers");
    assert!(!cold_heard, "the cold of faith heard a call that wasn't theirs");
}

#[test]
fn born_and_arriving_souls_carry_virtues() {
    let mut g = Game::new();
    // a settler steps in at the wall
    g.maybe_migrant(0);
    let m = *g.villagers.last().unwrap();
    assert!(g.world.get_component::<Virtues>(m).is_some(),
        "the newcomer arrived with no soul");
    // a child born to a full house
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|bb| bb.kind == BuildingKind::Home).unwrap_or(false)).unwrap();
    g.spawn_kid(home, 0);
    let k = *g.villagers.last().unwrap();
    assert!(g.world.get_component::<Virtues>(k).is_some(),
        "the newborn came into the world with no soul");
}
