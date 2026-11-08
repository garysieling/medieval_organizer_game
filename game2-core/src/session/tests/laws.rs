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
        tax_share: 0.33, assembly_cap: 1, drink_levy: 0.30, writ_mult: 2.0 });
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
