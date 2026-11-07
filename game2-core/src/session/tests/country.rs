use crate::session::*;
use crate::session::country::SiteKind;
use crate::session::routine::{Mode, Routine};
use crate::game::vocation::Kid;
use crate::game::villager::Pos;
use crate::game::tilemap::Tile;

#[test]
fn test_the_country_is_peopled_like_villages() {
    let g = Game::new_seeded(5);
    for (si, s) in g.sites.iter().enumerate() {
        let folk: Vec<Entity> = g.villagers.iter().copied()
            .filter(|&v| g.site_of(v) == Some(si)).collect();
        assert!(folk.len() >= 6, "{} holds only {} folk", s.name, folk.len());
        match s.kind {
            SiteKind::Town | SiteKind::Farm | SiteKind::Monastery => assert!(folk.iter().any(|&v|
                g.world.get_component::<Kid>(v).is_some()), "{} has children", s.name),
            SiteKind::Castle => {
                assert!(s.homes.len() >= 2, "{} has a hall beside the keep", s.name);
                assert!(folk.iter().all(|&v| g.world.get_component::<Kid>(v).is_none()),
                    "{} is a garrison, not a nursery", s.name);
            }
        }
        for &v in &folk {
            let i = g.villagers.iter().position(|x| *x == v).unwrap();
            assert_ne!(vill_name(i), "a newcomer", "{}'s folk all have names", s.name);
        }
    }
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
fn test_live_crop_fields_are_drawn_as_worked_ground() {
    for seed in [0u64, 1, 5, 42, 99] {
        let g = Game::new_seeded(seed);
        assert!(!g.crop_fields.is_empty(), "seed {seed} has live crop fields");
        for &field in &g.crop_fields {
            let p = g.world.get_component::<Pos>(field).expect("crop field has a position");
            let at = (p.x.round() as i32, p.y.round() as i32);
            let tile = g.map.get(at.0, at.1);
            assert!(matches!(tile, Some(Tile::Wheat | Tile::Corn)),
                "seed {seed} crop field at ({:.0},{:.0}) has worked ground: {:?}", p.x, p.y, tile);
            assert!(!g.map.trees.contains(&at), "seed {seed} crop field is not hidden by a tree");
        }
    }
}

#[test]
fn test_the_brothers_keep_the_office_at_their_abbey() {
    let mut g = Game::new_seeded(1);
    let Some(si) = g.sites.iter().position(|s| s.kind == SiteKind::Monastery) else { return };
    let brothers: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.site_of(v) == Some(si)).collect();
    assert!(brothers.len() >= 2, "the abbey has brothers");
    let abbey = *g.world.get_component::<Pos>(g.sites[si].homes[0]).unwrap();
    // run through the first holy day and give them the day to gather
    while g.tick < 7 { g.update(); }
    let n = g.ticks_per_frame() - 2;
    for _ in 0..n { g.update(); }
    let kept = brothers.iter().filter(|&&v| g.world.get_component::<Routine>(v)
        .map(|r| r.mode == Mode::Worshipping && r.day == 7 && r.reached)
        .unwrap_or(false)).count();
    assert!(kept >= 2, "the brothers reached their own abbey's office: {}", kept);
    // and the office is at the abbey's door, not the village church's
    let anchor = brothers.iter().filter_map(|&v| g.world.get_component::<Routine>(v)
        .filter(|r| r.mode == Mode::Worshipping && r.day == 7).map(|r| r.anchor)).next();
    if let Some((ax, ay)) = anchor {
        assert!((ax - abbey.x).abs() + (ay - abbey.y).abs() < 6.0,
            "the office gathers at the abbey, not off at ({:.1},{:.1})", ax, ay);
    }
}

#[test]
fn test_the_sites_reach_the_minimap() {
    let g = Game::new();
    let m = g.ui_model_at(0);
    assert!(!m.sites.is_empty(), "no sites on the minimap");
    // every site names itself
    assert!(m.sites.iter().all(|s| !s.name.is_empty()), "an unnamed site");
    // and a castle's keep names its lord
    let castle = m.sites.iter().find(|s| matches!(s.kind, country::SiteKind::Castle));
    if let Some(c) = castle {
        assert!(c.keeper.is_some(), "a keep with no keeper's name");
    }
}
