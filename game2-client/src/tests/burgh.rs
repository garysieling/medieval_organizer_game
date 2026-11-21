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

#[test]
fn test_the_pose_sheet_loads_and_the_model_poses_villagers() {
    // the 9-pose sheet wired (#1280): every cell loads, and a villager
    // mid-routine carries a pose the client can draw
    let art = Art::load_fs("../assets", 0);
    assert_eq!(art.poses.len(), 8, "a pose cell didn't load");
    assert!(art.pose("hammering").is_some() && art.pose("sleeping").is_some());
    // the model knows the mapping: a working routine poses a villager
    let mut g = Game::new_seeded(MAP_SEED);
    let v = g.villagers[0];
    g.world.add_component(v, game2_core::session::routine::Routine {
        mode: game2_core::session::routine::Mode::Working,
        spot: game2_core::session::routine::Spot::Here,
        anchor: (0.0, 0.0), face: None, drift: 0.0, leads: false,
        day: 0, reached: false });
    let m = g.ui_model();
    let pv = m.people.iter().find(|p| p.e == v).expect("the villager's in the model");
    assert!(matches!(pv.pose, 1 | 2), "a working routine gave pose {}", pv.pose);
}
