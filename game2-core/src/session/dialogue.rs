//! Talk in the village, through the conversation framework
//! (game::conversation + the village_library trees), kept on the TALK
//! panel at the foot of the side panel - apart from the event LOG.
//!
//! - Villager to villager: when two villagers meet in passing they hold a
//!   whole conversation. It isn't floated over the map (a street full of
//!   bubbles was noise); it goes into the talk history, and clicking
//!   either of them shows it on the TALK panel.
//! - The player's Talk: the villager opens from the trees, says what's on
//!   their mind (the concern or secret Talk has always drawn out), and the
//!   player answers by picking one of two replies - or takes their leave.
//!   Agreeing warms them a little; pushing back cools them.
//! - House meetings: what the attendees say is recorded on the panel as
//!   it's said (the meeting's own talk comes from conversation::run_meeting).
//!
//! The speakers are read off the world: their Personality traits, their
//! trade, what's on them right now (sick, hungry, drunk, grieving, old,
//! a child), and the bond between the two.

use crate::engine::ecs::{Entity, World};
use crate::engine::render::Buffer;
use crate::game::conversation::{converse, converse_with, Act, Choice, DialogueTree, Ending, Relation, Rng as TalkRng, Speaker};
use crate::game::dialogue_trees::village_library;
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::relationships::{self, Interaction, InteractionKind, RelKind};
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Craft, Kid, TownRole, Vocation};
use crate::sim::psychology::{AttachmentHistory, EpisodeKind};
use crate::ui::font::draw_text;
use super::model::{TalkPanelV, TalkRowV};
use super::{Game, UI_X, GLYPH_W};

/// Conversations kept in the talk history (oldest drop first)
pub const TALK_LOG_MAX: usize = 240;
/// The longest villager-to-villager exchange recorded
pub const CHAT_MAX_LINES: usize = 6;
/// The children's tree: a pair of kids keeps to it
pub const KID_TREE: &str = "children_at_play";
/// Walk further than this (tiles) from whoever you're talking with and
/// the talk is over - you've left
pub const PLAYER_TALK_RANGE: f32 = 6.0;
/// A reply not given inside this many days and they've gone back to work
pub const PLAYER_TALK_DAYS: u64 = 8;
/// What a reply does to the bond: agreeing warms, pushing back cools
const AGREE_WARMTH: f32 = 0.3;
const PUSHBACK_CHILL: f32 = 0.2;

/// What kind of talk a record holds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TalkKind {
    /// Two villagers in passing
    Chat,
    /// The player's Talk with a villager
    Talk,
    /// A house meeting
    Meeting,
}

/// One line on the record. `who` None is narration ("You take your leave.")
#[derive(Debug, Clone)]
pub struct TalkLine {
    pub who: Option<Entity>,
    pub name: String,
    pub text: String,
    /// The line that matters: a concern or secret drawn out
    pub gist: bool,
}

/// One conversation, as the TALK panel shows it
#[derive(Debug, Clone)]
pub struct TalkRecord {
    pub id: u64,
    pub tick: u64,
    pub kind: TalkKind,
    /// Everyone in it (the player too, for a Talk or a meeting)
    pub with: Vec<Entity>,
    pub lines: Vec<TalkLine>,
}

/// A Talk the player is in the middle of: replayed from its seed and the
/// replies picked so far each time a reply is picked (the framework runs a
/// conversation start to end; replaying a seeded run up to the next
/// choice is how it's paced across frames)
#[derive(Debug, Clone)]
pub struct PlayerTalk {
    pub id: u64,
    pub who: Entity,
    pub villager: Speaker,
    pub player: Speaker,
    pub rel: Relation,
    pub seed: u64,
    pub picks: Vec<usize>,
    /// What's on their mind, said after their opening line
    pub gist: String,
    /// The replies on offer now: (act, the line)
    pub options: Vec<(Act, String)>,
    pub since: u64,
    /// Narration after the gist ("You talk the helplessness out of them.")
    pub notes: Vec<String>,
}

/// The village dialogue library, built once
fn library() -> &'static [DialogueTree] {
    static LIB: std::sync::OnceLock<Vec<DialogueTree>> = std::sync::OnceLock::new();
    LIB.get_or_init(village_library)
}

/// The fact vocabulary the trees speak for a trade
fn trade_status(v: Vocation) -> Option<&'static str> {
    Some(match v {
        Vocation::Toolmaker => "smith",
        Vocation::Mason => "mason",
        Vocation::Shepherd => "shepherd",
        Vocation::Beekeeper => "beekeeper",
        Vocation::Weaver => "weaver",
        Vocation::Fisher => "fisher",
        Vocation::Brewer | Vocation::Alewife => "brewer",
        Vocation::Baker => "baker",
        Vocation::Taverner => "taverner",
        Vocation::Cooper => "carpenter",
        _ => return None,
    })
}

/// `who` as a conversation speaker, read off the world at `tick`
pub fn speaker_of(world: &World, who: Entity, name: &str, tick: u64) -> Speaker {
    let mut s = Speaker::new(name);
    if let Some(p) = world.get_component::<Personality>(who) { s.traits = p.traits.clone(); }
    if world.get_component::<Kid>(who).is_some() { s.status.push("child"); return s; }
    let craft = world.get_component::<Craft>(who).map(|c| c.vocation);
    s.ideology = match craft {
        Some(Vocation::Priest) => "clergy",
        Some(Vocation::Knight) => "lord",
        Some(Vocation::Perfumer) => "merchant",
        _ if crate::game::livelihood::has_role(world, who, TownRole::Trader) => "merchant",
        _ => "laborer",
    };
    if let Some(st) = craft.and_then(trade_status) { s.status.push(st); }
    if crate::game::livelihood::has_role(world, who, TownRole::Herbologist) { s.status.push("herb_wife"); }
    if super::is_sick(world, who) { s.status.push("sick"); }
    if world.get_component::<Hunger>(who).map(|h| h.value < 30.0).unwrap_or(false) { s.status.push("hungry"); }
    if world.get_component::<villager::Drunk>(who).map(|d| tick < d.until).unwrap_or(false) { s.status.push("drunk"); }
    if world.get_component::<Age>(who).map(|a| a.stage == LifeStage::Elder).unwrap_or(false) { s.status.push("elder"); }
    let grieving = world.get_component::<AttachmentHistory>(who).map(|h| h.episodes.iter()
        .any(|e| matches!(e.kind, EpisodeKind::Loss) && villager::is_dead(world, e.with))).unwrap_or(false);
    if grieving { s.status.push("grieving"); }
    s
}

/// The bond between two villagers, in the conversation's terms. Two
/// villagers who've never spoken are still neighbours - RelKind::Stranger
/// only means no interactions yet, and the trees' stranger material is
/// for travellers on the road ("who art thou?"), not the miller's wife
pub fn relation_of(kind: Option<RelKind>) -> Relation {
    match relation_with_wanderer(kind) {
        Relation::Stranger => Relation::Acquaintance,
        r => r,
    }
}

/// The bond between a villager and the player, who really is the
/// stranger on the road until they've talked
pub fn relation_with_wanderer(kind: Option<RelKind>) -> Relation {
    match kind {
        Some(RelKind::Family | RelKind::Lover) => Relation::Kin,
        Some(RelKind::Friend) => Relation::Friend,
        Some(RelKind::Rival) => Relation::Rival,
        Some(RelKind::Acquaintance | RelKind::Mentor | RelKind::Apprentice) => Relation::Acquaintance,
        Some(RelKind::Stranger) | None => Relation::Stranger,
    }
}

/// Replay a Talk: the lines so far (speaker 0 the villager, 1 the
/// player), the replies on offer if it's the player's turn, and how it
/// ended if it has
fn replay(t: &PlayerTalk) -> (Vec<(usize, String)>, Option<Vec<(Act, String)>>, Option<Ending>) {
    let mut n = 0;
    let mut offered: Option<Vec<(Act, String)>> = None;
    let tr = converse_with(&t.villager, &t.player, t.rel, library(), 1, &mut |ch: &[Choice]| {
        if n < t.picks.len() {
            n += 1;
            Some(t.picks[n - 1].min(ch.len().saturating_sub(1)))
        } else {
            offered = Some(ch.iter().map(|c| (c.act, c.preview.clone())).collect());
            None
        }
    }, &mut TalkRng::new(t.seed));
    let lines = tr.turns.iter().map(|x| (x.speaker, x.text.clone())).collect();
    let ending = if offered.is_some() { None } else { tr.ending };
    (lines, offered, ending)
}

impl Game {
    fn push_talk(&mut self, kind: TalkKind, with: Vec<Entity>, lines: Vec<TalkLine>) -> u64 {
        self.talk_next_id += 1;
        let id = self.talk_next_id;
        self.talk_log.push(TalkRecord { id, tick: self.tick, kind, with, lines });
        if self.talk_log.len() > TALK_LOG_MAX {
            let extra = self.talk_log.len() - TALK_LOG_MAX;
            self.talk_log.drain(..extra);
        }
        id
    }

    fn talk_record_mut(&mut self, id: u64) -> Option<&mut TalkRecord> {
        self.talk_log.iter_mut().rev().find(|r| r.id == id)
    }

    /// Add a line to a conversation on the record (a meeting, as it's said)
    pub fn note_talk(&mut self, id: u64, who: Option<Entity>, text: &str) {
        let name = who.map(|w| self.speaker_name(w)).unwrap_or_default();
        if let Some(r) = self.talk_record_mut(id) {
            r.lines.push(TalkLine { who, name, text: text.to_string(), gist: false });
        }
    }

    /// Start a house meeting's record: everyone who came, and the host
    pub fn open_meeting_record(&mut self, host: Entity, came: &[Entity]) -> u64 {
        let mut with = vec![host];
        with.extend_from_slice(came);
        self.push_talk(TalkKind::Meeting, with, vec![])
    }

    /// A meeting already talked through (a campaign meeting): put the
    /// whole exchange on the record at once. (speaker, name, text); a None
    /// speaker is the room together - named, but nobody in particular
    pub fn record_meeting(&mut self, with: Vec<Entity>, said: Vec<(Option<Entity>, String, String)>) {
        if said.is_empty() { return; }
        let lines = said.into_iter().map(|(who, name, text)| TalkLine { who, name, text, gist: false }).collect();
        self.push_talk(TalkKind::Meeting, with, lines);
    }

    /// A campaign's talk - a meeting's deliberation, or the arc that
    /// follows a march, a strike or a townhall - goes on the TALK panel in
    /// the order it was said, and the exchange that settled it hangs over
    /// the two who said it. The other side speaks as "The hall", the room
    /// together as "The room".
    pub fn record_campaign_talk(&mut self, with: Vec<Entity>, d: &crate::game::campaign_dialogue::Deliberation) {
        use crate::game::campaign_dialogue::Voice;
        let said: Vec<(Option<Entity>, String, String)> = d.said.iter().map(|s| {
            let name = match (s.speaker, s.voice) {
                (Some(e), _) => self.speaker_name(e),
                (None, Voice::Opponent) => "The hall".to_string(),
                (None, _) => "The room".to_string(),
            };
            (s.speaker, name, s.text.clone())
        }).collect();
        self.record_meeting(with, said);
        for s in d.decisive() {
            if let Some(e) = s.speaker { self.say(e, &s.text, 4); }
        }
    }

    /// The name a line is kept under: a player's own name (each seat's
    /// panel shows its own lines as "You" - talk_panel_at), else the villager's
    fn speaker_name(&self, e: Entity) -> String {
        match self.world.get_component::<crate::game::player::PlayerCharacter>(e) {
            Some(p) if self.seats.iter().any(|s| s.player == e) => p.name.clone(),
            _ => self.name_of(e),
        }
    }

    /// Two villagers meet: they hold a conversation, which goes on the
    /// record (not over the map). Children talk among themselves - a
    /// child and a grown-up just nod. Returns whether they talked.
    pub fn villagers_chat(&mut self, a: Entity, b: Entity) -> bool {
        if a == b { return false; }
        let (na, nb) = (self.name_of(a), self.name_of(b));
        let sa = speaker_of(&self.world, a, &na, self.tick);
        let sb = speaker_of(&self.world, b, &nb, self.tick);
        let kids = (sa.status.contains(&"child"), sb.status.contains(&"child"));
        if kids.0 != kids.1 { return false; }
        let rel = relation_of(self.rel_kind(a, b));
        // seeded off the meeting itself, not the game's rng: chatter on the
        // street mustn't shift every other roll in the sim
        let seed = self.tick.wrapping_mul(0x9E37_79B9)
            ^ ((a as u64) << 32) ^ (b as u64) ^ self.last_chat.len() as u64;
        let tr = converse(&sa, &sb, rel, library(), &mut TalkRng::new(seed));
        let lines: Vec<TalkLine> = tr.turns.iter()
            // kids keep to the children's tree: the rest is grown-up talk
            .take_while(|t| !kids.0 || t.tree == KID_TREE)
            .take(CHAT_MAX_LINES)
            .map(|t| {
                let (who, name) = if t.speaker == 0 { (a, na.clone()) } else { (b, nb.clone()) };
                TalkLine { who: Some(who), name, text: t.text.clone(), gist: false }
            }).collect();
        if lines.is_empty() { return false; }
        self.push_talk(TalkKind::Chat, vec![a, b], lines);
        true
    }

    /// The player has just talked with `v` (Talk's own effects are done):
    /// open the conversation on the TALK panel - their opening line, then
    /// `gist` (what Talk drew out of them), then the player's replies.
    /// `rel` is the bond as it stood before this talk.
    pub fn open_player_talk(&mut self, seat: usize, v: Entity, rel: Relation, gist: &str) {
        let pe = self.seats[seat].player;
        let name = self.name_of(v);
        let villager = speaker_of(&self.world, v, &name, self.tick);
        let pname = self.world.get_component::<crate::game::player::PlayerCharacter>(pe)
            .map(|p| p.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "friend".into());
        // spoken to, "The Wanderer" is "wanderer": "A peaceful day to thee, wanderer."
        let pname = match pname.strip_prefix("The ") { Some(rest) => rest.to_lowercase(), None => pname };
        let player = Speaker::new(&pname);
        let seed = self.tick.wrapping_mul(0x2545_F491) ^ ((v as u64) << 24) ^ self.talk_next_id;
        self.close_player_talk(seat, None);
        let id = self.push_talk(TalkKind::Talk, vec![pe, v], vec![]);
        self.seats[seat].talk = Some(PlayerTalk {
            id, who: v, villager, player, rel, seed, picks: vec![], gist: gist.to_string(),
            options: vec![], since: self.tick, notes: vec![],
        });
        self.refresh_player_talk(seat);
    }

    /// Re-run the seat's Talk up to now and put it on the record; once
    /// it's over (a farewell, a walk-off, nothing left to say) close it
    fn refresh_player_talk(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].talk.clone() else { return };
        let pe = self.seats[seat].player;
        let (said, offered, ending) = replay(&t);
        let vname = t.villager.name.clone();
        let mut lines: Vec<TalkLine> = vec![];
        for (k, (sp, text)) in said.iter().enumerate() {
            let (who, name) = if *sp == 0 { (t.who, vname.clone()) } else { (pe, self.speaker_name(pe)) };
            lines.push(TalkLine { who: Some(who), name, text: text.clone(), gist: false });
            if k == 0 { lines.extend(Self::gist_lines(&t, &vname)); }
        }
        if said.is_empty() { lines.extend(Self::gist_lines(&t, &vname)); }
        if let Some(r) = self.talk_record_mut(t.id) { r.lines = lines; }
        match offered {
            Some(opts) if !opts.is_empty() => {
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.options = opts; }
            }
            _ => {
                let note = match ending {
                    Some(Ending::WalksOff) => Some(format!("{} turns and walks off.", vname)),
                    _ => None,
                };
                self.close_player_talk(seat, note.as_deref());
            }
        }
    }

    /// A note on the seat's Talk record ("You talk the helplessness out of
    /// them."), kept through later replays
    pub fn narrate_talk(&mut self, seat: usize, text: &str) {
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.notes.push(text.to_string());
        self.refresh_player_talk(seat);
    }

    /// What Talk drew out of them, and any narration that came of it
    fn gist_lines(t: &PlayerTalk, vname: &str) -> Vec<TalkLine> {
        let mut v = vec![];
        if !t.gist.is_empty() {
            v.push(TalkLine { who: Some(t.who), name: vname.to_string(), text: t.gist.clone(), gist: true });
        }
        v.extend(t.notes.iter().map(|n| TalkLine { who: None, name: String::new(), text: n.clone(), gist: false }));
        v
    }

    /// End the seat's Talk, with a closing note on the record if given
    pub fn close_player_talk(&mut self, seat: usize, note: Option<&str>) {
        let Some(t) = self.seats[seat].talk.take() else { return };
        if let Some(n) = note {
            if let Some(r) = self.talk_record_mut(t.id) {
                r.lines.push(TalkLine { who: None, name: String::new(), text: n.to_string(), gist: false });
            }
        }
    }

    /// The player picks reply `pick` of those on offer, or None to take
    /// their leave
    pub fn talk_pick_at(&mut self, seat: usize, pick: Option<usize>) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let (who, n) = (t.who, t.options.len());
        let Some(i) = pick.filter(|&i| i < n) else {
            if pick.is_none() { self.close_player_talk(seat, Some("You take your leave.")); }
            return;
        };
        let act = t.options[i].0;
        let pe = self.seats[seat].player;
        // how you answer them counts for a little either way
        match act {
            Act::PushBack => { relationships::record_interaction(&mut self.world, pe,
                Interaction { other: who, kind: InteractionKind::Conflict, intensity: PUSHBACK_CHILL }); }
            Act::Agree | Act::Answer => { relationships::record_interaction(&mut self.world, pe,
                Interaction { other: who, kind: InteractionKind::Conversation, intensity: AGREE_WARMTH }); }
            _ => {}
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.picks.push(i); cur.since = self.tick; }
        self.refresh_player_talk(seat);
    }

    /// Each frame: a Talk left hanging ends once the player walks away,
    /// they're taken off the map, or too long passes without a reply
    pub fn advance_player_talks(&mut self) {
        for seat in 0..self.seats.len() {
            let Some(t) = self.seats[seat].talk.as_ref() else { continue };
            let who = t.who;
            let stale = self.tick > t.since + PLAYER_TALK_DAYS;
            let gone = villager::is_dead(&self.world, who) || crate::game::dungeon::unavailable(&self.world, who);
            let far = match (self.world.get_component::<villager::Pos>(who), self.world.get_component::<villager::Pos>(self.seats[seat].player)) {
                (Some(p), Some(q)) => (p.x - q.x).powi(2) + (p.y - q.y).powi(2) > PLAYER_TALK_RANGE * PLAYER_TALK_RANGE,
                _ => true,
            };
            if gone { self.close_player_talk(seat, None); }
            else if far { self.close_player_talk(seat, Some("You walk on.")); }
            else if stale { let n = format!("{} goes back to their work.", self.name_of(who)); self.close_player_talk(seat, Some(&n)); }
        }
    }

    /// The TALK panel for a seat: the picked villager's conversations
    /// (with you or anyone), or else your own - Talk and meetings - with
    /// the replies on offer if you're mid-Talk
    pub fn talk_panel_at(&self, seat: usize) -> TalkPanelV {
        let s = &self.seats[seat];
        let picked = (s.picked && s.selected_building.is_none())
            .then(|| self.villagers.get(s.selected_target).copied()).flatten();
        let focus = picked.unwrap_or(s.player);
        let title = match picked { Some(v) => format!("TALK - {}", self.name_of(v)), None => "TALK".to_string() };
        let mut rows: Vec<TalkRowV> = vec![];
        let mut shown: Vec<u64> = vec![];
        // newest conversations last, as many as could ever fit
        for r in self.talk_log.iter().rev().filter(|r| r.with.contains(&focus) && !r.lines.is_empty()).take(4) {
            shown.push(r.id);
            let mut block: Vec<TalkRowV> = vec![TalkRowV { name: None, text: format!("day {}", r.tick), gist: false, you: false, rule: true }];
            block.extend(r.lines.iter().map(|l| TalkRowV {
                // your own lines read "You"; another seat's keep their name
                name: if l.who == Some(s.player) { Some("You".into()) } else { (!l.name.is_empty()).then(|| l.name.clone()) },
                text: l.text.clone(), gist: l.gist,
                you: l.who.map(|w| w == s.player).unwrap_or(false), rule: false,
            }));
            block.extend(rows);
            rows = block;
        }
        let in_talk = matches!(&s.talk, Some(t) if shown.contains(&t.id));
        let options = match &s.talk {
            Some(t) if in_talk => t.options.iter().map(|o| o.1.clone()).collect(),
            _ => vec![],
        };
        let empty = if picked.is_some() { "Nothing heard from them yet." } else { "Talk to someone (T) and it shows here." };
        if rows.is_empty() { rows.push(TalkRowV { name: None, text: empty.into(), gist: false, you: false, rule: false }); }
        TalkPanelV { title, rows, options, in_talk }
    }
}

// ---------- the TALK panel ----------

/// The TALK box: under the minimap, the side panel's foot
pub const TALK_X: i32 = UI_X as i32 + 10;
pub const TALK_Y: i32 = 826;
pub const TALK_W: i32 = 320;
pub const TALK_H: i32 = 130;
const ROW_H: i32 = 10;
/// Characters to a row: the box less its padding
pub const TALK_CHARS: usize = ((TALK_W - 12) / GLYPH_W) as usize;
/// Rows the box holds under its title
const TALK_ROWS: usize = ((TALK_H - 18) / ROW_H) as usize;
/// An option takes at most this many rows; a longer one is cut with "..."
const OPTION_ROWS: usize = 2;

/// Where each clickable reply sits: (x, y, w, h, the pick - None is Leave)
pub type OptionRect = (i32, i32, i32, i32, Option<usize>);
/// A drawn row: (y, text, colour, the speaker's name leading it in its colour)
pub type TalkRow = (i32, String, (u8, u8, u8), Option<(String, (u8, u8, u8))>);

/// Word-wrap a reply as "1) text", at most OPTION_ROWS rows
fn option_rows(n: usize, text: &str) -> Vec<String> {
    let mut rows = super::wrap_text(&format!("{}) {}", n + 1, text), TALK_CHARS - 1);
    if rows.len() > OPTION_ROWS {
        rows.truncate(OPTION_ROWS);
        let last = rows.last_mut().unwrap();
        while last.len() + 3 > TALK_CHARS - 1 { last.pop(); }
        last.push_str("...");
    }
    rows
}

/// The rows the box draws, bottom-anchored like a chat - the newest line
/// sits just above the replies - and where the replies are, for clicks
pub fn talk_layout(p: &TalkPanelV) -> (Vec<TalkRow>, Vec<OptionRect>) {
    let mut opt_rows: Vec<(String, Option<usize>)> = vec![];
    for (i, o) in p.options.iter().enumerate() {
        for r in option_rows(i, o) { opt_rows.push((r, Some(i))); }
    }
    if p.in_talk { opt_rows.push(("x) Take your leave".into(), None)); }
    let room = TALK_ROWS.saturating_sub(opt_rows.len());
    // each line wrapped; the speaker's name leads the first row in its colour
    let mut lines: Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>)> = vec![];
    // which rows start a line: the view never opens on a line's tail
    let mut starts: Vec<bool> = vec![];
    for r in &p.rows {
        let before = lines.len();
        if r.rule { lines.push((r.text.clone(), (110, 110, 135), None)); starts.push(true); continue; }
        let body = if r.gist { (255, 215, 120) } else if r.name.is_none() { (150, 150, 170) } else { (215, 215, 200) };
        let ncol = if r.you { (255, 255, 120) } else { (150, 200, 255) };
        match &r.name {
            Some(n) => {
                let lead = format!("{}: ", n);
                for (k, w) in super::wrap_text(&format!("{}{}", lead, r.text), TALK_CHARS).into_iter().enumerate() {
                    match w.strip_prefix(lead.trim_end()) {
                        Some(rest) if k == 0 => lines.push((rest.trim_start().to_string(), body, Some((lead.clone(), ncol)))),
                        _ => lines.push((w, body, None)),
                    }
                }
            }
            None => for w in super::wrap_text(&r.text, TALK_CHARS) { lines.push((w, body, None)); },
        }
        starts.extend((before..lines.len()).map(|k| k == before));
    }
    let mut skip = lines.len().saturating_sub(room);
    while skip < lines.len() && !starts[skip] { skip += 1; }
    let top = TALK_Y + 16;
    let mut out: Vec<TalkRow> = vec![];
    for (k, (text, rgb, name)) in lines.into_iter().skip(skip).enumerate() {
        out.push((top + k as i32 * ROW_H, text, rgb, name));
    }
    // the replies sit right under the last line said
    let opt_top = top + out.len() as i32 * ROW_H + 2;
    let mut rects: Vec<OptionRect> = vec![];
    for (k, (text, pick)) in opt_rows.into_iter().enumerate() {
        let y = opt_top + k as i32 * ROW_H;
        out.push((y, text, if pick.is_some() { (140, 230, 140) } else { (200, 150, 130) }, None));
        // a reply's rows click as one
        match rects.last_mut() {
            Some(r) if r.4 == pick && pick.is_some() => r.3 += ROW_H,
            _ => rects.push((TALK_X + 2, y - 2, TALK_W - 4, ROW_H, pick)),
        }
    }
    (out, rects)
}

/// Draw the TALK box; a reply under the mouse lights up
pub fn draw_talk_panel(buf: &mut Buffer, p: &TalkPanelV, mouse: (i32, i32)) {
    buf.rect(TALK_X, TALK_Y, TALK_W, TALK_H, 25, 25, 40);
    draw_text(buf, TALK_X + 4, TALK_Y + 4, &p.title, 150, 150, 180);
    let (rows, rects) = talk_layout(p);
    for &(x, y, w, h, _) in &rects {
        if mouse.0 >= x && mouse.0 < x + w && mouse.1 >= y && mouse.1 < y + h {
            buf.rect(x, y, w, h, 45, 60, 50);
        }
    }
    for (y, text, (r, g, b), name) in rows {
        let mut x = TALK_X + 6;
        if let Some((n, (nr, ng, nb))) = name { x += draw_text(buf, x, y, &n, nr, ng, nb); } // draw_text returns the width
        draw_text(buf, x, y, &text, r, g, b);
    }
}

/// Which reply (if any) a click at (mx, my) on the TALK box picks:
/// Some(Some(i)) a reply, Some(None) Leave, None not a reply
pub fn talk_click(p: &TalkPanelV, mx: i32, my: i32) -> Option<Option<usize>> {
    talk_layout(p).1.into_iter()
        .find(|&(x, y, w, h, _)| mx >= x && mx < x + w && my >= y && my < y + h)
        .map(|r| r.4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::traits::Trait;
    use crate::game::villager::Pos;

    /// Living grown-up villager indices
    fn adults(g: &Game) -> Vec<usize> {
        (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
            && !villager::is_dead(&g.world, g.villagers[i])).collect()
    }

    #[test]
    fn test_speaker_reads_traits_trade_and_circumstance_off_the_world() {
        let mut g = Game::new();
        let v = g.villagers[adults(&g)[0]];
        g.world.add_component(v, Personality { traits: vec![Trait::Hotheaded] });
        g.world.add_component(v, crate::game::vocation::craft(Vocation::Weaver, 0.5));
        g.world.add_component(v, Hunger { value: 5.0 });
        let s = speaker_of(&g.world, v, "Eliza", g.tick);
        assert_eq!(s.traits, vec![Trait::Hotheaded]);
        assert!(s.status.contains(&"weaver") && s.status.contains(&"hungry"), "{:?}", s.status);
        g.world.add_component(v, crate::game::vocation::craft(Vocation::Priest, 0.5));
        assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).ideology, "clergy");
        g.world.add_component(v, Kid);
        assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).status, vec!["child"]);
    }

    #[test]
    fn test_neighbours_are_acquaintances_but_the_wanderer_is_a_stranger() {
        assert_eq!(relation_of(Some(RelKind::Family)), Relation::Kin);
        assert_eq!(relation_of(Some(RelKind::Rival)), Relation::Rival);
        assert_eq!(relation_of(Some(RelKind::Friend)), Relation::Friend);
        assert_eq!(relation_of(None), Relation::Acquaintance);
        assert_eq!(relation_of(Some(RelKind::Stranger)), Relation::Acquaintance);
        assert_eq!(relation_with_wanderer(None), Relation::Stranger);
    }

    #[test]
    fn test_villagers_chat_goes_on_the_record_not_over_the_map() {
        let mut g = Game::new();
        let ad = adults(&g);
        let (a, b) = (g.villagers[ad[0]], g.villagers[ad[1]]);
        g.bubbles.clear();
        assert!(g.villagers_chat(a, b));
        assert!(!g.bubbles.iter().any(|x| x.0 == a || x.0 == b), "no bubbles for villager chatter");
        let r = g.talk_log.last().unwrap().clone();
        assert_eq!(r.kind, TalkKind::Chat);
        assert!(r.lines.len() >= 2 && r.lines.iter().any(|l| l.who == Some(b)), "both speak: {:?}", r.lines);
        // clicking either of them shows it
        g.seats[0].selected_target = ad[1];
        g.seats[0].picked = true;
        let p = g.talk_panel_at(0);
        assert!(p.title.contains(&g.name_of(b)), "{}", p.title);
        assert!(p.rows.iter().any(|row| row.text == r.lines[0].text));
    }

    #[test]
    fn test_kids_keep_to_childrens_talk() {
        let mut g = Game::new();
        let ad = adults(&g);
        let (a, b, c) = (g.villagers[ad[0]], g.villagers[ad[1]], g.villagers[ad[2]]);
        g.world.add_component(a, Kid);
        g.world.add_component(b, Kid);
        assert!(!g.villagers_chat(a, c), "a child and a grown-up just nod");
        assert!(g.villagers_chat(a, b));
        let said: Vec<String> = g.talk_log.last().unwrap().lines.iter().map(|l| l.text.clone()).collect();
        let kids_lines: Vec<&str> = ["Oi, ", "Hast thou any errands", "Psst"].to_vec();
        assert!(kids_lines.iter().any(|k| said[0].contains(k)), "opens on the children's tree: {:?}", said);
    }

    /// Stand the player beside villager index `i` and Talk
    fn talk_beside(g: &mut Game, i: usize) -> Entity {
        let v = g.villagers[i];
        let p = *g.world.get_component::<Pos>(v).unwrap();
        *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
        g.seats[0].selected_target = i;
        g.seats[0].picked = true;
        g.act_at(0, 0);
        v
    }

    #[test]
    fn test_talk_opens_a_conversation_on_the_talk_panel_with_replies() {
        let mut g = Game::new();
        let i = adults(&g)[0];
        let log_before = g.log.len();
        let v = talk_beside(&mut g, i);
        let rec = g.talk_log.last().unwrap().clone();
        assert_eq!(rec.kind, TalkKind::Talk);
        assert!(rec.lines.iter().any(|l| l.gist && l.who == Some(v)), "what's on their mind is on the record");
        assert!(!g.log[log_before..].iter().any(|l| rec.lines.iter().any(|r| r.gist && l.contains(&r.text))),
            "the words aren't in the event log");
        let t = g.seats[0].talk.as_ref().expect("a talk under way").clone();
        assert!(!t.options.is_empty() && t.options.len() <= 2);
        let panel = g.talk_panel_at(0);
        assert!(panel.in_talk && panel.options.len() == t.options.len());
        // pick the first reply: it's said, as previewed
        g.talk_pick_at(0, Some(0));
        let rec = g.talk_log.iter().find(|r| r.id == rec.id).unwrap();
        assert!(rec.lines.iter().any(|l| l.who == Some(g.player) && l.text == t.options[0].1), "{:?}", rec.lines);
    }

    #[test]
    fn test_leaving_or_walking_off_ends_the_talk() {
        let mut g = Game::new();
        let ad = adults(&g);
        talk_beside(&mut g, ad[0]);
        assert!(g.seats[0].talk.is_some());
        g.talk_pick_at(0, None);
        assert!(g.seats[0].talk.is_none());
        assert_eq!(g.talk_log.last().unwrap().lines.last().unwrap().text, "You take your leave.");
        let v = talk_beside(&mut g, ad[1]);
        assert!(g.seats[0].talk.is_some());
        let p = *g.world.get_component::<Pos>(v).unwrap();
        *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + PLAYER_TALK_RANGE + 2.0, y: p.y };
        g.advance_player_talks();
        assert!(g.seats[0].talk.is_none(), "walking away ends it");
    }

    #[test]
    fn test_a_whole_talk_runs_to_its_end_by_picking_replies() {
        let mut g = Game::new();
        let i = adults(&g)[0];
        talk_beside(&mut g, i);
        let mut picks = 0;
        while g.seats[0].talk.is_some() && picks < 30 { g.talk_pick_at(0, Some(0)); picks += 1; }
        assert!(g.seats[0].talk.is_none(), "it ends: farewell, walk-off or out of things to say");
        assert!(picks >= 1);
    }

    #[test]
    fn test_long_lines_wrap_inside_the_talk_box_and_replies_click() {
        let long = "word ".repeat(40);
        let p = TalkPanelV {
            title: "TALK".into(),
            rows: (0..6).map(|k| TalkRowV { name: Some(format!("Speaker{}", k)), text: long.clone(), gist: k == 2, you: k % 2 == 1, rule: false }).collect(),
            options: vec![long.clone(), "Short one.".into()],
            in_talk: true,
        };
        let (rows, rects) = talk_layout(&p);
        for (y, text, _, name) in &rows {
            let n = name.as_ref().map(|n| n.0.len()).unwrap_or(0);
            assert!(n + text.len() <= TALK_CHARS, "row overflows: {:?}", text);
            assert!(*y >= TALK_Y + 14 && *y + 7 <= TALK_Y + TALK_H, "row at {} outside the box", y);
        }
        assert!(rows[0].3.is_some(), "the view opens on a speaker's line, not the tail of one");
        assert_eq!(rects.len(), 3, "two replies and Leave");
        let (x, y, w, h, _) = rects[1];
        assert_eq!(talk_click(&p, x + w / 2, y + h / 2), Some(Some(1)));
        let (x, y, _, _, _) = rects[2];
        assert_eq!(talk_click(&p, x + 1, y + 1), Some(None));
        assert_eq!(talk_click(&p, TALK_X - 5, y), None);
    }

    #[test]
    fn test_a_house_meeting_is_kept_on_the_talk_panel() {
        let mut g = Game::new();
        *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: 16.0, y: 16.0 };
        let (eliza, bram) = (g.villagers[1], g.villagers[2]);
        for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
        for (i, v) in g.villagers.clone().into_iter().enumerate() {
            if i != 1 && i != 2 { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
        }
        g.act(3);
        for _ in 0..super::super::FPS * 60 { if g.meeting.is_none() { break; } g.update(); }
        assert!(g.meeting.is_none(), "the meeting ran its course");
        let r = g.talk_log.iter().rev().find(|r| r.kind == TalkKind::Meeting).expect("a meeting on the record");
        assert!(r.with.contains(&g.player));
        assert!(r.lines.iter().any(|l| l.who == Some(g.player)) && r.lines.iter().any(|l| l.who == Some(eliza) || l.who == Some(bram)),
            "{:?}", r.lines);
        // the room's talk, not just the stock remarks
        assert!(r.lines.len() >= 5);
    }

    #[test]
    fn test_each_seat_sees_its_own_lines_as_you_and_hosts_its_own_meeting() {
        let mut g = Game::new();
        let s1 = g.add_seat();
        let (p0, p1) = (g.seats[0].player, g.seats[s1].player);
        let v = g.villagers[adults(&g)[0]];
        // seat 1 hosts: the prompts are theirs, not seat 0's
        let lines = g.meeting_lines(&[v], s1);
        assert!(lines.iter().any(|l| l.0 == p1) && !lines.iter().any(|l| l.0 == p0), "seat 1 asks the questions");
        let n1 = g.speaker_name(p1);
        g.record_meeting(vec![p1, p0, v], vec![(Some(p1), n1.clone(), "Thanks for coming.".into())]);
        let row = |g: &Game, seat: usize| g.talk_panel_at(seat).rows.into_iter()
            .find(|r| r.text == "Thanks for coming.").and_then(|r| r.name).unwrap();
        assert_eq!(row(&g, s1), "You", "the host's own panel");
        assert_eq!(row(&g, 0), n1, "the other seat sees who said it");
        assert_ne!(n1, "You");
    }

    #[test]
    fn test_villagers_talk_in_a_running_game() {
        let mut g = Game::new();
        for _ in 0..super::super::FPS * 60 * 2 { g.update(); }
        let chats = g.talk_log.iter().filter(|r| r.kind == TalkKind::Chat).count();
        assert!(chats > 0, "villagers who meet should talk");
    }
}
