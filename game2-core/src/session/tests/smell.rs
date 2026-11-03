use crate::session::*;

#[test]
fn test_the_fold_smells_of_the_fold() {
    let mut g = Game::new();
    let sh = g.shepherd.unwrap();
    for _ in 0..5 { g.smell_tick(1); }
    assert_eq!(g.smells_of(sh), Some("the fold"), "a week of the fold tells on a body");
    // the rain washes everyone clean
    g.precip = Some(crate::sim::precipitation::PrecipKind::Rain);
    g.smell_tick(2);
    assert!(g.smells_of(sh).is_none(), "a good rain settles it");
}

#[test]
fn test_the_herb_wife_carries_bitter_herbs() {
    let mut g = Game::new();
    let w = g.herbologist.unwrap();
    for _ in 0..5 { g.smell_tick(1); }
    assert_eq!(g.smells_of(w), Some("bitter herbs"));
    // the tanner's clean by comparison - a farmer smells of nothing much
    let farmer = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)).unwrap();
    assert!(g.smells_of(farmer).is_none(), "a farmer smells of nothing much");
}
