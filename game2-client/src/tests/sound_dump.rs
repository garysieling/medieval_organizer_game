//! `sound_dump`'s proofs (#1493): the six outputs land, a replayed
//! scenes file renders the same bytes as live, renders repeat, and the
//! JSONL roundtrips.
use crate::*;
use game2_audio::scenarios;
use game2_core::session::{UiModel, MAP_SEED};

/// N frames of the quiet village, models collected the way the bin
/// drives them.
fn quiet_models(frames: u64) -> Vec<UiModel> {
    let sc = scenarios::by_name("quiet_village").unwrap();
    let mut g = (sc.build)(MAP_SEED);
    (0..frames).map(|_| { g.update(); g.ui_model() }).collect()
}

#[test]
fn dump_writes_all_six_outputs() {
    let dir = std::env::temp_dir().join("game2-s3-dump");
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("s3").to_str().unwrap().to_string();
    let opts = sound_dump::DumpOpts { frames: 600, out: out.clone(),
        ..Default::default() };
    sound_dump::run(&opts).unwrap();
    for ext in ["wav", "trace.json", "metrics.json", "spectrogram.png", "roll.png", "timeline.png"] {
        let p = format!("{out}.{ext}");
        assert!(std::path::Path::new(&p).metadata().map(|m| m.len() > 0).unwrap_or(false),
            "{p} missing or empty");
    }
    // every metric key the ticket names is present
    let mj: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{out}.metrics.json")).unwrap()).unwrap();
    for k in ["lufs_integrated", "true_peak_dbtp", "clicks",
              "longest_silence_s", "voices_peak", "render_ms"] {
        assert!(mj.get(k).is_some(), "metrics.json lacks {k}: {mj}");
    }
}

#[test]
fn replay_matches_live_bit_exact() {
    let models = quiet_models(300);
    let (live, _) = sound_dump::render_models(models.iter());
    let dir = std::env::temp_dir().join("game2-s3-replay");
    std::fs::create_dir_all(&dir).unwrap();
    let scenes = dir.join("scenes.jsonl");
    sound_dump::write_scenes(
        &mut std::io::BufWriter::new(std::fs::File::create(&scenes).unwrap()),
        &models).unwrap();
    let out = dir.join("rep").to_str().unwrap().to_string();
    let opts = sound_dump::DumpOpts {
        scenes: Some(scenes.to_str().unwrap().into()), out: out.clone(),
        ..Default::default() };
    sound_dump::run(&opts).unwrap();
    // the replay's wav, sample-for-sample against the live render's
    let wav = std::fs::read(format!("{out}.wav")).unwrap();
    let live16: Vec<u8> = live.iter()
        .flat_map(|s| ((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
        .collect();
    assert_eq!(&wav[44..], &live16[..], "scene replay drifted off the live render");
}

#[test]
fn render_twice_identical() {
    let (a, _) = sound_dump::render_models(quiet_models(120).iter());
    let (b, _) = sound_dump::render_models(quiet_models(120).iter());
    assert_eq!(a, b, "the same village rendered twice differs");
}

#[test]
fn scenes_roundtrip_serde() {
    let models = quiet_models(3);
    let mut buf = vec![];
    sound_dump::write_scenes(&mut buf, &models).unwrap();
    let text = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    let first: UiModel = serde_json::from_str(lines[0]).unwrap();
    let second: UiModel = serde_json::from_str(lines[1]).unwrap();
    assert!(first.map.width > 0, "the first line should carry the map");
    assert_eq!(second.map.width, 0, "a later line should be lean");
    assert_eq!(first.seed, models[0].seed);
    assert_eq!(second.seed, models[1].seed);
    assert_eq!(second.sfx_v, models[1].sfx_v);
}
