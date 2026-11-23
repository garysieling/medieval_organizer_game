//! The analysis toolkit's proofs (S2, #1491). Names are as the ticket
//! requires.
use super::*;
use std::f32::consts::TAU;

const SR: u32 = 48000;

fn stereo(frames: usize, mut f: impl FnMut(usize) -> f32) -> Vec<f32> {
    let mut b = vec![0.0; frames * 2];
    for i in 0..frames { let s = f(i); b[2 * i] = s; b[2 * i + 1] = s; }
    b
}

fn sine(hz: f32, amp: f32, phase: f32, frames: usize) -> Vec<f32> {
    stereo(frames, |i| amp * (phase + TAU * hz * i as f32 / SR as f32).sin())
}

fn decode_png(path: &str) -> (u32, u32, Vec<u8>) {
    let d = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut r = d.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    (info.width, info.height, buf[..info.buffer_size()].to_vec())
}

#[test]
fn lufs_of_stereo_1khz_minus_20_dbfs_sine_is_minus_20_plus_minus_0_5() {
    // dual-mono sums channel energies (+3.01 dB): a sine at -20 dBFS
    // peak on both channels reads -20 LUFS integrated
    let buf = sine(1000.0, 0.1, 0.0, SR as usize);
    let l = loudness(&buf, SR);
    assert!((l.integrated_lufs - -20.0).abs() <= 0.5,
        "1 kHz stereo sine at -20 dBFS reads {:.2} LUFS", l.integrated_lufs);
}

#[test]
fn true_peak_catches_intersample_peak() {
    // a sine at fs/4 sampled off-phase: the samples top out at sin(pi/4)
    // while the real peak is the full amplitude - a dBFS meter would
    // miss it by ~3 dB
    let amp = 0.9;
    let buf = sine(12000.0, amp, PI / 4.0, SR as usize / 10);
    let sample_peak = buf.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let db_sample = 20.0 * sample_peak.log10();
    let tp = true_peak(&buf, SR);
    assert!(tp - db_sample >= 0.5,
        "intersample peak invisible: sample {db_sample:.2} dBFS, true {tp:.2} dBTP");
    assert!((tp - 20.0 * amp.log10()).abs() < 0.3, "true peak {tp} far off {amp}");
}

#[test]
fn clicks_none_in_clean_sine() {
    let buf = sine(440.0, 0.5, 0.0, SR as usize);
    assert!(clicks(&buf, SR).is_empty(), "clean sine flagged: {:?}", clicks(&buf, SR));
}

#[test]
fn clicks_finds_injected_step() {
    let mut buf = sine(440.0, 0.05, 0.0, SR as usize);
    let at = (SR as usize) / 2;
    // a DC step of 0.8 mid-buffer - plainly a click
    for i in at..at + 8 { buf[2 * i] += 0.8; buf[2 * i + 1] += 0.8; }
    let hits = clicks(&buf, SR);
    assert!(!hits.is_empty(), "an injected 0.8 step found no click");
    assert!(hits.iter().any(|&t| (t - 0.5).abs() < 0.01),
        "no click near the step at 0.5 s: {hits:?}");
}

#[test]
fn silence_finds_2s_gap_in_noise() {
    let mut rng = crate::rng::Rng::new(4);
    let frames = (SR * 5) as usize;
    let mut buf = stereo(frames, |_| rng.range(-0.2, 0.2));
    let (a, b) = (SR as usize, SR as usize * 3);
    for i in a..b { buf[2 * i] = 0.0; buf[2 * i + 1] = 0.0; }
    let gap = longest_silence(&buf, SR, -60.0);
    assert!((gap - 2.0).abs() <= 0.05, "2 s gap measured {gap} s");
}

#[test]
fn pitch_class_of_440hz_is_a() {
    let buf = sine(440.0, 0.5, 0.0, SR as usize / 4);
    let pc = pitch_classes(&buf, SR);
    let best = pc.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
    assert_eq!(best, 9, "440 Hz folded to class {best}, not A (9): {pc:?}");
    assert!((pc.iter().sum::<f32>() - 1.0).abs() < 1e-5, "not normalised: {}", pc.iter().sum::<f32>());
}

#[test]
fn spectrogram_png_dimensions_and_not_blank() {
    let buf = sine(1000.0, 0.5, 0.0, SR as usize * 2);
    let dir = std::env::temp_dir().join("game2-s2-spec");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("spec.png");
    spectrogram_png(&buf, SR, path.to_str().unwrap()).unwrap();
    let (w, h, px) = decode_png(path.to_str().unwrap());
    assert_eq!(h, 400);
    assert!(w > 100, "too thin: {w}");
    let bg = [0u8, 0, 0];
    let diff = px.chunks(3).filter(|c| c != &bg).count();
    assert!(diff > px.len() / 3 / 100, "spectrogram reads blank");
    // the sine should light one narrow band, not the whole field
    if std::env::var("GAME2_KEEP").is_ok() {
        // repo-root evidence/, alongside the other tickets' images
        let out = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../evidence/sound/s2"));
        std::fs::create_dir_all(out).unwrap();
        spectrogram_png(&buf, SR, out.join("example_spectrogram.png").to_str().unwrap()).unwrap();
    }
}

#[test]
fn piano_roll_marks_out_of_mode_red() {
    let input = RollInput {
        mode_pitch_classes: { let mut m = [false; 12]; m[0] = true; m[4] = true; m[7] = true; m },
        notes: vec![
            RollNote { start_s: 0.0, dur_s: 1.0, midi: 60, voice: 0, in_mode: true },
            RollNote { start_s: 1.0, dur_s: 1.0, midi: 61, voice: 0, in_mode: false },
            RollNote { start_s: 2.0, dur_s: 1.0, midi: 64, voice: 0, in_mode: true },
        ],
    };
    let dir = std::env::temp_dir().join("game2-s2-roll");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("roll.png");
    piano_roll_png(&input, path.to_str().unwrap()).unwrap();
    let (w, _, px) = decode_png(path.to_str().unwrap());
    let red = px.chunks(3).filter(|c| c[0] > 150 && c[1] < 80 && c[2] < 80).count();
    assert!(red > 20, "the out-of-mode note isn't red: {red} red pixels");
    // mode rows align with notes: C5 (row 96-72=24) washed, C#5 white -
    // an empty stretch of the row, before the first note starts
    let wash = |row: usize| -> [u8; 3] {
        let p = (row * 6 + 2) * w as usize * 3;
        [px[p], px[p + 1], px[p + 2]]
    };
    assert_eq!(wash(96 - 72), [232, 240, 252], "C5 mode row not washed");
    assert_eq!(wash(96 - 73), [255, 255, 255], "C#5 off-mode row washed");
}

#[test]
fn timeline_png_dimensions_and_not_blank() {
    let input = TimelineInput {
        tension: (0..60).map(|i| i as f32 / 60.0).collect(),
        sections: vec![
            Section { start_s: 0.0, end_s: 30.0, label: "calm".into() },
            Section { start_s: 30.0, end_s: 60.0, label: "build".into() },
        ],
        mode_lane: vec![(0.0, "aeolian".into()), (30.0, "dorian".into())],
        layers: vec![
            Lane { name: "drone".into(), density: vec![0.6; 15] },
            Lane { name: "melody".into(), density: (0..15).map(|i| i as f32 / 15.0).collect() },
        ],
        markers: vec![Marker { t_s: 30.0, label: "assembly".into() }],
    };
    let dir = std::env::temp_dir().join("game2-s2-time");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("time.png");
    timeline_png(&input, path.to_str().unwrap()).unwrap();
    let (w, h, px) = decode_png(path.to_str().unwrap());
    assert_eq!((w, h), (800, 360));
    let white = [255u8, 255, 255];
    let diff = px.chunks(3).filter(|c| c != &white).count();
    assert!(diff > px.len() / 3 / 100, "timeline reads blank");
}
