//! The player's verbs: what each action button and context action does, what blocks it, and walking over to do it.

use crate::session::*;

impl Game {
    /// Why action button `i` can't be used right now, if it can't: Build's
    /// own rules (gold, a free site), otherwise disabled_reason
    pub fn action_blocker_at(&self, seat: usize, i: usize) -> Option<String> {
        if i == 2 {
            let gold = self.world.get_component::<PlayerCharacter>(self.seats[seat].player).map(|p| p.resources).unwrap_or(0.0);
            if gold < BUILD_COST { return Some(format!("need {:.0} gold", BUILD_COST)); }
            if self.build_site_at(seat).is_none() { return Some("no room nearby".into()); }
            return None;
        }
        self.disabled_reason_at(seat, i)
    }

    /// Why a ctx button can't answer right now, if it can't - shown greyed
    /// with the reason beside it, like the main buttons' `blocked`
    pub fn ctx_blocker_at(&self, seat: usize, act: CtxAct) -> Option<String> {
        match act {
            CtxAct::Meet => {
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let due = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
                    .map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false);
                if due { None } else { Some("not due to meet".into()) }
            }
            CtxAct::Gather => {
                if self.meeting.is_some() { Some("a meeting is under way".into()) }
                else {
                    self.world.get_component::<crate::game::organizing::MeetingPlan>(self.seats[seat].org)
                        .map(|p| if p.ready() { None } else { Some(p.missing().to_string()) }).flatten()
                }
            }
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
                // still be pressed
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)?.issue.clone();
                let g = self.world.get_component::<crate::game::reactions::LordGrant>(self.seats[seat].org)?;
                (g.issue == issue && g.active(self.tick) && !g.partial && !g.hollow)
                    .then(|| format!("the Reeve has answered - the grant stands till day {}", g.until))
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
        // Recruit is pointless on a member: say so on the button, as Build
        // does, rather than fail in the log. Talk and Recruit need the
        // selected villager alive (they walk over to them)
        let target = self.villagers[if (i == 0 || i == 1) && self.seats[seat].picked { self.seats[seat].selected_target } else { self.reach_target_at(seat) }];
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let campaign = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org);
        let member = self.world.get_component::<Organization>(self.seats[seat].org).map(|o| o.members.contains(&target)).unwrap_or(false);
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(target).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(target).is_some();
        if (i == 0 || i == 1) && !target_alive { Some("they have died".into()) }
        else if (i == 0 || i == 1) && held { Some("in the dungeon".into()) }
        else if i == 1 && member { Some("already a member".into()) }
        else if i == 1 && campaign.is_none() { Some("no campaign to join".into()) }
        else if i == 1 && !campaign.map(|c| crate::game::player::sympathizes(&self.world, c, target)).unwrap_or(false)
            && crate::game::player::rapport(&self.world, self.seats[seat].player, target) < crate::game::player::RECRUIT_RAPPORT_NEEDED { Some("talk with them more".into()) }
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
        // Uncapped talk count - Rapport caps at 1.0 and would
        // freeze the concern/small-talk index after a few talks.
        let talks_before = self.world.get_component::<TalkCount>(v)
            .map(|c| c.value).unwrap_or(0);
        // a 1-1 has a small chance of drawing out one of their secrets:
        // a want, or the problem that's in their way
        // the fold's smell: folk hold back a touch with the shepherd
        let smelly = desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly);
        let revealed = if (self.rng.next() % 1000) < (desires::REVEAL_BASE * if smelly { 900.0 } else { 1000.0 }) as u64 {
            let pick = self.rng.next();
            desires::next_reveal(&self.world, v, self.tick, pick)
        } else { None };
        let (did, said) = match revealed {
            Some(r) => {
                let (d, s) = desires::reveal_speech(vill_name(who), &r);
                let pe = self.seats[seat].player;
                if let Some(issue) = desires::hear(&mut self.world, pe, v, r, self.tick) {
                    self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                    self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
                }
                (d, s)
            }
            None => talk_response(&self.world, v, vill_name(who), talks_before),
        };
        let pe = self.seats[seat].player;
        // the bond as it stood before this talk: a first talk is with a stranger
        let rel = dialogue::relation_with_wanderer(self.rel_kind(pe, v));
        act_talk(&mut self.world, pe, v, self.tick);
        self.say(v, &said, TALK_BUBBLE_SECS);
        // a blow-off leaves them hollow: a real talk coaches them back
        let coached = if self.world.get_component::<crate::game::actions::Demoralized>(v).is_some()
            && crate::game::actions::coach(&mut self.world, pe, v, self.tick) {
            " You talk the helplessness out of them."
        } else { "" };
        // the conversation itself goes on the TALK panel, the replies yours to pick
        self.open_player_talk(seat, v, rel, &said);
        if !coached.is_empty() { self.narrate_talk(seat, coached.trim()); }
        // and if they were the one the job was for, it's handed over now
        let quest_note = self.quest_talk_done(seat, v);
        let mut line = format!("{}: \"{}\"{}", did, said, coached);
        if let Some(q) = quest_note {
            self.log.push(q);
            line.push_str(" - and the job's done.");
        }
        line
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
        self.map.walk_path((px.round() as i32, py.round() as i32), (p.x.round() as i32, p.y.round() as i32))
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
                Some(format!("You walk over to {}.", vill_name(who)))
            }
            None => Some(format!("Can't {}: there's no way to reach {}.",
                ACTION_LABELS[action].to_lowercase(), vill_name(who))),
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
                None => { self.log.push(format!("t{}: You can't find a way to {}.", self.tick, vill_name(who))); return; }
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
            self.log.push(format!("t{}: You can't get through to {}.", self.tick, vill_name(who)));
            return;
        }
        self.seats[seat].approach = Some(a);
    }

    /// Take action button `i` (Talk, Recruit, Build)
    pub fn act_at(&mut self, seat: usize, i: usize) {
        let tick = self.tick;
        // Talk and Recruit go to whoever the player picked; otherwise to
        // whoever is in reach
        if (i == 0 || i == 1) && !self.seats[seat].picked { self.seats[seat].selected_target = self.reach_target_at(seat); }
        let selected_target = self.seats[seat].selected_target;
        // a Talk's words go on the TALK panel, not the event log
        let mut talked = false;
        let msg = if let Some(reason) = self.action_blocker_at(seat, i) {
            format!("Can't {}: {}.", ACTION_LABELS[i].to_lowercase(), reason)
        } else {
            match i {
                0 => match self.start_approach_at(seat, selected_target, 0) {
                    Some(walking) => walking,
                    None => { talked = true; self.talk_to_at(seat, selected_target) }
                },
                1 => if let Some(walking) = self.start_approach_at(seat, selected_target, 1) { walking } else {
                    let (pe, oe, te) = (self.seats[seat].player, self.seats[seat].org, self.villagers[selected_target]);
                    let r = act_recruit(&mut self.world, pe, oe, te);
                    if r.success { format!("{} joined!", vill_name(selected_target)) }
                    else { format!("{}: {}", vill_name(selected_target), r.description.to_lowercase()) }
                }
                2 => {
                    // action_blocker checked there's gold and a site
                    let (bx, by) = self.build_site_at(seat).unwrap();
                    if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.seats[seat].player) { p.resources -= BUILD_COST; }
                    self.buildings.push((bx, by, BSprite::House(self.rng.below(97) as usize)));
                    self.map.build_at(bx as i32, by as i32);
                    // the road is laid a tile at a time, then the house goes up
                    let road = self.map.plan_road((bx as i32, by as i32));
                    self.works.push(Works { building: self.buildings.len() - 1, road, frames: 0 });
                    self.built_count += 1;
                    format!("Work starts on a new house beside you - {:.0} gold", BUILD_COST)
                }
                _ => unreachable!("no action button {}", i),
            }
        };
        // every action line is dated like the rest of the log. It doesn't
        // show a score: the log reads as what happened in the village
        if !talked { self.log.push(format!("t{}: {}", tick, msg)); }
        // gold spent here is the same purse the day's trading counts
        let pe = self.seats[seat].player; sync_purse(&mut self.world, pe, true);
    }

    /// The contextual actions that apply right now: work at the player's
    /// shop, pair a picked junior with a master, send a kid on an errand
    pub fn update_ctx_actions_at(&mut self, seat: usize) {
        self.seats[seat].ctx_actions.clear();
        let (px, py) = self.player_pos_at(seat);
        // Work: the player's own craft, at the bench if they have one
        if let Some(c) = self.world.get_component::<Craft>(self.seats[seat].player) {
            let near = c.workplace
                .and_then(|b| self.world.get_component::<Pos>(b))
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < 4.0)
                .unwrap_or(true);
            if near {
                self.seats[seat].ctx_actions.push(CtxAct::Work);
            }
        }
        if self.seats[seat].selected_building.is_none() {
            let v = self.villagers[self.seats[seat].selected_target];
            if self.world.get_component::<Kid>(v).is_some() {
                self.seats[seat].ctx_actions.push(CtxAct::Errand);
            } else if let Some(c) = self.world.get_component::<Craft>(v) {
                let junior = vocation::level_of(c.skill) < Level::Master && c.master.is_none();
                let has_master = junior && self.villagers.iter().any(|&m| {
                    m != v && self.world.get_component::<Craft>(m)
                        .map(|mc| mc.vocation == c.vocation && vocation::level_of(mc.skill) == Level::Master).unwrap_or(false)
                });
                if has_master {
                    self.seats[seat].ctx_actions.push(CtxAct::PairUp);
                }
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
            // and while they're asking within earshot: the odd job
            // they've got going begging, till the seat's hands are full
            let in_earshot = self.world.get_component::<Pos>(v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= TALK_DIST)
                .unwrap_or(false);
            if self.seats[seat].quest.is_none() && self.quest_offer_at(v).is_some() && in_earshot {
                self.seats[seat].ctx_actions.push(CtxAct::TakeJob);
            }
            // a picked neighbour with no pact yet can be sworn to, wax
            // and ten gold willing
            let pe = self.seats[seat].player;
            let gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
            if self.seats[seat].picked && !villager::is_dead(&self.world, v)
                && self.world.get_component::<Kid>(v).is_none()
                && !crate::game::dungeon::unavailable(&self.world, v)
                && gold >= crate::game::contracts::PACT_AID
                && !crate::game::contracts::has_pact(&self.world, pe, v) {
                self.seats[seat].ctx_actions.push(CtxAct::Pact);
            }
        }
        // a stray cat keeping you company can be stroked - "Pet her"
        // shows while she's in reach, whatever else is going on
        if self.stray_cat_at(seat).is_some() {
            self.seats[seat].ctx_actions.push(CtxAct::PetCat);
        }
        // a house meeting is earned: the button shows once a place and a
        // promise are on the plan
        if self.world.get_component::<crate::game::organizing::MeetingPlan>(self.seats[seat].org)
            .map(|p| p.ready()).unwrap_or(false) {
            self.seats[seat].ctx_actions.push(CtxAct::Gather);
        }
        // winter's own button: once a day, everyone free can be called out
        // to a snowball fight - it shares Gather's row on the panel
        if self.world.get_component::<crate::sim::climate::Climate>(self.climate)
            .map(|c| c.season == crate::sim::climate::Season::Winter).unwrap_or(false)
            && self.snowball_day != self.tick {
            self.seats[seat].ctx_actions.push(CtxAct::Snowballs);
        }
        // the cause meets on its cadence and marches on the streets: the
        // buttons show once a campaign exists; seats go to the picked member
        let cause = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org).is_some();
        if cause {
            self.seats[seat].ctx_actions.push(CtxAct::Meet);
            self.seats[seat].ctx_actions.push(CtxAct::Cadence);
            self.seats[seat].ctx_actions.push(CtxAct::March);
        }
        if self.seats[seat].selected_building.is_none() && self.seats[seat].picked {
            let v = self.villagers[self.seats[seat].selected_target];
            if !crate::game::villager::is_dead(&self.world, v) {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                    // someone's inside: the way out is pressure - the
                    // letters go on whether the cause still marches or not
                    self.seats[seat].ctx_actions.push(CtxAct::Petition);
                } else if !crate::game::dungeon::unavailable(&self.world, v) {
                    let is_member = self.world.get_component::<Organization>(self.seats[seat].org)
                        .map(|o| o.members.contains(&v)).unwrap_or(false);
                    // a trusted ear can be won to the cause: it takes a tie
                    // of its own, not just the cause existing
                    let trust = crate::game::relationships::find_relationship(&self.world, self.seats[seat].player, v)
                        .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                        .map(|r| r.strength).unwrap_or(0.0);
                    if !is_member && trust >= crate::sim::community::RECRUIT_MIN_TRUST
                        && self.world.get_component::<crate::session::country::Outlander>(v).is_none() {
                        self.seats[seat].ctx_actions.push(CtxAct::Recruit);
                    }
                    if cause {
                        if is_member {
                            self.seats[seat].ctx_actions.push(CtxAct::Delegate);
                        }
                        if self.world.get_component::<Craft>(v).is_some() {
                            self.seats[seat].ctx_actions.push(CtxAct::Strike);
                        }
                        if crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)
                            .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
                            .map(|c| c.target == Some(v)).unwrap_or_else(|| Some(v) == self.collector) {
                            self.seats[seat].ctx_actions.push(CtxAct::Townhall);
                        }
                    }
                }
            }
        }
    }

    /// Take a contextual action (entry `i` of ctx_actions)
    pub fn ctx_act_at(&mut self, seat: usize, i: usize) {
        let Some(&act) = self.seats[seat].ctx_actions.get(i) else { return };
        let tick = self.tick;
        if let Some(reason) = self.ctx_blocker_at(seat, act) {
            self.log.push(format!("t{}: Can't {}: {}.", tick, ctx_button(act).label.to_lowercase(), reason));
            return;
        }
        let msg = match act {
            CtxAct::Work => {
                let pe = self.seats[seat].player;
                let (voc, workplace, tasked_by) = self.world.get_component::<Craft>(pe)
                    .map(|c| (Some(c.vocation), c.workplace, c.tasked_by)).unwrap_or((None, None, None));
                // nothing comes of a day without the makings: say what's
                // short and how to come by it, not just "nothing yet"
                let missing: Vec<&'static str> = voc.map(|v| vocation::inputs(v).iter().copied()
                    .filter(|g| {
                        // a slot's only missing when nothing that fills
                        // it is to hand - the named good or its fellows
                        std::iter::once(*g).chain(crate::game::recipes::fillers(g).iter().copied())
                            .all(|c| {
                                let held = self.world.get_component::<Satchel>(pe)
                                    .map(|s| s.goods.get(c).copied().unwrap_or(0)).unwrap_or(0);
                                let stocked = workplace
                                    .and_then(|b| self.world.get_component::<Building>(b))
                                    .map(|b| b.stock.get(c).copied().unwrap_or(0)).unwrap_or(0);
                                held + stocked < 1
                            })
                    }).collect()).unwrap_or_default();
                if !missing.is_empty() {
                    let how: Vec<String> = missing.iter().map(|g| {
                        let opts: Vec<&'static str> = std::iter::once(*g)
                            .chain(crate::game::recipes::fillers(g).iter().copied()).collect();
                        let what = opts.join(" or ");
                        if opts.iter().any(|o| vocation::gatherable(o)) { format!("{} - gather some", what) }
                        else { format!("{} - buy some", what) }
                    }).collect();
                    format!("You can't work: no {} to hand.", how.join(", "))
                } else {
                    let what = voc.and_then(vocation::output).unwrap_or("goods");
                    // where the day's work actually lands is read off the
                    // before-and-after: a full shelf spills to the satchel,
                    // and a full satchel loses it
                    let held_before = self.world.get_component::<Satchel>(pe)
                        .map(|s| s.goods.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    let stocked_before = workplace
                        .and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    // a tasked junior's share goes through the senior's shop
                    // and comes back as pay - neither is lost work
                    let senior_shop = tasked_by
                        .and_then(|s| self.world.get_component::<Craft>(s))
                        .and_then(|c| c.workplace);
                    let senior_before = senior_shop
                        .and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    let gold_before = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
                    let (made, up) = self.bench_work(pe, tick);
                    match up {
                        Some(l) => format!("You work - and reach {}!", vocation::level_name(l)),
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
                                    .map(|i| vill_name(i)).unwrap_or("your senior");
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
            }
            CtxAct::PairUp => {
                let v = self.villagers[self.seats[seat].selected_target];
                let voc = self.world.get_component::<Craft>(v).map(|c| c.vocation);
                let master = voc.and_then(|voc| self.villagers.iter().copied().find(|&m| {
                    m != v && self.world.get_component::<Craft>(m)
                        .map(|mc| mc.vocation == voc && vocation::level_of(mc.skill) == Level::Master).unwrap_or(false)
                        && !crate::game::dungeon::unavailable(&self.world, m)
                }));
                match master {
                    Some(m) => {
                        self.world.get_component_mut::<Craft>(v).unwrap().master = Some(m);
                        relationships::add_relationship(&mut self.world, relationships::Relationship {
                            from: v, other: m, kind: relationships::RelKind::Apprentice, strength: 0.3, interactions: 1 });
                        let mi = self.villagers.iter().position(|x| *x == m).unwrap();
                        format!("You pair {} with {} - he'll learn faster now.", vill_name(self.seats[seat].selected_target), vill_name(mi))
                    }
                    None => "No master of their trade to pair them with.".into(),
                }
            }
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
                            format!("You send {} to fetch {}.", vill_name(self.seats[seat].selected_target), g)
                        } else {
                            let pname = self.villagers.iter().position(|&p| p == hirer)
                                .map(vill_name).unwrap_or("the printer");
                            format!("You send {} to fetch {} for {}.",
                                vill_name(self.seats[seat].selected_target), g, pname)
                        }
                    }
                    None => "Nothing needs fetching right now.".into(),
                }
            }
            CtxAct::Meet => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
                    return;
                };
                match crate::game::campaign::hold_campaign_meeting(&mut self.world, c, tick) {
                    Some(out) => {
                        let who: Vec<String> = out.attendees.iter()
                            .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string()))
                            .collect();
                        // the room talks it through first: what it decided is
                        // what the notes and the consent report
                        let d = crate::game::campaign_dialogue::deliberate_meeting(&mut self.world, c, &out, tick);
                        let decided = d.as_ref().map(|d| d.decided).unwrap_or(true);
                        let consent = d.as_ref().map(|d| d.consent).unwrap_or(out.consent);
                        let mut msg = format!("{} meet about the campaign: {}", out.attendees.len(), who.join(", "));
                        if out.drifted { msg += " - run by whoever pulled hardest, nobody asked"; }
                        if out.noted && decided { msg += " - the notes keep the resolves"; }
                        if !decided { msg += " - nothing settled"; }
                        if consent { msg += " - consent banked for the next action"; }
                        if !out.healed.is_empty() {
                            let healed: Vec<String> = out.healed.iter()
                                .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string())).collect();
                            msg += &format!(" - {} heard out", healed.join(", "));
                        }
                        // the log keeps the outcome; what the room said goes
                        // on the TALK panel, in the order it was said
                        if let Some(d) = d {
                            let mut with = vec![self.seats[seat].player];
                            with.extend(out.attendees.iter().copied());
                            self.record_campaign_talk(with, &d);
                        }
                        msg
                    }
                    None => "The campaign isn't due to meet yet.".into(),
                }
            }
            CtxAct::Cadence => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
                    return;
                };
                // the organizer's recommendation: the meeting steps up or
                // down a notch - weekly for the long haul, daily for the fight
                let now = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
                    .map(|m| m.cadence_days).unwrap_or(3);
                let next = match now { 0..=3 => 7, 4..=7 => 14, 8..=14 => 30, _ => 3 };
                crate::game::campaign::set_cadence(&mut self.world, c, next, tick);
                format!("The cause will meet every {} days now - the organizer's word.", next)
            }
            CtxAct::Delegate => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let v = self.villagers[self.seats[seat].selected_target];
                let name = vill_name(self.seats[seat].selected_target);
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
                            Some(r) if crate::game::campaign::delegate_role(&mut self.world, c, r, v) =>
                                format!("You name {} the campaign's {}.", name, role_name(r)),
                            Some(_) => format!("{} can't hold a seat - they have to be a living member.", name),
                            None => "Every seat is spoken for - someone would have to step down.".into(),
                        }
                    }
                }
            }
            CtxAct::March => { let m = self.march_at(seat); self.log.push(format!("t{}: {}", tick, m)); return; }
            CtxAct::Strike => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let v = self.villagers[self.seats[seat].selected_target];
                let Some(voc) = self.world.get_component::<Craft>(v).map(|c| c.vocation) else {
                    return;
                };
                match crate::game::actions::strike_vote(&mut self.world, c, voc, tick) {
                    Some(out) => {
                        let ev = crate::game::campaign_arc::ArcEvent::Strike { called: out.called };
                        if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                            self.record_campaign_talk(vec![self.seats[seat].player, v], &d);
                        }
                        format!("{} of {} say yes{}. {}", out.yes, out.yes + out.no,
                            if out.called { " - the strike is on" } else { "" },
                            if out.called { "The trade stands together." } else { "Less than nine of ten: it fails in public." })
                    }
                    None => "Nobody of the trade to ballot.".into(),
                }
            }
            CtxAct::Townhall => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let Some(target) = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.target).unwrap_or(None).or(self.collector) else { return };
                use crate::game::actions::{townhall, TargetResponse};
                match townhall(&mut self.world, c, target, tick) {
                    Some(out) => { let msg = match out.response {
                        TargetResponse::Carries => {
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
                    if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                        let mut with = vec![self.seats[seat].player, target];
                        with.extend(out.turnout.iter().copied());
                        self.record_campaign_talk(with, &d);
                    }
                    msg },
                    None => "Nobody turns out for the townhall.".into(),
                }
            }
            CtxAct::Gather => self.call_meeting_at(seat),
            CtxAct::Petition => {
                let v = self.villagers[self.seats[seat].selected_target];
                // the cause's muscle signs the letters too - every member's
                // name on the vigil carries more than a stranger's plea
                let sway = crate::sim::community::exert_influence(&mut self.world,
                    self.seats[seat].org, "the release");
                crate::game::dungeon::press(&mut self.world, v, 0.3 + sway * 0.15);
                // the hold: what the state spends to keep them inside
                if crate::game::dungeon::release_check(&mut self.world, v, 1.0) {
                    let name = vill_name(self.seats[seat].selected_target);
                    self.toasts.push_at((format!("{} walks free", name), tick + 40), toast_spot(&self.world, v));
                    format!("The letters don't stop - and the cell opens. {} walks free.", name)
                } else {
                    let p = self.world.get_component::<crate::game::dungeon::ReleaseCampaign>(v)
                        .map(|r| r.pressure).unwrap_or(0.0);
                    format!("Letters, vigils, a line out the door ({:.0}% there).", p * 100.0)
                }
            }
            CtxAct::Recruit => {
                let v = self.villagers[self.seats[seat].selected_target];
                let oe = self.seats[seat].org;
                let trust = crate::game::relationships::find_relationship(&self.world, self.seats[seat].player, v)
                    .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                    .map(|r| r.strength).unwrap_or(0.0);
                if crate::sim::community::recruit(&mut self.world, oe, v, trust) {
                    format!("{} hears you out - and signs on with your cause.", self.name_of(v))
                } else {
                    format!("{} won't be won over - the trust isn't there yet.", self.name_of(v))
                }
            }
            CtxAct::Pact => {
                let v = self.villagers[self.seats[seat].selected_target];
                let pe = self.seats[seat].player;
                let mut gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
                match crate::game::contracts::seal_aid_pact(&mut self.world, pe, v, &mut gold) {
                    Some(_) => {
                        // both purses: trades move the Purse, the panel
                        // reads resources - a build only syncs one way
                        if let Some(p) = self.world.get_component_mut::<Purse>(pe) { p.gold = gold; }
                        if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(pe) { p.resources = gold; }
                        let n = self.name_of(v);
                        self.say(v, "It's sworn - you've a friend in me.", TALK_BUBBLE_SECS);
                        note_story(&mut self.tales, &mut self.tales_told, &mut self.log,
                            self.seats[seat].selected_target, tick,
                            crate::sim::story::EventKind::Alliance, "sworn to aid the Wanderer".into());
                        format!("You press wax with {} - sworn to aid each other ({:.0}g).", n, crate::game::contracts::PACT_AID)
                    }
                    None => "There's no sealing it - too poor, or sworn already.".into(),
                }
            }
            CtxAct::TakeJob => self.accept_quest_at(seat),
            CtxAct::PetCat => self.pet_stray_cat_at(seat),
            CtxAct::Snowballs => self.snowball_fight_at(seat),
        };
        if !msg.is_empty() {
            self.log.push(format!("t{}: {}", tick, msg));
        }
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
            self.log.push(format!("t{}: the stray cat winds round your ankles.", tick));
        } else if was < 0.5 && now >= 0.5 {
            self.log.push(format!("t{}: she purrs under your hand - she's warming to you.", tick));
        }
        String::new()
    }

    /// The street: the cause marches. Returns the log line - turnout, the
    /// lord's reaction, and whatever the day brings back to the room
    /// (a crackdown, a taking, the ban proclaimed).
    fn march_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
            return "There's no cause to march for.".into() };
        let pe = self.seats[seat].player;
        match crate::game::actions::demonstrate(&mut self.world, c, pe, tick) {
            Some(out) => {
                let n = out.turnout.len();
                // guards already posted stay posted - a second
                // crackdown doesn't pile more knights into the frame
                let posted_before = self.world
                    .get_component::<crate::game::reactions::GuardsPosted>(self.seats[seat].org).is_some();
                let r = crate::game::reactions::react(&mut self.world, c, &out.turnout, tick);
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
                let mut msg = match r {
                    Some(crate::game::reactions::Reaction::Crackdown) => {
                        self.toasts.push(("knights posted to the streets".into(), tick + 40));
                        format!("{} march in the street - and the knights come out.", n)
                    }
                    Some(crate::game::reactions::Reaction::Watched) =>
                        format!("{} march in the street - the names get taken.", n),
                    _ => format!("{} march in the street - beneath notice, for now.", n),
                };
                if woke > 0 { msg += &format!(" {} watching feel the cause wake.", woke); }
                // a crackdown takes the loudest - and the cause
                // becomes a prisoner
                if r == Some(crate::game::reactions::Reaction::Crackdown) {
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
                    }
                    // every crackdown can still take someone
                    if let Some(taken) = crate::game::dungeon::arrest(&mut self.world, c, tick) {
                        // hauled off mid-errand: whatever they were doing is dropped
                        self.travels.retain(|t| t.who != taken);
                        let tn = self.villagers.iter().position(|v| *v == taken).map(|i| vill_name(i))
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
                // the campaign talks over what the street brought
                // (after any arrest, so the taken are counted)
                let ev = crate::game::campaign_arc::ArcEvent::March(
                    r.unwrap_or(crate::game::reactions::Reaction::Ignored));
                if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                    let mut with = vec![pe];
                    with.extend(out.turnout.iter().copied());
                    self.record_campaign_talk(with, &d);
                }
                msg
            }
            None => "You call a march - and nobody comes. The street stays empty.".into(),
        }
    }

    /// The organizer's answer to the founding call on the conversation
    /// box: take the room's leaning up, framed as picked - or let it rest
    /// (the next house meeting can put it again)
    pub fn decide_at(&mut self, seat: usize, pick: usize) {
        let Some(d) = self.seats[seat].pending_founding.take() else { return };
        use crate::game::campaign::{self, DecisionRule, Framing};
        if let Some(mut vote) = d.vote {
            let framing = match pick {
                0 => Framing::SharedGrievance,
                1 => Framing::CommonFoe,
                2 => Framing::MoralDuty,
                _ => {
                    self.log.push(format!("t{}: The talk stays talk for now - no cause taken up.", self.tick));
                    self.note_talk(d.record, None, "The talk stays talk for now - no cause taken up.");
                    return;
                }
            };
            // who grumbles is re-read against the framing actually picked
            for b in &mut vote.ballots {
                b.grumbles = b.stake.as_ref().map(|s| !campaign::framing_suits(framing, s)).unwrap_or(false);
            }
            let msg = self.found_and_note(seat, &vote, framing, DecisionRule::Majority);
            self.log.push(format!("t{}: {}", self.tick, msg));
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
                let msg = "They'll gather again - you'll need a place to meet.".to_string();
                self.log.push(format!("t{}: {}", self.tick, msg));
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
        self.log.push(format!("t{}: The talk stays talk for now.", self.tick));
        self.note_talk(d.record, None, "The talk stays talk for now.");
    }
}
