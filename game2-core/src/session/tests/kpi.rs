//! Public KPIs come from a seat's view alone, and the secret gauges never
//! reach a model or an observation (docs/architecture.md, rule 4).
use crate::session::*;

/// Gauges only the omniscient side has
const SECRET_ONLY: [&str; 6] = ["org.power", "org.lord_esteem", "villagers.hungry", "villagers.starving", "trails.heat", "price.grain"];

#[test]
fn the_frame_carries_public_kpis_never_secret_ones() {
    let mut g = Game::new();
    while g.tick < 3 { g.update(); }
    let m = g.ui_model();
    assert_eq!(m.debug_lines, kpi::kpi_lines(&kpi::public_kpis(&m)), "the readout is the seat's own KPIs");
    let wire = serde_json::to_string(&m).unwrap();
    for k in SECRET_ONLY {
        assert!(!wire.contains(k), "the model on the wire leaks the secret gauge {k}");
    }
    // and the secret side still has them, for the tests and the balance runs
    let secret = g.secret_kpis();
    for k in SECRET_ONLY {
        assert!(secret.iter().any(|(s, _)| *s == k), "secret gauge {k} is gone");
    }
}

#[test]
fn public_kpis_cant_see_hidden_truth() {
    let mut g = Game::new();
    while g.tick < 3 { g.update(); }
    let before = kpi::public_kpis(&g.ui_model());
    let secret_before = g.secret_kpis();
    // the hidden truth moves: the org's real strength under the lord's guess,
    // and a villager's real hunger with no mark to show it yet
    let org = g.org;
    if let Some(p) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(org) {
        p.capacity += 5.0;
    }
    g.lord_esteem += 3.0;
    let after = kpi::public_kpis(&g.ui_model());
    assert_eq!(before, after, "a public KPI moved with something the seat can't see");
    assert_ne!(secret_before, g.secret_kpis(), "the secret gauges do see it");
}

#[test]
fn public_kpis_count_what_the_frame_shows() {
    let mut g = Game::new();
    while g.tick < 3 { g.update(); }
    let m = g.ui_model();
    let k = kpi::public_kpis(&m);
    let get = |name: &str| k.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).unwrap();
    assert_eq!(get("you.gold"), m.panel.gold as f64);
    assert_eq!(get("market.food"), m.panel.food as f64);
    let figures = m.people.iter().filter(|p| matches!(p.kind, PKind::Villager(_)) && !p.dead).count();
    assert_eq!(get("folk.seen"), figures as f64);
}

#[test]
fn stock_food_counts_the_whole_village_table() {
    let mut g = Game::new();
    assert!(!g.crop_fields.is_empty(), "the village has fields to eat");
    let stock = |g: &Game| g.secret_kpis().iter()
        .find(|(k, _)| *k == "stock.food").map(|(_, v)| *v).unwrap();
    let set = |g: &mut Game, e: Entity, n: u32| {
        g.world.get_component_mut::<villager::FoodSource>(e).unwrap().servings = n;
    };
    // the market runs bare but a field stands stocked - a hungry soul
    // walks to it, so the gauge has to see it (the old census read the
    // market pile alone and reported famine on a full table)
    let market = g.market_food;
    let fields: Vec<_> = g.crop_fields.clone();
    set(&mut g, market, 0);
    for (i, &f) in fields.iter().enumerate() { set(&mut g, f, if i == 0 { 40 } else { 0 }); }
    assert_eq!(stock(&g), 40.0, "a stocked field is still food on the village table");
    // and the other settlements' granaries - full, but somebody else's
    // table a world away - don't prop the village's cover up
    set(&mut g, fields[0], 0);
    assert_eq!(stock(&g), 0.0);
    assert!(g.sites.iter().all(|s| g.world.get_component::<villager::FoodSource>(s.store)
        .map(|f| f.servings > 0).unwrap_or(false)),
        "the granaries still hold - they just aren't the village's table");
}

#[test]
fn an_agent_gets_its_own_seat_and_kpis() {
    let mut g = Game::new();
    let Some(i) = g.free_villager() else { return };
    let seat = g.add_agent_seat(g.villagers[i]);
    let obs = g.agent_obs(seat).expect("the agent observes");
    let k = kpi::agent_kpis(&obs);
    let get = |name: &str| k.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).unwrap();
    assert_eq!(get("you.gold"), obs.gold as f64);
    assert_eq!(get("you.hunger"), obs.hunger as f64);
    let wire = serde_json::to_string(&g.ui_model_at(seat)).unwrap();
    for s in SECRET_ONLY {
        assert!(!wire.contains(s), "an agent seat's model leaks {s}");
    }
}
