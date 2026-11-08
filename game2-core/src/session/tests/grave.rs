use crate::session::*;

#[test]
fn test_the_dead_rest_in_the_graveyard() {
    let mut game = Game::new();
    game.speed = 8;
    let bram = game.villagers[2];
    game.world.get_component_mut::<Health>(bram).unwrap().condition = 0.0;
    while game.tick < 5 { game.update(); } // the death check turns them into a grave
    assert!(villager::is_dead(&game.world, bram));
    // the lying-in holds them where they fell a few days, then the
    // funeral carries them to the churchyard - once, and never again
    while game.tick < 60 { game.update(); }
    assert!(game.world.get_component::<villager::Buried>(bram).is_some(),
        "the funeral never came for them");
    let rest = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
    // their grave is one of the churchyard's plots (it may be the tile
    // they happened to fall on - the funeral still came for them)
    let gy = game.graveyard.as_ref().expect("the churchyard was planned");
    assert!(gy.slots.contains(&(rest.0.round() as i32, rest.1.round() as i32)),
        "the grave isn't in the churchyard");
    while game.tick < 200 { game.update(); }
    let now = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
    assert_eq!(now, rest, "a grave wandered");
}
