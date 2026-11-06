//! The public projections of the sim: a seat's UiModel (frame, panel, inspector) and an agent's AgentObs. What a seat may know is decided here.

use crate::session::*;

/// Gossip lines shown in the log at once; older gossip is hidden so it
/// can't push real events (illness, joins, deaths, guilds) out of view
pub(in crate::session) const LOG_GOSSIP_SHOWN: usize = 2;

/// Is this log entry idle gossip ("X tells Y - ...")?
pub fn is_gossip(entry: &str) -> bool { entry.contains(" tells ") }

/// Weather changes shown in the log at once (the HUD names today's weather)
pub(in crate::session) const LOG_WEATHER_SHOWN: usize = 1;

/// Is this log entry a routine weather change ("drizzle begins", "the rain stops")?
pub(in crate::session) fn is_weather_note(entry: &str) -> bool {
    entry.ends_with(" begins") || entry.contains("the rain stops") || entry.contains("the snow stops")
}

/// Town business shown in the log at once: promotions, the minstrel, the
/// tax man's visits - nice to know, but not worth pushing illness and
/// deaths out of view
pub(in crate::session) const LOG_TOWN_SHOWN: usize = 3;

/// Is this log entry routine town business?
pub(in crate::session) fn is_town_note(entry: &str) -> bool {
    entry.contains(" is now a ") || entry.contains("has come to town")
        // the country's carts and stewards on the road
        || entry.contains("'s cart ") || entry.contains("to fetch provisions")
        || entry.contains("takes his cut") || entry.contains("Organizer discovers issue")
        || entry.contains("the fold") || entry.contains("a fox")
        || entry.contains("brought") || entry.contains(" treats ")
        || entry.contains("skeps") || entry.contains("swarm")
        || entry.contains("takes up") || entry.contains("shuttered")
        || entry.contains("the talk turns") || entry.contains("cold shoulder")
        || entry.contains("boycott") || entry.contains("a crowd at")
        || entry.contains("the village gathers") || entry.contains("pays triple")
        || entry.contains("a pedlar") || entry.contains("pedlar's moved on")
        || entry.contains("rides over the walls") || entry.contains("circus")
        || entry.contains("bailiff's writ") || entry.contains("a summons")
        || entry.contains("winter") || entry.contains("thin tempers")
        || entry.contains("gets its colors")
}

/// An entry the player should still see rows later: illness and loss,
/// the exile's road, the discovery told
pub(in crate::session) fn is_important(entry: &str) -> bool {
    entry.contains("is taken with") || entry.contains("recovered") || entry.contains("died")
        || entry.contains("driven out") || entry.contains("run off")
        || entry.contains("whispers it about") || entry.contains("stocks")
        || entry.contains("born") || entry.contains("dead")
        || entry.contains("taken for questioning")
}

/// The log entries to show, newest first: every event, but only the latest
/// LOG_GOSSIP_SHOWN pieces of gossip and the latest weather change.
/// The moments that matter - sickness, death, an exile - pin to the top:
/// routine village business scrolls below them no matter how busy it gets.
pub(in crate::session) fn visible_log(log: &[String]) -> Vec<&String> {
    let (mut gossip, mut weather, mut town) = (0, 0, 0);
    let filtered: Vec<&String> = log.iter().rev().filter(|e| {
        if is_gossip(e) { gossip += 1; return gossip <= LOG_GOSSIP_SHOWN; }
        if is_weather_note(e) { weather += 1; return weather <= LOG_WEATHER_SHOWN; }
        if is_town_note(e) { town += 1; return town <= LOG_TOWN_SHOWN; }
        true
    }).collect();
    // stable partition: important entries first (already newest-first),
    // then the rest - a quiet death still outranks a busy skep-day
    let (important, rest): (Vec<_>, Vec<_>) = filtered.into_iter()
        .partition(|e| is_important(e));
    important.into_iter().chain(rest).collect()
}

/// A villager's attachment style as a player would put it: the card showed
/// the psychology term ("Style: Disorganized"), which reads like a fault
pub fn nature_word(style: crate::sim::psychology::AttachmentStyle) -> &'static str {
    use crate::sim::psychology::AttachmentStyle::*;
    match style {
        Secure => "steady and trusting",
        Anxious => "anxious, needs reassurance",
        Avoidant => "keeps to themselves",
        Disorganized => "wary and unpredictable",
    }
}

/// How fed a villager is, in words. `Hunger` stores fullness (100 = just
/// ate, 0 = starving), so the raw number reads backwards as "hunger".
pub fn fullness_word(fullness: f32) -> &'static str {
    if fullness >= 70.0 { "full" }
    else if fullness >= 40.0 { "peckish" }
    else if fullness >= 15.0 { "hungry" }
    else { "starving" }
}

/// What a villager is doing, in plain words
pub(in crate::session) fn activity_text(a: Activity) -> &'static str {
    match a {
        Activity::Idle => "idle",
        Activity::SeekingFood => "looking for food",
        Activity::Eating => "eating",
    }
}

impl Game {
    /// The frame a client draws: the authoritative sim's whole view of the
    /// world this seat should see. One per player seat per broadcast.
    pub fn ui_model_at(&self, seat: usize) -> UiModel {
        self.ui_model_with(seat, true)
    }

    /// The seat's frame without the tile map (an empty `TileMap` in its
    /// place): what the server streams, since the map - 98% of a frame -
    /// goes out separately, only when it changes (`ServerMsg::Map`)
    pub fn ui_model_lean_at(&self, seat: usize) -> UiModel {
        self.ui_model_with(seat, false)
    }

    fn ui_model_with(&self, seat: usize, with_map: bool) -> UiModel {
        let mut m = self.ui_model_parts_at(seat, with_map);
        // the debug readout shows what this seat could know, never the
        // secret gauges: the model goes over the wire to remote clients
        m.debug_lines = kpi::kpi_lines(&kpi::public_kpis(&m));
        m
    }

    /// The seat's frame, all but the lines derived from the frame itself
    pub(in crate::session) fn ui_model_parts_at(&self, seat: usize, with_map: bool) -> UiModel {
        let world = &self.world;
        let season = world.get_component::<climate::Climate>(self.climate).unwrap().season;
        let in_reach = self.villagers_in_reach_at(seat);
        let mut people: Vec<PersonV> = vec![];
        // the fight this org is in - woke only marks someone whose shock
        // matches it (a stranger woken to a different cause isn't your
        // recruit)
        let fight = crate::game::campaign::campaign_of(world, self.seats[seat].org)
            .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c))
            .map(|c| c.issue.clone());
        for (i, v) in self.villagers.iter().enumerate() {
            let Some(p) = world.get_component::<Pos>(*v) else { continue };
            // what the cause has done to them, read over their head:
            // taken (in the dungeon), hollow (a blow-off to coach),
            // woke (moved by the cause - recruitable)
            let cause_mark = if world.get_component::<crate::game::dungeon::Imprisoned>(*v).is_some() {
                " (taken)"
            } else if world.get_component::<crate::game::actions::Demoralized>(*v).is_some() {
                " (hollow)"
            } else if fight.as_ref().map(|issue|
                world.get_component::<crate::game::reactions::Sympathized>(*v)
                    .map(|s| s.issues.iter().any(|(i, _)| i == issue)).unwrap_or(false))
                .unwrap_or(false) {
                " (woke)"
            } else { "" };
            let kid = world.get_component::<Kid>(*v).is_some();
            // the marks go by the state, not the roll: a child grown
            // drops the (child), and whoever holds the herb hut is the herb-wife
            let role = if Some(*v) == self.herbologist { " (herb-wife)" }
                else if kid { " (child)" } else { "" };
            // the find box knows them by name and posting both: "heart
            // helper" finds the matchmaker even where her trade reads first
            let mut find = format!("{} {}",
                vill_name(i), crate::game::desires::occupation_of(world, *v).unwrap_or_default());
            if let Some(r) = world.get_component::<vocation::TownRole>(*v) {
                find.push(' ');
                find.push_str(crate::game::desires::town_role_name(*r));
            }
            people.push(PersonV {
                e: *v, kind: PKind::Villager(i), pos: (p.x, p.y),
                label: Some(format!("{}{}{}", vill_name(i), role, cause_mark)),
                find: find_key(&find),
                dead: villager::is_dead(world, *v),
                kid,
                selected: self.seats[seat].selected_target == i && self.seats[seat].picked,
                in_reach: in_reach.contains(v),
                hungry: world.get_component::<Hunger>(*v).map(|h| h.value < 40.0).unwrap_or(false),
                sick: is_sick(world, *v),
                act_mark: match world.get_component::<Villager>(*v).map(|v| v.activity) {
                    Some(Activity::SeekingFood) => 1, Some(Activity::Idle) | None => 0, _ => 2 },
                bond: false,
            });
        }
        for (i, v) in self.rivals.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                people.push(PersonV { e: *v, kind: PKind::Rival(i), pos: (p.x, p.y),
                    label: Some(RIVAL_NAMES[i].to_string()), find: find_key(RIVAL_NAMES[i]),
                    dead: false, kid: false,
                    selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        let (px, py) = self.player_pos_at(seat);
        people.push(PersonV { e: self.seats[seat].player, kind: PKind::Player, pos: (px, py),
            label: None, find: world.get_component::<PlayerCharacter>(self.seats[seat].player)
                .map(|p| find_key(&p.name)).unwrap_or_default(),
            dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        for (i, (x, y)) in self.guards.iter().copied()
            .chain(self.posted.iter().map(|p| (p.1, p.2))).enumerate() {
            people.push(PersonV { e: 0, kind: PKind::Guard(i), pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
        }
        people.push(PersonV { e: 0, kind: PKind::Priest, pos: self.priest,
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
        if let Some((tx, ty)) = self.traveller_pos() {
            people.push(PersonV { e: 0, kind: PKind::Traveller, pos: (tx, ty),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
        }
        if let Some(m) = &self.minstrel {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
            }
        }
        if let Some(m) = &self.pedlar {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
            }
        }
        if let Some(c) = &self.circus {
            if let Some(p) = world.get_component::<Pos>(c.e) {
                people.push(PersonV { e: c.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the circus".into()), find: find_key("the circus"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        if let Some(b) = &self.ink_boiler {
            if let Some(p) = world.get_component::<Pos>(b.e) {
                people.push(PersonV { e: b.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the ink-boiler".into()), find: find_key("the ink-boiler"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        // the rare events' figures: wandering trades drawn as travellers,
        // strays drawn as the beasts they are - all labelled by their errand
        for r in &self.rare {
            let Some(e) = r.e else { continue };
            let Some(p) = world.get_component::<Pos>(e) else { continue };
            let spec = events::spec(r.id);
            let kind = match spec.fig {
                events::Fig::Traveller => PKind::Traveller,
                events::Fig::Pet(s) => PKind::Pet(s),
            };
            people.push(PersonV { e, kind, pos: (p.x, p.y),
                label: Some(spec.name.into()), find: find_key(spec.name), dead: false, kid: false, selected: false,
                in_reach: false, hungry: false, sick: false, act_mark: 0,
                bond: world.get_component::<Pet>(e).map(|p| p.bond_strength >= 0.5).unwrap_or(false) });
        }
        for &(x, y) in &self.gulls {
            people.push(PersonV { e: 0, kind: PKind::Gull, pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
        }
        let hen_t = (self.frame_n / 12) as f32;
        people.push(PersonV { e: 0, kind: PKind::Chicken,
            pos: (self.chicken_at.0 + hen_t.cos() * 0.5, self.chicken_at.1 + (hen_t * 1.7).sin() * 0.4),
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
        for (pet_e, species, (ox, oy)) in &self.pets {
            if let Some(pet) = world.get_component::<Pet>(*pet_e) {
                if let Some(p) = world.get_component::<Pos>(pet.owner.unwrap_or(self.villagers[0])) {
                    people.push(PersonV { e: *pet_e, kind: PKind::Pet(*species),
                        pos: (p.x + ox, p.y + oy), label: None, dead: false, kid: false,
                        selected: false, in_reach: false, hungry: false, sick: false,
                        act_mark: 0, bond: pet.bond_strength > 0.5, find: String::new() });
                }
            }
        }
        for &a in &self.flock {
            if let (Some(l), Some(p)) = (world.get_component::<Livestock>(a), world.get_component::<Pos>(a)) {
                people.push(PersonV { e: a, kind: if l.kind == LivestockKind::Sheep { PKind::Sheep } else { PKind::Goat },
                    pos: (p.x, p.y), label: None, dead: false, kid: false, selected: false,
                    in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
            }
        }
        for &a in &self.hens {
            if let Some(p) = world.get_component::<Pos>(a) {
                people.push(PersonV { e: a, kind: PKind::Chicken, pos: (p.x, p.y), label: None,
                    dead: false, kid: false, selected: false, in_reach: false,
                    hungry: false, sick: false, act_mark: 0, bond: false, find: String::new() });
            }
        }
        people.sort_by(|a, b| (a.pos.0 + a.pos.1).partial_cmp(&(b.pos.0 + b.pos.1)).unwrap());

        let risen = |i: usize| self.works.iter().find(|w| w.building == i).map(|w| w.risen()).unwrap_or(1.0);
        let buildings: Vec<BldgV> = self.buildings.iter().enumerate()
            .map(|(i, (x, y, s))| BldgV { x: *x, y: *y, sprite: *s, risen: risen(i),
                derelict: self.derelict.contains(&i), smoke: i < 3 })
            .collect();
        let bubbles: Vec<BubbleV> = self.bubbles.iter().filter_map(|(e, text, _, thought, _)| {
            // the dead don't talk - a bubble over a grave is ghost chatter
            if villager::is_dead(world, *e) { return None; }
            let p = world.get_component::<Pos>(*e)?;
            let name = self.villagers.iter().position(|v| *v == *e).map(|i| vill_name(i).to_string())
                .or_else(|| self.rivals.iter().position(|r| *r == *e).map(|i| RIVAL_NAMES[i].to_string()));
            Some(BubbleV { e: *e, pos: (p.x, p.y), text: text.clone(), thought: *thought, name })
        }).collect();
        let org = world.get_component::<Organization>(self.seats[seat].org).unwrap();
        let members: Vec<String> = org.members.iter()
            .filter_map(|m| self.villagers.iter().position(|v| v == m).map(|i| {
                let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(*m).is_some();
                format!("{}{}", vill_name(i), if held { " (held)" } else { "" })
            }))
            .collect();
        let issues: Vec<String> = world.get_component::<Organizer>(self.seats[seat].player)
            .map(|o| o.issues.iter().map(|t| desires::issue_name(t.clone())).collect()).unwrap_or_default();
        // a player seat drives a PlayerCharacter; an agent seat drives a plain
        // villager, whose panel is their own name and purse
        let me = self.seats[seat].player;
        let (my_name, my_gold) = match world.get_component::<PlayerCharacter>(me) {
            Some(p) => (p.name.clone(), p.resources),
            None => (self.name_of(me),
                world.get_component::<Purse>(me).map(|p| p.gold).unwrap_or(0.0)),
        };
        let food = world.get_component::<villager::FoodSource>(self.market_food).map(|f| f.servings).unwrap_or(0);
        let guilds = world.query::<Guild>().len();
        let name_of = |e: Entity| {
            if e == self.seats[seat].player { return "you".to_string(); }
            self.villagers.iter().position(|v| *v == e).map(|i| vill_name(i).to_string())
                .or_else(|| self.rivals.iter().position(|r| *r == e).map(|i| RIVAL_NAMES[i].to_string()))
                .unwrap_or_else(|| "someone".to_string())
        };
        let campaign = crate::game::campaign::campaign_of(world, self.seats[seat].org).map(|e| {
            use crate::game::campaign::{Campaign, CampaignMeeting, Framing, Role};
            let c = world.get_component::<Campaign>(e).unwrap();
            let mtg = world.get_component::<CampaignMeeting>(e);
            let seats = [Role::TurnoutCoordinator, Role::NoteTaker, Role::Chair].iter().map(|&r| {
                let label = match r { Role::TurnoutCoordinator => "turnout", Role::NoteTaker => "notes", Role::Chair => "chair" };
                let holder = c.seats.iter().find(|s| s.role == r).map(|s| name_of(s.holder))
                    .unwrap_or_else(|| "(empty)".to_string());
                format!("{} {}", label, holder)
            }).collect();
            CampaignV {
                issue: c.issue.clone(),
                framing: match c.framing { Framing::SharedGrievance => "shared grievance",
                    Framing::CommonFoe => "common foe", Framing::MoralDuty => "moral duty" }.to_string(),
                dissenters: c.dissenters.len(), seats,
                resolves: mtg.map(|m| m.resolves).unwrap_or(0),
                due: mtg.map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false),
                cadence_days: mtg.map(|m| m.cadence_days).unwrap_or(0),
            }
        });
        let blocked = [self.action_blocker_at(seat, 0), self.action_blocker_at(seat, 1), self.action_blocker_at(seat, 2)];
        // the bond ledger: who the seat has spoken to, by how far each
        // relationship has climbed - the same stages the inspector names.
        // rapport is talk-only, so a barter or a shared meal can't pad
        // the "spoken to" headline the way any stage_of bond could
        let mut social = SocialV::default();
        for &v in &self.villagers {
            if crate::game::player::rapport(world, self.seats[seat].player, v) <= 0.0 { continue; }
            use crate::game::organizing::RelStage;
            match crate::game::organizing::stage_of(world, self.seats[seat].player, v) {
                RelStage::Stranger => {}
                RelStage::Acquaintance => { social.met += 1; social.acquaintances += 1; }
                RelStage::Known => { social.met += 1; social.known += 1; }
                RelStage::Follower => { social.met += 1; social.followers += 1; }
                RelStage::CoLeader => { social.met += 1; social.coleaders += 1; }
            }
        }
        // the meeting plan's yeses, by name - who's said they'd come and
        // what promises are filled, so planning shows its progress
        let meeting = self.world.get_component::<crate::game::organizing::MeetingPlan>(self.seats[seat].org)
            .filter(|p| !p.holders().is_empty())
            .map(|p| {
                let who = |e: Entity| name_of(e);
                let mut roles: Vec<String> = vec![];
                if let Some((host, _)) = p.venue { roles.push(format!("house: {}", who(host))); }
                for (slot, label) in [(p.snacks, "snacks"), (p.notes, "notes"), (p.opener, "opener"),
                    (p.song, "song"), (p.closer, "closer")] {
                    if let Some(e) = slot { roles.push(format!("{}: {}", label, who(e))); }
                }
                MeetingPlanV {
                    coming: p.coming.iter().map(|&e| who(e)).collect(),
                    spreaders: p.spreaders.iter().map(|&e| who(e)).collect(),
                    roles,
                }
            });
        let inspector = if self.seats[seat].selected_building.filter(|_| !self.seats[seat].picked).is_some() {
            self.building_inspector_at(seat)
        } else {
            Some(self.villager_inspector_at(seat))
        };
        UiModel {
            player_pose: (self.seats[seat].facing, self.seats[seat].walking),
            // the courier (entity 0 in people) faces the way it rides
            poses: self.villager_poses.iter().map(|(&e, &(_, f, w))| (e, f, w))
                .chain(self.traveller_pos().zip(self.traveller_pos_at(self.frame_n.wrapping_sub(1)))
                    .map(|(now, was)| (0, facing_on_screen(now.0 - was.0, now.1 - was.1), true))).collect(),
            tick: self.tick, frame: self.frame_n,
            map: if with_map { self.map.clone() } else { crate::game::tilemap::TileMap::default() }, town_wall: self.town_wall,
            season, precip: self.precip, rainbow_from: self.rainbow_from,
            paused: self.paused, speed: self.speed,
            stones: self.stones.clone(), buildings, people, bubbles,
            castle_keeps: self.map.castle_keeps.iter().map(|&(x, y)| (x as f32, y as f32)).collect(),
            settlement_houses: self.map.settlement_houses.iter().map(|&(x, y)|
                (x as f32, y as f32, BSprite::House((noise(x, y, 41) * 97.0) as usize))).collect(),
            toasts: self.toasts.iter().filter(|(_, until, _)| self.tick < *until)
                .map(|(m, _, at)| ToastV { text: m.clone(), at: *at }).collect(),
            // only the lines the window would show - cloning the whole
            // history every frame was the model's biggest copy
            log: visible_log(&self.log).into_iter().cloned().collect(),
            sel: Selection { target: self.seats[seat].selected_target, picked: self.seats[seat].picked, building: self.seats[seat].selected_building },
            reach: in_reach.iter().filter_map(|v| self.villagers.iter().position(|x| x == v)).collect(),
            reach_target: self.reach_target_at(seat),
            blocked, ctx: self.seats[seat].ctx_actions.clone(),
            ctx_blocked: self.seats[seat].ctx_actions.iter()
                .map(|a| self.ctx_blocker_at(seat, *a)).collect(),
            inspector,
            panel: PanelV {
                name: my_name, gold: my_gold,
                org: org.name.clone(), members, issues, guilds, food, campaign,
                quest: self.seats[seat].quest.as_ref().map(|q| self.quest_panel_line(q)),
                social, meeting,
            },
            mill: Some(self.mill),
            herbs: self.herb_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            mush: self.mush_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            skeps: self.hives.iter().filter_map(|&h| world.get_component::<Pos>(h)
                .map(|p| (p.x, p.y))).collect(),
            swarm: self.swarm.and_then(|(w, _)| world.get_component::<Pos>(w).map(|p| (p.x, p.y))),
            market: world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y)),
            feast: self.villagers.iter().any(|&v| world.get_component::<routine::Routine>(v)
                .is_some_and(|r| r.mode == routine::Mode::Feasting && r.day == self.tick)),
            stalls: self.stalls_standing(),
            trails: self.trails.iter().filter(|(_, t)| t.strength > 0.15)
                .map(|(&(x, y), t)| (x as f32, y as f32, t.strength)).collect(),
            stumps: self.land.tree_cover.iter().filter(|(_, &c)| c < 0.3)
                .map(|(&(x, y), _)| (x as f32, y as f32)).collect(),
            murk: self.land.effluent.iter().filter(|(_, &e)| e > 0.4)
                .map(|(&(x, y), _)| (x as f32, y as f32)).collect(),
            pond: if self.land.pit_water >= 0.7 {
                self.pit.map(|(x, y)| (x as f32, y as f32))
            } else { None },
            burning: self.fire.burning.keys().map(|&(x, y)| (x as f32, y as f32)).collect(),
            scorched: self.fire.scorched.keys().map(|&(x, y)| (x as f32, y as f32)).collect(),
            gardens: self.gardens.iter()
                .map(|&(_, (x, y), g, _)| (x as f32, y as f32, g.min(1.0))).collect(),
            sky: self.sky.map(|(k, _)| k.to_string()),
            painted: self.painted.iter()
                .filter_map(|&(b, _)| self.world.get_component::<Pos>(b).map(|p| (p.x, p.y))).collect(),
            cave: self.cave_pos.map(|(x, y)| (x as f32, y as f32)),
            talk: self.talk_panel_at(seat),
            convo: self.convo_panel_at(seat),
            debug_lines: vec![],
            sfx: self.sfx.iter().map(|c| c.kind.to_string()).collect(),
        }
    }

    /// The inspector card for the building this seat clicked
    pub(in crate::session) fn building_inspector_at(&self, seat: usize) -> Option<InspectorV> {
        let bi = self.seats[seat].selected_building?;
        let be = *self.building_entities.get(bi)?;
        let world = &self.world;
        let b = world.get_component::<Building>(be)?;
        let mut lines: Vec<Line> = vec![];
        let kind_word = match b.kind {
            BuildingKind::Home => "Home",
            BuildingKind::Shop(_) => "Workshop",
            BuildingKind::Church => "Church",
            BuildingKind::WatchPost => "Watch",
            BuildingKind::Mill => "Mill",
            BuildingKind::Granary => "Granary",
            BuildingKind::TraderStall => "Trader's stall",
            BuildingKind::Pasture => "Pasture",
            BuildingKind::Apiary => "Apiary",
            BuildingKind::Cellar => "Cheese cellar",
        };
        lines.push(Line { x: 18, y: 326, text: b.name.clone(), rgb: (255, 255, 255) });
        lines.push(Line { x: 18, y: 340, text: kind_word.into(), rgb: (200, 200, 230) });
        let worker = b.worker.and_then(|w| self.villagers.iter().position(|v| *v == w))
            .map(|i| vill_name(i).to_string())
            .or_else(|| b.worker.map(|_| "you".to_string()));
        let who = worker.map(|w| format!("worked by {}", w)).unwrap_or_else(|| "stands empty".to_string());
        let col = if household::derelict(b) { (230, 140, 90) } else { (190, 190, 210) };
        lines.push(Line { x: 18, y: 354, text: who, rgb: col });
        if let Some(o) = b.overseer {
            let on = self.villagers.iter().position(|v| *v == o).map(|i| vill_name(i)).unwrap_or("someone");
            lines.push(Line { x: 18, y: 368, text: format!("overseer: {}", on), rgb: (200, 160, 120) });
        }
        if b.kind == BuildingKind::Home {
            let names: Vec<&str> = b.occupants.iter()
                .filter_map(|o| self.villagers.iter().position(|v| *v == *o).map(|i| vill_name(i)))
                .collect();
            lines.push(Line { x: 18, y: 368,
                text: format!("lives here: {}", if names.is_empty() { "no one".into() } else { names.join(", ") }),
                rgb: (190, 190, 210) });
        }
        let offers = household::on_offer_at(&self.world, b);
        let stock_line = offers.iter().take(3).map(|(g, q, _)| format!("{} {}", g, q))
            .collect::<Vec<_>>().join(" \u{00b7} ");
        lines.push(Line { x: 18, y: 390,
            text: if stock_line.is_empty() { "stock: empty".into() } else { format!("stock: {}", stock_line) },
            rgb: (180, 190, 210) });
        if let Some((g, q, pr)) = offers.first() {
            lines.push(Line { x: 18, y: 404, text: format!("for offer: {} {} @ {:.0}g", q, g, pr), rgb: (150, 210, 150) });
        }
        Some(InspectorV { fig: None, kid: false, dead: false, died_day: None, lines })
    }

    /// The inspector card for the selected villager
    pub(in crate::session) fn villager_inspector_at(&self, seat: usize) -> InspectorV {
        let world = &self.world;
        let i = self.seats[seat].selected_target;
        let v = self.villagers[i];
        let mut lines: Vec<Line> = vec![];
        lines.push(Line { x: 60, y: 330, text: vill_name(i).into(), rgb: (255, 255, 255) });
        if let Some(day) = self.died_on[i] {
            lines.push(Line { x: 60, y: 344, text: format!("Died on day {}", day), rgb: (230, 90, 80) });
            lines.push(Line { x: 60, y: 358, text: "rest in peace".into(), rgb: (150, 150, 165) });
            return InspectorV { fig: None, kid: false, dead: true, died_day: Some(day), lines };
        }
        let org_members = world.get_component::<Organization>(self.seats[seat].org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let occ = desires::occupation_of(world, v).unwrap_or_else(|| "Villager".into());
        lines.push(Line { x: 60, y: 342,
            text: format!("{} \u{00b7} {}", occ, vocation::class_name(desires::class_of(world, v))), rgb: (200, 200, 230) });
        if livelihood::has_role(&self.world, v, TownRole::Herbologist) {
            let n = herbology::remedies_on_hand(world, v);
            lines.push(Line { x: 60, y: 354,
                text: format!("knows the cures \u{00b7} {} remedies", n), rgb: (150, 210, 150) });
        }
        if let Some(c) = world.get_component::<Craft>(v) {
            lines.push(Line { x: 60, y: 354,
                text: format!("\u{1}Craft\u{2}{}{:.0}%", c.skill, c.skill * 100.0), rgb: (0, 0, 0) });
        }
        if let Some(p) = world.get_component::<Purse>(v) {
            lines.push(Line { x: 60, y: 366, text: format!("Gold {:.0}", p.gold), rgb: (240, 210, 120) });
        }
        if let Some(h) = world.get_component::<Hunger>(v) {
            lines.push(Line { x: 60, y: 378,
                text: format!("\u{1}Fed\u{2}{}{}", h.value / 100.0, fullness_word(h.value)), rgb: (0, 0, 0) });
        }
        if let Some(he) = world.get_component::<Health>(v) {
            let note = disease_of(world, v).map(|(name, stage)| format!("{} ({:?})", name, stage))
                .unwrap_or_else(|| format!("{:.0}%", he.condition * 100.0));
            lines.push(Line { x: 60, y: 390, text: format!("\u{1}Health\u{2}{}{}", he.condition, note), rgb: (0, 0, 0) });
        }
        let home = household::home_of(world, v)
            .and_then(|h| world.get_component::<Building>(h)).map(|b| b.name.clone());
        let shop = world.get_component::<Craft>(v).and_then(|c| c.workplace)
            .and_then(|s| world.get_component::<Building>(s)).map(|b| b.name.clone());
        let roost = match (&home, &shop) {
            (Some(h), Some(s)) => format!("{} \u{00b7} {}", h, s),
            (Some(h), None) => h.clone(),
            (None, Some(s)) => format!("no home \u{00b7} {}", s),
            (None, None) => "no home".into(),
        };
        lines.push(Line { x: 18, y: 404, text: roost, rgb: (190, 190, 210) });
        // how well the organizer knows them: the stage word beside the
        // secrets they've told
        let stage = crate::game::organizing::stage_of(world, self.seats[seat].player, v);
        if stage > crate::game::organizing::RelStage::Stranger {
            lines.push(Line { x: 60, y: 428,
                text: crate::game::organizing::stage_label(stage).to_string(), rgb: (170, 200, 160) });
        }
        if let Some(k) = world.get_component::<Known>(v) {
            for (n, r) in k.revealed.iter().take(2).enumerate() {
                let text = match r {
                    desires::Reveal::Want(w) => desires::want_label(*w).to_string(),
                    desires::Reveal::Problem(p) => desires::problem_label(p),
                };
                lines.push(Line { x: 18, y: 416 + n as i32 * 11, text, rgb: (210, 170, 140) });
            }
        }
        if let Some(vil) = world.get_component::<Villager>(v) {
            let nature = world.get_component::<AttachmentHistory>(v)
                .map(|a| nature_word(crate::sim::psychology::infer_style(a)))
                .unwrap_or("just folk");
            // where they stand with the cause: inside it, woken by it,
            // hollowed out by a blow-off, or taken for it
            let fight = crate::game::campaign::campaign_of(world, self.seats[seat].org)
                .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c))
                .map(|c| c.issue.clone());
            let cause = if world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                " \u{00b7} taken"
            } else if world.get_component::<crate::game::actions::Demoralized>(v).is_some() {
                " \u{00b7} hollow"
            } else if fight.as_ref().map(|issue|
                world.get_component::<crate::game::reactions::Sympathized>(v)
                    .map(|s| s.issues.iter().any(|(i, _)| i == issue)).unwrap_or(false))
                .unwrap_or(false) {
                " \u{00b7} woke"
            } else if org_members.contains(&v) { " \u{00b7} with you" } else { "" };
            // what they're about: the day's plan, unless hunger has them
            let bench = world.get_component::<Craft>(v).and_then(|c| c.workplace)
                .and_then(|b| world.get_component::<Building>(b)).map(|b| b.name.clone());
            let doing = (vil.activity == Activity::Idle)
                .then(|| routine::routine_text(world, v, bench.as_deref())).flatten()
                .unwrap_or_else(|| activity_text(vil.activity).to_string());
            lines.push(Line { x: 18, y: 440, text: format!("{} \u{00b7} {}{}", nature, doing, cause), rgb: (200, 160, 160) });
        }
        // the biography the seat knows: years and build are public, but
        // temperament and kin take a bond close enough to have been told
        if let Some(bio) = self.bio_line_at(seat, v) {
            lines.push(Line { x: 18, y: 454, text: bio, rgb: (160, 180, 200) });
        }
        InspectorV { fig: Some(i), kid: world.get_component::<Kid>(v).is_some(), dead: false, died_day: None, lines }
    }

    /// The biography the seat knows of `v`: what they are and what they
    /// carry are plain to see; temperament, kin and origin take a bond
    /// known closely enough to have been told
    fn bio_line_at(&self, seat: usize, v: Entity) -> Option<String> {
        let world = &self.world;
        let mut parts: Vec<String> = vec![];
        if world.get_component::<Kid>(v).is_some() {
            parts.push("a child".into());
        } else if let Some(a) = world.get_component::<crate::game::lifecycle::Age>(v) {
            parts.push(format!("{:.0} winters", a.years));
        }
        if let Some(c) = world.get_component::<villager::Condition>(v) {
            parts.push(c.kind.to_string());
        }
        if crate::game::organizing::stage_of(world, self.seats[seat].player, v)
            >= crate::game::organizing::RelStage::Known {
            if let Some(p) = world.get_component::<crate::game::traits::Personality>(v) {
                let ts: Vec<&str> = p.traits.iter().map(|&t| trait_word(t)).collect();
                if !ts.is_empty() { parts.push(ts.join(", ")); }
            }
            let kin: Vec<String> = self.villagers.iter().copied()
                .filter(|&o| o != v && self.rel_kind(v, o) == Some(crate::game::relationships::RelKind::Family))
                .take(3)
                .filter_map(|o| self.villagers.iter().position(|&x| x == o).map(|i| vill_name(i).to_string()))
                .collect();
            if !kin.is_empty() { parts.push(format!("kin: {}", kin.join(", "))); }
            if world.get_component::<country::Outlander>(v).is_some() {
                parts.push("of the country".into());
            }
        }
        (!parts.is_empty()).then(|| parts.join(" \u{00b7} "))
    }

    /// What an agent sees: itself, its purse, its belly, who's near and
    /// what's under way. Compact - the client decides between observations.
    pub fn agent_obs(&self, seat: usize) -> Option<AgentObs> {
        let who = self.seats.get(seat)?.player;
        let p = self.world.get_component::<Pos>(who)?;
        let gold = self.world.get_component::<Purse>(who).map(|x| x.gold).unwrap_or(0.0);
        let hunger = self.world.get_component::<Hunger>(who).map(|x| x.value).unwrap_or(100.0);
        let busy = if self.travels.iter().any(|t| t.who == who) {
            Some("travelling".to_string())
        } else {
            self.world.get_component::<Villager>(who).map(|v| format!("{:?}", v.activity))
        };
        let mut near: Vec<ObsPerson> = self.villagers.iter().enumerate()
            .filter_map(|(i, v)| {
                if *v == who { return None; }
                let q = self.world.get_component::<Pos>(*v)?;
                let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
                (d < 8.0).then(|| ObsPerson { e: *v, name: vill_name(i).to_string(), dist: d,
                    dead: villager::is_dead(&self.world, *v) })
            }).collect();
        for s in self.seats.iter().filter(|s| !s.agent) {
            if s.player == who { continue; }
            if let Some(q) = self.world.get_component::<Pos>(s.player) {
                let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
                if d < 8.0 { near.push(ObsPerson { e: s.player, name: "a wanderer".into(), dist: d, dead: false }); }
            }
        }
        let near_buildings: Vec<ObsBuilding> = self.building_entities.iter().filter_map(|be| {
            let q = self.world.get_component::<Pos>(*be)?;
            let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
            if d >= 8.0 { return None; }
            let b = self.world.get_component::<Building>(*be)?;
            let kind = match b.kind {
                BuildingKind::Home => "home", BuildingKind::Shop(_) => "shop",
                BuildingKind::Church => "church", BuildingKind::WatchPost => "watch",
                BuildingKind::Mill => "mill", BuildingKind::Granary => "granary",
                BuildingKind::TraderStall => "stall", BuildingKind::Pasture => "pasture",
            BuildingKind::Apiary => "apiary",
            BuildingKind::Cellar => "cellar",
            };
            Some(ObsBuilding { e: *be, name: b.name.clone(), kind: kind.into(), dist: d })
        }).collect();
        let satchel = self.world.get_component::<Satchel>(who)
            .map(|s| s.goods.iter().map(|(g, q)| (g.clone(), *q)).collect()).unwrap_or_default();
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(who).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(who).is_some();
        // the political layer: every banner you march under (membership
        // or leadership - a villager can hold more than one), and what
        // each fights for. "-" marks a seat's placeholder org, not a cause
        let mut orgs: Vec<String> = vec![];
        let mut campaigns: Vec<String> = vec![];
        for e in self.world.query::<Organization>().iter() {
            let Some(o) = self.world.get_component::<Organization>(*e) else { continue };
            if o.name == "-" || !(o.members.contains(&who) || o.leader == who) { continue; }
            orgs.push(o.name.clone());
            if let Some(issue) = crate::game::campaign::campaign_of(&self.world, *e)
                .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
                .map(|c| c.issue.clone()) {
                campaigns.push(issue);
            }
        }
        // the fire on the wind: burning tiles close enough to matter
        let burning: Vec<(f32, f32)> = self.fire.burning.keys()
            .filter(|&&(x, y)| ((x as f32 - p.x).powi(2) + (y as f32 - p.y).powi(2)).sqrt() < 12.0)
            .map(|&(x, y)| (x as f32, y as f32)).collect();
        // what they know of themself: their trade, home and bench, the want
        // list - their own only
        let craft = self.world.get_component::<Craft>(who);
        let bench = craft.and_then(|c| c.workplace);
        let mut bench_stock: Vec<(String, u32)> = bench.and_then(|b| self.world.get_component::<Building>(b))
            .map(|b| b.stock.iter().filter(|(_, q)| **q > 0).map(|(g, q)| (g.clone(), *q)).collect()).unwrap_or_default();
        bench_stock.sort();
        let wants: Vec<String> = self.world.get_component::<InnerLife>(who)
            .map(|l| l.wants.iter().filter(|w| !desires::is_satisfied(&self.world, who, **w, self.tick))
                .map(|w| format!("{:?}", w)).collect()).unwrap_or_default();
        let me = ObsSelf {
            trade: craft.map(|c| vocation::vocation_name(c.vocation).to_string()),
            level: craft.map(|c| format!("{:?}", vocation::level_of(c.skill))),
            home: household::home_of(&self.world, who),
            bench,
            bench_stock,
            needs: livelihood::needs_input(&self.world, who).map(String::from),
            wants,
        };
        Some(AgentObs {
            tick: self.tick, you: who, pos: (p.x, p.y), gold, hunger, busy,
            near, near_buildings, satchel,
            walking: self.travels.iter().any(|t| t.who == who),
            held, orgs, campaigns, burning, me,
        })
    }
}

/// A trait in plain words, for the biography line
fn trait_word(t: crate::game::traits::Trait) -> &'static str {
    use crate::game::traits::Trait::*;
    match t {
        Lazy => "lazy", Ambitious => "ambitious", Kind => "kind", Stingy => "stingy",
        Pious => "pious", Hotheaded => "hot-headed", Gregarious => "gregarious", Proud => "proud",
    }
}
