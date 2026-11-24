//! The odd jobs villagers ask the Wanderer to run: a note to carry, a
//! parcel out to a far village, an egg or a stone to fetch back. Offers
//! come up at dawn for folk the player has gotten to know; the job is
//! taken from the picked villager's contextual button, the thing changes
//! hands on its own once you're standing on it, and it's handed over by
//! talking to whoever it's for.

use crate::engine::ecs::Entity;
use crate::game::livelihood;
use crate::session::*;

/// What the job is about - the plain kinds beside E5-7's five shapes
/// (the ghosted ask gets the wound's own errand)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestKind {
    Note, Parcel, Egg, Stone, Talk,
    /// A thing laid somewhere for them - the loaf to the cairn
    Deliver,
    /// A question put to who knows - was the debt ever written down?
    Ask,
    /// A sight to be seen - the manor roll itself
    FindOut,
    /// Word carried to the one they're estranged from
    CarryWord,
    /// A thing held a few days - it can't be found on them
    Keep,
}

/// Where the thing ends up
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuestTarget {
    Villager(Entity), Site(usize),
    /// A named spot that isn't a settlement - the old stones, the cairn
    Spot(f32, f32),
}

/// What an errand proves and when it's due (E5-7): Ask and Find out
/// carry the tag of the ghost-fact their answer confirms (the debt in
/// the manor roll); a Keep holds `due` - the day the bundle goes back
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Errand {
    /// the ghost's tag (`miller_debt`) - the errand's answer writes
    /// `ghost:<tag>` into the seat's ledger as a seen thing
    pub confirm: Option<&'static str>,
    /// a Keep's return day
    pub due: u64,
}

/// ToFetch: the thing's still out there - pick it up first. ToCarry:
/// it's in hand, see it to whoever it's for
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestStage { ToFetch, ToCarry }

/// A job the seat is running
#[derive(Clone, Debug)]
pub struct PlayerQuest {
    pub giver: Entity,
    pub kind: QuestKind,
    pub target: QuestTarget,
    pub stage: QuestStage,
    /// fetch jobs: where the thing lies, in tiles
    pub pickup: Option<(f32, f32)>,
    /// the day it goes stale
    pub expires: u64,
    /// an errand's proof and due day (E5-7), where the job is one
    pub errand: Option<Errand>,
}

/// A job going begging: somebody's asked, nobody's taken it yet
#[derive(Clone, Debug)]
pub struct PendingQuest {
    pub giver: Entity,
    pub kind: QuestKind,
    pub target: QuestTarget,
    pub pickup: Option<(f32, f32)>,
    pub expires: u64,
    /// an errand's proof and due day (E5-7), where the job is one
    pub errand: Option<Errand>,
}

/// How many jobs can go begging at once
const PENDING_MAX: usize = 3;
/// A job goes stale after this many days, taken or not
const QUEST_DAYS: u64 = 10;
/// A begging job waits this long for a taker
const OFFER_DAYS: u64 = 6;
/// Rapport a villager needs with the Wanderer before they'll ask
const OFFER_RAPPORT: f32 = 0.2;
/// Days before the same folk can ask again
const REST_DAYS: u64 = 20;
/// How close you stand to pick the thing up, or reach the far village
const REACH: f32 = 1.6;
/// Days a Keep's bundle stays hid before it goes back (E5-7)
const KEEP_DAYS: u64 = 4;

impl Game {
    /// The pending job `giver` is offering, if any
    pub fn quest_offer_at(&self, giver: Entity) -> Option<&PendingQuest> {
        self.pending_quests.iter().find(|q| q.giver == giver)
    }

    /// Folk who'll ask the Wanderer for something: grown, living, of the
    /// village (not the country's), and enough in their confidence
    fn quest_eligible(&self, v: Entity) -> bool {
        if self.world.get_component::<crate::game::vocation::Kid>(v).is_some()
            || crate::game::villager::is_dead(&self.world, v)
            || self.world.get_component::<crate::session::country::Outlander>(v).is_some() {
            return false;
        }
        // asked of "the Wanderer" broadly: it's enough that one of the
        // connected players has gotten to know them, not every seat
        self.seats.iter().any(|s| crate::game::player::rapport(&self.world, s.player, v) >= OFFER_RAPPORT)
    }

    /// The nearest passable tile to (x, y), spiralling out a few steps
    fn near_passable(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        (0..=3).flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (cx + dx, cy + dy))))
            .find(|&(tx, ty)| self.map.passable(tx, ty))
            .map(|(tx, ty)| (tx as f32 + 0.5, ty as f32 + 0.5))
    }

    /// A spot for the thing to be fetched from: the hen run for an egg,
    /// the boulders for a stone
    fn fetch_spot(&self, kind: QuestKind, rng: &mut Rng) -> Option<(f32, f32)> {
        match kind {
            QuestKind::Egg => self.near_passable(self.chicken_at.0 + rng.around(3) as f32 * 0.4,
                self.chicken_at.1 + rng.around(3) as f32 * 0.4),
            QuestKind::Stone => self.stones.first().and_then(|&(x, y)|
                self.near_passable(x as f32 + rng.around(3) as f32 * 0.5, y as f32 + rng.around(3) as f32 * 0.5)),
            _ => None,
        }
    }

    /// The ghost's own ask (E5-7, #1438): which of the five errand
    /// shapes the strongest wound wants, and where it lands - a loaf
    /// to the stones for grief, the debt question to the collector,
    /// the manor roll to be seen, word to the town one's estranged
    /// from, a bundle to keep hid. None when there's no wound, or the
    /// shape's payload can't be laid this day
    pub(in crate::session) fn errand_offer(&self, giver: Entity)
        -> Option<(QuestKind, QuestTarget, Errand)> {
        use crate::game::persona::{EventKind as EK, Persona};
        let p = self.world.get_component::<Persona>(giver)?;
        let g = p.ghosts.iter().max_by(|a, b|
            a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))?;
        let kind = match g.kind {
            EK::Fire | EK::Fever | EK::DeathDuty => QuestKind::Deliver,
            EK::MillerDebt | EK::Levy => QuestKind::Ask,
            EK::Eviction => QuestKind::FindOut,
            EK::BrokenBetrothal | EK::HarvestFeast => QuestKind::CarryWord,
            EK::OldFaith => QuestKind::Keep,
        };
        match kind {
            QuestKind::Deliver => {
                // the loaf to the cairn: the old stones take the offering
                let s = self.stones.first().and_then(|&(x, y)|
                    self.near_passable(x as f32, y as f32))?;
                Some((kind, QuestTarget::Spot(s.0, s.1), Errand { confirm: None, due: 0 }))
            }
            QuestKind::Ask => {
                // who keeps the roll: the collector first, any posting else -
                // never the asker themselves
                let t = self.villagers.iter().copied().find(|&v| v != giver
                    && crate::game::livelihood::has_role(&self.world, v,
                        crate::game::vocation::TownRole::TaxCollector))
                    .or_else(|| self.villagers.iter().copied().find(|&v| v != giver
                        && self.world.get_component::<crate::game::vocation::TownRole>(v).is_some()))?;
                Some((kind, QuestTarget::Villager(t),
                    Errand { confirm: Some(g.kind.tag()), due: 0 }))
            }
            QuestKind::FindOut => {
                // the manor roll: the castle keeps the writs
                let i = self.sites.iter().position(|s|
                    s.kind == crate::session::country::SiteKind::Castle)
                    .or(if self.sites.is_empty() { None } else { Some(0) })?;
                Some((kind, QuestTarget::Site(i),
                    Errand { confirm: Some(g.kind.tag()), due: 0 }))
            }
            QuestKind::CarryWord => {
                // a sister in the town: the estranged kin lives out there
                let i = self.sites.iter().position(|s|
                    s.kind == crate::session::country::SiteKind::Town)?;
                Some((kind, QuestTarget::Site(i), Errand { confirm: None, due: 0 }))
            }
            QuestKind::Keep =>
                Some((kind, QuestTarget::Villager(giver),
                    Errand { confirm: None, due: self.tick + KEEP_DAYS })),
            _ => None,
        }
    }

    /// Somebody to send a note to: another villager of the village
    pub(in crate::session) fn note_target(&self, giver: Entity, rng: &mut Rng) -> Option<Entity> {
        let folk: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| v != giver && self.quest_eligible(v)).collect();
        if folk.is_empty() { return None; }
        Some(folk[rng.below(folk.len() as i32) as usize])
    }

    /// Dawn of each day: stale jobs lapse, and now and then somebody
    /// who's gotten to know the Wanderer asks them for a small job
    pub(super) fn quest_dawn(&mut self, tick: u64, rng: &mut Rng) {
        // begging jobs nobody took, and jobs gone stale, lapse - and
        // folk whose ask went unanswered don't ask again straight off
        let ignored: Vec<Entity> = self.pending_quests.iter()
            .filter(|q| q.expires <= tick).map(|q| q.giver).collect();
        for g in ignored { self.quest_rest.insert(g, tick + REST_DAYS / 2); }
        self.pending_quests.retain(|q| q.expires > tick);
        for seat in 0..self.seats.len() {
            let stale = self.seats[seat].quest.as_ref()
                .map(|q| q.expires <= tick).unwrap_or(false);
            if stale {
                self.seats[seat].quest = None;
                self.log.push_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: the job went stale - folk stopped asking after it.",
                        tick
                    ),
                );
            }
            // a fetch whose thing can no longer be had, or a delivery
            // whose one's dead, comes to nothing
            if let Some(q) = &self.seats[seat].quest {
                let dead_end = match q.target {
                    QuestTarget::Villager(v) => crate::game::villager::is_dead(&self.world, v),
                    QuestTarget::Site(i) => i >= self.sites.len(),
                    QuestTarget::Spot(x, y) => !self.map.passable(x.floor() as i32, y.floor() as i32),
                } || q.pickup.map(|(x, y)| !self.map.passable(x.floor() as i32, y.floor() as i32)).unwrap_or(false);
                if dead_end {
                    let who = self.name_of(q.giver);
                    self.seats[seat].quest = None;
                    self.log.push_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: {} stops asking after it - there's no more to be done for it.",
                            tick, who
                        ),
                    );
                }
            }
        }
        // the day's asks: a few folk at most go begging
        if self.pending_quests.len() >= PENDING_MAX { return; }
        let folk = self.villagers.clone();
        for v in folk {
            if self.pending_quests.len() >= PENDING_MAX { break; }
            if !self.quest_eligible(v) { continue; }
            if self.pending_quests.iter().any(|q| q.giver == v) { continue; }
            if self.quest_rest.get(&v).copied().unwrap_or(0) > tick { continue; }
            if rng.below(100) >= 5 { continue; }
            // the ghost asks in its own shape first (E5-7); the plain
            // jobs roll for the rest
            let offer = self.errand_offer(v).map(|(k, t, e)| (k, t, None, Some(e)))
                .or_else(|| {
                    // what they need: a note carried, a parcel walked
                    // out, or a thing fetched back - whichever the
                    // world's got for
                    let roll = rng.below(100);
                    if roll < 30 {
                        self.note_target(v, rng).map(|t| (QuestKind::Note, QuestTarget::Villager(t), None))
                    } else if roll < 55 && !self.sites.is_empty() {
                        let i = rng.below(self.sites.len() as i32) as usize;
                        Some((QuestKind::Parcel, QuestTarget::Site(i), None))
                    } else {
                        let kind = if roll < 78 { QuestKind::Egg } else { QuestKind::Stone };
                        self.fetch_spot(kind, rng).map(|p| (kind, QuestTarget::Villager(v), Some(p)))
                    }
                    .map(|(k, t, p)| (k, t, p, None))
                });
            // one open errand a villager (E5-7): while the seat still
            // runs theirs, they don't go begging for another
            if self.seats.iter().any(|s| s.quest.as_ref()
                .map(|q| q.giver == v).unwrap_or(false)) { continue; }
            let Some((kind, target, pickup, errand)) = offer else { continue };
            self.notices.push(("quest".into(), format!("{} asks for help - the board carries it",
                self.name_of(v)), tick + 14));
            self.pending_quests.push(PendingQuest {
                giver: v,
                kind,
                target,
                pickup,
                expires: tick + OFFER_DAYS,
                errand,
            });
            self.say(
                v,
                &self.pair_said(v, self.player, ASK_LINES[rng.below(ASK_LINES.len() as i32) as usize]),
                TALK_BUBBLE_SECS * 2,
            );
            self.log_entity_ev(
                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                v,
                format!(
                    "t{}: {} has a small job going - pick them to hear it.",
                    tick,
                    self.name_of(v)
                ),
            );
        }
    }

    /// The opening victim: one villager of the village marked by a real,
    /// seeded injustice - the grievance drives their wants like any
    /// other wrong, and every seat's first job is simply to hear them
    /// out. Picked off its own stream so the village's rolls don't move
    pub(in crate::session) fn pick_intro_victim(&mut self) {
        let mut rng = Rng(self.seed ^ 0x0666_0000_0000_0007);
        let eligible: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
            self.world.get_component::<crate::game::vocation::Kid>(v).is_none()
                && !crate::game::villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::session::country::Outlander>(v).is_none()
        ).collect();
        if eligible.is_empty() { return; }
        let v = eligible[rng.below(eligible.len() as i32) as usize];
        // the curated wrongs - an injustice the village would recognize,
        // never the hermit's dark or a passing whim
        const WRONGS: [desires::Problem; 7] = [
            desires::Problem::Taxed, desires::Problem::WritBorne, desires::Problem::Blamed,
            desires::Problem::DeathDuty, desires::Problem::Bereaved,
            desires::Problem::ShortChanged, desires::Problem::OverseerCut];
        let wrong = WRONGS[rng.below(WRONGS.len() as i32) as usize];
        desires::add_grievance(&mut self.world, v, wrong, u64::MAX);
        self.intro_victim = Some((v, wrong));
        for seat in 0..self.seats.len() { self.intro_quest_for(seat); }
    }

    /// A seat's share of the opening: the victim's warmth (enough to
    /// open up, not best-friends) and the talk job itself. Seats that
    /// join later get the same through their own add path
    pub(in crate::session) fn intro_quest_for(&mut self, seat: usize) {
        let Some((v, _)) = self.intro_victim else { return };
        if crate::game::villager::is_dead(&self.world, v) { return; }
        // an agent driving the victim can't hear themselves out - the
        // job's for a listener, and they are the one with the trouble
        if self.seats[seat].player == v { return; }
        let pe = self.seats[seat].player;
        let mut rap = self.world.get_component::<crate::game::player::Rapport>(v)
            .cloned().unwrap_or_default();
        let e = rap.by_player.entry(pe).or_insert(0.0);
        *e = e.max(0.3);
        self.world.add_component(v, rap);
        if self.seats[seat].quest.is_none() {
            self.seats[seat].quest = Some(PlayerQuest {
                giver: v, kind: QuestKind::Talk, target: QuestTarget::Villager(v),
                stage: QuestStage::ToCarry, pickup: None, expires: self.tick + QUEST_DAYS,
                errand: None });
        }
    }

    /// The picked villager's job, taken on: the ask becomes the seat's
    /// to run. One job at a time - a second ask waits its turn
    pub fn accept_quest_at(&mut self, seat: usize) -> String {
        if self.seats[seat].quest.is_some() {
            return "You've already got a job on - see it through first.".into();
        }
        let v = self.villagers[self.seats[seat].selected_target];
        // they're still asking, but only within earshot - walk over first
        let (px, py) = self.player_pos_at(seat);
        let near = self.world.get_component::<Pos>(v)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= TALK_DIST)
            .unwrap_or(false);
        if !near { return "They're too far off to hear you - walk over first.".into(); }
        let Some(i) = self.pending_quests.iter().position(|q| q.giver == v) else {
            return "They haven't got anything to ask you.".into();
        };
        let quest = self.take_pending_quest(seat, i);
        // the log keeps it third-person; the bubble over their head is the ask
        format!("{}: \"{}\"", self.name_of(v), self.quest_ask_line(&quest))
    }

    /// The job's yours once it's off the begging list: the asker's ask
    /// lands as a PlayerQuest on the seat, they say it aloud, and they
    /// rest a while before they beg again. The call site's already
    /// checked the seat's free and the list's live - at the ear for a
    /// spoken ask, at the board for a posted one
    pub(in crate::session) fn take_pending_quest(&mut self, seat: usize, i: usize) -> PlayerQuest {
        let q = self.pending_quests.remove(i);
        let stage = if q.pickup.is_some() { QuestStage::ToFetch } else { QuestStage::ToCarry };
        let quest = PlayerQuest {
            giver: q.giver, kind: q.kind, target: q.target, stage, pickup: q.pickup,
            expires: self.tick + QUEST_DAYS, errand: q.errand };
        self.say(q.giver, &self.quest_ask_line(&quest), TALK_BUBBLE_SECS * 2);
        self.seats[seat].quest = Some(quest.clone());
        self.quest_rest.insert(q.giver, self.tick + REST_DAYS);
        quest
    }

    /// What they ask for, in their own words - the bubble on accepting
    /// and the panel's reminder both read off this
    pub fn quest_ask_line(&self, q: &PlayerQuest) -> String {
        let line = match (q.kind, q.target) {
            (QuestKind::Note, QuestTarget::Villager(t)) =>
                format!("{{2:will}} {{2:subj}} take this note to {}?", self.name_of(t)),
            (QuestKind::Parcel, QuestTarget::Site(i)) =>
                format!("{{2:would}} {{2:subj}} carry this parcel out to {}?", self.sites.get(i).map(|s| s.name).unwrap_or("the village")),
            (QuestKind::Egg, _) => "{2:could} {2:subj} fetch a fresh egg from the hen run?".into(),
            (QuestKind::Stone, _) => "{2:would} {2:subj} bring me a stone from the old boulders?".into(),
            (QuestKind::Talk, _) => "I carry a trouble only a listener can ease - {2:will} {2:subj} hear me out?".into(),
            (QuestKind::Deliver, _) => "{2:will} {2:subj} carry a loaf to the old stones for me? It's for one I lost.".into(),
            (QuestKind::Ask, QuestTarget::Villager(t)) =>
                format!("{{2:could}} {{2:subj}} ask {} whether the debt was ever written down?", self.name_of(t)),
            (QuestKind::FindOut, QuestTarget::Site(i)) =>
                format!("{{2:will}} {{2:subj}} go look at the manor roll in {} for me? I need to know what's writ.", self.sites.get(i).map(|s| s.name).unwrap_or("the castle")),
            (QuestKind::CarryWord, QuestTarget::Site(i)) =>
                format!("{{2:would}} {{2:subj}} carry word to {} for me? One of mine's there - we've not spoken in a year.", self.sites.get(i).map(|s| s.name).unwrap_or("the town")),
            (QuestKind::Keep, _) => "{2:will} {2:subj} hold a bundle for me, a few days? It can't be found on me.".into(),
            _ => "{2:will} {2:subj} do me a small job?".into(),
        };
        self.pair_said(q.giver, self.player, &line)
    }

    /// The begging job's ask in the giver's words - a Talk's opening
    /// names it the way the card's bubble would (#1336)
    pub(in crate::session) fn pending_ask_line(&self, q: &PendingQuest) -> String {
        self.quest_ask_line(&PlayerQuest {
            giver: q.giver, kind: q.kind, target: q.target,
            stage: QuestStage::ToCarry, pickup: q.pickup, expires: q.expires,
            errand: q.errand })
    }

    /// One frame of every job under way: standing on the thing picks it
    /// up, standing on the far village hands it over
    pub fn quest_frame(&mut self) {
        for seat in 0..self.seats.len() {
            let Some(q) = self.seats[seat].quest.clone() else { continue };
            let (px, py) = self.player_pos_at(seat);
            // fetch: the thing's where it lies - pick it up
            if q.stage == QuestStage::ToFetch {
                if let Some((x, y)) = q.pickup {
                    if (px - x).abs() + (py - y).abs() <= REACH {
                        if let Some(qq) = &mut self.seats[seat].quest {
                            qq.stage = QuestStage::ToCarry;
                        }
                        self.log.push_ev(
                            EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                            format!("t{}: you take {}.", self.tick, self.quest_thing(&q)),
                        );
                    }
                }
                continue;
            }
            // deliver to a far village or a named spot: standing on the
            // ground is the handover
            match q.target {
                QuestTarget::Site(i) => {
                    if let Some(site) = self.sites.get(i) {
                        let d = ((px - site.centre.0 as f32).powi(2) + (py - site.centre.1 as f32).powi(2)).sqrt();
                        if d <= site.radius as f32 + 1.0 {
                            self.finish_quest_site(seat, i);
                        }
                    }
                }
                QuestTarget::Spot(x, y) => {
                    if (px - x).abs() + (py - y).abs() <= REACH {
                        self.finish_quest_spot(seat);
                    }
                }
                QuestTarget::Villager(_) => {}
            }
        }
    }

    /// Where the job wants you: the thing's spot while it's a fetch,
    /// else the villager or the far village's centre you hand over at -
    /// the point the client's marker aims at
    pub fn quest_target_pos(&self, q: &PlayerQuest) -> Option<(f32, f32)> {
        if q.stage == QuestStage::ToFetch { return q.pickup; }
        match q.target {
            QuestTarget::Villager(v) =>
                self.world.get_component::<Pos>(v).map(|p| (p.x, p.y)),
            QuestTarget::Site(i) =>
                self.sites.get(i).map(|s| (s.centre.0 as f32, s.centre.1 as f32)),
            QuestTarget::Spot(x, y) => Some((x, y)),
        }
    }

    /// The job as the panel shows it: what's wanted and who's it for
    pub fn quest_panel_line(&self, q: &PlayerQuest) -> String {
        if q.kind == QuestKind::Talk {
            return format!("Job: hear {}'s trouble out", self.name_of(q.giver));
        }
        let thing = self.quest_thing(q);
        let where_to = match q.target {
            QuestTarget::Villager(t) => self.name_of(t),
            QuestTarget::Site(i) => self.sites.get(i).map(|s| s.name).unwrap_or("the village").to_string(),
            QuestTarget::Spot(..) => "the old stones".to_string(),
        };
        match (q.kind, q.stage) {
            (QuestKind::Deliver, _) => "Job: lay the loaf at the old stones".into(),
            (QuestKind::Ask, _) => format!("Job: ask {} about the debt", where_to),
            (QuestKind::FindOut, _) => format!("Job: see the manor roll at {}", where_to),
            (QuestKind::CarryWord, _) => format!("Job: carry word to {}", where_to),
            (QuestKind::Keep, _) if q.errand.map(|e| e.due > self.tick).unwrap_or(false) =>
                "Job: keep the bundle hid a few days".into(),
            (QuestKind::Keep, _) => format!("Job: carry {} back to {}", thing, self.name_of(q.giver)),
            (QuestKind::Egg, QuestStage::ToFetch) => "Job: fetch the egg from the hen run".into(),
            (QuestKind::Stone, QuestStage::ToFetch) => "Job: fetch a stone from the boulders".into(),
            (_, QuestStage::ToFetch) => format!("Job: get {}", thing),
            (_, QuestStage::ToCarry) if matches!(q.target, QuestTarget::Site(_)) =>
                format!("Job: carry {} to {}", thing, where_to),
            (_, QuestStage::ToCarry) => format!("Job: take {} to {}", thing, where_to),
        }
    }

    /// What the job's about, in words - "the note", "the egg"
    fn quest_thing(&self, q: &PlayerQuest) -> &'static str {
        match q.kind {
            QuestKind::Note => "the note",
            QuestKind::Parcel => "the parcel",
            QuestKind::Egg => "the egg",
            QuestKind::Stone => "the stone",
            QuestKind::Talk => "their trouble",
            QuestKind::Deliver => "the loaf",
            QuestKind::Ask => "the question",
            QuestKind::FindOut => "the errand",
            QuestKind::CarryWord => "the word",
            QuestKind::Keep => "the bundle",
        }
    }

    /// Talking to whoever it's for hands it over: the job's done, the
    /// giver's pleased, and there's a few coins in't for the trouble
    pub fn quest_talk_done(&mut self, seat: usize, v: Entity) -> Option<String> {
        let q = self.seats[seat].quest.clone()?;
        if q.stage != QuestStage::ToCarry { return None; }
        let delivers = match q.target { QuestTarget::Villager(t) => t == v, _ => false };
        if !delivers { return None; }
        // a Keep isn't handed back until its days are out (E5-7) - held
        // long enough, the bundle's safe to carry home
        if q.kind == QuestKind::Keep
            && q.errand.map(|e| e.due > self.tick).unwrap_or(false) {
            return None;
        }
        self.seats[seat].quest = None;
        let pe = self.seats[seat].player;
        // an Ask's answer settles the ledger (E5-7's done-when): who
        // keeps the roll says it plain - the debt's written, and the
        // seat saw proof rather than heard it
        if q.kind == QuestKind::Ask {
            if let Some(tag) = q.errand.and_then(|e| e.confirm) {
                crate::game::desires::confirm(&mut self.world, pe, q.giver,
                    &format!("ghost:{tag}"), self.tick);
            }
        }
        let giver = self.name_of(q.giver);
        // the opening ear earns no coin - the reward is the bond itself
        if q.kind == QuestKind::Talk {
            let known = self.quest_bond(seat, q.giver, q.kind);
            let mut line = format!("t{}: {} feels heard - the trouble's lighter for saying it.", self.tick, giver);
            if known {
                line.push_str(&format!(" - {} opens up to you; you know them now.", giver));
            }
            return Some(line);
        }
        let thing = self.quest_thing(&q);
        // a little gold for the leg-work, more for the long walk out
        let pay = match q.target { QuestTarget::Site(_) | QuestTarget::Spot(..) => 5.0, _ => 3.0 };
        // both purses: the panel reads resources, trades move the Purse,
        // and act_at's sync would wash out a payment in only one of them
        if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(pe) { p.resources += pay; }
        if let Some(p) = self.world.get_component_mut::<livelihood::Purse>(pe) { p.gold += pay; }
        let thanks = match q.kind {
            QuestKind::Note => format!("The note reached {} - and {} remembers it.", self.name_of(v), giver),
            _ => format!("The {} reached {} - and {} remembers it.", thing, self.name_of(v), giver),
        };
        let tl = self.pair_said(v, self.seats[seat].player, THANKS_LINES[q.kind as usize % THANKS_LINES.len()]);
        self.say(v, &tl, TALK_BUBBLE_SECS * 2);
        let known = self.quest_bond(seat, q.giver, q.kind);
        let mut line = format!("t{}: {} ({:.0}g)", self.tick, thanks, pay);
        if known {
            line.push_str(&format!(" - {} opens up to you; you know them now.", giver));
        }
        Some(line)
    }

    /// A job done for someone opens them up: the deed's recorded on the
    /// bond (which alone reads as knowing them), and they tell you a
    /// thing about why they asked. Returns whether they read as known.
    pub(in crate::session) fn quest_bond(&mut self, seat: usize, giver: Entity,
                                         kind: QuestKind) -> bool {
        crate::timed!("sim.quest_bond", self.quest_bond_inner(seat, giver, kind))
    }

    fn quest_bond_inner(&mut self, seat: usize, giver: Entity, kind: QuestKind) -> bool {
        use crate::game::organizing::{stage_of, Deeds, RelStage};
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind, Relationship};
        if crate::game::villager::is_dead(&self.world, giver) { return false; }
        let pe = self.seats[seat].player;
        let bond = record_interaction(&mut self.world, pe, Interaction {
            other: giver, kind: InteractionKind::Aid, intensity: 0.8 });
        // the shape presses the face it proves (E5-7, #1438): a thing
        // carried or a word borne reads kindness, a question settled or
        // a sight seen competence, a bundle kept hid the keeping itself
        if let Some(rel) = self.world.get_component_mut::<Relationship>(bond) {
            match kind {
                QuestKind::Deliver | QuestKind::CarryWord => rel.trust.benevolence += 0.12,
                QuestKind::Ask | QuestKind::FindOut => rel.trust.ability += 0.12,
                QuestKind::Keep => rel.trust.integrity += 0.15,
                _ => {}
            }
            rel.recompose();
        }
        let mut deeds = self.world.get_component::<Deeds>(bond).copied().unwrap_or_default();
        deeds.n += 1;
        self.world.add_component(bond, deeds);
        // a promise kept is the pair's own episode (E6-2): it can come
        // back to you at a later talk as a callback
        crate::game::episodes::file(&mut self.world, giver, pe,
            crate::game::episodes::MemKind::KeptPromise, self.tick, &["errand", "promise"]);
        // they open up about why they needed the job
        if let Some(r) = desires::next_reveal(&self.world, pe, giver, self.tick, self.tick) {
            if let Some(issue) = desires::hear(&mut self.world, pe, giver, r, self.tick) {
                self.toasts.push((
                    format!("Organizer discovers issue {}", issue),
                    self.tick + 40,
                ));
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                    format!("t{}: Organizer discovers issue {}", self.tick, issue),
                );
            }
        }
        stage_of(&self.world, pe, giver) >= RelStage::Known
    }

    /// A parcel walked out to a far village: the site's folk see it land
    fn finish_quest_site(&mut self, seat: usize, site: usize) {
        let Some(q) = self.seats[seat].quest.take() else { return };
        let name = self.sites.get(site).map(|s| s.name).unwrap_or("the village");
        let giver = self.name_of(q.giver);
        let pe = self.seats[seat].player;
        if let Some(p) = self
            .world
            .get_component_mut::<crate::game::player::PlayerCharacter>(pe)
        {
            p.resources += 5.0;
        }
        if let Some(p) = self.world.get_component_mut::<livelihood::Purse>(pe) {
            p.gold += 5.0;
        }
        let landed = match q.kind {
            QuestKind::FindOut => format!(
                "t{}: the roll at {} is seen - word gets back to {}. (5g)", self.tick, name, giver),
            QuestKind::CarryWord => format!(
                "t{}: word reaches {} - {} will hear of it soon. (5g)", self.tick, name, giver),
            _ => format!(
                "t{}: the parcel reaches {} - word gets back to {}. (5g)", self.tick, name, giver),
        };
        self.log.push_ev(
            EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor), landed);
        // a Find out's sight settles the ledger the same way an Ask's
        // answer does (E5-7): the roll was seen, not heard of
        if q.kind == QuestKind::FindOut {
            if let Some(tag) = q.errand.and_then(|e| e.confirm) {
                crate::game::desires::confirm(&mut self.world, pe, q.giver,
                    &format!("ghost:{tag}"), self.tick);
            }
        }
        if self.quest_bond(seat, q.giver, q.kind) {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: {} opens up to you; you know them now.",
                    self.tick, giver
                ),
            );
        }
        self.toasts.push((match q.kind {
            QuestKind::FindOut => format!("The roll at {} is seen", name),
            QuestKind::CarryWord => format!("Word reached {}", name),
            _ => format!("The parcel reached {}", name),
        }, self.tick + 60));
        self.seats[seat].quests_done += 1;
    }

    /// The loaf laid at the stones (E5-7): a named spot, not a
    /// settlement - the errand ends where the offering rests
    fn finish_quest_spot(&mut self, seat: usize) {
        let Some(q) = self.seats[seat].quest.take() else { return };
        let giver = self.name_of(q.giver);
        let pe = self.seats[seat].player;
        if let Some(c) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(pe) { c.resources += 3.0; }
        if let Some(c) = self.world.get_component_mut::<livelihood::Purse>(pe) { c.gold += 3.0; }
        self.log.push_ev(
            EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
            format!("t{}: the offering rests at the stones - {} will hear of it. (3g)",
                self.tick, giver),
        );
        if self.quest_bond(seat, q.giver, q.kind) {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                format!("t{}: {} opens up to you; you know them now.", self.tick, giver),
            );
        }
        self.seats[seat].quests_done += 1;
    }
}

/// What folk say when they ask for the job
const ASK_LINES: [&str; 4] = [
    "{2:could} {2:subj} do me a small job?",
    "{2:subj} {2:be} a reliable sort - {2:will} {2:subj} carry this for me?",
    "A small thing, if {2:subj} {2:would} be so good.",
    "I need a pair of hands I can trust.",
];

/// And when it's handed over
const THANKS_LINES: [&str; 3] = [
    "{2:subj} {2:be} a good sort - it'll not be forgot.",
    "Ta very much - {2:subj} {2:have} a friend in me.",
    "No trouble at all - folk like {2:obj} make a village.",
];
