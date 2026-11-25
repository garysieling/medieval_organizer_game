//! Text metrics - how wide a baked-glyph string draws, without a
//! buffer. Pure data (the generated `glyphs*` advance tables), so the
//! session may measure for its layout models: drawing stays in
//! `ui::font`, which reads the same tables.
use crate::engine::glyphs::ADVANCES;
use crate::engine::glyphs_book::BOOK_ADVANCES;

/// Which hand a string draws in - the book face (body text, the
/// default) or the blackletter rubric (titles, heads, quotes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face { Rubric, Book }

fn advances(face: Face) -> &'static [u8; 95] {
    match face {
        Face::Rubric => &ADVANCES,
        Face::Book => &BOOK_ADVANCES,
    }
}

/// The glyph's own advance: its ink plus a pixel of air. Unsupported
/// codes take the space advance.
pub(crate) fn advance_for(c: u8, face: Face) -> i32 {
    let advs = advances(face);
    if c < 32 || c > 126 { return advs[0] as i32; }
    advs[(c - 32) as usize] as i32
}

/// The string's true drawn width in pixels - what `draw_text` (the book
/// face) would return, without a buffer. Centering and bubble sizing
/// should ask this, not count chars times a fixed width.
pub fn measure(text: &str) -> i32 {
    text.chars().map(|c| advance_for(c as u8, Face::Book)).sum()
}

/// The string's width in the rubric hand - for centering titles and
/// heads, which draw in blackletter
pub fn measure_rubric(text: &str) -> i32 {
    text.chars().map(|c| advance_for(c as u8, Face::Rubric)).sum()
}
