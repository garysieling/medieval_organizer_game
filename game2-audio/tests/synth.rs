//! The cue voices and the offline lockstep (#1493).
use game2_audio::engine::{Bus, Engine, Source, VoiceParams};
use game2_audio::host::AudioHost;
use game2_audio::synth::SynthKind;

/// Every legacy cue plays something and never makes a NaN.
#[test]
fn legacy_cues_render_nonsilent_and_no_nan() {
    for name in ["rain", "anvil", "festival", "wonder", "bell", "snowballs", "splash", "mood"] {
        let mut e = Engine::new(44100, 7);
        e.play(Source::Synth(SynthKind::from_name(name)), Bus::Sfx, VoiceParams::default());
        let mut buf = vec![0.0f32; 44100 * 2];
        e.render(&mut buf);
        assert!(buf.iter().all(|s| s.is_finite()), "{name} emitted a NaN");
        let peak = buf.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.01, "{name} rendered silent (peak {peak})");
    }
}

/// 60 fps lockstep: the per-frame sample count carries its fraction so
/// a long run lands on exactly `frames * rate / 60`. 16000/60 isn't
/// whole - that's what the carry is for.
#[test]
fn lockstep_sample_count_exact_over_3600_frames() {
    let sr = 16000u32;
    let mut host = AudioHost::offline(sr);
    let mut total = 0usize;
    for _ in 0..3600 {
        let n = host.frame_samples();
        let mut buf = vec![0.0f32; n * 2];
        host.render_block(&mut buf);
        total += n;
    }
    assert_eq!(total, 3600 * sr as usize / 60, "lockstep drifted off 60 fps");
}
