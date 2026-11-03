use crate::session::*;

#[test]
fn test_rain_dims_and_cools_evenly() {
    let grass = (60, 140, 60);
    let wet = rain_wash(grass);
    assert!(wet.1 < grass.1, "rain should dim the land: {:?}", wet);
    // the same pixel gets the same wash on every row - no stripes
    assert_eq!(rain_wash(grass), rain_wash(grass));
    let (r, _, b) = rain_wash((200, 200, 200));
    assert!(b > r, "rain light should be cool, not neutral: r={} b={}", r, b);
}

#[test]
fn test_rainy_frame_has_no_row_banding() {
    // render a flat map region, apply the rain wash the way render() does,
    // and check neighbouring rows match
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, MAP_W as i32, H as i32, 140, 110, 50);
    for y in 0..H { for x in 0..MAP_W {
        let idx = (y * W + x) * 4;
        let (r, g, b) = rain_wash((buf.pixels[idx], buf.pixels[idx + 1], buf.pixels[idx + 2]));
        buf.set(x as i32, y as i32, r, g, b);
    }}
    let row = |y: usize| { let i = (y * W + 100) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
    for y in 0..12 { assert_eq!(row(y), row(y + 1), "rows {} and {} differ - banding", y, y + 1); }
}
