//! Fire on the map: a small cellular spread over the real TileMap.
//! From the oil-boiling lit review - the ink-maker boils his oil outside
//! the walls because the flame "would be glad to get to it" (Cennini) and
//! towns banished flame-and-stench crafts beyond the gates. The spread
//! rules were proven in sim/emergence.rs; this binds them to real
//! terrain: grass, crops and wooded hills burn, water and roads are
//! firebreaks, and burnt ground takes a month to go green again.

use std::collections::HashMap;
use super::tilemap::{noise, Tile, TileMap, NEIGHBOURS};

/// Ticks a tile burns before it's scorched
pub const BURN_TICKS: u8 = 3;
/// Days a scorched tile takes to grow back
pub const RECOVER_DAYS: u64 = 30;
/// Per-tick chance a burning tile lights a flammable neighbour
pub const SPREAD_CHANCE: f32 = 0.3;

/// The fire: what's alight, and what's burnt out and healing. Burning
/// tiles hold the ticks they have left; scorched tiles hold the tick
/// they're green again.
#[derive(Debug, Default)]
pub struct Fire {
    pub burning: HashMap<(i32, i32), u8>,
    pub scorched: HashMap<(i32, i32), u64>,
}

impl Fire {
    pub fn new() -> Self { Self::default() }

    /// Anything alight right now
    pub fn active(&self) -> bool { !self.burning.is_empty() }

    /// Whether a tile is on fire
    pub fn on_fire(&self, x: i32, y: i32) -> bool { self.burning.contains_key(&(x, y)) }

    /// Whether a tile is burnt out and waiting to recover
    pub fn is_scorched(&self, x: i32, y: i32) -> bool { self.scorched.contains_key(&(x, y)) }

    /// Whether a tile can catch: grass and standing crops burn, a hill
    /// burns where it carries a tree, and a building catches whatever it
    /// sits on; water, rock, road, bridge - and the stone of the wall -
    /// are the firebreaks
    pub fn flammable(map: &TileMap, x: i32, y: i32) -> bool {
        // the wall's stone doesn't burn, whatever ground it stands on
        if map.walls.iter().any(|&(p, _)| p == (x, y)) { return false; }
        match map.get(x, y) {
            Some(Tile::Grass) | Some(Tile::Wheat) | Some(Tile::Corn) => true,
            Some(Tile::Hill) => map.trees.contains(&(x, y)) || map.buildings.contains(&(x, y)),
            Some(_) => map.buildings.contains(&(x, y)),
            None => false,
        }
    }

    /// Light a tile if it can burn. Returns whether it caught.
    pub fn ignite(&mut self, map: &TileMap, x: i32, y: i32) -> bool {
        if self.burning.contains_key(&(x, y)) { return false; }
        if self.scorched.contains_key(&(x, y)) { return false; }
        if !Self::flammable(map, x, y) { return false; }
        self.burning.insert((x, y), BURN_TICKS);
        true
    }

    /// One step of the fire: burning tiles roll on their flammable
    /// neighbours, burn down to scorched ground (taking their trees with
    /// them), and old scars go green again. Returns the newly lit and the
    /// newly burnt out, so callers can tell the story.
    pub fn tick(&mut self, map: &mut TileMap, tick: u64) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
        self.tick_rolled(map, tick, |x, y, t| noise(x, y, t as u32 ^ 0xF1E))
    }

    /// The step with the dice handed in - tests roll their own
    fn tick_rolled(
        &mut self,
        map: &mut TileMap,
        tick: u64,
        roll: impl Fn(i32, i32, u64) -> f32,
    ) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
        // ignition is decided against what was burning when the step
        // began, so fire can't cross the map in a single tick
        let cells: Vec<(i32, i32)> = self.burning.keys().copied().collect();
        let mut ignited = vec![];
        for &(x, y) in &cells {
            for (dx, dy) in NEIGHBOURS {
                let n = (x + dx, y + dy);
                if self.burning.contains_key(&n) || self.scorched.contains_key(&n) { continue; }
                if !Self::flammable(map, n.0, n.1) { continue; }
                if roll(n.0, n.1, tick) < SPREAD_CHANCE {
                    ignited.push(n);
                }
            }
        }
        ignited.sort_unstable();
        ignited.dedup();
        for &n in &ignited {
            self.burning.insert(n, BURN_TICKS);
        }

        // burn down what was already alight (new catches burn full term)
        let mut out = vec![];
        for c in cells {
            let Some(left) = self.burning.get_mut(&c) else { continue };
            *left = left.saturating_sub(1);
            if *left == 0 {
                self.burning.remove(&c);
                self.scorched.insert(c, tick + RECOVER_DAYS);
                map.trees.retain(|&t| t != c);
                out.push(c);
            }
        }

        // the ground forgets, slowly
        let healed: Vec<(i32, i32)> = self.scorched.iter()
            .filter(|(_, &until)| tick >= until)
            .map(|(&c, _)| c).collect();
        for c in healed { self.scorched.remove(&c); }

        (ignited, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::terrain::Terrain;

    fn layout() -> crate::game::tilemap::Layout {
        crate::game::tilemap::Layout {
            town_wall: Some((8, 8, 21, 23)),
            settlements: vec![(16, 16, 8), (28, 14, 4)],
            river: vec![(23, 0), (24, 10), (24, 22)],
            bridge_near: (24, 15),
            fields: vec![(14, 14, Tile::Wheat), (14, 15, Tile::Corn)],
            buildings: vec![(13, 12), (19, 12), (28, 13), (31, 31)],
            road_hub: (16, 16),
            road_ends: vec![(28, 15)],
        }
    }

    fn map() -> TileMap {
        let mut t = Terrain::generate_country(33, 0.7, 42);
        TileMap::generate(&mut t, &layout())
    }

    fn find(m: &TileMap, want: Tile) -> (i32, i32) {
        (0..m.height as i32).flat_map(|y| (0..m.width as i32).map(move |x| (x, y)))
            .find(|&(x, y)| m.get(x, y) == Some(want))
            .unwrap_or_else(|| panic!("no {:?} on the map", want))
    }

    #[test]
    fn test_what_catches_and_what_doesnt() {
        let m = map();
        let mut f = Fire::new();
        let (gx, gy) = find(&m, Tile::Grass);
        assert!(f.ignite(&m, gx, gy), "grass catches");
        assert!(!f.ignite(&m, gx, gy), "twice doesn't count");
        let (rx, ry) = find(&m, Tile::Road);
        assert!(!f.ignite(&m, rx, ry), "the road is a firebreak");
        let (wx, wy) = find(&m, Tile::Wheat);
        assert!(f.ignite(&m, wx, wy), "a standing crop burns - that's the tragedy");
    }

    #[test]
    fn test_fire_spreads_burns_out_and_heals() {
        let mut m = map();
        let mut f = Fire::new();
        let (gx, gy) = find(&m, Tile::Grass);
        f.ignite(&m, gx, gy);
        // every neighbour always catches: the fire runs to its firebreaks
        let (ignited, _) = f.tick_rolled(&mut m, 1, |_, _, _| 0.0);
        assert!(!ignited.is_empty(), "an eager fire spreads");
        // let it run to the edges of its fuel
        for t in 2..600 {
            f.tick_rolled(&mut m, t, |_, _, _| 0.0);
            if !f.active() { break; }
        }
        assert!(!f.active(), "the fire always burns itself out");
        assert!(!f.scorched.is_empty(), "it left scorched ground");
        for &(x, y) in f.scorched.keys() {
            assert!(matches!(m.get(x, y), Some(Tile::Grass) | Some(Tile::Wheat) | Some(Tile::Corn) | Some(Tile::Hill)),
                "fire broke a firebreak at ({},{})", x, y);
        }
        // a month on, the meadow greens again
        let until = f.scorched.values().copied().max().unwrap();
        f.tick_rolled(&mut m, until, |_, _, _| 1.0);
        assert!(f.scorched.is_empty(), "the ground forgot");
    }

    #[test]
    fn test_a_shy_fire_can_die_alone() {
        let mut m = map();
        let mut f = Fire::new();
        let (gx, gy) = find(&m, Tile::Grass);
        f.ignite(&m, gx, gy);
        // nothing ever catches: the spark becomes a scorched patch
        for t in 1..=BURN_TICKS as u64 { f.tick_rolled(&mut m, t, |_, _, _| 1.0); }
        assert!(!f.active());
        assert!(f.is_scorched(gx, gy));
    }

    #[test]
    fn test_the_wall_is_a_firebreak() {
        let mut m = map();
        let mut f = Fire::new();
        let wall_cells: Vec<(i32, i32)> = m.walls.iter().map(|&(p, _)| p).collect();
        assert!(!wall_cells.is_empty());
        for &(x, y) in &wall_cells {
            assert!(!f.ignite(&m, x, y), "stone doesn't catch at ({},{})", x, y);
        }
        // fire on the meadow laps at the wall but never comes through it
        let (x, y) = (8..=21).flat_map(|x| (24..=27).map(move |y| (x, y)))
            .find(|&(x, y)| Fire::flammable(&m, x, y))
            .expect("there's meadow below the south wall");
        f.ignite(&m, x, y);
        for t in 1..80 {
            f.tick_rolled(&mut m, t, |_, _, _| 0.0);
            if !f.active() { break; }
        }
        assert!(f.scorched.keys().all(|c| !wall_cells.contains(c)),
            "the fire never took the wall");
    }

    #[test]
    fn test_burning_takes_the_tree_with_it() {
        let mut m = map();
        let mut f = Fire::new();
        let Some(&(tx, ty)) = m.trees.iter().find(|&&(x, y)| m.get(x, y) == Some(Tile::Hill)) else {
            panic!("the test map should have a wooded hill");
        };
        assert!(Fire::flammable(&m, tx, ty), "a wooded hill burns");
        f.ignite(&m, tx, ty);
        for t in 1..=BURN_TICKS as u64 { f.tick_rolled(&mut m, t, |_, _, _| 1.0); }
        assert!(!m.trees.contains(&(tx, ty)), "the tree is gone");
        assert!(!Fire::flammable(&m, tx, ty), "a bare hill doesn't catch");
    }

    #[test]
    fn test_a_house_built_on_a_bare_hill_can_catch() {
        let mut m = map();
        let Some(&(hx, hy)) = (0..m.height as i32).flat_map(|y| (0..m.width as i32).map(move |x| (x, y)))
            .collect::<Vec<_>>().iter()
            .find(|&&(x, y)| m.get(x, y) == Some(Tile::Hill) && !m.trees.contains(&(x, y)) && !m.is_occupied(x, y))
        else {
            panic!("the test map should have a bare, buildable hill");
        };
        assert!(!Fire::flammable(&m, hx, hy), "a bare hill carries nothing to burn yet");
        m.build_at(hx, hy);
        assert!(Fire::flammable(&m, hx, hy), "a house on the hill is fuel, tree or no tree");
        let mut f = Fire::new();
        assert!(f.ignite(&m, hx, hy), "the new house catches");
    }
}
