use crate::session::*;

#[test]
fn test_the_water_carries_who_boards() {
    let mut g = Game::new();
    let _p = g.seats[0].player;
    g.aboard = Some(0);
    // aboard, the water's walkable and the bank isn't
    g.map.set(10, 5, crate::game::tilemap::Tile::Water);
    g.map.set(11, 5, crate::game::tilemap::Tile::Grass);
    assert!(g.walkable(10.0, 5.0), "the boat can't take the water");
    assert!(!g.walkable(11.0, 5.0), "the boat ran aground");
}

#[test]
fn test_the_bank_lets_you_off() {
    let mut g = Game::new();
    g.aboard = Some(0);
    if !g.seats[0].ctx_actions.contains(&CtxAct::Disembark) {
        g.seats[0].ctx_actions.push(CtxAct::Disembark);
    }
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Disembark).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.aboard.is_none(), "the boat never let him off");
}
