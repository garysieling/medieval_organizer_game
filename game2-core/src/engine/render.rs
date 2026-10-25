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
