use crate::session::*;

#[test]
fn test_the_pedlar_pitches_and_sells() {
    let mut g = Game::new();
    // two hearts for finery, whatever the want-roll dealt the village
    for &v in &g.villagers[..2] {
        if let Some(l) = g.world.get_component_mut::<InnerLife>(v) {
            if !l.wants.contains(&Want::Finery) { l.wants.push(Want::Finery); }
        }
    }
    g.spawn_pedlar();
    assert!(g.pedlar.is_some());
    // walk him in - the pitch happens once he's stood at market
    for _ in 0..400 {
        if g.pedlar.as_ref().map(|p| p.selling_until.is_some()).unwrap_or(false) { break; }
        g.advance_pedlar();
    }
    let m = g.pedlar.as_ref().unwrap();
    assert!(m.stall.is_some(), "he pitches a stall");
    let stall = m.stall.unwrap();
    let ribbon = g.world.get_component::<Building>(stall)
        .map(|b| b.stock.get("ribbon").copied().unwrap_or(0)).unwrap_or(0);
    assert!(ribbon > 0, "oddities nothing local supplies");
    assert!(g.travels.iter().any(|t| matches!(t.goal, Goal::BuyFrom(s, "ribbon", _)
        if s == stall)), "the word goes out to the finery-wanters");
}

#[test]
fn test_the_pedlar_carries_the_famed_name() {
    let mut g = Game::new();
    // a famed printer with sheets on the shelf: the run already pulled
    let shop = household::spawn_building(&mut g.world,
        BuildingKind::Shop(Vocation::Printmaker), "the print shop", 5.0, 5.0, 400);
    let maker = g.villagers[0];
    g.world.get_component_mut::<Craft>(maker).unwrap().vocation = Vocation::Printmaker;
    g.world.get_component_mut::<Craft>(maker).unwrap().workplace = Some(shop);
    household::add_stock(g.world.get_component_mut::<Building>(shop).unwrap(), "prints", 8);
    let purse_before = g.world.get_component::<Purse>(maker).map(|p| p.gold).unwrap_or(0.0);
    g.famed_printer = Some(maker);
    g.spawn_pedlar();
    for _ in 0..400 {
        if g.pedlar.as_ref().map(|p| p.selling_until.is_some()).unwrap_or(false) { break; }
        g.advance_pedlar();
    }
    let m = g.pedlar.as_ref().expect("he pitches");
    assert!(m.famed_sheets > 0, "he takes a sheaf of the run");
    let left = g.world.get_component::<Building>(shop)
        .map(|b| b.stock.get("prints").copied().unwrap_or(0)).unwrap_or(0);
    assert_eq!(left, 8 - m.famed_sheets, "the sheaf comes off the shop's shelf");
    let purse_after = g.world.get_component::<Purse>(maker).map(|p| p.gold).unwrap_or(0.0);
    assert!(purse_after > purse_before, "the maker's paid for the sheets");
    // and when the road takes him, the name goes with the sheets
    g.tick = 10;
    for _ in 0..900 {
        if g.pedlar.is_none() { break; }
        g.advance_pedlar();
    }
    assert!(g.log.iter().any(|l| l.contains("the name with them")),
        "the name travels: {:?}", g.log.last());
}

#[test]
fn test_the_road_takes_him() {
    let mut g = Game::new();
    g.spawn_pedlar();
    for _ in 0..400 {
        if g.pedlar.as_ref().map(|p| p.path.is_empty() && p.selling_until.is_some()).unwrap_or(false) { break; }
        g.advance_pedlar();
    }
    // run him past his selling days
    g.tick = 10;
    for _ in 0..900 {
        g.advance_pedlar();
        if g.pedlar.is_none() { break; }
    }
    assert!(g.pedlar.is_none(), "the road takes him");
    assert!(g.next_pedlar > g.tick, "another comes by one day");
}
