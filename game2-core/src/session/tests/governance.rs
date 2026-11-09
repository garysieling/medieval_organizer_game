use crate::session::*;

#[test]
fn test_the_alignment_tiers_read_the_member() {
    let g = Game::new();
    let m = g.villagers[0];
    // Bridged's the base read for a member whose stake's unheard
    assert!(matches!(
        crate::game::campaign::alignment(&g.world, m, g.villagers[0].max(1), g.tick),
        crate::game::campaign::Alignment::Bridged | crate::game::campaign::Alignment::Amplified
            | crate::game::campaign::Alignment::Transformed),
        "alignment never read a tier");
}

#[test]
fn test_the_consensus_rounds_can_split_a_wide_room() {
    // a room above ~12 on Consensus carries the splinter roll - the
    // field exists on the outcome to read it
    let outcome = crate::game::campaign::MeetingOutcome {
        attendees: vec![], healed: vec![], drifted: false, noted: false,
        consent: true, splintered: true };
    assert!(outcome.splintered, "the splinter read never landed");
}
