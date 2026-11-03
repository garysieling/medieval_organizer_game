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
