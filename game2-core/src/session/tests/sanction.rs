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
