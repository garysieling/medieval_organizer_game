use crate::session::*;

#[test]
fn test_the_eyes_fall_on_the_weak() {
    let mut g = Game::new();
    // a weak man: sick and poor, the watcher's eyes find him
    let v = g.villagers[0];
    g.world.get_component_mut::<Health>(v).map(|h| h.condition = 0.2);
    g.world.get_component_mut::<Purse>(v).map(|p| p.gold = 0.0);
    let p0 = g.sites.iter().find_map(|s| s.lord
        .and_then(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l)
            .map(|w| w.paranoia))).unwrap_or(0.0);
    g.surveillance_tick(50);
    let p1 = g.sites.iter().find_map(|s| s.lord
        .and_then(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l)
            .map(|w| w.paranoia))).unwrap_or(0.0);
    assert!(p1 > p0, "the paranoia never fed");
}

/// A loud cause leaks just by being big: the eyes carry home more on a
/// village whose muscle's full than on one whose cause is thin - and no
/// security culture can shrink sheer size
#[test]
fn test_a_big_cause_draws_heavier_eyes() {
    let mut g = Game::new();
    assert!(g.sites.iter().any(|s| s.lord
        .map(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l).is_some())
        .unwrap_or(false)), "no watcher on this seed");
    // nobody's weak: whatever the eyes carry home is the cause's own
    // loudness, not the village's misery
    for v in g.villagers.clone() {
        g.world.get_component_mut::<Health>(v).map(|h| h.condition = 1.0);
        g.world.get_component_mut::<Purse>(v).map(|p| p.gold = 10.0);
    }
    let paranoia = |g: &Game| g.sites.iter().find_map(|s| s.lord
        .and_then(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l)
            .map(|w| w.paranoia))).unwrap_or(0.0);
    g.surveillance_tick(50);
    let thin = paranoia(&g);
    for s in g.sites.iter().flat_map(|s| s.lord).collect::<Vec<_>>() {
        g.world.get_component_mut::<crate::session::subsystems::society::Watcher>(s)
            .map(|w| w.paranoia = 0.0);
    }
    // the same village, the cause's muscle full: the watching worsens
    g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|p| p.capacity = 1.0);
    g.surveillance_tick(51);
    let loud = paranoia(&g);
    assert!(loud > thin, "a big cause never drew the eyes: {} vs {}", loud, thin);
}

#[test]
fn test_the_watchers_get_caught() {
    let mut g = Game::new();
    for s in g.sites.iter() {
        if let Some(l) = s.lord {
            g.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l)
                .map(|w| { w.paranoia = 0.9; w.legitimacy = 0.7 });
        }
    }
    let l0 = g.sites.iter().find_map(|s| s.lord
        .and_then(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l)
            .map(|w| w.legitimacy))).unwrap_or(1.0);
    for t in 0..2000 { g.surveillance_tick(t); }
    let l1 = g.sites.iter().find_map(|s| s.lord
        .and_then(|l| g.world.get_component::<crate::session::subsystems::society::Watcher>(l)
            .map(|w| w.legitimacy))).unwrap_or(1.0);
    assert!(l1 < l0, "the watchers never got caught");
}
