use crate::*;
use game2_core::engine::render::Buffer;

#[test]
fn test_rain_dims_and_cools_evenly() {
    let grass = (60, 140, 60);
    let wet = washed(PrecipKind::Rain, grass);
    assert!(wet.1 < grass.1, "rain should dim the land: {:?}", wet);
    // the same pixel gets the same wash on every row - no stripes
    assert_eq!(washed(PrecipKind::Rain, grass), washed(PrecipKind::Rain, grass));
    let (r, _, b) = washed(PrecipKind::HeavyRain, (200, 200, 200));
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
        let (r, g, b) = washed(PrecipKind::Rain, (buf.pixels[idx], buf.pixels[idx + 1], buf.pixels[idx + 2]));
        buf.set(x as i32, y as i32, r, g, b);
    }}
    let row = |y: usize| { let i = (y * W + 100) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
    for y in 0..12 { assert_eq!(row(y), row(y + 1), "rows {} and {} differ - banding", y, y + 1); }
}

fn washed(kind: PrecipKind, px: (u8, u8, u8)) -> (u8, u8, u8) {
    let lut = weather_lut(kind);
    (lut[0][px.0 as usize], lut[1][px.1 as usize], lut[2][px.2 as usize])
}

#[test]
fn test_the_haar_drowns_contrast_more_than_drizzle() {
    // a light and a dark patch of ground: fog pulls them together
    let (light, dark) = ((190, 170, 120), (40, 60, 30));
    let spread = |a: (u8, u8, u8), b: (u8, u8, u8)|
        (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs() + (a.2 as i32 - b.2 as i32).abs();
    let clear = spread(light, dark);
    let haar = spread(washed(PrecipKind::Haar, light), washed(PrecipKind::Haar, dark));
    let drizzle = spread(washed(PrecipKind::Drizzle, light), washed(PrecipKind::Drizzle, dark));
    assert!(haar * 2 < clear, "haar should halve the contrast: {} of {}", haar, clear);
    assert!(haar < drizzle, "haar {} should be flatter than drizzle {}", haar, drizzle);
    // and it's pale: the dark ground lifts
    let d = washed(PrecipKind::Haar, dark);
    assert!(d.0 > dark.0 && d.2 > dark.2, "haar should lift the dark ground: {:?}", d);
}

#[test]
fn test_each_weather_kind_washes_the_land_its_own_colour() {
    use PrecipKind::*;
    let grass = (60, 140, 60);
    let kinds = [Drizzle, LightRain, Showers, Rain, HeavyRain, Haar];
    let looks: Vec<_> = kinds.iter().map(|&k| washed(k, grass)).collect();
    for i in 0..looks.len() { for j in i + 1..looks.len() {
        assert_ne!(looks[i], looks[j], "{:?} and {:?} wash alike", kinds[i], kinds[j]);
    }}
    // the sunlit showers are warmer than the stinging squall
    let warm = |c: (u8, u8, u8)| c.0 as i32 - c.2 as i32;
    assert!(warm(washed(Showers, grass)) > warm(washed(HeavyRain, grass)));
}

#[test]
fn test_haar_frame_washes_without_particles() {
    // every map pixel of a flat field goes the same way: no streaks, no drops
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, MAP_W as i32, H as i32, 60, 140, 60);
    draw_weather(&mut buf, Some(PrecipKind::Haar), 7, None);
    let at = |x: usize, y: usize| { let i = (y * W + x) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
    let first = at(100, 100);
    assert_eq!(first, washed(PrecipKind::Haar, (60, 140, 60)));
    for y in (0..H).step_by(7) { for x in (0..MAP_W).step_by(5) {
        assert_eq!(at(x, y), first, "haar left a mark at {},{}", x, y);
    }}
}
