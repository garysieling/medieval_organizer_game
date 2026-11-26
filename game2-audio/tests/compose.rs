//! Property tests for the composer (M4): 1,000 seeded phrases run
//! through every invariant the design promises - mode purity, leap
//! rules, ouvert/clos endings, instrument ranges, the 8-bar rule.

use game2_audio::compose::{
    free_step, notes_hash, tempo_band, Composer, Ending, Meter, MotifId, NoteEvent,
    Phrase, PhraseSpec, Section, Voice,
};
use game2_audio::instruments::PatchId;
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
fn quiet_request_silences_now() {
    // an explicit all-zero spec mutes the phrase at once - the
    // one-voice-per-phrase walk is for fades, not for a hard quiet
    let mut c = Composer::new();
    let mut s = spec(0);
    let loud = c.compose_phrase(&s);
    assert!(!loud.notes.is_empty());
    s.density = [0.0; 8];
    let p = c.compose_phrase(&s);
    assert!(p.notes.is_empty(), "quiet spec still emitted {} notes", p.notes.len());
}

#[test]
fn free_notes_fill_dense_bars() {
    // above .45 the figure compresses and free notes fill the bar's
    // tail; below it the figure alone fills the bar
    let bar0_count = |d: f32| -> usize {
        let mut s = spec(7);
        s.density = [d; 8];
        let bar = bar_samples(&s);
        melody(&Composer::new().compose_phrase(&s).notes)
            .iter()
            .filter(|n| n.at_sample < bar)
            .count()
    };
    let sparse = bar0_count(0.3);
    let dense = bar0_count(0.8);
    assert!(sparse > 0, "motif unstated at low density");
    assert!(dense > sparse, "dense {dense} should out-sing sparse {sparse}");
}

#[test]
fn lead_sings_when_the_motif_fits() {
    // Old Faith's cell sits inside the psaltery's compass - its own
    // lead must sing it, coverage or no
    let mut s = spec(10);
    s.mode = Mode::Lydian;
    s.motif = MotifId::OldFaith;
    let p = Composer::new().compose_phrase(&s);
    let m = melody(&p.notes);
    assert!(!m.is_empty(), "old faith unplayed");
    assert_eq!(m[0].patch, PatchId::Psaltery);
    // and where the lead can't hold the cell - Cause's 12st span on a
    // 10st fife - a substitute steps in instead
    s.motif = MotifId::Cause;
    let p = Composer::new().compose_phrase(&s);
    let m = melody(&p.notes);
    assert!(!m.is_empty(), "cause unplayed");
    assert!(
        m.iter().all(|n| n.patch != PatchId::Fife),
        "cause motif on a fife that can't hold it",
    );
}

#[test]
fn cadence_shift_keeps_the_repeat_sounding() {
    // seed 6 D Lydian Village: the A-cadence must not pick a repeat
    // octave that drops every note of punctus B's body
    let mut s = spec(6);
    s.mode = Mode::Lydian;
    let p = Composer::new().compose_phrase(&s);
    let bar = bar_samples(&s);
    let m = melody(&p.notes);
    for b in 4..7u64 {
        assert!(
            m.iter().any(|n| n.at_sample / bar == b),
            "punctus B body bar {b} went silent",
        );
    }
}

/// Melody notes that sound within one bar.
fn bar_line(s: &PhraseSpec, notes: &[NoteEvent], b: u64) -> Vec<NoteEvent> {
    let bar = bar_samples(s);
    melody(notes)
        .into_iter()
        .filter(|n| n.at_sample / bar == b)
        .collect()
}

#[test]
fn the_repeat_restates_the_strain() {
    // punctus B opens with the strain's opening figure transposed a
    // step or two: bar 4's first notes parallel bar 0's at a fixed
    // shift (the mode's half-steps wobble it a semitone or two)
    for motif in [
        MotifId::Village,
        MotifId::Lord,
        MotifId::Church,
        MotifId::OldFaith,
        MotifId::Road,
        MotifId::Feast,
    ] {
        let mut ok = 0;
        let mut n = 0;
        for seed in 0..200u64 {
            let mut s = spec(seed);
            s.motif = motif;
            let m = melody(&Composer::new().compose_phrase(&s).notes);
            let (a, b) = (bar_line(&s, &m, 0), bar_line(&s, &m, 4));
            if a.len() < 4 || b.len() < 4 {
                continue;
            }
            n += 1;
            let diffs: Vec<f32> =
                a.iter().zip(&b).take(4).map(|(x, y)| y.midi - x.midi).collect();
            let hi = diffs.iter().cloned().fold(f32::MIN, f32::max);
            let lo = diffs.iter().cloned().fold(f32::MAX, f32::min);
            if hi - lo <= 4.0 {
                ok += 1;
            }
        }
        assert!(ok * 20 >= n * 19, "{motif:?}: {ok}/{n} restatements");
    }
}

#[test]
fn the_repeat_is_not_a_copy() {
    // B re-ornaments the strain: some body bar always differs from
    // its A twin (the old verbatim double is gone)
    for motif in MotifId::ALL {
        for seed in 0..200u64 {
            let mut s = spec(seed);
            s.motif = motif;
            let m = melody(&Composer::new().compose_phrase(&s).notes);
            let bar = bar_samples(&s);
            let line = |b: u64| {
                bar_line(&s, &m, b)
                    .iter()
                    .map(|n| (n.midi, n.at_sample % bar, n.dur_samples))
                    .collect::<Vec<_>>()
            };
            assert!(
                (0..3u64).any(|k| line(k) != line(4 + k)),
                "{motif:?} seed {seed}: punctus B is a verbatim copy",
            );
        }
    }
}

#[test]
fn phrase_apex_lives_in_the_repeat() {
    // the line's extreme - the top of a rising arc, the floor of a
    // falling one - sounds last inside punctus B's strain, not spent
    // early in A's
    for motif in MotifId::ALL {
        let d = motif.motif().degrees;
        let dir = if *d.last().unwrap() < d[0] { -1.0f32 } else { 1.0 };
        let mut late = 0;
        let mut n = 0;
        for seed in 0..200u64 {
            let mut s = spec(seed);
            s.motif = motif;
            let m = melody(&Composer::new().compose_phrase(&s).notes);
            if m.len() < 6 {
                continue;
            }
            n += 1;
            let bar = bar_samples(&s);
            let extr = m
                .iter()
                .map(|x| x.midi)
                .fold(if dir > 0.0 { f32::MIN } else { f32::MAX }, |a, b| {
                    if dir > 0.0 { a.max(b) } else { a.min(b) }
                });
            let last = m
                .iter()
                .rfind(|x| x.midi == extr)
                .map(|x| x.at_sample / bar)
                .unwrap_or(99);
            if last >= 4 {
                late += 1;
            }
        }
        assert!(late * 3 >= n * 2, "{motif:?}: apex late in {late}/{n}");
    }
}

#[test]
fn the_body_walks_into_the_cadence() {
    // the closing punctus's tail approaches its ending: the last
    // body-bar note sits within a fourth of the clos
    for motif in MotifId::ALL {
        let mut near = 0;
        let mut n = 0;
        for seed in 0..200u64 {
            let mut s = spec(seed);
            s.motif = motif;
            let m = melody(&Composer::new().compose_phrase(&s).notes);
            let bar = bar_samples(&s);
            let cad = m.iter().find(|x| x.at_sample / bar == 7);
            let tail = m.iter().rfind(|x| (4..=6).contains(&(x.at_sample / bar)));
            let (Some(c), Some(t)) = (cad, tail) else { continue };
            n += 1;
            if (c.midi - t.midi).abs() <= 5.0 {
                near += 1;
            }
        }
        assert!(near * 20 >= n * 17, "{motif:?}: {near}/{n} cadence approaches");
    }
}

#[test]
fn ground_and_strum_follow_the_meter() {
    // dense ground hits every pulse of whatever meter it plays in;
    // the lute's second strum lands mid-bar
    for meter in [Meter::SIX_EIGHT, Meter::TWO_FOUR] {
        let mut s = spec(0);
        s.meter = meter;
        s.density = [0.0; 8];
        s.density[Voice::Ground as usize] = 0.8;
        s.density[Voice::Harmony as usize] = 0.8;
        let p = Composer::new().compose_phrase(&s);
        let bar = bar_samples(&s);
        let beat = bar / meter.num as u64;
        for b in 0..meter.num as u64 {
            assert!(
                p.notes.iter().any(|n| {
                    n.voice == Voice::Ground && n.at_sample == b * beat
                }),
                "{:?} ground misses beat {b}",
                meter,
            );
        }
        assert!(
            p.notes.iter().any(|n| {
                n.voice == Voice::Harmony
                    && n.at_sample == (meter.num as u64 / 2) * beat
            }),
            "{:?} strum misses mid-bar",
            meter,
        );
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

#[test]
fn climax_lands_inside_the_strain() {
    // the strain's peak should be reached in the body, not saved
    // entirely for the last bar's cadence: across seeds most phrases
    // touch their high point before the final bar (#1618)
    let n = 60u64;
    let mut inside = 0;
    for seed in 0..n {
        let mut rng = Rng::new(seed ^ 0x0C11);
        let mut s = rand_spec(&mut rng, seed);
        s.density[Voice::Melody as usize] = 0.9;
        let m = melody(&Composer::new().compose_phrase(&s).notes);
        if m.is_empty() {
            continue;
        }
        let bar = bar_samples(&s);
        let peak = m.iter().map(|x| x.midi).fold(f32::MIN, f32::max);
        let last_peak_bar = m
            .iter()
            .filter(|x| x.midi == peak)
            .map(|x| x.at_sample / bar)
            .max()
            .unwrap();
        if last_peak_bar < 7 {
            inside += 1;
        }
    }
    assert!(
        inside * 2 >= n as usize,
        "only {inside}/{n} phrases climax before the last bar"
    );
}

#[test]
fn cadence_notes_take_the_downbeat() {
    // an ending note opens its bar - nothing so long slips onto a
    // weak sixteenth
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed ^ 0x0CAD);
        let s = rand_spec(&mut rng, seed);
        let bar = bar_samples(&s);
        for n in melody(&Composer::new().compose_phrase(&s).notes) {
            if n.dur_samples as u64 >= bar {
                assert_eq!(
                    n.at_sample % bar,
                    0,
                    "seed {seed}: cadence-length note off the downbeat"
                );
            }
        }
    }
}

#[test]
fn the_line_moves_mostly_by_step() {
    // a singable line steps: authored cells like Cause leap by design,
    // so the bound runs over the ensemble - on average at least half
    // the intervals stay within a third (#1618)
    let mut near = 0;
    let mut tot = 0;
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed ^ 0x0573);
        let s = rand_spec(&mut rng, seed);
        let m = melody(&Composer::new().compose_phrase(&s).notes);
        for w in m.windows(2) {
            if (w[1].midi - w[0].midi).abs() <= 4.0 {
                near += 1;
            }
            tot += 1;
        }
    }
    assert!(
        near * 2 >= tot,
        "only {near}/{tot} intervals within a third"
    );
}

#[test]
fn the_line_does_not_see_saw() {
    // an immediate A-B-A retraction is a figure's turn at its arch -
    // and OldFaith circles by design - so the bound runs over the
    // ensemble: on average no more than a fifth of the line seesaws
    // (#1618)
    let mut see = 0;
    let mut notes = 0;
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed ^ 0x05E5);
        let s = rand_spec(&mut rng, seed);
        let m = melody(&Composer::new().compose_phrase(&s).notes);
        for w in m.windows(3) {
            if w[0].midi == w[2].midi && w[0].midi != w[1].midi {
                see += 1;
            }
        }
        notes += m.len();
    }
    assert!(
        see * 5 <= notes,
        "{see} see-saws across {notes} melody notes"
    );
}

#[test]
fn downbeats_carry_the_accent() {
    // bar onsets sing stronger than the notes between them - the
    // velocity contour marks the meter (#1618)
    let mut down = 0f32;
    let mut d_n = 0usize;
    let mut off = 0f32;
    let mut o_n = 0usize;
    for seed in 0..40u64 {
        let mut rng = Rng::new(seed ^ 0x0ACC);
        let s = rand_spec(&mut rng, seed);
        let bar = bar_samples(&s);
        for n in melody(&Composer::new().compose_phrase(&s).notes) {
            if n.at_sample % bar == 0 {
                down += n.vel;
                d_n += 1;
            } else {
                off += n.vel;
                o_n += 1;
            }
        }
    }
    assert!(d_n > 0 && o_n > 0);
    assert!(
        down / d_n as f32 > off / o_n as f32,
        "downbeat {:.3} not above off-beat {:.3}",
        down / d_n as f32,
        off / o_n as f32
    );
}
