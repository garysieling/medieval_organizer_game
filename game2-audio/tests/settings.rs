//! Settings tests for #1505 (U1): the `AudioSettings` struct, its
//! serde contract, and what its rows do to the mixer. Names are as
//! the ticket requires.
use game2_audio::host::{AudioFrame, AudioHost, CueKind};
use game2_audio::settings::{AudioSettings, Presence};

const SR: u32 = 44100;

fn block(host: &mut AudioHost, n: usize) -> Vec<f32> {
    let mut b = vec![0.0; n * 2];
    host.render_block(&mut b);
    b
}

fn bell(gain: f32) -> AudioFrame {
    AudioFrame { cues: vec![CueKind::Bell { pan: 0.0, gain }], seed: 7 }
}

#[test]
fn settings_round_trip() {
    let mut s = AudioSettings::default();
    s.master = 45; s.presence = Presence::Sparse; s.mono = true; s.night_mode = true;
    let json = serde_json::to_string(&s).unwrap();
    let back: AudioSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

#[test]
fn clamp_snaps_to_steps_and_range() {
    let mut s = AudioSettings::default();
    s.master = 150;
    s.music = 73;
    s.ambience = 74;
    s.effects = 71;
    s.interface = 0;
    s.clamp();
    assert_eq!(s.master, 100, "over-range caps at 100");
    assert_eq!(s.music, 75, "73 rounds up to 75");
    assert_eq!(s.ambience, 75, "74 rounds up to 75");
    assert_eq!(s.effects, 70, "71 rounds down to 70");
    assert_eq!(s.interface, 0);
}

#[test]
fn defaults_match_design() {
    let s = AudioSettings::default();
    assert_eq!(s.version, 1);
    assert_eq!((s.master, s.music, s.ambience, s.effects, s.interface), (80, 70, 70, 80, 60));
    assert_eq!(s.presence, Presence::Normal);
    assert!(!s.mono && !s.night_mode && !s.captions);
    assert!(s.mute_in_background);
    // and the presence budgets the design table fixes
    assert_eq!(Presence::Off.budget(), 0.0);
    assert_eq!(Presence::Sparse.budget(), 0.35);
    assert_eq!(Presence::Normal.budget(), 0.60);
    assert_eq!(Presence::Rich.budget(), 0.85);
}

#[test]
fn unknown_fields_ignored_missing_fields_default() {
    let s: AudioSettings = serde_json::from_str(
        r#"{"version": 7, "master": 42, "surprise": [1, 2, 3]}"#).unwrap();
    assert_eq!(s.master, 42, "a known field still lands");
    assert_eq!(s.version, 7);
    assert_eq!(s.music, 70, "missing fields take defaults");
    assert_eq!(s.presence, Presence::Normal);
    assert!(s.mute_in_background);
    // an entirely empty file is all defaults
    let s: AudioSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(s, AudioSettings::default());
}

#[test]
fn apply_setting_rejects_bad_key_and_type() {
    let mut s = AudioSettings::default();
    assert!(s.apply_setting("bogus", serde_json::json!(1)).is_err());
    assert!(s.apply_setting("master", serde_json::json!("loud")).is_err());
    assert!(s.apply_setting("mono", serde_json::json!(1)).is_err());
    assert!(s.apply_setting("presence", serde_json::json!("lots")).is_err());
    // the good ones land, snapped to the grid
    s.apply_setting("master", serde_json::json!(73)).unwrap();
    assert_eq!(s.master, 75);
    s.apply_setting("presence", serde_json::json!("off")).unwrap();
    assert_eq!(s.presence, Presence::Off);
    s.apply_setting("night_mode", serde_json::json!(true)).unwrap();
    assert!(s.night_mode);
}

#[test]
fn volume_curve_is_square_law() {
    assert_eq!(AudioSettings::bus_amp(100), 1.0);
    assert_eq!(AudioSettings::bus_amp(0), 0.0);
    assert!((AudioSettings::bus_amp(50) - 0.25).abs() < 1e-6, "mid track is quarter power");
    assert!((AudioSettings::bus_amp(70) - 0.49).abs() < 1e-6);
    // and end to end: Master 50 renders a quarter of Master 100's
    // amplitude, sample for sample
    let render_at = |master: u8| -> Vec<f32> {
        let mut h = AudioHost::offline(SR);
        let mut s = AudioSettings::default();
        s.master = master;
        h.apply_settings(&s);
        h.push_frame(&bell(1.0));
        block(&mut h, 2048)
    };
    let hot = render_at(100);
    let half = render_at(50);
    for i in 0..hot.len() {
        if hot[i].abs() > 0.1 && hot[i].abs() < 0.7 {
            assert!((half[i] - hot[i] * 0.25).abs() < 0.01,
                "sample {i}: {} != {}*0.25", half[i], hot[i]);
        }
    }
}

#[test]
fn muted_renders_exact_zeros() {
    let mut h = AudioHost::offline(SR);
    h.apply_settings(&AudioSettings::default());
    h.set_muted(true);
    h.push_frame(&bell(1.0));
    let b = block(&mut h, 1024);
    assert!(b.iter().all(|&x| x == 0.0), "muted means exact silence");
    // unmute brings it back
    h.set_muted(false);
    h.push_frame(&bell(1.0));
    let b = block(&mut h, 1024);
    assert!(b.iter().any(|&x| x != 0.0));
}

#[test]
fn night_mode_reduces_peaks_of_a_hot_mix() {
    let peak = |night: bool| -> f32 {
        let mut h = AudioHost::offline(SR);
        let mut s = AudioSettings::default();
        s.night_mode = night;
        h.apply_settings(&s);
        h.push_frame(&bell(2.0)); // a hot cue, over the -24 dBFS threshold
        block(&mut h, 2048).iter().fold(0.0f32, |m, x| m.max(x.abs()))
    };
    let day = peak(false);
    let night = peak(true);
    assert!(day > 0.7, "the hot mix really is hot: {day}");
    assert!(night < day - 0.1, "night mode pulls the peaks down: {night} vs {day}");
}

#[test]
fn mono_setting_sums() {
    // a hard-right cue: stereo keeps it on the right, mono centres it
    let render = |mono: bool| -> Vec<f32> {
        let mut h = AudioHost::offline(SR);
        let mut s = AudioSettings::default();
        s.mono = mono;
        h.apply_settings(&s);
        h.push_frame(&AudioFrame {
            cues: vec![CueKind::Bell { pan: 1.0, gain: 1.0 }], seed: 7 });
        block(&mut h, 1024)
    };
    let stereo = render(false);
    assert!(stereo.chunks(2).any(|c| (c[0] - c[1]).abs() > 0.01),
        "a panned cue differs between ears");
    let mono = render(true);
    for (i, c) in mono.chunks(2).enumerate() {
        assert_eq!(c[0], c[1], "sample {i}: mono means identical ears");
    }
    assert!(mono.iter().any(|&x| x != 0.0), "mono sums, it doesn't mute");
}

#[test]
fn unfocused_window_fades_to_silence() {
    let mut h = AudioHost::offline(SR);
    h.apply_settings(&AudioSettings::default()); // mute_in_background on
    h.push_frame(&bell(1.0));
    h.set_focused(false);
    // render 400 ms' worth: the first part still sounds, the tail is gone
    let b = block(&mut h, (SR as usize) * 4 / 10);
    let first = b[..200].iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let last = b[b.len() - 200..].iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(first > 0.0, "the fade starts from sounding");
    assert_eq!(last, 0.0, "300 ms out the mix is silent");
    // refocus brings it back
    h.set_focused(true);
    h.push_frame(&bell(1.0));
    let b = block(&mut h, (SR as usize) * 4 / 10);
    assert!(b[b.len() - 200..].iter().any(|&x| x != 0.0), "focus restored");
}

#[test]
fn unfocused_without_bg_mute_keeps_sounding() {
    let mut h = AudioHost::offline(SR);
    let mut s = AudioSettings::default();
    s.mute_in_background = false;
    h.apply_settings(&s);
    h.push_frame(&bell(1.0));
    h.set_focused(false);
    let b = block(&mut h, (SR as usize) * 4 / 10);
    assert!(b[b.len() - 200..].iter().any(|&x| x != 0.0),
        "mute_in_background off means an unfocused window still plays");
}
