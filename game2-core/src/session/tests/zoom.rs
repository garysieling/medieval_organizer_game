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

#[test]
fn test_sprites_scale_with_zoom() {
    let sprite = Sprite { width: 20, height: 10, pixels: vec![255; 20 * 10 * 4] };
    let count = |scale: f32| {
        let mut buf = Buffer::new(100, 100);
        draw_zoomed(&mut buf, &sprite, 10, 10, scale);
        buf.pixels.chunks(4).filter(|p| p[0] == 255).count()
    };
    assert_eq!(count(1.0), 200);
    assert_eq!(count(0.5), 50, "half zoom should draw a quarter of the pixels");
    assert_eq!(count(2.0), 800);
    // and labels follow a zoomed head
    assert_eq!(head_y(300, 2.0), 300 + 2 * SPRITE_HEAD);
    assert_eq!(name_label_pos(0, label_anchor(300, 0.5), "X").1 + GLYPH_H + LABEL_GAP, head_y(300, 0.5));
}
