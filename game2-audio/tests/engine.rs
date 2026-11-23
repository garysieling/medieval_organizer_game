//! Engine (DSP) tests for #1486 - the mixer is sound. Names are as the
//! ticket requires.
use game2_audio::engine::{Bus, Engine, Source, VoiceParams, MAX_VOICES};

const SR: u32 = 48000;

fn render(e: &mut Engine, frames: usize) -> Vec<f32> {
    let mut b = vec![0.0; frames * 2];
    e.render(&mut b);
    b
}

/// Total-power RMS: sqrt(mean(l^2 + r^2)) - equal-power panning keeps a
/// voice's energy whole across the stereo pair.
fn power_rms(buf: &[f32]) -> f32 {
    let e: f32 = buf.chunks(2).map(|c| c[0] * c[0] + c[1] * c[1]).sum();
    (e / (buf.len() / 2) as f32).sqrt()
}

fn sine(hz: f32, gain: f32) -> (Source, VoiceParams) {
    (Source::Sine { hz }, VoiceParams { gain, ..VoiceParams::default() })
}

/// A 440 Hz sine at gain 0.5 has amplitude 0.5, total-power RMS
/// 0.5/sqrt(2) = 0.3536 -> -9.03 dBFS.
#[test]
fn sine_rms_within_0_1_db() {
    let mut e = Engine::new(SR, 7);
    let (s, p) = sine(440.0, 0.5);
    e.play(s, Bus::Music, p);
    let buf = render(&mut e, SR as usize);
    let db = 20.0 * power_rms(&buf).log10();
    assert!((db - -9.03).abs() <= 0.1, "sine reads {db:.2} dBFS, want -9.03 +/-0.1");
}

#[test]
fn bit_identical_two_renders() {
    let drive = |e: &mut Engine| {
        e.play(Source::Sine { hz: 220.0 }, Bus::Music,
            VoiceParams { gain: 0.4, pan: -0.3, reverb_send: 0.4, ..VoiceParams::default() });
        e.play(Source::Noise, Bus::Ambience,
            VoiceParams { gain: 0.1, pan: 0.5, lowpass_hz: 1200.0, ..VoiceParams::default() });
        e.play(Source::Sine { hz: 440.0 }, Bus::Sfx,
            VoiceParams { gain: 0.6, ..VoiceParams::default() });
        render(e, 24000)
    };
    let a = drive(&mut Engine::new(SR, 42));
    let b = drive(&mut Engine::new(SR, 42));
    assert_eq!(a, b, "same seed and calls must render bit-identically");
}

#[test]
fn peaks_below_minus_1_dbfs() {
    let mut e = Engine::new(SR, 1);
    for i in 0..12 {
        e.play(Source::Sine { hz: 100.0 + i as f32 * 37.0 }, Bus::Music,
            VoiceParams { gain: 0.9, pan: (i % 3) as f32 - 1.0, ..VoiceParams::default() });
    }
    let buf = render(&mut e, SR as usize);
    let peak = buf.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(peak <= 0.8913, "limiter let a peak through: {peak}");
}

#[test]
fn no_nan_or_inf() {
    let mut e = Engine::new(SR, 3);
    e.play(Source::Noise, Bus::Ambience, VoiceParams { gain: 1.5, lowpass_hz: 0.0, ..VoiceParams::default() });
    e.play(Source::Sine { hz: 0.0 }, Bus::Sfx, VoiceParams { pitch: 0.0, ..VoiceParams::default() });
    e.play(Source::Sine { hz: 20000.0 }, Bus::Ui, VoiceParams { pitch: 3.0, ..VoiceParams::default() });
    let buf = render(&mut e, SR as usize);
    assert!(buf.iter().all(|x| x.is_finite()), "non-finite sample in render");
}

#[test]
fn voice_cap_48_holds() {
    let mut e = Engine::new(SR, 5);
    for i in 0..MAX_VOICES + 12 {
        e.play(Source::Sine { hz: 200.0 + i as f32 }, Bus::Sfx,
            VoiceParams { gain: 0.1, ..VoiceParams::default() });
    }
    assert!(e.live_voices() <= MAX_VOICES, "{} voices live over the cap", e.live_voices());
    let buf = render(&mut e, 4800);
    assert!(buf.iter().all(|x| x.is_finite()));
}

#[test]
fn steal_has_no_click() {
    // A click is a discontinuity the material doesn't explain: render
    // the steal and an identical engine without it, and the steal's
    // worst sample-to-sample delta may not exceed the baseline's. A
    // hard removal of a 0.5 sine would add ~1.0; the 5 ms fade adds
    // ~0.002.
    let drive = |e: &mut Engine, steal: bool| -> Vec<f32> {
        for i in 0..MAX_VOICES {
            e.play(Source::Sine { hz: 300.0 + i as f32 * 11.0 }, Bus::Music,
                VoiceParams { gain: 0.5, ..VoiceParams::default() });
        }
        e.render(&mut vec![0.0; 960]); // let them all get going
        if steal {
            e.play(Source::Sine { hz: 999.0 }, Bus::Music,
                VoiceParams { gain: 0.5, ..VoiceParams::default() });
        }
        render(e, 4800) // 100 ms across the steal
    };
    let max_delta = |b: &[f32]| b.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
    let with = max_delta(&drive(&mut Engine::new(SR, 9), true));
    let without = max_delta(&drive(&mut Engine::new(SR, 9), false));
    assert!(with <= without + 0.1, "the steal added a click: {with} vs baseline {without}");
}

#[test]
fn muted_and_master_zero_are_silent() {
    let mut e = Engine::new(SR, 11);
    e.play(Source::Noise, Bus::Music, VoiceParams { gain: 0.9, ..VoiceParams::default() });
    e.set_bus_gain(Bus::Master, 0.0);
    let buf = render(&mut e, 4800);
    assert!(buf.iter().all(|&x| x == 0.0), "master 0 rendered nonzero samples");
    e.set_bus_gain(Bus::Master, 1.0);
    e.set_bus_gain(Bus::Music, 0.0);
    let buf = render(&mut e, 4800);
    assert!(buf.iter().all(|&x| x == 0.0), "a zeroed bus still sounded");
}

#[test]
fn mono_sums_correctly() {
    let call = |e: &mut Engine| {
        e.play(Source::Sine { hz: 330.0 }, Bus::Music,
            VoiceParams { gain: 0.5, pan: -0.7, ..VoiceParams::default() });
        e.play(Source::Sine { hz: 550.0 }, Bus::Music,
            VoiceParams { gain: 0.3, pan: 0.8, ..VoiceParams::default() });
    };
    let mut stereo = Engine::new(SR, 13);
    call(&mut stereo);
    let s = render(&mut stereo, 4800);
    let mut mono = Engine::new(SR, 13);
    call(&mut mono);
    mono.set_mono(true);
    let m = render(&mut mono, 4800);
    for i in (0..m.len()).step_by(2) {
        assert_eq!(m[i], m[i + 1], "mono channels differ at {i}");
        let want = (s[i] + s[i + 1]) * 0.5;
        assert!((m[i] - want).abs() < 1e-6, "mono {i} = {} not the stereo fold {want}", m[i]);
    }
}

#[test]
fn pan_is_equal_power() {
    // Total stereo power stays constant as the pan sweeps - that is what
    // equal-power means: l^2 + r^2 = s^2 for every pan.
    let mut totals = vec![];
    for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        let mut e = Engine::new(SR, 17);
        e.play(Source::Sine { hz: 440.0 }, Bus::Music,
            VoiceParams { gain: 0.5, pan, ..VoiceParams::default() });
        let buf = render(&mut e, 4800);
        let e2: f32 = buf.chunks(2).map(|c| c[0] * c[0] + c[1] * c[1]).sum();
        totals.push(e2);
    }
    let first = totals[0];
    for t in &totals { assert!((t - first).abs() < first * 0.01, "pan moved the power: {totals:?}"); }
    // and at the extremes one side really is silent
    let mut e = Engine::new(SR, 17);
    e.play(Source::Sine { hz: 440.0 }, Bus::Music,
        VoiceParams { gain: 0.5, pan: -1.0, ..VoiceParams::default() });
    let buf = render(&mut e, 4800);
    let right: f32 = buf.iter().skip(1).step_by(2).map(|x| x * x).sum();
    assert!(right < 1e-9, "hard-left pan still heard on the right: {right}");
}

#[test]
fn lowpass_attenuates_above_cutoff() {
    let open = {
        let mut e = Engine::new(SR, 19);
        e.play(Source::Sine { hz: 8000.0 }, Bus::Music,
            VoiceParams { gain: 0.5, ..VoiceParams::default() });
        power_rms(&render(&mut e, SR as usize))
    };
    let closed = {
        let mut e = Engine::new(SR, 19);
        e.play(Source::Sine { hz: 8000.0 }, Bus::Music,
            VoiceParams { gain: 0.5, lowpass_hz: 500.0, ..VoiceParams::default() });
        power_rms(&render(&mut e, SR as usize))
    };
    assert!(closed < open * 0.3, "8000 Hz through a 500 Hz corner: {closed:.4} vs open {open:.4}");
}
