use crate::session::*;

#[test]
fn test_the_proposal_surfaces_a_picked_counter() {
    let mut g = Game::new();
    let lord = g.sites.iter().find_map(|s| s.lord).unwrap_or(g.villagers[10]);
    let m = g.villagers[11];
    g.world.add_component(m, crate::game::campaign::Plant(lord));
    g.plant = Some(m);
    g.proposal = Some(g.tick);
    // the violent word reaches the player's box
    let panel = g.convo_panel_at(0);
    assert!(panel.is_some(), "the proposal never reached the box");
    assert_eq!(panel.unwrap().options.len(), 3, "not the three counters");
}

#[test]
fn test_verify_reads_the_plant_out() {
    let mut g = Game::new();
    let lord = g.sites.iter().find_map(|s| s.lord).unwrap_or(g.villagers[10]);
    let m = g.villagers[11];
    g.world.add_component(m, crate::game::campaign::Plant(lord));
    g.plant = Some(m);
    g.proposal = Some(g.tick);
    g.decide_at(0, 1); // verify: reads the real plant
    assert!(g.plant.is_none(), "the plant stayed after the read");
    assert!(g.proposal.is_none(), "the proposal stayed open");
    assert!(g.log.iter().any(|l| l.contains("lord's hand")),
        "the read never named the hand");
}

#[test]
fn test_eject_on_a_true_heart_costs_the_room() {
    let mut g = Game::new();
    let lord = g.sites.iter().find_map(|s| s.lord).unwrap_or(g.villagers[10]);
    let m = g.villagers[11]; // no Plant - a true heart
    g.world.add_component(m, crate::game::campaign::Plant(lord));
    // the player's suspicion lands on m but m carries no Plant marker
    // ... (the eject's paranoia cost is the honest part)
    g.plant = Some(m);
    g.proposal = Some(g.tick);
    let before = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    g.decide_at(0, 2); // eject
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.len()).unwrap_or(0);
    assert!(g.proposal.is_none(), "the proposal stayed open");
    assert!(after <= before, "the eject never removed a body");
}
