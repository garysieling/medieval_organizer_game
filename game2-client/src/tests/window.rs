use game2_core::engine::sprite::Sprite;

#[test]
fn test_the_stained_window_paints_a_rose() {
    // a blank church block: the glass goes over the facade's middle
    let block = Sprite { width: 40, height: 60, pixels: vec![200; 40 * 60 * 4] };
    let out = crate::art::stained_glass(&block);
    let at = |x: usize, y: usize| -> [u8; 4] {
        let i = (y * 40 + x) * 4;
        [out.pixels[i], out.pixels[i + 1], out.pixels[i + 2], out.pixels[i + 3]]
    };
    let (cx, cy) = (20, (60.0 * 0.32) as usize);
    assert_eq!(at(cx, cy)[0..3], [216, 164, 56], "the rose's heart isn't gold");
    let red = (168u8, 36u8, 40u8);
    let blue = (56u8, 88u8, 168u8);
    let mut saw_red = false;
    let mut saw_blue = false;
    for dy in -4i32..=4 {
        for dx in -4i32..=4 {
            if dx * dx + dy * dy > 16 { continue; }
            let p = at((cx as i32 + dx) as usize, (cy as i32 + dy) as usize);
            if (p[0], p[1], p[2]) == red { saw_red = true; }
            if (p[0], p[1], p[2]) == blue { saw_blue = true; }
            assert_eq!(p[3], 255, "a pane's translucent");
        }
    }
    assert!(saw_red && saw_blue, "the rose has one colour only");
    // outside the rose the facade's untouched
    assert_eq!(at(2, 58)[0..3], [200, 200, 200], "the glass leaked off the rose");
}
