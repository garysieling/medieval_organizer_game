//! A2 (#1498): the sampler - OGG/WAV decode, zone selection, pitch
//! playback, seamless loops, the missing-asset fallback and the
//! desktop directory loader.
use game2_audio::engine::{Bus, Engine, Source, VoiceParams};
use game2_audio::sampler::{self, Patch, PatchId, Sample, SampleBank, SampleId, Zone};

const SR: u32 = 44100;
const SINE_OGG: &[u8] = include_bytes!("fixtures/sine440.ogg");

fn mono_render(e: &mut Engine, frames: usize) -> Vec<f32> {
    let mut buf = vec![0.0f32; frames * 2];
    e.render(&mut buf);
    buf.chunks(2).map(|s| (s[0] + s[1]) * 0.5).collect()
}

fn sine_pcm(hz: f32, frames: usize, sr: u32) -> Vec<f32> {
    (0..frames).map(|i| (i as f32 * hz * std::f32::consts::TAU / sr as f32).sin()).collect()
}

/// A bank with one patch over the whole keyboard, rooted at C4.
fn one_patch(sample: Sample, name: &str) -> SampleBank {
    let mut bank = SampleBank::new();
    bank.samples.push(sample);
    bank.patches.push((name.into(), Patch { zones: vec![Zone {
        key_lo: 0, key_hi: 127, root: 60,
        loop_start: None, loop_end: None, sample: SampleId(0),
    }] }));
    bank
}

mod sampler_tests {
    use super::*;

        #[test]
    fn decodes_ogg_fixture_length_and_rate() {
        let s = sampler::decode(SINE_OGG).expect("the fixture decodes");
        assert_eq!(s.sr, SR);
        assert!((s.pcm.len() as i64 - SR as i64).unsigned_abs() < 2000,
            "1 s at 44.1 kHz: got {} frames", s.pcm.len());
    }

        #[test]
    fn decodes_wav() {
        // hand-roll a 16-bit mono WAV: RIFF + fmt + data
        let pcm = sine_pcm(220.0, 2205, SR);
        let data_bytes = pcm.len() * 2;
        let mut w = Vec::new();
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36u32 + data_bytes as u32).to_le_bytes());
        w.extend_from_slice(b"WAVE");
        w.extend_from_slice(b"fmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());      // PCM
        w.extend_from_slice(&1u16.to_le_bytes());      // mono
        w.extend_from_slice(&SR.to_le_bytes());
        w.extend_from_slice(&(SR * 2).to_le_bytes());  // byte rate
        w.extend_from_slice(&2u16.to_le_bytes());      // block align
        w.extend_from_slice(&16u16.to_le_bytes());     // bits
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(data_bytes as u32).to_le_bytes());
        for x in &pcm { w.extend_from_slice(&((x * 30000.0) as i16).to_le_bytes()); }
        let s = sampler::decode(&w).expect("plain 16-bit WAV decodes");
        assert_eq!(s.sr, SR);
        assert_eq!(s.pcm.len(), 2205);
        assert!(s.pcm.iter().all(|x| x.abs() <= 1.0));
    }

        #[test]
    fn zone_selection_nearest_root_within_3_semitones() {
        // two overlapping zones: the nearer root wins the note
        let patch = Patch { zones: vec![
            Zone { key_lo: 40, key_hi: 100, root: 60,
                   loop_start: None, loop_end: None, sample: SampleId(0) },
            Zone { key_lo: 55, key_hi: 90, root: 64,
                   loop_start: None, loop_end: None, sample: SampleId(1) },
        ] };
        assert_eq!(patch.zone_for(62).unwrap().sample, SampleId(0), "nearer 60");
        assert_eq!(patch.zone_for(65).unwrap().sample, SampleId(1), "nearer 64");
        assert_eq!(patch.zone_for(50).unwrap().sample, SampleId(0), "only 60 covers");
    }

        #[test]
    fn out_of_range_pitch_returns_none() {
        let bank = one_patch(Sample { pcm: sine_pcm(440.0, 4410, SR), sr: SR }, "bell");
        let patch = Patch { zones: vec![Zone {
            key_lo: 58, key_hi: 62, root: 60,
            loop_start: None, loop_end: None, sample: SampleId(0) }] };
        assert!(patch.zone_for(70).is_none(), "70 is outside 58..=62");
        assert!(bank.patch_id("bell").is_some());
    }

        #[test]
    fn pitch_shift_up_5_semitones_in_tune_within_10_cents() {
        // the fixture is 440 Hz at root 60; midi 65 shifts 2^(5/12)
        let mut e = Engine::new(SR, 1);
        let pid = e.bank().patch_id("x"); // nothing yet
        assert!(pid.is_none());
        let mut bank = SampleBank::new();
        bank.insert_bytes("sine", SINE_OGG).unwrap();
        e.set_bank(bank);
        let pid = e.bank().patch_id("sine").unwrap();
        e.play(Source::Sample { id: pid, midi: 65 }, Bus::Sfx, VoiceParams::default());
        let out = mono_render(&mut e, (SR / 2) as usize);

        // autocorrelation: first big peak's lag is the period
        let expect = 440.0 * 2f32.powf(5.0 / 12.0);
        let (lo, hi) = ((SR as f32 / (expect * 1.2)) as usize, (SR as f32 / (expect * 0.8)) as usize);
        let mut best = (0usize, f32::MIN);
        for lag in lo..=hi {
            let c: f32 = (0..out.len() - lag).map(|i| out[i] * out[i + lag]).sum();
            if c > best.1 { best = (lag, c); }
        }
        let measured = SR as f32 / best.0 as f32;
        let cents = 1200.0 * (measured / expect).log2().abs();
        assert!(cents < 10.0, "measured {measured:.1} Hz vs {expect:.1} Hz = {cents:.1} cents");
    }

        #[test]
    fn loop_is_seamless_no_click() {
        // a bed loops its whole buffer; the wrap must not click
        let bank = one_patch(Sample { pcm: sine_pcm(440.0, SR as usize, SR), sr: SR }, "wind");
        let mut e = Engine::new(SR, 1);
        e.set_bank(bank);
        let pid = e.bank().patch_id("wind").unwrap();
        e.play(Source::Bed { id: pid }, Bus::Ambience, VoiceParams::default());
        let out = mono_render(&mut e, (SR * 5 / 2) as usize); // 2.5 s > one wrap

        let attack = (SR as f32 * 0.005) as usize + 10;
        let max_delta = out[attack..].windows(2)
            .map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
        // 440 Hz full-scale slews ~0.063/frame; a click at the wrap would
        // jump far past that
        assert!(max_delta < 0.15, "click at the loop wrap: delta {max_delta}");
        let rms_late = (out[SR as usize..].iter().map(|x| x * x).sum::<f32>()
            / (out.len() - SR as usize) as f32).sqrt();
        assert!(rms_late > 0.3, "the loop keeps sounding past the wrap");
    }

        #[test]
    fn missing_asset_logs_once_and_plays_silence() {
        let mut e = Engine::new(SR, 1);
        let ghost = PatchId(99);
        for _ in 0..3 {
            e.play(Source::Bed { id: ghost }, Bus::Ambience, VoiceParams::default());
            e.play(Source::Sample { id: ghost, midi: 60 }, Bus::Sfx, VoiceParams::default());
        }
        let out = mono_render(&mut e, 2048);
        assert!(out.iter().all(|x| *x == 0.0), "a missing asset is silent");
        assert!(e.missing_logged(ghost), "its id logged");
        assert!(!e.missing_logged(PatchId(1)), "nothing else logged");
        // and a real asset still plays afterwards
        let mut bank = SampleBank::new();
        bank.insert_bytes("sine", SINE_OGG).unwrap();
        e.set_bank(bank);
        let pid = e.bank().patch_id("sine").unwrap();
        e.play(Source::Sample { id: pid, midi: 60 }, Bus::Sfx, VoiceParams::default());
        let out = mono_render(&mut e, 4410);
        assert!(out.iter().any(|x| x.abs() > 0.05), "the bank's sample sounds");
    }

        #[test]
    fn desktop_loader_reads_directory() {
        let dir = std::env::temp_dir().join(format!("game2-bank-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("beds")).unwrap();
        std::fs::create_dir_all(dir.join("_incoming")).unwrap();
        std::fs::write(dir.join("beds/river.ogg"), SINE_OGG).unwrap();
        std::fs::write(dir.join("beds/notes.txt"), b"not audio").unwrap();
        std::fs::write(dir.join("_incoming/draft.ogg"), SINE_OGG).unwrap();
        let bank = sampler::load_dir(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(bank.patch_id("river").is_some(),
            "beds/river.ogg becomes the 'river' patch");
        assert_eq!(bank.patches.len(), 1,
            "_incoming staging and non-audio files don't load");
    }
}
