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
