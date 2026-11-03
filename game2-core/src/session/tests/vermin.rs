use crate::session::*;

#[test]
fn test_bread_spoils_and_grain_holds() {
    let mut g = Game::new();
    let be = g.building_entities[5]; // the trader's stall, well-stocked
    if let Some(b) = g.world.get_component_mut::<Building>(be) {
        household::add_stock(b, "bread", 4); household::add_stock(b, "wheat", 4);
    }
    g.spoilage_tick(3); // the weekly rot - tick 3 lands on the wheat's turn too
    g.spoilage_tick(10); // and the next week's
    let b = g.world.get_component::<Building>(be).unwrap();
    assert_eq!(b.stock.get("bread").copied().unwrap_or(0), 2, "bread goes in a week");
    assert_eq!(b.stock.get("wheat").copied().unwrap_or(0), 3, "wheat goes in a month");
}

#[test]
fn test_a_hunting_cat_walks_the_yard() {
    let g = Game::new();
    // Mog hunts for villagers[1] - their home is watched
    let home = household::home_of(&g.world, g.villagers[1]);
    if let Some(h) = home {
        assert!(g.cat_watches(h), "the cat keeps the vermin off");
    }
    // and some building nowhere near a cat is not watched
    assert!(!g.cat_watches(g.building_entities[4]), "the church has no cat");
}

#[test]
fn test_a_poor_soul_eats_what_shouldnt_be_eaten() {
    let mut g = Game::new();
    let v = g.villagers[6];
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 0.5;
    g.world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;
    g.spoilage_tick(4);
    assert!(desires::problems_of(&g.world, v, 4).contains(&desires::Problem::SourGut),
        "a poor soul carries the sour gut");
}
