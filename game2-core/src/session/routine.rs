//! Daily routines: what a villager is about today - working, selling,
//! wandering, resting, worshipping, feasting - and the place that puts
//! them. Each day the plan picks a mode and a spot; through the day they
//! walk there (a real path, round walls and houses) and then keep to it:
//! a worker at the bench door, a stroller ambling the creek bank, a
//! congregation in rows facing the priest or in a ring at the stones.
//!
//! Movement between frames is `amble`: a slow drift about the anchor that
//! never steps onto a tile they can't stand on, and that calls a path when
//! they've been carried too far off - so nobody walks into a wall all day.

use super::*;

/// One day in seven is the holy day: no work, and the faithful gather
pub const HOLY_EVERY: u64 = 7;
/// Market day, once a month (the org's market_day pays out the same day)
pub(super) const MARKET_DAY: u64 = 20;
/// How far a free day's stroll reaches from where they stand
const STROLL_REACH: i32 = 14;
/// Farther than this from the anchor and they walk back on a path
const LOST_FROM_ANCHOR: f32 = 2.5;
/// How far the holy day's call carries: about a day's walk (a day is ~12
/// tiles at errand pace), so the congregation is in before it's over
const CALL_REACH: i32 = 14;
/// What somebody says as the faithful gather on the holy day
const PRAY_LINES: [&str; 4] = [
    "The bell calls - we gather.",
    "Holy days are kept.",
    "Pray for the village.",
    "The faith holds.",
];
/// And as the old faith keeps its rite at the stones or in the grove
const RITE_LINES: [&str; 3] = [
    "We keep the old ways.",
    "The stones remember.",
    "The old faith endures.",
];
/// Ambling pace, tiles a second (errands walk at MEETING_WALK, 2.5)
const AMBLE_SPEED: f32 = 0.7;

pub fn is_holy(tick: u64) -> bool { tick > 0 && tick % HOLY_EVERY == 0 }
pub fn is_market_day(tick: u64) -> bool { tick % 30 == MARKET_DAY }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Working, Selling, Wandering, Resting, Worshipping, Feasting }

/// Where the mode puts them
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    /// At their own bench (the workplace door, or home for a cottage trade)
    Bench,
    /// Out fetching the day's raw goods
    Supplies(&'static str),
    /// Off to buy something they want
    Errand(&'static str),
    /// A stall on the market square
    Market,
    /// Reading what's nailed up at the market cross
    Notices,
    Creek,
    Woods,
    Stones,
    Tavern,
    /// Gone to see whoever's come to the village: the friar, the pardoner
    Visitor,
    Home,
    /// Wherever they happen to be (no plan reached them today)
    Here,
    /// Children's games about the green
    Play,
    Church,
    StoneCircle,
    Grove,
    Green,
}

/// Today's plan for one villager
#[derive(Clone, Debug)]
pub struct Routine {
    pub mode: Mode,
    pub spot: Spot,
    /// Where they stand (a congregation's slot, a stall, a bench door)
    pub anchor: (f32, f32),
    /// What they turn to while standing: the leader, the flock of faithful
    pub face: Option<(f32, f32)>,
    /// How far they amble about the anchor (0 holds the spot)
    pub drift: f32,
    /// Leads the rite (stands before or among the others)
    pub leads: bool,
    /// The day the plan was made
    pub day: u64,
    /// Got to the anchor at some point today (a rite counts on arrival)
    pub reached: bool,
}

/// What the day loop should do with this villager after planning
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPlan { Work, Sell, Free }

/// A small, stable hash of an entity: who keeps the old faith, which way
/// a stroller's amble turns
fn mix(v: Entity, salt: u64) -> u64 {
    let mut z = (v as u64).wrapping_add(salt).wrapping_mul(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 29)).wrapping_mul(0xBF58476D1CE4E5B9);
    z ^ (z >> 32)
}

/// One in five keeps the old ways: the stones and the grove, not the church
pub fn keeps_old_ways(world: &World, v: Entity) -> bool {
    // a priest is the church's, whatever his grandmother taught him
    let priest = world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false);
    !priest && mix(v, 17) % 5 == 0
}

fn has_trait(world: &World, v: Entity, t: crate::game::traits::Trait) -> bool {
    world.get_component::<crate::game::traits::Personality>(v)
        .map(|p| p.traits.contains(&t)).unwrap_or(false)
}

/// Plain words for the inspector: "working at the smithy", "walking by the creek"
pub fn routine_text(world: &World, v: Entity, bench: Option<&str>) -> Option<String> {
    let r = world.get_component::<Routine>(v)?;
    let lead = r.leads;
    Some(match (r.mode, r.spot) {
        (_, Spot::Supplies(g)) => format!("fetching {}", g.replace('_', " ")),
        (_, Spot::Errand(g)) => format!("off to buy {}", g.replace('_', " ")),
        (Mode::Working, _) => match bench { Some(b) => format!("working at {}", b), None => "at work".into() },
        (Mode::Selling, _) => "selling at the market".into(),
        (_, Spot::Notices) => "reading the notices".into(),
        (_, Spot::Creek) => "walking by the creek".into(),
        (_, Spot::Woods) => "walking in the woods".into(),
        (_, Spot::Stones) => "out by the stones".into(),
        (_, Spot::Tavern) => "at the tavern".into(),
        (_, Spot::Visitor) => "gone to see the visitor".into(),
        (_, Spot::Play) => "playing".into(),
        (Mode::Resting, _) => "resting at home".into(),
        (Mode::Worshipping, Spot::Church) => if lead { "leading prayers".into() } else { "at prayer in church".into() },
        (Mode::Worshipping, Spot::StoneCircle) => if lead { "leading the old rite".into() } else { "at the stone circle".into() },
        (Mode::Worshipping, Spot::Grove) => if lead { "leading the old rite".into() } else { "at the grove".into() },
        (Mode::Worshipping, _) => "at prayer".into(),
        (Mode::Feasting, _) => "at the feast".into(),
        _ => "about the village".into(),
    })
}

/// Worshippers (and stall-holders) holding their slot don't step aside for
/// passers-by - a congregation shoved three tiles apart is no congregation
pub fn holds_slot(world: &World, v: Entity) -> bool {
    world.get_component::<Routine>(v).map(|r| {
        matches!(r.mode, Mode::Worshipping | Mode::Feasting | Mode::Selling) && r.drift == 0.0
            && world.get_component::<Pos>(v)
                .map(|p| (p.x - r.anchor.0).abs() + (p.y - r.anchor.1).abs() < 0.6).unwrap_or(false)
    }).unwrap_or(false)
}

/// The look index the view reads as the pedlar's striped booth
pub const STALL_TRADER: u8 = u8::MAX;

impl Game {
    /// The stalls standing today, for the view: one before each seller
    /// who's reached their pitch (between them and the square they face),
    /// and the pedlar's booth while he's selling
    pub(super) fn stalls_standing(&self) -> Vec<(f32, f32, u8)> {
        let mut out: Vec<(f32, f32, u8)> = self.villagers.iter().filter_map(|&v| {
            let r = self.world.get_component::<Routine>(v)?;
            if r.mode != Mode::Selling || r.day != self.tick || !r.reached { return None; }
            let (fx, fy) = r.face?;
            let (dx, dy) = (fx - r.anchor.0, fy - r.anchor.1);
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            Some((r.anchor.0 + 0.6 * dx / len, r.anchor.1 + 0.6 * dy / len, (mix(v, 9) % 200) as u8))
        }).collect();
        // (his stall entity outlives the selling: it goes when he's off the map)
        if let Some(p) = self.pedlar.as_ref().filter(|m| m.selling_until.is_some_and(|until| self.tick <= until))
            .and_then(|m| m.stall).and_then(|s| self.world.get_component::<Pos>(s)) {
            out.push((p.x, p.y, STALL_TRADER));
        }
        out
    }

    fn tile_of(&self, v: Entity) -> Option<(i32, i32)> {
        self.world.get_component::<Pos>(v).map(|p| (p.x.round() as i32, p.y.round() as i32))
    }

    pub(super) fn set_routine(&mut self, v: Entity, r: Routine) {
        if self.world.get_component::<Routine>(v).is_some() {
            *self.world.get_component_mut::<Routine>(v).unwrap() = r;
        } else {
            self.world.add_component(v, r);
            self.world.add_component(v, villager::Steered);
        }
    }

    /// Walk `v` to `anchor` (unless they're already there) and keep the
    /// plan. False if there's no way there from here
    fn go_keep(&mut self, v: Entity, r: Routine, reach: i32) -> bool {
        let Some(at) = self.world.get_component::<Pos>(v).copied() else { return false };
        let t = (r.anchor.0.round() as i32, r.anchor.1.round() as i32);
        let near = (at.x - r.anchor.0).abs() + (at.y - r.anchor.1).abs() <= 1.5;
        if !near && !self.send_tile_walk(v, t, Goal::Wander, reach) { return false; }
        self.set_routine(v, r);
        true
    }

    /// As go_keep, but only if the road there (round walls and houses, not
    /// as the crow flies) is at most `steps` long
    fn go_keep_within(&mut self, v: Entity, r: Routine, steps: usize) -> bool {
        if !self.go_keep(v, r, steps as i32) { return false; }
        let long = self.travels.iter().rev().find(|t| t.who == v).map(|t| t.path.len().saturating_sub(1) > steps).unwrap_or(false);
        if long {
            self.travels.retain(|t| t.who != v);
            if let Some(rm) = self.world.get_component_mut::<Routine>(v) { rm.mode = Mode::Resting; rm.spot = Spot::Here; }
            return false;
        }
        true
    }

    /// Whether `v`'s walk to today's worship actually landed them there
    /// (a failed `go_keep_within` leaves them `Resting`, or never got as
    /// far as setting `Routine` at all)
    fn reached_worship(&self, v: Entity, tick: u64) -> bool {
        self.world.get_component::<Routine>(v)
            .map(|r| r.mode == Mode::Worshipping && r.day == tick)
            .unwrap_or(false)
    }

    /// The passable tile nearest `t` (within 3) not in `taken`
    fn free_near(&self, t: (i32, i32), taken: &[(i32, i32)]) -> Option<(i32, i32)> {
        (0..=3).flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (t.0 + dx, t.1 + dy))))
            .find(|&q| self.map.passable(q.0, q.1) && !taken.contains(&q))
    }

    /// The building this person works at, if it's a real place on the map
    fn bench_of(&self, v: Entity) -> Option<Entity> {
        self.world.get_component::<Craft>(v).and_then(|c| c.workplace)
            .filter(|b| self.world.get_component::<Pos>(*b).is_some())
    }

    fn door_of(&self, b: Entity) -> Option<(i32, i32)> {
        let p = self.world.get_component::<Pos>(b)?;
        self.at_the_door(p.x.round() as i32, p.y.round() as i32)
    }

    /// The day's free choice for an adult the day loop reached: work,
    /// a stall on market day, a stroll somewhere, or a day at home
    pub(super) fn plan_day(&mut self, v: Entity, tick: u64, rng: &mut Rng) -> DayPlan {
        use crate::game::traits::Trait;
        let holy = is_holy(tick);
        let trade = self.world.get_component::<Craft>(v)
            .map(|c| vocation::output(c.vocation).is_some()).unwrap_or(false);
        let mut work = if holy || !trade { 0 } else { 70 };
        let mut wander = if trade { 18 } else { 55 };
        let mut rest = if trade { 12 } else { 30 };
        if has_trait(&self.world, v, Trait::Lazy) { work = work * 6 / 10; rest *= 2; }
        if has_trait(&self.world, v, Trait::Ambitious) { work = work * 13 / 10; }
        if has_trait(&self.world, v, Trait::Gregarious) { wander = wander * 16 / 10; }
        if is_sick(&self.world, v) { rest *= 4; work /= 2; }
        let roll = rng.below(work + wander + rest);
        if roll < work {
            // market day: a crafter with something made takes a stall
            if is_market_day(tick) && rng.below(2) == 0 && self.stall_for(v, tick) { return DayPlan::Sell; }
            // the input/bench branches in livelihood_tick start exactly one
            // walk for Work - don't start a second one here
            return DayPlan::Work;
        }
        if roll < work + wander && self.plan_stroll(v, tick, rng) { return DayPlan::Free; }
        self.plan_rest(v, tick);
        DayPlan::Free
    }

    /// Off to the bench: its door if they have a workshop, home otherwise
    /// (a cottage trade works at the hearth)
    pub(super) fn plan_bench(&mut self, v: Entity, tick: u64) {
        let door = self.bench_of(v).and_then(|b| self.door_of(b))
            .or_else(|| household::home_of(&self.world, v).and_then(|h| self.door_of(h)));
        let Some(door) = door else { return self.plan_here(v, tick, Mode::Working) };
        // two at one bench stand side by side, not on each other
        let o = (mix(v, 3) % 3) as f32 * 0.6 - 0.6;
        let anchor = (door.0 as f32 + o, door.1 as f32 + 0.3);
        let r = Routine { mode: Mode::Working, spot: Spot::Bench, anchor, face: None, drift: 0.35, leads: false, day: tick, reached: false };
        if !self.go_keep(v, r, 30) { self.plan_here(v, tick, Mode::Working); }
    }

    /// Out on an errand (the walk is already under way): when it's done
    /// they come back to the bench, or home
    pub(super) fn plan_errand(&mut self, v: Entity, tick: u64, spot: Spot) {
        let back = self.bench_of(v).and_then(|b| self.door_of(b))
            .or_else(|| household::home_of(&self.world, v).and_then(|h| self.door_of(h)))
            .map(|(x, y)| (x as f32, y as f32 + 0.3))
            .or_else(|| self.world.get_component::<Pos>(v).map(|p| (p.x, p.y)));
        let Some(anchor) = back else { return };
        self.set_routine(v, Routine { mode: Mode::Working, spot, anchor, face: None, drift: 0.35, leads: false, day: tick, reached: false });
    }

    /// A stall round the market square, if they've anything to sell
    fn stall_for(&mut self, v: Entity, tick: u64) -> bool {
        let Some(good) = self.world.get_component::<Craft>(v).and_then(|c| vocation::output(c.vocation)) else { return false };
        let in_bag = self.world.get_component::<Satchel>(v).map(|s| s.goods.get(good).copied().unwrap_or(0)).unwrap_or(0);
        let at_bench = self.bench_of(v).and_then(|b| self.world.get_component::<Building>(b))
            .map(|b| b.stock.get(good).copied().unwrap_or(0)).unwrap_or(0);
        if in_bag + at_bench == 0 { return false; }
        let Some(mp) = self.world.get_component::<Pos>(self.market_food).copied() else { return false };
        // stalls ring the square, three tiles out, each seller their own
        let k = mix(v, 5) % 10;
        let a = k as f32 / 10.0 * std::f32::consts::TAU;
        let want = ((mp.x + 3.0 * a.cos()).round() as i32, (mp.y + 3.0 * a.sin()).round() as i32);
        let taken: Vec<(i32, i32)> = self.anchors_today(tick, |m| m == Mode::Selling);
        let Some(t) = self.free_near(want, &taken) else { return false };
        let face = Some((mp.x, mp.y));
        let r = Routine { mode: Mode::Selling, spot: Spot::Market, anchor: (t.0 as f32, t.1 as f32), face, drift: 0.0, leads: false, day: tick, reached: false };
        self.go_keep(v, r, 30)
    }

    /// Anchors already handed out today (rounded), for modes matching `which`
    fn anchors_today(&self, tick: u64, which: impl Fn(Mode) -> bool) -> Vec<(i32, i32)> {
        self.villagers.iter().filter_map(|&o| self.world.get_component::<Routine>(o))
            .filter(|r| r.day == tick && which(r.mode))
            .map(|r| (r.anchor.0.round() as i32, r.anchor.1.round() as i32)).collect()
    }

    /// A day at home: the door, sitting out
    pub(super) fn plan_rest(&mut self, v: Entity, tick: u64) {
        let Some(door) = household::home_of(&self.world, v).and_then(|h| self.door_of(h)) else {
            return self.plan_here(v, tick, Mode::Resting);
        };
        let r = Routine { mode: Mode::Resting, spot: Spot::Home, anchor: (door.0 as f32, door.1 as f32 + 0.4), face: None, drift: 0.15, leads: false, day: tick, reached: false };
        if !self.go_keep(v, r, 30) { self.plan_here(v, tick, Mode::Resting); }
    }

    /// No plan that can reach: keep to where they stand, a little amble
    pub(super) fn plan_here(&mut self, v: Entity, tick: u64, mode: Mode) {
        let Some(p) = self.world.get_component::<Pos>(v).copied() else { return };
        self.set_routine(v, Routine { mode, spot: Spot::Here, anchor: (p.x, p.y), face: None, drift: 0.6, leads: false, day: tick, reached: false });
    }

    /// Come to see the visitor: stand about where they are now (the
    /// visitor tests set folk beside the figure)
    #[cfg(test)]
    pub(super) fn plan_visit(&mut self, v: Entity, tick: u64) {
        let Some(p) = self.world.get_component::<Pos>(v).copied() else { return };
        self.set_routine(v, Routine { mode: Mode::Wandering, spot: Spot::Visitor, anchor: (p.x, p.y), face: None, drift: 0.5, leads: false, day: tick, reached: false });
    }

    /// The places a free afternoon goes, within reach of `from`:
    /// (spot, tile, weight)
    fn stroll_options(&self, v: Entity, from: (i32, i32)) -> Vec<(Spot, (i32, i32), i32)> {
        use crate::game::traits::Trait;
        let near = |t: (i32, i32)| (t.0 - from.0).abs() + (t.1 - from.1).abs() <= STROLL_REACH;
        let pick = |tiles: Vec<(i32, i32)>, salt: u64| -> Option<(i32, i32)> {
            if tiles.is_empty() { None } else { Some(tiles[(mix(v, salt) as usize ^ self.tick as usize) % tiles.len()]) }
        };
        let gregarious = has_trait(&self.world, v, Trait::Gregarious);
        let mut out = vec![];
        // the notices nailed up at the market cross
        if let Some(mp) = self.world.get_component::<Pos>(self.market_food) {
            let (mx, my) = (mp.x.round() as i32, mp.y.round() as i32);
            let round: Vec<(i32, i32)> = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (mx + dx, my + dy)))
                .filter(|&(x, y)| self.map.passable(x, y) && near((x, y))).collect();
            if let Some(t) = pick(round, 1) { out.push((Spot::Notices, t, if gregarious { 5 } else { 3 })); }
        }
        // the creek bank: a dry tile beside running water
        let (w, h) = (self.map.width as i32, self.map.height as i32);
        let bank: Vec<(i32, i32)> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| near((x, y)) && self.map.passable(x, y)
                && NEIGHBOURS.iter().any(|(dx, dy)| self.map.get(x + dx, y + dy) == Some(Tile::River)))
            .collect();
        if let Some(t) = pick(bank, 2) { out.push((Spot::Creek, t, 3)); }
        // under the trees
        let wood: Vec<(i32, i32)> = self.map.trees.iter().copied()
            .filter(|&t| near(t) && self.map.passable(t.0, t.1)).collect();
        if let Some(t) = pick(wood, 3) { out.push((Spot::Woods, t, 3)); }
        // beside a boulder out in the fields
        let stones: Vec<(i32, i32)> = self.stones.iter()
            .flat_map(|&(x, y)| NEIGHBOURS.iter().map(move |(dx, dy)| (x + dx, y + dy)))
            .filter(|&t| near(t) && self.map.passable(t.0, t.1)).collect();
        if let Some(t) = pick(stones, 4) { out.push((Spot::Stones, t, 1)); }
        // the tavern's bench, for whoever likes a crowd
        let tavern = self.building_entities.iter().copied().find(|&b|
            self.world.get_component::<Building>(b).map(|bb| bb.kind == BuildingKind::Shop(Vocation::Taverner)).unwrap_or(false));
        if let Some(t) = tavern.and_then(|b| self.door_of(b)).filter(|&t| near(t)) {
            out.push((Spot::Tavern, t, if gregarious { 3 } else { 1 }));
        }
        // whoever's come to the village and stays a while draws a crowd
        for r in &self.rare {
            let (Some(_), Some(_), Some(at)) = (r.e, r.until, r.at) else { continue };
            if r.leaving || !near(at) { continue; }
            let round: Vec<(i32, i32)> = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (at.0 + dx, at.1 + dy)))
                .filter(|&(x, y)| (x, y) != at && self.map.passable(x, y)).collect();
            if let Some(t) = pick(round, 6) { out.push((Spot::Visitor, t, 6)); }
        }
        out
    }

    /// A free day out: pick a place by weight and walk there
    pub(super) fn plan_stroll(&mut self, v: Entity, tick: u64, rng: &mut Rng) -> bool {
        let Some(from) = self.tile_of(v) else { return false };
        let mut opts = self.stroll_options(v, from);
        while !opts.is_empty() {
            let total: i32 = opts.iter().map(|o| o.2).sum();
            let mut roll = rng.below(total);
            let i = opts.iter().position(|o| { if roll < o.2 { true } else { roll -= o.2; false } }).unwrap_or(0);
            let (spot, t, _) = opts.swap_remove(i);
            // the bank and the wood are for walking along; the notices for
            // standing at
            let drift = match spot { Spot::Creek | Spot::Woods => 1.8, Spot::Notices | Spot::Visitor => 0.5, Spot::Tavern => 0.4, _ => 0.9 };
            let r = Routine { mode: Mode::Wandering, spot, anchor: (t.0 as f32, t.1 as f32), face: None, drift, leads: false, day: tick, reached: false };
            if self.go_keep(v, r, STROLL_REACH + 2) { return true; }
        }
        false
    }

    /// A child's day off the errand list: games on the green or down by the water
    pub(super) fn plan_play(&mut self, v: Entity, tick: u64, rng: &mut Rng) {
        let Some(from) = self.tile_of(v) else { return };
        let opts = self.stroll_options(v, from);
        let mut picks: Vec<_> = opts.into_iter().filter(|o| matches!(o.0, Spot::Notices | Spot::Creek | Spot::Woods)).collect();
        if picks.is_empty() { return self.plan_here(v, tick, Mode::Wandering); }
        let (_, t, _) = picks.swap_remove(rng.below(picks.len() as i32) as usize);
        let r = Routine { mode: Mode::Wandering, spot: Spot::Play, anchor: (t.0 as f32, t.1 as f32), face: None, drift: 1.6, leads: false, day: tick, reached: false };
        if !self.go_keep(v, r, STROLL_REACH + 2) { self.plan_here(v, tick, Mode::Wandering); }
    }

    /// Can `v` be gathered today - for worship or a feast?
    fn free_to_gather(&self, v: Entity) -> bool {
        !crate::game::dungeon::unavailable(&self.world, v)
            && self.world.get_component::<TownRole>(v).is_none()
            && self.world.get_component::<Hunger>(v).map(|h| h.value >= 30.0).unwrap_or(true)
            && !self.travels.iter().any(|t| t.who == v)
            && !self.meeting.as_ref().map(|m| m.invited.contains(&v)).unwrap_or(false)
            && !self.seats.iter().any(|s| s.player == v)
            && !self.world.get_component::<villager::Drunk>(v).map(|d| self.tick < d.until).unwrap_or(false)
    }

    /// Rows facing `front` (the leader's place), stepping back along `back`
    fn rows_from(&self, front: (f32, f32), back: (i32, i32), n: usize) -> Vec<(f32, f32)> {
        let side = (back.1.abs(), back.0.abs()); // across the rows
        let cols = 4i32;
        let mut taken: Vec<(i32, i32)> = vec![(front.0.round() as i32, front.1.round() as i32)];
        let mut out = vec![];
        for k in 0..n as i32 {
            let (row, col) = (k / cols, k % cols);
            let off = col * 2 - (cols - 1); // -3, -1, 1, 3 in half-tiles
            let want = (front.0 + (back.0 * (2 + row)) as f32 + side.0 as f32 * off as f32 * 0.5,
                        front.1 + (back.1 * (2 + row)) as f32 + side.1 as f32 * off as f32 * 0.5);
            let t = (want.0.round() as i32, want.1.round() as i32);
            let t = if self.map.passable(t.0, t.1) && !taken.contains(&t) { Some(t) } else { self.free_near(t, &taken) };
            let Some(t) = t else { break };
            taken.push(t);
            // a slot keeps its half-tile offset when the tile under it is the one asked for
            out.push(if t == (want.0.round() as i32, want.1.round() as i32) { want } else { (t.0 as f32, t.1 as f32) });
        }
        out
    }

    /// A ring of `n` round `centre`, wide enough to stand a pace apart
    fn ring_round(&self, centre: (f32, f32), n: usize, min_r: f32) -> Vec<(f32, f32)> {
        let r = (n as f32 * 1.1 / std::f32::consts::TAU).max(min_r);
        let mut taken: Vec<(i32, i32)> = vec![(centre.0.round() as i32, centre.1.round() as i32)];
        let mut out = vec![];
        for k in 0..n {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            let want = (centre.0 + r * a.cos(), centre.1 + r * a.sin());
            let t = (want.0.round() as i32, want.1.round() as i32);
            let ok = self.map.passable(t.0, t.1) && !taken.contains(&t);
            let t = if ok { Some(t) } else { self.free_near(t, &taken) };
            let Some(t) = t else { break };
            taken.push(t);
            out.push(if ok { want } else { (t.0 as f32, t.1 as f32) });
        }
        out
    }

    /// Hand out slots and walk everyone to them. `leader` gets `lead_at`;
    /// everyone faces `face_of(slot)`. Returns how many are on their way
    /// `reach` is how far they'll walk for it: a day's walk, or the
    /// service is over before they're in
    fn assemble(&mut self, tick: u64, mode: Mode, spot: Spot, leader: Option<(Entity, (f32, f32))>,
                folk: &[Entity], slots: &[(f32, f32)], face: (f32, f32), reach: i32) -> usize {
        let mut going = 0;
        if let Some((l, at)) = leader {
            let r = Routine { mode, spot, anchor: at, face: Some(face), drift: 0.0, leads: true, day: tick, reached: false };
            if self.go_keep_within(l, r, reach as usize) { going += 1; }
        }
        let lead_pos = leader.map(|(_, at)| at);
        for (&v, &slot) in folk.iter().zip(slots) {
            let r = Routine { mode, spot, anchor: slot, face: Some(lead_pos.unwrap_or(face)), drift: 0.0, leads: false, day: tick, reached: false };
            if self.go_keep_within(v, r, reach as usize) { going += 1; } else { self.plan_rest(v, tick); }
        }
        going
    }

    /// Dawn of each day: count who kept yesterday's rite, then on a holy
    /// day call the faithful to theirs
    pub(super) fn routine_dawn(&mut self, tick: u64, rng: &mut Rng) {
        self.tally_rites(tick);
        if !is_holy(tick) { return; }
        let church = self.building_entities.iter().copied().find(|&b|
            self.world.get_component::<Building>(b).map(|bb| bb.kind == BuildingKind::Church).unwrap_or(false));
        let mut flock: Vec<Entity> = vec![];
        let mut old: Vec<Entity> = vec![];
        for v in self.villagers.clone() {
            if !self.free_to_gather(v) { continue; }
            let pious = has_trait(&self.world, v, crate::game::traits::Trait::Pious);
            if !pious && rng.below(100) >= 70 { continue; }
            if keeps_old_ways(&self.world, v) { old.push(v); } else { flock.push(v); }
        }
        // a monastery's brothers keep the day at their own abbey - the
        // village church's call can't carry out to them in the country
        let mut abbeys: Vec<(usize, Vec<Entity>)> = vec![];
        flock.retain(|&v| match self.site_of(v)
            .filter(|&si| self.sites[si].kind == country::SiteKind::Monastery) {
            Some(si) => {
                if let Some((_, g)) = abbeys.iter_mut().find(|(s, _)| *s == si) { g.push(v); }
                else { abbeys.push((si, vec![v])); }
                false
            }
            None => true,
        });
        for (si, brothers) in abbeys { self.abbey_rites(tick, rng, si, brothers); }
        // the church: rows before the door, the priest at its head facing them
        if !flock.is_empty() {
            let priest_v = flock.iter().copied().position(|v| self.world.get_component::<Craft>(v)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
            let lead_v = priest_v.map(|i| flock.remove(i));
            let (front, back) = match church.and_then(|c| self.world.get_component::<Pos>(c).copied()) {
                Some(cp) => {
                    let door = self.door_of(church.unwrap()).unwrap_or((cp.x.round() as i32, cp.y.round() as i32 + 1));
                    let back = ((door.0 - cp.x.round() as i32).signum(), (door.1 - cp.y.round() as i32).signum());
                    let back = if back == (0, 0) { (0, 1) } else { back };
                    // with no priest among them the church's own stands at the door
                    let front = if lead_v.is_some() { (door.0 as f32, door.1 as f32) } else { self.priest };
                    (front, back)
                }
                None => {
                    let mp = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y + 1.0)).unwrap_or(self.priest);
                    (mp, (0, 1))
                }
            };
            let slots = self.rows_from(front, back, flock.len());
            let spot = if church.is_some() { Spot::Church } else { Spot::Green };
            let leader_e = lead_v;
            let n = self.assemble(tick, Mode::Worshipping, spot, leader_e.map(|l| (l, front)), &flock, &slots, front, CALL_REACH);
            if n > 0 {
                self.log.push(format!("t{}: the holy day - {} gather at {} for prayers.", tick, n,
                    if church.is_some() { "the church" } else { "the market cross" }));
                // and it's seen and heard: somebody says the word, and
                // the bell rings over the square - but only one who
                // actually made it to the gathering
                let voice = leader_e.filter(|&l| self.reached_worship(l, tick))
                    .or_else(|| flock.iter().copied().find(|&v| self.reached_worship(v, tick)));
                if let Some(v) = voice {
                    self.say(v, PRAY_LINES[rng.below(PRAY_LINES.len() as i32) as usize], TALK_BUBBLE_SECS * 2);
                }
                if church.is_some() { self.emit_sfx("bell", front.0, front.1, 0.8); }
            }
        }
        // the old ways: a ring at the standing stones one holy day, round
        // their elder under the trees the next
        if old.len() >= 2 {
            let li = old.iter().position(|&v| has_trait(&self.world, v, crate::game::traits::Trait::Pious)).unwrap_or(0);
            let leader = old.remove(li);
            let Some(lt) = self.tile_of(leader) else { return };
            let dist = |t: &(i32, i32)| (t.0 - lt.0).abs() + (t.1 - lt.1).abs();
            let at_stones = (tick / HOLY_EVERY) % 2 == 0;
            let centre = if at_stones { self.stones.iter().copied().min_by_key(dist) }
                else { self.map.trees.iter().copied().filter(|t| self.map.passable(t.0, t.1)).min_by_key(dist) };
            let Some(c) = centre.filter(|c| dist(c) <= 24) else { return };
            let c = (c.0 as f32, c.1 as f32);
            if at_stones {
                // round the stone, the elder one of the ring
                let slots = self.ring_round(c, old.len() + 1, 1.6);
                if slots.len() < 2 { return; }
                let n = self.assemble(tick, Mode::Worshipping, Spot::StoneCircle, Some((leader, slots[0])), &old, &slots[1..], c, CALL_REACH + 4);
                if n > 1 {
                    self.log.push(format!("t{}: {} keep the old rite at the stones.", tick, n));
                    // the elder speaks if they made it; else whoever did
                    let speaker = [leader].into_iter().chain(old.iter().copied())
                        .find(|&v| self.reached_worship(v, tick));
                    if let Some(v) = speaker {
                        self.say(v, RITE_LINES[rng.below(RITE_LINES.len() as i32) as usize], TALK_BUBBLE_SECS * 2);
                    }
                }
            } else {
                // under the trees, the elder in the middle of them
                let slots = self.ring_round(c, old.len(), 1.6);
                let n = self.assemble(tick, Mode::Worshipping, Spot::Grove, Some((leader, c)), &old, &slots, c, CALL_REACH + 4);
                if n > 1 {
                    self.log.push(format!("t{}: {} keep the old rite in the grove.", tick, n));
                    let speaker = [leader].into_iter().chain(old.iter().copied())
                        .find(|&v| self.reached_worship(v, tick));
                    if let Some(v) = speaker {
                        self.say(v, RITE_LINES[rng.below(RITE_LINES.len() as i32) as usize], TALK_BUBBLE_SECS * 2);
                    }
                }
            }
        }
    }

    /// The brothers keep the holy day at their own abbey's door: one of
    /// their priests leads, the rest stand in rows before it - the same
    /// office as the village's, only theirs
    fn abbey_rites(&mut self, tick: u64, rng: &mut Rng, si: usize, mut brothers: Vec<Entity>) {
        let Some(&abbey) = self.sites[si].homes.first() else { return };
        let Some(ap) = self.world.get_component::<Pos>(abbey).copied() else { return };
        let door = self.door_of(abbey).unwrap_or((ap.x.round() as i32, ap.y.round() as i32 + 1));
        let lead = brothers.iter().position(|&v| self.world.get_component::<Craft>(v)
            .map(|c| c.vocation == Vocation::Priest).unwrap_or(false))
            .map(|i| brothers.remove(i));
        let front = (door.0 as f32, door.1 as f32);
        let back = ((door.0 - ap.x.round() as i32).signum(), (door.1 - ap.y.round() as i32).signum());
        let back = if back == (0, 0) { (0, 1) } else { back };
        let slots = self.rows_from(front, back, brothers.len());
        let n = self.assemble(tick, Mode::Worshipping, Spot::Church, lead.map(|l| (l, front)),
            &brothers, &slots, front, CALL_REACH);
        if n > 0 {
            self.log.push(format!("t{}: the holy day - {} keep the office at {}.", tick, n, self.sites[si].name));
            let voice = lead.filter(|&l| self.reached_worship(l, tick))
                .or_else(|| brothers.iter().copied().find(|&v| self.reached_worship(v, tick)));
            if let Some(v) = voice {
                self.say(v, PRAY_LINES[rng.below(PRAY_LINES.len() as i32) as usize], TALK_BUBBLE_SECS * 2);
            }
            self.emit_sfx("bell", front.0, front.1, 0.8);
        }
    }

    /// Yesterday's rites: whoever reached their place keeps their faith fed
    fn tally_rites(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            let Some(r) = self.world.get_component::<Routine>(v).cloned() else { continue };
            if r.mode != Mode::Worshipping || r.day + 1 != tick { continue; }
            if r.reached { desires::satisfy(&mut self.world, v, Want::Faith, tick + HOLY_EVERY); }
        }
    }

    /// A feast day: the village in a great ring round the market cross
    pub(super) fn gather_feast(&mut self, tick: u64, folk: &[Entity]) {
        let Some(mp) = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y)) else { return };
        let folk: Vec<Entity> = folk.iter().copied().filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).collect();
        let slots = self.ring_round(mp, folk.len(), 2.5);
        self.assemble(tick, Mode::Feasting, Spot::Green, None, &folk, &slots, mp, 30);
    }

    /// Between ticks: everyone with a plan and no walk under way keeps to
    /// it - a slow amble round the anchor, never onto a tile they can't
    /// stand on, and a path back if they've been carried off
    pub(super) fn amble(&mut self) {
        let t = self.frame_n as f32 / FPS as f32;
        let step = AMBLE_SPEED / FPS as f32;
        let mut lost: Vec<(Entity, (i32, i32))> = vec![];
        for i in 0..self.villagers.len() {
            let v = self.villagers[i];
            if crate::game::dungeon::unavailable(&self.world, v) { continue; }
            if self.world.get_component::<Villager>(v).map(|x| x.activity != Activity::Idle).unwrap_or(true) { continue; }
            if self.travels.iter().any(|tr| tr.who == v) { continue; }
            if self.meeting.as_ref().map(|m| m.invited.contains(&v)).unwrap_or(false) { continue; }
            // in a talk, or someone's character: they stand for it
            if self.seats.iter().any(|s| s.player == v || s.talk.as_ref().map(|k| k.who == v).unwrap_or(false)) { continue; }
            let Some(p) = self.world.get_component::<Pos>(v).copied() else { continue };
            if self.world.get_component::<Routine>(v).is_none() {
                let tick = self.tick;
                self.plan_here(v, tick, Mode::Wandering);
            }
            let r = self.world.get_component::<Routine>(v).cloned().unwrap();
            // no plan has them, and an errand left them out past the
            // village: home, on the road (their own door, or the market)
            if r.spot == Spot::Here {
                let home = household::home_of(&self.world, v).and_then(|h| self.door_of(h))
                    .or_else(|| self.door_of(self.market_food));
                if let Some(h) = home {
                    if (p.x - h.0 as f32).abs() + (p.y - h.1 as f32).abs() > VILLAGE_RADIUS + 2.0 {
                        lost.push((v, h));
                        continue;
                    }
                }
            }
            let (ax, ay) = r.anchor;
            if !r.reached && (p.x - ax).abs() + (p.y - ay).abs() < 1.0 {
                if let Some(rm) = self.world.get_component_mut::<Routine>(v) { rm.reached = true; }
            }
            if (p.x - ax).abs() + (p.y - ay).abs() > LOST_FROM_ANCHOR + r.drift {
                // today's place is worth walking back to; an old one isn't -
                // wherever the last errand left them is where they are now
                if r.day == self.tick && r.spot != Spot::Here { lost.push((v, (ax.round() as i32, ay.round() as i32))); }
                else { let tick = self.tick; self.plan_here(v, tick, r.mode); }
                continue;
            }
            // a lazy figure-eight about the anchor, each their own phase
            let ph = (mix(v, 11) % 628) as f32 / 100.0;
            let goal = (ax + r.drift * (t * 0.35 + ph).sin(), ay + r.drift * 0.8 * (t * 0.23 + ph * 1.3).cos());
            let (dx, dy) = (goal.0 - p.x, goal.1 - p.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d < 0.02 { continue; }
            let s = step.min(d);
            let (nx, ny) = (p.x + dx / d * s, p.y + dy / d * s);
            let ok = |x: f32, y: f32| self.map.passable(x.round() as i32, y.round() as i32);
            let to = if ok(nx, ny) { Some((nx, ny)) } else if ok(nx, p.y) { Some((nx, p.y)) } else if ok(p.x, ny) { Some((p.x, ny)) } else { None };
            if let (Some((x, y)), Some(pm)) = (to, self.world.get_component_mut::<Pos>(v)) { pm.x = x; pm.y = y; }
        }
        // carried off (shoved by a crowd, left at the end of an errand):
        // walk back on the road, or, with no way back, make here the place
        for (v, at) in lost {
            if !self.send_tile_walk(v, at, Goal::Wander, 40) {
                let mode = self.world.get_component::<Routine>(v).map(|r| r.mode).unwrap_or(Mode::Wandering);
                let tick = self.tick;
                self.plan_here(v, tick, mode);
            }
        }
    }

    /// Standing folk turn to what they're gathered for: the congregation
    /// to the priest, the priest to them, a seller to the square
    pub(super) fn face_routines(&mut self) {
        for &v in &self.villagers {
            let Some(r) = self.world.get_component::<Routine>(v) else { continue };
            let Some(f) = r.face else { continue };
            let Some(pose) = self.villager_poses.get_mut(&v) else { continue };
            if pose.2 { continue; }
            let (dx, dy) = (f.0 - pose.0.0, f.1 - pose.0.1);
            if dx.abs() + dy.abs() > 0.3 { pose.1 = facing_on_screen(dx, dy); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk_frames(g: &mut Game, n: u32) { for _ in 0..n { g.update(); } }

    #[test]
    fn test_one_day_in_seven_is_holy() {
        assert!(!is_holy(0));
        assert_eq!((1..=70).filter(|&d| is_holy(d)).count(), 10);
    }

    #[test]
    fn test_a_worker_walks_to_the_bench() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v| g.bench_of(v).is_some()
            && g.world.get_component::<Kid>(v).is_none()).expect("someone works a bench");
        let door = g.door_of(g.bench_of(v).unwrap()).unwrap();
        // start them across town
        let far = g.free_near((door.0 + 8, door.1 + 6), &[]).unwrap();
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: far.0 as f32, y: far.1 as f32 };
        g.travels.retain(|t| t.who != v);
        g.plan_bench(v, g.tick);
        walk_frames(&mut g, 12 * FPS);
        let p = *g.world.get_component::<Pos>(v).unwrap();
        assert!((p.x - door.0 as f32).abs() + (p.y - door.1 as f32).abs() < 2.5,
            "should be at the bench door {:?}, is at ({:.1},{:.1})", door, p.x, p.y);
    }

    #[test]
    fn test_ambling_never_stands_on_a_blocked_tile() {
        let mut g = Game::new();
        for d in 0..40 {
            let n = g.ticks_per_frame(); walk_frames(&mut g, n);
            for &v in &g.villagers {
                if villager::is_dead(&g.world, v) { continue; }
                let Some(p) = g.world.get_component::<Pos>(v) else { continue };
                assert!(!g.map.is_occupied(p.x.round() as i32, p.y.round() as i32),
                    "day {}: villager {} inside a building at ({:.1},{:.1})", d, v, p.x, p.y);
            }
        }
    }

    #[test]
    fn test_the_holy_day_gathers_a_congregation_facing_the_priest() {
        let mut g = Game::new();
        // run to just before the first holy day, then through it
        while g.tick < HOLY_EVERY { g.update(); }
        let worship: Vec<Entity> = g.villagers.iter().copied()
            .filter(|&v| g.world.get_component::<Routine>(v)
                .map(|r| r.mode == Mode::Worshipping && r.day == HOLY_EVERY).unwrap_or(false)).collect();
        assert!(worship.len() >= 4, "a holy day should gather a congregation, got {}", worship.len());
        // and they get there: give the day its frames
        let n = g.ticks_per_frame() - 2; walk_frames(&mut g, n);
        let there = worship.iter().filter(|&&v| {
            let r = g.world.get_component::<Routine>(v).unwrap();
            let p = g.world.get_component::<Pos>(v).unwrap();
            (p.x - r.anchor.0).abs() + (p.y - r.anchor.1).abs() < 1.0
        }).count();
        assert!(there * 2 >= worship.len(), "most should reach their places: {} of {}", there, worship.len());
        // slots are distinct
        let mut tiles: Vec<(i32, i32)> = worship.iter().map(|&v| {
            let r = g.world.get_component::<Routine>(v).unwrap();
            ((r.anchor.0 * 2.0).round() as i32, (r.anchor.1 * 2.0).round() as i32)
        }).collect();
        tiles.sort(); tiles.dedup();
        assert_eq!(tiles.len(), worship.len(), "no two worshippers share a place");
    }

    #[test]
    fn test_no_work_on_the_holy_day() {
        let mut g = Game::new();
        let mut rng = Rng(5);
        let v = g.villagers.iter().copied().find(|&v| g.bench_of(v).is_some()).unwrap();
        for _ in 0..30 {
            assert_ne!(g.plan_day(v, HOLY_EVERY * 3, &mut rng), DayPlan::Work);
        }
    }

    #[test]
    fn test_a_free_day_goes_somewhere_named() {
        let mut g = Game::new();
        let mut rng = Rng(9);
        let v = g.villagers[0];
        assert!(g.plan_stroll(v, 1, &mut rng), "somewhere should be in reach");
        let r = g.world.get_component::<Routine>(v).unwrap();
        assert!(matches!(r.spot, Spot::Notices | Spot::Creek | Spot::Woods | Spot::Stones | Spot::Tavern), "{:?}", r.spot);
        assert!(routine_text(&g.world, v, None).is_some());
    }
}
