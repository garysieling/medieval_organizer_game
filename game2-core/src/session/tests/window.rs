use crate::session::*;
use crate::session::events::{EventId, RareEvent};

fn glazier_in(g: &mut Game, until: u64) {
    g.rare.push(RareEvent { id: EventId::Glazier, e: None, path: vec![],
        until: Some(until), at: Some((10, 10)), leaving: false, aux: 0,
        petters: crate::engine::det::HashMap::default() });
}

fn purses_up(g: &mut Game, gold: f32) {
    for &v in g.villagers.clone().iter() {
        if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = gold; }
    }
}

#[test]
fn test_the_parish_raises_and_the_glazier_hangs() {
    let mut g = Game::new();
    purses_up(&mut g, 5.0);
    glazier_in(&mut g, 200);
    g.events_tick(1);
    assert!(g.window.is_some(), "the offer never opened");
    // the hat goes round till the price is met, then the bench runs a
    // stage a day - a thin purse's glass cracks once in the firing
    let mut hung = 0;
    for d in 2..80 {
        g.events_tick(d);
        if g.window.as_ref().map(|w| w.installed).unwrap_or(false) { hung = d; break; }
    }
    assert!(hung > 0, "the window never hung");
    assert!(g.log.iter().any(|l| l.contains("cracked")), "the thin purse's firing never cracked");
    assert!(g.log.iter().any(|l| l.contains("light through it preaches")), "the hanging never said");
    // and the church wears it on the wire
    let m = g.ui_model();
    let church = m.buildings.iter().find(|b| b.sprite == BSprite::Church)
        .expect("no church on the wire");
    assert!(church.windowed, "the window never reached the wire");
}

#[test]
fn test_a_fat_purses_glass_fires_clean() {
    let mut g = Game::new();
    glazier_in(&mut g, 200);
    g.events_tick(1);
    g.window.as_mut().unwrap().fund = 60.0;
    for d in 2..12 { g.events_tick(d); }
    let w = g.window.as_ref().unwrap();
    assert!(w.installed, "the fine window never hung: stage {}", w.stage);
    assert!(!w.cracked, "the fine glass shouldn't have cracked");
    assert!(!g.log.iter().any(|l| l.contains("cracked")), "a crack got logged anyway");
}
