//! The voice each cue asks for: a ledgered sample to prefer when the
//! bank holds it, the synth that stands in for it, the bus it plays on,
//! and how the cue's own pan and intensity land on the voice. A4's
//! recordings fill the ledger's sample names; until a name's banked
//! every cue resolves to its synth.

use crate::engine::Bus;
use crate::host::CueKind;
use crate::synth::SynthKind;

/// How a cue sounds: the patch to prefer, the synth fallback, the bus,
/// and the trims the cue's own numbers get.
#[derive(Clone, Copy, Debug)]
pub struct VoiceSpec {
    /// The ledger patch name to prefer - `None` means the cue only ever
    /// wants its synth. The host asks the bank for it each play.
    pub sample: Option<&'static str>,
    /// The voice used when no sample's banked for the cue.
    pub synth: SynthKind,
    /// The bus the voice plays on.
    pub bus: Bus,
    /// A loudness trim multiplied onto the cue's own intensity.
    pub gain: f32,
    /// Whether the cue's stereo place comes off the world's map x.
    /// Scene cues pan with the tile that raised them; a cue that wants
    /// the middle says so here.
    pub pan_from_pos: bool,
}

const fn spec(sample: Option<&'static str>, synth: SynthKind, gain: f32) -> VoiceSpec {
    VoiceSpec { sample, synth, bus: Bus::Sfx, gain, pan_from_pos: true }
}

/// The voice for a cue - an exhaustive match so a new `CueKind` can't
/// ship without its voice. The cue's pan and intensity are its own;
/// the spec's `gain` trims them.
pub fn voice_for(cue: CueKind) -> VoiceSpec {
    match cue {
        // the weather bed stays a synth - rain is noise, not a strike
        CueKind::Rain { .. } => spec(None, SynthKind::Rain, 1.0),
        CueKind::Anvil { .. } => spec(Some("anvil"), SynthKind::Anvil, 1.0),
        CueKind::Festival { .. } => spec(Some("festival"), SynthKind::Festival, 1.0),
        CueKind::Wonder { .. } => spec(Some("wonder"), SynthKind::Wonder, 1.0),
        CueKind::Bell { .. } => spec(Some("bell"), SynthKind::Bell, 1.0),
        CueKind::Snowballs { .. } => spec(Some("snowballs"), SynthKind::Snowballs, 1.0),
        CueKind::Splash { .. } => spec(Some("splash"), SynthKind::Splash, 1.0),
        CueKind::Alarm { .. } => spec(Some("alarm"), SynthKind::Alarm, 1.0),
        // the passing bell stays quieter than the fire bell
        CueKind::Toll { .. } => spec(Some("toll"), SynthKind::Toll, 0.9),
        CueKind::Cheer { .. } => spec(Some("cheer"), SynthKind::Cheer, 0.9),
        CueKind::Chant { .. } => spec(Some("chant"), SynthKind::Chant, 0.8),
        CueKind::Coins { .. } => spec(Some("coins"), SynthKind::Coins, 0.9),
        CueKind::Quill { .. } => spec(Some("quill"), SynthKind::Quill, 0.8),
        CueKind::Door { .. } => spec(Some("door"), SynthKind::Door, 1.0),
        CueKind::CartArrive { .. } => spec(Some("cart_arrive"), SynthKind::CartArrive, 0.9),
        CueKind::Gate { .. } => spec(Some("gate"), SynthKind::Gate, 0.9),
        // a wind sample can land with the A4b batch; the synth gusts until then
        CueKind::Gale { .. } => spec(Some("gale"), SynthKind::Gale, 0.8),
        CueKind::Murmur { .. } => spec(Some("murmur"), SynthKind::Murmur, 0.8),
        CueKind::Bells { .. } => spec(Some("bells"), SynthKind::Bells, 0.9),
        CueKind::Baying { .. } => spec(Some("baying"), SynthKind::Baying, 0.9),
        CueKind::Hens { .. } => spec(Some("hens"), SynthKind::Hens, 0.9),
        CueKind::Blip { .. } => spec(None, SynthKind::Blip, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Engine, Source};

    /// Every name a `SfxCue.kind` can carry - the sim's emit sites and
    /// the legacy cues. `from_name` must find a voice for each.
    const ALL_CUE_NAMES: &[&'static str] = &[
        "rain", "anvil", "festival", "wonder", "bell", "snowballs", "splash",
        "alarm", "toll", "cheer", "chant", "coins", "quill", "door",
        "cart_arrive", "gate", "gale", "murmur", "bells", "baying", "hens",
    ];

    fn cue_source(engine: &Engine, spec: VoiceSpec) -> Source {
        spec.sample
            .and_then(|n| engine.bank().patch_id(n))
            .map(|id| Source::Sample { id, midi: 60 })
            .unwrap_or(Source::Synth(spec.synth))
    }

    #[test]
    fn palette_covers_every_cue() {
        for name in ALL_CUE_NAMES {
            let cue = CueKind::from_name(name, 0.0, 0.5);
            let spec = voice_for(cue);
            assert!(spec.gain > 0.0, "{} has a voice", name);
            // and a ledger name asked for resolves to its patch, else
            // falls back to the synth rather than going silent
            let engine = Engine::new(48_000, 7);
            assert!(matches!(cue_source(&engine, spec), Source::Synth(_)),
                "{} resolves to its synth while the ledger's empty", name);
        }
    }

    #[test]
    fn ledgered_samples_win_when_banked() {
        // when the bank knows the name the cue wants, the sample plays
        // instead of the synth - the whole point of asking
        let mut w = Vec::new();
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&100u32.to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&48_000u32.to_le_bytes());
        w.extend_from_slice(&96_000u32.to_le_bytes());
        w.extend_from_slice(&2u16.to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&64u32.to_le_bytes());
        w.extend_from_slice(&[0u8; 64]);
        let bank = crate::sampler::SampleBank::from_bytes("bell", &w).unwrap();
        let mut engine = Engine::new(48_000, 7);
        engine.set_bank(bank);
        let spec = voice_for(CueKind::Bell { pan: 0.0, gain: 0.5 });
        assert!(matches!(cue_source(&engine, spec), Source::Sample { .. }),
            "a banked patch takes the voice");
    }
}
