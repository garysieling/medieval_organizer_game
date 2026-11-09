//! Video recorder: plays the village headless - faster than any human
//! could watch - and writes what the window would have shown to a video
//! file. For test footage and long-run review; one day the ffmpeg sink
//! could point at an rtmp endpoint and stream.
//!
//!   cargo run --release --bin record_video -- --play-time 60 --speedup 3 --out evidence/videos/minute.gif
//!
//! Flags: --play-time SECS   game-seconds to simulate (default 10)
//!        --speed PRESET     realtime | 1x | 2x | 5x | 10x | 60x (default realtime)
//!        --speedup X        or any number: watch time = play-time / X
//!        --fps N            output frames per second (default 12)
//!        --scale F          downscale frames, e.g. 0.5 for half size (default 1)
//!        --format FMT       gif | apng | avi | mp4 | webm | frames | ffmpeg (default gif)
//!        --audio            attach the synthesised sfx track (avi always does)
//!        --ffmpeg-path P    ffmpeg binary for mp4/webm (scripts/get-ffmpeg.ps1)
//!        --script FILE      timed commands - see client::record docs
//!        --game-speed N     in-game clock, 1 or 4 (default 1)
//!        --zoom F --cam X,Y --seed N --out PATH
//!
//! Examples:
//!   --play-time 10                       a ten-second clip, as played
//!   --play-time 1800 --speedup 60        half an hour in 30s of footage
//!   --script jobs/walkabout.txt          drive the player from a file

use game2_client::{self as client, Art, Format, RecordOpts};
use game2_core::session;

fn arg<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn main() {
    game2_core::infra::crash::install();
    let args: Vec<String> = std::env::args().collect();
    let mut opts = RecordOpts::default();
    opts.play_secs = arg(&args, "--play-time").and_then(|v| v.parse().ok()).unwrap_or(opts.play_secs);
    if let Some(s) = arg(&args, "--speed") {
        match client::speed_preset(s) {
            Some(v) => opts.speedup = v,
            None => { eprintln!("unknown --speed {} (realtime|1x|2x|5x|10x|60x)", s); std::process::exit(2); }
        }
    }
    opts.speedup = arg(&args, "--speedup").and_then(|v| v.parse().ok()).unwrap_or(opts.speedup);
    opts.fps = arg(&args, "--fps").and_then(|v| v.parse().ok()).unwrap_or(opts.fps);
    opts.scale = arg(&args, "--scale").and_then(|v| v.parse().ok()).unwrap_or(opts.scale);
    opts.zoom = arg(&args, "--zoom").and_then(|v| v.parse().ok()).unwrap_or(opts.zoom);
    opts.cam = arg(&args, "--cam")
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
    opts.game_speed = arg(&args, "--game-speed").and_then(|v| v.parse().ok()).unwrap_or(opts.game_speed);
    opts.format = match arg(&args, "--format").unwrap_or("gif") {
        "gif" => Format::Gif,
        "apng" => Format::Apng,
        "frames" => Format::Frames,
        "avi" => Format::Avi,
        "ffmpeg" | "mp4" | "webm" | "mov" => Format::Ffmpeg,
        other => { eprintln!("unknown --format {} (gif|apng|avi|frames|mp4|webm)", other); std::process::exit(2); }
    };
    opts.audio = args.iter().any(|a| a == "--audio");
    let die = |why: &str| -> ! { eprintln!("{}", why); std::process::exit(2) };
    if opts.fps == 0 || opts.fps > 240 { die("--fps must be 1-240"); }
    if opts.speedup <= 0.0 { die("--speedup must be positive"); }
    if !(0.05..=4.0).contains(&opts.scale) { die("--scale must be 0.05-4.0"); }
    if !matches!(opts.game_speed, 1 | 4) { die("--game-speed must be 1 or 4"); }
    if opts.audio && matches!(opts.format, Format::Gif | Format::Apng | Format::Frames) {
        eprintln!("warning: --audio does nothing for {}", arg(&args, "--format").unwrap_or("gif"));
    }
    if let Some(p) = arg(&args, "--ffmpeg-path") { opts.ffmpeg = p.to_string(); }
    if let Some(s) = arg(&args, "--script") {
        match std::fs::read_to_string(s).map_err(|e| e.to_string())
            .and_then(|t| client::parse_script(&t)) {
            Ok(ops) => opts.script = ops,
            Err(e) => { eprintln!("can't read script {}: {}", s, e); std::process::exit(2); }
        }
    }
    let seed: u64 = arg(&args, "--seed").and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed);
    opts.out = arg(&args, "--out").map(|s| s.to_string()).unwrap_or_else(|| {
        // the ffmpeg formats keep the extension the user asked for
        let ext = match arg(&args, "--format").unwrap_or("gif") {
            "gif" => "gif", "apng" => "png", "frames" => "", "avi" => "avi",
            "webm" => "webm", "mov" => "mov", _ => "mp4",
        };
        if ext.is_empty() { format!("evidence/videos/{}s", opts.play_secs) }
        else { format!("evidence/videos/play-{}s.{}", opts.play_secs, ext) }
    });
    if opts.format == Format::Ffmpeg && opts.out.starts_with('-') {
        die("ffmpeg reads a leading '-' in --out as one of its own flags");
    }
    if let Some(dir) = std::path::Path::new(&opts.out).parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).ok();
    }

    let art = Art::load_fs("assets", seed);
    if let Err(e) = client::run_recording(art, opts, seed) {
        eprintln!("recording failed: {}", e);
        std::process::exit(1);
    }
}
