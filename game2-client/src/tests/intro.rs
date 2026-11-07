use crate::*;

#[test]
fn test_the_card_draws_the_facts_centred() {
    let facts = vec![
        "The market towns of Dunmere and Ingleby send their carts up the road.".to_string(),
        "Bramble Keep and the High Fold keep the lord's peace from the heights.".to_string(),
        "Mysteries hide in the gossip - ask, and listen.".to_string(),
    ];
    let buf = render_intro(&facts);
    let bg = (15u8, 15u8, 25u8);
    let lit = |x: usize, y: usize| { let i = (y * W + x) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg };
    // every band of text on the card is centred, like the title's
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
    // title, subtitle, the three facts, the footer
    assert!(bands.len() >= 5, "expected title, subtitle, facts and footer; found {}", bands.len());
    for (top, lo, hi) in bands {
        let centre = (lo + hi) as i32 / 2;
        assert!((centre - W as i32 / 2).abs() <= 3, "line at y={} is centred at x={}, not {}", top, centre, W / 2);
    }
}

/// For eyes: the prologue card, rendered as it appears after the
/// picker. `INTRO_FRAME=<path.bmp> cargo test -p game2-client --lib
/// intro_card_frame -- --ignored`
#[test]
#[ignore]
fn intro_card_frame() {
    let Ok(path) = std::env::var("INTRO_FRAME") else { return };
    let game = Game::new_seeded(MAP_SEED);
    render_intro(&game.intro_facts()).save_bmp(&path).unwrap();
}
