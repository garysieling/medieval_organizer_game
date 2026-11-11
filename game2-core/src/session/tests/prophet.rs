use crate::session::*;
use crate::game::prophetic::{self, Follower, Movement};

/// Hand the village a prophet and a grievance pool: a movement's the
/// thing being tested, not the odds
fn seed_movement(g: &mut Game, tick: u64) {
    let players: Vec<Entity> = g.seats.iter().map(|s| s.player).collect();
    let prophet = prophetic::pick_prophet(&g.world, &g.villagers, &players)
        .expect("an adult should be callable");
    g.prophet_movement = Some(Movement {
        prophet, since: tick,
        deadline: tick + prophetic::DEADLINE_DAYS,
        remnant: false, martyred: false,
    });
    for &v in &g.villagers {
        if v == prophet { continue; }
        for p in [crate::game::desires::Problem::Taxed,
                  crate::game::desires::Problem::Broke,
                  crate::game::desires::Problem::Loss] {
            crate::game::desires::add_grievance(&mut g.world, v, p, tick + 200);
        }
    }
}

#[test]
fn test_the_calling_finds_a_voice() {
    let mut g = Game::new();
    seed_movement(&mut g, 100);
    let m = g.prophet_movement.as_ref().unwrap();
    assert_eq!(m.deadline, 100 + prophetic::DEADLINE_DAYS, "the vision names its day");
    assert!(!m.remnant);
    assert!(!villager::is_dead(&g.world, m.prophet), "the calling takes a living adult");
}

#[test]
fn test_the_crowd_grows_with_the_pool() {
    let mut g = Game::new();
    seed_movement(&mut g, 100);
    for t in 100..110 { g.prophet_tick(t); }
    let crowd = prophetic::followers(&g.world, &g.villagers);
    assert!(!crowd.is_empty(), "the grievance pool should feed the crowd");
    assert!(crowd.len() <= prophetic::CROWD_CAP, "crowds are broad, not endless");
    // the thin kind: none of them can hold a seat
    let oe = g.seats[0].org;
    for &v in &crowd {
        assert!(!crate::sim::community::recruit(&mut g.world, oe, v, 1.0),
            "a follower marches; they don't sign on");
    }
}

#[test]
fn test_a_quiet_village_hears_no_calling() {
    let mut g = Game::new();
    // no grievances pinned: the pool stays dry
    for t in 60..400 { g.prophet_tick(t); }
    assert!(g.prophet_movement.is_none(),
        "an unwronged village can't be prophesied to");
}

#[test]
fn test_the_day_passing_ends_or_hardens() {
    let mut g = Game::new();
    seed_movement(&mut g, 100);
    let deadline = g.prophet_movement.as_ref().unwrap().deadline;
    g.prophet_tick(deadline);
    match g.prophet_movement.as_ref() {
        None => {
            // demobilized: not a Follower left standing
            assert!(prophetic::followers(&g.world, &g.villagers).is_empty(),
                "the scattered crowd sheds its calling");
        }
        Some(m) => {
            assert!(m.remnant, "post-deadline is remnant mode or nothing");
            let kept = prophetic::followers(&g.world, &g.villagers);
            assert!(kept.len() <= prophetic::REMNANT_KEEP,
                "the remnant is the hardened few, not the crowd");
            assert!(kept.iter().all(|&v| g.world.get_component::<Follower>(v)
                .map(|f| f.remnant).unwrap_or(false)), "the kept few hardened");
        }
    }
}

#[test]
fn test_a_dead_prophet_ends_or_converts() {
    let mut g = Game::new();
    seed_movement(&mut g, 100);
    let m = g.prophet_movement.as_ref().unwrap();
    let prophet = m.prophet;
    // the head comes off: the Dead marker is what is_dead reads
    g.world.add_component(prophet, villager::Dead);
    g.prophet_tick(150);
    match g.prophet_movement.as_ref() {
        None => {
            assert!(prophetic::followers(&g.world, &g.villagers).is_empty(),
                "no head, no movement");
        }
        Some(_) => {
            // the martyrdom branch: a grave does the growing now
            assert!(g.martyr_grave.is_some(),
                "the conversion leaves the martyr's plot");
            assert!(!prophetic::followers(&g.world, &g.villagers).is_empty(),
                "the grave draws a crowd the man never did");
        }
    }
}

#[test]
fn test_the_remnant_stops_growing() {
    let mut g = Game::new();
    seed_movement(&mut g, 100);
    if let Some(m) = &mut g.prophet_movement { m.remnant = true; }
    let before = prophetic::followers(&g.world, &g.villagers).len();
    for t in 100..115 { g.prophet_tick(t); }
    assert_eq!(prophetic::followers(&g.world, &g.villagers).len(), before,
        "a remnant doesn't recruit");
}
