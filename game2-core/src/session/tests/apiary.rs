use crate::session::*;

#[test]
fn test_a_beekeeper_keeps_the_skeps() {
    let g = Game::new();
    let bk = g.beekeeper.expect("a beekeeper keeps the yard");
    assert_eq!(g.world.get_component::<Craft>(bk).unwrap().vocation, Vocation::Beekeeper);
    let ay = g.apiary.unwrap();
    let b = g.world.get_component::<Building>(ay).unwrap();
    assert_eq!(b.kind, BuildingKind::Apiary);
    assert_eq!(b.name, "the apiary");
    assert_eq!(b.worker, Some(bk));
    assert!(apiary::hives_of(&g.world, ay).len() >= 3, "a few skeps stand round it");
}

#[test]
fn test_the_skeps_yield_in_season() {
    let mut g = Game::new();
    let ay = g.apiary.unwrap();
    g.apiary_tick(50); // midsummer
    let b = g.world.get_component::<Building>(ay).unwrap();
    assert!(b.stock.get("honey").copied().unwrap_or(0) > 0, "honey for the table");
    // the wax went straight to the church's candles
    let cb = g.world.get_component::<Building>(g.church.unwrap()).unwrap();
    assert!(cb.stock.get("candle").copied().unwrap_or(0) > 0, "wax for the candles");
}

#[test]
fn test_a_dark_church_troubles_the_priest() {
    let mut g = Game::new();
    // burn day with no candles on the shelf
    g.apiary_tick(20);
    let priest = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
    if let Some(pr) = priest {
        assert!(desires::problems_of(&g.world, pr, 20).iter()
            .any(|p| matches!(p, desires::Problem::NoInput("wax"))), "no wax, dark church");
    }
}
