//! The mixer everything plays through: a fixed-size voice pool over
//! five buses, a reverb send per bus, and a soft limiter on Master.
//! `render` writes interleaved stereo f32 and is bit-identical for the
//! same calls - nothing here reads a clock or OS randomness.
use crate::rng::Rng;
use std::f32::consts::{FRAC_PI_4, TAU};

/// Where a voice's output mixes. Master is the final stage; a voice
/// played straight on Master skips the bus reverb.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bus { Master, Music, Ambience, Sfx, Ui }
const N_BUS: usize = 5;
const fn bi(b: Bus) -> usize { b as usize }

/// A playing or fading-out source.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoiceId(pub u32);

/// What a voice makes. The enum leaves room for the sampler variants
/// A2 adds; S3 adds the legacy cue synths.
#[derive(Clone, Copy, Debug)]
pub enum Source {
    Sine { hz: f32 },
    Noise,
    /// One of the game's cue voices - plays out its own envelope then
    /// retires itself
    Synth(crate::synth::SynthKind),
}

/// How a voice sounds: loudness, stereo place, speed, tone and how
/// much of it goes to its bus's reverb.
#[derive(Clone, Copy, Debug)]
pub struct VoiceParams {
    pub gain: f32,
    /// -1 hard left .. 1 hard right, equal-power.
    pub pan: f32,
    /// Rate multiplier over the source's own pitch.
    pub pitch: f32,
    /// One-pole lowpass corner; `f32::INFINITY` leaves the signal alone.
    pub lowpass_hz: f32,
    /// Extra copy sent to the bus reverb, 0..1.
    pub reverb_send: f32,
}
impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams { gain: 1.0, pan: 0.0, pitch: 1.0, lowpass_hz: f32::INFINITY, reverb_send: 0.0 }
    }
}

/// At most this many voices play at once; past it a `play` steals.
pub const MAX_VOICES: usize = 48;

struct Voice {
    id: u32,
    src: Source,
    bus: usize,
    params: VoiceParams,
    phase: f32,
    lp: f32,
    /// When this voice started, in rendered samples - the oldest-
    /// first tiebreak for stealing.
    born: u64,
    /// Anti-click attack ramp: samples of ramp still to go (the same
    /// 5 ms as the steal fade) - a voice entering mid-amplitude clicks
    /// exactly like one leaving.
    attack: u32,
}

/// A voice on its way out: the steal's 5 ms linear fade means the slot
/// frees without a click while the tail still decays.
struct Dying {
    v: Voice,
    left: u32,
    total: u32,
}

/// Fixed-feedback Schroeder comb (part of a per-bus reverb).
struct Comb { buf: Vec<f32>, i: usize, fb: f32, damp: f32, lp: f32 }
impl Comb {
    fn new(len: usize, fb: f32) -> Self { Comb { buf: vec![0.0; len], i: 0, fb, damp: 0.2, lp: 0.0 } }
    fn tick(&mut self, x: f32) -> f32 {
        let y = self.buf[self.i];
        self.lp = y + self.damp * (self.lp - y);
        self.buf[self.i] = x + self.lp * self.fb;
        self.i = (self.i + 1) % self.buf.len();
        y
    }
}
/// Schroeder allpass: diffuses without colouring the tail.
struct Allpass { buf: Vec<f32>, i: usize }
impl Allpass {
    fn new(len: usize) -> Self { Allpass { buf: vec![0.0; len], i: 0 } }
    fn tick(&mut self, x: f32) -> f32 {
        let d = self.buf[self.i];
        let y = d - 0.5 * x;
        self.buf[self.i] = x + 0.5 * d;
        self.i = (self.i + 1) % self.buf.len();
        y
    }
}
/// A small deterministic stereo reverb: a comb bank per channel into
/// an allpass each, with fixed seed-free delay lengths scaled from the
/// 48 kHz reference the design sets for offline renders.
struct Reverb { l: Vec<Comb>, r: Vec<Comb>, la: Allpass, ra: Allpass }
impl Reverb {
    fn new(sr: u32) -> Self {
        let s = |n: usize| (n * sr as usize) / 48000;
        let mk = |ns: &[usize]| ns.iter().map(|&n| Comb::new(s(n).max(1), 0.78)).collect();
        Reverb {
            l: mk(&[1116, 1188, 1277, 1356]), r: mk(&[1139, 1211, 1300, 1379]),
            la: Allpass::new(s(556).max(1)), ra: Allpass::new(s(579).max(1)),
        }
    }
    fn tick(&mut self, l: f32, r: f32) -> (f32, f32) {
        let wl = self.l.iter_mut().fold(0.0, |a, c| a + c.tick(l)) * 0.25;
        let wr = self.r.iter_mut().fold(0.0, |a, c| a + c.tick(r)) * 0.25;
        (self.la.tick(wl), self.ra.tick(wr))
    }
}

/// The soft clipper on Master: transparent under the knee, then a tanh
/// shoulder that asymptotes at the -1 dBFS ceiling and can never pass
/// it. A plain `tanh` would bend quiet material and fail the RMS test.
const CEILING: f32 = 0.891250938; // 10^(-1/20)
const KNEE: f32 = 0.7;
fn limit(x: f32) -> f32 {
    let a = x.abs();
    if a <= KNEE { return x; }
    let t = ((a - KNEE) / (CEILING - KNEE)).tanh();
    x.signum() * (KNEE + (CEILING - KNEE) * t)
}

/// The deterministic mixer. `new` fixes the sample rate and the seed
/// the noise source draws from; everything else is calls.
pub struct Engine {
    sr: f32,
    rng: Rng,
    next_id: u32,
    voices: Vec<Voice>,
    dying: Vec<Dying>,
    bus_gain: [f32; N_BUS],
    reverbs: [Reverb; N_BUS - 1],
    mono: bool,
    clock: u64,
    fade_samples: u32,
}

impl Engine {
    pub fn new(sample_rate: u32, seed: u64) -> Self {
        Engine {
            sr: sample_rate as f32,
            rng: Rng::new(seed),
            next_id: 0,
            voices: Vec::with_capacity(MAX_VOICES),
            dying: vec![],
            bus_gain: [1.0; N_BUS],
            reverbs: [Reverb::new(sample_rate), Reverb::new(sample_rate),
                      Reverb::new(sample_rate), Reverb::new(sample_rate)],
            mono: false,
            clock: 0,
            fade_samples: (sample_rate as f32 * 0.005) as u32,
        }
    }

    /// Start a source on a bus. When the pool is full the quietest voice
    /// goes (oldest on a tie), fading over 5 ms so nothing clicks.
    pub fn play(&mut self, src: Source, bus: Bus, params: VoiceParams) -> VoiceId {
        if self.voices.len() >= MAX_VOICES {
            let steal = self.voices.iter().enumerate()
                .min_by(|(_, a), (_, b)| a.params.gain.partial_cmp(&b.params.gain)
                    .unwrap_or(std::cmp::Ordering::Equal).then(a.born.cmp(&b.born)))
                .map(|(i, _)| i).unwrap();
            let v = self.voices.remove(steal);
            self.dying.push(Dying { v, left: self.fade_samples, total: self.fade_samples });
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.voices.push(Voice {
            id, src, bus: bi(bus), params,
            phase: 0.0, lp: 0.0, born: self.clock, attack: self.fade_samples,
        });
        VoiceId(id)
    }

    /// Stop a voice. Like the steal it fades over 5 ms rather than
    /// cutting dead mid-sample.
    pub fn stop(&mut self, id: VoiceId) {
        if let Some(i) = self.voices.iter().position(|v| v.id == id.0) {
            let v = self.voices.remove(i);
            self.dying.push(Dying { v, left: self.fade_samples, total: self.fade_samples });
        }
    }

    pub fn set_bus_gain(&mut self, bus: Bus, gain: f32) { self.bus_gain[bi(bus)] = gain; }
    /// Mono folds the two channels together; each side carries L+R.
    pub fn set_mono(&mut self, on: bool) { self.mono = on; }
    /// Voices currently playing (fading-out steals don't count): the
    /// cap and the perf budget both read this.
    pub fn live_voices(&self) -> usize { self.voices.len() }

    fn voice_sample(v: &mut Voice, sr: f32, rng: &mut Rng, age: u64) -> f32 {
        let s = match v.src {
            Source::Sine { hz } => {
                let y = (v.phase * TAU).sin();
                v.phase = (v.phase + hz * v.params.pitch / sr) % 1.0;
                y
            }
            Source::Noise => rng.range(-1.0, 1.0),
            Source::Synth(kind) => {
                let t = age as f32 / sr;
                if t > kind.len() { 0.0 } else { kind.sample(t, rng.range(-1.0, 1.0)) }
            }
        };
        let a = 1.0 - (-TAU * v.params.lowpass_hz / sr).exp(); // 1 - e^(-2pi fc / sr)
        v.lp += a.min(1.0) * (s - v.lp);
        v.lp * v.params.gain
    }

    /// Render interleaved stereo. `out` length must be even. With Master
    /// at zero gain the output is exact zeros - silence, not noise.
    pub fn render(&mut self, out: &mut [f32]) {
        debug_assert!(out.len() % 2 == 0);
        out.fill(0.0);
        if self.bus_gain[bi(Bus::Master)] == 0.0 { self.clock += (out.len() / 2) as u64; return; }
        for i in (0..out.len()).step_by(2) {
            let mut dry = [[0.0f32; 2]; N_BUS - 1];
            let mut wet = [[0.0f32; 2]; N_BUS - 1];
            let mut m = [0.0f32; 2];
            for v in self.voices.iter_mut() {
                let ramp = if v.attack > 0 {
                    let f = 1.0 - v.attack as f32 / self.fade_samples as f32;
                    v.attack -= 1;
                    f
                } else { 1.0 };
                let age = self.clock + (i / 2) as u64 - v.born;
                let s = Self::voice_sample(v, self.sr, &mut self.rng, age) * ramp;
                let th = (v.params.pan.clamp(-1.0, 1.0) + 1.0) * FRAC_PI_4;
                let (l, r) = (s * th.cos(), s * th.sin());
                if v.bus == bi(Bus::Master) { m[0] += l; m[1] += r; }
                else {
                    let b = v.bus - 1;
                    dry[b][0] += l; dry[b][1] += r;
                    wet[b][0] += l * v.params.reverb_send;
                    wet[b][1] += r * v.params.reverb_send;
                }
            }
            for d in self.dying.iter_mut() {
                let fade = d.left as f32 / d.total as f32;
                let age = self.clock + (i / 2) as u64 - d.v.born;
                let s = Self::voice_sample(&mut d.v, self.sr, &mut self.rng, age) * fade;
                let th = (d.v.params.pan.clamp(-1.0, 1.0) + 1.0) * FRAC_PI_4;
                let (l, r) = (s * th.cos(), s * th.sin());
                if d.v.bus == bi(Bus::Master) { m[0] += l; m[1] += r; }
                else { dry[d.v.bus - 1][0] += l; dry[d.v.bus - 1][1] += r; }
                d.left = d.left.saturating_sub(1);
            }
            for b in 0..N_BUS - 1 {
                let (rl, rr) = self.reverbs[b].tick(wet[b][0], wet[b][1]);
                m[0] += self.bus_gain[b + 1] * (dry[b][0] + rl);
                m[1] += self.bus_gain[b + 1] * (dry[b][1] + rr);
            }
            m[0] *= self.bus_gain[bi(Bus::Master)];
            m[1] *= self.bus_gain[bi(Bus::Master)];
            if self.mono {
                let s = (m[0] + m[1]) * 0.5;
                m = [s, s];
            }
            out[i] = limit(m[0]);
            out[i + 1] = limit(m[1]);
        }
        self.dying.retain(|d| d.left > 0);
        self.clock += (out.len() / 2) as u64;
        // a cue voice that's played itself out frees its slot
        let clock = self.clock;
        self.voices.retain(|v| match v.src {
            Source::Synth(kind) => (clock - v.born) as f32 <= kind.len() * self.sr,
            _ => true,
        });
    }
}
