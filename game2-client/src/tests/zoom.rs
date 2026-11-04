use crate::*;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;

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
