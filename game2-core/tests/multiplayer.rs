//! Multiplayer over real sockets: a server on an ephemeral port, played by
//! a human client, an agent and a probe through the public API only - the
//! same JSON-over-WebSocket protocol `game_ui --connect` and the bots speak.

use std::net::TcpStream;
use std::time::{Duration, Instant};

use game2_core::server::{serve, ServerConfig, ServerHandle};
use game2_core::session::{AgentCmd, ClientKind, ClientMsg, Command, SceneCache, ServerMsg, MAP_SEED};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

type Ws = WebSocket<MaybeTlsStream<TcpStream>>;

/// A fast server: 200 frames a second, so a scene comes every 10ms
fn start(probe_token: Option<&str>) -> ServerHandle {
    start_with(ServerConfig { probe_token: probe_token.map(String::from), ..fast() })
}

fn fast() -> ServerConfig {
    ServerConfig { bind: "127.0.0.1".into(), port: 0, seed: MAP_SEED, idle: Duration::from_secs(600),
        tick_hz: 200, probe_port: Some(0), ..ServerConfig::default() }
}

fn start_with(cfg: ServerConfig) -> ServerHandle {
    serve(cfg).expect("server starts")
}

/// Say Hello with a secret, and see whether the server lets us in
fn admitted(addr: std::net::SocketAddr, token: Option<String>, secret: Option<&str>) -> Result<String, String> {
    let mut ws = connect(addr);
    send(&mut ws, &ClientMsg::Hello { token, kind: ClientKind::Human, name: None, secret: secret.map(String::from) });
    until("a welcome or a refusal", &mut ws, 10, |m, _| match m {
        ServerMsg::Welcome { token, .. } => Some(Ok(token.clone())),
        ServerMsg::Denied { reason } => Some(Err(reason.clone())),
        _ => None,
    })
}

fn connect(addr: std::net::SocketAddr) -> Ws {
    let (ws, _) = tungstenite::connect(format!("ws://{addr}")).expect("connects");
    if let MaybeTlsStream::Plain(s) = ws.get_ref() { s.set_read_timeout(Some(Duration::from_millis(100))).ok(); }
    ws
}

fn send(ws: &mut Ws, m: &ClientMsg) {
    ws.send(Message::Text(serde_json::to_string(m).unwrap().into())).expect("sends");
}

enum Got { Msg(String), Closed, Nothing }

fn read_text(ws: &mut Ws) -> Got {
    match ws.read() {
        Ok(Message::Text(t)) => Got::Msg(t.to_string()),
        Ok(Message::Close(_)) => Got::Closed,
        Ok(_) => Got::Nothing,
        Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Got::Nothing,
        Err(_) => Got::Closed,
    }
}

/// Read server messages until `want` picks one out (or `secs` pass)
fn until<T>(what: &str, ws: &mut Ws, secs: u64, mut want: impl FnMut(&ServerMsg, &str) -> Option<T>) -> T {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if let Got::Msg(t) = read_text(ws) {
            let m: ServerMsg = serde_json::from_str(&t).expect("server sends ServerMsg JSON");
            if let Some(x) = want(&m, &t) { return x; }
        }
    }
    panic!("waited {secs}s for {what}");
}

fn hello(ws: &mut Ws, token: Option<String>, kind: ClientKind) -> (game2_core::engine::ecs::Entity, String) {
    send(ws, &ClientMsg::Hello { token, kind, name: Some("test".into()), secret: None });
    until("a welcome", ws, 10, |m, _| match m {
        ServerMsg::Welcome { you, token, .. } => Some((*you, token.clone())),
        _ => None,
    })
}

#[test]
fn a_human_and_an_agent_play_one_world() {
    let server = start(None);

    // a human claims a seat and the scenes stream in
    let mut human = connect(server.addr);
    let (me, token) = hello(&mut human, None, ClientKind::Human);
    assert!(!token.is_empty(), "a claim token to come back with");
    let at = |m: &ServerMsg| match m {
        ServerMsg::Scene(s) => s.people.iter().find(|p| p.e == me).map(|p| p.pos),
        _ => None,
    };
    // the map comes apart from the scenes: once after the Welcome, and
    // scenes carry an empty one - the map is 98% of a frame
    let map = until("the map", &mut human, 10, |m, _| match m { ServerMsg::Map(m) => Some(m.clone()), _ => None });
    assert!(map.width > 0 && map.height > 0, "a real map");
    let (lean, json_len) = until("a scene", &mut human, 10, |m, t| match m {
        ServerMsg::Scene(s) => Some((s.map.width, t.len())),
        _ => None,
    });
    assert_eq!(lean, 0, "scenes don't carry the map");
    assert!(json_len < 200_000, "a scene is {json_len} bytes - the map is riding along again?");
    // and a client puts the two back together
    let mut cache = SceneCache::new();
    assert!(cache.apply(ServerMsg::Map(map.clone())).is_none());
    let whole = until("a scene to rebuild", &mut human, 10, |m, _| match m {
        ServerMsg::Scene(_) => cache.apply(m.clone()),
        _ => None,
    });
    assert_eq!(whole.map.digest(), map.digest(), "the rebuilt scene has the map");
    let start_pos = until("my figure in a scene", &mut human, 10, |m, _| at(m));

    // commands land: a few steps east move the character east
    for _ in 0..20 { send(&mut human, &ClientMsg::Command(Command::Walk(1.0, 0.0))); }
    let moved = until("my figure to move", &mut human, 10, |m, _| at(m).filter(|p| (p.0 - start_pos.0).abs() + (p.1 - start_pos.1).abs() > 0.2));
    assert!(moved != start_pos);

    // the map isn't sent again while it hasn't changed
    let resent = {
        let end = Instant::now() + Duration::from_millis(500);
        let mut n = 0;
        while Instant::now() < end {
            if let Got::Msg(t) = read_text(&mut human) { if t.starts_with("{\"Map\"") { n += 1; } }
        }
        n
    };
    assert!(resent <= 1, "the unchanged map was sent {resent} times in half a second");

    // and the seat answers a ping
    send(&mut human, &ClientMsg::Ping(41));
    until("a pong", &mut human, 10, |m, _| matches!(m, ServerMsg::Pong(41)).then_some(()));

    // an agent takes a villager, watches it, and steers it
    let mut agent = connect(server.addr);
    let (villager, _) = hello(&mut agent, None, ClientKind::Agent);
    assert_ne!(villager, me, "the agent drives its own character");
    // an agent reads Obs, not scenes: it's never sent the map
    let mut agent_maps = 0;
    let obs_you = until("the agent's obs", &mut agent, 10, |m, _| match m {
        ServerMsg::Map(_) => { agent_maps += 1; None }
        ServerMsg::Obs(o) => Some(o.you),
        _ => None,
    });
    let end = Instant::now() + Duration::from_millis(300);
    while Instant::now() < end {
        if let Got::Msg(t) = read_text(&mut agent) { if t.starts_with("{\"Map\"") { agent_maps += 1; } }
    }
    assert_eq!(agent_maps, 0, "an agent was sent the map");
    assert_eq!(obs_you, villager, "the obs is about the agent's own villager");
    send(&mut agent, &ClientMsg::AgentCmd(AgentCmd::Idle));
    let ok = until("CmdDone", &mut agent, 10, |m, _| match m { ServerMsg::CmdDone { ok, .. } => Some(*ok), _ => None });
    assert!(ok, "Idle is always accepted");

    // nothing secret on the wire: the secret gauges' names never appear
    for _ in 0..5 {
        let scene = until("a scene", &mut human, 10, |m, t| matches!(m, ServerMsg::Scene(_)).then(|| t.to_string()));
        let obs = until("an obs", &mut agent, 10, |m, t| matches!(m, ServerMsg::Obs(_)).then(|| t.to_string()));
        for secret in ["org.power", "org.lord_esteem", "villagers.starving", "poison_mushrooms"] {
            assert!(!scene.contains(secret) && !obs.contains(secret), "{secret} leaked to a client");
        }
    }

    // a reconnect with the token gets the same character back, and the old
    // connection is retired: one character, one driver
    let mut again = connect(server.addr);
    let (me_again, _) = hello(&mut again, Some(token.clone()), ClientKind::Human);
    assert_eq!(me_again, me, "the claim resumes the same character");
    let end = Instant::now() + Duration::from_secs(10);
    let mut closed = false;
    while Instant::now() < end {
        if let Got::Closed = read_text(&mut human) { closed = true; break; }
    }
    assert!(closed, "the reclaimed seat's old socket is closed");
    until("my figure after the reclaim", &mut again, 10, |m, _| at(m));

    server.stop();
}

#[test]
fn the_probe_is_its_own_tokened_socket() {
    let server = start(Some("sesame"));
    let probe = server.probe_addr.expect("a probe port");
    assert!(probe.ip().is_loopback(), "the secret state never leaves the box");
    assert_ne!(probe.port(), server.addr.port());

    // the wrong word: nothing comes, and the socket goes nowhere
    let mut wrong = connect(probe);
    wrong.send(Message::Text("open up".into())).unwrap();
    let end = Instant::now() + Duration::from_secs(2);
    while Instant::now() < end {
        if let Got::Msg(_) = read_text(&mut wrong) { panic!("the probe talked without its token"); }
    }

    // the right word: the omniscient view streams
    let mut right = connect(probe);
    right.send(Message::Text("sesame".into())).unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    let mut seen = None;
    while Instant::now() < end && seen.is_none() {
        if let Got::Msg(t) = read_text(&mut right) { seen = Some(t); }
    }
    let pv: serde_json::Value = serde_json::from_str(&seen.expect("probe frames arrive")).unwrap();
    for k in ["lord_esteem", "org_power", "poison_mushrooms", "inner"] {
        assert!(pv.get(k).is_some(), "the probe shows {k}");
    }
    server.stop();
}

#[test]
fn a_stopped_server_lets_go_of_its_ports_even_with_a_silent_socket_open() {
    let server = start(None);
    let (addr, probe) = (server.addr, server.probe_addr.unwrap());
    // a socket that connects and never says a word - no handshake, no Hello
    let _silent = TcpStream::connect(addr).expect("connects");
    // and one that shakes hands but never says Hello
    let _mute = connect(addr);
    std::thread::sleep(Duration::from_millis(200));
    let t = Instant::now();
    server.stop();
    assert!(t.elapsed() < Duration::from_secs(5), "stop waited on a silent socket");
    // and the silent socket itself is let go soon after, not at the hello deadline
    let mut silent = _silent;
    silent.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut b = [0u8; 1];
    let t = Instant::now();
    let closed = matches!(std::io::Read::read(&mut silent, &mut b), Ok(0) | Err(_)) && t.elapsed() < Duration::from_secs(2);
    assert!(closed, "a silent socket outlived the stop by {:?}", t.elapsed());
    // the ports are free the moment stop returns
    assert!(std::net::TcpListener::bind(addr).is_ok(), "the stopped server still holds {addr}");
    assert!(std::net::TcpListener::bind(probe).is_ok(), "the stopped server still holds the probe {probe}");
}

#[test]
fn a_slow_handshake_still_gets_in() {
    // a client that pauses mid-handshake (over the server's poll interval)
    // is still let in, well inside the hello deadline
    use std::io::Write;
    let server = start(None);
    let mut tcp = TcpStream::connect(server.addr).unwrap();
    let req = format!("GET / HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n", server.addr);
    tcp.write_all(req.as_bytes()).unwrap();
    std::thread::sleep(Duration::from_millis(400)); // the pause
    tcp.write_all(b"Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n").unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut buf = [0u8; 256];
    let n = std::io::Read::read(&mut tcp, &mut buf).unwrap_or(0);
    let reply = String::from_utf8_lossy(&buf[..n]);
    assert!(reply.starts_with("HTTP/1.1 101"), "the slow client was turned away: {reply:?}");
    server.stop();
}

#[test]
fn a_browser_with_a_big_cookie_gets_in() {
    // upgrade headers well past 8 KB (a browser's cookies ride along)
    use std::io::Write;
    let server = start(None);
    let mut tcp = TcpStream::connect(server.addr).unwrap();
    let cookie = "x".repeat(20 * 1024);
    let req = format!("GET / HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nCookie: c={cookie}\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n", server.addr);
    tcp.write_all(req.as_bytes()).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut buf = [0u8; 256];
    let n = std::io::Read::read(&mut tcp, &mut buf).unwrap_or(0);
    assert!(String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/1.1 101"), "a 20 KB upgrade request was turned away");
    server.stop();
}

#[test]
fn a_failed_bind_leaves_nothing_running() {
    // a free port for the game, and the probe's port already taken
    let game_port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let r = serve(ServerConfig { bind: "127.0.0.1".into(), port: game_port, probe_port: Some(taken.local_addr().unwrap().port()),
        ..ServerConfig::default() });
    assert!(r.is_err(), "the probe's port was taken");
    // the game port wasn't left bound behind the error
    assert!(std::net::TcpListener::bind(("127.0.0.1", game_port)).is_ok(), "the game port leaked");
}

#[test]
fn a_join_secret_keeps_strangers_out() {
    let server = start_with(ServerConfig { join_secret: Some("open sesame".into()), ..fast() });
    assert_eq!(admitted(server.addr, None, None), Err("wrong join secret".into()), "no secret, no seat");
    assert_eq!(admitted(server.addr, None, Some("guess")), Err("wrong join secret".into()));
    assert!(admitted(server.addr, None, Some("open sesame")).is_ok(), "the right secret gets a seat");
    server.stop();
}

#[test]
fn the_village_hands_out_only_so_many_seats() {
    let server = start_with(ServerConfig { max_seats: 2, ..fast() });
    let first = admitted(server.addr, None, None).expect("seat one");
    admitted(server.addr, None, None).expect("seat two");
    assert_eq!(admitted(server.addr, None, None), Err("the village is full".into()), "no third seat");
    // but a claim still takes its own seat back when the village is full
    assert_eq!(admitted(server.addr, Some(first.clone()), None), Ok(first), "a reclaim always works");
    server.stop();
}
