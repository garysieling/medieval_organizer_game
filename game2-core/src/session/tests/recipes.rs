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
