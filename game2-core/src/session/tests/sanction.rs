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
