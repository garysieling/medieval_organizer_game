//! Deterministic audio for the game: a seeded mixer that renders
//! interleaved stereo f32 into a caller buffer, identically live and
//! offline. Everything that makes a random choice draws it from
//! `rng::Rng`, seeded by the caller - no OS randomness, no clocks.
pub mod engine;
pub mod rng;
pub mod wav;
