//! Talk and meetings: speech and thought bubbles, chat in passing, house meetings, and founding the org and its campaign.

use crate::session::*;

/// How warm a follower's tie to someone must be before they'll name them
/// as who they do it for (#1267)
const PROBE_NAME_TIE: f32 = 0.5;
use crate::game::organizing::TESTIMONY_COOLDOWN_DAYS;
/// How many of their own events the organizer picks among
const TESTIMONY_CHOICES: usize = 3;
/// The mean tie to the organizer a room needs for testimony to land full
const TESTIMONY_WARM_TIE: f32 = 0.3;
/// A told testimony opens the room: each attendee's share roll carries
/// this much more (#1333)
const TESTIMONY_OPEN: f32 = 0.10;

impl Game {
    /// The org behind a campaign as conversation `Speaker`s (E4-4): the
    /// arc's deliberations realize each line's thee and you against the
    /// pair it was actually said to
    pub(in crate::session) fn org_speakers(&self, c: Entity, tick: u64)
            -> crate::engine::det::HashMap<Entity, crate::game::conversation::Speaker> {
        let ms: Vec<Entity> = self.world.get_component::<crate::game::campaign::Campaign>(c)
            .and_then(|c| self.world.get_component::<crate::sim::community::Organization>(c.org))
            .map(|o| o.members.clone()).unwrap_or_default();
        ms.iter().map(|&m| (m, dialogue::speaker_of(&self.world, m, &self.name_of(m), tick))).collect()
    }

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
            .filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| self.cast_name(i)))
            .collect();
        // bringing them in is the craft the dropped Recruit button
        // practiced: a meeting that lands members teaches the host to
        // draw a crowd, a head at a time
        if !joined.is_empty() {
            crate::game::org_skills::practice(&mut self.world, pe,
                crate::game::org_skills::OrgSkill::Training,
                crate::game::org_skills::SEAT_PRACTICE * joined.len() as f32);
        }
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
                    // three voices at a meeting are the cause's asking
                    // done on this trouble (#1335)
                    crate::game::campaign::mark_legwork(&mut self.world, oe, tag.clone());
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
        // the called meeting IS the cadence (#1337): a cause come due
        // takes its floor on the resolve - the day's own tick stood
        // down for the gathering, so the slate opens here instead
        let floor = if let Some(c) = crate::game::campaign::campaign_of(&self.world, oe) {
            let due = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
                .map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false);
            if due && self.seats[seat].pending_agenda.is_none() {
                self.seats[seat].pending_agenda = Some((self.agenda_motions(seat, c), self.tick));
                true
            } else { false }
        } else { false };
        format!("House meeting - {} came. {}{}{}{}", invited.len(), outcome, found, founded,
            if floor { " The cause's business sits - the floor's yours." } else { "" })
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
            let tname = self.villagers.iter().position(|&v| v == t).map(|i| self.cast_name(i)).unwrap_or("the Reeve");
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
            let name = self.villagers.iter().position(|&v| v == d).map(|i| self.cast_name(i)).unwrap_or("someone");
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
                      c: Entity, tick: u64) -> Option<(String, Vec<Entity>)> {
        let out = crate::game::campaign::hold_campaign_meeting(&mut self.world, c, tick)?;
        let who: Vec<String> = out.attendees.iter()
            .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| self.cast_name(i).to_string()))
            .collect();
        // the room talks it through first: what it decided is what the
        // notes and the consent report - each voice's thee and you
        // realized against the speakers it already knows (E4-4)
        let speakers: crate::engine::det::HashMap<Entity, crate::game::conversation::Speaker> =
            out.attendees.iter()
                .map(|&v| (v, dialogue::speaker_of(&self.world, v, &self.name_of(v), tick))).collect();
        let d = crate::game::campaign_dialogue::deliberate_meeting(&mut self.world, c, &out, tick, &speakers);
        let decided = d.as_ref().map(|d| d.decided).unwrap_or(true);
        let consent = d.as_ref().map(|d| d.consent).unwrap_or(out.consent);
        let mut msg = format!("{} meet about the campaign: {}", out.attendees.len(), who.join(", "));
        if out.drifted { msg += " - run by whoever pulled hardest, nobody asked"; }
        if out.noted && decided { msg += " - the notes keep the resolves"; }
        if !decided { msg += " - nothing settled"; }
        if consent { msg += " - consent banked for the next action"; }
        if !out.healed.is_empty() {
            let healed: Vec<String> = out.healed.iter()
                .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| self.cast_name(i).to_string()))
                .collect();
            msg += &format!(" - {} heard out", healed.join(", "));
        }
        // the hat goes round while they're gathered: dues land, the
        // treasury carries a broke member while it can, a name missed
        // too long leaves the rolls (#1342)
        let dues = crate::game::campaign::dues_beat(&mut self.world, c);
        let names = |vs: &[Entity]| vs.iter()
            .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| self.cast_name(i).to_string()))
            .collect::<Vec<_>>().join(", ");
        if dues.paid > 0 || !dues.carried.is_empty() {
            let mut round = format!(" - the hat passes: {} pay", dues.paid);
            if !dues.carried.is_empty() { round += &format!(", the purse carries {}", names(&dues.carried)); }
            msg += &round;
        }
        if !dues.slipped.is_empty() {
            msg += &format!(" - {} let their dues slip", names(&dues.slipped));
        }
        if !dues.gone.is_empty() {
            msg += &format!(" - {} off the rolls", names(&dues.gone));
        }
        // the log keeps the outcome; what the room said goes on the
        // TALK panel, in the order it was said
        if let Some(d) = d {
            let mut with: Vec<Entity> = player.into_iter().collect();
            with.extend(out.attendees.iter().copied());
            self.record_campaign_talk(with, &d);
        }
        Some((msg, out.attendees))
    }

    /// The day's campaign meetings: a cause that comes due on its
    /// cadence meets on its own - the turnout seat pulls them in, or
    /// whoever's pulling, or the leader. The institution keeps its own
    /// dates like the assembly does; the seat's Meet button holds the
    /// same meeting on paths that never run the day's tick (#1159).
    pub fn campaign_meeting_tick(&mut self, tick: u64) {
        let campaigns = self.world.query::<crate::game::campaign::Campaign>();
        for c in campaigns {
            let seat = self.seats.iter()
                .find(|s| crate::game::campaign::campaign_of(&self.world, s.org) == Some(c));
            // an agenda slate already open on the box holds the date
            // itself - the floor's deciding, the day doesn't pre-empt
            // the vote with a meeting of its own (#1157)
            if seat.map(|s| s.pending_agenda.is_some()).unwrap_or(false) { continue; }
            // a called meeting under way for this org holds the cadence
            // itself (#1337): the gathering's the meeting - the tick
            // doesn't run a shadow one beside it. Checked against the
            // live gathering, not the plan's `for_day`: calling it
            // consumes that date before the gathering resolves, so a
            // stale-date check would miss it and double up the meeting
            let org = self.world.get_component::<crate::game::campaign::Campaign>(c).map(|cc| cc.org);
            let gathering = self.meeting.as_ref()
                .map(|m| Some(self.seats[m.seat].org) == org).unwrap_or(false);
            if gathering { continue; }
            let player = seat.map(|s| s.player);
            if let Some((msg, attendees)) = self.hold_and_note_campaign_meeting(player, c, tick) {
                // the cause met on its own day, without you (E6-2):
                // "we looked for you and you never came" can surface
                if let Some(pe) = player {
                    for a in &attendees {
                        crate::game::episodes::file(&mut self.world, *a, pe,
                            crate::game::episodes::MemKind::MissedMeeting, tick, &["meeting"]);
                    }
                }
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

    /// The tile a venue gathers on (#1337): a lent house's door, the
    /// church steps, the tavern - the square when it's a spot the map
    /// can't reach any more, and when there's no venue at all
    pub(in crate::session) fn venue_tile(&self, v: crate::game::organizing::Venue) -> (i32, i32) {
        use crate::game::organizing::Venue;
        let at = |b: Option<Entity>| b.and_then(|e| self.world.get_component::<Pos>(e))
            .map(|p| (p.x.round() as i32, p.y.round() as i32));
        match v {
            Venue::House(_, b) => at(Some(b)).unwrap_or(self.village),
            Venue::Square => self.village,
            Venue::Church => at(self.church).unwrap_or(self.village),
            Venue::Tavern => at(self.tavern).unwrap_or(self.village),
        }
    }

    /// How the word names the place (#1337): "Bram's house" when it's
    /// lent, else the spot - the square, the church steps, the tavern
    pub(in crate::session) fn venue_label(&self, v: crate::game::organizing::Venue) -> String {
        use crate::game::organizing::Venue;
        match v {
            Venue::House(h, _) => self.villagers.iter().position(|&v| v == h)
                .map(|i| format!("{}'s house", self.cast_name(i)))
                .unwrap_or_else(|| "a friend's house".into()),
            Venue::Square => "the square".into(),
            Venue::Church => "the church steps".into(),
            Venue::Tavern => "the tavern".into(),
        }
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
        // no venue named: the cross is the standing ground for a meeting
        // the word alone convened
        let venue = plan.venue.map(|v| self.venue_tile(v)).unwrap_or(self.village);
        let label = plan.venue.map(|v| self.venue_label(v)).unwrap_or_else(|| "the square".into());
        // the invited: those who said they'd come, those who said
        // they'd bring friends (and the friends they bring), every
        // promise-keeper - and a plus-one for each of the committed,
        // a yes is a door held open (#1075) - then the turnout roll,
        // roughly a coin's toss for the merely invited
        let mut pool: Vec<(Entity, org::Invite)> = plan.holders().iter()
            .map(|&h| (h, org::Invite::Yes)).collect();
        for &s in &plan.spreaders {
            // a bought spreader shows, but their friends weren't bought (#1225)
            if plan.bought.contains(&s) { continue; }
            for f in org::friends_of(&self.world, s, 2) { pool.push((f, org::Invite::Pulled)); }
        }
        for &h in &plan.holders() {
            if plan.bought.contains(&h) { continue; }
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
        let names: Vec<&str> = invited.iter().filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| self.cast_name(i))).collect();
        let mut msg = format!("Word goes out - {} gather at {}.", names.join(", "), label);
        if stayed > 0 {
            msg = format!("{} - {} of those asked stayed home.", msg.trim_end_matches('.'), stayed);
        }
        // an unanswered question-question leaves the matter (#1267)
        self.seats[seat].frame_offer = None;
        self.seats[seat].testimony_offer = None;
        self.meeting = Some(Meeting { label, venue, born: self.tick, invited, spots, paths, frames: 0, talk_from: None, lines: vec![], talk_id: 0, seat, roles, role_people, probe: plan.probe, testimony: plan.testimony, bought: plan.bought });
        msg
    }

    /// What's said at a meeting: the player opens, each villager there says
    /// what's on their mind - now and then a secret slips out, and every
    /// parallel complaint already voiced makes the next speaker likelier
    /// to share their own. Three voices in a family surface an issue.
    pub fn meeting_lines(&mut self, came: &[Entity], seat: usize, roles: (bool, bool, bool, bool, bool), probe: bool, testimony: Option<&str>) -> Vec<(Entity, String)> {
        // the seat that called it hosts: its player asks, and hears what comes out
        let host_e = self.seats[seat].player;
        if came.is_empty() { return vec![]; }
        // the probe replaces the host's question (#1267)
        let ask = if probe { crate::game::dialogue_trees::MEET_PROBE.text } else { MEETING_PROMPTS[0] };
        // each line carries whether run_meeting already rendered it -
        // the floor's replies come out realized (their own slots, their
        // own marks), and a second pass through pair_said would sign
        // them twice
        let mut lines = vec![(host_e, ask.to_string(), false)];
        // testimony opens ahead of the ask: Self, Us, Now (#1248)
        if let Some(ev) = testimony {
            let arc = self.testimony_arc(ev, came);
            lines.splice(0..0, arc.into_iter().map(|l| (host_e, l, false)));
            self.testify(seat, came);
        }
        self.voiced.clear();
        // what each attendee says when no secret comes out of them: the
        // meeting's talk from conversation::run_meeting - answers to the
        // question, agreement and pushback threaded on what's been said -
        // seeded off the meeting, not the game's rng
        let (mut said, mut host_q, rite, send_off) = {
            use crate::game::conversation::{run_meeting, MeetingLine, Rng as TalkRng, Speaker};
            use crate::game::dialogue_trees::{meeting_pool, MEET_PROBE, MEET_PROBE_LINES, MEET_PROMPT, MEET_RITES};
            let speakers: Vec<Speaker> = came.iter()
                .map(|&v| dialogue::speaker_of(&self.world, v, &self.name_of(v), self.tick)).collect();
            let pools: Vec<Vec<MeetingLine>> = speakers.iter().map(|s| {
                let mut p = meeting_pool(s);
                if probe { p.extend_from_slice(MEET_PROBE_LINES); }
                p
            }).collect();
            let prompt = if probe { &MEET_PROBE } else { &MEET_PROMPT };
            let pool_refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
            let refs: Vec<&Speaker> = speakers.iter().collect();
            let host = self.world.get_component::<PlayerCharacter>(host_e).map(|p| p.name.clone()).unwrap_or_default();
            // how each stands to the organizer - kin and the near speak
            // with the house's weight where no craft owns the matter (#1308)
            let ties: Vec<crate::game::conversation::Relation> = came.iter().map(|&v| {
                let kind = relationships::find_relationship(&self.world, v, host_e)
                    .and_then(|r| self.world.get_component::<relationships::Relationship>(r))
                    .map(|r| r.kind);
                dialogue::relation_with_wanderer(kind)
            }).collect();
            let seed = self.tick.wrapping_mul(0x51_7CC1) ^ came.iter().fold(0u64, |h, &v| h.rotate_left(7) ^ v as u64);
            let record = run_meeting(&Speaker::new(&host), &refs, &ties, prompt, &pool_refs, &MEET_RITES, &mut TalkRng::new(seed));
            // the record frames the floor: statement 0 is the rite, the
            // last is the send-off - they air first and last, whoever gave
            // them (the host stands in when the organizer did)
            let close_i = record.len() - 1;
            let who = |speaker: usize| if speaker == usize::MAX { host_e } else { came[speaker] };
            let rite = (who(record[0].speaker), record[0].text.clone());
            let send_off = (who(record[close_i].speaker), record[close_i].text.clone());
            // per attendee, in the order they said them: (statement index,
            // the statement it answers, the words)
            let mut q: Vec<Vec<(usize, Option<usize>, String)>> = vec![vec![]; came.len()];
            // the organizer's own calls - the pull-backs when the room
            // drifted - air as their turns come (#1186); the question
            // itself is covered by the host's prompt
            let mut host_q: Vec<(usize, String)> = vec![];
            for (k, st) in record.into_iter().enumerate() {
                // framing and the host's own question aren't floor
                if k == 0 || k == 1 || k == close_i { continue; }
                if st.speaker == usize::MAX { host_q.push((k, st.text)); }
                else if let Some(d) = q.get_mut(st.speaker) { d.push((k, st.replying_to, st.text)); }
            }
            (q, host_q, rite, send_off)
        };
        // the meeting runs in rounds, not in run_meeting's order, so a
        // reply is only said once what it answers has been (statements 0
        // and 1 - the rite and the host's question - air before any floor)
        let mut aired: crate::engine::det::HashSet<usize> = [0, 1].into_iter().collect();
        // the furthest floor statement heard so far - pull-backs air
        // once the room's reached where they happened
        let mut reached = 1usize;
        for round in 0..2 {
            for (ci, &v) in came.iter().enumerate() {
                let Some(i) = self.villagers.iter().position(|x| *x == v) else { continue };
                // the probe's first round: each answers as deep as they trust
                if probe && round == 0 {
                    if let Some(line) = self.probe_answer(host_e, v, came) {
                        lines.push((v, line, false));
                        continue;
                    }
                }
                let mine: crate::engine::det::HashSet<desires::IssueTag> =
                    desires::unrevealed(&self.world, host_e, v, self.tick).iter()
                        .map(|r| desires::issue_tag(&self.world, v, r)).collect();
                let parallel = self.voiced.iter().filter(|(_, t)| mine.contains(t)).count();
                let mut chance = desires::REVEAL_BASE + desires::REVEAL_PARALLEL * parallel as f32;
                // an opener or a song loosens tongues: each promised role
                // that came off warms the room to share a little more
                if roles.2 { chance += 0.05; }
                if roles.3 { chance += 0.05; }
                // and a told testimony opens the room (#1333): the story
                // they just carried home makes the next voice likelier
                if testimony.is_some() { chance += TESTIMONY_OPEN; }
                // the fold's smell hangs on a shepherd: a touch slower to share
                if desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly) { chance *= 0.9; }
                let roll = (self.rng.next() % 1000) as f32 / 1000.0;
                let mut shared = false;
                if roll < chance {
                    let pick = self.rng.next();
                    if let Some(r) = desires::next_reveal(&self.world, host_e, v, self.tick, pick) {
                        let (_, said) = desires::reveal_speech(self.cast_name(i), &r);
                        let tag = desires::issue_tag(&self.world, v, &r);
                        self.voiced.push((v, tag.clone()));
                        // a complaint aired is company (#1442): the
                        // speaker's heard, and every hearer whose own
                        // trouble echoes it stands a little less alone
                        self.parallel_push(v, &tag, came);
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
                        lines.push((v, said, false));
                        shared = true;
                    }
                }
                if !shared {
                    let ready = said[ci].iter().position(|(_, to, _)| to.map_or(true, |t| aired.contains(&t)));
                    let (line, done) = match ready {
                        Some(j) => { let (k, _, text) = said[ci].remove(j);
                            aired.insert(k); reached = reached.max(k); (text, true) }
                        None => (meeting_remark(&self.world, v, self.cast_name(i), round), false),
                    };
                    lines.push((v, line, done));
                }
            }
            // when the room drifted, the host calls it back to the
            // matter - airing near where it happened in the record
            while let Some(j) = host_q.iter().position(|(k, _)| *k <= reached) {
                let (k, text) = host_q.remove(j);
                aired.insert(k);
                lines.push((host_e, text, true));
            }
            lines.push((host_e, MEETING_PROMPTS[round + 1].to_string(), false));
        }
        // a pull-back past the furthest aired statement still lands
        // before the meeting's out - the room drifted on its way
        for (_, text) in host_q.drain(..) {
            lines.push((host_e, text, true));
        }
        // the rite opens it, ahead of the host's question; the send-off
        // is the last word of the night (#1185)
        lines.insert(0, (rite.0, rite.1, true));
        lines.push((send_off.0, send_off.1, true));
        // the question stays with them: each who heard it goes home with
        // a want they didn't carry, ripening in its own time (#1267)
        if probe {
            for &v in came {
                let seed = self.rng.next();
                desires::plant_want(&mut self.world, v, host_e, self.tick, seed);
            }
        }
        lines.into_iter()
            .map(|(e, t, done)| (e, if done { t } else { self.pair_said(e, self.seats[seat].player, &t) }))
            .collect()
    }

    /// The organizer's history worth testifying to (#1248): the latest
    /// hard news the log holds - a hanging, a writ, a fire, a death -
    /// told as it was logged, at most TESTIMONY_CHOICES of them
    pub fn testimony_events(&self) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        for i in (0..self.log.len()).rev() {
            let Some(ev) = self.log.ev(i) else { continue };
            if ev.valence != Valence::Negative || ev.sev == Sev::Minor
                || matches!(ev.thread, Thread::Weather | Thread::Player | Thread::Org | Thread::Other) { continue; }
            let line = &self.log[i];
            let text = match line.split_once(": ") {
                Some((t, rest)) if t.starts_with('t') && t[1..].chars().all(|c| c.is_ascii_digit()) => rest,
                _ => line.as_str(),
            };
            if !out.iter().any(|o| o == text) { out.push(text.to_string()); }
            if out.len() >= TESTIMONY_CHOICES { break; }
        }
        out
    }

    /// Testimony's on offer: off cooldown, and there's history to tell
    pub fn testimony_ready(&self, seat: usize) -> bool {
        self.seats[seat].last_testimony
            .map(|t| self.tick.saturating_sub(t) >= TESTIMONY_COOLDOWN_DAYS).unwrap_or(true)
            && !self.testimony_events().is_empty()
    }

    /// The three beats of testimony: Self (the event, told first-person),
    /// Us (what the room shares), Now (tonight's choice)
    fn testimony_arc(&self, ev: &str, came: &[Entity]) -> Vec<String> {
        let mut counts: Vec<(String, usize)> = vec![];
        for &v in came {
            for p in desires::problems_of(&self.world, v, self.tick) {
                let label = desires::problem_label(&p);
                match counts.iter_mut().find(|(l, _)| *l == label) {
                    Some(c) => c.1 += 1,
                    None => counts.push((label, 1)),
                }
            }
        }
        let shared = counts.iter().fold(None::<&(String, usize)>, |best, c| match best {
            Some(b) if b.1 >= c.1 => Some(b),
            _ => Some(c),
        });
        let us = match shared {
            Some((label, _)) => format!("And it's not mine alone - every house in this room knows {}.", label),
            None => "And it's not mine alone - every house in this room has felt the lord's hand.".into(),
        };
        vec![
            format!("I'll tell {{2p:obj}} why I stand here. I saw it: {} I couldn't keep quiet after that.", ev.trim_end_matches('.').to_string() + "."),
            us,
            "So tonight's the choosing: stand together on it, or each go home and bear it alone.".into(),
        ]
    }

    /// What testimony does to the room: never a loss, but it wants a
    /// warmed room - a warm one carries two moments of warmth with the
    /// organizer home (alignment deepens), a cold one hears a stranger's
    /// story and carries one. Starts the cooldown
    fn testify(&mut self, seat: usize, came: &[Entity]) {
        use crate::sim::psychology::{record_episode, Episode, EpisodeKind};
        let host = self.seats[seat].player;
        let room: Vec<Entity> = came.iter().copied()
            .filter(|&v| v != host && !villager::is_dead(&self.world, v)).collect();
        if room.is_empty() { return; }
        let warmth = room.iter().map(|&v| relationships::find_relationship(&self.world, v, host)
            .and_then(|r| self.world.get_component::<relationships::Relationship>(r))
            .map(|r| r.strength).unwrap_or(0.0)).sum::<f32>() / room.len() as f32;
        let warm = warmth >= TESTIMONY_WARM_TIE;
        for &v in &room {
            for _ in 0..if warm { 2 } else { 1 } {
                // the telling rides the journal as its own kind: warmth,
                // but a hearer is marked by it for good (#1333)
                record_episode(&mut self.world, v, Episode { with: host, kind: EpisodeKind::Testimony });
            }
        }
        self.seats[seat].last_testimony = Some(self.tick);
        let line = if warm {
            format!("t{}: the room hears your testimony and leans in - your story is theirs now too.", self.tick)
        } else {
            format!("t{}: you tell it to a cold room - they hear a stranger's story, but they hear it.", self.tick)
        };
        self.log.push_ev(EvTag::new(Thread::Org, if warm { Valence::Positive } else { Valence::Neutral }, Sev::Minor), line);
    }

    /// A complaint aired in the room is company (#1442): the speaker's
    /// heard, and every hearer whose own live trouble echoes the tag
    /// stands a little less alone - it lands on the act model
    fn parallel_push(&mut self, speaker: Entity, tag: &desires::IssueTag, came: &[Entity]) {
        crate::game::act_model::parallel_heard(&mut self.world, speaker, true);
        for &u in came {
            if u == speaker { continue; }
            let shares = desires::live_pool(&self.world, u, self.tick).iter()
                .any(|r| desires::issue_tag(&self.world, u, r) == *tag);
            if shares { crate::game::act_model::parallel_heard(&mut self.world, u, false); }
        }
    }

    /// One attendee's answer to the probe (#1267), as deep as they trust
    /// the asker: a follower names who they do it for - the room hears
    /// the bond, and so does the lord's ear if it's in the room - a
    /// guarded one gives up a want and no more. None: nothing to give,
    /// they take their turn like any other
    fn probe_answer(&mut self, asker: Entity, v: Entity, came: &[Entity]) -> Option<String> {
        use crate::game::organizing::{friends_of, stage_of, RelStage};
        let name = self.name_of(v);
        if stage_of(&self.world, asker, v) >= RelStage::Follower {
            let loved = friends_of(&self.world, v, 3).into_iter().find(|&f| f != asker
                && relationships::find_relationship(&self.world, v, f)
                    .and_then(|r| self.world.get_component::<relationships::Relationship>(r))
                    .map(|r| r.strength >= PROBE_NAME_TIE).unwrap_or(false));
            if let Some(l) = loved {
                use crate::game::dialogue_trees::PROBE_NAMES;
                let lname = self.name_of(l);
                let said = PROBE_NAMES[(self.rng.next() % PROBE_NAMES.len() as u64) as usize].replace("{name}", &lname);
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                    asker,
                    format!("t{}: {} names {} - who they're doing it for. The room knows it now.", self.tick, name, lname),
                );
                // the cost of a true answer: the lord's man takes the name home
                if self.plant.map(|p| came.contains(&p)).unwrap_or(false) {
                    desires::add_grievance(&mut self.world, l, desires::Problem::WritBorne, self.tick + 30);
                    self.log_entity_ev(
                        EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                        asker,
                        format!("t{}: the lord's ear was in the room - a writ finds {}.", self.tick, lname),
                    );
                }
                return Some(said);
            }
        }
        // one who's heard the organizer's testimony answers deeper:
        // the whole of what's pressing, not a want and no more (#1333)
        let heard = crate::sim::psychology::heard_testimony(&self.world, v, asker);
        let pool: Vec<desires::Reveal> = if heard {
            desires::unrevealed(&self.world, asker, v, self.tick)
        } else {
            desires::unrevealed(&self.world, asker, v, self.tick).into_iter()
                .filter(|r| matches!(r, desires::Reveal::Want(_))).collect()
        };
        if pool.is_empty() { return None; }
        let at = (self.rng.next() % pool.len() as u64) as usize;
        // a hearer's deeper answer goes to what presses first
        let r = if heard {
            pool.iter().find(|r| matches!(r, desires::Reveal::Problem(_)))
                .copied().unwrap_or(pool[at])
        } else { pool[at] };
        let (_, said) = desires::reveal_speech(&name, &r);
        let tag = desires::issue_tag(&self.world, v, &r);
        self.voiced.push((v, tag.clone()));
        self.parallel_push(v, &tag, came);
        if let Some(issue) = desires::hear(&mut self.world, asker, v, r, self.tick) {
            self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
            self.log_entity_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                asker,
                format!("t{}: Organizer discovers issue {}", self.tick, issue),
            );
        }
        Some(said)
    }

    /// The scheduled parts of a meeting, spliced into its lines and
    /// felt in the room: the opener welcomes first, the bread goes
    /// round early (and fills a food-want), the song warms the middle
    /// (and fills a music-want), the notes keep the word and the
    /// closer sends them off. A part only happens if the one who
    /// promised it made it there - the caller already kept `roles`
    /// honest about that (#1076)
    pub(in crate::session) fn meeting_beats(&mut self, m: &mut Meeting) {
        let [snacks, notes, opener, song, closer] = m.role_people;
        // the meeting's work teaches those who watch it (#1345): a kept
        // part turns the holder's mark green and lays a yellow on every
        // witness - the marks are the ladder's explainers
        use crate::game::leadership::{mark_observed, mark_performed, Job};
        let kept = [m.roles.0, m.roles.1, m.roles.2, m.roles.3, m.roles.4];
        for (i, job) in [Job::Snacks, Job::Notes, Job::Opener, Job::Song, Job::Closer].iter().enumerate() {
            if let (true, Some(holder)) = (kept[i], m.role_people[i]) {
                mark_performed(&mut self.world, holder, *job);
                // the role's efficacy credit lands only once they've
                // actually turned up to keep it (#1442)
                crate::game::act_model::ask_kept(&mut self.world, holder);
                for &a in &m.invited {
                    if a != holder { mark_observed(&mut self.world, a, *job); }
                }
            }
        }
        // the send-off is the last word of the night - kept promises land
        // just ahead of it, never after the benediction (#1185)
        if let (true, Some(n)) = (m.roles.1, notes) {
            m.lines.insert(m.lines.len().saturating_sub(1), (n, "Say it plain - I'll keep the word of it.".into()));
        }
        if let (true, Some(c)) = (m.roles.4, closer) {
            let close = self.pair_said(c, self.player, "Same time again, then - and each of {2p:obj} bring one more.");
            m.lines.insert(m.lines.len().saturating_sub(1), (c, close));
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
            let bread = self.pair_said(s, self.player, "Bread's on the table - help {2p:self}.");
            m.lines.insert(1.min(m.lines.len()), (s, bread));
        }
        if let (true, Some(o)) = (m.roles.2, opener) {
            let open = self.pair_said(o, self.player, "Friends - {2p:subj} came, and that's the start of it.");
            m.lines.insert(0, (o, open));
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
                    m.lines = self.meeting_lines(&m.invited, m.seat, m.credit_roles(), m.probe, m.testimony.as_deref());
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
                    else { self.resolve_meeting(&m.invited, m.seat, m.talk_id, m.credit_roles()) };
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
                            let said = self.pair_said(a, self.player, &passing_greeting(n, &name));
                            self.say_in_passing(a, &said, CHAT_BUBBLE_SECS);
                        }
                        RelStage::Acquaintance => {
                            let said = self.pair_said(a, self.player, passing_remark(n));
                            self.say_in_passing(a, &said, CHAT_BUBBLE_SECS);
                        }
                        RelStage::Stranger => {}
                    }
                }
                else { self.villagers_chat(a, b); }
            }
            let (pe, other) = (self.player, if b == self.player { a } else { b });
            // a greeting for the player stays what it was; two villagers
            // brushing past only nod - the chat's own typed delta does the
            // real warming when words were actually exchanged (#1230)
            relationships::record_interaction(&mut self.world, if b == pe { b } else { a }, relationships::Interaction {
                other, kind: relationships::InteractionKind::Conversation,
                intensity: if b == pe { CHAT_INTENSITY } else { PASS_INTENSITY },
            });
            self.last_chat.retain(|(p, _)| *p != (a, b));
            self.last_chat.push(((a, b), tick));
        }
        self.near = near;
    }
}
