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
use game2_core::infra::perf;
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
    /// The ear the emitters were scanned from (M5's trace needs it).
    pub listener: Listener,
    /// The score's intent for the frame (M5): `push` fills it from the
    /// host's `Director`; frames built for scene replay carry the
    /// ambient default.
    pub intent: crate::director::Intent,
}

impl Default for AudioFrame {
    fn default() -> Self {
        AudioFrame { cues: vec![], emitters: vec![], seed: 0,
            listener: Listener::at(0.0, 0.0, 1.0),
            intent: crate::director::Intent::default() }
    }
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
        let emitters = { let _s = perf::scope("audio.scan");
            crate::soundscape::scan(model, listener) };
        AudioFrame {
            cues: model.sfx_v.iter()
                .map(|(kind, x, _y, intensity)| {
                    let pan = (*x as f32 / MAP_TILES as f32).clamp(0.0, 1.0) * 2.0 - 1.0;
                    CueKind::from_name(kind, pan, *intensity)
                })
                .collect(),
            emitters,
            seed: model.seed,
            listener: listener.clone(),
            intent: crate::director::Intent::default(),
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
    /// M5: the director, seeded on the first pushed model. Lives on
    /// the game-side host (and the offline one); the device half only
    /// ever sees the `Intent`s frames carry.
    director: Option<crate::director::Director>,
    dir_seed: u64,
    /// The presence budget the settings last applied - a director
    /// seeded after the setting lands still hears it.
    presence: f32,
    /// The score state: kept in its own struct so the render loop can
    /// borrow it beside the `engine` living inside `link`.
    score: Score,
    link: Link,
}

/// The score's runtime state (M5): the intent the last frame carried,
/// M4's composer writing phrases on boundaries, the song clock the
/// schedule runs on, and the ledger the dump drains.
struct Score {
    intent: crate::director::Intent,
    composer: crate::compose::Composer,
    /// Rendered-samples clock the score schedules against
    song_pos: u64,
    /// The sample the next phrase starts on
    phrase_next: u64,
    /// Device samples the current phrase spans
    phrase_samples: u64,
    /// Composed phrases so far - `bar_index` = `phrase_idx * 8`
    phrase_idx: u64,
    /// Notes waiting on their onset sample, earliest first
    pending_notes: std::collections::VecDeque<(u64, crate::compose::NoteEvent)>,
    /// Stingers waiting on their downbeat: an intent carries each for
    /// a single frame, so they queue here and fire once.
    stinger_queue: Vec<crate::director::Stinger>,
    /// Whether the trace and note ledger record: the dump host turns
    /// it on, the live pump leaves it off so a day's session doesn't
    /// grow a history nobody drains.
    capture: bool,
    /// Per-bar records for `sound_dump`'s `.trace.json`
    traces: Vec<crate::trace::BarTrace>,
    /// Every note fired, for the dump's piano roll
    note_log: Vec<(u64, crate::compose::NoteEvent)>,
}

impl Score {
    fn new(capture: bool) -> Self {
        Score {
            intent: crate::director::Intent::default(),
            composer: crate::compose::Composer::new(),
            song_pos: 0, phrase_next: 0, phrase_samples: 0, phrase_idx: 0,
            pending_notes: std::collections::VecDeque::new(),
            stinger_queue: vec![],
            capture,
            traces: vec![], note_log: vec![],
        }
    }

    /// The stinger's voice and pitch (the design's palette: bells for
    /// wins, a low drum for a loss, a half-cadence for a stall).
    fn stinger_patch(s: crate::director::Stinger) -> (crate::instruments::PatchId, u8) {
        use crate::director::Stinger;
        use crate::instruments::PatchId;
        match s {
            Stinger::BellsPeal | Stinger::CauseClose => (PatchId::TubularBells, 72),
            Stinger::HalfCadence => (PatchId::HandBells, 60),
            Stinger::LowDrum => (PatchId::BassDrum, 40),
            Stinger::Alarm => (PatchId::TubularBells, 84),
        }
    }

    /// Queue-up stingers play once each, on the Music bus, as soon as
    /// the block that carries them renders - the director already
    /// timed the frame to a downbeat.
    fn fire_stingers(&mut self, engine: &mut Engine, sr: u32) {
        for s in std::mem::take(&mut self.stinger_queue) {
            let (patch, midi) = Self::stinger_patch(s);
            let src = crate::instruments::source(patch, engine.bank(), midi);
            engine.play(src, Bus::Music,
                VoiceParams { gain: 0.3, ..Default::default() });
            if self.capture {
                let phrase_start = self.phrase_next.saturating_sub(self.phrase_samples);
                self.note_log.push((self.song_pos, crate::compose::NoteEvent {
                    at_sample: self.song_pos.saturating_sub(phrase_start)
                        * crate::wav::SAMPLE_RATE as u64 / sr as u64,
                    dur_samples: crate::wav::SAMPLE_RATE / 4,
                    voice: crate::compose::Voice::Color,
                    midi: midi as f32, vel: 0.9, patch,
                }));
            }
        }
    }

    /// The intent's duck on the two buses it owns, on top of the
    /// settings mix.
    fn apply_duck(&self, mix: &Mix, engine: &mut Engine) {
        engine.set_bus_gain(Bus::Music,
            mix.gains[Bus::Music as usize] * db_amp(self.intent.duck.music_db));
        engine.set_bus_gain(Bus::Ambience,
            mix.gains[Bus::Ambience as usize] * db_amp(self.intent.duck.ambience_db));
    }

    /// Notes whose onset the song clock has reached enter the engine on
    /// the Music bus; a reached phrase boundary composes the next one.
    /// The score plays well under the bus (the mix bar sits the scene
    /// near -20 LUFS; 8 voices at velocity would eat the whole bar).
    fn fire_due(&mut self, engine: &mut Engine, seed: u64,
        emitters: &[Emitter], sr: u32) {
        // the score's per-block work: firing due notes, and the phrase
        // compose when a boundary lands mid-block (the rare spike)
        let _s = perf::scope("audio.compose");
        const NOTE_GAIN: f32 = 0.15;
        while let Some((t, _)) = self.pending_notes.front() {
            if *t > self.song_pos { break; }
            let (_, n) = self.pending_notes.pop_front().unwrap();
            let src = crate::instruments::source(n.patch, engine.bank(), n.midi as u8);
            engine.play(src, Bus::Music,
                VoiceParams { gain: n.vel * NOTE_GAIN, ..Default::default() });
            if self.capture {
                self.note_log.push((self.song_pos, n));
            }
        }
        if self.song_pos >= self.phrase_next {
            self.compose_next(seed, emitters, sr);
        }
    }

    /// Write the next 8 bars from the current intent: the composer's
    /// phrase becomes scheduled notes and its bars join the trace.
    fn compose_next(&mut self, seed: u64,
        emitters: &[Emitter], sr: u32) {
        use crate::compose::{PhraseSpec, Voice};
        let i = self.intent.clone();
        let meter = i.meter;
        let phrase_s = crate::director::BARS_PER_PHRASE as f64
            * meter.num as f64 * (4.0 / meter.den as f64)
            * 60.0 / i.tempo_bpm.max(30.0);
        let phrase_samples = (phrase_s * sr as f64) as u64;
        let spec = PhraseSpec {
            seed,
            bar_index: self.phrase_idx * crate::director::BARS_PER_PHRASE,
            tonic: i.tonic, mode: i.mode, section: i.section, meter,
            tempo_bpm: i.tempo_bpm, density: i.density, motif: i.motif,
            resolved: i.resolved,
        };
        let stingers = i.stinger.map(|s| vec![format!("{s:?}")]).unwrap_or_default();
        let (mut phrase, bars) = crate::trace::trace_phrase(
            &mut self.composer, &spec, &i.mood,
            self.phrase_idx * crate::director::BARS_PER_PHRASE,
            self.song_pos as f64 / sr as f64,
            emitters, stingers);
        // the composer writes at `wav::SAMPLE_RATE`; the render clock
        // runs at the device's rate, so offsets and cutoffs convert
        let to_dev = |at: u64| at * sr as u64 / crate::wav::SAMPLE_RATE as u64;
        // the build's one-bar rest: the last bar before the event
        // drops its melody so the arrival lands clean
        if i.rest_bar {
            let cut = phrase_samples - phrase_samples / crate::director::BARS_PER_PHRASE;
            phrase.notes.retain(|n| !(n.voice == Voice::Melody && to_dev(n.at_sample) >= cut));
        }
        // notes come out voice by voice - the firing queue wants them
        // earliest-onset-first or a late drone parks an early melody
        phrase.notes.sort_by_key(|n| n.at_sample);
        for n in phrase.notes {
            self.pending_notes.push_back((self.phrase_next + to_dev(n.at_sample), n));
        }
        if self.capture {
            self.traces.extend(bars);
        }
        self.phrase_samples = phrase_samples;
        self.phrase_next += phrase_samples;
        self.phrase_idx += 1;
    }
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
            director: None, dir_seed: u64::MAX, presence: 0.6,
            score: Score::new(true),
            link: Link::Offline { engine: Engine::new(sample_rate, 0) },
        }
    }

    /// The fallback every method no-ops on.
    pub fn silent() -> Self {
        AudioHost { sr: 44100, pending: vec![], carry: 0.0,
            voices_peak: 0, mix: Mix::default(), muted: false,
            focused: true, focus_gain: 1.0, ambience: Ambience::new(),
            cur_emitters: vec![], cur_seed: 0, frames_in: 0, amb_frame: 0,
            director: None, dir_seed: u64::MAX, presence: 0.6,
            score: Score::new(false),
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
        // the device-side score plays what frames ask for; only the
        // dump's offline host keeps the bar trace and note ledger
        pump.core.score.capture = false;
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
                director: None, dir_seed: u64::MAX, presence: 0.6,
                score: Score::new(false),
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

    /// The score's director, seeded by the first model and re-seeded if
    /// the world under us changes.
    fn director(&mut self, model: &UiModel) -> &mut crate::director::Director {
        if self.dir_seed != model.seed {
            let mut d = crate::director::Director::new(model.seed);
            d.set_presence(self.presence);
            self.director = Some(d);
            self.dir_seed = model.seed;
        }
        self.director.as_mut().unwrap()
    }

    /// A frame arrived: file its audio intent for the next block.
    /// `soundscape::scan` and `Director::update` run here, on the game
    /// thread - the audio side only composes what the frame asked for.
    pub fn push(&mut self, model: &UiModel) {
        let listener = Listener::from_model(model);
        self.push_with(model, &listener);
    }

    /// The same push at a caller's own ear (a scenario's `Look`).
    pub fn push_with(&mut self, model: &UiModel, listener: &Listener) {
        let intent = { let _s = perf::scope("audio.director");
            self.director(model).update(model) };
        let mut frame = AudioFrame::from_model_with(model, listener);
        frame.intent = intent;
        self.push_frame(&frame);
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
                // a stinger lives in its intent for this one frame -
                // queue it or the next intent would overwrite it
                if let Some(s) = frame.intent.stinger {
                    self.score.stinger_queue.push(s);
                }
                self.score.intent = frame.intent.clone();
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
        self.presence = s.presence.budget();
        if let Some(d) = &mut self.director {
            d.set_presence(self.presence);
        }
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
            engine.set_bus_gain(Bus::Music,
                self.mix.gains[Bus::Music as usize] * db_amp(self.score.intent.duck.music_db));
            engine.set_bus_gain(Bus::Ambience,
                self.mix.gains[Bus::Ambience as usize] * db_amp(self.score.intent.duck.ambience_db));
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
                    let _s = perf::scope("audio.ambience");
                    self.ambience.update(&self.cur_emitters, self.cur_seed,
                        self.frames_in, engine);
                }
                // the score (M5): the music and ambience buses take the
                // intent's duck on top of the settings mix, queued
                // stingers ring, then the block renders in chunks so
                // phrase boundaries and note onsets land on samples.
                self.score.apply_duck(&self.mix, engine);
                { let _s = perf::scope("audio.compose");
                    self.score.fire_stingers(engine, self.sr); }
                let mut done = 0usize;
                while done * 2 < out.len() {
                    let rem = (out.len() / 2 - done) as u64;
                    let next = self.score.pending_notes.front().map(|(t, _)| *t)
                        .unwrap_or(u64::MAX).min(self.score.phrase_next);
                    let gap = if next <= self.score.song_pos { 0 }
                        else if next == u64::MAX { rem }
                        else { (next - self.score.song_pos).min(rem) };
                    if gap == 0 {
                        self.score.fire_due(engine, self.cur_seed,
                            &self.cur_emitters, self.sr);
                        continue;
                    }
                    { let _s = perf::scope("audio.mix");
                        engine.render(&mut out[done * 2..(done + gap as usize) * 2]); }
                    self.score.song_pos += gap;
                    done += gap as usize;
                }
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

    /// The per-bar records the score has written so far (M5) -
    /// `sound_dump` drains them into `.trace.json`.
    pub fn drain_trace(&mut self) -> Vec<crate::trace::BarTrace> {
        std::mem::take(&mut self.score.traces)
    }

    /// Every note the score fired, in onset order - the dump's piano
    /// roll reads these.
    pub fn drain_notes(&mut self) -> Vec<(u64, crate::compose::NoteEvent)> {
        std::mem::take(&mut self.score.note_log)
    }

}

/// dB to amplitude, for the duck multipliers.
fn db_amp(db: f32) -> f32 { 10f32.powf(db / 20.0) }

#[cfg(test)]
mod tests {
    use super::*;

    /// The ledgers are diagnostics: `start()` clears the flag on the
    /// render core so a session doesn't grow a history nobody drains;
    /// only the offline dump host records.
    #[test]
    fn capture_gates_the_score_ledgers() {
        for capture in [false, true] {
            let mut s = Score::new(capture);
            let mut e = Engine::new(44100, 0);
            s.fire_due(&mut e, 7, &[], 44100); // composes phrase 0
            s.song_pos = 44_100;
            s.fire_due(&mut e, 7, &[], 44100); // fires the due notes
            s.stinger_queue.push(crate::director::Stinger::BellsPeal);
            s.fire_stingers(&mut e, 44100);
            assert_eq!(s.traces.is_empty(), !capture,
                "capture={capture}: the bar trace should follow the flag");
            assert_eq!(s.note_log.is_empty(), !capture,
                "capture={capture}: the note ledger should follow the flag");
        }
    }
}
