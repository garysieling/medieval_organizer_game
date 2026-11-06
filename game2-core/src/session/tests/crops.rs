use crate::session::*;
use crate::engine::ecs::System;
use crate::game::villager::*;

#[test]
fn test_the_fields_ripen_and_feed_the_closest() {
    let g = Game::new();
    // the fields stand around the edge
    let fields: Vec<Entity> = g.world.query::<Crop>();
    assert!(!fields.is_empty(), "no fields planted");
    // each carries a food source that starts empty of growth credit
    let f = fields[0];
    let s0 = g.world.get_component::<FoodSource>(f).map(|x| x.servings).unwrap_or(0);
    let mut g = g;
    // a spell of growing days swells the stock
    for t in 1..20 { g.crop_tick(t); }
    let s1 = g.world.get_component::<FoodSource>(f).map(|x| x.servings).unwrap_or(0);
    let grown = g.world.get_component::<Crop>(f).map(|c| c.grown).unwrap_or(0);
    assert!(s1 > s0 && grown > 0, "nothing ripened: {} -> {} (grown {})", s0, s1, grown);
    assert!(g.log.iter().any(|l| l.contains("ears come in")), "the first harvest went unremarked");
    // and the seeking can actually eat from a field
    let v = g.villagers[0];
    g.world.get_component_mut::<Pos>(v).unwrap().x =
        g.world.get_component::<Pos>(f).unwrap().x;
    g.world.get_component_mut::<Pos>(v).unwrap().y =
        g.world.get_component::<Pos>(f).unwrap().y;
    g.world.get_component_mut::<villager::Hunger>(v).unwrap().value = 20.0;
    let before = g.world.get_component::<FoodSource>(f).unwrap().servings;
    SeekFoodSystem.run(&mut g.world);
    let after = g.world.get_component::<FoodSource>(f).unwrap().servings;
    assert!(after < before, "nobody ate from the field");
}
