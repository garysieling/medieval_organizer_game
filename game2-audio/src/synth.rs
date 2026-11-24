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

// ---------- instrument voices (M2/#1499) ----------

/// midi note number to Hz (A4 = 69 = 440).
pub fn midi_hz(midi: u8) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

/// A synthesized instrument voice - the design's Instruments rows that
/// have no CC0 recording worth taking (Decision 12 synthesizes the
/// lute and the hurdy-gurdy). Voices are deterministic: state comes
/// from the engine's seeded `Rng` at trigger, the rest is a pure
/// function of `t`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SynthPatch {
    /// Karplus-Strong plucked string, with a one-pole body resonance
    Pluck { body: PluckBody },
    /// Bowed melody string over two drones, with the trompette's
    /// buzzing bridge
    HurdyGurdy,
    /// 'ooh' vowel pad - soft partials swelling in and out
    ChoirPad,
    /// Inharmonic struck bell - the church bell until a CC0 one lands
    Bell,
}

/// The plucked-string bodies: how fast the string's energy dies and
/// where the body rolls the top off.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PluckBody {
    /// Round-back lute: long ring, dark body
    Lute,
    /// Flat-back gittern: shorter, brighter
    Gittern,
}

/// The pluck's Karplus-Strong delay line: `ring` is one period of the
/// note, filled with the noise burst at trigger; each tick writes the
/// damped average of the sample and its neighbour back (the string's
/// own lowpass) and passes the read-out through `body_lp`. `frac`
/// interpolates the read between adjacent taps so the period isn't
/// quantised to whole samples - integer rings run several cents flat
/// at the top of the gittern.
pub struct NoteState {
    pub ring: Vec<f32>,
    pub pos: usize,
    pub damp: f32,
    pub frac: f32,
    pub prev: f32,
    pub body_lp: f32,
    pub body_a: f32,
}

/// A 1-amplitude sawtooth of `phase` (in cycles) - the bowed-string
/// shape.
fn saw(phase: f32) -> f32 { 2.0 * (phase - phase.floor()) - 1.0 }

/// Linear fade-in over `a` seconds, fade-out over the last `r` of `len`.
fn note_env(t: f32, a: f32, len: f32, r: f32) -> f32 {
    (t / a).min(1.0) * (1.0 - ((t - (len - r)).max(0.0) / r).min(1.0))
}

impl SynthPatch {
    /// Seconds the voice needs before its tail has died away - the
    /// engine frees the slot past it.
    pub fn len(self) -> f32 {
        match self {
            SynthPatch::Pluck { body: PluckBody::Lute } => 3.0,
            SynthPatch::Pluck { body: PluckBody::Gittern } => 2.0,
            SynthPatch::HurdyGurdy => 4.0,
            SynthPatch::ChoirPad => 4.0,
            SynthPatch::Bell => 3.0,
        }
    }

    /// Per-voice state at trigger. Only the plucks need memory (the
    /// delay line); everything else is a pure function of `t`.
    pub fn state(self, midi: u8, sr: f32, pitch: f32, rng: &mut crate::rng::Rng)
                 -> Option<NoteState> {
        let SynthPatch::Pluck { body } = self else { return None };
        // the damping average adds ~half a sample of group delay, so
        // the line runs a half-sample short to land on pitch
        let d = (sr / (midi_hz(midi) * pitch) - 0.5).max(2.0);
        let n = d.ceil() as usize;
        let frac = n as f32 - d;
        let (damp, body_hz) = match body {
            PluckBody::Lute => (0.9965, 1400.0),
            PluckBody::Gittern => (0.990, 3000.0),
        };
        Some(NoteState {
            ring: (0..n).map(|_| rng.range(-1.0, 1.0)).collect(),
            pos: 0, damp, frac, prev: 0.0, body_lp: 0.0,
            body_a: 1.0 - (-2.0 * PI * body_hz / sr).exp(),
        })
    }

    /// One output sample, `t` seconds into the note at `midi`; `noise`
    /// is the engine's per-sample -1..1 draw, `st` the voice's
    /// `NoteState` when it has one.
    pub fn sample(self, midi: u8, t: f32, noise: f32, st: Option<&mut NoteState>) -> f32 {
        match self {
            SynthPatch::Pluck { .. } => {
                let Some(st) = st else { return 0.0 };
                let n = st.ring.len();
                let a = st.ring[st.pos];
                let b = st.ring[(st.pos + 1) % n];
                let y = (1.0 - st.frac) * a + st.frac * b;
                st.ring[st.pos] = st.damp * 0.5 * (y + st.prev);
                st.prev = y;
                st.pos = (st.pos + 1) % n;
                st.body_lp += st.body_a * (y - st.body_lp);
                st.body_lp * 1.6 * note_env(t, 0.002, self.len(), 0.4)
            }
            SynthPatch::HurdyGurdy => {
                // melody string under a light vibrato, drones a fifth
                // and octave below; the trompette buzz rides the bow
                let f = midi_hz(midi);
                let ph = f * t + 0.004 * (2.0 * PI * 5.0 * t).sin();
                let mel = saw(ph) * 0.55 + saw(2.0 * ph) * 0.12;
                let drone = saw(f * 0.5 * t) * 0.3 + saw(f * 0.25 * t) * 0.12;
                let buzz = noise * 0.08 * (0.3 + 0.7 * (2.0 * PI * 43.0 * t).sin().abs());
                (mel + drone + buzz) * note_env(t, 0.05, self.len(), 0.6) * 0.5
            }
            SynthPatch::ChoirPad => {
                // 'ooh': a strong fundamental, a soft second, a second
                // voice a shade sharp for the chorus
                let f = midi_hz(midi);
                let v1 = (2.0 * PI * f * t).sin() + 0.35 * (2.0 * PI * 2.0 * f * t).sin()
                    + 0.12 * (2.0 * PI * 3.0 * f * t).sin();
                let v2 = (2.0 * PI * f * 1.004 * t).sin()
                    + 0.3 * (2.0 * PI * 2.008 * f * t).sin();
                (v1 * 0.7 + v2 * 0.5) * note_env(t, 0.35, self.len(), 0.8) * 0.35
            }
            SynthPatch::Bell => {
                // a bell's inharmonic partials, the higher ones dying
                // sooner, scaled to the struck note
                let f = midi_hz(midi);
                let partials = [(0.56f32, 0.5f32), (1.0, 1.0), (1.19, 0.45),
                    (1.56, 0.35), (2.01, 0.3), (2.51, 0.2)];
                let mut y = 0.0;
                for (i, (r, a)) in partials.iter().enumerate() {
                    y += a * (2.0 * PI * f * r * t).sin() * (-(1.5 + i as f32) * t).exp();
                }
                y * note_env(t, 0.002, self.len(), 0.5) * 0.5
            }
        }
    }
}

/// An ambience bed with no sampled asset of its own (A4): the design
/// calls for own-synth noise beds for wind and rain. These play
/// open-endedly (`Source::SynthBed` never retires) and are voiced the
/// same way the cue voices are - a -1..1 `noise` draw per sample from
/// the engine's rng, so a scene renders identically every time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BedKind {
    /// Open air over the hills: broadband noise that swells and settles
    /// on a slow cycle - the emitter's lowpass does the colouring.
    Wind,
    /// Falling water: brighter noise with a faint patter wobble.
    Rain,
    /// A column on the road: boots in step over a low chant - a
    /// tramping pulse of dark noise with a trailing scuff over the
    /// crowd's rumble.
    March,
}

impl BedKind {
    /// The integrator constant the engine band-limits this bed's
    /// noise draw with - wind is darker than rain's brighter patter,
    /// boots darker still.
    pub fn smooth(self) -> f32 {
        match self { BedKind::Wind => 0.30, BedKind::Rain => 0.55,
            BedKind::March => 0.22 }
    }

    /// One sample of the bed at `t` seconds, `noise` a -1..1 draw.
    /// The beds keep their headroom low (the mixer's emitter gain
    /// and the duck own the level).
    pub fn sample(self, t: f32, noise: f32) -> f32 {
        match self {
            // gusts wander on ~7 s and ~1.6 s cycles, never to silence
            BedKind::Wind => noise
                * (0.55 + 0.30 * (2.0 * PI * 0.14 * t).sin()
                    + 0.15 * (2.0 * PI * 0.61 * t + 1.7).sin())
                * 0.35,
            // steady hiss with a soft patter at ~11 Hz
            BedKind::Rain => noise * (0.85 + 0.15 * (2.0 * PI * 11.0 * t).sin()) * 0.30,
            // ~2.4 steps a second: a hard attack decaying over ~90 ms,
            // a faint scuff half a beat behind, and the column's low
            // chant murmuring under the boots
            BedKind::March => {
                let step = (1.0 - (t * 2.4).fract() / 0.09).max(0.0);
                let scuff = (1.0 - ((t * 2.4) + 0.55).fract() / 0.05).max(0.0) * 0.3;
                let chant = 0.38 + 0.08 * (2.0 * PI * 0.31 * t + 0.4).sin();
                noise * ((step + scuff) * 0.5 + chant)
            }
        }
    }
}
