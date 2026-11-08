//! The render model: everything a frame needs that isn't the player's own
//! art, camera or mouse. Produced server-side (or in-process for solo play)
//! by `Game::ui_model`, consumed by `View::render`. Serializable so a remote
//! client draws the exact same frame as a local one.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use crate::game::tilemap::TileMap;
use crate::game::pets::Species;
use crate::sim::climate::Season;
use crate::sim::precipitation::PrecipKind;
use super::{BSprite, CtxAct};

/// A country settlement on the minimap: where it stands, its name,
/// and who keeps it - the keep's first bed for a castle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteV {
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub kind: crate::session::country::SiteKind,
    pub keeper: Option<String>,
    /// The head's style and name ("Earl Godric"), when the site has one
    #[serde(default)]
    pub head: Option<String>,
    /// The site's treasury in gold - tribute in, gifts and the crown's
    /// cut out
    #[serde(default)]
    pub treasury: f32,
}

/// A villager, rival, pet or other figure standing on the map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonV {
    /// The person entity (for bubble/name lookups)
    pub e: Entity,
    /// What they are on the map
    pub kind: PKind,
    pub pos: (f32, f32),
    /// Name over the head (villagers/rivals only)
    pub label: Option<String>,
    /// Draw a grave instead of a figure
    pub dead: bool,
    /// Small figure + "(child)" label
    pub kid: bool,
    /// Ring highlights
    pub selected: bool,
    pub in_reach: bool,
    /// Hunger/illness indicators next to the name
    pub hungry: bool,
    pub sick: bool,
    /// Activity marker: 0 idle, 1 seeking food, 2 busy
    pub act_mark: u8,
    /// A pet bonded strongly to its owner (the <3 label)
    pub bond: bool,
    /// What the find box matches against, already folded by [`find_key`]:
    /// their name and posting ("eliza journeyman baker"), empty for
    /// creatures and fixtures nobody would look for
    #[serde(default)]
    pub find: String,
    /// The player's picked figure look (villager pack index); unused for
    /// everyone else, whose look comes from their roster slot
    #[serde(default)]
    pub look: u8,
    /// Picket-sign board while gathered for a protest (0 wooden
    /// hand-lettered, 1 printed placard); the client draws it beside
    /// the figure
    #[serde(default)]
    pub sign: Option<u8>,
    /// Riding a horse (a knight on a country journey): the client draws
    /// the mounted figure instead of the walker
    #[serde(default)]
    pub mounted: bool,
    /// The grave's epitaph for the hover card ("died day 42, of a fever")
    /// - only on the dead
    #[serde(default)]
    pub epitaph: Option<String>,
    /// Laid in the churchyard (vs still lying in where they fell): the
    /// client draws the tombstone variant for these
    #[serde(default)]
    pub buried: bool,
}

/// Fold a name or epithet to what the find box compares: lowercase words,
/// anything that isn't a letter or digit becomes a space
pub fn find_key(s: &str) -> String {
    s.chars().flat_map(|c| c.to_lowercase())
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PKind {
    /// Sprite index into art.villager_sprites
    Villager(usize),
    /// Sprite index into art.rival_sprites
    Rival(usize),
    Player,
    Pet(Species),
    /// Sprite index into art.guards
    Guard(usize),
    Priest,
    Traveller,
    Chicken,
    Gull,
    /// An animal of the fold, grazing the meadow
    Sheep,
    Goat,
}

/// A building on the map
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BldgV {
    pub x: f32,
    pub y: f32,
    pub sprite: BSprite,
    /// 0..1 while it rises (under construction)
    pub risen: f32,
    pub derelict: bool,
    /// Shown open today: the printer's shop stands as a cutaway
    #[serde(default)]
    pub cutaway: bool,
    /// Chimney smoke (the cottages nearest the market)
    pub smoke: bool,
}

/// A speech or thought bubble, already placed over the speaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleV {
    pub e: Entity,
    pub pos: (f32, f32),
    pub text: String,
    pub thought: bool,
    /// Speaker's name (to find the label height under the bubble)
    pub name: Option<String>,
}

/// A line of inspector text, pre-formatted server-side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub rgb: (u8, u8, u8),
}

/// The inspector card for the selection (or none when nothing's picked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorV {
    /// Villager sprite index for the portrait; None for a building card or grave
    pub fig: Option<usize>,
    /// The figure is a child: the portrait wears a young look, not the
    /// grown-up one the bare index would cycle to
    #[serde(default)]
    pub kid: bool,
    /// Draw a grave portrait (died on `died_day`)
    pub dead: bool,
    pub died_day: Option<u64>,
    /// The seat's friendship with this villager in hearts (0..=5, half
    /// steps); None on a building card or a grave
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hearts: Option<f32>,
    pub lines: Vec<Line>,
}

/// The campaign the org is running, for the panel's status line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignV {
    pub issue: String,
    pub framing: String,
    /// Grumblers the issue's loser's left
    pub dissenters: usize,
    /// "turnout: Aldric" / "notes: (empty)"
    pub seats: Vec<String>,
    /// The member whose pull runs the room's empty seats, if one's grown
    pub shadow: Option<String>,
    /// The cause's posture in a word: won, split, flagging, consents
    #[serde(default)]
    pub state: String,
    pub resolves: u32,
    /// A meeting is due on the cadence
    pub due: bool,
    /// Days between meetings, the organizer's recommendation
    pub cadence_days: u64,
}

/// The right-hand panel's player/org data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelV {
    pub name: String,
    pub gold: f32,
    pub org: String,
    /// Who heads it - the org's leader, by name; None before one's named
    #[serde(default)]
    pub leader: Option<String>,
    /// The powers' standing words on it: a lord's grant, knights in
    /// the streets, an assembly ban - empty when the state ignores it
    #[serde(default)]
    pub org_status: Vec<String>,
    /// The org's own muscle: capacity, the seat's turnout craft, and
    /// whether it's been used lately - None on an older wire
    #[serde(default)]
    pub power: Option<String>,
    pub members: Vec<String>,
    pub issues: Vec<String>,
    pub guilds: usize,
    /// Market servings (the HUD's "food")
    pub food: u32,
    /// The org's campaign, when it's founded one
    pub campaign: Option<CampaignV>,
    /// The odd job the seat is running, in a line ("take the note to Willa")
    pub quest: Option<String>,
    /// The village mystery the seat is piecing together, and how much
    /// of the gossip is in ("Mystery: ... - 3/5 whispers")
    #[serde(default)]
    pub mystery: Option<MysteryV>,
    /// Who the seat has gotten to know, by how far each bond runs
    #[serde(default)]
    pub social: SocialV,
    /// The house meeting being planned: who said yes to what, so the
    /// seat can see the plan fill before it calls the meeting
    pub meeting: Option<MeetingPlanV>,
}

/// The seat's bond ledger: folk they've talked to, counted by the stage
/// their relationship has reached - the reward for keeping company
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SocialV {
    /// Folk with any bond at all - someone the seat has spoken to
    pub met: usize,
    pub acquaintances: usize,
    pub known: usize,
    pub followers: usize,
    pub coleaders: usize,
}

/// The meeting plan's yeses, by name - who's said they'd come, who'll
/// bring friends, and the promises already filled ("house: Willa")
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeetingPlanV {
    pub coming: Vec<String>,
    pub spreaders: Vec<String>,
    /// Role promises as "role: name" lines (house, snacks, notes...)
    pub roles: Vec<String>,
}

/// Which of the action buttons is blocked and why
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub target: usize,
    pub picked: bool,
    pub building: Option<usize>,
}

/// One line on the TALK panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkRowV {
    /// Who said it - None for narration and the day rule
    pub name: Option<String>,
    pub text: String,
    /// A concern or secret drawn out: the line that matters
    pub gist: bool,
    /// Said by the player
    pub you: bool,
    /// A "day 12" divider between conversations
    pub rule: bool,
    /// The speaker as a villager index, when it's one of ours - a click on
    /// the line selects them (None: narration, the player, an outsider)
    #[serde(default)]
    pub who: Option<usize>,
}

/// The TALK panel: conversations, kept apart from the event log
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TalkPanelV {
    pub title: String,
    /// Oldest first; the panel shows the newest that fit
    pub rows: Vec<TalkRowV>,
    /// Replies on offer mid-Talk, in pick order
    pub options: Vec<String>,
    /// Mid-Talk: Leave is on offer too
    pub in_talk: bool,
}

/// The conversation box at the map's top-right: a Talk under way, a house
/// meeting's lines as they're said, or the call the meeting left the
/// organizer to make. Each line is its own button - a named one selects
/// the speaker - and the choices under it are clickable
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvoPanelV {
    pub title: String,
    /// Conversation lines, oldest first; the box shows the newest that fit
    pub rows: Vec<TalkRowV>,
    /// The choices on offer: Talk replies, or the founding call's framings
    pub options: Vec<String>,
    /// The options are the post-meeting call (Command::Decide), not Talk replies
    pub decides: bool,
    /// "Take your leave" is on offer (a Talk under way)
    pub leave: bool,
}

/// A big moment announced at the top of the map; `at` is where it
/// happened, when it happened somewhere - a click jumps the camera there
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastV {
    pub text: String,
    pub at: Option<(f32, f32)>,
    /// The model frame it was announced on, so the client can age it
    /// itself; absent from an older server reads as just arrived
    #[serde(default)]
    pub born: Option<u32>,
}

/// One event-log line plus where it happened / who it was about, when the
/// event had a position or a villager to point at (#645): a click jumps
/// the camera there and selects the villager. Older servers send bare
/// strings; those decode as plain, unclickable lines.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntryV {
    pub text: String,
    /// Where it happened, snapshotted at log time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<(f32, f32)>,
    /// The villager it was about (a villager index, like `Command::Select`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub who: Option<usize>,
}

impl<'de> Deserialize<'de> for LogEntryV {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Plain(String),
            Full {
                text: String,
                #[serde(default)] at: Option<(f32, f32)>,
                #[serde(default)] who: Option<usize>,
            },
        }
        Ok(match Repr::deserialize(d)? {
            Repr::Plain(text) => LogEntryV { text, at: None, who: None },
            Repr::Full { text, at, who } => LogEntryV { text, at, who },
        })
    }
}

/// The seat's mystery under way, structured so the panel can make the
/// asker's name a button: a click selects them and jumps there (#646).
/// Older servers send the whole line as a bare string; that decodes
/// with `title` empty and the line itself in `asker_name`, which the
/// client draws verbatim with nothing clickable.
#[derive(Debug, Clone, Serialize)]
pub struct MysteryV {
    pub title: String,
    pub heard: usize,
    pub total: usize,
    pub asker_name: String,
    /// The asker's villager index (like `Command::Select`), when they're
    /// still on the roll
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asker: Option<usize>,
    /// Where they stood this frame
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asker_at: Option<(f32, f32)>,
}

impl<'de> Deserialize<'de> for MysteryV {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Plain(String),
            Full {
                title: String,
                #[serde(default)] heard: usize,
                #[serde(default)] total: usize,
                asker_name: String,
                #[serde(default)] asker: Option<usize>,
                #[serde(default)] asker_at: Option<(f32, f32)>,
            },
        }
        Ok(match Repr::deserialize(d)? {
            Repr::Plain(line) => MysteryV { title: String::new(), heard: 0, total: 0,
                asker_name: line, asker: None, asker_at: None },
            Repr::Full { title, heard, total, asker_name, asker, asker_at } =>
                MysteryV { title, heard, total, asker_name, asker, asker_at },
        })
    }
}

/// Everything `View::render` draws that comes from the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModel {
    /// The viewing seat's character: facing (0 up, 1 right, 2 down, 3
    /// left) and whether it's mid-walk, for the walk cycle
    #[serde(default)]
    pub player_pose: (u8, bool),
    /// Villagers' (entity, facing, walking), for their walk cycles
    #[serde(default)]
    pub poses: Vec<(Entity, u8, bool)>,
    pub tick: u64,
    /// Animation frame count (mill sails, smoke, the courier's ride)
    pub frame: u32,
    /// Where the sim last asked the camera to look - (x, y, the frame it
    /// was asked on): an introduction's name, say. Older servers omit
    /// it; a client should ignore an absent field.
    #[serde(default)]
    pub focus: Option<(f32, f32, u32)>,
    pub map: TileMap,
    pub town_wall: (i32, i32, i32, i32),
    pub season: Season,
    /// How the season weighs on the fields (the climate's food
    /// multiplier) - 0 from an older server means no read
    #[serde(default)]
    pub food_yield: f32,
    /// Every other winter runs long: the snow keeps falling, the store
    /// thins - the banner says so when it's on
    #[serde(default)]
    pub harsh_winter: bool,
    pub precip: Option<PrecipKind>,
    /// Frame the last rain stopped (rainbow fades from there)
    pub rainbow_from: Option<u32>,
    pub paused: bool,
    /// Sim speed when local control is allowed (always 1 when networked)
    pub speed: u32,
    pub stones: Vec<(i32, i32)>,
    /// The standing monuments: where each stands, its kind (0 column,
    /// 1 the org's ring), and the words it carries
    pub monuments: Vec<(i32, i32, u8, String)>,
    pub buildings: Vec<BldgV>,
    /// Keep sprites at the centers of remote castle enclosures.
    pub castle_keeps: Vec<(f32, f32)>,
    /// House sprites in the remote town clusters; these are map scenery,
    /// not selectable/simulated buildings in the main village.
    pub settlement_houses: Vec<(f32, f32, BSprite)>,
    pub people: Vec<PersonV>,
    /// The seat's warmth for each figure it knows, as hearts - the map
    /// floats them over heads; (entity, hearts), only bonds with hearts
    /// to show, and absent from older servers
    #[serde(default)]
    pub bonds: Vec<(Entity, f32)>,
    /// The viewing seat's org roster - the map marks a member's name tag
    /// where their hearts show; absent from older servers
    #[serde(default)]
    pub org_members: Vec<Entity>,
    /// The seat's own bag - what's in the wanderer's satchel; absent
    /// from older servers
    #[serde(default)]
    pub player_satchel: Vec<(String, u32)>,
    /// The seat's belly, 0 to 100 - the strip marks it hungry under 40;
    /// absent from older servers, which read as a full belly
    #[serde(default)]
    pub player_hunger: f32,
    pub bubbles: Vec<BubbleV>,
    pub toasts: Vec<ToastV>,
    /// The event log, newest first after `visible_log`'s ordering; entries
    /// carrying `at`/`who` are clickable on the LOG panel (#645)
    #[serde(default)]
    pub log: Vec<LogEntryV>,
    /// How many log lines have already rolled out the top - the log's a
    /// rolling window; the view marks the drop with a count
    #[serde(default)]
    pub log_dropped: usize,
    /// The TALK panel, apart from the log
    #[serde(default)]
    pub talk: TalkPanelV,
    /// The conversation box, top-right of the map (a Talk or meeting under
    /// way, the call after one) - None most of the time
    #[serde(default)]
    pub convo: Option<ConvoPanelV>,
    pub sel: Selection,
    /// Villager indices within talk/meeting reach
    pub reach: Vec<usize>,
    /// Whom Talk/Recruit would go to now (a villager index)
    pub reach_target: usize,
    /// Disabled reason per action button
    pub blocked: [Option<String>; 3],
    pub ctx: Vec<CtxAct>,
    /// Disabled reason per ctx button, same index order as `ctx`
    pub ctx_blocked: Vec<Option<String>>,
    pub inspector: Option<InspectorV>,
    pub panel: PanelV,
    /// Mill stands at the map's mill tile when the art has a mill
    pub mill: Option<(f32, f32)>,
    /// Wild herb tufts that are ready to pick
    pub herbs: Vec<(f32, f32)>,
    /// Ready mushroom patches in the shade
    pub mush: Vec<(f32, f32)>,
    /// The skeps of the apiary, and a swarm out following someone
    pub skeps: Vec<(f32, f32)>,
    pub swarm: Option<(f32, f32)>,
    /// The market cross (where the notices go up), and whether today's a
    /// feast (the maypole stands there instead)
    #[serde(default)]
    pub market: Option<(f32, f32)>,
    /// Where the stocks stand by the market, once a sentence names their tile
    pub stocks: Option<(i32, i32)>,
    /// The village gaol's tile - the watch house's cellar, where the
    /// watch post stands; and the castle's own cell block
    pub village_gaol: Option<(i32, i32)>,
    pub castle_gaol: Option<(i32, i32)>,
    /// The water's cells: tile, depth band (1 shallow, 2 deep) - the
    /// client draws the overlay where the land lies under it
    pub flood: Vec<(i32, i32, u8)>,
    #[serde(default)]
    pub feast: bool,
    /// Stalls standing today: (x, y, look) - a seller's stall by look
    /// index, or STALL_TRADER for the pedlar's booth
    #[serde(default)]
    pub stalls: Vec<(f32, f32, u8)>,
    /// Where feet have worn the ground in (x, y, strength)
    pub trails: Vec<(f32, f32, f32)>,
    /// The land's score: stump tiles, the foul reach, the drowned pit
    pub stumps: Vec<(f32, f32)>,
    pub murk: Vec<(f32, f32)>,
    pub pond: Option<(f32, f32)>,
    /// What's alight on the map, and the ground it scorched
    pub burning: Vec<(f32, f32)>,
    pub scorched: Vec<(f32, f32)>,
    /// Kitchen-garden beds and how full they've grown (x, y, growth)
    pub gardens: Vec<(f32, f32, f32)>,
    /// Something riding over the walls tonight
    pub sky: Option<String>,
    /// The painted halls - color is clout that fades
    pub painted: Vec<(f32, f32)>,
    /// The cave mouth at the map's edge
    pub cave: Option<(f32, f32)>,
    /// The martyr's plot, when the gibbet made one
    pub grave: Option<(f32, f32)>,
    /// The churchyard's grave slots (tile coords) - the client draws
    /// worn ground under them
    #[serde(default)]
    pub graveyard: Vec<(i32, i32)>,
    /// The country's sites: name and kind for the minimap, the keeper
    /// when the keep's first bed holds one
    #[serde(default)]
    pub sites: Vec<SiteV>,
    /// The seat's public KPIs as lines, for a debug overlay. Never the
    /// secret gauges (`Game::secret_kpis`): this goes over the wire
    pub debug_lines: Vec<String>,
    /// The audio cues this frame raised, for the sound layer
    pub sfx: Vec<String>,
    /// Snowballs in the air this frame, where each has got to (x, y)
    #[serde(default)]
    pub snowballs: Vec<(f32, f32)>,
    /// The village's research tree: what's discovered, what's invested
    #[serde(default)]
    pub tech: Vec<crate::game::tech_tree::Tech>,
    /// Where the cause's flyers are posted: (x, y) tile centres
    #[serde(default)]
    pub flyers: Vec<(f32, f32)>,
    /// Freight on the roads: carts, tribute and gift trains - the wagon
    /// a carrier walks behind, as (x, y)
    #[serde(default)]
    pub trains: Vec<(f32, f32)>,
}
