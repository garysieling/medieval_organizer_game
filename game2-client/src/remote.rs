//! The live-server socket, shared by `game_ui` (play) and `sound_dump`
//! (`--connect`: join a `game_server` as a human seat and record its
//! scene stream, #1493). Extracted from the game_ui bin unchanged, plus
//! a `scenes` queue so a recorder can drain every frame instead of
//! polling `latest`.
use game2_core::session::{self, ClientKind, ClientMsg, SceneCache, ServerMsg, UiModel};
use std::sync::{Arc, Mutex};

/// A claim token kept on disk: relaunch and you're the same wanderer (the
/// browser keeps it in localStorage)
pub fn claim_token() -> Option<String> {
    std::fs::read_to_string("game2.claim").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
pub fn save_token(t: &str) { let _ = std::fs::write("game2.claim", t); }

/// A live server connection: the ws for sends, the latest scene for draws
pub struct Remote {
    pub ws: Mutex<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>,
    pub latest: Mutex<Option<UiModel>>,
    pub token_seen: Mutex<Option<String>>,
    /// The server turned us away (a wrong join secret, a full village)
    pub denied: Mutex<Option<String>>,
    /// The roster the Welcome dealt, for the character picker
    pub roster: Mutex<Option<Vec<session::roster::RosterCard>>>,
    /// The seed's stage-setting facts, for the prologue card after the picker
    pub intro: Mutex<Option<Vec<String>>>,
    /// Every scene as it arrived, for the scene recorder - `latest`
    /// keeps only the last one and a recorder must not lose frames
    pub scenes: Mutex<Vec<UiModel>>,
}

impl Remote {
    /// Take every scene queued since the last drain.
    pub fn drain_scenes(&self) -> Vec<UiModel> {
        std::mem::take(&mut *self.scenes.lock().unwrap())
    }
}

pub fn connect_remote(url: &str) -> Arc<Remote> {
    let (mut ws, _) = tungstenite::connect(url).expect("can't reach the server");
    ws.send(tungstenite::Message::Text(
        serde_json::to_string(&ClientMsg::Hello { token: claim_token(), kind: ClientKind::Human, name: None,
            secret: std::env::var("GAME2_JOIN_SECRET").ok().filter(|s| !s.is_empty()) }).unwrap().into()
    )).ok();
    let remote = Arc::new(Remote {
        ws: Mutex::new(ws), latest: Mutex::new(None), token_seen: Mutex::new(None),
        denied: Mutex::new(None), roster: Mutex::new(None), intro: Mutex::new(None),
        scenes: Mutex::new(vec![]),
    });
    {
        let remote = remote.clone();
        std::thread::spawn(move || {
            // scenes arrive without the map (it's sent apart, when it
            // changes): the cache puts the last one back in
            let mut cache = SceneCache::new();
            let mut welcomed = false;
            loop {
                let msg = remote.ws.lock().unwrap().read();
                match msg {
                    Ok(tungstenite::Message::Text(t)) => match serde_json::from_str::<ServerMsg>(&t) {
                        Ok(msg) => {
                            if let ServerMsg::Welcome { token, roster, intro, .. } = &msg {
                                *remote.token_seen.lock().unwrap() = Some(token.clone());
                                *remote.roster.lock().unwrap() = Some(roster.clone());
                                *remote.intro.lock().unwrap() = Some(intro.clone());
                            }
                            // a Denied before the Welcome is a refusal (we're not
                            // in); after it, it's a warning ("slow down") and play goes on
                            if let ServerMsg::Welcome { .. } = &msg { welcomed = true; }
                            if let ServerMsg::Denied { reason } = &msg {
                                if welcomed { eprintln!("remote: the server says: {reason}"); }
                                else { *remote.denied.lock().unwrap() = Some(reason.clone()); }
                            }
                            if let Some(scene) = cache.apply(msg) {
                                remote.scenes.lock().unwrap().push(scene.clone());
                                *remote.latest.lock().unwrap() = Some(scene);
                            }
                        }
                        Err(_) => {}
                    },
                    _ => break,
                }
            }
        });
    }
    remote
}
