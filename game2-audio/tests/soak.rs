//! The design's soak (I2, #1510): a staged village played headless at
//! speed 16 for `GAME2_SOUND_SOAK_DAYS` sim days (default 400), the
//! director and ambience on, audio rendered in lockstep like the live
//! game. Long-run health: no NaN/Inf, no sample over -1 dBFS, <= 48
//! voices, the first and last ten minutes within 4 LU, resident memory
//! bounded, and of course no panics. `#[ignore]`d - run it on demand
//! (and nightly if CI ever schedules it):
//!
//!   cargo test -p game2-audio --release soak -- --ignored
//!
//! `GAME2_SOUND_SOAK_DAYS` counts *sim* days; at speed 16 a day is ~56
//! frames, so 400 days renders only ~6 minutes of audio. The loudness
//! check needs a first and last ten minutes, so the run extends to the
//! audio floor (~20.5 min, about 1,300 days) when the day count alone
//! wouldn't reach it.
mod common;

use game2_audio::analysis;
use game2_audio::host::AudioHost;
use game2_audio::scenarios;
use game2_audio::soundscape::Listener;
use game2_audio::wav::SAMPLE_RATE;

/// Resident set size from `/proc/self/statm` (Linux only; the memory
/// assertion is skipped everywhere else).
#[cfg(target_os = "linux")]
fn resident_bytes() -> usize {
    std::fs::read_to_string("/proc/self/statm").ok()
        .and_then(|s| s.split_whitespace().nth(1)
            .and_then(|v| v.parse::<usize>().ok()))
        .unwrap_or(0) * 4096
}
#[cfg(not(target_os = "linux"))]
fn resident_bytes() -> usize { 0 }

#[test]
#[ignore = "a long soak - run it on demand, not in the suite"]
fn soak_long_run_stays_healthy() {
    let days: u64 = std::env::var("GAME2_SOUND_SOAK_DAYS")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(400);
    let sc = scenarios::by_name("quiet_village")
        .expect("the soak needs the quiet village");
    let mut g = (sc.build)(42);
    g.speed = 16;
    let frames_per_day = g.ticks_per_frame() as u64;
    // the two loudness windows - ten minutes of stereo each
    let win = 10 * 60 * SAMPLE_RATE as usize * 2;
    let frames = (days * frames_per_day)
        .max((2 * 10 * 60 * 60) as u64 + 600);
    eprintln!("soak: {days} days x {frames_per_day} f/day -> {frames} frames \
        ({:.1} min of audio)", frames as f64 / 3600.0);

    // Both windows are allocated and touched before the run so the
    // memory assertion measures *game* growth, not these buffers.
    // `last` is a ring - BS.1770 gating is order-insensitive, so it can
    // be measured without un-rotating.
    let mut first = vec![1e-30f32; win];
    let mut last = vec![1e-30f32; win];
    let (mut fi, mut li) = (0usize, 0usize);

    let mut host = AudioHost::offline(SAMPLE_RATE);
    host.set_bank(common::test_bank());
    let mut block = vec![];
    let mut peak = 0.0f32;
    let mut voices_max = 0usize;
    let mut res_tenth = 0usize;
    for f in 0..frames {
        if f == frames / 10 { res_tenth = resident_bytes(); }
        g.update();
        let m = g.ui_model();
        host.push_with(&m, &Listener::from_model(&m));
        let n = host.frame_samples();
        block.clear();
        block.resize(n * 2, 0.0);
        host.render_block(&mut block);
        for &s in &block {
            assert!(s.is_finite(), "frame {f}: non-finite sample {s}");
            peak = peak.max(s.abs());
        }
        if fi < win {
            let take = (win - fi).min(block.len());
            first[fi..fi + take].copy_from_slice(&block[..take]);
            fi += take;
        }
        for &s in &block {
            last[li] = s;
            li = (li + 1) % win;
        }
        voices_max = voices_max.max(host.voices_peak());
    }
    assert!(peak <= 10f32.powf(-1.0 / 20.0),
        "a sample clipped past -1 dBFS: {peak}");
    assert!(voices_max <= 48, "{voices_max} voices over the cap");

    let a = analysis::loudness(&first, SAMPLE_RATE);
    let b = analysis::loudness(&last, SAMPLE_RATE);
    assert!(a.true_peak_dbtp <= -1.0 && b.true_peak_dbtp <= -1.0,
        "true peak over -1 dBTP: {:.2} / {:.2}", a.true_peak_dbtp, b.true_peak_dbtp);
    eprintln!("soak: first10 {:.1} LUFS, last10 {:.1} LUFS, peak {peak:.3}, \
        {voices_max} voices", a.integrated_lufs, b.integrated_lufs);
    assert!((a.integrated_lufs - b.integrated_lufs).abs() <= 4.0,
        "loudness drifted {:.1} -> {:.1} LUFS over the run",
        a.integrated_lufs, b.integrated_lufs);

    if cfg!(target_os = "linux") {
        let res_end = resident_bytes();
        assert!(res_tenth == 0 || res_end <= res_tenth + res_tenth / 10,
            "resident memory grew {res_tenth} -> {res_end} (>10%)");
    }
}
