use crate::session::*;

fn grieve(g: &mut Game, v: Entity, n: usize, tick: u64) {
    let kinds = [desires::Problem::Blamed, desires::Problem::Broke,
        desires::Problem::Smelly, desires::Problem::SourGut,
        desires::Problem::NoHome, desires::Problem::Taxed];
    for i in 0..n {
        desires::add_grievance(&mut g.world, v, kinds[i % kinds.len()], tick + 60);
    }
}

#[test]
fn test_the_ladder_climbs_one_rung_at_a_time() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    grieve(&mut g, v, 13, 4);
    g.sanction_tick(4);
    // thirteen grievances earn the stocks, but the ladder is climbed -
    // the first landing is only the talk
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Gossip),
        "it starts with a bad word");
    assert!(!g.sanctioned.iter().any(|(e, s, _)| *e == v && *s > Sanction::Gossip),
        "nobody goes straight to the stocks");
}

#[test]
fn test_the_boycott_dries_up_the_shop() {
    let mut g = Game::new();
    let shop = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b).map(|b| b.worker.is_some()).unwrap_or(false)).unwrap();
    let worker = g.world.get_component::<Building>(shop).unwrap().worker.unwrap();
    assert!(!g.boycotted(shop));
    g.sanctioned.push((worker, Sanction::Boycott, 100));
    assert!(g.boycotted(shop), "a boycotted shop's trade dries up");
    assert!(g.shunned(worker), "the boycott is worse than the cold shoulder");
}

#[test]
fn test_a_respected_soul_walks_free_of_the_stocks() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Esteemed, score: 0.8, transition_rates: vec![] });
    let before = g.world.get_component::<ReputationState>(v).unwrap().score;
    g.apply_sanction(v, Sanction::Stocks, 10);
    let after = g.world.get_component::<ReputationState>(v).unwrap().score;
    assert!(after > before, "the crowd's sympathy lifts the wrongly shamed");
    assert!(g.log.iter().any(|l| l.contains("would have none of it")));
}

#[test]
fn test_the_worst_walk_out() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.apply_sanction(v, Sanction::Exile, 10);
    assert!(g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Exile)),
        "the exiled take the road");
    // and at the world's edge they're dead to the village
    g.arrive(v, Goal::Exile);
    assert!(villager::is_dead(&g.world, v), "gone is gone");
}

#[test]
fn test_the_card_names_the_rung() {
    let mut g = Game::new();
    let i = g.villagers.iter().position(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    let v = g.villagers[i];
    g.sanctioned.push((v, Sanction::Shun, 10));
    g.tick = 5;
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    let ins = g.ui_model_at(0).inspector.expect("no card");
    assert!(ins.lines.iter().any(|l| l.text.contains("the cold shoulder")),
        "the card doesn't say where they stand: {:?}",
        ins.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_the_stocks_hold_their_body() {
    let mut g = Game::new();
    let v = g.villagers[0];
    // no crowd to save them - the shame sticks
    g.world.get_component_mut::<crate::game::contracts::Reputation>(v)
        .map(|r| r.trust_score = 0.0);
    g.apply_sanction(v, crate::game::sanctions::Sanction::Stocks, 50);
    assert!(g.stocks_tile.is_some(), "no stocks tile picked");
    assert!(g.travels.iter().any(|t| t.who == v), "the condemned never walked");
    // the walk lands them in the stocks
    for d in 0..10 { g.stocks_tick(50 + d); }
    // the walk may still be in flight - force the arrive
    g.world.add_component(v, villager::InStocks { until: 57 });
    // pinned and fed: their body stays, their belly's seen to
    g.stocks_tick(52);
    assert!(g.world.get_component::<villager::InStocks>(v).is_some(), "they slipped the stocks");
    assert!(!g.send(v, Goal::HeadHome(g.market_food)), "a stocked body took an errand");
    // the sentence runs out
    g.stocks_tick(58);
    assert!(g.world.get_component::<villager::InStocks>(v).is_none(), "the sentence ran and nobody freed them");
    // but the sanction's memory stands its sixty days
    assert!(g.sanctioned.iter().any(|(e, _s, u)| *e == v && *u >= 100),
        "the shame didn't outlast the stocks");
}

#[test]
fn test_a_lapsed_shame_doesnt_reopen_on_the_same_sins() {
    // #1141: a villager's pile is answered once - the cold shoulder
    // lapses and stays lapsed until they earn *fresh* blame
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    let mut grieve_long = |n: usize, tick: u64| {
        let kinds = [desires::Problem::Blamed, desires::Problem::Broke,
            desires::Problem::Smelly, desires::Problem::SourGut];
        for i in 0..n {
            desires::add_grievance(&mut g.world, v, kinds[i % kinds.len()], tick + 120);
        }
    };
    grieve_long(4, 4);
    g.sanction_tick(4);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Gossip),
        "the pile earns its bad word");
    // the word lapses - and the same stale pile can't earn it again
    g.sanction_tick(67);
    assert!(!g.sanctioned.iter().any(|(e, _, _)| *e == v),
        "the same sins shouldn't bring the shame back");
}

#[test]
fn test_fresh_blame_reopens_the_ladder() {
    // #1141's other half: a bigger heap is a new offense, not the old
    // one returning - the village answers it again
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    let kinds = [desires::Problem::Blamed, desires::Problem::Broke,
        desires::Problem::Smelly, desires::Problem::SourGut,
        desires::Problem::NoHome, desires::Problem::Taxed];
    for i in 0..4 {
        desires::add_grievance(&mut g.world, v, kinds[i], 4 + 120);
    }
    g.sanction_tick(4);
    // it lapses, then the pile grows past what the village answered
    g.sanction_tick(67);
    assert!(!g.sanctioned.iter().any(|(e, _, _)| *e == v), "the lapse should stand");
    desires::add_grievance(&mut g.world, v, kinds[4], 67 + 120);
    g.sanction_tick(74);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Gossip),
        "fresh blame earns a fresh bad word");
}

#[test]
fn test_a_village_all_miserable_together_shames_no_one() {
    // #1141: the cold shoulder lands on the soul who stands apart -
    // two souls grieved alike share the misery, nobody's a wretch
    let mut g = Game::new();
    let mut souls = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<TownRole>(v).is_none());
    let (a, b) = (souls.next().unwrap(), souls.next().unwrap());
    let kinds = [desires::Problem::Blamed, desires::Problem::Broke,
        desires::Problem::Smelly, desires::Problem::SourGut,
        desires::Problem::NoHome, desires::Problem::Taxed];
    for i in 0..4 { desires::add_grievance(&mut g.world, a, kinds[i], 4 + 60); }
    for i in 0..3 { desires::add_grievance(&mut g.world, b, kinds[i], 4 + 60); }
    g.sanction_tick(4);
    assert!(g.sanctioned.is_empty(),
        "4 against the next soul's 3 is no clear wretch - the village just suffers");
    // pull one clear of the rest and the village finds its mark
    desires::add_grievance(&mut g.world, a, kinds[4], 4 + 60);
    desires::add_grievance(&mut g.world, a, kinds[5], 4 + 60);
    g.sanction_tick(11);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == a && *s == Sanction::Gossip),
        "the soul who stands apart earns the talk");
}

#[test]
fn test_the_gauge_counts_souls_under_the_shoulder() {
    // #1141: the KPI's "sanctioned" is souls living the cold shoulder
    // or worse - a bad word going round isn't a sanction, and
    // overlapping rungs are still the one soul
    let mut g = Game::new();
    let mut souls = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<TownRole>(v).is_none());
    let (a, b) = (souls.next().unwrap(), souls.next().unwrap());
    g.tick = 5;
    g.sanctioned.push((a, Sanction::Gossip, 60));
    g.sanctioned.push((b, Sanction::Shun, 60));
    g.sanctioned.push((b, Sanction::Boycott, 60));
    let n = g.secret_kpis().iter().find(|(k, _)| *k == "sanctioned").unwrap().1;
    assert_eq!(n, 1.0, "one soul under the shoulder, the gossip doesn't count");
}

/// A bond written straight onto the record: the interaction creates the
/// entity, then the kind and strength are set - the reckoning reads the
/// ledger, not the history (#1255)
fn bond(g: &mut Game, a: Entity, b: Entity,
    kind: crate::game::relationships::RelKind, strength: f32) {
    crate::game::relationships::record_interaction(&mut g.world, a,
        crate::game::relationships::Interaction { other: b,
            kind: crate::game::relationships::InteractionKind::Conversation,
            intensity: 0.1 });
    let re = crate::game::relationships::find_relationship(&g.world, a, b)
        .expect("the bond wrote");
    let r = g.world.get_component_mut::<crate::game::relationships::Relationship>(re).unwrap();
    r.kind = kind;
    r.strength = strength;
}

/// A friend steps up beside him - and wears the mark of it after (#1255)
#[test]
fn test_the_friend_steps_up_at_the_stocks() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let f = it.next().unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    bond(&mut g, v, f, crate::game::relationships::RelKind::Friend, 0.9);
    g.apply_sanction(v, Sanction::Stocks, 10);
    // the bond that steps up begs first - let the plea fall so the
    // walk can happen
    if g.plea.is_some() { g.decide_at(0, 1); }
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(f).is_some(),
        "the stander's marked");
    assert!(g.log.iter().any(|l| l.contains("step") && l.contains("melt into the crowd")),
        "the village reads who stood: {:?}", g.log.iter().last());
    assert!(!g.log.iter().any(|l| l.contains("nobody steps up")));
}

/// A bond too thin for what standing costs melts into the crowd - and
/// the melted carry the grief of it (#1255)
#[test]
fn test_the_thin_bond_melts() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let f = it.next().unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    bond(&mut g, v, f, crate::game::relationships::RelKind::Friend, 0.4);
    g.apply_sanction(v, Sanction::Stocks, 10);
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(f).is_none(),
        "a thin bond doesn't stand");
    assert!(g.log.iter().any(|l| l.contains("nobody steps up")),
        "the crowd reads that too");
    assert!(desires::problems_of(&g.world, f, 15).contains(&desires::Problem::Grief),
        "they watched themselves melt - and it sits with them");
}

/// Exposure outweighs the bond: a friend the crowd already blames
/// doesn't step up (#1255)
#[test]
fn test_the_exposed_melt() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let f = it.next().unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    bond(&mut g, v, f, crate::game::relationships::RelKind::Friend, 0.7);
    grieve(&mut g, f, 4, 8);
    g.apply_sanction(v, Sanction::Stocks, 10);
    if g.plea.is_some() { g.decide_at(0, 1); }
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(f).is_none(),
        "most to fear means melting");
    assert!(g.log.iter().any(|l| l.contains("nobody steps up")));
}

/// The road out draws the same reckoning - and his kin warm toward the
/// one who stood (#1255)
#[test]
fn test_the_road_out_reckons_and_kin_remember() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let f = it.next().unwrap();
    let k = it.next().unwrap();
    bond(&mut g, v, f, crate::game::relationships::RelKind::Friend, 0.9);
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    // the kin's own bond holds too, or they'd be counted a stander
    g.reckoning(v, "the road out");
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(f).is_some());
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(k).is_some(),
        "kin stand beside their own");
    let kf = crate::game::relationships::find_relationship(&g.world, k, f)
        .and_then(|re| g.world.get_component::<crate::game::relationships::Relationship>(re));
    assert!(kf.map(|r| r.strength > 0.0).unwrap_or(false),
        "his kin remember who stood");
}

/// The organizer's own reckoning: a member on the walk puts the fork on
/// the card (#1255)
#[test]
fn test_the_organizer_stands_or_keeps_distance() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    // a stranger to the cause walks it without the card
    g.apply_sanction(v, Sanction::Stocks, 10);
    assert!(g.dilemma.is_none() && g.reckoning.is_none(),
        "no member, no organizer's fork");
    // a member on the walk is asked for
    let oe = g.org;
    g.world.get_component_mut::<crate::sim::community::Organization>(oe)
        .unwrap().members.push(v);
    g.apply_sanction(v, Sanction::Stocks, 10);
    assert!(g.dilemma.is_some() && g.reckoning == Some(v),
        "the card asks who walks beside your own");
    let before = g.penetration;
    g.decide_at(0, 0);
    assert!(g.world.get_component::<crate::game::sanctions::StoodBy>(g.player).is_some(),
        "you're marked as theirs");
    assert!(g.penetration > before, "the informers read the leader's loyalty");
    assert!(g.log.iter().any(|l| l.contains("the whole village sees whose you are")));
    // and the quiet choice costs nothing but a line
    let mut g2 = Game::new();
    let v2 = g2.villagers.iter().copied().find(|&v|
        g2.world.get_component::<TownRole>(v).is_none()).unwrap();
    g2.world.get_component_mut::<crate::sim::community::Organization>(g2.org)
        .unwrap().members.push(v2);
    g2.reckoning(v2, "the stocks");
    assert!(g2.dilemma.is_some());
    g2.decide_at(0, 1);
    assert!(g2.log.iter().any(|l| l.contains("walks it alone")),
        "the distance is what the village reads");
    assert!(g2.world.get_component::<crate::game::sanctions::StoodBy>(g2.player).is_none());
}

/// Kin begs at the stocks - the sentence waits on the card (#1256)
#[test]
fn test_kin_begs_and_the_sentence_waits() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let k = it.next().unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    g.apply_sanction(v, Sanction::Stocks, 10);
    assert!(g.plea.map(|p| p.condemned == v && p.pleader == k).unwrap_or(false),
        "her voice waits on the card");
    assert!(g.dilemma.is_some(), "the fork's asked");
    assert!(!g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Stocks),
        "the stocks wait on the answer");
    assert!(g.log.iter().any(|l| l.contains("begs mercy")));
}

/// The beloved's plea lands: the rung steps down the ladder (#1256)
#[test]
fn test_a_beloved_thief_gets_a_lighter_rung() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let k = it.next().unwrap();
    // beloved pleader, middling condemned: mercy lands, but not so
    // loved that the crowd refuses the softened stocks outright
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.5, transition_rates: vec![] });
    g.world.add_component(k, ReputationState {
        stage: RepStage::Esteemed, score: 0.9, transition_rates: vec![] });
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    g.apply_sanction(v, Sanction::Exile, 10);
    assert!(g.plea.is_some(), "the plea waits");
    g.decide_at(0, 0);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Stocks),
        "mercy softens the road out to the stocks");
    assert!(!g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Exile));
    assert!(g.log.iter().any(|l| l.contains("sentence softens")));
}

/// The hated one's plea falls flat - the sentence lands as read, and
/// the begging leaves its bruise (#1256)
#[test]
fn test_a_hated_ones_plea_falls_flat() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let k = it.next().unwrap();
    for &e in &[v, k] {
        g.world.add_component(e, ReputationState {
            stage: RepStage::Unknown, score: 0.1, transition_rates: vec![] });
    }
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    g.apply_sanction(v, Sanction::Exile, 10);
    g.decide_at(0, 0);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Exile),
        "the bench doesn't bend for a hated name");
    assert!(g.log.iter().any(|l| l.contains("plea falls flat")));
    assert!(desires::problems_of(&g.world, k, 15).contains(&desires::Problem::Grief),
        "the refused begging sits with the beggar");
}

/// Letting it stand costs the pleader's warmth toward the voice that
/// let it (#1256)
#[test]
fn test_let_it_stand_chills_the_pleader() {
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let k = it.next().unwrap();
    for &e in &[v, k] {
        g.world.add_component(e, ReputationState {
            stage: RepStage::Unknown, score: 0.1, transition_rates: vec![] });
    }
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    g.apply_sanction(v, Sanction::Exile, 10);
    g.decide_at(0, 1);
    let kf = crate::game::relationships::find_relationship(&g.world, k, g.player)
        .and_then(|re| g.world.get_component::<crate::game::relationships::Relationship>(re));
    assert!(kf.map(|r| r.strength < 0.0).unwrap_or(false),
        "the line you take is remembered");
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Exile));
}

/// No bonded voice, no plea - the sentence just lands (#1256)
#[test]
fn test_no_voice_means_no_plea() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.world.add_component(v, ReputationState {
        stage: RepStage::Known, score: 0.3, transition_rates: vec![] });
    g.apply_sanction(v, Sanction::Stocks, 10);
    assert!(g.plea.is_none() && g.dilemma.is_none(), "nobody begs for a stranger");
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Stocks));
}

/// The rope waits on the crowd too: mercy turns the gibbet to the
/// cells, a flat plea lands the hanging (#1256)
#[test]
fn test_the_gibbet_hears_the_plea() {
    // mercy: beloved condemned, beloved pleader
    let mut g = Game::new();
    let mut it = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<TownRole>(v).is_none());
    let v = it.next().unwrap();
    let k = it.next().unwrap();
    for &e in &[v, k] {
        g.world.add_component(e, ReputationState {
            stage: RepStage::Esteemed, score: 0.9, transition_rates: vec![] });
    }
    bond(&mut g, v, k, crate::game::relationships::RelKind::Family, 0.9);
    assert!(g.try_plea(v, crate::game::sanctions::PleaSentence::Gibbet), "the rope waits");
    g.decide_at(0, 0);
    assert!(!villager::is_dead(&g.world, v), "the coil stays wound");
    assert!(g.dungeon.iter().any(|(e, _)| *e == v),
        "the cells take them - a ransom can still reach");
    assert!(g.log.iter().any(|l| l.contains("the castle's cells")));
    // and a flat plea lands the hanging as read
    let mut g2 = Game::new();
    let mut it2 = g2.villagers.iter().copied().filter(|&v|
        g2.world.get_component::<TownRole>(v).is_none());
    let v2 = it2.next().unwrap();
    let k2 = it2.next().unwrap();
    for &e in &[v2, k2] {
        g2.world.add_component(e, ReputationState {
            stage: RepStage::Unknown, score: 0.1, transition_rates: vec![] });
    }
    bond(&mut g2, v2, k2, crate::game::relationships::RelKind::Family, 0.9);
    assert!(g2.try_plea(v2, crate::game::sanctions::PleaSentence::Gibbet));
    g2.decide_at(0, 0);
    assert!(villager::is_dead(&g2.world, v2), "the bench doesn't bend");
    assert!(g2.log.iter().any(|l| l.contains("hanged for the defiance")));
}
