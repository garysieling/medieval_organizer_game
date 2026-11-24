//! M2 (#1499): the instrument patches. Every row of the design's
//! Instruments table renders across its range without NaN or clipping,
//! the pitched ones hold tune within 10 cents, the synth voices are
//! deterministic, and the committed assets still ledger under budget.
use game2_audio::engine::{Bus, Engine, VoiceParams};
use game2_audio::instruments::{self, PatchId};
use game2_audio::sampler::{self, SampleBank};
use game2_audio::synth::midi_hz;
use std::path::PathBuf;

const SR: u32 = 44100;

fn sound_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/sound")
}

/// The real bank: every committed asset decoded, all instrument
/// patches registered.
fn bank() -> SampleBank { sampler::load_dir(&sound_dir()) }

/// Render `frames` mono samples of `midi` on patch `id` at `gain`.
/// The engine owns its bank, so each render moves a clone in.
fn render_note(bank: &SampleBank, id: PatchId, midi: u8, gain: f32, frames: usize) -> Vec<f32> {
    let mut e = Engine::new(SR, 42);
    let src = instruments::source(id, bank, midi);
    e.set_bank(bank.clone());
    e.play(src, Bus::Music, VoiceParams { gain, ..Default::default() });
    let mut buf = vec![0.0f32; frames * 2];
    e.render(&mut buf);
    buf.chunks(2).map(|s| (s[0] + s[1]) * 0.5).collect()
}

/// First strong autocorrelation peak in one slice, in a lag window
/// around the expected period (a pure tone correlates nearly as well
/// at twice its period, so the first local peak within 80% of the max
/// wins over the global max). Parabolic fit for sub-lag precision.
fn slice_hz(pcm: &[f32], expect: f32) -> Option<f32> {
    let sr = SR as f32;
    let l_lo = (sr / (expect * 1.4)) as usize;
    let l_hi = ((sr / (expect * 0.6)) as usize).min(pcm.len() - 1);
    if l_lo >= l_hi { return None; }
    let corr = |lag: usize| -> f32 {
        let mut c = 0.0f32;
        for i in 0..pcm.len() - lag {
            c += pcm[i] * pcm[i + lag];
        }
        c
    };
    let cs: Vec<f32> = (l_lo..=l_hi).map(&corr).collect();
    let max_c = cs.iter().copied().fold(0.0f32, f32::max);
    if max_c <= 0.0 { return None; }
    let mut best_lag = l_lo;
    for i in 1..cs.len() - 1 {
        if cs[i] >= max_c * 0.8 && cs[i] >= cs[i - 1] && cs[i] >= cs[i + 1] {
            best_lag = l_lo + i;
            break;
        }
    }
    let lag = best_lag as f32 + if best_lag > l_lo && best_lag < l_hi {
        let (c0, c1, c2) = (corr(best_lag - 1), corr(best_lag), corr(best_lag + 1));
        let d = c0 - 2.0 * c1 + c2;
        if d.abs() > 1e-9 { (0.5 * (c0 - c2) / d).clamp(-0.5, 0.5) } else { 0.0 }
    } else { 0.0 };
    Some(sr / lag)
}

/// Median detected pitch over ~0.2 s slices - vibrato'd samples (the
/// vielle's susvib violin, the bass recorder's susvib lows) wobble the
/// instantaneous pitch; the median lands on the sounding center.
fn detect_hz(pcm: &[f32], expect: f32) -> f32 {
    let win = SR as usize / 5;
    let mut est: Vec<f32> = pcm
        .chunks(win)
        .filter_map(|c| slice_hz(c, expect))
        .collect();
    est.sort_by(|a, b| a.partial_cmp(b).unwrap());
    est.get(est.len() / 2).copied().unwrap_or(0.0)
}

mod instrument_tests {
    use super::*;

    #[test]
    fn every_patch_has_a_range_and_renders() {
        let bank = bank();
        for id in PatchId::ALL {
            let inst = id.instrument();
            let (lo, hi) = inst.range();
            assert!(lo < hi && hi <= 127, "{id:?} has a sane range");
            // step across the range including both ends
            let mut midis: Vec<u8> = (lo..=hi).step_by(4).collect();
            if *midis.last().unwrap() != hi { midis.push(hi); }
            for midi in midis {
                for gain in [0.4, 0.7, 1.0] {
                    let pcm = render_note(&bank, id, midi, gain, (SR as usize) / 3);
                    let peak = pcm.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
                    assert!(pcm.iter().all(|x| x.is_finite()),
                        "{id:?} midi {midi} gain {gain}: NaN/Inf in output");
                    assert!(peak < 0.99, "{id:?} midi {midi} gain {gain}: peak {peak}");
                    assert!(peak > 1e-4,
                        "{id:?} midi {midi} gain {gain}: silent (peak {peak})");
                }
            }
        }
    }

    #[test]
    fn pitch_within_10_cents_across_range() {
        let bank = bank();
        // the clearly-pitched voices; the percussion rows and the
        // inharmonic bells are out of this one's scope. The chanter is
        // in: its zones carry +cents tuning so the flat FreePats set
        // lands on concert pitch
        let pitched = [
            PatchId::RecorderSoprano, PatchId::RecorderAlto,
            PatchId::RecorderTenor, PatchId::RecorderBass,
            PatchId::Fife, PatchId::Shawm,
            PatchId::BagpipeChanter, PatchId::BagpipeDrone,
            PatchId::Organ, PatchId::FolkHarp, PatchId::Psaltery,
            PatchId::Strumstick, PatchId::Lute, PatchId::Gittern,
            PatchId::HurdyGurdy, PatchId::ChoirPad,
        ];
        for id in pitched {
            let (lo, hi) = id.instrument().range();
            let span = (hi - lo) as usize;
            for midi in [lo, lo + (span / 4) as u8, lo + (span / 2) as u8,
                         lo + (3 * span / 4) as u8, hi] {
                // skip the attack, analyse the body of the note
                let pcm = render_note(&bank, id, midi, 0.7, SR as usize);
                let body = &pcm[(SR as usize) / 4..(SR as usize) * 4 / 5];
                let expect = midi_hz(midi);
                let got = detect_hz(body, expect);
                let cents = 1200.0 * (got / expect).log2().abs();
                assert!(cents <= 10.0,
                    "{id:?} midi {midi}: expected {expect:.1} Hz, got {got:.1} ({cents:.0} cents)");
            }
        }
        // the vielle's VSCO violin set is susvib all the way down: its
        // instantaneous pitch wobbles ± a few tens of cents by design,
        // so it verifies looser and only unshifted at its zone roots
        // (the engine's rate maths is what the sweep above exercises;
        // 30c still catches a wrong file or a broken zone map)
        for midi in [55u8, 60, 64, 69, 76, 88] {
            let pcm = render_note(&bank, PatchId::Vielle, midi, 0.7, SR as usize);
            let body = &pcm[(SR as usize) / 4..(SR as usize) * 4 / 5];
            let expect = midi_hz(midi);
            let got = detect_hz(body, expect);
            let cents = 1200.0 * (got / expect).log2().abs();
            assert!(cents <= 30.0,
                "Vielle midi {midi}: expected {expect:.1} Hz, got {got:.1} ({cents:.0} cents)");
        }
    }

    #[test]
    fn synth_patches_deterministic_for_same_seed() {
        let synths = [PatchId::Lute, PatchId::Gittern, PatchId::HurdyGurdy,
                      PatchId::ChoirPad, PatchId::Bell];
        let bank = bank();
        for id in synths {
            for midi in [48u8, 60, 72] {
                let a = render_note(&bank, id, midi, 0.7, SR as usize / 2);
                let b = render_note(&bank, id, midi, 0.7, SR as usize / 2);
                assert_eq!(a, b, "{id:?} midi {midi}: same seed, different render");
            }
        }
    }

    #[test]
    fn ledger_still_clean_with_instrument_assets() {
        let violations = game2_audio::ledger::check_ledger(&sound_dir());
        assert!(violations.is_empty(), "ledger violations: {violations:?}");
    }

    #[test]
    fn instrument_assets_under_8mb() {
        let dir = sound_dir().join("instruments");
        let bytes: u64 = std::fs::read_dir(&dir).expect("instruments dir")
            .flatten()
            .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();
        assert!(bytes <= 8 * 1024 * 1024,
            "instrument assets {bytes} bytes over the 8 MB cap");
    }
}
