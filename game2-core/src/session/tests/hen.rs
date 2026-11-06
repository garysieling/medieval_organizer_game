use crate::session::*;

#[test]
fn test_hens_scratch_about_a_few_yards() {
    let g = Game::new();
    assert!(g.hen_houses.len() >= 3, "a few yards keep hens");
    assert!(g.hens.len() >= 8, "a few hens a yard");
    for &a in &g.hens {
        assert_eq!(g.world.get_component::<Livestock>(a).unwrap().kind, LivestockKind::Hen);
        let home = g.world.get_component::<Livestock>(a).unwrap().fold;
        assert!(g.hen_houses.contains(&home), "each hen belongs to a yard");
    }
}

#[test]
fn test_eggs_lay_and_a_hungry_house_eats_them() {
    let mut g = Game::new();
    let h = g.hen_houses[0];
    g.livestock_tick(1);
    let eggs = g.world.get_component::<Building>(h).unwrap().stock.get("eggs").copied().unwrap_or(0);
    assert!(eggs > 0, "a hen lays in the morning");
    // a hungry occupant takes one off the larder
    let occ = g.world.get_component::<Building>(h).unwrap().occupants[0];
    g.world.get_component_mut::<Hunger>(occ).unwrap().value = 20.0;
    g.livestock_tick(2);
    assert!(g.world.get_component::<Hunger>(occ).unwrap().value > 20.0, "eggs for breakfast");
}

#[test]
fn test_eggs_do_not_pile_past_the_larder() {
    let mut g = Game::new();
    let h = g.hen_houses[0];
    for t in 0..10 { g.apiary_tick(t); }
    assert!(g.world.get_component::<Building>(h).unwrap().stock.get("eggs").copied().unwrap_or(0) <= 6);
}

#[test]
fn test_the_fox_takes_a_hen_and_the_house_grieves() {
    let mut g = Game::new();
    let before = g.hens.len();
    g.fox_night(75);
    assert!(before - g.hens.len() <= 1, "a fox takes one, not the yard");
    assert!(g.log.iter().any(|l| l.contains("a fox")), "the yard says so");
}

#[test]
fn test_the_swarm_gives_them_their_little_fame() {
    let mut g = Game::new();
    // run the %90==70 days until a swarm takes its liking
    let mut who = None;
    for t in (70..4500).step_by(90) {
        g.apiary_tick(t);
        if let Some((w, _)) = g.swarm { who = Some(w); break; }
    }
    let who = who.expect("no swarm ever took a liking");
    assert!(g.log.iter().any(|l| l.contains("took a liking")), "the word never went out");
    assert!(desires::is_satisfied(&g.world, who, desires::Want::Joy, g.tick),
        "the memorable part never landed on them");
}
