use crate::session::*;

#[test]
fn test_no_one_starves_beside_a_full_store() {
    // the crowd-separation push (3 tiles) used to hold hungry villagers
    // out of eating reach (1 tile) of the store: on main they sat at zero
    // hunger for up to 75 days in a row while it held ~480 servings
    let mut game = Game::new();
    let (mut last, mut run, mut worst, mut who) = (0, vec![0u64; NAMES.len()], 0u64, 0usize);
    while game.tick < 720 {
        game.update();
        if game.tick == last { continue; }
        last = game.tick;
        // the roster grows: births and newcomers take index 32, 33...
        run.resize(game.villagers.len(), 0);
        for (k, &v) in game.villagers.iter().enumerate() {
            // the dead keep their last hunger (zero, if they starved): this
            // test is about the living being able to reach the store
            if game.world.get_component::<Health>(v).is_none() { run[k] = 0; continue; }
            let empty = game.world.get_component::<Hunger>(v).map(|h| h.value <= 0.0).unwrap_or(false);
            run[k] = if empty { run[k] + 1 } else { 0 };
            if run[k] > worst { (worst, who) = (run[k], k); }
        }
    }
    let servings = game.world.get_component::<villager::FoodSource>(game.market_food).unwrap().servings;
    assert!(servings > 0, "the store ran out - this test is about reach, not supply");
    let v = game.villagers[who];
    let sp = *game.world.get_component::<Pos>(v).unwrap();
    let st = (sp.x.round() as i32, sp.y.round() as i32);
    let home = game.world.get_component::<Pos>(game.market_food).unwrap();
    let route = game.map.walk_path(st, (home.x.round() as i32, home.y.round() as i32)).map(|p| p.len());
    let act = game.world.get_component::<Villager>(v).map(|x| x.activity);
    let on_job = game.travels.iter().any(|t| t.who == v) || game.world.get_component::<OddJob>(v).is_some();
    assert!(worst <= 5, "{} went {} days at zero hunger at {:?} (passable {}, route {:?}, act {:?}, job {}) with {} servings in the store",
        vill_name(who), worst, st, game.map.passable(st.0, st.1), route, act, on_job, servings);
}
