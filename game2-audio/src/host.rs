//! The façade every client holds (#1493): `push` files a frame's cues
//! from the public `UiModel`, `render_block` plays them and renders
//! stereo. The same path drives the speakers and the offline files,
//! so what the tests check is what the player hears.
use crate::engine::{Bus, Engine, Source, VoiceParams};
use crate::synth::SynthKind;
use game2_core::session::{UiModel, MAP_TILES};

/// What fired this frame: the cue's voice, its stereo place (-1 left
/// to 1 right, from the map x), and how loud the sim said it was.
#[derive(Clone, Copy, Debug)]
pub enum CueKind {
    Rain { pan: f32, gain: f32 },
    Anvil { pan: f32, gain: f32 },
    Festival { pan: f32, gain: f32 },
    Wonder { pan: f32, gain: f32 },
    Bell { pan: f32, gain: f32 },
    Snowballs { pan: f32, gain: f32 },
    Splash { pan: f32, gain: f32 },
    /// "mood" and cues without their own voice: a small blip
    Mood { pan: f32, gain: f32 },
}

impl CueKind {
    fn from_name(kind: &str, pan: f32, gain: f32) -> Self {
        match kind {
            "rain" => CueKind::Rain { pan, gain },
            "anvil" => CueKind::Anvil { pan, gain },
            "festival" => CueKind::Festival { pan, gain },
            "wonder" => CueKind::Wonder { pan, gain },
            "bell" => CueKind::Bell { pan, gain },
            "snowballs" => CueKind::Snowballs { pan, gain },
            "splash" => CueKind::Splash { pan, gain },
            _ => CueKind::Mood { pan, gain },
        }
    }

    fn parts(self) -> (f32, f32, SynthKind) {
        let (pan, gain, s) = match self {
            CueKind::Rain { pan, gain } => (pan, gain, SynthKind::Rain),
            CueKind::Anvil { pan, gain } => (pan, gain, SynthKind::Anvil),
            CueKind::Festival { pan, gain } => (pan, gain, SynthKind::Festival),
            CueKind::Wonder { pan, gain } => (pan, gain, SynthKind::Wonder),
            CueKind::Bell { pan, gain } => (pan, gain, SynthKind::Bell),
            CueKind::Snowballs { pan, gain } => (pan, gain, SynthKind::Snowballs),
            CueKind::Splash { pan, gain } => (pan, gain, SynthKind::Splash),
            CueKind::Mood { pan, gain } => (pan, gain, SynthKind::Mood),
        };
        (pan, gain, s)
    }
}

/// One frame's audio intent, pulled off the public model: which cues
/// fired and the world's seed. Later tickets add the mood scan's
/// emitters and the listener spot (`..` is the ticket's own license).
#[derive(Clone, Debug)]
pub struct AudioFrame {
    pub cues: Vec<CueKind>,
    pub seed: u64,
}

impl AudioFrame {
    /// The cues the model reports (`sfx_v`: kind, tile x/y, intensity).
    /// Pan comes off the map x like `record.rs` did, remapped to the
    /// engine's -1..1; y is kept for a later distance term.
    pub fn from_model(model: &UiModel) -> Self {
        AudioFrame {
            cues: model.sfx_v.iter()
                .map(|(kind, x, _y, intensity)| {
                    let pan = (*x as f32 / MAP_TILES as f32).clamp(0.0, 1.0) * 2.0 - 1.0;
                    CueKind::from_name(kind, pan, *intensity)
                })
                .collect(),
            seed: model.seed,
        }
    }
}

/// The client-facing audio object: an engine plus the frame queue in
/// front of it. `offline` is the deterministic renderer - the live
/// output tickets (S4, S5) drive the same struct from a callback.
pub struct AudioHost {
    engine: Engine,
    /// Cues pushed since the last render block: they enter at the
    /// block's first sample, like a live callback picking them up
    pending: Vec<CueKind>,
    /// Fractional-sample carry so 60 fps stays exact at any rate
    carry: f64,
    sr: u32,
    /// Most live voices the engine has held (`Metrics.voices_peak`)
    voices_peak: usize,
}

impl AudioHost {
    /// The offline lockstep host: push a model, then render exactly
    /// `frame_samples()` of stereo before the next push.
    pub fn offline(sample_rate: u32) -> Self {
        AudioHost {
            engine: Engine::new(sample_rate, 0),
            pending: vec![],
            carry: 0.0,
            sr: sample_rate,
            voices_peak: 0,
        }
    }

    /// A frame arrived: file its audio intent for the next block.
    pub fn push(&mut self, model: &UiModel) {
        self.push_frame(&AudioFrame::from_model(model));
    }

    /// The frame's intent straight in (scene replay builds frames
    /// without a model).
    pub fn push_frame(&mut self, frame: &AudioFrame) {
        self.pending.extend_from_slice(&frame.cues);
    }

    /// How many mono samples one 60 fps frame is worth this call: the
    /// fractional remainder carries, so a long run still lands exactly
    /// on `frames * rate / 60`.
    pub fn frame_samples(&mut self) -> usize {
        self.carry += self.sr as f64 / 60.0;
        let n = self.carry.floor() as usize;
        self.carry -= n as f64;
        n
    }

    /// Render `out` (interleaved stereo, even length); queued cues
    /// enter at the block's first sample on the Sfx bus.
    pub fn render_block(&mut self, out: &mut [f32]) {
        for c in self.pending.drain(..) {
            let (pan, gain, kind) = c.parts();
            self.engine.play(Source::Synth(kind), Bus::Sfx,
                VoiceParams { gain, pan, ..Default::default() });
        }
        self.engine.render(out);
        self.voices_peak = self.voices_peak.max(self.engine.live_voices());
    }

    /// The most voices the engine held at once, for `metrics.json`.
    pub fn voices_peak(&self) -> usize { self.voices_peak }
}
