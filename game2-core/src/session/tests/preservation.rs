use crate::session::*;

/// The bee is the slaughter day's own work: it answers only to the
/// Martinmas anchor in the FEASTS wheel, not the day beside it
#[test]
fn test_the_bee_fires_only_on_martinmas() {
    let mut g = Game::new();
    let be = g.building_entities[5]; // the trader's stall, well-stocked
    if let Some(b) = g.world.get_component_mut::<Building>(be) {
        household::add_stock(b, "fish", 4);
        household::add_stock(b, "salt", 4);
    }
    g.festival_tick(313);
    let fish_after_not_the_day = g.world.get_component::<Building>(be).unwrap()
        .stock.get("fish").copied().unwrap_or(0);
    assert_eq!(fish_after_not_the_day, 4, "no bee before the day");
    g.festival_tick(314);
    let b = g.world.get_component::<Building>(be).unwrap();
    assert_eq!(b.stock.get("fish").copied().unwrap_or(0), 0, "the day's own work");
    assert!(b.stock.get("smoked-fish").copied().unwrap_or(0) > 0, "into the smoke");
    assert!(g.log.iter().any(|l| l.contains("the Martinmas salting")),
        "the feast names itself: {:?}", g.log.last());
}

#[test]
fn test_the_salting_converts_in_place() {
    let mut g = Game::new();
    let be = g.building_entities[5];
    if let Some(b) = g.world.get_component_mut::<Building>(be) {
        household::add_stock(b, "fish", 6);
        household::add_stock(b, "meat", 4);
        household::add_stock(b, "salt", 10);
    }
    g.martinmas_bee(314);
    let b = g.world.get_component::<Building>(be).unwrap();
    assert_eq!(b.stock.get("fish").copied().unwrap_or(0), 0, "all the fish smoked");
    assert_eq!(b.stock.get("meat").copied().unwrap_or(0), 0, "all the meat brined");
    assert_eq!(b.stock.get("smoked-fish").copied().unwrap_or(0), 6);
    assert_eq!(b.stock.get("salt-pork").copied().unwrap_or(0), 4);
    // a salt per two perishables: ten units wanted five salt
    assert_eq!(b.stock.get("salt").copied().unwrap_or(0), 5, "five salt did ten units");
}

#[test]
fn test_a_salt_short_house_converts_what_its_salt_reaches() {
    let mut g = Game::new();
    let be = g.building_entities[5];
    if let Some(b) = g.world.get_component_mut::<Building>(be) {
        household::add_stock(b, "fish", 6);
        household::add_stock(b, "salt", 1); // salt for two, fish for six
    }
    g.martinmas_bee(314);
    let b = g.world.get_component::<Building>(be).unwrap();
    assert_eq!(b.stock.get("smoked-fish").copied().unwrap_or(0), 2, "a salt per two");
    assert_eq!(b.stock.get("fish").copied().unwrap_or(0), 4, "the rest waits to rot");
    assert_eq!(b.stock.get("salt").copied().unwrap_or(0), 0, "the salt's all spent");
}

#[test]
fn test_preserved_keeps_where_perishable_rots() {
    let mut g = Game::new();
    let be = g.building_entities[5];
    if let Some(b) = g.world.get_component_mut::<Building>(be) {
        household::add_stock(b, "fish", 3);
        household::add_stock(b, "smoked-fish", 3);
        household::add_stock(b, "meat", 3);
        household::add_stock(b, "salt-pork", 3);
    }
    g.spoilage_tick(3); // the weekly rot, flesh's own turn
    let b = g.world.get_component::<Building>(be).unwrap();
    assert_eq!(b.stock.get("fish").copied().unwrap_or(0), 2, "raw fish turns in a week");
    assert_eq!(b.stock.get("meat").copied().unwrap_or(0), 2, "raw meat turns in a week");
    assert_eq!(b.stock.get("smoked-fish").copied().unwrap_or(0), 3, "the smoke keeps it");
    assert_eq!(b.stock.get("salt-pork").copied().unwrap_or(0), 3, "the brine keeps it");
}

#[test]
fn test_the_flock_gives_its_third() {
    let mut g = Game::new();
    // a fold with a flock, wherever the seed put it
    let fold = g.fold.unwrap_or_else(|| {
        let f = household::spawn_building(&mut g.world,
            household::BuildingKind::Pasture, "the fold", 20.0, 20.0, 200);
        g.building_entities.push(f);
        g.fold = Some(f);
        f
    });
    for i in 0..9 {
        let a = crate::game::livestock::spawn_animal(&mut g.world,
            crate::game::livestock::LivestockKind::Sheep, fold, 21.0 + i as f32, 21.0);
        g.flock.push(a);
    }
    let cullable = |g: &Game| crate::game::livestock::flock(&g.world, fold).iter()
        .filter(|&&a| g.world.get_component::<crate::game::livestock::Livestock>(a)
            .map(|l| l.kind != crate::game::livestock::LivestockKind::Hen).unwrap_or(false))
        .count();
    let flock = cullable(&g);
    g.martinmas_bee(314);
    let left = cullable(&g);
    assert_eq!(left, flock - flock / 3, "the third goes to the block");
    let meat = g.world.get_component::<Building>(fold)
        .map(|b| b.stock.get("meat").copied().unwrap_or(0)).unwrap_or(0);
    assert!(meat > 0, "the cull fills the fold's stock");
    assert!(g.log.iter().any(|l| l.contains("the flock gives its third")),
        "the cull's told: {:?}", g.log.last());
}

/// Ahead of the bee the salt has to come home: a house holding
/// perishables with a salt-short larder sends a hand out for it while
/// a seller's salt's still to be had
#[test]
fn test_the_larder_lays_in_its_salt() {
    let mut g = Game::new();
    // a house with fish waiting on the bee and no salt of its own
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b).map(|bb|
            bb.kind == BuildingKind::Home && !bb.occupants.is_empty()).unwrap_or(false)).unwrap();
    if let Some(b) = g.world.get_component_mut::<Building>(home) {
        household::add_stock(b, "fish", 6);
    }
    // and salt on the road: a stall stocking it
    let stall = household::spawn_building(&mut g.world,
        BuildingKind::TraderStall, "the salt stall", 10.0, 10.0, 200);
    if let Some(b) = g.world.get_component_mut::<Building>(stall) {
        household::add_stock(b, "salt", 8);
    }
    g.salt_layin_tick(263); // the Harvest Home's turn: the season opens
    let hand = g.world.get_component::<Building>(home).unwrap()
        .occupants.iter().copied().find(|&o|
            g.travels.iter().any(|t| t.who == o));
    assert!(hand.is_some(), "a hand went out for the salt");
    assert!(g.travels.iter().any(|t| matches!(t.goal, Goal::BuyFrom(_, "salt", _))),
        "the errand's for salt: {:?}", g.travels.last().map(|t| format!("{:?}", t.goal)));
}
