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
    /// A fire bell or the knights riding out: a fast tolling clang
    Alarm,
    /// The passing bell: a single slow ring for a death
    Toll,
    /// The street sends the column home: a crowd noise swell
    Cheer,
    /// The march's voice on the road: a low drone under the column
    Chant,
    /// The till rings: a couple of metallic ticks
    Coins,
    /// A letter scratched out: a dry quill scratch
    Quill,
    /// A knuckled door: a low wooden knock
    Door,
    /// A cart or a levy train rolls in: wheels and hooves clatter
    CartArrive,
    /// The palisade gap's hinge: a slow creak
    Gate,
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
            "alarm" => SynthKind::Alarm,
            "toll" => SynthKind::Toll,
            "cheer" => SynthKind::Cheer,
            "chant" => SynthKind::Chant,
            "coins" => SynthKind::Coins,
            "quill" => SynthKind::Quill,
            "door" => SynthKind::Door,
            "cart_arrive" => SynthKind::CartArrive,
            "gate" => SynthKind::Gate,
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
            SynthKind::Alarm => 1.2,
            SynthKind::Toll => 2.2,
            SynthKind::Cheer => 1.3,
            SynthKind::Chant => 1.6,
            SynthKind::Coins => 0.5,
            SynthKind::Quill => 0.5,
            SynthKind::Door => 0.7,
            SynthKind::CartArrive => 1.0,
            SynthKind::Gate => 0.9,
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
            // the alarm: the bell struck four to the second, urgent
            SynthKind::Alarm => {
                let s = t % 0.25;
                ((2.0 * PI * 660.0 * s).sin() + 0.5 * (2.0 * PI * 1330.0 * s).sin())
                    * (-14.0 * s).exp() * (1.0 - t / 1.4).max(0.0) * 0.6
            }
            // the passing bell: one low ring, slow to fade
            SynthKind::Toll => ((2.0 * PI * 165.0 * t).sin() + 0.6 * (2.0 * PI * 392.0 * t).sin()
                + 0.35 * (2.0 * PI * 668.0 * t).sin()) * (-1.8 * t).exp() * 0.7,
            // the crowd behind the column: noise swelling up then away
            SynthKind::Cheer => noise * 0.6 * (1.0 - (-6.0 * t).exp()) * (-4.0 * t).exp()
                + (2.0 * PI * 523.0 * t).sin() * (-5.0 * t).exp() * 0.15,
            // the column's drone: a low voice that swells and holds
            SynthKind::Chant => ((2.0 * PI * 110.0 * t).sin() + 0.5 * (2.0 * PI * 165.0 * t).sin()
                + 0.3 * (2.0 * PI * 220.0 * t).sin())
                * (1.0 - (-5.0 * t).exp()) * (-2.5 * t).exp() * 0.5,
            // the till: three small coins, each a short high ping
            SynthKind::Coins => {
                let s = t % 0.16;
                ((2.0 * PI * 2093.0 * s).sin() + 0.4 * (2.0 * PI * 2960.0 * s).sin())
                    * (-40.0 * s).exp() * 0.5
            }
            // the quill: three strokes of dry scratch
            SynthKind::Quill => {
                let s = t % 0.17;
                noise * (-30.0 * s).exp() * (0.3 + 0.7 * (2.0 * PI * 40.0 * s).sin().abs()) * 0.4
            }
            // the knock: two raps on the wood - a low thud each
            SynthKind::Door => {
                let s = t % 0.35;
                ((2.0 * PI * 90.0 * s).sin() + noise * 0.3) * (-25.0 * s).exp() * 0.7
            }
            // the cart: wheels and hooves in a rolling clatter
            SynthKind::CartArrive => {
                let s = t % 0.2;
                ((2.0 * PI * 140.0 * s).sin() + noise * 0.5) * (-18.0 * s).exp() * 0.6
            }
            // the hinge: a low tone bent slowly upward, dry at the edges
            SynthKind::Gate => ((2.0 * PI * (180.0 + 160.0 * t) * t).sin()
                + noise * 0.15 * (2.0 * PI * 6.0 * t).sin().abs())
                * (1.0 - (-8.0 * t).exp()) * (-4.0 * t).exp() * 0.5,
            // anything else: a small blip so it isn't silent
            SynthKind::Mood => (2.0 * PI * 520.0 * t).sin() * (-15.0 * t).exp() * 0.4,
        }
    }
}
