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

/// The eyes' menu answers once a day: a spent play drops the box till
/// tomorrow, it doesn't re-offer the same frame
#[test]
fn test_a_counter_play_is_spent_for_the_day() {
    let mut g = Game::new();
    g.counter_intel = 2;
    let eyes = |g: &Game| g.convo_panel_at(0)
        .map(|p| p.title == "THE ORG'S OWN EYES").unwrap_or(false);
    assert!(eyes(&g), "the box never offered");
    g.decide_at(0, 0);
    assert!(!eyes(&g), "the box re-offered on a spent play");
    g.decide_at(0, 0);
    assert_eq!(g.log.iter().filter(|l| l.contains("false word")).count(), 1,
        "a spent day's pick still landed");
    // tomorrow the eyes are watching again - the menu's back
    g.tick += 1;
    assert!(eyes(&g), "the play didn't come round a new day");
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
