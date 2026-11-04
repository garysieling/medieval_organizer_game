use crate::session::*;

#[test]
fn test_rivals_and_travellers_get_poses() {
    // the rivals walk a cycle like villagers: they need a facing each
    let mut g = Game::new_seeded(7);
    for _ in 0..30 { g.update(); }
    let m = g.ui_model();
    assert!(!g.rivals.is_empty());
    assert!(g.rivals.iter().all(|r| m.poses.iter().any(|p| p.0 == *r)), "a pose for every rival");
    if g.traveller_pos().is_some() {
        assert!(m.poses.iter().any(|p| p.0 == 0 && p.2), "the courier rides with a facing");
    }
}

#[test]
fn test_walking_villagers_face_where_they_go() {
    let mut g = Game::new_seeded(7);
    for _ in 0..200 { g.update(); }
    let m = g.ui_model();
    assert!(g.villagers.iter().all(|v| m.poses.iter().any(|p| p.0 == *v)), "a pose for every villager");
    assert!(m.poses.iter().all(|p| p.1 < 4), "facings are walk-sheet rows");
    // the model straight after an update: whoever moved in that update
    // is mid-stride, facing that way; whoever didn't is standing (no
    // one-frame lag)
    let mut moved_some = false;
    for _ in 0..50 {
        let before: Vec<(f32, f32)> = g.villagers.iter().map(|&v| g.world.get_component::<Pos>(v).map(|p| (p.x, p.y)).unwrap_or_default()).collect();
        g.update();
        let m = g.ui_model();
        for (i, &v) in g.villagers.iter().enumerate() {
            let Some(p) = g.world.get_component::<Pos>(v) else { continue };
            let (dx, dy) = (p.x - before[i].0, p.y - before[i].1);
            let moved = dx.abs() + dy.abs() > 1e-4;
            let pose = m.poses.iter().find(|q| q.0 == v).unwrap();
            assert_eq!(pose.2, moved, "villager {}: walking shows this frame's movement", i);
            if moved { assert_eq!(pose.1, facing_on_screen(dx, dy)); moved_some = true; }
        }
    }
    assert!(moved_some, "someone walked in 50 frames");
}

#[test]
fn test_walking_turns_and_strides_the_player() {
    let mut g = Game::new_seeded(7);
    while g.tick < 5 { g.update(); }
    g.update();
    assert!(!g.ui_model().player_pose.1, "standing still isn't walking");
    // each way across the map shows its own row of the walk sheet
    let mut seen = vec![];
    for (dx, dy) in [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        for _ in 0..4 { g.command(Command::Walk(dx, dy)); g.update(); }
        let (facing, walking) = g.ui_model().player_pose;
        assert!(walking, "walking ({}, {}) shows the walk cycle", dx, dy);
        seen.push(facing);
    }
    assert_eq!(seen, vec![1, 2, 3, 0], "right, down, left, up on screen");
    g.update();
    assert!(!g.ui_model().player_pose.1, "stops striding when they stop");
}
