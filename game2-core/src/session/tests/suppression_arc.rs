use crate::session::*;

#[test]
fn test_the_arc_reads_which_rung_fired() {
    let mut g = Game::new();
    g.suppression_arc = 0;
    g.emergency = Some((g.tick, g.tick + 20));
    g.suppression_arc = 3; // the emergency's own rung
    assert_eq!(g.suppression_arc, 3, "the arc never read the ban");
}

#[test]
fn test_an_early_contest_costs_the_lord() {
    let mut g = Game::new();
    g.counter_intel = 2;
    g.suppression_arc = 1;
    let lord = g.sites.iter().find_map(|s| s.lord
        .filter(|&l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l).is_some()))
        .unwrap_or_else(|| {
            let l = g.villagers[10];
            g.world.add_component(l, crate::session::subsystems::society::Watcher { paranoia: 0.0, legitimacy: 0.7 });
            l
        });
    if let Some(w) = g.world.get_component_mut::<crate::session::subsystems::society::Watcher>(lord) {
        w.legitimacy = 0.7;
    }
    g.decide_at(0, 0);
    let after = g.world.get_component::<crate::session::subsystems::society::Watcher>(lord)
        .map(|w| w.legitimacy).unwrap_or(0.7);
    assert!(after < 0.7, "the early contest never cost him");
    assert!(g.log.iter().any(|l| l.contains("contested early")), "the contest never logged");
}
