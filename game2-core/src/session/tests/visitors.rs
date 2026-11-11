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

#[test]
fn test_the_crowds_word_draws_the_road() {
    // a market day pulls a traveller whose round was near into town
    // today (#1073)
    let mut g = Game::new();
    g.tick = 20; // market day: tick % 30 == 20
    g.next_minstrel = g.tick + 5;   // inside the draw
    g.next_circus = g.tick + 40;    // far out - untouched
    let tinker = crate::session::events::EventId::Tinker;
    g.next_rare.retain(|(id, _)| *id != tinker);
    g.next_rare.push((tinker, g.tick + 5));
    g.livelihood_tick(g.tick);
    assert!(g.minstrel.is_some(), "the word never reached the minstrel");
    assert!(g.rare.iter().any(|r| r.id == tinker), "the tinker never came for the crowd");
    assert!(g.circus.is_none() && g.next_circus == g.tick + 40,
        "a distant round shouldn't be drawn in");
}

#[test]
fn test_no_road_still_stands_his_pitch() {
    // a Visitor firing with no road to the pitch used to "arrive"
    // standing at the road's far end - in town in name only (#1073)
    let mut g = Game::new();
    g.traveller_path = vec![(9999, 9999)];
    let tinker = crate::session::events::EventId::Tinker;
    g.force_event(tinker);
    let r = g.rare.iter().find(|r| r.id == tinker).expect("the tinker never fired");
    let e = r.e.expect("no figure walked in");
    let p = g.world.get_component::<Pos>(e).unwrap();
    let mp = g.world.get_component::<Pos>(g.market_food)
        .map(|p| (p.x, p.y)).unwrap_or((g.village.0 as f32, g.village.1 as f32));
    let d = ((p.x - mp.0).abs() + (p.y - mp.1).abs()) as i32;
    assert!(d <= 10, "he stands {} tiles from the market - fired but never seen", d);
}

#[test]
fn test_the_minstrels_round_comes_back_sooner() {
    // the road's pace after a visit: season, not year (#1073)
    let mut g = Game::new();
    g.spawn_minstrel();
    for _ in 0..4000 {
        g.advance_minstrel();
        if g.minstrel.is_none() { break; }
        if g.minstrel.as_ref().and_then(|m| m.playing_until).map(|u| g.tick >= u).unwrap_or(false) { break; }
    }
    // his song's done: let the road take him
    g.tick += 3;
    for _ in 0..8000 {
        g.advance_minstrel();
        if g.minstrel.is_none() { break; }
    }
    assert!(g.minstrel.is_none(), "he never rode off");
    let gap = g.next_minstrel - g.tick;
    assert!((90..=150).contains(&gap), "his next round's {} days out - a year's drought again", gap);
}
