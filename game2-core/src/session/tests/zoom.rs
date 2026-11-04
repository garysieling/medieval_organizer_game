use crate::session::*;

#[test]
fn test_zoom_keeps_the_view_centre_still() {
    // whatever sits at the middle of the map view stays there as you zoom
    let (cx, cy) = (ZOOM_PIVOT.0 as i32, ZOOM_PIVOT.1 as i32);
    let (tx, ty) = (11.4f32, 11.4f32); // a tile near the view centre at zoom 1, camera at rest
    let at1 = iso_to_screen_cam(tx, ty, 0, 0, 1.0);
    for z in [0.5, 1.5, 2.0] {
        let at = iso_to_screen_cam(tx, ty, 0, 0, z);
        let (d1, dz) = ((at1.0 - cx).abs() + (at1.1 - cy).abs(), (at.0 - cx).abs() + (at.1 - cy).abs());
        assert!(dz as f32 <= d1 as f32 * z.max(1.0) + 2.0, "zoom {} moved the centre tile from {:?} to {:?}", z, at1, at);
    }
}
