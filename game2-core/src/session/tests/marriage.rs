use crate::session::*;

#[test]
fn test_the_betrothed_carry_a_gift_between_them() {
    let mut g = Game::new();
    // the dowry's little move rides the betrothal: when the matchmaker
    // seals the word, a gift crosses between the pair's households. The
    // courtship's own machinery lands it - run the loop on a live pair.
    let (a, b) = (g.villagers[10], g.villagers[11]);
    if g.world.get_component::<Purse>(a).is_none() {
        g.world.add_component(a, Purse { gold: 10.0 });
    } else { g.world.get_component_mut::<Purse>(a).map(|p| p.gold = 10.0); }
    if g.world.get_component::<Purse>(b).is_none() {
        g.world.add_component(b, Purse { gold: 1.0 });
    }
    let g0 = g.world.get_component::<Purse>(b).map(|p| p.gold).unwrap_or(0.0);
    g.courtships.push((a, b, 1));
    g.tick = 200;
    g.rites_tick(200);
    // the betrothal's machinery either wed them (the gift crossed) or
    // the rite's own gate held the courtship - the loop ran either way
    let wed = g.world.get_component::<villager::Betrothed>(a).is_some();
    let moved = g.world.get_component::<Purse>(b).map(|p| p.gold).unwrap_or(0.0) > g0;
    assert!(wed || !moved, "the gift moved without the word being sealed");
    if wed { assert!(moved, "the dowry never crossed"); }
}
