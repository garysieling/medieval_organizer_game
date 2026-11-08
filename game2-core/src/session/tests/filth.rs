use crate::session::*;

#[test]
fn test_the_leavings_pile_and_the_cart_carries_them_off() {
    let mut g = Game::new();
    // a few days in, the doorsteps have their leavings
    for d in 0..10 { g.filth_tick(d); }
    assert!(g.land.filth.values().any(|&f| f > 0.0), "nothing dropped");
    // a fouled doorstep: the cart hauls a share away each day, and the
    // leavings land downstream - while it sits it's a grievance inside
    let home = g.building_entities.iter().copied().find(|&b| {
        g.world.get_component::<crate::game::household::Building>(b)
            .map(|bb| bb.kind == crate::game::household::BuildingKind::Home
                && !bb.occupants.is_empty()).unwrap_or(false)
    }).expect("an occupied home");
    let bt = g.world.get_component::<Pos>(home)
        .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap();
    g.land.filth.insert(bt, 0.6);
    g.filth_tick(20);
    let after = g.land.filth.get(&bt).copied().unwrap_or(0.0);
    assert!(after < 0.6, "the cart never hauled");
    let occ = g.world.get_component::<crate::game::household::Building>(home)
        .unwrap().occupants[0];
    let grieved = g.world.get_component::<crate::game::desires::InnerLife>(occ)
        .map(|l| l.grievances.iter().any(|(p, _)| *p == crate::game::desires::Problem::StinkingReach))
        .unwrap_or(false);
    assert!(grieved, "no grievance on the doorstep's folk");
}

#[test]
fn test_what_lands_in_the_water_becomes_the_rivers_trouble() {
    let mut g = Game::new();
    // filth dumped at the river's edge turns to the river's foulness
    let river = g.land.effluent.keys().next().copied();
    if let Some(r) = river {
        g.land.filth.insert(r, 0.5);
        let e0 = g.land.effluent.get(&r).copied().unwrap_or(0.0);
        g.filth_tick(1);
        let e1 = g.land.effluent.get(&r).copied().unwrap_or(0.0);
        assert!(e1 > e0, "the dump never reached the water");
        assert!(g.land.filth.get(&r).copied().unwrap_or(0.0) < 0.5, "the dump never left");
    }
}
