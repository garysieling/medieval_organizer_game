//! Sprite loading and blitting - PNG files -> pixel buffers -> framebuffer.
//! Supports transparency (alpha channel).

use crate::engine::render::Buffer;
use std::path::Path;

/// A loaded sprite (RGBA pixels)
#[derive(Debug, Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA
}

impl Sprite {
    /// Load a PNG file
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("open {}: {}", path.display(), e))?;
        let decoder = png::Decoder::new(std::io::BufReader::new(file));
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
        let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;

        // Normalize to RGBA8
        let mut pixels = vec![0u8; (info.width * info.height * 4) as usize];
        match info.color_type {
            png::ColorType::Rgba => {
                let n = pixels.len().min(buf.len());
                pixels[..n].copy_from_slice(&buf[..n]);
            }
            png::ColorType::Rgb => {
                for i in 0..(info.width * info.height) as usize {
                    pixels[i*4]   = buf[i*3];
                    pixels[i*4+1] = buf[i*3+1];
                    pixels[i*4+2] = buf[i*3+2];
                    pixels[i*4+3] = 255;
                }
            }
            png::ColorType::Indexed => {
                // palette should have been expanded by decoder
                for i in 0..(info.width * info.height) as usize {
                    if i*4+3 < buf.len() {
                        pixels[i*4..i*4+4].copy_from_slice(&buf[i*4..i*4+4]);
                    }
                }
            }
            _ => return Err(format!("unsupported color type {:?}", info.color_type)),
        }

        Ok(Sprite { width: info.width as usize, height: info.height as usize, pixels })
    }

    /// Blit this sprite onto a buffer at (x,y) - top-left anchor
    pub fn blit(&self, buf: &mut Buffer, x: i32, y: i32) {
        for sy in 0..self.height {
            for sx in 0..self.width {
                let idx = (sy * self.width + sx) * 4;
                let a = self.pixels[idx + 3];
                if a > 128 { // skip transparent
                    buf.set(x + sx as i32, y + sy as i32,
                        self.pixels[idx], self.pixels[idx+1], self.pixels[idx+2]);
                }
            }
        }
    }

    /// Blit with center anchor (for isometric placement on tiles)
    pub fn blit_center(&self, buf: &mut Buffer, cx: i32, cy: i32) {
        self.blit(buf, cx - self.width as i32 / 2, cy - self.height as i32);
    }

    /// Blit with bottom-center anchor (isometric buildings sit on tile base)
    pub fn blit_bottom(&self, buf: &mut Buffer, cx: i32, bottom_y: i32) {
        self.blit(buf, cx - self.width as i32 / 2, bottom_y - self.height as i32);
    }

    /// Blit at a fractional scale (nearest-neighbor), bottom-center anchor
    pub fn blit_scaled_bottom(&self, buf: &mut Buffer, cx: i32, bottom_y: i32, scale: f32) {
        let w = (self.width as f32 * scale).round() as i32;
        let h = (self.height as f32 * scale).round() as i32;
        let (x0, y0) = (cx - w / 2, bottom_y - h);
        for dy in 0..h {
            let sy = ((dy as f32 / scale) as usize).min(self.height - 1);
            for dx in 0..w {
                let sx = ((dx as f32 / scale) as usize).min(self.width - 1);
                let idx = (sy * self.width + sx) * 4;
                if self.pixels[idx + 3] > 128 {
                    buf.set(x0 + dx, y0 + dy, self.pixels[idx], self.pixels[idx+1], self.pixels[idx+2]);
                }
            }
        }
    }

    /// Blit scaled (nearest-neighbor)
    pub fn blit_scaled(&self, buf: &mut Buffer, x: i32, y: i32, scale: usize) {
        for sy in 0..self.height {
            for sx in 0..self.width {
                let idx = (sy * self.width + sx) * 4;
                if self.pixels[idx + 3] > 128 {
                    let (r, g, b) = (self.pixels[idx], self.pixels[idx+1], self.pixels[idx+2]);
                    for dy in 0..scale { for dx in 0..scale {
                        buf.set(x + (sx*scale+dx) as i32, y + (sy*scale+dy) as i32, r, g, b);
                    }}
                }
            }
        }
    }

    /// Extract a sub-sprite from a spritesheet (for animated frames)
    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Sprite {
        let mut pixels = vec![0u8; w * h * 4];
        for dy in 0..h { for dx in 0..w {
            let src = ((y + dy) * self.width + (x + dx)) * 4;
            let dst = (dy * w + dx) * 4;
            if src + 3 < self.pixels.len() {
                pixels[dst..dst+4].copy_from_slice(&self.pixels[src..src+4]);
            }
        }}
        Sprite { width: w, height: h, pixels }
    }
}

/// Asset registry: maps roles to sprite paths
pub struct Assets {
    pub root: String,
}

impl Assets {
    pub fn new(root: &str) -> Self { Self { root: root.into() } }

    pub fn path(&self, rel: &str) -> std::path::PathBuf {
        Path::new(&self.root).join(rel)
    }

    /// Villager sprites by role
    pub fn villager_sprite(&self, role: &str, gender: &str) -> String {
        match (role, gender) {
            ("peasant", "m") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniVillagerMan.png".into(),
            ("peasant", "f") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniVillagerWoman.png".into(),
            ("worker", _) => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniWorker.png".into(),
            ("elder", "m") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniOldMan.png".into(),
            ("elder", "f") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniOldWoman.png".into(),
            ("noble", "m") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniNobleMan.png".into(),
            ("noble", "f") => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniNobleWoman.png".into(),
            _ => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png".into(),
        }
    }

    /// Building sprites by type
    pub fn building_sprite(&self, kind: &str, color: &str) -> String {
        match kind {
            "house" => format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/White houses/{} house/white_{}_house(1).png", capitalize(color), color),
            "woodhouse" => format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/Wood houses/{} house/wood_{}_house(1).png", capitalize(color), color),
            "blacksmith" => format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/blacksmith_{}(1).png", color),
            "mill" => format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill_{}.png", color),
            _ => format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/Wood houses/Wood house/wood_house(1).png"),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
