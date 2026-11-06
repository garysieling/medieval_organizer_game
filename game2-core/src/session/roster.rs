//! The starting roster: the title screen's choice of who you are. A seed
//! deals a handful of premade wanderers - trades, children, misfits - and
//! picking one rewrites the seat's character to match. `roster` is a pure
//! function of the world seed, so a server and every client deal the same
//! hand without asking.

use crate::game::desires::{self, Want};
use crate::game::livelihood;
use crate::game::vocation::{self, SocialClass, Vocation};

use super::{roll_vocation, Game, Rng, NEW_NAMES};

/// How many wanderers a seed deals
const ROSTER_SIZE: usize = 7;

/// One premade character, ready to become a seat's player
#[derive(Debug, Clone)]
pub struct CharacterSheet {
    pub name: String,
    /// A line or two of who they are, for the picker card
    pub backstory: String,
    /// The short epithet under the name: a trade, or "a child"
    pub work: String,
    pub kid: bool,
    /// Which of the pack's figure looks (children only ever wear the
    /// two youngest - the view mods it through `figure_look`)
    pub look: usize,
    /// The trade, for a grown-up; None for a child
    pub vocation: Option<Vocation>,
    pub class: SocialClass,
    pub skill: f32,
    pub gold: f32,
    pub wants: Vec<Want>,
}

/// What a picker card needs - everything but the sim parts. This is what
/// rides in the Welcome, so a remote client can show the server's roster
/// before it has picked anyone.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RosterCard {
    pub name: String,
    pub backstory: String,
    pub work: String,
    pub kid: bool,
    pub look: usize,
}

impl From<&CharacterSheet> for RosterCard {
    fn from(s: &CharacterSheet) -> Self {
        RosterCard { name: s.name.clone(), backstory: s.backstory.clone(),
            work: s.work.clone(), kid: s.kid, look: s.look }
    }
}

/// The openings a backstory can start from
const ORIGINS: [&str; 8] = [
    "Born to {trade} folk two villages over.",
    "Came down from the hills with the spring flocks.",
    "Shipwrecked off the coast and stayed for the bread.",
    "Sold into service young; ran the first winter.",
    "A pardoner's grandson who took up letters, not sins.",
    "Found at the church door one Martinmas and taken in.",
    "The miller's daughter, quicker with figures than grain.",
    "A soldier's orphan, mustered out with a sword and nothing else.",
];

/// The second line: what they're like
const QUIRKS: [&str; 8] = [
    "Talks to horses.",
    "Counts everything twice.",
    "Sings to the chickens.",
    "Hums when nervous.",
    "Never sits with their back to a door.",
    "Writes letters to people who moved away.",
    "Whistles through the left nostril only.",
    "Collects buttons.",
];

/// Childhoods, for the young hands on the roster
const CHILDHOODS: [&str; 8] = [
    "Orphaned young, raised by an aunt who meant well.",
    "The youngest of seven; nobody remembers their name right.",
    "Found wandering the fair and claimed by the sexton.",
    "A twin - the other one didn't come home, they say.",
    "Never stayed anywhere longer than a season.",
    "Talks to animals more than people.",
    "Once stole a pie and never confessed.",
    "The only child their grandmother ever kept.",
];

/// The roster `seed` deals: adults and children, each with a trade (or a
/// childhood), a look, a purse and an inner life. Deterministic - the
/// same seed deals the same hand anywhere.
pub fn roster(seed: u64) -> Vec<CharacterSheet> {
    let mut rng = Rng(seed ^ 0x5EED_5EED_5EED_5EED);
    // names without repeats: deal off NEW_NAMES (the villagers' own
    // names stay theirs)
    let mut names: Vec<&'static str> = NEW_NAMES.to_vec();
    let mut out = vec![];
    for i in 0..ROSTER_SIZE {
        let ni = rng.below(names.len() as i32) as usize;
        let name = names.swap_remove(ni).to_string();
        // children turn up on about a third of the cards; the first card
        // is always a grown-up and the last always a child, so the roster
        // always offers both
        let kid = i == ROSTER_SIZE - 1 || (i != 0 && rng.below(10) < 3);
        if kid {
            let backstory = format!("{} {}", CHILDHOODS[rng.below(CHILDHOODS.len() as i32) as usize],
                QUIRKS[rng.below(QUIRKS.len() as i32) as usize]);
            let roll = (rng.next() % 1000) as f32 / 1000.0;
            let class = SocialClass::Working;
            out.push(CharacterSheet {
                name, backstory,
                work: "a child".into(), kid: true,
                look: rng.below(2) as usize,
                vocation: None, class, skill: 0.0,
                gold: livelihood::seed_gold(class, true, roll),
                wants: desires::sample_wants(class, None, true, &mut || rng.next()),
            });
        } else {
            let voc = roll_vocation(&mut rng);
            // a wanderer can't own a workshop, so petit bourgeois is a
            // class the road never deals - `class_of` would read them
            // Working anyway; masters are for those who keep the bench
            let class = match rng.below(10) { 0..=8 => SocialClass::Working, _ => SocialClass::ProfessionalMiddle };
            let voc = if class == SocialClass::ProfessionalMiddle && !vocation::is_pmc(voc) {
                if rng.below(2) == 0 { Vocation::Knight } else { Vocation::Priest }
            } else { voc };
            let skill = 0.1 + rng.below(20) as f32 / 100.0;
            let trade = vocation::vocation_name(voc).to_lowercase();
            let backstory = format!("{} {}", ORIGINS[rng.below(ORIGINS.len() as i32) as usize].replace("{trade}", &trade),
                QUIRKS[rng.below(QUIRKS.len() as i32) as usize]);
            let roll = (rng.next() % 1000) as f32 / 1000.0;
            out.push(CharacterSheet {
                name, backstory,
                work: vocation::vocation_name(voc).into(), kid: false,
                look: rng.below(8) as usize,
                vocation: Some(voc), class, skill,
                gold: livelihood::seed_gold(class, false, roll),
                wants: desires::sample_wants(class, Some(voc), false, &mut || rng.next()),
            });
        }
    }
    out
}

impl Game {
    /// Make `seat`'s player the `idx`-th wanderer the seed dealt: name,
    /// trade or childhood, purse and inner life all rewritten to the
    /// sheet. Returns false for a seat or card that isn't there. The
    /// world seed must be set - a journaled replay rebuilds it the same.
    pub fn pick_character_at(&mut self, seat: usize, idx: usize) -> bool {
        let Some(sheet) = roster(self.seed).get(idx).cloned() else { return false };
        // only a wanderer seat has a player to rewrite - an agent's seat
        // is a villager, and picking would stamp a stranger's parts on it
        let Some(seat_ref) = self.seats.get(seat) else { return false };
        if seat_ref.agent { return false; }
        let player = seat_ref.player;
        // one pick per seat: a second would mint a fresh purse out of
        // thin air, gold spent or no
        if self.world.get_component::<crate::game::player::Avatar>(player).is_some() { return false; }
        if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(player) {
            p.name = sheet.name.clone();
        }
        // a re-pick swaps trades for childhoods cleanly
        self.world.remove_component::<vocation::Craft>(player);
        self.world.remove_component::<vocation::Kid>(player);
        self.world.remove_component::<vocation::KidGrowth>(player);
        if sheet.kid {
            self.world.add_component(player, vocation::Kid);
            self.world.add_component(player, vocation::KidGrowth { age_days: 0, errands: vec![] });
        } else if let Some(voc) = sheet.vocation {
            self.world.add_component(player, vocation::craft(voc, sheet.skill));
        }
        self.world.add_component(player, livelihood::Purse { gold: sheet.gold });
        if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(player) {
            p.resources = sheet.gold;
        }
        self.world.add_component(player, desires::InnerLife {
            wants: sheet.wants.clone(), grievances: vec![], satisfied: vec![] });
        self.world.add_component(player, crate::game::player::Avatar {
            look: sheet.look, kid: sheet.kid });
        true
    }
}
