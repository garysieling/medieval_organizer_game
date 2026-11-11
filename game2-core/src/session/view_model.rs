//! The public projections of the sim: a seat's UiModel (frame, panel, inspector) and an agent's AgentObs. What a seat may know is decided here.

use crate::session::*;

/// Gossip lines shown in the log at once; older gossip is hidden so it
/// can't push real events (illness, joins, deaths, guilds) out of view
pub(in crate::session) const LOG_GOSSIP_SHOWN: usize = 2;
/// The newest few headlines pin to the log's top: enough that a quiet
/// death outranks a busy skep-day, not so many that yesterday's news
/// buries today's warning
pub(in crate::session) const LOG_PINNED_SHOWN: usize = 4;

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
    entry.contains(" is now a ") || entry.contains(" is now an ")
        || entry.contains("has come to town")
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
pub(in crate::session) fn visible_log(log: &[String]) -> Vec<usize> {
    let (mut gossip, mut weather, mut town) = (0, 0, 0);
    let filtered: Vec<(usize, &String)> = log.iter().enumerate().rev().filter(|(_, e)| {
        if is_gossip(e) { gossip += 1; return gossip <= LOG_GOSSIP_SHOWN; }
        if is_weather_note(e) { weather += 1; return weather <= LOG_WEATHER_SHOWN; }
        if is_town_note(e) { town += 1; return town <= LOG_TOWN_SHOWN; }
        true
    }).collect();
    // stable partition: the newest few important entries pin to the top,
    // then the rest - a quiet death still outranks a busy skep-day, but a
    // wall of old headlines can't bury today's warning
    let mut imp_rank = 0usize;
    let (important, rest): (Vec<_>, Vec<_>) = filtered.into_iter()
        .partition(|(_, e)| {
            if !is_important(e) { return false; }
            imp_rank += 1;
            imp_rank <= LOG_PINNED_SHOWN
        });
    important.into_iter().chain(rest).map(|(i, _)| i).collect()
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
        // who's carrying which board while a protest is on: folk actually
        // holding a rally slot today, not just names on the call
        let signs: crate::engine::det::HashMap<Entity, u8> = self.protests.iter()
            .filter(|p| self.tick < p.until)
            .flat_map(|p| p.folk.iter().copied().filter(|&v| {
                world.get_component::<routine::Routine>(v)
                    .map(|r| r.mode == routine::Mode::Protesting && r.day == self.tick)
                    .unwrap_or(false)
            }).map(move |v| (v, p.sign)))
            .collect();
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
            let dead = villager::is_dead(world, *v);
            people.push(PersonV {
                e: *v, kind: PKind::Villager(i), pos: (p.x, p.y),
                label: Some(format!("{}{}{}", vill_name(i), role, cause_mark)),
                find: find_key(&find),
                dead,
                epitaph: dead.then(|| {
                    let day = self.died_on.get(i).copied().flatten().map(|d| format!("died day {}", d))
                        .unwrap_or_else(|| "the dead".into());
                    let cause = world.get_component::<crate::game::lifecycle::Vitality>(*v)
                        .and_then(|vit| vit.cause_of_death.as_deref())
                        .map(|c| format!(", of {}", c)).unwrap_or_default();
                    format!("{}{}", day, cause)
                }),
                buried: world.get_component::<villager::Buried>(*v).is_some(),
                kid,
                selected: self.seats[seat].selected_target == i && self.seats[seat].picked,
                in_reach: in_reach.contains(v),
                hungry: world.get_component::<Hunger>(*v).map(|h| h.value < 40.0).unwrap_or(false),
                sick: is_sick(world, *v),
                act_mark: match world.get_component::<Villager>(*v).map(|v| v.activity) {
                    Some(Activity::SeekingFood) => 1, Some(Activity::Idle) | None => 0, _ => 2 },
                bond: false, look: 0, sign: signs.get(v).copied(),
                mounted: world.get_component::<villager::Mounted>(*v).is_some(),
            });
        }
        for (i, v) in self.rivals.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: *v, kind: PKind::Rival(i), pos: (p.x, p.y),
                    label: Some(RIVAL_NAMES[i].to_string()), find: find_key(RIVAL_NAMES[i]),
                    dead: false, kid: false,
                    selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
            }
        }
        let (px, py) = self.player_pos_at(seat);
        let avatar = world.get_component::<crate::game::player::Avatar>(self.seats[seat].player);
        people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: self.seats[seat].player, kind: PKind::Player, pos: (px, py),
            label: None, find: world.get_component::<PlayerCharacter>(self.seats[seat].player)
                .map(|p| find_key(&p.name)).unwrap_or_default(),
            dead: false, kid: avatar.map(|a| a.kid).unwrap_or(false),
            selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false,
            look: avatar.map(|a| a.look as u8).unwrap_or(0) });
        for (i, (x, y)) in self.guards.iter().copied()
            .chain(self.posted.iter().map(|p| (p.1, p.2))).enumerate() {
            people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: 0, kind: PKind::Guard(i), pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
        }
        people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: 0, kind: PKind::Priest, pos: self.priest,
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
        if let Some((tx, ty)) = self.traveller_pos() {
            people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: 0, kind: PKind::Traveller, pos: (tx, ty),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
        }
        if let Some(d) = &self.doctor {
            if let Some(p) = self.world.get_component::<Pos>(d.e).copied() {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: d.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the plague doctor".into()), dead: false, kid: false, selected: false, in_reach: false,
                    hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
            }
        }
        if let Some(m) = &self.minstrel {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the minstrel".into()), find: find_key("the minstrel"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
            }
        }
        if let Some(m) = &self.pedlar {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the pedlar".into()), find: find_key("the pedlar"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
            }
        }
        for c in &self.couriers {
            if let Some(p) = world.get_component::<Pos>(c.e) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: c.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the courier".into()), find: find_key("the courier"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
            }
        }
        if let Some(c) = &self.circus {
            if let Some(p) = world.get_component::<Pos>(c.e) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: c.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the circus".into()), find: find_key("the circus"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
            }
            // the troupe around him: no labels - they're the camp's
            // background, not its news
            for &m in &c.members {
                if let Some(p) = world.get_component::<Pos>(m) {
                    people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: m, kind: PKind::Traveller, pos: (p.x, p.y),
                        label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
                }
            }
        }
        if let Some(b) = &self.ink_boiler {
            if let Some(p) = world.get_component::<Pos>(b.e) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: b.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the ink-boiler".into()), find: find_key("the ink-boiler"), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0 });
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
            people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e, kind, pos: (p.x, p.y),
                label: Some(spec.name.into()), find: find_key(spec.name), dead: false, kid: false, selected: false,
                in_reach: false, hungry: false, sick: false, act_mark: 0,
                bond: world.get_component::<Pet>(e).map(|p| p.bond_strength >= 0.5).unwrap_or(false), look: 0 });
        }
        for &(x, y) in &self.gulls {
            people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: 0, kind: PKind::Gull, pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
        }
        let hen_t = (self.frame_n / 12) as f32;
        people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: 0, kind: PKind::Chicken,
            pos: (self.chicken_at.0 + hen_t.cos() * 0.5, self.chicken_at.1 + (hen_t * 1.7).sin() * 0.4),
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
        for (pet_e, species, (ox, oy)) in &self.pets {
            if let Some(pet) = world.get_component::<Pet>(*pet_e) {
                // a tamed animal has feet of its own - it draws where it
                // stands; a hearth pet stays tucked by its owner's side
                let at = world.get_component::<Pos>(*pet_e).map(|p| (p.x, p.y))
                    .or_else(|| world.get_component::<Pos>(pet.owner.unwrap_or(self.villagers[0]))
                        .map(|p| (p.x + ox, p.y + oy)));
                if let Some((x, y)) = at {
                    people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: *pet_e, kind: PKind::Pet(*species),
                        pos: (x, y), label: None, dead: false, kid: false,
                        selected: false, in_reach: false, hungry: false, sick: false,
                        act_mark: 0, bond: pet.bond_strength > 0.5, look: 0, find: String::new() });
                }
            }
        }
        for &a in &self.flock {
            if let (Some(l), Some(p)) = (world.get_component::<Livestock>(a), world.get_component::<Pos>(a)) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: a, kind: if l.kind == LivestockKind::Sheep { PKind::Sheep } else { PKind::Goat },
                    pos: (p.x, p.y), label: None, dead: false, kid: false, selected: false,
                    in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
            }
        }
        for &a in &self.hens {
            if let Some(p) = world.get_component::<Pos>(a) {
                people.push(PersonV { sign: None, mounted: false, epitaph: None, buried: false, e: a, kind: PKind::Chicken, pos: (p.x, p.y), label: None,
                    dead: false, kid: false, selected: false, in_reach: false,
                    hungry: false, sick: false, act_mark: 0, bond: false, look: 0, find: String::new() });
            }
        }
        people.sort_by(|a, b| (a.pos.0 + a.pos.1).partial_cmp(&(b.pos.0 + b.pos.1)).unwrap());

        // buildings and building_entities only line up for the founding
        // village: country sprites and grown roofs sit at different
        // indices, so each sprite asks which entity stands on its tile
        let mut bldg_at: crate::engine::det::HashMap<(i32, i32), usize> = crate::engine::det::HashMap::default();
        for (i, &e) in self.building_entities.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(e) {
                bldg_at.insert((p.x.round() as i32, p.y.round() as i32), i);
            }
        }
        let bldg_idx = |x: f32, y: f32| bldg_at.get(&(x.round() as i32, y.round() as i32)).copied();
        let risen = |i: usize| self.works.iter().find(|w| w.building == i).map(|w| w.risen()).unwrap_or(1.0);
        let buildings: Vec<BldgV> = self.buildings.iter().enumerate()
            .map(|(i, (x, y, s))| {
                let ei = bldg_idx(*x, *y);
                BldgV { x: *x, y: *y, sprite: *s, risen: risen(i),
                    derelict: ei.map(|e| self.derelict.contains(&e) || self.ruined.contains(&e)).unwrap_or(false),
                    cutaway: ei.map(|e| self.cutaways.contains(&e)).unwrap_or(false), smoke: i < 3 }
            })
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
            // the shadow only pulls when no live coordinator holds the
            // seat - and the name's only theirs to claim once the room's
            // observably followed them
            let o = world.get_component::<Organization>(c.org).unwrap();
            let coordinator = crate::game::campaign::seat_holder(world, e, Role::TurnoutCoordinator)
                .filter(|&h| !crate::game::dungeon::unavailable(world, h)
                    && (h == o.leader || o.members.contains(&h)));
            let shadow = if coordinator.is_none() {
                let pulled = crate::game::campaign::shadow_elite(world, e);
                let driven = mtg.and_then(|m| m.driven_by);
                if pulled.is_some() && pulled == driven { pulled.map(|s| name_of(s)) } else { None }
            } else { None };
            // the cause's posture: the arc's own facts in a word, so the
            // panel says more than the vote's date
            let facts = crate::game::campaign_arc::arc_facts(world, e, self.tick);
            let state = [
                ("arc:won", "won through"), ("arc:lost", "their demand's refused"),
                ("arc:delayed", "their answer's put off"), ("arc:partial", "won only part"),
                ("arc:expired", "the grant's run out"), ("arc:diluted", "the grant went hollow"),
                ("arc:leader_taken", "their leader's held"),
                ("arc:crackdown", "crackdown's on"), ("arc:countered", "the streets are banned"),
                ("arc:promise_broken", "their word's broken"), ("arc:split", "the room's split"),
                ("arc:fatigue", "the cause's weary"), ("arc:consent", "the room consents"),
                ("arc:strong", "runs strong"), ("arc:weak", "runs thin"),
            ].iter().find(|(f, _)| facts.has(f)).map(|(_, w)| w.to_string()).unwrap_or_default();
            // the assembly's standing: the agenda posted, or the
            // pledges the room's keeping count of - a broken word
            // shows till the cap's drained it
            let assembly = world.get_component::<crate::game::campaign::Assembly>(e)
                .map(|a| {
                    if a.announced.is_some() { return "assembly called - the agenda's posted".to_string(); }
                    let broken = a.pledges.iter().filter(|p| p.kept == Some(false)).count();
                    let open = a.pledges.iter().filter(|p| p.kept.is_none()).count();
                    let kept = a.pledges.iter().filter(|p| p.kept == Some(true)).count();
                    let mut parts = vec![];
                    if broken > 0 { parts.push(format!("{} pledge{} broke", broken, if broken == 1 { "" } else { "s" })); }
                    if open > 0 { parts.push(format!("{} pledge{} stand{}", open, if open == 1 { "" } else { "s" }, if open == 1 { "s" } else { "" })); }
                    if kept > 0 && broken == 0 { parts.push(format!("{} pledge{} kept", kept, if kept == 1 { "" } else { "s" })); }
                    parts.join(" - ")
                }).unwrap_or_default();
            CampaignV {
                issue: c.issue.clone(),
                shadow,
                state,
                framing: match c.framing { Framing::SharedGrievance => "shared grievance",
                    Framing::CommonFoe => "common foe", Framing::MoralDuty => "moral duty" }.to_string(),
                dissenters: c.dissenters.len(), seats,
                resolves: mtg.map(|m| m.resolves).unwrap_or(0),
                due: mtg.map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false),
                cadence_days: mtg.map(|m| m.cadence_days).unwrap_or(0),
                assembly,
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
        // the powers' standing words on the org - short enough that
        // all three still fit the one line the panel gives them
        let org_status = {
            let mut v = vec![];
            if let Some(gr) = world.get_component::<crate::game::reactions::LordGrant>(self.seats[seat].org) {
                if self.tick <= gr.until {
                    let word = if gr.hollow { "hollow grant" }
                        else if gr.partial { "part grant" } else { "lord's grant" };
                    v.push(format!("{}: {}", word, gr.issue));
                }
            }
            if let Some(g) = world.get_component::<crate::game::reactions::GuardsPosted>(self.seats[seat].org) {
                v.push(format!("{} knights posted", g.knights));
            }
            if let Some(b) = world.get_component::<crate::game::reactions::AssemblyBan>(self.seats[seat].org) {
                v.push(format!("assembly ban >{}", b.cap));
            }
            v
        };
        // the org's real power vs the seat's craft: the muscle's
        // capacity, the turnout skill, and whether it's gone soft
        // a word, not the gauge: the muscle's strength and the seat's
        // craft in public terms - the exact numbers stay the probe's
        let power = world.get_component::<crate::sim::organized_power::OrganizedPower>(self.seats[seat].org).map(|p| {
            let turnout = world.get_component::<crate::game::org_skills::Organizing>(me)
                .map(|o| o.turnout).unwrap_or(0.0);
            let muscle = if p.capacity >= 0.3 { "a muscle to count on" }
                else if p.capacity > 0.0 { "a thin muscle" } else { "no muscle left" };
            let craft = if turnout >= 0.5 { "a practiced call" }
                else if turnout > 0.0 { "a rough call" } else { "no call to bring them" };
            let temper = if p.idle_ticks == 0 { "fresh" }
                else if p.idle_ticks < 7 { "settled" } else { "going soft" };
            format!("{} \u{b7} {} \u{b7} {}", muscle, craft, temper)
        });
        // each figure's warmth for the seat, read as hearts - the map
        // floats them over heads, the inspector reads the same number
        // off the bond; only warmth worth showing makes the list
        let seat_player = self.seats[seat].player;
        let bonds: Vec<(Entity, f32)> = self.villagers.iter().chain(self.rivals.iter())
            .filter_map(|&v| {
                crate::game::relationships::find_relationship(world, seat_player, v)
                    .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
                    .map(|r| (v, crate::game::relationships::hearts_of(r.strength)))
                    .filter(|&(_, h)| h > 0.0)
            }).collect();
        // the seat's own roster - the map marks a member's name where
        // their hearts show; the player's dart already marks them
        let org_members: Vec<Entity> = world.get_component::<Organization>(self.seats[seat].org)
            .map(|o| o.members.iter().copied().filter(|&m| m != seat_player).collect())
            .unwrap_or_default();
        UiModel {
            player_pose: (self.seats[seat].facing, self.seats[seat].walking),
            // the courier (entity 0 in people) faces the way it rides
            poses: self.villager_poses.iter().map(|(&e, &(_, f, w))| (e, f, w))
                .chain(self.traveller_pos().zip(self.traveller_pos_at(self.frame_n.wrapping_sub(1)))
                    .map(|(now, was)| (0, facing_on_screen(now.0 - was.0, now.1 - was.1), true))).collect(),
            tick: self.tick, frame: self.frame_n, focus: self.seats[seat].focus,
            player_event: self.player_event_at(seat),
            map: if with_map { self.map.clone() } else { crate::game::tilemap::TileMap::default() }, town_wall: self.town_wall,
            season, precip: self.precip, rainbow_from: self.rainbow_from,
            food_yield: world.get_component::<climate::Climate>(self.climate)
                .map(|c| climate::climate_multiplier(c, "food")).unwrap_or(1.0),
            harsh_winter: self.harsh_winter,
            paused: self.paused,
            stones: self.stones.clone(), buildings, people, bonds, org_members,
            player_satchel: self.world.get_component::<Satchel>(seat_player)
                .map(|s| { let mut g: Vec<_> = s.goods.iter().filter(|(_, &n)| n > 0)
                    .map(|(k, &v)| (k.clone(), v)).collect(); g.sort(); g })
                .unwrap_or_default(),
            player_hunger: self.world.get_component::<Hunger>(seat_player)
                .map(|h| h.value).unwrap_or(100.0),
            bubbles,
            castle_keeps: self.map.castle_keeps.iter().map(|&(x, y)| (x as f32, y as f32)).collect(),
            settlement_houses: self.map.settlement_houses.iter().map(|&(x, y)|
                (x as f32, y as f32, BSprite::House((noise(x, y, 41) * 97.0) as usize))).collect(),
            toasts: self.toasts.iter().filter(|(_, until, _, _)| self.tick < *until)
                .map(|(m, _, at, born)| ToastV { text: m.clone(), at: *at, born: Some(*born) }).collect(),
            // only the lines the window would show - cloning the whole
            // history every frame was the model's biggest copy; the meta
            // rides along so a click can jump to where it happened (#645)
            log: visible_log(&self.log).into_iter().map(|i| {
                let meta = self.log.meta(i);
                LogEntryV { text: self.log[i].clone(), at: meta.at, who: meta.who }
            }).collect(),
            log_dropped: self.log_dropped,
            sel: Selection { target: self.seats[seat].selected_target, picked: self.seats[seat].picked, building: self.seats[seat].selected_building },
            reach: in_reach.iter().filter_map(|v| self.villagers.iter().position(|x| x == v)).collect(),
            reach_target: self.reach_target_at(seat),
            talk_target: self.talk_target_at(seat),
            blocked, ctx: self.seats[seat].ctx_actions.clone(),
            ctx_blocked: self.seats[seat].ctx_actions.iter()
                .map(|a| self.ctx_blocker_at(seat, *a)).collect(),
            ctx_labels: self.seats[seat].ctx_actions.iter()
                .map(|a| self.ctx_label_at(seat, *a)).collect(),
            ctx_hint: self.seats[seat].ctx_actions.iter()
                .map(|a| self.ctx_hint_at(seat, *a)).collect(),
            inspector,
            panel: PanelV {
                name: my_name, gold: my_gold,
                org: org.name.clone(), leader: Some(name_of(org.leader)),
                org_status, power,
                members, issues, guilds, food, campaign,
                quest: self.seats[seat].quest.as_ref().map(|q| self.quest_panel_line(q)),
                mystery: self.mystery_panel_line(seat),
                social, meeting,
            },
            mill: Some(self.mill),
            herbs: self.herb_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            mush: self.mush_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            stolen: self.rebel_stores.iter().filter(|s| self.tick < s.until)
                .map(|s| (s.tile.0 as f32, s.tile.1 as f32)).collect(),
            skeps: self.hives.iter().filter_map(|&h| world.get_component::<Pos>(h)
                .map(|p| (p.x, p.y))).collect(),
            swarm: self.swarm.and_then(|(w, _)| world.get_component::<Pos>(w).map(|p| (p.x, p.y))),
            market: world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y)),
            feast: self.villagers.iter().any(|&v| world.get_component::<routine::Routine>(v)
                .is_some_and(|r| r.mode == routine::Mode::Feasting && r.day == self.tick)),
            stalls: self.stalls_standing(),
            trails: self.trails.iter().filter(|(_, t)| t.strength > 0.3)
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
            cave_zone: self.cave_zone.map(|((ax, ay), (bx, by))|
                ((ax as f32, ay as f32), (bx as f32, by as f32))),
            grave: self.martyr_grave.map(|(x, y)| (x as f32, y as f32)),
            graveyard: self.graveyard.as_ref().map(|g| g.slots.clone()).unwrap_or_default(),
            lychgate: self.graveyard.as_ref().and_then(|g| g.gate),
            stocks: self.stocks_tile,
            well: self.well_tile,
            hall: self.hall_tile,
            village_gaol: self.village_gaol,
            castle_gaol: self.castle_gaol,
            flood: self.flood.cells.iter().map(|(&(x, y), &b)| (x, y, b)).collect(),
            notices: self.notices.iter().filter(|(_, _, u)| self.tick < *u)
                .map(|(k, t, _)| (k.clone(), t.clone())).collect(),
            music: self.world.get_component::<crate::game::music::MusicState>(self.music)
                .map(|st| (format!("{:?}", st.mood),
                    st.layers.iter().map(|l| (l.name.clone(), l.intensity)).collect())),
            sfx_v: self.sfx.iter()
                .map(|s|  (s.kind.to_string(), s.x.round() as i32, s.y.round() as i32, s.intensity)).collect(),
            boats: self.boats.clone(),
            notice_panel: self.seats[seat].notice_panel,
            postings: self.building_entities.iter().filter_map(|&be|
                self.world.get_component::<Building>(be)
                    .and_then(|b| b.flyer.as_ref().map(|f|
                        self.world.get_component::<Pos>(be)
                            .map(|p| (p.x.round() as i32, p.y.round() as i32, f.clone())))))
                .flatten().collect(),
            monuments: self.monuments.iter().filter(|m| m.stood_on.is_some())
                .map(|m| (m.pos.0, m.pos.1,
                    if m.kind == MonumentKind::Circle { 1u8 } else { 0u8 },
                    if m.kind == MonumentKind::Circle { m.title.clone() }
                    else { format!("{} - {}",
                        if m.honoree == usize::MAX { "the one who moved on" } else { vill_name(m.honoree) },
                        m.title) })).collect(),
            sites: self.sites.iter().map(|s| {
                // the keeper: who holds the keep's first bed, when the
                // site has one - the castle's lord, a town's first house
                let keeper = s.homes.first()
                    .and_then(|&h| world.get_component::<Building>(h))
                    .and_then(|b| b.occupants.first().copied())
                    .and_then(|o| self.villagers.iter().position(|v| *v == o))
                    .map(|i| vill_name(i).to_string());
                // the head, styled: "Earl Godric" - the site's lord by
                // the title their house holds
                let head = s.lord
                    .and_then(|l| self.villagers.iter().position(|v| *v == l))
                    .map(|i| format!("{} {}", s.title.unwrap_or("the head"), vill_name(i)));
                SiteV { x: s.centre.0 as f32, y: s.centre.1 as f32,
                    name: s.name.to_string(), kind: s.kind.clone(), keeper,
                    head, treasury: s.treasury }
            }).collect(),
            talk: self.talk_panel_at(seat),
            convo: self.convo_panel_at(seat),
            debug_lines: vec![],
            sfx: self.sfx.iter().map(|c| c.kind.to_string()).collect(),
            snowballs: self.snowballs.iter().map(|s| s.pos_at(self.frame_n)).collect(),
            tech: self.tech.clone(),
            flyers: self.flyers.clone(),
            // the freight on the roads: whoever's driving a cart, a
            // levy, a remit, a tribute or a gift train pulls a wagon
            trains: self.travels.iter()
                .filter(|t| matches!(t.goal, Goal::Cart(_) | Goal::Levy(..)
                    | Goal::Deliver(_) | Goal::Remit(..) | Goal::Owe(..)
                    | Goal::Gift(..)))
                .filter_map(|t| world.get_component::<Pos>(t.who)
                    .map(|p| (p.x, p.y))).collect(),
        }
    }

    /// The civic event this seat's player is part of, for the camera:
    /// a march the seat called (the column's live head) or a house
    /// meeting it called (its venue). A march outranks a meeting when
    /// both stand; anything else civic plugs into the same field
    fn player_event_at(&self, seat: usize) -> Option<EventFocusV> {
        if let Some(f) = &self.formation {
            if f.seat == seat && f.kind == formation::FormationKind::March {
                if let Some(h) = f.members.first()
                    .and_then(|&m| self.world.get_component::<Pos>(m)) {
                    return Some(EventFocusV {
                        epoch: f.born * 4, kind: 0,
                        pos: (h.x, h.y), extent: 18.0, moving: true });
                }
            }
        }
        if let Some(m) = &self.meeting {
            if m.seat == seat {
                return Some(EventFocusV {
                    epoch: m.born * 4 + 1, kind: 1,
                    pos: (m.venue.0 as f32, m.venue.1 as f32),
                    extent: 6.0, moving: false });
            }
        }
        None
    }

    /// The inspector card for the building this seat clicked
    pub(in crate::session) fn building_inspector_at(&self, seat: usize) -> Option<InspectorV> {
        let bi = self.seats[seat].selected_building?;
        let (bx, by) = { let b = self.buildings.get(bi)?; (b.0, b.1) };
        // the sprite's index means nothing among the entities: country
        // sprites and grown roofs sit at different indices, so the card
        // shows the building standing on the sprite's own tile
        let be = self.building_entities.iter().copied().find(|&e| {
            self.world.get_component::<Pos>(e)
                .map(|p| (p.x - bx).abs() < 0.75 && (p.y - by).abs() < 0.75)
                .unwrap_or(false)
        })?;
        let world = &self.world;
        let b = world.get_component::<Building>(be)?;
        let mut lines: Vec<Line> = vec![];
        // the same page as a person's card: each row takes only the room
        // it needs, long rows wrapped inside the card's edge
        let mut y = 326;
        let kind_word = match b.kind {
            BuildingKind::Home => "Home",
            BuildingKind::Shop(_) => "Workshop",
            BuildingKind::Church => "Church",
            BuildingKind::Schoolhouse => "School",
            BuildingKind::PostOffice => "Post",
            BuildingKind::WatchPost => "Watch",
            BuildingKind::Mill => "Mill",
            BuildingKind::Granary => "Granary",
            BuildingKind::TraderStall => "Trader's stall",
            BuildingKind::Pasture => "Pasture",
            BuildingKind::Apiary => "Apiary",
            BuildingKind::Cellar => "Cheese cellar",
            BuildingKind::Dungeon => "The dungeon",
            BuildingKind::Plantation => "The plantation",
        };
        lines.push(Line { x: 18, y, text: b.name.clone(), rgb: (255, 255, 255) });
        // the fire's near rides the kind word: flames close enough that
        // this roof could carry them home
        let near_fire = world.get_component::<Pos>(be)
            .map(|p| self.fire.burns_near(p.x as i32, p.y as i32, 4)).unwrap_or(false);
        y += 14;
        lines.push(Line { x: 18, y, rgb: (200, 200, 230),
            text: format!("{}{}", kind_word, if near_fire { " \u{00b7} the fire's near" } else { "" }) });
        let worker = b.worker.and_then(|w| self.villagers.iter().position(|v| *v == w))
            .map(|i| vill_name(i).to_string())
            .or_else(|| b.worker.map(|_| "you".to_string()));
        let who = worker.map(|w| format!("worked by {}", w)).unwrap_or_else(|| "stands empty".to_string());
        let col = if household::derelict(b) { (230, 140, 90) } else { (190, 190, 210) };
        y += 12;
        lines.push(Line { x: 18, y, text: who, rgb: col });
        if let Some(o) = b.overseer {
            let on = self.villagers.iter().position(|v| *v == o).map(|i| vill_name(i)).unwrap_or("someone");
            y += 12;
            lines.push(Line { x: 18, y, text: format!("overseer: {}", on), rgb: (200, 160, 120) });
        }
        if b.kind == BuildingKind::Home {
            let names: Vec<&str> = b.occupants.iter()
                .filter_map(|o| self.villagers.iter().position(|v| *v == *o).map(|i| vill_name(i))
                    .or_else(|| self.rivals.iter().position(|r| *r == *o).map(|i| RIVAL_NAMES[i])))
                .collect();
            // a home's other faces: rooms to let when it stands empty,
            // or the bench someone's laid up in it
            let empty = b.occupants.is_empty() && !self.ruined.contains(&bi);
            let works_here = self.villagers.iter().any(|&v| {
                match world.get_component::<Craft>(v) {
                    Some(c) => match c.workplace {
                        Some(w) => w == be,
                        // no bench of their own - a cottage trade works
                        // its own door, like plan_bench falls back to home
                        None => household::home_of(&self.world, v) == Some(be),
                    },
                    None => false,
                }
            });
            let also = match (empty, works_here) {
                (true, _) => " \u{00b7} to let",
                (false, true) => " \u{00b7} a workshop too",
                (false, false) => "",
            };
            card_prop(&mut lines, &mut y, "Lives here",
                &format!("{}{}", if names.is_empty() { "no one".into() } else { names.join(", ") }, also),
                (190, 190, 210));
        }
        let offers = household::on_offer_at(&self.world, b);
        let stock_line = offers.iter().take(3).map(|(g, q, _)| format!("{}{} {}",
            g, if goods::tier(g) == goods::Tier::Luxury { "*" } else { "" }, q))
            .collect::<Vec<_>>().join(" \u{00b7} ");
        y += 12;
        for w in wrap_text(&if stock_line.is_empty() { "stock: empty".into() } else { format!("stock: {}", stock_line) },
            card_chars(18)) {
            lines.push(Line { x: 18, y, text: w, rgb: (180, 190, 210) });
            y += CARD_ROW;
        }
        if let Some((g, q, pr)) = offers.first() {
            // the street reads a price against the ordinary: scarce
            // shelves ask dear, a glut sells cheap
            let base = goods::worth(g);
            let lean = if *pr > base * 1.1 { " \u{00b7} dear" }
                else if *pr < base * 0.9 { " \u{00b7} cheap" } else { "" };
            // a luxury reads so - the street knows silk from salt
            let kind = if goods::tier(g) == goods::Tier::Luxury { " \u{00b7} a luxury (* = luxury)" } else { "" };
            for w in wrap_text(&format!("for offer: {} {} @ {:.0}g{}{}", q, g, pr, lean, kind), card_chars(18)) {
                lines.push(Line { x: 18, y, text: w, rgb: (150, 210, 150) });
                y += CARD_ROW;
            }
        }
        // the chain the shop worked today: what went in and what came out
        if let Some((_, flow)) = self.shop_flow.get(&be).filter(|(d, _)| *d == self.tick) {
            let txt = flow.iter().map(|(g, n)| format!("{}{} {}", if *n < 0 { "" } else { "+" }, n, g))
                .collect::<Vec<_>>().join("  ");
            if !txt.is_empty() {
                y += 12;
                lines.push(Line { x: 18, y, text: format!("today: {}", txt), rgb: (200, 190, 160) });
            }
        }
        // the printer's bench is the village's research desk: what it's
        // puzzling out now, and how far along
        if matches!(b.kind, BuildingKind::Shop(Vocation::Printmaker)) {
            if let Some(t) = self.tech.iter()
                .filter(|t| !t.discovered && crate::game::tech_tree::can_research(&self.tech, &t.name))
                .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap()) {
                y += 12;
                lines.push(Line { x: 18, y,
                    text: format!("learning: {} ({:.0}/{:.0})", t.name, t.progress, t.cost),
                    rgb: (170, 190, 230) });
            }
        }
        // the cellar's character and the rack: how many wheels down,
        // and the oldest one's age - the number the monger pays for
        if b.kind == BuildingKind::Cellar {
            let n = self.cheeses.len();
            let oldest = self.cheeses.first().map(|t| self.tick - t).unwrap_or(0);
            y += 12;
            lines.push(Line { x: 18, y,
                text: if n == 0 { format!("{} - empty", self.cellar_trait) }
                    else { format!("{} - {} wheels, oldest {}d", self.cellar_trait, n, oldest) },
                rgb: (230, 200, 140) });
        }
        // a home that keeps hens says so - the roost's count with it
        if self.hen_houses.contains(&be) {
            let n = livestock::flock(&world, be).len();
            y += 12;
            lines.push(Line { x: 18, y,
                text: if n == 0 { "roost empty".into() } else { format!("keeps hens - {} on the roost", n) },
                rgb: (220, 180, 120) });
        }
        // the tavern's register: who kept the table's company these
        // last nights - the keeper's till knows the thirsty by name
        if Some(be) == self.tavern {
            // latest nights first, a patron only once
            let mut seen = crate::engine::det::HashSet::default();
            let drank: Vec<String> = self.tavern_table.iter().rev()
                .filter(|(_, d)| self.tick.saturating_sub(*d) < 30)
                .filter(|(w, _)| seen.insert(*w))
                .map(|(w, _)| self.villagers.iter().position(|v| *v == *w)
                    .map(|i| vill_name(i).to_string()).unwrap_or_else(|| "someone".into()))
                .collect();
            // the card's a fixed width: the last two nights' company,
            // then a count
            let company = match drank.len() {
                0 => String::new(),
                1..=2 => format!(" - {}", drank.join(", ")),
                n => format!(" - {}, +{}", drank[..2].join(", "), n - 2),
            };
            y += 12;
            for w in wrap_text(&format!("the table's company{}", company), card_chars(18)) {
                lines.push(Line { x: 18, y, text: w, rgb: (220, 180, 140) });
                y += CARD_ROW;
            }
        }
        // the herb-wife's tally, and who's called at her hut lately
        if Some(be) == self.herb_hut {
            let callers: Vec<String> = self.remedy_visits.iter()
                .filter(|(_, d)| self.tick - d <= 15)
                .map(|(w, _)| self.villagers.iter().position(|v| *v == *w)
                    .map(|i| vill_name(i).to_string()).unwrap_or_else(|| "someone".into()))
                .collect();
            // the card's a fixed width: the first two callers, then a count
            let late = match callers.len() {
                0 => String::new(),
                1..=2 => format!(" - lately: {}", callers.join(", ")),
                n => format!(" - lately: {}, +{}", callers[..2].join(", "), n - 2),
            };
            y += 12;
            for w in wrap_text(&format!("her tally: {} cures{}", self.cured, late), card_chars(18)) {
                lines.push(Line { x: 18, y, text: w, rgb: (190, 210, 150) });
                y += CARD_ROW;
            }
        }
        Some(InspectorV { fig: None, kid: false, dead: false, died_day: None, hearts: None, lines })
    }

    /// Every live feud `v` is party to, burning ones first: who it's
    /// with and whether it burns
    fn feuds_of(&self, v: Entity) -> Vec<(Entity, bool)> {
        let mut out: Vec<(Entity, bool)> = self.world.query::<crate::game::feuds::Feud>().into_iter()
            .filter_map(|e| {
                let f = self.world.get_component::<crate::game::feuds::Feud>(e)?;
                if f.status == crate::game::feuds::FeudStatus::Resolved { return None; }
                let hot = f.status == crate::game::feuds::FeudStatus::Burning;
                if f.offender == v { Some((f.aggrieved, hot)) }
                else if f.aggrieved == v { Some((f.offender, hot)) }
                else { None }
            }).collect();
        out.sort_by_key(|(_, hot)| !hot);
        out
    }

    /// The inspector card for the selected villager: a properties page.
    /// The name and the seat's read of them sit beside the portrait up
    /// top; the rest is labelled rows and bulleted sections, each row
    /// taking only the room it needs, so a long telling wraps inside the
    /// card rather than running off across the map
    pub(in crate::session) fn villager_inspector_at(&self, seat: usize) -> InspectorV {
        let world = &self.world;
        let i = self.seats[seat].selected_target;
        let v = self.villagers[i];
        let mut lines: Vec<Line> = vec![];
        let mut y = 330;
        lines.push(Line { x: 60, y, text: vill_name(i).into(), rgb: (255, 255, 255) });
        // the family's record: the last beats of a life in a word each
        // - built before the dead's branch so a grave keeps it too
        let beats: Vec<&'static str> = self.tales.get(&i).map(|t| t.iter().rev().take(2).map(|e| match e.kind {
            crate::sim::story::EventKind::Birth => "born",
            crate::sim::story::EventKind::Death => "died",
            crate::sim::story::EventKind::Illness => "took ill",
            crate::sim::story::EventKind::Recovery => "healed",
            crate::sim::story::EventKind::Peace => "made peace",
            crate::sim::story::EventKind::Bonding => "grew close",
            crate::sim::story::EventKind::Betrayal => "betrayed",
            crate::sim::story::EventKind::Recruitment => "joined the cause",
            crate::sim::story::EventKind::Rejection => "turned away",
            crate::sim::story::EventKind::Alliance => "swore in",
            crate::sim::story::EventKind::Conflict => "bad blood",
            crate::sim::story::EventKind::Discovery => "found out",
            crate::sim::story::EventKind::Achievement => "made their name",
            crate::sim::story::EventKind::Hardship => "knew the empty pot",
            crate::sim::story::EventKind::Unrest => "felt the anger rise",
        }).collect()).unwrap_or_default();
        let record = if beats.is_empty() { String::new() } else { format!(" ({})", beats.join("\u{00b7}")) };
        if let Some(day) = self.died_on.get(i).copied().flatten() {
            y += 14;
            lines.push(Line { x: 60, y, text: format!("Died on day {}", day), rgb: (230, 90, 80) });
            y += 13;
            lines.push(Line { x: 60, y, text: "rest in peace".into(), rgb: (150, 150, 165) });
            // who carries the grief for them - the losses the day their
            // death's mourned left behind
            // villagers and the seats' wanderers alike - grief records
            // losses for both
            // the seats' wanderers lead - a player's own grief shouldn't
            // trail the whole village and fall off the card's three
            let seats: Vec<Entity> = self.seats.iter().map(|s| s.player).collect();
            let mut folk: Vec<Entity> = seats.clone();
            folk.extend(self.villagers.iter().copied().filter(|v| !seats.contains(v)));
            let mourners: Vec<String> = folk.iter().copied()
                .filter(|&m| m != v && self.world.get_component::<crate::sim::psychology::AttachmentHistory>(m)
                    .map(|h| h.episodes.iter().any(|e|
                        matches!(e.kind, crate::sim::psychology::EpisodeKind::Loss) && e.with == v))
                    .unwrap_or(false))
                .take(3)
                .map(|m| self.villagers.iter().position(|&x| x == m)
                    .map(|mi| vill_name(mi).to_string())
                    .or_else(|| self.world.get_component::<PlayerCharacter>(m).map(|p| p.name.clone()))
                    .unwrap_or_else(|| "someone".into())).collect();
            // clear of the grave's portrait, then the plain record
            y = y.max(398);
            card_prop(&mut lines, &mut y, "Mourned by", &mourners.join(", "), (170, 160, 190));
            card_prop(&mut lines, &mut y, "A life",
                record.trim().trim_start_matches('(').trim_end_matches(')'), (160, 180, 200));
            return InspectorV { fig: None, kid: false, dead: true, died_day: Some(day), hearts: None, lines };
        }
        let org_members = world.get_component::<Organization>(self.seats[seat].org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let occ = desires::occupation_of(world, v).unwrap_or_else(|| "Villager".into());
        y += 12;
        lines.push(Line { x: 60, y,
            text: format!("{} \u{00b7} {}", occ, vocation::class_name(desires::class_of(world, v))), rgb: (200, 200, 230) });
        if livelihood::has_role(&self.world, v, TownRole::Herbologist) {
            let n = herbology::remedies_on_hand(world, v);
            // what's on the shelf: the remedies' herbs named, so a
            // caller knows what she can brew for them
            let on_shelf: Vec<String> = world.get_component::<herbology::Apothecary>(v)
                .map(|a| a.remedies.iter().take(3).filter_map(|r|
                    world.get_component::<herbology::Herb>(r.herb)
                        .map(|h| h.name.replace('_', " "))).collect())
                .unwrap_or_default();
            let what = if on_shelf.is_empty() { String::new() }
                else { format!(" \u{00b7} {}", on_shelf.join(", ")) };
            y += 12;
            for w in wrap_text(&format!("knows the cures \u{00b7} {} remedies{}", n, what), card_chars(60)) {
                lines.push(Line { x: 60, y, text: w, rgb: (150, 210, 150) });
                y += CARD_ROW;
            }
        }
        if let Some(c) = world.get_component::<Craft>(v) {
            // the day's work on the same line: what the bench made today
            let made = self.produced.get(&v).filter(|(d, _)| *d == self.tick)
                .map(|(_, m)| *m).unwrap_or(0.0);
            let made_txt = if made > 0.0 {
                format!(" \u{00b7} made {:.0} {}", made, vocation::output(c.vocation).unwrap_or("goods"))
            } else { String::new() };
            // their rung first - it can't ride off the note's edge
            // behind a busy day's make
            let level = vocation::level_of(c.skill);
            y += 12;
            lines.push(Line { x: 60, y,
                text: format!("\u{1}Craft\u{2}{}:{:.0}% \u{b7} {:?}{}", c.skill, c.skill * 100.0, level, made_txt), rgb: (0, 0, 0) });
        }
        if let Some(p) = world.get_component::<Purse>(v) {
            y += 12;
            lines.push(Line { x: 60, y, text: format!("Gold {:.0}", p.gold), rgb: (240, 210, 120) });
        }
        if let Some(h) = world.get_component::<Hunger>(v) {
            y += 12;
            lines.push(Line { x: 60, y,
                text: format!("\u{1}Fed\u{2}{}:{}", h.value / 100.0, fullness_word(h.value)), rgb: (0, 0, 0) });
        }
        if let Some(he) = world.get_component::<Health>(v) {
            let note = disease_of(world, v).map(|(name, stage)| format!("{} ({:?})", name, stage))
                .unwrap_or_else(|| format!("{:.0}%", he.condition * 100.0));
            y += 12;
            lines.push(Line { x: 60, y, text: format!("\u{1}Health\u{2}{}:{}", he.condition, note), rgb: (0, 0, 0) });
        }
        // how well the organizer knows them: the stage word rides the
        // name row, and it decides how much of the page they let you read
        let stage = crate::game::organizing::stage_of(world, self.seats[seat].player, v);
        let known = stage >= crate::game::organizing::RelStage::Known;
        // the odds before the ask: a campaign under way, a member not yet
        // made - shown for a sympathetic stranger too, since their ask
        // lands without rapport at all
        let oe = self.seats[seat].org;
        let member = world.get_component::<crate::sim::community::Organization>(oe)
            .map(|o| o.members.contains(&v)).unwrap_or(true);
        let campaign = crate::game::campaign::campaign_of(world, oe);
        let hope = match (member, campaign) {
            (false, Some(c)) => {
                let pe = self.seats[seat].player;
                let rp = crate::game::player::rapport(world, pe, v);
                let personal = crate::game::relationships::find_relationship(world, pe, v)
                    .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
                    .map(|r| r.strength).unwrap_or(0.0);
                if crate::game::player::sympathizes(world, c, v) { "sure to sign on" }
                else if rp < crate::game::player::RECRUIT_RAPPORT_NEEDED { "a stranger's ask" }
                else if (personal + rp * 0.4).min(1.0) >= crate::sim::community::RECRUIT_MIN_TRUST { "likely to join" }
                else { "a cold ask" }
            }
            _ => "",
        };
        if stage > crate::game::organizing::RelStage::Stranger {
            let tail = if hope.is_empty() { String::new() } else { format!(" \u{00b7} {}", hope) };
            lines.push(Line { x: 150, y: 330,
                text: format!("{}{}", crate::game::organizing::stage_label(stage), tail), rgb: (170, 200, 160) });
        } else if !hope.is_empty() {
            lines.push(Line { x: 150, y: 330, text: hope.into(), rgb: (170, 200, 160) });
        }
        // below the portrait the page's fields begin: each a labelled
        // row, its value wrapped inside the card
        y = y.max(400);
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
        card_prop(&mut lines, &mut y, "Home", &roost, (190, 190, 210));
        let faith = world.get_component::<crate::sim::religion::Faith>(v)
            .map(|f| if f.devotion > 0.6 { "devout" }
                else if f.devotion < 0.25 { "lapsed" } else { "" }).unwrap_or("");
        // and a soul far enough along gets a name for it
        let stage_tag = world.get_component::<crate::sim::religion::FaithStage>(v)
            .and_then(|st| st.title()).unwrap_or_default();
        card_prop(&mut lines, &mut y, "Faith",
            &[faith, stage_tag].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" \u{00b7} "),
            (190, 190, 210));
        // the guild's badge: a trader in the mutual-protection compact
        let guilded = world.query::<crate::game::guilds::Guild>().first().copied()
            .and_then(|g| world.get_component::<crate::game::guilds::Guild>(g))
            .map(|g| g.members.contains(&v)).unwrap_or(false);
        // the village's word on them: a name in good standing or in ill
        // repute - the stage they've earned, not the bond they hold
        let standing = world.get_component::<crate::sim::reputation_qsp::ReputationState>(v)
            .map(|r| match r.stage {
                crate::sim::reputation_qsp::RepStage::Esteemed => "esteemed",
                crate::sim::reputation_qsp::RepStage::Trusted => "well thought of",
                crate::sim::reputation_qsp::RepStage::Disgraced => "ill-spoken of",
                _ => "" }).unwrap_or("");
        card_prop(&mut lines, &mut y, "Repute",
            &[standing, if guilded { "of the guild" } else { "" }].into_iter()
                .filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" \u{00b7} "),
            (190, 190, 210));
        // kin by name take a bond close enough to have been told
        if known {
            let kin: Vec<String> = self.villagers.iter().copied()
                .filter(|&o| o != v && self.rel_kind(v, o) == Some(crate::game::relationships::RelKind::Family))
                .take(3)
                .filter_map(|o| self.villagers.iter().position(|&x| x == o).map(|i| vill_name(i).to_string()))
                .collect();
            card_prop(&mut lines, &mut y, "Kin", &kin.join(", "), (190, 190, 210));
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
                "taken"
            } else if world.get_component::<crate::game::actions::Demoralized>(v).is_some() {
                "hollow"
            } else if fight.as_ref().map(|issue|
                world.get_component::<crate::game::reactions::Sympathized>(v)
                    .map(|s| s.issues.iter().any(|(i, _)| i == issue)).unwrap_or(false))
                .unwrap_or(false) {
                "woke"
            } else if org_members.contains(&v) { "with you" } else { "" };
            // what they're about: the day's plan, unless hunger has them
            let bench = world.get_component::<Craft>(v).and_then(|c| c.workplace)
                .and_then(|b| world.get_component::<Building>(b)).map(|b| b.name.clone());
            let doing = (vil.activity == Activity::Idle)
                .then(|| routine::routine_text(world, v, bench.as_deref())).flatten()
                .unwrap_or_else(|| activity_text(vil.activity).to_string());
            card_prop(&mut lines, &mut y, "Nature", nature, (200, 160, 160));
            card_prop(&mut lines, &mut y, "Today", &doing, (200, 160, 160));
            card_prop(&mut lines, &mut y, "Cause", cause, (200, 160, 160));
            // a thing made to outlast, in their hands - one name and a
            // count, told only once they're known to you
            let held: Vec<&str> = if known { self.heirlooms.iter()
                .filter_map(|&i| world.get_component::<villager::Heirloom>(i))
                .filter(|h| h.owner == v).map(|h| h.name).collect() } else { vec![] };
            match held.as_slice() {
                [] => {}
                [one] => card_prop(&mut lines, &mut y, "Holds", one, (220, 200, 160)),
                [first, rest @ ..] => card_prop(&mut lines, &mut y, "Holds",
                    &format!("{} +{}", first, rest.len()), (220, 200, 160)),
            }
        }
        // what they're made of, once they've let you close enough to see:
        // each trait and what it does to their day, their temperament
        // under it all - a word the village would use
        if known {
            let mut told: Vec<String> = world.get_component::<crate::game::traits::Personality>(v)
                .map(|p| p.traits.iter().map(|&t|
                    format!("{} ({})", trait_word(t), trait_hint(t))).collect())
                .unwrap_or_default();
            if let Some(p) = world.get_component::<crate::sim::psychology::Personality>(v) {
                told.push(crate::sim::psychology::disposition_word(p).into());
            }
            if !told.is_empty() {
                card_head(&mut lines, &mut y, "TRAITS");
                for t in told { card_bullet(&mut lines, &mut y, t, (215, 200, 160)); }
            }
        }
        // what the village holds against them or has let slip: bad blood
        // first - it burns brightest in the talk - then the sworn word,
        // the secrets told, the stocks' verdict
        let mut issues: Vec<(String, (u8, u8, u8))> = vec![];
        let feuds = self.feuds_of(v);
        for (n, (other, burning)) in feuds.iter().take(2).enumerate() {
            let more = if n == 1 && feuds.len() > 2 { format!(" +{}", feuds.len() - 2) } else { String::new() };
            issues.push((format!("feuds with {}{}{}", self.name_of(*other), more,
                if *burning { " - it burns" } else { "" }), (230, 140, 110)));
        }
        // a sworn hand shows in the same list - the pact's no secret
        if crate::game::contracts::pact_live(world, self.seats[seat].player, v) {
            issues.push(("sworn to you".into(), (150, 210, 150)));
        }
        if let Some(k) = world.get_component::<Known>(v) {
            let mut told = 0;
            for r in k.revealed.iter() {
                if told >= 3 { break; }
                let text = match r {
                    desires::Reveal::Want(w) => desires::want_label(*w).to_string(),
                    desires::Reveal::Problem(p) => desires::problem_label(p),
                };
                // a worry heard twice still reads once
                if !issues.iter().any(|(t, _)| *t == text) {
                    issues.push((text, (210, 170, 140)));
                    told += 1;
                }
            }
        }
        // the village's answer, public as the stocks: the rung they're on
        if let Some(s) = self.sanctioned.iter()
            .filter(|(e, _, u)| *e == v && self.tick < *u)
            .map(|(_, s, _)| *s).max() {
            issues.push((crate::game::sanctions::label(s).into(), (230, 140, 110)));
        }
        if !issues.is_empty() {
            card_head(&mut lines, &mut y, "ISSUES");
            for (t, rgb) in issues { card_bullet(&mut lines, &mut y, t, rgb); }
        }
        // the biography the seat knows: years and build are public, but
        // temperament and kin take a bond close enough to have been told
        // the tale the days are writing: once a theme takes shape the
        // card says what kind of story it is - the biography carries it
        // when one's known, or it stands alone for a stranger
        let tale = self.tales.get(&i).and_then(|t| crate::sim::story::detect_story(t))
            .map(|s| match s.theme {
                crate::sim::story::StoryTheme::Survival => "a tale of survival",
                crate::sim::story::StoryTheme::Rise => "a tale of rising",
                crate::sim::story::StoryTheme::Fall => "a tale of falling",
                crate::sim::story::StoryTheme::Romance => "a tale of the heart",
                crate::sim::story::StoryTheme::Redemption => "a tale of redemption",
                crate::sim::story::StoryTheme::Conflict => "a tale of bad blood",
                crate::sim::story::StoryTheme::Community => "a tale of the commons",
                crate::sim::story::StoryTheme::Calamity => "a tale of calamity",
            });
        let bio = self.bio_line_at(seat, v);
        if bio.is_some() || tale.is_some() || !record.is_empty() {
            card_head(&mut lines, &mut y, "LIFE");
            let text = match bio {
                Some(b) => format!("{}{}{}", b,
                    tale.map(|t| format!(" - {}", t)).unwrap_or_default(), record),
                None => format!("{}{}", tale.unwrap_or_default(), record),
            };
            card_para(&mut lines, &mut y, &text, (160, 180, 200), LIFE_LINES);
        }
        // the seat's hearts for them: warmth read in half-heart steps,
        // five at most - empty for a stranger, full for a true friend
        let hearts = crate::game::relationships::find_relationship(world, self.seats[seat].player, v)
            .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
            .map(|r| crate::game::relationships::hearts_of(r.strength))
            .unwrap_or(0.0);
        InspectorV { fig: Some(i), kid: world.get_component::<Kid>(v).is_some(), dead: false, died_day: None,
            hearts: Some(hearts), lines }
    }

    /// The biography the seat knows of `v`: what they are and what they
    /// carry are plain to see; temperament, kin and origin take a bond
    /// known closely enough to have been told
    fn bio_line_at(&self, seat: usize, v: Entity) -> Option<String> {
        let world = &self.world;
        let mut parts: Vec<String> = vec![];
        if world.get_component::<Kid>(v).is_some() {
            parts.push("a child".into());
            // the errand they're on, if one's live - what they're
            // carrying, and for whom
            if let Some(j) = world.get_component::<crate::game::livelihood::OddJob>(v) {
                let who = world.get_component::<Craft>(j.hirer)
                    .map(|c| vocation::vocation_name(c.vocation).to_lowercase())
                    .unwrap_or_else(|| "someone".into());
                parts.push(format!("running {} for the {}", j.good, who));
            }
            // their leaning, if one hirer keeps them busiest - the
            // errands they run most pick their trade at coming of age
            if let Some(g) = world.get_component::<vocation::KidGrowth>(v) {
                if let Some((master, _)) = g.errands.iter().max_by_key(|(_, n)| *n)
                    .filter(|(_, n)| *n >= 3)
                    .filter(|(m, _)| !villager::is_dead(world, *m)) {
                    if let Some(mc) = world.get_component::<Craft>(*master) {
                        parts.push(format!("runs for the {}",
                            vocation::vocation_name(mc.vocation).to_lowercase()));
                    }
                }
            }
        } else if let Some(a) = world.get_component::<crate::game::lifecycle::Age>(v) {
            parts.push(format!("{:.0} winters", a.years));
        }
        if let Some(c) = world.get_component::<villager::Condition>(v) {
            if self.tick < c.until { parts.push(c.kind.to_string()); }
        }
        // how the sickness runs: the stage's word, if they're ill -
        // getting worse reads differently to on the mend
        if let Some(stage) = world.query::<crate::sim::illness::Disease>().iter()
            .filter_map(|&e| world.get_component::<crate::sim::illness::Disease>(e)
                .filter(|d| d.patient == v).map(|d| d.stage))
            .max_by_key(|s| match *s {
                // the worst case tells: a recovering belly can hide a
                // failing liver if the first match wins
                crate::sim::illness::Stage::Latent => 0,
                crate::sim::illness::Stage::Recovering => 1,
                crate::sim::illness::Stage::Mild => 2,
                crate::sim::illness::Stage::Moderate => 3,
                crate::sim::illness::Stage::Severe => 4,
                crate::sim::illness::Stage::Critical => 5,
            }) {
            use crate::sim::illness::Stage::*;
            let word = match stage {
                Latent => "", Mild => "a touch of it", Moderate => "taken ill",
                Severe => "laid low", Critical => "near death", Recovering => "on the mend",
            };
            if !word.is_empty() { parts.push(word.into()); }
        }
        // how warm they are to you: the talk-built rapport, plain to
        // you whether or not the village knows them - a rival's many
        // talks don't make them warm
        let rival = self.rel_kind(self.seats[seat].player, v) == Some(crate::game::relationships::RelKind::Rival);
        let warmth = crate::game::player::rapport(world, self.seats[seat].player, v);
        if !rival {
            if warmth >= 0.7 { parts.push("close to you".into()); }
            else if warmth >= 0.3 { parts.push("warm to you".into()); }
            else if warmth > 0.0 { parts.push("has your ear".into()); }
        }
        // the body's strength: the live condition sickness and hunger
        // wear at, plain to see whether or not the village knows them
        if let Some(h) = world.get_component::<crate::sim::illness::Health>(v) {
            if h.condition < 0.35 { parts.push("failing".into()); }
            else if h.condition < 0.7 { parts.push("slowing".into()); }
        }
        // what the body's learned, or still hides: a survivor's
        // hardiness, and the seed a sickness leaves to wake again
        if let Some(h) = world.get_component::<crate::sim::illness::Health>(v) {
            if h.immunity > 0.5 { parts.push("weathered".into()); }
        }
        // the fold they keep, if the bench behind their day's a
        // pasture - its live count named, ahead of the wordy parts
        // a stranger's line can still read
        if let Some(fold) = world.get_component::<Craft>(v).and_then(|c| c.workplace)
            .filter(|&b| world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Pasture).unwrap_or(false)) {
            let (sheep, goats) = livestock::counts(&self.world, fold);
            let mut keeps = vec![];
            if sheep > 0 { keeps.push(format!("{} sheep", sheep)); }
            if goats > 0 { keeps.push(format!("{} goats", goats)); }
            if !keeps.is_empty() { parts.push(format!("keeps {}", keeps.join(" and "))); }
        }
        // their pet, if a companion's bonded to them - the creature's
        // name and kind, plain to see
        if let Some(pet) = world.query::<crate::game::pets::Pet>().iter()
            .find_map(|&e| world.get_component::<crate::game::pets::Pet>(e)
                .filter(|p| p.owner == Some(v) && p.bond_strength >= 0.3)) {
            parts.push(format!("keeps {} the {}",
                pet.name, format!("{:?}", pet.species).to_lowercase()));
        }
        // the tie between you: friend, rival, kin - plain to you
        // whether or not the village knows them yet
        if let Some((tie, them_senior)) = crate::game::relationships::find_relationship(world, self.seats[seat].player, v)
            .and_then(|e| world.get_component::<crate::game::relationships::Relationship>(e))
            .map(|r| (r.kind, match r.kind {
                // a Mentor tie runs mentor->pupil; an Apprentice tie
                // runs pupil->master - either way the teacher's the
                // end this side isn't
                crate::game::relationships::RelKind::Mentor => r.from == v,
                crate::game::relationships::RelKind::Apprentice => r.other == v,
                _ => false,
            })) {
            use crate::game::relationships::RelKind::*;
            let word = match tie {
                Family => "kin to you", Lover => "your lover",
                Friend => "your friend", Rival => "your rival",
                Mentor | Apprentice if them_senior => "your teacher",
                Mentor | Apprentice => "your apprentice",
                Stranger | Acquaintance => "",
            };
            if !word.is_empty() { parts.push(word.into()); }
        }
        // a loss they've carried: the episode names who, grief's
        // plain to see whether or not the bond's known
        if let Some(gone) = world.get_component::<AttachmentHistory>(v)
            .and_then(|h| h.episodes.iter()
                .find(|e| matches!(e.kind, crate::sim::psychology::EpisodeKind::Loss)
                    && villager::is_dead(world, e.with))
                .map(|e| e.with)) {
            let name = self.villagers.iter().position(|&x| x == gone)
                .map(|i| vill_name(i).to_string())
                .or_else(|| world.get_component::<PlayerCharacter>(gone).map(|p| p.name.clone()))
                .unwrap_or_else(|| "someone".into());
            parts.push(format!("grieves for {}", name));
        }
        if crate::game::organizing::stage_of(world, self.seats[seat].player, v)
            >= crate::game::organizing::RelStage::Known {
            if world.get_component::<country::Outlander>(v).is_some() {
                parts.push(if world.get_component::<rebels::Rebel>(v).is_some() {
                    "a rebel of the woods".into()
                } else { "of the country".into() });
            }
            // a seed they carry: something only the ones who've earned
            // their confidence could know
            if world.get_component::<crate::sim::illness::Relapse>(v).is_some() {
                parts.push("it sleeps in them".into());
            }
            // their cause-craft, if they've a real one - the skill that
            // leads, named by what it does
            if let Some(o) = world.get_component::<crate::game::org_skills::Organizing>(v) {
                use crate::game::org_skills::OrgSkill::*;
                let (best_s, best_v) = [(OneOnOne, o.one_on_one), (HouseMeeting, o.house_meeting),
                    (Turnout, o.turnout), (Agitation, o.agitation), (Training, o.training)]
                    .into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
                if best_v >= 0.5 {
                    let word = match best_s {
                        OneOnOne => "reads a person",
                        HouseMeeting => "works a room",
                        Turnout => "brings them out",
                        Agitation => "stirs a room",
                        Training => "brings others on",
                        SecurityCulture => "keeps counsel",
                        Pulpit => "borrows the pulpit",
                        Press => "owns the pages",
                        SignalNet => "carries the word fast",
                        Symbol => "spreads the sign",
                    };
                    parts.push(word.into());
                }
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
                    dead: villager::is_dead(&self.world, *v),
                    job_offer: self.quest_offer_at(*v).map(|q| format!("{:?}", q.kind)) })
            }).collect();
        for s in self.seats.iter().filter(|s| !s.agent) {
            if s.player == who { continue; }
            if let Some(q) = self.world.get_component::<Pos>(s.player) {
                let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
                if d < 8.0 { near.push(ObsPerson { e: s.player, name: "a wanderer".into(), dist: d,
                    dead: false, job_offer: None }); }
            }
        }
        let owned_pets = self.world.query::<crate::game::pets::Pet>().iter()
            .filter(|&&e| self.world.get_component::<crate::game::pets::Pet>(e).map(|p| p.owner == Some(who)).unwrap_or(false))
            .count();
        let mut animals: Vec<Entity> = self.flock.iter().chain(self.hens.iter()).copied().collect();
        animals.extend(self.pets.iter().map(|(e, _, _)| *e));
        animals.sort_unstable();
        animals.dedup();
        let near_animals = animals.into_iter().filter_map(|e| {
            let q = self.world.get_component::<Pos>(e)?;
            let dist = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
            if dist >= 8.0 { return None; }
            let pet = self.world.get_component::<crate::game::pets::Pet>(e);
            let kind = self.world.get_component::<crate::game::livestock::Livestock>(e)
                .map(|l| format!("{:?}", l.kind))
                .or_else(|| self.hens.contains(&e).then(|| "Chicken".into()))
                .or_else(|| pet.map(|p| format!("{:?}", p.species)))?;
            Some(ObsAnimal { e, kind, pos: (q.x, q.y), dist,
                tameable: owned_pets < 3 && pet.map(|p| p.owner.is_none()).unwrap_or(true) })
        }).collect();
        let near_buildings: Vec<ObsBuilding> = self.building_entities.iter().filter_map(|be| {
            let q = self.world.get_component::<Pos>(*be)?;
            let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
            if d >= 8.0 { return None; }
            let b = self.world.get_component::<Building>(*be)?;
            let kind = match b.kind {
                BuildingKind::Home => "home", BuildingKind::Shop(_) => "shop",
                BuildingKind::Church => "church", BuildingKind::WatchPost => "watch",
                BuildingKind::Mill => "mill", BuildingKind::Granary => "granary",
                BuildingKind::Schoolhouse => "school",
                BuildingKind::PostOffice => "post",
                BuildingKind::TraderStall => "stall", BuildingKind::Pasture => "pasture",
            BuildingKind::Apiary => "apiary",
            BuildingKind::Cellar => "cellar",
            BuildingKind::Dungeon => "dungeon",
            BuildingKind::Plantation => "plantation",
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
            // a faction is a power center, not a banner you march under
            if self.world.get_component::<crate::sim::factions::Faction>(*e).is_some() { continue; }
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
            near, near_animals, near_buildings, satchel,
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

/// What a trait does to their day, for the tooltip - the sim's real
/// pull, not a description: lazy rests, ambitious works, and so on
fn trait_hint(t: crate::game::traits::Trait) -> &'static str {
    use crate::game::traits::Trait::*;
    match t {
        Lazy => "rests more", Ambitious => "works harder",
        Kind => "a warm word", Stingy => "tight-fisted",
        Pious => "keeps the faith", Hotheaded => "quick to anger",
        Gregarious => "strolls the village", Proud => "a proud bearing",
    }
}

/// The inspector card's text margins, in the card's own coordinate
/// space: the plate's edge sits a few px further right, and rows wrap
/// before they reach it rather than running off across the map
pub(in crate::session) const CARD_X: i32 = 18;
/// The right edge card text may reach, in the same space
pub(in crate::session) const CARD_RX: i32 = 366;
/// A card row's pitch
const CARD_ROW: i32 = 11;
/// The LIFE paragraph's cap: a life's longest telling earns a trailing
/// "..." rather than a card that swallows the map
const LIFE_LINES: usize = 6;

/// How many glyphs fit on a card row from `x` to the card's edge
fn card_chars(x: i32) -> usize { ((CARD_RX - x) / GLYPH_W).max(10) as usize }

/// A labelled property row: the label dim, its value in the row's
/// colour, a long value wrapping with the continuation hanging under
/// the value
fn card_prop(lines: &mut Vec<Line>, y: &mut i32, label: &str, value: &str, rgb: (u8, u8, u8)) {
    if value.is_empty() { return; }
    let vx = CARD_X + (label.chars().count() as i32 + 2) * GLYPH_W;
    lines.push(Line { x: CARD_X, y: *y, text: format!("{}:", label), rgb: (150, 150, 170) });
    for w in wrap_text(value, card_chars(vx)) {
        lines.push(Line { x: vx, y: *y, text: w, rgb });
        *y += CARD_ROW;
    }
}

/// A section heading, set off a little from the rows above it
fn card_head(lines: &mut Vec<Line>, y: &mut i32, head: &str) {
    *y += 5;
    lines.push(Line { x: CARD_X, y: *y, text: head.into(), rgb: (255, 215, 0) });
    *y += CARD_ROW + 2;
}

/// One of a section's items: "- ..." hung so a wrapped point still
/// reads as one
fn card_bullet(lines: &mut Vec<Line>, y: &mut i32, text: String, rgb: (u8, u8, u8)) {
    for (n, w) in wrap_text(&text, card_chars(CARD_X + 12)).into_iter().enumerate() {
        lines.push(Line { x: CARD_X, y: *y,
            text: format!("{}{}", if n == 0 { "- " } else { "  " }, w), rgb });
        *y += CARD_ROW;
    }
}

/// A wrapped paragraph row block (the LIFE section), capped at
/// `max` lines with a trailing "..." when the telling runs longer
fn card_para(lines: &mut Vec<Line>, y: &mut i32, text: &str, rgb: (u8, u8, u8), max: usize) {
    let mut ws = wrap_text(text, card_chars(CARD_X + 8));
    let more = ws.len() > max;
    ws.truncate(max);
    if more {
        if let Some(l) = ws.last_mut() { l.push_str(" ..."); }
    }
    for w in ws {
        lines.push(Line { x: CARD_X + 8, y: *y, text: w, rgb });
        *y += CARD_ROW;
    }
}
