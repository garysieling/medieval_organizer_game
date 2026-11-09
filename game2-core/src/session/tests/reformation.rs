use crate::session::*;

/// A village adult of the fold who keeps the parish ways - the raw
/// material of a reformer
fn parish_villager(g: &Game) -> Entity {
    g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Kid>(v).is_none()
            && g.site_of(v).is_none()
            && g.world.get_component::<crate::sim::religion::Faith>(v).is_some()
            && !routine::keeps_old_ways(&g.world, v)
            && g.world.get_component::<Craft>(v)
                .map(|c| c.vocation != Vocation::Priest).unwrap_or(true))
        .expect("a parish villager")
}

/// Expire every live grievance in the village except `keep`'s - the
/// test controls who the soil is ready for
fn clear_grievances(g: &mut Game, keep: Option<Entity>) {
    for &v in &g.villagers {
        if Some(v) == keep { continue; }
        if let Some(l) = g.world.get_component_mut::<InnerLife>(v) {
            l.grievances.clear();
        }
    }
}

/// Make a soul the readiest soil in the village: devotion gone thin,
/// two live grievances burning
fn make_susceptible(g: &mut Game, v: Entity, tick: u64) {
    if let Some(f) = g.world.get_component_mut::<crate::sim::religion::Faith>(v) {
        f.devotion = 0.15;
    }
    desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, tick + 90);
    desires::add_grievance(&mut g.world, v, desires::Problem::Broke, tick + 90);
}

/// Spark the reformation and run the tick that spends it; returns the
/// reformer the fellowship chose
fn ignite(g: &mut Game, tick: u64) -> Entity {
    let v = parish_villager(g);
    clear_grievances(g, Some(v));
    make_susceptible(g, v, tick);
    g.spark_reformation(tick, "the test's provocation");
    g.reformation_tick(tick);
    let sg = g.splinter.expect("the fellowship never stood");
    g.world.get_component::<crate::sim::religion::Congregation>(sg)
        .and_then(|c| c.leader).expect("the fellowship has no leader")
}

fn splinter_members(g: &Game) -> Vec<Entity> {
    g.splinter.map(|sg| g.world.get_component::<crate::sim::religion::Congregation>(sg)
        .map(|c| c.members.clone()).unwrap_or_default()).unwrap_or_default()
}

fn parish_members(g: &Game) -> Vec<Entity> {
    g.world.get_component::<crate::sim::religion::Congregation>(g.congregation)
        .map(|c| c.members.clone()).unwrap_or_default()
}

#[test]
fn test_a_spark_finds_the_readiest_soul_and_the_fellowship_stands() {
    let mut g = Game::new();
    let v = parish_villager(&g);
    clear_grievances(&mut g, Some(v));
    make_susceptible(&mut g, v, 100);
    g.spark_reformation(100, "the test's provocation");
    g.reformation_tick(100);

    let sg = g.splinter.expect("the fellowship never stood");
    let cong = g.world.get_component::<crate::sim::religion::Congregation>(sg).unwrap();
    assert_eq!(cong.leader, Some(v), "the readiest soul didn't take the word");
    assert_eq!(cong.name, "the green fellowship");
    assert!(cong.orthodoxy < 0.5, "a splinter isn't orthodox");
    assert!(parish_members(&g).iter().all(|&m| m != v), "the reformer stayed in the fold");
    assert!(g.world.get_component::<crate::sim::religion::Apostate>(v).is_some());
    assert!(g.world.get_component::<crate::sim::religion::Reformed>(v).is_some());
    let f = g.world.get_component::<crate::sim::religion::Faith>(v).unwrap();
    assert_eq!(f.religion, "the green fellowship");
    assert!(f.traditions.iter().any(|t| t == "the green word"));
    assert!(g.world.get_component::<InnerLife>(v).unwrap().grievances
        .iter().any(|(p, _)| *p == desires::Problem::Doctrine), "no doctrine grievance");
    assert!(g.reformation_spark.is_none(), "the spark wasn't spent");
    // and nobody else was swept up in it
    assert!(g.villagers.iter().filter(|&&w| w != v)
        .all(|&w| g.world.get_component::<crate::sim::religion::Reformed>(w).is_none()));
    // the parish answered: the log saw it and the priest spoke against it
    assert!(g.log.iter().any(|l| l.contains("green word")), "no ignition in the log");
}

#[test]
fn test_no_spark_no_fellowship() {
    let mut g = Game::new();
    for t in 100..140 { g.reformation_tick(t); }
    assert!(g.splinter.is_none());
    assert!(g.villagers.iter().all(|&v|
        g.world.get_component::<crate::sim::religion::Reformed>(v).is_none()));
}

#[test]
fn test_a_spark_fades_when_the_soil_is_too_thin() {
    let mut g = Game::new();
    clear_grievances(&mut g, None); // nobody carries the burn the reformer needs
    for &v in &g.villagers {
        if let Some(f) = g.world.get_component_mut::<crate::sim::religion::Faith>(v) {
            f.devotion = 0.9;
        }
    }
    g.spark_reformation(100, "the test's provocation");
    for t in 100..=131 { g.reformation_tick(t); }
    assert!(g.splinter.is_none(), "a fellowship rose without soil");
    assert!(g.reformation_spark.is_none(), "the spark should have burned out");
}

#[test]
fn test_the_faithful_drift_to_the_green() {
    let mut g = Game::new();
    let reformer = ignite(&mut g, 100);
    assert_eq!(splinter_members(&g).len(), 1);
    // six more parish hearts gone thin - no grievance needed to drift
    let mut prepared = 0;
    for &v in &g.villagers.clone() {
        if prepared >= 6 || v == reformer { continue; }
        if g.world.get_component::<Kid>(v).is_some() || g.site_of(v).is_some()
            || routine::keeps_old_ways(&g.world, v) { continue; }
        if g.world.get_component::<crate::sim::religion::Faith>(v).is_none() { continue; }
        if g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false) { continue; }
        if let Some(f) = g.world.get_component_mut::<crate::sim::religion::Faith>(v) {
            f.devotion = 0.2;
        }
        prepared += 1;
    }
    assert!(prepared >= 4, "too few driftable hearts to test");
    for t in 101..=190 { g.reformation_tick(t); }
    let members = splinter_members(&g);
    assert!(members.len() >= 3, "the fellowship never grew: {}", members.len());
    assert!(members.contains(&reformer), "the reformer fell out of their own fold");
    // a defector wears the marks: out of the parish, into the green word
    let defector = members.iter().copied().find(|&m| m != reformer).expect("no defector");
    assert!(!parish_members(&g).contains(&defector), "the defector stayed in the fold");
    assert!(g.world.get_component::<crate::sim::religion::Reformed>(defector).is_some());
    assert_eq!(g.world.get_component::<crate::sim::religion::Faith>(defector)
        .unwrap().religion, "the green fellowship");
    assert!(g.world.get_component::<InnerLife>(defector).unwrap().grievances
        .iter().any(|(p, _)| *p == desires::Problem::Doctrine));
}

#[test]
fn test_the_green_keeps_its_own_holy_day() {
    let mut g = Game::new();
    let reformer = ignite(&mut g, 100);
    // grow the fellowship a little so the green has company
    for &v in &g.villagers.clone() {
        if v == reformer { continue; }
        if g.world.get_component::<Kid>(v).is_some() || g.site_of(v).is_some()
            || routine::keeps_old_ways(&g.world, v) { continue; }
        if let Some(f) = g.world.get_component_mut::<crate::sim::religion::Faith>(v) {
            f.devotion = 0.15;
        }
    }
    for t in 101..=140 { g.reformation_tick(t); }
    g.travels.clear(); // nobody's mid-walk when the call comes
    let holy = 105; // a holy day, after the kindling
    g.routine_dawn(holy, &mut Rng(3));

    let green: Vec<Entity> = g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<routine::Routine>(v)
            .map(|r| r.mode == routine::Mode::Worshipping && r.spot == routine::Spot::Green && r.day == holy)
            .unwrap_or(false)).collect();
    assert!(green.contains(&reformer), "the reformer didn't answer the green call");
    assert!(green.len() >= 2, "the green fellowship worshipped alone: {}", green.len());
    assert!(green.iter().all(|&v| g.world.get_component::<crate::sim::religion::Reformed>(v).is_some()),
        "a parishioner slipped into the green word");
    // and the parish still keeps its own rows at the church
    let church = g.villagers.iter().filter(|&&v| parish_members(&g).contains(&v)).filter(|&&v|
        g.world.get_component::<routine::Routine>(v)
            .map(|r| r.mode == routine::Mode::Worshipping && r.spot == routine::Spot::Church && r.day == holy)
            .unwrap_or(false)).count();
    assert!(church > 0, "the parish kept no holy day of its own");
    // the inspector says whose word it is
    let text = routine::routine_text(&g.world, reformer, None).unwrap_or_default();
    assert!(text.contains("green") || text.contains("preaching"), "the routine text hid the green: {}", text);
}

#[test]
fn test_the_green_word_feeds_devotion_but_not_the_parish_box() {
    let mut g = Game::new();
    let reformer = ignite(&mut g, 100);
    let parishioner = parish_members(&g).into_iter()
        .find(|&v| g.world.get_component::<Kid>(v).is_none()).expect("a parishioner");
    let holy = routine::HOLY_EVERY;
    for (v, spot) in [(reformer, routine::Spot::Green), (parishioner, routine::Spot::Church)] {
        g.world.add_component(v, routine::Routine {
            day: holy, mode: routine::Mode::Worshipping, spot,
            anchor: (0.0, 0.0), face: None, drift: 0.0, leads: false, reached: false });
    }
    // the reformer goes hungry: the parish's poor box isn't theirs
    g.world.get_component_mut::<Hunger>(reformer).unwrap().value = 5.0;
    let d0 = g.world.get_component::<crate::sim::religion::Faith>(reformer).unwrap().devotion;
    let p0 = g.world.get_component::<crate::sim::religion::Faith>(parishioner).unwrap().devotion;
    g.service_tick(holy);
    let d1 = g.world.get_component::<crate::sim::religion::Faith>(reformer).unwrap().devotion;
    assert!(d1 > d0, "the green word fed no devotion: {} -> {}", d0, d1);
    assert_eq!(g.world.get_component::<Hunger>(reformer).unwrap().value, 5.0,
        "the parish's poor box reached into the green fellowship");
    // and the parishioner still got the parish's own office
    let p1 = g.world.get_component::<crate::sim::religion::Faith>(parishioner).unwrap().devotion;
    assert!(p1 > p0, "the parish office fed no devotion: {} -> {}", p0, p1);
}

#[test]
fn test_a_strict_parish_answers_with_shame() {
    // high orthodoxy: the parish reaches for the ladder at once
    let mut g = Game::new();
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.orthodoxy = 0.8;
    }
    let v = ignite(&mut g, 100);
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v
        && *s == crate::game::sanctions::Sanction::Gossip),
        "a strict parish let the reformer go unmarked");

    // a lax parish mutters but doesn't reach
    let mut g2 = Game::new();
    if let Some(c) = g2.world.get_component_mut::<crate::sim::religion::Congregation>(g2.congregation) {
        c.orthodoxy = 0.4;
    }
    let v2 = ignite(&mut g2, 100);
    assert!(!g2.sanctioned.iter().any(|(e, _, _)| *e == v2),
        "a lax parish sanctioned its own dissident");
}

#[test]
fn test_the_old_ways_are_not_moved() {
    let mut g = Game::new();
    // the readiest old-faith soul in the village
    let old = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Kid>(v).is_none()
            && g.site_of(v).is_none()
            && g.world.get_component::<crate::sim::religion::Faith>(v).is_some()
            && routine::keeps_old_ways(&g.world, v)).expect("an old-faith villager");
    make_susceptible(&mut g, old, 100);
    // the spark lands on somebody else - the old ways can't be the reformer
    let reformer = ignite(&mut g, 100);
    assert_ne!(reformer, old, "the old faith took the green word");
    // and no amount of drift moves them: their stream is their own
    for t in 101..=220 { g.reformation_tick(t); }
    assert!(g.world.get_component::<crate::sim::religion::Reformed>(old).is_none(),
        "the stones' keeper defected to the green");
    assert!(parish_members(&g).contains(&old) || g.world.get_component::<crate::sim::religion::Apostate>(old).is_some(),
        "the old-faith keeper left the fold by another road");
}

#[test]
fn test_the_player_can_stand_with_the_green_word() {
    let mut g = Game::new();
    let reformer = ignite(&mut g, 100);
    let holy = 105;
    g.tick = holy;
    g.travels.clear();
    g.routine_dawn(holy, &mut Rng(5));
    // the player joins the gathering the holy day offers - the green
    // word counts as a gathering like any other
    if !g.seats[0].ctx_actions.contains(&crate::session::seat::CtxAct::Attend) {
        g.seats[0].ctx_actions.push(crate::session::seat::CtxAct::Attend);
    }
    let i = g.seats[0].ctx_actions.iter()
        .position(|a| *a == crate::session::seat::CtxAct::Attend).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.attended_days.iter().any(|&(p, d)| p == g.seats[0].player && d == holy),
        "the player couldn't stand in the green word");
    assert!(g.world.get_component::<routine::Routine>(reformer)
        .map(|r| r.mode == routine::Mode::Worshipping && r.spot == routine::Spot::Green).unwrap_or(false),
        "the reformer wasn't at the green to be attended");
}

#[test]
fn test_a_fallen_reformer_hands_on_the_word() {
    let mut g = Game::new();
    let reformer = ignite(&mut g, 100);
    // several hearts gone thin, so the fellowship gains a second soul
    let mut prepared = 0;
    for &v in &g.villagers.clone() {
        if prepared >= 4 || v == reformer { continue; }
        if g.world.get_component::<Kid>(v).is_some() || g.site_of(v).is_some()
            || routine::keeps_old_ways(&g.world, v) { continue; }
        if g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false) { continue; }
        if let Some(f) = g.world.get_component_mut::<crate::sim::religion::Faith>(v) {
            f.devotion = 0.1;
        }
        prepared += 1;
    }
    for t in 101..=200 {
        g.reformation_tick(t);
        if splinter_members(&g).len() >= 2 { break; }
    }
    let members = splinter_members(&g);
    assert!(members.len() >= 2, "the fellowship never gained a second soul");
    // the reformer dies: the word passes to the next soul
    g.world.add_component(reformer, crate::game::villager::Dead);
    g.reformation_tick(161);
    let sg = g.splinter.unwrap();
    let cong = g.world.get_component::<crate::sim::religion::Congregation>(sg).unwrap();
    let leader = cong.leader.expect("the fellowship lost its leader entirely");
    assert_ne!(leader, reformer, "a dead man still leads the green word");
    assert!(cong.members.contains(&leader));
}

#[test]
fn test_the_same_seed_reforms_alike() {
    let run = || {
        let mut g = Game::new();
        let v = parish_villager(&g);
        clear_grievances(&mut g, Some(v));
        make_susceptible(&mut g, v, 100);
        g.spark_reformation(100, "the test's provocation");
        for t in 100..=130 { g.reformation_tick(t); }
        let leader = g.splinter.and_then(|sg| g.world
            .get_component::<crate::sim::religion::Congregation>(sg).and_then(|c| c.leader));
        (leader, splinter_members(&g).len())
    };
    let (a_leader, a_n) = run();
    let (b_leader, b_n) = run();
    assert_eq!(a_leader, b_leader, "the same seed chose different reformers");
    assert_eq!(a_n, b_n, "the same seed grew a different fellowship");
}
