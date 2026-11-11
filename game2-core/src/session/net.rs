//! The wire protocol between clients (desktop window, browser, API bots)
//! and the authoritative server. JSON over WebSocket: small, debuggable,
//! and readable by agents in any language.
//!
//! Sessions are claimed, not logged in: the server hands every connection a
//! character and an opaque claim token; presenting the token on reconnect
//! resumes the same character. When a seat goes quiet past its idle
//! timeout, the world's own logic drives the character until they return.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use super::{Command, UiModel};
use crate::game::tilemap::TileMap;

/// Who is connecting: a person behind a UI, or an API agent steering a villager
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClientKind {
    Human,
    Agent,
}

/// Client -> server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    /// First message on any connection
    Hello {
        /// Previously issued claim token, or None for a fresh character
        token: Option<String>,
        kind: ClientKind,
        /// Display name for the roster
        name: Option<String>,
        /// The server's join secret, when it runs with one (`game_server
        /// --join-secret`); clients take it from GAME2_JOIN_SECRET
        #[serde(default)]
        secret: Option<String>,
    },
    /// One input toward the player's character
    Command(Command),
    /// Agent steering: what the bot wants its villager to do next
    AgentCmd(AgentCmd),
    /// Take a card off the roster the Welcome dealt: becomes your character
    Pick(usize),
    /// Keep the default wanderer instead of picking
    DeclinePick,
    /// Keepalive / latency probe
    Ping(u64),
}

/// Server -> client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    /// Answer to Hello: who you are, your claim token, and the first frame
    Welcome {
        /// The character you're driving
        you: Entity,
        /// Present this on reconnect to keep the same character
        token: String,
        /// Your seat's first render model
        model: Box<UiModel>,
        /// The seed's premade roster, for the picker a fresh seat sees.
        /// Older servers omit it - read it as empty (no picker).
        #[serde(default)]
        roster: Vec<crate::session::roster::RosterCard>,
        /// Stage-setting facts about this seed's world, for the prologue
        /// card after the picker. Older servers omit it - read it as
        /// empty (no card).
        #[serde(default)]
        intro: Vec<String>,
    },
    /// A frame for your seat (sent at the broadcast cadence). Its `map` is
    /// empty: the map comes as `Map` (see `SceneCache`)
    Scene(Box<UiModel>),
    /// An agent's observation of the world around its villager
    Obs(Box<AgentObs>),
    /// A steering command finished (or couldn't)
    CmdDone { ok: bool, note: String },
    /// Refusal of the last message
    Denied { reason: String },
    /// Answer to Ping
    Pong(u64),
    /// The tile map. Scenes and the Welcome carry an empty `map` - it's 98%
    /// of a frame - so the server sends this after the Welcome and again
    /// whenever the map changes. Keep the last one and put it into each
    /// scene before drawing (`SceneCache` does that).
    Map(Box<TileMap>),
}

/// Client-side reassembly of the scene stream: remembers the last `Map` and
/// puts it into every scene, so a frame is whole again before it's drawn
#[derive(Default)]
pub struct SceneCache {
    map: Option<TileMap>,
}

impl SceneCache {
    pub fn new() -> SceneCache { SceneCache::default() }

    /// Take a server message. A `Map` is kept; a `Scene` (or the Welcome's
    /// first frame) comes back with the kept map in it. Anything else, or a
    /// scene before any map, gives None.
    pub fn apply(&mut self, msg: ServerMsg) -> Option<UiModel> {
        match msg {
            ServerMsg::Map(m) => { self.map = Some(*m); None }
            ServerMsg::Scene(s) | ServerMsg::Welcome { model: s, .. } => {
                let mut s = *s;
                s.map = self.map.as_ref()?.clone();
                Some(s)
            }
            _ => None,
        }
    }

    /// The map as last sent, if one's come
    pub fn map(&self) -> Option<&TileMap> { self.map.as_ref() }
}

// ---------------- the agent (bot) surface ----------------

/// What a villager-driving agent sees: itself, its surroundings, and the
/// little it needs to pick its next move. Compact on purpose - agents act
/// on day boundaries and on arrival, not every frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObs {
    pub tick: u64,
    /// The villager you're driving
    pub you: Entity,
    pub pos: (f32, f32),
    pub gold: f32,
    pub hunger: f32,
    /// What the village is up to right now (if the sim tracks it)
    pub busy: Option<String>,
    /// Nearby people (villagers, the wanderers): entity, name, distance
    pub near: Vec<ObsPerson>,
    /// Fold and yard animals within reach of a walk, including whether this
    /// villager could tame them with a contextual action
    #[serde(default)]
    pub near_animals: Vec<ObsAnimal>,
    /// Nearby buildings worth knowing about: entity, name, kind
    pub near_buildings: Vec<ObsBuilding>,
    /// Your satchel's contents
    pub satchel: Vec<(String, u32)>,
    /// The walk under way, if any
    pub walking: bool,
    /// Whether the cell holds you - no errands until you're out
    #[serde(default)]
    pub held: bool,
    /// The orgs you march under - every membership or leadership, since
    /// a villager can belong to more than one
    #[serde(default)]
    pub orgs: Vec<String>,
    /// The issues your orgs' campaigns are fighting
    #[serde(default)]
    pub campaigns: Vec<String>,
    /// Burning tiles near you - walk around, not through
    #[serde(default)]
    pub burning: Vec<(f32, f32)>,
    /// What your villager knows about themself: their trade, home and
    /// bench, what the bench lacks, what they want. Only their own - other
    /// people's wants stay theirs (the probe's, not the agent's)
    #[serde(default)]
    pub me: ObsSelf,
    /// The market board's postings - (kind, text) like the market panel
    /// shows, and only while you stand close enough to read them
    #[serde(default)]
    pub notices: Vec<(String, String)>,
    /// Where the board stands, so walking there is a decision, not a hunt
    #[serde(default)]
    pub board: Option<(f32, f32)>,
}

/// An agent's villager's knowledge of themself, to plan by rather than
/// only react
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObsSelf {
    /// The trade ("baker") and its rung ("Journeyman"), if they have one
    pub trade: Option<String>,
    pub level: Option<String>,
    /// Their home and their bench (workplace), as entities near_buildings
    /// and AgentCmd use
    pub home: Option<Entity>,
    pub bench: Option<Entity>,
    /// What's on the bench's shelf
    pub bench_stock: Vec<(String, u32)>,
    /// The raw good the trade is short of, if it is: fetch it or buy it
    pub needs: Option<String>,
    /// What they want and haven't got ("Joy", "BetterTools"...)
    pub wants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsPerson {
    pub e: Entity,
    pub name: String,
    pub dist: f32,
    pub dead: bool,
    /// The job kind they currently have going begging, if any
    #[serde(default)]
    pub job_offer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsAnimal {
    pub e: Entity,
    pub kind: String,
    pub pos: (f32, f32),
    pub dist: f32,
    pub tameable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsBuilding {
    pub e: Entity,
    pub name: String,
    pub kind: String,
    pub dist: f32,
}

/// Context actions a villager agent may request. Campaign leadership actions
/// remain exclusive to human organizer seats.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentAction {
    TakeJob,
    Douse,
    Tame,
    Recruit,
    Matchmake,
    Errand,
    PairUp,
    ShareFindings,
    PickCaps,
    Snowballs,
    PetCat,
    Pact,
}

/// What an agent can tell its villager to do. Movement and shopping are
/// walks; the sim does the errand on arrival (like the villagers' own
/// Goal/errand machinery).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCmd {
    /// Stand down for a bit (still driven between commands)
    Idle,
    /// Walk to a tile
    GoTo { x: i32, y: i32 },
    /// Walk to a person or building to talk with them
    TalkTo { e: Entity },
    /// Buy `qty` of `good` from a shop's stockpile or a person's satchel
    BuyFrom { seller: Entity, good: String, qty: u32 },
    /// Buy `good` at a shop (satisfies a want)
    BuyAt { shop: Entity, good: String },
    /// Gather `good` at a tile (wood, stone, sand)
    Gather { good: String, x: i32, y: i32 },
    /// Put in a day at the craft (if there's a craft and inputs)
    WorkDay,
    /// Eat at the nearest food source (hunger first)
    Eat,
    /// Sign on with the cause: join the org when its issue has woken you
    /// or the fight is your own
    JoinOrg,
    /// Write letters for a prisoner's release - your voice adds pressure
    /// the jailer can't ignore
    Press { prisoner: Entity },
    /// Get the raw good the bench is short of (`AgentObs.me.needs`): the
    /// villager's own errand - send a kid, or go to where it's had
    FetchInput,
    /// Take an available contextual action. `target` is a nearby villager
    /// for actions such as TakeJob, Recruit, Errand, or PairUp; actions that
    /// use the agent's own position can leave it empty.
    Context { action: AgentAction, target: Option<Entity> },
    /// Take the board's oldest posted job - a quest advertised on the
    /// notice board is accepted where it hangs, not at the asker's door
    TakeNotice,
}
