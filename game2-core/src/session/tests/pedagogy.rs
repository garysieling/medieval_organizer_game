use crate::session::*;

#[test]
fn test_the_raising_lowers_the_domination() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.8, critical_agency: 0.0, experience: 0.0 });
    g.world.add_component(m, crate::game::desires::InnerLife {
        wants: vec![crate::game::desires::Want::GoodFood],
        grievances: vec![], satisfied: vec![] });
    g.pedagogy_tick(39);
    let after = g.world.get_component::<crate::game::desires::Domination>(m)
        .map(|d| d.internalized_domination).unwrap_or(0.8);
    assert!(after < 0.8, "the domination never lowered: {}", after);
}

#[test]
fn test_the_praxis_needs_both_halves() {
    let mut g = Game::new();
    let m = g.villagers[0];
    // action experience + reflection agency - the loop converts
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.0, critical_agency: 0.5, experience: 0.5 });
    let mut o = crate::game::org_skills::Organizing::default();
    o.one_on_one = 0.2;
    g.world.add_component(m, o);
    g.pedagogy_tick(39);
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.one_on_one).unwrap_or(0.2);
    assert!(after > 0.2, "the praxis never converted: {}", after);

    // experience alone plateaus - no agency, no conversion
    let mut g2 = Game::new();
    let m2 = g2.villagers[0];
    g2.world.add_component(m2, crate::game::desires::Domination {
        internalized_domination: 0.0, critical_agency: 0.0, experience: 0.5 });
    let mut o2 = crate::game::org_skills::Organizing::default();
    o2.one_on_one = 0.2;
    g2.world.add_component(m2, o2);
    g2.pedagogy_tick(39);
    let after2 = g2.world.get_component::<crate::game::org_skills::Organizing>(m2)
        .map(|o| o.one_on_one).unwrap_or(0.2);
    assert!(after2 <= 0.2, "action alone plateaued wrong: {}", after2);
}
