use crate::session::*;

#[test]
fn test_the_canvass_counts_and_develops() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.one_on_one = 0.2;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.petition_count = 0;
    g.canvass_tick(28);
    assert!(g.petition_count > 0, "the canvass never counted");
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.one_on_one).unwrap_or(0.2);
    assert!(after > 0.2, "the carrying never developed: {}", after);
}

#[test]
fn test_the_ban_backfires_only_with_the_frame() {
    let mut g = Game::new();
    g.frame_readiness = true;
    g.petition_count = 10;
    let m = g.villagers[0];
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.8, legitimacy: 0.8 });
        }
    }
    g.canvass_tick(28);
    assert!(g.log.iter().any(|l| l.contains("the ban's the story")),
        "the frame never caught the ban");
}
