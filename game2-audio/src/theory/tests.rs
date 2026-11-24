use super::*;

/// D Lydian's signature is F# C# G# - the design's worked example that
/// "the key signature really changes" when a verdict moves the mode.
#[test]
fn d_phrygian_has_exactly_bb_and_eb() {
    let sig = key_signature(2, Mode::Phrygian);
    assert!(sig.sharps.is_empty());
    assert_eq!(sig.flats, vec![10, 3]); // Bb, Eb in signature order
}

#[test]
fn key_signatures_for_all_six_modes_on_d() {
    let expect: [(&[u8], &[u8]); 6] = [
        (&[6, 1, 8], &[]), // Lydian: F# C# G#
        (&[6, 1], &[]),    // Ionian: F# C#
        (&[6], &[]),       // Mixolydian: F#
        (&[], &[]),        // Dorian: none
        (&[], &[10]),      // Aeolian: Bb
        (&[], &[10, 3]),   // Phrygian: Bb Eb
    ];
    for (mode, (sharps, flats)) in Mode::ALL.iter().zip(expect.iter()) {
        let sig = key_signature(2, *mode);
        assert_eq!(&sig.sharps[..], &sharps[..], "{mode:?} sharps");
        assert_eq!(&sig.flats[..], &flats[..], "{mode:?} flats");
    }
}

#[test]
fn ladder_clamps_at_ends() {
    assert_eq!(Mode::Lydian.steps_from(3), Mode::Lydian);
    assert_eq!(Mode::Phrygian.steps_from(-3), Mode::Phrygian);
    assert_eq!(Mode::Dorian.steps_from(2), Mode::Ionian); // a win brightens
    assert_eq!(Mode::Mixolydian.steps_from(2), Mode::Lydian);
    assert_eq!(Mode::Dorian.steps_from(-1), Mode::Aeolian); // a loss darkens
    assert_eq!(Mode::Dorian.steps_from(0), Mode::Dorian);
}

#[test]
fn triad_stacks_thirds_in_every_mode() {
    for mode in Mode::ALL {
        for deg in 1..=7i8 {
            let t = triad(60, mode, Degree(deg));
            // Every step of the stack is a diatonic third: 3 or 4 semitones.
            assert!((3..=4).contains(&(t[1] - t[0])), "{mode:?} deg {deg}");
            assert!((3..=4).contains(&(t[2] - t[1])), "{mode:?} deg {deg}");
        }
    }
}

#[test]
fn every_t_voice_note_is_in_the_triad() {
    for mode in Mode::ALL {
        let t = triad(62, mode, Degree(1));
        let pcs: Vec<u8> = t.iter().map(|n| n % 12).collect();
        for note in 36..96u8 {
            let tv = tintinnabuli_t(note, t);
            assert!(pcs.contains(&(tv % 12)), "{mode:?} note {note} -> {tv}");
            assert!((tv as i32 - note as i32).abs() <= 6);
        }
    }
}

#[test]
fn euclid_3_8_and_5_8() {
    assert_eq!(
        euclid(3, 8),
        vec![true, false, false, true, false, false, true, false]
    );
    assert_eq!(
        euclid(5, 8),
        vec![true, false, true, true, false, true, true, false]
    );
}

#[test]
fn euclid_pulse_count_equals_k() {
    for n in 1..=16 {
        for k in 0..=n {
            assert_eq!(euclid(k, n).iter().filter(|&&b| b).count(), k, "E({k},{n})");
        }
    }
}

#[test]
fn motif_inversion_twice_is_identity() {
    let m = Motif::new(&[1, 4, 8, 7, 6], &[4, 4, 8, 4, 4]);
    assert_eq!(m.invert().invert(), m);
}

#[test]
fn motif_augment_doubles_duration() {
    let m = Motif::new(&[1, 3, 5], &[4, 4, 8]);
    let aug = m.augment(2);
    let orig: u32 = m.rhythm.iter().map(|&r| r as u32).sum();
    let doubled: u32 = aug.rhythm.iter().map(|&r| r as u32).sum();
    assert_eq!(doubled, orig * 2);
    assert_eq!(aug.degrees, m.degrees); // pitches untouched
}

#[test]
fn ranges_are_sane_and_ordered() {
    for i in Instrument::ALL {
        let (lo, hi) = i.range();
        assert!(lo < hi, "{i:?}");
        assert!(lo >= 21 && hi <= 108, "{i:?} outside piano midi");
    }
    let (s, _) = Instrument::RecorderSoprano.range();
    let (a, _) = Instrument::RecorderAlto.range();
    assert!(s > a, "soprano recorder must sit above alto");
}
