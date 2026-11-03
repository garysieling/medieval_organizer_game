use crate::session::*;

#[test]
fn test_screen_and_tile_projections_agree_on_the_minimap() {
    // a tile drawn on screen lands at the same minimap spot as the tile itself
    let (w_t, h_t) = (MAP_TILES as f32, MAP_TILES as f32);
    for &(tx, ty, cam_x, cam_y, zoom) in &[(10.0f32, 16.0f32, 0, 0, 1.0f32), (28.0, 13.0, -120, 40, 1.0), (5.0, 2.0, 30, -10, 1.5)] {
        let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
        let a = screen_to_minimap(sx as f32, sy as f32, cam_x, cam_y, zoom, w_t, h_t);
        let b = tile_to_minimap(tx, ty, mm_k(w_t, h_t));
        assert!((a.0 - b.0).abs() < 1.0 && (a.1 - b.1).abs() < 1.0, "{:?} vs {:?}", a, b);
    }
}

#[test]
fn test_whole_map_fits_in_the_minimap_box() {
    let (w_t, h_t, n) = (MAP_TILES as f32, MAP_TILES as f32, MAP_TILES as f32 - 1.0);
    for &(tx, ty) in &[(0.0, 0.0), (n, 0.0), (0.0, n), (n, n)] {
        let (x, y) = tile_to_minimap(tx, ty, mm_k(w_t, h_t));
        assert!(x >= MM_LEFT as f32 && x <= (MM_LEFT + MM_W) as f32, "x {} for ({},{})", x, tx, ty);
        assert!(y >= MM_TOP as f32 && y <= (MM_TOP + MM_H) as f32, "y {} for ({},{})", y, tx, ty);
    }
}

#[test]
fn test_minimap_and_map_share_terrain_colours() {
    // heights 0.15-0.35 are grass on the map; the old minimap drew them as water
    assert_eq!(terrain_colour(0.25), (60, 140, 60));
    assert_eq!(terrain_colour(0.05), (30, 60, 180));
}
