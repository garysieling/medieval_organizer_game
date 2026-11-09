use crate::session::*;

#[test]
fn test_the_faithful_walk_out_when_the_load_outgrows_the_faith() {
    let mut g = Game::new();
    let cong = g.congregation;
    let m = g.world.get_component::<crate::sim::religion::Congregation>(cong)
        .unwrap().members.iter().copied()
        .find(|&v| g.world.get_component::<crate::game::desires::InnerLife>(v).is_some())
        .expect("a member with an inner life");
    // a drifting heart carrying a season of grievances: the tithe's call
    // alone won't hold them
    for (i, p) in [crate::game::desires::Problem::Taxed, crate::game::desires::Problem::Loss,
        crate::game::desires::Problem::Broke, crate::game::desires::Problem::StinkingReach,
        crate::game::desires::Problem::OverseerCut].into_iter().enumerate() {
        crate::game::desires::add_grievance(&mut g.world, m, p, 60 + i as u64);
    }
    g.world.get_component_mut::<crate::sim::religion::Faith>(m)
        .map(|f| f.devotion = 0.0);
    let before = g.world.get_component::<crate::sim::religion::Congregation>(cong)
        .unwrap().members.len();
    // find the day the det-mix lets them walk
    for t in 0..40 { g.exit_tick(t); }
    let live = g.world.get_component::<crate::game::desires::InnerLife>(m)
        .map(|l| l.grievances.len()).unwrap_or(0);
    eprintln!("m={} live={} members_before={} members_after={}", m, live,
        before, g.world.get_component::<crate::sim::religion::Congregation>(cong).unwrap().members.len());
    let after = g.world.get_component::<crate::sim::religion::Congregation>(cong)
        .unwrap().members.len();
    assert!(after < before, "the exit never fired");
    assert!(g.world.get_component::<crate::sim::religion::Apostate>(m).is_some(),
        "the walkout's mark never landed");
}
