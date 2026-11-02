//! The agent bundle: N bot connections steering villagers over the agent
//! API. Each bot watches its AgentObs and sends AgentCmd when it's free -
//! the sim does the walking, so the wire stays quiet.
//!
//!   game_bots ws://localhost:8787 [count]
//!
//! Bots claim villager seats server-side; drop one and its villager's own
//! goals take back over after the idle window.

use game2_core::session::{self, AgentObs, ClientKind, ClientMsg, ServerMsg};
use game2_core::session::driver::drive_agent;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(1).cloned().unwrap_or_else(|| "ws://localhost:8787".into());
    let count: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(5);

    for i in 0..count {
        let url = url.clone();
        std::thread::spawn(move || run_bot(&url, i));
    }
    loop { std::thread::sleep(std::time::Duration::from_secs(60)); }
}

/// A claim token kept on disk like the windowed client's `game2.claim`:
/// restart the bots and each reclaims its own villager instead of
/// doubling the cast
fn claim_file(n: usize) -> String { format!("game2-bot-{}.claim", n) }

fn run_bot(url: &str, n: usize) {
    let saved = std::fs::read_to_string(claim_file(n)).ok()
        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let (mut ws, _) = match tungstenite::connect(url) {
        Ok(x) => x, Err(e) => { eprintln!("bot {}: connect failed: {}", n, e); return; }
    };
    ws.send(tungstenite::Message::Text(
        serde_json::to_string(&ClientMsg::Hello { token: saved, kind: ClientKind::Agent, name: Some(format!("bot-{}", n)) }).unwrap().into()
    )).ok();
    eprintln!("bot {}: connected", n);
    let mut last_tick = 0u64;
    loop {
        let msg = match ws.read() { Ok(m) => m, Err(_) => { eprintln!("bot {}: dropped", n); return; } };
        let tungstenite::Message::Text(t) = msg else { continue };
        let Ok(sm) = serde_json::from_str::<ServerMsg>(&t) else { continue };
        match sm {
            ServerMsg::Welcome { you, token, .. } => {
                eprintln!("bot {}: claimed entity {}", n, you);
                let _ = std::fs::write(claim_file(n), token);
            }
            ServerMsg::Obs(obs) => {
                // act only on a fresh observation, and only when free - a bot
                // that talks every frame would drown the wire
                if obs.tick == last_tick { continue; }
                last_tick = obs.tick;
                if obs.walking { continue; }
                let cmd = drive_agent(&obs);
                if matches!(cmd, session::AgentCmd::Idle) { continue; }
                ws.send(tungstenite::Message::Text(
                    serde_json::to_string(&ClientMsg::AgentCmd(cmd)).unwrap().into())).ok();
            }
            ServerMsg::CmdDone { ok, note } => {
                if !ok { eprintln!("bot {}: cmd refused: {}", n, note); }
            }
            _ => {}
        }
        let _: AgentObs;
    }
}
