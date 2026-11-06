use crate::session::*;

#[test]
fn test_a_slot_fills_from_its_fellows() {
    let mut g = Game::new();
    // a baker with rye on the shelf but no wheat still bakes a day
    let (v, shop) = {
        let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
            .map(|c| c.vocation == Vocation::Baker).unwrap_or(false)).expect("no baker");
        let shop = g.world.get_component::<Craft>(v).unwrap().workplace.expect("baker has no shop");
        (v, shop)
    };
    let b = g.world.get_component_mut::<Building>(shop).unwrap();
    b.stock.retain(|g, _| g != "wheat");
    *b.stock.entry("rye".into()).or_insert(0) = 2;
    // empty their satchel of wheat so the shelf is the only source
    if let Some(s) = g.world.get_component_mut::<Satchel>(v) {
        s.goods.retain(|g, _| g != "wheat");
    }
    let (made, _) = g.bench_work(v, g.tick);
    assert!(made > 0.0, "a baker with rye on hand couldn't work the day");
    assert_eq!(g.world.get_component::<Building>(shop).unwrap().stock.get("rye").copied().unwrap_or(0), 1,
        "the rye wasn't spent");
    // and the shop's card owns up to it
    let bi = g.building_entities.iter().position(|b| *b == shop).unwrap();
    g.seats[0].selected_building = Some(bi);
    let insp = g.building_inspector_at(0).unwrap();
    assert!(insp.lines.iter().any(|l| l.text.contains("-1 rye")),
        "the ledger hides the swap: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}

#[test]
fn test_substitute_stock_counts_as_work_to_do() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Baker).unwrap_or(false)).expect("no baker");
    let shop = g.world.get_component::<Craft>(v).unwrap().workplace.unwrap();
    let b = g.world.get_component_mut::<Building>(shop).unwrap();
    b.stock.retain(|g, _| g != "wheat");
    *b.stock.entry("rye".into()).or_insert(0) = 1;
    if let Some(s) = g.world.get_component_mut::<Satchel>(v) {
        s.goods.retain(|g, _| g != "wheat" && g != "rye");
    }
    // rye on the shelf is a day's work, not a shortage
    assert!(livelihood::needs_input(&g.world, v).is_none(),
        "rye in the shop still read as missing wheat");
}

#[test]
fn test_a_missing_second_slot_spends_nothing() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Brewer).unwrap_or(false)).expect("no brewer");
    // barley for the grain slot, nothing at all for hops
    if let Some(s) = g.world.get_component_mut::<Satchel>(v) {
        s.goods.retain(|g, _| !["wheat", "hops"].contains(&g.as_str()));
        *s.goods.entry("barley".into()).or_insert(0) = 1;
    }
    if let Some(w) = g.world.get_component::<Craft>(v).unwrap().workplace {
        let b = g.world.get_component_mut::<Building>(w).unwrap();
        b.stock.retain(|g, _| !["wheat", "hops", "barley"].contains(&g.as_str()));
    }
    let (made, _) = g.bench_work(v, g.tick);
    assert_eq!(made, 0.0);
    assert_eq!(g.world.get_component::<Satchel>(v).unwrap().goods.get("barley").copied().unwrap_or(0), 1,
        "a day that couldn't run spent the grain anyway");
}
