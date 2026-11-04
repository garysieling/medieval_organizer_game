//! The authoritative server: owns one Game, hands each connection a
//! character, ticks the world in real time, streams each seat its frame.
//! `game_server` is a thin command line over `serve`; tests start one on an
//! ephemeral port and play it over real sockets (tests/multiplayer.rs).
//!
//! Clients are two kinds:
//!   - Human: gets a wanderer seat and a UiModel every broadcast; sends
//!     Commands. Idle past `idle`, the fallback driver steers them.
//!   - Agent: gets a villager seat and AgentObs; sends AgentCmd. Go quiet
//!     and the villager's own goals take back over (their travels run out).
//!
//! The probe is a THIRD, separate API: `probe_port` opens a read-only
//! listener that streams the secret state (session/probe.rs) to tests.
//! It is never on the player/agent port, it binds to localhost only, and
//! `probe_token` makes the client present the word first. Player-facing
//! messages carry none of it - agents see exactly what players see.
//!
//! This sits beside `session`, not in it, and uses only its public API.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::session::{driver, ClientKind, ClientMsg, Command, Game, ServerMsg};

/// Frames between broadcasts to a client (2 = ~10 scenes a second at 20Hz)
const SCENE_EVERY: u64 = 2;

/// How a server runs
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address the game port binds to ("0.0.0.0" to serve the network)
    pub bind: String,
    /// The game port (0 picks a free one; `ServerHandle::addr` says which)
    pub port: u16,
    /// The map's seed
    pub seed: u64,
    /// A human seat quiet this long gets the fallback driver
    pub idle: Duration,
    /// Frames a second the world simulates; the windowed game runs 20
    pub tick_hz: u64,
    /// The probe's port (localhost only), if any (0 picks a free one)
    pub probe_port: Option<u16>,
    /// The word a probe client must send first, if any
    pub probe_token: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { bind: "0.0.0.0".into(), port: 8787, seed: crate::session::MAP_SEED,
            idle: Duration::from_secs(300), tick_hz: 20, probe_port: None, probe_token: None }
    }
}

/// A running server: where it listens, and the switch that stops it
pub struct ServerHandle {
    pub addr: SocketAddr,
    pub probe_addr: Option<SocketAddr>,
    stop: Arc<AtomicBool>,
    world: Option<std::thread::JoinHandle<()>>,
}

impl ServerHandle {
    /// Stop the world loop and the listeners; connections close as their
    /// threads notice. Returns once the world loop has ended.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(w) = self.world.take() { let _ = w.join(); }
    }

    /// Block until the server stops (it doesn't, unless `stop` is called
    /// from elsewhere) - what `game_server` does
    pub fn wait(mut self) {
        if let Some(w) = self.world.take() { let _ = w.join(); }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) { self.stop.store(true, Ordering::Relaxed); }
}

/// One connected client. The conn thread owns the socket: it polls reads on
/// a short timeout and drains `outbox`, so a blocking read never stalls the
/// world loop.
struct Conn {
    seat: usize,
    kind: ClientKind,
    last_active: Instant,
    /// Serialized replies queued for this client, in order (welcome, pongs,
    /// command results): never dropped
    outbox: Vec<String>,
    /// The newest unsent snapshot (a scene, an obs, a probe frame). Snapshots
    /// replace each other: a slow client gets the freshest one instead of a
    /// backlog that grows without bound
    latest: Option<String>,
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

/// Queue a snapshot, replacing any the client hasn't been sent yet
fn send_snapshot(c: &mut Conn, msg: &ServerMsg) {
    if let Ok(s) = serde_json::to_string(msg) { c.latest = Some(s); }
}

impl Conn {
    fn new(seat: usize, kind: ClientKind) -> Conn {
        Conn { seat, kind, last_active: Instant::now(), outbox: vec![], latest: None, dead: false }
    }

    /// What to write next: the queued replies, then the newest snapshot
    fn take_out(&mut self) -> Vec<String> {
        let mut out: Vec<String> = self.outbox.drain(..).collect();
        out.extend(self.latest.take());
        out
    }
}

/// Accept on `listener` until `stop`, handing each stream to `handle`
fn accept_loop(listener: TcpListener, stop: Arc<AtomicBool>, handle: impl Fn(TcpStream) + Send + 'static) {
    // non-blocking accepts, polled, so a stopped server lets go of its port
    listener.set_nonblocking(true).ok();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).ok();
                    stream.set_nodelay(true).ok();
                    handle(stream);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(10)),
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    });
}

/// Start a server: bind its ports, start the world loop, and return at once
pub fn serve(cfg: ServerConfig) -> std::io::Result<ServerHandle> {
    let server = Arc::new(Mutex::new(Server {
        game: Game::new_seeded(cfg.seed),
        claims: Default::default(),
        conns: vec![],
        probes: vec![],
        frame: 0,
        idle: cfg.idle,
    }));
    let stop = Arc::new(AtomicBool::new(false));

    let listener = TcpListener::bind((cfg.bind.as_str(), cfg.port))?;
    let addr = listener.local_addr()?;
    {
        let (server, stop2) = (Arc::clone(&server), Arc::clone(&stop));
        accept_loop(listener, Arc::clone(&stop), move |stream| {
            let (server, stop) = (Arc::clone(&server), Arc::clone(&stop2));
            std::thread::spawn(move || handle_conn(stream, server, stop));
        });
    }

    // the probe: a second, read-only listener for tests - the secret API.
    // It's bound to localhost only (the secret state never leaves the box),
    // and probe_token makes the client present the word before it reads
    let mut probe_addr = None;
    if let Some(pp) = cfg.probe_port {
        let pl = TcpListener::bind(("127.0.0.1", pp))?;
        probe_addr = Some(pl.local_addr()?);
        let (server, stop2, token) = (Arc::clone(&server), Arc::clone(&stop), cfg.probe_token.clone());
        accept_loop(pl, Arc::clone(&stop), move |stream| {
            let (server, stop, token) = (Arc::clone(&server), Arc::clone(&stop2), token.clone());
            std::thread::spawn(move || handle_probe(stream, server, token, stop));
        });
    }

    // sim + broadcast loop
    let frame_len = Duration::from_micros(1_000_000 / cfg.tick_hz.max(1));
    let world = {
        let (server, stop) = (Arc::clone(&server), Arc::clone(&stop));
        std::thread::spawn(move || world_loop(server, stop, frame_len))
    };
    Ok(ServerHandle { addr, probe_addr, stop, world: Some(world) })
}

fn world_loop(server: Arc<Mutex<Server>>, stop: Arc<AtomicBool>, frame_len: Duration) {
    while !stop.load(Ordering::Relaxed) {
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
                        send_snapshot(&mut c, &ServerMsg::Scene(Box::new(model)));
                    }
                    ClientKind::Agent => match s.game.agent_obs(c.seat) {
                        Some(obs) => send_snapshot(&mut c, &ServerMsg::Obs(Box::new(obs))),
                        None => send_msg(&mut c, &ServerMsg::Denied { reason: "seat gone".into() }),
                    },
                }
            }
            // probe sockets get the secret view, on their own channel
            if !s.probes.is_empty() {
                let pv = serde_json::to_string(&s.game.probe()).unwrap_or_default();
                for conn in s.probes.iter() {
                    let mut c = conn.lock().unwrap();
                    c.latest = Some(pv.clone());
                }
            }
        }
        drop(s);
        let spent = at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }
}

fn handle_conn(stream: TcpStream, server: Arc<Mutex<Server>>, stop: Arc<AtomicBool>) {
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
                // their own seat's frame, never seat 0's; agents mostly read Obs
                let first = ServerMsg::Welcome { you, token: tok, model: Box::new(s.game.ui_model_at(seat)) };
                let conn = Arc::new(Mutex::new(Conn::new(seat, kind)));
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
        if conn.lock().unwrap().dead || stop.load(Ordering::Relaxed) { break; }
        let out = conn.lock().unwrap().take_out();
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
                let seat = conn.lock().unwrap().seat;
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
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
    // dropped: the seat keeps its character; claims still map it for reclaim
    let _ = ws.close(None);
    let _ = ws.flush();
    let mut s = server.lock().unwrap();
    s.conns.retain(|c| !Arc::ptr_eq(c, &conn));
}

/// A probe connection: the test API. It accepts the websocket handshake,
/// demands the token when one's set, then only ever writes - ProbeView
/// frames at the broadcast cadence. Anything else the client sends is
/// ignored: the secret view gives nothing back to play with.
fn handle_probe(stream: TcpStream, server: Arc<Mutex<Server>>, token: Option<String>, stop: Arc<AtomicBool>) {
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
    let conn = Arc::new(Mutex::new(Conn::new(0, ClientKind::Agent)));
    {
        let mut s = server.lock().unwrap();
        s.probes.push(conn.clone());
    }
    loop {
        if conn.lock().unwrap().dead || stop.load(Ordering::Relaxed) { break; }
        let out = conn.lock().unwrap().take_out();
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
