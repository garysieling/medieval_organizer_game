use crate::session::*;

#[test]
fn test_the_conflict_forms_between_the_castles() {
    let mut g = Game::new();
    g.war_tick(111);
    assert!(g.war.is_some(), "the war never formed");
    assert!(g.log.iter().any(|l| l.contains("the levies are called")),
        "the call never logged");
}

#[test]
fn test_the_lords_own_standing_pays() {
    let mut g = Game::new();
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.8 });
        }
    }
    g.war_tick(111);
    let mut after = 0.8;
    for site in &g.sites {
        if let Some(l) = site.lord {
            if let Some(w) = g.world.get_component::<crate::session::subsystems::society::Watcher>(l) {
                after = w.legitimacy;
            }
        }
    }
    assert!(after < 0.8, "the tribute never cost: {}", after);
}

#[test]
fn test_the_levied_carry_the_grievance() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::InnerLife {
        wants: vec![], grievances: vec![], satisfied: vec![] });
    g.war_tick(111);
    let grieved = g.world.get_component::<crate::game::desires::InnerLife>(m)
        .map(|l| !l.grievances.is_empty()).unwrap_or(false);
    assert!(grieved, "the levied never grieved");
}
