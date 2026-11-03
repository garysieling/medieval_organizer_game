//! Weather visuals: falling rain/snow particles, overcast dimming, rainbow.
//! Drawn over the map viewport (x < map_w). Marks are faint over terrain so the
//! map stays readable, and heavier over the empty background around it.

use crate::engine::render::Buffer;
use crate::sim::precipitation::PrecipKind;

fn hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn blend(buf: &mut Buffer, x: i32, y: i32, c: (u8, u8, u8), a: f32) {
    if x < 0 || y < 0 || x >= buf.width as i32 || y >= buf.height as i32 || a <= 0.0 { return; }
    let i = (y as usize * buf.width + x as usize) * 4;
    let a = a.min(1.0);
    let mix = |dst: u8, src: u8| (dst as f32 + (src as f32 - dst as f32) * a) as u8;
    buf.pixels[i] = mix(buf.pixels[i], c.0);
    buf.pixels[i + 1] = mix(buf.pixels[i + 1], c.1);
    buf.pixels[i + 2] = mix(buf.pixels[i + 2], c.2);
}

/// Which pixels of the map viewport are empty background (true) vs terrain.
/// Take this before drawing overlays so weather can tell sky from ground.
pub fn background_mask(buf: &Buffer, map_w: usize, bg: (u8, u8, u8)) -> Vec<bool> {
    let mut m = vec![false; map_w * buf.height];
    for y in 0..buf.height {
        for x in 0..map_w {
            let i = (y * buf.width + x) * 4;
            m[y * map_w + x] = buf.pixels[i] == bg.0 && buf.pixels[i + 1] == bg.1 && buf.pixels[i + 2] == bg.2;
        }
    }
    m
}

struct Layer<'a> { buf: &'a mut Buffer, mask: &'a [bool], map_w: i32 }

impl Layer<'_> {
    fn outside(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.map_w && (y as usize) < self.buf.height
            && self.mask[y as usize * self.map_w as usize + x as usize]
    }
    /// Blend with a light alpha over terrain and a heavy one over background
    fn mark(&mut self, x: i32, y: i32, c: (u8, u8, u8), a_map: f32, a_out: f32) {
        if x < 0 || x >= self.map_w { return; }
        let a = if self.outside(x, y) { a_out } else { a_map };
        blend(self.buf, x, y, c, a);
    }
}

/// Dim/haze the map viewport to match the sky
pub fn overcast(buf: &mut Buffer, map_w: usize, kind: PrecipKind) {
    let k = kind.intensity();
    // snow skies are grey: dimming the (often snow-white) ground lets white flakes show
    let (tint, a): ((u8, u8, u8), f32) = if kind.is_snow() { ((110, 115, 135), 0.14 + 0.18 * k) } else { ((40, 50, 75), 0.10 + 0.22 * k) };
    // the same mix as blend(), per channel, as tables: a million float
    // blends a frame was a visible cost
    let mix = |dst: u8, src: u8| (dst as f32 + (src as f32 - dst as f32) * a.min(1.0)) as u8;
    let lut: [[u8; 256]; 3] = std::array::from_fn(|ch| std::array::from_fn(|v| mix(v as u8, [tint.0, tint.1, tint.2][ch])));
    let w = map_w.min(buf.width);
    for y in 0..buf.height {
        let row = &mut buf.pixels[y * buf.width * 4..(y * buf.width + w) * 4];
        for px in row.chunks_exact_mut(4) {
            px[0] = lut[0][px[0] as usize]; px[1] = lut[1][px[1] as usize]; px[2] = lut[2][px[2] as usize];
        }
    }
}

/// Falling precipitation for this frame (~12 fps animation clock)
pub fn draw_precip(buf: &mut Buffer, map_w: usize, mask: &[bool], kind: PrecipKind, frame: u64) {
    let h = buf.height as i32;
    let mut l = Layer { buf, mask, map_w: map_w as i32 };
    if kind.is_snow() { draw_snow(&mut l, h, kind, frame) } else { draw_rain(&mut l, h, kind, frame) }
}

fn draw_rain(l: &mut Layer, h: i32, kind: PrecipKind, frame: u64) {
    // (count, streak length, speed px/frame, slant dx per dy, color, alpha map, alpha outside)
    let (count, len, speed, slant, c, a_map, a_out) = match kind {
        PrecipKind::Drizzle   => (280, 3, 7, 0.08, (195, 205, 220), 0.32, 0.60),
        PrecipKind::LightRain => (110, 5, 20, 0.12, (160, 180, 215), 0.34, 0.65),
        PrecipKind::Showers   => (240, 7, 26, 0.25, (160, 185, 225), 0.38, 0.70),
        PrecipKind::Rain      => (200, 9, 30, 0.20, (155, 175, 215), 0.40, 0.75),
        _                     => (380, 14, 38, 0.38, (180, 200, 235), 0.45, 0.90),
    };
    // showers come in bursts: density swells and fades every few seconds
    let density = if kind == PrecipKind::Showers {
        let t = frame as f32 / 20.0;
        0.15 + 0.85 * (0.5 + 0.5 * t.sin()).powf(1.5)
    } else { 1.0 };
    let shown = (count as f32 * density) as u64;
    let wrap = h + 80;
    for i in 0..shown {
        let r = hash(i * 7919 + 13);
        let y = ((r % wrap as u64) as i64 + (frame as i64 * speed as i64 * (90 + (r >> 40) as i64 % 20) / 100)) % wrap as i64 - 40;
        let y = y as i32;
        let x0 = ((r >> 16) % (l.map_w as u64 + 300)) as i32 - 150;
        let x = x0 + (y as f32 * slant) as i32;
        let x = x.rem_euclid(l.map_w);
        // drops over the background get longer streaks; over terrain they stay short
        let len = if l.outside(x, y) { len + len / 2 } else { len };
        for k in 0..len {
            let fy = y - k;
            let fx = x - (k as f32 * slant) as i32;
            // tail fades
            let t = 1.0 - k as f32 / (len as f32 + 1.0);
            l.mark(fx, fy, c, a_map * t, a_out * t);
        }
    }
    // splashes on the ground for real rain (not drizzle): little crowns that blink
    let splashes = match kind {
        PrecipKind::Rain => 30, PrecipKind::HeavyRain => 60,
        PrecipKind::Showers => (40.0 * density) as u64, _ => 0,
    };
    for i in 0..splashes {
        let r = hash(i * 31 + frame / 2 * 1_000_003);
        let x = (r % l.map_w as u64) as i32;
        let y = ((r >> 20) % h as u64) as i32;
        if l.outside(x, y) { continue; }
        for (dx, dy) in [(-1, 0), (1, 0), (-2, 1), (2, 1)] { l.mark(x + dx, y + dy, c, 0.45, 0.0); }
    }
}

fn draw_snow(l: &mut Layer, h: i32, kind: PrecipKind, frame: u64) {
    // (count, speed px/frame, wind dx per dy, sway px)
    let (count, speed, wind, sway) = match kind {
        PrecipKind::Flurries => (70, 2.0, 0.1, 6.0),
        PrecipKind::Snow     => (180, 3.0, 0.15, 5.0),
        _                    => (380, 6.0, 0.55, 3.0),
    };
    let c = (240, 245, 255);
    let wrap = h + 20;
    for i in 0..count {
        let r = hash(i * 104_729 + 7);
        let fall = speed * (0.6 + ((r >> 32) % 80) as f32 / 100.0);
        let y = ((r % wrap as u64) as f32 + frame as f32 * fall) as i32 % wrap - 10;
        let phase = ((r >> 24) % 628) as f32 / 100.0;
        let x0 = ((r >> 8) % (l.map_w as u64 + 300)) as i32 - 150;
        let x = x0 + (y as f32 * wind) as i32 + (sway * (frame as f32 / 7.0 + phase).sin()) as i32;
        let x = x.rem_euclid(l.map_w);
        // flake size: flurries are specks, heavy snow is big clumps.
        // Over background each flake grows one size so the edges read heavier.
        let base = match kind { PrecipKind::Flurries => (r >> 50) % 2, PrecipKind::Snow => 1 + (r >> 50) % 2, _ => 2 + (r >> 50) % 2 };
        let size = if l.outside(x, y) { (base + 1).min(3) } else { base };
        let (am, ao) = (0.7, 1.0);
        // a faint shadow under each flake so it still shows over snowy ground
        let shadow = (90, 100, 130);
        let reach = size.min(2) as i32;
        l.mark(x + 1, y + reach + 1, shadow, 0.5, 0.0);
        l.mark(x + reach + 1, y + 1, shadow, 0.35, 0.0);
        match size {
            0 => l.mark(x, y, c, am, ao),
            1 => { for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] { l.mark(x + dx, y + dy, c, am, ao); } }
            2 => { for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] { l.mark(x + dx, y + dy, c, am, ao); } }
            _ => {
                // six-armed asterisk
                for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-2, 0), (2, 0), (0, -2), (0, 2),
                                 (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    l.mark(x + dx, y + dy, c, am * 0.9, ao * 0.9);
                }
            }
        }
    }
}

/// Rainbow opacity given frames since the rain stopped: fade in, linger, fade out.
pub const RAINBOW_FRAMES: u64 = 260;
pub fn rainbow_alpha(frames_since_clear: u64) -> f32 {
    let f = frames_since_clear as f32;
    let (fade_in, hold_until, end) = (30.0, 150.0, RAINBOW_FRAMES as f32);
    if f < fade_in { f / fade_in }
    else if f < hold_until { 1.0 }
    else if f < end { 1.0 - (f - hold_until) / (end - hold_until) }
    else { 0.0 }
}

/// Arc across the map viewport, centered below the bottom edge
pub fn draw_rainbow(buf: &mut Buffer, map_w: usize, mask: &[bool], alpha: f32) {
    if alpha <= 0.0 { return; }
    const BANDS: [(u8, u8, u8); 7] = [
        (235, 60, 50), (245, 140, 40), (245, 220, 60), (80, 200, 90),
        (60, 140, 230), (80, 70, 190), (150, 80, 200),
    ];
    let band_w = 6.0;
    let h = buf.height as f32;
    let cx = map_w as f32 / 2.0;
    let r_out = map_w as f32 * 0.46;
    let cy = r_out + h * 0.1; // top of the bow sits near the top of the view
    let r_in = r_out - band_w * BANDS.len() as f32;
    let mut l = Layer { buf, mask, map_w: map_w as i32 };
    for y in 0..h as i32 {
        let dy = cy - y as f32;
        if dy > r_out || dy <= 0.0 { continue; }
        // legs fade out as they come down to the horizon
        let leg = (dy / (r_out * 0.6)).clamp(0.0, 1.0);
        for x in 0..map_w as i32 {
            let dx = x as f32 - cx;
            let d = (dx * dx + dy * dy).sqrt();
            if d < r_in || d >= r_out { continue; }
            let depth = (r_out - d) / band_w;
            let band = (depth as usize).min(BANDS.len() - 1);
            // soft outer and inner edges
            let edge = (depth.min(BANDS.len() as f32 - depth) / 1.5).clamp(0.0, 1.0);
            let a = alpha * leg * edge;
            l.mark(x, y, BANDS[band], a * 0.35, a * 0.6);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rainbow_fades_in_and_out() {
        assert_eq!(rainbow_alpha(0), 0.0);
        assert_eq!(rainbow_alpha(100), 1.0);
        assert!(rainbow_alpha(200) > 0.0 && rainbow_alpha(200) < 1.0);
        assert_eq!(rainbow_alpha(RAINBOW_FRAMES), 0.0);
    }

    #[test]
    fn test_marks_lighter_on_terrain_than_background() {
        let (w, h) = (100usize, 100usize);
        let mut buf = Buffer::new(w, h);
        buf.rect(0, 0, w as i32, h as i32, 25, 25, 38);
        buf.rect(0, 50, w as i32, 50, 60, 140, 60); // bottom half = terrain
        let mask = background_mask(&buf, w, (25, 25, 38));
        draw_precip(&mut buf, w, &mask, PrecipKind::HeavyRain, 5);
        let changed = |y0: usize, y1: usize| -> f32 {
            let mut d = 0u32;
            for y in y0..y1 { for x in 0..w {
                let i = (y * w + x) * 4;
                let base = if y < 50 { (25u8, 25u8, 38u8) } else { (60, 140, 60) };
                d += (buf.pixels[i] as i32 - base.0 as i32).unsigned_abs()
                    + (buf.pixels[i + 1] as i32 - base.1 as i32).unsigned_abs()
                    + (buf.pixels[i + 2] as i32 - base.2 as i32).unsigned_abs();
            }}
            d as f32
        };
        assert!(changed(0, 50) > changed(50, 100), "background should carry heavier marks");
    }
}
