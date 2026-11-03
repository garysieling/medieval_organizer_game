use crate::session::*;

#[test]
fn test_the_cellar_ages_wheels() {
    let mut g = Game::new();
    assert!(g.cellar.is_some(), "a cellar stands by the fold");
    assert_eq!(g.world.get_component::<Building>(g.cellar.unwrap()).unwrap().kind,
        household::BuildingKind::Cellar);
    // milk in the fold makes wheels in the dark
    if let Some(b) = g.world.get_component_mut::<Building>(g.fold.unwrap()) {
        household::add_stock(b, "milk", 6);
    }
    for t in [6u64, 13, 20] { g.cheese_tick(t); }
    assert!(!g.cheeses.is_empty(), "milk down becomes wheels");
}

#[test]
fn test_a_noble_pays_triple_for_an_aged_wheel() {
    let mut g = Game::new();
    g.cheeses = vec![0]; // a wheel laid down at the start of time
    let sh = g.shepherd.unwrap();
    g.world.get_component_mut::<Purse>(sh).unwrap().gold = 0.0;
    g.cheese_tick(75);
    let gold = g.world.get_component::<Purse>(sh).unwrap().gold;
    assert!(gold > vocation::price("cheese") * 2.0, "aged cheese pays triple");
    assert!(g.log.iter().any(|l| l.contains("stone-cellar")),
        "the village's cheese has a name");
}
