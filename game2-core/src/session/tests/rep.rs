use crate::session::*;
use crate::sim::reputation_qsp::{RepAction, RepStage};

#[test]
fn test_the_cards_word_on_their_standing() {
    let mut g = Game::new();
    let i = 0;
    let v = g.villagers[i];
    g.seats[0].selected_target = i;
    // nobody says a word about them yet
    let insp = g.villager_inspector_at(0);
    assert!(!insp.lines.iter().any(|l| l.text.contains("thought of") || l.text.contains("ill-spoken")));
    // good works lift them to trusted
    for _ in 0..12 { g.rep_apply(v, RepAction::GoodDeed); }
    g.rep_apply(v, RepAction::KeptPromise);
    let stage = g.world.get_component::<crate::sim::reputation_qsp::ReputationState>(v).unwrap().stage;
    assert!(matches!(stage, RepStage::Trusted | RepStage::Esteemed), "{:?}", stage);
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("thought of") || l.text.contains("esteemed")),
        "the standing's unsaid: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
    // and betrayal writes them disgraced
    for _ in 0..4 { g.rep_apply(v, RepAction::Betrayal); }
    let stage = g.world.get_component::<crate::sim::reputation_qsp::ReputationState>(v).unwrap().stage;
    assert_eq!(stage, RepStage::Disgraced);
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("ill-spoken")),
        "the disgrace's unsaid");
}
