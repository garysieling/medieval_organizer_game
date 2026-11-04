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
/// Sockets open at once, game and probe together; past it new ones are
/// turned away at the door, so a flood can't run the box out of threads
const MAX_CONNS: usize = 256;
/// How long a new socket has to finish the WebSocket handshake and say
/// Hello before it's dropped
const HELLO_WITHIN: Duration = Duration::from_secs(10);
/// How often a waiting connection looks up from its socket (for the
/// deadline and a stop)
const POLL: Duration = Duration::from_millis(100);
/// The longest upgrade request let in (headers and all): a browser's cookies
/// ride along, so this is generous
const MAX_UPGRADE_REQUEST: usize = 64 * 1024;

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
    /// The word every Hello must carry, if any: a light gate on who may play
    pub join_secret: Option<String>,
    /// New seats this server will hand out, in all. Anyone can say Hello, and
    /// every fresh Hello mints a character, so the world's size is capped.
    /// Reclaiming a seat with its token always works.
    pub max_seats: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { bind: "0.0.0.0".into(), port: 8787, seed: crate::session::MAP_SEED,
            idle: Duration::from_secs(300), tick_hz: 20, probe_port: None, probe_token: None,
            join_secret: None, max_seats: 64 }
    }
}

/// A running server: where it listens, and the switch that stops it
pub struct ServerHandle {
    pub addr: SocketAddr,
    pub probe_addr: Option<SocketAddr>,
    stop: Arc<AtomicBool>,
    world: Option<std::thread::JoinHandle<()>>,
    /// The accept loops: joined on stop, so their ports are free after
    accepts: Vec<std::thread::JoinHandle<()>>,
}

impl ServerHandle {
    /// Stop the world loop and the listeners; connections close as their
    /// threads notice (each polls the stop flag). Returns once the world
    /// loop has ended and the listening ports are released.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(w) = self.world.take() { let _ = w.join(); }
        for a in self.accepts.drain(..) { let _ = a.join(); }
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
    /// The newest unsent map (a snapshot as well, ~1 MB: a slow client
    /// gets only the latest, written before the scene that's drawn on it)
    latest_map: Option<String>,
    /// Another connection reclaimed this seat: the read loop ends
    dead: bool,
    /// The digest of the map this client was last sent (`TileMap::digest`):
    /// the map goes out again only when the world's moves off it
    map_sent: Option<u64>,
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
    join_secret: Option<String>,
    max_seats: usize,
    /// Seats handed out so far (see ServerConfig::max_seats)
    minted: usize,
}

/// A claim token: 128 bits no one can guess. std's `RandomState` keys its
/// SipHash from the OS's randomness, so hashing a counter (and the time)
/// through two of them gives output an outsider can't predict - without
/// pulling in a crate for it. The token is the seat's only credential.
fn token() -> String {
    use std::hash::{BuildHasher, Hash, Hasher};
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let half = |salt: u64| {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        (salt, n, std::time::SystemTime::now()).hash(&mut h);
        h.finish()
    };
    format!("{:016x}{:016x}", half(0x5eed), half(0xc1a1))
}

fn send_msg(c: &mut Conn, msg: &ServerMsg) {
    if let Ok(s) = serde_json::to_string(msg) { c.outbox.push(s); }
}

/// Queue the map, replacing any the client hasn't been sent yet
fn send_map(c: &mut Conn, map: &crate::game::tilemap::TileMap) {
    if let Ok(s) = serde_json::to_string(&ServerMsg::Map(Box::new(map.clone()))) { c.latest_map = Some(s); }
}

/// Queue a snapshot, replacing any the client hasn't been sent yet
fn send_snapshot(c: &mut Conn, msg: &ServerMsg) {
    if let Ok(s) = serde_json::to_string(msg) { c.latest = Some(s); }
}

impl Conn {
    fn new(seat: usize, kind: ClientKind) -> Conn {
        Conn { seat, kind, last_active: Instant::now(), outbox: vec![], latest: None, latest_map: None, dead: false, map_sent: None }
    }

    /// What to write next: the queued replies, then the newest snapshot
    fn take_out(&mut self) -> Vec<String> {
        let mut out: Vec<String> = self.outbox.drain(..).collect();
        out.extend(self.latest_map.take());
        out.extend(self.latest.take());
        out
    }
}

/// Sockets open now (see MAX_CONNS)
type Open = Arc<std::sync::atomic::AtomicUsize>;

/// Counts a socket open for as long as its thread holds it
struct Slot(Open);
impl Drop for Slot {
    fn drop(&mut self) { self.0.fetch_sub(1, Ordering::Relaxed); }
}

/// Accept on `listener` until `stop`, handing each stream (on its own
/// thread, holding a slot) to `handle`. Past MAX_CONNS a socket is closed
/// at once.
fn accept_loop(listener: TcpListener, stop: Arc<AtomicBool>, open: Open,
    handle: impl Fn(TcpStream) + Send + Sync + 'static) -> std::thread::JoinHandle<()> {
    // non-blocking accepts, polled, so a stopped server lets go of its port
    listener.set_nonblocking(true).ok();
    let handle = Arc::new(handle);
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if open.fetch_add(1, Ordering::Relaxed) >= MAX_CONNS {
                        open.fetch_sub(1, Ordering::Relaxed);
                        continue; // dropped: the door's shut
                    }
                    let slot = Slot(Arc::clone(&open));
                    stream.set_nonblocking(false).ok();
                    stream.set_nodelay(true).ok();
                    // reads poll, so a silent socket notices the hello
                    // deadline and a stop (see accept_ws)
                    stream.set_read_timeout(Some(POLL)).ok();
                    let handle = Arc::clone(&handle);
                    std::thread::spawn(move || { handle(stream); drop(slot); });
                }
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    })
}

/// The WebSocket handshake, within the hello deadline and abandoned on a
/// stop. The upgrade request is waited for with `peek` on the short poll
/// timeout - reading nothing - until it's all there; only then does
/// tungstenite read it, so a slow client is let in (a timed-out read mid-
/// handshake would fail it: on Windows that's TimedOut, not WouldBlock) and
/// a silent one is let go on a stop.
fn accept_ws(stream: TcpStream, stop: &AtomicBool) -> Option<tungstenite::WebSocket<TcpStream>> {
    let until = Instant::now() + HELLO_WITHIN;
    // room for a browser's upgrade request with a big cookie header; only
    // held while the handshake's under way
    let mut buf = vec![0u8; MAX_UPGRADE_REQUEST];
    loop {
        if stop.load(Ordering::Relaxed) || Instant::now() > until { return None; }
        match stream.peek(&mut buf) {
            Ok(0) => return None, // closed
            // the request ends at a blank line (CRLF, or the bare LF the
            // HTTP parser also takes)
            Ok(n) if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") || buf[..n].windows(2).any(|w| w == b"\n\n") => break,
            Ok(n) if n == buf.len() => return None, // no upgrade request is this long
            Ok(_) => std::thread::sleep(Duration::from_millis(20)), // part of it: wait for the rest
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(_) => return None,
        }
    }
    // the request's all buffered: tungstenite reads it in one go
    stream.set_read_timeout(Some(until.saturating_duration_since(Instant::now()).max(POLL))).ok();
    let ws = tungstenite::accept(stream).ok()?;
    ws.get_ref().set_read_timeout(Some(POLL)).ok();
    Some(ws)
}

/// Turn a client away before it has a seat: say why, then close
fn deny(mut ws: tungstenite::WebSocket<TcpStream>, reason: &str) {
    if let Ok(t) = serde_json::to_string(&ServerMsg::Denied { reason: reason.into() }) {
        let _ = ws.send(tungstenite::Message::Text(t.into()));
    }
    let _ = ws.close(None);
    let _ = ws.flush();
}

/// A read that timed out (nothing yet), as opposed to one that failed
fn idle_read(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(e)
        if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut)
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
        join_secret: cfg.join_secret.clone(),
        max_seats: cfg.max_seats,
        minted: 0,
    }));
    let stop = Arc::new(AtomicBool::new(false));
    let open: Open = Default::default();

    // bind every port before starting anything, so a failed bind leaves
    // nothing running behind it
    let listener = TcpListener::bind((cfg.bind.as_str(), cfg.port))?;
    let addr = listener.local_addr()?;
    // the probe: a second, read-only listener for tests - the secret API.
    // It's bound to localhost only (the secret state never leaves the box),
    // and probe_token makes the client present the word before it reads
    let probe = match cfg.probe_port {
        Some(pp) => Some(TcpListener::bind(("127.0.0.1", pp))?),
        None => None,
    };
    let probe_addr = match &probe { Some(pl) => Some(pl.local_addr()?), None => None };

    let mut accepts = vec![];
    {
        let (server, stop2) = (Arc::clone(&server), Arc::clone(&stop));
        accepts.push(accept_loop(listener, Arc::clone(&stop), Arc::clone(&open), move |stream| {
            handle_conn(stream, Arc::clone(&server), Arc::clone(&stop2))
        }));
    }
    if let Some(pl) = probe {
        let (server, stop2, token) = (Arc::clone(&server), Arc::clone(&stop), cfg.probe_token.clone());
        accepts.push(accept_loop(pl, Arc::clone(&stop), Arc::clone(&open), move |stream| {
            handle_probe(stream, Arc::clone(&server), token.clone(), Arc::clone(&stop2))
        }));
    }

    // sim + broadcast loop
    let frame_len = Duration::from_micros(1_000_000 / cfg.tick_hz.max(1));
    let world = {
        let (server, stop) = (Arc::clone(&server), Arc::clone(&stop));
        std::thread::spawn(move || world_loop(server, stop, frame_len))
    };
    Ok(ServerHandle { addr, probe_addr, stop, world: Some(world), accepts })
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
            // the map is 98% of a frame: scenes go without it, and a seat is
            // sent the map only when it's new to them
            let digest = s.game.map.digest();
            for conn in s.conns.iter() {
                let mut c = conn.lock().unwrap();
                match c.kind {
                    ClientKind::Human => {
                        if c.map_sent != Some(digest) {
                            send_map(&mut c, &s.game.map);
                            c.map_sent = Some(digest);
                        }
                        let model = s.game.ui_model_lean_at(c.seat);
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
    let Some(mut ws) = accept_ws(stream, &stop) else { return };
    // the first message must be Hello: claim or be assigned a seat - and
    // soon, or the socket's dropped
    let hello_by = Instant::now() + HELLO_WITHIN;
    let conn = loop {
        if stop.load(Ordering::Relaxed) || Instant::now() > hello_by { return; }
        let msg = match ws.read() {
            Ok(m) => m,
            Err(e) if idle_read(&e) => continue,
            Err(_) => return,
        };
        if let tungstenite::Message::Text(t) = msg {
            let Ok(cm) = serde_json::from_str::<ClientMsg>(&t) else { continue };
            if let ClientMsg::Hello { token: tok, kind, name: _, secret } = cm {
                let mut s = server.lock().unwrap();
                // the gate: the join secret, when the server has one
                if s.join_secret.is_some() && secret != s.join_secret {
                    drop(s);
                    return deny(ws, "wrong join secret");
                }
                let claimed = tok.as_deref().and_then(|t| s.claims.get(t).copied());
                // a fresh character, unless the world's handed out its last
                if claimed.is_none() && s.minted >= s.max_seats {
                    drop(s);
                    return deny(ws, "the village is full");
                }
                let seat = claimed.unwrap_or_else(|| {
                    s.minted += 1;
                    match kind {
                        ClientKind::Human => s.game.add_seat(),
                        ClientKind::Agent => match s.game.free_villager() {
                            Some(i) => { let v = s.game.villagers[i]; s.game.add_agent_seat(v) },
                            None => s.game.add_seat(), // village full: park them on a wanderer
                        },
                    }
                });
                // a known token keeps its seat; a new seat always gets a token
                // of the server's making, never one the client picked
                let tok = match (claimed, tok) {
                    (Some(_), Some(t)) => t,
                    _ => token(),
                };
                s.claims.insert(tok.clone(), seat);
                let you = s.game.seats[seat].player;
                // their own seat's frame, never seat 0's; agents mostly read Obs
                let first = ServerMsg::Welcome { you, token: tok, model: Box::new(s.game.ui_model_lean_at(seat)) };
                let conn = Arc::new(Mutex::new(Conn::new(seat, kind)));
                {
                    // the Welcome, then (for a player, who draws scenes) the
                    // map its frame is drawn on; an agent reads Obs, no map
                    let mut c = conn.lock().unwrap();
                    send_msg(&mut c, &first);
                    if kind == ClientKind::Human {
                        send_map(&mut c, &s.game.map);
                        c.map_sent = Some(s.game.map.digest());
                    }
                }
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
    let Some(mut ws) = accept_ws(stream, &stop) else { return };
    // the word, when one's set: first frame must be it (and soon) or the
    // conn goes nowhere
    if let Some(t) = &token {
        let by = Instant::now() + HELLO_WITHIN;
        let ok = loop {
            if stop.load(Ordering::Relaxed) || Instant::now() > by { break false; }
            match ws.read() {
                Ok(tungstenite::Message::Text(m)) => break *m == *t,
                Err(e) if idle_read(&e) => continue,
                _ => break false,
            }
        };
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
