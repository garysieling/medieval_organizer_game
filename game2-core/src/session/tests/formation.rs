use crate::session::*;

fn call_the_march(g: &mut Game) -> Vec<Entity> {
    // three members sworn in, all taxed, and the issue heard - the same
    // founding the campaign tests set
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g.seats[0].ctx_actions = vec![CtxAct::March];
    g.ctx_act_at(0, 0);
    g.formation.as_ref().map(|f| f.members.clone()).unwrap_or_default()
}

#[test]
fn test_the_march_columns_and_comes_home() {
    let mut g = Game::new();
    let members = call_the_march(&mut g);
    let f = g.formation.as_ref().expect("no column formed");
    assert!(f.members.len() >= 3, "a thin column: {}", f.members.len());
    assert!(f.route.len() >= 8, "a short route: {}", f.route.len());
    for &m in &members {
        assert!(g.world.get_component::<villager::InFormation>(m).is_some(),
            "a marcher never fell in");
    }
    // the head walks the route out
    let head = members[0];
    let p0 = g.world.get_component::<Pos>(head).copied().unwrap();
    for _ in 0..900 { g.update(); }
    if let Some(f) = &g.formation {
        let p1 = g.world.get_component::<Pos>(head).copied().unwrap();
        let moved = ((p1.x - p0.x).powi(2) + (p1.y - p0.y).powi(2)).sqrt();
        assert!(f.at > 0 || moved > 0.5, "the column never stepped: at={} moved={:.2}", f.at, moved);
        // and the files keep their pace behind the head - a body walled
        // in a yard may lag, but most of the column closes up
        let tight = f.members.windows(2).filter(|w| {
            let (a, b) = (g.world.get_component::<Pos>(w[0]).copied().unwrap(),
                          g.world.get_component::<Pos>(w[1]).copied().unwrap());
            ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt() < 8.0
        }).count();
        assert!(tight * 2 >= f.members.len() - 1, "the column never formed: {}/{}", tight, f.members.len() - 1);
    }
    // out the route and home: the street answers, the column breaks
    for _ in 0..6000 {
        if g.formation.is_none() { break; }
        g.update();
    }
    assert!(g.formation.is_none(), "the march never came home");
    for &m in &members {
        assert!(g.world.get_component::<villager::InFormation>(m).is_none(),
            "a marcher never stood down");
    }
    assert!(g.log.iter().any(|l| l.contains("march in the street") || l.contains("march dissolved")),
        "the street never answered: {:?}", g.log.iter().filter(|l| l.contains("march")).collect::<Vec<_>>());
}
