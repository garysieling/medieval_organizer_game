use crate::session::*;
use super::fixtures::plan_ready_meeting;

/// Walk to each villager, hear them out, then win them over at house
/// meetings: the whole village joining the cause is the win it can give.
#[test]
fn test_the_village_can_still_be_won_when_people_join_at_meetings() {
    let mut game = Game::new();
    // who was here when we started: visitors wander in over the years this
    // takes, and the village's win is over its own folk, not passing trade
    let folk = game.villagers.clone();
    for i in 0..folk.len() {
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
            // apart won't climb past it unless the talking keeps up.
            // Earning the meeting: their own house is the venue, and
            // they've said they'll come.
            game.act(0);
            let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
            plan_ready_meeting(&mut game, v, (vx, vy), &[v]);
            game.call_meeting();
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
    // and every living soul from the start is with the cause - the dead
    // keep their names on the roll, and so do the visitors who passed through
    let members = game.world.get_component::<Organization>(game.org).unwrap().members.clone();
    for &v in &folk {
        if !villager::is_dead(&game.world, v) {
            assert!(members.contains(&v), "{} is alive but never joined", game.name_of(v));
        }
    }
}
