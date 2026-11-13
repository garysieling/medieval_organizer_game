//! Seats: who's connected (players and agents), the commands they send, and the single-seat wrappers solo play uses.

use crate::session::*;

/// Walking over to someone to talk to or recruit them
pub struct Approach {
    pub who: usize,              // villager index
    pub action: usize,           // the action button to take on arrival (Talk or Recruit)
    pub path: Vec<(i32, i32)>,   // tiles still to walk, their tile last
    pub stuck: u32,              // frames without headway
}

/// Walking over to a clicked building's shelf to buy one of what's on
/// offer - each Buy click is one unit, so another click walks again
pub struct BuyWalk {
    pub building: usize,         // index into building_entities
    pub path: Vec<(i32, i32)>,   // tiles still to walk, the shop's tile last
    pub stuck: u32,              // frames without headway
}

/// What a seat tells the sim: the same actions the window's keys and clicks
/// map to, so a local loop, a remote client and a test all steer the same way.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Command {
    /// One of the five action buttons (Talk, Recruit, Invest, Organize, Build)
    Act(usize),
    /// A contextual action (Work, Pair up, Errand)
    CtxAct(usize),
    /// Held-direction walk, one frame's step
    Walk(f32, f32),
    /// Pick a villager by index (the 1-8 keys)
    Select(usize),
    /// Click the map through this camera (select villager or building)
    ClickMap { mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32 },
    TogglePause,
    /// Drop any walk-over-to-talk in progress
    CancelApproach,
    /// Answer the villager mid-Talk with reply `n`, or None to take your leave
    TalkPick(Option<usize>),
    /// Answer the founding call a house meeting left open on the
    /// conversation box: pick `n` of the framings, or the last option
    /// (let it rest)
    Decide(usize),
    /// Let the pick go: the ring, the walk-over and the talk under way
    /// (the window's double-tap of Talk)
    Unselect,
}

/// A contextual action offered under the main buttons: what the player can
/// do depends on where they stand and who's picked.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CtxAct {
    /// Work the player's own craft (at their bench if they have one)
    Work,
    /// Pair the picked junior with a master of their trade
    PairUp,
    /// Send the picked child on a paid errand
    Errand,
    /// Put a word between two hearts the organizer knows are lonely
    Matchmake,
    /// Hold the campaign's meeting, on its cadence
    Meet,
    /// Take the campaign to the streets: a demonstration at the player
    March,
    /// Give the picked member a campaign seat (turnout, notes, chair)
    Delegate,
    /// Ballot the picked member's trade: nine of ten, or a public failure
    Strike,
    /// Put the campaign's demands to the collector, in public
    Townhall,
    /// Keep pressing for the prisoner's release
    Petition,
    /// Sound the campaign's target out: where they stand on the issue,
    /// and the stake that puts them there - rapport earns the hearing
    Interview,
    /// The debrief: present members ballot the next step and the vote
    /// sets the campaign's directive
    Debrief,
    /// Join the gathering at hand: a service, a feast, a birthday -
    /// the player's presence counts for what the day means
    Attend,
    /// Take a boat at the bank: the water carries you from here
    Board,
    /// Put in at the bank: the water lets you off here
    Disembark,
    /// Drop a line off the boat: the river feeds you where it runs
    Fish,
    /// Mend the meeting hall a storm took - the org's treasury pays
    MendHall,
    /// Set how often the cause meets: the organizer's recommendation
    Cadence,
    /// Call the accountability assembly: the agenda goes up and the
    /// leader's yes and no go on record
    Assembly,
    /// Call the house meeting the organizer has earned - a venue promised
    /// and folk who've said they'll come
    Gather,
    /// Take on the odd job the picked villager is asking for
    TakeJob,
    /// Press wax with the picked villager: ten gold of aid, sworn
    Pact,
    /// Stroke the stray cat who's keeping you company
    PetCat,
    /// Call a snowball fight: everyone free comes out to play (winter)
    Snowballs,
    /// Pass the hat, buy pamphlets off the print shop and post them
    /// around the streets for the cause
    Flyers,
    /// Win the picked villager to the cause - it takes real trust first
    Recruit,
    /// Pick a cap off the patch you're standing by - the wood feeds,
    /// and teaches its lessons the hard way
    PickCaps,
    /// Tell the picked villager what the gossip added up to - the
    /// mystery's asker, once it's whole
    ShareFindings,
    /// Throw water on the flames - offered standing by a fire
    Douse,
    /// Give the picked villager something from your satchel - the meal
    /// for the hungry, the thing they're after, or the best of the bag
    Give,
    /// Buy from a shop you're standing by - one of what it makes, at
    /// the street's price, out of your walking purse
    Buy,
    /// Stroke the animal you're standing by - the kind hand, again and
    /// again, until it leaves the fold and follows
    Tame,
    /// Raise the schoolhouse, or send for a teacher to keep it - asked
    /// only near where it'd stand, while the treasury can pay
    School,
    /// Charter the fire brigade: a hall to gather in, the chest's gold
    /// for buckets, and a crew sworn to answer every bell
    Brigade,
    /// Take the cache the woods' bands left at the edge of the trees -
    /// offered standing by it before it rots or is found
    TakeCache,
}

/// One player's hold on the game: which character they steer, what they've
/// picked, the walk-over-to-talk under way, their contextual buttons. A solo
/// game has exactly one seat; a server keeps one per claimed connection.
pub struct Seat {
    /// The seat's character entity (a wanderer for people, a villager for agents)
    pub player: Entity,
    /// The seat's organization entity
    pub org: Entity,
    /// They picked the selected villager themselves and haven't walked off
    /// since: Talk and Recruit go to them, however far
    pub picked: bool,
    /// The picked one was within a shout when picked, or has come within
    /// one since: a pick that drifts past a shout lets go. A pick made
    /// from further off was for a look, and keeps until they come near
    /// or you pick another (#944)
    pub pick_near: bool,
    /// A walk-over-to-act under way
    pub approach: Option<Approach>,
    /// A walk-over-to-buy under way: the clicked building, its own field
    /// since Approach's `who` is a villager index
    pub buy_walk: Option<BuyWalk>,
    /// Index into villagers[] of the villager currently selected
    pub selected_target: usize,
    /// A building the seat clicked, or None while a villager is selected
    pub selected_building: Option<usize>,
    /// Contextual actions that depend on where the player stands and who's picked
    pub ctx_actions: Vec<CtxAct>,
    /// An API-driven seat steers a villager, not a wanderer
    pub agent: bool,
    /// A Talk under way, waiting on the player's reply (session::dialogue)
    pub talk: Option<dialogue::PlayerTalk>,
    /// The founding call this seat's house meeting left open on the
    /// conversation box, waiting on their pick (Command::Decide). Each
    /// organizer holds their own - another seat's meeting can't erase it
    pub pending_founding: Option<PendingFounding>,
    /// The day-question a Gather press leaves open on the conversation
    /// box, waiting on their pick (Command::Decide). The tick it was
    /// asked, so an unanswered one can lapse like the founding's
    pub meet_offer: Option<u64>,
    /// The odd job this seat is running for a villager, if any
    /// (session::quests)
    pub quest: Option<quests::PlayerQuest>,
    /// The village mystery this seat is piecing together, if any
    /// (session::mystery)
    pub mystery: Option<mystery::SeatMystery>,
    /// The day another mystery can start, once one's been told
    pub mystery_wait: u64,
    /// The last mystery dealt, so the same one doesn't come round twice
    pub mystery_last: usize,
    /// Whether they're stood reading the market board: the notices panel
    /// opens on this seat's screen alone (#819)
    pub notice_panel: bool,
    /// The boat this seat's player is on (an index into boats), or None
    /// on the bank: aboard, the water carries them - it's per-seat or one
    /// boarder floods everyone's shoes (#822)
    pub aboard: Option<usize>,
    /// Where the sim last asked the camera to look - (x, y, the frame it
    /// was asked on), so a client can tell a new ask from a repeated one
    pub focus: Option<(f32, f32, u32)>,
    /// Where the character stood last frame, which way they face (0 up,
    /// 1 right, 2 down, 3 left on screen) and whether they moved: drives
    /// the walk cycle
    pub last_pos: Option<(f32, f32)>,
    pub facing: u8,
    pub walking: bool,
    /// This seat's pair of hands has stopped and the work's been passed
    /// on (or died with them) - the succession runs once, per seat (#962)
    pub player_passed: bool,
}

impl Default for Seat {
    fn default() -> Seat {
        Seat { player: 0, org: 0, picked: false, approach: None, buy_walk: None, selected_target: 0,
               selected_building: None, ctx_actions: vec![], agent: false, talk: None,
               pick_near: false,
               pending_founding: None, meet_offer: None, quest: None, mystery: None, mystery_wait: 0,
               mystery_last: usize::MAX, notice_panel: false, aboard: None,
               focus: None, last_pos: None, facing: 2, walking: false,
               player_passed: false }
    }
}

impl Game {
    /// One input from this seat: what a key press or click means to the sim.
    pub fn command_at(&mut self, seat: usize, cmd: Command) {
        // the day's own record: the autosave replays it to the same village
        self.journal.record(self.frame_n as u64, crate::session::journal::Input::Command {
            seat, cmd: cmd.clone() });
        match cmd {
            Command::Act(i) => self.act_at(seat, i),
            Command::CtxAct(i) => self.ctx_act_at(seat, i),
            Command::Walk(dx, dy) => { self.seats[seat].approach = None; self.seats[seat].buy_walk = None; self.seats[seat].picked = false; self.seats[seat].pick_near = false; self.walk_at(seat, dx, dy); }
            Command::Select(i) => {
                // an index off the list is refused, not stored - a client
                // that sends one can't crash the next scene
                if i < self.villagers.len() {
                    self.seats[seat].selected_target = i;
                    self.seats[seat].picked = true;
                    self.seats[seat].pick_near = self.dist_to_at(seat, i) <= SHOUT_DIST;
                }
            }
            Command::ClickMap { mx, my, cam_x, cam_y, zoom } => self.click_map_at(seat, mx, my, cam_x, cam_y, zoom),
            Command::TogglePause => self.paused = !self.paused,
            Command::CancelApproach => { self.seats[seat].approach = None; self.seats[seat].buy_walk = None; }
            Command::TalkPick(n) => self.talk_pick_at(seat, n),
            Command::Decide(i) => self.decide_at(seat, i),
            Command::Unselect => self.unselect_at(seat),
        }
    }

    /// A new wanderer joins at the market: the same spawn as the first
    /// player, with a class, a trade, a purse and an inner life.
    pub fn add_seat(&mut self) -> usize {
        let player = self.world.create_entity();
        let gold = livelihood::seed_gold(SocialClass::Working, false, self.rng.next() as f32 / u64::MAX as f32);
        self.world.add_component(player, PlayerCharacter {
            name: "A Wanderer".into(), resources: gold });
        let mp = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x + 2.0, p.y + 1.0)).unwrap_or((10.0, 10.0));
        self.world.add_component(player, Pos { x: mp.0, y: mp.1 });
        self.world.add_component(player, vocation::craft(roll_vocation(&mut self.rng), 0.2));
        self.world.add_component(player, Satchel::default());
        self.world.add_component(player, Purse { gold });
        self.world.add_component(player, crate::sim::psychology::AttachmentHistory {
            anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
            self.world.add_component(player, crate::sim::psychology::personality_from_seed(0xBEEF));
        self.world.add_component(player, Organizer::default());
        let org = self.world.create_entity();
        self.world.add_component(org, Organization {
            name: "The Free Folk".into(), members: vec![], leader: player,
            influence_pool: 0.0, resources: 0.0, charter: "Newcomers".into() , successors: vec![] });
        // every cause starts with the muscle to spend - a quiet one fades
        self.world.add_component(org, crate::sim::organized_power::OrganizedPower::new());
        self.journal.record(self.frame_n as u64, crate::session::journal::Input::AddSeat);
        self.seats.push(Seat { player, org, ..Seat::default() });
        let seat = self.seats.len() - 1;
        self.intro_quest_for(seat);
        seat
    }

    /// An API agent takes the reins of a villager: the seat's character is
    /// the villager entity itself; no PlayerCharacter, no org of their own.
    pub fn add_agent_seat(&mut self, villager: Entity) -> usize {
        let org = self.world.create_entity();
        self.world.add_component(org, Organization {
            name: "-".into(), members: vec![], leader: villager,
            influence_pool: 0.0, resources: 0.0, charter: "-".into() , successors: vec![] });
        self.world.add_component(org, crate::sim::organized_power::OrganizedPower::new());
        // their own ledger of what they've heard: it lets a villager agent
        // gather the grievances folk tell them, as a wanderer does
        self.world.add_component(villager, Organizer::default());
        self.journal.record(self.frame_n as u64, crate::session::journal::Input::AddAgentSeat { villager });
        self.seats.push(Seat { player: villager, org, agent: true, ..Seat::default() });
        let seat = self.seats.len() - 1;
        self.intro_quest_for(seat);
        seat
    }

    /// The first living villager nobody drives yet. A posting isn't up
    /// for the taking - a bot can claim a free hand, not the collector,
    /// the herb-wife, the knight or the priest (postings ride either
    /// TownRole or a PMC Craft); and a child is nobody's worker. The
    /// town's duties stay the town's.
    pub fn free_villager(&self) -> Option<usize> {
        // an agent needs a villager who can answer - the grave and the
        // lock-up keep their counsel
        self.villagers.iter().position(|v| !crate::game::dungeon::unavailable(&self.world, *v)
            && !self.seats.iter().any(|s| s.player == *v)
            && self.world.get_component::<TownRole>(*v).is_none()
            && self.world.get_component::<Kid>(*v).is_none()
            && !self.world.get_component::<Craft>(*v)
                .map(|c| vocation::is_pmc(c.vocation)).unwrap_or(false))
    }

    /// Run a villager agent's contextual action through the same availability
    /// and blocker checks used by the player context menu.
    fn agent_context_action(
        &mut self,
        seat: usize,
        action: AgentAction,
        target: Option<Entity>,
    ) -> Result<(), String> {
        let who = self.seats.get(seat).map(|s| s.player)
            .ok_or_else(|| "no such seat".to_string())?;
        if target.is_none() && matches!(action,
            AgentAction::TakeJob | AgentAction::Recruit | AgentAction::Matchmake
                | AgentAction::Errand | AgentAction::PairUp | AgentAction::ShareFindings | AgentAction::Pact) {
            return Err("that action needs a target".into());
        }
        let target = target.unwrap_or(who);
        let target_index = self.villagers.iter().position(|&v| v == target)
            .ok_or_else(|| "target isn't a villager".to_string())?;
        let ctx = match action {
            AgentAction::TakeJob => CtxAct::TakeJob,
            AgentAction::Douse => CtxAct::Douse,
            AgentAction::Tame => CtxAct::Tame,
            AgentAction::Recruit => CtxAct::Recruit,
            AgentAction::Matchmake => CtxAct::Matchmake,
            AgentAction::Errand => CtxAct::Errand,
            AgentAction::PairUp => CtxAct::PairUp,
            AgentAction::ShareFindings => CtxAct::ShareFindings,
            AgentAction::PickCaps => CtxAct::PickCaps,
            AgentAction::Snowballs => CtxAct::Snowballs,
            AgentAction::PetCat => CtxAct::PetCat,
            AgentAction::Pact => CtxAct::Pact,
        };
        let old_target = self.seats[seat].selected_target;
        let old_building = self.seats[seat].selected_building;
        let old_actions = self.seats[seat].ctx_actions.clone();
        self.seats[seat].selected_target = target_index;
        self.seats[seat].selected_building = None;
        self.update_ctx_actions_at(seat);
        let result = if let Some(i) = self.seats[seat].ctx_actions.iter().position(|&a| a == ctx) {
            if let Some(reason) = self.ctx_blocker_at(seat, ctx) {
                Err(reason)
            } else {
                self.ctx_act_at(seat, i);
                Ok(())
            }
        } else {
            Err("that action isn't available here".to_string())
        };
        self.seats[seat].selected_target = old_target;
        self.seats[seat].selected_building = old_building;
        self.seats[seat].ctx_actions = old_actions;
        result
    }

    /// What an agent asks its villager to do. Maps to the same travel/goal
    /// machinery the village's own errands use, so nothing happens until the
    /// walking is done - arrival triggers the trade, the chat, the gather.
    pub fn agent_command(&mut self, seat: usize, cmd: AgentCmd) -> Result<(), String> {
        self.journal.record(self.frame_n as u64, crate::session::journal::Input::AgentCmd {
            seat, cmd: cmd.clone() });
        let who = self.seats.get(seat).map(|s| s.player).ok_or_else(|| "no such seat".to_string())?;
        // the held can only stand down - walks, errands and gathers all
        // pass the jailer first
        if (self.world.get_component::<crate::game::dungeon::Imprisoned>(who).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(who).is_some())
            && !matches!(cmd, AgentCmd::Idle) {
            return Err("held - the cell allows no errands".into());
        }
        match cmd {
            AgentCmd::Idle => { self.travels.retain(|t| t.who != who); Ok(()) }
            AgentCmd::GoTo { x, y } => {
                let Some(p) = self.world.get_component::<Pos>(who) else { return Err("you aren't in the world".into()) };
                let from = (p.x.round() as i32, p.y.round() as i32);
                match self.trail_path(from, (x, y)) {
                    Some(mut path) => {
                        path.push((x, y));
                        self.travels.retain(|t| t.who != who);
                        self.travels.push(Travel { who, path, goal: Goal::Wander });
                        Ok(())
                    }
                    None => Err("no way there".into()),
                }
            }
            AgentCmd::TalkTo { e } => {
                // a fresh command supersedes the walk underway
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::TalkTo(e)) { Ok(()) } else { Err("no way to them".into()) }
            }
            AgentCmd::BuyFrom { seller, good, qty } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::BuyFrom(seller, g, qty)) { Ok(()) } else { Err("no way to them".into()) }
            }
            AgentCmd::BuyAt { shop, good } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::BuyAt(shop, g, None)) { Ok(()) } else { Err("no way to that shop".into()) }
            }
            AgentCmd::Gather { good, x, y } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send_gather(who, g, (x, y)) { Ok(()) } else { Err("nothing to gather there".into()) }
            }
            AgentCmd::FetchInput => {
                // the villagers' own errand: a kid runs it, or they go
                let Some(input) = livelihood::needs_input(&self.world, who) else {
                    return Err("the bench wants for nothing".into()) };
                // one errand at a time: the bench stays empty until a fetch
                // lands, so a second ask would buy the same goods twice
                // (a child under way on the job: one who broke off for food
                // keeps the OddJob but not the walk, and doesn't count)
                let hired = self.travels.iter().any(|t| matches!(t.goal, Goal::RunOddJob)
                    && self.world.get_component::<OddJob>(t.who).is_some_and(|j| j.hirer == who));
                if hired || self.travels.iter().any(|t| t.who == who) {
                    return Err("someone's already fetching it".into());
                }
                let i = self.villagers.iter().position(|&v| v == who).unwrap_or(0);
                if self.errand_for_input(who, input, i) { Ok(()) } else { Err(format!("no {} to be had", input)) }
            }
            AgentCmd::WorkDay => {
                let (made, up) = self.bench_work(who, self.tick);
                if made <= 0.0 && up.is_none() { Err("no craft to work".into()) } else { Ok(()) }
            }
            AgentCmd::Eat => {
                // step to the market store; the hunger logic does the rest
                let Some(p) = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x.round() as i32, p.y.round() as i32)) else {
                    return Err("no store".into()) };
                match self.trail_path(
                    self.world.get_component::<Pos>(who).map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(p), p)
                {
                    // already standing at the store: eat now, no walk to take
                    Some(path) if path.is_empty() => { self.arrive(who, Goal::Eat); Ok(()) }
                    Some(path) => { self.travels.retain(|t| t.who != who); self.travels.push(Travel { who, path, goal: Goal::Eat }); Ok(()) }
                    None => Err("no way to the store".into()),
                }
            }
            AgentCmd::JoinOrg => {
                let oe = self.seats[seat].org;
                if self.world.get_component::<Organization>(oe)
                    .map(|o| o.members.contains(&who)).unwrap_or(false) { return Ok(()); }
                // you sign on when the cause has woken you, or the fight's
                // your own - either way the issue itself organizes you
                let issue = crate::game::campaign::campaign_of(&self.world, oe)
                    .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
                    .map(|c| c.issue.clone());
                let Some(issue) = issue else { return Err("no cause to sign on with".into()) };
                let woken = self.world.get_component::<crate::game::reactions::Sympathized>(who)
                    .map(|s| s.issues.iter().any(|(i, _)| *i == issue)).unwrap_or(false);
                let own = crate::game::campaign::stake_for(&self.world, who, &issue, self.tick).is_some();
                if !(woken || own) { return Err("the cause hasn't reached you - come to a meeting first".into()); }
                if let Some(o) = self.world.get_component_mut::<Organization>(oe) { o.members.push(who); }
                Ok(())
            }
            AgentCmd::Press { prisoner } => {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(prisoner).is_none() {
                    return Err("no one inside by that name".into());
                }
                // a letter home, a vigil, a name kept spoken
                crate::game::dungeon::press(&mut self.world, prisoner, 0.1);
                Ok(())
            }
            AgentCmd::TakeNotice => {
                // you can only take what you can read: the board's the
                // medium, so the asker needn't be within earshot
                let Some(bp) = self.world.get_component::<Pos>(self.market_food).map(|q| (q.x, q.y))
                    else { return Err("no board".into()) };
                let p = self.world.get_component::<Pos>(who).map(|p| (p.x, p.y)).unwrap_or(bp);
                if ((bp.0 - p.0).powi(2) + (bp.1 - p.1).powi(2)).sqrt() >= 8.0 {
                    return Err("too far off to read the board".into());
                }
                if self.seats[seat].quest.is_some() {
                    return Err("you've already got a job on - see it through first".into());
                }
                if self.pending_quests.is_empty() {
                    return Err("nothing's posted you could take".into());
                }
                // the oldest posting goes first, same as the board reads
                self.take_pending_quest(seat, 0);
                self.log.push_ev(
                    EvTag::new(Thread::Economy, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: {} takes a job off the board.",
                        self.tick,
                        self.name_of(who)
                    ),
                );
                Ok(())
            }
            AgentCmd::Context { action, target } => self.agent_context_action(seat, action, target),
        }
    }

    /// One frame's step of the player in direction (dx, dy) (tile axes),
    /// quicker along a road and slower through the woods. Blocked straight on, they slide along whatever
    /// is in the way where they can.
    pub fn walk_at(&mut self, seat: usize, dx: f32, dy: f32) {
        let (px, py) = self.player_pos_at(seat);
        let at = (px.round() as i32, py.round() as i32);
        // a well-worn path is quicker underfoot, whoever's feet wear it
        let worn = self.trails.get(&at).map(|t| t.strength).unwrap_or(0.0);
        let ailing = self.world.get_component::<villager::Condition>(self.seats[seat].player)
            .map(|c| self.tick < c.until).unwrap_or(false);
        let spd = WALK_SPEED * self.walk_pace() / trails::movement_cost(1.0, worn, TRAIL_K) * match self.map.get(at.0, at.1) {
            Some(Tile::Road) | Some(Tile::Bridge) => ROAD_SPEEDUP,
            Some(Tile::Ford) => WADE_SLOWDOWN,
            _ if self.map.trees.contains(&at) => WOODS_SLOWDOWN,
            _ => 1.0,
        } * if ailing { 0.7 } else { 1.0 };
        // the same pace whichever way they go, diagonals included
        let len = (dx * dx + dy * dy).sqrt();
        // a still stick or noise on the wire: no step, and never a NaN position
        if !(len.is_finite() && len > 0.0) { return; }
        let (nx, ny) = (px + dx / len * spd, py + dy / len * spd);
        // a fast clock moves more than a tile a frame: take the stride in
        // half-tile steps so a wall or the river between here and there is
        // checked, not hopped. Each step keeps the slide round whatever's
        // in the way
        let steps = ((spd / 0.5).ceil() as u32).max(1);
        let (ux, uy) = ((nx - px) / steps as f32, (ny - py) / steps as f32);
        let mut at = (px, py);
        for _ in 0..steps {
            let Some(nxt) = [(at.0 + ux, at.1 + uy), (at.0 + ux, at.1), (at.0, at.1 + uy)]
                .into_iter().find(|&(x, y)| self.walkable_for(seat, x, y)) else { break };
            let ot = (at.0.round() as i32, at.1.round() as i32);
            at = nxt;
            let nt = (at.0.round() as i32, at.1.round() as i32);
            // feet wear the ground the player walks, same as any villager's
            if nt != ot {
                let tr = self.trails.entry(nt).or_insert(Trail::wild());
                trails::step_trail(tr, 1, 0.0, &TRAIL_PARAMS);
            }
        }
        if let Some(p) = self.world.get_component_mut::<Pos>(self.seats[seat].player) {
            p.x = at.0;
            p.y = at.1;
        }
    }

    pub fn player_pos_at(&self, seat: usize) -> (f32, f32) {
        self.world.get_component::<Pos>(self.seats[seat].player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
    }

    /// A left click on the map at screen point (mx, my), seen through the
    /// caller's camera: rivals rebuff or greet you, villagers get selected
    pub fn click_map_at(&mut self, seat: usize, mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32) {
        let tick = self.tick;
        // the market board: click near the market cross and you're
        // reading what's pinned - the panel opens, the notices stand
        if let Some(mp) = self.world.get_component::<Pos>(self.market_food).copied() {
            let (sx, sy) = iso_to_screen_cam(mp.x, mp.y, cam_x, cam_y, zoom);
            let d = (((sx - mx) as f32).powi(2) + ((sy - my) as f32).powi(2)).sqrt();
            if mx < MAP_W as i32 && d < 30.0 {
                self.seats[seat].notice_panel = !self.seats[seat].notice_panel;
                self.log.push_ev(
                    EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: {} the board's postings.",
                        tick,
                        if self.seats[seat].notice_panel {
                            "you read"
                        } else {
                            "you step away from"
                        }
                    ),
                );
                return;
            }
        }
        // click on a rival villager -> they rebuff you (foreign village is wary)
        for (i, rv) in self.rivals.clone().into_iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(rv) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                if mx < MAP_W as i32 && d < 24.0 {
                    // the rivals read the cause, not a score: leading an
                    // org with members opens the door, a cause with anyone
                    // in it gets a wary word, and otherwise you're a stranger
                    let (is_leader, has_cause) = self.world
                        .get_component::<Organization>(self.seats[seat].org)
                        .map(|o| (o.leader == self.seats[seat].player,
                            o.members.iter().any(|&m| !crate::game::villager::is_dead(&self.world, m))))
                        .unwrap_or((false, false));
                    let (msg, tag) = if is_leader {
                        (format!("{}: we've heard of you - the organizer.", RIVAL_NAMES[i]), "opened")
                    } else if has_cause {
                        (format!("{}: word travels. state your business.", RIVAL_NAMES[i]), "wary")
                    } else {
                        (format!("{}: we don't know you, outsider.", RIVAL_NAMES[i]), "rebuffed")
                    };
                    self.say(rv, &msg, TALK_BUBBLE_SECS);
                    self.log.push_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!("t{}: {} {} your approach.", tick, RIVAL_NAMES[i], tag),
                    );
                }
            }
        }
        // click on map selects nearest villager - by the figure's box,
        // not its feet: a body is ~14px across standing ~30px tall (zoom
        // scales both), so the part that shows past a roof still answers
        // the click (#944)
        let on_map = mx < MAP_W as i32;
        let z = zoom.max(0.5);
        let box_d = |sx: i32, sy: i32| {
            let dx = ((mx - sx).abs() as f32 - (14.0 * z + 4.0)).max(0.0);
            let dy = (if my <= sy { (sy - my) as f32 - (34.0 * z + 6.0) }
                else { (my - sy) as f32 - (8.0 * z + 4.0) }).max(0.0);
            (dx * dx + dy * dy).sqrt()
        };
        // the road people answer a hail with their pitch: the rare
        // visitors carry their own cry, the named travellers a line of
        // their trade - none are on the roster, so a click hails rather
        // than selects
        let mut hailed: Vec<(Entity, &'static str, String)> = vec![];
        for r in &self.rare {
            let Some(e) = r.e else { continue };
            let s = events::spec(r.id);
            if !matches!(s.fig, events::Fig::Traveller) { continue; }
            hailed.push((e, s.name, s.cry.to_string()));
        }
        if let Some(d) = &self.doctor { hailed.push((d.e, "the plague doctor", "Stand back - the miasma's about.".into())); }
        if let Some(m) = &self.minstrel { hailed.push((m.e, "the minstrel", "A tune, then - what's your fancy?".into())); }
        if let Some(m) = &self.pedlar { hailed.push((m.e, "the pedlar", "Pots, pins, and word of the next village.".into())); }
        if let Some(c) = &self.circus { hailed.push((c.e, "the circus", "Come see! Wonders from over the hill!".into())); }
        if let Some(b) = &self.ink_boiler { hailed.push((b.e, "the ink-boiler", "Ink for the printer - and your crusts come back fried.".into())); }
        for c in &self.couriers { hailed.push((c.e, "the courier", "Letters for the far town - I can't linger.".into())); }
        for (e, name, line) in hailed {
            let Some(p) = self.world.get_component::<Pos>(e) else { continue };
            let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
            if on_map && box_d(sx, sy) < 4.0 {
                self.say(e, &line, TALK_BUBBLE_SECS);
                self.log.push_ev(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!("t{}: you hail {}.", tick, name),
                );
            }
        }
        let mut best = self.seats[seat].selected_target;
        let mut best_d = f32::MAX;
        let mut best_sy = i32::MIN;
        for (i, v) in self.villagers.iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(*v) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = box_d(sx, sy);
                // inside two overlapping boxes, the one in front answers
                if on_map && d < 4.0 && (d < best_d - 1e-3 || ((d - best_d).abs() < 1e-3 && sy > best_sy)) {
                    best = i; best_d = d; best_sy = sy;
                }
            }
        }
        // click on a building: the inspector shows what it's for, who
        // works or lives there, and what it has for offer
        let mut best_b: Option<usize> = None;
        let mut best_bd = 9999.0f32;
        for (i, (bx, by, _)) in self.buildings.iter().enumerate() {
            let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
            let d = ((sx as f32 - mx as f32).powi(2) + ((sy as f32 - 14.0 * zoom) - my as f32).powi(2)).sqrt();
            if on_map && d < 42.0 && d < best_bd { best_b = Some(i); best_bd = d; }
        }
        // the villager wins a shared click - they're the one in front
        let picked_b = if best_d <= best_bd { None } else { best_b };
        if picked_b != self.seats[seat].selected_building {
            // a new pick ends any walk to the old one's shelf
            self.seats[seat].buy_walk = None;
        }
        self.seats[seat].selected_building = picked_b;
        if self.seats[seat].selected_building.is_some() { self.seats[seat].picked = false; self.seats[seat].pick_near = false; return; }
        if best_d < 4.0 {
            self.seats[seat].picked = true;
            self.seats[seat].pick_near = self.dist_to_at(seat, best) <= SHOUT_DIST;
        }
        self.seats[seat].selected_target = best;
    }

    /// The camera centre for the map tile under minimap point (mx, my),
    /// if (mx, my) is on the minimap
    pub fn minimap_look(&self, mx: i32, my: i32) -> Option<(i32, i32)> {
        minimap_look_map(&self.map, mx, my)
    }

    // ---- seat-0 conveniences: the solo game and tests drive seat 0 ----
    pub fn act(&mut self, i: usize) { self.act_at(0, i) }

    pub fn ctx_act(&mut self, i: usize) { self.ctx_act_at(0, i) }

    pub fn walk(&mut self, dx: f32, dy: f32) { self.walk_at(0, dx, dy) }

    pub fn click_map(&mut self, mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32) { self.click_map_at(0, mx, my, cam_x, cam_y, zoom) }

    pub fn command(&mut self, cmd: Command) { self.command_at(0, cmd) }

    pub fn ui_model(&self) -> UiModel { self.ui_model_at(0) }

    pub fn action_blocker(&self, i: usize) -> Option<String> { self.action_blocker_at(0, i) }

    pub fn disabled_reason(&self, i: usize) -> Option<String> { self.disabled_reason_at(0, i) }

    pub fn villagers_in_reach(&self) -> Vec<Entity> { self.villagers_in_reach_at(0) }

    pub fn reach_target(&self) -> usize { self.reach_target_at(0) }

    pub fn player_pos(&self) -> (f32, f32) { self.player_pos_at(0) }

    pub fn update_ctx_actions(&mut self) { self.update_ctx_actions_at(0) }

    pub fn advance_approach(&mut self) { self.advance_approach_at(0) }

    pub fn advance_selection(&mut self) { self.advance_selection_at(0) }

    pub fn call_meeting(&mut self) -> String {
        // calling a meeting is an action: the line goes in the log like
        // the buttons' lines do (ctx_act_at logs for the Gather path)
        let msg = self.call_meeting_at(0);
        self.log.push_ev(
            EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
            format!("t{}: {}", self.tick, msg),
        );
        msg
    }

    pub fn talk_to(&mut self, i: usize) -> String { self.talk_to_at(0, i) }

    pub fn dist_to(&self, who: usize) -> f32 { self.dist_to_at(0, who) }

    pub fn path_to(&self, who: usize) -> Option<Vec<(i32, i32)>> { self.path_to_at(0, who) }

    pub fn start_approach(&mut self, who: usize, action: usize) -> Option<String> { self.start_approach_at(0, who, action) }

    pub fn build_site(&self) -> Option<(f32, f32)> { self.build_site_at(0) }
}
