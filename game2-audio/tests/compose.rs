//! Property tests for the composer (M4): 1,000 seeded phrases run
//! through every invariant the design promises - mode purity, leap
//! rules, ouvert/clos endings, instrument ranges, the 8-bar rule.

use game2_audio::compose::{
    free_step, notes_hash, tempo_band, Composer, Ending, Meter, MotifId, NoteEvent,
    Phrase, PhraseSpec, Section, Voice,
};
use game2_audio::rng::Rng;
use game2_audio::theory::Mode;

fn spec(seed: u64) -> PhraseSpec {
    PhraseSpec {
        seed,
        bar_index: 0,
        tonic: 62, // D4 - the fixed tonic register
        mode: Mode::Dorian,
        section: Section::Theme,
        meter: Meter::FOUR_FOUR,
        tempo_bpm: 90.0,
        density: [0.8; 8],
        motif: MotifId::Village,
        resolved: true,
    }
}

fn rand_spec(rng: &mut Rng, seed: u64) -> PhraseSpec {
    let meter = [
        Meter::FOUR_FOUR,
        Meter::THREE_FOUR,
        Meter::SIX_EIGHT,
        Meter::TWO_FOUR,
    ][rng.next_u32() as usize % 4];
    PhraseSpec {
        seed,
        bar_index: (rng.next_u32() % 64) as u64,
        tonic: 50 + (rng.next_u32() % 15) as u8,
        mode: Mode::ALL[rng.next_u32() as usize % 6],
        section: [
            Section::Ambient,
            Section::Theme,
            Section::Build,
            Section::Peak,
            Section::Release,
            Section::Festive,
            Section::Sacred,
            Section::Lament,
        ][rng.next_u32() as usize % 8],
        meter,
        tempo_bpm: rng.range(50.0, 140.0) as f64,
        density: [
            rng.next_f32(), rng.next_f32(), rng.next_f32(), rng.next_f32(),
            rng.next_f32(), rng.next_f32(), rng.next_f32(), rng.next_f32(),
        ],
        motif: MotifId::ALL[rng.next_u32() as usize % 8],
        resolved: rng.chance(0.6),
    }
}

fn bar_samples(s: &PhraseSpec) -> u64 {
    let bpm = match tempo_band(s.section) {
        Some((lo, hi)) => s.tempo_bpm.clamp(lo, hi),
        None => s.tempo_bpm.max(1.0),
    };
    let beat = ((240.0 / bpm) / s.meter.den as f64 * 44100.0).round() as u64;
    beat * s.meter.num as u64
}

fn melody(notes: &[NoteEvent]) -> Vec<NoteEvent> {
    let mut m: Vec<NoteEvent> = notes.iter().filter(|n| n.voice == Voice::Melody).cloned().collect();
    m.sort_by(|a, b| a.at_sample.cmp(&b.at_sample));
    m
}

/// Semitones a melody note sits above the tonic, mod 12.
fn pc(s: &PhraseSpec, n: &NoteEvent) -> u8 {
    (((n.midi as u8 % 12) as i32 - (s.tonic % 12) as i32).rem_euclid(12)) as u8
}

/// The degree-1-based pcs of a mode: {1,2,3,4,5,6,7} in semitones.
fn mode_pcs(s: &PhraseSpec) -> [u8; 7] {
    s.mode.intervals()
}

/// Melody notes that start a punctus's ending bar and hold it.
fn ending_notes(s: &PhraseSpec, notes: &[NoteEvent]) -> Vec<NoteEvent> {
    let bar = bar_samples(s);
    melody(notes)
        .into_iter()
        .filter(|n| {
            let at_bar = n.at_sample / bar;
            n.at_sample % bar == 0 && (at_bar == 3 || at_bar == 7) && n.dur_samples as u64 == bar
        })
        .collect()
}

#[test]
fn all_notes_in_mode() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0xA11);
        let s = rand_spec(&mut rng, seed);
        let p = Composer::new().compose_phrase(&s);
        let pcs = mode_pcs(&s);
        for n in &p.notes {
            // Pulse is unpitched percussion in a play band, not a scale
            if n.voice != Voice::Pulse {
                assert!(
                    pcs.contains(&pc(&s, n)),
                    "seed {seed}: {:?} note {} out of {:?}",
                    n.voice, n.midi, s.mode,
                );
            }
        }
    }
}

#[test]
fn leaps_at_most_a_fifth_and_resolve() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0xB22);
        let s = rand_spec(&mut rng, seed);
        let p = Composer::new().compose_phrase(&s);
        let bar = bar_samples(&s);
        let m = melody(&p.notes);
        for w in m.windows(2) {
            let d = (w[1].midi - w[0].midi).abs();
            assert!(d <= 7.0, "seed {seed}: melody leaps a {}st", d);
        }
        // The Village motif is stepwise and its transforms stay stepwise,
        // so every >=5st interval in these phrases is a free-note leap
        // and must resolve by a step the other way - unless the next
        // note is the ending bar's held cadence.
        if s.motif == MotifId::Village {
            for i in 0..m.len().saturating_sub(1) {
                let leap = m[i + 1].midi - m[i].midi;
                if leap.abs() >= 5.0 {
                    let next_is_cadence = m[i + 1].at_sample % bar == 0
                        && m[i + 1].dur_samples as u64 == bar;
                    if !next_is_cadence && i + 2 < m.len() {
                        let step = m[i + 2].midi - m[i + 1].midi;
                        assert!(
                            step.abs() <= 4.0 && step.signum() == -leap.signum(),
                            "seed {seed}: leap {leap} resolved by {step}",
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn ouvert_endings_on_degree_2_or_5_clos_on_1() {
    for seed in 0..500u64 {
        let mut rng = Rng::new(seed ^ 0xC33);
        let mut s = rand_spec(&mut rng, seed);
        s.density[Voice::Melody as usize] = 0.9; // melody must be playing
        let p = Composer::new().compose_phrase(&s);
        if p.ending == Ending::Clos {
            assert!(s.resolved, "clos on an unresolved arc, seed {seed}");
        }
        let endings = ending_notes(&s, &p.notes);
        if endings.is_empty() {
            continue;
        }
        let iv = s.mode.intervals();
        let bar = bar_samples(&s);
        // each placed punctus cadence is checked at its own bar
        for e in &endings {
            let at_bar = e.at_sample / bar;
            let want: Vec<u8> = if at_bar == 3 || !s.resolved {
                vec![iv[1], iv[4]] // ouvert cadences land on 2 or 5
            } else {
                vec![iv[0]] // clos on the final
            };
            assert!(
                want.contains(&pc(&s, e)),
                "seed {seed}: bar {at_bar} cadence on pc {}",
                pc(&s, e),
            );
        }
    }
}

#[test]
fn unresolved_keeps_both_endings_ouvert() {
    for seed in 0..500u64 {
        let mut rng = Rng::new(seed ^ 0xD44);
        let mut s = rand_spec(&mut rng, seed);
        s.resolved = false;
        s.density[Voice::Melody as usize] = 0.9;
        let p = Composer::new().compose_phrase(&s);
        assert_eq!(p.ending, Ending::Ouvert, "seed {seed}");
        let endings = ending_notes(&s, &p.notes);
        let iv = s.mode.intervals();
        for e in &endings {
            assert!(
                [iv[1], iv[4]].contains(&pc(&s, e)),
                "seed {seed}: unresolved cadence on pc {}",
                pc(&s, e),
            );
        }
    }
}

#[test]
fn notes_within_instrument_range() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0xE55);
        let s = rand_spec(&mut rng, seed);
        let p = Composer::new().compose_phrase(&s);
        for n in &p.notes {
            assert!(
                n.patch.instrument().in_range(n.midi as u8),
                "seed {seed}: {:?} note {} outside {:?} range",
                n.voice, n.midi, n.patch,
            );
        }
    }
}

#[test]
fn no_bar_repeats_twice_running() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0xF66);
        let s = rand_spec(&mut rng, seed);
        let p = Composer::new().compose_phrase(&s);
        let bar = bar_samples(&s);
        let m = melody(&p.notes);
        // consecutive melody-bar contents must differ
        let mut bars: Vec<Vec<(u8, u32)>> = vec![Vec::new(); 8];
        for n in m {
            let b = (n.at_sample / bar).min(7) as usize;
            bars[b].push((n.midi as u8, n.dur_samples));
        }
        for w in bars.windows(2) {
            assert!(
                w[0] != w[1] || w[0].is_empty() && w[1].is_empty(),
                "seed {seed}: identical consecutive bars",
            );
        }
    }
}

#[test]
fn density_is_monotonic_in_note_count() {
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed ^ 0x077);
        let mut s = rand_spec(&mut rng, seed);
        let mut last = 0usize;
        for step in 0..=5 {
            s.density = [step as f32 / 5.0; 8];
            // a fresh composer each time: the arranger primes on the
            // first phrase, so applied == requested
            let p = Composer::new().compose_phrase(&s);
            assert!(
                p.notes.len() >= last,
                "seed {seed}: density {} gave fewer notes than before",
                s.density[0],
            );
            last = p.notes.len();
        }
    }
}

#[test]
fn same_seed_same_notes_and_hash() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0x088);
        let s = rand_spec(&mut rng, seed);
        let a: Phrase = Composer::new().compose_phrase(&s);
        let b: Phrase = Composer::new().compose_phrase(&s);
        assert_eq!(a.notes, b.notes, "seed {seed}");
        assert_eq!(notes_hash(&a.notes), notes_hash(&b.notes), "seed {seed}");
    }
}

#[test]
fn step_leap_distribution_70_20_10_within_8_points() {
    let mut rng = Rng::new(0x5EED);
    let mut resolve: Option<i8> = None;
    let (mut steps, mut thirds, mut leaps) = (0u64, 0u64, 0u64);
    let mut drawn = 0u64;
    while drawn < 20_000 {
        // a resolution step belongs to its leap, not the distribution
        if resolve.is_none() {
            match free_step(&mut rng, &mut resolve).abs() {
                1 => steps += 1,
                2 => thirds += 1,
                _ => leaps += 1,
            }
            drawn += 1;
        } else {
            free_step(&mut rng, &mut resolve);
        }
    }
    let pct = |n: u64| n as f64 / drawn as f64 * 100.0;
    assert!((pct(steps) - 70.0).abs() <= 8.0, "steps {}", pct(steps));
    assert!((pct(thirds) - 20.0).abs() <= 8.0, "thirds {}", pct(thirds));
    assert!((pct(leaps) - 10.0).abs() <= 8.0, "leaps {}", pct(leaps));
    // every leap is at most a 5th: |step| <= 4 degrees
    let mut rng = Rng::new(0x5EED2);
    let mut resolve = None;
    for _ in 0..20_000 {
        assert!(free_step(&mut rng, &mut resolve).abs() <= 4);
    }
}

#[test]
fn arranger_changes_at_most_one_voice() {
    let mut c = Composer::new();
    let mut rng = Rng::new(0xA44A);
    let mut s = rand_spec(&mut rng, 0);
    for i in 0..1000u64 {
        s.seed = i;
        s.density = [
            rng.next_f32(), rng.next_f32(), rng.next_f32(), rng.next_f32(),
            rng.next_f32(), rng.next_f32(), rng.next_f32(), rng.next_f32(),
        ];
        let p = c.compose_phrase(&s);
        assert!(p.voices_changed <= 1, "phrase {i}: {}", p.voices_changed);
    }
}

#[test]
fn tintinnabuli_voice_always_in_triad() {
    for seed in 0..1000u64 {
        let mut rng = Rng::new(seed ^ 0x123);
        let s = rand_spec(&mut rng, seed);
        let p = Composer::new().compose_phrase(&s);
        let iv = s.mode.intervals();
        let triad = [iv[0], iv[2], iv[4]];
        for n in &p.notes {
            if n.voice == Voice::Counter {
                assert!(
                    triad.contains(&pc(&s, n)),
                    "seed {seed}: T-voice on pc {}", n.midi,
                );
            }
        }
    }
}

#[test]
fn cause_motif_shape() {
    // first interval a rising 4th, second a rising 5th, then the
    // fall to degree 7 - checked in Ionian where the semitones are
    // exactly 5 and 7.
    for seed in 0..200u64 {
        let mut s = spec(seed);
        s.motif = MotifId::Cause;
        s.mode = Mode::Ionian;
        let p = Composer::new().compose_phrase(&s);
        let m = melody(&p.notes);
        assert!(m.len() >= 3, "seed {seed}: cause motif unplayed");
        let iv = |a: &NoteEvent, b: &NoteEvent| b.midi - a.midi;
        assert_eq!(iv(&m[0], &m[1]), 5.0, "seed {seed}: first interval");
        assert_eq!(iv(&m[1], &m[2]), 7.0, "seed {seed}: second interval");
        if m.len() > 3 {
            assert!(iv(&m[2], &m[3]) < 0.0, "seed {seed}: hangs on the 7");
        }
    }
}

#[test]
fn tempo_bands_enforced() {
    for &section in &[Section::Ambient, Section::Theme, Section::Build,
        Section::Peak, Section::Festive]
    {
        let (lo, hi) = tempo_band(section).unwrap();
        for seed in 0..50u64 {
            let mut s = spec(seed);
            s.section = section;
            // an absurd tempo clamps to the band edge - the phrase is
            // identical to one asked at the edge itself
            s.tempo_bpm = 1.0;
            let slow = Composer::new().compose_phrase(&s);
            s.tempo_bpm = lo;
            let at_lo = Composer::new().compose_phrase(&s);
            assert_eq!(slow.notes, at_lo.notes, "{:?} seed {seed} low", section);
            s.tempo_bpm = 999.0;
            let fast = Composer::new().compose_phrase(&s);
            s.tempo_bpm = hi;
            let at_hi = Composer::new().compose_phrase(&s);
            assert_eq!(fast.notes, at_hi.notes, "{:?} seed {seed} high", section);
        }
    }
}
