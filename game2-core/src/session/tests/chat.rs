use crate::session::*;

fn trust(game: &Game, a: Entity, b: Entity) -> f32 {
    relationships::find_relationship(&game.world, a, b)
        .and_then(|r| game.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0)
}

/// Bram alone in reach of the player, everyone else far off - and the
/// pair's roll padded so a remark lands whatever came before it
fn bram_in_reach(game: &mut Game) {
    let v = game.villagers[2];
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 1.0, y: vy };
    // just the two of them: villagers meeting each other talk on the
    // record, not over the map, so a neighbour mustn't take the turn
    for (i, o) in game.villagers.clone().into_iter().enumerate() {
        if o != v { *game.world.get_component_mut::<Pos>(o).unwrap() = Pos { x: vx + 30.0 + 3.0 * i as f32, y: vy + 30.0 }; }
    }
    while (game.last_chat.len() + v as usize + game.tick as usize) % 3 != 0 {
        game.last_chat.push(((v, v), 0));
    }
}

#[test]
fn test_talk_puts_their_reply_in_a_speech_bubble() {
    let mut game = Game::new();
    let v = game.villagers[1];
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
    game.selected_target = 1;
    game.act(0);
    let (who, said, until, _, _) = game.bubbles.iter().find(|b| b.0 == v).expect("no speech bubble over Eliza").clone();
    assert_eq!(who, v);
    assert!(game.talk_log.last().unwrap().lines.iter().any(|l| l.gist && l.text == said),
        "the bubble says what the TALK panel records");
    assert!(until > game.frame_n + FPS, "the bubble stays up for seconds, not a frame");
}

#[test]
fn test_passing_someone_gets_a_remark_and_a_little_warmth() {
    let mut game = Game::new();
    let (pe, v) = (game.player, game.villagers[2]);
    // an acquaintance: one real exchange's worth of warmth already
    relationships::record_interaction(&mut game.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    bram_in_reach(&mut game);
    let before = trust(&game, pe, v);
    game.chat_in_passing();
    assert!(game.bubbles.iter().any(|b| b.0 == v), "Bram should say something as you pass");
    assert!((trust(&game, game.player, v) - before - 0.01).abs() < 1e-4, "a chat in passing is worth +0.01");
    // standing about together doesn't keep generating chatter
    game.bubbles.clear();
    for _ in 0..10 { game.chat_in_passing(); }
    assert!(!game.bubbles.iter().any(|b| b.0 == v), "they already said hello");
}

#[test]
fn test_a_stranger_walks_past_quiet() {
    let mut game = Game::new();
    let v = game.villagers[2];
    bram_in_reach(&mut game);
    let before = trust(&game, game.player, v);
    game.chat_in_passing();
    // they don't know you yet: no hello - but the passing still counts
    assert!(!game.bubbles.iter().any(|b| b.0 == v), "a stranger doesn't say hi");
    assert!(trust(&game, game.player, v) > before, "the passing still warms the bond");
}

#[test]
fn test_someone_who_knows_you_says_hi() {
    use crate::game::desires::{Known, Problem, Reveal, Want};
    let mut game = Game::new();
    let (pe, v) = (game.player, game.villagers[2]);
    // known: a bond plus three pieces of their self-interest heard
    relationships::record_interaction(&mut game.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    game.world.add_component(v, Known { revealed: vec![
        Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love), Reveal::Problem(Problem::Taxed)] });
    bram_in_reach(&mut game);
    game.chat_in_passing();
    let bubble = game.bubbles.iter().find(|b| b.0 == v).expect("someone who knows you says hi");
    let name = game.speaker_name(pe);
    assert!(bubble.1.contains(&name), "the hello names who it greets: {}", bubble.1);
}
