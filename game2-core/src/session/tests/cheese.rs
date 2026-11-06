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

#[test]
fn test_the_cellars_card_names_its_cheese() {
    let mut g = Game::new();
    g.tick = 80;
    g.cheeses = vec![5, 30, 60]; // three wheels down, the oldest since day 5
    let bi = g.villagers.len(); // buildings sit past the figures? no - find its index
    let cellar_i = g.building_entities.iter().position(|&b| b == g.cellar.unwrap()).unwrap();
    let _ = bi;
    g.seats[0].selected_building = Some(cellar_i);
    g.seats[0].picked = false;
    let m = g.ui_model_at(0);
    let ins = m.inspector.expect("no card for the cellar");
    assert!(ins.lines.iter().any(|l| l.text.contains("stone-cellar - 3 wheels, oldest 75d")),
        "the card doesn't name the rack: {:?}",
        ins.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}
