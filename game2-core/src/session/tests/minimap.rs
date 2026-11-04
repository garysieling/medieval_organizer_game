use crate::session::*;

#[test]
fn test_whole_map_fits_in_the_minimap_box() {
    let (w_t, h_t, n) = (MAP_TILES as f32, MAP_TILES as f32, MAP_TILES as f32 - 1.0);
    for &(tx, ty) in &[(0.0, 0.0), (n, 0.0), (0.0, n), (n, n)] {
        let (x, y) = tile_to_minimap(tx, ty, mm_k(w_t, h_t));
        assert!(x >= MM_LEFT as f32 && x <= (MM_LEFT + MM_W) as f32, "x {} for ({},{})", x, tx, ty);
        assert!(y >= MM_TOP as f32 && y <= (MM_TOP + MM_H) as f32, "y {} for ({},{})", y, tx, ty);
    }
}
