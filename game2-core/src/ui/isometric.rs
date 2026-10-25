//! Isometric 2.5D board renderer - draws terrain tiles as diamonds.
//! Renders to a headless Buffer; save_bmp gives real visual evidence.

use crate::engine::render::Buffer;
use crate::game::terrain::Terrain;

/// Render isometric terrain board to buffer
/// Water=blue, grass=green, hills=brown, peaks=white
pub fn render_terrain(terrain: &Terrain, width: usize, height: usize) -> Buffer {
    let mut buf = Buffer::new(width, height);
    buf.rect(0, 0, width as i32, height as i32, 20, 20, 30); // bg

    let cx = width as i32 / 2;
    let cy = 40;
    let tw = 8;  // tile half-width
    let th = 4;  // tile half-height

    for ty in 0..terrain.height.min(20) {
        for tx in 0..terrain.width.min(20) {
            let h = terrain.get(tx, ty);
            let (r, g, b) = if h < 0.15 { (30, 60, 180) }     // water
                else if h < 0.35 { (60, 140, 60) }           // grass
                else if h < 0.55 { (140, 110, 50) }          // hills
                else if h < 0.75 { (110, 90, 60) }           // highlands
                else { (220, 220, 220) };                    // peaks

            let sx = cx + (tx as i32 - ty as i32) * tw;
            let sy = cy + (tx as i32 + ty as i32) * th;
            buf.diamond(sx, sy, tw, th, r, g, b);
            // darker edge for depth
            buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
        }
    }
    buf
}

/// Render a simple entity (villager dot) on the board
pub fn render_entity(buf: &mut Buffer, world_x: f32, world_y: f32, r: u8, g: u8, b: u8) {
    let cx = buf.width as i32 / 2;
    let cy = 40;
    let sx = cx + (world_x as i32 - world_y as i32) * 8;
    let sy = cy + (world_x as i32 + world_y as i32) * 4;
    buf.diamond(sx, sy - 6, 3, 4, r, g, b); // villager = small diamond above tile
}
