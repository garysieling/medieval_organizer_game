use crate::session::*;

#[test]
fn test_the_lords_word_beats_the_crowns_where_its_set() {
    let g = Game::new();
    let realm = g.law_book.law_for(99);
    assert!((realm.tax_share - 0.20).abs() < 0.01, "the realm's fifth reads wrong");
}

#[test]
fn test_a_local_override_sits_over_the_realm() {
    let mut g = Game::new();
    g.law_book.overrides.insert(1, crate::game::laws::Law {
        tax_share: 0.33, assembly_cap: 1, drink_levy: 0.30, writ_mult: 2.0, wage_cap: 0.0 });
    let law = g.law_book.law_for(1);
    assert!((law.tax_share - 0.33).abs() < 0.01, "the lord's override didn't take");
    assert!((g.law_book.law_for(2).tax_share - 0.20).abs() < 0.01, "the realm moved for one lord");
}

#[test]
fn test_the_book_answers_a_caught_wrong() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.get_component_mut::<Purse>(v).map(|p| p.gold = 20.0);
    g.apply_penalty(0, v, "theft", 50);
    let gold = g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(99.0);
    assert!(gold < 20.0, "the fine never took");
    g.apply_penalty(0, v, "defiance", 51);
    assert!(crate::game::dungeon::unavailable(&g.world, v), "the defier never took the cell");
}

#[test]
fn test_the_statute_posts_when_hands_run_short_and_the_cause_stirs() {
    let mut g = Game::new();
    // hands run scarce: the wage climbs on its own
    for &v in g.villagers.iter().skip(2).take(16) {
        g.world.add_component(v, crate::game::villager::Dead);
    }
    assert!(g.wage_mult() > 1.4, "the wage's running hot: {}", g.wage_mult());
    // and the cause's already talking about it
    let m = g.villagers[0];
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .map(|o| o.members.push(m));
    g.world.add_component(m, crate::game::org_skills::Organizing {
        turnout: 0.4, agitation: 0.3, ..Default::default() });
    g.wage_edict_tick(19 + 30);
    assert!(g.law_book.realm.wage_cap > 0.0, "the cap never posted");
    assert!(g.log.iter().any(|l| l.contains("statute")),
        "the writ never read out: {:?}", g.log.iter().rev().take(3).collect::<Vec<_>>());
}

#[test]
fn test_the_statute_stays_quiet_while_the_cause_is() {
    let mut g = Game::new();
    for &v in g.villagers.iter().skip(2).take(16) {
        g.world.add_component(v, crate::game::villager::Dead);
    }
    // hot wages, but nobody's combining - the lord has no grievance
    g.wage_edict_tick(19 + 30);
    assert_eq!(g.law_book.realm.wage_cap, 0.0, "the cap posted on wages alone");
}

#[test]
fn test_over_cap_pay_feeds_the_informers() {
    let mut g = Game::new();
    g.law_book.realm.wage_cap = 4.0;
    g.penetration = 1.0; // the eyes are everywhere
    for &v in g.villagers.iter().take(8) {
        g.world.get_component_mut::<Purse>(v).map(|p| p.gold = 20.0);
        g.excess_wages.push(v);
    }
    g.wage_edict_tick(19 + 30);
    assert!(g.excess_wages.is_empty(), "the report queue never drained");
    let fined = g.log.iter().filter(|l| l.contains("statute's rate")).count();
    assert!(fined > 0, "no over-cap report ever landed: {:?}", g.log.iter().rev().take(5).collect::<Vec<_>>());
}

#[test]
fn test_combining_is_the_conspiracy_charge() {
    let mut g = Game::new();
    g.law_book.realm.wage_cap = 4.0;
    g.penetration = 1.0;
    // the whole cause's in the stocks' reach over a few beats
    let members: Vec<Entity> = g.villagers.iter().copied().take(6).collect();
    for &m in &members {
        g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
            .map(|o| o.members.push(m));
    }
    for t in (19..=19 + 30 * 30).step_by(30) { g.wage_edict_tick(t); }
    let taken = g.log.iter().filter(|l| l.contains("combining")).count();
    assert!(taken > 0, "nobody was ever taken for combining");
    assert!(g.sanctioned.iter().any(|(e, ..)| members.contains(e)),
        "the charge never landed a sanction");
}
