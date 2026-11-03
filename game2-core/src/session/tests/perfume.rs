use crate::session::*;

#[test]
fn test_the_perfumer_keeps_the_still() {
    let g = Game::new();
    let p = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Perfumer).unwrap_or(false));
    assert!(p.is_some(), "a perfumer keeps the still");
    // Hungary water: garden flowers steeped in the water of life
    assert_eq!(vocation::inputs(Vocation::Perfumer), &["marigold", "lavender", "spirits"]);
    assert_eq!(vocation::output(Vocation::Perfumer), Some("perfume"));
}

#[test]
fn test_a_dab_of_perfume_smells_of_roses() {
    let mut g = Game::new();
    let buyer = g.villagers[0];
    let shop = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b).map(|bb| bb.worker.is_some()).unwrap_or(false)).unwrap();
    // stock the shelf, give the buyer a purse, and let them buy
    g.world.get_component_mut::<Building>(shop).unwrap()
        .stock.insert("perfume".to_string(), 2);
    g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 30.0;
    g.arrive(buyer, Goal::BuyFrom(shop, "perfume", 1));
    assert_eq!(g.smells_of(buyer), Some("roses"), "the buyer smells of roses");
    // and perfume is no smell at all to the heart-helper
    let smelly = [buyer].iter().any(|&v|
        g.smells_of(v).map(|k| k != "roses").unwrap_or(false));
    assert!(!smelly, "a dab of roses counts for a clean suitor");
}
