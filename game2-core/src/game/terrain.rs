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
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) as f32) / (u32::MAX as f32)
    }
    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

impl Terrain {
    /// Generate terrain using midpoint displacement (diamond-square)
    pub fn generate(size: usize, roughness: f32, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        // size must be 2^n + 1 for diamond-square; round up to nearest valid
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

        Self { width: size, height: size, heights, seed }
    }

    /// Get height at coordinate
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.heights[y][x]
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
}
