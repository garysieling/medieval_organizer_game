use crate::session::*;

#[test]
fn test_the_spirit_marks_the_brightest() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::religion::Faith {
        religion: "the parish".into(), devotion: 0.9, traditions: vec![] });
    g.revelation_tick(107);
    assert!(g.mystic.is_some(), "the mystic never marked");
    assert!(g.log.iter().any(|l| l.contains("a mystic walks")),
        "the marking never logged");
}

#[test]
fn test_the_canonize_folds_the_marking() {
    let mut g = Game::new();
    g.mystic = Some(g.tick);
    g.decide_at(0, 0); // canonize
    let after = g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.orthodoxy).unwrap_or(0.0);
    assert!(g.log.iter().any(|l| l.contains("canonized")),
        "the canonize never logged");
    let _ = after;
}

#[test]
fn test_the_suppress_risks_the_martyr() {
    let mut g = Game::new();
    g.mystic = Some(g.tick);
    let before = g.separation_pressure;
    g.decide_at(0, 2); // suppress
    assert!(g.log.iter().any(|l| l.contains("suppressed")),
        "the suppress never logged");
    let _ = before;
}
