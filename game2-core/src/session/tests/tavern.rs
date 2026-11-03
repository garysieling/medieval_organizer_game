use crate::session::*;
use crate::game::relationships::{find_relationship, Relationship};
use crate::game::villager::Drunk;

/// Whoever keeps the bench
fn keeper(g: &Game) -> Entity {
    g.world.get_component::<Building>(g.tavern.unwrap()).unwrap().worker
        .expect("the tavern is not left dark at spawn")
}

/// The shop of whichever drink trade this village rolled
fn maker(g: &Game) -> Entity {
    g.building_entities.iter().copied().find(|&be|
        g.world.get_component::<Building>(be)
            .map(|b| matches!(b.kind, BuildingKind::Shop(
                Vocation::Alewife | Vocation::Brewer | Vocation::Vintner | Vocation::Distiller))
                && b.worker.is_some())
            .unwrap_or(false)).expect("every village keeps a drink trade")
}

/// Clear the green: move everyone but `keep` well out of the table's
/// reach, so a test's bench holds only who it sat there
fn clear_green(g: &mut Game, keep: &[Entity]) {
    let at = *g.world.get_component::<Pos>(g.tavern.unwrap()).unwrap();
    for &v in &g.villagers.clone() {
        if keep.contains(&v) { continue; }
        let p = g.world.get_component_mut::<Pos>(v).unwrap();
        p.x = at.x + 40.0; p.y = at.y + 40.0;
    }
}

/// Sit `who` down at the tavern's table
fn seat(g: &mut Game, who: Entity) {
    let at = *g.world.get_component::<Pos>(g.tavern.unwrap()).unwrap();
    let p = g.world.get_component_mut::<Pos>(who).unwrap();
    p.x = at.x; p.y = at.y;
}

#[test]
fn test_every_village_keeps_a_tavern_and_a_keeper() {
    let g = Game::new();
    let tavern = g.tavern.expect("the green has its public house");
    let b = g.world.get_component::<Building>(tavern).unwrap();
    assert_eq!(b.kind, BuildingKind::Shop(Vocation::Taverner));
    assert_eq!(b.name, "the tavern");
    assert!(b.worker.is_some(), "and someone behind the bench");
    assert!(b.stock.get("ale").copied().unwrap_or(0) > 0, "a barrel on opening day");
    assert!(g.log.iter().any(|l| l.contains("the tavern on the green")));
    // the keeper's trade is the bench, not the vat
    let k = b.worker.unwrap();
    assert_eq!(vocation::inputs(Vocation::Taverner), &[] as &[&str],
        "he brews nothing - he buys it in");
    assert_eq!(g.world.get_component::<Craft>(k).unwrap().vocation, Vocation::Taverner);
}

#[test]
fn test_the_keeper_buys_the_drink_trade_out() {
    let mut g = Game::new();
    let (tavern, k, m) = (g.tavern.unwrap(), keeper(&g), maker(&g));
    let made = g.world.get_component::<Building>(m)
        .map(|b| match b.kind { BuildingKind::Shop(v) => vocation::output(v).unwrap(), _ => "ale" })
        .unwrap();
    let worker = g.world.get_component::<Building>(m).unwrap().worker.unwrap();
    // clear the green first: this test is about the lay-in, and a
    // maker who wanders over for a cup of his own beer on the same
    // tick muddies his own till
    clear_green(&mut g, &[]);
    // a dry bar and a full vat
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.clear();
    g.world.get_component_mut::<Building>(m).unwrap().stock.insert(made.into(), 10);
    g.world.get_component_mut::<Purse>(k).unwrap().gold = 100.0;
    let paid_before = g.world.get_component::<Purse>(worker).unwrap().gold;
    // the lay-in leans the good dear when it bares the shelf, so the
    // deal is priced off what it was worth going in
    let worth_before = goods::worth_at(&g.world, made);
    g.tavern_tick(2); // an even day - restock day
    let left = g.world.get_component::<Building>(m).unwrap()
        .stock.get(made).copied().unwrap_or(0);
    let bought = 10 - left;
    assert!(bought > 0, "the barrel came off the maker's shelf");
    // what came in is on the bar, less whatever the bench drank on
    // the way past - the lay-in and the pour share a tick
    let bar = g.world.get_component::<Building>(tavern).unwrap()
        .stock.get(made).copied().unwrap_or(0);
    assert!(bar <= bought, "the bar holds what came in, less what was drunk");
    // and the coin went the other way, at the wholesale cut. The
    // keeper's own purse also takes the bench's money on this tick,
    // so what the maker banked is the honest measure of the deal
    let earned = g.world.get_component::<Purse>(worker).unwrap().gold - paid_before;
    let street = worth_before * bought as f32;
    assert!((earned - street * TAVERN_WHOLESALE).abs() < 0.01,
        "the maker banked the wholesale price of what left his shelf ({} of {} for {})",
        earned, made, bought);
    assert!(earned < street, "he buys under the street price - that margin is his living");
    assert!(g.log.iter().any(|l| l.contains("lays in") && l.contains("for the tavern")),
        "{:?}", g.log.last());
}

#[test]
fn test_a_dry_keeper_with_an_empty_purse_lays_in_nothing() {
    let mut g = Game::new();
    let (tavern, k, m) = (g.tavern.unwrap(), keeper(&g), maker(&g));
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.clear();
    g.world.get_component_mut::<Building>(m).unwrap().stock.insert("ale".into(), 10);
    g.world.get_component_mut::<Purse>(k).unwrap().gold = 0.0;
    g.tavern_tick(2);
    assert!(g.world.get_component::<Building>(tavern).unwrap()
        .stock.values().all(|&q| q == 0), "no coin, no barrel");
}

#[test]
fn test_the_common_table_makes_strangers_acquaintances() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    // two who've never met, both at the bench
    let pair: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| v != k && g.world.get_component::<Kid>(v).is_none())
        .take(2).collect();
    clear_green(&mut g, &pair);
    for &v in &pair { seat(&mut g, v); }
    assert!(find_relationship(&g.world, pair[0], pair[1]).is_none()
        || g.world.get_component::<Relationship>(
            find_relationship(&g.world, pair[0], pair[1]).unwrap()).unwrap().interactions == 0,
        "strangers to start");
    g.tavern_tick(3);
    let r = find_relationship(&g.world, pair[0], pair[1])
        .expect("a shared bench is a shared acquaintance");
    assert!(g.world.get_component::<Relationship>(r).unwrap().strength > 0.0,
        "and the bond warms");
    // the keeper hears everyone's business - that's the trade
    for &v in &pair {
        assert!(find_relationship(&g.world, k, v).is_some(), "the keeper knows them all");
    }
    // and the table registers who sat at it
    assert_eq!(g.tavern_table.iter().filter(|&&(_, d)| d == 3).count(), 2);
}

#[test]
fn test_a_dry_house_draws_nobody() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.clear();
    let k = keeper(&g);
    let pair: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| v != k && g.world.get_component::<Kid>(v).is_none()).take(2).collect();
    for &v in &pair { seat(&mut g, v); }
    g.tavern_tick(3); // odd day: no restock, so the bar stays dry
    assert!(g.tavern_table.is_empty(), "an empty barrel keeps its own company");
}

#[test]
fn test_a_cup_at_the_bench_is_bought_and_drunk() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    let v = g.villagers.iter().copied()
        .find(|&v| v != k && g.world.get_component::<Kid>(v).is_none()).unwrap();
    let other = g.villagers.iter().copied()
        .find(|&o| o != k && o != v && g.world.get_component::<Kid>(o).is_none()).unwrap();
    clear_green(&mut g, &[v, other, k]);
    for w in [v, other] { seat(&mut g, w); }
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    g.world.get_component_mut::<InnerLife>(v).unwrap().satisfied.clear();
    let till = g.world.get_component::<Purse>(k).unwrap().gold;
    g.tavern_tick(3); // odd tick: the table, no lay-in
    assert!(desires::is_satisfied(&g.world, v, Want::Drink, 3), "the cup went down");
    assert!(g.world.get_component::<Purse>(k).unwrap().gold > till,
        "and the coin went into the keeper's till - that's his living");
    assert!(g.world.get_component::<Building>(tavern).unwrap()
        .stock.get("ale").copied().unwrap_or(0) < 6, "off the bar");
    // drunk at the bench, not carried home
    assert_eq!(g.world.get_component::<Satchel>(v).unwrap()
        .goods.get("ale").copied().unwrap_or(0), 0);
}

#[test]
fn test_a_thirsty_soul_with_no_coin_drinks_nothing() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    let pair: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| v != k && g.world.get_component::<Kid>(v).is_none()).take(2).collect();
    clear_green(&mut g, &[pair[0], pair[1], k]);
    for &v in &pair {
        seat(&mut g, v);
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 0.0;
        g.world.get_component_mut::<InnerLife>(v).unwrap().satisfied.clear();
    }
    g.tavern_tick(3); // odd tick: the table, no lay-in
    assert_eq!(g.world.get_component::<Building>(tavern).unwrap()
        .stock.get("ale").copied().unwrap_or(0), 6, "nothing leaves the bar on credit");
    for &v in &pair {
        assert!(!desires::is_satisfied(&g.world, v, Want::Drink, 3), "and no one drinks free");
    }
}

/// The brawl is the AlehouseBrawl event's job, not a second mechanism's -
/// but it has to find the tavern, which is the village's taproom now
#[test]
fn test_the_brawl_finds_the_tavern() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    // the tavern has the barrel; strip every other drink shop bare so
    // only the tavern can be the taproom
    for &be in &g.building_entities.clone() {
        if be == tavern { continue; }
        if let Some(b) = g.world.get_component_mut::<Building>(be) {
            for d in ["ale", "beer", "wine", "spirits"] { b.stock.remove(d); }
        }
    }
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    let at = *g.world.get_component::<Pos>(tavern).unwrap();
    // two drunks on the tavern's doorstep
    let pair: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<TownRole>(v).is_none()).take(2).collect();
    for &v in &pair {
        let p = g.world.get_component_mut::<Pos>(v).unwrap();
        p.x = at.x; p.y = at.y;
        g.world.add_component(v, Drunk { until: g.tick + 10 });
    }
    g.force_event(events::EventId::AlehouseBrawl);
    assert!(g.log.iter().any(|l| l.contains("brawl")), "{:?}", g.log.last());
}

#[test]
fn test_a_bed_at_the_tavern_answers_the_want_of_a_roof() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    // the keeper's own day's work is the bed, not the barrel
    assert_eq!(vocation::output(Vocation::Taverner), Some("lodging"));
    assert!(vocation::price("lodging") > 0.0, "and a bed is paid for");
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("lodging".into(), 2);
    // someone with the want, the coin, and nowhere to sleep
    let v = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<Building>(tavern).unwrap().worker != Some(v)).unwrap();
    // turn them out of doors: a bed at the tavern is for whoever
    // hasn't one of their own
    if let Some(h) = household::home_of(&g.world, v) {
        g.world.get_component_mut::<Building>(h).unwrap().occupants.retain(|&o| o != v);
    }
    assert!(household::home_of(&g.world, v).is_none(), "nowhere to lay their head");
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    let life = g.world.get_component_mut::<InnerLife>(v).unwrap();
    life.wants.clear();
    life.wants.push(Want::Shelter);
    life.satisfied.clear();
    let goal = g.want_shopping(v, 5).expect("a roofless soul walks to the tavern");
    match goal {
        Goal::BuyAt(shop, good, Some(Want::Shelter)) => {
            assert_eq!(shop, tavern); assert_eq!(good, "lodging");
        }
        other => panic!("expected a bed at the tavern, got {:?}", other),
    }
    g.arrive(v, goal);
    assert!(desires::is_satisfied(&g.world, v, Want::Shelter, 5), "a night out of the weather");
    // and the bed isn't carried off in a satchel
    assert_eq!(g.world.get_component::<Satchel>(v).unwrap()
        .goods.get("lodging").copied().unwrap_or(0), 0);
}

#[test]
fn test_a_dark_house_does_nothing() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.clear();
    g.world.get_component_mut::<Building>(tavern).unwrap().worker = None;
    let m = maker(&g);
    g.world.get_component_mut::<Building>(m).unwrap().stock.insert("ale".into(), 10);
    g.world.get_component_mut::<Purse>(k).unwrap().gold = 100.0;
    g.tavern_tick(2);
    assert!(g.world.get_component::<Building>(tavern).unwrap()
        .stock.values().all(|&q| q == 0), "no keeper, no barrel, no bench");
    assert!(g.tavern_table.is_empty());
}

/// A lone soul at the bench is still a customer - the pairwise
/// company is a separate thing from the sale
#[test]
fn test_a_lone_patron_still_buys_a_cup() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    let v = g.villagers.iter().copied()
        .find(|&v| v != k && g.world.get_component::<Kid>(v).is_none()).unwrap();
    clear_green(&mut g, &[v, k]);
    seat(&mut g, v);
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    g.world.get_component_mut::<InnerLife>(v).unwrap().satisfied.clear();
    let till = g.world.get_component::<Purse>(k).unwrap().gold;
    g.tavern_tick(3); // odd tick: the table, no lay-in
    assert!(desires::is_satisfied(&g.world, v, Want::Drink, 3),
        "one patron is still a patron");
    assert!(g.world.get_component::<Purse>(k).unwrap().gold > till,
        "and the keeper still takes his coin");
}

/// Patrons who bought nothing - dry shelf or empty purse - aren't
/// company. The bench's social effects follow the coin, not the seat
#[test]
fn test_a_dry_patron_gains_no_company() {
    let mut g = Game::new();
    let tavern = g.tavern.unwrap();
    let k = keeper(&g);
    g.world.get_component_mut::<Building>(tavern).unwrap().stock.insert("ale".into(), 6);
    let pair: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| v != k && g.world.get_component::<Kid>(v).is_none()).take(2).collect();
    clear_green(&mut g, &[pair[0], pair[1], k]);
    for &v in &pair {
        seat(&mut g, v);
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 0.0;
        g.world.get_component_mut::<InnerLife>(v).unwrap().satisfied.clear();
    }
    g.tavern_tick(3); // odd tick: the table, no lay-in
    for &v in &pair {
        assert!(!desires::is_satisfied(&g.world, v, Want::Joy, 3),
            "no cup, no company");
    }
    assert!(find_relationship(&g.world, pair[0], pair[1]).is_none(),
        "two empty purses at the same table are still strangers");
    assert!(find_relationship(&g.world, k, pair[0]).is_none(),
        "and the keeper hears nothing from a dry patron");
}

/// A migrant turned away for want of a spare bed should actually ask
/// for one - the grievance alone never sent anyone to the tavern
#[test]
fn test_a_roofless_migrant_wants_a_bed() {
    let mut g = Game::new();
    // fill every home past capacity so the newcomer finds no room
    let filler = g.villagers[0];
    for &be in &g.building_entities.clone() {
        if let Some(b) = g.world.get_component_mut::<Building>(be) {
            if b.kind == BuildingKind::Home {
                while b.occupants.len() < 4 { b.occupants.push(filler); }
            }
        }
    }
    let before = g.villagers.len();
    g.maybe_migrant(5);
    let newcomer = *g.villagers.last().unwrap();
    assert_eq!(g.villagers.len(), before + 1);
    assert!(household::home_of(&g.world, newcomer).is_none(), "no room was found");
    assert!(g.world.get_component::<InnerLife>(newcomer).unwrap()
        .wants.contains(&Want::Shelter), "so they want a bed");
}

#[test]
fn generated_country_is_five_times_larger_per_side() {
    let game = Game::new();
    assert_eq!(MAP_TILES, 245);
    assert_eq!(game.map.width, 245);
    assert_eq!(game.map.height, 245);
}

#[test]
fn generated_country_has_repeatable_connected_satellite_sites() {
    let game = Game::new_seeded(231);
    let replay = Game::new_seeded(231);
    assert_eq!(game.map.castles.len(), 2, "two satellite castles plus the main walled seat");
    assert_eq!(game.map.castle_keeps.len(), 2);
    assert_eq!(game.map.towns.len() + 2, 4, "main and rival towns plus two satellites");
    assert_eq!(game.map.settlement_houses.len(), 10);
    assert_eq!(game.map.castles, replay.map.castles);
    assert_eq!(game.map.towns, replay.map.towns);
    assert_eq!(game.map.settlement_houses, replay.map.settlement_houses);
    for &(x, y) in &game.map.castle_keeps {
        assert!(!game.map.passable(x, y), "keep tile is blocked");
    }
    for &(x, y) in &game.map.castle_approaches {
        assert_eq!(game.map.get(x, y), Some(Tile::Road), "castle courtyard has a road approach");
        assert!(game.map.walk_path((16, 16), (x, y)).is_some(), "castle is walk-connected to the village");
    }
    for (&keep, &approach) in game.map.castle_keeps.iter().zip(&game.map.castle_approaches) {
        assert!(game.map.walk_path(approach, keep).is_none(), "the keep blocks its own tile");
    }
    for &(x, y) in &game.map.settlement_houses {
        assert!(NEIGHBOURS.iter().any(|&(dx, dy)| game.map.get(x + dx, y + dy) == Some(Tile::Road)),
            "satellite town house has a road");
    }
    let model = game.ui_model();
    assert_eq!(model.castle_keeps.len(), 2, "castle keeps render in their courtyards");
    assert_eq!(model.settlement_houses.len(), 10, "remote towns render as map scenery");
    assert_eq!(game.guards.len(), 4, "the village watch is posted only at the main gates");
    for seed in [0, 1, 42, 9999] {
        let sample = Game::new_seeded(seed);
        for &(x, y) in &sample.map.castle_keeps {
            assert!(!sample.map.passable(x, y), "seed {} castle keep is blocked", seed);
        }
        for &(x, y) in &sample.map.castle_approaches {
            assert_eq!(sample.map.get(x, y), Some(Tile::Road), "seed {} castle approach", seed);
            assert!(sample.map.walk_path((16, 16), (x, y)).is_some(), "seed {} castle route", seed);
        }
        for &(x, y) in &sample.map.settlement_houses {
            assert!(NEIGHBOURS.iter().any(|&(dx, dy)| sample.map.get(x + dx, y + dy) == Some(Tile::Road)),
                "seed {} satellite house route", seed);
        }
    }
}
