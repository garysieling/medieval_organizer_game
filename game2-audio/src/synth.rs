//! The eight legacy cue voices, ported from `record.rs::synth_track`'s
//! `voice()` (#1493): same partials and envelopes so the old recordings'
//! timbre carries over. `noise` is a -1..1 draw from the engine's rng,
//! the legacy `x * 2 - 1`.
use std::f32::consts::PI;

/// One of the sim's sfx cue voices.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SynthKind {
    Rain,
    Anvil,
    Festival,
    Wonder,
    Bell,
    Snowballs,
    Splash,
    /// "mood" and any cue without a voice of its own: a small blip.
    Mood,
}

impl SynthKind {
    /// A cue name's voice; an unknown name gets the blip, like the
    /// legacy match's catch-all arm.
    pub fn from_name(kind: &str) -> Self {
        match kind {
            "rain" => SynthKind::Rain,
            "anvil" => SynthKind::Anvil,
            "festival" => SynthKind::Festival,
            "wonder" => SynthKind::Wonder,
            "bell" => SynthKind::Bell,
            "snowballs" => SynthKind::Snowballs,
            "splash" => SynthKind::Splash,
            _ => SynthKind::Mood,
        }
    }

    /// How long the voice rings, in seconds (the legacy `voice_len`).
    pub fn len(self) -> f32 {
        match self {
            SynthKind::Rain => 0.8,
            SynthKind::Anvil => 0.4,
            SynthKind::Festival => 1.4,
            SynthKind::Wonder => 1.5,
            SynthKind::Bell => 2.0,
            SynthKind::Snowballs => 0.9,
            SynthKind::Splash => 0.3,
            SynthKind::Mood => 0.25,
        }
    }

    /// One sample of the voice at `t` seconds from its start, `noise`
    /// a -1..1 draw for the voices that want it.
    pub fn sample(self, t: f32, noise: f32) -> f32 {
        match self {
            // hiss under the weather: noise, mid decay
            SynthKind::Rain => noise * (-3.0 * t).exp() * 0.5,
            // the smithy: an inharmonic clang, fast decay
            SynthKind::Anvil => ((2.0 * PI * 815.0 * t).sin() + 0.5 * (2.0 * PI * 2130.0 * t).sin())
                * (-12.0 * t).exp() * 0.6,
            // a fair day: a major triad, a long ring
            SynthKind::Festival => ((2.0 * PI * 392.0 * t).sin() + (2.0 * PI * 494.0 * t).sin()
                + (2.0 * PI * 587.0 * t).sin()) / 3.0 * (-3.0 * t).exp() * 0.7,
            // a comet or a blessing: a high tone that swells then fades
            SynthKind::Wonder => (2.0 * PI * 1175.0 * t).sin()
                * (1.0 - (-4.0 * t).exp()) * (-3.0 * t).exp() * 0.6,
            // the church bell: low inharmonic partials, a long ring
            SynthKind::Bell => ((2.0 * PI * 220.0 * t).sin() + 0.6 * (2.0 * PI * 523.0 * t).sin()
                + 0.4 * (2.0 * PI * 891.0 * t).sin()) * (-2.5 * t).exp() * 0.7,
            // the snowball fight's hubbub: warbling children shrieks, short
            SynthKind::Snowballs => ((2.0 * PI * 660.0 * t).sin() + (2.0 * PI * 880.0 * t).sin())
                * (0.5 + 0.5 * (2.0 * PI * 9.0 * t).sin()) * (-8.0 * t).exp() * 0.5,
            // a bucket on the flames: a short crackle of water
            SynthKind::Splash => noise.signum() * noise.abs() * (-14.0 * t).exp() * 0.5,
            // anything else: a small blip so it isn't silent
            SynthKind::Mood => (2.0 * PI * 520.0 * t).sin() * (-15.0 * t).exp() * 0.4,
        }
    }
}
