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
