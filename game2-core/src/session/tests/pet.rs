use crate::session::*;

fn tame_a_sheep(g: &mut Game) -> Entity {
    let sheep = g.flock[0];
    let pp = g.world.get_component::<Pos>(g.seats[0].player).copied().unwrap();
    *g.world.get_component_mut::<Pos>(sheep).unwrap() = Pos { x: pp.x + 1.0, y: pp.y };
    for _ in 0..8 {
        if g.world.get_component::<Pet>(sheep).map(|p| p.owner.is_some()).unwrap_or(false) { break; }
        g.seats[0].ctx_actions = vec![CtxAct::Tame];
        g.ctx_act_at(0, 0);
        g.frame_n += events::STRAY_CAT_PET_COOLDOWN as u32 + 1;
    }
    sheep
}

#[test]
fn test_the_kind_hand_wins_a_sheep() {
    let mut g = Game::new();
    let sheep = tame_a_sheep(&mut g);
    let pe = g.seats[0].player;
    let pet = g.world.get_component::<Pet>(sheep).expect("no Pet on her");
    assert_eq!(pet.owner, Some(pe), "she never took an owner");
    assert!(!g.flock.contains(&sheep), "she never left the fold");
    assert!(g.pets.iter().any(|(e, _, _)| *e == sheep), "she's not in the pets");
    assert!(g.log.iter().any(|l| l.contains("leaves the fold")), "her leaving went unsaid");
    // and the model draws her where she stands, not at your heel
    let pp = g.world.get_component::<Pos>(g.seats[0].player).copied().unwrap();
    *g.world.get_component_mut::<Pos>(sheep).unwrap() = Pos { x: pp.x + 8.0, y: pp.y + 8.0 };
    let m = g.ui_model();
    let her = m.people.iter().find(|p| matches!(p.kind, PKind::Pet(_)) && p.e == sheep)
        .expect("the tamed sheep isn't in the model");
    assert!((her.pos.0 - (pp.x + 8.0)).abs() < 0.01 && (her.pos.1 - (pp.y + 8.0)).abs() < 0.01,
        "she draws at {:?}, not her own feet", her.pos);
}

#[test]
fn test_a_pet_trails_the_one_who_won_it() {
    let mut g = Game::new();
    let sheep = tame_a_sheep(&mut g);
    let pe = g.seats[0].player;
    // walk off across the meadow - and set her on open ground beside
    // your path, since no ewe climbs a fold wall to follow
    let pp = g.world.get_component::<Pos>(pe).copied().unwrap();
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: pp.x + 12.0, y: pp.y };
    *g.world.get_component_mut::<Pos>(sheep).unwrap() = Pos { x: pp.x + 6.0, y: pp.y + 4.0 };
    let before = { let a = g.world.get_component::<Pos>(sheep).copied().unwrap();
        ((a.x - pp.x - 12.0).powi(2) + (a.y - pp.y).powi(2)).sqrt() };
    for _ in 0..600 { g.update(); }
    let (a, p) = (g.world.get_component::<Pos>(sheep).copied().unwrap(),
                  g.world.get_component::<Pos>(pe).copied().unwrap());
    let after = ((a.x - p.x).powi(2) + (a.y - p.y).powi(2)).sqrt();
    assert!(after < before, "she never came after you: {} -> {}", before, after);
    assert!(after <= 3.0, "she's still strung out: {}", after);
}

#[test]
fn test_a_friend_at_heel_warms_the_talk() {
    let mut g = Game::new();
    let pe = g.seats[0].player;
    let (a, b) = (g.villagers[0], g.villagers[1]);
    let cat = pets::spawn_pet(&mut g.world, "moggie", Species::Cat, pets::PetRole::Companion);
    g.world.get_component_mut::<Pet>(cat).unwrap().bond_strength = 0.8;
    crate::game::player::act_talk(&mut g.world, pe, a, 0);
    pets::adopt(&mut g.world, cat, pe);
    crate::game::player::act_talk(&mut g.world, pe, b, 0);
    let rap = |t: Entity| g.world.get_component::<crate::game::player::Rapport>(t)
        .map(|r| r.by_player.get(&pe).copied().unwrap_or(0.0)).unwrap_or(0.0);
    assert!(rap(b) > rap(a), "the pet made no difference: {} vs {}", rap(b), rap(a));
}

#[test]
fn test_the_kind_hand_names_the_beast_it_reaches_for() {
    // "Stroke it" read oddly - the button says whose hand it's for (#1056)
    let mut g = Game::new();
    let pp = g.world.get_component::<Pos>(g.seats[0].player).copied().unwrap();
    let sheep = g.flock[0];
    *g.world.get_component_mut::<Pos>(sheep).unwrap() = Pos { x: pp.x + 1.0, y: pp.y };
    g.update();
    let m = g.ui_model();
    let i = m.ctx.iter().position(|&a| a == CtxAct::Tame).expect("the kind hand on offer");
    assert_eq!(m.ctx_labels[i], "Pet the sheep", "the button names her");
    assert_eq!(g.ctx_label_at(0, CtxAct::Tame), "Pet the sheep",
        "the Can't-line wording matches the button");
    // a yard hen reads her own name
    let hen = g.hens[0];
    *g.world.get_component_mut::<Pos>(sheep).unwrap() = Pos { x: pp.x + 30.0, y: pp.y + 30.0 };
    *g.world.get_component_mut::<Pos>(hen).unwrap() = Pos { x: pp.x + 1.0, y: pp.y };
    g.update();
    let m = g.ui_model();
    let i = m.ctx.iter().position(|&a| a == CtxAct::Tame).expect("the kind hand on offer");
    assert_eq!(m.ctx_labels[i], "Pet the hen");
}
