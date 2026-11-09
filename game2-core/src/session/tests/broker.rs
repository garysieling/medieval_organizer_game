use crate::session::*;

#[test]
fn test_the_broker_carries_both_holds() {
    let b = crate::game::vocation::Broker { power_loyalty: 0.7, community_loyalty: 0.4, pressure: 0.0 };
    assert_eq!(b.power_loyalty, 0.7, "the castle's hold never read");
    assert_eq!(b.community_loyalty, 0.4, "the community's hold never read");
}

#[test]
fn test_the_squeeze_flips_the_broker_to_the_cause() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::vocation::Broker {
        power_loyalty: 0.2, community_loyalty: 0.9, pressure: 0.85 });
    // a castle's eyes on the cause: the threat's real
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.broker_tick(42);
    let in_org = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.contains(&m)).unwrap_or(false);
    assert!(in_org, "the broker never came over");
    assert!(g.log.iter().any(|l| l.contains("intermediary comes over")),
        "the flip never logged");
}
