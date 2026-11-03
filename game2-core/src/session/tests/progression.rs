use crate::session::*;

fn influence(game: &Game) -> f32 {
    game.world.get_component::<PlayerCharacter>(game.player).unwrap().influence
}

/// Walk to each villager, hear them out, then win them over at house
/// meetings instead of with Recruit; finally invest the gold.
#[test]
fn test_the_village_can_still_be_won_when_people_join_at_meetings() {
    let mut game = Game::new();
    for i in 0..game.villagers.len() {
        let v = game.villagers[i];
        // folk die over the years this takes - a corpse can't come to a
        // meeting, whatever the player says beside it
        if villager::is_dead(&game.world, v) { continue; }
        let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
        { let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = vx + 0.5; p.y = vy; }
        game.selected_target = i;
        for _ in 0..8 { game.act(0); } // talk until there's nothing new to hear
        for _ in 0..40 {
            if game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v)
                || villager::is_dead(&game.world, v) { break; }
            // they go about their days between meetings: walk back over
            if let Some(&Pos { x: vx, y: vy }) = game.world.get_component::<Pos>(v) {
                let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = vx + 0.5; p.y = vy;
            }
            // trust decays a little every day - a meeting a fortnight
            // apart won't climb past it unless the talking keeps up
            game.act(0);
            game.act(3);
            finish_meeting(&mut game);
        }
        // he may have died mid-courtship - the years this takes carry
        // folk off, and a corpse can't come to a meeting
        if villager::is_dead(&game.world, v) { continue; }
        if !game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v) {
            let rel = relationships::find_relationship(&game.world, game.player, v)
                .and_then(|r| game.world.get_component::<Relationship>(r)).map(|r| r.strength);
            let last: Vec<&String> = game.log.iter().rev().take(4).collect();
            panic!("{} never joined at a meeting (dead {}, trust {:?}, last log {:?})",
                vill_name(i), villager::is_dead(&game.world, v), rel, last);
        }
    }
    while game.disabled_reason(2).is_none()
        && game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources >= 10.0 {
        game.act(2);
    }
    assert!(influence(&game) > WIN_INFLUENCE,
        "everyone joined and all gold invested, yet influence is only {:.2} (win at {})", influence(&game), WIN_INFLUENCE);
}
