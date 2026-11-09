use crate::session::*;

#[test]
fn test_a_hard_head_pushes_the_dials() {
    let mut g = Game::new();
    // a pressured hard head and a strict fold: the dials should rise
    g.world.add_component(g.congregation, crate::sim::religion::Totalism {
        dials: [0; 8], idealism: 1.0, exit_cost: 0.0, isolation: 0.0 });
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.orthodoxy = 0.9;
    }
    for t in (10..300).step_by(30) { g.totalism_tick(t); }
    let sum = g.world.get_component::<crate::sim::religion::Totalism>(g.congregation)
        .map(|t| t.dials.iter().map(|&d| d as u16).sum::<u16>()).unwrap_or(0);
    assert!(sum > 0, "no dial ever rose");
}

#[test]
fn test_the_extraction_spends_the_idealism() {
    let mut g = Game::new();
    g.world.add_component(g.congregation, crate::sim::religion::Totalism {
        dials: [5; 8], idealism: 1.0, exit_cost: 0.0, isolation: 0.0 });
    g.totalism_tick(10);
    let after = g.world.get_component::<crate::sim::religion::Totalism>(g.congregation)
        .map(|t| t.idealism).unwrap_or(1.0);
    assert!(after < 1.0, "the idealism was never spent");
    let cost = g.world.get_component::<crate::sim::religion::Totalism>(g.congregation)
        .map(|t| t.exit_cost).unwrap_or(0.0);
    assert!(cost > 0.0, "the cage never raised its cost");
}
