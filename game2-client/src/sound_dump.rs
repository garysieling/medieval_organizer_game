//! `sound_dump`: the audio twin of `--dump-frame` (#1493). Build a game
//! (`--seed`, `--stage`, `--scenario`), run N frames through `ui_model()`,
//! render audio offline in lockstep through `AudioHost`, and write WAV +
//! trace + metrics + PNGs. `--scenes` replays a captured `UiModel` stream
//! with no `Game` at all; `--connect` records one off a live server.
use crate::remote;
use game2_audio::analysis;
use game2_audio::host::AudioHost;
use game2_audio::scenarios::{self, Op, ScriptOp};
use game2_audio::soundscape::Listener;
use game2_audio::wav::{self, SAMPLE_RATE};
use game2_core::game::tilemap::TileMap;
use game2_core::session::stage::{self, Stage};
use game2_core::session::{Game, UiModel, MAP_SEED};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

/// One dump run's knobs, parsed by the bin.
pub struct DumpOpts {
    pub seed: u64,
    pub stage: Option<Stage>,
    pub scenario: Option<String>,
    pub frames: u64,
    pub out: String,
    /// Replay this scenes.jsonl instead of building a game
    pub scenes: Option<String>,
    /// Join this server as a human seat and record its scenes
    pub connect: Option<String>,
}

impl Default for DumpOpts {
    fn default() -> Self {
        DumpOpts { seed: MAP_SEED, stage: None, scenario: None,
            frames: 600, out: "sound_dump".into(), scenes: None, connect: None }
    }
}

/// The frame's audio slice: push the model at the listener's ear,
/// render one 60 fps frame of stereo onto `pcm`.
fn render_frame(host: &mut AudioHost, model: &UiModel, listener: &Listener,
                pcm: &mut Vec<f32>) {
    host.push_with(model, listener);
    let n = host.frame_samples();
    let start = pcm.len();
    pcm.resize(start + n * 2, 0.0);
    host.render_block(&mut pcm[start..]);
}

/// A scenario op against the running game: the verbs `record_video`'s
/// scripts use. `Look*` doesn't touch the game - it steers the
/// listener, which `run_live` carries per frame.
fn apply_op(op: &Op, game: &mut Game) {
    match op {
        Op::Cmd(c) => game.command(c.clone()),
        Op::ForceEvent(id) => game.force_event(*id),
        Op::RoadMarch(si) => { let _ = game.force_road_march(0, *si); }
        Op::Stage(s) => stage::stage(game, *s),
        _ => {}
    }
}

/// The ear a `Look` op last moved: resolved against each frame's model
/// (the market, skeps, church and gate move with the seed).
fn listener_at(look: &Option<Op>, m: &UiModel) -> Listener {
    look.as_ref().and_then(|op| scenarios::look_spot(op, m))
        .map(|(x, y)| Listener::at(x, y, 1.0))
        .unwrap_or_else(|| Listener::from_model(m))
}

/// Live play: a seeded or scenario game run N frames.
fn run_live(opts: &DumpOpts, host: &mut AudioHost, pcm: &mut Vec<f32>) -> io::Result<()> {
    let (mut game, script) = match &opts.scenario {
        Some(name) => {
            let sc = scenarios::by_name(name).ok_or_else(|| io::Error::new(io::ErrorKind::Other,
                format!("no scenario named {name}")))?;
            let g = (sc.build)(opts.seed);
            let mut by_frame: BTreeMap<u64, Vec<Op>> = BTreeMap::new();
            for ScriptOp { frame, op } in sc.script { by_frame.entry(frame).or_default().push(op); }
            (g, by_frame)
        }
        None => (Game::new_seeded(opts.seed), BTreeMap::new()),
    };
    let mut script = script;
    let mut look: Option<Op> = None;
    if let Some(s) = opts.stage { stage::stage(&mut game, s); }
    for f in 0..opts.frames {
        if let Some(ops) = script.remove(&f) {
            for op in &ops {
                if matches!(op, Op::Look(..) | Op::LookMarket
                    | Op::LookSkeps | Op::LookChurch | Op::LookGate) {
                    look = Some(match op {
                        Op::Look(x, y) => Op::Look(*x, *y),
                        Op::LookMarket => Op::LookMarket,
                        Op::LookSkeps => Op::LookSkeps,
                        Op::LookChurch => Op::LookChurch,
                        Op::LookGate => Op::LookGate,
                        _ => unreachable!(),
                    });
                } else { apply_op(op, &mut game); }
            }
        }
        game.update();
        let m = game.ui_model();
        let l = listener_at(&look, &m);
        render_frame(host, &m, &l, pcm);
    }
    Ok(())
}

/// Write models as a scenes.jsonl stream: the first line carries the
/// static map, later lines go lean like the wire protocol's scenes.
pub fn write_scenes<W: Write>(w: &mut W, models: &[UiModel]) -> io::Result<()> {
    for (i, m) in models.iter().enumerate() {
        let mut m = m.clone();
        if i > 0 { m.map = TileMap::default(); }
        serde_json::to_writer(&mut *w, &m).map_err(io_err)?;
        w.write_all(b"\n")?;
    }
    Ok(())
}

fn io_err<E: std::fmt::Display>(e: E) -> io::Error { io::Error::new(io::ErrorKind::Other, e.to_string()) }

/// Replay a scenes.jsonl: each line a `UiModel`, the map patched back
/// from the last line that carried it. No `Game` anywhere.
fn run_scenes(path: &str, host: &mut AudioHost, pcm: &mut Vec<f32>) -> io::Result<()> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut map: Option<TileMap> = None;
    for line in file.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let mut m: UiModel = serde_json::from_str(&line).map_err(io_err)?;
        if m.map.width > 0 { map = Some(m.map.clone()); }
        else if let Some(saved) = &map { m.map = saved.clone(); }
        render_frame(host, &m, &Listener::from_model(&m), pcm);
    }
    Ok(())
}

/// Join a server as a human seat, drain its scene stream until `frames`
/// scenes arrive, write the capture to `<out>.scenes.jsonl`, and render
/// what arrived.
fn run_connect(url: &str, opts: &DumpOpts, host: &mut AudioHost, pcm: &mut Vec<f32>) -> io::Result<()> {
    let r = remote::connect_remote(url);
    let mut got = 0u64;
    let mut models: Vec<UiModel> = vec![];
    // a wall-clock cap so a dead seat doesn't hang the dump forever;
    // scenes at irregular intervals are the point of this path
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(120 + opts.frames / 10);
    while got < opts.frames {
        for m in r.drain_scenes() {
            render_frame(host, &m, &Listener::from_model(&m), pcm);
            models.push(m);
            got += 1;
        }
        if let Some(reason) = r.denied.lock().unwrap().as_ref() {
            return Err(io_err(format!("the server turned the seat away: {reason}")));
        }
        if std::time::Instant::now() > deadline { break; }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let path = format!("{}.scenes.jsonl", opts.out);
    write_scenes(&mut io::BufWriter::new(std::fs::File::create(&path)?), &models)?;
    Ok(())
}

/// Render a model stream to stereo f32 through a fresh offline host.
/// The scene roundtrip test renders live models and replayed ones
/// through this same path and compares bytes.
pub fn render_models<'a, I: Iterator<Item = &'a UiModel>>(models: I) -> (Vec<f32>, usize) {
    let mut host = AudioHost::offline(SAMPLE_RATE);
    let mut pcm = vec![];
    for m in models { render_frame(&mut host, m, &Listener::from_model(m), &mut pcm); }
    (pcm, host.voices_peak())
}

/// The run: fill `pcm` from whichever source the flags name, then write
/// the six outputs.
pub fn run(opts: &DumpOpts) -> io::Result<()> {
    let t0 = std::time::Instant::now();
    let mut host = AudioHost::offline(SAMPLE_RATE);
    // the dump's bank comes off the disk like game_ui's; the web
    // loader fetches its bank instead, and sound_dump is a native tool
    #[cfg(not(target_arch = "wasm32"))]
    host.set_bank(game2_audio::sampler::load_dir(std::path::Path::new("assets/sound")));
    let mut pcm = vec![];
    if let Some(path) = &opts.scenes {
        run_scenes(path, &mut host, &mut pcm)?;
    } else if let Some(url) = &opts.connect {
        run_connect(url, opts, &mut host, &mut pcm)?;
    } else {
        run_live(opts, &mut host, &mut pcm)?;
    }
    let render_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let traces = host.drain_trace();
    let notes = host.drain_notes();
    write_outputs(&opts.out, &pcm, host.voices_peak(), render_ms, &traces, &notes)
}

/// `<out>.wav`, `.trace.json`, `.metrics.json`, `.spectrogram.png`,
/// `.roll.png`, `.timeline.png` - the trace is one JSON line per bar
/// the director wrote; the roll and timeline draw from it.
pub fn write_outputs(out: &str, pcm: &[f32], voices_peak: usize, render_ms: f64,
    traces: &[game2_audio::trace::BarTrace],
    notes: &[(u64, game2_audio::compose::NoteEvent)]) -> io::Result<()> {
    if let Some(dir) = std::path::Path::new(out).parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let pcm16: Vec<i16> = pcm.iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).collect();
    wav::write_wav(&format!("{out}.wav"), &pcm16, 2)?;
    let mut tw = io::BufWriter::new(
        std::fs::File::create(format!("{out}.trace.json"))?);
    game2_audio::trace::write_trace(&mut tw, traces)?;
    let loud = analysis::loudness(pcm, SAMPLE_RATE);
    let m = analysis::Metrics {
        lufs_integrated: loud.integrated_lufs,
        true_peak_dbtp: loud.true_peak_dbtp,
        clicks: analysis::clicks(pcm, SAMPLE_RATE).len(),
        longest_silence_s: analysis::longest_silence(pcm, SAMPLE_RATE, -60.0),
        voices_peak,
        render_ms: render_ms as f32,
    };
    std::fs::write(format!("{out}.metrics.json"),
        format!("{}\n", serde_json::to_string_pretty(&m).map_err(io_err)?))?;
    analysis::spectrogram_png(pcm, SAMPLE_RATE, &format!("{out}.spectrogram.png"))?;
    analysis::piano_roll_png(&roll_input(traces, notes), &format!("{out}.roll.png"))?;
    analysis::timeline_png(&timeline_input(traces), &format!("{out}.timeline.png"))?;
    Ok(())
}

/// A mode name's pitch classes on the fixed tonic D (62).
fn mode_pcs(mode: &str) -> [bool; 12] {
    use game2_audio::theory::Mode;
    let m = match mode {
        "Lydian" => Mode::Lydian,
        "Ionian" => Mode::Ionian,
        "Mixolydian" => Mode::Mixolydian,
        "Aeolian" => Mode::Aeolian,
        "Phrygian" => Mode::Phrygian,
        _ => Mode::Dorian,
    };
    let mut pcs = [false; 12];
    for i in m.intervals() {
        pcs[((2 + i) % 12) as usize] = true; // tonic D
    }
    pcs
}

/// The piano roll's input: every fired note against the mode its bar
/// traced, red-flagging anything off the ladder.
fn roll_input(traces: &[game2_audio::trace::BarTrace],
    notes: &[(u64, game2_audio::compose::NoteEvent)]) -> analysis::RollInput {
    // per-bar pitch-class tables, in bar order
    let bar_mode: Vec<([bool; 12], f64, f64)> = traces.iter()
        .map(|b| {
            let end = traces.iter().find(|n| n.bar == b.bar + 1)
                .map(|n| n.at_s).unwrap_or(b.at_s + 2.0);
            (mode_pcs(&b.mode), b.at_s, end)
        }).collect();
    let mut out = [false; 12];
    if let Some((pcs, ..)) = bar_mode.last() { out = *pcs; }
    analysis::RollInput {
        notes: notes.iter().map(|(at, n)| {
            let t = *at as f64 / SAMPLE_RATE as f64;
            let in_mode = bar_mode.iter()
                .find(|(.., lo, hi)| t >= *lo && t < *hi)
                .map(|(pcs, ..)| pcs[(n.midi as usize) % 12])
                .unwrap_or(true);
            analysis::RollNote {
                start_s: t as f32,
                dur_s: n.dur_samples as f32 / SAMPLE_RATE as f32,
                midi: n.midi as u8,
                voice: n.voice as u8,
                in_mode,
            }
        }).collect(),
        mode_pitch_classes: out,
    }
}

/// The timeline's input: tension per second, section bands, the mode
/// strip, per-voice lanes and the stinger markers.
fn timeline_input(traces: &[game2_audio::trace::BarTrace]) -> analysis::TimelineInput {
    const VOICES: [&str; 8] =
        ["Drone", "Pulse", "Ground", "Harmony", "Melody", "Counter", "Color", "Choir"];
    let end_s = traces.last().map(|b| b.at_s + 2.0).unwrap_or(0.0);
    let mut tension = vec![0.0f32; end_s.ceil() as usize + 1];
    let mut sections: Vec<analysis::Section> = vec![];
    let mut mode_lane: Vec<(f32, String)> = vec![];
    let mut lanes: Vec<analysis::Lane> = VOICES.iter()
        .map(|n| analysis::Lane { name: n.to_string(), density: vec![] }).collect();
    let mut markers: Vec<analysis::Marker> = vec![];
    for b in traces {
        let t0 = b.at_s;
        let t1 = traces.iter().find(|n| n.bar == b.bar + 1)
            .map(|n| n.at_s).unwrap_or(end_s);
        for s in t0.floor() as usize..(t1.ceil() as usize).min(tension.len()) {
            tension[s] = b.mood[0];
        }
        if sections.last().map(|s| s.label.as_str()) != Some(b.section.as_str()) {
            sections.push(analysis::Section {
                start_s: t0 as f32, end_s: t1 as f32, label: b.section.clone() });
        } else if let Some(s) = sections.last_mut() {
            s.end_s = t1 as f32;
        }
        if mode_lane.last().map(|(_, m)| m.as_str()) != Some(b.mode.as_str()) {
            mode_lane.push((t0 as f32, b.mode.clone()));
        }
        for (i, l) in lanes.iter_mut().enumerate() {
            l.density.push(b.layers_density[i]);
        }
        for st in &b.stingers {
            markers.push(analysis::Marker { t_s: t0 as f32, label: st.clone() });
        }
    }
    analysis::TimelineInput { tension, sections, mode_lane, layers: lanes, markers }
}
