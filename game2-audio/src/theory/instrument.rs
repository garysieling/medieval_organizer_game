//! The medieval palette's instruments and their sounding ranges in
//! midi notes. The composer clips every generated note to `range()` so
//! a line is always playable by the patch that voices it.
//!
//! Range sources: recorders per VCSL's baroque set (S C5-D7, A F4-G6,
//! T C4-D6, B F3-G5, the family written on C/F); fife and shawm via the
//! VSCO 2 CE patches that voice them (piccolo D5-C8, oboe Bb3-A6, both
//! band-limited by their patches); vielle to its modern ancestor the
//! viola (C3-E6); bagpipe chanter FreePats' stated F4-G6 with the two
//! drones an octave band; lute the renaissance 8-course span (E2-D5,
//! guitar-like); gittern the lute's smaller cousin a fourth up;
//! folk harp the celtic range (C3-E6); psaltery the bowed instrument's
//! usual G3-C6; renaissance organ C2-C6; bells hand-bell/tubular C5-C8;
//! hurdy-gurdy chanter+drone G2-D5. Where a medieval instrument's
//! compass is unverifiable, its modern ancestor's range stands in.

/// An instrument the composer can write for. `PatchId` (M2) maps these
/// to sampled or synthesized voices; this enum is the theory-level name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Instrument {
    RecorderSoprano,
    RecorderAlto,
    RecorderTenor,
    RecorderBass,
    Fife,
    Shawm,
    Vielle,
    /// FreePats bagpipe in G: F4-G6 sounding.
    BagpipeChanter,
    /// The two drones: a narrow low band, effectively fixed.
    BagpipeDrone,
    Lute,
    Gittern,
    FolkHarp,
    Psaltery,
    Organ,
    /// Hand bells, tubular bells, finger cymbals, sleigh bells.
    Bells,
    HurdyGurdy,
}

impl Instrument {
    pub const ALL: [Instrument; 16] = [
        Instrument::RecorderSoprano,
        Instrument::RecorderAlto,
        Instrument::RecorderTenor,
        Instrument::RecorderBass,
        Instrument::Fife,
        Instrument::Shawm,
        Instrument::Vielle,
        Instrument::BagpipeChanter,
        Instrument::BagpipeDrone,
        Instrument::Lute,
        Instrument::Gittern,
        Instrument::FolkHarp,
        Instrument::Psaltery,
        Instrument::Organ,
        Instrument::Bells,
        Instrument::HurdyGurdy,
    ];

    /// Sounding range as inclusive midi notes (low, high).
    pub fn range(self) -> (u8, u8) {
        match self {
            Instrument::RecorderSoprano => (72, 86), // C5-D7
            Instrument::RecorderAlto => (65, 91),    // F4-G6
            Instrument::RecorderTenor => (60, 74),   // C4-D6
            Instrument::RecorderBass => (53, 67),    // F3-G5
            Instrument::Fife => (86, 96),            // D6-C8 (piccolo family)
            Instrument::Shawm => (58, 93),           // Bb3-A6 (oboe, filtered)
            Instrument::Vielle => (48, 88),          // C3-E6 (viola ancestor)
            Instrument::BagpipeChanter => (65, 91),  // F4-G6, FreePats
            Instrument::BagpipeDrone => (31, 43),    // G1-G2 band
            Instrument::Lute => (40, 74),            // E2-D5
            Instrument::Gittern => (48, 72),         // C3-C5
            Instrument::FolkHarp => (48, 88),        // C3-E6
            Instrument::Psaltery => (55, 84),        // G3-C6
            Instrument::Organ => (36, 84),           // C2-C6
            Instrument::Bells => (72, 96),           // C5-C8
            Instrument::HurdyGurdy => (43, 74),      // G2-D5
        }
    }

    /// True when `note` sits inside the instrument's sounding range.
    pub fn in_range(self, note: u8) -> bool {
        let (lo, hi) = self.range();
        (lo..=hi).contains(&note)
    }
}
