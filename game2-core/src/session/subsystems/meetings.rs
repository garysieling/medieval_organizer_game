//! Talk and meetings: speech and thought bubbles, chat in passing, house meetings, and founding the org and its campaign.

use crate::session::*;

impl Game {
    /// The end of a house meeting, once those who came have talked it over.
    /// Those who already trust you join the org; everyone who came gets to
    /// know you a little better (organizing::house_meeting). The meeting
    /// belongs to the seat that called it; `record` is its talk_log id, so
    /// a founding call can keep its tail showing on the conversation box.
    pub fn resolve_meeting(&mut self, invited: &[Entity], seat: usize, record: u64, roles: (bool, bool, bool, bool, bool)) -> String {
        let (oe, pe) = (self.seats[seat].org, self.seats[seat].player);
        let members_before = self.world.get_component::<Organization>(oe).map(|o| o.members.clone()).unwrap_or_default();
        let r = crate::game::organizing::house_meeting(&mut self.world, oe, invited, self.tick);
        // the attendees' meeting lesson comes out of house_meeting itself;
        // here a seated campaign hand practices their seat's craft, and the
        // host's HouseMeeting grows by running the room
        if let Some(c) = crate::game::campaign::campaign_of(&self.world, oe) {
            let seats = self.world.get_component::<crate::game::campaign::Campaign>(c)
                .map(|cc| cc.seats.clone()).unwrap_or_default();
            for s in &seats {
                if invited.contains(&s.holder) {
                    crate::game::org_skills::practice(&mut self.world, s.holder,
                        crate::game::org_skills::role_skill(s.role),
                        crate::game::org_skills::SEAT_PRACTICE);
                }
            }
        }
        crate::game::org_skills::practice(&mut self.world, pe,
            crate::game::org_skills::OrgSkill::HouseMeeting,
            crate::game::org_skills::MEETING_PRACTICE);
        let joined: Vec<&str> = invited.iter()
            .filter(|v| !members_before.contains(v))
            .filter(|v| self.world.get_component::<Organization>(oe).map(|o| o.members.contains(v)).unwrap_or(false))
            .filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| vill_name(i)))
            .collect();
        let outcome = if joined.is_empty() {
            "No one joins yet, but they'll remember the conversation.".to_string()
        } else {
            format!("{} joined{}!", joined.join(", "), if r.formed_committee { " and formed a committee" } else { "" })
        };
        // a meeting also succeeds when it surfaces a pattern: three or more
        // voices with versions of the same trouble become an issue
        let mut tags: crate::engine::det::HashMap<desires::IssueTag, Vec<Entity>> = Default::default();
        for (who, tag) in &self.voiced { tags.entry(tag.clone()).or_default().push(*who); }
        let mut found = String::new();
        for (tag, voices) in &tags {
            let mut vs = voices.clone();
            vs.sort_unstable();
            vs.dedup();
            if vs.len() >= 3 {
                if let Some(issue) = desires::found_issue(&mut self.world, pe, tag.clone()) {
                    self.toasts.push((
                        format!("Organizer discovers issue {}", issue),
                        self.tick + 40,
                    ));
                    self.log_entity_ev(
                        EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                        pe,
                        format!("t{}: Organizer discovers issue {}", self.tick, issue),
                    );
                    found = format!(" Organizer discovers issue {}.", issue);
                }
            }
        }
        // three voices of one trade converge; the same trouble said by
        // three trades splinters - a mason's grief isn't a baker's
        if found.is_empty() {
            let mut bases: crate::engine::det::HashMap<String, crate::engine::det::HashSet<Entity>> = Default::default();
            for (tag, voices) in &tags {
                let base = tag.split_once(" (").map(|(b, _)| b).unwrap_or(tag.as_str());
                bases.entry(base.to_string()).or_default().extend(voices.iter().copied());
            }
            if bases.values().any(|vs| vs.len() >= 3) {
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    pe,
                    format!("t{}: the talk splinters by trade.", self.tick),
                );
                found = " The talk splinters by trade.".into();
            }
        }
        // the roles that were kept make the meeting: a shared table warms
        // every tie, notes keep what was settled
        let (snacks, notes, _opener, _song, _closer) = roles;
        let roles_filled = [snacks, notes, _opener, _song, _closer].iter().filter(|r| **r).count();
        if snacks {
            let leader = crate::game::organizing::living_leader(&mut self.world, oe);
            for &a in invited {
                if Some(a) == leader { continue; }
                relationships::record_interaction(&mut self.world, pe, relationships::Interaction {
                    other: a, kind: relationships::InteractionKind::SharedMeal, intensity: 0.4 });
            }
        }
        if notes { self.note_talk(record, None, "The notes keep what was settled."); }
        // a meeting in the street is power spent: the lord notices a
        // full kitchen - and a thin one, thinly run
        self.record_power(if invited.len() >= 3 || roles_filled >= 3 {
            crate::sim::organized_power::ActionQuality::Transformational
        } else if invited.len() >= 2 || roles_filled >= 1 {
            crate::sim::organized_power::ActionQuality::Transactional
        } else { crate::sim::organized_power::ActionQuality::Overreach });
        let founded = self.meeting_call(seat, record, invited).map(|s| format!(" {}", s)).unwrap_or_default();
        format!("House meeting - {} came. {}{}{}", invited.len(), outcome, found, founded)
    }

    /// The founding vote, when the org can take one: enough living
    /// members and a heard issue. The organizer's ledger supplies the
    /// candidates; the room's leaning is read for a shared grievance
    /// (grumbles are re-read against the framing actually picked).
    /// No campaign is founded - that's the organizer's call (decide_at),
    /// or maybe_found_campaign's in tests
    pub(in crate::session) fn founding_vote(&self, seat: usize) -> Option<(crate::game::campaign::IssueVote, String)> {
        use crate::game::campaign;
        let org = self.seats[seat].org;
        if campaign::campaign_of(&self.world, org).is_some() { return None; }
        let living = self.world.get_component::<Organization>(org)
            .map(|o| o.members.iter().filter(|&&m| !villager::is_dead(&self.world, m)).count())
            .unwrap_or(0);
        if living < 3 { return None; }
        let issues = self.world.get_component::<desires::Organizer>(self.seats[seat].player)
            .map(|o| o.issues.clone()).unwrap_or_default();
        if issues.is_empty() { return None; }
        // the ballot names the themes the issues gather under, not the
        // thin tags they were heard as (#1072)
        let candidates: Vec<desires::IssueTag> = desires::platforms(&issues).into_iter().map(|(c, _)| c).collect();
        let vote = campaign::hold_issue_vote(&self.world, org, &candidates, campaign::Framing::SharedGrievance, self.tick)?;
        let oname = self.world.get_component::<Organization>(org).map(|o| o.name.clone()).unwrap_or_default();
        Some((vote, oname))
    }

    /// Found the campaign off a settled vote: the entity, its meeting
    /// cadence, the toast and the "they'll fight for" line for the log
    pub(in crate::session) fn found_and_note(&mut self, seat: usize, vote: &crate::game::campaign::IssueVote,
                      framing: crate::game::campaign::Framing, rule: crate::game::campaign::DecisionRule) -> String {
        use crate::game::campaign;
        let org = self.seats[seat].org;
        let oname = self.world.get_component::<Organization>(org).map(|o| o.name.clone()).unwrap_or_default();
        // a woman's banner doesn't fly while the vote's unasked - the
        // room won't follow what the village won't call a voice
        let woman_led = self.world.get_component::<Organization>(org)
            .map(|o| self.world.get_component::<villager::Sex>(o.leader)
                .map(|s| *s == villager::Sex::Female).unwrap_or(false))
            .unwrap_or(false);
        if woman_led && !self.suffrage_won && vote.winner != "the vote" {
            return "the room won't follow - not while a woman's say in the village goes unasked.".into();
        }
        let c = campaign::found_campaign(&mut self.world, org, vote, framing, rule, self.tick);
        // the vote is won when a room carries it - the question put to
        // the village is the answer it needed
        if vote.winner == "the vote" { self.suffrage_won = true; }
        // a cut issue names its target: the Reeve for the taxes, the
        // overseer for his cut - a hurt nobody stands over, the village
        // mends itself
        if let Some(t) = crate::game::organizing::target_for(&self.world, &vote.winner, self.collector) {
            if let Some(cc) = self.world.get_component_mut::<campaign::Campaign>(c) { cc.target = Some(t); }
            let tname = self.villagers.iter().position(|&v| v == t).map(|i| vill_name(i)).unwrap_or("the Reeve");
            self.log_entity_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                t,
                format!("t{}: the room names {}.", self.tick, tname),
            );
        }
        self.world.add_component(c, campaign::CampaignMeeting {
            cadence_days: 3, last_held: self.tick, held: 0, resolves: 0, driven_by: None });
        // a cause that takes a vote takes a hall: the room it gathers in,
        // till a storm or the roof's own mending says otherwise
        self.world.add_component(org, crate::game::organizing::MeetingHall);
        self.toasts.push((format!("{} take a vote", oname), self.tick + 40));
        let mut msg = format!("{} take a vote - they'll fight for {}", oname, vote.winner);
        // they came along, but when the framing sits wrong on a stake they
        // grumble - heard in the log, not just counted on the ballots
        let grumbler = vote.ballots.iter().find(|b| b.grumbles).map(|b| b.member);
        if let Some(d) = grumbler {
            let name = self.villagers.iter().position(|&v| v == d).map(|i| vill_name(i)).unwrap_or("someone");
            msg += &format!(" - {} {}", name, campaign::grumble_line(framing));
        }
        msg
    }

    /// After a house meeting: the organizer's call. A founding vote when
    /// the org can take one; always the follow-up steps - meet again with
    /// the same folks, or (once a cause exists) take it to the street.
    /// Returns the log line's tail.
    pub(in crate::session) fn meeting_call(&mut self, seat: usize, record: u64, came: &[Entity]) -> Option<String> {
        let org = self.seats[seat].org;
        if came.is_empty() { return None; }
        let street = crate::game::campaign::campaign_of(&self.world, org).is_some();
        let (vote, oname) = match self.founding_vote(seat) {
            Some((v, n)) => (Some(v), n),
            None => (None, self.world.get_component::<Organization>(org).map(|o| o.name.clone()).unwrap_or_default()),
        };
        let lean = match &vote {
            Some(v) => format!("{} lean to {} - your call.", oname, desires::issue_name(v.winner.clone())),
            None => "They'd gather again - or take it further. Your call.".into(),
        };
        self.seats[seat].pending_founding = Some(PendingFounding {
            record, vote, org_name: oname, meet_again: came.to_vec(), street, offered_tick: self.tick });
        Some(lean)
    }

    /// After a house meeting: found the campaign on the vote straight
    /// away, without asking the organizer - kept for tests and for
    /// callers that don't offer the founding call
    pub fn maybe_found_campaign(&mut self, seat: usize) -> Option<String> {
        let (vote, _) = self.founding_vote(seat)?;
        Some(self.found_and_note(seat, &vote,
            crate::game::campaign::Framing::SharedGrievance, crate::game::campaign::DecisionRule::Majority))
    }

    /// Hold a due campaign meeting and note it: the log line names who
    /// came, the room's deliberation goes on the TALK panel. `player` is
    /// the seat's own entity when a seat stands behind the cause, so the
    /// record is filed where they'd look for it; a campaign with no seat
    /// records under the room alone. None when the meeting isn't due or
    /// nobody living can pull it - an empty chair holds no meeting.
    pub(in crate::session) fn hold_and_note_campaign_meeting(&mut self, player: Option<Entity>,
                      c: Entity, tick: u64) -> Option<String> {
        let out = crate::game::campaign::hold_campaign_meeting(&mut self.world, c, tick)?;
        let who: Vec<String> = out.attendees.iter()
            .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string()))
            .collect();
        // the room talks it through first: what it decided is what the
        // notes and the consent report
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
                .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string()))
                .collect();
            msg += &format!(" - {} heard out", healed.join(", "));
        }
        // the log keeps the outcome; what the room said goes on the
        // TALK panel, in the order it was said
        if let Some(d) = d {
            let mut with: Vec<Entity> = player.into_iter().collect();
            with.extend(out.attendees.iter().copied());
            self.record_campaign_talk(with, &d);
        }
        Some(msg)
    }

    /// The day's campaign meetings: a cause that comes due on its
    /// cadence meets on its own - the turnout seat pulls them in, or
    /// whoever's pulling, or the leader. The institution keeps its own
    /// dates like the assembly does; the seat's Meet button holds the
    /// same meeting on paths that never run the day's tick (#1159).
    pub fn campaign_meeting_tick(&mut self, tick: u64) {
        let campaigns = self.world.query::<crate::game::campaign::Campaign>();
        for c in campaigns {
            let player = self.seats.iter()
                .find(|s| crate::game::campaign::campaign_of(&self.world, s.org) == Some(c))
                .map(|s| s.player);
            if let Some(msg) = self.hold_and_note_campaign_meeting(player, c, tick) {
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                    format!("t{}: {}", tick, msg));
            }
        }
    }

    /// Put a speech bubble over `who` for `secs` seconds (replacing any
    /// they already had)
    pub fn say(&mut self, who: Entity, text: &str, secs: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + secs * FPS, false, false));
    }

    /// Put a thought bubble over `who` for `frames` frames (at a meeting)
    pub fn think(&mut self, who: Entity, text: &str, frames: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + frames, true, false));
    }

    /// Ambient chatter as people pass each other: unlike `say`, this never
    /// overwrites a bubble that isn't itself passing chatter while that
    /// bubble is still up (e.g. a Talk reply), so a reply can't be clobbered
    /// by a passing remark on the very next frame. Returns whether the line
    /// was actually shown, so a caller spending a scripted turn (a
    /// conversation line) can tell it was blocked and hold the turn open.
    pub fn say_in_passing(&mut self, who: Entity, text: &str, secs: u32) -> bool {
        let frame = self.frame_n;
        let blocked = self.bubbles.iter().any(|(w, _, until, _, passing)| *w == who && !*passing && frame < *until);
        if blocked { return false; }
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), frame + secs * FPS, false, true));
        true
    }

    /// Organize: call a house meeting at the venue named on the org's
    /// MeetingPlan - or the square, when nobody's lent a house and the
    /// word alone brings them out (#1065) - for everyone the plan's
    /// promises and spread brought in. A meeting is earned, not
    /// summoned: without a ready plan (someone willing to come) this
    /// refuses rather than falling back to rounding up whoever's
    /// simply standing nearby - that was the un-earned meeting this
    /// plan replaced. They walk over; see advance_meeting.
    pub fn call_meeting_at(&mut self, seat: usize) -> String {
        if self.meeting.is_some() { return "A meeting is already under way.".into(); }
        let (oe, pe) = (self.seats[seat].org, self.seats[seat].player);
        use crate::game::organizing::{self as org, MeetingPlan};
        let plan = self.world.get_component::<MeetingPlan>(oe);
        if !plan.map(|p| p.ready()).unwrap_or(false) {
            let empty = MeetingPlan::default();
            let missing = self.world.get_component::<MeetingPlan>(oe).unwrap_or(&empty).missing();
            return format!("No meeting to call yet - {}.", missing);
        }
        let plan: MeetingPlan = std::mem::take(self.world.get_component_mut::<MeetingPlan>(oe).unwrap());
        // no house lent: the cross is the standing ground for a meeting
        // the word alone convened
        let venue = plan.venue.map(|(_, b)| self.world.get_component::<Pos>(b)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or((0, 0)))
            .unwrap_or(self.village);
        let label = plan.venue.and_then(|(h, _)| self.villagers.iter().position(|&v| v == h))
            .map(|i| format!("{}'s house", vill_name(i))).unwrap_or_else(|| "the square".into());
        // the invited: those who said they'd come, those who said
        // they'd bring friends (and the friends they bring), every
        // promise-keeper - and a plus-one for each of the committed,
        // a yes is a door held open (#1075) - then the turnout roll,
        // roughly a coin's toss for the merely invited
        let mut pool: Vec<(Entity, org::Invite)> = plan.holders().iter()
            .map(|&h| (h, org::Invite::Yes)).collect();
        for &s in &plan.spreaders {
            for f in org::friends_of(&self.world, s, 2) { pool.push((f, org::Invite::Pulled)); }
        }
        for &h in &plan.holders() {
            for f in org::friends_of(&self.world, h, 1) { pool.push((f, org::Invite::Pulled)); }
        }
        pool.retain(|(e, _)| *e != pe && self.villagers.contains(e)
            // the woods' bands don't answer village calls
            && self.world.get_component::<rebels::Rebel>(*e).is_none());
        // one invite per person, however many roles brought them up -
        // dedup_by only catches neighbours, and a holder can recur
        // non-adjacently (e.g. the venue-lender also takes notes, with
        // someone else's promise listed between the two); a yes
        // outranks a pull
        let mut deduped: Vec<(Entity, org::Invite)> = vec![];
        for (e, k) in pool {
            match deduped.iter_mut().find(|(de, _)| *de == e) {
                Some((_, dk)) => if k == org::Invite::Yes { *dk = k; },
                None => deduped.push((e, k)),
            }
        }
        let pool = deduped;
        let mut invited: Vec<Entity> = pool.iter()
            .filter(|&&(e, k)| org::shows_up(&self.world, pe, e, venue, k,
                (self.rng.next() % 1000) as f32 / 1000.0, self.tick))
            .map(|&(e, _)| e).collect();
        // the door's thinner than the asking - a count, not a roll of names
        let stayed = pool.len() - invited.len();
        invited.truncate(10);
        // a Talk one of them is standing in gives way to the call -
        // held mid-line they'd never make the door
        for s in 0..self.seats.len() {
            if self.seats[s].talk.as_ref().map(|t| invited.contains(&t.who)).unwrap_or(false) {
                self.close_player_talk(s, None);
            }
        }
        if invited.is_empty() {
            return format!("Word goes out - and nobody comes to {}. You'll have to ask again.", label);
        }
        self.travels.retain(|t| !invited.contains(&t.who));
        let mut ground: Vec<(i32, i32)> = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (venue.0 + dx, venue.1 + dy)))
            .filter(|&at| at != venue && self.map.passable(at.0, at.1)).collect();
        ground.sort_by_key(|&(x, y)| (x - venue.0).abs() + (y - venue.1).abs());
        let spots: Vec<(i32, i32)> = (0..invited.len()).map(|i| *ground.get(i).unwrap_or(&venue)).collect();
        let paths = invited.iter().zip(&spots).map(|(v, &spot)| {
            let p = self.world.get_component::<Pos>(*v).copied()?;
            let from = (p.x.round() as i32, p.y.round() as i32);
            if from == spot { Some(vec![spot]) } else { self.trail_path(from, spot) }
        }).collect();
        let roles = (plan.snacks.is_some(), plan.notes.is_some(), plan.opener.is_some(),
            plan.song.is_some(), plan.closer.is_some());
        let role_people = [plan.snacks, plan.notes, plan.opener, plan.song, plan.closer];
        let names: Vec<&str> = invited.iter().filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| vill_name(i))).collect();
        let mut msg = format!("Word goes out - {} gather at {}.", names.join(", "), label);
        if stayed > 0 {
            msg = format!("{} - {} of those asked stayed home.", msg.trim_end_matches('.'), stayed);
        }
        self.meeting = Some(Meeting { label, venue, born: self.tick, invited, spots, paths, frames: 0, talk_from: None, lines: vec![], talk_id: 0, seat, roles, role_people });
        msg
    }

    /// What's said at a meeting: the player opens, each villager there says
    /// what's on their mind - now and then a secret slips out, and every
    /// parallel complaint already voiced makes the next speaker likelier
    /// to share their own. Three voices in a family surface an issue.
    pub fn meeting_lines(&mut self, came: &[Entity], seat: usize, roles: (bool, bool, bool, bool, bool)) -> Vec<(Entity, String)> {
        // the seat that called it hosts: its player asks, and hears what comes out
        let host_e = self.seats[seat].player;
        if came.is_empty() { return vec![]; }
        let mut lines = vec![(host_e, MEETING_PROMPTS[0].to_string())];
        self.voiced.clear();
        // what each attendee says when no secret comes out of them: the
        // meeting's talk from conversation::run_meeting - answers to the
        // question, agreement and pushback threaded on what's been said -
        // seeded off the meeting, not the game's rng
        let mut said = {
            use crate::game::conversation::{run_meeting, MeetingLine, Rng as TalkRng, Speaker};
            use crate::game::dialogue_trees::{meeting_pool, MEET_PROMPT};
            let speakers: Vec<Speaker> = came.iter()
                .map(|&v| dialogue::speaker_of(&self.world, v, &self.name_of(v), self.tick)).collect();
            let pools: Vec<Vec<MeetingLine>> = speakers.iter().map(meeting_pool).collect();
            let pool_refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
            let refs: Vec<&Speaker> = speakers.iter().collect();
            let host = self.world.get_component::<PlayerCharacter>(host_e).map(|p| p.name.clone()).unwrap_or_default();
            let seed = self.tick.wrapping_mul(0x51_7CC1) ^ came.iter().fold(0u64, |h, &v| h.rotate_left(7) ^ v as u64);
            let record = run_meeting(&Speaker::new(&host), &refs, &MEET_PROMPT, &pool_refs, &mut TalkRng::new(seed));
            // per attendee, in the order they said them: (statement index,
            // the statement it answers, the words)
            let mut q: Vec<Vec<(usize, Option<usize>, String)>> = vec![vec![]; came.len()];
            for (k, st) in record.into_iter().enumerate().skip(1) {
                if let Some(d) = q.get_mut(st.speaker) { d.push((k, st.replying_to, st.text)); }
            }
            q
        };
        // the meeting runs in rounds, not in run_meeting's order, so a
        // reply is only said once what it answers has been (statement 0 is
        // the host's question) - never an answer ahead of its question
        let mut aired: crate::engine::det::HashSet<usize> = [0].into_iter().collect();
        for round in 0..2 {
            for (ci, &v) in came.iter().enumerate() {
                let Some(i) = self.villagers.iter().position(|x| *x == v) else { continue };
                let mine: crate::engine::det::HashSet<desires::IssueTag> =
                    desires::unrevealed(&self.world, v, self.tick).iter()
                        .map(|r| desires::issue_tag(&self.world, v, r)).collect();
                let parallel = self.voiced.iter().filter(|(_, t)| mine.contains(t)).count();
                let mut chance = desires::REVEAL_BASE + desires::REVEAL_PARALLEL * parallel as f32;
                // an opener or a song loosens tongues: each promised role
                // that came off warms the room to share a little more
                if roles.2 { chance += 0.05; }
                if roles.3 { chance += 0.05; }
                // the fold's smell hangs on a shepherd: a touch slower to share
                if desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly) { chance *= 0.9; }
                let roll = (self.rng.next() % 1000) as f32 / 1000.0;
                let mut shared = false;
                if roll < chance {
                    let pick = self.rng.next();
                    if let Some(r) = desires::next_reveal(&self.world, v, self.tick, pick) {
                        let (_, said) = desires::reveal_speech(vill_name(i), &r);
                        self.voiced.push((v, desires::issue_tag(&self.world, v, &r)));
                        let pe = host_e;
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
                        lines.push((v, said));
                        shared = true;
                    }
                }
                if !shared {
                    let ready = said[ci].iter().position(|(_, to, _)| to.map_or(true, |t| aired.contains(&t)));
                    let line = match ready {
                        Some(j) => { let (k, _, text) = said[ci].remove(j); aired.insert(k); text }
                        None => meeting_remark(&self.world, v, vill_name(i), round),
                    };
                    lines.push((v, line));
                }
            }
            lines.push((host_e, MEETING_PROMPTS[round + 1].to_string()));
        }
        lines
    }

    /// The scheduled parts of a meeting, spliced into its lines and
    /// felt in the room: the opener welcomes first, the bread goes
    /// round early (and fills a food-want), the song warms the middle
    /// (and fills a music-want), the notes keep the word and the
    /// closer sends them off. A part only happens if the one who
    /// promised it made it there - the caller already kept `roles`
    /// honest about that (#1076)
    fn meeting_beats(&mut self, m: &mut Meeting) {
        let [snacks, notes, opener, song, closer] = m.role_people;
        if let (true, Some(n)) = (m.roles.1, notes) {
            m.lines.push((n, "Say it plain - I'll keep the word of it.".into()));
        }
        if let (true, Some(c)) = (m.roles.4, closer) {
            m.lines.push((c, "Same time again, then - and each of you bring one more.".into()));
        }
        if let (true, Some(s)) = (m.roles.3, song) {
            for &a in &m.invited {
                desires::satisfy(&mut self.world, a, desires::Want::Music, self.tick + 30);
            }
            let mid = m.lines.len() / 2;
            m.lines.insert(mid, (s, "The rain came down on the road to the fair, and who should meet me there...".into()));
        }
        if let (true, Some(s)) = (m.roles.0, snacks) {
            for &a in &m.invited {
                desires::satisfy(&mut self.world, a, desires::Want::GoodFood, self.tick + 15);
            }
            m.lines.insert(1.min(m.lines.len()), (s, "Bread's on the table - help yourselves.".into()));
        }
        if let (true, Some(o)) = (m.roles.2, opener) {
            m.lines.insert(0, (o, "Friends - you came, and that's the start of it.".into()));
        }
    }

    /// One frame of a house meeting: while people gather, each walks a
    /// step towards their spot; once they're all there (or the host has
    /// waited GATHER_FRAMES) they talk in turns, then it resolves
    pub fn advance_meeting(&mut self) {
        // the meeting slows the calendar: read the stretched day before
        // the take below hides it, so its lines and its gather keep the
        // same share of a meeting-day whatever the watch speed - else a
        // fast day ticks by mid-sentence
        let day = self.ticks_per_frame() as u64;
        let Some(mut m) = self.meeting.take() else { return };
        m.frames += 1;
        let step = MEETING_WALK * self.walk_pace();
        let line = ((LINE_FRAMES as u64 * day) / (2 * 21 * FPS as u64)).max(1) as u32;
        let gather = ((GATHER_FRAMES as u64 * day) / (2 * 21 * FPS as u64)).max(1) as u32;
        // those who've arrived stand at their spot (no strolling off mid-meeting);
        // a dead invitee's grave stays where they died, it doesn't keep walking
        for (i, v) in m.invited.iter().enumerate() {
            if crate::game::dungeon::unavailable(&self.world, *v) { continue; }
            // one in a Talk with a player isn't walked to their spot mid-line
            if self.world.get_component::<villager::InPlayerTalk>(*v).is_some() { continue; }
            let Some(p) = self.world.get_component_mut::<Pos>(*v) else { continue };
            match &mut m.paths[i] {
                Some(path) if !path.is_empty() => { Self::follow_path(p, path, step); }
                Some(_) => { p.x = m.spots[i].0 as f32; p.y = m.spots[i].1 as f32; }
                None => {}
            }
        }
        // arrived at their spot, and alive - a dead invitee never counts as
        // having attended even if they got there before dying
        let arrived = |m: &Meeting, world: &World, i: usize| {
            !crate::game::dungeon::unavailable(world, m.invited[i]) && m.paths[i].as_ref().map(|p| p.is_empty()).unwrap_or(false)
        };
        match m.talk_from {
            None => {
                if (0..m.invited.len()).all(|i| arrived(&m, &self.world, i)) || m.frames >= gather {
                    // whoever didn't get there in time (or died on the way) isn't at the meeting
                    let came: Vec<usize> = (0..m.invited.len()).filter(|&i| arrived(&m, &self.world, i)).collect();
                    m.invited = came.iter().map(|&i| m.invited[i]).collect();
                    m.spots = came.iter().map(|&i| m.spots[i]).collect();
                    m.paths = came.iter().map(|&i| m.paths[i].clone()).collect();
                    // a role counts kept only if its keeper made it: a
                    // promised song nobody sang warms nothing
                    let in_room = |e: Option<Entity>| e.map(|e| m.invited.contains(&e)).unwrap_or(false);
                    m.roles = (in_room(m.role_people[0]), in_room(m.role_people[1]),
                        in_room(m.role_people[2]), in_room(m.role_people[3]), in_room(m.role_people[4]));
                    m.lines = self.meeting_lines(&m.invited, m.seat, m.roles);
                    self.meeting_beats(&mut m);
                    m.talk_from = Some(m.frames);
                    // the TALK panel keeps the meeting's record as it's said
                    if !m.invited.is_empty() {
                        let host = self.seats[m.seat].player;
                        m.talk_id = self.open_meeting_record(host, &m.invited);
                    }
                    // show the opening line right away - otherwise the first
                    // "since" tick after talk_from is 1, not 0, and the
                    // player's opening prompt (lines[0]) is skipped
                    if let Some((who, text)) = m.lines.first().cloned() {
                        self.think(who, &text, line);
                        self.note_talk(m.talk_id, Some(who), &text);
                    }
                }
                self.meeting = Some(m);
            }
            Some(start) => {
                let (since, k) = (m.frames - start, ((m.frames - start) / line) as usize);
                if since % line == 0 && k < m.lines.len() {
                    let (who, text) = m.lines[k].clone();
                    self.think(who, &text, line); // gone as the next one speaks
                    if k > 0 { self.note_talk(m.talk_id, Some(who), &text); }
                }
                if k < m.lines.len() { self.meeting = Some(m); return; }
                let msg = if m.invited.is_empty() { format!("No one made it to the house meeting at {}.", m.label) }
                    else { self.resolve_meeting(&m.invited, m.seat, m.talk_id, m.roles) };
                let at = toast_spot(&self.world, self.seats[m.seat].player);
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}", self.tick, msg),
                    at,
                    None,
                );
            }
        }
    }

    /// When the player and a villager, or two villagers, come within
    /// CHAT_RANGE of each other, one of them passes a remark - at most once
    /// a CHAT_COOLDOWN days a pair - and the pair get on a little better
    pub fn chat_in_passing(&mut self) {
        let mut people: Vec<Entity> = self.villagers.iter().copied()
            .filter(|v| !crate::game::dungeon::unavailable(&self.world, *v)).collect();
        people.push(self.player);
        // positions once for the whole pair pass - a lookup per pair was
        // thousands of hashmap hits a frame on a crowded street
        let spots: Vec<Option<(f32, f32)>> = people.iter()
            .map(|&e| self.world.get_component::<Pos>(e).map(|p| (p.x, p.y))).collect();
        let mut near = vec![];
        for (i, &a) in people.iter().enumerate() {
            for (j, &b) in people[i + 1..].iter().enumerate() {
                let (Some(pa), Some(pb)) = (spots[i], spots[i + 1 + j]) else { continue };
                if (pa.0 - pb.0).powi(2) + (pa.1 - pb.1).powi(2) < CHAT_RANGE * CHAT_RANGE { near.push((a, b)); }
            }
        }
        let tick = self.tick;
        for &(a, b) in &near {
            if self.near.contains(&(a, b)) { continue; } // already together: they've said hello
            let last = self.last_chat.iter().find(|(p, _)| *p == (a, b)).map(|(_, t)| *t);
            if last.map(|t| tick < t + CHAT_COOLDOWN).unwrap_or(false) { continue; }
            // the player is listed last, so a villager always does the talking.
            // Only some meetings get a bubble - every pair passing a remark was
            // a constant stream of chatter (the chat still counts either way)
            let n = self.last_chat.len() + (a as usize) + tick as usize;
            // a friendly animal at heel gets you greeted more: folk warm
            // to the one the village's own beasts trust
            let pet_charmed = self.pets.iter().any(|(e, _, _)| {
                self.world.get_component::<Pet>(*e)
                    .map(|p| p.owner == Some(self.player) && p.bond_strength >= pets::COMFORT_BOND)
                    .unwrap_or(false)
            });
            if n % if pet_charmed { 2 } else { 3 } == 0 {
                // two villagers hold a whole conversation - on the record
                // for the TALK panel, not floated over the street
                // (session::dialogue); the player gets a word in passing -
                // but only people who know you say hi: a friend greets,
                // an acquaintance nods, a stranger walks past quiet
                if b == self.player {
                    use crate::game::organizing::RelStage;
                    match crate::game::organizing::stage_of(&self.world, self.player, a) {
                        RelStage::Known | RelStage::Follower | RelStage::CoLeader => {
                            let name = self.speaker_name(self.player);
                            self.say_in_passing(a, &passing_greeting(n, &name), CHAT_BUBBLE_SECS);
                        }
                        RelStage::Acquaintance => { self.say_in_passing(a, passing_remark(n), CHAT_BUBBLE_SECS); }
                        RelStage::Stranger => {}
                    }
                }
                else { self.villagers_chat(a, b); }
            }
            let (pe, other) = (self.player, if b == self.player { a } else { b });
            relationships::record_interaction(&mut self.world, if b == pe { b } else { a }, relationships::Interaction {
                other, kind: relationships::InteractionKind::Conversation, intensity: CHAT_INTENSITY,
            });
            self.last_chat.retain(|(p, _)| *p != (a, b));
            self.last_chat.push(((a, b), tick));
        }
        self.near = near;
    }
}
