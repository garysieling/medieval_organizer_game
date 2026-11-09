use crate::session::*;

#[test]
fn test_the_sequence_advances_through_the_cycle() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.turnout = 0.8;
    o.agitation = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.inquisition = Some(0);
    g.inquisition_tick(32);
    assert_eq!(g.inquisition, Some(1), "the summons never advanced to the harvest");
    g.inquisition_tick(122);
    assert_eq!(g.inquisition, Some(2), "the harvest never advanced to the informers");
}

#[test]
fn test_the_hidden_fold_halves_the_raid() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.turnout = 0.8;
    o.agitation = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.hidden_institution = true;
    g.inquisition = Some(3); // the prosecution
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.inquisition_tick(32);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.8);
    // the hidden fold takes half the raid an open one would
    assert!(after >= 0.75, "the dissimulation never armored: {}", after);
}

#[test]
fn test_cutting_loose_with_support_cuts_the_leverage() {
    let mut g = Game::new();
    g.inquisition = Some(4); // the relapse
    g.penetration = 0.6;
    g.decide_at(0, 0); // cut them loose with support
    assert!(g.penetration < 0.6, "the support never cut the leverage");
    assert!(g.log.iter().any(|l| l.contains("walks free with bread")),
        "the release never logged");
}
