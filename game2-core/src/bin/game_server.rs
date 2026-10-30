//! The authoritative server: owns one Game, hands each connection a
//! character, ticks the world in real time, streams each seat its frame.
//!
//!   game_server [--port 8787] [--seed N] [--idle-secs 300]
//!
//! Clients are two kinds:
//!   - Human: gets a wanderer seat and a UiModel every broadcast; sends
//!     Commands. Idle past --idle-secs, the fallback driver steers them.
//!   - Agent: gets a villager seat and AgentObs; sends AgentCmd. Go quiet
//!     and the villager's own goals take back over (their travels run out).

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use game2_core::session::{self, ClientKind, ClientMsg, ServerMsg, Command, Game};
use game2_core::session::driver;

/// Frames a second the world simulates; the windowed game runs the same pace
const TICK_HZ: u64 = 20;
/// Frames between broadcasts to a client (2 = ~10 scenes a second)
const SCENE_EVERY: u64 = 2;

/// One connected client
struct Conn {
    ws: tungstenite::WebSocket<TcpStream>,
    seat: usize,
    kind: ClientKind,
    last_active: Instant,
}

struct Server {
    game: Game,
    /// claim token -> seat index
    claims: std::collections::HashMap<String, usize>,
    conns: Vec<Arc<Mutex<Conn>>>,
    frame: u64,
    idle: Duration,
}

fn token() -> String {
    // unguessable-enough session token: time + counter, hashed
    use std::hash::{Hash, Hasher};
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (Instant::now(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)).hash(&mut h);
    format!("{:x}", h.finish())
}

fn send_msg(c: &mut Conn, msg: &ServerMsg) -> bool {
    match serde_json::to_string(msg) {
        Ok(s) => c.ws.send(tungstenite::Message::Text(s.into())).is_ok(),
        Err(_) => false,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.iter().position(|a| a == "--port").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or(8787);
    let seed: u64 = args.iter().position(|a| a == "--seed").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed);
    let idle_secs: u64 = args.iter().position(|a| a == "--idle-secs").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or(300);

    let server = Arc::new(Mutex::new(Server {
        game: Game::new_seeded(seed),
        claims: Default::default(),
        conns: vec![],
        frame: 0,
        idle: Duration::from_secs(idle_secs),
    }));

    let listener = TcpListener::bind(("0.0.0.0", port)).expect("bind failed");
    eprintln!("game_server: listening on 0.0.0.0:{} (seed {})", port, seed);

    // accept loop
    {
        let server = Arc::clone(&server);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                stream.set_nodelay(true).ok();
                let server = Arc::clone(&server);
                std::thread::spawn(move || handle_conn(stream, server));
            }
        });
    }

    // sim + broadcast loop
    let frame_len = Duration::from_millis(1000 / TICK_HZ);
    loop {
        let at = Instant::now();
        let mut s = server.lock().unwrap();
        s.game.update();
        s.frame += 1;
        // idle wanderers get steered; agent seats need nothing - their
        // villager's own goals resume when their travels run out
        if s.frame % driver::DRIVE_EVERY == 0 {
            let idle_for: Vec<usize> = s.conns.iter()
                .filter_map(|c| {
                    let c = c.lock().unwrap();
                    (c.kind == ClientKind::Human && c.last_active.elapsed() > s.idle).then_some(c.seat)
                }).collect();
            for seat in idle_for { driver::drive_player(&mut s.game, seat); }
        }
        if s.frame % SCENE_EVERY == 0 {
            let mut dead = vec![];
            for (i, conn) in s.conns.iter().enumerate() {
                let mut c = conn.lock().unwrap();
                let ok = match c.kind {
                    ClientKind::Human => {
                        let model = s.game.ui_model_at(c.seat);
                        send_msg(&mut c, &ServerMsg::Scene(Box::new(model)))
                    }
                    ClientKind::Agent => match s.game.agent_obs(c.seat) {
                        Some(obs) => send_msg(&mut c, &ServerMsg::Obs(Box::new(obs))),
                        None => send_msg(&mut c, &ServerMsg::Denied { reason: "seat gone".into() }),
                    },
                };
                if !ok { dead.push(i); }
            }
            for i in dead.into_iter().rev() { s.conns.remove(i); }
        }
        drop(s);
        let spent = at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }
}

fn handle_conn(stream: TcpStream, server: Arc<Mutex<Server>>) {
    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws, Err(_) => return,
    };
    // the first message must be Hello: claim or be assigned a seat
    let seat = loop {
        let msg = match ws.read() { Ok(m) => m, Err(_) => return };
        if let tungstenite::Message::Text(t) = msg {
            let Ok(cm) = serde_json::from_str::<ClientMsg>(&t) else { continue };
            if let ClientMsg::Hello { token: tok, kind, name: _ } = cm {
                let mut s = server.lock().unwrap();
                let seat = tok.as_deref().and_then(|t| s.claims.get(t).copied())
                    .unwrap_or_else(|| match kind {
                        ClientKind::Human => s.game.add_seat(),
                        ClientKind::Agent => match s.game.free_villager() {
                            Some(i) => { let v = s.game.villagers[i]; s.game.add_agent_seat(v) },
                            None => s.game.add_seat(), // village full: park them on a wanderer
                        },
                    });
                let tok = tok.unwrap_or_else(token);
                s.claims.insert(tok.clone(), seat);
                let you = s.game.seats[seat].player;
                let first = match kind {
                    ClientKind::Human => ServerMsg::Welcome {
                        you, token: tok, model: Box::new(s.game.ui_model_at(seat)) },
                    ClientKind::Agent => ServerMsg::Welcome {
                        you, token: tok,
                        model: Box::new(s.game.ui_model_at(0)) }, // agents see the world via Obs
                };
                let conn = Arc::new(Mutex::new(Conn { ws, seat, kind, last_active: Instant::now() }));
                if !send_msg(&mut conn.lock().unwrap(), &first) { return; }
                s.conns.push(conn.clone());
                break conn;
            }
        }
    };
    // command loop: read messages until disconnect
    loop {
        let msg = match seat.lock().unwrap().ws.read() {
            Ok(m) => m, Err(_) => break,
        };
        let tungstenite::Message::Text(t) = msg else { continue };
        let Ok(cm) = serde_json::from_str::<ClientMsg>(&t) else { continue };
        let mut s = server.lock().unwrap();
        let mut c = seat.lock().unwrap();
        c.last_active = Instant::now();
        match cm {
            ClientMsg::Command(cmd) => {
                // speed and pause are local-only: the shared world runs at one pace
                if !matches!(cmd, Command::TogglePause | Command::ToggleSpeed) {
                    s.game.command_at(c.seat, cmd);
                }
            }
            ClientMsg::AgentCmd(ac) => {
                let note = match s.game.agent_command(c.seat, ac) {
                    Ok(()) => "ok".to_string(), Err(e) => e,
                };
                send_msg(&mut c, &ServerMsg::CmdDone { ok: note == "ok", note });
            }
            ClientMsg::Ping(n) => { send_msg(&mut c, &ServerMsg::Pong(n)); }
            ClientMsg::Hello { .. } => {}
        }
    }
    // dropped: the seat keeps its character; claims still map it for reclaim
    let mut s = server.lock().unwrap();
    s.conns.retain(|c| !Arc::ptr_eq(c, &seat));
}
