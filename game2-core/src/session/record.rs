//! Headless video recording: run the sim as fast as the CPU allows and
//! encode the rendered frames into a video file - the same `View::render`
//! the windowed front-end uses, so a clip is exactly what a player would
//! have seen.
//!
//! "Play time" is measured in game frames (`FPS` per second of play); the
//! `--speedup` option compresses that into watch time, so a half hour of
//! village life becomes a reviewable clip. Sinks write GIF (no external
//! tools needed), APNG (lossless), a PNG frame sequence, or pipe raw
//! frames to an `ffmpeg` binary for mp4/webm (and, later, an rtmp stream).
//!
//! A script file can drive the player for test footage: one command per
//! line, `<frame> <verb> [args]` -
//!   120 walk 1 0     held walk until `stop` (like WASD held)
//!   340 stop         stop walking
//!   360 act 0        press action button i, 0-based (0 Talk .. 4 Build)
//!   400 select 3     the 1-8 keys
//!   420 ctx 0        a contextual action (Work, Pair up, ...)
//!   440 click 700 400   click the map through the camera
//!   500 pause        toggle pause
//!   600 speed        toggle 1x/4x
//!   700 look 40 60   move the camera to a map tile
//!   750 cancel       drop a walk-over-to-act in progress

use std::collections::BTreeMap;
use std::io::Write;

use crate::engine::render::Buffer;
use super::{action_buttons, camera_on, Art, Command, Game, View, FPS};

/// Which file the frames land in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    /// Animated GIF - plays anywhere, quantized to 256 colours a frame
    Gif,
    /// Animated PNG - lossless, plays in browsers, larger files
    Apng,
    /// One PNG per frame in a directory - for post-hoc tooling
    Frames,
    /// Pipe raw frames to an `ffmpeg` binary (mp4, webm, ... or rtmp)
    Ffmpeg,
}

/// One line of a script: either a player Command or a camera move (the
/// camera lives in the View, not the Game, so it isn't a Command)
#[derive(Debug, Clone)]
pub enum ScriptOp {
    Cmd(Command),
    /// Move the camera to a map tile
    Look(i32, i32),
}

/// Everything a recording run needs to know
pub struct RecordOpts {
    /// Seconds of play time to simulate
    pub play_secs: u64,
    /// Game-time compression: watch seconds = play_secs / speedup
    pub speedup: f64,
    /// Video frames per second in the output file
    pub fps: u32,
    /// Downscale factor for the output frames (1.0 = full 1440x960)
    pub scale: f32,
    pub format: Format,
    /// Output path (a directory for `Frames`)
    pub out: String,
    pub zoom: f32,
    pub cam: Option<(i32, i32)>,
    /// In-game speed flag: 1 = normal days, 4 = turbo days
    pub game_speed: u32,
    /// A script of timed ops to feed the player seat and camera
    pub script: Vec<(u64, ScriptOp)>,
    /// Path to an ffmpeg binary for `Format::Ffmpeg`
    pub ffmpeg: String,
}

impl Default for RecordOpts {
    fn default() -> Self {
        Self {
            play_secs: 10,
            speedup: 1.0,
            fps: 12,
            scale: 1.0,
            format: Format::Gif,
            out: String::new(),
            zoom: 1.0,
            cam: None,
            game_speed: 1,
            script: Vec::new(),
            ffmpeg: "ffmpeg".to_string(),
        }
    }
}

/// Game frames between captured frames: at 1x a 12fps video keeps every
/// fifth render, at 60x it keeps one in three hundred
pub fn capture_every(speedup: f64, fps: u32) -> u64 {
    ((FPS as f64 * speedup) / fps.max(1) as f64).round().max(1.0) as u64
}

/// Parse a script file: `<frame> <verb> [args...]`, `#` comments, blanks ok
pub fn parse_script(text: &str) -> Result<Vec<(u64, ScriptOp)>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() { continue; }
        let err = |why: &str| format!("script line {}: {} ({:?})", n + 1, why, line);
        let mut it = line.split_whitespace();
        let frame: u64 = it.next().and_then(|v| v.parse().ok()).ok_or_else(|| err("bad frame number"))?;
        let verb = it.next().ok_or_else(|| err("missing verb"))?;
        let num = |it: &mut std::str::SplitWhitespace| -> Result<f32, String> {
            it.next().and_then(|v| v.parse().ok()).ok_or_else(|| err("bad number"))
        };
        let op = match verb {
            "walk" => ScriptOp::Cmd(Command::Walk(num(&mut it)?, num(&mut it)?)),
            "stop" => ScriptOp::Cmd(Command::Walk(0.0, 0.0)),
            // act N beyond the buttons panics in the sim - catch it at parse time
            "act" => {
                let i = num(&mut it)? as usize;
                if i >= action_buttons().len() { return Err(err("act index out of range")); }
                ScriptOp::Cmd(Command::Act(i))
            }
            "ctx" => ScriptOp::Cmd(Command::CtxAct(num(&mut it)? as usize)),
            "select" => ScriptOp::Cmd(Command::Select(num(&mut it)? as usize)),
            "click" => ScriptOp::Cmd(Command::ClickMap {
                mx: num(&mut it)? as i32, my: num(&mut it)? as i32,
                cam_x: 0, cam_y: 0, zoom: 1.0, // filled in at apply time
            }),
            "look" => ScriptOp::Look(num(&mut it)? as i32, num(&mut it)? as i32),
            "pause" => ScriptOp::Cmd(Command::TogglePause),
            "speed" => ScriptOp::Cmd(Command::ToggleSpeed),
            "cancel" => ScriptOp::Cmd(Command::CancelApproach),
            _ => return Err(err("unknown verb")),
        };
        out.push((frame, op));
    }
    out.sort_by_key(|(f, _)| *f);
    Ok(out)
}

/// What a frame lands in. `push` takes the buffer by value - the GIF sink
/// quantizes it in place.
trait Sink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()>;
    fn finish(self: Box<Self>) -> std::io::Result<()>;
}

/// gif crate's encoder over a File
struct GifSink { enc: gif::Encoder<std::fs::File>, delay_cs: u16 }

/// png crate's APNG encoder; num/den hold the per-frame delay fraction
struct ApngSink { w: png::Writer<std::io::BufWriter<std::fs::File>>, num: u16, den: u16 }

/// `out/frame-00000.png`, `frame-00001.png`, ...
struct FramesSink { dir: std::path::PathBuf, n: u32 }

/// Raw RGBA frames piped to an ffmpeg child's stdin
struct FfmpegSink { child: std::process::Child }

impl Drop for FfmpegSink {
    // an error mid-run mustn't leave an ffmpeg holding an open pipe
    fn drop(&mut self) { let _ = self.child.kill(); }
}

fn open_sink(opts: &RecordOpts, w: usize, h: usize, n_frames: u64) -> std::io::Result<Box<dyn Sink>> {
    let other = |e: String| std::io::Error::new(std::io::ErrorKind::Other, e);
    match opts.format {
        Format::Gif => {
            let f = std::fs::File::create(&opts.out)?;
            let mut enc = gif::Encoder::new(f, w as u16, h as u16, &[])
                .map_err(|e| other(format!("gif encoder: {}", e)))?;
            enc.set_repeat(gif::Repeat::Infinite).map_err(|e| other(e.to_string()))?;
            // 2cs floor: viewers clamp shorter delays to ~50fps anyway
            Ok(Box::new(GifSink { enc, delay_cs: (100.0 / opts.fps.max(1) as f64).round().max(2.0) as u16 }))
        }
        Format::Apng => {
            let f = std::io::BufWriter::new(std::fs::File::create(&opts.out)?);
            let mut enc = png::Encoder::new(f, w as u32, h as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::Fast);
            enc.set_animated(n_frames.max(1) as u32, 0).map_err(|e| other(e.to_string()))?;
            let mut w = enc.write_header().map_err(|e| other(e.to_string()))?;
            w.set_frame_delay(1, opts.fps.max(1) as u16).map_err(|e| other(e.to_string()))?;
            Ok(Box::new(ApngSink { w, num: 1, den: opts.fps.max(1) as u16 }))
        }
        Format::Frames => {
            let dir = std::path::PathBuf::from(&opts.out);
            std::fs::create_dir_all(&dir)?;
            Ok(Box::new(FramesSink { dir, n: 0 }))
        }
        Format::Ffmpeg => {
            // codec follows the extension: webm gets vp9, the rest h264
            let codec: &[&str] = if opts.out.ends_with(".webm") { &["-c:v", "libvpx-vp9", "-b:v", "2M"] }
                else { &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20"] };
            let child = std::process::Command::new(&opts.ffmpeg)
                .args(["-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba",
                    "-s", &format!("{}x{}", w, h), "-r", &opts.fps.to_string(), "-i", "-"])
                .args(codec).arg(&opts.out)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| other(format!("can't run {}: {} - install ffmpeg or pass --ffmpeg-path", opts.ffmpeg, e)))?;
            Ok(Box::new(FfmpegSink { child }))
        }
    }
}

impl Sink for GifSink {
    fn push(&mut self, mut buf: Buffer) -> std::io::Result<()> {
        let mut frame = gif::Frame::from_rgba_speed(buf.width as u16, buf.height as u16, &mut buf.pixels, 10);
        frame.delay = self.delay_cs;
        self.enc.write_frame(&frame).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
    fn finish(self: Box<Self>) -> std::io::Result<()> { Ok(()) } // Drop writes the trailer
}

impl Sink for ApngSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        self.w.set_frame_delay(self.num, self.den)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        self.w.write_image_data(&buf.pixels)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
    fn finish(self: Box<Self>) -> std::io::Result<()> {
        self.w.finish().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

impl Sink for FramesSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        let other = |e: png::EncodingError| std::io::Error::new(std::io::ErrorKind::Other, e.to_string());
        let path = self.dir.join(format!("frame-{:05}.png", self.n));
        self.n += 1;
        let f = std::io::BufWriter::new(std::fs::File::create(path)?);
        let mut enc = png::Encoder::new(f, buf.width as u32, buf.height as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        let mut w = enc.write_header().map_err(other)?;
        w.write_image_data(&buf.pixels).map_err(other)
    }
    fn finish(self: Box<Self>) -> std::io::Result<()> { Ok(()) }
}

impl Sink for FfmpegSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        let stdin = self.child.stdin.as_mut().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "ffmpeg stdin closed"))?;
        stdin.write_all(&buf.pixels)
    }
    fn finish(mut self: Box<Self>) -> std::io::Result<()> {
        drop(self.child.stdin.take());
        let status = self.child.wait()?;
        if status.success() { Ok(()) } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("ffmpeg exited {}", status)))
        }
    }
}

/// One scripted op's effect on the run: camera ops touch the View,
/// commands go to the game, and `walk` arms the held-key resend
fn apply_op(op: &ScriptOp, game: &mut Game, view: &mut View, held_walk: &mut Option<(f32, f32)>) {
    match op {
        ScriptOp::Look(tx, ty) => { (view.cam_x, view.cam_y) = camera_on(*tx as f32, *ty as f32); }
        // a `stop` lifts the key; the held resend below walks a real `walk`
        ScriptOp::Cmd(Command::Walk(dx, dy)) => {
            *held_walk = if *dx == 0.0 && *dy == 0.0 { None } else { Some((*dx, *dy)) };
        }
        ScriptOp::Cmd(Command::ClickMap { mx, my, .. }) => game.command(Command::ClickMap {
            mx: *mx, my: *my, cam_x: view.cam_x, cam_y: view.cam_y, zoom: view.zoom,
        }),
        ScriptOp::Cmd(c) => game.command(c.clone()),
    }
}

/// Play the game with no window for `play_secs` of play time, rendering a
/// video frame every `capture_every` game frames. Returns the output path.
pub fn run_recording(art: Art, mut opts: RecordOpts, seed: u64) -> std::io::Result<String> {
    let mut game = Game::new_seeded(seed);
    game.speed = opts.game_speed;
    let mut view = View::new(art);
    view.zoom = opts.zoom;
    if let Some(cam) = opts.cam { (view.cam_x, view.cam_y) = cam; }

    let mut script: BTreeMap<u64, Vec<ScriptOp>> = BTreeMap::new();
    for (f, op) in std::mem::take(&mut opts.script) { script.entry(f).or_default().push(op); }
    let mut held_walk: Option<(f32, f32)> = None;
    // ops keyed to the first frame run before its render
    if let Some(ops) = script.remove(&0) {
        for op in &ops { apply_op(op, &mut game, &mut view, &mut held_walk); }
    }

    let every = capture_every(opts.speedup, opts.fps);
    let total_frames = opts.play_secs * FPS as u64;
    let n_frames = total_frames.div_ceil(every);

    // a probe frame sizes the sinks (and the scale option) before the run
    let probe = view.render(&game.ui_model(), (-1, -1));
    let probe = if (opts.scale - 1.0).abs() < f32::EPSILON { probe } else { probe.scaled(opts.scale) };
    let mut sink = open_sink(&opts, probe.width, probe.height, n_frames)?;
    sink.push(probe)?;

    let mut captured = 1u64;
    for f in 1..total_frames {
        if let Some(ops) = script.remove(&f) {
            for op in &ops { apply_op(op, &mut game, &mut view, &mut held_walk); }
        }
        if let Some((dx, dy)) = held_walk { game.command(Command::Walk(dx, dy)); }
        game.update();
        if f % every == 0 {
            let buf = view.render(&game.ui_model(), (-1, -1));
            sink.push(if (opts.scale - 1.0).abs() < f32::EPSILON { buf } else { buf.scaled(opts.scale) })?;
            captured += 1;
            if captured % 25 == 0 {
                println!("{}/{} frames ({}% of play time)", captured, n_frames, f * 100 / total_frames.max(1));
            }
        }
    }
    if let Some((&f, _)) = script.iter().next() {
        eprintln!("warning: script ops at frame {}+ never ran (recording ends at {})", f, total_frames - 1);
    }
    sink.finish()?;
    if opts.format == Format::Frames {
        println!("{} video frames over {} play-seconds -> {}", captured, opts.play_secs, opts.out);
    } else {
        let size = std::fs::metadata(&opts.out).map(|m| m.len()).unwrap_or(0);
        println!("{} video frames over {} play-seconds -> {} ({} bytes)", captured, opts.play_secs, opts.out, size);
    }
    Ok(opts.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_every_scales_with_speedup() {
        // 60fps game, 12fps video: every 5th frame at 1x, 300th at 60x
        assert_eq!(capture_every(1.0, 12), 5);
        assert_eq!(capture_every(60.0, 12), 300);
        assert_eq!(capture_every(1.0, 60), 1);
        assert_eq!(capture_every(0.0, 12), 1); // degenerate: never divide to zero
    }

    #[test]
    fn test_parse_script_commands() {
        let s = "# a comment\n\n120 walk 1 0\n340 stop\n360 act 1\n400 select 3\n420 ctx 0\n440 click 700 400\n500 pause\n600 speed\n700 look 40 60\n750 cancel\n";
        let ops = parse_script(s).unwrap();
        assert_eq!(ops.len(), 10);
        assert!(matches!(ops[0], (120, ScriptOp::Cmd(Command::Walk(dx, dy))) if dx == 1.0 && dy == 0.0));
        assert!(matches!(ops[1], (340, ScriptOp::Cmd(Command::Walk(0.0, 0.0)))));
        assert!(matches!(ops[2], (360, ScriptOp::Cmd(Command::Act(1)))));
        assert!(matches!(ops[3], (400, ScriptOp::Cmd(Command::Select(3)))));
        assert!(matches!(ops[4], (420, ScriptOp::Cmd(Command::CtxAct(0)))));
        assert!(matches!(ops[5], (440, ScriptOp::Cmd(Command::ClickMap { mx: 700, my: 400, .. }))));
        assert!(matches!(ops[6], (500, ScriptOp::Cmd(Command::TogglePause))));
        assert!(matches!(ops[7], (600, ScriptOp::Cmd(Command::ToggleSpeed))));
        assert!(matches!(ops[8], (700, ScriptOp::Look(40, 60))));
        assert!(matches!(ops[9], (750, ScriptOp::Cmd(Command::CancelApproach))));
        assert!(parse_script("oops").is_err());
        assert!(parse_script("10 frobnicate").is_err());
    }

    #[test]
    fn test_script_sorts_by_frame() {
        let ops = parse_script("300 act 0\n10 pause\n").unwrap();
        assert_eq!(ops[0].0, 10);
        assert_eq!(ops[1].0, 300);
    }

    #[test]
    fn test_short_recording_gif_and_apng() {
        let dir = std::env::temp_dir().join(format!("game2-rec-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // one second of play -> five 12fps frames; assets load relative to
        // the crate root the test runner cd's into
        for (format, name, magic) in [
            (Format::Gif, "t.gif", b"GIF8".as_slice()),
            (Format::Apng, "t.png", b"\x89PNG".as_slice()),
        ] {
            let out = dir.join(name).to_string_lossy().to_string();
            run_recording(Art::load_fs("../assets", 7),
                RecordOpts { play_secs: 1, out: out.clone(), format, ..Default::default() }, 7).unwrap();
            let bytes = std::fs::read(&out).unwrap();
            assert!(bytes.len() > magic.len() && &bytes[..magic.len()] == magic, "{} bad magic", name);
        }
        // the APNG must carry an acTL chunk to animate
        let apng = std::fs::read(dir.join("t.png")).unwrap();
        assert!(apng.windows(4).any(|w| w == b"acTL"), "no animation control chunk");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
