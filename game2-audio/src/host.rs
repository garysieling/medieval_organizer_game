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
use crate::settings::AudioSettings;
use crate::soundscape::ambience::Ambience;
use crate::soundscape::{Emitter, Listener};
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
    Alarm { pan: f32, gain: f32 },
    Toll { pan: f32, gain: f32 },
    Cheer { pan: f32, gain: f32 },
    Chant { pan: f32, gain: f32 },
    Coins { pan: f32, gain: f32 },
    Quill { pan: f32, gain: f32 },
    Door { pan: f32, gain: f32 },
    CartArrive { pan: f32, gain: f32 },
    Gate { pan: f32, gain: f32 },
    /// "mood" and cues without their own voice: a small blip
    Mood { pan: f32, gain: f32 },
}

impl CueKind {
    pub(crate) fn from_name(kind: &str, pan: f32, gain: f32) -> Self {
        match kind {
            "rain" => CueKind::Rain { pan, gain },
            "anvil" => CueKind::Anvil { pan, gain },
            "festival" => CueKind::Festival { pan, gain },
            "wonder" => CueKind::Wonder { pan, gain },
            "bell" => CueKind::Bell { pan, gain },
            "snowballs" => CueKind::Snowballs { pan, gain },
            "splash" => CueKind::Splash { pan, gain },
            "alarm" => CueKind::Alarm { pan, gain },
            "toll" => CueKind::Toll { pan, gain },
            "cheer" => CueKind::Cheer { pan, gain },
            "chant" => CueKind::Chant { pan, gain },
            "coins" => CueKind::Coins { pan, gain },
            "quill" => CueKind::Quill { pan, gain },
            "door" => CueKind::Door { pan, gain },
            "cart_arrive" => CueKind::CartArrive { pan, gain },
            "gate" => CueKind::Gate { pan, gain },
            _ => CueKind::Mood { pan, gain },
        }
    }

    /// The cue's own numbers - where in the stereo field it sits and
    /// how loud the sim called it. The palette's `VoiceSpec` turns the
    /// kind into a voice.
    fn pos(self) -> (f32, f32) {
        match self {
            CueKind::Rain { pan, gain } | CueKind::Anvil { pan, gain }
            | CueKind::Festival { pan, gain } | CueKind::Wonder { pan, gain }
            | CueKind::Bell { pan, gain } | CueKind::Snowballs { pan, gain }
            | CueKind::Splash { pan, gain } | CueKind::Alarm { pan, gain }
            | CueKind::Toll { pan, gain } | CueKind::Cheer { pan, gain }
            | CueKind::Chant { pan, gain } | CueKind::Coins { pan, gain }
            | CueKind::Quill { pan, gain } | CueKind::Door { pan, gain }
            | CueKind::CartArrive { pan, gain } | CueKind::Gate { pan, gain }
            | CueKind::Mood { pan, gain } => (pan, gain),
        }
    }
}

/// One frame's audio intent, pulled off the public model: which cues
/// fired, which emitters the listener can hear (A4), and the world's
/// seed.
#[derive(Clone, Debug)]
pub struct AudioFrame {
    pub cues: Vec<CueKind>,
    pub emitters: Vec<Emitter>,
    pub seed: u64,
}

impl AudioFrame {
    /// The cues the model reports (`sfx_v`: kind, tile x/y, intensity)
    /// plus the soundscape scan at the model's own ear. Pan comes off
    /// the map x like `record.rs` did, remapped to the engine's -1..1;
    /// y is kept for a later distance term.
    pub fn from_model(model: &UiModel) -> Self {
        Self::from_model_with(model, &Listener::from_model(model))
    }

    /// The same intent at a caller's own ear - a client with a real
    /// camera (or a scenario's `Look` op) passes its `Listener` in.
    pub fn from_model_with(model: &UiModel, listener: &Listener) -> Self {
        AudioFrame {
            cues: model.sfx_v.iter()
                .map(|(kind, x, _y, intensity)| {
                    let pan = (*x as f32 / MAP_TILES as f32).clamp(0.0, 1.0) * 2.0 - 1.0;
                    CueKind::from_name(kind, pan, *intensity)
                })
                .collect(),
            emitters: crate::soundscape::scan(model, listener),
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

/// What `AudioSettings` resolves to on the mixer (#1505): the five
/// bus gains already square-law'd, the mono and night switches, and
/// whether an unfocused window should fade out. `Copy` so the live
/// host can hand the device half a fresh copy each callback.
#[derive(Clone, Copy, Debug)]
struct Mix {
    gains: [f32; N_BUS],
    mono: bool,
    night: bool,
    bg_mute: bool,
}
const N_BUS: usize = 5;

/// The mix an unconfigured host runs: every bus at unity, no extras -
/// the historical behaviour for callers that never apply settings.
impl Default for Mix {
    fn default() -> Self {
        Mix { gains: [1.0; N_BUS], mono: false, night: false, bg_mute: false }
    }
}

/// The background fade's travel: 300 ms window-blur to silence and
/// back, from the settings spec.
const FOCUS_FADE_S: f32 = 0.3;

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
    /// renders them. `muted`, `mix` and `focused` are shared with the
    /// device half, `pending_bank` hands it a sample bank once set
    Live { frames: Arc<Mutex<VecDeque<AudioFrame>>>, muted: Arc<AtomicBool>,
        mix: Arc<Mutex<Mix>>, focused: Arc<AtomicBool>,
        pending_bank: Arc<Mutex<Option<crate::sampler::SampleBank>>> },
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
    /// The resolved settings mix (`apply_settings` keeps it current)
    mix: Mix,
    /// M's choice - `set_muted` keeps it next to the mix so applying a
    /// fresh mix doesn't unmute by accident
    muted: bool,
    /// Whether the window has focus (the offline host's copy; the live
    /// host's lives in the link)
    focused: bool,
    /// Where the background fade currently stands, 1 sounding to 0 gone
    focus_gain: f32,
    /// A4's ambience mixer: beds, one-shot schedules and the duck
    ambience: Ambience,
    /// The latest pushed frame's emitters and seed, and a count of
    /// frames pushed - the ambience update's arguments
    cur_emitters: Vec<Emitter>,
    cur_seed: u64,
    frames_in: u64,
    /// The last frame count the ambience update ran on - a second
    /// render block on the same pushed frame mustn't re-draw its
    /// Poisson fires
    amb_frame: u64,
    link: Link,
}

/// The device half of a live host: the sink calls `pump` in its audio
/// callback - it drains the frame queue and renders.
pub struct LivePump {
    core: AudioHost,
    frames: Arc<Mutex<VecDeque<AudioFrame>>>,
    muted: Arc<AtomicBool>,
    mix: Arc<Mutex<Mix>>,
    focused: Arc<AtomicBool>,
    pending_bank: Arc<Mutex<Option<crate::sampler::SampleBank>>>,
}

impl LivePump {
    pub fn pump(&mut self, out: &mut [f32]) {
        if let Some(bank) = self.pending_bank.lock().unwrap().take() {
            self.core.set_bank(bank);
        }
        {
            let mut q = self.frames.lock().unwrap();
            for f in q.drain(..) { self.core.push_frame(&f); }
        }
        self.core.mix = *self.mix.lock().unwrap();
        self.core.focused = self.focused.load(Ordering::Relaxed);
        self.core.apply_mix();
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
            mix: Mix::default(),
            muted: false,
            focused: true,
            focus_gain: 1.0,
            ambience: Ambience::new(),
            cur_emitters: vec![],
            cur_seed: 0,
            frames_in: 0, amb_frame: 0,
            link: Link::Offline { engine: Engine::new(sample_rate, 0) },
        }
    }

    /// The fallback every method no-ops on.
    pub fn silent() -> Self {
        AudioHost { sr: 44100, pending: vec![], carry: 0.0,
            voices_peak: 0, mix: Mix::default(), muted: false,
            focused: true, focus_gain: 1.0, ambience: Ambience::new(),
            cur_emitters: vec![], cur_seed: 0, frames_in: 0, amb_frame: 0,
            link: Link::Silent }
    }

    /// A live host: `sink` gets the render pump (a `LivePump` over the
    /// frame queue) and starts the device; failure means a warning line
    /// and a silent host, never a crash. The engine renders at the
    /// sink's own rate, not the requested one.
    pub fn start(settings: &HostSettings, sink: &mut dyn AudioSink) -> AudioHost {
        let frames = Arc::new(Mutex::new(VecDeque::with_capacity(FRAME_QUEUE)));
        let muted = Arc::new(AtomicBool::new(false));
        let mix = Arc::new(Mutex::new(Mix::default()));
        let focused = Arc::new(AtomicBool::new(true));
        let pending_bank = Arc::new(Mutex::new(None));
        let mut pump = LivePump {
            core: AudioHost::offline(sink.sample_rate()),
            frames: frames.clone(),
            muted: muted.clone(),
            mix: mix.clone(),
            focused: focused.clone(),
            pending_bank: pending_bank.clone(),
        };
        match sink.open(Box::new(move |out: &mut [f32]| pump.pump(out))) {
            Ok(()) => AudioHost {
                sr: settings.sample_rate,
                pending: vec![],
                carry: 0.0,
                voices_peak: 0,
                mix: Mix::default(),
                muted: false,
                focused: true,
                focus_gain: 1.0,
                ambience: Ambience::new(),
                cur_emitters: vec![], cur_seed: 0, frames_in: 0, amb_frame: 0,
                link: Link::Live { frames, muted, mix, focused, pending_bank },
            },
            Err(e) => {
                eprintln!("audio: couldn't open the output device ({e}) - running silent");
                AudioHost::silent()
            }
        }
    }

    /// Whether this host is the no-op fallback.
    pub fn is_silent(&self) -> bool { matches!(self.link, Link::Silent) }

    /// Hand the engine its decoded assets (A4's ambience batch). On a
    /// live host the bank crosses to the device half on its next
    /// callback; on a silent host it drops.
    pub fn set_bank(&mut self, bank: crate::sampler::SampleBank) {
        match &mut self.link {
            Link::Offline { engine } => engine.set_bank(bank),
            Link::Live { pending_bank, .. } => *pending_bank.lock().unwrap() = Some(bank),
            Link::Silent => {}
        }
    }

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
            Link::Offline { .. } => {
                self.pending.extend_from_slice(&frame.cues);
                self.cur_emitters.clone_from(&frame.emitters);
                self.cur_seed = frame.seed;
                self.frames_in += 1;
            }
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
    /// A later `apply_settings` keeps the mute.
    pub fn set_muted(&mut self, on: bool) {
        self.muted = on;
        match &mut self.link {
            Link::Live { muted, .. } => muted.store(on, Ordering::Relaxed),
            Link::Offline { .. } => self.apply_mix(),
            Link::Silent => {}
        }
    }

    /// Apply a persisted `AudioSettings` (#1505): sliders land as
    /// square-law bus gains, mono and night-mode flip their engine
    /// switches, `mute_in_background` arms the focus fade. On a live
    /// host the device half picks the new mix up on its next callback.
    pub fn apply_settings(&mut self, s: &AudioSettings) {
        self.mix = Mix {
            gains: [AudioSettings::bus_amp(s.master), AudioSettings::bus_amp(s.music),
                AudioSettings::bus_amp(s.ambience), AudioSettings::bus_amp(s.effects),
                AudioSettings::bus_amp(s.interface)],
            mono: s.mono, night: s.night_mode, bg_mute: s.mute_in_background,
        };
        match &mut self.link {
            Link::Offline { .. } => self.apply_mix(),
            Link::Live { mix, .. } => *mix.lock().unwrap() = self.mix,
            Link::Silent => {}
        }
    }

    /// Whether the window has focus: with `mute_in_background` the mix
    /// fades to silence over 300 ms when it hasn't, and back when it
    /// has. The client calls this every frame from `window.is_active()`.
    pub fn set_focused(&mut self, on: bool) {
        match &mut self.link {
            Link::Offline { .. } => self.focused = on,
            Link::Live { focused, .. } => focused.store(on, Ordering::Relaxed),
            Link::Silent => {}
        }
    }

    /// Push the resolved mix into the offline engine. Master honours
    /// the mute switch on top of the settings gain, so an apply can't
    /// undo an M.
    fn apply_mix(&mut self) {
        if let Link::Offline { engine } = &mut self.link {
            engine.set_bus_gain(Bus::Master,
                if self.muted { 0.0 } else { self.mix.gains[Bus::Master as usize] });
            engine.set_bus_gain(Bus::Music, self.mix.gains[Bus::Music as usize]);
            engine.set_bus_gain(Bus::Ambience, self.mix.gains[Bus::Ambience as usize]);
            engine.set_bus_gain(Bus::Sfx, self.mix.gains[Bus::Sfx as usize]);
            engine.set_bus_gain(Bus::Ui, self.mix.gains[Bus::Ui as usize]);
            engine.set_mono(self.mix.mono);
            engine.set_night(self.mix.night);
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
    /// enter at the block's first sample on the Sfx bus, and the latest
    /// pushed emitters drive the ambience mixer (A4). Live and silent
    /// hosts render zeros - the device side owns the engine.
    pub fn render_block(&mut self, out: &mut [f32]) {
        match &mut self.link {
            Link::Offline { engine } => {
                for c in self.pending.drain(..) {
                    let (pan, gain) = c.pos();
                    let spec = crate::palette::voice_for(c);
                    // a ledgered patch takes the voice when the bank has
                    // it; the synth stands in until a name's banked
                    let source = spec.sample
                        .and_then(|n| engine.bank().patch_id(n))
                        .map(|id| Source::Sample { id, midi: 60 })
                        .unwrap_or(Source::Synth(spec.synth));
                    engine.play(source, spec.bus,
                        VoiceParams { gain: gain * spec.gain,
                            pan: if spec.pan_from_pos { pan } else { 0.0 },
                            ..Default::default() });
                }
                if self.frames_in != self.amb_frame {
                    self.amb_frame = self.frames_in;
                    self.ambience.update(&self.cur_emitters, self.cur_seed,
                        self.frames_in, engine);
                }
                engine.render(out);
                self.voices_peak = self.voices_peak.max(engine.live_voices());
                // the background fade (#1505): unfocused with
                // mute_in_background ramps the block toward silence
                // over 300 ms; focused ramps back. Parked at silence
                // it stays exact zeros.
                let want = if self.mix.bg_mute && !self.focused { 0.0 } else { 1.0 };
                if want < 1.0 || self.focus_gain < 1.0 {
                    let step = 1.0 / (self.sr as f32 * FOCUS_FADE_S);
                    for i in (0..out.len()).step_by(2) {
                        self.focus_gain = if self.focus_gain < want {
                            (self.focus_gain + step).min(want)
                        } else if self.focus_gain > want {
                            (self.focus_gain - step).max(want)
                        } else { want };
                        out[i] *= self.focus_gain;
                        out[i + 1] *= self.focus_gain;
                    }
                }
            }
            _ => out.fill(0.0),
        }
    }

    /// The most voices the engine held at once, for `metrics.json`.
    pub fn voices_peak(&self) -> usize { self.voices_peak }
}
