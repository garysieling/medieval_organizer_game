use crate::session::*;

#[test]
fn test_the_path_memo_holds_for_the_day() {
    let mut g = Game::new();
    let p1 = g.walk_memo((10, 10), (15, 15));
    assert!(p1.is_some(), "the walk's still walkable");
    assert!(g.path_cache.contains_key(&((10, 10), (15, 15))), "it's remembered");
    // a day older and it's stale
    g.tick = 1;
    g.walk_memo((10, 10), (15, 15));
    assert_eq!(g.path_tick, 1, "the cache turned over");
    assert_eq!(g.path_cache.len(), 1, "fresh, not accumulated");
}
