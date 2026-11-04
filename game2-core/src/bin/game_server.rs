//! The authoritative server: owns one Game, hands each connection a
//! character, ticks the world in real time, streams each seat its frame.
//!
//!   game_server [--port 8787] [--seed N] [--idle-secs 300] [--probe-port N]
//!               [--probe-token W] [--join-secret W] [--max-seats 64]
//!               [--save FILE] [--save-every 30]
//!
//! Clients are two kinds:
//!   - Human: gets a wanderer seat and a UiModel every broadcast; sends
//!     Commands. Idle past --idle-secs, the fallback driver steers them.
//!   - Agent: gets a villager seat and AgentObs; sends AgentCmd. Go quiet
//!     and the villager's own goals take back over (their travels run out).
//!
//! The probe is a THIRD, separate API: `--probe-port` opens a read-only
//! listener that streams the secret state (session/probe.rs) to tests.
//! It is never on the player/agent port, it binds to localhost only, and
//! `--probe-token` (or GAME2_PROBE_TOKEN) makes the client present the
//! word first. Player-facing messages carry none of it - agents see
//! exactly what players see.

use std::time::Duration;

use game2_core::server::{serve, ServerConfig};
use game2_core::session;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let cfg = ServerConfig {
        bind: "0.0.0.0".into(),
        port: arg("--port").and_then(|v| v.parse().ok()).unwrap_or(8787),
        seed: arg("--seed").and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed),
        idle: Duration::from_secs(arg("--idle-secs").and_then(|v| v.parse().ok()).unwrap_or(300)),
        tick_hz: 20,
        probe_port: arg("--probe-port").and_then(|v| v.parse().ok()),
        probe_token: arg("--probe-token").or_else(|| std::env::var("GAME2_PROBE_TOKEN").ok()),
        join_secret: arg("--join-secret").or_else(|| std::env::var("GAME2_JOIN_SECRET").ok()).filter(|s| !s.is_empty()),
        max_seats: arg("--max-seats").and_then(|v| v.parse().ok()).unwrap_or(64),
        save: arg("--save").map(std::path::PathBuf::from),
        save_every: Duration::from_secs(arg("--save-every").and_then(|v| v.parse().ok()).unwrap_or(30)),
    };
    let tokened = cfg.probe_token.is_some();
    let h = serve(cfg).expect("can't start the server");
    if let Some(r) = &h.restored {
        eprintln!("game_server: restored {} frames, {} inputs from the save ({})", r.frames, r.inputs,
            if r.digest_matched { "state matches" } else { "STATE DIFFERS" });
    }
    eprintln!("game_server: listening on {} (seed {})", h.addr, h.seed);
    if let Some(p) = h.probe_addr {
        eprintln!("game_server: probe on {} (test API, secret state{})", p, if tokened { ", tokened" } else { "" });
    }
    // run until asked to stop (SIGTERM from docker stop / kubernetes, or
    // Ctrl-C), then stop properly: the final save goes to disk
    term::install();
    while !term::asked() { std::thread::sleep(Duration::from_millis(200)); }
    eprintln!("game_server: stopping (saving the village first, if --save)");
    h.stop();
    eprintln!("game_server: stopped");
}

/// Shutdown on a signal. In a container the server is PID 1, and Linux
/// ignores SIGTERM for a PID 1 with no handler, so without this `docker
/// stop` waits out its timeout and kills the process before the save.
/// libc's `signal` directly, so no crate for one flag.
#[cfg(unix)]
mod term {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ASKED: AtomicBool = AtomicBool::new(false);
    extern "C" fn on_signal(_: i32) { ASKED.store(true, Ordering::SeqCst); }
    extern "C" { fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize; }
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    pub fn install() {
        // SAFETY: the handler only stores to an atomic, which is signal-safe
        unsafe { signal(SIGTERM, on_signal); signal(SIGINT, on_signal); }
    }
    pub fn asked() -> bool { ASKED.load(Ordering::SeqCst) }
}

/// Elsewhere (Windows) Ctrl-C ends the process as before; periodic saves
/// cover it
#[cfg(not(unix))]
mod term {
    pub fn install() {}
    pub fn asked() -> bool { false }
}
