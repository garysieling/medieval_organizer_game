use crate::session::*;

#[test]
fn test_the_charter_lands_on_the_watch() {
    let mut g = Game::new();
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.incorporation_tick(63);
    assert!(g.incorporation_offer.is_some(), "the charter never came");
    assert!(g.log.iter().any(|l| l.contains("a charter offered")),
        "the offer never logged");
}

#[test]
fn test_the_accept_pays_and_dulls() {
    let mut g = Game::new();
    g.incorporation_offer = Some(g.tick);
    g.decide_at(0, 0); // accept
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.routinization).unwrap_or(0.0);
    assert!(after > 0.0, "the routinization never grew: {}", after);
    assert!(g.log.iter().any(|l| l.contains("the charter's taken")),
        "the accept never logged");
}

#[test]
fn test_the_refuse_keeps_the_edge() {
    let mut g = Game::new();
    g.incorporation_offer = Some(g.tick);
    g.decide_at(0, 1); // refuse
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.routinization).unwrap_or(0.0);
    assert!(after == 0.0, "the refusal dulled anyway: {}", after);
    assert!(g.log.iter().any(|l| l.contains("the charter's refused")),
        "the refuse never logged");
}
