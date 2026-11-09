use crate::session::*;

#[test]
fn test_the_take_falls_on_the_purses() {
    let mut g = Game::new();
    let m = g.villagers[0];
    if let Some(p) = g.world.get_component_mut::<crate::session::Purse>(m) {
        p.gold = 20.0;
    }
    g.raid_tick(200);
    assert!(g.raid.is_some(), "the raid never landed");
    let after = g.world.get_component::<crate::session::Purse>(m)
        .map(|p| p.gold).unwrap_or(20.0);
    assert!(after < 20.0, "the take never fell: {}", after);
}

#[test]
fn test_the_castles_watch_halves_the_take() {
    let mut g = Game::new();
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    let m = g.villagers[0];
    if let Some(p) = g.world.get_component_mut::<crate::session::Purse>(m) {
        p.gold = 20.0;
    }
    g.raid_tick(200);
    let after = g.world.get_component::<crate::session::Purse>(m)
        .map(|p| p.gold).unwrap_or(20.0);
    // the watch halves it - the take's 2, not 5
    assert!(after >= 18.0, "the watch never halved: {}", after);
}

#[test]
fn test_the_raided_grieve() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::InnerLife {
        wants: vec![], grievances: vec![], satisfied: vec![] });
    g.raid_tick(200);
    let grieved = g.world.get_component::<crate::game::desires::InnerLife>(m)
        .map(|l| !l.grievances.is_empty()).unwrap_or(false);
    assert!(grieved, "the raided never grieved");
}
