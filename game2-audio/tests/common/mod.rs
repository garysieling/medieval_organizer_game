//! The scenario render harness shared by the mix, golden and bit-exact
//! suites (I1): a `SampleBank` of the real `assets/sound` content plus
//! synthesised stand-ins for whatever the fetch hasn't landed, and a
//! `render_scenario` that runs a named scenario through `AudioHost`
//! exactly the way `sound_dump` does - ops applied on their frames,
//! `Look*` steering the ear, one 60 fps block per update, and the
//! director writing the score off each pushed model.
#![allow(dead_code)]

use game2_audio::host::{AudioHost};
use game2_audio::sampler::SampleBank;
use game2_audio::scenarios::{self, look_spot, Op, ScriptOp};
use game2_audio::soundscape::Listener;
use game2_audio::trace::BarTrace;
use game2_audio::wav::SAMPLE_RATE;
use game2_audio::compose::NoteEvent;
use std::collections::BTreeMap;

/// Every patch name the ambience mixer may ask for
pub const BATCH: &[&str] = &[
    "birds", "insects", "bees", "river", "village_murmur", "market_crowd",
    "fire_crackle", "church_bell", "anvil_hit", "sheep_bleat", "chicken",
    "dog_bark", "door_creak", "cart_wheel", "gate_creak", "crow", "hoofbeat",
];

/// A tiny deterministic 16-bit mono WAV: a sine partial over shaped
/// noise, `amp` peak - enough body to exercise the mix without
/// pretending to be the real recording. `seed` varies the texture so
/// distinct ids don't share a waveform. Loops crossfade their tail
/// into the head, like the fetch pipeline does, so the wrap is seamless.
pub fn fake_wav(id: &str, secs: f32, sr: u32, amp: f32, loops: bool) -> Vec<u8> {
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
pub fn test_bank() -> SampleBank {
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

/// What one scenario render produced: the stereo signal, the voices
/// the engine peaked at, wall time, and the score's ledgers (bar
/// trace and note log - the golden file and gap checks read these).
pub struct Render {
    pub pcm: Vec<f32>,
    pub voices_peak: usize,
    pub render_secs: f32,
    pub traces: Vec<BarTrace>,
    pub notes: Vec<(u64, NoteEvent)>,
}

/// A scenario rendered through the host like `sound_dump` does: ops
/// applied on their frames, `Look*` steering the listener, and
/// `push_with` so the director writes the score off the same `UiModel`
/// the game thread hands the live host.
pub fn render_scenario(name: &str, frames: u64) -> Render {
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
                    Op::Stage(s) => game2_core::session::stage::stage(&mut g, *s),
                    Op::FireMarket => { scenarios::ignite_at_market(&mut g); }
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
        host.push_with(&m, &l);
        let n = host.frame_samples();
        let start = pcm.len();
        pcm.resize(start + n * 2, 0.0);
        let t = std::time::Instant::now();
        host.render_block(&mut pcm[start..]);
        render_secs += t.elapsed().as_secs_f32();
    }
    Render {
        pcm,
        voices_peak: host.voices_peak(),
        render_secs,
        traces: host.drain_trace(),
        notes: host.drain_notes(),
    }
}
