use crate::session::*;

#[test]
fn test_the_counter_meeting_warms_the_acquaintance() {
    let mut g = Game::new();
    let (buyer, seller) = (g.villagers[1], g.villagers[2]);
    // a fair swap: the buyer's bread for the seller's ale
    g.world.get_component_mut::<Satchel>(buyer)
        .map(|s| s.goods.insert("bread".into(), 4));
    g.world.get_component_mut::<Satchel>(seller)
        .map(|s| s.goods.insert("ale".into(), 4));
    g.world.get_component_mut::<Purse>(buyer).map(|p| p.gold = 5.0);
    let deal = crate::game::livelihood::barter_person(&mut g.world, buyer, seller,
        &[("ale", 2)], &[("bread", 2)]);
    assert!(deal.is_some(), "the swap never went through");
    let met = crate::game::relationships::find_relationship(&g.world, buyer, seller)
        .and_then(|e| g.world.get_component::<crate::game::relationships::Relationship>(e)
            .map(|r| r.interactions)).unwrap_or(0);
    assert!(met >= 1, "the deal never registered");
}
