use crate::session::*;

#[test]
fn test_villagers_stay_in_their_own_village() {
    // on main, Aldric was at x 27.5 by day 1050 - the rivals' village -
    // and stayed there, eating from their store
    let mut game = Game::new();
    let home = *game.world.get_component::<Pos>(game.market_food).unwrap();
    // where each villager actually lives - an outskirts cottage is
    // legitimately far from the store, that's not straying
    let homes: Vec<Option<Pos>> = game.villagers.iter().map(|&v| {
        household::home_of(&game.world, v).and_then(|h| game.world.get_component::<Pos>(h)).map(|p| *p)
    }).collect();
    // and where they work - a farm or a mason's yard south of the wall
    // is a day's post, not wandering
    let posts: Vec<Option<Pos>> = game.villagers.iter().map(|&v| {
        game.world.get_component::<Craft>(v).and_then(|c| c.workplace)
            .and_then(|b| game.world.get_component::<Pos>(b)).map(|p| *p)
    }).collect();
    // a stray is being far from the store AND from your own house for
    // days on end - an errand out and back isn't one
    let mut run = vec![0u64; NAMES.len()];
    let (mut worst, mut who_was_it) = (0u64, 0usize);
    let mut last = 0;
    while game.tick < 1500 {
        game.update();
        if game.tick == last { continue; }
        last = game.tick;
        // the roster grows: births and newcomers take index 32, 33...
        run.resize(game.villagers.len(), 0);
        for (i, v) in game.villagers.iter().enumerate() {
            // the dead lie where they fell - a grave isn't a stray;
            // and the held are at the castle, not wandering - hauled
            // off mid-errand they stand where they were taken, but
            // they're in custody, not adrift
            if villager::is_dead(&game.world, *v) { run[i] = 0; continue; }
            if crate::game::dungeon::unavailable(&game.world, *v) { run[i] = 0; continue; }
            if game.travels.iter().any(|t| t.who == *v)
                || game.world.get_component::<OddJob>(*v).is_some()
                || Some(*v) == game.collector { run[i] = 0; continue; }
            let p = game.world.get_component::<Pos>(*v).unwrap();
            let d = |hp: Pos| ((p.x - hp.x).powi(2) + (p.y - hp.y).powi(2)).sqrt();
            // straying means leaving town: outside the wall (with room
            // for the fields just past it), not the far corner of the
            // market square, which is still home
            let (wx0, wy0, wx1, wy1) = game.town_wall;
            let outside = p.x < wx0 as f32 - 4.0 || p.x > wx1 as f32 + 4.0
                || p.y < wy0 as f32 - 4.0 || p.y > wy1 as f32 + 4.0;
            // newcomers (born or settled mid-run) have no slot in the
            // snapshot - look theirs up live
            let home = homes.get(i).copied().flatten()
                .or_else(|| household::home_of(&game.world, *v)
                    .and_then(|h| game.world.get_component::<Pos>(h)).map(|p| *p));
            let post = posts.get(i).copied().flatten()
                .or_else(|| game.world.get_component::<Craft>(*v)
                    .and_then(|c| c.workplace)
                    .and_then(|b| game.world.get_component::<Pos>(b)).map(|p| *p));
            let adrift = outside
                && home.map(|hp| d(hp) > 4.0).unwrap_or(true)
                && post.map(|pp| d(pp) > 4.0).unwrap_or(true);
            run[i] = if adrift { run[i] + 1 } else { 0 };
            if run[i] > worst { (worst, who_was_it) = (run[i], i); }
        }
    }
    let sp = *game.world.get_component::<Pos>(game.villagers[who_was_it]).unwrap();
    let st = (sp.x.round() as i32, sp.y.round() as i32);
    let route = game.map.walk_path(st, (home.x.round() as i32, home.y.round() as i32)).map(|p| p.len());
    let around: Vec<String> = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| format!("{:?}", game.map.get(st.0 + dx, st.1 + dy))).collect();
    // a month adrift is a defection; a fortnight's foraging east is an
    // excursion that comes home (on main, Aldric stayed for good)
    assert!(worst <= 30, "{} stayed {} days away at {:?} (route home: {:?}) - neighbours {:?}",
        vill_name(who_was_it), worst, st, route, around);
}
