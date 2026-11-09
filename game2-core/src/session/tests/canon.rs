use crate::session::*;

#[test]
fn test_the_redacted_telling_dulls_the_edge() {
    let mut g = Game::new();
    g.redacted_canon = true;
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.agitation = 0.5;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.canon_tick(46);
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.agitation).unwrap_or(0.5);
    assert!(after < 0.5, "the redaction never dulled: {}", after);
}

#[test]
fn test_the_vernacular_reads_past_it() {
    let mut g = Game::new();
    g.redacted_canon = true;
    g.vernacular = true;
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.agitation = 0.5;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.canon_tick(46);
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.agitation).unwrap_or(0.5);
    assert!(after > 0.5, "the vernacular never restored: {}", after);
}

#[test]
fn test_the_literate_fold_reads_past_it() {
    let mut g = Game::new();
    g.redacted_canon = true;
    g.numeracy = crate::session::Numeracy::Positional;
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.agitation = 0.5;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.canon_tick(46);
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(m)
        .map(|o| o.agitation).unwrap_or(0.5);
    assert!(after > 0.5, "the letters never read past: {}", after);
}
