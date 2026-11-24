//! The shared table (#1241): a warm bond asks the organizer in to eat,
//! and the meal is its own beat - three courses over which the household
//! says what the doorstep doesn't: its troubles, who it's feared for, and
//! where it stands on the cause. Rising before the last cup leaves the
//! rest unsaid.

use crate::session::*;
use crate::game::relationships::{self, Interaction, InteractionKind, RelKind, Relationship};
use crate::session::dialogue::{TalkKind, TalkLine};

/// The bond strength at which they'll ask you in
pub const INVITE_TIE: f32 = 0.5;
/// The day's chance a warm bond asks: one in this many
pub const INVITE_IN: u64 = 6;
/// Days an invitation stays open
pub const INVITE_DAYS: u64 = 2;
/// Days before the same household lays a place for you again
pub const TABLE_GAP_DAYS: u64 = 7;
/// Frames between courses
pub const COURSE_FRAMES: u32 = 2 * LINE_FRAMES;

/// One line served at the table: who says it (None: narration), and
/// whether it's the line that matters
type Served = (Option<Entity>, String, bool);

/// A meal under way on a seat
#[derive(Debug, Clone)]
pub struct Table {
    pub host: Entity,
    pub home: Entity,
    /// The courses still to come, in order
    pub courses: Vec<Vec<Served>>,
    pub frames: u32,
    pub talk_id: u64,
    /// What the household let slip, heard by the organizer once the
    /// meal's eaten through
    pub reveals: Vec<(Entity, desires::Reveal)>,
    /// Where the organizer sat down: straying past earshot of it ends the meal
    pub at: (f32, f32),
}

impl Game {
    /// Everyone living under the host's roof, alive and not held at the keep
    fn household(&self, home: Entity) -> Vec<Entity> {
        self.world.get_component::<Building>(home)
            .map(|b| b.occupants.iter().copied()
                .filter(|&o| !villager::is_dead(&self.world, o)
                    && !crate::game::dungeon::unavailable(&self.world, o)).collect())
            .unwrap_or_default()
    }

    fn table_laid_lately(&self, seat: usize, home: Entity, tick: u64) -> bool {
        self.seats[seat].tables_laid.iter()
            .any(|&(h, t)| h == home && tick < t + TABLE_GAP_DAYS)
    }

    /// The day's invitations: a warm bond with a roof of their own may
    /// ask the organizer to their table
    pub fn table_tick(&mut self, tick: u64) {
        for seat in 0..self.seats.len() {
            if self.seats[seat].agent || self.seats[seat].table.is_some() { continue; }
            if let Some((_, until)) = self.seats[seat].table_invite {
                if tick <= until { continue; }
                self.seats[seat].table_invite = None;
            }
            let pe = self.seats[seat].player;
            let host = self.villagers.iter().copied().find(|&v| {
                if villager::is_dead(&self.world, v) || self.world.get_component::<Kid>(v).is_some()
                    || crate::game::dungeon::unavailable(&self.world, v) { return false; }
                if crate::game::organizing::stage_of(&self.world, pe, v) < crate::game::organizing::RelStage::Known { return false; }
                let warm = relationships::find_relationship(&self.world, pe, v)
                    .and_then(|r| self.world.get_component::<Relationship>(r))
                    .map(|r| r.strength >= INVITE_TIE).unwrap_or(false);
                if !warm { return false; }
                let Some(home) = household::home_of(&self.world, v) else { return false };
                !self.table_laid_lately(seat, home, tick)
                    && crate::game::mentorship::roll(tick, v, 0x7AB1E, INVITE_IN)
            });
            let Some(host) = host else { continue };
            self.seats[seat].table_invite = Some((host, tick + INVITE_DAYS));
            self.say(host, "Come and eat with us - there's a place at our table.", TALK_BUBBLE_SECS);
            let n = self.name_of(host);
            self.log.push_ev(EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                format!("t{}: {} asks you to their table.", tick, n));
        }
    }

    /// Whether the seat's been asked in by `v` and can sit down now
    pub fn table_offered(&self, seat: usize, v: Entity) -> bool {
        self.seats[seat].table.is_none()
            && matches!(self.seats[seat].table_invite, Some((h, until)) if h == v && self.tick <= until)
    }

    /// Accept the invitation: sit down, and the first course comes out
    pub fn sit_at_table(&mut self, seat: usize) -> String {
        let Some((host, _)) = self.seats[seat].table_invite.take() else { return String::new() };
        let Some(home) = household::home_of(&self.world, host) else { return String::new() };
        let tick = self.tick;
        let pe = self.seats[seat].player;
        let house = self.household(home);
        let (courses, reveals) = self.courses(seat, host, &house);
        let roof = self.world.get_component::<Building>(home).map(|b| b.name.clone()).unwrap_or_default();
        let mut with = vec![pe];
        with.extend(house.iter().copied());
        let opening = TalkLine { who: None, name: String::new(), text: format!("You sit down at the table in {}.", roof), gist: false };
        let talk_id = self.push_talk(TalkKind::Meal, with, vec![opening]);
        // the kitchen table's the hidden transcript's own room (E5-9)
        if let Some(r) = self.talk_record_mut(talk_id) {
            r.setting = crate::session::dialogue::HOUSE_SETTING;
        }
        self.seats[seat].tables_laid.retain(|&(_, t)| tick < t + TABLE_GAP_DAYS);
        self.seats[seat].tables_laid.push((home, tick));
        let at = self.world.get_component::<villager::Pos>(pe).map(|p| (p.x, p.y)).unwrap_or_default();
        self.seats[seat].table = Some(Table { host, home, courses, frames: 0, talk_id, reveals, at });
        format!("You sit down to eat with {}.", self.name_of(host))
    }

    /// The meal's three courses, built when you sit down
    fn courses(&self, seat: usize, host: Entity, house: &[Entity]) -> (Vec<Vec<Served>>, Vec<(Entity, desires::Reveal)>) {
        let tick = self.tick;
        let mut reveals = vec![];
        // the bread: what's been heavy on the house
        let mut bread: Vec<Served> = vec![(None, "The bread goes round.".into(), false)];
        for &o in house {
            let griefs = self.world.get_component::<desires::InnerLife>(o)
                .map(|l| l.grievances.iter().filter(|(_, until)| *until > tick).map(|(p, _)| *p).collect::<Vec<_>>())
                .unwrap_or_default();
            for p in griefs.into_iter().take(2) {
                bread.push((Some(host), format!("{} {} - it's been heavy on this house.", self.name_of(o), desires::problem_label(&p)), true));
                reveals.push((o, desires::Reveal::Problem(p)));
            }
        }
        if bread.len() == 1 { bread.push((Some(host), "Nothing new under this roof, God be thanked.".into(), false)); }
        // the pottage: who they're feared for, in the house or the family
        let mut pottage: Vec<Served> = vec![(None, "The pottage comes out.".into(), false)];
        // the whole roof, the ones held at the keep too
        let mut kin: Vec<Entity> = household::home_of(&self.world, host).and_then(|h| self.world.get_component::<Building>(h))
            .map(|b| b.occupants.iter().copied().filter(|&o| !villager::is_dead(&self.world, o)).collect())
            .unwrap_or_else(|| house.to_vec());
        for e in self.world.query::<Relationship>() {
            let Some(r) = self.world.get_component::<Relationship>(e) else { continue };
            if r.kind != RelKind::Family { continue; }
            for (a, b) in [(r.from, r.other), (r.other, r.from)] {
                if house.contains(&a) && !kin.contains(&b) { kin.push(b); }
            }
        }
        for &k in &kin {
            if villager::is_dead(&self.world, k) { continue; }
            let why = if crate::game::dungeon::unavailable(&self.world, k) { Some("they're held at the keep") }
                else if is_sick(&self.world, k) { Some("the sickness has them") }
                else { None };
            if let Some(why) = why {
                pottage.push((Some(host), format!("We're feared for {} - {}.", self.name_of(k), why), true));
            }
        }
        if pottage.len() == 1 { pottage.push((Some(host), "All of ours are hale, for once.".into(), false)); }
        // the last cup: where the house stands
        let mut cup: Vec<Served> = vec![(None, "The last cup is poured.".into(), false)];
        let grown: Vec<Entity> = house.iter().copied().filter(|&o| self.world.get_component::<Kid>(o).is_none()).collect();
        match crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) {
            Some(c) => {
                let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| desires::issue_name(cc.issue.clone())).unwrap_or_default();
                for &o in &grown {
                    let text = if crate::game::player::sympathizes(&self.world, c, o) {
                        format!("On {}, this one's with {{2:obj}}.", issue)
                    } else {
                        format!("{}? Not at this table.", issue)
                    };
                    cup.push((Some(o), self.pair_said(o, self.seats[seat].player, &text), true));
                }
            }
            None => {
                let taxed = grown.iter().any(|&o| self.world.get_component::<desires::InnerLife>(o)
                    .map(|l| l.grievances.iter().any(|(p, u)| *p == desires::Problem::Taxed && *u > tick)).unwrap_or(false));
                cup.push((Some(host), if taxed { "The tax-man's been at us - the whole house feels it." }
                    else { "The taxes? We pay, and we say nothing." }.into(), taxed));
            }
        }
        (vec![bread, pottage, cup], reveals)
    }

    /// The courses come out on their beat; the last cup drunk, the
    /// household's warmer and the organizer's heard what it carries
    pub fn advance_tables(&mut self) {
        for seat in 0..self.seats.len() {
            let Some(mut t) = self.seats[seat].table.take() else { continue };
            if villager::is_dead(&self.world, t.host) {
                self.note_talk(t.talk_id, None, "The meal breaks up.");
                continue;
            }
            let pe = self.seats[seat].player;
            let gone = self.world.get_component::<villager::Pos>(pe)
                .map(|p| (p.x - t.at.0).hypot(p.y - t.at.1) > crate::session::dialogue::PLAYER_TALK_RANGE)
                .unwrap_or(true);
            if gone {
                self.seats[seat].table = Some(t);
                self.rise_from_table(seat);
                continue;
            }
            t.frames += 1;
            if t.frames % COURSE_FRAMES == 1 {
                if t.courses.is_empty() {
                    self.clear_table(seat, t);
                    continue;
                }
                for (who, text, gist) in t.courses.remove(0) {
                    let name = who.map(|w| self.speaker_name(w)).unwrap_or_default();
                    if let Some(r) = self.talk_record_mut(t.talk_id) {
                        r.lines.push(TalkLine { who, name, text, gist });
                    }
                }
            }
            self.seats[seat].table = Some(t);
        }
    }

    fn clear_table(&mut self, seat: usize, t: Table) {
        let pe = self.seats[seat].player;
        let tick = self.tick;
        for o in self.household(t.home) {
            relationships::record_interaction(&mut self.world, pe, Interaction {
                other: o, kind: InteractionKind::SharedMeal, intensity: 0.4 });
        }
        for (o, r) in t.reveals {
            desires::hear(&mut self.world, pe, o, r, tick);
        }
        self.note_talk(t.talk_id, None, "The table's cleared. You thank them and take your leave.");
    }

    /// Up before the last cup: the rest goes unsaid, and the meal's
    /// warmth with it
    pub fn rise_from_table(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].table.take() else { return };
        self.note_talk(t.talk_id, None, "You rise before the meal's done - the rest goes unsaid.");
    }
}
