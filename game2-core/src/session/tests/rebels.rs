//! The woods' rebel bands (#652): outlaws camped in the trees who keep
//! their own hours, fall on country caravans that pass close, and
//! leave caches for whoever finds them first.

use crate::session::*;
use crate::session::country::SiteKind;
use crate::game::villager::Pos;

/// A seeded game that actually dealt a band - most seeds do, so the
/// scan ends fast; the empty-woods seeds are the exception
fn game_with_rebels() -> Game {
    for seed in 1..60u64 {
        let g = Game::new_seeded(seed);
        if !g.rebel_camps.is_empty() { return g; }
    }
    panic!("no seed in 1..60 dealt a rebel band");
}

/// Stand `v` on `t` (tile coords, centre)
fn put_at(g: &mut Game, v: Entity, t: (i32, i32)) {
    if let Some(p) = g.world.get_component_mut::<Pos>(v) {
        p.x = t.0 as f32 + 0.5;
        p.y = t.1 as f32 + 0.5;
    }
}

#[test]
fn test_the_woods_keep_their_own_hours() {
    let mut g = game_with_rebels();
    let rebels: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<rebels::Rebel>(v).is_some()).collect();
    assert!(!rebels.is_empty(), "no rebels despite camps");
    // every one is an outlander at the sentinel site: the country's
    // rolls - food, meetings, the collector, the fire bell - all pass
    // them by on the same check
    for &v in &rebels {
        assert!(matches!(g.world.get_component::<country::Outlander>(v),
            Some(country::Outlander { site: usize::MAX })));
        assert!(g.world.get_component::<villager::Hunger>(v).is_none(),
            "a rebel went hungry - the woods feed them");
        assert!(g.world.get_component::<Craft>(v).is_none(),
            "a rebel took up a trade");
    }
    // five days of the daily loop: every rebel's plan stays a wander
    // in the woods, and nobody walks to the village's business
    for day in 1..=5u64 {
        g.livelihood_tick(day);
    }
    for &v in &rebels {
        let r = g.world.get_component::<routine::Routine>(v)
            .expect("a rebel with no plan");
        assert!(matches!(r.mode, routine::Mode::Wandering), "a rebel's day: {:?}", r.mode);
        assert!(matches!(r.spot, routine::Spot::Woods), "a rebel's ground: {:?}", r.spot);
        let band = g.world.get_component::<rebels::Rebel>(v).unwrap().band;
        let (cx, cy) = g.rebel_camps[band];
        let p = g.world.get_component::<Pos>(v).unwrap();
        let d = ((p.x - cx as f32).abs() + (p.y - cy as f32).abs()) as i32;
        assert!(d <= 14, "a rebel wandered {} tiles from camp {}", d, band);
    }
}

#[test]
fn test_a_caravan_ambushed_turns_for_home_and_may_leave_a_cache() {
    let mut g = game_with_rebels();
    // a carrier out on the road: the cart's errand, fabricated the way
    // country_tick sends it - the test only needs the goal and a
    // traveller standing on it
    let who = g.villagers[0];
    let si = g.sites.iter().position(|s| matches!(s.kind, SiteKind::Town))
        .expect("no town for the cart's index");
    let wild = g.rebel_camps[0];
    let band = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<rebels::Rebel>(v)
            .map(|r| r.band == 0).unwrap_or(false))
        .next().expect("band 0 has no members");
    // walk the days: each dawn a fresh cart rides through the band, the
    // band's own roll decides, and a good haul may leave a cache
    let mut robbed = false;
    for day in 1..=300u64 {
        g.travels.retain(|t| !matches!(t.goal, Goal::Cart(_)));
        g.travels.push(Travel { who, path: vec![], goal: Goal::Cart(si) });
        put_at(&mut g, who, wild);
        put_at(&mut g, band, (wild.0 + 1, wild.1));
        g.tick = day;
        g.rebel_ambush();
        if g.travels.iter().any(|t| t.who == who
            && matches!(t.goal, Goal::HeadHome(_))) {
            if !robbed {
                robbed = true;
                assert!(g.log.iter().any(|l| l.contains("rebels fall on")),
                    "the ambush went unlogged: {:?}",
                    g.log.iter().rev().take(8).collect::<Vec<_>>());
            }
        }
        if !g.rebel_stores.is_empty() { break; }
    }
    assert!(robbed, "the band never fell on the cart in 300 days");
    assert!(!g.rebel_stores.is_empty(),
        "no cache in 300 robbed-or-not days - the 45% roll never came up");
    // standing by it offers the taking; taking fills the satchel and
    // empties the store
    let store = g.rebel_stores[0].clone();
    assert!(!store.goods.is_empty());
    let pe = g.seats[0].player;
    put_at(&mut g, pe, store.tile);
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::TakeCache)
        .expect("no Take the cache offered");
    let before: u32 = store.goods.iter().map(|&(_, n)| n).sum();
    let satchel_before = g.world.get_component::<Satchel>(pe)
        .map(|s| s.goods.values().sum::<u32>()).unwrap_or(0);
    g.ctx_act_at(0, i);
    assert!(g.rebel_stores.is_empty(), "the cache is still there");
    let satchel_after = g.world.get_component::<Satchel>(pe)
        .map(|s| s.goods.values().sum::<u32>()).unwrap_or(0);
    assert_eq!(satchel_after - satchel_before, before, "the satchel didn't take the cache");
    // and once it's gone the verb's gone with it
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::TakeCache));
}

#[test]
fn test_a_cache_left_too_long_is_gone() {
    let mut g = game_with_rebels();
    let tile = g.rebel_camps[0];
    g.rebel_stores.push(rebels::RebelStore {
        tile, goods: vec![("wheat", 2)], until: g.tick + 5 });
    let pe = g.seats[0].player;
    put_at(&mut g, pe, tile);
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::TakeCache),
        "a fresh cache didn't offer itself");
    // past its day the store's gone: the woods took it back
    g.tick += 6;
    g.rebel_ambush(); // the per-frame tidy drops what's gone off
    assert!(g.rebel_store_at(0).is_none());
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::TakeCache));
}
