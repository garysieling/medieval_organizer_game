use crate::session::*;
use crate::session::country::SiteKind;
use crate::session::routine::{Mode, Routine};
use crate::game::livelihood::Purse;
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
            SiteKind::Town | SiteKind::Farm | SiteKind::Monastery | SiteKind::Mine => assert!(folk.iter().any(|&v|
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

#[test]
fn test_every_site_has_a_head_and_a_liege() {
    for seed in [0u64, 1, 2, 3, 7, 42, 99] {
        let g = Game::new_seeded(seed);
        let castles: Vec<usize> = g.sites.iter().enumerate()
            .filter(|(_, s)| s.kind == SiteKind::Castle).map(|(i, _)| i).collect();
        assert_eq!(castles.len(), 2, "seed {} drew its castles", seed);
        for s in g.sites.iter() {
            assert!(s.lord.is_some(), "seed {} {} has no head", seed, s.name);
            assert!(s.title.is_some(), "seed {} {} has no style", seed, s.name);
            match s.kind {
                // castles answer to the crown, which is off the map
                SiteKind::Castle => assert!(s.liege.is_none(), "{} claims a liege", s.name),
                _ => {
                    let liege = s.liege.expect("a holding with no liege");
                    assert!(matches!(g.sites[liege].kind, SiteKind::Castle),
                        "{} answers to a {:?}, not a castle", s.name, g.sites[liege].kind);
                }
            }
        }
        // and the same seed draws the same peerage
        let again = Game::new_seeded(seed);
        assert_eq!(g.sites.iter().map(|s| (s.title, s.liege)).collect::<Vec<_>>(),
            again.sites.iter().map(|s| (s.title, s.liege)).collect::<Vec<_>>(),
            "seed {}'s peerage moved", seed);
    }
}

#[test]
fn test_the_minimap_names_the_heads_and_the_treasuries() {
    let mut g = Game::new_seeded(3);
    // fill one treasury so the gold shows
    let ci = g.sites.iter().position(|s| s.kind == SiteKind::Castle).unwrap();
    g.sites[ci].treasury = 87.0;
    let m = g.ui_model_at(0);
    let castle = m.sites.iter().find(|s| s.kind == country::SiteKind::Castle).unwrap();
    let head = castle.head.as_ref().expect("the keep's head unnamed");
    assert!(head.contains(' '), "a style and a name: {}", head);
    assert!((castle.treasury - 87.0).abs() < 0.01, "the treasury didn't reach the wire");
}

#[test]
fn test_a_sites_tribute_reaches_the_lieges_treasury() {
    let mut g = Game::new_seeded(5);
    let si = g.sites.iter().position(|s| s.liege.is_some()
        && matches!(s.kind, SiteKind::Town)).expect("no tributary town");
    let liege = g.sites[si].liege.unwrap();
    let runner = g.sites[si].lord.expect("the town has no head");
    if let Some(p) = g.world.get_component_mut::<Purse>(runner) { p.gold = 200.0; }
    // the tribute's own day for this site (a zero residue means the
    // period's end, not its start)
    let r = (si as u64 * 7) % 60;
    g.country_tick(if r == 0 { 60 } else { r });
    let t = g.travels.iter().find(|t| t.who == runner && matches!(t.goal, Goal::Owe(..)))
        .expect("no tribute rode out");
    let Goal::Owe(_, to, gold) = t.goal.clone() else { unreachable!() };
    assert_eq!(to, liege);
    let purse = g.world.get_component::<Purse>(runner).map(|p| p.gold).unwrap();
    assert!(purse < 200.0, "the purse never paid: {:.0}", purse);
    // and on arrival the keep's books take it
    let before = g.sites[liege].treasury;
    g.arrive(runner, Goal::Owe(si, liege, gold));
    assert!((g.sites[liege].treasury - before - gold).abs() < 0.01,
        "the treasury took {:.0}, not {:.0}", g.sites[liege].treasury - before, gold);
}

#[test]
fn test_the_village_remit_fills_the_keeps_books() {
    let mut g = Game::new_seeded(2);
    let c = g.collector.expect("no collector");
    g.takings = 30.0;
    g.livelihood_tick(45);
    // the remit rides to the NEAREST keep - the same pick the day's
    // bookkeeping makes
    let cp = g.world.get_component::<Pos>(c).map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
    let castle = g.sites.iter().enumerate()
        .filter(|(_, s)| s.kind == SiteKind::Castle)
        .min_by_key(|(_, s)| ((s.centre.0 as f32 - cp.0).powi(2)
            + (s.centre.1 as f32 - cp.1).powi(2)) as i64)
        .map(|(i, _)| i).expect("no castle");
    // the ride's a long one: walk it the way the takings test does
    for _ in 0..6000 {
        g.advance_travels();
        if !g.travels.iter().any(|t| t.who == c) { break; }
    }
    assert!(g.sites[castle].treasury >= 30.0,
        "the lord's cut never reached the books: {:.0}", g.sites[castle].treasury);
}

#[test]
fn test_a_gift_train_goes_down_with_escorts() {
    let mut g = Game::new_seeded(7);
    let ci = g.sites.iter().position(|s| s.kind == SiteKind::Castle).unwrap();
    let ti = (0..g.sites.len()).find(|&t| g.sites[t].liege == Some(ci)).expect("no tributary");
    g.sites[ci].treasury = 200.0;
    // this (castle, tributary) pair's own day
    let r = (ci * 31 + ti * 7) as u64 % 45;
    g.country_tick(if r == 0 { 45 } else { r });
    let runner = g.sites[ci].lord.or(g.sites[ci].carrier).unwrap();
    let t = g.travels.iter().find(|t| t.who == runner && matches!(t.goal, Goal::Gift(..)))
        .expect("no gift train rode out");
    assert!(matches!(t.goal, Goal::Gift(c, tt) if c == ci && tt == ti), "wrong train: {:?}", t.goal);
    // the guard rides too: castle blades on the same road
    let escorts = g.travels.iter().filter(|t| matches!(t.goal, Goal::Escort(..))).count();
    assert!(escorts >= 1, "the train rode unguarded");
    // and on arrival the tributary's books take the cargo's worth
    let before = g.sites[ti].treasury;
    g.arrive(runner, Goal::Gift(ci, ti));
    assert!(g.sites[ti].treasury > before, "the gift sold for nothing");
}

#[test]
fn test_a_rich_castle_remits_to_the_crown() {
    let mut g = Game::new_seeded(11);
    let ci = g.sites.iter().position(|s| s.kind == SiteKind::Castle).unwrap();
    g.sites[ci].treasury = 200.0;
    let r = (ci as u64 * 11) % 90;
    g.country_tick(if r == 0 { 90 } else { r });
    let kept = g.sites[ci].treasury;
    assert!(kept < 200.0, "the crown's cut never left: {:.0}", kept);
    assert!(kept >= 40.0 - 0.01, "the reserve went with it: {:.0}", kept);
    assert!(g.log.iter().any(|l| l.contains("crown")), "the remittance went unsaid");
}
