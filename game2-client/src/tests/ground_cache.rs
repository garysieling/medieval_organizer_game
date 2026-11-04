use crate::*;
use game2_core::engine::render::Buffer;

#[test]
fn test_recycled_frames_match_fresh_ones() {
    // render draws into the last frame handed back with recycle(),
    // without clearing it: every pixel must still be repainted. Two
    // views go through the same frames - a pan (ground drawn straight
    // in), then still (cached ground), in rain - one recycling, one not
    let mut g = Game::new_seeded(3);
    for _ in 0..30 { g.update(); }
    g.precip = Some(PrecipKind::Rain);
    let recycling = View::new(Art::load_fs("../assets", 0));
    let fresh = View::new(Art::load_fs("../assets", 0));
    let mut cams = vec![(0, 0), (240, -90), (240, -90), (-300, 200), (-300, 200)];
    let (mut recycling, mut fresh) = (recycling, fresh);
    for (i, cam) in cams.drain(..).enumerate() {
        for v in [&mut recycling, &mut fresh] { (v.cam_x, v.cam_y) = cam; v.zoom = if i < 3 { 1.0 } else { 1.6 }; }
        let m = g.ui_model();
        let a = recycling.render(&m, (-1, -1));
        let b = fresh.render(&m, (-1, -1));
        assert!(a.pixels == b.pixels, "frame {} (camera {:?}): a recycled buffer showed through", i, cam);
        recycling.recycle(a);
        g.update();
    }
}

#[test]
fn test_cached_ground_matches_drawing_it_straight() {
    // the world-space layer blitted each frame must be exactly what the
    // tile pass would have drawn - any tile dropped or shifted shows up
    // as bare background or a seam
    let game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    // exact comparison at integer zooms, where the layer's draw shift is
    // a whole number of pixels so it lands exactly where the culled pass
    // put it (at fractional zooms the layer can sit a sub-pixel off and
    // still be right)
    for &zoom in &[1.0f32, 2.0] {
        for camera in [(0, 0), (35, -20)] {
        let mut direct = Buffer::new(W, H);
        direct.rect(0, 0, W as i32, H as i32, 25, 25, 38);
        view.draw_ground(&game.ui_model(), &mut direct, camera.0, camera.1, zoom);
        let g = view.render_ground(&game.ui_model(), zoom, camera);
        let mut cached = Buffer::new(W, H);
        cached.rect(0, 0, W as i32, H as i32, 25, 25, 38);
        cached.blit_from(&g.buf, 0, 0, 0, 0, MAP_W as i32, H as i32);
        if std::env::var("DUMP_GROUND").is_ok() {
            direct.save_bmp(&format!("../evidence/ground-direct-{}.bmp", (zoom * 100.0) as i32)).ok();
            cached.save_bmp(&format!("../evidence/ground-cache-{}.bmp", (zoom * 100.0) as i32)).ok();
            g.buf.save_bmp(&format!("../evidence/ground-layer-{}.bmp", (zoom * 100.0) as i32)).ok();
        }
        // the layer can legitimately show MORE than the old culled pass
        // (edge blocks that used to pop in), but nothing it drew may be
        // missing or misplaced inside the map viewport
        let bg = [25u8, 25, 38, 255];
        let (mut missing, mut bounds) = (0, [i32::MAX, i32::MAX, i32::MIN, i32::MIN]);
        for y in 0..H { for x in 0..MAP_W {
            let i = (y * W + x) * 4;
            if direct.pixels[i..i + 4] != bg && direct.pixels[i..i + 4] != cached.pixels[i..i + 4] {
                missing += 1;
                bounds = [bounds[0].min(x as i32), bounds[1].min(y as i32), bounds[2].max(x as i32), bounds[3].max(y as i32)];
            }
        }}
        assert_eq!(missing, 0, "zoom {} camera {:?}: ground pixels dropped by the cache in {:?}", zoom, camera, bounds);
        }
    }
}

#[test]
fn test_expanded_map_ground_cache_is_viewport_sized_at_zoom_two() {
    let game = Game::new_seeded(231);
    assert_eq!(game.map.width, 245);
    let mut view = View::new(Art::load_fs("../assets", 0));
    let model = game.ui_model();
    view.zoom = 2.0;
    // The first pass at a new zoom draws directly while input may still
    // be changing it; a settled pass creates only the viewport cache.
    view.render(&model, (-1, -1));
    view.render(&model, (-1, -1));
    {
        let cache = view.ground.borrow();
        let cache = cache.as_ref().expect("settled zoom should cache the viewport");
        assert_eq!((cache.buf.width, cache.buf.height), (MAP_W, H));
        assert_eq!(cache.buf.pixels.len(), MAP_W * H * 4);
    }
    let bounds = visible_tile_bounds(view.cam_x, view.cam_y, view.zoom, MAP_W, H, 245, 245);
    let visited = (bounds.2 - bounds.0 + 1) * (bounds.3 - bounds.1 + 1);
    assert!(visited < 25_000, "viewport culling should skip most of the 60,025 map tiles (visited {})", visited);
    view.cam_x += 35;
    view.render(&model, (-1, -1));
    view.render(&model, (-1, -1));
    assert_eq!(view.ground.borrow().as_ref().unwrap().camera, (view.cam_x, view.cam_y));
}
