use crate::session::*;

#[test]
fn test_the_dusty_trades_accrue_their_toll() {
    let mut g = Game::new();
    let m = g.villagers.iter().copied().find(|&v| g.world.get_component::<crate::game::vocation::Craft>(v)
        .map(|c| c.vocation == crate::game::vocation::Vocation::Mason).unwrap_or(false));
    let Some(m) = m else { return }; // no mason on this seed - the test's a no-op
    g.trade_exposure_tick(3);
    let ex = g.world.get_component::<crate::game::vocation::Craft>(m).map(|c| c.exposure).unwrap_or(0.0);
    assert!(ex > 0.0, "the mason never accrued the dust");
}

#[test]
fn test_past_the_threshold_the_trade_hits() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::vocation::Craft {
        vocation: crate::game::vocation::Vocation::Mason, skill: 0.5,
        workplace: None, master: None, exposure: 0.5, tasked_by: None, pending: 0.0, apprentice_years: 0, stage: 0 });
    g.world.add_component(m, crate::sim::illness::Health { condition: 0.8, immunity: 0.0 });
    g.trade_exposure_tick(3);
    let cond = g.world.get_component::<crate::sim::illness::Health>(m).map(|h| h.condition).unwrap_or(0.8);
    assert!(cond < 0.8, "the trade's own toll never landed: {}", cond);
    assert!(g.log.iter().any(|l| l.contains("the trade's own toll")),
        "the toll never logged");
}
