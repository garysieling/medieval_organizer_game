//! The ambience mixer (A4/#1500): plays `scan`'s emitters on the
//! engine. One update per rendered frame (60 fps): beds hold one
//! looping voice per kind whose gain eases toward `weight * gain`
//! over ~2 s, one-shot emitters fire on a Poisson draw against
//! `rate_hz` honouring `min_gap_s`, and a fresh signal ducks the beds
//! by 2 dB. Wind and rain have no sampled asset - they play
//! `synth::BedKind` noise beds, per the design. Every choice draws
//! from `Rng::new(seed ^ frame_hash)` so the same scene renders the
//! same sound.
use crate::engine::{Bus, Engine, Source, VoiceId, VoiceParams};
use crate::rng::Rng;
use crate::soundscape::{Emitter, EmitterKind, Tier};
use crate::synth::BedKind;
use std::collections::BTreeMap;

/// One rendered frame in seconds: `update` runs at the game's 60 fps
const DT: f32 = 1.0 / 60.0;
/// Bed gain's ease constant: a fresh bed swells over ~2 s
const EASE: f32 = 2.0;
/// The signal duck: beds drop 2 dB under a one-shot, releasing over
/// 500 ms ("10 ms attack" is under a frame - the dip lands whole)
const DUCK: f32 = 0.794_328_2; // 10^(-2/20)
const DUCK_STEP: f32 = (1.0 - DUCK) / (0.5 / DT);
/// The design's one-shot voice budget: at most 2 per kind at once
const MAX_SHOT_VOICES: usize = 2;
/// ±3% pitch and ±2 dB gain on a fired one-shot
const PITCH_JITTER: f32 = 0.03;
const GAIN_JITTER_DB: f32 = 2.0;
/// One-shot headroom: the fetch pipeline loudnorms shots to -14 LUFS
/// (-3 dBFS peak), hotter than the -20 beds - a -5 dB trim keeps a
/// busy scene's sum under the mix bar
const SHOT_LEVEL: f32 = 0.562;
/// Bed headroom: a scene runs several beds at once - the eased target
/// is `weight * gain` scaled so the sum sits inside -20±3 LUFS.
/// Calibrated on the real A4b batch: recorded beds carry dynamics
/// (quiet stretches gate out of the LUFS sum) where the synth/fake
/// beds were steady-state, so the real batch needs ~+1.5 dB on 0.7.
const BED_LEVEL: f32 = 0.84;
/// A bed this quiet is faded out - `stop` frees its slot
const GONE: f32 = 0.001;

/// The sampled beds of the first asset batch, ledger ids = patch names
const fn bed_asset(kind: EmitterKind) -> Option<&'static str> {
    match kind {
        EmitterKind::Songbirds => Some("birds"),
        EmitterKind::Insects => Some("insects"),
        EmitterKind::Bees => Some("bees"),
        EmitterKind::River => Some("river"),
        EmitterKind::VillageMurmur => Some("village_murmur"),
        EmitterKind::Market => Some("market_crowd"),
        EmitterKind::Fire => Some("fire_crackle"),
        _ => None,
    }
}

/// The own-synth beds: wind and rain per the design, and the standing
/// stones' wind-hum soundmark rides the wind bed's timbre
const fn synth_bed(kind: EmitterKind) -> Option<BedKind> {
    match kind {
        EmitterKind::Wind | EmitterKind::StandingStones => Some(BedKind::Wind),
        EmitterKind::Rain => Some(BedKind::Rain),
        EmitterKind::March => Some(BedKind::March),
        _ => None,
    }
}

/// One-shot variants per kind, ledger ids. Round-robin order, never
/// an immediate repeat once a kind has more than one variant.
const fn shot_assets(kind: EmitterKind) -> &'static [&'static str] {
    match kind {
        EmitterKind::ChurchBell => &["church_bell"],
        EmitterKind::Smithy | EmitterKind::Work => &["anvil_hit"],
        EmitterKind::Fold => &["sheep_bleat"],
        EmitterKind::Homesteads => &["chicken", "dog_bark", "door_creak"],
        EmitterKind::Cart => &["cart_wheel", "hoofbeat"],
        EmitterKind::Gate => &["gate_creak"],
        EmitterKind::Graveyard => &["crow"],
        // the scan's own wording: while something burns the parish
        // bell tolls the alarm
        EmitterKind::Fire => &["church_bell"],
        _ => &[],
    }
}

/// The emitter kinds that modify the whole ambience rather than voice
/// themselves: snow hush caps every bed's low-pass (birds are already
/// suppressed by the scan); haar caps lower and deepens the reverb.
fn master_lowpass(e: &Emitter) -> Option<f32> {
    match e.kind {
        EmitterKind::SnowHush | EmitterKind::Haar => Some(e.lowpass_hz),
        _ => None,
    }
}

/// Splitmix64-style mix of the frame index - `seed ^ frame_hash` keeps
/// adjacent frames' draws decorrelated while staying exact.
fn frame_hash(frame: u64) -> u64 {
    let mut z = frame.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A live bed voice and where its gain is heading
struct Bed {
    voice: VoiceId,
    /// The eased gain, toward `want` (duck applies on top)
    cur: f32,
    want: f32,
    params: VoiceParams,
}

/// Per-emitter one-shot state: the round-robin variant cursor, the
/// last frame it fired (for `min_gap_s`) and its live voices
struct Shots {
    rr: usize,
    /// The frame the last fire happened on; `None` until the first
    last: Option<u64>,
    /// The variant index the last fire played - the tests' window on
    /// the round-robin
    last_idx: usize,
    voices: Vec<VoiceId>,
}

/// The mixer. Held by the host (and by tests) next to an `Engine`.
pub struct Ambience {
    beds: BTreeMap<EmitterKind, Bed>,
    shots: BTreeMap<(EmitterKind, i32, i32), Shots>,
    /// Bed-bus duck gain under fresh one-shots: 1 sounding, `DUCK` dipped
    duck: f32,
}

impl Default for Ambience {
    fn default() -> Self { Self::new() }
}

impl Ambience {
    pub fn new() -> Self {
        Ambience { beds: BTreeMap::new(), shots: BTreeMap::new(), duck: 1.0 }
    }

    /// The source a bed-path emitter voices: its sampled asset when the
    /// bank knows it, else its own-synth bed, else nothing (a kind the
    /// first batch doesn't cover - tavern, chant, mill, castle, cave,
    /// boats - is quiet by design until its ticket lands).
    fn bed_source(&self, kind: EmitterKind, engine: &Engine) -> Option<Source> {
        if let Some(name) = bed_asset(kind) {
            if let Some(id) = engine.bank().patch_id(name) {
                return Some(Source::Bed { id });
            }
        }
        synth_bed(kind).map(Source::SynthBed)
    }

    /// One frame of the soundscape. `frame` counts rendered frames and
    /// feeds the per-frame draw seed; calling with the same arguments
    /// makes the same moves.
    pub fn update(&mut self, emitters: &[Emitter], seed: u64, frame: u64,
                engine: &mut Engine) {
        let mut rng = Rng::new(seed ^ frame_hash(frame));
        // the hush modifiers: lowest cap wins, haar's reverb floor rides
        let mut lp_cap = f32::INFINITY;
        let mut rv_floor = 0.0f32;
        for e in emitters {
            if let Some(lp) = master_lowpass(e) {
                lp_cap = lp_cap.min(lp);
                rv_floor = rv_floor.max(e.reverb);
            }
        }
        // the duck's 500 ms release runs every frame; fresh one-shots
        // dip it again below
        self.duck = (self.duck + DUCK_STEP).min(1.0);

        // beds: voice each bed-path emitter, ease toward weight*gain
        let mut present = BTreeMap::new();
        for e in emitters {
            if e.tier != Tier::Bed || master_lowpass(e).is_some() { continue; }
            let target = e.weight * e.gain * BED_LEVEL;
            // a kind's several bed emitters keep the loudest voice
            present.entry(e.kind)
                .and_modify(|(w, p, lp, rv): &mut (f32, f32, f32, f32)| {
                    if target > *w {
                        *w = target; *p = e.pan;
                        *lp = e.lowpass_hz; *rv = e.reverb;
                    }
                })
                .or_insert((target, e.pan, e.lowpass_hz, e.reverb));
        }
        let kinds: Vec<EmitterKind> =
            self.beds.keys().chain(present.keys()).copied().collect();
        for kind in kinds {
            let want = present.get(&kind).copied().unwrap_or((0.0, 0.0, 16000.0, 0.0));
            match self.beds.get_mut(&kind) {
                Some(b) => {
                    b.want = want.0;
                    b.cur += (b.want - b.cur) * (DT / EASE).min(1.0);
                    b.params.gain = b.cur * self.duck;
                    b.params.pan = want.1;
                    b.params.lowpass_hz = want.2.min(lp_cap);
                    b.params.reverb_send = want.3.max(rv_floor);
                    engine.set_params(b.voice, b.params);
                    if !engine.alive(b.voice) || (b.cur < GONE && want.0 == 0.0) {
                        engine.stop(b.voice);
                        self.beds.remove(&kind);
                    }
                }
                None if want.0 > 0.0 => {
                    if let Some(src) = self.bed_source(kind, engine) {
                        let params = VoiceParams {
                            gain: 0.0, pan: want.1, pitch: 1.0,
                            lowpass_hz: want.2.min(lp_cap),
                            reverb_send: want.3.max(rv_floor),
                        };
                        let voice = engine.play(src, Bus::Ambience, params);
                        self.beds.insert(kind,
                            Bed { voice, cur: 0.0, want: want.0, params });
                    }
                }
                _ => {}
            }
        }

        // one-shots: Poisson at rate_hz against min_gap_s, round-robin
        // variants, pitch/gain jitter, two live voices per kind
        for e in emitters {
            if e.rate_hz <= 0.0 || e.tier == Tier::Bed { continue; }
            let variants = shot_assets(e.kind);
            if variants.is_empty() { continue; }
            let (qx, qy) = e.pos.map(|(x, y)| (x.round() as i32, y.round() as i32))
                .unwrap_or((i32::MIN, i32::MIN));
            let st = self.shots.entry((e.kind, qx, qy))
                .or_insert(Shots { rr: 0, last: None, last_idx: 0, voices: vec![] });
            st.voices.retain(|&v| engine.alive(v));
            let draw = rng.next_f32();
            let gap_ok = st.last.map_or(true, |l| (frame - l) as f32 * DT >= e.min_gap_s);
            if draw >= e.rate_hz * DT || !gap_ok { continue; }
            if st.voices.len() >= MAX_SHOT_VOICES { continue; }
            st.last = Some(frame);
            let name = variants[st.rr];
            st.last_idx = st.rr;
            st.rr = (st.rr + 1) % variants.len();
            let pitch = 1.0 + rng.range(-PITCH_JITTER, PITCH_JITTER);
            let gain = e.gain * SHOT_LEVEL
                * 10f32.powf(rng.range(-GAIN_JITTER_DB, GAIN_JITTER_DB) / 20.0);
            if let Some(id) = engine.bank().patch_id(name) {
                let voice = engine.play(Source::Sample { id, midi: 60 },
                    Bus::Ambience,
                    VoiceParams { gain, pan: e.pan, pitch,
                        lowpass_hz: e.lowpass_hz.min(lp_cap),
                        reverb_send: e.reverb.max(rv_floor) });
                st.voices.push(voice);
                self.duck = DUCK;
            }
        }
        // stale one-shot states keep the map from growing: an emitter
        // that left earshot (or went quiet) drops out once its voices
        // have rung out
        let live: std::collections::BTreeSet<(EmitterKind, i32, i32)> = emitters.iter()
            .filter(|e| e.rate_hz > 0.0 && e.tier != Tier::Bed)
            .map(|e| {
                let (qx, qy) = e.pos.map(|(x, y)| (x.round() as i32, y.round() as i32))
                    .unwrap_or((i32::MIN, i32::MIN));
                (e.kind, qx, qy)
            })
            .collect();
        self.shots.retain(|k, st| live.contains(k) || !st.voices.is_empty());
    }

    /// Voices the mixer is holding, for tests and the voice budget
    pub fn held_voices(&self) -> usize {
        self.beds.len() + self.shots.values().map(|s| s.voices.len()).sum::<usize>()
    }

    #[doc(hidden)]
    /// Test window: the signal duck's current gain (1 sounding, ~0.794 dipped)
    pub fn probe_duck(&self) -> f32 { self.duck }

    #[doc(hidden)]
    /// Test window: a bed's eased gain, before the duck applies
    pub fn probe_bed_gain(&self, kind: EmitterKind) -> Option<f32> {
        self.beds.get(&kind).map(|b| b.cur)
    }

    #[doc(hidden)]
    /// Test window: the gain actually applied to a bed voice this
    /// frame (eased gain x duck)
    pub fn probe_bed_applied(&self, kind: EmitterKind) -> Option<f32> {
        self.beds.get(&kind).map(|b| b.params.gain)
    }

    #[doc(hidden)]
    /// Test window: the last variant index a kind's emitter fired
    pub fn probe_shot_variant(&self, kind: EmitterKind) -> Option<usize> {
        self.shots.iter().find(|((k, _, _), _)| *k == kind)
            .map(|(_, s)| s.last_idx)
    }
}
