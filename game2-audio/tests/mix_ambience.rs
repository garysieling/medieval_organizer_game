//! A4's mix tests (#1500): the four place scenarios render 60 s
//! through the same `AudioHost` path the dump and the speakers use,
//! then the design's Mix bar applies - loudness, true peak, clicks,
//! ambience never fully silent, and the render's speed budget. Plus
//! the ambience mixer's own behaviours.
//!
//! The bank is the real `assets/sound` content plus test-synthesised
//! stand-ins for whatever the first batch hasn't fetched yet, so the
//! whole-scene assertions always exercise audio; a missing real asset
//! is silence by design (A2) but would leave these scenes thin.
use game2_audio::analysis;
use game2_audio::engine::Engine;
use game2_audio::host::{AudioFrame, AudioHost};
use game2_audio::sampler::SampleBank;
use game2_audio::scenarios::{self, look_spot, Op, ScriptOp};
use game2_audio::soundscape::ambience::Ambience;
use game2_audio::soundscape::{Emitter, EmitterKind, Listener, Tier};
use game2_audio::wav::SAMPLE_RATE;
use game2_core::session::stage;
use std::collections::BTreeMap;

/// Every patch name the ambience mixer may ask for
const BATCH: &[&str] = &[
    "birds", "insects", "bees", "river", "village_murmur", "market_crowd",
    "fire_crackle", "church_bell", "anvil_hit", "sheep_bleat", "chicken",
    "dog_bark", "door_creak", "cart_wheel", "gate_creak", "crow", "hoofbeat",
];

/// A tiny deterministic 16-bit mono WAV: a sine partial over shaped
/// noise, `amp` peak - enough body to exercise the mix without
/// pretending to be the real recording. `seed` varies the texture so
/// distinct ids don't share a waveform. Loops crossfade their tail
/// into the head, like the fetch pipeline does, so the wrap is seamless.
fn fake_wav(id: &str, secs: f32, sr: u32, amp: f32, loops: bool) -> Vec<u8> {
    let mut z = id.bytes().fold(0x9E37_79B9_7F4A_7C15u64, |a, b| {
        (a ^ b as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
    });
    let n = (secs * sr as f32) as usize;
    let hz = 180.0 + (z % 700) as f32;
    let mut smp = vec![0f32; n];
    for i in 0..n {
        z ^= z >> 12; z ^= z << 25; z ^= z >> 27;
        let noise = ((z >> 40) as f32 / 16_777_216.0) - 1.0;
        let t = i as f32 / sr as f32;
        smp[i] = (noise * 0.5 + (2.0 * std::f32::consts::PI * hz * t).sin() * 0.5) * amp;
    }
    if loops {
        // blend head and tail toward each other so the wrap step is a
        // small fraction of the signal - `sound_process.py` acrossfades
        // real beds the same way
        let fade = ((sr / 2) as usize).min(n / 4);
        for i in 0..fade {
            let w = i as f32 / fade as f32;
            let (h, t) = (smp[i], smp[n - fade + i]);
            smp[i] = h * w + t * (1.0 - w);
            smp[n - fade + i] = t * w + h * (1.0 - w);
        }
    }
    let mut pcm = vec![0u8; n * 2];
    for (i, &s) in smp.iter().enumerate() {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        pcm[2 * i..2 * i + 2].copy_from_slice(&v.to_le_bytes());
    }
    let mut w = Vec::with_capacity(44 + pcm.len());
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&sr.to_le_bytes());
    w.extend_from_slice(&(sr * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    w.extend_from_slice(&pcm);
    w
}

/// The real assets plus a stand-in for each batch id the fetch hasn't
/// landed yet - so every emitter kind voices in these tests.
fn test_bank() -> SampleBank {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../assets/sound");
    let mut bank = game2_audio::sampler::load_dir(&dir);
    for &id in BATCH {
        if bank.patch_id(id).is_none() {
            // beds loop long loops; one-shots want a shorter ring - the
            // mixer doesn't care, the mix does
            let loops = !matches!(id, "anvil_hit" | "church_bell" | "crow"
                | "sheep_bleat" | "chicken" | "dog_bark" | "door_creak"
                | "cart_wheel" | "gate_creak" | "hoofbeat");
            let secs = if loops { 3.0 } else { 0.8 };
            // beds loudnorm to ~-20 LUFS, shots to ~-14: this texture's
            // RMS is ~0.45x its peak, so the amps land the fakes on the
            // same targets the real pipeline hits
            let amp = if secs > 1.0 { 0.22 } else { 0.45 };
            bank.insert_bytes(id, &fake_wav(id, secs, 22050, amp, loops)).unwrap();
        }
    }
    bank
}

/// A scenario rendered through the host like `sound_dump` does: ops
/// applied on their frames, `Look*` steering the listener, one
/// 60 fps block per update.
fn render_scenario(name: &str, frames: u64) -> (Vec<f32>, usize, f32) {
    let sc = scenarios::by_name(name).unwrap_or_else(|| panic!("no scenario {name}"));
    let mut g = (sc.build)(42);
    let mut by_frame: BTreeMap<u64, Vec<Op>> = BTreeMap::new();
    for ScriptOp { frame, op } in sc.script { by_frame.entry(frame).or_default().push(op); }
    let mut look: Option<Op> = None;
    let mut host = AudioHost::offline(SAMPLE_RATE);
    host.set_bank(test_bank());
    let mut pcm = vec![];
    let mut render_secs = 0.0f32;
    for f in 0..frames {
        if let Some(ops) = by_frame.remove(&f) {
            for op in &ops {
                match op {
                    Op::Cmd(c) => g.command(c.clone()),
                    Op::ForceEvent(id) => g.force_event(*id),
                    Op::RoadMarch(si) => { let _ = g.force_road_march(0, *si); }
                    Op::Stage(s) => stage::stage(&mut g, *s),
                    _ => look = Some(match op {
                        Op::Look(x, y) => Op::Look(*x, *y),
                        Op::LookMarket => Op::LookMarket,
                        Op::LookSkeps => Op::LookSkeps,
                        Op::LookChurch => Op::LookChurch,
                        Op::LookGate => Op::LookGate,
                        _ => unreachable!(),
                    }),
                }
            }
        }
        g.update();
        let m = g.ui_model();
        let l = look.as_ref().and_then(|op| look_spot(op, &m))
            .map(|(x, y)| Listener::at(x, y, 1.0))
            .unwrap_or_else(|| Listener::from_model(&m));
        host.push_frame(&AudioFrame::from_model_with(&m, &l));
        let n = host.frame_samples();
        let start = pcm.len();
        pcm.resize(start + n * 2, 0.0);
        let t = std::time::Instant::now();
        host.render_block(&mut pcm[start..]);
        render_secs += t.elapsed().as_secs_f32();
    }
    (pcm, host.voices_peak(), render_secs)
}

/// The design's Mix bar for a scenario render (60 s at 60 fps)
fn check_scenario(name: &str) {
    let (pcm, voices, elapsed) = render_scenario(name, 3600);
    let loud = analysis::loudness(&pcm, SAMPLE_RATE);
    eprintln!("{name}: {:.1} LUFS, tp {:.2} dBTP, {} voices, render {elapsed:.2}s",
        loud.integrated_lufs, loud.true_peak_dbtp, voices);
    assert!(loud.integrated_lufs > -23.0 && loud.integrated_lufs < -17.0,
        "{name}: integrated {:.1} LUFS outside -20±3", loud.integrated_lufs);
    assert!(loud.true_peak_dbtp <= -1.0,
        "{name}: true peak {:.2} dBTP over -1", loud.true_peak_dbtp);
    let clicks = analysis::clicks(&pcm, SAMPLE_RATE);
    assert!(clicks.is_empty(), "{name}: clicks at {clicks:?}");
    let gap = analysis::longest_silence(&pcm, SAMPLE_RATE, -60.0);
    assert!(gap < 3.0, "{name}: ambience silent for {gap:.1}s");
    assert!(voices <= 48, "{name}: {voices} voices over the cap");
    if cfg!(not(debug_assertions)) {
        // the design's 3 s bar is an unloaded machine; under the
        // parallel suite render threads contend, so this stays a
        // regression tripwire rather than the bar itself (a clean run
        // is ~1-2 s, printed above)
        assert!(elapsed < 8.0,
            "{name}: 60s took {elapsed:.1}s to render (unloaded budget 3s)");
    }
}

#[test]
fn mix_ambience_apiary_spring() { check_scenario("apiary_spring"); }
#[test]
fn mix_ambience_holy_day_church() { check_scenario("holy_day_church"); }
#[test]
fn mix_ambience_market_square() { check_scenario("market_square"); }
#[test]
fn mix_ambience_winter_gate() { check_scenario("winter_gate"); }

// ------------------------------------------------------------ the batch

/// The first batch's ledger ids, as the mixer names them
#[track_caller]
fn assert_shipped(ids: &[&str]) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sound");
    let bank = game2_audio::sampler::load_dir(&dir);
    for &id in ids {
        assert!(bank.patch_id(id).is_some(), "batch asset {id} not in the bank");
    }
}

#[test]
fn batch_one_shots_shipped() {
    // the A4 batch, minus the four that need Freesound (#1575)
    assert_shipped(&["church_bell", "anvil_hit", "door_creak",
        "gate_creak", "cart_wheel", "hoofbeat"]);
}

#[test]
#[ignore = "needs A4b assets: sheep_bleat, chicken, dog_bark, crow"]
fn batch_animal_shots_shipped() { assert_shipped(&["sheep_bleat", "chicken", "dog_bark", "crow"]); }

#[test]
#[ignore = "needs A4b assets: the seven sampled beds"]
fn batch_beds_shipped() {
    assert_shipped(&["birds", "insects", "bees", "river",
        "village_murmur", "market_crowd", "fire_crackle"]);
}

// ------------------------------------------------------------- unit probes

fn wind_emitter() -> Emitter {
    Emitter { kind: EmitterKind::Wind, tier: Tier::Bed, pos: None,
        weight: 0.8, gain: 0.8, pan: 0.0, lowpass_hz: 4000.0,
        reverb: 0.1, rate_hz: 0.0, min_gap_s: 0.0 }
}

fn homes_emitter(rate_hz: f32) -> Emitter {
    Emitter { kind: EmitterKind::Homesteads, tier: Tier::Signal,
        pos: Some((10.0, 10.0)), weight: 0.5, gain: 0.6, pan: 0.2,
        lowpass_hz: 16000.0, reverb: 0.1, rate_hz, min_gap_s: 0.0 }
}

fn bank_with(ids: &[&str], secs: f32) -> SampleBank {
    let mut bank = SampleBank::new();
    for &id in ids {
        bank.insert_bytes(id, &fake_wav(id, secs, 22050, 0.3, false)).unwrap();
    }
    bank
}

#[test]
fn one_shots_are_deterministic_for_same_seed() {
    let render = || {
        let mut amb = Ambience::new();
        let mut engine = Engine::new(44100, 1);
        engine.set_bank(bank_with(&["chicken", "dog_bark", "door_creak"], 0.8));
        let mut pcm = vec![];
        let emitters = [homes_emitter(0.5)];
        for f in 0..600u64 {
            amb.update(&emitters, 99, f, &mut engine);
            let mut block = [0.0f32; 2 * 735];
            engine.render(&mut block);
            pcm.extend_from_slice(&block);
        }
        pcm
    };
    assert_eq!(render(), render(),
        "same seed, same emitters: the one-shot draws must be identical");
}

#[test]
fn round_robin_never_repeats_immediately() {
    let mut amb = Ambience::new();
    let mut engine = Engine::new(44100, 1);
    engine.set_bank(bank_with(&["chicken", "dog_bark", "door_creak"], 0.05));
    let emitters = [homes_emitter(60.0)]; // fires every frame the cap allows
    let mut seen: Vec<usize> = vec![];
    for f in 0..600u64 {
        amb.update(&emitters, 7, f, &mut engine);
        let mut block = [0.0f32; 2 * 735];
        engine.render(&mut block);
        if let Some(v) = amb.probe_shot_variant(EmitterKind::Homesteads) {
            if seen.last() != Some(&v) { seen.push(v); }
        }
    }
    assert!(seen.len() >= 4, "the round-robin never cycled: {seen:?}");
    for w in seen.windows(2) {
        assert_ne!(w[0], w[1], "variant repeated immediately: {seen:?}");
        assert!(w[1] < 3);
    }
}

#[test]
fn max_two_voices_per_kind() {
    let mut amb = Ambience::new();
    let mut engine = Engine::new(44100, 1);
    engine.set_bank(bank_with(&["anvil_hit"], 3.0)); // a long ring keeps voices alive
    let emitters = [homes_emitter(60.0)];
    for f in 0..120u64 {
        amb.update(&emitters, 3, f, &mut engine);
        let mut block = [0.0f32; 2 * 735];
        engine.render(&mut block);
        assert!(engine.live_voices() <= 2,
            "more than two one-shot voices of a kind at frame {f}");
    }
}

#[test]
fn signal_ducks_bed_by_2db() {
    let mut amb = Ambience::new();
    let mut engine = Engine::new(44100, 1);
    engine.set_bank(bank_with(&["chicken", "dog_bark", "door_creak"], 0.8));
    let wind = wind_emitter(); // the own-synth bed: no asset needed
    for f in 0..300u64 {
        amb.update(&[wind], 5, f, &mut engine);
        let mut block = [0.0f32; 2 * 735];
        engine.render(&mut block);
    }
    let settled = amb.probe_bed_applied(EmitterKind::Wind)
        .expect("the wind bed should be playing");
    assert!(settled > 0.3, "bed never eased up: {settled}");
    // a firing one-shot dips the duck to -2 dB
    let mut fired = false;
    for f in 300..360u64 {
        let emitters = [wind, homes_emitter(60.0)];
        amb.update(&emitters, 5, f, &mut engine);
        let mut block = [0.0f32; 2 * 735];
        engine.render(&mut block);
        if amb.probe_duck() < 1.0 { fired = true; break; }
    }
    assert!(fired, "no one-shot fired to duck the beds");
    // beds set their applied gain before the one-shot dips the duck in
    // the same update - the dipped applied gain shows next frame
    amb.update(&[wind], 5, 400, &mut engine);
    let dipped = amb.probe_bed_applied(EmitterKind::Wind).unwrap();
    let ratio = dipped / settled;
    assert!((ratio - 0.7943).abs() < 0.05,
        "bed dipped {ratio:.3}, expected ~0.794 (-2 dB)");
    // and it releases toward unity over ~500 ms
    for f in 360..420u64 {
        amb.update(&[wind], 5, f, &mut engine);
        let mut block = [0.0f32; 2 * 735];
        engine.render(&mut block);
    }
    assert!(amb.probe_duck() > 0.9, "duck never released: {}", amb.probe_duck());
}
