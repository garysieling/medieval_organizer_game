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

/// The sidebar's bond ledger: folk the seat has spoken to, counted by
/// the stage each relationship has reached
#[test]
fn test_the_panel_counts_the_bonds_the_seat_has_made() {
    let mut game = Game::new();
    while game.tick < 3 { game.update(); }
    let n = game.villagers.len().min(4);
    let (player, tick) = (game.player, game.tick);
    for i in 0..n {
        let v = game.villagers[i];
        crate::game::player::act_talk(&mut game.world, player, v, tick);
    }
    let m = game.ui_model();
    // rapport is talk-only: the ledger counts exactly who this seat
    // has conversed with, not bonds made some other way
    assert_eq!(m.panel.social.met, n, "each talked-to villager counts");
    let stages = m.panel.social.acquaintances + m.panel.social.known
        + m.panel.social.followers + m.panel.social.coleaders;
    assert_eq!(stages, m.panel.social.met, "met is the stages summed");
    assert!(m.panel.social.acquaintances >= 1, "a first conversation reaches acquaintance");
}

/// The print shop's work is the village's research: the tree rides the
/// game, the model carries it to the client, and a printmaker's days
/// feed the next art on the track
#[test]
fn test_the_print_shop_researches_the_village() {
    let mut game = Game::new();
    assert!(!game.tech.is_empty(), "the village keeps a research tree");
    assert!(game.tech.iter().all(|t| !t.discovered), "nothing's discovered on day one");
    // the model carries the tree for the client
    assert_eq!(game.ui_model().tech.len(), game.tech.len());
    // put a printmaker on the bench and let the days turn
    let v = game.villagers.iter().copied()
        .find(|&v| game.world.get_component::<Kid>(v).is_none()
            && !villager::is_dead(&game.world, v)).unwrap();
    if let Some(c) = game.world.get_component_mut::<Craft>(v) {
        c.vocation = Vocation::Printmaker;
    }
    // a day off the bench is a day off the research too
    game.plan_rest(v, game.tick);
    game.tech_tick(game.tick);
    assert!(game.tech.iter().all(|t| t.progress == 0.0 && !t.discovered),
        "a resting printmaker researches nothing");
    // at the bench the days add up
    game.plan_bench(v, game.tick);
    for _ in 0..40 { game.tech_tick(game.tick); }
    let invested: f32 = game.tech.iter().map(|t| t.progress).sum();
    assert!(invested > 0.0 || game.tech.iter().any(|t| t.discovered),
        "the print shop's work researches the track");
    // and the model sees the investment
    let m = game.ui_model();
    let seen: f32 = m.tech.iter().map(|t| t.progress).sum();
    assert!(seen > 0.0 || m.tech.iter().any(|t| t.discovered));
}
