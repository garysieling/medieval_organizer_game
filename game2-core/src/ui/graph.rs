//! Relationship graph: visualize social bonds between villagers.
//! Nodes = villagers (colored by attachment style), edges = bonds (thickness = strength).

use crate::engine::render::Buffer;

/// A node in the social graph
#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub style_color: (u8, u8, u8), // attachment style color
}

/// An edge (bond) between two people
#[derive(Debug)]
pub struct Bond {
    pub from: usize,
    pub to: usize,
    pub strength: f32, // -1 to 1
}

/// Render social graph to buffer
/// Nodes = circles, edges = lines (thicker = stronger, green=positive, red=negative)
pub fn render_graph(people: &[Person], bonds: &[Bond], w: usize, h: usize) -> Buffer {
    let mut buf = Buffer::new(w, h);
    buf.rect(0, 0, w as i32, h as i32, 15, 15, 25); // bg

    // Draw edges first (behind nodes)
    for bond in bonds {
        let a = &people[bond.from];
        let b = &people[bond.to];
        let (r, g, bl) = if bond.strength > 0.0 { (60, 180, 60) } else { (180, 60, 60) };
        draw_line(&mut buf, a.x as i32, a.y as i32, b.x as i32, b.y as i32, r, g, bl, (bond.strength.abs() * 3.0) as i32 + 1);
    }

    // Draw nodes
    for p in people {
        let r = 8;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx*dx + dy*dy <= r*r {
                    buf.set((p.x + dx as f32) as i32, (p.y + dy as f32) as i32,
                        p.style_color.0, p.style_color.1, p.style_color.2);
                }
            }
        }
    }
    buf
}

/// Draw a thick line between two points
fn draw_line(buf: &mut Buffer, x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8, thickness: i32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x0 + (dx as f32 * t) as i32;
        let y = y0 + (dy as f32 * t) as i32;
        for dt in -(thickness/2)..=(thickness/2) {
            buf.set(x + dt, y + dt, r, g, b);
        }
    }
}
