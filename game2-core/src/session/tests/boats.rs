use crate::session::*;

#[test]
fn test_the_water_carries_who_boards() {
    let mut g = Game::new();
    let _p = g.seats[0].player;
    g.seats[0].aboard = Some(0);
    // aboard, the water's walkable and the bank isn't - for that seat
    g.map.set(10, 5, crate::game::tilemap::Tile::Water);
    g.map.set(11, 5, crate::game::tilemap::Tile::Grass);
    assert!(g.walkable_for(0, 10.0, 5.0), "the boat can't take the water");
    assert!(!g.walkable_for(0, 11.0, 5.0), "the boat ran aground");
    // and nobody else's feet get wet or dry on their say-so (#822)
    assert!(!g.walkable_for(1, 10.0, 5.0), "a boarder's boat carried a stranger");
    assert!(g.walkable_for(1, 11.0, 5.0), "a boarder grounded the whole bank");
}

#[test]
fn test_the_bank_lets_you_off() {
    let mut g = Game::new();
    g.seats[0].aboard = Some(0);
    if !g.seats[0].ctx_actions.contains(&CtxAct::Disembark) {
        g.seats[0].ctx_actions.push(CtxAct::Disembark);
    }
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Disembark).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.seats[0].aboard.is_none(), "the boat never let him off");
}
