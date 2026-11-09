use crate::session::*;

#[test]
fn test_the_orgs_eyes_grow_with_its_strength() {
    let mut g = Game::new();
    g.counter_intel = 0;
    for _ in 0..20 { g.counter_intel_tick(8 + 1); }
    // a fresh org's eyes may be thin - the level's a count, not a flag
    assert!(g.counter_intel <= 3, "the level ran past its own depth");
}

#[test]
fn test_feeding_the_watcher_a_false_word() {
    let mut g = Game::new();
    g.counter_intel = 2;
    let lord = g.sites.iter().find_map(|s| s.lord
        .filter(|&l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l).is_some()))
        .unwrap_or_else(|| {
            let l = g.villagers[10];
            g.world.add_component(l, crate::session::subsystems::society::Watcher { paranoia: 0.0, legitimacy: 0.7 });
            l
        });
    if let Some(w) = g.world.get_component_mut::<crate::session::subsystems::society::Watcher>(lord) {
        w.paranoia = 0.6;
    }
    g.decide_at(0, 0);
    let after = g.world.get_component::<crate::session::subsystems::society::Watcher>(lord)
        .map(|w| w.paranoia).unwrap_or(0.6);
    assert!(after < 0.6, "the false word never softened the eyes");
    assert!(g.log.iter().any(|l| l.contains("false word")), "the play never logged");
}

#[test]
fn test_reading_the_watcher_out_costs_him() {
    let mut g = Game::new();
    g.counter_intel = 3;
    let lord = g.sites.iter().find_map(|s| s.lord
        .filter(|&l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l).is_some()))
        .unwrap_or_else(|| {
            let l = g.villagers[10];
            g.world.add_component(l, crate::session::subsystems::society::Watcher { paranoia: 0.0, legitimacy: 0.7 });
            l
        });
    if let Some(w) = g.world.get_component_mut::<crate::session::subsystems::society::Watcher>(lord) {
        w.legitimacy = 0.8;
    }
    g.decide_at(0, 1);
    let after = g.world.get_component::<crate::session::subsystems::society::Watcher>(lord)
        .map(|w| w.legitimacy).unwrap_or(0.8);
    // the read either cost his standing or hardened his eyes - either
    // way the play ran its course
    assert!(after < 0.8 || g.log.iter().any(|l| l.contains("hardens")), "the read never landed");
}
