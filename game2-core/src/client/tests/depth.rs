use crate::client::*;
use crate::engine::render::Buffer;
use crate::game::villager::Pos;

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
