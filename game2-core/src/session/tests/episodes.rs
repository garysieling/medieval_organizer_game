use crate::session::*;
use crate::game::episodes::{self, MemKind};

/// E6-2's done-when, the "callback truth is 100%" half: a filed
/// episode surfaces as a `mem:*` fact and a callback line at the next
/// talk - and nothing is remembered that wasn't filed
#[test]
fn test_a_filed_episode_surfaces_as_a_callback() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let pe = g.seats[0].player;
    episodes::file(&mut g.world, v, pe, MemKind::KeptPromise, g.tick, &["errand", "promise"]);
    g.open_player_talk(0, v, crate::game::conversation::Relation::Acquaintance, "the day's work");
    let t = g.seats[0].talk.as_ref().expect("a talk under way");
    assert!(t.villager.facts.open.iter().any(|f| f == "mem:you_kept_promise"),
        "the episode sets its mem fact: {:?}", t.villager.facts.open);
    assert!(t.notes.iter().any(|n| n.contains("kept your word")),
        "the callback line lands: {:?}", t.notes);
    // truth: the only mem facts on the speaker are the filed episodes'
    for f in t.villager.facts.open.iter().filter(|f| f.starts_with("mem:")) {
        assert_eq!(f, "mem:you_kept_promise", "an unfiled memory fired: {f}");
    }
}

/// No episode, no memory: the talk opens clean of `mem:*`
#[test]
fn test_no_episode_no_callback() {
    let mut g = Game::new();
    let v = g.villagers[7];
    g.open_player_talk(0, v, crate::game::conversation::Relation::Acquaintance, "the day's work");
    let t = g.seats[0].talk.as_ref().expect("a talk under way");
    assert!(!t.villager.facts.open.iter().any(|f| f.starts_with("mem:")),
        "a memory surfaced from nothing: {:?}", t.villager.facts.open);
    assert!(!t.notes.iter().any(|n| n.contains("kept your word") || n.contains("looked for you")),
        "a callback spoke of nothing: {:?}", t.notes);
}

/// Seat-scoped: an episode filed against a different seat's player is
/// not this seat's memory
#[test]
fn test_another_seats_memory_stays_theirs() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let other_pe = g.villagers[9];
    assert_ne!(other_pe, g.seats[0].player);
    episodes::file(&mut g.world, v, other_pe, MemKind::StoodWithUs, g.tick, &["meeting"]);
    g.open_player_talk(0, v, crate::game::conversation::Relation::Acquaintance, "the day's work");
    let t = g.seats[0].talk.as_ref().expect("a talk under way");
    assert!(!t.villager.facts.open.iter().any(|f| f.starts_with("mem:")),
        "another seat's memory leaked: {:?}", t.villager.facts.open);
}
