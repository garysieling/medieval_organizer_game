use crate::session::*;

#[test]
fn test_the_frame_cited_demand_extracts() {
    let mut g = Game::new();
    g.petition_count = 20;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.5;
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.8 });
        }
    }
    g.paternalism_tick(41);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.5);
    assert!(after > 0.5, "the frame's own demand never extracted: {}", after);
}

#[test]
fn test_the_ignoring_costs_the_standing() {
    let mut g = Game::new();
    g.petition_count = 20;
    let mut before = 0.8;
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.8 });
        }
    }
    g.paternalism_tick(41);
    for site in &g.sites {
        if let Some(l) = site.lord {
            if let Some(w) = g.world.get_component::<crate::session::subsystems::society::Watcher>(l) {
                before = w.legitimacy;
            }
        }
    }
    assert!(before < 0.8, "the ignoring never cost: {}", before);
}

#[test]
fn test_the_low_standing_extracts_nothing() {
    let mut g = Game::new();
    g.petition_count = 20;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.5;
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.3 });
        }
    }
    g.paternalism_tick(41);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.5);
    assert!(after <= 0.5, "the weak frame extracted anyway: {}", after);
}
