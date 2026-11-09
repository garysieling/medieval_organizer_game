use crate::session::*;

#[test]
fn test_the_stall_calls_the_council() {
    let mut g = Game::new();
    g.separation_pressure = 0.8;
    g.council_tick(113);
    assert!(g.council.is_some(), "the stall never called the council");
    assert!(g.log.iter().any(|l| l.contains("a council's called")),
        "the call never logged");
}

#[test]
fn test_the_convene_settles_and_costs() {
    let mut g = Game::new();
    g.council = Some(g.tick);
    g.separation_pressure = 0.8;
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.8 });
        }
    }
    g.decide_at(0, 0); // convene
    assert!(g.separation_pressure < 0.8, "the convene never settled");
    assert!(g.log.iter().any(|l| l.contains("the council's convened")),
        "the convene never logged");
    let mut after = 0.8;
    for site in &g.sites {
        if let Some(l) = site.lord {
            if let Some(w) = g.world.get_component::<crate::session::subsystems::society::Watcher>(l) {
                after = w.legitimacy;
            }
        }
    }
    assert!(after < 0.8, "the convening never cost: {}", after);
}

#[test]
fn test_the_stall_deepens_unconvened() {
    let mut g = Game::new();
    g.council = Some(g.tick);
    g.separation_pressure = 0.5;
    g.decide_at(0, 1); // let it stall
    assert!(g.separation_pressure > 0.5, "the stall never deepened");
}
