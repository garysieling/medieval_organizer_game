//! `sound_dump`: the audio twin of `--dump-frame` (#1493).
//!
//! ```text
//! sound_dump [--seed N] [--stage NAME] [--scenario NAME] [--frames N]
//!            --out <prefix> | --scenes <file.jsonl> | --connect <ws://host:port>
//! ```
//! Writes `<out>.wav`, `.trace.json`, `.metrics.json`, `.spectrogram.png`,
//! `.roll.png`, `.timeline.png`.
use game2_client::sound_dump::{self, DumpOpts};
use game2_core::session::stage::Stage;

fn arg<T: std::str::FromStr>(args: &[String], flag: &str) -> Option<T> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok())
}

fn stage_of(name: &str) -> Option<Stage> {
    Some(match name.to_ascii_lowercase().as_str() {
        "ordinary" => Stage::Ordinary,
        "fever" => Stage::Fever,
        "famine" => Stage::Famine,
        "hardwinter" => Stage::HardWinter,
        "strongorg" => Stage::StrongOrg,
        "unrest" => Stage::Unrest,
        "flush" => Stage::Flush,
        "bereft" => Stage::Bereft,
        _ => return None,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("usage: sound_dump [--seed N] [--stage NAME] [--scenario NAME] [--frames N] --out P | --scenes F | --connect WS");
        std::process::exit(0);
    }
    let mut opts = DumpOpts::default();
    if let Some(v) = arg::<u64>(&args, "--seed") { opts.seed = v; }
    if let Some(v) = arg::<u64>(&args, "--frames") { opts.frames = v; }
    if let Some(v) = arg::<String>(&args, "--out") { opts.out = v; }
    opts.stage = arg::<String>(&args, "--stage")
        .and_then(|s| stage_of(&s).or_else(|| {
            eprintln!("unknown stage {s}: Ordinary|Fever|Famine|HardWinter|StrongOrg|Unrest|Flush|Bereft");
            std::process::exit(2);
        }));
    opts.scenario = arg::<String>(&args, "--scenario");
    opts.scenes = arg::<String>(&args, "--scenes");
    opts.connect = arg::<String>(&args, "--connect");
    match sound_dump::run(&opts) {
        Ok(()) => game2_client::sayln!("{}: wav + trace + metrics + 3 PNGs written", opts.out),
        Err(e) => { eprintln!("sound_dump: {e}"); std::process::exit(1); }
    }
}
