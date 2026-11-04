use crate::session::*;

#[test]
fn test_a_shepherd_keeps_the_fold() {
    let g = Game::new();
    let sh = g.shepherd.expect("a shepherd keeps the fold");
    assert_eq!(g.world.get_component::<Craft>(sh).unwrap().vocation, Vocation::Shepherd);
    let fold = g.fold.unwrap();
    let b = g.world.get_component::<Building>(fold).unwrap();
    assert_eq!(b.kind, BuildingKind::Pasture);
    assert_eq!(b.name, "the fold");
    assert_eq!(b.worker, Some(sh), "the fold knows its shepherd");
    assert_eq!(g.world.get_component::<Craft>(sh).unwrap().workplace, Some(fold));
    assert!(g.flock.len() >= 8, "a decent flock, got {}", g.flock.len());
}

#[test]
fn test_the_sheepdog_belongs_to_the_shepherd() {
    let g = Game::new();
    let sh = g.shepherd.unwrap();
    assert!(g.pets.iter().any(|(e, s, _)| *s == Species::Dog
        && g.world.get_component::<Pet>(*e).map(|p| p.owner == Some(sh)).unwrap_or(false)),
        "a herding dog at his heel");
}

#[test]
fn test_milk_and_wool_come_off_the_fold() {
    let mut g = Game::new();
    let fold = g.fold.unwrap();
    let sh = g.shepherd.unwrap();
    for t in 0..3 { livelihood::work_day(&mut g.world, sh, t); }
    assert!(g.world.get_component::<Building>(fold).unwrap().stock.get("milk").copied().unwrap_or(0) > 0,
        "the goats give milk daily");
    g.livestock_tick(45); // shearing day, mid-season
    assert!(g.world.get_component::<Building>(fold).unwrap().stock.get("wool").copied().unwrap_or(0) > 0,
        "the sheep are shorn in season");
}

#[test]
fn test_the_shepherd_smells_of_it() {
    let g = Game::new();
    let sh = g.shepherd.unwrap();
    assert!(desires::problems_of(&g.world, sh, 0).contains(&desires::Problem::Smelly));
}

#[test]
fn test_a_wolf_night_losses_and_the_loss_grievance() {
    let mut g = Game::new();
    let fold = g.fold.unwrap();
    let before = livestock::flock(&g.world, fold).len();
    g.wolf_night(60);
    let after = livestock::flock(&g.world, fold).len();
    assert!(before - after <= 2, "wolves take a couple at most");
    if let Some(sh) = g.shepherd {
        assert!(desires::problems_of(&g.world, sh, 60).contains(&desires::Problem::Loss),
            "the loss sticks on him for the season");
    }
}

#[test]
fn test_the_flock_stays_near_the_fold() {
    let mut g = Game::new();
    let fold = g.fold.unwrap();
    let fp = *g.world.get_component::<Pos>(fold).unwrap();
    for _ in 0..(10 * FPS) { g.update(); }
    for &a in &g.flock {
        let p = g.world.get_component::<Pos>(a).unwrap();
        let d = (p.x - fp.x).abs() + (p.y - fp.y).abs();
        assert!(d < 20.0, "an animal strayed too far ({:.0} tiles)", d);
    }
}

#[test]
fn test_the_flock_grazes_as_individuals_and_stays_off_the_walls() {
    let mut g = Game::new();
    for _ in 0..(4 * FPS) { g.update(); }
    // each wanders on its own clock - the old lockstep drift set them
    // all the same target at once
    let dues: crate::engine::det::HashSet<u32> = g.flock.iter()
        .filter_map(|a| g.flock_wander.get(a).map(|w| w.2)).collect();
    assert!(dues.len() > 1, "the animals wander on their own clocks");
    let spots: crate::engine::det::HashSet<(i32, i32)> = g.flock.iter()
        .filter_map(|a| g.flock_wander.get(a).map(|w| (w.0 as i32, w.1 as i32))).collect();
    assert!(spots.len() > 1, "the animals head for their own spots");
    // and over a few seconds none of them ends up on a wall or in the water
    for _ in 0..(2 * FPS) {
        g.update();
        for &a in &g.flock {
            let p = g.world.get_component::<Pos>(a).unwrap();
            assert!(g.walkable(p.x, p.y), "an animal off the ground at {:.1},{:.1}", p.x, p.y);
        }
    }
}
