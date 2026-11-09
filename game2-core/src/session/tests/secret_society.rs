use crate::session::*;

#[test]
fn test_the_deoathing_frays_the_oath() {
    let mut g = Game::new();
    g.oath_intensity = 2;
    // a castle's eyes on the cause
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.counter_ritual_tick(27);
    assert!(g.oath_intensity < 2, "the oath never frayed");
    assert!(g.log.iter().any(|l| l.contains("de-oathing")),
        "the ceremony never logged");
}

#[test]
fn test_the_front_halves_the_exposure() {
    let mut g = Game::new();
    g.front = true;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::psychology::Bond { target: g.villagers[1], security: 0.5 });
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !o.members.contains(&m) { o.members.push(m); }
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.counter_ritual_tick(27);
    let after = g.world.get_component::<crate::sim::psychology::Bond>(m)
        .map(|b| b.security).unwrap_or(0.5);
    // the front's own armor: the wing takes a light exposure, not a raid
    assert!(after >= 0.48, "the front never armored: {}", after);
}
