use crate::session::*;

#[test]
fn test_the_choice_comes_home() {
    let mut g = Game::new();
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    assert_eq!(g.dilemma_debts.len(), 1, "the debt never wrote");
    assert_eq!(g.dilemma_debts[0].witness, g.player, "the answering pair of hands remembers");
    g.tick = g.dilemma_debts[0].due;
    g.dilemma_tick(g.tick);
    assert!(g.dilemma_debts.is_empty(), "the debt never landed");
    // and there's no virtue stat - the remembering's in the episodes
    assert!(g.log.iter().any(|l| l.contains("someone remembers")));
}

#[test]
fn test_the_remembering_lands_on_who_answered() {
    // the knot's global - one granary, one answer - but whose hands made
    // the pick is the answering seat's, not seat 0's (#962)
    let mut g = Game::new();
    let s1 = g.add_seat();
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(s1, 0);
    assert_eq!(g.dilemma_debts.len(), 1);
    assert_eq!(g.dilemma_debts[0].witness, g.seats[s1].player,
        "seat 0's body would carry a remembering it never made");
    assert_ne!(g.dilemma_debts[0].witness, g.seats[0].player);
}
