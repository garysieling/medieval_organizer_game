//! The country past the walls: market towns, castles, remote farms and a
//! monastery placed fresh for every map, the folk who live in them, and
//! the carts on the roads between.
//!
//! Each seed draws its own country: the towns stand on dry lowland, the
//! castles on the high ground, the farms and the abbey out on their own
//! ground, all spread apart, clear of the river and on land that walks
//! back to the village. Townsfolk farm and send a cart to the village
//! market each month; the castles hold the knights and a chaplain, and
//! their steward rides to the nearest town for provisions. The farms and
//! the abbey keep their own stores and their own company. The home
//! village keeps its spot in the corner (every system is laid out round
//! its market), and its own folk sort themselves too: farmers and herders
//! take the cottages out among the fields, the gentry the houses inside
//! the wall.

use super::*;
use crate::game::tilemap::WATER_LEVEL;

/// Carts and horses cover this many times a walker's ground
pub const RIDE_PACE: f32 = 2.0;
/// A day on the road by cart: MEETING_WALK tiles a second, five seconds a day
const TILES_PER_DAY: f32 = 12.5 * RIDE_PACE;
/// A castle stands within this many tiles of the town that keeps it
const CASTLE_TO_TOWN: f32 = 75.0;
/// How many days a cart stays at the market before heading home
const MARKET_STAY: u64 = 2;
/// How often a castle's steward rides out for provisions
const LEVY_EVERY: u64 = 20;
/// The most a steward brings back in one trip
const LEVY_TAKE: u32 = 12;
/// A town's granary tops out here; each working townsman adds a serving a day
const TOWN_STORE_CAP: u32 = 320;
/// Strays farther than this from their own town walk home
const STRAY_FROM_SITE: f32 = 12.0;

const TOWN_NAMES: [&str; 8] = ["Ashby", "Thornford", "Marrowdale", "Kelby",
    "Holloway", "Wenlock", "Brackenhurst", "Oakham"];
const CASTLE_NAMES: [&str; 6] = ["Ravensholt", "Greymoor", "Dunmore",
    "Blackcrag", "Highwold", "Corvel"];
const FARM_NAMES: [&str; 6] = ["Ashford", "Hartley", "Thornby",
    "Greenshaw", "Longacre", "Fordley"];
const HOLY_NAMES: [&str; 4] = ["Greyfriars", "Frosterley", "Ingleby", "Halewick"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteKind { Town, Castle, Farm, Monastery }

/// A settlement out in the country, and who keeps it
#[derive(Clone, Debug)]
pub struct Site {
    pub name: &'static str,
    pub kind: SiteKind,
    pub centre: (i32, i32),
    pub radius: i32,
    /// The granary (a FoodSource entity) its folk eat from
    pub store: Entity,
    /// Its houses (Home buildings), the keep first for a castle
    pub homes: Vec<Entity>,
    /// Who drives the cart (a town) or rides for provisions (a castle)
    pub carrier: Option<Entity>,
    /// While the carrier's away: the day they turn for home
    pub away_until: Option<u64>,
}

/// Lives out at `sites[site]`, not in the village
pub struct Outlander { pub site: usize }

/// The country a seed draws, before the map's made: where each town and
/// castle stands and where the town houses go
pub struct CountryPlan {
    pub towns: Vec<(i32, i32, i32)>,
    pub castles: Vec<(i32, i32, i32)>,
    pub farms: Vec<(i32, i32, i32)>,
    pub monasteries: Vec<(i32, i32, i32)>,
    pub town_names: Vec<&'static str>,
    pub castle_names: Vec<&'static str>,
    pub farm_names: Vec<&'static str>,
    pub monastery_names: Vec<&'static str>,
    /// House sites round the towns (decorative scenery and real homes)
    pub houses: Vec<(i32, i32)>,
}

/// Distance from `p` to the segment a-b
fn to_segment(p: (i32, i32), a: (i32, i32), b: (i32, i32)) -> f32 {
    let (px, py, ax, ay, bx, by) = (p.0 as f32, p.1 as f32, a.0 as f32, a.1 as f32, b.0 as f32, b.1 as f32);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = (dx * dx + dy * dy).max(1e-4);
    let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
}

/// How near `p` comes to the river: its waypoints, then on the same way
/// to the map's edge (where carve_river takes it)
fn to_river(p: (i32, i32), river: &[(i32, i32)], size: i32) -> f32 {
    let mut best = f32::MAX;
    for w in river.windows(2) { best = best.min(to_segment(p, w[0], w[1])); }
    if river.len() >= 2 {
        let (a, b) = (river[river.len() - 2], river[river.len() - 1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        // carve_river steps along whichever way is further to go
        let step = if dx.abs() > dy.abs() { (dx.signum(), 0) } else { (0, dy.signum()) };
        let end = (b.0 + step.0 * size, b.1 + step.1 * size);
        best = best.min(to_segment(p, b, end));
    }
    best
}

/// Dry land joined to the village on foot (sea and lakes cut it; the
/// river doesn't, it has a bridge), as the terrain stands before the map
pub(super) fn reachable_land(t: &Terrain, from: (i32, i32)) -> Vec<bool> {
    let (w, h) = (t.width as i32, t.height as i32);
    let mut seen = vec![false; (w * h) as usize];
    let dry = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && t.get(x as usize, y as usize) >= WATER_LEVEL + 0.02;
    let mut q = std::collections::VecDeque::from([from]);
    seen[(from.1 * w + from.0) as usize] = true;
    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in NEIGHBOURS {
            let (nx, ny) = (x + dx, y + dy);
            if !dry(nx, ny) || seen[(ny * w + nx) as usize] { continue; }
            seen[(ny * w + nx) as usize] = true;
            q.push_back((nx, ny));
        }
    }
    seen
}

/// Draw the country for this seed: two market towns on the lowland and
/// two castles on the heights, each spread from the rest
pub fn plan_country(t: &Terrain, river: &[(i32, i32)], village: (i32, i32), rng: &mut Rng) -> CountryPlan {
    let size = t.width as i32;
    let land = reachable_land(t, village);
    let (town_r, castle_r) = (8, 5);
    let mut taken: Vec<(i32, i32, i32)> = vec![(village.0, village.1, 30)];
    // whether a site of radius r could stand at c: on the map with room
    // to spare, clear of the river and the other sites, and on dry land
    // the village can walk to (sampled over its footprint)
    let fits = |c: (i32, i32), r: i32, taken: &[(i32, i32, i32)]| -> Option<f32> {
        let margin = r + 6;
        if c.0 < margin || c.1 < margin || c.0 >= size - margin || c.1 >= size - margin { return None; }
        if to_river(c, river, size) < (r + 5) as f32 { return None; }
        for &(x, y, tr) in taken {
            let d = (((c.0 - x).pow(2) + (c.1 - y).pow(2)) as f32).sqrt();
            if d < (tr + r + 34) as f32 { return None; }
        }
        let mut sum = 0.0;
        let mut n = 0.0;
        for dy in (-r - 1..=r + 1).step_by(2) {
            for dx in (-r - 1..=r + 1).step_by(2) {
                let (x, y) = (c.0 + dx, c.1 + dy);
                let h = t.get(x as usize, y as usize);
                if h < WATER_LEVEL + 0.04 || !land[(y * size + x) as usize] { return None; }
                sum += h;
                n += 1.0;
            }
        }
        Some(sum / n)
    };
    // the best of a few dozen draws: towns want the low ground, castles
    // the high
    let pick = |r: i32, high: bool, near: &[(i32, i32, i32)], taken: &mut Vec<(i32, i32, i32)>, rng: &mut Rng| -> Option<(i32, i32, i32)> {
        let mut best: Option<((i32, i32), f32)> = None;
        for _ in 0..200 {
            let c = (r + 6 + rng.below(size - 2 * (r + 6)), r + 6 + rng.below(size - 2 * (r + 6)));
            // a castle keeps watch over a town: within a ride of one
            if !near.is_empty() && !near.iter().any(|&(x, y, _)|
                (((c.0 - x).pow(2) + (c.1 - y).pow(2)) as f32).sqrt() <= CASTLE_TO_TOWN) { continue; }
            let Some(h) = fits(c, r, taken) else { continue };
            let score = if high { h } else { -(h - 0.35).abs() };
            if best.map(|(_, s)| score > s).unwrap_or(true) { best = Some((c, score)); }
        }
        let (c, _) = best?;
        taken.push((c.0, c.1, r));
        Some((c.0, c.1, r))
    };
    let mut towns = vec![];
    let mut castles = vec![];
    for _ in 0..2 { if let Some(s) = pick(town_r, false, &[], &mut taken, rng) { towns.push(s); } }
    // each castle over its own town where it can
    for k in 0..2 {
        let near: Vec<(i32, i32, i32)> = towns.get(k).or(towns.first()).copied().into_iter().collect();
        if let Some(s) = pick(castle_r, true, &near, &mut taken, rng).or_else(|| pick(castle_r, true, &[], &mut taken, rng)) {
            castles.push(s);
        }
    }
    // and the remote steadings: farms on the lowland, a monastery or two
    // out on their own ground - spread from everything else like the rest
    let farm_r = 4;
    let mut farms = vec![];
    for _ in 0..3 { if let Some(s) = pick(farm_r, false, &[], &mut taken, rng) { farms.push(s); } }
    let mut monasteries = vec![];
    for _ in 0..1 + rng.below(2) {
        if let Some(s) = pick(castle_r, false, &[], &mut taken, rng) { monasteries.push(s); }
    }
    // names: drawn without repeats
    let mut tn: Vec<&'static str> = TOWN_NAMES.to_vec();
    let mut cn: Vec<&'static str> = CASTLE_NAMES.to_vec();
    let mut fnames: Vec<&'static str> = FARM_NAMES.to_vec();
    let mut mnames: Vec<&'static str> = HOLY_NAMES.to_vec();
    let town_names = towns.iter().map(|_| tn.remove(rng.below(tn.len() as i32) as usize)).collect();
    let castle_names = castles.iter().map(|_| cn.remove(rng.below(cn.len() as i32) as usize)).collect();
    let farm_names = farms.iter().map(|_| fnames.remove(rng.below(fnames.len() as i32) as usize)).collect();
    let monastery_names = monasteries.iter().map(|_| mnames.remove(rng.below(mnames.len() as i32) as usize)).collect();
    // five to seven houses round each town's green, a loose ring
    let mut houses = vec![];
    for &(cx, cy, _) in &towns {
        let n = 5 + rng.below(3);
        let turn = rng.below(360) as f32;
        for k in 0..n {
            let a = (turn + k as f32 * 360.0 / n as f32).to_radians();
            let rr = 3.0 + rng.below(3) as f32;
            houses.push(((cx as f32 + rr * a.cos()).round() as i32, (cy as f32 + rr * a.sin()).round() as i32));
        }
    }
    CountryPlan { towns, castles, farms, monasteries, town_names, castle_names,
        farm_names, monastery_names, houses }
}

/// Who lives out in the country, site by site. A market town is a
/// household trade: farmers work the fields, the rest keep the fold,
/// loom, river, bakehouse and tavern, and children play underfoot
/// (`None` is a child, not a trade). A castle is a garrison: the lord's
/// blade first, the provisions-rider second (that's the carrier slot),
/// then knights and - at the first castle only - its chaplain
fn site_folk(kind: SiteKind, nth: usize) -> Vec<Option<Vocation>> {
    const TOWN: [Option<Vocation>; 13] = [
        Some(Vocation::Farmer), Some(Vocation::Farmer), Some(Vocation::Farmer),
        Some(Vocation::Farmer), Some(Vocation::Shepherd), Some(Vocation::Weaver),
        Some(Vocation::Fisher), Some(Vocation::Taverner), Some(Vocation::Baker),
        Some(Vocation::Beekeeper), Some(Vocation::Farmer), None, None,
    ];
    const CASTLE_GENTRY: [Option<Vocation>; 7] = [
        Some(Vocation::Knight), Some(Vocation::Knight), Some(Vocation::Priest),
        Some(Vocation::Knight), Some(Vocation::Knight), Some(Vocation::Priest),
        Some(Vocation::Knight),
    ];
    const CASTLE_BLADES: [Option<Vocation>; 7] = [
        Some(Vocation::Knight), Some(Vocation::Knight), Some(Vocation::Knight),
        Some(Vocation::Knight), Some(Vocation::Knight), Some(Vocation::Knight),
        Some(Vocation::Knight),
    ];
    match kind {
        SiteKind::Town => TOWN.to_vec(),
        SiteKind::Castle if nth == 0 => CASTLE_GENTRY.to_vec(),
        SiteKind::Castle => CASTLE_BLADES.to_vec(),
        // a farmstead: the hands and the shepherd, and children underfoot
        SiteKind::Farm => [
            Some(Vocation::Farmer), Some(Vocation::Farmer), Some(Vocation::Farmer),
            Some(Vocation::Shepherd), None, None,
        ].to_vec(),
        // the abbot, his prior, three lay brothers, and a novice underfoot
        SiteKind::Monastery => [
            Some(Vocation::Priest), Some(Vocation::Priest), Some(Vocation::Farmer),
            Some(Vocation::Farmer), Some(Vocation::Farmer), None,
        ].to_vec(),
    }
}

/// How many villagers the country holds (for the roster's other lists)
pub fn country_folk(plan: &CountryPlan) -> usize {
    plan.towns.len() * site_folk(SiteKind::Town, 0).len()
        + plan.castles.iter().enumerate()
            .map(|(i, _)| site_folk(SiteKind::Castle, i).len()).sum::<usize>()
        + plan.farms.len() * site_folk(SiteKind::Farm, 0).len()
        + plan.monasteries.len() * site_folk(SiteKind::Monastery, 0).len()
}

impl Game {
    /// Settle the country the plan drew: a granary and homes at each
    /// site, and its folk in them (appended to the roster after the
    /// village's own - the village's order is fixed)
    pub(super) fn settle_country(&mut self, plan: &CountryPlan, seed: u64) {
        let mut rng = Rng(seed ^ 0xC0_FFEE);
        let mut town_houses = self.map.settlement_houses.clone();
        let mut sites: Vec<Site> = vec![];
        let towns = plan.towns.iter().zip(&plan.town_names).map(|(t, n)| (SiteKind::Town, *t, *n));
        let castles = plan.castles.iter().zip(&plan.castle_names).map(|(c, n)| (SiteKind::Castle, *c, *n));
        let farms = plan.farms.iter().zip(&plan.farm_names).map(|(f, n)| (SiteKind::Farm, *f, *n));
        let monasteries = plan.monasteries.iter().zip(&plan.monastery_names).map(|(m, n)| (SiteKind::Monastery, *m, *n));
        for (kind, (cx, cy, r), name) in towns.chain(castles).chain(farms).chain(monasteries) {
            // the granary stands on the green (a castle keeps its stores
            // in the courtyard, beside the keep); it's sited after the
            // roofs below so it keeps clear of them
            let green = if kind == SiteKind::Castle { (cx, cy + 1) } else { (cx, cy) };
            let servings = match kind {
                SiteKind::Town => 200, SiteKind::Castle => 140,
                SiteKind::Farm => 160, SiteKind::Monastery => 100,
            };
            let mut homes = vec![];
            match kind {
                SiteKind::Town => {
                    // this town's houses: the plan's ring round its green
                    let (mine, rest): (Vec<_>, Vec<_>) = town_houses.iter().copied()
                        .partition(|&(x, y)| (x - cx).abs() <= r + 2 && (y - cy).abs() <= r + 2);
                    town_houses = rest;
                    for (x, y) in mine {
                        let label = format!("a cottage in {}", name);
                        homes.push(self.country_building(BuildingKind::Home, label, x, y));
                    }
                }
                SiteKind::Castle => {
                    let keep = self.country_building(BuildingKind::Home, format!("the keep at {}", name), cx, cy);
                    homes.push(keep);
                    // and the hall beside it: a garrison of seven under
                    // one roof never fills a cradle
                    let hall_c = (cx + 2, cy);
                    let hall_at = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (hall_c.0 + dx, hall_c.1 + dy)))
                        .filter(|&(x, y)| self.map.passable(x, y))
                        .min_by_key(|&(x, y)| (x - hall_c.0).abs() + (y - hall_c.1).abs())
                        .unwrap_or(hall_c);
                    let hall = self.country_building(BuildingKind::Home, format!("the hall at {}", name), hall_at.0, hall_at.1);
                    homes.push(hall);
                    // and it must be seen: a hall nobody can see is no hall
                    self.buildings.push((hall_at.0 as f32, hall_at.1 as f32,
                        BSprite::House((noise(hall_at.0, hall_at.1, 41) * 97.0) as usize)));
                }
                SiteKind::Farm => {
            // the farmstead: strips of crop in the lee of the buildings,
            // a farmhouse and its byre - the Carcassonne farm, not a cottage
            for dy in 2..=5 {
                for dx in -4..=4 {
                    let (x, y) = (cx + dx, cy + dy);
                    if matches!(self.map.get(x, y), Some(Tile::Grass)) && !self.map.is_occupied(x, y) {
                        self.map.set(x, y, if dx % 3 == 0 { Tile::Corn } else { Tile::Wheat });
                    }
                }
            }
            for (k, (dx, dy)) in [(0, 0), (2, 1)].iter().enumerate() {
                let Some((bx, by)) = self.map.nearest_buildable(cx + dx, cy + dy) else { continue };
                let label = if k == 0 { format!("the farm at {}", name) } else { format!("the byre at {}", name) };
                homes.push(self.country_building(BuildingKind::Home, label, bx, by));
                self.buildings.push((bx as f32, by as f32,
                    BSprite::House((noise(bx, by, 41) * 97.0) as usize)));
                self.map.connect((bx, by));
            }
                }
                SiteKind::Monastery => {
            // the abbey, drawn as a church overseas rather than a cottage
            if let Some((bx, by)) = self.map.nearest_buildable(cx, cy) {
                homes.push(self.country_building(BuildingKind::Home,
                    format!("the abbey at {}", name), bx, by));
                self.buildings.push((bx as f32, by as f32, BSprite::Church));
                self.map.connect((bx, by));
            }
                }
            }
            let store_at = (-3..=3).flat_map(|dy| (-3..=3).map(move |dx| (green.0 + dx, green.1 + dy)))
                .filter(|&(x, y)| self.map.passable(x, y))
                .min_by_key(|&(x, y)| (x - green.0).abs() + (y - green.1).abs())
                .unwrap_or(green);
            let store = villager::spawn_food_source(&mut self.world, store_at.0 as f32, store_at.1 as f32, servings);
            sites.push(Site { name, kind, centre: (cx, cy), radius: r, store, homes, carrier: None, away_until: None });
        }
        // the folk, a household to a site under their own roofs; the
        // roster puts children last so the carrier slots - the first
        // farmer, the second knight - always land on grown folk
        let mut castles_seen = 0;
        let mut towns_seen = 0;
        for si in 0..sites.len() {
            let kind = sites[si].kind;
            let nth = if kind == SiteKind::Castle { castles_seen } else { towns_seen };
            if kind == SiteKind::Castle { castles_seen += 1; } else { towns_seen += 1; }
            let folk = site_folk(kind, nth);
            let homes = sites[si].homes.clone();
            if homes.is_empty() { continue; }
            for (k, spec) in folk.into_iter().enumerate() {
                let home = homes[k % homes.len()];
                let v = self.spawn_outlander(si, spec, home, k, &mut rng);
                // only the towns and castles run errands to the village:
                // a farm's surplus and an abbey's alms stay home
                if matches!(kind, SiteKind::Town | SiteKind::Castle)
                    && k == if kind == SiteKind::Town { 0 } else { 1 } {
                    sites[si].carrier = Some(v);
                }
            }
        }
        self.sites = sites;
    }

    /// A Building entity out in the country (its sprite is the map's
    /// settlement scenery, or the keep - not the village's building list).
    /// The tile is marked built-on so folk walk round it, not through
    fn country_building(&mut self, kind: BuildingKind, name: String, x: i32, y: i32) -> Entity {
        self.map.occupy(x, y);
        let b = household::spawn_building(&mut self.world, kind, "a house", x as f32, y as f32, 20);
        if let Some(bb) = self.world.get_component_mut::<Building>(b) { bb.name = name; }
        self.building_entities.push(b);
        b
    }

    /// One of the country's folk: everything a villager carries, homed
    /// at `home` out at site `si`. `None` is a child - no trade until
    /// they come of age, like the village's own young
    fn spawn_outlander(&mut self, si: usize, voc: Option<Vocation>, home: Entity, k: usize, rng: &mut Rng) -> Entity {
        let hp = self.world.get_component::<Pos>(home).map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or((0, 0));
        let door = self.at_the_door(hp.0, hp.1).unwrap_or(hp);
        let v = villager::spawn_villager(&mut self.world, door.0 as f32 + k as f32 * 0.4, door.1 as f32 + 0.3);
        let i = self.villagers.len();
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32 % 6, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(i as u64 * 7 + 3));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        match voc {
            // the lord of a castle is a master of arms; the rest are rolled
            Some(voc) => {
                let skill = if voc == Vocation::Knight && k == 0 { 0.85 } else { 0.3 + rng.below(40) as f32 / 100.0 };
                self.world.add_component(v, vocation::craft(voc, skill));
            }
            None => {
                self.world.add_component(v, Kid);
                // staggered ages, like the village's - they come of age one by one
                self.world.add_component(v, KidGrowth { age_days: (k as u64 * 211) % 700, errands: vec![] });
            }
        }
        self.world.add_component(v, Outlander { site: si });
        if let Some(b) = self.world.get_component_mut::<Building>(home) { b.occupants.push(v); }
        let class = desires::class_of(&self.world, v);
        let gold = if voc.is_none() { 0.0 } else { livelihood::seed_gold(class, false, (rng.next() % 1000) as f32 / 1000.0) };
        self.world.add_component(v, Purse { gold });
        let wants = if voc.is_some() {
            desires::sample_wants(class, voc, false, &mut || rng.next())
        } else { vec![] };
        self.world.add_component(v, InnerLife { wants, grievances: vec![], satisfied: vec![] });
        self.villagers.push(v);
        self.was_sick.push(None);
        self.died_on.push(None);
        v
    }

    /// Which site `v` lives at, if they live out in the country
    pub fn site_of(&self, v: Entity) -> Option<usize> {
        self.world.get_component::<Outlander>(v).map(|o| o.site).filter(|&s| s < self.sites.len())
    }

    /// The granary `v` eats from: the village's for its own folk; for the
    /// country's, the nearest with food in it - their own at home, the
    /// market for a carter there, the town's for a steward come for stores
    pub fn store_of(&self, v: Entity) -> Entity {
        let Some(own) = self.site_of(v).map(|s| self.sites[s].store) else { return self.market_food };
        let Some(p) = self.world.get_component::<Pos>(v).copied() else { return own };
        let has = |f: Entity| self.world.get_component::<villager::FoodSource>(f).map(|x| x.servings > 0).unwrap_or(false);
        let d = |f: Entity| self.world.get_component::<Pos>(f).map(|q| (q.x - p.x).abs() + (q.y - p.y).abs()).unwrap_or(f32::MAX);
        self.sites.iter().map(|s| s.store).chain(std::iter::once(self.market_food))
            .filter(|&f| has(f))
            .min_by(|&a, &b| d(a).partial_cmp(&d(b)).unwrap())
            .unwrap_or(own)
    }

    /// Where `v`'s home ground is centred: their site's granary on the
    /// green, or the village market
    pub fn home_ground(&self, v: Entity) -> Option<(f32, f32)> {
        self.world.get_component::<Pos>(self.store_of(v)).map(|p| (p.x, p.y))
    }

    /// Out on the country's business: a cart at the market, a steward on
    /// the road (the day's routine leaves them be)
    pub fn is_away(&self, v: Entity) -> bool {
        self.site_of(v).map(|s| self.sites[s].carrier == Some(v) && self.sites[s].away_until.is_some()).unwrap_or(false)
    }

    /// A long walk with no errand cap: the roads between the towns
    pub(super) fn send_far(&mut self, who: Entity, to: (i32, i32), goal: Goal) -> bool {
        if crate::game::dungeon::unavailable(&self.world, who) { return false; }
        let Some(p) = self.world.get_component::<Pos>(who).copied() else { return false };
        let from = (p.x.round() as i32, p.y.round() as i32);
        match self.walk_memo(from, to) {
            Some(mut path) => {
                path.push(to);
                self.travels.retain(|t| t.who != who);
                self.travels.push(Travel { who, path, goal });
                true
            }
            None => false,
        }
    }

    /// Where a carter stands to sell: a free tile on the market's ring
    fn market_pitch(&self, v: Entity) -> Option<(i32, i32)> {
        let mp = self.world.get_component::<Pos>(self.market_food).copied()?;
        let a = (v as u64 % 16) as f32 / 16.0 * std::f32::consts::TAU;
        let want = ((mp.x + 4.0 * a.cos()).round() as i32, (mp.y + 4.0 * a.sin()).round() as i32);
        (0..=3).flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (want.0 + dx, want.1 + dy))))
            .find(|&(x, y)| self.map.passable(x, y))
    }

    /// The country's day: the granaries fill with the towns' work, the
    /// carts set out for market (timed to arrive for market day) and come
    /// home after, the stewards ride for provisions, and strays walk home
    pub(super) fn country_tick(&mut self, tick: u64) {
        for si in 0..self.sites.len() {
            let site = self.sites[si].clone();
            let folk: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&v| self.site_of(v) == Some(si) && !villager::is_dead(&self.world, v)).collect();
            // a farm's or an abbey's fields feed its granary too
            if matches!(site.kind, SiteKind::Town | SiteKind::Farm | SiteKind::Monastery) {
                let hands = folk.iter().filter(|&&v| self.world.get_component::<Kid>(v).is_none()
                    && !self.is_away(v)).count() as u32;
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(site.store) {
                    f.servings = (f.servings + hands * 2).min(TOWN_STORE_CAP);
                }
            }
            // strays (an errand cancelled on the road, a feast's pull)
            // walk home rather than standing lost in the country
            for &v in &folk {
                if self.is_away(v) || self.travels.iter().any(|t| t.who == v)
                    || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                let Some(p) = self.world.get_component::<Pos>(v).copied() else { continue };
                let (cx, cy) = (site.centre.0 as f32, site.centre.1 as f32);
                if ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt() > STRAY_FROM_SITE + site.radius as f32 {
                    if let Some(home) = household::home_of(&self.world, v) {
                        let hp = self.world.get_component::<Pos>(home).map(|p| (p.x.round() as i32, p.y.round() as i32));
                        if let Some(door) = hp.and_then(|(x, y)| self.at_the_door(x, y)) {
                            self.send_far(v, door, Goal::HeadHome(home));
                        }
                    }
                }
            }
            let Some(c) = site.carrier.filter(|&c| !villager::is_dead(&self.world, c)
                && !crate::game::dungeon::unavailable(&self.world, c)) else { continue };
            match (site.kind, site.away_until) {
                // home time: the cart turns for home
                (_, Some(until)) if tick >= until => {
                    let home = household::home_of(&self.world, c);
                    let door = home.and_then(|h| self.world.get_component::<Pos>(h).copied())
                        .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32));
                    self.sites[si].away_until = None;
                    if let (Some(h), Some(d)) = (home, door) {
                        if !self.travels.iter().any(|t| t.who == c) { self.send_far(c, d, Goal::HeadHome(h)); }
                    }
                }
                (kind, Some(until)) => {
                    if self.travels.iter().any(|t| t.who == c) { continue; }
                    if until == u64::MAX {
                        // the road was broken off (hunger, a meeting):
                        // take it up again from wherever they stand
                        let resumed = match kind {
                            SiteKind::Town => self.market_pitch(c).map(|p| self.send_far(c, p, Goal::Cart(si))).unwrap_or(false),
                            // farms and abbeys never run an errand to the
                            // village: their carrier stays home
                            SiteKind::Farm | SiteKind::Monastery => false,
                            SiteKind::Castle => {
                                // loaded at the granary already? carry the
                                // provisions on home. otherwise the steward
                                // never reached the town: head there still
                                let has_provisions = self.world.get_component::<Satchel>(c)
                                    .map(|s| s.goods.get("provisions").copied().unwrap_or(0) > 0).unwrap_or(false);
                                if has_provisions {
                                    let keep = site.homes.first().and_then(|&k| self.world.get_component::<Pos>(k).copied())
                                        .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32));
                                    keep.map(|d| self.send_far(c, d, Goal::Deliver(si))).unwrap_or(false)
                                } else {
                                    (0..self.sites.len()).filter(|&t| self.sites[t].kind == SiteKind::Town)
                                        .min_by_key(|&t| (self.sites[t].centre.0 - site.centre.0).abs() + (self.sites[t].centre.1 - site.centre.1).abs())
                                        .and_then(|ti| self.world.get_component::<Pos>(self.sites[ti].store).copied().map(|p| (ti, p)))
                                        .and_then(|(ti, p)| self.at_the_door(p.x.round() as i32, p.y.round() as i32).map(|d| (ti, d)))
                                        .map(|(ti, d)| self.send_far(c, d, Goal::Levy(si, ti))).unwrap_or(false)
                                }
                            }
                        };
                        if !resumed { self.sites[si].away_until = None; }
                    } else if let Some(r) = self.world.get_component_mut::<routine::Routine>(c) {
                        // at the market: keep the pitch for the day
                        r.day = tick;
                    }
                }
                // farms and abbeys send no cart and ride no levy
                (SiteKind::Farm | SiteKind::Monastery, None) => {}
                (SiteKind::Town, None) => {
                    // set out so as to come in for market day
                    let fed = self.world.get_component::<Hunger>(c).map(|h| h.value > 55.0).unwrap_or(false);
                    if !fed || self.travels.iter().any(|t| t.who == c) { continue; }
                    let Some(pitch) = self.market_pitch(c) else { continue };
                    let Some(p) = self.world.get_component::<Pos>(c).copied() else { continue };
                    let Some(path) = self.walk_memo((p.x.round() as i32, p.y.round() as i32), pitch) else { continue };
                    let days = (path.len() as f32 / TILES_PER_DAY).ceil() as u64;
                    let set_out = (routine::MARKET_DAY + 30 - days.min(29)) % 30;
                    if tick % 30 != set_out { continue; }
                    // the cart's load: what the town's hands have grown,
                    // their satchels' grain, and the granary's spare
                    let mut load = 0u32;
                    for &v in &folk {
                        if v == c { continue; }
                        if let Some(s) = self.world.get_component_mut::<Satchel>(v) {
                            let n = s.goods.remove("wheat").unwrap_or(0);
                            load += n;
                        }
                    }
                    if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(site.store) {
                        let spare = f.servings.saturating_sub(120).min(8);
                        f.servings -= spare;
                        load += spare;
                    }
                    if load > 0 {
                        if let Some(s) = self.world.get_component_mut::<Satchel>(c) {
                            *s.goods.entry("wheat".into()).or_insert(0) += load;
                        }
                    }
                    if self.send_far(c, pitch, Goal::Cart(si)) {
                        self.sites[si].away_until = Some(u64::MAX);
                        let name = self.villagers.iter().position(|&x| x == c).map(vill_name).unwrap_or("a carter");
                        self.log.push(format!("t{}: {}'s cart sets out from {} for market day.", tick, name, site.name));
                    }
                }
                (SiteKind::Castle, None) => {
                    if tick % LEVY_EVERY != (si as u64 * 7) % LEVY_EVERY || self.travels.iter().any(|t| t.who == c) { continue; }
                    // the nearest town's granary keeps the castle
                    let Some(ti) = (0..self.sites.len()).filter(|&t| self.sites[t].kind == SiteKind::Town)
                        .min_by_key(|&t| (self.sites[t].centre.0 - site.centre.0).abs() + (self.sites[t].centre.1 - site.centre.1).abs()) else { continue };
                    let Some(sp) = self.world.get_component::<Pos>(self.sites[ti].store).copied() else { continue };
                    let Some(to) = self.at_the_door(sp.x.round() as i32, sp.y.round() as i32) else { continue };
                    if self.send_far(c, to, Goal::Levy(si, ti)) {
                        self.sites[si].away_until = Some(u64::MAX);
                        let name = self.villagers.iter().position(|&x| x == c).map(vill_name).unwrap_or("the steward");
                        self.log.push(format!("t{}: {} rides out of {} to fetch provisions from {}.", tick, name, site.name, self.sites[ti].name));
                    }
                }
            }
        }
    }

    /// A cart comes in to the village market: the carter takes a pitch on
    /// the square, sells the town's grain from the satchel (the bakers
    /// buy there like from any seller) and stays a couple of days
    pub(super) fn arrive_cart(&mut self, who: Entity, si: usize) {
        let tick = self.tick;
        let Some(site) = self.sites.get(si).cloned() else { return };
        let load = self.world.get_component::<Satchel>(who).map(|s| s.goods.get("wheat").copied().unwrap_or(0)).unwrap_or(0);
        // a meal in town after the road
        if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
            if f.servings > 0 {
                f.servings -= 1;
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) { h.value = 100.0; }
            }
        }
        let Some(p) = self.world.get_component::<Pos>(who).copied() else { return };
        let face = self.world.get_component::<Pos>(self.market_food).map(|m| (m.x, m.y));
        self.set_routine(who, routine::Routine { mode: routine::Mode::Selling, spot: routine::Spot::Market,
            anchor: (p.x, p.y), face, drift: 0.0, leads: false, day: tick, reached: true });
        // stay through market day if they got there early, but a cart
        // delayed past market day just sells for its two days from
        // whenever it actually arrives, rather than waiting a month
        // for the next one
        let into = tick % 30;
        let stay_from = if into <= routine::MARKET_DAY { tick - into + routine::MARKET_DAY } else { tick };
        self.sites[si].away_until = Some(stay_from + MARKET_STAY);
        let name = self.villagers.iter().position(|&x| x == who).map(vill_name).unwrap_or("a carter");
        self.log.push(format!("t{}: {}'s cart comes in from {}{}.", tick, name, site.name,
            if load > 0 { format!(" with {} wheat for the market", load) } else { String::new() }));
    }

    /// The steward at the town granary: loads what the castle needs
    pub(super) fn arrive_levy(&mut self, who: Entity, castle: usize, town: usize) {
        let Some(store) = self.sites.get(town).map(|s| s.store) else { return };
        // a meal at the granary after the ride
        if self.world.get_component::<Hunger>(who).map(|h| h.value < 70.0).unwrap_or(false) {
            if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(store) {
                if f.servings > 0 {
                    f.servings -= 1;
                    if let Some(h) = self.world.get_component_mut::<Hunger>(who) { h.value = 100.0; }
                }
            }
        }
        let took = self.world.get_component_mut::<villager::FoodSource>(store)
            .map(|f| { let n = f.servings.saturating_sub(40).min(LEVY_TAKE); f.servings -= n; n }).unwrap_or(0);
        if took > 0 {
            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                *s.goods.entry("provisions".into()).or_insert(0) += took;
            }
        }
        // and back up to the castle with it
        let Some(&keep) = self.sites.get(castle).and_then(|s| s.homes.first()) else { return };
        let door = self.world.get_component::<Pos>(keep).copied()
            .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32));
        let sent = door.map(|d| self.send_far(who, d, Goal::Deliver(castle))).unwrap_or(false);
        if !sent { self.arrive_deliver(who, castle); }
    }

    /// Home at the castle with the provisions: into its stores
    pub(super) fn arrive_deliver(&mut self, who: Entity, castle: usize) {
        let n = self.world.get_component_mut::<Satchel>(who).and_then(|s| s.goods.remove("provisions")).unwrap_or(0);
        if let Some(store) = self.sites.get(castle).map(|s| s.store) {
            if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(store) { f.servings += n; }
        }
        if let Some(s) = self.sites.get_mut(castle) { s.away_until = None; }
    }
}

/// The village's homes inside the wall and out: which a villager of this
/// standing would rather take. Farmers and herders live out among the
/// fields; the gentry, the lord's men and the masters inside the wall
pub fn prefers_outside(world: &World, v: Entity) -> Option<bool> {
    if let Some(c) = world.get_component::<Craft>(v) {
        if matches!(c.vocation, Vocation::Farmer | Vocation::Shepherd | Vocation::Beekeeper
            | Vocation::Fisher) { return Some(true); }
        if vocation::is_pmc(c.vocation) { return Some(false); }
    }
    if matches!(world.get_component::<TownRole>(v),
        Some(TownRole::Overseer | TownRole::TaxCollector | TownRole::Trader)) { return Some(false); }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_country_moves_with_the_seed() {
        let a = Game::new_seeded(1);
        let b = Game::new_seeded(2);
        assert_eq!(a.sites.iter().filter(|s| s.kind == SiteKind::Town).count(), 2, "two towns");
        assert_eq!(a.sites.iter().filter(|s| s.kind == SiteKind::Castle).count(), 2, "two castles");
        assert_eq!(a.sites.iter().filter(|s| s.kind == SiteKind::Farm).count(), 3, "three farms");
        assert!(a.sites.iter().any(|s| s.kind == SiteKind::Monastery), "a monastery or two");
        assert_ne!(a.map.towns, b.map.towns, "towns stand somewhere new each map");
        assert_ne!(a.map.castles, b.map.castles, "and so do the castles");
        let again = Game::new_seeded(1);
        assert_eq!(a.map.towns, again.map.towns, "but a seed's country is its own");
        assert_eq!(a.sites.iter().map(|s| (s.kind, s.centre)).collect::<Vec<_>>(),
            again.sites.iter().map(|s| (s.kind, s.centre)).collect::<Vec<_>>(),
            "and its steadings stand fast");
    }

    #[test]
    fn test_the_far_farms_and_the_abbey_keep_their_own() {
        for seed in [0u64, 1, 2, 3, 7, 42, 99] {
            let g = Game::new_seeded(seed);
            for (si, s) in g.sites.iter().enumerate() {
                if !matches!(s.kind, SiteKind::Farm | SiteKind::Monastery) { continue }
                let folk = g.villagers.iter().filter(|&&v| g.site_of(v) == Some(si)).count();
                assert!(folk >= 2, "seed {} {:?} has folk", seed, s.kind);
                assert!(!s.homes.is_empty(), "seed {} {:?} has a roof", seed, s.kind);
            }
        }
        // the abbey draws as a church among the buildings, not a cottage
        let g = Game::new_seeded(1);
        let abbey = g.sites.iter().find(|s| s.kind == SiteKind::Monastery).unwrap();
        let p = g.world.get_component::<Pos>(abbey.homes[0]).unwrap();
        let (bx, by) = (p.x, p.y);
        assert!(g.buildings.iter().any(|&(x, y, s)|
            (x - bx).abs() < 0.5 && (y - by).abs() < 0.5 && s == BSprite::Church),
            "the abbey wears a church");
        // and a farm's strips are sown
        let farm = g.sites.iter().find(|s| s.kind == SiteKind::Farm).unwrap();
        let (cx, cy) = farm.centre;
        let sown = (cy + 2..=cy + 5).flat_map(|y| (cx - 4..=cx + 4).map(move |x| (x, y)))
            .filter(|&(x, y)| matches!(g.map.get(x, y), Some(Tile::Wheat) | Some(Tile::Corn))).count();
        assert!(sown >= 10, "the farm's fields are sown: {} tiles", sown);
    }

    #[test]
    fn test_country_sites_are_dry_apart_and_reachable() {
        for seed in [0u64, 1, 2, 3, 7, 42, 99, 231, 777, 9999] {
            let g = Game::new_seeded(seed);
            assert!(g.sites.len() >= 3, "seed {} drew only {} sites", seed, g.sites.len());
            for (i, s) in g.sites.iter().enumerate() {
                let sp = g.world.get_component::<Pos>(s.store).copied().unwrap();
                let st = (sp.x.round() as i32, sp.y.round() as i32);
                assert!(g.map.passable(st.0, st.1), "seed {} {}'s granary stands on open ground", seed, s.name);
                assert!(g.map.walk_path((g.village.0, g.village.1 + 1), st).is_some(), "seed {} {} walks to the village", seed, s.name);
                for t in &g.sites[i + 1..] {
                    let d = (((s.centre.0 - t.centre.0).pow(2) + (s.centre.1 - t.centre.1).pow(2)) as f32).sqrt();
                    assert!(d > 30.0, "seed {} {} and {} crowd each other", seed, s.name, t.name);
                }
            }
        }
    }

    #[test]
    fn test_castles_hold_the_gentry_and_towns_the_folk() {
        let g = Game::new_seeded(5);
        for v in g.villagers.iter().copied() {
            let Some(si) = g.site_of(v) else { continue };
            let class = desires::class_of(&g.world, v);
            match g.sites[si].kind {
                SiteKind::Castle => assert_eq!(class, SocialClass::ProfessionalMiddle, "castle folk are gentry"),
                // town trades and children are all working folk - never
                // the gentry's trades
                SiteKind::Town | SiteKind::Farm => {
                    assert_eq!(class, SocialClass::Working, "farm and town folk work for a living");
                    if let Some(c) = g.world.get_component::<Craft>(v) {
                        assert!(!vocation::is_pmc(c.vocation), "no knights or priests in the fields");
                    }
                }
                // the abbey's brothers: the ordination is gentry, the
                // lay brothers work the demesne
                SiteKind::Monastery => {
                    if let Some(c) = g.world.get_component::<Craft>(v) {
                        assert!(matches!(c.vocation, Vocation::Priest | Vocation::Farmer),
                            "the abbey keeps only its own");
                    }
                }
            }
            let home = household::home_of(&g.world, v).expect("country folk have a roof");
            let hp = g.world.get_component::<Pos>(home).unwrap();
            let (cx, cy) = g.sites[si].centre;
            assert!((hp.x - cx as f32).abs() <= 12.0 && (hp.y - cy as f32).abs() <= 12.0, "homed at their own site");
        }
        let out = g.villagers.iter().filter(|&&v| g.site_of(v).is_some()).count();
        let want: usize = g.sites.iter().map(|s| site_folk(s.kind, 0).len()).sum();
        assert_eq!(out, want, "every site settled its household");
        assert!(out > 8, "the country is peopled, not a pair to a site");
    }

    #[test]
    fn test_village_farmers_live_outside_the_wall() {
        let mut outside_farmers = 0;
        let mut farmers = 0;
        for seed in [1u64, 2, 3, 42] {
            let g = Game::new_seeded(seed);
            let (x0, y0, x1, y1) = g.town_wall;
            for &v in &g.villagers {
                if g.site_of(v).is_some() || prefers_outside(&g.world, v) != Some(true) { continue; }
                let Some(h) = household::home_of(&g.world, v) else { continue };
                let hp = g.world.get_component::<Pos>(h).unwrap();
                farmers += 1;
                let inside = (x0 as f32..=x1 as f32).contains(&hp.x) && (y0 as f32..=y1 as f32).contains(&hp.y);
                if !inside { outside_farmers += 1; }
            }
        }
        assert!(outside_farmers * 3 >= farmers * 2, "most of the land's hands live out by the fields ({}/{})", outside_farmers, farmers);
    }

    #[test]
    fn test_a_town_cart_comes_to_market_and_goes_home() {
        let mut g = Game::new_seeded(3);
        let n = g.ticks_per_frame();
        let town = g.sites.iter().position(|s| s.kind == SiteKind::Town).unwrap();
        let carter = g.sites[town].carrier.unwrap();
        let (mut came, mut home_again) = (false, false);
        for _ in 0..(70 * n) {
            g.update();
            let at = *g.world.get_component::<Pos>(carter).unwrap();
            let mp = *g.world.get_component::<Pos>(g.market_food).unwrap();
            if !came && (at.x - mp.x).abs() + (at.y - mp.y).abs() < 8.0 { came = true; }
            let (cx, cy) = g.sites[town].centre;
            if came && (at.x - cx as f32).abs() + (at.y - cy as f32).abs() < 14.0 { home_again = true; break; }
        }
        assert!(came, "the cart reached the market: {:?}", g.log.iter().filter(|l| l.contains("cart")).collect::<Vec<_>>());
        assert!(home_again, "and went home after");
    }

    #[test]
    fn test_a_steward_fetches_provisions_for_the_castle() {
        let mut g = Game::new_seeded(3);
        let n = g.ticks_per_frame();
        let castle = g.sites.iter().position(|s| s.kind == SiteKind::Castle).unwrap();
        let store = g.sites[castle].store;
        let before = g.world.get_component::<villager::FoodSource>(store).unwrap().servings;
        let mut rode = false;
        for _ in 0..(60 * n) {
            g.update();
            if g.log.iter().any(|l| l.contains("to fetch provisions")) { rode = true; }
            let now = g.world.get_component::<villager::FoodSource>(store).unwrap().servings;
            if rode && now > before { return; }
        }
        panic!("the castle's stores never filled (rode out: {})", rode);
    }

    #[test]
    fn test_country_folk_eat_at_their_own_granary() {
        let mut g = Game::new_seeded(4);
        let n = g.ticks_per_frame();
        let farmer = g.sites.iter().find(|s| s.kind == SiteKind::Town).and_then(|s| s.homes.first().copied())
            .and_then(|h| g.world.get_component::<Building>(h).and_then(|b| b.occupants.get(1).copied())).unwrap();
        g.world.get_component_mut::<Hunger>(farmer).unwrap().value = 20.0;
        let si = g.site_of(farmer).unwrap();
        let (cx, cy) = g.sites[si].centre;
        for _ in 0..(6 * n) { g.update(); }
        let h = g.world.get_component::<Hunger>(farmer).unwrap().value;
        let p = *g.world.get_component::<Pos>(farmer).unwrap();
        assert!(h > 50.0, "they ate (hunger {})", h);
        assert!((p.x - cx as f32).abs() + (p.y - cy as f32).abs() < 16.0, "at home, not off to the village market");
    }

    /// The country at a glance, a character a tile every few tiles:
    /// `cargo test -p game2-core --lib country_overview -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn country_overview() {
        for seed in [1u64, 2] {
            let g = Game::new_seeded(seed);
            println!("seed {}", seed);
            for y in (0..g.map.height as i32).step_by(5) {
                let row: String = (0..g.map.width as i32).step_by(3).map(|x| {
                    if g.sites.iter().any(|s| (s.centre.0 - x).abs() <= 2 && (s.centre.1 - y).abs() <= 3) {
                        let at = |k: SiteKind| g.sites.iter().any(|s| s.kind == k
                            && (s.centre.0 - x).abs() <= 2 && (s.centre.1 - y).abs() <= 3);
                        return if at(SiteKind::Castle) { 'C' }
                            else if at(SiteKind::Farm) { 'F' }
                            else if at(SiteKind::Monastery) { 'M' } else { 'T' };
                    }
                    match g.map.get(x, y) { Some(Tile::Water) => '~', Some(Tile::River) => '=', Some(Tile::Road) | Some(Tile::Bridge) => '+',
                        Some(Tile::Hill) => '^', Some(Tile::Rock) => 'A', Some(Tile::Wheat) | Some(Tile::Corn) => '#', _ => '.' }
                }).collect();
                println!("{}", row);
            }
        }
    }
}
