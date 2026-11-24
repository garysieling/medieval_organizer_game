//! The composer (M4): turns a `PhraseSpec` into 8 bars of note events
//! in the estampie grammar - a punctus of 3 bars' body plus an ouvert
//! ending, then the same body with the clos ending (ouvert again while
//! the cause is unresolved). Every voice is a deterministic function
//! of the spec, and the arranger lets at most one voice's density move
//! per phrase (the 8-bar rule). Times are sample offsets from the
//! phrase start at `wav::SAMPLE_RATE`; the director's clock lands the
//! phrase on a downbeat at its own rate.

use super::motifs::MotifId;
use super::{Meter, NoteEvent, Voice};
use crate::instruments::PatchId;
use crate::rng::Rng;
use crate::theory::{self, Degree, Instrument, Mode};
use crate::wav::SAMPLE_RATE;

/// The score's sections, in the design's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    Silence,
    Ambient,
    Theme,
    Build,
    Peak,
    Release,
    Festive,
    Sacred,
    Lament,
}

/// How a phrase's punctus cadences. Ouvert hangs on degree 2 or 5 -
/// the question stays open; clos lands on the final.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    Ouvert,
    Clos,
}

/// The tempo bands a section's phrases stay inside (bpm). The
/// composer clamps the spec's tempo into its band, so a director
/// slipping out of band is corrected, not obeyed.
pub const TEMPO_AMBIENT: (f64, f64) = (60.0, 72.0);
pub const TEMPO_THEME: (f64, f64) = (84.0, 96.0);
/// Build and march (Peak) share a band.
pub const TEMPO_BUILD: (f64, f64) = (96.0, 112.0);
pub const TEMPO_FEAST: (f64, f64) = (108.0, 126.0);

/// Which band governs a section, if one does. Release, Sacred and
/// Lament phrases take their tempo from the arc around them and are
/// left unbanded.
pub fn tempo_band(s: Section) -> Option<(f64, f64)> {
    match s {
        Section::Ambient => Some(TEMPO_AMBIENT),
        Section::Theme => Some(TEMPO_THEME),
        Section::Build | Section::Peak => Some(TEMPO_BUILD),
        Section::Festive => Some(TEMPO_FEAST),
        _ => None,
    }
}

/// Everything a phrase is written from: the seed makes it
/// deterministic, the rest is the musical request the director makes.
pub struct PhraseSpec {
    pub seed: u64,
    /// The phrase's first bar in the score's bar count; salts the seed
    /// so two identical requests a phrase apart don't sound identical.
    pub bar_index: u64,
    /// Midi note number of the tonic (its octave sets the register).
    pub tonic: u8,
    pub mode: Mode,
    pub section: Section,
    pub meter: Meter,
    pub tempo_bpm: f64,
    /// Per-voice density 0..1, indexed by `Voice`'s order. The
    /// arranger may apply only some of it (the 8-bar rule).
    pub density: [f32; 8],
    pub motif: MotifId,
    /// Whether the arc resolves here: clos ending when true, ouvert
    /// endings both times while it stays unresolved.
    pub resolved: bool,
}

/// A composed phrase: its events (offsets from the phrase start), the
/// ending it actually cadenced on, and how many voices the arranger
/// let move this phrase (0 or 1).
pub struct Phrase {
    pub notes: Vec<NoteEvent>,
    pub ending: Ending,
    pub voices_changed: u8,
}

/// FNV-1a over the event fields, for golden traces.
pub fn notes_hash(notes: &[NoteEvent]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    };
    for n in notes {
        eat(&n.at_sample.to_le_bytes());
        eat(&n.dur_samples.to_le_bytes());
        eat(&[n.voice as u8]);
        eat(&n.midi.to_le_bytes());
        eat(&n.vel.to_le_bytes());
        eat(&[n.patch as u8]);
    }
    h
}

/// The stateful composer: it remembers the last phrase's applied
/// densities, since the arranger's 8-bar rule is a promise between
/// phrases.
pub struct Composer {
    prev_density: Option<[f32; 8]>,
}

const BARS_PER_PUNCTUS: u64 = 4;
const BODY_BARS: u64 = 3;

impl Composer {
    pub fn new() -> Composer {
        Composer { prev_density: None }
    }

    /// Write the next phrase. Silence and a dead-quiet spec both give
    /// back an empty phrase on a clos ending.
    pub fn compose_phrase(&mut self, spec: &PhraseSpec) -> Phrase {
        // The 8-bar rule: at most one voice's density moves per phrase.
        let applied = match self.prev_density {
            None => spec.density,
            Some(prev) => {
                let mut out = prev;
                for i in 0..8 {
                    if spec.density[i] != prev[i] {
                        out[i] = spec.density[i];
                        break;
                    }
                }
                out
            }
        };
        let voices_changed = self.prev_density.map_or(0, |prev| {
            applied.iter().zip(prev.iter()).filter(|(a, p)| a != p).count() as u8
        });
        self.prev_density = Some(applied);

        let ending = if spec.resolved { Ending::Clos } else { Ending::Ouvert };
        let mut phrase = Phrase { notes: Vec::new(), ending, voices_changed };
        if spec.section == Section::Silence || applied.iter().all(|&d| d <= 0.0) {
            return phrase;
        }

        // Time: clamp the tempo into the section's band, then count
        // everything in samples from the phrase's downbeat.
        let bpm = match tempo_band(spec.section) {
            Some((lo, hi)) => spec.tempo_bpm.clamp(lo, hi),
            None => spec.tempo_bpm.max(1.0),
        };
        let t = Time { bpm, meter: spec.meter };
        let mut rng = Rng::new(spec.seed ^ spec.bar_index.wrapping_mul(0x9E3779B97F4A7C15));

        // The drone and ground sit under everything; the melody is
        // written once for punctus A, then counters and organum read it,
        // and punctus B is a copy four bars over.
        self.drone(&mut phrase, spec, &applied, t);
        self.pulse(&mut phrase, spec, &applied, t);
        self.ground(&mut phrase, spec, &applied, t);
        let melody = self.melody(&mut phrase, spec, &applied, t, &mut rng);
        self.counter(&mut phrase, spec, &applied, &melody);
        self.harmony(&mut phrase, spec, &applied, t, &melody);
        self.color(&mut phrase, spec, &applied, t);
        self.choir(&mut phrase, spec, &applied, t);
        phrase
    }

    /// Density of a voice in the applied arrangement.
    fn d(applied: &[f32; 8], v: Voice) -> f32 {
        applied[v as usize]
    }

    /// Tonic and fifth, held: a note per punctus.
    fn drone(&self, phrase: &mut Phrase, spec: &PhraseSpec, applied: &[f32; 8], t: Time) {
        let d = Self::d(applied, Voice::Drone);
        if d <= 0.0 {
            return;
        }
        let patch = if spec.section == Section::Sacred {
            PatchId::Organ
        } else {
            PatchId::HurdyGurdy
        };
        let bar = t.bar_samples();
        for p in 0..2u64 {
            let at = p * BARS_PER_PUNCTUS * bar;
            for dgr in [1i8, 5] {
                let Some(midi) = place(
                    theory::midi(spec.tonic, spec.mode, Degree(dgr)),
                    patch.instrument(),
                ) else {
                    continue;
                };
                phrase.notes.push(NoteEvent {
                    at_sample: at,
                    dur_samples: (BARS_PER_PUNCTUS * bar) as u32,
                    voice: Voice::Drone,
                    midi: midi as f32,
                    vel: 0.3 + 0.4 * d,
                    patch,
                });
            }
        }
    }

    /// Euclidean percussion: hits = ceil(density * 5) spread over the
    /// bar's eighth-note grid, so E(3,8) is the mid-density pattern
    /// and E(5,8) the busy one.
    fn pulse(&self, phrase: &mut Phrase, spec: &PhraseSpec, applied: &[f32; 8], t: Time) {
        let d = Self::d(applied, Voice::Pulse);
        if d <= 0.0 {
            return;
        }
        let patch = match spec.section {
            Section::Peak | Section::Build => PatchId::Tabor,
            Section::Festive => PatchId::Tambourine,
            _ => PatchId::FrameDrum,
        };
        let steps = t.eighth_steps_per_bar();
        let hits = ((d * 5.0).ceil() as usize).min(steps);
        let pattern = theory::euclid(hits, steps);
        let (lo, hi) = patch.instrument().range();
        let step = t.bar_samples() / steps as u64;
        for bar in 0..8u64 {
            for (s, &hit) in pattern.iter().enumerate() {
                if hit {
                    phrase.notes.push(NoteEvent {
                        at_sample: bar * t.bar_samples() + s as u64 * step,
                        dur_samples: (step / 2).max(1) as u32,
                        voice: Voice::Pulse,
                        // the downbeat's low head, the rest its rim
                        midi: if s == 0 { lo as f32 } else { hi as f32 },
                        vel: 0.4 + 0.4 * d,
                        patch,
                    });
                }
            }
        }
    }

    /// Bass on beats 1 and 3; a busier ground doubles to every beat
    /// past mid-density.
    fn ground(&self, phrase: &mut Phrase, spec: &PhraseSpec, applied: &[f32; 8], t: Time) {
        let d = Self::d(applied, Voice::Ground);
        if d <= 0.0 {
            return;
        }
        let patch = match spec.section {
            Section::Sacred => PatchId::Organ,
            Section::Peak | Section::Build => PatchId::Timpani,
            _ => PatchId::FolkHarp,
        };
        let beats: &[u32] = if d > 0.55 {
            &[0, 1, 2, 3]
        } else {
            &[0, 2]
        };
        let beat = t.beat_samples();
        let Some(midi) = place(
            theory::midi(spec.tonic, spec.mode, Degree(1)) as i32 - 12,
            patch.instrument(),
        ) else {
            return;
        };
        for bar in 0..8u64 {
            for &b in beats {
                if b < spec.meter.num as u32 {
                    phrase.notes.push(NoteEvent {
                        at_sample: bar * t.bar_samples() + b as u64 * beat,
                        dur_samples: beat as u32,
                        voice: Voice::Ground,
                        midi: midi as f32,
                        vel: 0.4 + 0.3 * d,
                        patch,
                    });
                }
            }
        }
    }

    /// The motif stated on bar 1, varied across bars 2 and 3; the
    /// ending bar holds the ouvert or clos degree. Punctus B copies
    /// the body verbatim. Returns punctus A's events so the counter
    /// and organum can follow them.
    fn melody(
        &self,
        phrase: &mut Phrase,
        spec: &PhraseSpec,
        applied: &[f32; 8],
        t: Time,
        rng: &mut Rng,
    ) -> Vec<NoteEvent> {
        let d = Self::d(applied, Voice::Melody);
        if d <= 0.0 {
            return Vec::new();
        }
        let patch = spec.motif.lead_patch();
        let bar16 = t.sixteenths_per_bar() as i32;
        let motif = spec.motif.motif();
        let punctus16 = bar16 * BARS_PER_PUNCTUS as i32;

        // Three body bars: statement, then two variations chosen by
        // seed. A variation that lands identical is pushed a step up -
        // no bar may repeat its predecessor.
        let mut figures: Vec<crate::theory::Motif> = Vec::new();
        figures.push(motif.clone());
        let med = |m: &crate::theory::Motif| -> i32 {
            let mut d = m.degrees.to_vec();
            d.sort();
            d[d.len() / 2] as i32
        };
        let med0 = med(&motif);
        for _ in 0..2 {
            let base = figures.last().unwrap();
            let mut v = match rng.next_u32() % 5 {
                0 => base.transpose(1),
                1 => base.transpose(-1),
                2 => base.invert(),
                3 if base.degrees.len() > 2 => base.fragment(base.degrees.len() - 1),
                _ => base.transpose(2),
            };
            if v.degrees == base.degrees {
                v = v.transpose(1);
            }
            // A variation sings in the motif's register: an inversion
            // that dives or soars comes back by octaves (seven steps,
            // same pitch class) so the lead instrument can hold the
            // whole line without folding a strain away.
            while med(&v) > med0 + 4 {
                v = v.transpose(-7);
            }
            while med(&v) < med0 - 4 {
                v = v.transpose(7);
            }
            figures.push(v);
        }

        // How much of the body sings: low density states the motif and
        // falls silent; past a quarter the variations join; past .45
        // free notes fill what rhythm is left.
        let play_bars = if d < 0.25 { 1 } else { BODY_BARS } as usize;
        let mut events: Vec<MelodyNote> = Vec::new();
        let mut last_deg = 1i8;
        let mut resolve_step: Option<i8> = None;
        for bar in 0..play_bars {
            let mut fig = figures[bar].clone();
            // The strain steps back in: an entry that would leap a
            // fourth or more and not resolve by stepping back the way
            // it came is snapped onto the degree the last bar ended
            // on - the figure keeps its shape, the boundary steps.
            let entry = fig.degrees[0] as i32 - last_deg as i32;
            let resolves = fig.degrees.get(1).is_some_and(|&s| {
                let st = s as i32 - fig.degrees[0] as i32;
                st.abs() <= 2 && st.signum() == -entry.signum()
            });
            if entry.abs() >= 3 && !resolves {
                fig = fig.transpose(last_deg - fig.degrees[0]);
            }
            let base16 = bar as i32 * bar16;
            // The figure fills its bar whatever the meter holds: in a
            // short bar the rhythm compresses onto the sixteenth grid,
            // notes stay ordered, nothing spills into the next strain.
            let total16: i32 = fig.rhythm.iter().map(|&r| r as i32).sum();
            let mut t16 = 0i32;
            let mut prev16 = -1i32;
            for (i, &deg) in fig.degrees.iter().enumerate() {
                let mut at16 = base16 + t16 * bar16 / total16;
                at16 = at16.max(prev16 + 1);
                if at16 >= base16 + bar16 || at16 >= punctus16 {
                    break;
                }
                let next16 = base16 + (t16 + fig.rhythm[i] as i32) * bar16 / total16;
                let dur16 = (next16 - at16).max(1).min(base16 + bar16 - at16);
                events.push(MelodyNote { at16, dur16, deg, cadence: false });
                last_deg = deg;
                prev16 = at16;
                t16 += fig.rhythm[i] as i32;
            }
            // Free notes fill what the figure left: step 70%, a 3rd
            // 20%, a leap of at most a 5th 10%, each leap resolved by
            // a step the other way.
            if d > 0.45 {
                let mut fill16 = events.last().map_or(0, |m| {
                    if m.at16 / bar16 == bar as i32 { m.at16 - base16 + m.dur16 } else { 0 }
                });
                while fill16 + 2 <= bar16 {
                    let step = free_step(rng, &mut resolve_step);
                    last_deg += step;
                    events.push(MelodyNote {
                        at16: base16 + fill16,
                        dur16: 2,
                        deg: last_deg,
                        cadence: false,
                    });
                    fill16 += 2;
                }
            }
        }

        // Draft the whole 8-bar line before placing it: punctus A's
        // body and ouvert cadence, punctus B's body copy and the
        // phrase's real cadence. The ouvert degree is whichever of 2
        // or 5 lands nearest (by an octave) on the strain's last note
        // - the cadence steps off, it doesn't vault.
        let near = |dg: i8| -> i32 {
            (-2..=2i8)
                .map(|j| (dg + 7 * j - last_deg) as i32)
                .map(i32::abs)
                .min()
                .unwrap()
        };
        let deg_a = match (near(2), near(5)) {
            (a, b) if a < b => 2,
            (a, b) if b < a => 5,
            _ => {
                if rng.chance(0.5) {
                    2
                } else {
                    5
                }
            }
        };
        let deg_b = match phrase.ending {
            Ending::Clos => 1,
            Ending::Ouvert => deg_a,
        };
        let end16 = bar16 * BODY_BARS as i32;
        let mut line: Vec<MelodyNote> = Vec::new();
        for m in &events {
            if m.at16 < end16 {
                line.push(MelodyNote { cadence: false, ..*m });
            }
        }
        line.push(MelodyNote { at16: end16, dur16: bar16, deg: deg_a, cadence: true });
        for m in &events {
            if m.at16 < end16 {
                line.push(MelodyNote {
                    at16: m.at16 + end16 + bar16,
                    dur16: m.dur16,
                    deg: m.deg,
                    cadence: false,
                });
            }
        }
        line.push(MelodyNote {
            at16: end16 + end16 + bar16,
            dur16: bar16,
            deg: deg_b,
            cadence: true,
        });
        line.sort_by_key(|m| m.at16);

        // The lead patch: the motif's own instrument when the motif
        // itself fits its compass unfolded, else the voice that holds
        // the motif and covers the most of the line (a 12st cause-cell
        // can't sing on a 10st fife - a recorder stands in rather than
        // break the motif's shape). Stragglers outside it fold or rest.
        let midis: Vec<i32> = line
            .iter()
            .map(|m| theory::midi(spec.tonic, spec.mode, Degree(m.deg)) as i32)
            .collect();
        let n_motif = line.iter().take_while(|m| m.at16 < bar16).count();
        // The octave that centres the motif in the compass - a wide
        // instrument must not drag the whole line to its bottom.
        let motif_k = |inst: Instrument| -> Option<i32> {
            let (lo, hi) = (inst.range().0 as i32, inst.range().1 as i32);
            let (mn, mx) = (
                *midis[..n_motif].iter().min().unwrap_or(&0),
                *midis[..n_motif].iter().max().unwrap_or(&0),
            );
            let k0 = ((lo + hi - mn - mx) as f64 / 24.0).round() as i32;
            [k0, k0 - 1, k0 + 1, k0 - 2, k0 + 2, k0 - 3, k0 + 3]
                .into_iter()
                .find(|&k| {
                    midis[..n_motif]
                        .iter()
                        .all(|m| (lo..=hi).contains(&(m + 12 * k)))
                })
        };
        let mut pick: Option<(PatchId, i32, usize)> = None;
        for p in [
            patch, PatchId::RecorderSoprano, PatchId::RecorderAlto,
            PatchId::RecorderTenor, PatchId::Vielle, PatchId::FolkHarp,
            PatchId::Shawm,
        ] {
            let inst = p.instrument();
            if let Some(k) = motif_k(inst) {
                // coverage counts notes that sit UNFOLDED: octaves are
                // legal, but a folded straggler next to a direct note
                // can flip a leap's direction mid-figure
                let (lo, hi) = inst.range();
                let cov = midis
                    .iter()
                    .filter(|&&m| (lo as i32..=hi as i32).contains(&(m + 12 * k)))
                    .count();
                if pick.map_or(true, |(_, _, c)| cov > c) {
                    pick = Some((p, k, cov));
                }
            }
        }
        let (patch, line_k) = pick.map(|(p, k, _)| (p, k)).unwrap_or((patch, 0));
        let inst = patch.instrument();
        let mut placed: Vec<Option<i32>> = midis
            .iter()
            .map(|&m| place(m + 12 * line_k, inst))
            .collect();

        // Emit, then drop any note whose fold still left a gap wider
        // than a fifth to its neighbour (never a cadence - its
        // neighbour goes instead).
        // Sixteenth slots divide the bar exactly - a truncated grid
        // would land cadences a few samples shy of the bar line.
        let at = |at16: i32| at16 as u64 * t.bar_samples() / bar16 as u64;
        let mut notes: Vec<NoteEvent> = Vec::new();
        let mut cadence: Vec<bool> = Vec::new();
        let mut prev: Option<i32> = None;
        for (i, m) in line.iter().enumerate() {
            let pl = placed[i];
            // A cadence sings its degree in the octave that lands
            // best: near the strain it ends AND so the way out is a
            // leap the next note can resolve. The repeat may enter an
            // octave over (b_off) to make that exit clean - a resolvable
            // exit outranks a clean landing, since the drop pass can
            // silence a vaulting entry but nothing repairs an
            // unresolved leap out of a cadence.
            let midi_opt = if m.cadence {
                let (lo, hi) = inst.range();
                let is_a = m.at16 == end16;
                let next: Option<i32> =
                    placed[i + 1..].iter().find_map(|o| *o);
                let next2: Option<i32> =
                    placed[i + 1..].iter().flatten().nth(1).copied();
                let degs: &[i8] = if is_a || !spec.resolved {
                    &[2, 5]
                } else {
                    &[1]
                };
                let mut best: Option<(i32, i32, i32, i32, i32)> = None;
                let mut chosen: Option<(i32, i32)> = None;
                for &dg in degs {
                    let base = theory::midi(spec.tonic, spec.mode, Degree(dg)) as i32;
                    for j in -3i32..=3 {
                        let c = base + 12 * j;
                        if !(lo as i32..=hi as i32).contains(&c) {
                            continue;
                        }
                        let entry = prev.map(|p| (p - c).abs()).unwrap_or(0);
                        // the repeat may sit an octave over - the exit
                        // is judged against the octave that leaves best
                        let offs: &[i32] = if is_a { &[0, 1, -1] } else { &[0] };
                        for &boff in offs {
                            let n = next.map(|x| x + 12 * boff);
                            let n2 = next2.map(|x| x + 12 * boff);
                            let dirty = match n {
                                Some(n) => {
                                    let gap = n - c;
                                    !(gap.abs() <= 7
                                        && (gap.abs() < 5
                                            || n2.map_or(false, |n2| {
                                                let st = n2 - n;
                                                st.signum() == -gap.signum()
                                                    && st.abs() <= 4
                                            })))
                                }
                                None => false,
                            };
                            let score = (
                                dirty as i32,
                                (entry > 7) as i32,
                                entry,
                                boff.abs(),
                                j.abs(),
                            );
                            if best.map_or(true, |s| score < s) {
                                best = Some(score);
                                chosen = Some((c, boff));
                            }
                        }
                    }
                }
                chosen.map(|(c, boff)| {
                    if is_a && boff != 0 {
                        // a strain that can't take the new register
                        // rests - folding back would saw the line in
                        // two registers inside one bar
                        for p in placed[i + 1..].iter_mut() {
                            if let Some(x) = p {
                                let y = *x + 12 * boff;
                                *p = if (lo as i32..=hi as i32).contains(&y) {
                                    Some(y)
                                } else {
                                    None
                                };
                            }
                        }
                    }
                    // and where no octave leaves clean, the repeat's
                    // head rests until the cadence's way out steps or
                    // resolves - the grammar keeps its endings
                    let mut k = i + 1;
                    while let Some(ni) = (k..line.len()).find(|&x| placed[x].is_some()) {
                        let n = placed[ni].unwrap();
                        let gap = n - c;
                        let clean = gap.abs() <= 7
                            && (gap.abs() < 5
                                || (ni + 1..line.len())
                                    .find_map(|x| placed[x])
                                    .map_or(false, |n2| {
                                        let st = n2 - n;
                                        st.signum() == -gap.signum() && st.abs() <= 4
                                    }));
                        if clean {
                            break;
                        }
                        placed[ni] = None;
                        k = ni + 1;
                    }
                    c
                })
            } else {
                pl
            };
            if let Some(midi) = midi_opt {
                let start = at(m.at16);
                notes.push(NoteEvent {
                    at_sample: start,
                    dur_samples: (at(m.at16 + m.dur16.max(1)) - start) as u32,
                    voice: Voice::Melody,
                    midi: midi as f32,
                    vel: if m.cadence { 0.6 + 0.3 * d } else { 0.5 + 0.3 * d },
                    patch,
                });
                cadence.push(m.cadence);
                prev = Some(midi);
            }
        }
        // Two repairs that chase each other - a fifth-rule drop can
        // leave two bars identical, a bar-nudge can open a gap - so
        // they run to a bounded fixpoint (every round either shrinks
        // the line or shifts a bar's register, so it settles well
        // inside the bound).
        let bar = t.bar_samples();
        for _ in 0..64 {
            let mut changed = false;
            // the fifth rule: no interval wider than a fifth survives;
            // the leapt-to note rests (never a cadence).
            let mut i = 0;
            while i + 1 < notes.len() {
                if (notes[i + 1].midi - notes[i].midi).abs() > 7.0 {
                    let drop_i = if cadence[i + 1] { i } else { i + 1 };
                    notes.remove(drop_i);
                    cadence.remove(drop_i);
                    changed = true;
                } else {
                    i += 1;
                }
            }
            // no two consecutive bars run identical: the repeat takes
            // its notes up a scale step where the instrument holds
            // them, else its last note rests.
            let seq = |b: usize| -> Vec<(i64, u32)> {
                notes.iter()
                    .filter(|n| (n.at_sample / bar) as usize == b)
                    .map(|n| (n.midi as i64, n.dur_samples))
                    .collect()
            };
            for b in 1..8usize {
                if seq(b).is_empty() || seq(b) != seq(b - 1) {
                    continue;
                }
                // cadence notes keep their ending degree; only the
                // bar's free notes can move
                let idx: Vec<usize> = notes
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| {
                        (notes[*i].at_sample / bar) as usize == b && !cadence[*i]
                    })
                    .map(|(i, _)| i)
                    .collect();
                let stepped: Vec<Option<i32>> = idx
                    .iter()
                    .map(|&i| {
                        let d = deg_of(spec.tonic, spec.mode, notes[i].midi as u8);
                        place(
                            theory::midi(spec.tonic, spec.mode, Degree(d + 1)),
                            inst,
                        )
                    })
                    .collect();
                if stepped.iter().all(Option::is_some) {
                    for (&i, m) in idx.iter().zip(stepped) {
                        notes[i].midi = m.unwrap() as f32;
                    }
                } else if let Some(&j) = idx.iter().rev().find(|&&j| !cadence[j]) {
                    notes.remove(j);
                    cadence.remove(j);
                }
                changed = true;
                break;
            }
            if !changed {
                break;
            }
        }
        phrase.notes.extend(notes.iter().cloned());
        notes
    }

    /// The tintinnabuli T-voice: for each melody note, the nearest
    /// tonic-triad tone. Density decides what share of the melody is
    /// shadowed (from the first notes on).
    fn counter(
        &self,
        phrase: &mut Phrase,
        spec: &PhraseSpec,
        applied: &[f32; 8],
        melody: &[NoteEvent],
    ) {
        let d = Self::d(applied, Voice::Counter);
        if d <= 0.0 || melody.is_empty() {
            return;
        }
        let triad = theory::triad(spec.tonic, spec.mode, Degree(1));
        let patch = PatchId::Psaltery;
        let inst = patch.instrument();
        let n = (melody.len() as f32 * d).ceil() as usize;
        for m in melody.iter().take(n) {
            phrase.notes.push(NoteEvent {
                at_sample: m.at_sample,
                dur_samples: m.dur_samples,
                voice: Voice::Counter,
                midi: match place(
                    theory::tintinnabuli_t(m.midi as u8, triad),
                    inst,
                ) {
                    Some(m) => m as f32,
                    None => continue,
                },
                vel: m.vel * 0.6,
                patch,
            });
        }
    }

    /// Open fifths on the lute at each bar start (a second strum
    /// mid-bar past .6); in Sacred, organum instead: the melody
    /// doubled a perfect fifth below on the organ.
    fn harmony(
        &self,
        phrase: &mut Phrase,
        spec: &PhraseSpec,
        applied: &[f32; 8],
        t: Time,
        melody: &[NoteEvent],
    ) {
        let d = Self::d(applied, Voice::Harmony);
        if d <= 0.0 {
            return;
        }
        if spec.section == Section::Sacred {
            let patch = PatchId::Organ;
            for m in melody {
                let Some(midi) = place(
                    theory::midi(spec.tonic, spec.mode, Degree(
                        // parallel 5th below the melody's degree is a
                        // diatonic shift of -4
                        deg_of(spec.tonic, spec.mode, m.midi as u8) - 4,
                    )),
                    patch.instrument(),
                ) else {
                    continue;
                };
                phrase.notes.push(NoteEvent {
                    at_sample: m.at_sample,
                    dur_samples: m.dur_samples,
                    voice: Voice::Harmony,
                    midi: midi as f32,
                    vel: m.vel * 0.5,
                    patch,
                });
            }
            return;
        }
        let patch = PatchId::Lute;
        let inst = patch.instrument();
        let strums: &[u32] = if d > 0.6 { &[0, 2] } else { &[0] };
        let (Some(fifth), Some(root)) = (
            place(theory::midi(spec.tonic, spec.mode, Degree(5)), inst),
            place(theory::midi(spec.tonic, spec.mode, Degree(1)), inst),
        ) else {
            return;
        };
        for bar in 0..8u64 {
            for &b in strums {
                if b < spec.meter.num as u32 {
                    for midi in [root, fifth] {
                        phrase.notes.push(NoteEvent {
                            at_sample: bar * t.bar_samples() + b as u64 * t.beat_samples(),
                            dur_samples: (t.beat_samples() / 2) as u32,
                            voice: Voice::Harmony,
                            midi: midi as f32,
                            vel: 0.35 + 0.35 * d,
                            patch,
                        });
                    }
                }
            }
        }
    }

    /// Bells and finger cymbals on the punctus starts; busier
    /// arrangements add them mid-punctus too.
    fn color(&self, phrase: &mut Phrase, spec: &PhraseSpec, applied: &[f32; 8], t: Time) {
        let d = Self::d(applied, Voice::Color);
        if d <= 0.0 {
            return;
        }
        let patch = if spec.section == Section::Sacred {
            PatchId::TubularBells
        } else {
            PatchId::HandBells
        };
        let Some(midi) = place(
            theory::midi(spec.tonic, spec.mode, Degree(8)),
            patch.instrument(),
        ) else {
            return;
        };
        let bar = t.bar_samples();
        let hits: &[u64] = if d > 0.6 { &[0, 2, 4, 6] } else { &[0, 4] };
        for &b in hits {
            phrase.notes.push(NoteEvent {
                at_sample: b * bar,
                dur_samples: (bar / 2) as u32,
                voice: Voice::Color,
                midi: midi as f32,
                vel: 0.3 + 0.5 * d,
                patch,
            });
        }
    }

    /// The choir pad holds the triad through the phrase; Sacred only.
    fn choir(&self, phrase: &mut Phrase, spec: &PhraseSpec, applied: &[f32; 8], t: Time) {
        let d = Self::d(applied, Voice::Choir);
        if spec.section != Section::Sacred || d <= 0.0 {
            return;
        }
        let patch = PatchId::ChoirPad;
        let bar = t.bar_samples();
        for deg in [1i8, 3, 5] {
            let Some(midi) = place(
                theory::midi(spec.tonic, spec.mode, Degree(deg)),
                patch.instrument(),
            ) else {
                continue;
            };
            phrase.notes.push(NoteEvent {
                at_sample: 0,
                dur_samples: (8 * bar) as u32,
                voice: Voice::Choir,
                midi: midi as f32,
                vel: 0.25 + 0.35 * d,
                patch,
            });
        }
    }
}

/// The free-note step rule, in diatonic steps: a step 70% of the
/// time, a 3rd 20%, a leap of at most a 5th 10%. A leap queues a
/// resolution in `resolve` - the next free note steps the other way.
/// Exported so tests can measure the distribution itself.
pub fn free_step(rng: &mut Rng, resolve: &mut Option<i8>) -> i8 {
    match resolve.take() {
        Some(s) => s,
        None => {
            let r = rng.next_f32();
            let sign = if rng.chance(0.5) { 1i8 } else { -1 };
            if r < 0.7 {
                sign
            } else if r < 0.9 {
                2 * sign
            } else {
                let leap = if rng.chance(0.5) { 3 } else { 4 } * sign;
                *resolve = Some(-sign);
                leap
            }
        }
    }
}

/// A melody draft note in scale-degree + sixteenth space, before it
/// is realised to midi and samples.
struct MelodyNote {
    at16: i32,
    dur16: i32,
    deg: i8,
    /// A punctus-ending note: never dropped by the leap pass.
    cadence: bool,
}

/// Recover the scale degree of a midi note (nearest diatonic step);
/// the organum voice needs the degree to shift it a fifth.
fn deg_of(tonic: u8, mode: Mode, midi: u8) -> i8 {
    let mut best = 1i8;
    let mut best_d = i32::MAX;
    for d in -7..=15i8 {
        let m = theory::midi(tonic, mode, Degree(d)) as i32;
        let dist = (m - midi as i32).abs();
        if dist < best_d {
            best = d;
            best_d = dist;
        }
    }
    best
}

/// Place one note into an instrument's range by octave folds; a
/// note that can't fold in is dropped (a rest where the line can't
/// reach), never clamped off-pitch.
fn place<M: Into<i32>>(midi: M, inst: Instrument) -> Option<i32> {
    let (lo, hi) = (inst.range().0 as i32, inst.range().1 as i32);
    let mut m = midi.into();
    for _ in 0..10 {
        if (lo..=hi).contains(&m) {
            return Some(m);
        }
        m += if m < lo { 12 } else { -12 };
    }
    None
}

/// Musical time in samples at the canonical rate.
#[derive(Clone, Copy)]
struct Time {
    bpm: f64,
    meter: Meter,
}

impl Time {
    fn beat_secs(self) -> f64 {
        (240.0 / self.bpm) / self.meter.den as f64
    }
    fn beat_samples(self) -> u64 {
        (self.beat_secs() * SAMPLE_RATE as f64).round() as u64
    }
    fn bar_samples(self) -> u64 {
        self.beat_samples() * self.meter.num as u64
    }
    /// Sixteenths in a bar: the motif rhythm's grid.
    fn sixteenths_per_bar(self) -> u32 {
        self.meter.num as u32 * 16 / self.meter.den as u32
    }
    /// Eighth-note steps in a bar: the euclidean grid (8 for x/4
    /// meters, `num` for x/8).
    fn eighth_steps_per_bar(self) -> usize {
        (self.meter.num as usize * 8 / self.meter.den as usize).max(1)
    }
}
