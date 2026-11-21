use crate::*;

#[test]
fn test_the_market_towns_draw_the_burgh_set() {
    // the country settlements' own skyline (#1272): the region set
    // loads at burgh height, the model marks town houses Burgh, and a
    // crag stands for the castles' keeps
    let art = Art::load_fs("../assets", 0);
    assert_eq!(art.burgh.len(), 20, "the burgh set didn't load whole");
    assert!(art.burgh.iter().all(|s| s.is_some()), "a burgh piece is missing");
    assert!(art.burgh.iter().flatten().all(|s| s.height == 80),
        "a burgh piece kept its 160px height");
    assert!(art.burgh_keep.is_some(), "the castles' crag didn't load");
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    assert!(!m.settlement_houses.is_empty(), "the country settled no towns");
    assert!(m.settlement_houses.iter().all(|&(.., s)| matches!(s, BSprite::Burgh(_))),
        "a town house still wears a cottage");
}
