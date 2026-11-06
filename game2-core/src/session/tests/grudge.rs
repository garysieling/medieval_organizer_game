use crate::session::*;

#[test]
fn test_the_castles_grudge_lands_on_the_next_writ() {
    let mut g = Game::new();
    // a defiance sat on the books: the next writ lands double
    g.repression_debt = 1;
    g.edict_tick(80); // kind 0 - the grain levy
    assert_eq!(g.repression_debt, 0, "the debt wasn't spent");
    assert!(g.log.iter().any(|l| l.contains("Double, for the last defiance")),
        "the grudge didn't show: {:?}", g.log.last());
    // a clean slate lands a straight writ
    g.edict_tick(200); // 200 % 120 = 80, kind (200/120)%4 = 1 - the fee
    assert!(!g.log.iter().last().unwrap().contains("Double"),
        "a straight writ carried a grudge");
}
