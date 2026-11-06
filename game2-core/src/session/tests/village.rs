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
        let sprites: Vec<BSprite> = g.buildings.iter().map(|b| b.2).collect();
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
    let again: Vec<BSprite> = Game::new_seeded(seed).buildings.iter().map(|b| b.2).collect();
    assert_eq!(sprites, again, "seed {} drew two towns", seed);
    assert!(zoned > 0, "no seed zoned a company town");
    assert!(plain > 0, "every seed zoned - the coin never lands plain");
    assert!(mansions > 0 && terraces > 0 && barracks > 0,
        "districts missing: {} mansions, {} terraces, {} barracks", mansions, terraces, barracks);
    assert!(factories > 0, "no seed moved the weaver's bench to the river works");
}
