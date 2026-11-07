use crate::session::*;

fn able_fire_spot(g: &Game) -> (i32, i32) {
    (5..g.map.width as i32 - 5).flat_map(|y| (5..g.map.height as i32 - 5).map(move |x| (x, y)))
        .filter(|&(x, y)| g.map.get(x, y).map(|t| matches!(t, Tile::Grass)).unwrap_or(false)
            && (0..10).flat_map(|dy| (0..10).map(move |dx| (x + dx - 5, y + dy - 5)))
                .filter(|&b| b != (x, y))
                .all(|b| g.map.get(b.0, b.1).map(|t| matches!(t, Tile::Grass)).unwrap_or(false)))
        .min_by_key(|&(x, y)| ((x - g.village.0).abs() + (y - g.village.1).abs(), x, y))
        .expect("the map has a burnable spot")
}

#[test]
fn test_the_brigade_needs_a_hall_and_a_chest() {
    let mut g = Game::new();
    g.seats[0].ctx_actions = vec![CtxAct::Brigade];
    // no hall: the charter can't be raised
    assert!(g.ctx_blocker_at(0, CtxAct::Brigade).map(|b| b.contains("hall")).unwrap_or(false),
        "a charter with no hall to muster in");
    let oe = g.seats[0].org;
    g.world.add_component(oe, crate::game::organizing::MeetingHall);
    // no gold: the bell stays silent
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 0.0;
    assert!(g.ctx_blocker_at(0, CtxAct::Brigade).map(|b| b.contains("chest")).unwrap_or(false),
        "a charter the chest can't outfit");
    // hall and gold: the bell has hands
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 50.0;
    g.ctx_act_at(0, 0);
    let b = g.brigade.expect("no brigade chartered");
    let o = g.world.get_component::<Organization>(b).unwrap();
    assert_eq!(o.name, "the fire brigade");
    assert!(o.members.len() >= 2, "a brigade of one isn't a crew");
    assert_eq!(g.brigade_gear, 3, "the chest bought no buckets");
    assert!(g.world.get_component::<Organization>(oe).unwrap().resources < 50.0,
        "the outfitting was free");
    // and once sworn, there's no second charter
    g.ctx_act_at(0, 0);
    assert_eq!(g.brigade, Some(b), "the bell was re-sworn");
}

#[test]
fn test_the_sworn_answer_before_the_willing() {
    let mut g = Game::new();
    g.speed = 4;
    let oe = g.seats[0].org;
    g.world.add_component(oe, crate::game::organizing::MeetingHall);
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 50.0;
    g.seats[0].ctx_actions = vec![CtxAct::Brigade];
    g.ctx_act_at(0, 0);
    let sworn: Vec<Entity> = g.world.get_component::<Organization>(g.brigade.unwrap())
        .unwrap().members.clone();
    // a volunteer closer to the flames than any sworn hand
    let (fx, fy) = able_fire_spot(&g);
    // put every sworn hand far off, one volunteer on the doorstep
    let volunteer = g.villagers.iter().copied()
        .find(|&v| !sworn.contains(&v)
            && !villager::is_dead(&g.world, v)
            && g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<country::Outlander>(v).is_none())
        .expect("no free neighbour to put by the fire");
    *g.world.get_component_mut::<Pos>(volunteer).unwrap() = Pos { x: (fx + 1) as f32, y: fy as f32 };
    // the sworn stand a few paces farther out than the willing
    // neighbour - still within the bell's reach
    for &m in &sworn {
        *g.world.get_component_mut::<Pos>(m).unwrap() =
            Pos { x: (fx + 3) as f32, y: fy as f32 };
    }
    g.fire.ignite(&g.map, fx, fy, g.tick);
    for _ in 0..8 { g.update(); }
    assert!(!g.fire_crew.is_empty(), "no line formed at all");
    // a sworn hand stands ahead of the willing neighbour
    assert!(g.fire_crew.iter().any(|&c| sworn.contains(&c)),
        "no sworn hand on the line - the volunteers beat the bell");
}

#[test]
fn test_the_chests_gear_throws_more_water() {
    let mut g = Game::new();
    let (fx, fy) = able_fire_spot(&g);
    g.fire.ignite(&g.map, fx, fy, g.tick);
    g.fire.ignite(&g.map, fx + 1, fy, g.tick);
    g.brigade_gear = 3; // the chest bought good buckets
    let v = g.villagers.iter().copied().find(|&v|
        !villager::is_dead(&g.world, v) && g.world.get_component::<Kid>(v).is_none()).unwrap();
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: (fx + 1) as f32, y: fy as f32 };
    g.fire_crew.push(v);
    g.arrive(v, Goal::DouseFire((fx, fy)));
    assert!(!g.fire.on_fire(fx, fy), "her bucket missed the flames");
    assert!(!g.fire.on_fire(fx + 1, fy),
        "good gear throws past the bucket - the next flame's still alight");
}
