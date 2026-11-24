//! Pure music theory for the composer and director (sound design M1):
//! church modes on a fixed tonic, key signatures spelled on the tonic's
//! letter, diatonic triads, the tintinnabuli T-voice rule, authored
//! motifs and their transforms, Euclidean rhythms, and instrument
//! ranges. No audio, no sim access, no rng - every function is pure.

mod instrument;
mod motif;

pub use instrument::Instrument;
pub use motif::{euclid, Motif};

/// A MIDI pitch class, 0..12, C = 0. Spelled against letter names in
/// `key_signature`, so F# and Gb are the same `Pc` here and differ only
/// in which list they land in.
pub type Pc = u8;

/// The six-mode brightness ladder, index 0 = brightest. Verdict steps
/// move along it (win +2 toward Lydian, loss -1 toward Phrygian); the
/// tonic never moves - on D each step adds or removes one accidental.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Lydian,
    Ionian,
    Mixolydian,
    Dorian,
    Aeolian,
    Phrygian,
}

impl Mode {
    pub const ALL: [Mode; 6] = [
        Mode::Lydian,
        Mode::Ionian,
        Mode::Mixolydian,
        Mode::Dorian,
        Mode::Aeolian,
        Mode::Phrygian,
    ];

    /// Semitones of each scale degree above the tonic, degrees 1..7.
    pub fn intervals(self) -> [u8; 7] {
        match self {
            Mode::Lydian => [0, 2, 4, 6, 7, 9, 11],
            Mode::Ionian => [0, 2, 4, 5, 7, 9, 11],
            Mode::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            Mode::Dorian => [0, 2, 3, 5, 7, 9, 10],
            Mode::Aeolian => [0, 2, 3, 5, 7, 8, 10],
            Mode::Phrygian => [0, 1, 3, 5, 7, 8, 10],
        }
    }

    /// Steps along the brightness ladder: positive moves toward Lydian,
    /// negative toward Phrygian, clamped at the ends. A win's +2 from
    /// the Dorian resting point lands on Lydian; a loss's -1 on Aeolian.
    pub fn steps_from(self, n: i8) -> Mode {
        let i = self as i32 - n as i32;
        Mode::ALL[i.clamp(0, 5) as usize]
    }
}

/// Letter name of a pitch class when spelled natural, 0 = C .. 6 = B.
/// Each letter's natural pitch class: C0 D2 E4 F5 G7 A9 B11.
const LETTER_NATURAL: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Sharp side of the circle of fifths, F# first: the order accidentals
/// appear in a key signature.
const SHARP_ORDER: [Pc; 7] = [6, 1, 8, 3, 10, 5, 0];
/// Flat order is the sharp order reversed, Bb first.
const FLAT_ORDER: [Pc; 7] = [10, 3, 8, 1, 6, 11, 4];

/// A key signature spelled on the tonic's letter: on D, Lydian is
/// sharps [F#, C#, G#] and Phrygian is flats [Bb, Eb] - the "key
/// signature really changes" claim of the design, made literal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeySig {
    /// Altered pitch classes in conventional signature order (F# C# G# ...).
    pub sharps: Vec<Pc>,
    /// Altered pitch classes in conventional signature order (Bb Eb Ab ...).
    pub flats: Vec<Pc>,
}

/// The accidentals a mode needs on a given tonic, spelled against the
/// tonic's own letter so D's signature is always written over D E F G A
/// B C. An altered degree becomes a sharp when the mode's pitch sits a
/// semitone above the natural on that letter, a flat when below; a
/// degree never moves by more than a semitone between church modes.
pub fn key_signature(tonic: Pc, mode: Mode) -> KeySig {
    let tonic_pc = tonic % 12;
    // The tonic's letter: find the letter whose natural pitch class is
    // closest to the tonic (the tonic itself may carry an accidental).
    let tonic_letter = (0..7)
        .min_by_key(|&l| {
            let d = (tonic_pc as i32 - LETTER_NATURAL[l] as i32).rem_euclid(12);
            d.min(12 - d)
        })
        .unwrap();
    let mut sharps = Vec::new();
    let mut flats = Vec::new();
    for (deg, &iv) in mode.intervals().iter().enumerate() {
        let letter = (tonic_letter + deg) % 7;
        let pitch = (tonic_pc as i32 + iv as i32).rem_euclid(12) as Pc;
        let diff = (pitch as i32 - LETTER_NATURAL[letter] as i32).rem_euclid(12);
        match diff {
            1 => sharps.push(pitch),
            11 => flats.push(pitch),
            _ => {}
        }
    }
    let order = |v: &mut Vec<Pc>, ord: &[Pc; 7]| {
        v.sort_by_key(|pc| ord.iter().position(|o| o == pc).unwrap_or(7));
    };
    order(&mut sharps, &SHARP_ORDER);
    order(&mut flats, &FLAT_ORDER);
    KeySig { sharps, flats }
}

/// A scale degree, 1-based: 1 is the tonic, 8 the octave above, 0 and
/// negatives reach below the tonic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Degree(pub i8);

/// MIDI note for a degree in a mode over a tonic (also a midi note
/// number; its octave sets the register). Degrees outside 1..=7 wrap
/// through the scale with octave shifts.
pub fn midi(tonic: u8, mode: Mode, d: Degree) -> u8 {
    let k = (d.0 as i32 - 1).rem_euclid(7);
    let oct = (d.0 as i32 - 1).div_euclid(7);
    (tonic as i32 + mode.intervals()[k as usize] as i32 + 12 * oct) as u8
}

/// The diatonic triad on a degree: degrees d, d+2, d+4 stacked in
/// thirds, as absolute midi notes.
pub fn triad(tonic: u8, mode: Mode, d: Degree) -> [u8; 3] {
    [
        midi(tonic, mode, d),
        midi(tonic, mode, Degree(d.0 + 2)),
        midi(tonic, mode, Degree(d.0 + 4)),
    ]
}

/// The tintinnabuli T-voice: for each melody note, the nearest triad
/// tone - always consonant for free. Candidates come from the triad in
/// either octave direction; ties go down, per Pärt's practice.
pub fn tintinnabuli_t(note: u8, triad: [u8; 3]) -> u8 {
    let mut best = note;
    let mut best_dist = i32::MAX;
    for &t in &triad {
        // The triad pitch nearest `note`, wrapped within +-6 semitones.
        let cand = (note as i32 + (t as i32 - note as i32 + 6).rem_euclid(12) - 6).clamp(0, 127);
        let dist = (cand - note as i32).abs();
        if dist < best_dist || (dist == best_dist && cand < best as i32) {
            best = cand as u8;
            best_dist = dist;
        }
    }
    best
}

#[cfg(test)]
mod tests;
