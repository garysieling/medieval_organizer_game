use crate::session::*;

#[test]
fn test_the_council_settles_and_the_loser_splinters() {
    let mut g = Game::new();
    g.world.add_component(g.congregation, crate::sim::religion::Doctrine {
        canon: vec![], heresy_vector: vec![] });
    g.doctrine_tick(109); // a council tick
    let d = g.world.get_component::<crate::sim::religion::Doctrine>(g.congregation);
    let canon = d.map(|d| d.canon.len()).unwrap_or(0);
    let heresy = g.world.get_component::<crate::sim::religion::Doctrine>(g.congregation)
        .map(|d| d.heresy_vector.len()).unwrap_or(0);
    assert!(canon > 0, "the council never settled: {}", canon);
    assert!(heresy > 0, "the loser never splintered: {}", heresy);
}

#[test]
fn test_the_grievance_converts_to_devotion() {
    let mut g = Game::new();
    g.suffering_meaning = 0.8;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::religion::Faith {
        religion: "the parish".into(), devotion: 0.5, traditions: vec![] });
    g.world.add_component(m, crate::game::desires::InnerLife {
        wants: vec![], satisfied: vec![],
        grievances: vec![(crate::game::desires::Problem::Loss, g.tick + 30)] });
    g.doctrine_tick(49);
    let after = g.world.get_component::<crate::sim::religion::Faith>(m)
        .map(|f| f.devotion).unwrap_or(0.5);
    assert!(after > 0.5, "the grievance never converted: {}", after);
}
