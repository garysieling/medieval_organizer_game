use crate::session::*;
use crate::game::villager::Drunk;

/// The village's taproom: whichever of the four trades pours here
fn drink_shop(g: &Game) -> Option<Entity> {
    g.building_entities.iter().copied().find(|&be|
        g.world.get_component::<Building>(be)
            .map(|b| matches!(b.kind, BuildingKind::Shop(
                Vocation::Alewife | Vocation::Brewer | Vocation::Vintner | Vocation::Distiller))
                && b.worker.is_some())
            .unwrap_or(false))
}

#[test]
fn test_every_village_keeps_a_drink_trade() {
    let g = Game::new();
    assert!(DRINK_TRADES.iter().any(|&dv| g.villagers.iter().any(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == dv).unwrap_or(false))),
        "one of the four pours here");
    assert!(drink_shop(&g).is_some(), "and the taproom's a real place");
    assert!(g.log.iter().any(|l| l.contains("The village keeps")));
}

#[test]
fn test_small_ale_sours_and_the_spoil_sells() {
    let mut g = Game::new();
    let shop = drink_shop(&g).unwrap();
    {
        let b = g.world.get_component_mut::<Building>(shop).unwrap();
        b.stock.clear();
        b.stock.insert("ale".into(), 8);
    }
    g.drink_tick(2);
    let b = g.world.get_component::<Building>(shop).unwrap();
    assert!(b.stock.get("ale").copied().unwrap_or(0) < 8, "the ale's turning");
    assert!(b.stock.get("vinegar").copied().unwrap_or(0) > 0, "the spoil's vinegar");
    assert!(vocation::price("vinegar") > 0.0, "and vinegar still sells");
}

#[test]
fn test_wine_turns_on_a_shelf_and_keeps_in_the_dark() {
    let mut g = Game::new();
    let shelf = g.building_entities.iter().copied().find(|&be|
        g.world.get_component::<Building>(be)
            .map(|b| b.kind != BuildingKind::Cellar).unwrap_or(false)).unwrap();
    {
        let b = g.world.get_component_mut::<Building>(shelf).unwrap();
        b.stock.clear();
        b.stock.insert("wine".into(), 4);
    }
    if let Some(cellar) = g.cellar {
        g.world.get_component_mut::<Building>(cellar).unwrap().stock.insert("wine".into(), 4);
    }
    g.drink_tick(12); // a wine day, not a casked one
    assert_eq!(g.world.get_component::<Building>(shelf).unwrap()
        .stock.get("wine").copied().unwrap_or(0), 3, "a shelf bottle turns");
    if let Some(cellar) = g.cellar {
        assert_eq!(g.world.get_component::<Building>(cellar).unwrap()
            .stock.get("wine").copied().unwrap_or(0), 4, "the dark keeps it");
    }
}

#[test]
fn test_a_cask_buys_the_wine_time() {
    let mut g = Game::new();
    let mut bare = None;
    let mut casked = None;
    for &be in &g.building_entities {
        let is_cellar = g.world.get_component::<Building>(be)
            .map(|b| b.kind == BuildingKind::Cellar).unwrap_or(false);
        if is_cellar { continue; }
        if bare.is_none() { bare = Some(be); }
        else if casked.is_none() { casked = Some(be); break; }
    }
    let (bare, casked) = (bare.unwrap(), casked.unwrap());
    {
        let b = g.world.get_component_mut::<Building>(bare).unwrap();
        b.stock.clear(); b.stock.insert("wine".into(), 4);
    }
    {
        let b = g.world.get_component_mut::<Building>(casked).unwrap();
        b.stock.clear(); b.stock.insert("wine".into(), 4); b.stock.insert("casks".into(), 2);
    }
    g.drink_tick(12);
    assert_eq!(g.world.get_component::<Building>(bare).unwrap()
        .stock.get("wine").copied().unwrap_or(0), 3, "a bare bottle turns");
    assert_eq!(g.world.get_component::<Building>(casked).unwrap()
        .stock.get("wine").copied().unwrap_or(0), 4, "the cask holds it");
}

#[test]
fn test_a_cup_goes_down_at_the_counter() {
    let mut g = Game::new();
    let shop = drink_shop(&g).unwrap();
    g.world.get_component_mut::<Building>(shop).unwrap().stock.insert("ale".into(), 5);
    let buyer = g.villagers[8];
    g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 20.0;
    g.arrive(buyer, Goal::BuyAt(shop, "ale", Some(Want::Drink)));
    assert!(desires::is_satisfied(&g.world, buyer, Want::Drink, g.tick),
        "the thirst's answered");
    assert_eq!(g.world.get_component::<Satchel>(buyer).unwrap()
        .goods.get("ale").copied().unwrap_or(0), 0, "nothing carried home");
    assert_eq!(g.world.get_component::<Building>(shop).unwrap().stock["ale"], 4);
}

#[test]
fn test_enough_cups_and_the_morning_after_comes() {
    let mut g = Game::new();
    let shop = drink_shop(&g).unwrap();
    g.world.get_component_mut::<Building>(shop).unwrap().stock.insert("beer".into(), 40);
    let buyer = g.villagers[9];
    g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 500.0;
    for _ in 0..40 {
        if g.world.get_component::<Drunk>(buyer).is_some() { break; }
        g.arrive(buyer, Goal::BuyAt(shop, "beer", Some(Want::Drink)));
    }
    assert!(g.world.get_component::<Drunk>(buyer).is_some(), "the morning after comes");
}

#[test]
fn test_the_collector_counts_the_barrels() {
    let mut g = Game::new();
    let v = g.villagers[5];
    let shop = g.building_entities.iter().copied().find(|&be|
        g.world.get_component::<Building>(be)
            .map(|b| b.worker.is_none() && household::is_workplace(b.kind)).unwrap_or(false))
        .unwrap_or_else(|| drink_shop(&g).unwrap());
    {
        let b = g.world.get_component_mut::<Building>(shop).unwrap();
        b.stock.clear();
        b.stock.insert("beer".into(), 10); // 70 gold of drink
    }
    g.world.get_component_mut::<Craft>(v).unwrap().workplace = Some(shop);
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    let c = g.collector.unwrap();
    let before = g.world.get_component::<Purse>(c).unwrap().gold;
    g.arrive(c, Goal::CollectTax(v));
    // the year's cut plus the ale excise: 5.0 + (70 * 0.15) = 5 + 11
    let after = g.world.get_component::<Purse>(c).unwrap().gold;
    assert!((after - before - 16.0).abs() < 0.5,
        "the purse cut and the barrel count both came ({})", after - before);
    assert!(g.log.iter().any(|l| l.contains("ale excise")));
}

#[test]
fn test_the_still_owes_the_wise_woman() {
    let mut g = Game::new();
    let still = g.building_entities[10];
    {
        let b = g.world.get_component_mut::<Building>(still).unwrap();
        b.kind = BuildingKind::Shop(Vocation::Distiller);
        b.stock.insert("spirits".into(), 3);
    }
    let hut = g.herb_hut.expect("the herb hut stands on the classic map");
    g.world.get_component_mut::<Building>(hut).unwrap().stock.remove("spirits");
    g.drink_tick(10); // the fortnight's measure
    assert_eq!(g.world.get_component::<Building>(hut).unwrap()
        .stock.get("spirits").copied().unwrap_or(0), 1, "a measure for the tinctures");
    assert_eq!(g.world.get_component::<Building>(still).unwrap()
        .stock.get("spirits").copied().unwrap_or(0), 2);
}

#[test]
fn test_a_watch_earns_its_keep_at_the_cellar() {
    let mut g = Game::new();
    let shop = drink_shop(&g).unwrap();
    {
        let b = g.world.get_component_mut::<Building>(shop).unwrap();
        b.stock.clear();
        b.stock.insert("spirits".into(), 8); // 200 gold - worth stealing
    }
    // post a watch - the classic village rolls none of its own
    g.world.add_component(g.villagers[3], vocation::craft(Vocation::Knight, 0.5));
    g.force_event(events::EventId::CellarThief);
    assert!(g.log.iter().any(|l| l.contains("catches a thief")),
        "a living watch catches him: {:?}", g.log.last());
    assert_eq!(g.world.get_component::<Building>(shop).unwrap()
        .stock.get("spirits").copied().unwrap_or(0), 8, "the good stock stays");
    // no watch, no lock: hang the knights and the drink walks
    for &v in &g.villagers.clone() {
        if g.world.get_component::<Craft>(v)
            .map(|c| c.vocation == Vocation::Knight).unwrap_or(false) {
            g.world.add_component(v, villager::Dead);
        }
    }
    g.force_event(events::EventId::CellarThief);
    assert_eq!(g.world.get_component::<Building>(shop).unwrap()
        .stock.get("spirits").copied().unwrap_or(0), 4, "half the drink walks off");
    assert!(g.log.iter().any(|l| l.contains("thief in the night")));
}
