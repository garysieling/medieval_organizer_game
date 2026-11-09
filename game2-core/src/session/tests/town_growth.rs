use crate::session::*;

/// Fill every purse and the store so the village reads prosperous
fn prosper(g: &mut Game) {
    for &v in &g.villagers {
        if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = 100.0; }
    }
    if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) {
        f.servings = 500;
    }
}

/// Empty every purse and the store so the village reads poor
fn impoverish(g: &mut Game) {
    for &v in &g.villagers {
        if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = 0.0; }
    }
    if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) {
        f.servings = 0;
    }
}

/// The growth check's monthly turns: the month after market day
fn months(n: usize) -> Vec<u64> { (0..n as u64).map(|m| 25 + m * 30).collect() }

#[test]
fn test_prosperity_must_hold_a_season_before_the_village_grows() {
    let mut g = Game::new();
    let b0 = g.buildings.len();
    // a single good month is weather, not a season
    prosper(&mut g);
    g.growth_tick(25);
    assert_eq!(g.growth_streak, 1);
    // and a poor month breaks the run
    impoverish(&mut g);
    g.growth_tick(55);
    assert_eq!(g.growth_streak, 0);
    // three straight prosperous months and something rises
    prosper(&mut g);
    g.growth_tick(85);
    g.growth_tick(115);
    assert_eq!(g.growth_streak, 2);
    assert_eq!(g.buildings.len(), b0, "two months in, no roof yet");
    g.growth_tick(145);
    assert!(g.log.iter().any(|l| l.contains("prosper")),
        "the third prosperous month grew nothing");
}

#[test]
fn test_vacant_street_plots_take_new_roofs_while_prosperity_holds() {
    let mut g = Game::new();
    let b0 = g.buildings.len();
    let e0 = g.building_entities.len();
    prosper(&mut g);
    // empty benches are re-staffed first, a roof a month after; a score
    // of months covers whatever the founding layout left going begging
    for t in months(30) { g.growth_tick(t); }
    assert!(g.buildings.len() > b0, "no roof rose in thirty prosperous months");
    assert!(g.building_entities.len() > e0, "no building entity was raised");
    assert!(g.log.iter().any(|l| l.contains("new roofs rise")), "the log never said why");
    // the newest roof stands on a street-fronting plot inside the wall
    let e = *g.building_entities.last().unwrap();
    let p = g.world.get_component::<Pos>(e).unwrap();
    let (x0, y0, x1, y1) = g.town_wall;
    assert!((x0 + 1..x1).contains(&(p.x as i32)) && (y0 + 1..y1).contains(&(p.y as i32)),
        "the new roof stands outside the wall at ({},{})", p.x, p.y);
    assert!(g.map.is_occupied(p.x as i32, p.y as i32), "the plot isn't taken");
}

#[test]
fn test_an_empty_bench_takes_a_free_hand_before_any_roof_rises() {
    let mut g = Game::new();
    // empty a workshop and free a practitioner of its trade
    let shop = g.building_entities.iter().copied().find(|&e| {
        g.world.get_component::<Building>(e)
            .map(|b| matches!(b.kind, BuildingKind::Shop(_)) && b.worker.is_some())
            .unwrap_or(false)
    }).expect("no staffed workshop");
    g.world.get_component_mut::<Building>(shop).unwrap().worker = None;
    let voc = match g.world.get_component::<Building>(shop).unwrap().kind {
        BuildingKind::Shop(v) => v,
        _ => unreachable!(),
    };
    let hand = g.villagers.iter().copied().find(|&v| {
        !villager::is_dead(&g.world, v)
            && g.world.get_component::<country::Outlander>(v).is_none()
            && g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<Craft>(v)
                .map(|c| c.vocation == voc).unwrap_or(false)
    }).expect("nobody practices the trade");
    g.world.get_component_mut::<Craft>(hand).unwrap().workplace = None;
    let staffed = |g: &Game| g.building_entities.iter().filter(|&&e| {
        g.world.get_component::<Building>(e).map(|b| b.worker.is_some()).unwrap_or(false)
    }).count();
    let (s0, b0, e0) = (staffed(&g), g.buildings.len(), g.building_entities.len());
    prosper(&mut g);
    for t in months(3) { g.growth_tick(t); }
    assert_eq!(g.buildings.len(), b0, "a re-staffing raised a roof anyway");
    assert_eq!(g.building_entities.len(), e0);
    assert_eq!(staffed(&g), s0 + 1, "no bench was re-staffed");
    assert!(g.log.iter().any(|l| l.contains("takes up the")),
        "the re-staffing was never said");
}

#[test]
fn test_a_country_town_raises_a_roof_on_its_own_treasury() {
    let mut g = Game::new();
    let si = g.sites.iter().position(|s| s.kind == country::SiteKind::Town)
        .expect("no market town");
    let homes0 = g.sites[si].homes.len();
    let b0 = g.buildings.len();
    // a thin treasury buys nothing, even on the site's own month
    g.sites[si].treasury = 100.0;
    let m = (4 - si % 4) % 4;
    let t = m as u64 * 30 + 25;
    g.growth_tick(t);
    assert_eq!(g.sites[si].homes.len(), homes0, "a poor site still grew");
    // flush with gold, the same month raises a cottage on the ring
    g.sites[si].treasury = 200.0;
    g.growth_tick(t);
    assert_eq!(g.sites[si].homes.len(), homes0 + 1, "the town didn't grow");
    assert!(g.sites[si].treasury < 200.0, "the roof wasn't paid for");
    assert!(g.buildings.len() > b0, "the new roof has no sprite");
    let e = *g.sites[si].homes.last().unwrap();
    assert!(matches!(g.world.get_component::<Building>(e).unwrap().kind, BuildingKind::Home));
    assert!(g.log.iter().any(|l| l.contains("new roof rises")),
        "the town's growth was never said");
}

#[test]
fn test_the_building_card_finds_a_grown_roof_by_its_tile() {
    let mut g = Game::new();
    prosper(&mut g);
    for t in months(30) { g.growth_tick(t); }
    // the newest sprite's index means nothing among the entities:
    // buildings and building_entities only line up for the founding
    // village, so the card must show the building on the sprite's tile
    let bi = g.buildings.len() - 1;
    let (bx, by) = (g.buildings[bi].0, g.buildings[bi].1);
    let want = g.building_entities.iter().copied().find(|&e| {
        g.world.get_component::<Pos>(e)
            .map(|p| (p.x - bx).abs() < 0.75 && (p.y - by).abs() < 0.75)
            .unwrap_or(false)
    }).map(|e| g.world.get_component::<Building>(e).unwrap().name.clone());
    assert!(want.is_some(), "no entity stands on the newest sprite's tile");
    g.seats[0].selected_building = Some(bi);
    let insp = g.building_inspector_at(0).expect("the roof should open a card");
    assert_eq!(insp.lines.first().map(|l| l.text.as_str()), want.as_deref(),
        "the card shows the wrong building");
}


