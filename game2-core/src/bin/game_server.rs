//! The authoritative server: owns one Game, hands each connection a
//! character, ticks the world in real time, streams each seat its frame.
//!
//!   game_server [--port 8787] [--seed N] [--idle-secs 300] [--probe-port N]
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

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use game2_core::session::{self, ClientKind, ClientMsg, ServerMsg, Command, Game};
use game2_core::session::driver;

/// Frames a second the world simulates; the windowed game runs the same pace
const TICK_HZ: u64 = 20;
/// Frames between broadcasts to a client (2 = ~10 scenes a second)
const SCENE_EVERY: u64 = 2;

/// One connected client. The conn thread owns the socket: it polls reads on
/// a short timeout and drains `outbox`, so a blocking read never stalls the
/// world loop.
struct Conn {
    seat: usize,
    kind: ClientKind,
    last_active: Instant,
    /// Serialized messages queued for this client
    outbox: Vec<String>,
    /// Another connection reclaimed this seat: the read loop ends
    dead: bool,
}

struct Server {
    game: Game,
    /// claim token -> seat index
    claims: std::collections::HashMap<String, usize>,
    conns: Vec<Arc<Mutex<Conn>>>,
    /// Probe sockets: tests watching the secret state (read-only)
    probes: Vec<Arc<Mutex<Conn>>>,
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

fn send_msg(c: &mut Conn, msg: &ServerMsg) {
    if let Ok(s) = serde_json::to_string(msg) { c.outbox.push(s); }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.iter().position(|a| a == "--port").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or(8787);
    let seed: u64 = args.iter().position(|a| a == "--seed").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed);
    let idle_secs: u64 = args.iter().position(|a| a == "--idle-secs").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or(300);
    let probe_port: Option<u16> = args.iter().position(|a| a == "--probe-port").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok());

    let server = Arc::new(Mutex::new(Server {
        game: Game::new_seeded(seed),
        claims: Default::default(),
        conns: vec![],
        probes: vec![],
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

    // the probe: a second, read-only listener for tests - the secret API.
    // It's bound to localhost only (the secret state never leaves the box),
    // and --probe-token makes the client present the word before it reads
    let probe_token: Option<String> = args.iter().position(|a| a == "--probe-token")
        .and_then(|i| args.get(i + 1)).cloned()
        .or_else(|| std::env::var("GAME2_PROBE_TOKEN").ok());
    if let Some(pp) = probe_port {
        let pl = TcpListener::bind(("127.0.0.1", pp)).expect("probe bind failed");
        eprintln!("game_server: probe on 127.0.0.1:{} (test API, secret state{})", pp,
            if probe_token.is_some() { ", tokened" } else { "" });
        let server = Arc::clone(&server);
        std::thread::spawn(move || {
            for stream in pl.incoming() {
                let Ok(stream) = stream else { continue };
                stream.set_nodelay(true).ok();
                let server = Arc::clone(&server);
                let token = probe_token.clone();
                std::thread::spawn(move || handle_probe(stream, server, token));
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
            for conn in s.conns.iter() {
                let mut c = conn.lock().unwrap();
                match c.kind {
                    ClientKind::Human => {
                        let model = s.game.ui_model_at(c.seat);
                        send_msg(&mut c, &ServerMsg::Scene(Box::new(model)));
                    }
                    ClientKind::Agent => match s.game.agent_obs(c.seat) {
                        Some(obs) => send_msg(&mut c, &ServerMsg::Obs(Box::new(obs))),
                        None => send_msg(&mut c, &ServerMsg::Denied { reason: "seat gone".into() }),
                    },
                }
            }
            // probe sockets get the secret view, on their own channel
            if !s.probes.is_empty() {
                let pv = serde_json::to_string(&s.game.probe()).unwrap_or_default();
                for conn in s.probes.iter() {
                    let mut c = conn.lock().unwrap();
                    c.outbox.push(pv.clone());
                }
            }
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
    let conn = loop {
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
                let conn = Arc::new(Mutex::new(Conn { seat, kind, last_active: Instant::now(), outbox: vec![], dead: false }));
                send_msg(&mut conn.lock().unwrap(), &first);
                // a claim takes the seat back: any earlier connection
                // steering it is stale, so one character has one driver
                for c in s.conns.iter() {
                    let mut c = c.lock().unwrap();
                    if c.seat == seat { c.dead = true; }
                }
                s.conns.retain(|c| c.lock().unwrap().seat != seat);
                s.conns.push(conn.clone());
                break conn;
            }
        }
    };
    // command loop: poll reads on a short timeout and drain the outbox the
    // world loop fills, so a quiet client never stalls the broadcast
    ws.get_ref().set_read_timeout(Some(Duration::from_millis(50))).ok();
    loop {
        if conn.lock().unwrap().dead { break; }
        let out: Vec<String> = { let mut c = conn.lock().unwrap(); c.outbox.drain(..).collect() };
        let mut alive = true;
        for s_out in out {
            if ws.send(tungstenite::Message::Text(s_out.into())).is_err() { alive = false; break; }
        }
        if !alive { break; }
        match ws.read() {
            Ok(tungstenite::Message::Text(t)) => {
                let Ok(cm) = serde_json::from_str::<ClientMsg>(&t) else { continue };
                let mut s = server.lock().unwrap();
                // reclaimed while the message waited: the seat has a new
                // driver - don't run this conn's stale command on it
                if conn.lock().unwrap().dead { break; }
                let (seat, kind) = { let c = conn.lock().unwrap(); (c.seat, c.kind) };
                conn.lock().unwrap().last_active = Instant::now();
                match cm {
                    ClientMsg::Command(cmd) => {
                        // speed and pause are local-only: the shared world runs at one pace
                        if !matches!(cmd, Command::TogglePause | Command::ToggleSpeed) {
                            s.game.command_at(seat, cmd);
                        }
                    }
                    ClientMsg::AgentCmd(ac) => {
                        let note = match s.game.agent_command(seat, ac) {
                            Ok(()) => "ok".to_string(), Err(e) => e,
                        };
                        send_msg(&mut conn.lock().unwrap(), &ServerMsg::CmdDone { ok: note == "ok", note });
                    }
                    ClientMsg::Ping(n) => { send_msg(&mut conn.lock().unwrap(), &ServerMsg::Pong(n)); }
                    ClientMsg::Hello { .. } => {}
                }
                let _ = kind;
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
    // dropped: the seat keeps its character; claims still map it for reclaim
    let mut s = server.lock().unwrap();
    s.conns.retain(|c| !Arc::ptr_eq(c, &conn));
}

/// A probe connection: the test API. It accepts the websocket handshake,
/// demands the token when one's set, then only ever writes - ProbeView
/// frames at the broadcast cadence. Anything else the client sends is
/// ignored: the secret view gives nothing back to play with.
fn handle_probe(stream: TcpStream, server: Arc<Mutex<Server>>, token: Option<String>) {
    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws, Err(_) => return,
    };
    ws.get_ref().set_read_timeout(Some(Duration::from_secs(5))).ok();
    // the word, when one's set: first frame must be it or the conn goes nowhere
    if let Some(t) = &token {
        let ok = matches!(ws.read(), Ok(tungstenite::Message::Text(m)) if *m == *t);
        if !ok { return; }
    }
    ws.get_ref().set_read_timeout(Some(Duration::from_millis(50))).ok();
    let conn = Arc::new(Mutex::new(Conn { seat: 0, kind: ClientKind::Agent,
        last_active: Instant::now(), outbox: vec![], dead: false }));
    {
        let mut s = server.lock().unwrap();
        s.probes.push(conn.clone());
    }
    loop {
        if conn.lock().unwrap().dead { break; }
        let out: Vec<String> = { let mut c = conn.lock().unwrap(); c.outbox.drain(..).collect() };
        let mut alive = true;
        for s_out in out {
            if ws.send(tungstenite::Message::Text(s_out.into())).is_err() { alive = false; break; }
        }
        if !alive { break; }
        match ws.read() {
            // reads are ignored on purpose - but a close or an error ends it
            Err(tungstenite::Error::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
            Ok(tungstenite::Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }
    let mut s = server.lock().unwrap();
    s.probes.retain(|c| !Arc::ptr_eq(c, &conn));
}
