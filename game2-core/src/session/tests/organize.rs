use crate::session::*;
use crate::game::relationships::RelKind;

#[test]
fn test_organize_holds_a_house_meeting_with_nearby_villagers() {
    let mut game = Game::new();
    let (eliza, bram, corvin) = (game.villagers[1], game.villagers[2], game.villagers[3]);
    // the player stands among Eliza and Bram; Corvin is far away
    game.world.get_component_mut::<Pos>(game.player).unwrap().x = 16.0;
    game.world.get_component_mut::<Pos>(game.player).unwrap().y = 16.0;
    for (v, x, y) in [(eliza, 17.0, 16.0), (bram, 15.0, 17.0), (corvin, 2.0, 2.0)] {
        let p = game.world.get_component_mut::<Pos>(v).unwrap();
        p.x = x; p.y = y;
    }
    // everyone else is off across the fields, well out of reach
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32 * 0.5, y: 3.0 }; }
    }
    // Eliza already trusts the player; Bram is a stranger
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });

    assert_eq!(game.disabled_reason(3), None, "a house meeting needs someone nearby, not the selected villager");
    game.act(3);
    finish_meeting(&mut game);

    let members = game.world.get_component::<Organization>(game.org).unwrap().members.clone();
    assert_eq!(members, vec![eliza], "the trusted neighbour joins; the stranger and the absent don't");
    let last = game.log.last().unwrap();
    assert!(last.contains("House meeting - 2 came") && last.contains("Eliza joined"), "{}", last);
    assert!(relationships::find_relationship(&game.world, game.player, bram).is_some(), "Bram got to know the player");
    // a distant villager doesn't attend - the members list above
    // already says Eliza alone; Corvin's absence is no member and
    // no meeting-bond either (long sim windows make incidental bonds)
    assert!(!members.contains(&corvin), "Corvin wasn't there");
}

#[test]
fn test_a_meeting_takes_time_people_gather_then_talk_then_it_ends() {
    let mut game = Game::new();
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (eliza, bram) = (game.villagers[1], game.villagers[2]);
    for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
    }
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });
    game.act(3);
    assert!(game.log.last().unwrap().contains("You call a house meeting at"), "{}", game.log.last().unwrap());
    assert!(game.world.get_component::<Organization>(game.org).unwrap().members.is_empty(), "nothing's decided yet");
    assert_eq!(game.disabled_reason(3).as_deref(), Some("a meeting is under way"));
    // they walk over...
    let spots = game.meeting.as_ref().unwrap().spots.clone();
    let far = |game: &Game| [eliza, bram].iter().zip(&spots).map(|(v, s)| {
        let p = game.world.get_component::<Pos>(*v).unwrap(); (p.x - s.0 as f32).abs() + (p.y - s.1 as f32).abs()
    }).sum::<f32>();
    let start = far(&game);
    for _ in 0..FPS { game.update(); }
    assert!(far(&game) < start, "people walk to the meeting");
    // ...then talk it over in thought bubbles, and only then does it end
    let mut thoughts = vec![];
    let mut frames = 0;
    while game.meeting.is_some() {
        game.update();
        frames += 1;
        for b in &game.bubbles { if b.3 && !thoughts.contains(&(b.0, b.1.clone())) { thoughts.push((b.0, b.1.clone())); } }
        assert!(frames < 60 * FPS, "the meeting never ended");
    }
    assert!(thoughts.iter().any(|t| t.0 == eliza) && thoughts.iter().any(|t| t.0 == bram) && thoughts.iter().any(|t| t.0 == game.player),
        "everyone had their say: {:?}", thoughts);
    assert!(frames > 5 * LINE_FRAMES, "a meeting takes a while ({} frames)", frames);
    assert_eq!(game.world.get_component::<Organization>(game.org).unwrap().members, vec![eliza]);
    assert!(game.log.last().unwrap().contains("Eliza joined"), "{}", game.log.last().unwrap());
}

/// Three villagers aggrieved over the taxes, in the player's org, and the
/// organizer holding the issue: a house meeting will end in the founding call
fn org_ready(g: &mut Game) {
    for &v in &g.villagers[..3] {
        crate::game::desires::add_grievance(&mut g.world, v, crate::game::desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<crate::game::desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
}

#[test]
fn test_an_unanswered_founding_call_lapses_instead_of_halving_pace_forever() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id);
    assert!(g.seats[0].pending_founding.is_some());
    // held open: the calendar runs at the house-meeting's half pace
    let base = if g.speed == 1 { 7 * FPS } else { 5 * FPS / 4 };
    assert_eq!(g.ticks_per_frame(), base * 2);
    // short of a week unanswered: still held open, still slow
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS - 1);
    assert!(g.seats[0].pending_founding.is_some());
    assert_eq!(g.ticks_per_frame(), base * 2);
    // a week unanswered (no client ever sent Command::Decide): the
    // call lapses on its own, same as letting it rest, and the
    // calendar returns to its ordinary pace
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS);
    assert!(g.seats[0].pending_founding.is_none());
    assert_eq!(g.ticks_per_frame(), base);
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_none());
}
