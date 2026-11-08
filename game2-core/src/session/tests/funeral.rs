//! The funeral procession (#674): the mourners' column that carries a
//! body from where it fell to its churchyard plot

use crate::session::*;

fn pos_of(g: &Game, v: Entity) -> (f32, f32) {
    let p = g.world.get_component::<Pos>(v).unwrap();
    (p.x, p.y)
}

fn kill_on(g: &mut Game, i: usize, day: u64) {
    let v = g.villagers[i];
    g.world.add_component(v, villager::Dead);
    g.died_on[i] = Some(day);
}

fn drain_formation(g: &mut Game) {
    for _ in 0..40000 {
        if g.formation.is_none() { return; }
        g.formation_step();
    }
    panic!("the funeral column never arrived");
}

fn funeral_of(g: &Game, dead: Entity) -> villager::Funeral {
    g.world.get_component::<villager::Funeral>(dead)
        .map(|f| villager::Funeral { slot: f.slot, carriers: f.carriers.clone(),
            officiant: f.officiant, carried: f.carried, since: f.since })
        .expect("no funeral state on the dead")
}

fn priest_of(g: &Game) -> Option<Entity> {
    g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v)
            .map(|c| c.vocation == Vocation::Priest).unwrap_or(false))
}

#[test]
fn test_the_column_forms_with_shoulders_officiant_and_mourners() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    let dead = g.villagers[0];
    g.graveyard_tick(3);
    let f = g.formation.as_ref().expect("no procession formed");
    assert!(matches!(f.kind, formation::FormationKind::Funeral(d) if d == dead),
        "the column isn't carrying the dead");
    let fun = funeral_of(&g, dead);
    assert!(!fun.carriers.is_empty() && fun.carriers.len() <= 4,
        "the box needs its pallbearers, got {}", fun.carriers.len());
    for &c in &fun.carriers {
        assert!(!villager::is_dead(&g.world, c), "a pallbearer is dead");
        assert!(g.villagers.contains(&c), "a pallbearer isn't the village's");
    }
    // the mourners: a handful behind the box, none of them the dead
    let mourners = f.members.len() - fun.carriers.len()
        - if fun.officiant.is_some() { 1 } else { 0 };
    assert!((4..=7).contains(&mourners), "{} mourners fell in", mourners);
    assert!(!f.members.contains(&dead), "the dead march themselves");
    // the priest keeps the office when the parish has one and he's free
    // (a priest already shoulder-deep in the box can't also say the words)
    if let Some(p) = priest_of(&g) {
        if !fun.carriers.contains(&p) {
            assert_eq!(fun.officiant, Some(p), "the priest didn't lead the rite");
        }
        if let Some(o) = fun.officiant {
            assert_eq!(f.members.first(), Some(&o), "the officiant doesn't walk ahead");
        }
    }
}

#[test]
fn test_the_procession_still_goes_without_clergy() {
    let mut g = Game::new();
    // no priest on this village: the column must still form and finish
    for &v in &g.villagers {
        if g.world.get_component::<Craft>(v)
            .map(|c| c.vocation == Vocation::Priest).unwrap_or(false) {
            g.world.add_component(v, villager::Dead);
        }
    }
    kill_on(&mut g, 0, 0);
    let dead = g.villagers[0];
    g.graveyard_tick(3);
    assert!(g.formation.is_some(), "no clergy, no procession - wrong");
    drain_formation(&mut g);
    assert!(g.world.get_component::<villager::Buried>(dead).is_some(),
        "the body never reached its plot");
}

#[test]
fn test_the_friendless_dead_get_neighbour_pallbearers() {
    let mut g = Game::new();
    // a soul with no warm bonds: the neighbours nearest the body carry
    let i = (0..g.villagers.len()).find(|&i| {
        let v = g.villagers[i];
        relationships::bonds_of(&g.world, v).iter().all(|&r| {
            g.world.get_component::<relationships::Relationship>(r)
                .map(|rel| rel.strength <= 0.0).unwrap_or(true)
        })
    }).expect("every villager has a bond on this seed");
    kill_on(&mut g, i, 0);
    let dead = g.villagers[i];
    g.graveyard_tick(3);
    let fun = funeral_of(&g, dead);
    assert!(!fun.carriers.is_empty(), "nobody shouldered the friendless dead");
    for &c in &fun.carriers {
        assert!(!villager::is_dead(&g.world, c));
    }
}

#[test]
fn test_the_body_rides_with_the_column_to_the_plot() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    let fell = pos_of(&g, g.villagers[0]);
    g.graveyard_tick(3);
    let dead = g.villagers[0];
    let slot = funeral_of(&g, dead).slot;
    // the body stays where it fell until the column reaches it - then
    // it walks the road on the bearers' shoulders, never teleporting
    let mut seen_move = false;
    for _ in 0..4000 {
        if g.formation.is_none() { break; }
        g.formation_step();
        if let Some(f) = g.formation.as_ref() {
            if f.at == 0 {
                assert_eq!(pos_of(&g, dead), fell,
                    "the body left the death tile before the column arrived");
            }
        }
        let p = pos_of(&g, dead);
        if (p.0 - fell.0).abs() + (p.1 - fell.1).abs() > 0.5 { seen_move = true; }
        if g.world.get_component::<villager::Buried>(dead).is_some() { break; }
    }
    assert!(seen_move, "the body never left the death tile");
    assert!(g.world.get_component::<villager::Buried>(dead).is_some(),
        "the column never laid the body in");
    let rest = pos_of(&g, dead);
    assert_eq!((rest.0.round() as i32, rest.1.round() as i32), slot,
        "the grave isn't the slot the column marched to");
}

#[test]
fn test_the_mourners_leave_a_little_more_devoted() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    g.graveyard_tick(3);
    let members: Vec<Entity> = g.formation.as_ref().unwrap().members.clone();
    let devotion = |g: &Game, e: Entity| g.world.get_component::<crate::sim::religion::Faith>(e)
        .map(|f| f.devotion).unwrap_or(0.0);
    let before: Vec<f32> = members.iter().map(|&m| devotion(&g, m)).collect();
    drain_formation(&mut g);
    for (k, &m) in members.iter().enumerate() {
        assert!(devotion(&g, m) >= before[k], "{} lost devotion at a funeral", k);
    }
    assert!(members.iter().enumerate().any(|(k, &m)| devotion(&g, m) > before[k]),
        "no mourner's faith was touched");
    assert!(g.log.iter().any(|l| l.contains("funeral procession")),
        "the village didn't mark the funeral");
}

#[test]
fn test_two_funerals_same_day_march_one_at_a_time() {
    let mut g = Game::new();
    kill_on(&mut g, 0, 0);
    kill_on(&mut g, 1, 0);
    g.graveyard_tick(3);
    // the first due burial forms the column; the second waits its turn
    let first = g.formation.as_ref().and_then(|f| match f.kind {
        formation::FormationKind::Funeral(d) => Some(d), _ => None });
    assert!(first.is_some(), "no procession for the first death");
    drain_formation(&mut g);
    // the next day's check sends the second body on its way
    g.graveyard_tick(4);
    let second = g.formation.as_ref().and_then(|f| match f.kind {
        formation::FormationKind::Funeral(d) => Some(d), _ => None });
    assert!(second.is_some(), "the second funeral never formed");
    assert_ne!(second, first);
    drain_formation(&mut g);
    assert!(g.world.get_component::<villager::Buried>(g.villagers[0]).is_some());
    assert!(g.world.get_component::<villager::Buried>(g.villagers[1]).is_some());
}

#[test]
fn test_the_procession_is_deterministic() {
    let run = || {
        let mut g = Game::new();
        kill_on(&mut g, 0, 0);
        g.graveyard_tick(3);
        let members = g.formation.as_ref().map(|f| f.members.clone());
        let dead = g.formation.as_ref().and_then(|f| match f.kind {
            formation::FormationKind::Funeral(d) => Some(d), _ => None });
        (members, dead)
    };
    let (m1, k1) = run();
    let (m2, k2) = run();
    assert_eq!(m1, m2, "the same seed picked different mourners");
    assert_eq!(k1, k2);
}

