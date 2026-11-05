use crate::*;
use game2_core::engine::render::Buffer;

#[test]
fn test_the_marker_is_a_bobbing_gold_dart() {
    // the gold centroid of the marker on a frame: it should hover above
    // the given top and bob a couple of px as the frame counter turns
    let centroid = |frame: u32| {
        let mut buf = Buffer::new(120, 120);
        draw_player_marker(&mut buf, 60, 60, 1.0, frame);
        let (mut n, mut sy) = (0i64, 0i64);
        for (i, p) in buf.pixels.chunks(4).enumerate() {
            if p[0] == 255 && p[1] == 215 && p[2] == 0 {
                n += 1;
                sy += (i / 120) as i64;
            }
        }
        assert!(n > 0, "frame {frame}: the marker painted no gold");
        sy as f32 / n as f32
    };
    let (still, bobbed) = (centroid(0), centroid(13));
    assert!(still < 60.0, "the dart hovers above the name top: {still}");
    assert!((still - bobbed).abs() > 0.5, "the dart should bob between frames: {still} vs {bobbed}");
}
