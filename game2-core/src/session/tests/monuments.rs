use crate::session::*;
use crate::sim::story::{StoryEvent, EventKind};

fn give_tale(g: &mut Game, i: usize, intensity: f32) {
    let evs = vec![
        StoryEvent { tick: 40, kind: EventKind::Conflict, description: "the fire".into() },
        StoryEvent { tick: 41, kind: EventKind::Achievement, description: "they beat it".into() },
        StoryEvent { tick: 42, kind: EventKind::Bonding, description: "the village knows".into() },
        StoryEvent { tick: 43, kind: EventKind::Recovery, description: "a scar".into() },
        StoryEvent { tick: 44, kind: EventKind::Peace, description: "put to rest".into() },
        StoryEvent { tick: 45, kind: EventKind::Achievement, description: "the deed".into() },
        StoryEvent { tick: 46, kind: EventKind::Bonding, description: "the toast".into() },
    ];
    g.tales.insert(i, evs);
    let _ = intensity;
}

#[test]
fn test_a_worthy_tale_rises_a_monument() {
    let mut g = Game::new();
    give_tale(&mut g, 0, 0.8);
    // the lord can pay - fund them all, whichever's nearest fronts it
    for s in g.sites.iter_mut() { s.treasury = 100.0; }
    g.monument_tick(50);
    assert_eq!(g.monuments.len(), 1, "no monument proposed");
    let m = &g.monuments[0];
    assert_eq!(m.honoree, 0);
    assert!(!m.title.is_empty());
    assert!(m.stood_on.is_none(), "it stood too soon");
    assert!(g.sites.iter().any(|s| (s.treasury - 60.0).abs() < 0.1), "no lord paid");
    // and it stands when its days are done
    g.monument_tick(50 + 20);
    assert!(g.monuments[0].stood_on.is_some(), "it never stood");
    // the model names it
    let v = g.ui_model();
    assert!(v.monuments.iter().any(|(_, _, w)| !w.is_empty()), "the stone's words are missing");
}

#[test]
fn test_no_gold_no_stone() {
    let mut g = Game::new();
    give_tale(&mut g, 0, 0.8);
    for s in g.sites.iter_mut() { s.treasury = 0.0; }
    g.monument_tick(50);
    assert!(g.monuments.is_empty(), "the lord's empty purse raised a monument");
}

#[test]
fn test_the_village_honors_only_its_greatest() {
    let mut g = Game::new();
    for i in 0..8 { give_tale(&mut g, i, 0.8); }
    for s in g.sites.iter_mut() { s.treasury = 500.0; }
    for _ in 0..8 { g.monument_tick(50); }
    assert!(g.monuments.len() <= 4, "the map is all columns now: {}", g.monuments.len());
}
