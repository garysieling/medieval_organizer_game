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
        Self::decode_with(decoder)
    }

    /// Decode a PNG already in memory (e.g. fetched over the wire)
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        Self::decode_with(png::Decoder::new(std::io::BufReader::new(std::io::Cursor::new(bytes))))
    }

    fn decode_with<R: std::io::BufRead + std::io::Seek>(decoder: png::Decoder<R>) -> Result<Self, String> {
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
        self.blit_resized(buf, x, y, self.width, self.height);
    }

    /// Draw this sprite resized to w x h (nearest-neighbour) with its
    /// top-left at (x, y), straight into the buffer: the same pixels as
    /// `scaled(w, h).blit(..)` without building the resized copy, and
    /// only the rows and columns that land on the buffer are visited
    pub fn blit_resized(&self, buf: &mut Buffer, x: i32, y: i32, w: usize, h: usize) {
        if w == 0 || h == 0 || self.width == 0 || self.height == 0 { return; }
        let (x0, x1) = (x.max(0), (x + w as i32).min(buf.width as i32));
        let (y0, y1) = (y.max(0), (y + h as i32).min(buf.height as i32));
        if x0 >= x1 || y0 >= y1 { return; }
        let same = w == self.width && h == self.height;
        // source column for each destination column, worked out once
        let cols: Vec<usize> = if same { vec![] } else { (x0..x1).map(|dx| (dx - x) as usize * self.width / w).collect() };
        for dy in y0..y1 {
            let sy = if same { (dy - y) as usize } else { (dy - y) as usize * self.height / h };
            let srow = &self.pixels[sy * self.width * 4..(sy + 1) * self.width * 4];
            let drow = &mut buf.pixels[dy as usize * buf.width * 4..(dy as usize + 1) * buf.width * 4];
            for (i, dx) in (x0..x1).enumerate() {
                let sx = if same { (dx - x) as usize } else { cols[i] };
                let src = &srow[sx * 4..sx * 4 + 4];
                if src[3] > 128 { // skip transparent
                    let di = dx as usize * 4;
                    drow[di..di + 3].copy_from_slice(&src[..3]);
                    drow[di + 3] = 255;
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

    /// Blit scaled (nearest-neighbor)
    pub fn blit_scaled(&self, buf: &mut Buffer, x: i32, y: i32, scale: usize) {
        self.blit_resized(buf, x, y, self.width * scale, self.height * scale);
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

    /// Resize to w x h (nearest-neighbor), e.g. to fit a sprite drawn for a
    /// bigger tile grid onto the map's tiles
    pub fn scaled(&self, w: usize, h: usize) -> Sprite {
        let mut pixels = vec![0u8; w * h * 4];
        if self.width == 0 || self.height == 0 { return Sprite { width: w, height: h, pixels }; }
        for dy in 0..h { for dx in 0..w {
            let sx = dx * self.width / w;
            let sy = dy * self.height / h;
            let src = (sy * self.width + sx) * 4;
            let dst = (dy * w + dx) * 4;
            pixels[dst..dst+4].copy_from_slice(&self.pixels[src..src+4]);
        }}
        Sprite { width: w, height: h, pixels }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blit_resized_matches_scaling_then_blitting() {
        // blit_resized replaced scaled(w, h).blit(..) on every draw (it was
        // most of the frame); it must put down exactly the same pixels,
        // clipped at every edge
        let (sw, sh) = (7, 5);
        let pixels: Vec<u8> = (0..sw * sh).flat_map(|i| {
            let i = i as u8;
            [i.wrapping_mul(37), i.wrapping_mul(91), i.wrapping_mul(13), if i % 3 == 0 { 0 } else { 255 }]
        }).collect();
        let s = Sprite { width: sw, height: sh, pixels };
        for (w, h) in [(7, 5), (14, 10), (3, 2), (11, 9), (1, 1)] {
            for (x, y) in [(0, 0), (5, 4), (-4, -3), (16, 12), (-20, 0)] {
                let mut a = Buffer::new(20, 15);
                let mut b = Buffer::new(20, 15);
                // the old way: a resized copy, then a pixel-by-pixel set()
                let r = s.scaled(w, h);
                for sy in 0..r.height { for sx in 0..r.width {
                    let i = (sy * r.width + sx) * 4;
                    if r.pixels[i + 3] > 128 { a.set(x + sx as i32, y + sy as i32, r.pixels[i], r.pixels[i + 1], r.pixels[i + 2]); }
                }}
                s.blit_resized(&mut b, x, y, w, h);
                assert_eq!(a.pixels, b.pixels, "{}x{} at ({}, {})", w, h, x, y);
            }
        }
    }

    #[test]
    fn test_scaled_resizes_and_keeps_corners() {
        // 4x2 sprite: left half red, right half blue
        let mut pixels = vec![];
        for _ in 0..2 { for x in 0..4 {
            pixels.extend_from_slice(if x < 2 { &[255, 0, 0, 255] } else { &[0, 0, 255, 255] });
        }}
        let s = Sprite { width: 4, height: 2, pixels };
        let small = s.scaled(2, 1);
        assert_eq!((small.width, small.height), (2, 1));
        assert_eq!(&small.pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&small.pixels[4..8], &[0, 0, 255, 255]);
        let big = s.scaled(8, 4);
        assert_eq!(big.pixels.len(), 8 * 4 * 4);
        assert_eq!(&big.pixels[(3 * 8 + 7) * 4..(3 * 8 + 8) * 4], &[0, 0, 255, 255]);
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
            ("princess", _) => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPrincess.png".into(),
            ("queen", _) => "MinifolksVillagers/MinifolksVillagers/Without Outline/MiniQueen.png".into(),
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
