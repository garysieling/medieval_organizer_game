use crate::session::*;

#[test]
fn test_word_rides_ahead_of_the_minstrel() {
    let mut g = Game::new();
    let due = g.next_minstrel;
    g.livelihood_tick(due - 3);
    assert!(g.log.iter().any(|l| l.contains("a minstrel's on the road")),
        "no word that he's due");
    assert!(g.minstrel.is_none(), "the word landed him early");
    g.livelihood_tick(due);
    assert!(g.minstrel.is_some(), "he never arrived on his day");
}

#[test]
fn test_everyone_gets_an_early_call() {
    // a long drought of no shows ends: every kind's owed one inside
    // the first season, then the road's pace resumes
    let g = Game::new();
    assert!(g.next_minstrel <= 90, "the minstrel's first call is late: {}", g.next_minstrel);
    assert!(g.next_pedlar <= 120, "the pedlar's first call is late: {}", g.next_pedlar);
    assert!(g.next_circus <= 90, "the circus's first call is late: {}", g.next_circus);
    assert!(g.next_ink_boiler <= 90, "the ink-maker's first call is late: {}", g.next_ink_boiler);
}

#[test]
fn test_the_minstrel_really_walks_in() {
    let mut g = Game::new();
    g.spawn_minstrel();
    let m = g.minstrel.as_ref().expect("he never showed");
    assert!(!m.path.is_empty(), "he stands where he spawned - no road walked");
    let me = m.e;
    let before = *g.world.get_component::<Pos>(me).unwrap();
    let village = g.village;
    for _ in 0..400 { g.advance_minstrel(); }
    let after = *g.world.get_component::<Pos>(me).unwrap();
    // he's closer to the market than where the road let him off
    let d = |p: &Pos| (p.x - village.0 as f32).abs() + (p.y - village.1 as f32).abs();
    assert!(d(&after) < d(&before), "he never made ground - a word, not a man");
    // and the town heard it: a toast names him
    assert!(g.toasts.iter().any(|t| t.0.contains("minstrel")), "no word that he's in");
}

#[test]
fn test_no_road_no_show_no_lie() {
    let mut g = Game::new();
    // a road that can't be walked: the far end's off the map's face
    g.traveller_path = vec![(9999, 9999)];
    g.spawn_minstrel();
    assert!(g.minstrel.is_none(), "a ghost minstrel stands off the map");
    assert!(!g.log.iter().any(|l| l.contains("minstrel")),
        "the log claimed a visitor who never walked");
    assert_eq!(g.next_minstrel, g.tick + 1, "he isn't trying the road again tomorrow");
}

#[test]
fn test_the_troupe_pitches_its_camp() {
    let mut g = Game::new();
    g.spawn_circus();
    let c = g.circus.as_ref().expect("no circus");
    assert_eq!(c.members.len(), 4, "a one-man show isn't a troupe");
    // walk them in: the camp pitches at the meadow
    for _ in 0..3000 { g.advance_circus(); }
    let c = g.circus.as_ref().expect("they never made the meadow");
    assert!(c.camped_until.is_some(), "they're still walking");
    // and the performers are at the camp, not back down the road
    let (cx, cy) = (c.camp.0 as f32 + 0.5, c.camp.1 as f32 + 0.5);
    for &m in &c.members {
        let p = g.world.get_component::<Pos>(m).unwrap();
        assert!((p.x - cx).abs() + (p.y - cy).abs() < 12.0,
            "a performer's out on the moor ({:.0},{:.0}) while the tents go up ({:.0},{:.0})",
            p.x, p.y, cx, cy);
    }
}
