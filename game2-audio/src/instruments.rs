//! The instrument patches (M2/#1499): a voice for every row of the
//! design's Instruments table. Sampled patches are zone tables over
//! committed `assets/sound/instruments/*.ogg` buffers (VCSL, VSCO 2 CE
//! and the FreePats bagpipe - all CC0, all in the ledger); the rest are
//! synthesized (`SynthPatch` in synth.rs). `register_all` wires the
//! sampled defs into a `SampleBank` once its buffers are loaded.
use crate::engine::Source;
use crate::sampler::{Patch, SampleBank, Zone};
use crate::synth::SynthPatch;
use crate::theory::Instrument;

/// Every instrument the composer can write for (M2). `bank_name` is
/// the name `SampleBank::patch_id` resolves - the composer triggers
/// `Source::Sample { id, midi }` against it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PatchId {
    RecorderSoprano,
    RecorderAlto,
    RecorderTenor,
    RecorderBass,
    Fife,
    Shawm,
    Vielle,
    BagpipeChanter,
    BagpipeDrone,
    Organ,
    FolkHarp,
    Psaltery,
    Strumstick,
    Lute,
    Gittern,
    HurdyGurdy,
    ChoirPad,
    FrameDrum,
    Tabor,
    Timpani,
    BassDrum,
    Tambourine,
    FingerCymbals,
    HandBells,
    TubularBells,
    SleighBells,
    /// The inharmonic synth bell - church bell until a CC0 one turns up
    Bell,
}

/// A sampled patch's recipe: the zone table naming each buffer, plus a
/// patch-level lowpass (the shawm's oboe band-limited into the
/// middle ages).
pub struct SampledDef {
    /// The patch's tone corner; `f32::INFINITY` leaves it alone
    pub lowpass_hz: f32,
    pub zones: &'static [ZoneDef],
}

/// One zone of a sampled patch: `key_lo..=key_hi` plays `asset`
/// pitch-shifted from `root`, `tune`-cents off nominal. Assets are
/// ledger ids under `assets/sound/instruments/` (no extension).
pub struct ZoneDef {
    pub key_lo: u8,
    pub key_hi: u8,
    pub root: u8,
    /// cents off the zone rate - each file's measured drift off
    /// concert pitch (the chanter set runs ~40 flat, the fife ~10 sharp)
    pub tune: i8,
    pub asset: &'static str,
}

/// What `patch` hands back: buffers to wire, or a voice to synthesize.
pub enum PatchSpec {
    Sampled(&'static SampledDef),
    Synth(SynthPatch),
}

static RECORDER_SOP: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    // VCSL names run an octave under sounding pitch; roots are the
    // measured notes
    zones: &[
        ZoneDef { key_lo: 72, key_hi: 75, root: 72, tune: 0, asset: "recorder-sop-c4" },
        ZoneDef { key_lo: 76, key_hi: 79, root: 76, tune: -10, asset: "recorder-sop-e4" },
        ZoneDef { key_lo: 80, key_hi: 83, root: 80, tune: 0, asset: "recorder-sop-gs4" },
        ZoneDef { key_lo: 84, key_hi: 86, root: 84, tune: 0, asset: "recorder-sop-c5" },
    ],
};
static RECORDER_ALTO: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 65, key_hi: 68, root: 65, tune: 0, asset: "recorder-alt-f3" },
        ZoneDef { key_lo: 69, key_hi: 72, root: 70, tune: 0, asset: "recorder-alt-as3" },
        ZoneDef { key_lo: 73, key_hi: 75, root: 74, tune: -10, asset: "recorder-alt-d4" },
        ZoneDef { key_lo: 76, key_hi: 77, root: 78, tune: -10, asset: "recorder-alt-fs4" },
        ZoneDef { key_lo: 78, key_hi: 83, root: 82, tune: 0, asset: "recorder-alt-as4" },
        ZoneDef { key_lo: 84, key_hi: 85, root: 86, tune: 0, asset: "recorder-alt-d5" },
        ZoneDef { key_lo: 86, key_hi: 91, root: 88, tune: 0, asset: "recorder-alt-e5" },
    ],
};
static RECORDER_TENOR: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 60, key_hi: 62, root: 60, tune: 0, asset: "recorder-ten-c3" },
        ZoneDef { key_lo: 63, key_hi: 66, root: 64, tune: 10, asset: "recorder-ten-e3" },
        ZoneDef { key_lo: 67, key_hi: 69, root: 68, tune: 0, asset: "recorder-ten-gs3" },
        ZoneDef { key_lo: 70, key_hi: 71, root: 70, tune: 0, asset: "recorder-ten-as3" },
        ZoneDef { key_lo: 72, key_hi: 73, root: 72, tune: -10, asset: "recorder-ten-c4" },
        ZoneDef { key_lo: 74, key_hi: 74, root: 74, tune: 0, asset: "recorder-ten-d4" },
    ],
};
static RECORDER_BASS: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 53, key_hi: 56, root: 53, tune: -10, asset: "recorder-bass-f2" },
        ZoneDef { key_lo: 57, key_hi: 60, root: 58, tune: 0, asset: "recorder-bass-as2" },
        ZoneDef { key_lo: 61, key_hi: 63, root: 62, tune: -15, asset: "recorder-bass-d3" },
        ZoneDef { key_lo: 64, key_hi: 67, root: 65, tune: 0, asset: "recorder-bass-f3" },
    ],
};
static FIFE: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 86, key_hi: 92, root: 91, tune: -10, asset: "fife-g6" },
        ZoneDef { key_lo: 93, key_hi: 96, root: 96, tune: -10, asset: "fife-c6" },
    ],
};
/// The shawm is the VSCO oboe past a patch-level lowpass: a nasal reed
/// rather than an orchestra seat
static SHAWM: SampledDef = SampledDef {
    lowpass_hz: 2600.0,
    zones: &[
        ZoneDef { key_lo: 58, key_hi: 61, root: 58, tune: 0, asset: "shawm-as2" },
        ZoneDef { key_lo: 62, key_hi: 67, root: 65, tune: 0, asset: "shawm-f3" },
        ZoneDef { key_lo: 68, key_hi: 71, root: 70, tune: 0, asset: "shawm-as3" },
        ZoneDef { key_lo: 72, key_hi: 75, root: 74, tune: 0, asset: "shawm-d4" },
        ZoneDef { key_lo: 76, key_hi: 79, root: 77, tune: 0, asset: "shawm-f4" },
        ZoneDef { key_lo: 80, key_hi: 83, root: 82, tune: 0, asset: "shawm-as4" },
        ZoneDef { key_lo: 84, key_hi: 85, root: 86, tune: 0, asset: "shawm-d5" },
        ZoneDef { key_lo: 86, key_hi: 93, root: 89, tune: 0, asset: "shawm-f5" },
    ],
};
static VIELLE: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 48, key_hi: 57, root: 55, tune: 0, asset: "vielle-g3" },
        ZoneDef { key_lo: 58, key_hi: 62, root: 60, tune: 0, asset: "vielle-c4" },
        ZoneDef { key_lo: 63, key_hi: 66, root: 64, tune: 0, asset: "vielle-e4" },
        ZoneDef { key_lo: 67, key_hi: 72, root: 69, tune: -20, asset: "vielle-a4" },
        ZoneDef { key_lo: 73, key_hi: 81, root: 76, tune: -20, asset: "vielle-e5" },
        ZoneDef { key_lo: 82, key_hi: 88, root: 88, tune: -20, asset: "vielle-e6" },
    ],
};
static BAGPIPE_CHANTER: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    // the FreePats set sits ~40 cents flat of concert pitch - a real
    // G in the drones' world; roots follow the file labels
    zones: &[
        ZoneDef { key_lo: 65, key_hi: 68, root: 65, tune: 40, asset: "bagpipe-f4" },
        ZoneDef { key_lo: 69, key_hi: 72, root: 70, tune: 40, asset: "bagpipe-as4" },
        ZoneDef { key_lo: 73, key_hi: 75, root: 74, tune: 20, asset: "bagpipe-d5" },
        ZoneDef { key_lo: 76, key_hi: 78, root: 77, tune: 50, asset: "bagpipe-f5" },
        ZoneDef { key_lo: 79, key_hi: 91, root: 79, tune: 20, asset: "bagpipe-g5" },
    ],
};
static BAGPIPE_DRONE: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 31, key_hi: 43, root: 43, tune: 0, asset: "bagpipe-drone-g2" },
        ZoneDef { key_lo: 44, key_hi: 55, root: 55, tune: 0, asset: "bagpipe-drone-g3" },
    ],
};
static ORGAN: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 36, key_hi: 40, root: 36, tune: 0, asset: "organ-c1" },
        ZoneDef { key_lo: 41, key_hi: 46, root: 45, tune: 0, asset: "organ-a1" },
        ZoneDef { key_lo: 47, key_hi: 51, root: 48, tune: 0, asset: "organ-c2" },
        ZoneDef { key_lo: 52, key_hi: 56, root: 54, tune: 0, asset: "organ-fs2" },
        ZoneDef { key_lo: 57, key_hi: 60, root: 58, tune: 0, asset: "organ-as2" },
        ZoneDef { key_lo: 61, key_hi: 66, root: 63, tune: 0, asset: "organ-ds3" },
        ZoneDef { key_lo: 67, key_hi: 71, root: 69, tune: 0, asset: "organ-a3" },
        ZoneDef { key_lo: 72, key_hi: 74, root: 74, tune: 0, asset: "organ-d4" },
        ZoneDef { key_lo: 75, key_hi: 78, root: 75, tune: 0, asset: "organ-ds4" },
        ZoneDef { key_lo: 79, key_hi: 81, root: 81, tune: 0, asset: "organ-a4" },
        ZoneDef { key_lo: 82, key_hi: 84, root: 84, tune: 0, asset: "organ-c5" },
    ],
};
static FOLK_HARP: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 48, key_hi: 50, root: 48, tune: 0, asset: "harp-c2" },
        ZoneDef { key_lo: 51, key_hi: 54, root: 52, tune: 0, asset: "harp-e2" },
        ZoneDef { key_lo: 55, key_hi: 59, root: 58, tune: 0, asset: "harp-as2" },
        ZoneDef { key_lo: 60, key_hi: 63, root: 60, tune: 0, asset: "harp-c3" },
        ZoneDef { key_lo: 64, key_hi: 68, root: 66, tune: 0, asset: "harp-fs3" },
        ZoneDef { key_lo: 69, key_hi: 71, root: 70, tune: 0, asset: "harp-as3" },
        ZoneDef { key_lo: 72, key_hi: 75, root: 74, tune: 0, asset: "harp-d4" },
        ZoneDef { key_lo: 76, key_hi: 79, root: 78, tune: 0, asset: "harp-fs4" },
        ZoneDef { key_lo: 80, key_hi: 83, root: 82, tune: 0, asset: "harp-as4" },
        ZoneDef { key_lo: 84, key_hi: 85, root: 86, tune: 0, asset: "harp-d5" },
        ZoneDef { key_lo: 86, key_hi: 88, root: 90, tune: 0, asset: "harp-fs5" },
    ],
};
static PSALTERY: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 55, key_hi: 71, root: 70, tune: 0, asset: "psaltery-as3" },
        ZoneDef { key_lo: 72, key_hi: 73, root: 72, tune: 0, asset: "psaltery-c4" },
        ZoneDef { key_lo: 74, key_hi: 75, root: 74, tune: 0, asset: "psaltery-d4" },
        ZoneDef { key_lo: 76, key_hi: 77, root: 76, tune: 0, asset: "psaltery-e4" },
        ZoneDef { key_lo: 78, key_hi: 80, root: 78, tune: 0, asset: "psaltery-fs4" },
        ZoneDef { key_lo: 81, key_hi: 83, root: 82, tune: 0, asset: "psaltery-as4" },
        ZoneDef { key_lo: 84, key_hi: 84, root: 84, tune: 0, asset: "psaltery-c5" },
    ],
};
static STRUMSTICK: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 50, key_hi: 53, root: 50, tune: 0, asset: "strumstick-d2" },
        ZoneDef { key_lo: 54, key_hi: 60, root: 57, tune: 0, asset: "strumstick-a2" },
        ZoneDef { key_lo: 61, key_hi: 66, root: 64, tune: 0, asset: "strumstick-e3" },
        ZoneDef { key_lo: 67, key_hi: 72, root: 69, tune: 0, asset: "strumstick-a3" },
        ZoneDef { key_lo: 73, key_hi: 79, root: 79, tune: -10, asset: "strumstick-g4" },
    ],
};
/// Unpitched patches: one or two hit buffers covering their whole play
/// band (the root is a formality - hits play near-rate)
static FRAME_DRUM: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 36, key_hi: 42, root: 40, tune: 0, asset: "frame-drum-1" },
        ZoneDef { key_lo: 43, key_hi: 48, root: 45, tune: 0, asset: "frame-drum-2" },
    ],
};
static TABOR: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 48, key_hi: 54, root: 50, tune: 0, asset: "tabor-1" },
        ZoneDef { key_lo: 55, key_hi: 60, root: 57, tune: 0, asset: "tabor-2" },
    ],
};
static TIMPANI: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 38, key_hi: 47, root: 43, tune: 0, asset: "timpani-1" },
        ZoneDef { key_lo: 48, key_hi: 55, root: 50, tune: 0, asset: "timpani-2" },
    ],
};
static BASS_DRUM: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 24, key_hi: 36, root: 30, tune: 0, asset: "bass-drum-1" },
        ZoneDef { key_lo: 37, key_hi: 48, root: 42, tune: 0, asset: "bass-drum-2" },
    ],
};
static TAMBOURINE: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 60, key_hi: 72, root: 66, tune: 0, asset: "tambourine-1" },
        ZoneDef { key_lo: 73, key_hi: 84, root: 78, tune: 0, asset: "tambourine-2" },
    ],
};
static FINGER_CYMBALS: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 84, key_hi: 108, root: 96, tune: 0, asset: "finger-cymbals-1" },
    ],
};
static HAND_BELLS: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 72, key_hi: 75, root: 72, tune: 0, asset: "hand-bells-c4" },
        ZoneDef { key_lo: 76, key_hi: 80, root: 78, tune: -10, asset: "hand-bells-fs4" },
        ZoneDef { key_lo: 81, key_hi: 85, root: 84, tune: 0, asset: "hand-bells-c5" },
        ZoneDef { key_lo: 86, key_hi: 89, root: 88, tune: 0, asset: "hand-bells-e5" },
        ZoneDef { key_lo: 90, key_hi: 93, root: 92, tune: -10, asset: "hand-bells-gs5" },
        ZoneDef { key_lo: 94, key_hi: 96, root: 94, tune: 0, asset: "hand-bells-as5" },
    ],
};
static TUBULAR_BELLS: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 72, key_hi: 78, root: 76, tune: 0, asset: "tubular-c4" },
        ZoneDef { key_lo: 79, key_hi: 96, root: 80, tune: 0, asset: "tubular-e4" },
    ],
};
static SLEIGH_BELLS: SampledDef = SampledDef {
    lowpass_hz: f32::INFINITY,
    zones: &[
        ZoneDef { key_lo: 72, key_hi: 84, root: 78, tune: 0, asset: "sleigh-1" },
        ZoneDef { key_lo: 85, key_hi: 96, root: 90, tune: 0, asset: "sleigh-2" },
    ],
};

impl PatchId {
    pub const ALL: [PatchId; 27] = [
        PatchId::RecorderSoprano, PatchId::RecorderAlto, PatchId::RecorderTenor,
        PatchId::RecorderBass, PatchId::Fife, PatchId::Shawm, PatchId::Vielle,
        PatchId::BagpipeChanter, PatchId::BagpipeDrone, PatchId::Organ,
        PatchId::FolkHarp, PatchId::Psaltery, PatchId::Strumstick,
        PatchId::Lute, PatchId::Gittern, PatchId::HurdyGurdy, PatchId::ChoirPad,
        PatchId::FrameDrum, PatchId::Tabor, PatchId::Timpani, PatchId::BassDrum,
        PatchId::Tambourine, PatchId::FingerCymbals, PatchId::HandBells,
        PatchId::TubularBells, PatchId::SleighBells, PatchId::Bell,
    ];

    /// The M1 instrument the patch stands in for (its range, its role)
    pub fn instrument(self) -> Instrument {
        match self {
            PatchId::RecorderSoprano => Instrument::RecorderSoprano,
            PatchId::RecorderAlto => Instrument::RecorderAlto,
            PatchId::RecorderTenor => Instrument::RecorderTenor,
            PatchId::RecorderBass => Instrument::RecorderBass,
            PatchId::Fife => Instrument::Fife,
            PatchId::Shawm => Instrument::Shawm,
            PatchId::Vielle => Instrument::Vielle,
            PatchId::BagpipeChanter => Instrument::BagpipeChanter,
            PatchId::BagpipeDrone => Instrument::BagpipeDrone,
            PatchId::Organ => Instrument::Organ,
            PatchId::FolkHarp => Instrument::FolkHarp,
            PatchId::Psaltery => Instrument::Psaltery,
            PatchId::Strumstick => Instrument::Strumstick,
            PatchId::Lute => Instrument::Lute,
            PatchId::Gittern => Instrument::Gittern,
            PatchId::HurdyGurdy => Instrument::HurdyGurdy,
            PatchId::ChoirPad => Instrument::ChoirPad,
            PatchId::FrameDrum => Instrument::FrameDrum,
            PatchId::Tabor => Instrument::Tabor,
            PatchId::Timpani => Instrument::Timpani,
            PatchId::BassDrum => Instrument::BassDrum,
            PatchId::Tambourine => Instrument::Tambourine,
            PatchId::FingerCymbals => Instrument::FingerCymbals,
            PatchId::HandBells | PatchId::TubularBells | PatchId::SleighBells
                | PatchId::Bell => Instrument::Bells,
        }
    }

    /// The `SampleBank::patch_id` name a composer looks the patch up
    /// by (sampled) or the name the synth spec carries (synth).
    pub fn bank_name(self) -> &'static str {
        match self {
            PatchId::RecorderSoprano => "instr.recorder-soprano",
            PatchId::RecorderAlto => "instr.recorder-alto",
            PatchId::RecorderTenor => "instr.recorder-tenor",
            PatchId::RecorderBass => "instr.recorder-bass",
            PatchId::Fife => "instr.fife",
            PatchId::Shawm => "instr.shawm",
            PatchId::Vielle => "instr.vielle",
            PatchId::BagpipeChanter => "instr.bagpipe-chanter",
            PatchId::BagpipeDrone => "instr.bagpipe-drone",
            PatchId::Organ => "instr.organ",
            PatchId::FolkHarp => "instr.folk-harp",
            PatchId::Psaltery => "instr.psaltery",
            PatchId::Strumstick => "instr.strumstick",
            PatchId::Lute => "instr.lute",
            PatchId::Gittern => "instr.gittern",
            PatchId::HurdyGurdy => "instr.hurdy-gurdy",
            PatchId::ChoirPad => "instr.choir-pad",
            PatchId::FrameDrum => "instr.frame-drum",
            PatchId::Tabor => "instr.tabor",
            PatchId::Timpani => "instr.timpani",
            PatchId::BassDrum => "instr.bass-drum",
            PatchId::Tambourine => "instr.tambourine",
            PatchId::FingerCymbals => "instr.finger-cymbals",
            PatchId::HandBells => "instr.hand-bells",
            PatchId::TubularBells => "instr.tubular-bells",
            PatchId::SleighBells => "instr.sleigh-bells",
            PatchId::Bell => "instr.bell",
        }
    }
}

/// The patch behind an id: a zone recipe for the sampler, or a voice
/// for the synth.
pub fn patch(id: PatchId) -> PatchSpec {
    match id {
        PatchId::RecorderSoprano => PatchSpec::Sampled(&RECORDER_SOP),
        PatchId::RecorderAlto => PatchSpec::Sampled(&RECORDER_ALTO),
        PatchId::RecorderTenor => PatchSpec::Sampled(&RECORDER_TENOR),
        PatchId::RecorderBass => PatchSpec::Sampled(&RECORDER_BASS),
        PatchId::Fife => PatchSpec::Sampled(&FIFE),
        PatchId::Shawm => PatchSpec::Sampled(&SHAWM),
        PatchId::Vielle => PatchSpec::Sampled(&VIELLE),
        PatchId::BagpipeChanter => PatchSpec::Sampled(&BAGPIPE_CHANTER),
        PatchId::BagpipeDrone => PatchSpec::Sampled(&BAGPIPE_DRONE),
        PatchId::Organ => PatchSpec::Sampled(&ORGAN),
        PatchId::FolkHarp => PatchSpec::Sampled(&FOLK_HARP),
        PatchId::Psaltery => PatchSpec::Sampled(&PSALTERY),
        PatchId::Strumstick => PatchSpec::Sampled(&STRUMSTICK),
        PatchId::FrameDrum => PatchSpec::Sampled(&FRAME_DRUM),
        PatchId::Tabor => PatchSpec::Sampled(&TABOR),
        PatchId::Timpani => PatchSpec::Sampled(&TIMPANI),
        PatchId::BassDrum => PatchSpec::Sampled(&BASS_DRUM),
        PatchId::Tambourine => PatchSpec::Sampled(&TAMBOURINE),
        PatchId::FingerCymbals => PatchSpec::Sampled(&FINGER_CYMBALS),
        PatchId::HandBells => PatchSpec::Sampled(&HAND_BELLS),
        PatchId::TubularBells => PatchSpec::Sampled(&TUBULAR_BELLS),
        PatchId::SleighBells => PatchSpec::Sampled(&SLEIGH_BELLS),
        PatchId::Lute => PatchSpec::Synth(SynthPatch::Pluck { body: crate::synth::PluckBody::Lute }),
        PatchId::Gittern => PatchSpec::Synth(SynthPatch::Pluck { body: crate::synth::PluckBody::Gittern }),
        PatchId::HurdyGurdy => PatchSpec::Synth(SynthPatch::HurdyGurdy),
        PatchId::ChoirPad => PatchSpec::Synth(SynthPatch::ChoirPad),
        PatchId::Bell => PatchSpec::Synth(SynthPatch::Bell),
    }
}

/// Assemble a playable `Patch` from a def: each zone's buffer comes
/// from the bank; a missing asset drops its zone (the bank plays
/// silence for an empty patch - the ledger gate owns loudness).
pub fn build_patch(def: &SampledDef, bank: &SampleBank) -> Patch {
    let zones = def.zones.iter()
        .filter_map(|z| bank.sample_id(z.asset).map(|sample| Zone {
            key_lo: z.key_lo, key_hi: z.key_hi, root: z.root, tune: z.tune,
            loop_start: None, loop_end: None, sample,
        }))
        .collect();
    Patch { zones, tone_hz: def.lowpass_hz }
}

/// The `Source` a note on `id` plays: sampled patches come from the
/// bank, synth patches voice themselves. A bank missing its buffers
/// resolves to silence (the engine logs it once).
pub fn source(id: PatchId, bank: &SampleBank, midi: u8) -> Source {
    match patch(id) {
        PatchSpec::Sampled(_) => Source::Sample {
            id: bank.patch_id(id.bank_name())
                .unwrap_or(crate::sampler::PatchId(u32::MAX)),
            midi,
        },
        PatchSpec::Synth(p) => Source::NoteSynth { patch: p, midi },
    }
}

/// Wire every sampled patch onto the bank under its `bank_name` -
/// called after the instrument buffers are in (`load_dir`, the web
/// loader).
pub fn register_all(bank: &mut SampleBank) {
    for id in PatchId::ALL {
        if let PatchSpec::Sampled(def) = patch(id) {
            bank.register_patch(id.bank_name(), build_patch(def, bank));
        }
    }
}
