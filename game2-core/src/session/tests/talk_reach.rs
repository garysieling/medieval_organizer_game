use crate::session::*;

#[test]
fn test_talk_goes_to_whoever_is_beside_you_not_the_last_clicked() {
    // on main, Talk with Aldric selected (the default) said "too far
    // away" while the player stood right next to Bram
    let mut game = Game::new();
    let (px, py) = game.player_pos();
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        let far = if i == 2 { (px + 1.0, py) } else { (px + 20.0 + i as f32, py + 20.0) };
        *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: far.0, y: far.1 };
    }
    game.selected_target = 0;
    assert_eq!(game.disabled_reason(0), None, "someone is beside you, so Talk should be usable");
    game.act(0);
    assert_eq!(game.selected_target, 2, "Talk should turn to the villager in reach");
    assert!(!game.log.last().map(|l| l.contains("Can't")).unwrap_or(false), "{:?}", game.log.last());
    let talk = game.talk_log.last().expect("a talk on the TALK panel");
    assert!(talk.with.contains(&game.villagers[2]), "talked with {}", vill_name(2));
}

/// A villager the player can walk to who's well out of talking distance
fn far_villager(game: &Game) -> usize {
    (0..game.villagers.len())
        // a villager across the village, not one out in the country
        .filter(|&i| game.site_of(game.villagers[i]).is_none())
        .filter(|&i| game.dist_to(i) > 4.0 && game.path_to(i).is_some())
        .max_by(|&a, &b| game.dist_to(a).partial_cmp(&game.dist_to(b)).unwrap())
        .expect("someone to walk over to")
}

#[test]
fn test_talk_on_a_picked_villager_walks_over_then_talks() {
    let mut game = Game::new();
    while game.tick < 3 { game.update(); }
    let who = far_villager(&game);
    game.selected_target = who;
    game.picked = true; // clicked on them
    let v = game.villagers[who];
    let talks = |g: &Game| g.world.get_component::<TalkCount>(v).map(|c| c.value).unwrap_or(0);
    let before = talks(&game);
    let start = game.dist_to(who);

    game.act(0);
    assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", vill_name(who))), "{}", game.log.last().unwrap());
    assert_eq!(talks(&game), before, "no talking from across the village");

    for _ in 0..(30 * FPS) {
        if game.approach.is_none() { break; }
        game.update();
    }
    assert!(game.approach.is_none(), "still walking to {} ({} tiles off, started {})", vill_name(who), game.dist_to(who), start);
    assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), vill_name(who));
    assert_eq!(talks(&game), before + 1, "talked once on arrival");
    assert!(game.talk_log.iter().any(|r| r.with.contains(&game.villagers[who]) && r.with.contains(&game.player)),
        "no talk on the TALK panel");
}

#[test]
fn test_recruit_on_a_picked_villager_walks_over_then_recruits() {
    let mut game = Game::new();
    while game.tick < 3 { game.update(); }
    let who = far_villager(&game);
    game.selected_target = who;
    game.picked = true;
    let org = game.org;
    crate::game::campaign::found_for_test(&mut game.world, org);
    game.world.add_component(game.villagers[who], crate::game::player::Rapport::for_player(game.player, 1.0));
    // and a bond deep enough to say yes to
    let rel = game.world.create_entity();
    game.world.add_component(rel, crate::game::relationships::Relationship {
        from: game.player, other: game.villagers[who],
        kind: crate::game::relationships::RelKind::Friend, strength: 0.7, interactions: 8 });
    assert_eq!(game.disabled_reason(1), None, "Recruit walks over, so distance doesn't grey it out");

    game.act(1);
    assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", vill_name(who))), "{}", game.log.last().unwrap());
    assert!(game.approach.as_ref().map(|a| a.action) == Some(1));

    for _ in 0..(30 * FPS) {
        if game.approach.is_none() { break; }
        game.update();
    }
    assert!(game.approach.is_none(), "still walking to {}", vill_name(who));
    assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), vill_name(who));
    // the recruit attempt happened on arrival (joined or turned the player down)
    let line = game.log.last().unwrap();
    assert!(line.contains(vill_name(who)) && !line.contains("walk over") && !line.contains("Can't"), "{}", line);
}

#[test]
fn test_recruit_is_hidden_until_a_campaign_exists() {
    let mut game = Game::new();
    while game.tick < 3 { game.update(); }
    let who = game.reach_target();
    game.selected_target = who;
    assert!(!action_shown(&game.ui_model(), 1), "no Recruit button before a campaign");
    assert_eq!(game.disabled_reason(1).as_deref(), Some("no campaign to join"));
    let org = game.org;
    crate::game::campaign::found_for_test(&mut game.world, org);
    assert!(action_shown(&game.ui_model(), 1), "Recruit appears once there's a campaign");
    // and even then it takes a few real conversations with them
    assert_eq!(game.disabled_reason(1).as_deref(), Some("talk with them more"));
    game.world.add_component(game.villagers[who], crate::game::player::Rapport::for_player(game.player, 1.0));
    assert_eq!(game.disabled_reason(1), None);
}

#[test]
fn test_talk_is_never_too_far_away() {
    let mut game = Game::new();
    let (px, py) = game.player_pos();
    let from = (px.round() as i32, py.round() as i32);
    // a tile the player can walk to, well out of reach
    let spot = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| ((x as f32 - px).powi(2) + (y as f32 - py).powi(2)).sqrt() > MEETING_REACH + 2.0
            && game.map.walk_path(from, (x, y)).is_some())
        .expect("somewhere far but reachable");
    let v = game.villagers[2];
    *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: spot.0 as f32, y: spot.1 as f32 };
    game.selected_target = 2;
    game.picked = true;
    assert_eq!(game.disabled_reason(0), None, "Talk walks over, so distance doesn't grey it out");
    game.act(0);
    assert!(game.approach.is_some() && game.log.last().unwrap().contains("You walk over to"), "{}", game.log.last().unwrap());
}
