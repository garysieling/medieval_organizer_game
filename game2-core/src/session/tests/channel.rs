use crate::session::*;

#[test]
fn test_the_channels_accrue_on_the_work() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.turnout = 0.8;
    o.agitation = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.channel_tick(51);
    let pulpit = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Pulpit)).unwrap_or(0.0);
    assert!(pulpit > 0.0, "the channels never accrued");
}

#[test]
fn test_the_fast_channel_leaks_to_the_castle() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.signal_net = 0.8;
    o.turnout = 0.8;
    o.agitation = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    let mut before = 0.0;
    for site in &g.sites {
        if let Some(l) = site.lord {
            g.world.add_component(l, crate::session::subsystems::society::Watcher {
                paranoia: 0.5, legitimacy: 0.8 });
        }
    }
    g.channel_tick(51);
    for site in &g.sites {
        if let Some(l) = site.lord {
            if let Some(w) = g.world.get_component::<crate::session::subsystems::society::Watcher>(l) {
                before = w.paranoia;
            }
        }
    }
    assert!(before > 0.5, "the fast channel never leaked: {}", before);
}
