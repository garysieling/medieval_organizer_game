use crate::session::*;

#[test]
fn test_feet_wear_the_ground_in() {
    let mut g = Game::new();
    // send someone out and back a few times - a desire-line
    // is a way walked over and over
    let who = g.villagers[0];
    let p = g.world.get_component::<Pos>(who).unwrap();
    let start = (p.x.round() as i32, p.y.round() as i32);
    for _ in 0..4 {
        let now = {
            let p = g.world.get_component::<Pos>(who).unwrap();
            (p.x.round() as i32, p.y.round() as i32)
        };
        let dest = if now == start { (start.0 + 4, start.1) } else { start };
        let path = g.map.walk_path(now, dest).unwrap_or_else(|| vec![dest]);
        g.travels.push(Travel { who, path, goal: Goal::Wander });
        for _ in 0..400 { g.advance_travels(); }
    }
    assert!(!g.trails.is_empty(), "feet leave a mark");
    assert!(g.trails.values().any(|t| t.strength > 0.2), "the way wears in");
}

#[test]
fn test_an_unwalked_path_greens_over() {
    let mut g = Game::new();
    g.trails.insert((5, 5), Trail { strength: 0.4 });
    for _ in 0..30 { g.trail_tick(); }
    let s = g.trails.get(&(5, 5)).map(|t| t.strength).unwrap_or(0.0);
    assert!(s < 0.3, "the grass takes it back");
}

/// An open east-west run near the village, with a field south of it -
/// (from, to, row) for the first row whose straight way keeps clear of
/// the cheap ground
fn open_run(g: &Game) -> Option<((i32, i32), (i32, i32), i32)> {
    let (vx, vy) = g.village;
    for row in vy - 6..vy + 30 {
        let (from, to) = ((vx - 10, row), (vx + 10, row));
        let Some(direct) = g.map.walk_path(from, to) else { continue };
        let cheap = |x: i32, y: i32| (vx - 6..=vx + 6).contains(&x) && (row + 5..=row + 9).contains(&y);
        if direct.iter().filter(|&&(x, y)| cheap(x, y)).count() >= 5 { continue; }
        return Some((from, to, row));
    }
    None
}

#[test]
fn a_cheaper_way_wins_even_when_longer() {
    let g = Game::new();
    let (from, to, row) = open_run(&g).expect("an open run with a field beside it");
    let (vx, _) = g.village;
    let cheap = |x: i32, y: i32| (vx - 6..=vx + 6).contains(&x) && (row + 5..=row + 9).contains(&y);
    let cost = |path: &[(i32, i32)]| path.iter().map(|&(x, y)| if cheap(x, y) { 0.1 } else { 1.0 }).sum::<f32>();
    let direct = g.map.walk_path(from, to).unwrap();
    let wise = g.map.walk_path_weighted(from, to, 0.1, |x, y| if cheap(x, y) { 0.1 } else { 1.0 })
        .expect("still a way there");
    assert!(wise.iter().all(|&(x, y)| g.map.passable(x, y)), "only open ground");
    assert_eq!(wise.last(), Some(&to));
    assert!(wise.iter().any(|&(x, y)| cheap(x, y)), "it keeps to the cheap ground");
    assert!(cost(&wise) < cost(&direct), "the cheap way beats the short one");
}

#[test]
fn the_worn_way_is_the_walk() {
    let g = Game::new();
    let (from, to, row) = open_run(&g).expect("an open run with a field beside it");
    let (vx, _) = g.village;
    // wear the field south of the run: folk should dip down into it
    let mut worn = crate::engine::det::HashMap::default();
    for y in row + 5..=row + 9 {
        for x in vx - 6..=vx + 6 {
            worn.insert((x, y), Trail { strength: 0.85 });
        }
    }
    let path = Game::trail_path_on(&g.map, &worn, &g.flood, from, to).expect("still a way there");
    assert!(path.iter().any(|&(x, y)| worn.contains_key(&(x, y))), "folk keep to the worn way");
    assert!(path.iter().all(|&(x, y)| g.map.passable(x, y)), "only open ground");
    assert_eq!(path.last(), Some(&to));
}
