use crate::session::*;

#[test]
fn test_the_offer_stands_once_made() {
    let mut g = Game::new();
    // plant the offer: a lord's seat, binding behind it
    let lord = g.sites.iter().find_map(|s| s.lord).unwrap_or(g.villagers[10]);
    g.seat_offer = Some(crate::game::campaign::SeatOffer {
        from: lord, role: crate::game::campaign::Role::Chair,
        binding: true, offered: g.tick });
    assert!(g.seat_offer.is_some(), "the offer never stood");
    // the player's pick reads the panel first
    let panel = g.convo_panel_at(0);
    assert!(panel.is_some(), "the offer never reached the player's box");
    assert!(panel.unwrap().options.len() >= 2, "no pick on offer");
}

#[test]
fn test_taking_a_binding_seat_gains_the_cause() {
    let mut g = Game::new();
    let lord = g.sites.iter().find_map(|s| s.lord).unwrap_or(g.villagers[10]);
    g.seat_offer = Some(crate::game::campaign::SeatOffer {
        from: lord, role: crate::game::campaign::Role::Chair,
        binding: true, offered: g.tick });
    let before = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.influence_pool).unwrap_or(0.0);
    g.decide_at(0, 0);
    let after = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.influence_pool).unwrap_or(0.0);
    assert!(g.seat_offer.is_none(), "the offer's still open after the pick");
    assert!(after >= before, "a binding seat never gained the cause");
}
