//! The musical clock: counts time in samples so offline renders are
//! exact. Beat, bar and phrase boundaries accumulate in f64 and round
//! only when emitted, so cumulative error stays under a sample over
//! any horizon the game plays for. Tempo and meter changes land only
//! on quantized boundaries - never mid-bar.

use super::{Meter, NoteEvent, Quantize};

/// A time boundary reached during `advance`. At a phrase downbeat all
/// three kinds share one `at_sample` and emit in Phrase, Bar, Beat
/// order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tick {
    /// A phrase (8 bars) boundary; `phrase` counts from 0.
    Phrase { phrase: u64, at_sample: u64 },
    /// A bar boundary; `bar` counts bars from 0.
    Bar { bar: u64, at_sample: u64 },
    /// A beat boundary: `bar` is its bar, `beat` the index within it.
    Beat { bar: u64, beat: u8, at_sample: u64 },
}

impl Tick {
    pub fn at_sample(&self) -> u64 {
        match *self {
            Tick::Phrase { at_sample, .. }
            | Tick::Bar { at_sample, .. }
            | Tick::Beat { at_sample, .. } => at_sample,
        }
    }
}

/// Sample-counting sequencer clock. Musical time is real seconds
/// (decision 9): the tempo never follows the game's speed setting.
pub struct Clock {
    pub sample_rate: u32,
    /// Current tempo; changes only at quantized boundaries.
    pub tempo_bpm: f64,
    /// Current meter; changes only at phrase boundaries.
    pub meter: Meter,
    /// Samples rendered so far; `advance` moves it forward.
    pub sample_pos: u64,
    /// NoteEvents scheduled before the playhead. Kept sorted lazily at
    /// drain time (schedule rate is low; a sorted insert would cost
    /// more than the rare sort).
    events: Vec<NoteEvent>,
    /// Events dropped because they were scheduled in the past: exact
    /// timing beats never-late, so they're counted, not fired.
    pub dropped_late: u64,

    // Boundary accumulators: absolute sample position of the next
    // boundary, kept fractional so nothing drifts.
    next_beat_at: f64,
    next_bar_at: f64,
    next_phrase_at: f64,
    bar_index: u64,
    beat_in_bar: u8,
    phrase_index: u64,

    pending_tempo: Option<(f64, Quantize)>,
    pending_meter: Option<Meter>,
}

const BARS_PER_PHRASE: u8 = 8;

impl Clock {
    /// A clock at a downbeat: the first `advance` emits Phrase/Bar/Beat
    /// ticks at sample 0.
    pub fn new(sample_rate: u32, tempo_bpm: f64, meter: Meter) -> Clock {
        Clock {
            sample_rate,
            tempo_bpm,
            meter,
            sample_pos: 0,
            events: Vec::new(),
            dropped_late: 0,
            next_beat_at: 0.0,
            next_bar_at: 0.0,
            next_phrase_at: 0.0,
            bar_index: 0,
            beat_in_bar: 0,
            phrase_index: 0,

            pending_tempo: None,
            pending_meter: None,
        }
    }

    /// Seconds per beat of the current meter (a 1/den note: 4/4's beat
    /// is a quarter, 6/8's an eighth).
    fn beat_secs(&self) -> f64 {
        (240.0 / self.tempo_bpm) / self.meter.den as f64
    }
    fn beat_samples(&self) -> f64 {
        self.beat_secs() * self.sample_rate as f64
    }
    fn bar_samples(&self) -> f64 {
        self.beat_samples() * self.meter.num as f64
    }
    fn phrase_samples(&self) -> f64 {
        self.bar_samples() * BARS_PER_PHRASE as f64
    }

    /// Move the playhead forward `n` samples, emitting every boundary
    /// crossed - Phrase before Bar before Beat when they coincide.
    pub fn advance(&mut self, n: usize) -> Vec<Tick> {
        let end = self.sample_pos + n as u64;
        let mut ticks = Vec::new();
        loop {
            // Earliest due boundary; phrase outranks bar outranks beat
            // at a shared sample.
            let Some((kind, at)) = [
                (0u8, self.next_phrase_at),
                (1u8, self.next_bar_at),
                (2u8, self.next_beat_at),
            ]
            .into_iter()
            .filter(|&(_, t)| t <= end as f64)
            .min_by(|&(ka, ta), &(kb, tb)| {
                ta.partial_cmp(&tb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(ka.cmp(&kb))
            }) else {
                break;
            };
            let at_sample = at.round() as u64;
            match kind {
                0 => {
                    // Phrase boundary: apply pending requests first so
                    // following intervals use the new values. A phrase
                    // downbeat is a bar boundary too, so a bar-quantized
                    // tempo pending here applies as well.
                    if let Some((bpm, _)) = self.pending_tempo.take() {
                        self.tempo_bpm = bpm;
                    }
                    if let Some(m) = self.pending_meter.take() {
                        self.meter = m;
                    }
                    ticks.push(Tick::Phrase {
                        phrase: self.phrase_index,
                        at_sample,
                    });
                    self.phrase_index += 1;
                    self.next_phrase_at += self.phrase_samples();
                }
                1 => {
                    if matches!(self.pending_tempo, Some((_, Quantize::Bar))) {
                        self.tempo_bpm = self.pending_tempo.take().unwrap().0;
                    }
                    ticks.push(Tick::Bar {
                        bar: self.bar_index,
                        at_sample,
                    });
                    self.bar_index += 1;
        
                    self.beat_in_bar = 0;
                    self.next_bar_at += self.bar_samples();
                }
                _ => {
                    ticks.push(Tick::Beat {
                        bar: self.bar_index,
                        beat: self.beat_in_bar,
                        at_sample,
                    });
                    self.beat_in_bar += 1;
                    self.next_beat_at += self.beat_samples();
                }
            }
        }
        self.sample_pos = end;
        ticks
    }

    /// Queue a note; events scheduled in the past are dropped and
    /// counted rather than fired late.
    pub fn schedule(&mut self, ev: NoteEvent) {
        if ev.at_sample < self.sample_pos {
            self.dropped_late += 1;
            return;
        }
        self.events.push(ev);
    }

    /// All events due at or before `up_to_sample`, in (at_sample,
    /// voice) order, each exactly once.
    pub fn drain_due(&mut self, up_to_sample: u64) -> Vec<NoteEvent> {
        let (due, keep): (Vec<_>, Vec<_>) = self
            .events
            .drain(..)
            .partition(|e| e.at_sample <= up_to_sample);
        self.events = keep;
        let mut due = due;
        due.sort_by(|a, b| {
            a.at_sample
                .cmp(&b.at_sample)
                .then(a.voice.cmp(&b.voice))
        });
        due
    }

    /// Ask for a tempo change at a boundary; a second request replaces
    /// the pending one. Builds express their +2 bpm/phrase ramp as one
    /// request per phrase.
    pub fn request_tempo(&mut self, bpm: f64, q: Quantize) {
        self.pending_tempo = Some((bpm, q));
    }

    /// Ask for a meter change; always phrase-quantized (a meter change
    /// mid-phrase is nonsensical).
    pub fn request_meter(&mut self, m: Meter) {
        self.pending_meter = Some(m);
    }

    /// Bar and phrase counters, for trace output.
    pub fn bar(&self) -> u64 {
        self.bar_index
    }
    pub fn phrase(&self) -> u64 {
        self.phrase_index
    }
}
