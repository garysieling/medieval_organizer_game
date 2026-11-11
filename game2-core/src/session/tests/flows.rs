//! The surplus's road (#970): the market's own claim on the stalls'
//! fees rides guarded to the keep; the crown's levy loads as a real
//! cart the road can test; and the woods answer both - an escort that
//! stands to them, or spoils scattered and carried off.

use crate::session::*;
use crate::session::country::SiteKind;
use crate::game::villager::Pos;

/// Stand `v` on `t` (tile coords, centre)
fn put_at(g: &mut Game, v: Entity, t: (i32, i32)) {
    if let Some(p) = g.world.get_component_mut::<Pos>(v) {
        p.x = t.0 as f32 + 0.5;
        p.y = t.1 as f32 + 0.5;
    }
}

/// A seeded game that actually dealt a band
fn game_with_rebels() -> Game {
    for seed in 1..60u64 {
        let g = Game::new_seeded(seed);
        if !g.rebel_camps.is_empty() { return g; }
    }
    panic!("no seed in 1..60 dealt a rebel band");
}

fn castle_index(g: &Game) -> usize {
    g.sites.iter().position(|s| matches!(s.kind, SiteKind::Castle))
        .expect("no castle site dealt")
}

#[test]
fn test_the_stalls_fees_stay_with_the_market_and_the_ride_goes_guarded() {
    let mut g = Game::new();
    let claimant = g.market_claimant().expect("no market faction stands");
    let before = g.world.get_component::<Organization>(claimant).unwrap().resources;
    g.takings = 30.0;
    g.livelihood_tick(45); // the remit day
    let after = g.world.get_component::<Organization>(claimant).unwrap().resources;
    assert!(after > before, "the claim drew nothing from the takings");
    // the lord's cut rides diminished by what the market kept
    let c = g.collector.unwrap();
    let t = g.travels.iter().find(|t| t.who == c).expect("the collector never left");
    let Goal::Remit(_, amt) = t.goal else { panic!("his errand's no remit: {:?}", t.goal) };
    assert!(amt < 30.0, "the ride still carries the market's share: {}", amt);
    // and the claim's own people see it home: a faction escort rides beside him
    let si = g.sites.iter().position(|s| matches!(s.kind, SiteKind::Castle))
        .expect("no castle for the ride");
    assert!(g.travels.iter().any(|t| t.who != c
        && matches!(t.goal, Goal::Escort(_, to) if to == si)),
        "nobody rode escort on the gold");
    assert!(g.log.iter().any(|l| l.contains("the stalls' fees stay in the market")),
        "the claim went unsaid");
}

#[test]
fn test_the_crowns_cart_reaches_the_keep() {
    let mut g = Game::new();
    g.imperial_project = Some(ImperialProject { until: 400, rate: 6, eased_until: 0 });
    if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) { f.servings = 200; }
    g.imperial_tick(65); // a cart day
    let c = g.collector.unwrap();
    let t = g.travels.iter().find(|t| t.who == c).expect("the cart never left");
    let Goal::LevyCart(si) = t.goal else { panic!("the errand's no cart: {:?}", t.goal) };
    let store = g.sites[si].store;
    let before = g.world.get_component::<villager::FoodSource>(store).unwrap().servings;
    // ride the road out: the flag holds the whole way, and the keep's
    // shelves take the load (less the rider's own meal off the top)
    for _ in 0..12000 {
        g.advance_travels();
        if !g.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::LevyCart(_))) { break; }
    }
    assert!(!g.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::LevyCart(_))),
        "the cart never got there");
    let after = g.world.get_component::<villager::FoodSource>(store).unwrap().servings;
    assert!(after >= before + 59, "the keep's shelves took {} of a 60-load", after - before);
}

#[test]
fn test_an_escort_answers_the_ambush() {
    let mut g = game_with_rebels();
    let who = g.villagers[0];
    let escort = g.villagers[1];
    let si = castle_index(&g);
    let wild = g.rebel_camps[0];
    let band = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<rebels::Rebel>(v)
            .map(|r| r.band == 0).unwrap_or(false))
        .expect("band 0 has no members");
    // walk the days: each dawn the guarded ride passes the band again,
    // until the band's roll comes up and the escort answers it
    let mut repelled = false;
    for day in 1..=400u64 {
        g.travels.retain(|t| t.who != who && t.who != escort);
        g.travels.push(Travel { who, path: vec![], goal: Goal::Remit(si, 30.0) });
        g.travels.push(Travel { who: escort, path: vec![], goal: Goal::Escort(usize::MAX, si) });
        put_at(&mut g, who, wild);
        put_at(&mut g, escort, (wild.0 + 1, wild.1));
        put_at(&mut g, band, (wild.0 - 1, wild.1));
        g.tick = day;
        g.rebel_ambush();
        if g.log.iter().any(|l| l.contains("the escort stands to them")) {
            repelled = true;
            // and the ride rides on: no turn for home, the gold still his
            assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::Remit(..))),
                "the repelled ride turned back anyway");
            break;
        }
    }
    assert!(repelled, "the escort never answered in 400 days");
}

#[test]
fn test_an_unguarded_gold_ride_pays_the_woods() {
    let mut g = game_with_rebels();
    let who = g.villagers[0];
    let si = castle_index(&g);
    let wild = g.rebel_camps[0];
    let band = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<rebels::Rebel>(v)
            .map(|r| r.band == 0).unwrap_or(false))
        .expect("band 0 has no members");
    // an unguarded remittance past the camp: when the band's roll comes
    // up the ride turns for home and the gold's gone either way -
    // carried off to a cache or scattered on the road
    let mut robbed = false;
    for day in 1..=400u64 {
        g.travels.retain(|t| t.who != who);
        g.travels.push(Travel { who, path: vec![], goal: Goal::Remit(si, 25.0) });
        put_at(&mut g, who, wild);
        put_at(&mut g, band, (wild.0 + 1, wild.1));
        g.tick = day;
        g.rebel_ambush();
        if g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::HeadHome(_))) {
            robbed = true;
            break;
        }
    }
    assert!(robbed, "the band never fell on the gold in 400 days");
}

#[test]
fn test_a_taken_cache_pays_its_gold() {
    let mut g = game_with_rebels();
    let tile = g.rebel_camps[0];
    g.rebel_stores.push(rebels::RebelStore {
        tile, goods: vec![("wheat", 2)], gold: 12.0, until: g.tick + 5 });
    let pe = g.seats[0].player;
    put_at(&mut g, pe, tile);
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::TakeCache)
        .expect("no Take the cache offered");
    let before = g.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
    g.ctx_act_at(0, i);
    let after = g.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
    assert_eq!(after - before, 12.0, "the cache's gold didn't reach the purse");
    assert!(g.rebel_stores.is_empty(), "the cache is still there");
}
