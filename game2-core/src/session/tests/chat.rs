use crate::session::*;

fn trust(game: &Game, a: Entity, b: Entity) -> f32 {
    relationships::find_relationship(&game.world, a, b)
        .and_then(|r| game.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0)
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
    let v = game.villagers[2];
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    let before = trust(&game, game.player, v);
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 1.0, y: vy };
    // just the two of them: villagers meeting each other talk on the
    // record, not over the map, so a neighbour mustn't take the turn
    for (i, o) in game.villagers.clone().into_iter().enumerate() {
        if o != v { *game.world.get_component_mut::<Pos>(o).unwrap() = Pos { x: vx + 30.0 + 3.0 * i as f32, y: vy + 30.0 }; }
    }
    // only some meetings get a bubble - pad the ledger so Bram's pair
    // lands on the remark roll whatever came before it
    while (game.last_chat.len() + v as usize + game.tick as usize) % 3 != 0 {
        game.last_chat.push(((v, v), 0));
    }
    game.chat_in_passing();
    assert!(game.bubbles.iter().any(|b| b.0 == v), "Bram should say something as you pass");
    assert!((trust(&game, game.player, v) - before - 0.01).abs() < 1e-4, "a chat in passing is worth +0.01");
    // standing about together doesn't keep generating chatter
    game.bubbles.clear();
    for _ in 0..10 { game.chat_in_passing(); }
    assert!(!game.bubbles.iter().any(|b| b.0 == v), "they already said hello");
}
