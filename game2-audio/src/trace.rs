//! The score's ledger (#1503): one `BarTrace` per bar of music, written
//! as JSON lines so `sound_dump` can draw the piano roll and timeline
//! and tests can assert the arc without listening. The composer writes
//! the notes; the director's intent fills the bar's context.

use crate::compose::{Composer, Meter, MotifId, Phrase, PhraseSpec, Section, notes_hash};
use crate::soundscape::{Emitter, EmitterKind};
use crate::theory::Mode;
use serde::Serialize;
use std::io::{self, Write};

/// What one bar of the score was: where it sat in the arc, what it was
/// asked to be, what it sounded like, and what was loudest around it.
#[derive(Debug, Clone, Serialize)]
pub struct BarTrace {
    /// Global bar index (0 = the score's first bar).
    pub bar: u64,
    /// Wall-clock start of the bar, seconds (60 fps lockstep).
    pub at_s: f64,
    pub section: String,
    pub mode: String,
    pub tempo: f64,
    pub meter: String,
    /// Per-voice density the phrase was written at.
    pub layers_density: [f32; 8],
    /// The motif the melody carries, by name.
    pub motif_ids: Vec<String>,
    /// The eased mood vector during the bar (tension, valence, energy,
    /// sacred, grief, wonder, festivity) - the timeline's curves.
    pub mood: [f32; 7],
    /// FNV-1a of the bar's note events (`compose::notes_hash`); two
    /// runs of the same seed produce the same hashes, so a trace diff
    /// catches any compositional drift.
    pub notes_hash: u64,
    /// The five loudest/most-present emitter kinds this bar.
    pub top_emitters: Vec<String>,
    /// Stingers that fired during the bar.
    pub stingers: Vec<String>,
}

/// Rank the soundscape's emitters for the trace: most-present kinds
/// first (count, then enum order for determinism).
pub fn top_emitters(emitters: &[Emitter], n: usize) -> Vec<EmitterKind> {
    let mut counts = std::collections::BTreeMap::new();
    for e in emitters {
        *counts.entry(e.kind).or_insert(0usize) += 1;
    }
    let mut v: Vec<(EmitterKind, usize)> = counts.into_iter().collect();
    v.sort_by_key(|(k, c)| (std::cmp::Reverse(*c), *k as u8));
    v.into_iter().take(n).map(|(k, _)| k).collect()
}

/// Compose the phrase an intent asks for and slice it into bar traces.
/// `bar0` is the phrase's first bar index; `at0` its start in seconds.
/// Returns the phrase too so callers can fill the piano roll.
pub fn trace_phrase(
    composer: &mut Composer,
    spec: &PhraseSpec,
    mood: &crate::director::Mood,
    bar0: u64,
    at0: f64,
    emitters: &[Emitter],
    stingers: Vec<String>,
) -> (Phrase, Vec<BarTrace>) {
    let phrase = composer.compose_phrase(spec);
    let bar_s = spec.meter.num as f64 * (4.0 / spec.meter.den as f64) * 60.0 / spec.tempo_bpm;
    let bar_samples = (bar_s * crate::wav::SAMPLE_RATE as f64) as u64;
    let top = top_emitters(emitters, 5);
    let mut bars = Vec::with_capacity(crate::director::BARS_PER_PHRASE as usize);
    for b in 0..crate::director::BARS_PER_PHRASE {
        let lo = b * bar_samples;
        let hi = lo + bar_samples;
        let notes: Vec<_> = phrase.notes.iter()
            .filter(|n| n.at_sample >= lo && n.at_sample < hi)
            .copied().collect();
        bars.push(BarTrace {
            bar: bar0 + b,
            at_s: at0 + b as f64 * bar_s,
            section: format!("{:?}", spec.section),
            mode: format!("{:?}", spec.mode),
            tempo: spec.tempo_bpm,
            meter: format!("{}/{}", spec.meter.num, spec.meter.den),
            layers_density: spec.density,
            motif_ids: vec![format!("{:?}", spec.motif)],
            mood: [mood.tension, mood.valence, mood.energy, mood.sacred,
                mood.grief, mood.wonder, mood.festivity],
            notes_hash: notes_hash(&notes),
            top_emitters: top.iter().map(|k| format!("{k:?}")).collect(),
            stingers: if b == 0 { stingers.clone() } else { vec![] },
        });
    }
    (phrase, bars)
}

/// Append the bars as JSON lines.
pub fn write_trace(w: &mut dyn Write, bars: &[BarTrace]) -> io::Result<()> {
    for b in bars {
        let line = serde_json::to_string(b)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        writeln!(w, "{line}")?;
    }
    Ok(())
}

/// Unused-name placemarks so the serialise derive doesn't trip on the
/// enum imports the doc mentions.
#[allow(dead_code)]
fn _ty(_: Meter, _: MotifId, _: Section, _: Mode) {}
