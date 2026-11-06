use crate::*;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;

#[test]
fn test_ground_block_top_face_covers_one_tile() {
    // the pack's ground blocks are 94x97 cubes drawn for a bigger grid
    let block = Sprite { width: 94, height: 97, pixels: vec![255; 94 * 97 * 4] };
    for zoom in [0.5, 1.0, 2.0] {
        let (sx, sy) = (300, 200);
        let (w, h, x, y) = block_rect(&block, sx, sy, zoom);
        let face_w = BLOCK_FACE.0 / 94.0 * w as f32;
        let face_top = y as f32 + BLOCK_FACE.2 * h as f32 / 97.0;
        assert!((face_w - 2.0 * TILE_W as f32 * zoom).abs() <= 1.0, "zoom {}: face {} px wide", zoom, face_w);
        assert!((face_top - (sy as f32 - TILE_H as f32 * zoom)).abs() <= 1.0, "zoom {}: face starts at {}", zoom, face_top);
        assert!((x + w as i32 / 2 - sx).abs() <= 1, "zoom {}: block centred on its tile", zoom);
    }
}

#[test]
fn test_a_building_stands_with_its_base_on_its_tile() {
    let mut buf = Buffer::new(W, H);
    let house = Sprite { width: 70, height: 67, pixels: vec![255; 70 * 67 * 4] };
    draw_on_tile(&mut buf, &house, 300, 200, 1.0);
    let lit: Vec<(usize, usize)> = (0..H).flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|&(x, y)| buf.pixels[(y * W + x) * 4] == 255).collect();
    let (x0, x1) = (lit.iter().map(|p| p.0).min().unwrap(), lit.iter().map(|p| p.0).max().unwrap());
    let bottom = lit.iter().map(|p| p.1).max().unwrap();
    assert_eq!(x1 - x0 + 1, (2.0 * TILE_W as f32 * BUILDING_SCALE) as usize, "a building's base is BUILDING_SCALE tiles wide");
    assert_eq!(bottom as i32 + 1, 200 + TILE_H, "it stands on the tile's front corner");
}

#[test]
fn test_name_label_sits_just_above_the_head_and_centred() {
    let (sx, sy) = (300, 200);
    let (lx, ly) = name_label_pos(sx, sy, "Aldric");
    let label_bottom = ly + GLYPH_H;
    let head = sy + SPRITE_HEAD;
    assert!(label_bottom <= head && head - label_bottom <= 6,
        "label should end just above the head (bottom {} vs head {})", label_bottom, head);
    let centre = lx + "Aldric".len() as i32 * GLYPH_W / 2;
    assert_eq!(centre, sx, "label should be centred over the character");
}
