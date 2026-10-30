//! Procedural terrain generation
//!
//! Fractal midpoint displacement (diamond-square) for realistic
//! heightmaps. Emergent property: complexity from simple rules.

use crate::engine::ecs::World;

/// Terrain heightmap
#[derive(Debug)]
pub struct Terrain {
    pub width: usize,
    pub height: usize,
    pub heights: Vec<Vec<f32>>,
    pub seed: u64,
}

/// Simple seeded RNG for reproducible terrain
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    /// Uniform in [0, 1): the top 31 bits of the LCG state, scaled by 2^31
    /// (scaling by u32::MAX instead capped every draw below 0.5, so every
    /// displacement in range(-s, s) pushed terrain downward)
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) as f32) / ((1u64 << 31) as f32)
    }
    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

impl Terrain {
    /// Fractal heights by midpoint displacement (diamond-square), spread
    /// over the full 0-1 range: the raw relief both shapes of map start from
    fn fractal(size: usize, roughness: f32, seed: u64) -> (usize, Vec<Vec<f32>>) {
        let mut rng = Rng::new(seed);
        // size must be 2^n + 1 for diamond-square; round up to nearest valid
        let want = size;
        let n = ((size - 1) as f32).log2().ceil() as u32;
        let size = (1usize << n) + 1;

        let mut heights = vec![vec![0.0f32; size]; size];

        // Initialize corners
        heights[0][0] = rng.range(0.0, 1.0);
        heights[0][size - 1] = rng.range(0.0, 1.0);
        heights[size - 1][0] = rng.range(0.0, 1.0);
        heights[size - 1][size - 1] = rng.range(0.0, 1.0);

        let mut step = size - 1;
        let mut scale = roughness;

        while step > 1 {
            let half = step / 2;

            // Diamond step: center of each square (interior points only)
            for y in (half..size - 1).step_by(step) {
                for x in (half..size - 1).step_by(step) {
                    let avg = (heights[y - half][x - half]
                        + heights[y - half][x + half]
                        + heights[y + half][x - half]
                        + heights[y + half][x + half]) / 4.0;
                    heights[y][x] = (avg + rng.range(-scale, scale)).max(0.0).min(1.0);
                }
            }

            // Square step: midpoints of diamond edges
            for y in 0..size {
                for x in ((y + half) % step..size).step_by(step) {
                    let mut sum = 0.0;
                    let mut count = 0;
                    if x >= half { sum += heights[y][x - half]; count += 1; }
                    if x + half < size { sum += heights[y][x + half]; count += 1; }
                    if y >= half { sum += heights[y - half][x]; count += 1; }
                    if y + half < size { sum += heights[y + half][x]; count += 1; }
                    heights[y][x] = ((sum / count as f32) + rng.range(-scale, scale)).max(0.0).min(1.0);
                }
            }

            step = half;
            scale *= 0.5;
        }

        // Normalize to full 0-1 range
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for row in &heights {
            for &h in row {
                min = min.min(h);
                max = max.max(h);
            }
        }
        let range = (max - min).max(0.0001);
        for row in &mut heights {
            for h in row {
                *h = (*h - min) / range;
            }
        }

        // diamond-square only makes 2^n + 1 grids: when the caller asked for
        // a size that isn't one, resample the (bigger) grid down to it
        if size != want && want > 1 {
            let mut out = vec![vec![0.0f32; want]; want];
            for y in 0..want {
                let sy = y as f32 * (size - 1) as f32 / (want - 1) as f32;
                let (y0, fy) = (sy as usize, sy.fract());
                for x in 0..want {
                    let sx = x as f32 * (size - 1) as f32 / (want - 1) as f32;
                    let (x0, fx) = (sx as usize, sx.fract());
                    let (x1, y1) = ((x0 + 1).min(size - 1), (y0 + 1).min(size - 1));
                    let top = heights[y0][x0] + (heights[y0][x1] - heights[y0][x0]) * fx;
                    let bot = heights[y1][x0] + (heights[y1][x1] - heights[y1][x0]) * fx;
                    out[y][x] = top + (bot - top) * fy;
                }
            }
            heights = out;
        }

        (want, heights)
    }

    /// Generate terrain using midpoint displacement (diamond-square), on a
    /// dome: an island, land in the middle and sea all round
    pub fn generate(size: usize, roughness: f32, seed: u64) -> Self {
        let (size, mut heights) = Self::fractal(size, roughness, seed);

        // Continent bias: strong land dome in center, water only at rim
        let center = (size / 2) as f32;
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32 - center) / center;
                let dy = (y as f32 - center) / center;
                let dist = (dx * dx + dy * dy).sqrt(); // 0 center -> ~1.4 corner
                // Islands: land fills center, rim is water; cap height so center isn't all peaks
                let dome = (1.0 - dist * dist).max(0.0);
                let target = dome * 0.55 + 0.12; // center ~0.67 (grass/hills), rim ~0.12 (water)
                heights[y][x] = (heights[y][x] * 0.35 + target).clamp(0.0, 1.0);
            }
        }

        // Histogram spread: mild stretch for variety without eating the land
        for row in &mut heights {
            for h in row {
                *h = (*h * 1.15 - 0.05).clamp(0.0, 1.0);
            }
        }

        Self { width: size, height: size, heights, seed }
    }

    /// A stretch of country rather than an island: hills and bare rock to
    /// the north (the top of the screen), falling away to a coast along the
    /// south, with the fractal relief giving it ridges, valleys and the odd
    /// lake
    pub fn generate_country(size: usize, roughness: f32, seed: u64) -> Self {
        let (size, mut heights) = Self::fractal(size, roughness, seed);
        let span = (2 * (size - 1)) as f32;
        for y in 0..size {
            for x in 0..size {
                let south = (x + y) as f32 / span; // 0 at the top corner, 1 at the bottom
                let slope = 0.95 - 1.05 * south;
                heights[y][x] = (heights[y][x] * 0.5 + slope * 0.75).clamp(0.0, 1.0);
            }
        }
        Self { width: size, height: size, heights, seed }
    }

    /// Get height at coordinate
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.heights[y][x]
    }

    /// Map a height value to its terrain tile color (water/grass/hills/highlands/peaks).
    /// Shared by every renderer (game_ui, playtest_gate) so they agree on what each
    /// height band looks like.
    pub fn tile_color(h: f32) -> (u8, u8, u8) {
        if h < 0.15 { (30, 60, 180) }
        else if h < 0.35 { (60, 140, 60) }
        else if h < 0.55 { (140, 110, 50) }
        else if h < 0.75 { (110, 90, 60) }
        else { (220, 220, 220) }
    }

    /// ASCII visualization for screenshots
    pub fn ascii(&self) -> String {
        let chars = " .:-=+*#%@";
        let mut s = String::new();
        for row in &self.heights {
            for &h in row {
                let idx = (h * (chars.len() - 1) as f32) as usize;
                s.push(chars.chars().nth(idx).unwrap());
            }
            s.push('\n');
        }
        s
    }
}

/// Spawn terrain as ECS component
pub fn spawn_terrain(world: &mut World, terrain: Terrain) {
    let e = world.create_entity();
    world.add_component(e, terrain);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let t = Terrain::generate(9, 0.5, 42);
        assert_eq!(t.width, t.height);
        assert!(t.heights.iter().all(|r| r.iter().all(|&h| h >= 0.0 && h <= 1.0)));
    }

    #[test]
    fn test_deterministic() {
        let t1 = Terrain::generate(9, 0.5, 42);
        let t2 = Terrain::generate(9, 0.5, 42);
        assert_eq!(t1.heights, t2.heights); // same seed = same terrain
    }

    #[test]
    fn test_different_seeds() {
        let t1 = Terrain::generate(9, 0.5, 1);
        let t2 = Terrain::generate(9, 0.5, 2);
        assert_ne!(t1.heights, t2.heights); // different seeds = different terrain
    }

    #[test]
    fn test_rng_covers_whole_unit_interval() {
        let mut rng = Rng::new(42);
        let draws: Vec<f32> = (0..10_000).map(|_| rng.next_f32()).collect();
        assert!(draws.iter().all(|&x| (0.0..1.0).contains(&x)));
        let upper = draws.iter().filter(|&&x| x >= 0.5).count();
        assert!(upper > 4_000, "about half the draws should land in [0.5, 1), got {}", upper);
        let mean: f32 = draws.iter().sum::<f32>() / draws.len() as f32;
        assert!((mean - 0.5).abs() < 0.05, "mean should be ~0.5, got {}", mean);
    }

    #[test]
    fn test_displacement_is_symmetric() {
        let mut rng = Rng::new(7);
        let ups = (0..10_000).filter(|_| rng.range(-1.0, 1.0) > 0.0).count();
        assert!(ups > 4_000, "noise should push terrain up as often as down, got {} ups", ups);
    }
}
