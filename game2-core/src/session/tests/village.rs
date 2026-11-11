//! The village's spot on the map: the classic seed keeps its corner spot,
//! every other seed roams the middle country - and the same seed always
//! draws the same village.
use crate::session::*;

#[test]
fn the_classic_map_keeps_its_corner() {
    let g = Game::new();
    assert_eq!(g.village, (16, 16));
    assert_eq!(g.map.get(16, 16), Some(Tile::Road), "the market cross is the road hub");
}

#[test]
fn every_seed_draws_a_village_inside_the_country() {
    for seed in [0u64, 1, 2, 3, 7, 99, 231, 777, 1234, 9999, 31337] {
        let g = Game::new_seeded(seed);
        let (vx, vy) = g.village;
        let n = g.map.width as i32;
        // the cluster - wall, fields, the rivals' bank - fits round it
        assert!(vx - 12 >= 4 && vx + 20 < n - 4 && vy - 12 >= 4 && vy + 24 < n - 4,
            "seed {} village ({},{}) leaves the map no room", seed, vx, vy);
        // the market stands on the road hub, open to walk on
        assert_eq!(g.map.get(vx, vy), Some(Tile::Road), "seed {} market cross", seed);
        // the player spawns beside it, on open ground
        let (px, py) = g.player_pos();
        assert!(g.map.passable(px.round() as i32, py.round() as i32),
            "seed {} player spawn ({},{}) blocked", seed, px, py);
        // and the country walks home: every site's granary reaches the market
        for s in &g.sites {
            let sp = g.world.get_component::<Pos>(s.store).copied().unwrap();
            let st = (sp.x.round() as i32, sp.y.round() as i32);
            assert!(g.map.walk_path((vx, vy), st).is_some(),
                "seed {} {} can't walk to the village", seed, s.name);
        }
    }
}

#[test]
fn the_village_moves_with_the_seed() {
    let spots: Vec<(i32, i32)> = [0u64, 1, 2, 3, 7, 99, 231, 777, 9999]
        .iter().map(|&s| Game::new_seeded(s).village).collect();
    assert!(spots.iter().any(|&v| v != (16, 16)), "the village never left its corner");
    let mut distinct: crate::engine::det::HashSet<&(i32, i32)> = crate::engine::det::HashSet::default();
    distinct.extend(spots.iter());
    assert!(distinct.len() > 4, "the village should vary, got {:?}", spots);
}

#[test]
fn a_seed_always_draws_the_same_village() {
    for seed in [0u64, 7, 99, 9999] {
        assert_eq!(Game::new_seeded(seed).village, Game::new_seeded(seed).village,
            "seed {} drew two villages", seed);
    }
}

#[test]
fn some_seeds_zone_their_town_into_company_districts() {
    // the company-town coin is per seed: some villages keep the loose
    // scatter, some zone the manager's row, the twins, the terraces and
    // the barracks by the gate - and the weaver's bench moves to a
    // riverbank works when the seed's map has a bank for it
    let (mut zoned, mut plain) = (0, 0);
    let (mut mansions, mut terraces, mut barracks, mut factories) = (0, 0, 0, 0);
    let mut first_zoned = None;
    for seed in 0u64..16 {
        let g = Game::new_seeded(seed);
        // the company district is a village read - a farm's manor out in
        // the country isn't its own zoning
        let (vx, vy) = g.village;
        let sprites: Vec<BSprite> = g.buildings.iter()
            .filter(|b| (b.0 - vx as f32).abs() + (b.1 - vy as f32).abs() < 60.0)
            .map(|b| b.2).collect();
        let is_zone = |s: &BSprite| matches!(s, BSprite::Mansion | BSprite::Rowhouse(_) | BSprite::Barracks(_));
        if sprites.iter().any(is_zone) {
            zoned += 1;
            first_zoned.get_or_insert((seed, sprites.clone()));
            mansions += sprites.iter().filter(|s| matches!(s, BSprite::Mansion)).count();
            terraces += sprites.iter().filter(|s| matches!(s, BSprite::Rowhouse(_))).count();
            barracks += sprites.iter().filter(|s| matches!(s, BSprite::Barracks(_))).count();
        } else { plain += 1; }
        factories += sprites.iter().filter(|s| matches!(s, BSprite::Work(WorkLook::Factory, _))).count();
    }
    // the same seed always draws the same town
    let (seed, sprites) = first_zoned.expect("no seed zoned a company town");
    let g2 = Game::new_seeded(seed);
    let (vx2, vy2) = g2.village;
    let again: Vec<BSprite> = g2.buildings.iter()
        .filter(|b| (b.0 - vx2 as f32).abs() + (b.1 - vy2 as f32).abs() < 60.0)
        .map(|b| b.2).collect();
    assert_eq!(sprites, again, "seed {} drew two towns", seed);
    assert!(zoned > 0, "no seed zoned a company town");
    assert!(plain > 0, "every seed zoned - the coin never lands plain");
    assert!(mansions > 0 && terraces > 0 && barracks > 0,
        "districts missing: {} mansions, {} terraces, {} barracks", mansions, terraces, barracks);
    assert!(factories > 0, "no seed moved the weaver's bench to the river works");
}

#[test]
fn the_well_and_hall_stand_on_clear_grass_by_the_market() {
    let mut found = 0;
    for seed in [0u64, 1, 2, 3, 7, 99, 231, 777, 1234, 9999] {
        let g = Game::new_seeded(seed);
        let (vx, vy) = g.village;
        for t in [g.well_tile, g.hall_tile].into_iter().flatten() {
            found += 1;
            let (x, y) = t;
            assert!(g.map.passable(x, y) && !g.map.is_occupied(x, y),
                "seed {} prop on a blocked tile ({},{})", seed, x, y);
            assert!(matches!(g.map.get(x, y), Some(Tile::Grass | Tile::Hill)),
                "seed {} prop off open ground ({},{})", seed, x, y);
            assert!((x - vx).abs().max((y - vy).abs()) >= 3,
                "seed {} prop inside the market square ({},{})", seed, x, y);
        }
        // the same seed plants the same pair, and the model carries them
        let g2 = Game::new_seeded(seed);
        assert_eq!(g.well_tile, g2.well_tile);
        assert_eq!(g.hall_tile, g2.hall_tile);
        let m = g.ui_model();
        assert_eq!(m.well, g.well_tile);
        assert_eq!(m.hall, g.hall_tile);
    }
    assert!(found > 0, "no seed found room for the commons");
}

#[test]
fn the_shops_wear_their_own_looks() {
    use BuildingKind::*;
    for (v, look) in [(Vocation::Tailor, WorkLook::Tailor), (Vocation::Glazier, WorkLook::Glazier),
            (Vocation::Printmaker, WorkLook::Printshop), (Vocation::Mason, WorkLook::MasonsYard),
            (Vocation::Luthier, WorkLook::Luthier), (Vocation::Farmer, WorkLook::Farmhouse)] {
        assert_eq!(WorkLook::of(Shop(v)), Some(look), "{:?} shop", v);
    }
    // the smithy keeps its own sprite path, and the catch-all workshop
    // still stands in for the trades without art of their own
    assert_eq!(WorkLook::of(Shop(Vocation::Toolmaker)), None);
    assert_eq!(WorkLook::of(Shop(Vocation::Weaver)), Some(WorkLook::Workshop));
    assert_eq!(WorkLook::file(WorkLook::MasonsYard), "masons-yard");
}
