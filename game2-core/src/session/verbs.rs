//! The player's verbs: what each action button and context action does, what blocks it, and walking over to do it.

use crate::session::*;

/// The chest's price for outfitting a brigade: the buckets,
/// the bell, and the first crew's share
const BRIGADE_COST: f32 = 30.0;
/// What the treasury pays to raise the schoolhouse - the ask's
/// hidden while the chest's short of it
const SCHOOL_FOUND_COST: f32 = 30.0;
/// A teacher's season's wage from the chest
const SCHOOL_TEACHER_COST: f32 = 10.0;

/// Days' presence one bench batch costs (#1338): the click used to
/// hand a day for a press; standing at the bench pays one slower
pub(in crate::session) const AMBIENT_WORK_DAYS: f32 = 2.0;

impl Game {
    /// Why action button `i` can't be used right now, if it can't
    pub fn action_blocker_at(&self, seat: usize, i: usize) -> Option<String> {
        self.disabled_reason_at(seat, i)
    }

    /// Why a ctx button can't answer right now, if it can't - shown greyed
    /// with the reason beside it, like the main buttons' `blocked`
    pub fn ctx_blocker_at(&self, seat: usize, act: CtxAct) -> Option<String> {
        match act {
            CtxAct::Snowballs => {
                if self.snowball_day == self.tick { Some("everyone's already played today".into()) }
                else if self.world.get_component::<crate::sim::climate::Climate>(self.climate)
                    .map(|c| c.season != crate::sim::climate::Season::Winter).unwrap_or(true) {
                    Some("snowballs are a winter game".into())
                } else { None }
            }
            CtxAct::Townhall => {
                // the Reeve won't hear the same demands while he's honouring
                // a full grant for them; a part-grant or a hollowed one can
                // still be pressed - and either way he needs a day to answer
                if self.townhall_day == self.tick {
                    Some("the Reeve needs a day to answer".into())
                } else {
                    let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                    let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)?.issue.clone();
                    let g = self.world.get_component::<crate::game::reactions::LordGrant>(self.seats[seat].org)?;
                    (g.issue == issue && g.active(self.tick) && !g.partial && !g.hollow)
                        .then(|| format!("the Reeve has answered - the grant stands till day {}", g.until))
                }
            }
            CtxAct::Reach => {
                let v = self.villagers[self.seats[seat].selected_target];
                if self.world.get_component::<crate::game::leadership::Reach>(v).is_some() {
                    Some("already out talking".into())
                } else if crate::game::leadership::reach_target(&self.world, self.seats[seat].org, v, &self.villagers).is_none() {
                    Some("no one left to ask".into())
                } else { None }
            }
            CtxAct::Delegate => {
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let open = [crate::game::campaign::Role::TurnoutCoordinator,
                    crate::game::campaign::Role::NoteTaker,
                    crate::game::campaign::Role::Chair].iter().copied()
                    .any(|r| crate::game::campaign::seat_holder(&self.world, c, r).is_none());
                if open { None } else { Some("every seat's spoken for".into()) }
            }
            CtxAct::PetCat => self.rare.iter().find(|r| r.id == events::EventId::StrayCat && !r.leaving)
                .filter(|r| r.aux > self.frame_n as u64)
                .map(|_| "she needs a moment".to_string()),
            _ => None,
        }
    }

    /// Why action button `i` can't be used right now for this seat, if it
    /// can't - the seat's own selection, reach, purse and org answer, not
    /// seat 0's
    pub fn disabled_reason_at(&self, seat: usize, i: usize) -> Option<String> {
        // nobody within a shout to talk to: the button says so outright
        if i == 0 && self.talk_target_at(seat).is_none()
            && self.visitor_talk_target_at(seat).is_none() { return Some("no one near enough".into()); }
        // Talk needs the picked or nearest villager alive (it walks over)
        let target = self.villagers[self.talk_target_at(seat).unwrap_or(self.seats[seat].selected_target)];
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(target).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(target).is_some();
        if !target_alive { Some("they have died".into()) }
        else if held { Some("in the dungeon".into()) }
        else { None }
    }

    /// The nearest free tile to the player where a house can go: dry,
    /// buildable land (not water or peaks), with no building (or the
    /// windmill) on it or next to it - a house sprite is ~2 tiles wide - not
    /// on a road, field or food store, and not right on top of the player
    pub fn build_site_at(&self, seat: usize) -> Option<(f32, f32)> {
        let (px, py) = self.player_pos_at(seat);
        let (cx, cy) = (px.round() as i32, py.round() as i32);
        let taken = |x: i32, y: i32| {
            let on = |bx: f32, by: f32| bx.round() as i32 == x && by.round() as i32 == y;
            let beside = |bx: f32, by: f32| (bx.round() as i32 - x).abs() <= 1 && (by.round() as i32 - y).abs() <= 1;
            self.buildings.iter().any(|(bx, by, _)| beside(*bx, *by))
                || beside(self.mill.0, self.mill.1)
                || self.world.query::<villager::FoodSource>().iter()
                    .filter_map(|f| self.world.get_component::<Pos>(*f))
                    .any(|p| on(p.x, p.y))
        };
        let mut best: Option<((f32, f32), i32)> = None;
        for y in (cy - BUILD_SEARCH)..=(cy + BUILD_SEARCH) {
            for x in (cx - BUILD_SEARCH)..=(cx + BUILD_SEARCH) {
                if x < 1 || y < 1 || x as usize >= self.map.width - 1 || y as usize >= self.map.height - 1 { continue; }
                if !self.map.buildable(x, y) || taken(x, y) { continue; }
                let d = (x - cx).abs() + (y - cy).abs();
                if d >= 2 && best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x as f32, y as f32), d)); }
            }
        }
        best.map(|(site, _)| site)
    }

    /// Who Talk lands on now, if anyone: the picked villager, near or
    /// far - past a shout you walk over to them, same as Recruit (#1066;
    /// a far pick was only ever for a look until you act on it - #944),
    /// else someone near: the least lately heard first, nearer on a tie.
    /// None when nobody's picked and nobody's near enough to hear.
    pub fn talk_target_at(&self, seat: usize) -> Option<usize> {
        let s = &self.seats[seat];
        if s.picked && s.selected_building.is_none() {
            return Some(s.selected_target);
        }
        self.shout_pick_at(seat)
    }

    /// The best someone within a shout to single out with a Talk:
    /// living, free, and sorted so the folk you talked to most recently
    /// fall to the bottom (never heard from is on top), nearer on a tie
    fn shout_pick_at(&self, seat: usize) -> Option<usize> {
        let pe = self.seats[seat].player;
        let (px, py) = self.player_pos_at(seat);
        let mut best: Option<(usize, u64, f32)> = None;
        for (i, v) in self.villagers.iter().enumerate() {
            if self.world.get_component::<Health>(*v).is_none() { continue; }
            if self.shunned(*v) || crate::game::dungeon::unavailable(&self.world, *v) { continue; }
            let Some(p) = self.world.get_component::<Pos>(*v) else { continue };
            let d = ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt();
            if d > SHOUT_DIST { continue; }
            let last = self.world.get_component::<TalkCount>(*v)
                .and_then(|c| c.last_day_by.get(&pe).copied()).unwrap_or(0);
            if best.map(|(_, l, n)| (last, d) < (l, n)).unwrap_or(true) { best = Some((i, last, d)); }
        }
        best.map(|(i, _, _)| i)
    }

    /// Each frame: a pick made within a shout lets go once they've
    /// drifted past it - you can't talk to them from there. A pick from
    /// further out was for a look and keeps, coming within a shout makes
    /// it a near one. A walk-over to them in progress holds the pick.
    pub fn advance_selection_at(&mut self, seat: usize) {
        if !self.seats[seat].picked { return; }
        if self.seats[seat].approach.as_ref().map(|a| a.who) == Some(self.seats[seat].selected_target) { return; }
        if self.dist_to_at(seat, self.seats[seat].selected_target) <= SHOUT_DIST {
            self.seats[seat].pick_near = true;
        } else if self.seats[seat].pick_near {
            self.seats[seat].picked = false;
            self.seats[seat].pick_near = false;
        }
    }

    /// Let the pick go outright: the ring, the walk-over and the talk
    /// under way - the leave-taking goes on the record as one
    pub fn unselect_at(&mut self, seat: usize) {
        self.seats[seat].picked = false;
        self.seats[seat].pick_near = false;
        self.seats[seat].approach = None;
        self.close_player_talk(seat, Some("You take your leave."));
    }

    /// Who Talk and Recruit go to: the selected villager if they're alive
    /// and in reach, otherwise the nearest living villager who is - the
    /// player who walks up to someone means them, not whoever was clicked
    /// last. With no one in reach it stays the selected villager.
    pub fn reach_target_at(&self, seat: usize) -> usize {
        let in_reach = self.villagers_in_reach_at(seat);
        if in_reach.contains(&self.villagers[self.seats[seat].selected_target]) { return self.seats[seat].selected_target; }
        let (px, py) = self.player_pos_at(seat);
        let dist = |v: &Entity| self.world.get_component::<Pos>(*v).map(|p| (p.x - px).powi(2) + (p.y - py).powi(2)).unwrap_or(f32::MAX);
        in_reach.iter()
            .min_by(|a, b| dist(a).partial_cmp(&dist(b)).unwrap())
            .and_then(|v| self.villagers.iter().position(|x| x == v))
            .unwrap_or(self.seats[seat].selected_target)
    }

    /// Living villagers within MEETING_REACH of the player
    pub fn villagers_in_reach_at(&self, seat: usize) -> Vec<Entity> {
        let (px, py) = self.player_pos_at(seat);
        self.villagers.iter().copied().filter(|v| {
            let alive = self.world.get_component::<Health>(*v).is_some();
            let near = self.world.get_component::<Pos>(*v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < MEETING_REACH)
                .unwrap_or(false);
            // the shunned aren't asked to sit, and the held aren't around at all
            alive && near && !self.shunned(*v)
                && !crate::game::dungeon::unavailable(&self.world, *v)
        }).collect()
    }

    /// Talk to villager `who`: hear what they say. Returns the log line
    pub fn talk_to_at(&mut self, seat: usize, who: usize) -> String {
        let v = self.villagers[who];
        // a talk with them is already open: pressing T again mustn't
        // close it and start over - the conversation's still going,
        // the replies still on offer (#1058)
        if self.seats[seat].talk.as_ref().map(|t| t.who) == Some(v) {
            return format!("You are already speaking with {}.", self.cast_name(who));
        }
        // Uncapped talk count - Rapport caps at 1.0 and would
        // freeze the concern/small-talk index after a few talks.
        let talks_before = self.world.get_component::<TalkCount>(v)
            .map(|c| c.value).unwrap_or(0);
        // a 1-1 has a small chance of drawing out one of their secrets:
        // a want, or the problem that's in their way
        // the fold's smell: folk hold back a touch with the shepherd
        let smelly = desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly);
        // how the last talk ended opens or closes what they'll share
        // (#1183): a warm ending loosens tongues, a cold one shuts them
        let mood = self.talk_mood_of(v);
        // the hog doesn't trade (#1252): their trouble comes out of the
        // monologue if you hold the space, rarely out of the usual draw
        let hog = self.world.get_component::<crate::game::traits::Personality>(v)
            .map(|p| p.traits.contains(&crate::game::traits::Trait::SelfAbsorbed))
            .unwrap_or(false);
        let hog_factor = if hog { 0.3 } else { 1.0 };
        let reveal_factor = match mood {
            Some(dialogue::TalkMood::Slighted) => 0.35,
            Some(dialogue::TalkMood::Warmed) => 1.6,
            _ => 1.0,
        } * hog_factor;
        let pe = self.seats[seat].player;
        let revealed = if (self.rng.next() % 1000) < (desires::REVEAL_BASE * reveal_factor * if smelly { 900.0 } else { 1000.0 }) as u64 {
            let pick = self.rng.next();
            desires::next_reveal(&self.world, pe, v, self.tick, pick)
        } else { None };
        let mut whispered = false;
        let (did, said) = match revealed {
            Some(r) => {
                let (d, s) = desires::reveal_speech(self.cast_name(who), &r);
                if let Some(issue) = desires::hear(&mut self.world, pe, v, r, self.tick) {
                    self.toasts.push((
                        format!("Organizer discovers issue {}", issue),
                        self.tick + 40,
                    ));
                    self.log_entity_ev(
                        EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                        pe,
                        format!("t{}: Organizer discovers issue {}", self.tick, issue),
                    );
                }
                (d, s)
            }
            // still smarting: what's on their mind is you
            None if mood == Some(dialogue::TalkMood::Slighted) =>
                crate::game::organizing::cold_gist(self.cast_name(who)),
            None => match if hog { None } else { self.mystery_fact_at(seat) } {
                // gossip carries a piece of the mystery, now and then
                Some(text) => {
                    whispered = true;
                    (format!("{} passes on a rumor", self.cast_name(who)), text)
                }
                // a sworn villager's opening is the cause's weather, not
                // the sky's (#1228); no live cause, ordinary small talk -
                // and the hog's small talk is themselves (#1252)
                None => self.cause_gist(seat, v)
                    .or_else(|| hog.then(|| {
                        let gists = crate::game::conversation::HOG_GISTS;
                        let g = gists[(self.rng.next() as usize) % gists.len()];
                        // raw on its way out: pair_said below puts it in
                        // the pair's register and signs it once
                        (format!("{} holds forth", self.cast_name(who)), g.to_string())
                    }))
                    .unwrap_or_else(|| talk_response(&self.world, v, self.cast_name(who), talks_before, self.tick)),
            },
        };
        let pe = self.seats[seat].player;
        // the bond as it stood before this talk: a first talk is with a stranger
        let rel = dialogue::relation_with_wanderer(self.rel_kind(pe, v));
        // the talks behind the pair, before act_talk lands this one's (#1424)
        let talks = crate::game::relationships::find_relationship(&self.world, pe, v)
            .and_then(|e| self.world.get_component::<crate::game::relationships::Relationship>(e))
            .map(|r| r.interactions).unwrap_or(0);
        act_talk(&mut self.world, pe, v, self.tick);
        // their opening speaks in the pair's register (E4-4): the
        // ambient strings carry {2:} slots now
        let said = self.pair_said(v, pe, &said);
        self.say(v, &said, TALK_BUBBLE_SECS);
        // a blow-off leaves them hollow: a real talk coaches them back
        let coached = if self.world.get_component::<crate::game::actions::Demoralized>(v).is_some()
            && crate::game::actions::coach(&mut self.world, pe, v, self.tick) {
            " You talk the helplessness out of them."
        } else { "" };
        // the conversation itself goes on the TALK panel, the replies yours to pick
        self.open_player_talk_familiar(seat, v, rel, &said, Some(talks));
        if !coached.is_empty() { self.narrate_talk(seat, coached.trim()); }
        // a clue landing mid-talk is marked in the moment (#1184)
        if whispered {
            if let Some(m) = &self.seats[seat].mystery {
                let def = &crate::game::mysteries::MYSTERIES[m.def];
                let got = m.known.count_ones();
                self.narrate_talk(seat, &format!(
                    "A piece of {} - {} of {} heard. Worth remembering.",
                    def.title, got, def.clues.len()
                ));
            }
        }
        // and if they were the one the job was for, it's handed over now
        let quest_note = self.quest_talk_done(seat, v);
        let mut line = format!("{}: \"{}\"{}", did, said, coached);
        if let Some(q) = quest_note {
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
                q,
            );
            line.push_str(" - and the job's done.");
        }
        line
    }

    /// The road visitor standing close enough to hail: an arrived rare
    /// event's figure inside talking reach, nearest first (#1168). A
    /// visitor only answers a Talk you walk up to - there's no calling
    /// one across the green
    fn visitor_talk_target_at(&self, seat: usize) -> Option<(Entity, f32)> {
        let (px, py) = self.player_pos_at(seat);
        let mut best: Option<(Entity, f32)> = None;
        for r in &self.rare {
            // still walking in, or already on the road home
            if !r.path.is_empty() || r.leaving { continue; }
            if events::visit_slug(r.id).is_none() { continue; }
            let Some(e) = r.e else { continue };
            let Some(p) = self.world.get_component::<Pos>(e) else { continue };
            let d = ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt();
            if d <= dialogue::PLAYER_TALK_RANGE
                && best.map(|(_, b)| d < b).unwrap_or(true) {
                best = Some((e, d));
            }
        }
        best
    }

    /// Talk with the road visitor `e`: their name, their cry for the
    /// gist, and the event's own tree - a stranger, whatever the village
    /// ledger says (#1168)
    fn talk_visitor_at(&mut self, seat: usize, e: Entity) -> String {
        if self.seats[seat].talk.as_ref().map(|t| t.who) == Some(e) {
            return "You are already speaking with them.".into();
        }
        let id = self.rare.iter().find(|r| r.e == Some(e)).map(|r| r.id)
            .expect("visitor talk target");
        let s = events::spec(id);
        let pe = self.seats[seat].player;
        let talks = crate::game::relationships::find_relationship(&self.world, pe, e)
            .and_then(|en| self.world.get_component::<crate::game::relationships::Relationship>(en))
            .map(|r| r.interactions).unwrap_or(0);
        act_talk(&mut self.world, pe, e, self.tick);
        self.say(e, s.cry, TALK_BUBBLE_SECS);
        self.open_player_talk_familiar(seat, e, crate::game::conversation::Relation::Stranger, s.cry,
            Some(talks));
        format!("You talk with {}.", s.name)
    }

    /// Distance from the player to villager `who`, in tiles
    pub fn dist_to_at(&self, seat: usize, who: usize) -> f32 {
        let (px, py) = self.player_pos_at(seat);
        self.world.get_component::<Pos>(self.villagers[who])
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt()).unwrap_or(f32::MAX)
    }

    /// The tiles to walk to reach villager `who`, round walls and buildings
    pub fn path_to_at(&self, seat: usize, who: usize) -> Option<Vec<(i32, i32)>> {
        let (px, py) = self.player_pos_at(seat);
        let p = self.world.get_component::<Pos>(self.villagers[who])?;
        self.trail_path((px.round() as i32, py.round() as i32), (p.x.round() as i32, p.y.round() as i32))
    }

    /// Talk or Recruit (action button `action`) on someone who isn't beside
    /// the player: set off to walk over to them (the action happens on
    /// arrival; see advance_approach). None when they're already close enough
    /// to act on now.
    pub fn start_approach_at(&mut self, seat: usize, who: usize, action: usize) -> Option<String> {
        if self.dist_to_at(seat, who) <= TALK_DIST { return None; }
        match self.path_to_at(seat, who) {
            Some(path) => {
                self.seats[seat].approach = Some(Approach { who, action, path, stuck: 0 });
                Some(format!("You walk over to {}.", self.cast_name(who)))
            }
            None => Some(format!("Can't {}: there's no way to reach {}.",
                ACTION_LABELS[action].to_lowercase(), self.cast_name(who))),
        }
    }

    /// One frame of walking over to someone: follow the path (finding a new
    /// one if they've wandered off), and talk or recruit once beside them
    pub fn advance_approach_at(&mut self, seat: usize) {
        let Some(mut a) = self.seats[seat].approach.take() else { return };
        let who = a.who;
        // they died on the way: nothing to say; hauled off: no one to walk to
        if crate::game::dungeon::unavailable(&self.world, self.villagers[who]) { return; }
        if self.dist_to_at(seat, who) <= TALK_DIST {
            // beside them now: take the action on them (act checks it can
            // still be done - they may have joined, or influence slipped)
            self.seats[seat].selected_target = who;
            self.seats[seat].picked = true;
            self.seats[seat].pick_near = true;
            self.act_at(seat, a.action);
            return;
        }
        let Some(&Pos { x: vx, y: vy }) = self.world.get_component::<Pos>(self.villagers[who]) else { return };
        let headed_off = a.path.last()
            .map(|&(x, y)| ((x as f32 - vx).powi(2) + (y as f32 - vy).powi(2)).sqrt() > TALK_DIST)
            .unwrap_or(true);
        if headed_off {
            match self.path_to_at(seat, who) {
                Some(path) => a.path = path,
                None => {
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Negative, Sev::Minor),
                        format!(
                            "t{}: You can't find a way to {}.",
                            self.tick,
                            self.cast_name(who)
                        ),
                    );
                    return;
                }
            }
        }
        let (px, py) = self.player_pos_at(seat);
        // drop tiles already reached; the last one is their tile, so head for them
        while a.path.len() > 1 {
            let (x, y) = a.path[0];
            if (x as f32 - px).abs() < 0.25 && (y as f32 - py).abs() < 0.25 { a.path.remove(0); } else { break; }
        }
        let (tx, ty) = match a.path.as_slice() { [(x, y), _, ..] => (*x as f32, *y as f32), _ => (vx, vy) };
        let (dx, dy) = (tx - px, ty - py);
        if dx.abs() + dy.abs() > 1e-3 { self.walk_at(seat, dx, dy); }
        a.stuck = if self.player_pos_at(seat) != (px, py) { 0 } else { a.stuck + 1 };
        if a.stuck > APPROACH_STUCK_FRAMES {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Negative, Sev::Minor),
                format!(
                    "t{}: You can't get through to {}.",
                    self.tick,
                    self.cast_name(who)
                ),
            );
            return;
        }
        self.seats[seat].approach = Some(a);
    }

    /// Take action button `i` (Talk). An index past the buttons
    /// is ignored - a stale client or recording can't panic the sim
    pub fn act_at(&mut self, seat: usize, i: usize) {
        if i >= ACTION_LABELS.len() { return; }
        let tick = self.tick;
        // Talk singles out someone to hear: the picked one, near or far -
        // you walk over to a far one - else the best near,
        // the least lately heard first (#944, #1066). A road visitor in
        // reach answers instead when they're the nearer of the two - but
        // a picked villager keeps the word they were clicked for (#1168)
        let mut visitor_talk: Option<String> = None;
        if i == 0 && self.action_blocker_at(seat, i).is_none() {
            let clicked = self.seats[seat].picked && self.seats[seat].selected_building.is_none();
            if !clicked {
                if let Some((e, vd)) = self.visitor_talk_target_at(seat) {
                    let nd = self.talk_target_at(seat)
                        .map(|t| self.dist_to_at(seat, t)).unwrap_or(f32::MAX);
                    if vd < nd { visitor_talk = Some(self.talk_visitor_at(seat, e)); }
                }
            }
            if visitor_talk.is_none() {
                if let Some(t) = self.talk_target_at(seat) {
                    self.seats[seat].selected_target = t;
                    self.seats[seat].selected_building = None;
                    self.seats[seat].picked = true;
                    self.seats[seat].pick_near = self.dist_to_at(seat, t) <= SHOUT_DIST;
                }
            }
        }
        let selected_target = self.seats[seat].selected_target;
        // a Talk's words go on the TALK panel, not the event log
        let mut talked = false;
        let msg = if let Some(reason) = self.action_blocker_at(seat, i) {
            format!("Can't {}: {}.", ACTION_LABELS[i].to_lowercase(), reason)
        } else {
            match i {
                0 => match visitor_talk {
                    Some(m) => { talked = true; m }
                    None => match self.start_approach_at(seat, selected_target, 0) {
                        Some(walking) => walking,
                        None => { talked = true; self.talk_to_at(seat, selected_target) }
                    },
                },
                _ => unreachable!("no action button {}", i),
            }
        };
        // every action line is dated like the rest of the log. It doesn't
        // show a score: the log reads as what happened in the village
        if !talked {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                format!("t{}: {}", tick, msg),
            );
        }
        // gold spent here is the same purse the day's trading counts
        let pe = self.seats[seat].player; sync_purse(&mut self.world, pe, true);
    }

    /// What a ctx button reads: `ctx_button`'s static label, except the
    /// kind hand names the animal it reaches for (#1056)
    pub fn ctx_label_at(&self, seat: usize, act: CtxAct) -> String {
        match act {
            CtxAct::Tame => self.tame_animal_at(seat)
                .map(|(_, s)| format!("Pet {}", match s {
                    Species::Sheep => "the sheep",
                    Species::Goat => "the goat",
                    Species::Chicken => "the hen",
                    _ => "the stray",
                }))
                .unwrap_or_else(|| ctx_button(act).label),
            _ => ctx_button(act).label,
        }
    }

    /// The makings a bench day is short of, phrased how to come by them
    /// (#1338): "wheat - gather some" when the hedgerow gives it, else
    /// "- buy some"
    fn bench_missing(&self, pe: Entity) -> Vec<String> {
        let (voc, workplace) = self.world.get_component::<Craft>(pe)
            .map(|c| (Some(c.vocation), c.workplace)).unwrap_or((None, None));
        let Some(voc) = voc else { return vec![] };
        vocation::inputs(voc).iter().copied().filter(|g| {
            // a slot's only missing when nothing that fills it is to
            // hand - the named good or its fellows
            std::iter::once(*g).chain(crate::game::recipes::fillers(g).iter().copied())
                .all(|c| {
                    let held = self.world.get_component::<Satchel>(pe)
                        .map(|s| s.goods.get(c).copied().unwrap_or(0)).unwrap_or(0);
                    let stocked = workplace
                        .and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(c).copied().unwrap_or(0)).unwrap_or(0);
                    held + stocked < 1
                })
        }).map(|g| {
            let opts: Vec<&'static str> = std::iter::once(g)
                .chain(crate::game::recipes::fillers(g).iter().copied()).collect();
            let what = opts.join(" or ");
            if opts.iter().any(|o| vocation::gatherable(o)) { format!("{} - gather some", what) }
            else { format!("{} - buy some", what) }
        }).collect()
    }

    /// One day's work at the bench, and the words for where it went:
    /// a full shelf spills to the satchel, a tasked hand's share runs
    /// through the senior's shop, a full satchel loses it
    fn work_bench_msg(&mut self, pe: Entity, tick: u64) -> String {
        let (voc, workplace, tasked_by) = self.world.get_component::<Craft>(pe)
            .map(|c| (Some(c.vocation), c.workplace, c.tasked_by)).unwrap_or((None, None, None));
        let what = voc.and_then(vocation::output).unwrap_or("goods");
        let held_before = self.world.get_component::<Satchel>(pe)
            .map(|s| s.goods.get(what).copied().unwrap_or(0)).unwrap_or(0);
        let stocked_before = workplace
            .and_then(|b| self.world.get_component::<Building>(b))
            .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
        let senior_shop = tasked_by
            .and_then(|s| self.world.get_component::<Craft>(s))
            .and_then(|c| c.workplace);
        let senior_before = senior_shop
            .and_then(|b| self.world.get_component::<Building>(b))
            .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
        let gold_before = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
        let (made, up) = self.bench_work(pe, tick);
        match up {
            Some(l) => {
                let lvl = vocation::level_name(l);
                let title = self.world.get_component::<Craft>(pe)
                    .map(|c| format!("{} {}", lvl, vocation::vocation_name(c.vocation)))
                    .unwrap_or_else(|| lvl.to_string());
                format!("You work - and reach {}!", title)
            }
            None if made > 0.0 => {
                let held = self.world.get_component::<Satchel>(pe)
                    .map(|s| s.goods.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - held_before as i32;
                let stocked = workplace
                    .and_then(|b| self.world.get_component::<Building>(b))
                    .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - stocked_before as i32;
                // an heir keeps the senior's bench: when it's the
                // same shelf, its take is already counted above
                let senior = if senior_shop == workplace { 0 } else {
                    senior_shop.and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - senior_before as i32
                };
                let gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0) - gold_before;
                let shop = workplace.and_then(|b| self.world.get_component::<Building>(b))
                    .map(|b| b.name.as_str()).unwrap_or("the shop");
                let mut parts: Vec<String> = vec![];
                if stocked > 0 { parts.push(format!("{} stocked at {}", stocked, shop)); }
                if held > 0 { parts.push(format!("{} in your satchel", held)); }
                if senior > 0 {
                    let sname = tasked_by.and_then(|s| self.villagers.iter().position(|v| *v == s))
                        .map(|i| self.cast_name(i)).unwrap_or("your senior");
                    parts.push(format!("{} through {}'s shop", senior, sname));
                }
                if gold > 0.0 { parts.push(format!("{:.0}g earned", gold)); }
                let lost = made as i32 - stocked - held - senior;
                if lost > 0 { parts.push(format!("{} spoiled - no room for it", lost)); }
                let went = if parts.is_empty() { String::new() } else { format!(" - {}", parts.join(", ")) };
                format!("You work - {:.0} {} made{}.", made, what, went)
            }
            None => "You put in the day, but make nothing yet.".into(),
        }
    }

    /// Ambient craft (#1338): the work is being there, not a button. A
    /// day at the bench banks a day's presence - AMBIENT_WORK_DAYS of
    /// standing makes one day's batch, gentler than the click's - and
    /// a bench short of its makings says so once a day
    pub(in crate::session) fn ambient_bench_at(&mut self, seat: usize) {
        if self.paused || self.seats[seat].talk.is_some() { return; }
        let pe = self.seats[seat].player;
        let bench = self.world.get_component::<Craft>(pe).and_then(|c| c.workplace);
        let Some(bench) = bench else { self.seats[seat].ambient_bench = 0.0; return; };
        let (px, py) = self.player_pos_at(seat);
        let near = self.world.get_component::<Pos>(bench)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < 4.0).unwrap_or(false);
        // banked days keep; being elsewhere or talking just doesn't add
        if !near { return; }
        self.seats[seat].ambient_bench += 1.0 / self.ticks_per_frame() as f32;
        if self.seats[seat].ambient_bench < AMBIENT_WORK_DAYS { return; }
        let tick = self.tick;
        let missing = self.bench_missing(pe);
        if !missing.is_empty() {
            // the bench's want has a home in the log, once a day - the
            // button's "you can't work" nag, ambient now
            if self.seats[seat].bench_nudged != tick {
                self.seats[seat].bench_nudged = tick;
                self.log.push_ev(
                    EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                    format!("t{}: The bench stands idle: no {} to hand.", tick, missing.join(", ")));
            }
            return;
        }
        self.seats[seat].ambient_bench = 0.0;
        let msg = self.work_bench_msg(pe, tick);
        self.log.push_ev(
            EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
            format!("t{}: {}", tick, msg));
    }

    /// The contextual actions that apply right now: pair a picked
    /// junior with a master, send a kid on an errand
    pub fn update_ctx_actions_at(&mut self, seat: usize) {
        self.seats[seat].ctx_actions.clear();
        let (px, py) = self.player_pos_at(seat);
        if self.seats[seat].selected_building.is_none() {
            let v = self.villagers[self.seats[seat].selected_target];
            if self.world.get_component::<Kid>(v).is_some() {
                self.seats[seat].ctx_actions.push(CtxAct::Errand);
            }
            // you know two hearts that are looking: put a word between them
            let pe = self.seats[seat].player;
            let lonely_known = self.world.get_component::<desires::Organizer>(pe)
                .map(|o| o.heard.iter()
                    .filter(|(t, _, u)| *t == "loneliness" && *u > self.tick)
                    .map(|(_, e, _)| *e).collect::<crate::engine::det::HashSet<_>>().len())
                .unwrap_or(0);
            if lonely_known >= 2 {
                self.seats[seat].ctx_actions.push(CtxAct::Matchmake);
            }
            let in_earshot = self.world.get_component::<Pos>(v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= TALK_DIST)
                .unwrap_or(false);
            // they've asked you in: the place at their table (#1241)
            if in_earshot && self.table_offered(seat, v) {
                self.seats[seat].ctx_actions.push(CtxAct::Dine);
            }
            // a deep bond can be asked after the wound they carry (#1243)
            if in_earshot && !villager::is_dead(&self.world, v) && crate::game::hooks::open(&self.world, v).is_some()
                && crate::game::organizing::stage_of(&self.world, self.seats[seat].player, v) >= crate::game::organizing::RelStage::Follower {
                self.seats[seat].ctx_actions.push(CtxAct::AskAfter);
            }
            // a solved mystery wants delivering to whoever's been asking
            if let Some(m) = &self.seats[seat].mystery {
                if m.solved() && v == m.asker {
                    self.seats[seat].ctx_actions.push(CtxAct::ShareFindings);
                }
            }
            // the paper regime's asks (#1261): a held member's book
            // wants marking free; a masterless one wants a forgery;
            // a carried pass can be reported away
            let pe = self.seats[seat].player;
            let org = self.seats[seat].org;
            let org_has = |x| self.world.get_component::<crate::sim::community::Organization>(org)
                .map(|o| o.members.contains(&x)).unwrap_or(false);
            if org_has(v) && crate::game::pass::held(&self.world, v) {
                self.seats[seat].ctx_actions.push(CtxAct::MarkBook);
            }
            if self.world.get_component::<crate::game::pass::Masterless>(v).is_some()
                && self.villagers.iter().any(|&m| org_has(m) && self.literate(m)) {
                self.seats[seat].ctx_actions.push(CtxAct::ForgePass);
            }
            if self.world.get_component::<crate::game::pass::Pass>(v).is_some()
                && !org_has(v) && org_has(pe) {
                self.seats[seat].ctx_actions.push(CtxAct::ReportMasterless);
            }
        }
        // a stray cat keeping you company can be stroked - "Pet her"
        // shows while she's in reach, whatever else is going on
        if self.stray_cat_at(seat).is_some() {
            self.seats[seat].ctx_actions.push(CtxAct::PetCat);
        }
        // and a fold beast or yard hen in reach takes the kind hand too -
        // it shares her slot, and wins it when they're both about
        if self.tame_animal_at(seat).is_some() {
            self.seats[seat].ctx_actions.push(CtxAct::Tame);
        }
        // a cache the bands left: standing by it offers the taking
        if self.rebel_store_at(seat).is_some() {
            self.seats[seat].ctx_actions.push(CtxAct::TakeCache);
        }
        // a meeting the word alone convenes earns no button (#1596):
        // the call's day is picked in talk now. The acts below still
        // stand on the same gate - a ready plan, no meeting under way
        if self.meeting.is_none()
            && self.world.get_component::<crate::game::organizing::MeetingPlan>(self.seats[seat].org)
                .map(|p| p.ready()).unwrap_or(false) {
            // a gathering today the player can join: the service, the
            // feast, or a celebrant's day - standing in it counts
            if routine::is_holy(self.tick) || self.tick % 90 == 45
                || self.villagers.iter().any(|&v|
                    self.world.get_component::<crate::game::lifecycle::Age>(v)
                        .map(|a| a.born_day == (self.tick % 360) as u16).unwrap_or(false)) {
                self.seats[seat].ctx_actions.push(CtxAct::Attend);
            }
            // the water's verbs: board at a bank, disembark at a bank,
            // and a line off the boat where the river runs
            if self.seats[seat].aboard.is_some() {
                self.seats[seat].ctx_actions.push(CtxAct::Disembark);
                self.seats[seat].ctx_actions.push(CtxAct::Fish);
            } else if self.boats.iter().any(|&(bx, by)| {
                let p = self.world.get_component::<Pos>(self.seats[seat].player).copied()
                    .unwrap_or(Pos { x: 0.0, y: 0.0 });
                (p.x.round() as i32 - bx).abs() + (p.y.round() as i32 - by).abs() < 4 }) {
                self.seats[seat].ctx_actions.push(CtxAct::Board);
            }
        }
        // winter's own button: once a day, everyone free can be called out
        // to a snowball fight - it shares Gather's row on the panel
        if self.world.get_component::<crate::sim::climate::Climate>(self.climate)
            .map(|c| c.season == crate::sim::climate::Season::Winter).unwrap_or(false)
            && self.snowball_day != self.tick {
            self.seats[seat].ctx_actions.push(CtxAct::Snowballs);
        }
        // the cause's verbs aren't buttons any more (#1337): the cadence
        // tick sits the floor on its own dates, and a called gathering
        // resolving on a due cause opens the slate - the panel motions
        // carry what the buttons did
        if self.seats[seat].selected_building.is_none()
            && (self.seats[seat].picked || self.seats[seat].agent) {
            let v = self.villagers[self.seats[seat].selected_target];
            // the cause's target can be sounded out: the interview earns
            // their stance and the stake that puts them there
            if let Some(ci) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) {
                if self.world.get_component::<crate::game::campaign::Campaign>(ci)
                    .map(|cc| cc.target == Some(v)).unwrap_or(false) {
                    self.seats[seat].ctx_actions.push(CtxAct::Interview);
                }
            }
            if !crate::game::villager::is_dead(&self.world, v) {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                    // someone's inside: the way out is pressure - the
                    // letters go on whether the cause still marches or not
                    self.seats[seat].ctx_actions.push(CtxAct::Petition);
                } else if !crate::game::dungeon::unavailable(&self.world, v) {
                    let is_member = self.world.get_component::<Organization>(self.seats[seat].org)
                        .map(|o| o.members.contains(&v)).unwrap_or(false);
                    // a leader - one who's brought someone in - can go out
                    // and hold a one-on-one of their own
                    if is_member && crate::game::leadership::rung(&self.world, self.seats[seat].org, v)
                            == crate::game::leadership::Rung::Leader {
                        self.seats[seat].ctx_actions.push(CtxAct::Reach);
                    }
                    if let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) {
                        // seats go to the picked member, but only while one
                        // stands empty - a full roster offers nothing
                        if is_member && [crate::game::campaign::Role::TurnoutCoordinator,
                            crate::game::campaign::Role::NoteTaker,
                            crate::game::campaign::Role::Chair].iter().copied()
                            .any(|r| crate::game::campaign::seat_holder(&self.world, c, r).is_none()) {
                            self.seats[seat].ctx_actions.push(CtxAct::Delegate);
                        }
                    }
                }
            }
        }
        // a fire close enough to throw on: the player can pitch in -
        // pushed last so it wins the shared row slot over a cat or a
        // snowball
        if self.fire_douse_spot_at(seat).is_some() {
            self.seats[seat].ctx_actions.push(CtxAct::Douse);
        }
    }

    /// Take a contextual action (entry `i` of ctx_actions)
    pub fn ctx_act_at(&mut self, seat: usize, i: usize) {
        let Some(&act) = self.seats[seat].ctx_actions.get(i) else { return };
        let tick = self.tick;
        if let Some(reason) = self.ctx_blocker_at(seat, act) {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Negative, Sev::Minor),
                format!(
                    "t{}: Can't {}: {}.",
                    tick,
                    self.ctx_label_at(seat, act).to_lowercase(),
                    reason
                ),
            );
            return;
        }
        let msg = match act {
            CtxAct::PairUp => self.pair_up_at(self.seats[seat].selected_target),
            CtxAct::Matchmake => {
                let v = self.villagers[self.seats[seat].selected_target];
                self.match_attempt(v, tick, Some(self.seats[seat].player), "You");
                "You have a word in both their ears.".into()
            }
            CtxAct::Errand => {
                let k = self.villagers[self.seats[seat].selected_target];
                let player = self.seats[seat].player;
                // the player's own needs first; with none going spare,
                // the print shop's - a kid can run paper and ink in for
                // the printer, paid from the printer's own purse. A
                // printer already hiring for the same good, or locked
                // up, can't take another job
                let printer = self.villagers.iter().copied()
                    .filter(|&p| p != k && !villager::is_dead(&self.world, p)
                        && !crate::game::dungeon::unavailable(&self.world, p)
                        && self.world.get_component::<Kid>(p).is_none()
                        && self.world.get_component::<Craft>(p)
                            .map(|c| c.vocation == Vocation::Printmaker).unwrap_or(false))
                    .find_map(|p| {
                        let g = livelihood::needs_input(&self.world, p)?;
                        let hired = self.villagers.iter().any(|&e| {
                            e != p && self.world.get_component::<OddJob>(e)
                                .map(|j| j.hirer == p && j.good == g).unwrap_or(false)
                        });
                        (!hired).then_some((p, g))
                    });
                let fetch = |hirer: Entity, g: &'static str| {
                    let pos = *self.world.get_component::<Pos>(hirer)?;
                    livelihood::seller_for(&self.world, g, pos)
                        .filter(|&s| !self.boycotted(s))
                        .map(|from| (hirer, g, from))
                };
                let job = livelihood::needs_input(&self.world, player)
                    .and_then(|g| fetch(player, g))
                    .or_else(|| printer.and_then(|(hirer, g)| fetch(hirer, g)));
                match job {
                    Some((hirer, g, from)) => {
                        self.world.add_component(k, OddJob {
                            hirer, good: g.to_string(), qty: 2, from, to: hirer,
                            pay: 3.0, loaded: false });
                        // a fresh errand supersedes the day's walk
                        self.travels.retain(|t| t.who != k);
                        self.send(k, Goal::RunOddJob);
                        if hirer == player {
                            format!("You send {} to fetch {}.", self.cast_name(self.seats[seat].selected_target), g)
                        } else {
                            let pname = self.villagers.iter().position(|&p| p == hirer)
                                .map(|i| cast_of(&self.names, i)).unwrap_or("the printer");
                            format!("You send {} to fetch {} for {}.",
                                self.cast_name(self.seats[seat].selected_target), g, pname)
                        }
                    }
                    None => "Nothing needs fetching right now.".into(),
                }
            }
            CtxAct::Delegate => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let v = self.villagers[self.seats[seat].selected_target];
                let name = self.cast_name(self.seats[seat].selected_target);
                use crate::game::campaign::Role;
                let role_name = |r: Role| match r {
                    Role::TurnoutCoordinator => "turnout coordinator",
                    Role::NoteTaker => "note taker",
                    Role::Chair => "chair",
                };
                // a seated member doesn't get re-seated silently - their
                // old seat would empty behind the new one
                let already = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .and_then(|c| c.seats.iter().find(|s| s.holder == v)).map(|s| s.role);
                match already {
                    Some(r) => format!("{} already holds the {} seat.", name, role_name(r)),
                    // the first seat standing empty: turnout, notes, then chair
                    None => {
                        let open = [Role::TurnoutCoordinator, Role::NoteTaker, Role::Chair].iter().copied()
                            .find(|&r| crate::game::campaign::seat_holder(&self.world, c, r).is_none());
                        match open {
                            Some(_) if !self.suffrage_won
                                && self.world.get_component::<villager::Sex>(v)
                                    .map(|s| *s == villager::Sex::Female).unwrap_or(false) =>
                                format!("{} can't hold a seat - not while a woman's say in the village goes unasked.", name),
                            Some(r) if crate::game::campaign::delegate_role(&mut self.world, c, r, v) => {
                                // naming a seat teaches its craft: a mentor
                                // who's built it hands the trade down
                                crate::game::org_skills::teach(&mut self.world,
                                    Some(c), self.seats[seat].player, v,
                                    crate::game::org_skills::role_skill(r), tick);
                                format!("You name {} the campaign's {}.", name, role_name(r))
                            }
                            Some(_) => format!("{} can't hold a seat - they have to be a living member.", name),
                            None => "Every seat is spoken for - someone would have to step down.".into(),
                        }
                    }
                }
            }
            // strike and townhall aren't buttons any more - they're
            // motions the floor votes (#1157); the arms stay for any
            // wire-side CtxAct that still comes in
            CtxAct::Strike => self.strike_at(seat),
            CtxAct::Reach => {
                let v = self.villagers[self.seats[seat].selected_target];
                let name = self.cast_name(self.seats[seat].selected_target);
                let org = self.seats[seat].org;
                match crate::game::leadership::reach_target(&self.world, org, v, &self.villagers) {
                    Some(t) => {
                        let tn = self.name_of(t);
                        self.world.add_component(v, crate::game::leadership::Reach {
                            org, target: t, due: tick + crate::game::leadership::REACH_DAYS });
                        format!("{} goes out to sit down with {} - on their own, without you.", name, tn)
                    }
                    None => format!("{} has no one left to ask.", name),
                }
            }
            CtxAct::Townhall => self.townhall_at(seat),
            CtxAct::Gather => {
                // wire-only on a new server (#1596) - never pushed, so
                // unreachable in practice; if it fires, the date
                // question is still the right landing
                self.seats[seat].meet_offer = Some(self.tick);
                "The plan's made - now the word wants a day.".into()
            }
            CtxAct::Interview => {
                let t = self.villagers[self.seats[seat].selected_target];
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)
                else {
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                        format!("t{}: no cause to sound anyone out for.", tick),
                    );
                    return;
                };
                let Some(target) = self
                    .world
                    .get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.target)
                    .unwrap_or(None)
                else {
                    return;
                };
                if target != t {
                    return;
                }
                // rapport earns the hearing: a stranger's stance is a
                // wall's, a warm ear's is an answer
                let warm = crate::game::player::rapport(&self.world, self.seats[seat].player, t);
                if warm < 0.3 { return; }
                // the stance is computed, deterministic: their own
                // concerns decide where they land on the cause's issue
                let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.issue.clone()).unwrap_or_default();
                let shares = self.world.get_component::<desires::InnerLife>(t)
                    .map(|l| l.grievances.iter().any(|g|
                        desires::issue_tag(&self.world, t, &desires::Reveal::Problem(g.0)) == issue))
                    .unwrap_or(false);
                let stance = if shares { "for it" }
                    else if self.world.get_component::<desires::InnerLife>(t)
                        .map(|l| !l.grievances.is_empty()).unwrap_or(false) { "against it" }
                    else { "undecided" };
                // the stake is their top concern - the thing that names
                // where they'd put the cause's weight
                let stake = self.world.get_component::<desires::InnerLife>(t)
                    .and_then(|l| l.grievances.first().map(|g|
                        desires::issue_tag(&self.world, t, &desires::Reveal::Problem(g.0))))
                    .unwrap_or_else(|| issue.clone());
                if let Some(cc) = self.world.get_component_mut::<crate::game::campaign::Campaign>(c) {
                    cc.dossier = Some(crate::game::campaign::Dossier {
                        target: t, stance, stake: stake.clone() });
                }
                format!("You learn where {} stands on it - {} - and what's behind it: {}.",
                    self.name_of(t), stance, stake)
            }
            CtxAct::Attend => {
                // the player's presence counts for what the day means:
                // a service, a feast, a birthday - the gathering's warmth
                // lands on whoever stands inside it
                let player = self.seats[seat].player;
                // whose gathering is at hand: anyone holding a slot today
                // at worship or feast, or the celebrant whose birthday it is
                let gathering = routine::is_holy(tick) || tick % 90 == 45
                    || self.villagers.iter().any(|&v|
                        self.world.get_component::<routine::Routine>(v)
                            .map(|r| r.day == tick
                                && matches!(r.mode, routine::Mode::Worshipping | routine::Mode::Feasting))
                            .unwrap_or(false));
                let birthday = self.villagers.iter().any(|&v|
                    self.world.get_component::<crate::game::lifecycle::Age>(v)
                        .map(|a| a.born_day == (tick % 360) as u16).unwrap_or(false));
                if !gathering && !birthday {
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                        format!("t{}: there's no gathering to join.", tick),
                    );
                    return;
                }
                // attending once per day - the day's own rite counts once
                let already = self
                    .attended_days
                    .iter()
                    .any(|&(p, d)| p == player && d == tick);
                if already {
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                        format!("t{}: you've already been to the day's gathering.", tick),
                    );
                    return;
                }
                self.attended_days.push((player, tick));
                // the warmth lands: a small devotion for the standing, a
                // rapport for the company, the want for the gathering met
                if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(player) {
                    l.wants.retain(|w| *w != desires::Want::Faith);
                }

                // and the folk standing with you warm a little: presence
                // is the bond's own language
                for v in self.villagers.iter().copied().collect::<Vec<_>>() {
                    if self.world.get_component::<Pos>(v).zip(self.world.get_component::<Pos>(player))
                        .map(|(a, b)| (a.x - b.x).abs() + (a.y - b.y).abs() < 12.0).unwrap_or(false) {
                        crate::game::player::note_conversation(&mut self.world, player, v, tick);
                    }
                }
                self.log.push_ev(
                    EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                    format!(
                        "t{}: you stand in the gathering - the day's rite counts for you.",
                        tick
                    ),
                );
                "".into()
            }
            CtxAct::Board => {
                // take the boat at the bank: the water carries you
                if let Some(bi) = self.boats.iter().position(|&(bx, by)| {
                    let p = self.world.get_component::<Pos>(self.seats[seat].player).copied()
                        .unwrap_or(Pos { x: 0.0, y: 0.0 });
                    (p.x.round() as i32 - bx).abs() + (p.y.round() as i32 - by).abs() < 4 }) {
                    self.seats[seat].aboard = Some(bi);
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                        format!("t{}: you push off - the river takes the boat.", tick),
                    );
                }
                "".into()
            }
            CtxAct::Disembark => {
                // put in at the bank: the water lets you off
                if self.seats[seat].aboard.is_some() {
                    self.seats[seat].aboard = None;
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                        format!("t{}: you put in - the bank's under your feet.", tick),
                    );
                }
                "".into()
            }
            CtxAct::Fish => {
                // a line off the boat: the river feeds you where it runs
                if self.seats[seat].aboard.is_some() {
                    if let Some(s) = self.world.get_component_mut::<Satchel>(self.seats[seat].player) {
                        let good = if tick % 365 >= 180 && tick % 365 < 270 { "eel" }
                            else { "fish" };
                        *s.goods.entry(good.into()).or_insert(0) += 1;
                    }
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                        format!(
                            "t{}: you drop a line - the water gives you {}.",
                            tick,
                            if tick % 365 >= 180 && tick % 365 < 270 {
                                "an eel"
                            } else {
                                "a fish"
                            }
                        ),
                    );
                }
                "".into()
            }
            CtxAct::Petition => {
                let v = self.villagers[self.seats[seat].selected_target];
                // every press is a letter scratched out - the quill
                // scratches however often the day hears about it
                let (qx, qy) = self.player_pos_at(seat);
                self.emit_sfx("quill", qx, qy, 0.5);
                // the letters are the mechanic - every press still lands -
                // but the log hears about them once a day, not per click
                let repeat = self.petition_day == self.tick;
                self.petition_day = self.tick;
                // the cause's muscle signs the letters too - every member's
                // name on the vigil carries more than a stranger's plea
                let sway = crate::sim::community::exert_influence(&mut self.world,
                    self.seats[seat].org, "the release");
                crate::game::org_skills::practice(&mut self.world, self.seats[seat].player,
                    crate::game::org_skills::OrgSkill::Agitation,
                    crate::game::org_skills::SEAT_PRACTICE);
                crate::game::dungeon::press(&mut self.world, v, 0.3 + sway * 0.15);
                // the hold: what the state spends to keep them inside -
                // the cell opening is news however often the letters go
                let freed = crate::game::dungeon::release_check(&mut self.world, v, 1.0);
                let said = if freed {
                    let name = self.cast_name(self.seats[seat].selected_target);
                    self.toasts.push_at((format!("{} walks free", name), tick + 40), toast_spot(&self.world, v));
                    format!("The letters don't stop - and the cell opens. {} walks free.", name)
                } else {
                    let p = self.world.get_component::<crate::game::dungeon::ReleaseCampaign>(v)
                        .map(|r| r.pressure).unwrap_or(0.0);
                    format!("Letters, vigils, a line out the door ({:.0}% there).", p * 100.0)
                };
                if repeat && !freed { String::new() } else { said }
            }
            // wire-only on a new server (#1594-#1597): never pushed, so
            // unreachable in practice; the sim helpers still do the work
            CtxAct::PickCaps => self.pick_caps_at(seat),
            CtxAct::Give => {
                let v = self.villagers[self.seats[seat].selected_target];
                self.give_at(self.seats[seat].player, v)
            }
            CtxAct::Pact => {
                let v = self.villagers[self.seats[seat].selected_target];
                self.pact_at(self.seats[seat].player, v)
            }
            CtxAct::TakeCache => {
                match self.rebel_store_at(seat).map(|si| self.rebel_stores.remove(si)) {
                    Some(st) => {
                        let pe = self.seats[seat].player;
                        if let Some(s) = self.world.get_component_mut::<Satchel>(pe) {
                            for (g, n) in &st.goods {
                                *s.goods.entry(g.to_string()).or_insert(0) += n;
                            }
                        }
                        if st.gold > 0.0 {
                            if let Some(p) = self.world.get_component_mut::<Purse>(pe) { p.gold += st.gold; }
                            if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(pe) { p.resources += st.gold; }
                        }
                        let goods = st.goods.iter().map(|&(g, n)| format!("{} {}", n, g))
                            .collect::<Vec<_>>().join(" and ");
                        let what = if st.gold > 0.0 && !goods.is_empty() {
                            format!("{:.0} gold and {}", st.gold, goods)
                        } else if st.gold > 0.0 {
                            format!("{:.0} gold", st.gold)
                        } else {
                            goods
                        };
                        self.log.push_ev(
                            EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                            format!(
                                "t{}: you take the cache the rebels left - {}.",
                                self.tick, what
                            ),
                        );
                        format!("Taken: {}.", what)
                    }
                    None => String::new(),
                }
            }
            CtxAct::ShareFindings => self.mystery_deliver_at(seat),
            CtxAct::Dine => self.sit_at_table(seat),
            CtxAct::AskAfter => self.ask_after_at(seat),
            CtxAct::MarkBook => {
                // the liberation ask: their book's marked free - out
                // from under the master, and paperless with it (#1261)
                let v = self.villagers[self.seats[seat].selected_target];
                if let Some(c) = self.world.get_component_mut::<Craft>(v) { c.tasked_by = None; }
                self.world.remove_component::<crate::game::pass::Pass>(v);
                self.world.add_component(v, crate::game::pass::Masterless);
                let n = self.name_of(v);
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), v,
                    format!("t{}: {}'s book is marked free - out of service, and paperless.", tick, n));
                format!("{}'s book is marked free - they're out, and masterless.", n)
            }
            CtxAct::ForgePass => {
                // the hidden scribe's real work: a literate member
                // writes the masterless one a pass (#1261)
                let v = self.villagers[self.seats[seat].selected_target];
                let org = self.seats[seat].org;
                let scribe = self.villagers.iter().copied().find(|&m| {
                    self.world.get_component::<crate::sim::community::Organization>(org)
                        .map(|o| o.members.contains(&m)).unwrap_or(false) && self.literate(m)
                });
                match scribe {
                    Some(s) => {
                        self.world.remove_component::<crate::game::pass::Masterless>(v);
                        self.world.add_component(v, crate::game::pass::Pass {
                            marked_by: Some(s), since: tick, forged: true });
                        let n = self.name_of(v);
                        format!("{} carries a fresh pass - {}'s hand wrote it.", n, self.name_of(s))
                    }
                    None => "No one in the cause can write it.".into(),
                }
            }
            CtxAct::ReportMasterless => {
                // the org's own sanction: the word to the watch that
                // their paper's no good - and word gets out who spoke
                // (#1261)
                let v = self.villagers[self.seats[seat].selected_target];
                let pe = self.seats[seat].player;
                self.world.remove_component::<crate::game::pass::Pass>(v);
                if let Some(c) = self.world.get_component_mut::<Craft>(v) { c.tasked_by = None; }
                self.world.add_component(v, crate::game::pass::Masterless);
                crate::game::relationships::record_interaction(&mut self.world, pe,
                    crate::game::relationships::Interaction { other: v,
                        kind: crate::game::relationships::InteractionKind::Conflict, intensity: 0.6 });
                let n = self.name_of(v);
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), v,
                    format!("t{}: word reaches the watch - {} has no mark worth showing.", tick, n));
                format!("{}'s reported masterless - they'll feel it, and word gets out who spoke.", n)
            }
            CtxAct::PetCat => self.pet_stray_cat_at(seat),
            CtxAct::Tame => self.tame_stroke_at(seat),
            CtxAct::Snowballs => self.snowball_fight_at(seat),
            CtxAct::Douse => self.player_douse_at(seat),
        };
        if !msg.is_empty() {
            self.log.push_ev(
                EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                format!("t{}: {}", tick, msg),
            );
        }
    }

    /// Hand `v` something out of `pe`'s satchel: the meal for the
    /// hungry, the thing they're after, else the best of the bag. The
    /// Give button's gone (#1594); the giving itself stays for whoever
    /// has it to do. The job's cargo isn't yours to hand round (#1603)
    pub fn give_at(&mut self, pe: Entity, v: Entity) -> String {
        let job_good = self.quest_cargo(pe);
        let bag: Vec<String> = self.world.get_component::<Satchel>(pe)
            .map(|s| s.goods.iter()
                .filter(|(g, &n)| n > 0 && Some(g.as_str()) != job_good)
                .map(|(g, _)| g.clone()).collect())
            .unwrap_or_default();
        let edible = |g: &str| goods::spec(g)
            .map(|sp| [goods::Tag::StapleFood, goods::Tag::Fruit, goods::Tag::Sweet]
                .iter().any(|t| sp.tags.contains(t)))
            .unwrap_or(false);
        let their_hungry = self.world.get_component::<Hunger>(v)
            .map(|h| h.value < 30.0).unwrap_or(false);
        // the gift picks itself: a meal for the hungry, the
        // thing they're after, else the best of the bag
        let gift = if their_hungry { bag.iter().find(|g| edible(g)).cloned() } else { None }
            .or_else(|| {
                let wants: Vec<desires::Want> = self.world.get_component::<desires::InnerLife>(v)
                    .map(|l| l.wants.iter().copied()
                        .filter(|w| !l.satisfied.iter().any(|(s, _)| *s == *w)).collect())
                    .unwrap_or_default();
                bag.iter().find(|g| wants.iter().any(|w| match w {
                    desires::Want::GoodFood => edible(g),
                    desires::Want::Finery => goods::spec(g).map(|s| s.tags.contains(&goods::Tag::Finery)).unwrap_or(false),
                    desires::Want::Drink => goods::spec(g).map(|s| s.tags.contains(&goods::Tag::Drink)).unwrap_or(false),
                    desires::Want::Instrument => g.as_str() == "lute",
                    desires::Want::BetterTools => g.as_str() == "tools",
                    _ => false,
                })).cloned()
            })
            .or_else(|| bag.iter()
                .max_by(|a, b| goods::worth_at(&self.world, a)
                    .partial_cmp(&goods::worth_at(&self.world, b)).unwrap_or(std::cmp::Ordering::Equal))
                .cloned());
        match gift {
            Some(g) => {
                let worth = goods::worth_at(&self.world, &g);
                if let Some(s) = self.world.get_component_mut::<Satchel>(pe) {
                    if let Some(n) = s.goods.get_mut(&g) { *n -= 1; if *n == 0 { s.goods.remove(&g); } }
                }
                if let Some(s) = self.world.get_component_mut::<Satchel>(v) {
                    *s.goods.entry(g.clone()).or_insert(0) += 1;
                }
                let shared_meal = edible(&g);
                let kind = if shared_meal { crate::game::relationships::InteractionKind::SharedMeal }
                    else { crate::game::relationships::InteractionKind::Aid };
                crate::game::relationships::record_interaction(&mut self.world, pe,
                    crate::game::relationships::Interaction {
                        other: v, kind, intensity: (worth / 10.0).clamp(0.5, 2.0) });
                if shared_meal {
                    crate::game::episodes::file(&mut self.world, v, pe,
                        crate::game::episodes::MemKind::SharedMeal, self.tick, &["meal", "gift"]);
                }
                let who = self.villagers.iter().position(|&x| x == v)
                    .map(|i| self.cast_name(i)).unwrap_or_else(|| "them".into());
                format!("you give {} the {} - and it warms them.", who, g)
            }
            None => "your satchel's empty - nothing to give.".into(),
        }
    }

    /// The oath's price of entry: they've heard you speak, and the purse
    /// can cover the wax (#1061). The button's gone (#1595) but the gate
    /// still stands for whoever swears it
    pub(in crate::session) fn pact_blocker(&self, pe: Entity, v: Entity) -> Option<String> {
        if crate::game::player::rapport(&self.world, pe, v) <= 0.0 {
            Some("they don't know you - talk first".into())
        } else {
            let gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
            (gold < crate::game::contracts::PACT_AID)
                .then(|| format!("the oath's wax costs {:.0} gold", crate::game::contracts::PACT_AID))
        }
    }

    /// Press wax with `v`: ten gold of aid, sworn both ways. The Swear
    /// aid button's gone (#1595); the oath itself stays in the sim for
    /// the agents who still seal pacts between them
    pub(in crate::session) fn pact_at(&mut self, pe: Entity, v: Entity) -> String {
        if let Some(reason) = self.pact_blocker(pe, v) { return reason; }
        let mut gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
        match crate::game::contracts::seal_aid_pact(&mut self.world, pe, v, &mut gold) {
            Some(_) => {
                // both purses: trades move the Purse, the panel
                // reads resources - a build only syncs one way
                if let Some(p) = self.world.get_component_mut::<Purse>(pe) { p.gold = gold; }
                if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(pe) { p.resources = gold; }
                let n = self.name_of(v);
                self.say(v, "It's sworn - you've a friend in me.", TALK_BUBBLE_SECS);
                if let Some(vi) = self.villagers.iter().position(|&x| x == v) {
                    note_story(&mut self.tales, &mut self.tales_told, &mut self.log, &self.names,
                        vi, self.tick,
                        crate::sim::story::EventKind::Alliance, "sworn to aid the Wanderer".into());
                }
                format!("You press wax with {} - sworn to aid each other ({:.0}g).", n, crate::game::contracts::PACT_AID)
            }
            None => "There's no sealing it - too poor, or sworn already.".into(),
        }
    }

    /// Pick a cap off the flushed patch `seat`'s player stands by, if the
    /// belly doesn't know it for a killer - the Pick caps button's gone
    /// (#1597); agents forage the same patches through their own ask.
    /// Empty on a taken pick, the reason when there's nothing safe to take
    pub(in crate::session) fn pick_caps_at(&mut self, seat: usize) -> String {
        let Some(pi) = self.shroom_patch_at(seat) else { return String::new() };
        if self.lore_of(self.seats[seat].player).contains(&pi) {
            return "those caps are death - you know it".into();
        }
        let kind = self.mush_patches[pi].kind;
        if let Some(p) = self.mush_patches.get_mut(pi) { p.next_ready = self.tick + 60; }
        let pe = self.seats[seat].player;
        self.shroom_take(pe, pi, kind, self.tick);
        String::new()
    }

    /// Pair `villagers[vi]` - a junior of their trade with no master -
    /// with the first free master of it; the Pair up button's gone
    /// (#1598), agents apprentice through their own ask. Returns the
    /// line the press spoke, "No master..." when there's no one to take them
    pub(in crate::session) fn pair_up_at(&mut self, vi: usize) -> String {
        let v = self.villagers[vi];
        match self.pair_master_for(v) {
            Some(m) => {
                self.world.get_component_mut::<Craft>(v).unwrap().master = Some(m);
                relationships::add_relationship(&mut self.world, relationships::Relationship {
                    from: v, other: m, kind: relationships::RelKind::Apprentice, strength: 0.3, interactions: 1, ..Default::default() });
                let mi = self.villagers.iter().position(|x| *x == m).unwrap();
                format!("You pair {} with {} - he'll learn faster now.", self.cast_name(vi), self.cast_name(mi))
            }
            None => "No master of their trade to pair them with.".into(),
        }
    }

    /// The first available master of `v`'s trade - the pairing the old
    /// Pair up button offered, and what the agent's ask still checks
    pub(in crate::session) fn pair_master_for(&self, v: Entity) -> Option<Entity> {
        let voc = self.world.get_component::<Craft>(v).map(|c| c.vocation)?;
        self.villagers.iter().copied().find(|&m| {
            m != v && self.world.get_component::<Craft>(m)
                .map(|mc| mc.vocation == voc && vocation::level_of(mc.skill) == Level::Master).unwrap_or(false)
                && !crate::game::dungeon::unavailable(&self.world, m)
        })
    }

    /// The stray cat's entity when she's afoot, hasn't chosen yet, and the
    /// seat's player is close enough to stroke her. Cooldown isn't checked
    /// here - the button stays and greys with the reason instead
    fn stray_cat_at(&self, seat: usize) -> Option<Entity> {
        let r = self.rare.iter().find(|r| r.id == events::EventId::StrayCat && !r.leaving)?;
        let e = r.e?;
        // she's only a pet once she's made her pitch - the Pet lands on arrival
        self.world.get_component::<Pet>(e)?;
        let cp = self.world.get_component::<Pos>(e)?;
        let (px, py) = self.player_pos_at(seat);
        (((cp.x - px).powi(2) + (cp.y - py).powi(2)).sqrt() <= events::STRAY_CAT_PET_DIST)
            .then_some(e)
    }

    /// The nearest flushed patch within a gatherer's reach of the seat's
    /// player - a pick, player's or agent's, lives and dies by this
    pub(in crate::session) fn shroom_patch_at(&self, seat: usize) -> Option<usize> {
        let (px, py) = self.player_pos_at(seat);
        self.mush_patches.iter().enumerate()
            .filter(|(_, p)| self.tick >= p.next_ready)
            .map(|(i, p)| { let dx = p.tile.0 as f32 - px; let dy = p.tile.1 as f32 - py;
                (i, dx * dx + dy * dy) })
            .filter(|(_, d2)| *d2 <= 36.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// The burning tile nearest the seat's player within throwing
    /// distance, if any - the Throw water button lives and dies by this
    fn fire_douse_spot_at(&self, seat: usize) -> Option<(i32, i32)> {
        let (px, py) = self.player_pos_at(seat);
        self.fire.burning.keys().copied()
            .map(|t| {
                let (dx, dy) = (t.0 as f32 - px, t.1 as f32 - py);
                (dx * dx + dy * dy, t)
            })
            .filter(|&(d2, _)| d2 <= 9.0)
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)))
            .map(|(_, t)| t)
    }

    /// Throw a bucket on the nearest flame in reach. Returns the log
    /// line; "" when there's nothing alight close enough
    fn player_douse_at(&mut self, seat: usize) -> String {
        let Some(t) = self.fire_douse_spot_at(seat) else { return String::new() };
        if self.fire.douse(t.0, t.1, self.tick) {
            let (px, py) = self.player_pos_at(seat);
            self.emit_sfx("splash", px, py, 0.6);
            "You throw a bucket of water on the flames.".into()
        } else {
            String::new()
        }
    }

    /// Stroke the stray cat: a little warmth each time, then she needs a
    /// moment - and a kind hand often keeps her when she chooses. The log
    /// line is the caller's; this returns "" so routine pets stay quiet
    /// and only the milestones speak
    fn pet_stray_cat_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some(e) = self.stray_cat_at(seat) else { return String::new() };
        let on_cooldown = self.rare.iter().find(|r| r.id == events::EventId::StrayCat)
            .map(|r| r.aux > self.frame_n as u64).unwrap_or(false);
        if on_cooldown { return String::new() }
        let (was, now) = self.world.get_component::<Pet>(e)
            .map(|p| (p.bond_strength, (p.bond_strength + events::STRAY_CAT_PET_BOND).min(1.0)))
            .unwrap_or((0.0, 0.0));
        if let Some(p) = self.world.get_component_mut::<Pet>(e) { p.bond_strength = now; }
        if let Some(r) = self.rare.iter_mut().find(|r| r.id == events::EventId::StrayCat) {
            r.aux = self.frame_n as u64 + events::STRAY_CAT_PET_COOLDOWN;
            *r.petters.entry(self.seats[seat].player).or_default() += 1;
        }
        self.say(e, "*purrr*", 2);
        if was <= 0.0 {
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                format!("t{}: the stray cat winds round your ankles.", tick),
            );
        } else if was < 0.5 && now >= 0.5 {
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: she purrs under your hand - she's warming to you.",
                    tick
                ),
            );
        }
        String::new()
    }

    /// The animal the Stroke button reaches for: the nearest tameable
    /// beast within reach - a sheep or goat of the fold, a yard hen, or
    /// a straying pet nobody owns - that's not already yours. The stray
    /// cat keeps her own event's button. None when the seat already
    /// trails three
    fn tame_animal_at(&self, seat: usize) -> Option<(Entity, Species)> {
        let pe = self.seats[seat].player;
        let (px, py) = self.player_pos_at(seat);
        let mine = self.pets.iter().filter(|(e, _, _)| {
            self.world.get_component::<Pet>(*e)
                .map(|p| p.owner == Some(pe)).unwrap_or(false)
        }).count();
        if mine >= 3 { return None; }
        let stray = self.rare.iter().find(|r| r.id == events::EventId::StrayCat)
            .and_then(|r| r.e);
        let near = |e: Entity| {
            if Some(e) == stray { return false; }
            let owned = self.world.get_component::<Pet>(e)
                .map(|p| p.owner.is_some()).unwrap_or(false);
            if owned { return false; }
            self.world.get_component::<Pos>(e)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= 4.0)
                .unwrap_or(false)
        };
        self.flock.iter().copied().filter(|&a| near(a))
            .filter_map(|a| self.world.get_component::<crate::game::livestock::Livestock>(a)
                .map(|l| (a, if l.kind == crate::game::livestock::LivestockKind::Sheep { Species::Sheep } else { Species::Goat })))
            .chain(self.hens.iter().copied().filter(|&a| near(a)).map(|a| (a, Species::Chicken)))
            .chain(self.pets.iter()
                .filter(|&&(e, _, _)| near(e))
                .filter_map(|&(e, _, _)| self.world.get_component::<Pet>(e).map(|p| (e, p.species))))
            .min_by(|a, b| {
                let d = |e: Entity| self.world.get_component::<Pos>(e)
                    .map(|p| (p.x - px).powi(2) + (p.y - py).powi(2)).unwrap_or(f32::MAX);
                d(a.0).partial_cmp(&d(b.0)).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Stroke the animal till it leaves the fold: the stray cat's own
    /// rhythm - a little warmth a stroke, a moment between, and at the
    /// threshold it's yours and follows. The log line is the caller's;
    /// "" stays quiet
    fn tame_stroke_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some((e, species)) = self.tame_animal_at(seat) else { return String::new() };
        let cooling = self.pet_cool.get(&e).copied().unwrap_or(0) > self.frame_n as u64;
        if cooling { return String::new() }
        // a fold beast or a yard hen isn't a Pet yet - the first stroke
        // makes it one
        if self.world.get_component::<Pet>(e).is_none() {
            let name = match species {
                Species::Sheep => "the sheep",
                Species::Goat => "the goat",
                Species::Chicken => "the hen",
                _ => "the stray",
            };
            self.world.add_component(e, Pet {
                name: name.into(), species, owner: None,
                bond_strength: 0.0, mood_effect: 0.3 });
        }
        let (was, now) = self.world.get_component::<Pet>(e)
            .map(|p| (p.bond_strength, (p.bond_strength + events::STRAY_CAT_PET_BOND).min(1.0)))
            .unwrap_or((0.0, 0.0));
        if let Some(p) = self.world.get_component_mut::<Pet>(e) { p.bond_strength = now; }
        self.pet_cool.insert(e, self.frame_n as u64 + events::STRAY_CAT_PET_COOLDOWN);
        self.say(e, "*noses your hand*", 2);
        if was <= 0.0 {
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: {} lets you near.",
                    tick,
                    match species {
                        Species::Sheep => "the sheep",
                        Species::Goat => "the goat",
                        Species::Chicken => "the hen",
                        _ => "the stray",
                    }
                ),
            );
        }
        if now >= events::STRAY_CAT_TAME_BOND {
            let pe = self.seats[seat].player;
            pets::adopt(&mut self.world, e, pe);
            self.flock.retain(|&a| a != e);
            self.hens.retain(|&a| a != e);
            self.flock_wander.remove(&e);
            // she stops being a beast of the fold: the shepherd leaves
            // her be, the wool count forgets her
            self.world.remove_component::<crate::game::livestock::Livestock>(e);
            self.pets.push((e, species, (0.6, 0.4)));
            let name = self
                .world
                .get_component::<Pet>(e)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "it".into());
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
                format!("t{}: {} leaves the fold and follows you.", tick, name),
            );
            self.toasts.push((format!("{} is yours", name), self.tick));
            return format!("{} warms to you - it follows now.", name);
        }
        String::new()
    }

    /// The street: the cause marches. Returns the log line - turnout, the
    /// lord's reaction, and whatever the day brings back to the room
    /// (a crackdown, a taking, the ban proclaimed).
    /// Call the cause out: the column forms and marches - the town's
    /// ways mostly, and some days the road out to where the grievance
    /// lives. What the street makes of it lands when the march comes
    /// home. Returns the log line
    pub(in crate::session) fn march_at(&mut self, seat: usize) -> String {
        if crate::game::campaign::campaign_of(&self.world, self.seats[seat].org).is_none() {
            return "There's no cause to march for.".into() }
        if self.formation.is_some() { return "the column's already out.".into() }
        // the column: the cause's own, file by file
        let members = self.march_members(seat);
        if members.is_empty() { return "there's nobody to march with you.".into() }
        // the route: a day in three the cause takes the road to where
        // the grievance lives - the lord's gate, the sisters, the town
        // - and comes home with that place's answer; the rest of the
        // time it rings the town's paved ways as it always has
        let org = self.seats[seat].org;
        let hp = self.world.get_component::<Pos>(members[0]).copied().unwrap();
        let road = if routine::mix(org, self.tick) % 3 == 0 {
            self.march_destination(org)
                .and_then(|si| self.road_route(si, (hp.x, hp.y)).map(|r| (si, r)))
        } else { None };
        let (dest, route) = match road {
            Some((si, r)) => (Some(si), r),
            None => (None, self.street_route((hp.x, hp.y), 24)),
        };
        if route.is_empty() { return "the streets won't hold a march.".into() }
        let n = members.len();
        self.form_march(seat, members, dest, route);
        match dest {
            Some(si) => format!("the cause marches on {}: {} take the road.",
                self.sites[si].name, n),
            None => format!("the cause marches: {} take to the streets.", n),
        }
    }

    /// Who answers the call: the org's members, living, free and on the
    /// map, file by file
    fn march_members(&mut self, seat: usize) -> Vec<Entity> {
        let mut members: Vec<Entity> = self.world.get_component::<Organization>(self.seats[seat].org)
            .map(|o| o.members.iter().copied().collect()).unwrap_or_default();
        members.retain(|&v| !villager::is_dead(&self.world, v)
            && !crate::game::dungeon::unavailable(&self.world, v)
            && self.world.get_component::<Pos>(v).is_some());
        members.sort();
        members
    }

    /// The column forms on the given route: whoever was walking
    /// somewhere, they march now instead
    fn form_march(&mut self, seat: usize, members: Vec<Entity>, dest: Option<usize>, route: Vec<(f32, f32)>) {
        for &m in &members {
            self.travels.retain(|t| t.who != m);
            self.world.remove_component::<routine::Routine>(m);
            self.world.add_component(m, villager::InFormation);
        }
        self.formation = Some(formation::Formation {
            kind: formation::FormationKind::March, seat, born: self.tick,
            members, route, at: 0, stuck: 0, waited: 0, leg0: -1.0,
            cheered: Default::default(), dest });
    }

    /// The cause takes the road to `si` on demand - for dumps and the
    /// ws test surface, like force_event. False when the column's
    /// already out, there's nobody to march, or no road reaches the site
    pub fn force_road_march(&mut self, seat: usize, si: usize) -> bool {
        if self.formation.is_some() || si >= self.sites.len() { return false }
        let members = self.march_members(seat);
        let Some(hp) = members.first().and_then(|&m|
            self.world.get_component::<Pos>(m).copied()) else { return false };
        let Some(route) = self.road_route(si, (hp.x, hp.y)) else { return false };
        self.form_march(seat, members, Some(si), route);
        true
    }

    /// Where a road march goes: the site the day's issue names - the
    /// lord's gate when the grievance is his (the taxes, his writs, the
    /// death duty), the sisters when the cause is a faith's, the market
    /// town for the rest. None when the country has nowhere fitting -
    /// the cause rings the town instead.
    pub(in crate::session) fn march_destination(&self, org: Entity) -> Option<usize> {
        let c = crate::game::campaign::campaign_of(&self.world, org)?;
        let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .map(|c| c.issue.clone())?;
        // a conversion campaign's issue is the faith's own name, so the
        // faiths name themselves beside the two issue tags
        let is_faith = matches!(issue.as_str(),
            "faith" | "the true faith" | "the parish" | "the old faith");
        let want = if is_faith { country::SiteKind::Monastery }
            else if crate::game::desires::cluster_of(&issue) == Some("lords_hand") { country::SiteKind::Castle }
            else { country::SiteKind::Town };
        let m = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y));
        self.sites.iter().enumerate()
            .filter(|(_, s)| s.kind == want)
            .min_by_key(|(_, s)| m.map(|(x, y)|
                ((s.centre.0 as f32 - x).powi(2) + (s.centre.1 as f32 - y).powi(2)) as i64)
                .unwrap_or(0))
            .map(|(i, _)| i)
    }

    /// The road the column takes to `site`: the beaten way out to its
    /// door, a turn round the ground while the cry's given, then the
    /// way home to the market cross - the same paths the carts and the
    /// stewards walk. None when no path reaches the door.
    pub(in crate::session) fn road_route(&mut self, si: usize, from: (f32, f32)) -> Option<Vec<(f32, f32)>> {
        let (keep, centre) = { let s = &self.sites[si]; (s.homes.first().copied(), s.centre) };
        let door = keep
            .and_then(|k| self.world.get_component::<Pos>(k).copied())
            .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32))
            .unwrap_or(centre);
        let from_t = (from.0.round() as i32, from.1.round() as i32);
        let mut out = self.walk_memo(from_t, door)?;
        out.push(door);
        // a turn round the ground: the ring of clear tiles about the
        // site, so the cry goes up at every face of it
        let ring: Vec<(i32, i32)> = (0..8).map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            (centre.0 + (5.0 * a.cos()).round() as i32, centre.1 + (5.0 * a.sin()).round() as i32)
        }).filter(|&t| self.map.passable(t.0, t.1)).collect();
        let home = self.world.get_component::<Pos>(self.market_food)
            .map(|p| (p.x.round() as i32, p.y.round() as i32))?;
        let mut back = self.walk_memo(door, home)?;
        back.push(home);
        let mid = |t: (i32, i32)| (t.0 as f32 + 0.5, t.1 as f32 + 0.5);
        Some(out.into_iter().chain(ring).chain(back).map(mid).collect())
    }

    /// The march comes home: the street answers it - the turnout, the
    /// knights or the names taken, who watched and woke. A road march
    /// brings the destination's own answer: the sheriff takes the names
    /// at the lord's gate, the town's folk line the road, the sisters
    /// bless the cause. Runs when the column's walked its route out
    pub(crate) fn march_resolved(&mut self, seat: usize, site: Option<usize>) {
        let tick = self.tick;
        // the street answers once a day - the cooldown runs from home
        self.march_day = tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
        let pe = self.seats[seat].player;
        let msg = match crate::game::actions::demonstrate(&mut self.world, c, pe, tick) {
            Some(out) => {
                let n = out.turnout.len();
                // the turnout doesn't just count - they gather in the
                // street with signs for the day of the march
                let (mx, my) = self.player_pos_at(seat);
                let crowd: Vec<Entity> = out.turnout.iter().copied()
                    .filter(|e| self.villagers.contains(e)).collect();
                self.call_protest((mx, my), crowd, (routine::mix(c, 29) % 2) as u8, tick + 1, tick);
                // guards already posted stay posted - a second
                // crackdown doesn't pile more knights into the frame
                let posted_before = self.world
                    .get_component::<crate::game::reactions::GuardsPosted>(self.seats[seat].org).is_some();
                let mut r = crate::game::reactions::react(&mut self.world, c, &out.turnout, tick);
                // a column at the lord's gate is never beneath notice:
                // the sheriff takes the names himself
                let dest = site.map(|si| (self.sites[si].kind, self.sites[si].name));
                if matches!(dest, Some((country::SiteKind::Castle, _)))
                    && r == Some(crate::game::reactions::Reaction::Ignored) {
                    r = Some(crate::game::reactions::Reaction::Watched);
                }
                // everyone who saw it and felt the cause wake in them
                let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.issue.clone()).unwrap_or_default();
                let members: crate::engine::det::HashSet<Entity> =
                    self.world.get_component::<Organization>(self.seats[seat].org)
                        .map(|o| o.members.iter().copied().collect()).unwrap_or_default();
                let mut woke = 0;
                for &v in &self.villagers {
                    // a march is seen, not invited to: shunned or
                    // not, whoever shares the stake or the friend
                    // can feel the cause wake
                    if !members.contains(&v) {
                        woke += crate::game::reactions::witness(&mut self.world, v, &issue, &out.turnout, tick) as usize;
                    }
                }
                // where the cry went up: the street, or the road and
                // the place the road led
                let way = dest.map(|(kind, name)| match kind {
                    country::SiteKind::Castle => format!(" on {}", name),
                    _ => format!(" to {}", name),
                }).unwrap_or_else(|| " in the street".into());
                let mut msg = match (r, dest) {
                    (Some(crate::game::reactions::Reaction::Crackdown), Some((country::SiteKind::Castle, name))) => {
                        self.toasts.push(("knights posted to the streets".into(), tick + 40));
                        format!("{} march on {} - and the knights ride out from the gate.", n, name)
                    }
                    (Some(crate::game::reactions::Reaction::Crackdown), _) => {
                        self.toasts.push(("knights posted to the streets".into(), tick + 40));
                        format!("{} march{} - and the knights come out.", n, way)
                    }
                    (Some(crate::game::reactions::Reaction::Watched), Some((country::SiteKind::Castle, name))) =>
                        format!("{} march on {} - the sheriff takes the names at the gate.", n, name),
                    (Some(crate::game::reactions::Reaction::Watched), _) =>
                        format!("{} march{} - the names get taken.", n, way),
                    (_, Some((country::SiteKind::Town, name))) =>
                        format!("{} march to {} - the folk line the road, and word goes home with the cart.", n, name),
                    (_, Some((country::SiteKind::Monastery, name))) =>
                        format!("{} march to {} - the sisters bless the cause.", n, name),
                    _ => format!("{} march in the street - beneath notice, for now.", n),
                };
                // and the sisters' blessing stays with the marchers
                if matches!(dest, Some((country::SiteKind::Monastery, _))) {
                    for &m in &out.turnout {
                        if let Some(fa) = self.world.get_component_mut::<crate::sim::religion::Faith>(m) {
                            fa.devotion = (fa.devotion + 0.04).min(1.0);
                        }
                    }
                }
                if woke > 0 { msg += &format!(" {} watching feel the cause wake.", woke); }
                // a crackdown takes the loudest - and the cause
                // becomes a prisoner
                if r == Some(crate::game::reactions::Reaction::Crackdown) {
                    self.emit_sfx("alarm", mx, my, 1.0);
                    // the knights stand in the streets where everyone
                    // can see them - once; posted guards stay posted
                    if !posted_before {
                        let (px, py) = self.player_pos_at(seat);
                        let o = self.seats[seat].org;
                        self.posted.push((o, px + 1.5, py));
                        self.posted.push((o, px - 1.5, py));
                    }
                    // a campaign that keeps drawing the knights gets
                    // a rule written against it - the movement
                    // mattered enough to be outlawed
                    if posted_before
                        && self.world.get_component::<crate::game::reactions::AssemblyBan>(self.seats[seat].org).is_none() {
                        self.world.add_component(self.seats[seat].org,
                            crate::game::reactions::AssemblyBan { cap: 2 });
                        msg += " The Reeve proclaims it law: no more than two assembled.";
                        self.toasts.push(("the ban is proclaimed".into(), tick + 40));
                        self.notices.push(("ban".into(), "no more than two assembled - the Reeve's law".into(), tick + 60));
                    }
                    // every crackdown can still take someone
                    if let Some(taken) = crate::game::dungeon::arrest(&mut self.world, c, tick) {
                        // the watch house stands? then the gaol's its
                        // cellar, and the arrested walks there under
                        // escort - the village sees them go
                        if self.village_gaol.is_none() {
                            let watch = self.building_entities.iter().copied().find(|&b|
                                self.world.get_component::<Building>(b)
                                    .map(|bb| bb.kind == BuildingKind::WatchPost).unwrap_or(false));
                            if let Some(wb) = watch {
                                if let Some(wp) = self.world.get_component::<Pos>(wb).copied() {
                                    let (mx, my) = (wp.x.round() as i32, wp.y.round() as i32);
                                    self.village_gaol = (0..4).map(|i| (mx + [1, -1, 0, 0][i], my + [0, 0, 1, -1][i]))
                                        .find(|&(tx, ty)| self.map.passable(tx, ty) && !self.map.is_occupied(tx, ty))
                                        .or(Some((mx, my + 1)));
                                }
                            }
                        }
                        if let Some(g) = self.village_gaol {
                            self.send_far(taken, g, Goal::GoGaol);
                        }
                        // hauled off under escort - the crowd reads who
                        // steps up beside them (#1255)
                        self.reckoning(taken, "the gaol");
                        
                        // hauled off mid-errand: whatever they were doing is dropped
                        self.travels.retain(|t| t.who != taken);
                        let tn = self.villagers.iter().position(|v| *v == taken).map(|i| self.cast_name(i))
                            .unwrap_or("someone");
                        msg += &format!(" {} is taken.", tn);
                        // the release effort is a campaign of its own:
                        // the issue is a person now, not a grievance
                        let rc = crate::game::dungeon::found_release_campaign(&mut self.world,
                            self.seats[seat].org, taken, tick);
                        // those who watched *them* hauled off - a friend
                        // of the taken, not just anyone in the crowd -
                        // sign on for their release, no organizing needed
                        let scene = [taken];
                        let mut signed = 0usize;
                        for &v in &self.villagers.clone() {
                            if crate::game::reactions::witness(&mut self.world, v,
                                    &"the prisoners".to_string(), &scene, tick)
                                && crate::game::dungeon::join(&mut self.world, rc, v) {
                                signed += 1;
                            }
                        }
                        if signed > 0 { msg += &format!(" {} sign on to free them.", signed); }
                        self.toasts.push_at((format!("{} is taken", tn), tick + 40), toast_spot(&self.world, taken));
                    }
                }
                else if r == Some(crate::game::reactions::Reaction::Watched) || woke > 0
                    || matches!(dest, Some((country::SiteKind::Town | country::SiteKind::Monastery, _))) {
                    // a good reaction - the street sends the column home
                    // with a cheer behind it
                    self.emit_sfx("cheer", mx, my, 0.8);
                }
                // the campaign talks over what the street brought
                // (after any arrest, so the taken are counted)
                let ev = crate::game::campaign_arc::ArcEvent::March(
                    r.unwrap_or(crate::game::reactions::Reaction::Ignored));
                let speakers = self.org_speakers(c, tick);
                if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick, &speakers) {
                    let mut with = vec![pe];
                    with.extend(out.turnout.iter().copied());
                    self.record_campaign_talk(with, &d);
                }
                msg
            }
            None => "the march dissolved into the streets.".into(),
        };
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
            format!("t{}: {}", tick, msg),
        );
    }

    /// The schoolhouse ask: raise the room the first time, send for a
    /// teacher after - the org's treasury pays either way
    /// The charter: the chest outfits a crew, and the able and willing
    /// take the bell - the brigade answers fires before the line forms
    fn brigade_at(&mut self, seat: usize) -> String {
        let oe = self.seats[seat].org;
        if self.brigade.is_some() {
            return "The brigade already answers.".into();
        }
        if self.world.get_component::<crate::game::organizing::MeetingHall>(oe).is_none() {
            return "The brigade needs a hall to muster in.".into();
        }
        let Some(o) = self.world.get_component_mut::<Organization>(oe)
            .filter(|o| o.resources >= BRIGADE_COST) else {
            return "The chest can't outfit a crew.".into();
        };
        o.resources -= BRIGADE_COST;
        let leader = o.leader;
        // the bell goes out: the able and willing step forward - the
        // cause's own first, then the neighbours who'll answer
        let mut members = vec![leader];
        for &v in &self.villagers.clone() {
            if members.len() >= 5 { break; }
            if v == leader || villager::is_dead(&self.world, v)
                || self.world.get_component::<Kid>(v).is_some()
                || self.world.get_component::<crate::game::prophetic::Follower>(v).is_some()
                || self.world.get_component::<country::Outlander>(v).is_some() { continue; }
            members.push(v);
        }
        let e = self.world.create_entity();
        self.world.add_component(e, Organization {
            name: "the fire brigade".into(),
            members, leader,
            influence_pool: 0.0, resources: 0.0,
            charter: "answer every fire bell".into(), successors: vec![],
        });
        self.brigade = Some(e);
        self.brigade_gear = 3;
        self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Major), format!("t{}: the fire brigade is chartered - sworn hands, oiled buckets, every bell answered.", self.tick));
        "The brigade's chartered - the chest bought the buckets, and the bell has hands.".into()
    }

    /// What pressing a ctx button does, told ahead - the hover line that
    /// doesn't bar the press (#1071). None when nothing needs explaining.
    pub fn ctx_hint_at(&self, _seat: usize, _act: CtxAct) -> Option<String> {
        None
    }

    pub(in crate::session) fn school_at(&mut self, seat: usize) -> String {
        let oe = self.seats[seat].org;
        if self.school.is_none() {
            let paid = self.world.get_component_mut::<Organization>(oe)
                .map(|o| if o.resources < SCHOOL_FOUND_COST { false } else { o.resources -= SCHOOL_FOUND_COST; true })
                .unwrap_or(false);
            if !paid { return "the treasury can't raise it.".into() }
            self.found_school();
            self.hire_teacher(90);
            return "the treasury raises the schoolhouse - a teacher's sent for.".into();
        }
        if self.teacher.is_none() {
            let paid = self.world.get_component_mut::<Organization>(oe)
                .map(|o| if o.resources < SCHOOL_TEACHER_COST { false } else { o.resources -= SCHOOL_TEACHER_COST; true })
                .unwrap_or(false);
            if !paid { return "the treasury can't send for anyone.".into() }
            self.hire_teacher(90);
            return "the treasury sends for a teacher - a schoolmistress comes for the season.".into();
        }
        "the school stands, and someone's at the desk.".into()
    }

    /// Raise the one-room school: a buildable plot near the market, a
    /// door the kids can find
    fn found_school(&mut self) {
        let mp = self.world.get_component::<Pos>(self.market_food)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(self.village);
        let at = self.map.nearest_buildable(mp.0 + 3, mp.1 + 3).unwrap_or((mp.0 + 3, mp.1 + 3));
        let e = self.world.create_entity();
        self.world.add_component(e, Pos { x: at.0 as f32, y: at.1 as f32 });
        self.world.add_component(e, crate::game::household::Building {
            kind: crate::game::household::BuildingKind::Schoolhouse,
            name: "the schoolhouse".into(), occupants: vec![], worker: None,
            overseer: None, stock: crate::engine::det::HashMap::default(),
                    flyer: None, capacity: 0 });
        self.building_entities.push(e);
        self.school = Some(e);
    }

    /// A teacher for the season: a traveller keeps the desk till the
    /// day the road takes them again
    fn hire_teacher(&mut self, days: u64) {
        let Some(s) = self.school else { return };
        let at = self.world.get_component::<Pos>(s).map(|p| (p.x + 0.5, p.y + 0.5))
            .unwrap_or((0.0, 0.0));
        let e = self.world.create_entity();
        self.world.add_component(e, Pos { x: at.0, y: at.1 });
        self.world.add_component(e, Villager { activity: Activity::Idle, target_food: None });
        self.world.add_component(e, TownRole::Teacher);
        if let Some(traits) = self.deal_cast(e, None, None) {
            self.world.add_component(e, traits);
        }
        self.villagers.push(e);
        // the parallel rolls grow with the roster or the next lookup panics
        self.was_sick.push(None);
        self.died_on.push(None);
        self.teacher = Some((e, self.tick + days));
        self.log.push_ev(
            EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
            format!(
                "t{}: a schoolmistress takes the desk for the season.",
                self.tick
            ),
        );
    }

    /// Who sees a seat's panel pick (#1236): the cause's own members
    /// within earshot of them - a pick nobody's near to see only feeds
    /// the org's record
    pub(in crate::session) fn rhetoric_witnesses(&self, seat: usize) -> Vec<Entity> {
        const REACH: f32 = 10.0;
        let pe = self.seats[seat].player;
        let Some(at) = self.world.get_component::<villager::Pos>(pe).copied() else { return vec![] };
        self.world.get_component::<crate::sim::community::Organization>(self.seats[seat].org)
            .map(|o| o.members.iter().copied().filter(|&m| m != pe && !villager::is_dead(&self.world, m)
                && self.world.get_component::<villager::Pos>(m)
                    .map(|p| (p.x - at.x).hypot(p.y - at.y) <= REACH).unwrap_or(false)).collect())
            .unwrap_or_default()
    }

    /// The word goes out, `days` from now at the plan's venue (#1337):
    /// the same landing the call panel's day pick makes, whether the
    /// panel put it or an invitation's when/where did - the question
    /// the call opens with still waits its answer (#1267)
    pub(in crate::session) fn the_words_out(&mut self, seat: usize, days: u64) {
        let oe = self.seats[seat].org;
        let for_day = self.tick + days;
        let label = self.world.get_component::<crate::game::organizing::MeetingPlan>(oe)
            .and_then(|p| p.venue).map(|v| self.venue_label(v))
            .unwrap_or_else(|| "the square".into());
        if let Some(p) = self.world.get_component_mut::<crate::game::organizing::MeetingPlan>(oe) {
            p.for_day = Some(for_day);
        }
        self.seats[seat].frame_offer = Some(self.tick);
        self.notices.push((
            "meet".into(),
            format!("the word's out - folk gather at {} in {} days", label, days),
            for_day,
        ));
        self.toasts
            .push(("a meeting's called - the word's out".into(), self.tick + 40));
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
            format!(
                "t{}: the word's out - they gather at {} in {} days.",
                self.tick, label, days
            ),
        );
    }

    /// The organizer's answer to the founding call on the conversation
    /// box: take the room's leaning up, framed as picked - or let it rest
    /// (the next house meeting can put it again)
    pub fn decide_at(&mut self, seat: usize, pick: usize) {
        // the answering seat's own cause bears what their pick costs (#962)
        let oe = self.seats[seat].org;
        // the pick belongs to the box that's actually up (#1628): the
        // traveller's ask sits last on the render chain, so when his
        // card IS showing nothing louder may eat the pick first
        if self.seats[seat].survey.is_some()
            && self.convo_panel_at(seat)
                .map(|p| p.title == "THE TRAVELLER ASKS").unwrap_or(false) {
            self.survey_answered(seat, pick);
            return;
        }
        // the Act's officer waits on his answer: the scratch, the
        // fine, or the village's own counter-institution
        if let Some(offer) = self.vax_offer.take() {
            self.resolve_vaccination(offer, pick);
            return;
        }
        // the compliance work's its own answer: spend the cause's own
        // resources to satisfy the lord's postings, else the burden
        // ticks the capacity down
        if self.compliance_burden > 0.0 && pick == 0 && self.dilemma.is_none() {
            let paid = if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                if o.resources > 0.05 { o.resources -= 0.05; true } else { false }
            } else { false };
            if paid {
                self.compliance_burden = (self.compliance_burden - 0.3).max(0.0);
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                    format!(
                        "t{}: the postings are answered - the burden's satisfied for the day.",
                        self.tick
                    ),
                );
            } else {
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the postings go unanswered - the cause hasn't the coin to satisfy them.", self.tick));
            }
            return;
        }
        // the council's own answer: convene and settle it by the
        // influence, or let the stall go - the Constantine's own price
        if self.council.is_some() && pick <= 1 && self.dilemma.is_none() {
            self.council = None;
            if pick == 0 { // convene: the influence settles, the convener pays
                self.separation_pressure = (self.separation_pressure - 0.3).max(0.0);
                if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
                    c.orthodoxy = (c.orthodoxy + 0.1).min(1.0);
                }
                // the Constantine's own price: the convener's own
                // standing pays for the breaking
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                        w.legitimacy = (w.legitimacy - 0.1).max(0.0);
                    }
                }
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the council's convened - the influence settles the stall, and the convener's own standing pays.", self.tick));
            } else { // let it stall: the pressure grows
                self.separation_pressure = (self.separation_pressure + 0.1).min(1.0);
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: the stall goes on - the fold's own silence deepens.",
                        self.tick
                    ),
                );
            }
            return;
        }
        // the mystic's own answer: canonize the spirit's own marking,
        // license it under conditions, or suppress it - and risk the
        // martyrdom's own conversion
        if self.mystic.is_some() && pick <= 2 && self.dilemma.is_none() {
            self.mystic = None;
            match pick {
                0 => { // canonize: the charisma absorbed, a saint's own asset
                    if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
                        c.orthodoxy = (c.orthodoxy + 0.1).min(1.0);
                    }
                    self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the mystic's canonized - the spirit's own marking folded into the fold's own telling.", self.tick));
                }
                1 => { // license: tolerated under conditions - bounded influence
                    self.separation_pressure = (self.separation_pressure - 0.1).max(0.0);
                    self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), format!("t{}: the mystic's licensed - the spirit's own marking tolerated under the fold's own terms.", self.tick));
                }
                _ => { // suppress: the removal - and the martyrdom's own risk
                    let martyr = (self.tick.wrapping_mul(37).wrapping_add(oe as u64)) % 100 < 40;
                    if martyr {
                        self.separation_pressure = (self.separation_pressure + 0.3).min(1.0);
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Major), format!("t{}: the mystic's suppressed - and the dead one's bigger than the living was.", self.tick));
                    } else {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the mystic's suppressed - the spirit's own marking quieted, for now.", self.tick));
                    }
                }
            }
            return;
        }
        // the charter's own answer: the gold for the routinization, or
        // the edge for the refusal
        if self.incorporation_offer.is_some() && pick <= 1 && self.dilemma.is_none() {
            self.incorporation_offer = None;
            if pick == 0 { // accept: the gold, the standing, the drift
                if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                    o.resources += 20.0;
                }
                if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
                    c.routinization = (c.routinization + 0.2).min(1.0);
                }
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                    format!(
                        "t{}: the charter's taken - the fold's own gold and the crown's own terms.",
                        self.tick
                    ),
                );
            } else {
                // refuse: the edge keeps, the assets forgone
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), format!("t{}: the charter's refused - the fold keeps its own edge, and its own poverty.", self.tick));
            }
            return;
        }
        // the exposed one's own release: cut loose WITH the support -
        // the compassion's the counter-intelligence, the informer's
        // leverage fails
        if self.inquisition.is_some() && self.penetration > 0.3 && pick == 0 && self.dilemma.is_none() {
            self.penetration = (self.penetration - 0.2).max(0.0);
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: an exposed one walks free with bread and a door - the leverage the eyes built on them fails.", self.tick));
            return;
        }
        // the parley's own choice: ride to it and risk the ambush, hold
        // the distance, or demand hostages - the guarantee costs the
        // castle what the ambush would have cost the cause
        if self.parley.is_some() && self.dilemma.is_none() {
            match pick {
                0 => { // ride to the parley: a real settlement, or the trap
                    let ambush = (self.tick.wrapping_mul(41).wrapping_add(oe as u64)) % 100 < 40;
                    self.parley = None;
                    if ambush {
                        // the negotiate-and-decapitate play: the leader's
                        // taken, and the flag's own violation costs the
                        // castle's own standing
                        if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                            let leader = o.leader;
                            o.members.retain(|&m| m != leader);
                        }
                        for site in &self.sites {
                            let Some(l) = site.lord else { continue };
                            if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                                w.legitimacy = (w.legitimacy - 0.3).max(0.0);
                            }
                        }
                        self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: the parley was a trap - the leader's taken under a white flag, and the castle's own honor's spent for it.", self.tick));
                    } else {
                        self.log.push_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Notable), format!("t{}: the talk was real - the cause's own grievance's heard, for now.", self.tick));
                    }
                    return;
                }
                1 => { // hold the distance: safe, but the talk never happens
                    self.parley = None;
                    self.log.push_ev(
                        EvTag::new(Thread::Law, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: the cause holds its distance - no talk, no trap.",
                            self.tick
                        ),
                    );
                    return;
                }
                _ => { // demand hostages: the guarantee costs the castle
                    self.parley = None;
                    for site in &self.sites {
                        let Some(l) = site.lord else { continue };
                        if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                            w.legitimacy = (w.legitimacy - 0.1).max(0.0);
                        }
                    }
                    self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: hostages demanded for the talking - the castle's own guarantee costs its own standing.", self.tick));
                    return;
                }
            }
        }
        // the deference's own refusal: a small thing alone - the honor
        // unpaid, the oath unsworn - but a commitment-display for the
        // movement, and a cost on the castle's own standing
        if self.separation_pressure > 0.4 && pick == 1 && self.dilemma.is_none() {
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                    w.legitimacy = (w.legitimacy - 0.05).max(0.0);
                }
            }
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable), format!("t{}: the honor goes unsworn - a small thing alone, but the castle reads what it means.", self.tick));
            return;
        }
        // the cult's own counters: shelter a soul out, break the milieu,
        // or call the lord's hand - the last replicates the cage's own
        let cultish = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| c.cultish).unwrap_or(false);
        if cultish && self.dilemma.is_none() {
            match pick {
                0 => { // shelter: a soul walks free, the cage's cost aside
                    let m = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
                        .and_then(|c| c.members.first().copied());
                    if let Some(m) = m {
                        if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
                            c.members.retain(|&x| x != m);
                        }
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: a soul walks free of the fold - the cage's own cost let them.", self.tick));
                    }
                    return;
                }
                1 => { // the outside word: the milieu breaks, the dials drop
                    if let Some(t) = self.world.get_component_mut::<crate::sim::religion::Totalism>(self.congregation) {
                        for d in t.dials.iter_mut() { *d = d.saturating_sub(2); }
                        t.isolation = (t.isolation - 0.2).max(0.0);
                    }
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                        format!(
                            "t{}: the outside word comes in - the fold's own walls thin.",
                            self.tick
                        ),
                    );
                    return;
                }
                _ => { // the coercion: the lord's own hand, and the cage's own method
                    for site in &self.sites {
                        let Some(l) = site.lord else { continue };
                        if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                            w.paranoia = (w.paranoia + 0.3).min(1.0);
                        }
                    }
                    self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: the lord's hand's called on the fold - the coercion's own method replicates the cage.", self.tick));
                    return;
                }
            }
        }
        // the arc's own early contest: while the watching's young,
        // contest the word - the next rung costs him more
        if self.suppression_arc > 0 && pick == 0 && self.counter_intel >= 2
            && self.counter_played != Some(self.tick) {
            self.counter_played = Some(self.tick);
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                    w.legitimacy = (w.legitimacy - 0.05).max(0.0);
                }
            }
            let rung = match self.suppression_arc {
                1 => "the offer",
                2 => "the planted word",
                _ => "the ban",
            };
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the cause reads {} coming - the word's contested early, the lord's own standing wears.", self.tick, rung));
            return;
        }
        // the org's own counter-intel plays: feed the watcher a false
        // word, or read his papers out to the room - once a day each.
        // the gate mirrors the box's own so a hidden menu can't eat
        // another panel's pick
        if self.counter_intel >= 2 && self.dilemma.is_none()
            && self.proposal.is_none() && self.seat_offer.is_none()
            && self.emergency.is_none()
            && self.counter_played != Some(self.tick) {
            // the last row's the decline - the watching stands (#1627)
            let declined = pick > if self.counter_intel >= 3 { 1 } else { 0 };
            if declined {
                self.counter_played = Some(self.tick);
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: the watching's let be - the eyes read what they read.", self.tick));
                return;
            }
            if pick == 0 {
                self.counter_played = Some(self.tick);
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                        w.paranoia = (w.paranoia - 0.2).max(0.0);
                    }
                }
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                    format!(
                        "t{}: a false word's fed to the lord's eyes - the watching softens.",
                        self.tick
                    ),
                );
                return;
            }
            if pick == 1 && self.counter_intel >= 3 {
                self.counter_played = Some(self.tick);
                let bad = (self.tick.wrapping_mul(41).wrapping_add(oe as u64)) % 100 < 20;
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                        if bad {
                            w.paranoia = (w.paranoia + 0.3).min(1.0);
                        } else {
                            w.legitimacy = (w.legitimacy - 0.3).max(0.0);
                            w.paranoia = 0.0;
                        }
                    }
                }
                self.log.push_ev(
                    if bad {
                        EvTag::new(Thread::Law, Valence::Negative, Sev::Notable)
                    } else {
                        EvTag::new(Thread::Org, Valence::Positive, Sev::Notable)
                    },
                    format!(
                        "t{}: {}.",
                        self.tick,
                        if bad {
                            "the reading's found out - the watching hardens"
                        } else {
                            "the watcher's papers read out - the lord's program's spent"
                        }
                    ),
                );
                return;
            }
        }
        // the emergency's own window answers first: gather the
        // witnesses against the lord's staged tale - or let the ban
        // stand for the day (#1627). on a declined day the box stays
        // hidden, so its picks can't eat a lower panel's
        if self.emergency.is_some() && self.dilemma.is_none()
            && self.emergency_declined != Some(self.tick) && pick == 1 {
            self.emergency_declined = Some(self.tick);
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                format!("t{}: the tale's let stand for now - the square stays banned while it holds.", self.tick));
            return;
        }
        if self.emergency.is_some() && self.dilemma.is_none()
            && self.emergency_declined != Some(self.tick) && pick == 0 {
            self.witnesses += 1;
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                format!(
                    "t{}: a witness steps up - the tale has {} telling what they saw.",
                    self.tick, self.witnesses
                ),
            );
            if self.witnesses >= 4 {
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    if let Some(w) = self.world.get_component_mut::<crate::session::subsystems::society::Watcher>(l) {
                        w.legitimacy = (w.legitimacy - 0.2).max(0.0);
                        w.paranoia = 0.0;
                    }
                }
                self.emergency = None;
                self.witnesses = 0;
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the witnesses stand four deep - the lord's staged brawl's read for what it is, the square's free again.", self.tick));
            }
            return;
        }
        // the plant's violent word answers first: discipline the cause,
        // verify who pushed it, or throw them out on suspicion
        if self.proposal.is_some() && self.plant.is_some() {
            let m = self.plant.unwrap();
            match pick {
                0 => { // marshal: the cause stays honest, the plant's read
                    self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the discipline holds - the cause marches honest, and the torch-word dies.", self.tick));
                    self.plant = None;
                }
                1 => { // verify: the ask reads the plant
                    if self.world.get_component::<crate::game::campaign::Plant>(m).is_some() {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the asking reads it out - the plant's own name surfaces, the lord's hand behind the word.", self.tick));
                        if let Some(or) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                            or.influence_pool = (or.influence_pool + 0.05).min(1.0);
                        }
                        self.plant = None;
                    } else {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: no hand behind the word - the suspicion cost the room its trust.", self.tick));
                    }
                }
                _ => { // eject: suspicion enough - right or wrong, it costs
                    let real = self.world.get_component::<crate::game::campaign::Plant>(m).is_some();
                    if let Some(or) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                        or.members.retain(|&x| x != m);
                    }
                    if real {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the plant's thrown out - the lord's word's gone, and the room knows it.", self.tick));
                    } else {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: a true heart's thrown out on suspicion - the room's trust wears thin.", self.tick));
                        if let Some(or) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                            or.influence_pool *= 0.9;
                        }
                    }
                    self.plant = None;
                }
            }
            self.proposal = None;
            return;
        }
        // the lord's own counter-offer answers first: take the seat,
        // refuse it, or ask what it really carries
        if let Some(o) = self.seat_offer.take() {
            match pick {
                0 => {
                    if o.binding {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the seat's real - a binding word from the board, and the cause gains it.", self.tick));
                        if let Some(or) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                            or.influence_pool = (or.influence_pool + 0.1).min(1.0);
                        }
                    } else {
                        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the seat was air - the cause's own fire's spent at their table, and its name's spent with it.", self.tick));
                        if let Some(or) = self.world.get_component_mut::<crate::sim::community::Organization>(oe) {
                            or.influence_pool *= 0.8;
                            or.resources = (or.resources * 0.9).max(0.0);
                        }
                    }
                }
                1 => {
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                        format!(
                            "t{}: the offer's refused - the lord notes the defiance in his book.",
                            self.tick
                        ),
                    );
                }
                _ => {
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                        format!(
                            "t{}: the seat's read first - it {} the board's real word.",
                            self.tick,
                            if o.binding { "carries" } else { "lacks" }
                        ),
                    );
                    self.seat_offer = Some(o);
                }
            }
            return;
        }
        // the ewe's fork answers its own way (#1254): the name goes
        // public and his standing collapses, or it stays in your pocket
        if self.ewe_theft.as_ref().map(|t| t.fork_open).unwrap_or(false)
            && self.dilemma.is_some() {
            self.dilemma = None;
            self.resolve_ewe_fork(pick);
            return;
        }
        // the reckoning's fork answers its own way too (#1255): stand
        // beside your member where all can see, or keep the distance -
        // but only when the card's still the reckoning's own knot
        let ours = match (self.reckoning, &self.dilemma) {
            (Some(c), Some(k)) => k.branches.len() == 2
                && k.branches.iter().all(|b| b.victim == c),
            _ => false,
        };
        if ours {
            self.dilemma = None;
            self.resolve_reckoning_fork(seat, pick);
            return;
        }
        // a different knot took the card off it - the walk's over
        if self.reckoning.is_some() && self.dilemma.is_none() {
            self.reckoning = None;
        }
        // the plea answers its own way (#1256): speak for mercy or let
        // it stand - only when the card's still the plea's own knot
        let ours = match (self.plea, &self.dilemma) {
            (Some(p), Some(k)) => k.branches.len() == 2
                && k.branches.iter().all(|b| b.victim == p.condemned),
            _ => false,
        };
        if ours {
            self.dilemma = None;
            self.resolve_plea(seat, pick);
            return;
        }
        // the card was lost under the plea: the sentence lands as read
        // - the begging never reached the court's ear. The plea stays
        // set through the landing so it can't re-beg itself (#1256)
        if self.plea.is_some() && self.dilemma.is_none() {
            if let Some(p) = self.plea {
                self.land_sentence(p, self.tick);
            }
            self.plea = None;
        }
        // the false traveller's cards answer their own way (#1258):
        // the plays, then which lie - only when the card's still his
        // own knot (every branch's victim is his figure)
        let ours = match (self.false_traveller, &self.dilemma) {
            (Some(ft), Some(k)) => ft.card != 0
                && k.branches.iter().all(|b| b.victim == ft.e),
            _ => false,
        };
        if ours {
            let stage = self.false_traveller.map(|ft| ft.card).unwrap_or(0);
            self.dilemma = None;
            self.resolve_false_fork(seat, stage, pick);
            return;
        }
        // a different knot took the card off him - the plays are lost,
        // he carries what he carries
        if self.false_traveller.map(|ft| ft.card != 0).unwrap_or(false)
            && self.dilemma.is_none() {
            if let Some(ft) = self.false_traveller {
                self.false_traveller = Some(
                    crate::session::events::FalseTraveller { card: 0, ..ft });
            }
        }
        // the survey's issue card answers its own way (#1270): every
        // branch hangs on the campaign, pick and the asking starts
        let ours = match (&self.survey_pick, &self.dilemma) {
            (Some((c, issues)), Some(k)) => k.branches.len() == issues.len()
                && k.branches.iter().all(|b| b.victim == *c),
            _ => false,
        };
        if ours {
            let (c, issues) = self.survey_pick.take().unwrap();
            self.dilemma = None;
            let issue = issues.get(pick).cloned().unwrap_or_else(|| issues[0].clone());
            self.survey_begin(seat, c, issue);
            return;
        }
        // a different knot took the card off it - the asking waits
        if self.survey_pick.is_some() && self.dilemma.is_none() {
            self.survey_pick = None;
        }
        // the burdened man's knot answers first: a pick writes its debt
        // and the debt comes home when its day does
        if let Some(knot) = self.dilemma.take() {
            if let Some(b) = knot.branches.get(pick) {
                // the members at hand see which way you cut it (#1236): the
                // wounding branch is the hard line, the softest the mercy
                let (lo, hi) = knot.branches.iter().fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x.debt), hi.max(x.debt)));
                let line = if b.debt > lo { Some(crate::game::rhetoric::Line::Exacting) }
                    else if b.debt < hi { Some(crate::game::rhetoric::Line::Merciful) } else { None };
                if let Some(l) = line {
                    let who = self.rhetoric_witnesses(seat);
                    crate::game::rhetoric::witness_all(&mut self.world, &who, oe, l, self.tick);
                }
                self.dilemma_debts
                    .push(crate::session::events::DilemmaDebt {
                        branch: pick,
                        due: self.tick + 3,
                        // the remembering lands on the pair of hands that
                        // made the pick, whoever answered (#962)
                        witness: self.seats[seat].player,
                        victim: b.victim,
                        weight: b.debt,
                    });
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                    format!(
                        "t{}: {} - the choice is carried out, and the remembering begins.",
                        self.tick, b.text
                    ),
                );
            } else {
                // a Decide always echoes something (#1628): a pick past
                // the card's rows still closes it, and says so
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: the knot's let be - the answer never landed.", self.tick),
                );
            }
            return;
        }
        // the call's question: put the matter, or probe what they
        // actually want (#1267)
        // which of their own history the organizer tells (#1248)
        if let Some((_, events)) = self.seats[seat].testimony_offer.take() {
            if let Some(ev) = events.get(pick) {
                if let Some(p) = self.world.get_component_mut::<crate::game::organizing::MeetingPlan>(oe) {
                    p.testimony = Some(ev.clone());
                }
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: you'll open with your own story - why you're here.", self.tick),
                );
            }
            return;
        }
        if self.seats[seat].frame_offer.take().is_some() {
            if pick == 2 && self.testimony_ready(seat) {
                self.seats[seat].testimony_offer = Some((self.tick, self.testimony_events()));
                return;
            }
            let probe = pick == 1;
            // asking before telling is the patient line, and the members
            // at hand hear you choose it (#1236)
            if probe {
                let who = self.rhetoric_witnesses(seat);
                crate::game::rhetoric::witness_all(&mut self.world, &who, oe, crate::game::rhetoric::Line::Patient, self.tick);
            }
            if let Some(p) = self.world.get_component_mut::<crate::game::organizing::MeetingPlan>(oe) {
                p.probe = probe;
            }
            if probe {
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: you'll not put the matter this time - you'll ask them what they actually want.", self.tick),
                );
            }
            return;
        }
        // the call's own day: two days out, four, or a week - the plan
        // keeps taking yeses and roles until the door
        if self.seats[seat].meet_offer.take().is_some() {
            let n = [2u64, 4, 7][pick.min(2)];
            self.the_words_out(seat, n);
            return;
        }
        // the agenda the cadence's own date left open (#1337): the room
        // votes the pick and the meeting runs on it - a founding call
        // sitting on the same box resolves first, matching what the
        // panel showed
        if self.seats[seat].pending_founding.is_none() {
            if let Some((motions, _)) = self.seats[seat].pending_agenda.take() {
                // past the slate: "just meet", then the adjourn - the
                // floor keeps for another night (#1627)
                if pick > motions.len() {
                    self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                        format!("t{}: the floor's adjourned for tonight - the cause meets again soon enough.", self.tick));
                    return;
                }
                let motion = motions.get(pick).copied();
                let msg = self.meet_with_motion(seat, motion);
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                    format!("t{}: {}", self.tick, msg));
                return;
            }
        }
        // the traveller's ask (#1085): it shows past every real
        // decision and past a live talk or meeting, so it answers only
        // when all of those are off the box
        if self.seats[seat].pending_founding.is_none()
            && self.seats[seat].pending_agenda.is_none()
            && self.seats[seat].talk.is_none()
            && self.meeting.as_ref().map_or(true, |m| m.seat != seat)
            && self.seats[seat].survey.is_some() {
            self.survey_answered(seat, pick);
            return;
        }
        let Some(d) = self.seats[seat].pending_founding.take() else { return };
        use crate::game::campaign::{self, DecisionRule, Framing};
        if let Some(mut vote) = d.vote {
            let framing = match pick {
                0 => Framing::SharedGrievance,
                1 => Framing::CommonFoe,
                2 => Framing::MoralDuty,
                _ => {
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: The talk stays talk for now - no cause taken up.",
                            self.tick
                        ),
                    );
                    self.note_talk(
                        d.record,
                        None,
                        "The talk stays talk for now - no cause taken up.",
                    );
                    return;
                }
            };
            // who grumbles is re-read against the framing actually picked
            for b in &mut vote.ballots {
                b.grumbles = b.stake.as_ref().map(|s| !campaign::framing_suits(framing, s)).unwrap_or(false);
            }
            let mut msg = self.found_and_note(seat, &vote, framing, DecisionRule::Majority);
            // the vote that founds enrolls the room: everyone who came
            // joins the cause as a body (#1342)
            let joined = crate::game::campaign::enroll_room(
                &mut self.world, self.seats[seat].org, &d.meet_again);
            if !joined.is_empty() {
                let names: Vec<String> = joined.iter()
                    .filter_map(|e| self.villagers.iter().position(|v| v == e)
                        .map(|i| self.cast_name(i).to_string()))
                    .collect();
                // one name takes the singular (#1626): "Greta joins",
                // not "Greta join"
                let verb = if names.len() == 1 { "joins" } else { "join" };
                msg += &format!(" - {} {} the rolls", names.join(", "), verb);
            }
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                format!("t{}: {}", self.tick, msg),
            );
            self.note_talk(d.record, None, &msg);
            return;
        }
        // no founding on the table: the follow-up steps - meet again with
        // the same folks, take it to the street, or let it rest
        let mut idx = 0;
        if !d.meet_again.is_empty() {
            if pick == idx {
                let oe = self.seats[seat].org;
                let plan = crate::game::organizing::plan_for(&mut self.world, oe);
                for &e in &d.meet_again { if !plan.coming.contains(&e) { plan.coming.push(e); } }
                // the button that asked the day's gone (#1596): the
                // yeses are already in, so the box asks the day itself
                self.seats[seat].meet_offer = Some(self.tick);
                let msg = "They'll gather again - you'll need a place to meet.".to_string();
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}", self.tick, msg),
                );
                self.note_talk(d.record, None, &msg);
                return;
            }
            idx += 1;
        }
        if d.street && pick == idx {
            let msg = self.march_at(seat);
            self.note_talk(d.record, None, &msg);
            return;
        }
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
            format!("t{}: The talk stays talk for now.", self.tick),
        );
        self.note_talk(d.record, None, "The talk stays talk for now.");
    }

    /// What a member's purse will stand for the hat
    const HAT_ASK: f32 = 2.0;
    /// Most flyers one collection posts
    const FLYER_CAP: usize = 8;

    /// Pass the hat and post the cause's flyers: living members chip
    /// in, the treasury buys pamphlets off the print shop, and they go
    /// up around the streets - folk who read them wake to the issue
    pub(in crate::session) fn flyers_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let org = self.seats[seat].org;
        let Some(campaign) = crate::game::campaign::campaign_of(&self.world, org) else { return String::new() };
        let issue = self.world.get_component::<crate::game::campaign::Campaign>(campaign)
            .map(|c| c.issue.clone());
        // who's asked: living grown members not banged up
        let asked: Vec<Entity> = self.world.get_component::<Organization>(org)
            .map(|o| o.members.clone()).unwrap_or_default().into_iter().filter(|&m| {
                !villager::is_dead(&self.world, m)
                    && self.world.get_component::<Kid>(m).is_none()
                    && !crate::game::dungeon::unavailable(&self.world, m)
            }).collect();
        if asked.is_empty() { return "No one to ask.".into() }
        // the print shop's shelf
        let Some(shop) = self.building_entities.iter().copied().find(|&b| {
            self.world.get_component::<Building>(b)
                .map(|bb| matches!(bb.kind, crate::game::household::BuildingKind::Shop(Vocation::Printmaker))
                    && !crate::game::household::derelict(bb))
                .unwrap_or(false)
        }) else { return "No print shop to buy from.".into() };
        // the printer pays himself: the shop's own man, else the first about
        let printer = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Printmaker).unwrap_or(false))
            .max_by_key(|&v| {
                self.world.get_component::<Craft>(v).and_then(|c| c.workplace) == Some(shop)
            });
        // the hat: everyone asked gives what they can spare
        let mut raised = 0.0;
        for &m in &asked {
            let give = self.world.get_component::<Purse>(m).map(|p| p.gold.min(Self::HAT_ASK)).unwrap_or(0.0);
            if give > 0.0 {
                if let Some(p) = self.world.get_component_mut::<Purse>(m) { p.gold -= give; }
                raised += give;
            }
        }
        if let Some(o) = self.world.get_component_mut::<Organization>(org) { o.resources += raised; }
        // where they go up: the market cross first, then the doors of
        // the town's buildings in a fixed scatter, so the same
        // collection always posts the same
        let mut spots: Vec<(i32, i32)> = vec![];
        if let Some(mp) = self.world.get_component::<Pos>(self.market_food) {
            spots.push((mp.x.round() as i32, mp.y.round() as i32));
        }
        let mut doors: Vec<(i32, i32)> = vec![];
        for &b in &self.building_entities {
            if let Some(p) = self.world.get_component::<Pos>(b) {
                let t = (p.x.round() as i32, p.y.round() as i32);
                if !spots.contains(&t) && !doors.contains(&t) { doors.push(t); }
            }
        }
        doors.sort_by(|a, b| noise(a.0, a.1, 0xF1EE9).partial_cmp(&noise(b.0, b.1, 0xF1EE9)).unwrap());
        spots.extend(doors);
        spots.truncate(Self::FLYER_CAP);
        // the treasury buys what it can: the shop's shelf first, then
        // the printer's own satchel
        let price = crate::game::goods::worth_at(&self.world, "pamphlets").max(0.1);
        let want = asked.len().min(Self::FLYER_CAP) as u32;
        let treasury = self.world.get_component::<Organization>(org).map(|o| o.resources).unwrap_or(0.0);
        let affordable = (treasury / price).floor() as u32;
        let mut avail = self.world.get_component::<Building>(shop)
            .map(|b| b.stock.get("pamphlets").copied().unwrap_or(0)).unwrap_or(0);
        if let Some(pr) = printer {
            avail += self.world.get_component::<Satchel>(pr)
                .map(|s| s.goods.get("pamphlets").copied().unwrap_or(0)).unwrap_or(0);
        }
        let qty = want.min(affordable).min(avail).min(spots.len() as u32);
        if qty == 0 {
            return if affordable == 0 {
                format!("The hat raises {:.0} gold - not enough for a flyer.", raised)
            } else { "The print shop's out of pamphlets.".into() };
        }
        let mut left = qty;
        if let Some(b) = self.world.get_component_mut::<Building>(shop) {
            let take = b.stock.get("pamphlets").copied().unwrap_or(0).min(left);
            if take > 0 { *b.stock.get_mut("pamphlets").unwrap() -= take; left -= take; }
        }
        if left > 0 {
            if let Some(pr) = printer {
                if let Some(s) = self.world.get_component_mut::<Satchel>(pr) {
                    let take = s.goods.get("pamphlets").copied().unwrap_or(0).min(left);
                    if take > 0 { *s.goods.get_mut("pamphlets").unwrap() -= take; }
                }
            }
        }
        let cost = qty as f32 * price;
        if let Some(o) = self.world.get_component_mut::<Organization>(org) { o.resources -= cost; }
        if let Some(pr) = printer {
            if let Some(p) = self.world.get_component_mut::<Purse>(pr) { p.gold += cost; }
        }
        for &(sx, sy) in spots.iter().take(qty as usize) {
            self.flyers.push((sx as f32 + 0.5, sy as f32 + 0.5));
        }
        // folk living near a posting read it: the issue gets into their
        // heart whether or not anyone dragged them to a meeting
        let mut woken = 0usize;
        if let Some(issue) = &issue {
            let members: Vec<Entity> = self.world.get_component::<Organization>(org)
                .map(|o| o.members.clone()).unwrap_or_default();
            let mut reached: Vec<Entity> = vec![];
            for &(sx, sy) in spots.iter().take(qty as usize) {
                for &v in &self.villagers {
                    if reached.contains(&v) || members.contains(&v)
                        || villager::is_dead(&self.world, v)
                        || self.world.get_component::<Kid>(v).is_some() { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x - sx as f32 - 0.5).abs() + (p.y - sy as f32 - 0.5).abs() <= 4.0)
                        .unwrap_or(false);
                    if near { reached.push(v); }
                }
            }
            for v in reached {
                if self.world.get_component::<crate::game::reactions::Sympathized>(v).is_none() {
                    self.world.add_component(v, crate::game::reactions::Sympathized::default());
                }
                if let Some(s) = self.world.get_component_mut::<crate::game::reactions::Sympathized>(v) {
                    s.issues.retain(|(i, _)| i != issue);
                    s.issues.push((issue.clone(), tick));
                }
                woken += 1;
            }
        }
        let mut msg = format!("The hat raises {:.0} gold; {} {} up around the streets.",
            raised, qty, if qty == 1 { "flyer goes" } else { "flyers go" });
        if woken > 0 { msg += &format!(" - {} wake{} to the issue.", woken, if woken == 1 { "s" } else { "" }); }
        msg
    }

    /// The slate Meet opens on the conversation box: what the cause's
    /// size can carry (campaign::motions_open), less the doors today
    /// closes - a column already out or home, demands already answered,
    /// no print shop, no trade of theirs to ballot, nobody to spare
    /// for the road or no town left to raise (#1157)
    pub(in crate::session) fn agenda_motions(&self, seat: usize, c: Entity) -> Vec<crate::game::campaign::Motion> {
        use crate::game::campaign::Motion;
        crate::game::campaign::motions_open(&self.world, c).into_iter().filter(|&m| match m {
            Motion::March | Motion::TwoFronts =>
                self.formation.is_none() && self.march_day != self.tick,
            Motion::Strike => self.strike_voc(c).is_some(),
            Motion::Townhall => {
                self.townhall_day != self.tick
                    // a full grant still honoured needs no demands put again
                    && !self.world.get_component::<crate::game::reactions::LordGrant>(self.seats[seat].org)
                        .map(|g| {
                            let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                                .map(|cc| cc.issue.clone()).unwrap_or_default();
                            g.issue == issue && g.active(self.tick) && !g.partial && !g.hollow
                        }).unwrap_or(false)
            }
            Motion::Flyers => self.building_entities.iter().any(|&b| {
                self.world.get_component::<Building>(b)
                    .map(|bb| matches!(bb.kind, crate::game::household::BuildingKind::Shop(Vocation::Printmaker))
                        && !crate::game::household::derelict(bb)).unwrap_or(false)
            }),
            Motion::SendOrganizer =>
                self.spare_organizer(seat, c).is_some() && self.open_post(c).is_some(),
            // one round of asking at a time - and the floor's question
            // card can't ride while another knot's out (#1270)
            Motion::Survey => self.org_survey.is_none() && self.survey_pick.is_none()
                && self.dilemma.is_none(),
            // a warm post is always worth staging - the vote's the point
            Motion::ParallelEvent(_) => true,
            // the pledges that put it on the slate carry it through
            Motion::GroupBuy(..) => true,
            // the charter wants its hall and chest, and there's no
            // second brigade (#1346)
            Motion::Brigade => self.brigade.is_none()
                && self.world.get_component::<crate::game::organizing::MeetingHall>(self.seats[seat].org).is_some()
                && self.world.get_component::<Organization>(self.seats[seat].org)
                    .map(|o| o.resources >= BRIGADE_COST).unwrap_or(false),
            // the party needs its ground: a flushed patch to pick, a
            // free plot to raise on (#1344)
            Motion::WorkParty(crate::game::campaign::PartyKind::Forage) =>
                self.party_patch().is_some(),
            Motion::WorkParty(crate::game::campaign::PartyKind::Raise) =>
                self.build_site_at(seat).is_some(),
            // the rhythm sits whenever the world gate put it there -
            // a short purse's no slate-gate, the carried spend reports
            // the bill instead (#1337)
            Motion::MendHall | Motion::Cadence | Motion::Debrief | Motion::Assembly => true,
            // the chain below owns School's seat on the slate - if the
            // world gate ever learned to push it, it'd sit twice
            Motion::School => false,
        }).collect::<Vec<_>>().into_iter().chain(
            // the schoolhouse is Game state, not world: the slate adds
            // the ask while the desk's unraised or the chair's empty
            if self.school.is_none() || self.teacher.is_none() {
                vec![Motion::School]
            } else { vec![] })
        .collect()
    }

    /// The trade the room can ballot: the craft most of the cause's
    /// living grown members work - ties go to whoever's on the roll
    /// first, so the pick's the same every time it's asked
    fn strike_voc(&self, c: Entity) -> Option<Vocation> {
        let org = self.world.get_component::<crate::game::campaign::Campaign>(c).map(|cc| cc.org)?;
        let members: Vec<Entity> = self.world.get_component::<Organization>(org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let mut counts: Vec<(Vocation, usize)> = vec![];
        for m in members {
            if villager::is_dead(&self.world, m)
                || self.world.get_component::<Kid>(m).is_some()
                || crate::game::dungeon::unavailable(&self.world, m) { continue; }
            let Some(cr) = self.world.get_component::<Craft>(m) else { continue };
            match counts.iter_mut().find(|(v, _)| *v == cr.vocation) {
                Some(e) => e.1 += 1,
                None => counts.push((cr.vocation, 1)),
            }
        }
        counts.into_iter().fold(None, |best: Option<(Vocation, usize)>, (v, n)| {
            if best.map(|(_, b)| n > b).unwrap_or(true) { Some((v, n)) } else { best }
        }).map(|(v, _)| v)
    }

    /// Who the cause can spare for the road: a campaign seat-holder
    /// first (turnout, notes, chair), else a grown free member who
    /// isn't a seat's own player and isn't already posted (#1157)
    fn spare_organizer(&self, seat: usize, c: Entity) -> Option<Entity> {
        use crate::game::campaign::Role::*;
        let free = |w: &World, m: Entity| !villager::is_dead(w, m)
            && w.get_component::<Kid>(m).is_none()
            && !crate::game::dungeon::unavailable(w, m)
            && w.get_component::<crate::game::campaign::FieldPost>(m).is_none()
            // not mid-line with a player, not marching with the column -
            // the road takes someone who can go
            && w.get_component::<villager::InPlayerTalk>(m).is_none()
            && w.get_component::<villager::InFormation>(m).is_none()
            && !self.seats.iter().any(|s| s.player == m);
        [TurnoutCoordinator, NoteTaker, Chair].iter()
            .filter_map(|&r| crate::game::campaign::seat_holder(&self.world, c, r))
            .find(|&h| free(&self.world, h))
            .or_else(|| self.world.get_component::<Organization>(self.seats[seat].org)
                .and_then(|o| o.members.iter().copied().find(|&m| free(&self.world, m))))
    }

    /// The first market town no post is warming yet
    fn open_post(&self, c: Entity) -> Option<usize> {
        let warming: Vec<usize> = self.world.get_component::<crate::game::campaign::Outreach>(c)
            .map(|o| o.posts.iter().map(|(s, _, _)| *s).collect()).unwrap_or_default();
        self.sites.iter().enumerate()
            .find(|(i, s)| s.kind == crate::session::country::SiteKind::Town && !warming.contains(i))
            .map(|(i, _)| i)
    }

    /// A campaign meeting on the floor: the room talks it through, and
    /// what it settles is what the picked motion does - none and it
    /// meets for the resolves alone (#1157)
    fn meet_with_motion(&mut self, seat: usize, motion: Option<crate::game::campaign::Motion>) -> String {
        let tick = self.tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
            return "There's no cause to meet on.".into();
        };
        let pe = self.seats[seat].player;
        match self.hold_and_note_campaign_meeting(Some(pe), c, tick) {
            Some((mut msg, attendees)) => {
                // you stood at the meeting with them (E6-2): the room
                // keeps the episode against this seat's pair memory
                for a in &attendees {
                    crate::game::episodes::file(&mut self.world, *a, pe,
                        crate::game::episodes::MemKind::StoodWithUs, tick, &["meeting", "cause"]);
                }
                // the pick IS the vote: the agenda the floor was handed
                // carries - unless carrying it costs the room itself
                // (#1337): a floor that empties on the line leaves the
                // motion lying there, unseconded
                if let Some(m) = motion {
                    // the line the floor was handed is witnessed by the
                    // room (#1310) - and a member who walked in set
                    // against it, or sets against it as it lands, walks
                    let mut emptied = false;
                    if let Some(line) = crate::game::campaign::motion_line(m) {
                        let oe = self.seats[seat].org;
                        let walked_in_against: Vec<Entity> = attendees.iter().copied()
                            .filter(|&a| crate::game::rhetoric::set_against(&self.world, a, oe, line))
                            .collect();
                        crate::game::rhetoric::witness_all(&mut self.world, &attendees, oe, line, tick);
                        let out = self.line_defections(seat, oe, line, &attendees, &walked_in_against);
                        if !out.is_empty() { msg += &format!(" - {} storms out", out.join(", ")); }
                        emptied = out.len() * 2 >= attendees.len();
                    }
                    if emptied {
                        msg += " - the floor empties on it, the motion falls";
                    } else {
                        msg += " - "; msg += &self.run_motion(seat, c, m, &attendees);
                    }
                }
                msg
            }
            None => "The campaign isn't due to meet yet.".into(),
        }
    }

    /// Who `line` cost the cause: the attendees sworn in and set against
    /// it walk when it gets taken again under their nose (#1310) - those
    /// already hardened against it going in (`walked_in`, since the
    /// witness itself would soften the mark away), and those whose drift
    /// crystallizes as the pick lands. Their seats pass with them; the
    /// cause keeps the line and loses the voice
    fn line_defections(&mut self, seat: usize, oe: Entity,
                       line: crate::game::rhetoric::Line, attendees: &[Entity],
                       walked_in: &[Entity]) -> Vec<String> {
        use crate::game::rhetoric;
        let pe = self.seats[seat].player;
        let mut names = vec![];
        for &m in attendees {
            if m == pe || villager::is_dead(&self.world, m) { continue; }
            let Some(org) = self.world.get_component::<Organization>(oe) else { break };
            if m == org.leader || !org.members.contains(&m) { continue; }
            if !walked_in.contains(&m) && !rhetoric::set_against(&self.world, m, oe, line) { continue; }
            if let Some(o) = self.world.get_component_mut::<Organization>(oe) {
                o.members.retain(|&x| x != m);
            }
            if let Some(c) = crate::game::campaign::campaign_of(&self.world, oe) {
                if let Some(cc) = self.world.get_component_mut::<crate::game::campaign::Campaign>(c) {
                    cc.seats.retain(|s| s.holder != m);
                }
            }
            names.push(self.name_of(m));
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                format!("t{}: {} storms out - the {} line once too often.",
                    self.tick, self.name_of(m), line.status()),
            );
        }
        names
    }

    /// What a carried motion does: the street verbs and the rhythm
    /// verbs, voted first (#1157, #1337). `attendees` is the room that
    /// carried it - the debrief tallies their next step
    fn run_motion(&mut self, seat: usize, c: Entity, motion: crate::game::campaign::Motion,
                  attendees: &[Entity]) -> String {
        use crate::game::campaign::Motion;
        match motion {
            Motion::March => self.march_at(seat),
            Motion::Strike => self.strike_at(seat),
            Motion::Townhall => self.townhall_at(seat),
            Motion::Flyers => self.flyers_at(seat),
            // hands enough for two fronts at once: the column and the
            // ballot on the same day
            Motion::TwoFronts => format!("{} {}", self.march_at(seat), self.strike_at(seat)),
            Motion::SendOrganizer => self.dispatch_organizer(seat, c),
            Motion::ParallelEvent(site) => self.parallel_event_at(seat, site),
            Motion::Survey => self.survey_motion(seat, c),
            Motion::GroupBuy(gi, _) => self.group_buy_at(seat, c, gi),
            Motion::WorkParty(kind) => self.work_party_at(seat, c, kind),
            Motion::Brigade => self.brigade_at(seat),
            Motion::MendHall => self.mend_hall_at(seat),
            Motion::Cadence => self.cadence_at(c),
            Motion::Debrief => self.debrief_at(seat, attendees),
            Motion::Assembly => self.assembly_at(seat, c),
            Motion::School => self.school_at(seat),
        }
    }

    /// The carried roof-mending, on the button or the floor (#1337):
    /// the purse meets the carpenter's bill or the motion fails on it.
    /// A hall already mended by the other path doesn't bill twice -
    /// the slate can still offer it stale, picked after the button beat
    /// it to the repair
    pub(in crate::session) fn mend_hall_at(&mut self, seat: usize) -> String {
        let oe = self.seats[seat].org;
        if self.world.get_component::<crate::game::organizing::HallDamaged>(oe).is_none() {
            return "The hall doesn't need mending.".into();
        }
        let purse = self.world.get_component::<Organization>(oe)
            .map(|o| o.resources).unwrap_or(0.0);
        if purse < crate::game::organizing::HALL_REPAIR_COST {
            let oname = self.world.get_component::<Organization>(oe)
                .map(|o| o.name.clone()).unwrap_or_default();
            format!("{} can't make the carpenter's bill - {} gold in the chest.", oname, purse as i32)
        } else {
            self.world.get_component_mut::<Organization>(oe).unwrap().resources
                -= crate::game::organizing::HALL_REPAIR_COST;
            self.world.remove_component::<crate::game::organizing::HallDamaged>(oe);
            "The carpenters patch the roof - the hall's fit to meet in again.".into()
        }
    }

    /// The carried retiming, on the button or the floor (#1337): the
    /// meeting steps up or down a notch - weekly for the long haul,
    /// daily for the fight
    pub(in crate::session) fn cadence_at(&mut self, c: Entity) -> String {
        let tick = self.tick;
        self.cadence_day = tick;
        let now = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
            .map(|m| m.cadence_days).unwrap_or(3);
        let next = match now { 0..=3 => 7, 4..=7 => 14, 8..=14 => 30, _ => 3 };
        crate::game::campaign::set_cadence(&mut self.world, c, next, tick);
        format!("The cause will meet every {} days now - the organizer's word.", next)
    }

    /// The carried early assembly, on the button or the floor (#1337)
    pub(in crate::session) fn assembly_at(&mut self, seat: usize, c: Entity) -> String {
        let tick = self.tick;
        self.assembly_day = tick;
        match crate::game::campaign::announce_assembly(&mut self.world, c, tick) {
            Some(questions) => {
                let oname = self.world.get_component::<Organization>(self.seats[seat].org)
                    .map(|o| o.name.clone()).unwrap_or_default();
                format!("{} call the accountability assembly - the agenda's posted: {}.",
                    oname, questions.join("; "))
            }
            None => "There's no assembly to call - the campaign's still finding its feet.".into(),
        }
    }

    /// The carried tally (#1337): `present` is the room answering - the
    /// members standing at the organizer on the button's path, the
    /// meeting's attendees on the floor's
    pub(in crate::session) fn debrief_at(&mut self, seat: usize, present: &[Entity]) -> String {
        let tick = self.tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
            return String::new();
        };
        if present.is_empty() { return "Nobody stood to tally.".into(); }
        let options = ["the petition", "the march", "the strike", "a town hall"];
        // each present member's choice is deterministic: their own
        // grievance's tag picks the step that answers it
        let mut tally: crate::engine::det::HashMap<&str, usize> = crate::engine::det::HashMap::default();
        for m in present {
            let pick = options[*m as usize % options.len()];
            *tally.entry(pick).or_insert(0) += 1;
        }
        let winner = options.iter()
            .max_by_key(|o| tally.get(*o).copied().unwrap_or(0))
            .copied().unwrap_or("the petition");
        let mandate = if present.len() < 3 { " - a thin room's weak word" } else { "" };
        if let Some(cc) = self.world.get_component_mut::<crate::game::campaign::Campaign>(c) {
            cc.directive = Some(winner.to_string());
        }
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
            format!(
                "t{}: the debrief settles it - the room's for {}{}.",
                tick, winner, mandate
            ),
        );
        // the settled word is the cause's line, heard by every member
        // standing there (#1310) - the paper asks are the patient work,
        // the streets the firebrand's
        let line = match winner {
            "the petition" => Some(crate::game::rhetoric::Line::Patient),
            "the march" | "the strike" | "a town hall" => Some(crate::game::rhetoric::Line::Firebrand),
            _ => None,
        };
        let mut walked = String::new();
        if let Some(l) = line {
            let oe = self.seats[seat].org;
            let walked_in_against: Vec<Entity> = present.iter().copied()
                .filter(|&a| crate::game::rhetoric::set_against(&self.world, a, oe, l))
                .collect();
            crate::game::rhetoric::witness_all(&mut self.world, present, oe, l, tick);
            let out = self.line_defections(seat, oe, l, present, &walked_in_against);
            if !out.is_empty() { walked = format!(" {} storms out over it.", out.join(", ")); }
            // the room's reaction is heard (#1619): a murmur when the
            // word settles, louder when folk storm over it
            let (px, py) = self.player_pos_at(seat);
            self.emit_sfx("murmur", px, py, if walked.is_empty() { 0.6 } else { 0.85 });
        }
        format!("The room settles on {}{}.{}", winner, mandate, walked)
    }

    /// A work party carried (#1344): the organizer picks the ground and
    /// sends the hands - members with a guest each for the picking, the
    /// village called out for a raising. The work itself resolves when
    /// the hands arrive (party_sweep); the organizer's hands stay clean
    fn work_party_at(&mut self, seat: usize, c: Entity,
            kind: crate::game::campaign::PartyKind) -> String {
        use crate::game::campaign::PartyKind;
        let org = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .map(|cc| cc.org).unwrap_or(0);
        let (tile, patch) = match kind {
            PartyKind::Forage => match self.party_patch() {
                Some(pi) => (self.mush_patches[pi].tile, Some(pi)),
                None => return "No patch is flushed - the wood's been picked.".into(),
            },
            PartyKind::Raise => match self.build_site_at(seat) {
                Some((x, y)) => ((x as i32, y as i32), None),
                None => return "There's no plot free to raise on.".into(),
            },
        };
        // the hands: the living members (the leader organizes, doesn't
        // work), each bringing their warmest non-member friend along -
        // the party doubles as the soft ask
        let members = crate::game::campaign::living_member_list(&self.world, c);
        let leader = crate::game::organizing::living_leader(&mut self.world, org);
        let members: Vec<Entity> = members.into_iter()
            .filter(|&m| Some(m) != leader).collect();
        let member_set: crate::engine::det::HashSet<Entity> =
            members.iter().copied().chain(leader).collect();
        let mut hands: Vec<Entity> = vec![];
        let mut hosts: Vec<(Entity, Entity)> = vec![];
        for &m in &members {
            if let Some(g) = self.party_guest(m, &member_set, &hands) {
                hosts.push((g, m));
                hands.push(g);
            }
        }
        // a raising is the public do: the folk nearest the plot turn
        // out whether or not a member asked them
        if matches!(kind, PartyKind::Raise) {
            for &v in self.villagers.clone().iter() {
                if hands.iter().filter(|h| !member_set.contains(h)).count() >= 6 { break; }
                if member_set.contains(&v) || hands.contains(&v) { continue; }
                if !self.party_guest_able(v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - tile.0 as f32).abs() + (p.y - tile.1 as f32).abs() <= 14.0)
                    .unwrap_or(false);
                if !near { continue; }
                // their host is the member they're warmest with - or
                // the organizer whose call they answered
                let host = self.party_host(v, &members)
                    .or(leader).or_else(|| members.first().copied());
                let Some(host) = host else { break };
                hosts.push((v, host));
                hands.push(v);
            }
        }
        hands.extend(members.iter().copied());
        let mut went = 0usize;
        for &h in &hands.clone() {
            if self.send_far(h, tile, Goal::WorkParty(self.work_parties.len())) { went += 1; }
        }
        if went == 0 {
            return "Nobody could get away for it - the party stands down.".into();
        }
        let idx = self.work_parties.len();
        self.work_parties.push(WorkParty {
            org, kind, tile, patch,
            hands: hands.into_iter()
                .filter(|h| self.travels.iter().any(|t| t.who == *h
                    && matches!(t.goal, Goal::WorkParty(i) if i == idx)))
                .collect(),
            hosts, arrived: vec![], due: self.tick + 3,
        });
        match kind {
            PartyKind::Forage =>
                format!("{} set out under the trees for the caps - the take comes back to the chest.", went),
            PartyKind::Raise =>
                format!("{} turn out to raise the roof - the work will be seen.", went),
        }
    }

    /// The flushed patch nearest the market a party could walk to
    fn party_patch(&self) -> Option<usize> {
        let mp = self.world.get_component::<Pos>(self.market_food).copied()?;
        self.mush_patches.iter().enumerate()
            .filter(|(_, p)| self.tick >= p.next_ready && !p.deadly())
            .map(|(i, p)| { let dx = p.tile.0 as f32 - mp.x; let dy = p.tile.1 as f32 - mp.y;
                (i, dx * dx + dy * dy) })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// Can `v` be called out for the day: a living adult villager not
    /// otherwise held
    fn party_guest_able(&self, v: Entity) -> bool {
        !villager::is_dead(&self.world, v)
            && self.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !crate::game::dungeon::unavailable(&self.world, v)
            && !self.in_player_talk(v)
    }

    /// The member `v` is warmest with (the host an open-call guest
    /// answers to), strongest tie first
    fn party_host(&self, v: Entity, members: &[Entity]) -> Option<Entity> {
        self.world.query::<crate::game::relationships::Relationship>().iter()
            .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
            .filter_map(|r| {
                let other = if r.from == v { Some(r.other) }
                    else if r.other == v { Some(r.from) } else { None }?;
                members.contains(&other).then_some((other, r.strength))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(m, _)| m)
    }

    /// The guest a member brings: their warmest tie who isn't already
    /// a member, isn't coming on another invite, and can get away
    fn party_guest(&self, m: Entity, member_set: &crate::engine::det::HashSet<Entity>,
            hands: &[Entity]) -> Option<Entity> {
        self.world.query::<crate::game::relationships::Relationship>().iter()
            .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
            .filter_map(|r| {
                let other = if r.from == m { Some(r.other) }
                    else if r.other == m { Some(r.from) } else { None }?;
                Some((other, r.strength))
            })
            .filter(|&(o, _)| !member_set.contains(&o) && !hands.contains(&o)
                && self.villagers.contains(&o) && self.party_guest_able(o))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(o, _)| o)
    }

    /// A party's hands have come in - or the day it gave them is out:
    /// the take pools to the chest, the shared day warms the hands, and
    /// the guests carry the warmth back to whoever brought them
    pub(in crate::session) fn party_sweep(&mut self) {
        let mut i = 0;
        while i < self.work_parties.len() {
            let p = &self.work_parties[i];
            let ready = p.arrived.len() >= p.hands.len() || self.tick >= p.due;
            if ready {
                let p = self.work_parties.remove(i);
                self.resolve_party(p);
            } else {
                i += 1;
            }
        }
    }

    /// The work's done: take to the chest, warmth between the hands
    fn resolve_party(&mut self, party: WorkParty) {
        use crate::game::campaign::PartyKind;
        let tick = self.tick;
        let WorkParty { org, kind, tile, patch, hosts, arrived, .. } = party;
        // the hands who made the ground and still stand - the take and
        // the warmth count them
        let present: Vec<Entity> = arrived.into_iter()
            .filter(|&h| !villager::is_dead(&self.world, h)).collect();
        match kind {
            PartyKind::Forage => {
                if let Some(pi) = patch {
                    if let Some(p) = self.mush_patches.get_mut(pi) { p.next_ready = tick + 60; }
                    let worth = crate::game::goods::worth_at(&self.world, "mushrooms");
                    let take = worth * present.len() as f32;
                    if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(org) {
                        o.resources += take;
                    }
                    if let Some(&h) = present.first() {
                        self.say(h, "Good caps, these - the chest eats well tonight.", TALK_BUBBLE_SECS);
                    }
                    self.log.push_ev(
                        EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
                        format!("t{}: the picking party comes back - {:.0} gold of caps for the chest.",
                            tick, take),
                    );
                }
            }
            PartyKind::Raise => {
                // the road reaches out, then the roof rises - the same
                // scene a player's own build gets
                if self.map.buildable(tile.0, tile.1) {
                    let (bx, by) = (tile.0 as f32, tile.1 as f32);
                    self.buildings.push((bx, by, BSprite::House(self.rng.below(97) as usize)));
                    self.map.build_at(tile.0, tile.1);
                    let road = self.map.plan_road(tile);
                    self.works.push(Works { building: self.buildings.len() - 1, road, frames: 0 });
                    self.built_count += 1;
                }
                // the hat goes round the watching street: the cause's
                // chest takes the day's collection
                let hat = present.len() as f32 * 0.5;
                if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(org) {
                    o.resources += hat;
                }
                if let Some(&h) = present.first() {
                    self.say(h, "She'll stand a hundred years.", TALK_BUBBLE_SECS);
                }
                self.log.push_ev(
                    EvTag::new(Thread::Player, Valence::Positive, Sev::Major),
                    format!("t{}: the roof went up - the whole village saw the cause at work.", tick),
                );
            }
        }
        // the shared day warms the hands to each other - and a guest to
        // the member who brought them, a little closer
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        for (a, &m1) in present.iter().enumerate() {
            for &m2 in &present[a + 1..] {
                record_interaction(&mut self.world, m1,
                    Interaction { other: m2, kind: InteractionKind::SharedWork, intensity: 0.4 });
                record_interaction(&mut self.world, m2,
                    Interaction { other: m1, kind: InteractionKind::SharedWork, intensity: 0.4 });
            }
        }
        for &(g, host) in &hosts {
            if !present.contains(&g) || villager::is_dead(&self.world, host) { continue; }
            record_interaction(&mut self.world, g,
                Interaction { other: host, kind: InteractionKind::SharedWork, intensity: 0.6 });
        }
        crate::sim::organized_power::record_action(&mut self.world, org,
            crate::sim::organized_power::ActionQuality::Transformational);
    }

    /// The pooled buy carried (#1343): the pledges collect, the shop
    /// bends its price on the order, shares go home member-first, and
    /// the surplus walks out the door below street for the cause
    fn group_buy_at(&mut self, _seat: usize, c: Entity, gi: usize) -> String {
        let good = crate::game::campaign::BUY_GOODS[gi];
        match crate::game::campaign::group_buy(&mut self.world, c, gi) {
            Some(out) => {
                self.penetration = (self.penetration + 0.03).min(1.0);
                format!("The room goes in on the {} - {} shares home, {} more out the door below street for the cause.",
                    good, out.shares.len(), out.surplus)
            }
            None => format!("The pledges wouldn't reach - the {} stays on the shelf.", good),
        }
    }

    /// The strike motion carried: the trade the room works balloted -
    /// nine of ten or a public failure
    fn strike_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
            return "There's no cause to ballot for.".into();
        };
        let Some(voc) = self.strike_voc(c) else { return "No trade of theirs to ballot.".into() };
        match crate::game::actions::strike_vote(&mut self.world, c, voc, tick) {
            Some(out) => {
                crate::game::org_skills::practice(&mut self.world, self.seats[seat].player,
                    crate::game::org_skills::OrgSkill::Turnout,
                    crate::game::org_skills::SEAT_PRACTICE);
                let ev = crate::game::campaign_arc::ArcEvent::Strike { called: out.called };
                let speakers = self.org_speakers(c, tick);
                if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick, &speakers) {
                    self.record_campaign_talk(vec![self.seats[seat].player], &d);
                }
                // the ballot's read aloud: the floor's murmur answers it
                let (px, py) = self.player_pos_at(seat);
                self.emit_sfx("murmur", px, py, if out.called { 0.85 } else { 0.5 });
                format!("{} of {} say yes{}. {}", out.yes, out.yes + out.no,
                    if out.called { " - the strike is on" } else { "" },
                    if out.called { "The trade stands together." } else { "Less than nine of ten: it fails in public." })
            }
            None => "Nobody of the trade to ballot.".into(),
        }
    }

    /// The demands put to the target in public: the townhall motion
    /// carried
    fn townhall_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
            return "There's no cause to put demands for.".into();
        };
        self.townhall_day = self.tick;
        let Some(target) = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .map(|cc| cc.target).unwrap_or(None).or(self.collector) else { return String::new() };
        use crate::game::actions::{townhall, TargetResponse};
        match townhall(&mut self.world, c, target, tick) {
            Some(out) => { crate::game::org_skills::practice(&mut self.world, self.seats[seat].player,
                    crate::game::org_skills::OrgSkill::Agitation,
                    crate::game::org_skills::SEAT_PRACTICE);
                let msg = match out.response {
                TargetResponse::Carries => {
                    let (cx, cy) = self.player_pos_at(seat);
                    self.emit_sfx("cheer", cx, cy, 1.0);
                    self.toasts.push(("the Reeve takes your demands to the king".into(), tick + 40));
                    // the carried demand lands as a grant: the load
                    // the issue names eases for the cause's people,
                    // and this year's taxing stops counting - but
                    // only the issue fought for is answered
                    let oe = self.seats[seat].org;
                    let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                        .map(|cc| cc.issue.clone()).unwrap_or_default();
                    self.world.add_component(oe, crate::game::reactions::LordGrant::full(
                        &issue, tick, tick + crate::game::desires::YEAR_DAYS));
                    // the tag the tax fight is voted under
                    if issue == "the taxes" {
                        if let Some(o) = self.world.get_component::<Organization>(oe).map(|o| o.members.clone()) {
                            for m in o {
                                crate::game::desires::lift_grievance(&mut self.world, m, crate::game::desires::Problem::Taxed);
                            }
                        }
                    }
                    // a carried demand is the writ beaten in
                    // public: the reach thins for it (#971)
                    self.lord_strength = (self.lord_strength - 0.06).max(0.0);
                    // a carried demand walks the streets: the
                    // village takes the win out in the open
                    self.victory_parade(Some(self.seats[seat].player));
                    format!("{} face the Reeve - and he says the king will hear of this.", out.turnout.len())
                }
                TargetResponse::Concedes => {
                    // it touches him, but the room was thin: part of
                    // it, for half a year - the grievance stands
                    self.toasts.push(("the Reeve grants a part of it".into(), tick + 40));
                    let oe = self.seats[seat].org;
                    let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                        .map(|cc| cc.issue.clone()).unwrap_or_default();
                    // a part already granted and honoured isn't
                    // granted again: pressing a thin room twice
                    // doesn't stretch it
                    if out.nothing_new {
                        format!("{} face the Reeve - he grants no more than he already has.", out.turnout.len())
                    } else {
                        // a concession's a smaller beat, but a
                        // beat all the same (#971)
                        self.lord_strength = (self.lord_strength - 0.03).max(0.0);
                        self.world.add_component(oe, crate::game::reactions::LordGrant::partial(
                            &issue, tick, tick + crate::game::desires::YEAR_DAYS / 2));
                        format!("{} face the Reeve - he grants a part of it, and no more.", out.turnout.len())
                    }
                }
                TargetResponse::Deflects =>
                    format!("The Reeve sends regards and a clerk. {} went home hollow.", out.turnout.len()),
                TargetResponse::Refuses =>
                    format!("The doors stay shut on {} voices. They go home hollow.", out.turnout.len()),
            };
            // the answer at the door is the arc's outcome (read
            // after any grant above has landed)
            let ev = crate::game::campaign_arc::ArcEvent::Townhall(out.response);
            let speakers = self.org_speakers(c, tick);
            if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick, &speakers) {
                let mut with = vec![self.seats[seat].player, target];
                with.extend(out.turnout.iter().copied());
                self.record_campaign_talk(with, &d);
            }
            msg },
            None => "Nobody turns out for the townhall.".into(),
        }
    }

    /// The send-an-organizer motion carried: a spare member rides out
    /// to a market town the reach hasn't touched, and the post warms
    /// the town day by day till it can stage its own event (#1157)
    fn dispatch_organizer(&mut self, seat: usize, c: Entity) -> String {
        let tick = self.tick;
        let Some(who) = self.spare_organizer(seat, c) else {
            return "there's no member to spare for the road.".into();
        };
        let Some(site) = self.open_post(c) else {
            return "every town's already got the word.".into();
        };
        let Some(to) = self.world.get_component::<Pos>(self.sites[site].store)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)) else {
            return "their ground isn't found.".into();
        };
        self.world.add_component(who, crate::game::campaign::FieldPost { site, since: tick });
        if self.world.get_component::<crate::game::campaign::Outreach>(c).is_none() {
            self.world.add_component(c, crate::game::campaign::Outreach::default());
        }
        if let Some(o) = self.world.get_component_mut::<crate::game::campaign::Outreach>(c) {
            o.posts.push((site, who, 0.0));
        }
        self.send_far(who, to, Goal::FieldPost(site));
        let name = self.villagers.iter().position(|&v| v == who)
            .map(|i| cast_of(&self.names, i)).unwrap_or("one of theirs");
        format!("the vote sends {} to raise {} for the cause.", name, self.sites[site].name)
    }

    /// The survey motion carried (#1270): the floor's asked which
    /// question rides - the campaign's own cause plus what the
    /// organizer's heard ledger knows. One candidate and the asking
    /// starts at once
    fn survey_motion(&mut self, seat: usize, c: Entity) -> String {
        let Some(cc) = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .map(|cc| cc.issue.clone()) else {
            return "There's no cause to ask for.".into();
        };
        let mut issues: Vec<crate::game::desires::IssueTag> = vec![cc];
        let pe = self.seats[seat].player;
        if let Some(o) = self.world.get_component::<crate::game::desires::Organizer>(pe) {
            for i in &o.issues {
                if !issues.contains(i) { issues.push(i.clone()); }
            }
        }
        if issues.len() == 1 || self.dilemma.is_some() {
            // one question, or another knot holding the card - either
            // way the cause's own question rides
            let name = crate::game::desires::issue_name(issues[0].clone());
            self.survey_begin(seat, c, issues[0].clone());
            return format!("the vote sends the doors asking about {name}.");
        }
        // which question rides is the floor's pick on the card
        self.survey_pick = Some((c, issues.clone()));
        self.dilemma = Some(crate::session::events::MoralKnot {
            text: "the survey's carried - which question do the doors get asked?".into(),
            branches: issues.iter().map(|i| crate::session::events::DilemmaBranch {
                text: format!("ask about {}", crate::game::desires::issue_name(i.clone())),
                debt: 0.0, victim: c }).collect(),
        });
        "the vote carries the asking - now the floor picks the question.".into()
    }

    /// The survey's own start (#1270): the free grown members carry
    /// the question door to door - three days' asking, then the count
    pub(in crate::session) fn survey_begin(&mut self, seat: usize, c: Entity,
        issue: crate::game::desires::IssueTag) {
        use crate::game::campaign::{OrgSurvey, SURVEY_MAX_HANDS};
        let org = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .map(|cc| cc.org).unwrap_or(self.seats[seat].org);
        let canvassers: Vec<Entity> = self.world.get_component::<Organization>(org)
            .map(|o| o.members.iter().copied()
                .filter(|&m| !villager::is_dead(&self.world, m)
                    && self.world.get_component::<Kid>(m).is_none()
                    && !crate::game::dungeon::unavailable(&self.world, m)
                    && !self.seats.iter().any(|s| s.player == m))
                .take(SURVEY_MAX_HANDS).collect())
            .unwrap_or_default();
        if canvassers.is_empty() { return; }
        let name = crate::game::desires::issue_name(issue.clone());
        self.org_survey = Some(OrgSurvey {
            campaign: c, issue, canvassers,
            days_left: crate::game::campaign::SURVEY_DAYS,
            knocked: crate::engine::det::HashSet::default(),
        });
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
            format!(
                "t{}: the cause's survey goes out - hands on the doors, three days' asking about {name}.",
                self.tick
            ),
        );
    }

    /// The parallel-event motion carried: the posted organizer stages
    /// the cause's day in their town - the folk the warmth gathered
    /// wake to the issue, and the organizer comes home (#1157)
    fn parallel_event_at(&mut self, seat: usize, site: usize) -> String {
        let tick = self.tick;
        let sname = self.sites[site].name;
        let issue = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)
            .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c)
                .map(|cc| cc.issue.clone()))
            .unwrap_or_default();
        // the town wakes: its own folk mark the issue theirs to watch
        let mut woke = 0usize;
        for &v in self.villagers.clone().iter() {
            if villager::is_dead(&self.world, v) { continue; }
            if self.world.get_component::<crate::session::country::Outlander>(v)
                .map(|o| o.site != site).unwrap_or(true) { continue; }
            if self.world.get_component::<crate::game::reactions::Sympathized>(v).is_none() {
                self.world.add_component(v, crate::game::reactions::Sympathized::default());
            }
            if let Some(s) = self.world.get_component_mut::<crate::game::reactions::Sympathized>(v) {
                s.issues.retain(|(i, _)| i != &issue);
                s.issues.push((issue.clone(), tick));
            }
            woke += 1;
        }
        crate::sim::organized_power::record_action(&mut self.world, self.seats[seat].org,
            crate::sim::organized_power::ActionQuality::Transformational);
        // the post's done its work: the organizer walks home
        let mut home = None;
        if let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) {
            if let Some(o) = self.world.get_component_mut::<crate::game::campaign::Outreach>(c) {
                if let Some(i) = o.posts.iter().position(|(s, _, _)| *s == site) {
                    home = Some(o.posts.remove(i).1);
                }
            }
        }
        if let Some(who) = home {
            self.world.remove_component::<crate::game::campaign::FieldPost>(who);
            if let Some(hp) = crate::game::household::home_of(&self.world, who)
                .and_then(|h| self.world.get_component::<Pos>(h).copied()) {
                self.send_far(who, (hp.x.round() as i32, hp.y.round() as i32), Goal::Wander);
            }
        }
        format!("{} rises for the cause - {} wake to {}; the organizer comes home.", sname, woke, issue)
    }
}
