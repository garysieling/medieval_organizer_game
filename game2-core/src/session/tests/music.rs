use crate::session::*;
use crate::game::music::{self, Mood};
use crate::game::feuds;
use crate::engine::ecs::System;

#[test]
fn test_the_villages_tune_follows_the_day_it_had() {
    let mut g = Game::new();
    // open conflict long enough to hold: the tune turns tense and the
    // log says so
    let (a, b) = (g.villagers[0], g.villagers[1]);
    feuds::spawn_feud(&mut g.world, b, a, "a slight");
    feuds::spawn_feud(&mut g.world, a, b, "an answer");
    for _ in 0..20 { feuds::FeudSystem.run(&mut g.world); }
    for t in 0..20 { g.music_tick(t); }
    let mood = g.world.get_component::<music::MusicState>(g.music).unwrap().mood;
    assert_eq!(mood, Mood::Tense, "burning feuds never turned the tune tense");
    assert!(g.log.iter().any(|l| l.contains("tune turns")), "the turn went unsaid");
}
