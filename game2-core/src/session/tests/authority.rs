use crate::session::*;
use crate::game::villager::Pos;

#[test]
fn test_the_org_weakens_the_lord() {
    let mut g = Game::new();
    g.lord_strength = 0.8;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.8;
    }
    g.authority_tick(49);
    assert!(g.lord_strength < 0.8, "the org never dissolved: {}", g.lord_strength);
}

#[test]
fn test_the_weak_lord_feeds_the_org() {
    let mut g = Game::new();
    g.lord_strength = 0.2;
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.3;
    }
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        org.resources = 0.0;
    }
    g.authority_tick(49);
    let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|o| o.capacity).unwrap_or(0.3);
    assert!(after > 0.3, "the weak lord never fed: {}", after);
    let resources = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .map(|o| o.resources).unwrap_or(0.0);
    assert!(resources > 0.0, "the self-org never answered: {}", resources);
}

fn grieve(g: &mut Game, v: Entity, tick: u64) {
    crate::game::desires::add_grievance(&mut g.world, v,
        crate::game::desires::Problem::Taxed, tick + 60);
}

/// A village the writ no longer reaches: nothing lands on the road
/// and nobody stands content
fn broken_reach(g: &mut Game) {
    g.lord_flow = 0.0;
    for &v in g.villagers.clone().iter() { grieve(g, v, g.tick); }
}

#[test]
fn test_the_writ_runs_out_and_the_vacuum_fills() {
    let mut g = Game::new();
    g.lord_strength = 0.14;
    broken_reach(&mut g);
    g.authority_tick(49);
    assert_eq!(g.lord_gone, Some(49), "the writ never ran out");
    let filler = g.vacuum_filler.expect("the vacuum filled to nobody");
    // and the reach stops working: no round, no ride - and what the
    // books still hold stays with the claimant the vacuum named
    g.takings = 20.0;
    g.livelihood_tick(45);
    let c = g.collector.unwrap();
    assert!(!g.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::Remit(..))),
        "a dissolved writ still rode the gold out");
    assert_eq!(g.takings, 0.0, "the take never found its claimant");
    let res = g.world.get_component::<Organization>(filler).map(|o| o.resources).unwrap_or(0.0);
    assert!(res >= 20.0, "the vacuum's claimant got nothing: {}", res);
    assert!(g.log.iter().any(|l| l.contains("the writ runs out")),
        "the dissolution went unsaid");
}

#[test]
fn test_the_vacuum_fills_to_the_strongest_claim() {
    let mut g = Game::new();
    // the cause grown strong is the strongest claim in the village
    if let Some(o) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        o.capacity = 0.9;
    }
    g.lord_strength = 0.14;
    broken_reach(&mut g);
    g.authority_tick(49);
    assert_eq!(g.vacuum_filler, Some(g.org),
        "the vacuum didn't fill to the cause that broke it");
}

#[test]
fn test_the_writ_comes_back_late_and_dear() {
    let mut g = Game::new();
    g.lord_strength = 0.14;
    broken_reach(&mut g);
    g.authority_tick(49);
    assert!(g.lord_gone.is_some());
    // a season and a third later the crown answers: a new bailiff, a
    // harsher hand, and a reach the village can contest again
    g.authority_tick(49 + 120);
    assert!(g.lord_gone.is_none(), "the writ never came back");
    assert_eq!(g.lord_strength, 0.45, "the return wasn't a shadow of the old reach");
    assert!(g.repression_debt >= 2, "the return came cheap");
}

#[test]
fn test_a_rising_thins_the_writ() {
    let mut g = Game::new();
    g.lord_strength = 0.8;
    for v in g.villagers.iter().take(9).copied().collect::<Vec<_>>() { grieve(&mut g, v, 55); }
    g.uprising_tick(55);
    assert!(g.lord_strength <= 0.71, "the rising cost the writ nothing: {}", g.lord_strength);
}

#[test]
fn test_the_village_own_rules_shrink_the_writ() {
    let mut g = Game::new();
    g.lord_strength = 0.8;
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap();
    for p in [crate::game::desires::Problem::ThinWood, crate::game::desires::Problem::DryPit,
              crate::game::desires::Problem::StinkingReach] {
        crate::game::desires::add_grievance(&mut g.world, v, p, 70);
    }
    g.bylaw_tick(10);
    assert!(!g.bylaws.is_empty(), "the friction never became a rule");
    assert!(g.lord_strength < 0.8, "the village's own rule left the writ whole");
}

#[test]
fn test_a_carried_taxes_grant_holds_the_ride_back() {
    let mut g = Game::new();
    let oe = g.seats[0].org;
    g.world.add_component(oe, crate::game::reactions::LordGrant::full("the taxes", 0, 100));
    let res_before = g.world.get_component::<Organization>(oe).map(|o| o.resources).unwrap_or(0.0);
    g.takings = 30.0;
    g.livelihood_tick(45);
    let c = g.collector.unwrap();
    let t = g.travels.iter().find(|t| t.who == c).expect("the collector never left");
    let Goal::Remit(_, amt) = t.goal else { panic!("his errand's no remit: {:?}", t.goal) };
    // 30 of takings, the stalls' 40% kept, then the refusal's 40% of
    // what remained - about eleven rides, not eighteen
    assert!(amt <= 11.0, "the refusal never held the take back: {}", amt);
    let res_after = g.world.get_component::<Organization>(oe).map(|o| o.resources).unwrap_or(0.0);
    assert!(res_after > res_before, "what stayed home never reached the cause");
}

#[test]
fn test_an_intercepted_ride_thins_the_reach() {
    let mut g = {
        let mut g = None;
        for seed in 1..60u64 {
            let gg = Game::new_seeded(seed);
            if !gg.rebel_camps.is_empty() { g = Some(gg); break; }
        }
        g.expect("no seed in 1..60 dealt a rebel band")
    };
    let who = g.villagers[0];
    let si = g.sites.iter().position(|s| matches!(s.kind, crate::session::country::SiteKind::Castle))
        .expect("no castle for the ride");
    let wild = g.rebel_camps[0];
    let band = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<rebels::Rebel>(v)
            .map(|r| r.band == 0).unwrap_or(false))
        .expect("band 0 has no members");
    let mut robbed = false;
    for day in 1..=400u64 {
        g.travels.retain(|t| t.who != who);
        g.travels.push(Travel { who, path: vec![], goal: Goal::Remit(si, 25.0) });
        if let Some(p) = g.world.get_component_mut::<Pos>(who) {
            p.x = wild.0 as f32 + 0.5; p.y = wild.1 as f32 + 0.5;
        }
        if let Some(p) = g.world.get_component_mut::<Pos>(band) {
            p.x = wild.0 as f32 + 1.5; p.y = wild.1 as f32 + 0.5;
        }
        g.tick = day;
        g.rebel_ambush();
        if g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::HeadHome(_))) {
            robbed = true;
            break;
        }
    }
    assert!(robbed, "the band never fell on the gold in 400 days");
    assert!(g.lord_flow <= 0.4 + 1e-6,
        "a take off the road didn't thin the reach: {}", g.lord_flow);
}
