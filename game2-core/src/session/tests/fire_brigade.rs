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

/// Set the cause up so its floor could carry the charter: three grown
/// members, the asking done on its own issue, a hall, a fat chest.
/// `fire` adds the grief the brigade answers - voiced, or worn by a member
fn brigade_ready(g: &mut Game, fire: bool) {
    let oe = g.org;
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    g.world.get_component_mut::<Organization>(oe).unwrap().members = grown.clone();
    // the cause's own trouble: the taxes weigh on the room
    for &v in &grown[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    if fire {
        desires::add_grievance(&mut g.world, grown[3], desires::Problem::Blamed, u64::MAX);
    }
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, oe).unwrap();
    let issue = g.world.get_component::<crate::game::campaign::Campaign>(c).unwrap().issue.clone();
    crate::game::campaign::mark_legwork(&mut g.world, oe, issue);
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.world.add_component(oe, crate::game::organizing::MeetingHall);
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 50.0;
}

/// The floor's slate after the cadence's day sits it (#1337): picks
/// the charter's spot if offered - a slate still sitting from the last
/// call clears first, the way the week's own lapse does
fn brigade_pick(g: &mut Game) -> Option<usize> {
    g.seats[0].pending_agenda = None;
    super::sit_floor(g, 0);
    g.seats[0].pending_agenda.as_ref().map(|(slate, _)| {
        slate.iter().position(|&m| m == crate::game::campaign::Motion::Brigade)
    }).flatten()
}

#[test]
fn test_the_brigade_waits_for_a_named_fire() {
    // no fire grief's ever been heard -> the charter doesn't reach the
    // floor however fat the chest (#1346)
    let mut g = Game::new();
    brigade_ready(&mut g, false);
    assert!(brigade_pick(&mut g).is_none(),
        "the cause crewed a bucket line for a fire it never heard: {:?}",
        g.seats[0].pending_agenda.as_ref().map(|(s, _)| s));
}

#[test]
fn test_a_voiced_fire_or_a_burned_member_puts_the_charter_on_the_floor() {
    use crate::game::campaign::Motion;
    // a member still wearing the blame crews the brigade herself
    let mut g = Game::new();
    brigade_ready(&mut g, true);
    let pick = brigade_pick(&mut g).expect("a burned hand puts the bell on the slate");
    g.decide_at(0, pick);
    let b = g.brigade.expect("the vote chartered no brigade");
    let o = g.world.get_component::<Organization>(b).unwrap();
    assert_eq!(o.name, "the fire brigade");
    assert!(o.members.len() >= 2, "a brigade of one isn't a crew");
    assert_eq!(g.brigade_gear, 3, "the chest bought no buckets");
    assert!(g.world.get_component::<Organization>(g.org).unwrap().resources < 50.0,
        "the outfitting was free");

    // a fire the meeting voiced counts too - the cause needn't carry it
    let mut g = Game::new();
    brigade_ready(&mut g, false);
    let oe = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, oe, "the bad bread".into());
    assert!(brigade_pick(&mut g).is_some(), "a voiced fire still uncrewed");
    // but it's still the cause's to vote: a thin chest stays quiet
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 0.0;
    assert!(brigade_pick(&mut g).is_none(), "the slate offered buckets the chest can't buy");
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 50.0;
    g.world.remove_component::<crate::game::organizing::MeetingHall>(g.org);
    assert!(brigade_pick(&mut g).is_none(), "the slate offered a crew with no hall");
    let _ = Motion::Brigade;
}

#[test]
fn test_the_sworn_answer_before_the_willing() {
    let mut g = Game::new();
    g.speed = 4;
    brigade_ready(&mut g, true);
    let pick = brigade_pick(&mut g).expect("the charter's on the floor");
    g.decide_at(0, pick);
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
