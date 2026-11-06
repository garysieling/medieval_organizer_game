use crate::session::*;
use crate::sim::psychology::*;

#[test]
fn test_a_talk_is_a_warmth_beat_and_the_style_reads_it() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    let w0 = g.world.get_component::<AttachmentHistory>(a).map(|h| h.warmth_episodes).unwrap_or(0);
    // a talk between them is a warmth beat for both
    g.push_talk(crate::session::dialogue::TalkKind::Meeting, vec![a, b], vec![crate::session::dialogue::TalkLine { who: Some(a), name: "A".into(), text: "how do ye?".into(), gist: false }]);
    let h = g.world.get_component::<AttachmentHistory>(a).unwrap();
    assert!(h.warmth_episodes > w0, "no warmth counted");
    assert!(h.episodes.iter().any(|e| matches!(e.kind, EpisodeKind::Warmth) && e.with == b));
    // and a bond forms
    assert!(g.world.get_component::<Bond>(a).map(|bo| bo.target == b).unwrap_or(false));
    // the style shapes how a heart warms: an avoidant one warms slow
    let d = g.villagers[3];
    g.world.add_component(d, AttachmentHistory { anxiety_episodes: 2, warmth_episodes: 1, episodes: vec![] });
    let c = g.villagers[2];
    g.world.add_component(c, AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 8, episodes: vec![] });
    record_episode(&mut g.world, d, Episode { with: b, kind: EpisodeKind::Warmth });
    record_episode(&mut g.world, c, Episode { with: b, kind: EpisodeKind::Warmth });
    let avoid = g.world.get_component::<Bond>(d).map(|bo| bo.security).unwrap_or(0.0);
    let sec = g.world.get_component::<Bond>(c).map(|bo| bo.security).unwrap_or(0.0);
    assert!(sec > avoid, "the avoidant heart warmed as fast: {} vs {}", sec, avoid);
}
