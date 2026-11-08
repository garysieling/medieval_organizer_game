use crate::session::*;

#[test]
fn test_the_choice_comes_home() {
    let mut g = Game::new();
    let w = g.player;
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(), witness: w,
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    assert_eq!(g.dilemma_debts.len(), 1, "the debt never wrote");
    g.tick = g.dilemma_debts[0].due;
    g.dilemma_tick(g.tick);
    assert!(g.dilemma_debts.is_empty(), "the debt never landed");
    // and there's no virtue stat - the remembering's in the episodes
    assert!(g.log.iter().any(|l| l.contains("someone remembers")));
}
