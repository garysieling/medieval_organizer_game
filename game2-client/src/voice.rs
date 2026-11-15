//! Push-to-talk dev feedback (#1083), compiled only under the
//! `dev-voice` feature: hold F9 while playing, say what's wrong or
//! what's wanted, let go - the clip's transcribed on-device by the
//! fetched whisper-cli and filed as a `dev-report` GitHub issue with
//! the game's state attached, so a report replays without a second
//! interview.
//!
//! The tool is `scripts/get-whisper.ps1`'s fetch: `.tools/whisper/`
//! for the binary, `assets/models/ggml-tiny.en.bin` for the model, the
//! user's own `gh` for the filing - nothing secret rides in the build.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Whisper wants 16kHz mono f32; the clip's capped at this many seconds
/// so a held key can't grow memory without bound
const SAMPLE_RATE: u32 = 16_000;
const CLIP_MAX_SECS: usize = 30;
/// The bundle folder shared with the traveller's-ask bundles (#1085)
const FEEDBACK_DIR: &str = "feedback";
/// The transcript lands under this label so voice-filed tickets are
/// findable at a glance
const REPORT_LABEL: &str = "dev-report";
const MODEL_PATH: &str = "assets/models/ggml-tiny.en.bin";
/// Where the fetched binary can sit: the zip's Release/ dir first
/// (v1.8.2's top-level whisper-cli.exe is a 28KB deprecation stub,
/// the real one needs its DLLs beside it), then the flat spots
const TOOL_PATHS: [&str; 4] = [
    ".tools/whisper/Release/whisper-cli.exe",
    ".tools/whisper/Release/main.exe",
    ".tools/whisper/whisper-cli.exe",
    ".tools/whisper-cli.exe",
];

/// The world state the report carries, snapped at release so the worker
/// thread needs nothing off the live game
pub struct VoiceCtx {
    pub seed: u64,
    pub tick: u64,
    pub frame: u32,
    pub camera: Option<(f32, f32)>,
    /// The log's recent lines, oldest first
    pub log_tail: Vec<String>,
}

/// A live mic capture: the stream keeps itself alive; dropping it ends
/// the take
struct Take {
    stream: cpal::Stream,
    buf: Arc<Mutex<Vec<f32>>>,
    rate: u32,
    channels: usize,
}

/// The push-to-talk box: idle until F9 goes down, one take at a time
pub struct Voice {
    take: Option<Take>,
    /// The last complaint printed, so a missing tool nags once not every frame
    warned: Option<String>,
}

impl Voice {
    pub fn new() -> Self {
        Voice { take: None, warned: None }
    }

    /// F9 down: open the mic. Any failure prints once per kind - the
    /// game's never stopped for a dev tool
    pub fn press(&mut self) {
        if self.take.is_some() { return; }
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        let host = cpal::default_host();
        let out = (|| -> Result<Take, String> {
            let dev = host.default_input_device()
                .ok_or("no input device - is a mic plugged in?")?;
            let cfg = dev.default_input_config()
                .map_err(|e| format!("no input config: {}", e))?;
            let buf = Arc::new(Mutex::new(Vec::<f32>::new()));
            let push = |buf: &Arc<Mutex<Vec<f32>>>| {
                let b = buf.clone();
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    b.lock().unwrap().extend_from_slice(data)
                }
            };
            let err = |e| eprintln!("[voice] stream error: {}", e);
            let config = cfg.config();
            let stream = match cfg.sample_format() {
                cpal::SampleFormat::F32 => dev.build_input_stream(&config, push(&buf), err, None),
                cpal::SampleFormat::I16 => {
                    let b = buf.clone();
                    dev.build_input_stream(&config,
                        move |d: &[i16], _| {
                            b.lock().unwrap().extend(d.iter().map(|s| *s as f32 / 32768.0))
                        }, err, None)
                }
                cpal::SampleFormat::U16 => {
                    let b = buf.clone();
                    dev.build_input_stream(&config,
                        move |d: &[u16], _| {
                            b.lock().unwrap().extend(d.iter().map(|s| *s as f32 / 32768.0 - 1.0))
                        }, err, None)
                }
                _ => return Err(format!("unsupported sample format {:?}", cfg.sample_format())),
            }.map_err(|e| format!("couldn't open the mic: {}", e))?;
            stream.play().map_err(|e| format!("couldn't start the mic: {}", e))?;
            Ok(Take { stream, buf,
                rate: cfg.sample_rate().0,
                channels: cfg.channels() as usize })
        })();
        match out {
            Ok(t) => {
                self.take = Some(t);
                eprintln!("[voice] listening - let go of F9 to file it");
            }
            Err(why) => self.warn(&why),
        }
    }

    /// F9 up: end the take and hand it to a worker - transcription and
    /// the `gh` call take seconds, the game doesn't wait on them
    pub fn release(&mut self, ctx: VoiceCtx) {
        let Some(take) = self.take.take() else { return };
        drop(take.stream);
        let raw = std::mem::take(&mut *take.buf.lock().unwrap());
        let clip = resample_16k(&raw, take.rate, take.channels);
        if clip.len() < SAMPLE_RATE as usize / 4 {
            eprintln!("[voice] under a quarter second - not enough to file");
            return;
        }
        std::thread::spawn(move || file_report(&clip, &ctx));
    }

    fn warn(&mut self, why: &str) {
        if self.warned.as_deref() != Some(why) {
            eprintln!("[voice] {}", why);
            self.warned = Some(why.to_string());
        }
    }
}

/// Interleaved channels down to mono, decimated to 16kHz - plain speech
/// needs no better than linear
fn resample_16k(raw: &[f32], rate: u32, channels: usize) -> Vec<f32> {
    let channels = channels.max(1);
    let frames = raw.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    for f in 0..frames {
        let mut sum = 0.0;
        for c in 0..channels { sum += raw[f * channels + c]; }
        mono.push(sum / channels as f32);
    }
    let rate = rate.max(1) as usize;
    let target = SAMPLE_RATE as usize;
    if rate == target { return mono; }
    let out_len = mono.len() * target / rate;
    let mut out = Vec::with_capacity(out_len.min(SAMPLE_RATE as usize * CLIP_MAX_SECS));
    for i in 0..out_len {
        let src = i as f64 * rate as f64 / target as f64;
        let lo = src as usize;
        let hi = (lo + 1).min(mono.len().saturating_sub(1));
        let t = (src - lo as f64) as f32;
        out.push(mono[lo] * (1.0 - t) + mono[hi] * t);
    }
    out
}

/// A PCM16 WAV in memory - 16kHz mono, what whisper-cli takes on `-f`
fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_len as usize);
    w.write_all(b"RIFF").unwrap();
    w.write_all(&(36 + data_len).to_le_bytes()).unwrap();
    w.write_all(b"WAVE").unwrap();
    w.write_all(b"fmt ").unwrap();
    w.write_all(&16u32.to_le_bytes()).unwrap();
    w.write_all(&1u16.to_le_bytes()).unwrap();          // PCM
    w.write_all(&1u16.to_le_bytes()).unwrap();          // mono
    w.write_all(&SAMPLE_RATE.to_le_bytes()).unwrap();
    w.write_all(&(SAMPLE_RATE * 2).to_le_bytes()).unwrap(); // byte rate
    w.write_all(&2u16.to_le_bytes()).unwrap();          // block align
    w.write_all(&16u16.to_le_bytes()).unwrap();         // bits
    w.write_all(b"data").unwrap();
    w.write_all(&data_len.to_le_bytes()).unwrap();
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        w.write_all(&v.to_le_bytes()).unwrap();
    }
    w
}

/// The fetched binary, wherever the script put it
fn whisper_exe() -> Option<PathBuf> {
    TOOL_PATHS.iter().map(PathBuf::from).find(|p| p.exists())
}

/// whisper-cli's stdout, trimmed to the spoken words - `-nt` leaves one
/// text line per segment; anything bracketed is a log line not speech
fn transcript_from(stdout: &str) -> String {
    stdout.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('['))
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The issue's first line: the report's own opening words, clipped to a
/// title's length
fn clip_title(said: &str) -> String {
    let words: Vec<&str> = said.split_whitespace().take(9).collect();
    let t = words.join(" ");
    let t = t.trim_end_matches(['.', ',', ';', ':']);
    if t.len() > 72 { format!("{}...", &t[..69]) }
    else if t.is_empty() { "voice report".into() }
    else { t.to_string() }
}

/// The body the issue files with: what was said, then everything needed
/// to stand where they stood - the same shape the traveller's-ask
/// bundles take (#1085)
fn draft_body(said: &str, ctx: &VoiceCtx) -> String {
    let mut b = String::new();
    b.push_str("## Transcript\n\n");
    b.push_str(said);
    b.push_str("\n\n## Context\n\n");
    b.push_str(&format!("- seed: {}\n", if ctx.seed == 0 { "unknown".into() } else { ctx.seed.to_string() }));
    b.push_str(&format!("- tick: {}\n", ctx.tick));
    b.push_str(&format!("- frame: {}\n", ctx.frame));
    match ctx.camera {
        Some((x, y)) => b.push_str(&format!("- camera: ({:.1}, {:.1})\n", x, y)),
        None => b.push_str("- camera: unset\n"),
    }
    if !ctx.log_tail.is_empty() {
        b.push_str("\n## Recent log\n\n");
        for line in &ctx.log_tail { b.push_str(&format!("- {}\n", line)); }
    }
    b.push_str("\n_Filed from in-game push-to-talk (hold F9). Transcribed locally._\n");
    b
}

/// The worker's whole job: WAV out, whisper in, issue filed. Every
/// missing piece says so and quits - a dev tool never crashes the game
fn file_report(clip: &[f32], ctx: &VoiceCtx) {
    let Some(exe) = whisper_exe() else {
        eprintln!("[voice] no whisper binary under .tools/ - run scripts/get-whisper.ps1");
        return;
    };
    if !PathBuf::from(MODEL_PATH).exists() {
        eprintln!("[voice] no model at {} - run scripts/get-whisper.ps1", MODEL_PATH);
        return;
    }
    let clip = &clip[..clip.len().min(SAMPLE_RATE as usize * CLIP_MAX_SECS)];
    let _ = std::fs::create_dir_all(FEEDBACK_DIR);
    let wav = PathBuf::from(FEEDBACK_DIR).join(format!("voice-f{}.wav", ctx.frame));
    if std::fs::write(&wav, wav_bytes(clip)).is_err() {
        eprintln!("[voice] couldn't write {}", wav.display());
        return;
    }
    eprintln!("[voice] transcribing...");
    let out = Command::new(&exe)
        .args(["-m", MODEL_PATH, "-f", &wav.to_string_lossy(), "-nt", "-np", "-l", "en"])
        .output();
    let Ok(out) = out else {
        eprintln!("[voice] couldn't run {}", exe.display());
        return;
    };
    let said = transcript_from(&String::from_utf8_lossy(&out.stdout));
    if said.is_empty() {
        eprintln!("[voice] nothing heard in the clip (status {:?}) - not filing", out.status.code());
        return;
    }
    let body = draft_body(&said, ctx);
    match Command::new("gh")
        .args(["issue", "create", "--title", &clip_title(&said), "--body", &body,
               "--label", REPORT_LABEL])
        .output() {
        Ok(o) if o.status.success() =>
            eprintln!("[voice] filed: {}", String::from_utf8_lossy(&o.stdout).trim()),
        Ok(o) => eprintln!("[voice] gh said no: {}", String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => eprintln!("[voice] couldn't run gh: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_bytes_writes_a_pcm16_header_and_the_samples() {
        let w = wav_bytes(&[0.0, 0.5, -1.0]);
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(&w[8..12], b"WAVE");
        assert_eq!(&w[12..16], b"fmt ");
        assert_eq!(u16::from_le_bytes([w[20], w[21]]), 1, "PCM");
        assert_eq!(u32::from_le_bytes([w[24], w[25], w[26], w[27]]), SAMPLE_RATE);
        assert_eq!(u16::from_le_bytes([w[34], w[35]]), 16, "16-bit");
        let data = &w[44..];
        assert_eq!(data.len(), 6);
        assert_eq!(i16::from_le_bytes([data[0], data[1]]), 0);
        assert_eq!(i16::from_le_bytes([data[2], data[3]]), (0.5 * 32767.0) as i16);
        assert_eq!(i16::from_le_bytes([data[4], data[5]]), -32767);
    }

    #[test]
    fn resample_downmixes_stereo_and_decimates() {
        // 48kHz stereo: left +1, right -1 -> mono silence, a third the frames
        let raw: Vec<f32> = (0..96).flat_map(|_| [1.0, -1.0]).collect();
        let out = resample_16k(&raw, 48_000, 2);
        assert_eq!(out.len(), 32);
        assert!(out.iter().all(|s| s.abs() < 1e-6));
    }

    #[test]
    fn transcript_strips_log_lines_and_collapses_whitespace() {
        let out = "  hello   world  \n[00:00.000 --> 00:01.000] timing\n second line \n";
        assert_eq!(transcript_from(out), "hello world second line");
    }

    #[test]
    fn clip_title_takes_the_opening_words() {
        assert_eq!(clip_title("the market prices are wrong today"), "the market prices are wrong today");
        assert_eq!(clip_title(""), "voice report");
        assert!(clip_title(&"word ".repeat(30)).len() <= 76);
    }

    #[test]
    fn the_body_carries_the_words_and_the_state() {
        let ctx = VoiceCtx { seed: 42, tick: 314, frame: 99,
            camera: Some((12.5, 8.0)),
            log_tail: vec!["t314: the mill turns".into()] };
        let b = draft_body("the mill grinds too slowly", &ctx);
        assert!(b.contains("the mill grinds too slowly"));
        assert!(b.contains("seed: 42") && b.contains("tick: 314"));
        assert!(b.contains("(12.5, 8.0)"));
        assert!(b.contains("t314: the mill turns"));
        let unknown = VoiceCtx { seed: 0, tick: 0, frame: 0, camera: None, log_tail: vec![] };
        assert!(draft_body("x", &unknown).contains("seed: unknown"));
    }
}
