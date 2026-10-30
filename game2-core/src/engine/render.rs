//! Headless software renderer - renders to an offscreen pixel buffer.
//! The app "thinks it's rendering"; we save the buffer to BMP/PNG files.
//! Evidence: actual rendered output from the software, not desktop captures.

/// RGBA pixel buffer
pub struct Buffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA, row-major
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![0; width * height * 4] }
    }

    /// Set pixel (clipped)
    pub fn set(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
        let i = (y as usize * self.width + x as usize) * 4;
        self.pixels[i] = r; self.pixels[i+1] = g; self.pixels[i+2] = b; self.pixels[i+3] = 255;
    }

    /// Fill rectangle
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
        for dy in 0..h { for dx in 0..w { self.set(x+dx, y+dy, r, g, b); } }
    }

    /// Draw diamond (isometric tile)
    pub fn diamond(&mut self, cx: i32, cy: i32, hw: i32, hh: i32, r: u8, g: u8, b: u8) {
        for dy in -hh..=hh {
            let span = hw * (hh - dy.abs()) / hh;
            for dx in -span..=span {
                self.set(cx + dx, cy + dy, r, g, b);
            }
        }
    }

    /// Load a BMP written by save_bmp (uncompressed 24-bit, bottom-up)
    pub fn load_bmp(path: &str) -> std::io::Result<Buffer> {
        let data = std::fs::read(path)?;
        let bad = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
        if data.len() < 54 || &data[0..2] != b"BM" { return Err(bad("not a BMP")); }
        let u32_at = |i: usize| u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        let (offset, w, h) = (u32_at(10) as usize, u32_at(18) as usize, u32_at(22) as usize);
        if u16::from_le_bytes([data[28], data[29]]) != 24 { return Err(bad("only 24-bit BMPs are supported")); }
        let row = (w * 3 + 3) / 4 * 4;
        if data.len() < offset + row * h { return Err(bad("truncated BMP")); }
        let mut buf = Buffer::new(w, h);
        for y in 0..h {
            let src = offset + (h - 1 - y) * row; // rows are stored bottom-up
            for x in 0..w {
                let (b, g, r) = (data[src + x * 3], data[src + x * 3 + 1], data[src + x * 3 + 2]);
                buf.set(x as i32, y as i32, r, g, b);
            }
        }
        Ok(buf)
    }

    /// Save as BMP (uncompressed 24-bit, no deps)
    pub fn save_bmp(&self, path: &str) -> std::io::Result<()> {
        let w = self.width as u32;
        let h = self.height as u32;
        let row_pad = (4 - (w * 3) % 4) % 4;
        let img_size = (w * 3 + row_pad) * h;
        let mut out = Vec::with_capacity(54 + img_size as usize);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(54 + img_size).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&img_size.to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes()); // x ppm
        out.extend_from_slice(&2835u32.to_le_bytes()); // y ppm
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        // BMP is bottom-up BGR
        for y in (0..h as usize).rev() {
            for x in 0..w as usize {
                let i = (y * self.width + x) * 4;
                out.push(self.pixels[i+2]); // B
                out.push(self.pixels[i+1]); // G
                out.push(self.pixels[i]);   // R
            }
            for _ in 0..row_pad { out.push(0); }
        }
        std::fs::write(path, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmp_round_trips() {
        // odd width exercises the 4-byte row padding
        let mut buf = Buffer::new(5, 3);
        buf.set(0, 0, 255, 0, 0);
        buf.set(4, 2, 10, 20, 30);
        let path = std::env::temp_dir().join(format!("roundtrip-{}.bmp", std::process::id()));
        buf.save_bmp(path.to_str().unwrap()).unwrap();
        let back = Buffer::load_bmp(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!((back.width, back.height), (5, 3));
        let px = |b: &Buffer, x: usize, y: usize| { let i = (y * b.width + x) * 4; (b.pixels[i], b.pixels[i + 1], b.pixels[i + 2]) };
        assert_eq!(px(&back, 0, 0), (255, 0, 0));
        assert_eq!(px(&back, 4, 2), (10, 20, 30));
        assert_eq!(px(&back, 2, 1), px(&buf, 2, 1));
    }
}
