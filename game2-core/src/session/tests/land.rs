use crate::session::*;

#[test]
fn test_gathering_thins_the_wood_and_the_undisturbed_regrow() {
    let mut g = Game::new();
    let who = g.villagers[0];
    g.world.get_component_mut::<Satchel>(who);
    let tile = g.map.trees[0];
    let before = g.land.tree_cover.get(&tile).copied().unwrap_or(0.85);
    g.arrive(who, Goal::Gather("wood", tile));
    let after = g.land.tree_cover.get(&tile).copied().unwrap();
    assert!(after < before, "a gather takes canopy");
    // undisturbed ground creeps back
    g.land_tick(5);
    let regrown = g.land.tree_cover.get(&tile).copied().unwrap();
    assert!(regrown > after, "undisturbed canopy regrows");
    // trampled ground does not
    g.trails.insert(tile, Trail { strength: 0.5 });
    let c = g.land.tree_cover.get(&tile).copied().unwrap();
    g.land_tick(12);
    assert_eq!(g.land.tree_cover.get(&tile).copied().unwrap(), c, "a worn tile stays bare");
}

#[test]
fn test_the_pit_drowns_and_the_mason_knows_it() {
    let mut g = Game::new();
    let who = g.villagers[0];
    let t = g.pit.unwrap_or((6, 19));
    for _ in 0..20 { g.arrive(who, Goal::Gather("stone", t)); }
    assert!(g.land.pit_water >= 0.7, "the pit fills as it's dug");
    let mason = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Mason).unwrap_or(false));
    if let Some(m) = mason {
        g.land_tick(2);
        assert!(desires::problems_of(&g.world, m, 2).contains(&desires::Problem::DryPit),
            "the drowned pit lands on the trade that needs it");
    }
}

#[test]
fn test_the_reach_carries_downstream_only() {
    let mut g = Game::new();
    for _ in 0..30 { g.land_tick(3); }
    let (wx0, _, _, wy1) = g.town_wall;
    // every river tile above the town stays clean
    let upstream_dirty = (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| g.map.get(x, y) == Some(Tile::River) && y < wy1 - 4)
        .any(|t| g.land.effluent.get(&t).copied().unwrap_or(0.0) > 0.2);
    assert!(!upstream_dirty, "the stink never travels upriver");
    let _ = wx0;
}

#[test]
fn test_foul_water_brings_the_waterborne_sickness() {
    let mut g = Game::new();
    // foul the water beside a home - the reach giving back what it got
    let v = g.villagers[0];
    let hp = household::home_of(&g.world, v)
        .and_then(|h| g.world.get_component::<Pos>(h).copied()).expect("villager 0 has a home");
    g.land.effluent.insert((hp.x.round() as i32 + 1, hp.y.round() as i32), 0.9);
    for _ in 0..300 { g.land_tick(2); }
    let waterborne = g.world.query::<Disease>().iter().any(|&d|
        g.world.get_component::<Disease>(d)
            .map(|dd| dd.kind.route() == illness::Route::Water).unwrap_or(false));
    assert!(waterborne, "foul water should sicken somebody");
}

#[test]
fn test_clean_water_carries_no_sickness() {
    let mut g = Game::new();
    for _ in 0..100 { g.land_tick(2); }
    assert!(g.world.query::<Disease>().iter().all(|&d|
        g.world.get_component::<Disease>(d)
            .map(|dd| dd.kind.route() != illness::Route::Water).unwrap_or(true)),
        "clean water sicks no one");
}

#[test]
fn test_the_water_sick_foul_the_reach_further() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let home = household::home_of(&g.world, v)
        .and_then(|h| g.world.get_component::<Pos>(h).copied()).expect("villager 0 has a home");
    let d = g.world.create_entity();
    g.world.add_component(d, Disease::new(DiseaseKind::Cholera, v, 0.7));
    // lands on the foul-water check; no accretion day, so what climbs
    // is what the sick poured back
    g.land_tick(2);
    let nearest = g.reach.iter().copied()
        .min_by_key(|&(x, y)| ((x as f32 - home.x).abs() + (y as f32 - home.y).abs()) as i32)
        .expect("there's a reach");
    let e = g.land.effluent.get(&nearest).copied().unwrap_or(0.0);
    assert!(e > 0.15, "the sick pour it back fouler");
}

#[test]
fn test_the_autumn_marsh_sends_the_ague() {
    let mut g = Game::new();
    // a home right on the river - the marsh's own doorstep
    let v = g.villagers[0];
    let river = (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| g.map.get(x, y) == Some(Tile::River)).expect("there's a river");
    let home = household::home_of(&g.world, v).expect("villager 0 has a home");
    if let Some(hp) = g.world.get_component_mut::<Pos>(home) {
        hp.x = river.0 as f32; hp.y = (river.1 - 1) as f32;
    }
    // out of season the marsh keeps its shakes
    for _ in 0..60 { g.land_tick(100); }
    assert!(g.world.query::<Disease>().iter().all(|&d|
        g.world.get_component::<Disease>(d).map(|dd| dd.kind != DiseaseKind::Ague).unwrap_or(true)),
        "the ague waits for autumn");
    // in season, it comes calling
    for _ in 0..400 { g.land_tick(247); }
    assert!(g.world.query::<Disease>().iter().any(|&d|
        g.world.get_component::<Disease>(d).map(|dd| dd.kind == DiseaseKind::Ague).unwrap_or(false)),
        "the autumn marsh should send the ague");
}

#[test]
fn test_sickness_abroad_finds_a_scapegoat() {
    let mut g = Game::new();
    // two down with la grippe, symptoms showing
    for &v in g.villagers.iter().take(2) {
        let d = g.world.create_entity();
        let mut dis = Disease::new(DiseaseKind::Influenza, v, 0.6);
        dis.incubation = 0; dis.stage = Stage::Moderate;
        g.world.add_component(d, dis);
    }
    for _ in 0..30 { g.land_tick(2); }
    let blamed = g.villagers.iter().copied().any(|v|
        desires::problems_of(&g.world, v, 2).contains(&desires::Problem::Blamed));
    assert!(blamed, "when sickness is abroad, somebody takes the blame");
}

#[test]
fn test_a_thin_wood_troubles_the_toolmaker() {
    let mut g = Game::new();
    for c in g.land.tree_cover.values_mut() { *c = 0.2; }
    let tool = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Toolmaker).unwrap_or(false));
    if let Some(t) = tool {
        g.land_tick(2);
        assert!(desires::problems_of(&g.world, t, 2).contains(&desires::Problem::ThinWood),
            "the shrinking wood lands on the wood trade");
    }
}

/// A spark's worth of fuel not far from the village: a flammable tile
/// whose neighbours mostly won't carry the fire far
fn fire_spot(g: &Game) -> (i32, i32) {
    (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| crate::game::fire::Fire::flammable(&g.map, x, y))
        .filter(|&(x, y)| [(-1, 0), (0, -1), (1, 0), (0, 1)].iter()
            .filter(|&&(dx, dy)| crate::game::fire::Fire::flammable(&g.map, x + dx, y + dy))
            .count() <= 2)
        .min_by_key(|&(x, y)| ((x - g.village.0).abs() + (y - g.village.1).abs(), x, y))
        .expect("the map has a burnable spot")
}

#[test]
fn test_the_bucket_line_answers_the_fire() {
    let mut g = Game::new();
    g.speed = 4;
    let (fx, fy) = fire_spot(&g);
    g.fire.ignite(&g.map, fx, fy, g.tick);
    // a few frames and the line's formed - four at most, all walking
    // the bucket's legs
    for _ in 0..8 { g.update(); }
    assert!(!g.fire_crew.is_empty(), "the bucket line forms");
    assert!(g.fire_crew.len() <= 4, "four is the whole line");
    assert!(g.travels.iter().any(|t| g.fire_crew.contains(&t.who)
        && matches!(t.goal, Goal::FillBucket | Goal::DouseFire(_))),
        "they're walking the line, not running");
    // the day turns and the crew holds its ground where the rest run
    g.fire_tick(g.tick);
    let crew = g.fire_crew.clone();
    for &v in &crew {
        assert!(!g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::HeadHome(_))),
            "the bucket line doesn't run for home");
    }
    // and a bucket finds the flames before the little fire's burnt out
    let mut saw_douse = false;
    for _ in 0..3000 {
        g.update();
        if !g.fire.doused.is_empty() { saw_douse = true; }
        if !g.fire.active() { break; }
    }
    assert!(saw_douse, "a bucket found the flames");
    assert!(!g.fire.active(), "the line saw the little fire out");
}

#[test]
fn test_a_bucket_on_the_flames_puts_it_out() {
    let mut g = Game::new();
    let (fx, fy) = fire_spot(&g);
    g.fire.ignite(&g.map, fx, fy, g.tick);
    let v = g.villagers.iter().copied().find(|&v|
        !villager::is_dead(&g.world, v) && g.world.get_component::<Kid>(v).is_none()).unwrap();
    // she stands a pace from the flames with a full pail, on the line
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: (fx + 1) as f32, y: fy as f32 };
    g.fire_crew.push(v);
    g.arrive(v, Goal::DouseFire((fx, fy)));
    assert!(!g.fire.on_fire(fx, fy), "one bucket, one tile out");
    assert!(g.fire.is_doused(fx, fy, g.tick), "and the ground stays wet");
    // the pail's still full but nothing's alight: she doesn't walk the line
    assert!(!g.travels.iter().any(|t| t.who == v), "nothing left to fetch");
}

#[test]
fn test_the_player_can_pitch_in() {
    let mut g = Game::new();
    let (fx, fy) = fire_spot(&g);
    g.fire.ignite(&g.map, fx, fy, g.tick);
    // stand the Wanderer a pace from the flames
    let pe = g.seats[0].player;
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: (fx + 1) as f32, y: fy as f32 };
    g.update_ctx_actions_at(0);
    let Some(i) = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::Douse) else {
        panic!("Throw water shows by the flames");
    };
    g.ctx_act_at(0, i);
    assert!(!g.fire.on_fire(fx, fy), "the player's bucket found the flames");
    assert!(g.fire.is_doused(fx, fy, g.tick));
    // out of reach the button's gone
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: 2.0, y: 2.0 };
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Douse), "no flames near, no bucket");
}

#[test]
fn test_a_seat_alike_is_neither_drafted_nor_sent_running() {
    let mut g = Game::new();
    g.speed = 4;
    let (fx, fy) = fire_spot(&g);
    g.fire.ignite(&g.map, fx, fy, g.tick);
    // claim a neighbour of the flames before the line musters, errand in hand
    let v = g.villagers.iter().copied().find(|&v|
        !villager::is_dead(&g.world, v) && g.world.get_component::<Kid>(v).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: (fx + 1) as f32, y: fy as f32 };
    g.add_agent_seat(v);
    g.travels.push(Travel { who: v, path: vec![(fx, fy + 3)], goal: Goal::Gather("wood", (fx, fy)) });
    for _ in 0..8 { g.update(); }
    assert!(!g.fire_crew.contains(&v), "the line doesn't draft a seat's own");
    g.fire_tick(g.tick);
    assert!(g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Gather(..))),
        "and the flames don't cancel her errand either");
}

#[test]
fn test_seed_42_fire_does_not_run_away() {
    let mut g = Game::new();
    g.speed = 16;
    while g.tick < 1500 { g.update(); }
    assert!(!g.fire.active(), "fire still has {} burning tiles and {} scorched at tick {}",
        g.fire.burning.len(), g.fire.scorched.len(), g.tick);
    assert!(g.ruined.len() < 5, "one seed lost {} buildings to fires", g.ruined.len());
}

#[test]
fn test_the_village_names_the_ground_it_stands_on() {
    let g = Game::new();
    if let Some(n) = &g.landform {
        assert!(!n.is_empty());
        assert!(g.log.iter().any(|l| l.contains(n) && l.contains("makes its home")),
            "the ground's name went untold: {:?} / {}", n, g.log.last().unwrap());
    }
    // the shape is the seed's: same seed, same ground
    let g2 = Game::new();
    assert_eq!(g.landform, g2.landform);
}
