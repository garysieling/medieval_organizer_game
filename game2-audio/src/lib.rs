//! Deterministic audio for the game: a seeded mixer that renders
//! interleaved stereo f32 into a caller buffer, identically live and
//! offline. Everything that makes a random choice draws it from
//! `rng::Rng`, seeded by the caller - no OS randomness, no clocks.
pub mod engine;
pub mod host;
pub mod rng;
pub mod settings;
pub mod synth;
pub mod wav;

#[cfg(feature = "analysis")]
pub mod analysis;
/// The scenario library names the sim by design - a test rig, not the
/// audio path (the layering test allowlists this one file)
#[cfg(feature = "scenarios")]
pub mod scenarios;
