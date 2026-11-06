use crate::session::*;

#[test]
fn test_the_herb_wife_is_one_and_keeps_her_own_hut() {
    let game = Game::new();
    let hers = game.villagers.iter()
        .filter(|&&v| game.world.get_component::<TownRole>(v) == Some(&TownRole::Herbologist))
        .count();
    assert_eq!(hers, 1, "there is exactly one herb-wife");
    assert_eq!(game.herbologist, Some(game.villagers[HERB_IDX]));
    let hut = game.herb_hut.expect("the hut should stand");
    let b = game.world.get_component::<Building>(hut).unwrap();
    assert_eq!(b.name, "the herb hut");
    assert_eq!(b.kind, BuildingKind::Home);
    assert!(b.occupants.contains(&game.villagers[HERB_IDX]), "she lives in it");
}

#[test]
fn test_herb_patches_grow_wild_outside_the_wall() {
    let game = Game::new();
    assert!(game.herb_patches.len() >= 8, "a dozen or so tufts");
    let (x0, y0, x1, y1) = game.town_wall;
    for p in &game.herb_patches {
        assert!(game.map.passable(p.tile.0, p.tile.1), "patch on a walkable tile");
        let inside = (x0..=x1).contains(&p.tile.0) && (y0..=y1).contains(&p.tile.1);
        assert!(!inside, "herbs grow wild, not in the streets");
    }
}

#[test]
fn test_she_brews_what_the_hut_stocks() {
    let mut game = Game::new();
    let hut = game.herb_hut.unwrap();
    game.world.get_component_mut::<Building>(hut).unwrap().stock.insert("willow_bark".into(), 2);
    game.herbologist_tick(5);
    let her = game.herbologist.unwrap();
    assert!(herbology::remedies_on_hand(&game.world, her) >= 1, "a remedy on the shelf");
    assert_eq!(game.world.get_component::<Building>(hut).unwrap().stock["willow_bark"], 1, "one herb went in");
}

#[test]
fn test_a_forage_trip_tops_up_the_hut() {
    let mut game = Game::new();
    let her = game.herbologist.unwrap();
    let pi = (0..game.herb_patches.len())
        .find(|&i| game.send_forage(her, i)).expect("a patch within her reach");
    let herb = game.herb_patches[pi].herb;
    for _ in 0..(16 * FPS) { game.update(); }
    let hut = game.herb_hut.unwrap();
    // two bundles picked; she may already have brewed some of them
    let got = game.world.get_component::<Building>(hut).unwrap().stock.get(herb).copied().unwrap_or(0) as usize
        + herbology::remedies_on_hand(&game.world, game.herbologist.unwrap());
    assert!(got >= 2, "the pickings get dried at the hut ({} stocked + brewed)", got);
    assert!(game.herb_patches[pi].next_ready > game.tick, "the patch needs a season to come back");
}

#[test]
fn test_the_sick_walk_out_and_a_cure_turns_the_disease() {
    let mut game = Game::new();
    let v = game.villagers[5];
    let d = game.world.create_entity();
    game.world.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: v, stage: Stage::Moderate, severity: 0.45, contagious: false, incubation: 0 });
    // stock the shelf so a cure's in hand
    let hut = game.herb_hut.unwrap();
    game.world.get_component_mut::<Building>(hut).unwrap().stock.insert("yarrow".into(), 2);
    let before_gold = game.world.get_component::<Purse>(v).unwrap().gold;
    game.arrive(v, Goal::SeekRemedy(hut));
    let sev = game.world.query::<Disease>().iter()
        .filter_map(|d| game.world.get_component::<Disease>(*d))
        .find(|d| d.patient == v).map(|d| (d.severity, d.stage));
    let (sev, stage) = sev.unwrap_or((0.0, Stage::Recovering));
    assert!(sev < 0.45 || stage == Stage::Recovering, "the remedy bit into it");
    assert!(game.world.get_component::<Purse>(v).unwrap().gold < before_gold, "she doesn't work for free");
    assert!(game.remedy_visits.iter().any(|(e, _)| *e == v), "the call is remembered");
}

#[test]
fn test_nothing_in_the_hedge_says_so() {
    let mut game = Game::new();
    let v = game.villagers[5];
    let d = game.world.create_entity();
    game.world.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: v, stage: Stage::Mild, severity: 0.2, contagious: false, incubation: 0 });
    let hut = game.herb_hut.unwrap();
    game.arrive(v, Goal::SeekRemedy(hut));
    assert!(game.bubbles.iter().any(|(e, t, _, _, _)| *e == game.herbologist.unwrap() && t.contains("Nothing")),
        "she says when she can't help");
}

#[test]
fn test_the_huts_card_keeps_her_tally() {
    let mut g = Game::new();
    g.tick = 20;
    g.cured = 4;
    let who = g.villagers[3];
    g.remedy_visits.push((who, 18));
    let hut_i = g.building_entities.iter().position(|&b| b == g.herb_hut.unwrap()).unwrap();
    g.seats[0].selected_building = Some(hut_i);
    g.seats[0].picked = false;
    let ins = g.ui_model_at(0).inspector.expect("no card for the hut");
    assert!(ins.lines.iter().any(|l| l.text.contains("her tally: 4 cures")
        && l.text.contains("lately:")),
        "the card doesn't keep her tally: {:?}",
        ins.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}
