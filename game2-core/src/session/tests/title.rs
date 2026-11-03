use crate::session::*;

#[test]
fn test_title_lines_are_centred_and_say_how_to_win() {
    let buf = render_title();
    let bg = (15u8, 15u8, 25u8);
    let lit = |x: usize, y: usize| { let i = (y * W + x) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg };
    // find each band of text rows and check it's centred on the screen
    let mut bands = vec![];
    let mut y = 0;
    while y < H {
        if (0..W).any(|x| lit(x, y)) {
            let top = y;
            while y < H && (0..W).any(|x| lit(x, y)) { y += 1; }
            let (mut lo, mut hi) = (W, 0);
            for yy in top..y { for x in 0..W { if lit(x, yy) { lo = lo.min(x); hi = hi.max(x); } } }
            bands.push((top, lo, hi));
        }
        y += 1;
    }
    assert!(bands.len() >= 5, "expected title, subtitle, goal, keys and start lines; found {}", bands.len());
    for (top, lo, hi) in bands {
        let centre = (lo + hi) as i32 / 2;
        assert!((centre - W as i32 / 2).abs() <= 3, "line at y={} is centred at x={}, not {}", top, centre, W / 2);
    }
}
