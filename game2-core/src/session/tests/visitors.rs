use crate::session::*;

#[test]
fn test_word_rides_ahead_of_the_minstrel() {
    let mut g = Game::new();
    let due = g.next_minstrel;
    g.livelihood_tick(due - 3);
    assert!(g.log.iter().any(|l| l.contains("a minstrel's on the road")),
        "no word that he's due");
    assert!(g.minstrel.is_none(), "the word landed him early");
    g.livelihood_tick(due);
    assert!(g.minstrel.is_some(), "he never arrived on his day");
}
