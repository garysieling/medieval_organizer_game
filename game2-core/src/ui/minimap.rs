//! Minimap with fog-of-war memory fade.
//! From backlog: "if you haven't seen an area for a long time i think it might fade"
//! Explored areas show terrain; unexplored are black; old memories fade.

use crate::engine::render::Buffer;

/// Fog-of-war state per tile
#[derive(Debug)]
pub struct FogOfWar {
    pub width: usize,
    pub height: usize,
    /// 0=unseen, higher=recently seen (decays over time)
    pub memory: Vec<Vec<f32>>,
}

impl FogOfWar {
    pub fn new(w: usize, h: usize) -> Self {
        Self { width: w, height: h, memory: vec![vec![0.0; w]; h] }
    }

    /// Player sees an area - sets memory to fresh
    pub fn see(&mut self, x: usize, y: usize, radius: usize) {
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    let d = ((dx*dx + dy*dy) as f32).sqrt();
                    let v = (1.0 - d / (radius as f32 + 1.0)).max(0.0);
                    let m = &mut self.memory[ny as usize][nx as usize];
                    *m = m.max(v);
                }
            }
        }
    }

    /// Decay memory each tick (the fade)
    pub fn decay(&mut self, rate: f32) {
        for row in &mut self.memory {
            for m in row {
                *m = (*m - rate).max(0.0);
            }
        }
    }

    /// Render minimap: unseen=black, old memory=faded, fresh=bright
    pub fn render(&self, terrain_heights: &[Vec<f32>]) -> Buffer {
        let mut buf = Buffer::new(self.width * 4, self.height * 4);
        for y in 0..self.height {
            for x in 0..self.width {
                let mem = self.memory[y][x];
                let h = terrain_heights.get(y).and_then(|r| r.get(x)).copied().unwrap_or(0.5);
                // Base color from height
                let (r, g, b) = if h < 0.3 { (20, 40, 120) }
                    else if h < 0.6 { (40, 100, 40) }
                    else { (140, 140, 140) };
                // Apply memory fade: unseen=black, fading=darker
                let fade = mem; // 0=black, 1=bright
                let (fr, fg, fb) = (
                    (r as f32 * fade) as u8,
                    (g as f32 * fade) as u8,
                    (b as f32 * fade) as u8,
                );
                for dy in 0..4 { for dx in 0..4 {
                    buf.set((x*4+dx) as i32, (y*4+dy) as i32, fr, fg, fb);
                }}
            }
        }
        buf
    }
}
