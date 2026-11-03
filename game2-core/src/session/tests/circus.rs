use crate::session::*;

#[test]
fn test_the_circus_camps_and_tempts() {
    let mut g = Game::new();
    g.spawn_circus();
    // walk it in
    for _ in 0..600 {
        if g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false) { break; }
        g.advance_circus();
    }
    assert!(g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false),
        "the tents go up on the meadow");
    // a villager by the camp gets the wonder and feels the pull
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    let camp = g.circus.as_ref().unwrap().camp;
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: camp.1 as f32 };
    g.circus_tick(1);
    assert!(desires::problems_of(&g.world, v, 1).contains(&desires::Problem::Tempted),
        "the bored feel the road's pull");
}

#[test]
fn test_the_loneliest_runs_off_with_it() {
    let mut g = Game::new();
    g.spawn_circus();
    for _ in 0..600 {
        if g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false) { break; }
        g.advance_circus();
    }
    // tempt someone, then let the tents fold
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    desires::add_grievance(&mut g.world, v, desires::Problem::Tempted, 100);
    let until = g.circus.as_ref().unwrap().camped_until.unwrap();
    g.tick = until;
    g.circus_tick(until);
    assert!(g.circus.is_none(), "the tents fold");
    if g.travels.iter().any(|t| matches!(t.goal, Goal::Exile)) {
        assert!(g.log.iter().any(|l| l.contains("run off with the circus")),
            "the road takes the one it tempted");
    }
}
