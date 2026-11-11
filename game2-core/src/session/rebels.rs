//! Rebel bands in the woods (#652): outlaws living rough round a camp
//! in the trees, tagged `Rebel` - and `Outlander` at a sentinel site,
//! so every country-folk exclusion (the food rolls' reach, meetings,
//! quests, the collector's roll, recruiting, the fire bell) covers
//! them without a second audit. They work no trade and answer to
//! nobody; their day is the woods, and now and then a country's
//! caravan pays for passing close.

use super::*;

/// Which band a rebel rides with - `Game::rebel_camps` is indexed the
/// same, so the camp anchor is one lookup away
#[derive(Debug, Clone, Copy)]
pub struct Rebel { pub band: usize }

/// A cache a band leaves at the woods' edge after a good hit: stolen
/// goods and lifted gold for whoever finds them first - the player's
/// Take verb when they stand by it
#[derive(Debug, Clone)]
pub struct RebelStore {
    pub tile: (i32, i32),
    /// (good, count) pairs - satchel goods' own names
    pub goods: Vec<(&'static str, u32)>,
    /// coin taken out of a gold ride (#970)
    pub gold: f32,
    /// pickable until this day
    pub until: u64,
}

/// The woods' own chance-stream salt - band rolls, ambush rolls and
/// cache rolls come off it, never off `rng`, so the woods don't shift
/// what the village would have rolled
const REBEL_SALT: u64 = 0xBA7A_5EED;
/// A band's chance of falling on a caravan it meets, per day met
const RAID_CHANCE: i32 = 35;
/// How close a rebel must stand to a traveler for the ambush roll
const RAID_REACH: f32 = 3.0;
/// A wander day's walk reaches this far from wherever they stand
const REBEL_REACH: i32 = 16;
/// How many days a left cache keeps, plus a few
const CACHE_DAYS: u64 = 6;
/// An escort standing this close answers the ambush for its convoy
const ESCORT_REACH: f32 = 8.0;
/// The escort roll's own salt - kept off REBEL_SALT so an escort's
/// answer doesn't move the band's ambush rolls
const ESCORT_SALT: u64 = 0xE5C0_7EAD;

/// The goals that carry the country's wealth on the road - carts,
/// provisions rides, remittances, the crown's levy cart and the
/// tribute trains. Escort rides guard but carry nothing, and a Levy
/// rides empty out
fn raid_goal(g: &Goal) -> bool {
    matches!(g, Goal::Cart(_) | Goal::Levy(..) | Goal::Deliver(_)
        | Goal::Remit(..) | Goal::Owe(..) | Goal::Gift(..) | Goal::LevyCart(_))
}

/// Which site a raid goal is bound for - the village is MAX, same
/// sentinel the market's escorts carry
fn dest_site(g: &Goal) -> usize {
    match g {
        Goal::Remit(to, _) | Goal::Owe(_, to, _) | Goal::Gift(_, to)
        | Goal::Deliver(to) | Goal::LevyCart(to) => *to,
        _ => usize::MAX,
    }
}

/// A stable u64 for one (band, traveler, day, salt) meeting - the same
/// inputs always mix the same, so a roll can't flicker between frames
fn meet_seed(band: usize, who: Entity, tick: u64, salt: u64) -> u64 {
    (band as u64).wrapping_mul(0x9E37_79B9)
        ^ (who as u64).wrapping_mul(0x85EB_CA6B)
        ^ tick.wrapping_mul(0xC2B2_AE3D) ^ salt
}

impl Game {
    /// Settle the rebels the seed dealt: nothing, one band or two,
    /// each camping a clearing in the woods with three to five souls.
    /// Appended after the country's folk - the roster's own order holds
    pub(super) fn settle_rebels(&mut self, seed: u64) {
        let mut rng = Rng(seed ^ REBEL_SALT);
        let bands = match rng.below(100) { 0..=9 => 2, 10..=54 => 1, _ => 0 };
        for band in 0..bands {
            let Some(camp) = self.rebel_camp_site(&mut rng) else { continue };
            let size = (3 + rng.below(3)) as usize;
            self.rebel_camps.push(camp);
            for k in 0..size {
                self.spawn_rebel(band, camp, k, &mut rng);
            }
        }
    }

    /// A clearing in the woods for a band's camp: a passable tile with
    /// trees at its shoulders, away from the other camps
    fn rebel_camp_site(&self, rng: &mut Rng) -> Option<(i32, i32)> {
        let trees: Vec<(i32, i32)> = self.map.trees.iter().copied().collect();
        if trees.is_empty() { return None; }
        for _ in 0..40 {
            let &(tx, ty) = &trees[rng.below(trees.len() as i32) as usize];
            // spiral out from the tree for a clearing that still smells
            // of wood: passable, treeless itself, trees close by
            'probe: for r in 1..=4 {
                for (dx, dy) in [(0, 0), (r, 0), (-r, 0), (0, r), (0, -r),
                        (r, r), (-r, -r), (r, -r), (-r, r)] {
                    let at = (tx + dx, ty + dy);
                    if !self.map.passable(at.0, at.1) || self.map.trees.contains(&at) { continue; }
                    let near_trees = self.map.trees.iter()
                        .filter(|&&(x, y)| (x - at.0).abs() + (y - at.1).abs() <= 3).count();
                    if near_trees < 2 { continue; }
                    if self.rebel_camps.iter().any(|&c|
                        (c.0 - at.0).abs() + (c.1 - at.1).abs() < 14) { continue 'probe; }
                    return Some(at);
                }
            }
        }
        None
    }

    /// One of the band: everything a villager carries, homed nowhere -
    /// no trade, no bed, no hunger; they eat what they take
    fn spawn_rebel(&mut self, band: usize, camp: (i32, i32), k: usize, rng: &mut Rng) {
        // a spot on the camp's edge, spread a little so they don't stack
        let spots: Vec<(i32, i32)> = (0..=3)
            .flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (camp.0 + dx, camp.1 + dy))))
            .filter(|&(x, y)| self.map.passable(x, y)).collect();
        let at = spots.get((k * 5 + rng.below(3) as usize) % spots.len().max(1)).copied().unwrap_or(camp);
        let v = villager::spawn_villager(&mut self.world, at.0 as f32 + 0.5, at.1 as f32 + 0.5);
        // the woods feed them: no hunger means no seeking, no starving,
        // and no place in the village's food rolls
        self.world.remove_component::<villager::Hunger>(v);
        let i = self.villagers.len();
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32 % 6, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
        self.world.add_component(v, crate::sim::psychology::personality_from_seed(i as u64));
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(i as u64 * 7 + 3));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        self.world.add_component(v, Rebel { band });
        // the sentinel site: every "is this country folk" check reads
        // them as outlanders; `site_of` filters the index away, so no
        // site ever claims them
        self.world.add_component(v, country::Outlander { site: usize::MAX });
        let years = 19.0 + (rng.next() % 42) as f32;
        self.world.add_component(v, crate::game::lifecycle::Age {
            years, stage: crate::game::lifecycle::stage_for(years),
            born_day: (v as u16) % 360 });
        self.world.add_component(v, crate::game::lifecycle::Vitality {
            health: 1.0, max_age: 58.0 + (rng.next() % 35) as f32, cause_of_death: None });
        self.world.add_component(v, Purse { gold: (rng.next() % 15) as f32 });
        self.world.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        self.villagers.push(v);
        self.was_sick.push(None);
        self.died_on.push(None);
    }

    /// A rebel's day: drift the woods round their camp - no work, no
    /// worship, no market; the trees are the whole of it
    pub(super) fn plan_rebel_day(&mut self, v: Entity, tick: u64, rng: &mut Rng) {
        let band = self.world.get_component::<Rebel>(v).map(|r| r.band).unwrap_or(0);
        let Some(&(cx, cy)) = self.rebel_camps.get(band) else { return };
        let at = self.world.get_component::<Pos>(v).map(|p| (p.x, p.y)).unwrap_or((cx as f32, cy as f32));
        for _ in 0..14 {
            let t = (cx + rng.below(21) - 10, cy + rng.below(21) - 10);
            if !self.map.passable(t.0, t.1) { continue; }
            let near = (at.0 - t.0 as f32).abs() + (at.1 - t.1 as f32).abs() <= 1.5;
            if !near && !self.send_tile_walk(v, t, Goal::Wander, REBEL_REACH) { continue; }
            self.set_routine(v, routine::Routine { mode: routine::Mode::Wandering, spot: routine::Spot::Woods,
                anchor: (t.0 as f32, t.1 as f32), face: None, drift: 1.4,
                leads: false, day: tick, reached: false });
            return;
        }
        // nowhere walkable drew: hold the camp itself
        self.set_routine(v, routine::Routine { mode: routine::Mode::Wandering, spot: routine::Spot::Woods,
            anchor: (cx as f32, cy as f32), face: None, drift: 0.8,
            leads: false, day: tick, reached: false });
    }

    /// Per-frame, after the travels step: a rebel standing close to a
    /// country caravan may fall on it - the roll is the band's own, once
    /// a day per traveler, so a long escort can't be farmed
    pub fn rebel_ambush(&mut self) {
        // caches past their day are done
        self.rebel_stores.retain(|s| self.tick < s.until);
        // no bands, no work - most seeds deal none
        if self.rebel_camps.is_empty() { return; }
        let rebels: Vec<(Entity, usize, f32, f32)> = self.villagers.iter()
            .filter_map(|&v| {
                let b = self.world.get_component::<Rebel>(v)?.band;
                let p = self.world.get_component::<Pos>(v)?;
                Some((v, b, p.x, p.y))
            }).collect();
        if rebels.is_empty() { return; }
        let mut hits: Vec<(usize, usize)> = vec![];
        let mut met: crate::engine::det::HashSet<(usize, Entity)> = Default::default();
        for (ti, t) in self.travels.iter().enumerate() {
            if !raid_goal(&t.goal) { continue; }
            let Some(tp) = self.world.get_component::<Pos>(t.who) else { continue };
            for &(rv, band, rx, ry) in &rebels {
                if villager::is_dead(&self.world, rv) { continue; }
                if (tp.x - rx).powi(2) + (tp.y - ry).powi(2) > RAID_REACH * RAID_REACH { continue; }
                if !met.insert((band, t.who)) { continue; }
                if Rng(meet_seed(band, t.who, self.tick, REBEL_SALT)).below(100) >= RAID_CHANCE { continue; }
                hits.push((ti, band));
                break;
            }
        }
        for (ti, band) in hits { self.rob_caravan(ti, band); }
    }

    /// The band falls on the caravan: a riding escort may drive them
    /// off; else the claimant loses the cargo either way - carried off
    /// to a cache when the haul's worth hiding, scattered when it
    /// isn't (#970)
    fn rob_caravan(&mut self, ti: usize, band: usize) {
        let Some(t) = self.travels.get(ti) else { return };
        let (who, goal) = (t.who, t.goal.clone());
        let Some(at) = self.world.get_component::<Pos>(who).map(|p| (p.x, p.y)) else { return };
        let at_tile = (at.0.round() as i32, at.1.round() as i32);
        let name = self.villagers.iter().position(|&x| x == who).map(vill_name).unwrap_or("a traveller");
        // the convoy's escort answers first: a guard bound for the same
        // door standing close enough reads the ambush and stands to it
        let escorted = self.travels.iter().any(|e| e.who != who
            && matches!(e.goal, Goal::Escort(_, to) if to == dest_site(&goal))
            && !villager::is_dead(&self.world, e.who)
            && self.world.get_component::<Pos>(e.who)
                .map(|p| (p.x - at.0).powi(2) + (p.y - at.1).powi(2) <= ESCORT_REACH * ESCORT_REACH)
                .unwrap_or(false));
        if escorted && Rng(meet_seed(band, who, self.tick, ESCORT_SALT)).below(100) < 60 {
            let members: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&v| self.world.get_component::<Rebel>(v).map(|r| r.band) == Some(band))
                .collect();
            let mut rng = Rng(meet_seed(band, who, self.tick, REBEL_SALT ^ 0x5CA7_1A9E));
            for &v in &members {
                if !villager::is_dead(&self.world, v) { self.plan_rebel_day(v, self.tick, &mut rng); }
            }
            self.log.push_at_who(format!("t{}: rebels come out of the trees at {} - the escort stands to them and they melt away.", self.tick, name),
                Some((at.0, at.1)), self.villagers.iter().position(|&x| x == who));
            return;
        }
        // turn them back first: if the road home can't be had the band
        // lets them pass - the take only counts when the errand's off
        let home_door = household::home_of(&self.world, who)
            .and_then(|h| self.world.get_component::<Pos>(h).copied()
                .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32))
                .map(|d| (h, d)));
        let Some((home, door)) = home_door else { return; };
        if !self.send_far(who, door, Goal::HeadHome(home)) { return; }
        // the take: what the goal says is riding, out of the satchel -
        // the gold goals carry their take in the errand itself, so
        // turning back loses it whole
        let mut took: Vec<(&'static str, u32)> = vec![];
        let keys: &[&'static str] = match &goal {
            Goal::Cart(_) => &["wheat", "ore"],
            Goal::Deliver(_) | Goal::LevyCart(_) => &["provisions"],
            Goal::Gift(..) => &["wheat", "tools"],
            _ => &[],
        };
        if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
            for &k in keys {
                if let Some(n) = s.goods.remove(k) { if n > 0 { took.push((k, n)); } }
            }
        }
        let gold = match &goal {
            Goal::Remit(_, g) | Goal::Owe(.., g) => *g,
            _ => 0.0,
        };
        // the band melts back into the woods
        let members: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| self.world.get_component::<Rebel>(v).map(|r| r.band) == Some(band))
            .collect();
        let mut rng = Rng(meet_seed(band, who, self.tick, REBEL_SALT ^ 0x5CA7_1A9E));
        for &v in &members {
            if !villager::is_dead(&self.world, v) { self.plan_rebel_day(v, self.tick, &mut rng); }
        }
        let what = match &goal {
            Goal::Cart(_) => "the cart's load",
            Goal::Levy(..) => "the steward's ride",
            Goal::Deliver(_) => "the provisions",
            Goal::LevyCart(_) => "the crown's cart",
            Goal::Remit(..) | Goal::Owe(..) => "the gold",
            Goal::Gift(..) => "the gift train",
            _ => "the errand",
        };
        // spoils or denial: the claimant loses it either way - the band
        // profits only when the take's worth hiding in the trees
        let spoils = (!took.is_empty() || gold > 0.0)
            && rng.below(100) < 55
            && self.cache_spot(at_tile).is_some();
        let line = if took.is_empty() && gold == 0.0 {
            format!("t{}: rebels fall on {} on the road - {} is turned back.", self.tick, name, what)
        } else if spoils {
            let goods = took.iter().map(|&(g, n)| format!("{} {}", n, g)).collect::<Vec<_>>().join(" and ");
            let taken = if gold > 0.0 && !goods.is_empty() { format!("{:.0} gold and {}", gold, goods) }
                else if gold > 0.0 { format!("{:.0} gold", gold) } else { goods };
            format!("t{}: rebels fall on {} on the road - {} carried off: {}.", self.tick, name, what, taken)
        } else {
            format!("t{}: rebels fall on {} on the road - {} is scattered: nothing comes through.", self.tick, name, what)
        };
        self.log.push_at_who(line, Some((at.0, at.1)),
            self.villagers.iter().position(|&x| x == who));
        self.toasts.push_at((format!("Rebels hit {} on the road!", name), self.tick + 40), Some((at.0, at.1)));
        if spoils {
            let tile = self.cache_spot(at_tile).unwrap();
            self.rebel_stores.push(RebelStore { tile, goods: took, gold,
                until: self.tick + CACHE_DAYS + rng.below(6) as u64 });
        }
    }

    /// A passable tile by the trees near `at` for a cache to sit on
    fn cache_spot(&self, at: (i32, i32)) -> Option<(i32, i32)> {
        (0..=4).flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (at.0 + dx, at.1 + dy))))
            .find(|&(x, y)| self.map.passable(x, y)
                && self.map.trees.iter().any(|&(tx, ty)| (tx - x).abs() + (ty - y).abs() <= 3))
    }

    /// The store the seat's player stands by, if any - the Take verb's
    /// reach is the same arm's length the shroom patches use
    pub fn rebel_store_at(&self, seat: usize) -> Option<usize> {
        let p = self.world.get_component::<Pos>(self.seats[seat].player)?;
        self.rebel_stores.iter().position(|s| self.tick < s.until
            && (p.x - s.tile.0 as f32 - 0.5).abs() + (p.y - s.tile.1 as f32 - 0.5).abs() <= 1.6)
    }
}
