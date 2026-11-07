use crate::session::*;

#[test]
fn test_he_pitches_outside_the_walls() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    assert!(g.ink_boiler.is_some());
    // walk him out to the meadow
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let b = g.ink_boiler.as_ref().unwrap();
    assert!(b.working_until.is_some(), "the cauldron's set up");
    // the law wants a fire like his beyond the south wall
    assert!(b.camp.1 > g.town_wall.3, "he boils outside the gate, not in the street");
    assert!(!g.log.iter().any(|l| l.is_empty()));
}

#[test]
fn test_the_road_takes_him_too() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.path.is_empty() && b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    // run him past his boiling days
    g.tick = 100;
    for _ in 0..2000 {
        g.advance_ink_boiler();
        if g.ink_boiler.is_none() { break; }
    }
    assert!(g.ink_boiler.is_none(), "the road takes him");
    assert!(g.next_ink_boiler > g.tick, "he'll be back in a year or so");
}

#[test]
fn test_he_shows_up_on_the_map() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    let e = g.ink_boiler.as_ref().unwrap().e;
    let model = g.ui_model();
    assert!(model.people.iter().any(|p| p.e == e && matches!(p.kind, PKind::Traveller)),
        "the ink-maker is on the map the moment he's spawned, not just his speech bubble");
}

#[test]
fn test_the_recipe_walks_its_days() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
    for d in 0..INK_BOIL_DAYS { g.ink_boiler_tick(pitched + d); }
    assert!(g.log.iter().any(|l| l.contains("varnish")), "the slate says varnish");
    // the print shop - or the trader if there's no printer - is stocked
    let stocked = g.building_entities.iter().copied().find(|&e| {
        g.world.get_component::<Building>(e)
            .map(|b| matches!(b.kind, household::BuildingKind::Shop(Vocation::Printmaker)))
            .unwrap_or(false)
    }).or(g.trader_stall).unwrap();
    let ink = g.world.get_component::<Building>(stocked)
        .and_then(|b| b.stock.get("ink").copied()).unwrap_or(0);
    assert!(ink >= INK_YIELD, "there's ink where the road used to bring it");
}

#[test]
fn test_the_crusts_come_back_fried() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let (camp, be) = {
        let b = g.ink_boiler.as_ref().unwrap();
        (b.camp, b.e)
    };
    // someone walks out with the day's crusts
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 - 2) as f32 };
    let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
    g.tick = pitched + 2;
    g.ink_boiler_tick(g.tick);
    assert!(g.travels.iter().any(|t| matches!(t.goal, Goal::TalkTo(e) if e == be)),
        "the crusts get carried out to the pot");
    // and the bearer gets the taste of them
    g.arrive(v, Goal::TalkTo(be));
    assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
        "they come back fried");
}

#[test]
fn test_the_meadow_can_burn() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let camp = g.ink_boiler.as_ref().unwrap().camp;
    assert!(g.fire.ignite(&g.map, camp.0, camp.1, g.tick), "the cauldron's tile can take fire");
    // it runs its course and leaves scorched ground behind
    let mut scorched_any = false;
    for t in 1..40 {
        g.fire_tick(t);
        if !g.fire.scorched.is_empty() { scorched_any = true; break; }
    }
    assert!(scorched_any, "it burned out to scorched ground");
}

#[test]
fn test_the_smoke_reaches_the_downwind() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let camp = g.ink_boiler.as_ref().unwrap().camp;
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 - 3) as f32 };
    let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
    g.tick = pitched;
    g.ink_boiler_tick(g.tick);
    assert!(desires::problems_of(&g.world, v, g.tick).contains(&desires::Problem::Smelly),
        "the acrolein's sharp on the eyes");
}

#[test]
fn test_folk_beside_the_flames_run_for_home() {
    let mut g = Game::new();
    g.spawn_ink_boiler();
    for _ in 0..600 {
        if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
        g.advance_ink_boiler();
    }
    let camp = g.ink_boiler.as_ref().unwrap().camp;
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 + 1) as f32 };
    g.fire.ignite(&g.map, camp.0, camp.1, g.tick);
    g.fire_tick(1);
    assert!(g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::HeadHome(_))),
        "she runs for home");
    // and mid-errand is no excuse to walk through flames
    let v2 = g.villagers.iter().copied().find(|&x| x != v
        && g.world.get_component::<TownRole>(x).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v2).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 + 1) as f32 };
    g.travels.push(Travel { who: v2, path: vec![(5, 5)], goal: Goal::Wander });
    g.fire_tick(2);
    assert!(g.travels.iter().all(|t| t.who != v2 || matches!(t.goal, Goal::HeadHome(_))),
        "the errand's dropped for the run home");
}
