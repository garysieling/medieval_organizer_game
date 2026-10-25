//! Character sheet: traits, skills, health, relationships.
//! From backlog: "discuss character sheets", realistic relationship modeling.

use crate::engine::render::Buffer;

/// A character for the sheet
#[derive(Debug)]
pub struct Character {
    pub name: String,
    pub age: f32,
    pub health: f32,
    pub skills: Vec<(String, f32)>,
    pub traits: Vec<String>,
    pub relationships: Vec<(String, f32)>, // name -> strength
}

/// Render character sheet to buffer (panel layout, no font)
/// Left: portrait box. Right: bars for skills. Bottom: relationship chips.
pub fn render_sheet(c: &Character, w: usize, h: usize) -> Buffer {
    let mut buf = Buffer::new(w, h);
    buf.rect(0, 0, w as i32, h as i32, 30, 30, 40); // bg

    // Portrait area (top-left)
    buf.rect(10, 10, 80, 100, 60, 60, 80);
    buf.rect(15, 15, 70, 90, 100, 100, 120); // inner frame

    // Health bar
    let hb = (c.health * 200.0) as i32;
    buf.rect(100, 20, 200, 12, 60, 30, 30);
    buf.rect(100, 20, hb, 12, 60, 200, 60);

    // Skills bars (right side)
    for (i, (skill, level)) in c.skills.iter().enumerate() {
        let y = 50 + (i as i32 * 20);
        let lvl = (level * 150.0) as i32;
        buf.rect(100, y, 150, 10, 40, 40, 60);      // bar bg
        buf.rect(100, y, lvl, 10, 80, 160, 220);   // filled
    }

    // Relationship chips (bottom)
    for (i, (_name, strength)) in c.relationships.iter().enumerate() {
        let x = 10 + (i as i32 * 70);
        let col = if *strength > 0.0 { (60, 160, 60) } else { (160, 60, 60) };
        buf.rect(x, h as i32 - 40, 60, 30, col.0, col.1, col.2);
    }
    buf
}
