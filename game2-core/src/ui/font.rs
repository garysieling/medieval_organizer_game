//! Bitmap font renderer - draws text into pixel buffers.
//! Jacquarda Bastarda 9 blackletter, baked to 14x13 cells by
//! scripts/bake_font.py (table in glyphs.rs). No runtime font dep.
//!
//! Each glyph carries its own advance - the blackletter reads tracked
//! tight, the way the face was drawn, rather than monospaced.

use crate::engine::render::Buffer;
use crate::ui::glyphs::{ADVANCES, GLYPHS};

/// Cell columns / rows, matching layout's GLYPH_H (the width's per
/// glyph now - `measure` gives a string's true pixels)
const COLS: usize = 14;

/// Fix glyph table - entry 106 was a slice hack, normalize
fn glyph_for(c: u8) -> [u16; 13] {
    if c < 32 || c > 126 { return [0;13]; }
    GLYPHS[(c - 32) as usize]
}

/// The glyph's own advance: its ink plus a pixel of air. Unsupported
/// codes take the space advance.
fn advance_for(c: u8) -> i32 {
    if c < 32 || c > 126 { return ADVANCES[0] as i32; }
    ADVANCES[(c - 32) as usize] as i32
}

/// The string's true drawn width in pixels - what `draw_text` would
/// return, without a buffer. Centering and bubble sizing should ask
/// this, not count chars times a fixed width.
pub fn measure(text: &str) -> i32 {
    text.chars().map(|c| advance_for(c as u8)).sum()
}

/// Draw a single char at (x,y). Returns advance width.
pub fn draw_char(buf: &mut Buffer, x: i32, y: i32, c: char, r: u8, g: u8, b: u8) -> i32 {
    let glyph = glyph_for(c as u8);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..COLS {
            if bits & (1 << (COLS - 1 - col)) != 0 {
                let px = x + col as i32;
                let py = y + row as i32;
                if px >= 0 && py >= 0 && (px as usize) < buf.width && (py as usize) < buf.height {
                    buf.set(px, py, r, g, b);
                }
            }
        }
    }
    advance_for(c as u8)
}

/// Draw a string at (x,y). Returns total width drawn.
pub fn draw_text(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    let mut cx = x;
    for c in text.chars() {
        cx += draw_char(buf, cx, y, c, r, g, b);
    }
    cx - x
}

/// Draw a string with a 1px dark outline, so it stays readable over busy
/// art (map labels over roofs, fields and sprites). Returns width drawn.
pub fn draw_text_outlined(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        draw_text(buf, x + dx, y + dy, text, 12, 12, 18);
    }
    draw_text(buf, x, y, text, r, g, b)
}

/// Draw text with a scale factor (2 = double size)
pub fn draw_text_scaled(buf: &mut Buffer, x: i32, y: i32, text: &str, scale: usize, r: u8, g: u8, b: u8) -> i32 {
    if scale <= 1 { return draw_text(buf, x, y, text, r, g, b); }
    let mut cx = x;
    for c in text.chars() {
        let glyph = glyph_for(c as u8);
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..COLS {
                if bits & (1 << (COLS - 1 - col)) != 0 {
                    for sy in 0..scale { for sx in 0..scale {
                        let px = cx + (col * scale + sx) as i32;
                        let py = y + (row * scale + sy) as i32;
                        if px >= 0 && py >= 0 && (px as usize) < buf.width && (py as usize) < buf.height {
                            buf.set(px, py, r, g, b);
                        }
                    }}
                }
            }
        }
        cx += advance_for(c as u8) * scale as i32;
    }
    cx - x
}

/// Draw text tracked out: a pixel of air between glyphs, the way a
/// rubric reads in a manuscript (#1321). Returns width drawn.
pub fn draw_text_tracked(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    let mut cx = x;
    for c in text.chars() {
        cx += draw_char(buf, cx, y, c, r, g, b) + 1;
    }
    cx - x
}

/// The conversation box's text tiers (#1321): roles, not pixels, so a
/// face swap slots underneath - H1 the panel title, H2 the day rules,
/// H3 a speaker's name lead
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTier { H1, H2, H3 }

/// Draw text at a tier's weight: the title doubled, a rubric tracked
/// and capital, a name capital. Returns width drawn.
pub fn draw_tier(buf: &mut Buffer, x: i32, y: i32, text: &str, tier: TextTier, r: u8, g: u8, b: u8) -> i32 {
    match tier {
        TextTier::H1 => draw_text_scaled(buf, x, y, text, 2, r, g, b),
        TextTier::H2 => draw_text_tracked(buf, x, y, &text.to_uppercase(), r, g, b),
        TextTier::H3 => draw_text(buf, x, y, &text.to_uppercase(), r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ascii_glyph_draws() {
        // the whole printable band must put ink down - a hole in the
        // baked table would silently drop a character's shape (#1322)
        for c in 33u8..=126 {
            let ink: u32 = glyph_for(c).iter().map(|r| r.count_ones()).sum();
            assert!(ink > 0, "'{}' (ascii {}) draws nothing", c as char, c);
        }
        assert_eq!(glyph_for(b' '), [0; 13], "space stays empty");
        assert_eq!(glyph_for(127), [0; 13], "unsupported codes draw nothing");
    }

    #[test]
    fn measure_matches_the_drawn_width() {
        let mut buf = Buffer::new(200, 20);
        for s in ["ill", "www", "the taxes", "MW", "a"] {
            assert_eq!(measure(s), draw_text(&mut buf, 0, 0, s, 255, 255, 255),
                "measure should agree with the pen for {s:?}");
        }
    }

    #[test]
    fn thin_letters_track_tighter_than_wide() {
        // the whole point of per-glyph advance: "i" shouldn't take an
        // m's room
        assert!(measure("i") < measure("m"), "thin letters advance less");
        assert!(measure("illicit") < "illicit".len() as i32 * 8,
            "the face reads tight, not monospaced");
    }

    #[test]
    fn test_outlined_text_has_a_dark_border_and_coloured_glyph() {
        let mut buf = Buffer::new(40, 24);
        buf.rect(0, 0, 40, 24, 250, 250, 250); // white background, like a roof
        draw_text_outlined(&mut buf, 5, 5, "I", 255, 220, 0);
        let px = |x: i32, y: i32| {
            let i = (y as usize * 40 + x as usize) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2])
        };
        // somewhere in the glyph is the text colour, and right next to the
        // glyph (where the white background was) is the dark outline
        let mut coloured = 0;
        let mut outlined = 0;
        for y in 3..15 { for x in 3..14 {
            match px(x, y) {
                (255, 220, 0) => coloured += 1,
                (12, 12, 18) => outlined += 1,
                _ => {}
            }
        }}
        assert!(coloured > 0, "glyph should be drawn in the text colour");
        assert!(outlined > coloured, "glyph should be ringed by a dark outline");
    }

    #[test]
    fn the_tiers_draw_at_their_weights() {
        // H1 doubles the advance; H2 tracks a pixel of air a glyph; H3
        // capitalizes at plain width
        let mut buf = Buffer::new(80, 30);
        let w1 = draw_tier(&mut buf, 0, 0, "AB", TextTier::H1, 255, 255, 255);
        let w3 = draw_tier(&mut buf, 0, 0, "ab", TextTier::H3, 255, 255, 255);
        assert_eq!(w1, w3 * 2, "H1 doubles the advance");
        let mut buf = Buffer::new(80, 30);
        let plain = draw_text(&mut buf, 0, 0, "AB", 255, 255, 255);
        let tracked = draw_tier(&mut buf, 40, 0, "ab", TextTier::H2, 255, 255, 255);
        assert_eq!(tracked, plain + 2, "a tracked row takes a px a glyph");
    }
}
