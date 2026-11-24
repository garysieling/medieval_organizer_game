//! Musical composition: the clock that counts time in samples (M3),
//! the phrase grammar, motif library and composer (M4). Everything is
//! deterministic - a bar index is a count of boundaries, a note time is
//! a sample index, and offline renders are bit-exact.

pub mod clock;
pub mod motifs;
mod phrase;
#[cfg(test)]
mod clock_tests;

pub use clock::{Clock, Tick};
pub use motifs::MotifId;
pub use phrase::{free_step, notes_hash, tempo_band, Composer, Ending, Phrase, PhraseSpec,
    Section, TEMPO_AMBIENT, TEMPO_BUILD, TEMPO_FEAST, TEMPO_THEME};

/// The eight voices a phrase is arranged over, in the design's order.
/// Density arrays (`[f32; 8]` in M4's spec) index by this order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Voice {
    /// Tonic and fifth, held.
    Drone,
    /// Euclidean percussion (tabor, frame drum) scaled by density.
    Pulse,
    /// Bass on beats 1 and 3.
    Ground,
    /// Lute open fifths; organum parallel 4ths/5ths in Sacred.
    Harmony,
    /// The motif, stated then varied.
    Melody,
    /// The tintinnabuli T-voice, confined to the tonic triad.
    Counter,
    /// Bells and finger cymbals on phrase starts.
    Color,
    /// Formant "ooh" pad; Sacred sections only.
    Choir,
}

/// A scheduled note: sample-exact onset, duration in samples, the
/// voice that owns it, a fractional midi pitch (fractional cents carry
/// detune for sampled patches; composition writes whole semitones),
/// velocity, and the patch it plays on (M2's `instruments::PatchId`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NoteEvent {
    pub at_sample: u64,
    pub dur_samples: u32,
    pub voice: Voice,
    pub midi: f32,
    /// 0.0..1.0
    pub vel: f32,
    pub patch: crate::instruments::PatchId,
}

/// Time signature: `num` beats of `den`-note length per bar. The
/// composer supports 4/4, 3/4, 6/8 and 2/4 - in 6/8 the clock counts
/// six eighth-note beats, the composer writes the compound feel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Meter {
    pub num: u8,
    pub den: u8,
}

impl Meter {
    pub const FOUR_FOUR: Meter = Meter { num: 4, den: 4 };
    pub const THREE_FOUR: Meter = Meter { num: 3, den: 4 };
    pub const SIX_EIGHT: Meter = Meter { num: 6, den: 8 };
    pub const TWO_FOUR: Meter = Meter { num: 2, den: 4 };
}

/// When a tempo or meter request takes effect. Changes land only on
/// boundaries, never mid-bar (the iMUSE rule).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quantize {
    /// Applies at the next bar boundary.
    Bar,
    /// Applies at the next phrase boundary (every 8 bars).
    Phrase,
}
