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
mod common;
use common::{fake_wav, render_scenario};
use game2_audio::analysis;
use game2_audio::engine::Engine;
use game2_audio::sampler::SampleBank;
use game2_audio::soundscape::ambience::Ambience;
use game2_audio::soundscape::{Emitter, EmitterKind, Tier};
use game2_audio::wav::SAMPLE_RATE;

/// The design's Mix bar for a scenario render (60 s at 60 fps)
fn check_scenario(name: &str) {
    let r = render_scenario(name, 3600);
    let pcm = r.pcm;
    let voices = r.voices_peak;
    let elapsed = r.render_secs;
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
fn batch_animal_shots_shipped() { assert_shipped(&["sheep_bleat", "chicken", "dog_bark", "crow"]); }

#[test]
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
