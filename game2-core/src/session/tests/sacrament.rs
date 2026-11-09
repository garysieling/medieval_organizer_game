use crate::session::*;

#[test]
fn test_the_parallel_fold_splits_on_its_own_weight() {
    let mut g = Game::new();
    g.separation_pressure = 1.0;
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.orthodoxy = 0.6;
    }
    g.sacrament_tick(56);
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.orthodoxy).unwrap_or(0.6);
    assert!(after < 0.6, "the schism never dropped the orthodoxy");
    assert!(g.log.iter().any(|l| l.contains("the fold splits")),
        "the schism never logged");
}

#[test]
fn test_the_lords_silence_drains_the_flock() {
    let mut g = Game::new();
    g.interdict = true;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::religion::Faith {
        religion: "the parish".into(), devotion: 0.5, traditions: vec![] });
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        if !c.members.contains(&m) { c.members.push(m); }
    }
    g.sacrament_tick(56);
    let after = g.world.get_component::<crate::sim::religion::Faith>(m)
        .map(|f| f.devotion).unwrap_or(0.5);
    assert!(after < 0.5, "the silence never drained: {}", after);
}
