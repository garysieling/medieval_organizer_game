use crate::session::*;

#[test]
fn test_the_desk_makes_the_subject() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::session::Kid);
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.3, critical_agency: 0.0, experience: 0.0 });
    g.world.add_component(m, crate::session::routine::Routine {
        mode: crate::session::routine::Mode::Playing,
        spot: crate::session::routine::Spot::Play, anchor: (0.0, 0.0),
        face: None, drift: 0.0, leads: false, day: 33, reached: true });
    g.discipline_tick(33);
    let dom = g.world.get_component::<crate::game::desires::Domination>(m)
        .map(|d| d.internalized_domination).unwrap_or(0.3);
    assert!(dom > 0.3, "the desk never made the subject: {}", dom);
}

#[test]
fn test_the_fold_school_makes_the_agency() {
    let mut g = Game::new();
    g.folk_school = true;
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.3, critical_agency: 0.2, experience: 0.0 });
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !o.members.contains(&m) { o.members.push(m); }
    }
    g.discipline_tick(33);
    let agency = g.world.get_component::<crate::game::desires::Domination>(m)
        .map(|d| d.critical_agency).unwrap_or(0.2);
    assert!(agency > 0.2, "the fold's school never made the agency: {}", agency);
}
