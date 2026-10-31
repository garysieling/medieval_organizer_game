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
        // clip once, then fill whole rows at a time (this used to call set()
        // per pixel - fine for small boxes, but the full-screen clears and
        // panel fills made it the hottest line in the frame)
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = ((x + w).min(self.width as i32), (y + h).min(self.height as i32));
        if x0 >= x1 || y0 >= y1 { return; }
        let px = [r, g, b, 255];
        for dy in y0..y1 {
            let row = &mut self.pixels[(dy as usize * self.width + x0 as usize) * 4..(dy as usize * self.width + x1 as usize) * 4];
            for p in row.chunks_exact_mut(4) { p.copy_from_slice(&px); }
        }
    }

    /// Copy a w x h region of `src` (its top-left at src_x, src_y) onto this
    /// buffer at (dst_x, dst_y), clipped to both buffers
    pub fn blit_from(&mut self, src: &Buffer, src_x: i32, src_y: i32, dst_x: i32, dst_y: i32, w: i32, h: i32) {
        // clip to the destination: shift the source window to match
        let (mut sx, mut sy, mut dx, mut dy, mut w, mut h) = (src_x, src_y, dst_x, dst_y, w, h);
        if dx < 0 { sx -= dx; w += dx; dx = 0; }
        if dy < 0 { sy -= dy; h += dy; dy = 0; }
        // clip to the source: shift the destination to match
        if sx < 0 { dx -= sx; w += sx; sx = 0; }
        if sy < 0 { dy -= sy; h += sy; sy = 0; }
        w = w.min(self.width as i32 - dx).min(src.width as i32 - sx);
        h = h.min(self.height as i32 - dy).min(src.height as i32 - sy);
        if w <= 0 || h <= 0 { return; }
        let bytes = w as usize * 4;
        for row in 0..h as usize {
            let s = ((sy as usize + row) * src.width + sx as usize) * 4;
            let d = ((dy as usize + row) * self.width + dx as usize) * 4;
            self.pixels[d..d + bytes].copy_from_slice(&src.pixels[s..s + bytes]);
        }
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

    /// Nearest-neighbour resample to `scale` of the size - a video can
    /// trade pixels for a much smaller file
    pub fn scaled(&self, scale: f32) -> Buffer {
        let w = ((self.width as f32 * scale).round() as usize).max(1);
        let h = ((self.height as f32 * scale).round() as usize).max(1);
        let mut out = Buffer::new(w, h);
        let xmap: Vec<usize> = (0..w).map(|x| ((x as f32 / scale) as usize).min(self.width - 1)).collect();
        for y in 0..h {
            let sy = ((y as f32 / scale) as usize).min(self.height - 1);
            for (x, &sx) in xmap.iter().enumerate() {
                let s = (sy * self.width + sx) * 4;
                let d = (y * w + x) * 4;
                out.pixels[d..d + 4].copy_from_slice(&self.pixels[s..s + 4]);
            }
        }
        out
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
