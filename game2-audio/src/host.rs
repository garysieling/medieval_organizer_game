//! The façade every client holds (#1493, #1494): `push` files a frame's
//! cues from the public `UiModel`, `render_block` plays them and renders
//! stereo. The same path drives the speakers and the offline files, so
//! what the tests check is what the player hears.
//!
//! `AudioHost::offline` renders in the caller's lockstep; `start`
//! hands the device half (`LivePump`) to an `AudioSink` whose callback
//! renders it; `silent` no-ops everything - a missing device is a
//! warning, never a crash.
use crate::engine::{Bus, Engine, Source, VoiceParams};
use crate::synth::SynthKind;
use game2_core::session::{UiModel, MAP_TILES};
use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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

/// How a live host opens its device: U1's persisted `AudioSettings`
/// overrides these later.
#[derive(Clone, Copy, Debug)]
pub struct HostSettings {
    pub sample_rate: u32,
    /// Device buffer target; ~100 ms keeps the callback off the game loop's
    /// critical path
    pub buffer_ms: u32,
}
impl Default for HostSettings {
    fn default() -> Self { HostSettings { sample_rate: 44100, buffer_ms: 100 } }
}

/// Frames of intent the game loop may have queued before the device
/// drains them - deeper than the 100 ms buffer at 60 fps.
pub const FRAME_QUEUE: usize = 8;

/// What a real device is, so the bridge is testable with a mock: `open`
/// takes a pump (the live host's render callback) and starts calling it
/// whenever the device wants samples. An error is fine - the host goes
/// silent rather than crash the game.
pub trait AudioSink {
    /// The rate the device will actually run at: the engine renders at
    /// it or the pitch goes sideways. Called before `open`.
    fn sample_rate(&self) -> u32;
    fn open(&mut self, pump: Box<dyn FnMut(&mut [f32]) + Send>) -> Result<(), io::Error>;
}

/// How this host delivers sound.
enum Link {
    /// `offline()`: the engine lives here; the caller renders blocks
    Offline { engine: Engine },
    /// `start()`: frames go down this bounded queue; a sink's callback
    /// renders them. `muted` is shared with the device half
    Live { frames: Arc<Mutex<VecDeque<AudioFrame>>>, muted: Arc<AtomicBool> },
    /// `silent()`: every method is a no-op
    Silent,
}

/// The client-facing audio object. `offline` is the deterministic
/// renderer; `start` is the live device host; `silent` the fallback.
pub struct AudioHost {
    sr: u32,
    /// Cues pushed since the last render block: they enter at the
    /// block's first sample, like a live callback picking them up
    pending: Vec<CueKind>,
    /// Fractional-sample carry so 60 fps stays exact at any rate
    carry: f64,
    /// Most live voices the engine has held (`Metrics.voices_peak`)
    voices_peak: usize,
    link: Link,
}

/// The device half of a live host: the sink calls `pump` in its audio
/// callback - it drains the frame queue and renders.
pub struct LivePump {
    core: AudioHost,
    frames: Arc<Mutex<VecDeque<AudioFrame>>>,
    muted: Arc<AtomicBool>,
}

impl LivePump {
    pub fn pump(&mut self, out: &mut [f32]) {
        {
            let mut q = self.frames.lock().unwrap();
            for f in q.drain(..) { self.core.push_frame(&f); }
        }
        self.core.render_block(out);
        if self.muted.load(Ordering::Relaxed) { out.fill(0.0); }
    }
}

impl AudioHost {
    /// The offline lockstep host: push a model, then render exactly
    /// `frame_samples()` of stereo before the next push.
    pub fn offline(sample_rate: u32) -> Self {
        AudioHost {
            sr: sample_rate,
            pending: vec![],
            carry: 0.0,
            voices_peak: 0,
            link: Link::Offline { engine: Engine::new(sample_rate, 0) },
        }
    }

    /// The fallback every method no-ops on.
    pub fn silent() -> Self {
        AudioHost { sr: 44100, pending: vec![], carry: 0.0,
            voices_peak: 0, link: Link::Silent }
    }

    /// A live host: `sink` gets the render pump (a `LivePump` over the
    /// frame queue) and starts the device; failure means a warning line
    /// and a silent host, never a crash. The engine renders at the
    /// sink's own rate, not the requested one.
    pub fn start(settings: &HostSettings, sink: &mut dyn AudioSink) -> AudioHost {
        let frames = Arc::new(Mutex::new(VecDeque::with_capacity(FRAME_QUEUE)));
        let muted = Arc::new(AtomicBool::new(false));
        let mut pump = LivePump {
            core: AudioHost::offline(sink.sample_rate()),
            frames: frames.clone(),
            muted: muted.clone(),
        };
        match sink.open(Box::new(move |out: &mut [f32]| pump.pump(out))) {
            Ok(()) => AudioHost {
                sr: settings.sample_rate,
                pending: vec![],
                carry: 0.0,
                voices_peak: 0,
                link: Link::Live { frames, muted },
            },
            Err(e) => {
                eprintln!("audio: couldn't open the output device ({e}) - running silent");
                AudioHost::silent()
            }
        }
    }

    /// Whether this host is the no-op fallback.
    pub fn is_silent(&self) -> bool { matches!(self.link, Link::Silent) }

    /// The live queue, for the bridge tests: frames waiting on the device.
    pub fn live_queue(&self) -> Option<Arc<Mutex<VecDeque<AudioFrame>>>> {
        match &self.link { Link::Live { frames, .. } => Some(frames.clone()), _ => None }
    }

    /// A frame arrived: file its audio intent for the next block.
    pub fn push(&mut self, model: &UiModel) {
        self.push_frame(&AudioFrame::from_model(model));
    }

    /// The frame's intent straight in (scene replay builds frames
    /// without a model). A full live queue drops its OLDEST frame -
    /// the game loop never waits on audio.
    pub fn push_frame(&mut self, frame: &AudioFrame) {
        match &mut self.link {
            Link::Offline { .. } => self.pending.extend_from_slice(&frame.cues),
            Link::Live { frames, .. } => {
                let mut q = frames.lock().unwrap();
                if q.len() >= FRAME_QUEUE { q.pop_front(); }
                q.push_back(frame.clone());
            }
            Link::Silent => {}
        }
    }

    /// Mute or unmute: live output goes to zeros, offline renders
    /// silence (Master at zero gives exact zeros), silent stays silent.
    pub fn set_muted(&mut self, on: bool) {
        match &mut self.link {
            Link::Live { muted, .. } => muted.store(on, Ordering::Relaxed),
            Link::Offline { engine } => engine.set_bus_gain(Bus::Master, if on { 0.0 } else { 1.0 }),
            Link::Silent => {}
        }
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
    /// enter at the block's first sample on the Sfx bus. Live and
    /// silent hosts render zeros - the device side owns the engine.
    pub fn render_block(&mut self, out: &mut [f32]) {
        match &mut self.link {
            Link::Offline { engine } => {
                for c in self.pending.drain(..) {
                    let (pan, gain, kind) = c.parts();
                    engine.play(Source::Synth(kind), Bus::Sfx,
                        VoiceParams { gain, pan, ..Default::default() });
                }
                engine.render(out);
                self.voices_peak = self.voices_peak.max(engine.live_voices());
            }
            _ => out.fill(0.0),
        }
    }

    /// The most voices the engine held at once, for `metrics.json`.
    pub fn voices_peak(&self) -> usize { self.voices_peak }
}
