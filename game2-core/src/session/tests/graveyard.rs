use crate::session::*;

fn pos_of(g: &Game, v: Entity) -> (f32, f32) {
    let p = g.world.get_component::<Pos>(v).unwrap();
    (p.x, p.y)
}

/// Kill roster member `i` on `day` without running the whole frame:
/// the Dead marker is what is_dead sees, died_on is the funeral's clock
fn kill_on(g: &mut Game, i: usize, day: u64) {
    let v = g.villagers[i];
    g.world.add_component(v, villager::Dead);
    g.died_on[i] = Some(day);
}

#[test]
fn test_the_dead_lie_in_then_rest_in_the_graveyard() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    let v = g.villagers[0];
    let fell = pos_of(&g, v);
    // the lying-in: the body stays where it fell so folk can see it
    g.graveyard_tick(2);
    assert!(g.world.get_component::<villager::Buried>(v).is_none(), "buried inside the lying-in");
    assert_eq!(pos_of(&g, v), fell, "moved before the funeral");
    // day 3: carried to the churchyard, once
    g.graveyard_tick(3);
    assert!(g.world.get_component::<villager::Buried>(v).is_some(), "no burial on the funeral day");
    let rest = pos_of(&g, v);
    assert_ne!(rest, fell, "still lying where they fell");
    let gy = g.graveyard.as_ref().expect("the churchyard was planned");
    assert!(gy.slots.contains(&(rest.0.round() as i32, rest.1.round() as i32)),
        "the grave isn't one of the churchyard's plots");
    // and it never moves again
    g.graveyard_tick(4);
    g.graveyard_tick(90);
    assert_eq!(pos_of(&g, v), rest, "a grave wandered");
    assert!(g.log.iter().any(|l| l.contains("laid to rest")), "the log didn't mark the funeral");
}

#[test]
fn test_the_graveyard_fills_in_the_order_they_died() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 10);
    kill_on(&mut g, 1, 5);
    kill_on(&mut g, 2, 20);
    g.graveyard_tick(30);
    let gy = g.graveyard.as_ref().unwrap();
    assert_eq!(gy.taken, 3, "not every eligible dead was buried");
    // slots go in death order: day 5, then day 10, then day 20
    for (slot, i) in gy.slots.iter().take(3).zip([1usize, 0, 2]) {
        let p = pos_of(&g, g.villagers[i]);
        assert_eq!((p.0, p.1), (slot.0 as f32, slot.1 as f32),
            "villager {} took the wrong plot", i);
    }
}

#[test]
fn test_country_folk_and_the_martyr_keep_their_graves() {
    let mut g = Game::new();
    // an Outlander's body stays where it fell - the country's dead
    // keep the old custom
    let Some(&out) = g.villagers.iter().find(|&&v|
        g.world.get_component::<country::Outlander>(v).is_some()) else {
        // no country folk on this seed's roster - nothing to assert
        return;
    };
    let oi = g.villagers.iter().position(|&v| v == out).unwrap();
    kill_on(&mut g, oi, 0);
    let out_fell = pos_of(&g, out);
    g.graveyard_tick(10);
    assert!(g.world.get_component::<villager::Buried>(out).is_none(),
        "a country death was carried to the village churchyard");
    assert_eq!(pos_of(&g, out), out_fell);
    // the martyr's plot is the marker: it stays where the gibbet made it
    let mi = 4;
    kill_on(&mut g, mi, 0);
    let m = g.villagers[mi];
    let mp = pos_of(&g, m);
    g.martyr_grave = Some((mp.0.round() as i32, mp.1.round() as i32));
    g.graveyard_tick(10);
    assert!(g.world.get_component::<villager::Buried>(m).is_none(),
        "the martyr was carried off the gibbet's plot");
    assert_eq!(pos_of(&g, m), mp);
}

#[test]
fn test_the_model_marks_the_buried_and_their_epitaphs() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    g.graveyard_tick(3);
    let m = g.ui_model();
    let dead = m.people.iter().find(|p| p.kind == PKind::Villager(0)).unwrap();
    assert!(dead.dead && dead.buried, "the model doesn't know they're buried");
    let epi = dead.epitaph.as_deref().unwrap_or("");
    assert!(epi.contains("died day 0"), "epitaph reads {:?}", epi);
    assert!(!m.graveyard.is_empty(), "the model doesn't carry the churchyard");
    // a lying-in corpse isn't marked buried and has no epitaph day yet
    kill_on(&mut g, 1, 2);
    let m2 = g.ui_model();
    let lying = m2.people.iter().find(|p| p.kind == PKind::Villager(1)).unwrap();
    assert!(lying.dead && !lying.buried);
}

