use crate::session::*;

/// Found a campaign on seat 0's org - the same rig the campaign tests
/// use, so a march has something to march for
fn founded(g: &mut Game) -> Entity {
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    crate::game::campaign::campaign_of(&g.world, g.org).unwrap()
}

fn set_issue(g: &mut Game, c: Entity, issue: &str) {
    g.world.get_component_mut::<crate::game::campaign::Campaign>(c).unwrap().issue = issue.into();
}

#[test]
fn test_a_road_march_goes_where_the_issue_lives() {
    let mut g = Game::new();
    let c = founded(&mut g);
    // the lord's own grievances march on his gate
    set_issue(&mut g, c, "the taxes");
    let si = g.march_destination(g.org).expect("the taxes has somewhere to go");
    assert_eq!(g.sites[si].kind, country::SiteKind::Castle);
    // a faith cause marches on the sisters - when the country has an abbey
    set_issue(&mut g, c, "the old faith");
    let si = g.march_destination(g.org);
    let abbey = g.sites.iter().any(|s| s.kind == country::SiteKind::Monastery);
    assert_eq!(si.map(|i| g.sites[i].kind), abbey.then_some(country::SiteKind::Monastery),
        "a faith cause wants the abbey or stays home");
    // and the rest take the market town
    set_issue(&mut g, c, "hunger");
    let si = g.march_destination(g.org).expect("hunger has somewhere to go");
    assert_eq!(g.sites[si].kind, country::SiteKind::Town);
}

#[test]
fn test_the_road_route_leaves_the_wall_and_comes_home() {
    let mut g = Game::new();
    let c = founded(&mut g);
    set_issue(&mut g, c, "the taxes");
    let si = g.march_destination(g.org).unwrap();
    let hp = g.world.get_component::<Pos>(g.villagers[0]).map(|p| (p.x, p.y)).unwrap();
    let route = g.road_route(si, hp).expect("a road reaches the castle");
    let (x0, y0, x1, y1) = g.town_wall;
    assert!(route.iter().any(|&(x, y)|
            x < x0 as f32 || y < y0 as f32 || x > x1 as f32 || y > y1 as f32),
        "the road march never leaves the walls");
    // and it comes home: the last waypoint stands near the market cross
    let m = g.world.get_component::<Pos>(g.market_food).unwrap();
    let &(ex, ey) = route.last().unwrap();
    assert!((ex - m.x).abs() + (ey - m.y).abs() < 12.0,
        "the march doesn't come home ({},{} vs {},{})", ex, ey, m.x, m.y);
}

#[test]
fn test_a_march_takes_the_road_some_days_not_all() {
    let g = Game::new();
    // the roll's a day coin: across a season it lands about a third
    let hits = (0..90u64).filter(|&d| routine::mix(g.org, d) % 3 == 0).count();
    assert!(hits > 10 && hits < 50, "the road roll landed {} of 90 days", hits);
}

#[test]
fn test_a_castle_march_is_never_beneath_notice() {
    let mut g = Game::new();
    let _c = founded(&mut g);
    // a whisper of a cause: the leader alone can't reach WATCHED_AT's
    // two - the street would ignore them, the gate can't
    let leader = g.world.get_component::<Organization>(g.org).unwrap().leader;
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = vec![leader];
    let si = g.sites.iter().position(|s| s.kind == country::SiteKind::Castle).unwrap();
    g.march_resolved(0, Some(si));
    assert!(g.log.iter().any(|l| l.contains("sheriff takes the names")),
        "a lone column at the gate still gets its names taken");
}
