use crate::*;
use game2_core::game::villager::Pos;
use game2_core::session::{Goal, Travel};

#[test]
fn test_a_freight_train_pulls_a_wagon() {
    // whoever's driving a cart, a levy, a remit, a tribute or a gift
    // train has a wagon at their feet - the UiModel says where, the
    // view draws it
    let game = Game::new_seeded(MAP_SEED);
    let mut view = View::new(Art::load_fs("../assets", 0));
    // park the camera on the village market so the train's on screen
    let (mx, my) = game.world.get_component::<Pos>(game.market_food)
        .map(|p| (p.x, p.y)).unwrap_or((20.0, 20.0));
    (view.cam_x, view.cam_y) = cam_centred_on(mx, my);
    let mut m = game.ui_model();
    m.trains = vec![(mx, my)];
    let with = view.render(&m, (-1, -1));
    m.trains.clear();
    let without = view.render(&m, (-1, -1));
    let diff = with.pixels.iter().zip(without.pixels.iter())
        .filter(|(a, b)| a != b).count();
    assert!(diff > 0, "the wagon never drew");

    // and the core projects the freight: a traveller on a freight errand
    // shows up in `trains` at their position
    let mut g = Game::new_seeded(MAP_SEED);
    let who = g.villagers[0];
    g.travels.push(Travel { who, path: vec![], goal: Goal::Gift(0, 1) });
    let p = g.world.get_component::<Pos>(who).map(|p| (p.x, p.y)).unwrap();
    let m2 = g.ui_model();
    assert!(m2.trains.iter().any(|&(x, y)| (x - p.0).abs() < 0.5 && (y - p.1).abs() < 0.5),
        "the gift train left no wagon behind: {:?}", m2.trains);
}
