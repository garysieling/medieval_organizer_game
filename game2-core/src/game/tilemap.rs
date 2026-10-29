//! The map as tiles: what stands on each cell of the terrain - water,
//! grass, wooded hills, rock, fields, roads, a river - one to one with the
//! isometric pack's ground blocks, and laid out so the settlements make
//! sense: they sit on dry land, the river runs down to the sea, roads lead
//! from the market to every building and cross the river by a bridge.

use crate::game::terrain::Terrain;

/// What a map cell is. Each has its own ground block in the art pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Water,
    Grass,
    /// Higher ground, where the woods grow
    Hill,
    /// The highest, bare ground
    Rock,
    Wheat,
    Corn,
    Road,
    River,
    /// Where a road crosses the river
    Bridge,
}

/// Height bands of the terrain: below WATER_LEVEL is sea, then grass, hills
/// from HILL_LEVEL, bare rock from ROCK_LEVEL
pub const WATER_LEVEL: f32 = 0.15;
pub const HILL_LEVEL: f32 = 0.55;
pub const ROCK_LEVEL: f32 = 0.75;

/// The four neighbours of a tile, as (dx, dy), in the order of the
/// connection bits autotiling uses. On screen +x runs down-right and +y
/// down-left, so these are the tile's NW, NE, SE and SW edges.
pub const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];
const NW: u8 = 1;
const NE: u8 = 2;
const SE: u8 = 4;
const SW: u8 = 8;

/// Which road or river block joins up which edges: (connections, piece
/// number in the pack - road(n).png, and river(n).png but for its two
/// straights, which are the other way round: river_piece)
const PIECES: [(u8, u8); 15] = [
    (NW | NE | SE | SW, 1),
    (NW | SE, 2), (NE | SW, 3),
    (NW | SW | SE, 4), (NE | SE | SW, 5), (NW | NE | SE, 6), (NW | NE | SW, 7),
    (NW | SW, 8), (NE | SE, 9), (NW | NE, 10), (SW | SE, 11),
    (SE, 12), (SW, 13), (NE, 14), (NW, 15),
];
/// Bridge blocks: a road running NW-SE over a NE-SW river, and the reverse
pub const BRIDGE_ALONG_X: u8 = 16;
pub const BRIDGE_ALONG_Y: u8 = 17;

/// A piece of a town wall, by which way it runs (x: NW-SE on screen, y:
/// NE-SW). A gate is a gatehouse the road runs through, at right angles
/// to its wall; towers stand at the corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallPiece { AlongX, AlongY, GateAlongX, GateAlongY, Tower }

/// Where things go on the map. Coordinates are tiles.
#[derive(Debug, Clone, Default)]
pub struct Layout {
    /// (x, y, radius): ground kept dry and fairly level for a settlement
    pub settlements: Vec<(i32, i32, i32)>,
    /// The river's course from its spring, through each waypoint; past the
    /// last it carries on the same way until it reaches the sea
    pub river: Vec<(i32, i32)>,
    /// The bridge goes on the river tile nearest here where the river runs straight
    pub bridge_near: (i32, i32),
    pub fields: Vec<(i32, i32, Tile)>,
    /// Where each building wants to stand
    pub buildings: Vec<(i32, i32)>,
    /// Roads all start from here (the market)
    pub road_hub: (i32, i32),
    /// Places a road should reach that aren't buildings (another village's centre)
    pub road_ends: Vec<(i32, i32)>,
    /// (x0, y0, x1, y1): a wall round the town on this rectangle (corners
    /// included), with a tower at each corner and a gate midway along each side
    pub town_wall: Option<(i32, i32, i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    /// Where each layout building ended up (moved off water if it had to be)
    pub buildings: Vec<(i32, i32)>,
    /// Cells with a building on them
    occupied: Vec<bool>,
    /// Trees in the woods on the hills, and the odd one on open grass
    pub trees: Vec<(i32, i32)>,
    /// The town wall, piece by piece
    pub walls: Vec<((i32, i32), WallPiece)>,
    /// Where all roads start (Layout::road_hub)
    hub: (i32, i32),
}

/// A repeatable per-tile random number in 0..1
fn noise(x: i32, y: i32, salt: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ salt.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

/// How big a patch of woods runs, in tiles
const WOODS_SCALE: i32 = 6;

/// Noise that varies smoothly over `scale` tiles (noise() at a coarse grid,
/// blended between its corners), for things that come in patches
fn smooth_noise(x: i32, y: i32, scale: i32, salt: u32) -> f32 {
    let (gx, gy) = (x.div_euclid(scale), y.div_euclid(scale));
    let (fx, fy) = (x.rem_euclid(scale) as f32 / scale as f32, y.rem_euclid(scale) as f32 / scale as f32);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (ex, ey) = (ease(fx), ease(fy));
    let n = |dx: i32, dy: i32| noise(gx + dx, gy + dy, salt);
    let top = n(0, 0) + (n(1, 0) - n(0, 0)) * ex;
    let bottom = n(0, 1) + (n(1, 1) - n(0, 1)) * ex;
    top + (bottom - top) * ey
}

/// Sink every patch of land but the biggest: the height dome leaves odd
/// specks of land out in the corners, and a building nudged off the water
/// shouldn't end up stranded on one
fn sink_islets(terrain: &mut Terrain) {
    let (w, h) = (terrain.width as i32, terrain.height as i32);
    let land = |t: &Terrain, x: i32, y: i32| t.heights[y as usize][x as usize] >= WATER_LEVEL;
    let mut patch = vec![usize::MAX; (w * h) as usize];
    let mut sizes = vec![];
    for y in 0..h { for x in 0..w {
        if !land(terrain, x, y) || patch[(y * w + x) as usize] != usize::MAX { continue; }
        let id = sizes.len();
        let mut stack = vec![(x, y)];
        let mut n = 0;
        patch[(y * w + x) as usize] = id;
        while let Some((px, py)) = stack.pop() {
            n += 1;
            for (dx, dy) in NEIGHBOURS {
                let (qx, qy) = (px + dx, py + dy);
                if qx < 0 || qy < 0 || qx >= w || qy >= h || !land(terrain, qx, qy) || patch[(qy * w + qx) as usize] != usize::MAX { continue; }
                patch[(qy * w + qx) as usize] = id;
                stack.push((qx, qy));
            }
        }
        sizes.push(n);
    }}
    let Some(main) = (0..sizes.len()).max_by_key(|&i| sizes[i]) else { return };
    for y in 0..h { for x in 0..w {
        let p = patch[(y * w + x) as usize];
        if p != usize::MAX && p != main { terrain.heights[y as usize][x as usize] = WATER_LEVEL - 0.03; }
    }}
}

/// The river block numbered as road piece `p` joins up: the same, but for
/// the straights - river(2) runs NE-SW and river(3) NW-SE, unlike the roads
fn river_piece(p: u8) -> u8 {
    match p { 2 => 3, 3 => 2, p => p }
}

impl TileMap {
    /// Lay the map out on `terrain`: settlements get dry land (the terrain
    /// is raised there, so everything reading heights agrees), then the
    /// river, fields, buildings, roads and woods go down in that order
    pub fn generate(terrain: &mut Terrain, layout: &Layout) -> TileMap {
        let (w, h) = (terrain.width, terrain.height);
        for &(cx, cy, r) in &layout.settlements {
            for y in (cy - r).max(0)..=(cy + r).min(h as i32 - 1) {
                for x in (cx - r).max(0)..=(cx + r).min(w as i32 - 1) {
                    let d = (((x - cx).pow(2) + (y - cy).pow(2)) as f32).sqrt();
                    if d > r as f32 { continue; }
                    let v = &mut terrain.heights[y as usize][x as usize];
                    *v = v.clamp(0.25, HILL_LEVEL - 0.05);
                }
            }
        }
        if let Some((x0, y0, x1, y1)) = layout.town_wall {
            for y in y0.max(0)..=y1.min(h as i32 - 1) {
                for x in x0.max(0)..=x1.min(w as i32 - 1) {
                    let v = &mut terrain.heights[y as usize][x as usize];
                    *v = v.clamp(0.25, HILL_LEVEL - 0.05);
                }
            }
        }
        sink_islets(terrain);
        let tiles = (0..w * h).map(|i| {
            let v = terrain.heights[i / w][i % w];
            if v < WATER_LEVEL { Tile::Water } else if v < HILL_LEVEL { Tile::Grass } else if v < ROCK_LEVEL { Tile::Hill } else { Tile::Rock }
        }).collect();
        let mut map = TileMap { width: w, height: h, tiles, buildings: vec![], occupied: vec![false; w * h], trees: vec![], walls: vec![], hub: layout.road_hub };
        map.carve_river(&layout.river);
        map.place_bridge(layout.bridge_near);
        for &(x, y, crop) in &layout.fields {
            if map.get(x, y) == Some(Tile::Grass) { map.set(x, y, crop); }
        }
        if let Some(rect) = layout.town_wall { map.build_wall(rect); }
        for &(x, y) in &layout.buildings {
            let (bx, by) = map.nearest_buildable(x, y).unwrap_or((x, y));
            map.occupy(bx, by);
            map.buildings.push((bx, by));
        }
        let (hx, hy) = layout.road_hub;
        map.set(hx, hy, Tile::Road);
        let ends: Vec<(i32, i32)> = map.buildings.clone().into_iter().chain(layout.road_ends.iter().copied()).collect();
        for (x, y) in ends { map.lay_road(layout.road_hub, (x, y)); }
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                if map.is_occupied(x, y) { continue; }
                // woods grow in patches (smooth noise), thickest on the hills;
                // the odd lone tree stands out on open grass. Not in a settlement.
                let settled = layout.settlements.iter().any(|&(cx, cy, r)| (x - cx).pow(2) + (y - cy).pow(2) <= r * r)
                    || layout.town_wall.map(|(x0, y0, x1, y1)| (x0..=x1).contains(&x) && (y0..=y1).contains(&y)).unwrap_or(false);
                let woods = smooth_noise(x, y, WOODS_SCALE, 3);
                let wooded = !settled && match map.get(x, y) {
                    Some(Tile::Hill) => woods > 0.35 && noise(x, y, 1) < 0.85,
                    Some(Tile::Grass) => (woods > 0.68 && noise(x, y, 1) < 0.75) || noise(x, y, 2) < 0.02,
                    _ => false,
                };
                if wooded { map.trees.push((x, y)); }
            }
        }
        map
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height { return None; }
        Some(self.tiles[y as usize * self.width + x as usize])
    }

    fn set(&mut self, x: i32, y: i32, t: Tile) {
        if self.get(x, y).is_some() { self.tiles[y as usize * self.width + x as usize] = t; }
    }

    pub fn is_occupied(&self, x: i32, y: i32) -> bool {
        self.get(x, y).is_some() && self.occupied[y as usize * self.width + x as usize]
    }

    /// Mark a cell as built on
    pub fn occupy(&mut self, x: i32, y: i32) {
        if self.get(x, y).is_some() { self.occupied[y as usize * self.width + x as usize] = true; }
    }

    /// Wall in the rectangle (x0, y0)-(x1, y1): towers at the corners, a
    /// gate midway along each side (left open, so the roads go through them),
    /// wall everywhere else. Wall and towers can't be built on or walked through.
    fn build_wall(&mut self, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
        let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (on_x_side, on_y_side) = (y == y0 || y == y1, x == x0 || x == x1);
                let piece = match (on_x_side, on_y_side) {
                    (true, true) => WallPiece::Tower,
                    (true, false) if x == mx => WallPiece::GateAlongX,
                    (true, false) => WallPiece::AlongX,
                    (false, true) if y == my => WallPiece::GateAlongY,
                    (false, true) => WallPiece::AlongY,
                    _ => continue,
                };
                if self.get(x, y).is_none() { continue; }
                if matches!(self.get(x, y), Some(Tile::Water) | Some(Tile::River)) { self.set(x, y, Tile::Grass); }
                if !matches!(piece, WallPiece::GateAlongX | WallPiece::GateAlongY) { self.occupy(x, y); }
                self.walls.push(((x, y), piece));
            }
        }
    }

    /// Run a road from the hub to a new building at `to`
    pub fn connect(&mut self, to: (i32, i32)) {
        self.lay_road(self.hub, to);
    }

    /// The new road a building at `to` needs to join the hub's roads, in
    /// the order it would be laid - from where it leaves the existing road
    /// out to the building - without laying it (see pave)
    pub fn plan_road(&self, to: (i32, i32)) -> Vec<(i32, i32)> {
        let mut path = self.road_path(self.hub, to).unwrap_or_default();
        path.retain(|&(x, y)| matches!(self.get(x, y), Some(t) if t != Tile::Road && t != Tile::Bridge) && !self.is_occupied(x, y));
        path
    }

    /// Lay one tile of road
    pub fn pave(&mut self, x: i32, y: i32) {
        if !matches!(self.get(x, y), Some(Tile::Bridge) | Some(Tile::Water) | Some(Tile::River) | None) { self.set(x, y, Tile::Road); }
    }

    /// Whether a building could go here: open dry ground, nothing on it
    pub fn buildable(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), Some(Tile::Grass) | Some(Tile::Hill))
            && !self.is_occupied(x, y)
            && !self.trees.contains(&(x, y))
    }

    /// The buildable cell nearest (x, y), searching outward ring by ring
    pub fn nearest_buildable(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        (0..self.width.max(self.height) as i32).find_map(|r| {
            let ring = (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (dx, dy)))
                .filter(move |(dx, dy)| dx.abs().max(dy.abs()) == r);
            ring.map(|(dx, dy)| (x + dx, y + dy)).find(|&(cx, cy)| self.buildable(cx, cy))
        })
    }

    /// Run the river through its waypoints, mostly straight with the odd
    /// jog, then on to the sea. It springs where the course first reaches
    /// land (a course may start out at sea) and ends at the next water.
    fn carve_river(&mut self, course: &[(i32, i32)]) {
        let Some(&start) = course.first() else { return };
        let mut at = start;
        let mut last_step = (0, 1);
        let mut on_land = false;
        // false once the river has run into the sea
        let mut flow = |map: &mut TileMap, at: (i32, i32)| match map.get(at.0, at.1) {
            None => !on_land,
            Some(Tile::Water) => !on_land,
            Some(_) => { on_land = true; map.set(at.0, at.1, Tile::River); true }
        };
        if !flow(self, at) { return; }
        for &goal in &course[1..] {
            while at != goal {
                let (dx, dy) = (goal.0 - at.0, goal.1 - at.1);
                // step along whichever way is further to go
                last_step = if dx.abs() > dy.abs() { (dx.signum(), 0) } else { (0, dy.signum()) };
                at = (at.0 + last_step.0, at.1 + last_step.1);
                if !flow(self, at) { return; }
            }
        }
        while self.get(at.0 + last_step.0, at.1 + last_step.1).is_some() {
            at = (at.0 + last_step.0, at.1 + last_step.1);
            if !flow(self, at) { return; }
        }
    }

    /// Whether the river runs straight through (x, y) along x (true), along
    /// y (false), or isn't a straight run there (None)
    fn straight_river(&self, x: i32, y: i32) -> Option<bool> {
        let wet = |dx: i32, dy: i32| matches!(self.get(x + dx, y + dy), Some(Tile::River) | Some(Tile::Water) | Some(Tile::Bridge));
        let dry = |dx: i32, dy: i32| matches!(self.get(x + dx, y + dy), Some(t) if t != Tile::River && t != Tile::Water);
        if wet(-1, 0) && wet(1, 0) && dry(0, -1) && dry(0, 1) { Some(true) }
        else if wet(0, -1) && wet(0, 1) && dry(-1, 0) && dry(1, 0) { Some(false) }
        else { None }
    }

    fn place_bridge(&mut self, near: (i32, i32)) {
        let mut best: Option<((i32, i32), i32)> = None;
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if self.get(x, y) != Some(Tile::River) || self.straight_river(x, y).is_none() { continue; }
                let d = (x - near.0).abs() + (y - near.1).abs();
                if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x, y), d)); }
            }
        }
        if let Some(((x, y), _)) = best { self.set(x, y, Tile::Bridge); }
    }

    /// Whether a road may step from `a` onto `b`: over dry, unbuilt ground
    /// (not through fields), or across the bridge at right angles to the river
    fn road_step_ok(&self, a: (i32, i32), b: (i32, i32), goal: (i32, i32)) -> bool {
        let across = |at: (i32, i32)| match self.straight_river(at.0, at.1) {
            Some(river_along_x) => (a.0 == b.0) == river_along_x, // a step along y crosses an x-running river
            None => false,
        };
        let leaves_ok = self.get(a.0, a.1) != Some(Tile::Bridge) || across(a);
        let enters_ok = match self.get(b.0, b.1) {
            Some(Tile::Bridge) => across(b),
            Some(Tile::Grass) | Some(Tile::Hill) | Some(Tile::Rock) | Some(Tile::Road) => !self.is_occupied(b.0, b.1) || b == goal,
            _ => b == goal && !matches!(self.get(b.0, b.1), Some(Tile::Water) | Some(Tile::River) | None),
        };
        leaves_ok && enters_ok
    }

    /// Lay a road from `from` to the front of `to` (the building itself
    /// isn't paved)
    fn lay_road(&mut self, from: (i32, i32), to: (i32, i32)) {
        for (x, y) in self.road_path(from, to).unwrap_or_default() {
            if !self.is_occupied(x, y) && self.get(x, y) != Some(Tile::Bridge) { self.set(x, y, Tile::Road); }
        }
    }

    /// The way a road from `from` to `to` would go, `from` first: the
    /// cheapest way over the ground, where new road costs more than
    /// following road already laid, so roads share a trunk. None if `to`
    /// can't be reached.
    fn road_path(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let (w, h) = (self.width as i32, self.height as i32);
        let idx = |p: (i32, i32)| (p.1 * w + p.0) as usize;
        let mut cost = vec![u32::MAX; (w * h) as usize];
        let mut prev: Vec<Option<(i32, i32)>> = vec![None; (w * h) as usize];
        let mut open = std::collections::BinaryHeap::new();
        cost[idx(from)] = 0;
        open.push(std::cmp::Reverse((0u32, from)));
        while let Some(std::cmp::Reverse((c, p))) = open.pop() {
            if p == to { break; }
            if c > cost[idx(p)] { continue; }
            for (dx, dy) in NEIGHBOURS {
                let q = (p.0 + dx, p.1 + dy);
                if q.0 < 0 || q.1 < 0 || q.0 >= w || q.1 >= h || !self.road_step_ok(p, q, to) { continue; }
                let step = match self.get(q.0, q.1) { Some(Tile::Road) | Some(Tile::Bridge) => 1, Some(Tile::Hill) | Some(Tile::Rock) => 6, _ => 3 };
                if c + step < cost[idx(q)] {
                    cost[idx(q)] = c + step;
                    prev[idx(q)] = Some(p);
                    open.push(std::cmp::Reverse((c + step, q)));
                }
            }
        }
        if cost[idx(to)] == u32::MAX { return None; }
        let mut path = vec![to];
        let mut at = prev[idx(to)];
        while let Some(p) = at {
            path.push(p);
            at = prev[idx(p)];
        }
        path.reverse();
        Some(path)
    }

    /// Which of the pack's numbered blocks to draw for a road, river or
    /// bridge tile, joined up with its neighbours (None for other tiles)
    pub fn piece(&self, x: i32, y: i32) -> Option<u8> {
        let tile = self.get(x, y)?;
        let joins = |t: Option<Tile>| match tile {
            Tile::Road => matches!(t, Some(Tile::Road) | Some(Tile::Bridge)),
            Tile::River => matches!(t, Some(Tile::River) | Some(Tile::Bridge) | Some(Tile::Water)),
            _ => false,
        };
        match tile {
            Tile::Bridge => Some(if self.straight_river(x, y) == Some(false) { BRIDGE_ALONG_X } else { BRIDGE_ALONG_Y }),
            Tile::Road | Tile::River => {
                let mask = NEIGHBOURS.iter().enumerate()
                    .filter(|(_, (dx, dy))| joins(self.get(x + dx, y + dy)))
                    .fold(0u8, |m, (i, _)| m | 1 << i);
                let p = PIECES.iter().find(|(m, _)| *m == mask).map(|(_, p)| *p).unwrap_or(if tile == Tile::Road { 15 } else { 2 });
                Some(if tile == Tile::River { river_piece(p) } else { p })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout {
            town_wall: Some((8, 8, 21, 23)),
            settlements: vec![(16, 16, 8), (28, 14, 4)],
            river: vec![(23, 0), (24, 10), (24, 22)], // springs inland, past the northern shore
            bridge_near: (24, 15),
            fields: vec![(14, 14, Tile::Wheat), (14, 15, Tile::Corn)],
            buildings: vec![(13, 12), (19, 12), (28, 13), (31, 31)],
            road_hub: (16, 16),
            road_ends: vec![(28, 15)],
        }
    }

    fn map() -> (Terrain, TileMap) {
        let mut t = Terrain::generate_country(33, 0.7, 42);
        let m = TileMap::generate(&mut t, &layout());
        (t, m)
    }

    #[test]
    fn test_settlements_and_buildings_are_on_dry_land() {
        let (t, m) = map();
        assert_eq!(m.get(16, 16), Some(Tile::Road), "the market is the road hub");
        for &(x, y) in &m.buildings {
            assert!(matches!(m.get(x, y), Some(Tile::Grass) | Some(Tile::Hill)), "building at ({},{}) on {:?}", x, y, m.get(x, y));
            assert!(t.get(x as usize, y as usize) >= WATER_LEVEL);
        }
        // (31, 31) is out at sea: that building was moved ashore
        assert_ne!(m.buildings[3], (31, 31));
    }

    #[test]
    fn test_the_river_runs_to_the_sea_and_a_road_bridges_it() {
        let (_, m) = map();
        let river: Vec<(i32, i32)> = (0..33).flat_map(|y| (0..33).map(move |x| (x, y)))
            .filter(|&(x, y)| matches!(m.get(x, y), Some(Tile::River) | Some(Tile::Bridge))).collect();
        assert!(river.len() > 10, "a river of {} tiles", river.len());
        assert!(river.iter().any(|&(x, y)| NEIGHBOURS.iter().any(|(dx, dy)| m.get(x + dx, y + dy) == Some(Tile::Water))),
            "the river never reaches the sea");
        let bridges: Vec<_> = river.iter().filter(|&&(x, y)| m.get(x, y) == Some(Tile::Bridge)).collect();
        assert_eq!(bridges.len(), 1);
        let &&(bx, by) = bridges.first().unwrap();
        let roads = NEIGHBOURS.iter().filter(|(dx, dy)| m.get(bx + dx, by + dy) == Some(Tile::Road)).count();
        assert_eq!(roads, 2, "a road should run onto the bridge from both banks");
    }

    #[test]
    fn test_roads_reach_every_building() {
        let (_, m) = map();
        for &(x, y) in &m.buildings {
            assert!(NEIGHBOURS.iter().any(|(dx, dy)| m.get(x + dx, y + dy) == Some(Tile::Road)), "no road to the building at ({},{})", x, y);
        }
    }

    #[test]
    fn test_pieces_join_their_neighbours() {
        let (_, m) = map();
        for y in 0..33 { for x in 0..33 {
            if m.get(x, y) != Some(Tile::Road) { continue; }
            let p = m.piece(x, y).unwrap();
            let (mask, _) = PIECES.iter().find(|(_, n)| *n == p).unwrap();
            for (i, (dx, dy)) in NEIGHBOURS.iter().enumerate() {
                let joined = matches!(m.get(x + dx, y + dy), Some(Tile::Road) | Some(Tile::Bridge));
                if mask & (1 << i) != 0 || p != 15 { assert_eq!(mask & (1 << i) != 0, joined, "road piece {} at ({},{})", p, x, y); }
            }
        }}
        assert_eq!(m.piece(0, 0), None, "open sea has no piece");
        // the river runs along y (NE-SW) through the middle: river(2)
        assert_eq!(m.get(24, 18), Some(Tile::River));
        assert_eq!(m.piece(24, 18), Some(2));
    }

    #[test]
    fn test_a_new_building_gets_a_road() {
        let (_, mut m) = map();
        let site = m.nearest_buildable(10, 20).unwrap();
        m.occupy(site.0, site.1);
        m.connect(site);
        assert!(NEIGHBOURS.iter().any(|(dx, dy)| m.get(site.0 + dx, site.1 + dy) == Some(Tile::Road)));
    }

    #[test]
    fn test_a_planned_road_runs_out_from_the_network_and_is_laid_by_paving() {
        let (_, mut m) = map();
        let site = m.nearest_buildable(10, 20).unwrap();
        m.occupy(site.0, site.1);
        let plan = m.plan_road(site);
        assert!(!plan.is_empty());
        assert!(plan.iter().all(|&(x, y)| m.get(x, y) != Some(Tile::Road)), "only new road is planned");
        let first = plan[0];
        assert!(NEIGHBOURS.iter().any(|(dx, dy)| matches!(m.get(first.0 + dx, first.1 + dy), Some(Tile::Road))), "it starts off the existing road");
        let last = *plan.last().unwrap();
        assert!(NEIGHBOURS.iter().any(|(dx, dy)| (last.0 + dx, last.1 + dy) == site), "it ends at the building");
        for &(x, y) in &plan { m.pave(x, y); }
        assert!(m.plan_road(site).is_empty(), "once laid there's nothing left to build");
    }

    #[test]
    fn test_the_town_is_walled_with_corner_towers_and_gates_the_roads_use() {
        let (_, m) = map();
        let piece = |x: i32, y: i32| m.walls.iter().find(|(p, _)| *p == (x, y)).map(|(_, w)| *w);
        for corner in [(8, 8), (21, 8), (8, 23), (21, 23)] { assert_eq!(piece(corner.0, corner.1), Some(WallPiece::Tower)); }
        assert_eq!(piece(12, 8), Some(WallPiece::AlongX));
        assert_eq!(piece(8, 12), Some(WallPiece::AlongY));
        let gates: Vec<(i32, i32)> = m.walls.iter().filter(|(_, w)| matches!(w, WallPiece::GateAlongX | WallPiece::GateAlongY)).map(|(p, _)| *p).collect();
        assert_eq!(gates.len(), 4);
        for &(x, y) in &m.walls.iter().map(|(p, _)| *p).collect::<Vec<_>>() {
            let gate = gates.contains(&(x, y));
            assert_eq!(m.is_occupied(x, y), !gate, "wall at ({},{}) should block, a gate shouldn't", x, y);
        }
        // the road to the rivals' village leaves through a gate
        assert!(gates.iter().any(|&(x, y)| m.get(x, y) == Some(Tile::Road)
            && NEIGHBOURS.iter().filter(|(dx, dy)| m.get(x + dx, y + dy) == Some(Tile::Road)).count() == 2));
    }

    #[test]
    fn test_woods_grow_in_patches() {
        let (_, m) = map();
        // most trees have another tree beside them: woods, not a scatter
        let in_woods = m.trees.iter().filter(|&&(x, y)| NEIGHBOURS.iter().any(|(dx, dy)| m.trees.contains(&(x + dx, y + dy)))).count();
        assert!(in_woods * 3 >= m.trees.len() * 2, "{} of {} trees stand in woods", in_woods, m.trees.len());
    }

    #[test]
    fn test_woods_grow_on_hills_not_on_roads_or_buildings() {
        let (_, m) = map();
        assert!(!m.trees.is_empty());
        for &(x, y) in &m.trees {
            assert!(matches!(m.get(x, y), Some(Tile::Hill) | Some(Tile::Grass)) && !m.is_occupied(x, y));
            assert!(!m.buildable(x, y), "a tree's tile can't take a building");
        }
    }
}
