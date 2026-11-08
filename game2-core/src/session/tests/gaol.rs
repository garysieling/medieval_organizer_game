use crate::session::*;

#[test]
fn test_the_held_hold_at_their_cell() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.village_gaol = Some((10, 10));
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 50, charge: "sedition".into() });
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 30.0, y: 30.0 };
    g.gaol_tick(51);
    let p = g.world.get_component::<Pos>(v).unwrap();
    assert_eq!((p.x as i32, p.y as i32), (10, 10), "the held aren't at their cell");
    assert!(crate::game::dungeon::unavailable(&g.world, v), "the cell lets them out of everything");
}

#[test]
fn test_the_door_is_honest_and_the_wall_is_not() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.village_gaol = Some((10, 10));
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 50, charge: "sedition".into() });
    // enough ticks and a small seeded roll slips them free
    for d in 0..200 {
        g.gaol_tick(51 + d);
        if g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none() { break; }
    }
    assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
        "two hundred days and nobody slipped the cell");
}

#[test]
fn test_the_cell_lets_go_and_they_walk_home() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 50, charge: "sedition".into() });
    // at the gaol, away from home - the walk out has somewhere to go
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 10.0, y: 10.0 };
    g.free_prisoner(v, 60);
    assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none());
    assert!(g.travels.iter().any(|t| t.who == v), "the cell let go and nobody walked out");
}
