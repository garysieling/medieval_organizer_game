//! Tech tree constellation: visualize tech progression as a node graph.
//! Printmaking track shown as connected "stars" per Gary's constellation idea.

use crate::engine::render::Buffer;
use crate::game::tech_tree::Tech;

/// Render tech tree as a constellation of nodes
/// Nodes = techs (lit=discovered, dim=locked), lines = prereq links
pub fn render_tech_tree(techs: &[Tech], positions: &[(String, f32, f32)], w: usize, h: usize) -> Buffer {
    let mut buf = Buffer::new(w, h);
    buf.rect(0, 0, w as i32, h as i32, 10, 10, 30); // night sky bg

    // Draw prereq links first
    for t in techs {
        let (x, y) = positions.iter().find(|(n, _, _)| n == &t.name)
            .map(|(_, x, y)| (*x, *y)).unwrap_or((0.0, 0.0));
        for prereq in &t.prereqs {
            if let Some((_, px, py)) = positions.iter().find(|(n, _, _)| n == prereq) {
                draw_edge(&mut buf, *px as i32, *py as i32, x as i32, y as i32, 80, 80, 120);
            }
        }
    }

    // Draw nodes
    for t in techs {
        let (x, y) = positions.iter().find(|(n, _, _)| n == &t.name)
            .map(|(_, x, y)| (*x, *y)).unwrap_or((0.0, 0.0));
        let (r, g, b) = if t.discovered { (255, 220, 80) } else { (80, 80, 100) };
        let radius = 6;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    buf.set((x + dx as f32) as i32, (y + dy as f32) as i32, r, g, b);
                }
            }
        }
        // Glow for discovered
        if t.discovered {
            for dy in -radius*2..=radius*2 {
                for dx in -radius*2..=radius*2 {
                    let d2 = dx*dx + dy*dy;
                    if d2 <= radius*radius*4 && d2 > radius*radius {
                        buf.set((x + dx as f32) as i32, (y + dy as f32) as i32, r/4, g/4, b/4);
                    }
                }
            }
        }
    }
    buf
}

fn draw_edge(buf: &mut Buffer, x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        buf.set(x0 + (dx as f32 * t) as i32, y0 + (dy as f32 * t) as i32, r, g, b);
    }
}
