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
