use crate::session::*;

#[test]
fn test_a_writ_levies_the_store() {
    let mut g = Game::new();
    if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
        household::add_stock(b, "grain", 20);
    }
    g.edict_tick(80);
    assert!(g.log.iter().any(|l| l.contains("bailiff's writ")), "a writ comes down");
    let grain = g.world.get_component::<Building>(g.market_food)
        .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0);
    assert!(grain < 20, "the levy takes its share");
    let grieved = g.villagers.iter().filter(|&&v|
        desires::problems_of(&g.world, v, 80).contains(&desires::Problem::WritBorne)).count();
    assert!(grieved > 0, "the writ's a thing to organize about");
}

#[test]
fn test_a_strong_org_stands_against_the_summons() {
    let mut g = Game::new();
    g.lord_esteem = 0.8;
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    for _ in 0..4 { desires::add_grievance(&mut g.world, m, desires::Problem::Broke, 100); }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
    // summons day: tick 320 -> (320/120)%3 == 2
    g.edict_tick(320);
    assert!(g.log.iter().any(|l| l.contains("stands against")),
        "a strong org says no");
    assert!(g.repression_debt > 0, "the defiance will be paid for");
    assert!(!g.travels.iter().any(|t| t.who == m && matches!(t.goal, Goal::Exile)),
        "he stays - for now");
}

#[test]
fn test_a_hanging_radicalizes_the_witnesses() {
    let mut g = Game::new();
    g.lord_esteem = 0.8;
    g.repression_debt = 2;
    let m = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    for _ in 0..4 { desires::add_grievance(&mut g.world, m, desires::Problem::Broke, 100); }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
    // the debt comes due: the third defiance is a gibbet, and everyone
    // who saw it carries the spark
    g.edict_tick(320);
    assert!(g.world.get_component::<villager::Dead>(m).is_some(), "the writ hangs him");
    let marked = g.villagers.iter().filter(|&&v|
        g.world.get_component::<crate::game::reactions::Radicalized>(v).is_some()).count();
    assert!(marked > 0, "the wronged remember - {} marked", marked);
    assert!(g.log.iter().any(|l| l.contains("the wronged remember")),
        "the village knows what it saw");
}

#[test]
fn test_a_debt_wrapped_writ_radicalizes_the_market() {
    let mut g = Game::new();
    g.repression_debt = 1;
    // a levy day: tick 80 -> kind 0, mult = 2 "Double, for the last defiance"
    g.edict_tick(80);
    assert!(g.log.iter().any(|l| l.contains("Double, for the last defiance")),
        "the debt comes down heavier");
    let marked = g.villagers.iter().filter(|&&v|
        g.world.get_component::<crate::game::reactions::Radicalized>(v).is_some()).count();
    assert!(marked > 0, "a writ seen is a writ remembered");
}

#[test]
fn test_the_wronged_recruit_easier() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Organization>(g.org).map(|o| !o.members.contains(&v)).unwrap_or(true)
            && !villager::is_dead(&g.world, v)).unwrap();
    let org = g.org;
    // below the trust bar: refused
    assert!(!crate::sim::community::recruit(&mut g.world, org, v, 0.3),
        "stranger's trust isn't enough");
    // the spark of repression covers the gap: recruits arrive on a delay
    g.world.add_component(v, crate::game::reactions::Radicalized { until: 9999 });
    assert!(crate::sim::community::recruit(&mut g.world, org, v, 0.3),
        "the wronged sign on with less persuading");
}

#[test]
fn test_the_spark_fades() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::reactions::Radicalized { until: 50 });
    g.radical_decay(49);
    assert!(g.world.get_component::<crate::game::reactions::Radicalized>(v).is_some(),
        "the spark still burns");
    g.radical_decay(50);
    assert!(g.world.get_component::<crate::game::reactions::Radicalized>(v).is_none(),
        "un-caught, it fades");
}
