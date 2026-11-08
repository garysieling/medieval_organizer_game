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

//!   700 look 40 60   move the camera to a map tile
//!   750 cancel       drop a walk-over-to-act in progress

use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::io::Write;

use game2_core::engine::render::Buffer;
use game2_core::session::{action_buttons, camera_on, Command, Game, FPS, MAP_TILES};
use super::{Art, View};

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
    /// MJPEG + PCM in an AVI - sound without any external tools
    Avi,
}

/// The named time compressions: `--speed` takes one of these, `--speedup`
/// takes any number. A 10x clip squeezes ten game-seconds into one.
pub fn speed_preset(name: &str) -> Option<f64> {
    match name {
        "realtime" | "1x" => Some(1.0),
        "2x" => Some(2.0),
        "5x" => Some(5.0),
        "10x" => Some(10.0),
        "60x" => Some(60.0),
        _ => None,
    }
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
    /// Synth the game's sfx cues into a soundtrack (Avi always has one;
    /// for Ffmpeg this opts in)
    pub audio: bool,
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
            audio: false,
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
            "cancel" => ScriptOp::Cmd(Command::CancelApproach),
            _ => return Err(err("unknown verb")),
        };
        out.push((frame, op));
    }
    out.sort_by_key(|(f, _)| *f);
    Ok(out)
}

/// A sound cue caught in flight: the game frame it fired on, what it was,
/// and where on the map it happened (for stereo pan)
#[derive(Debug, Clone)]
pub struct AudioEvent {
    pub frame: u64,
    pub kind: &'static str,
    pub pan: f32,   // 0 left .. 1 right
    pub gain: f32,  // 0..1, the cue's intensity
}

pub const SAMPLE_RATE: u32 = 44100;

/// One cue's voice: a short synthesised something for each kind the game
/// names. Returns interleaved stereo samples laid on the `pcm` timeline.
fn voice(kind: &str, t: f32, x: f32) -> f32 {
    // x is a quiet noise source reused by several voices
    let noise = x * 2.0 - 1.0;
    match kind {
        // hiss under the weather: low-passed noise, mid decay
        "rain" => noise * (-3.0 * t).exp() * 0.5,
        // the smithy: an inharmonic clang, fast decay
        "anvil" => ((2.0 * PI * 815.0 * t).sin() + 0.5 * (2.0 * PI * 2130.0 * t).sin())
            * (-12.0 * t).exp() * 0.6,
        // a fair day: a major triad, a long ring
        "festival" => ((2.0 * PI * 392.0 * t).sin() + (2.0 * PI * 494.0 * t).sin()
            + (2.0 * PI * 587.0 * t).sin()) / 3.0 * (-3.0 * t).exp() * 0.7,
        // the church bell: low inharmonic partials, a long ring
        "bell" => ((2.0 * PI * 220.0 * t).sin() + 0.6 * (2.0 * PI * 523.0 * t).sin()
            + 0.4 * (2.0 * PI * 891.0 * t).sin()) * (-2.5 * t).exp() * 0.7,
        // a comet or a blessing: a high tone that swells then fades
        "wonder" => (2.0 * PI * 1175.0 * t).sin()
            * (1.0 - (-4.0 * t).exp()) * (-3.0 * t).exp() * 0.6,
        // the snowball fight's hubbub: warbling children shrieks, short
        "snowballs" => ((2.0 * PI * 660.0 * t).sin() + (2.0 * PI * 880.0 * t).sin())
            * (0.5 + 0.5 * (2.0 * PI * 9.0 * t).sin()) * (-8.0 * t).exp() * 0.5,
        // a bucket on the flames: a short crackle of water
        "splash" => noise.signum() * noise.abs() * (-14.0 * t).exp() * 0.5,
        // anything else: a small blip so it isn't silent
        _ => (2.0 * PI * 520.0 * t).sin() * (-15.0 * t).exp() * 0.4,
    }
}

/// How long each cue's voice rings for, in seconds
fn voice_len(kind: &str) -> f32 {
    match kind { "rain" => 0.8, "anvil" => 0.4, "festival" => 1.4, "wonder" => 1.5, "bell" => 2.0, "snowballs" => 0.9, "splash" => 0.3, _ => 0.25 }
}

/// Render captured cues into 16-bit stereo PCM on the WATCH timeline:
/// game frame `f` lands `f / (every * fps)` seconds into the video.
pub fn synth_track(events: &[AudioEvent], every: u64, fps: u32, watch_secs: f64) -> Vec<i16> {
    let rate = SAMPLE_RATE as usize;
    let mut pcm = vec![0i16; (watch_secs * rate as f64).ceil() as usize * 2];
    let mut rng: u32 = 0x9e3779b9;
    for ev in events {
        let at = (ev.frame as f64 / (every as u64 * fps.max(1) as u64) as f64 * rate as f64) as usize;
        let pan = ev.pan.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
        let (lg, rg) = (pan.cos(), pan.sin());
        let n = (voice_len(ev.kind) * rate as f32) as usize;
        for i in 0..n {
            // cheap noise for the voices that want it
            rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
            let x = rng as f32 / u32::MAX as f32;
            let t = i as f32 / rate as f32;
            // a ~15ms attack and a tail taper - hard edges click
            let env = (1.0 - (-60.0 * t).exp()) * ((n - i).min(200) as f32 / 200.0).min(1.0);
            let s = voice(ev.kind, t, x) * ev.gain * env;
            let (li, ri) = ((at + i) * 2, (at + i) * 2 + 1);
            if ri < pcm.len() {
                pcm[li] = pcm[li].saturating_add((s * lg * i16::MAX as f32) as i16);
                pcm[ri] = pcm[ri].saturating_add((s * rg * i16::MAX as f32) as i16);
            }
        }
    }
    pcm
}

/// A 16-bit PCM WAV: 44-byte header + samples. For the ffmpeg mux pass
/// and for anyone wanting the soundtrack on its own.
pub fn write_wav(path: &str, pcm: &[i16], channels: u16) -> std::io::Result<()> {
    let data = (pcm.len() * 2) as u32;
    let byte_rate = SAMPLE_RATE * channels as u32 * 2;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF"); out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt "); out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data"); out.extend_from_slice(&data.to_le_bytes());
    for s in pcm { out.extend_from_slice(&s.to_le_bytes()); }
    std::fs::write(path, out)
}

/// What a frame lands in. `push` takes the buffer by value - the GIF sink
/// quantizes it in place. `finish` gets the synthesised soundtrack
/// (interleaved stereo at SAMPLE_RATE) for formats that carry one.
trait Sink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()>;
    fn finish(self: Box<Self>, audio: Option<&[i16]>) -> std::io::Result<()>;
}

/// gif crate's encoder over a File
struct GifSink { enc: gif::Encoder<std::fs::File>, delay_cs: u16 }

/// png crate's APNG encoder; num/den hold the per-frame delay fraction
struct ApngSink { w: png::Writer<std::io::BufWriter<std::fs::File>>, num: u16, den: u16 }

/// `out/frame-00000.png`, `frame-00001.png`, ...
struct FramesSink { dir: std::path::PathBuf, n: u32 }

/// Raw RGBA frames piped to an ffmpeg child's stdin; when a soundtrack is
/// coming, the video lands at `video_path` first and a second ffmpeg pass
/// muxes it with the wav into `out`
struct FfmpegSink {
    child: std::process::Child,
    ffmpeg: String,
    video_path: String,
    out: String,
}

impl Drop for FfmpegSink {
    // an error mid-run mustn't leave an ffmpeg holding an open pipe
    fn drop(&mut self) { let _ = self.child.kill(); }
}

/// An AVI written by hand: MJPEG frames (`00dc`) plus a PCM track
/// (`01wb`). Video chunks stream out as they render; the soundtrack lands
/// at the end and `idx1` indexes the lot. Header chunks are written as
/// placeholders up front and overwritten once the counts are known.
struct AviSink {
    w: std::io::BufWriter<std::fs::File>,
    pos: u64,                // running file offset
    riff_size_at: u64,       // offset of the RIFF size field
    movi_size_at: u64,
    movi_data_at: u64,       // first byte after the 'movi' fourcc
    patch_ats: Vec<u64>,     // data offsets of the five placeholder chunks
    idx: Vec<u8>,            // idx1 payload, built as chunks land
    n_frames: u32, max_chunk: u32,
    width: u16, height: u16, fps: u32,
}

impl AviSink {
    /// 'LIST' + placeholder size + fourcc; returns the size field's offset
    fn list_open(&mut self, kind: &[u8; 4]) -> std::io::Result<u64> {
        let at = self.pos + 4;
        self.w.write_all(b"LIST")?;
        self.w.write_all(&0u32.to_le_bytes())?;
        self.w.write_all(kind)?;
        self.pos += 12;
        Ok(at)
    }

    /// A chunk's bytes on the wire (header chunks don't enter the index)
    fn write_chunk(&mut self, fourcc: &[u8; 4], data: &[u8]) -> std::io::Result<()> {
        self.w.write_all(fourcc)?;
        self.w.write_all(&(data.len() as u32).to_le_bytes())?;
        self.w.write_all(data)?;
        if data.len() % 2 == 1 { self.w.write_all(&[0])?; }
        self.pos += 8 + data.len() as u64 + (data.len() % 2) as u64;
        Ok(())
    }

    /// A movi chunk: on the wire plus an idx1 entry (offsets are relative
    /// to the start of the movi list's data)
    fn idx_chunk(&mut self, fourcc: &[u8; 4], data: &[u8], flags: u32) -> std::io::Result<()> {
        self.idx.extend_from_slice(fourcc);
        self.idx.extend_from_slice(&flags.to_le_bytes());
        // idx1 offsets run from the 'movi' fourcc itself (ffmpeg's
        // convention), and movi_data_at sits four bytes past it
        self.idx.extend_from_slice(&((self.pos - self.movi_data_at + 4) as u32).to_le_bytes());
        self.idx.extend_from_slice(&(data.len() as u32).to_le_bytes());
        self.write_chunk(fourcc, data)
    }

    /// Zeroed header chunk: bytes now, content patched in at finish
    fn placeholder(&mut self, fourcc: &[u8; 4], len: usize) -> std::io::Result<()> {
        self.patch_ats.push(self.pos + 8);
        self.write_chunk(fourcc, &vec![0u8; len])
    }

    /// A list's size field = bytes after it (data start - field offset - 4)
    fn list_close(&mut self, size_at: u64) -> std::io::Result<()> {
        self.patch_u32(size_at, (self.pos - size_at - 4) as u32)
    }

    fn patch_u32(&mut self, at: u64, v: u32) -> std::io::Result<()> {
        self.patch_data(at, &v.to_le_bytes())
    }

    /// Flush, seek, overwrite, seek back to the end
    fn patch_data(&mut self, at: u64, data: &[u8]) -> std::io::Result<()> {
        use std::io::Seek;
        self.w.flush()?;
        let f = self.w.get_mut();
        f.seek(std::io::SeekFrom::Start(at))?;
        f.write_all(data)?;
        f.seek(std::io::SeekFrom::End(0))?;
        Ok(())
    }
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
            // codec follows the extension: webm gets vp9, the rest h264.
            // With audio, the video-only pass lands in a temp file first.
            let webm = opts.out.ends_with(".webm");
            let codec: &[&str] = if webm { &["-c:v", "libvpx-vp9", "-b:v", "2M"] }
                else { &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20"] };
            let video_path = if opts.audio { format!("{}.tmpvideo", opts.out) } else { opts.out.clone() };
            let child = std::process::Command::new(&opts.ffmpeg)
                .args(["-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba",
                    "-s", &format!("{}x{}", w, h), "-r", &opts.fps.max(1).to_string(), "-i", "-"])
                .args(codec)
                .args(["-f", if webm { "webm" } else { "mp4" }])
                .arg(&video_path)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| other(format!("can't run {}: {} - install ffmpeg (scripts/get-ffmpeg.ps1) or pass --ffmpeg-path", opts.ffmpeg, e)))?;
            Ok(Box::new(FfmpegSink { child, ffmpeg: opts.ffmpeg.clone(), video_path, out: opts.out.clone() }))
        }
        Format::Avi => {
            let file = std::fs::File::create(&opts.out)?;
            let mut s = AviSink {
                w: std::io::BufWriter::new(file), pos: 0,
                riff_size_at: 0, movi_size_at: 0, movi_data_at: 0,
                patch_ats: Vec::new(), idx: Vec::new(),
                n_frames: 0, max_chunk: 0,
                width: w as u16, height: h as u16, fps: opts.fps.max(1),
            };
            s.w.write_all(b"RIFF\0\0\0\0AVI ")?; // size patched at finish
            s.riff_size_at = s.pos + 4;
            s.pos += 12;
            let hdrl = s.list_open(b"hdrl")?;
            s.placeholder(b"avih", 56)?;
            let vstrl = s.list_open(b"strl")?;
            s.placeholder(b"strh", 56)?;
            s.placeholder(b"strf", 40)?;
            s.list_close(vstrl)?;
            let astrl = s.list_open(b"strl")?;
            s.placeholder(b"strh", 56)?;
            s.placeholder(b"strf", 16)?;
            s.list_close(astrl)?;
            s.list_close(hdrl)?;
            s.movi_size_at = s.list_open(b"movi")?;
            s.movi_data_at = s.pos;
            Ok(Box::new(s))
        }
    }
}

impl Sink for GifSink {
    fn push(&mut self, mut buf: Buffer) -> std::io::Result<()> {
        let mut frame = gif::Frame::from_rgba_speed(buf.width as u16, buf.height as u16, &mut buf.pixels, 10);
        frame.delay = self.delay_cs;
        self.enc.write_frame(&frame).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
    fn finish(self: Box<Self>, _audio: Option<&[i16]>) -> std::io::Result<()> { Ok(()) } // Drop writes the trailer
}

impl Sink for ApngSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        self.w.set_frame_delay(self.num, self.den)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        self.w.write_image_data(&buf.pixels)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
    fn finish(self: Box<Self>, _audio: Option<&[i16]>) -> std::io::Result<()> {
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
    fn finish(self: Box<Self>, _audio: Option<&[i16]>) -> std::io::Result<()> { Ok(()) }
}

impl Sink for FfmpegSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        let stdin = self.child.stdin.as_mut().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "ffmpeg stdin closed"))?;
        stdin.write_all(&buf.pixels)
    }
    fn finish(mut self: Box<Self>, audio: Option<&[i16]>) -> std::io::Result<()> {
        let other = |e: String| std::io::Error::new(std::io::ErrorKind::Other, e);
        drop(self.child.stdin.take());
        if !self.child.wait()?.success() {
            let _ = std::fs::remove_file(&self.video_path);
            return Err(other("ffmpeg video pass failed".into()));
        }
        let Some(pcm) = audio else {
            return Ok(()); // the video landed at out directly
        };
        // mux pass: video stream copy + the wav as aac/opus, webm speaks
        // opus; faststart puts the mp4 header up front for browsers
        let wav = format!("{}.tmpsound.wav", self.out);
        if let Err(e) = write_wav(&wav, pcm, 2) {
            let _ = std::fs::remove_file(&self.video_path);
            return Err(e);
        }
        let acodec: &[&str] = if self.out.ends_with(".webm") { &["-c:a", "libopus"] }
            else { &["-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart"] };
        let status = std::process::Command::new(&self.ffmpeg)
            .args(["-loglevel", "error", "-y", "-i", &self.video_path, "-i", &wav])
            .args(["-c:v", "copy"]).args(acodec).args(["-shortest", &self.out])
            .status();
        let _ = std::fs::remove_file(&wav);
        let _ = std::fs::remove_file(&self.video_path);
        let status = status.map_err(|e| other(e.to_string()))?;
        if status.success() { Ok(()) } else { Err(other(format!("ffmpeg mux pass exited {}", status))) }
    }
}

impl Sink for AviSink {
    fn push(&mut self, buf: Buffer) -> std::io::Result<()> {
        // RGBA -> RGB for the JPEG
        let mut rgb = Vec::with_capacity(buf.width * buf.height * 3);
        for px in buf.pixels.chunks_exact(4) { rgb.extend_from_slice(&px[..3]); }
        let mut jpeg = Vec::new();
        jpeg_encoder::Encoder::new(&mut jpeg, 85)
            .encode(&rgb, buf.width as u16, buf.height as u16, jpeg_encoder::ColorType::Rgb)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        self.max_chunk = self.max_chunk.max(jpeg.len() as u32);
        self.n_frames += 1;
        self.idx_chunk(b"00dc", &jpeg, 0x10) // AVIIF_KEYFRAME
    }

    fn finish(mut self: Box<Self>, audio: Option<&[i16]>) -> std::io::Result<()> {
        // the soundtrack lands as one chunk at the end of the movi list -
        // legal RIFF and idx1 points at it fine (cues only exist once the
        // run is over, so interleaving as we go isn't an option)
        let pcm_bytes: Vec<u8> = audio.map(|pcm| {
            pcm.iter().flat_map(|s| s.to_le_bytes()).collect()
        }).unwrap_or_default();
        self.idx_chunk(b"01wb", &pcm_bytes, 0)?;
        self.list_close(self.movi_size_at)?;
        let idx = std::mem::take(&mut self.idx);
        self.write_chunk(b"idx1", &idx)?;
        self.patch_u32(self.riff_size_at, (self.pos - 8) as u32)?;

        // the five placeholders, in the order they were laid down:
        // avih, video strh+strf, audio strh+strf
        let (w, h, nf) = (self.width as u32, self.height as u32, self.n_frames);
        let n_samples = (pcm_bytes.len() / 4) as u32; // stereo frames
        let byte_rate = SAMPLE_RATE * 4;

        let mut avih = Vec::with_capacity(56);
        avih.extend_from_slice(&(1_000_000 / self.fps).to_le_bytes()); // µs/frame
        avih.extend_from_slice(&0u32.to_le_bytes());                   // max bytes/sec
        avih.extend_from_slice(&0u32.to_le_bytes());                   // padding
        avih.extend_from_slice(&0x10u32.to_le_bytes());                // AVIF_HASINDEX
        avih.extend_from_slice(&nf.to_le_bytes());
        avih.extend_from_slice(&0u32.to_le_bytes());                   // initial frames
        avih.extend_from_slice(&2u32.to_le_bytes());                   // streams
        avih.extend_from_slice(&self.max_chunk.to_le_bytes());
        avih.extend_from_slice(&w.to_le_bytes());
        avih.extend_from_slice(&h.to_le_bytes());
        avih.extend_from_slice(&[0; 16]);

        let mut vstrh = Vec::with_capacity(56);
        vstrh.extend_from_slice(b"vidsMJPG");
        vstrh.extend_from_slice(&[0; 12]);                             // flags, prio, lang, initial
        vstrh.extend_from_slice(&1u32.to_le_bytes());                  // scale
        vstrh.extend_from_slice(&self.fps.to_le_bytes());              // rate
        vstrh.extend_from_slice(&0u32.to_le_bytes());                  // start
        vstrh.extend_from_slice(&nf.to_le_bytes());                    // length
        vstrh.extend_from_slice(&self.max_chunk.to_le_bytes());
        vstrh.extend_from_slice(&u32::MAX.to_le_bytes());              // quality -1
        vstrh.extend_from_slice(&0u32.to_le_bytes());                  // sample size
        vstrh.extend_from_slice(&[0, 0, 0, 0]);                        // rcFrame top-left
        vstrh.extend_from_slice(&(w as u16).to_le_bytes());
        vstrh.extend_from_slice(&(h as u16).to_le_bytes());

        let mut vstrf = Vec::with_capacity(40); // BITMAPINFOHEADER
        vstrf.extend_from_slice(&40u32.to_le_bytes());
        vstrf.extend_from_slice(&(w as i32).to_le_bytes());
        vstrf.extend_from_slice(&(-(h as i32)).to_le_bytes());         // top-down
        vstrf.extend_from_slice(&1u16.to_le_bytes());                  // planes
        vstrf.extend_from_slice(&24u16.to_le_bytes());                 // bpp
        vstrf.extend_from_slice(b"MJPG");
        vstrf.extend_from_slice(&(w * h * 3).to_le_bytes());
        vstrf.extend_from_slice(&[0; 16]);

        let mut astrh = Vec::with_capacity(56);
        astrh.extend_from_slice(b"auds");
        astrh.extend_from_slice(&[0; 16]);                             // handler, flags, prio, lang, initial
        astrh.extend_from_slice(&4u32.to_le_bytes());                  // scale = block align
        astrh.extend_from_slice(&byte_rate.to_le_bytes());             // rate = bytes/sec
        astrh.extend_from_slice(&0u32.to_le_bytes());                  // start
        astrh.extend_from_slice(&n_samples.to_le_bytes());             // length in sample frames
        astrh.extend_from_slice(&byte_rate.to_le_bytes());
        astrh.extend_from_slice(&u32::MAX.to_le_bytes());
        astrh.extend_from_slice(&4u32.to_le_bytes());                  // sample size = block align
        astrh.extend_from_slice(&[0; 8]);                              // rcFrame

        let mut astrf = Vec::with_capacity(16); // WAVEFORMATEX, PCM
        astrf.extend_from_slice(&1u16.to_le_bytes());
        astrf.extend_from_slice(&2u16.to_le_bytes());                  // stereo
        astrf.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        astrf.extend_from_slice(&byte_rate.to_le_bytes());
        astrf.extend_from_slice(&4u16.to_le_bytes());                  // block align
        astrf.extend_from_slice(&16u16.to_le_bytes());                 // bits

        let patches: [&[u8]; 5] = [&avih, &vstrh, &vstrf, &astrh, &astrf];
        let ats = self.patch_ats.clone();
        for (at, data) in ats.iter().zip(patches) { self.patch_data(*at, data)?; }
        self.w.flush()
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
    match opts.cam {
        Some(cam) => (view.cam_x, view.cam_y) = cam,
        // no explicit camera: start over this seed's village instead of
        // View::new's fixed corner, in case it moved
        None => if let Some((mx, my)) = game.ui_model().market { (view.cam_x, view.cam_y) = camera_on(mx, my); },
    }

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

    let mut events: Vec<AudioEvent> = Vec::new();
    let mut captured = 1u64;
    for f in 1..total_frames {
        if let Some(ops) = script.remove(&f) {
            for op in &ops { apply_op(op, &mut game, &mut view, &mut held_walk); }
        }
        if let Some((dx, dy)) = held_walk { game.command(Command::Walk(dx, dy)); }
        game.update();
        // sound cues live exactly one frame; catch them on the way past
        for c in &game.sfx {
            events.push(AudioEvent { frame: f, kind: c.kind,
                pan: c.x / MAP_TILES as f32, gain: c.intensity });
        }
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
    // sound-capable formats get the cues synthesised on the watch timeline
    let wants_audio = opts.format == Format::Avi || (opts.format == Format::Ffmpeg && opts.audio);
    let pcm = wants_audio.then(|| {
        let secs = n_frames as f64 / opts.fps.max(1) as f64;
        synth_track(&events, every, opts.fps, secs)
    });
    sink.finish(pcm.as_deref())?;
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
        let s = "# a comment\n\n120 walk 1 0\n340 stop\n360 act 1\n400 select 3\n420 ctx 0\n440 click 700 400\n500 pause\n700 look 40 60\n750 cancel\n";
        let ops = parse_script(s).unwrap();
        assert_eq!(ops.len(), 9);
        assert!(matches!(ops[0], (120, ScriptOp::Cmd(Command::Walk(dx, dy))) if dx == 1.0 && dy == 0.0));
        assert!(matches!(ops[1], (340, ScriptOp::Cmd(Command::Walk(0.0, 0.0)))));
        assert!(matches!(ops[2], (360, ScriptOp::Cmd(Command::Act(1)))));
        assert!(matches!(ops[3], (400, ScriptOp::Cmd(Command::Select(3)))));
        assert!(matches!(ops[4], (420, ScriptOp::Cmd(Command::CtxAct(0)))));
        assert!(matches!(ops[5], (440, ScriptOp::Cmd(Command::ClickMap { mx: 700, my: 400, .. }))));
        assert!(matches!(ops[6], (500, ScriptOp::Cmd(Command::TogglePause))));
        assert!(matches!(ops[7], (700, ScriptOp::Look(40, 60))));
        assert!(matches!(ops[8], (750, ScriptOp::Cmd(Command::CancelApproach))));
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
    fn test_speed_presets() {
        assert_eq!(speed_preset("realtime"), Some(1.0));
        assert_eq!(speed_preset("1x"), Some(1.0));
        assert_eq!(speed_preset("2x"), Some(2.0));
        assert_eq!(speed_preset("60x"), Some(60.0));
        assert_eq!(speed_preset("ludicrous"), None);
    }

    #[test]
    fn test_wav_header_shape() {
        let path = std::env::temp_dir().join(format!("game2-wav-{}.wav", std::process::id()));
        let p = path.to_string_lossy().to_string();
        write_wav(&p, &[0i16, 1000, -1000, 0], 2).unwrap();
        let b = std::fs::read(&p).unwrap();
        assert_eq!(&b[..4], b"RIFF");
        assert_eq!(&b[8..12], b"WAVE");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_synth_puts_sound_where_the_cue_was() {
        // a cue at game frame 600 in a 60x@12fps recording lands at 1/6s in
        let events = vec![AudioEvent { frame: 600, kind: "anvil", pan: 0.5, gain: 0.8 }];
        let pcm = synth_track(&events, 300, 12, 1.0);
        assert_eq!(pcm.len(), (SAMPLE_RATE as usize) * 2, "one watch-second, stereo");
        let hit = 600.0 / 300.0 / 12.0 * SAMPLE_RATE as f64; // = 0.166s in
        let before = pcm[..(hit as usize - 500) * 2].iter().all(|&s| s == 0);
        let around = pcm[(hit as usize) * 2..].iter().take(2000).any(|&s| s != 0);
        assert!(before && around, "silence before, clang at {}", hit);
    }

    #[test]
    fn test_avi_riff_layout() {
        let dir = std::env::temp_dir().join(format!("game2-avi-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("t.avi").to_string_lossy().to_string();
        run_recording(Art::load_fs("../assets", 7),
            RecordOpts { play_secs: 1, out: out.clone(), format: Format::Avi, ..Default::default() }, 7).unwrap();
        let b = std::fs::read(&out).unwrap();
        assert_eq!(&b[..4], b"RIFF");
        assert_eq!(&b[8..12], b"AVI ");
        // RIFF size field covers the whole file after itself
        let riff = u32::from_le_bytes([b[4], b[5], b[6], b[7]]) as usize;
        assert_eq!(riff + 8, b.len());
        for fourcc in [b"avih", b"vids", b"auds", b"movi", b"00dc", b"01wb", b"idx1"] {
            assert!(b.windows(4).any(|w| w == fourcc), "missing {}", String::from_utf8_lossy(fourcc));
        }
        let _ = std::fs::remove_dir_all(&dir);
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
