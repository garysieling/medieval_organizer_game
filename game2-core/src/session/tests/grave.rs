use crate::session::*;

#[test]
fn test_a_grave_stays_where_the_villager_died() {
    let mut game = Game::new();
    let bram = game.villagers[2];
    game.world.get_component_mut::<Health>(bram).unwrap().condition = 0.0;
    while game.tick < 5 { game.update(); } // the death check turns them into a grave
    assert!(villager::is_dead(&game.world, bram));
    let at = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
    while game.tick < 200 { game.update(); }
    let now = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
    assert_eq!(now, at, "the dead don't wander or get jostled");
}
