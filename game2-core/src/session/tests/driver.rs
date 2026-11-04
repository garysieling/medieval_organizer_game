//! The fallback driver's choices for an agent (session::driver)
use crate::session::*;
use crate::session::driver::drive_agent;

fn obs() -> AgentObs {
    AgentObs {
        tick: 0, you: 1, pos: (0.0, 0.0), gold: 10.0, hunger: 80.0,
        busy: Some("Idle".into()), near: vec![], near_buildings: vec![],
        satchel: vec![], walking: false, held: false,
        orgs: vec![], campaigns: vec![], burning: vec![], me: Default::default(),
    }
}

#[test]
fn test_the_driver_fetches_what_the_bench_is_short_of() {
    let mut o = obs();
    o.tick = 1; // not a talk day
    assert!(matches!(drive_agent(&o), AgentCmd::WorkDay), "nothing short: a day's work");
    o.me.needs = Some("grain".into());
    assert!(matches!(drive_agent(&o), AgentCmd::FetchInput), "short of grain: fetch it");
    o.tick = 5; // the third day: to the bench anyway
    assert!(matches!(drive_agent(&o), AgentCmd::WorkDay));
    o.tick = 1;
    o.hunger = 10.0;
    assert!(matches!(drive_agent(&o), AgentCmd::Eat), "hunger still comes first");
}
