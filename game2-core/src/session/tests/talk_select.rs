use crate::session::*;

/// Stand villager `i` at (`dx`, `dy`) off the player and every other
/// villager far away - tests that steer a specific someone need the rest
/// out of a shout
fn alone_at(g: &mut Game, i: usize, dx: f32, dy: f32) {
    let (px, py) = g.player_pos();
    for (k, v) in g.villagers.clone().into_iter().enumerate() {
        *g.world.get_component_mut::<Pos>(v).unwrap() = if k == i {
            Pos { x: px + dx, y: py + dy }
        } else {
            Pos { x: px + 40.0 + k as f32, y: py + 40.0 }
        };
    }
}

/// A spot the player can walk to, `lo`..`hi` tiles off
fn walk_spot(g: &Game, lo: f32, hi: f32) -> (i32, i32) {
    let (px, py) = g.player_pos();
    let from = (px.round() as i32, py.round() as i32);
    (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| {
            let d = ((x as f32 - px).powi(2) + (y as f32 - py).powi(2)).sqrt();
            d > lo && d < hi && g.map.walk_path(from, (x, y)).is_some()
        }).expect("somewhere walkable in range")
}

#[test]
fn test_talk_with_no_pick_singles_out_the_least_lately_heard_nearby() {
    let mut g = Game::new();
    // a couple of days in, so "talked to today" isn't the never-heard 0
    while g.tick < 2 { g.update(); }
    let (px, py) = g.player_pos();
    // villager 1 nearer but talked to today; 3 a step further, never heard
    for (i, v) in g.villagers.clone().into_iter().enumerate() {
        let at = match i { 1 => (px + 1.0, py), 3 => (px + 1.4, py), _ => (px + 40.0 + i as f32, py + 40.0) };
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: at.0, y: at.1 };
    }
    g.world.add_component(g.villagers[1], TalkCount {
        value: 1, last_day_by: [(g.player, g.tick)].into_iter().collect() });
    g.act(0);
    assert_eq!(g.selected_target, 3, "the unheard one over the one just talked to");
    assert!(g.picked && g.pick_near, "T's choice becomes the pick");
    assert!(g.seats[0].talk.is_some(), "a Talk opened");
}

#[test]
fn test_a_pick_too_far_to_talk_re_picks_someone_near() {
    // clicked across the map for a look: too far to talk, so T turns to
    // whoever's within a shout instead of walking over
    let mut g = Game::new();
    alone_at(&mut g, 2, 1.0, 0.0);
    g.selected_target = 5;
    g.picked = true;
    g.pick_near = false;
    g.act(0);
    assert_eq!(g.selected_target, 2, "Talk turns to who's within a shout");
    assert!(g.picked && g.pick_near);
}

#[test]
fn test_a_near_pick_that_drifts_past_a_shout_lets_go() {
    let mut g = Game::new();
    let (px, py) = g.player_pos();
    g.selected_target = 2;
    g.picked = true;
    g.pick_near = true;
    *g.world.get_component_mut::<Pos>(g.villagers[2]).unwrap() =
        Pos { x: px + SHOUT_DIST + 5.0, y: py };
    g.advance_selection();
    assert!(!g.picked, "past a shout and the pick lets go");
}

#[test]
fn test_a_far_pick_keeps_for_the_look_until_they_come_near() {
    let mut g = Game::new();
    let (px, py) = g.player_pos();
    g.selected_target = 2;
    g.picked = true;
    g.pick_near = false;
    *g.world.get_component_mut::<Pos>(g.villagers[2]).unwrap() =
        Pos { x: px + SHOUT_DIST + 5.0, y: py };
    g.advance_selection();
    assert!(g.picked && !g.pick_near, "a far pick is for a look: it keeps");
    // they come within a shout - now a near pick, and drifting out lets go
    *g.world.get_component_mut::<Pos>(g.villagers[2]).unwrap() = Pos { x: px + 2.0, y: py };
    g.advance_selection();
    assert!(g.picked && g.pick_near, "within a shout it's a near pick");
    *g.world.get_component_mut::<Pos>(g.villagers[2]).unwrap() =
        Pos { x: px + SHOUT_DIST + 5.0, y: py };
    g.advance_selection();
    assert!(!g.picked, "drifted back out of a shout");
}

#[test]
fn test_the_talk_done_the_pick_lets_go() {
    let mut g = Game::new();
    alone_at(&mut g, 2, 1.0, 0.0);
    g.selected_target = 2;
    g.picked = true;
    g.pick_near = true;
    g.act(0);
    assert!(g.seats[0].talk.is_some(), "a Talk under way");
    g.command(Command::TalkPick(None)); // take your leave
    assert!(!g.picked, "the talk over, the pick lets go");
}

#[test]
fn test_talk_with_nobody_within_a_shout_says_so() {
    let mut g = Game::new();
    let (px, py) = g.player_pos();
    for (i, v) in g.villagers.clone().into_iter().enumerate() {
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: px + 40.0 + i as f32, y: py + 40.0 };
    }
    assert_eq!(g.disabled_reason(0).as_deref(), Some("no one near enough"));
    g.act(0);
    assert!(g.log.last().unwrap().contains("no one near enough"), "{}", g.log.last().unwrap());
    assert!(g.approach.is_none() && g.seats[0].talk.is_none());
}

#[test]
fn test_unselect_lets_go_the_pick_and_the_talk() {
    let mut g = Game::new();
    alone_at(&mut g, 2, 1.0, 0.0);
    g.selected_target = 2;
    g.picked = true;
    g.pick_near = true;
    g.command(Command::Act(0));
    assert!(g.seats[0].talk.is_some(), "a Talk under way");
    g.command(Command::Unselect);
    assert!(!g.picked && g.approach.is_none() && g.seats[0].talk.is_none(),
        "the pick, the walk-over and the talk all let go");
}

#[test]
fn test_unselect_calls_off_the_walk_over() {
    let mut g = Game::new();
    let spot = walk_spot(&g, 3.0, SHOUT_DIST - 1.0);
    *g.world.get_component_mut::<Pos>(g.villagers[2]).unwrap() =
        Pos { x: spot.0 as f32, y: spot.1 as f32 };
    g.selected_target = 2;
    g.picked = true;
    g.pick_near = true;
    g.command(Command::Act(0));
    assert!(g.approach.is_some(), "walking over to them");
    g.command(Command::Unselect);
    assert!(g.approach.is_none() && !g.picked, "the walk-over's off");
}

#[test]
fn test_a_click_on_the_figure_selects_them() {
    // the box is the figure, not its feet: zoomed in, a click on the body
    // lands where the feet-circle missed the part that shows
    let mut g = Game::new();
    let (px, py) = g.player_pos();
    alone_at(&mut g, 2, 4.0, 4.0);
    let (sx, sy) = iso_to_screen_cam(px + 4.0, py + 4.0, 0, 0, 2.0);
    g.click_map(sx, sy - 40, 0, 0, 2.0); // the chest, well above the feet
    assert!(g.picked && g.selected_target == 2 && g.pick_near,
        "the body's the box, zoomed or not");
}

#[test]
fn test_a_villager_half_behind_a_building_still_clicks() {
    // they're the one in front of the roof: the click answers them, not
    // the building whose shadow they stand in
    let mut g = Game::new();
    let (px, py) = g.player_pos();
    alone_at(&mut g, 2, 4.0, 4.0);
    g.buildings.push((px + 5.0, py + 4.0, BSprite::House(0)));
    let (sx, sy) = iso_to_screen_cam(px + 4.0, py + 4.0, 0, 0, 1.0);
    // inside the villager's box and inside the roof's radius: they win
    g.click_map(sx + 10, sy - 10, 0, 0, 1.0);
    assert!(g.picked && g.selected_target == 2 && g.selected_building.is_none(),
        "the figure in front answers the click");
}
