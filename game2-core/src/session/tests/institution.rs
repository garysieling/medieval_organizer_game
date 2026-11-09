use crate::session::*;

#[test]
fn test_the_pillars_pay_their_dues() {
    let mut g = Game::new();
    g.institutions = 3;
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 10.0;
    }
    g.institution_tick(43);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(10.0);
    assert!(after > 10.0, "the dues never paid: {}", after);
}

#[test]
fn test_the_sponsored_holds_against_the_patron() {
    let mut g = Game::new();
    g.institutions = 3;
    g.locally_sponsored = true;
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.institution_tick(43);
    assert_eq!(g.institutions, 3, "the sponsored pillar drifted");
}

#[test]
fn test_the_unsponsored_drifts_to_the_patron() {
    let mut g = Game::new();
    g.institutions = 3;
    g.locally_sponsored = false;
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.institution_tick(43);
    assert_eq!(g.institutions, 2, "the unsponsored never drifted: {}", g.institutions);
}
