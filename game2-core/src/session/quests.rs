//! The odd jobs villagers ask the Wanderer to run: a note to carry, a
//! parcel out to a far village, an egg or a stone to fetch back. Offers
//! come up at dawn for folk the player has gotten to know; the job is
//! taken from the picked villager's contextual button, the thing changes
//! hands on its own once you're standing on it, and it's handed over by
//! talking to whoever it's for.

use crate::engine::ecs::Entity;
use crate::game::livelihood;
use crate::session::*;

/// What the job is about
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestKind { Note, Parcel, Egg, Stone }

/// Where the thing ends up
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuestTarget { Villager(Entity), Site(usize) }

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
}

/// A job going begging: somebody's asked, nobody's taken it yet
#[derive(Clone, Debug)]
pub struct PendingQuest {
    pub giver: Entity,
    pub kind: QuestKind,
    pub target: QuestTarget,
    pub pickup: Option<(f32, f32)>,
    pub expires: u64,
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

    /// Somebody to send a note to: another villager of the village
    fn note_target(&self, giver: Entity, rng: &mut Rng) -> Option<Entity> {
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
                self.log.push(format!("t{}: the job went stale - folk stopped asking after it.", tick));
            }
            // a fetch whose thing can no longer be had, or a delivery
            // whose one's dead, comes to nothing
            if let Some(q) = &self.seats[seat].quest {
                let dead_end = match q.target {
                    QuestTarget::Villager(v) => crate::game::villager::is_dead(&self.world, v),
                    QuestTarget::Site(i) => i >= self.sites.len(),
                } || q.pickup.map(|(x, y)| !self.map.passable(x.floor() as i32, y.floor() as i32)).unwrap_or(false);
                if dead_end {
                    let who = self.name_of(q.giver);
                    self.seats[seat].quest = None;
                    self.log.push(format!("t{}: {} stops asking after it - there's no more to be done for it.", tick, who));
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
            // what they need: a note carried, a parcel walked out, or
            // a thing fetched back - whichever the world's got for
            let roll = rng.below(100);
            let offer = if roll < 30 {
                self.note_target(v, rng).map(|t| (QuestKind::Note, QuestTarget::Villager(t), None))
            } else if roll < 55 && !self.sites.is_empty() {
                let i = rng.below(self.sites.len() as i32) as usize;
                Some((QuestKind::Parcel, QuestTarget::Site(i), None))
            } else {
                let kind = if roll < 78 { QuestKind::Egg } else { QuestKind::Stone };
                self.fetch_spot(kind, rng).map(|p| (kind, QuestTarget::Villager(v), Some(p)))
            };
            let Some((kind, target, pickup)) = offer else { continue };
            self.pending_quests.push(PendingQuest {
                giver: v, kind, target, pickup, expires: tick + OFFER_DAYS });
            self.say(v, ASK_LINES[rng.below(ASK_LINES.len() as i32) as usize], TALK_BUBBLE_SECS * 2);
            self.log.push(format!("t{}: {} has a small job going - pick them to hear it.", tick, self.name_of(v)));
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
        let q = self.pending_quests.remove(i);
        let stage = if q.pickup.is_some() { QuestStage::ToFetch } else { QuestStage::ToCarry };
        let quest = PlayerQuest {
            giver: q.giver, kind: q.kind, target: q.target, stage, pickup: q.pickup,
            expires: self.tick + QUEST_DAYS };
        let ask = self.quest_ask_line(&quest);
        self.say(v, &ask, TALK_BUBBLE_SECS * 2);
        self.seats[seat].quest = Some(quest);
        self.quest_rest.insert(v, self.tick + REST_DAYS);
        // the log keeps it third-person; the bubble over their head is the ask
        format!("{}: \"{}\"", self.name_of(v), ask)
    }

    /// What they ask for, in their own words - the bubble on accepting
    /// and the panel's reminder both read off this
    pub fn quest_ask_line(&self, q: &PlayerQuest) -> String {
        match (q.kind, q.target) {
            (QuestKind::Note, QuestTarget::Villager(t)) =>
                format!("Will you take this note to {}?", self.name_of(t)),
            (QuestKind::Parcel, QuestTarget::Site(i)) =>
                format!("Would you carry this parcel out to {}?", self.sites.get(i).map(|s| s.name).unwrap_or("the village")),
            (QuestKind::Egg, _) => "Could you fetch a fresh egg from the hen run?".into(),
            (QuestKind::Stone, _) => "Would you bring me a stone from the old boulders?".into(),
            _ => "Will you do me a small job?".into(),
        }
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
                        if let Some(qq) = &mut self.seats[seat].quest { qq.stage = QuestStage::ToCarry; }
                        self.log.push(format!("t{}: you take {}.", self.tick, self.quest_thing(&q)));
                    }
                }
                continue;
            }
            // deliver to a far village: standing on its ground is the handover
            if let QuestTarget::Site(i) = q.target {
                if let Some(site) = self.sites.get(i) {
                    let d = ((px - site.centre.0 as f32).powi(2) + (py - site.centre.1 as f32).powi(2)).sqrt();
                    if d <= site.radius as f32 + 1.0 {
                        self.finish_quest_site(seat, i);
                    }
                }
            }
        }
    }

    /// The job as the panel shows it: what's wanted and who's it for
    pub fn quest_panel_line(&self, q: &PlayerQuest) -> String {
        let thing = self.quest_thing(q);
        let where_to = match q.target {
            QuestTarget::Villager(t) => self.name_of(t),
            QuestTarget::Site(i) => self.sites.get(i).map(|s| s.name).unwrap_or("the village").to_string(),
        };
        match (q.kind, q.stage) {
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
        }
    }

    /// Talking to whoever it's for hands it over: the job's done, the
    /// giver's pleased, and there's a few coins in't for the trouble
    pub fn quest_talk_done(&mut self, seat: usize, v: Entity) -> Option<String> {
        let q = self.seats[seat].quest.clone()?;
        if q.stage != QuestStage::ToCarry { return None; }
        let delivers = match q.target { QuestTarget::Villager(t) => t == v, _ => false };
        if !delivers { return None; }
        self.seats[seat].quest = None;
        let giver = self.name_of(q.giver);
        let thing = self.quest_thing(&q);
        // a little gold for the leg-work, more for the long walk out
        let pay = match q.target { QuestTarget::Site(_) => 5.0, _ => 3.0 };
        let pe = self.seats[seat].player;
        // both purses: the panel reads resources, trades move the Purse,
        // and act_at's sync would wash out a payment in only one of them
        if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(pe) { p.resources += pay; }
        if let Some(p) = self.world.get_component_mut::<livelihood::Purse>(pe) { p.gold += pay; }
        let thanks = match q.kind {
            QuestKind::Note => format!("The note reached {} - and {} remembers it.", self.name_of(v), giver),
            _ => format!("The {} reached {} - and {} remembers it.", thing, self.name_of(v), giver),
        };
        self.say(v, THANKS_LINES[q.kind as usize % THANKS_LINES.len()], TALK_BUBBLE_SECS * 2);
        let known = self.quest_bond(seat, q.giver);
        let mut line = format!("t{}: {} ({:.0}g)", self.tick, thanks, pay);
        if known {
            line.push_str(&format!(" - {} opens up to you; you know them now.", giver));
        }
        Some(line)
    }

    /// A job done for someone opens them up: the deed's recorded as aid,
    /// and enough of their self-interest comes out with the thanks that
    /// they read as known, not an acquaintance. Returns whether they
    /// climbed to known.
    fn quest_bond(&mut self, seat: usize, giver: Entity) -> bool {
        use crate::game::organizing::{stage_of, RelStage, KNOWN_PIECES};
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        if crate::game::villager::is_dead(&self.world, giver) { return false; }
        let pe = self.seats[seat].player;
        record_interaction(&mut self.world, pe, Interaction {
            other: giver, kind: InteractionKind::Aid, intensity: 0.8 });
        for i in 0..KNOWN_PIECES {
            if stage_of(&self.world, pe, giver) >= RelStage::Known { break; }
            let Some(r) = desires::next_reveal(&self.world, giver, self.tick,
                self.tick.wrapping_add(i as u64)) else { break };
            if let Some(issue) = desires::hear(&mut self.world, pe, giver, r, self.tick) {
                self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
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
        if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(pe) { p.resources += 5.0; }
        if let Some(p) = self.world.get_component_mut::<livelihood::Purse>(pe) { p.gold += 5.0; }
        self.log.push(format!("t{}: the parcel reaches {} - word gets back to {}. (5g)", self.tick, name, giver));
        if self.quest_bond(seat, q.giver) {
            self.log.push(format!("t{}: {} opens up to you; you know them now.", self.tick, giver));
        }
        self.toasts.push((format!("The parcel reached {}", name), self.tick + 60));
    }
}

/// What folk say when they ask for the job
const ASK_LINES: [&str; 4] = [
    "Could you do me a small job?",
    "You're a reliable sort - will you carry this for me?",
    "A small thing, if you'd be so good.",
    "I need a pair of hands I can trust.",
];

/// And when it's handed over
const THANKS_LINES: [&str; 3] = [
    "You're a good sort - it'll not be forgot.",
    "Ta very much - you've a friend in me.",
    "No trouble at all - folk like you make a village.",
];
