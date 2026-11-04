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

/// The original walk_path: a fresh breadth-first search per call. The
/// reused-scratch version must return exactly what this does.
fn reference_walk_path(map: &crate::game::tilemap::TileMap, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    use crate::game::tilemap::NEIGHBOURS;
    let (w, h) = (map.width as i32, map.height as i32);
    if !map.passable(to.0, to.1) || from.0 < 0 || from.1 < 0 || from.0 >= w || from.1 >= h { return None; }
    let idx = |p: (i32, i32)| (p.1 * w + p.0) as usize;
    let mut prev: Vec<Option<(i32, i32)>> = vec![None; (w * h) as usize];
    let mut seen = vec![false; (w * h) as usize];
    let mut queue = std::collections::VecDeque::from([from]);
    seen[idx(from)] = true;
    while let Some(p) = queue.pop_front() {
        if p == to {
            let mut path = vec![];
            let mut at = Some(p);
            while let Some(q) = at {
                if q != from { path.push(q); }
                at = prev[idx(q)];
            }
            path.reverse();
            return Some(path);
        }
        for (dx, dy) in NEIGHBOURS {
            let q = (p.0 + dx, p.1 + dy);
            if q.0 < 0 || q.1 < 0 || q.0 >= w || q.1 >= h || seen[idx(q)] || !map.passable(q.0, q.1) { continue; }
            seen[idx(q)] = true;
            prev[idx(q)] = Some(p);
            queue.push_back(q);
        }
    }
    None
}

#[test]
fn walk_path_matches_the_plain_breadth_first_search() {
    let g = Game::new();
    let (w, h) = (g.map.width as i32, g.map.height as i32);
    let mut rng = Rng(7);
    let mut found = 0;
    for _ in 0..400 {
        let from = ((rng.next() % w as u64) as i32, (rng.next() % h as u64) as i32);
        // mostly near walks, the way the village walks, and some across the map
        let to = if rng.next() % 4 == 0 {
            ((rng.next() % w as u64) as i32, (rng.next() % h as u64) as i32)
        } else {
            (from.0 + (rng.next() % 41) as i32 - 20, from.1 + (rng.next() % 41) as i32 - 20)
        };
        let want = reference_walk_path(&g.map, from, to);
        assert_eq!(g.map.walk_path(from, to), want, "{from:?} -> {to:?}");
        found += want.is_some() as u32;
        // and the same pair again: the reused scratch must not remember the last search
        assert_eq!(g.map.walk_path(from, to), want);
    }
    assert!(found > 100, "most pairs have a walk ({found}/400)");
    assert_eq!(g.map.walk_path((10, 10), (10, 10)), reference_walk_path(&g.map, (10, 10), (10, 10)));
}
