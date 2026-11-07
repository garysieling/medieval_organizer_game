use crate::*;
use game2_core::engine::render::Buffer;
use game2_core::game::villager::Pos;

#[test]
fn test_hiding_someone_doesnt_flip_the_wall_piece_in_front() {
    // someone standing just inside the town wall is hidden by it - but the
    // piece hiding them must not paint over the piece in FRONT of it (the
    // occluder re-draw used to flip the seam back to front, leaving a
    // glitched spot on the wall)
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    // a spot where the last wall piece and the front corner tower
    // overlap: the tower stands in front, so it owns the seam (the map
    // is perturbed per seed - find the corner, don't hardcode it)
    let (wx0, wy0, wx1, wy1) = game.town_wall;
    let ((cx, cy), _) = game.map.walls.iter()
        .filter(|((x, y), p)| *p == WallPiece::Tower
            && (wx0..=wx1).contains(x) && (wy0..=wy1).contains(y))
        .max_by_key(|((x, y), _)| x + y).copied().expect("a corner tower");
    let (sx, sy) = iso_to_screen_cam(cx as f32, cy as f32, view.cam_x, view.cam_y, view.zoom);
    let seam = |buf: &Buffer| { let i = ((sy - 30) as usize * W + (sx - 17) as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
    let render_at = |game: &mut Game, x: f32, y: f32| {
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
        view.render(&game.ui_model(), (-1, -1))
    };
    let clear = render_at(&mut game, 16.0, 16.0);
    let hidden = render_at(&mut game, 21.0, 23.0);
    assert_eq!(seam(&clear), seam(&hidden), "hiding the player flipped the tower in front of the wall piece");
}

#[test]
fn test_the_keep_draws_at_its_native_height() {
    let game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let (kx, ky) = *game.map.castle_keeps.first().expect("a castle keep");
    let mut view = view;
    (view.cam_x, view.cam_y) = cam_centred_on(kx as f32, ky as f32);
    let with = view.render(&game.ui_model(), (-1, -1));
    let mut m = game.ui_model();
    m.castle_keeps.clear();
    let without = view.render(&m, (-1, -1));
    let (sx, _) = iso_to_screen_cam(kx as f32, ky as f32, view.cam_x, view.cam_y, view.zoom);
    let mut diff = 0;
    let mut min_y = i32::MAX;
    let mut max_y = 0;
    for y in 0..H as i32 {
        for x in (sx - 80)..(sx + 80) {
            let i = (y as usize * W + x as usize) * 4;
            if with.pixels[i..i+3] != without.pixels[i..i+3] {
                diff += 1;
                min_y = min_y.min(y); max_y = max_y.max(y);
            }
        }
    }
    // keep1 is 116x202 at zoom 1: a stretched keep would tower ~600px; the
    // grand set must draw it near its generated height instead.
    assert!(diff > 5000, "the keep should cover thousands of px, got {}", diff);
    assert!(max_y - min_y > 150 && max_y - min_y < 260, "keep should be ~200px tall, got {}", max_y - min_y);
}

#[test]
fn test_the_keep_sorts_over_whoever_is_behind_it() {
    // the keep is the tallest thing in the country, drawn at its own
    // size: the shared painter's order still puts whoever stands in
    // front of it over its base, and whoever's behind it under it
    let mut game = Game::new_seeded(MAP_SEED);
    let (kx, ky) = *game.map.castle_keeps.first().expect("a castle keep");
    let mut view = View::new(Art::load_fs("../assets", 0));
    (view.cam_x, view.cam_y) = cam_centred_on(kx as f32, ky as f32);
    assert!(view.art.keep_s.is_some() && view.art.castle_grand.is_some(),
        "the grand castle set loads");
    // the player's gold ring is drawn under them: behind the keep it's
    // covered, in front it shows
    let ring_px = |game: &mut Game, x: f32, y: f32| {
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (px, py) = game.player_pos();
        let (sx, sy) = iso_to_screen_cam(px, py, view.cam_x, view.cam_y, view.zoom);
        (sx - 12..sx + 12).flat_map(|x| (sy - 4..sy - 1).map(move |y| (x, y)))
            .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H)
            .filter(|&(x, y)| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (255, 215, 0) })
            .count()
    };
    let _ = ring_px(&mut game, kx as f32 + 8.0, ky as f32 + 8.0); // warm the caches
    // just south of the keep's tile, clear of the castle's curtain;
    // behind it, tucked against its north face
    let in_front = ring_px(&mut game, kx as f32, ky as f32 + 6.0);
    let behind = ring_px(&mut game, kx as f32 - 0.4, ky as f32 - 0.4);
    assert!(in_front > 20, "the ring shows in front of the keep ({} px)", in_front);
    assert!(behind < in_front / 2, "behind the keep the ring should be hidden ({} vs {} px)", behind, in_front);
}

#[test]
fn test_walking_behind_a_house_hides_the_player() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let (bx, by, _) = game.buildings[0];
    // the player's yellow ring is drawn under them: behind the house it's covered
    let ring_px = |game: &Game| {
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (px, py) = game.player_pos();
        let (sx, sy) = iso_to_screen_cam(px, py, view.cam_x, view.cam_y, view.zoom);
        (sx - 12..sx + 12).flat_map(|x| (sy - 4..sy - 1).map(move |y| (x, y)))
            .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H)
            .filter(|&(x, y)| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (255, 215, 0) })
            .count()
    };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: bx + 1.0, y: by + 1.0 };
    let in_front = ring_px(&game);
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: bx - 0.6, y: by - 0.6 };
    let behind = ring_px(&game);
    assert!(in_front > 20, "the ring shows in front of the house ({} px)", in_front);
    assert!(behind < in_front / 2, "behind the house the ring should be hidden ({} vs {} px)", behind, in_front);
}
