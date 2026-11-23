//! The crate's only source of randomness: xorshift64*, seeded by the
//! caller so one scene always renders the same audio.

/// xorshift64* - small, fast, and exactly reproducible on every target
/// (f32 math aside, this is the whole determinism story for choice).
pub struct Rng(u64);

impl Rng {
    /// Seed 0 would lock the generator at zero forever, so it maps to a
    /// fixed nonzero constant.
    pub fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn next_u32(&mut self) -> u32 { (self.next_u64() >> 32) as u32 }
    /// In [0, 1).
    pub fn next_f32(&mut self) -> f32 { self.next_u32() as f32 / 4294967296.0 }
    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 { lo + self.next_f32() * (hi - lo) }
    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool { self.next_f32() < p }
}
