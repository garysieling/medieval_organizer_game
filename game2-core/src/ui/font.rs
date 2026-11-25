//! Bitmap font renderer - draws text into pixel buffers.
//! Two hands, baked to the same 14x13 cells by scripts/bake_font.py and
//! scripts/bake_book_font.py (tables in engine/glyphs.rs /
//! engine/glyphs_book.rs), so a face swap never moves a layout - plus
//! the book's 22x20 tier in engine/glyphs_book_big.rs (--big, #1602).
//! No runtime font dep.
//!
//! - `Face::Rubric` - Jacquarda Bastarda 9 blackletter, the manuscript's
//!   rubric hand: titles, section heads, quotes, the explanatory words a
//!   reader doesn't lean on.
//! - `Face::Book` - IM Fell English, the book hand: names, lists, the log,
//!   everything read often. `draw_text` and `measure` are this face; ask
//!   for the rubric by name.
//! - `Face::BookBig` - the same Fell letter a tier up (#1602): 22x20
//!   cells, u32 rows. The conversation box sets its lines in it.
//!
//! Each glyph carries its own advance - all the faces read tracked the
//! way they were drawn rather than monospaced.

use crate::engine::render::Buffer;
use crate::engine::glyphs::GLYPHS;
use crate::engine::glyphs_book::BOOK_GLYPHS;
use crate::engine::glyphs_book_big::BOOK_BIG_GLYPHS;
// the metrics live in `engine::text` (pure data - the session's layout
// reads them too); font::measure stays the name callers know
pub use crate::engine::text::{measure, measure_face, measure_rubric, Face};
pub(crate) use crate::engine::text::advance_for;

/// Cell columns / rows, matching layout's GLYPH_H (the width's per
/// glyph now - `measure` gives a string's true pixels)
const COLS: usize = 14;
/// The book's larger tier sets wider cells (book_big 22x20, u32 rows)
const BIG_COLS: usize = 22;

fn glyphs(face: Face) -> &'static [[u16; 13]; 95] {
    match face {
        Face::Rubric => &GLYPHS,
        Face::Book => &BOOK_GLYPHS,
        Face::BookBig => unreachable!("BookBig's table is u32 - draw_char branches first"),
    }
}

/// Set every ink pixel of a cell - bit `cols-1` of each row = the
/// leftmost column. Shared by the 14x13 faces and the 22x20 tier.
fn draw_glyph<T: Into<u64> + Copy>(buf: &mut Buffer, x: i32, y: i32, glyph: &[T], cols: usize, r: u8, g: u8, b: u8) {
    for (row, bits) in glyph.iter().enumerate() {
        let bits: u64 = (*bits).into();
        for col in 0..cols {
            if bits & (1 << (cols - 1 - col)) != 0 {
                let px = x + col as i32;
                let py = y + row as i32;
                if px >= 0 && py >= 0 && (px as usize) < buf.width && (py as usize) < buf.height {
                    buf.set(px, py, r, g, b);
                }
            }
        }
    }
}

/// Draw a single char at (x,y) in `face`. Returns advance width.
pub fn draw_char(buf: &mut Buffer, x: i32, y: i32, c: char, face: Face, r: u8, g: u8, b: u8) -> i32 {
    let code = c as u8;
    if face == Face::BookBig {
        let glyph = if !(32..=126).contains(&code) { [0u32; 20] } else { BOOK_BIG_GLYPHS[(code - 32) as usize] };
        draw_glyph(buf, x, y, &glyph, BIG_COLS, r, g, b);
        return advance_for(code, face);
    }
    let glyphs = glyphs(face);
    let glyph = if code < 32 || code > 126 { [0u16; 13] } else { glyphs[(code - 32) as usize] };
    draw_glyph(buf, x, y, &glyph, COLS, r, g, b);
    advance_for(code, face)
}

/// Draw a string at (x,y) in the book hand. Returns total width drawn.
pub fn draw_text(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    draw_text_face(buf, x, y, text, Face::Book, r, g, b)
}

/// Draw a string at (x,y) in the blackletter rubric. Returns width drawn.
pub fn draw_rubric(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    draw_text_face(buf, x, y, text, Face::Rubric, r, g, b)
}

/// Draw a string at (x,y) in `face`. Returns total width drawn.
pub fn draw_text_face(buf: &mut Buffer, x: i32, y: i32, text: &str, face: Face, r: u8, g: u8, b: u8) -> i32 {
    let mut cx = x;
    for c in text.chars() {
        cx += draw_char(buf, cx, y, c, face, r, g, b);
    }
    cx - x
}

/// Draw a string with a 1px dark outline in the book hand, so it stays
/// readable over busy art (map labels over roofs, fields and sprites).
/// Returns width drawn.
pub fn draw_text_outlined(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    draw_text_outlined_face(buf, x, y, text, Face::Book, r, g, b)
}

/// Outlined text in `face` - the rubric takes the same dark ring
pub fn draw_text_outlined_face(buf: &mut Buffer, x: i32, y: i32, text: &str, face: Face, r: u8, g: u8, b: u8) -> i32 {
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        draw_text_face(buf, x + dx, y + dy, text, face, 12, 12, 18);
    }
    draw_text_face(buf, x, y, text, face, r, g, b)
}

/// Draw text with a scale factor in the rubric hand - titles and heads
/// are the blackletter's home, and scale-1 callers get it too (2 =
/// double size)
pub fn draw_text_scaled(buf: &mut Buffer, x: i32, y: i32, text: &str, scale: usize, r: u8, g: u8, b: u8) -> i32 {
    if scale <= 1 { return draw_rubric(buf, x, y, text, r, g, b); }
    let mut cx = x;
    for c in text.chars() {
        let glyphs = glyphs(Face::Rubric);
        let glyph = if (c as u8) < 32 || (c as u8) > 126 { [0u16; 13] } else { glyphs[(c as u8 - 32) as usize] };
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
        cx += advance_for(c as u8, Face::Rubric) * scale as i32;
    }
    cx - x
}

/// Draw text tracked out in the rubric: a pixel of air between glyphs,
/// the way a rubric reads in a manuscript (#1321). Returns width drawn.
pub fn draw_text_tracked(buf: &mut Buffer, x: i32, y: i32, text: &str, r: u8, g: u8, b: u8) -> i32 {
    let mut cx = x;
    for c in text.chars() {
        cx += draw_char(buf, cx, y, c, Face::Rubric, r, g, b) + 1;
    }
    cx - x
}

/// The conversation box's text tiers (#1321): roles, not pixels, so a
/// face swap slots underneath - H1 the panel title, H2 the day rules,
/// H3 a speaker's name lead. All three draw in the rubric hand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTier { H1, H2, H3 }

/// Draw text at a tier's weight: the title doubled, a rubric tracked
/// and capital, a name capital. Returns width drawn.
pub fn draw_tier(buf: &mut Buffer, x: i32, y: i32, text: &str, tier: TextTier, r: u8, g: u8, b: u8) -> i32 {
    match tier {
        TextTier::H1 => draw_text_scaled(buf, x, y, text, 2, r, g, b),
        TextTier::H2 => draw_text_tracked(buf, x, y, &text.to_uppercase(), r, g, b),
        TextTier::H3 => draw_rubric(buf, x, y, &text.to_uppercase(), r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink_of(c: u8, face: Face) -> u32 {
        let glyphs = glyphs(face);
        if c < 32 || c > 126 { return 0; }
        glyphs[(c - 32) as usize].iter().map(|r| r.count_ones()).sum()
    }

    #[test]
    fn every_ascii_glyph_draws() {
        // the whole printable band must put ink down in BOTH hands - a
        // hole in a baked table would silently drop a character's shape
        // (#1322)
        for face in [Face::Rubric, Face::Book] {
            for c in 33u8..=126 {
                assert!(ink_of(c, face) > 0,
                    "'{}' (ascii {}) draws nothing in {:?}", c as char, c, face);
            }
        }
        let book = glyphs(Face::Book);
        assert_eq!(book[0], [0; 13], "space stays empty");
        let rubric = glyphs(Face::Rubric);
        assert_eq!(rubric[0], [0; 13], "space stays empty");
    }

    #[test]
    fn measure_matches_the_drawn_width() {
        let mut buf = Buffer::new(200, 20);
        for s in ["ill", "www", "the taxes", "MW", "a"] {
            assert_eq!(measure(s), draw_text(&mut buf, 0, 0, s, 255, 255, 255),
                "measure should agree with the book pen for {s:?}");
            assert_eq!(measure_rubric(s), draw_rubric(&mut buf, 0, 0, s, 255, 255, 255),
                "measure_rubric should agree with the rubric pen for {s:?}");
        }
    }

    #[test]
    fn thin_letters_track_tighter_than_wide() {
        // the whole point of per-glyph advance: "i" shouldn't take an
        // m's room, in either hand
        assert!(measure("i") < measure("m"), "thin letters advance less");
        assert!(measure_rubric("i") < measure_rubric("m"), "thin letters advance less");
        assert!(measure("illicit") < "illicit".len() as i32 * 8,
            "the book face reads tight, not monospaced");
    }

    #[test]
    fn the_book_hand_reads_plainer_than_the_rubric() {
        // the readability fix's check: a lowercase word's book letters
        // carry conventional Latin shapes - its 'o' is an open ring of
        // ink, where the blackletter's is a closed loom. The book 'o'
        // must be wider than it is ornate: more columns of ink than the
        // rubric's (which packs its bowl into fewer, denser columns)
        let book_w = advance_for(b'o', Face::Book);
        assert!(book_w >= 6 && book_w <= 10,
            "the book 'o' should be a normal-width letter, got {book_w}");
        assert!(measure("organize the harvest") < measure_rubric("organize the harvest"),
            "the book hand runs tighter than the blackletter");
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
        // H1 doubles the rubric's advance; H2 tracks a pixel of air a
        // glyph; H3 capitalizes at plain rubric width
        let mut buf = Buffer::new(80, 30);
        let w1 = draw_tier(&mut buf, 0, 0, "AB", TextTier::H1, 255, 255, 255);
        let w3 = draw_tier(&mut buf, 0, 0, "ab", TextTier::H3, 255, 255, 255);
        assert_eq!(w1, w3 * 2, "H1 doubles the advance");
        let mut buf = Buffer::new(80, 30);
        let plain = draw_rubric(&mut buf, 0, 0, "AB", 255, 255, 255);
        let tracked = draw_tier(&mut buf, 40, 0, "ab", TextTier::H2, 255, 255, 255);
        assert_eq!(tracked, plain + 2, "a tracked row takes a px a glyph");
    }
}
