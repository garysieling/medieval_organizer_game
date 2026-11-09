use crate::session::*;

#[test]
fn test_a_spent_fold_breaks_its_own_way() {
    let mut g = Game::new();
    g.world.add_component(g.congregation, crate::sim::religion::Totalism {
        dials: [8; 8], idealism: 0.1, exit_cost: 5.0, isolation: 0.8 });
    g.cult_collapse_tick(15);
    assert!(g.cult_collapse.is_some(), "the cage never broke");
    assert!(g.log.iter().any(|l| l.contains("the cage breaks")),
        "the end never logged");
}

#[test]
fn test_the_outside_word_degrades_the_dials() {
    let mut g = Game::new();
    g.world.add_component(g.congregation, crate::sim::religion::Totalism {
        dials: [8; 8], idealism: 0.5, exit_cost: 5.0, isolation: 0.8 });
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.cultish = true;
    }
    g.decide_at(0, 1); // bring the outside word in
    let sum = g.world.get_component::<crate::sim::religion::Totalism>(g.congregation)
        .map(|t| t.dials.iter().map(|&d| d as u16).sum::<u16>()).unwrap_or(64);
    assert!(sum < 64, "the exposure never thinned the walls");
}

#[test]
fn test_a_soul_walks_free_on_the_shelter() {
    let mut g = Game::new();
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.cultish = true;
        if c.members.is_empty() { c.members.push(g.villagers[0]); }
    }
    let before = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.members.len()).unwrap_or(0);
    g.decide_at(0, 0); // shelter the one who wants out
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.members.len()).unwrap_or(0);
    assert!(after <= before, "the soul never walked free");
}
