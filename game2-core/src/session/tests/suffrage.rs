use crate::session::*;

#[test]
fn test_the_village_knows_what_bodies_it_gave() {
    let g = Game::new();
    let (women, men): (usize, usize) = g.villagers.iter()
        .fold((0, 0), |(w, m), &v| match g.world.get_component::<villager::Sex>(v) {
            Some(villager::Sex::Female) => (w + 1, m),
            Some(villager::Sex::Male) => (w, m + 1),
            None => (w, m),
        });
    assert!(women > 5 && men > 5, "the village isn't half of each: {} women, {} men", women, men);
}

#[test]
fn test_a_womans_seat_is_unasked_until_the_vote() {
    let mut g = Game::new();
    let org = g.org;
    let leader = g.world.get_component::<Organization>(org).unwrap().leader;
    // a woman leads the org
    g.world.add_component(leader, villager::Sex::Female);
    assert!(!g.suffrage_won, "the vote's already won");
    // her banner doesn't fly while the say goes unasked
    let vote = crate::game::campaign::IssueVote { winner: "the taxes".into(), ballots: vec![] };
    let got = g.found_and_note(0, &vote,
        crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority);
    assert!(got.contains("unasked"), "a woman's banner flew for taxes: {}", got);
    assert!(!g.suffrage_won);
}

#[test]
fn test_the_vote_won_opens_the_seats() {
    let mut g = Game::new();
    let org = g.org;
    let leader = g.world.get_component::<Organization>(org).unwrap().leader;
    g.world.add_component(leader, villager::Sex::Female);
    // the suffrage question itself gets carried - the room won't refuse
    // the vote because it's the vote that's being asked
    let vote = crate::game::campaign::IssueVote { winner: "the vote".into(), ballots: vec![] };
    let got = g.found_and_note(0, &vote,
        crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority);
    assert!(g.suffrage_won, "the suffrage banner flew but the vote didn't carry: {}", got);
    // and after, a woman's banner flies for any cause
    g.world.remove_component::<crate::game::campaign::Campaign>(
        crate::game::campaign::campaign_of(&g.world, org).unwrap());
    let vote2 = crate::game::campaign::IssueVote { winner: "the taxes".into(), ballots: vec![] };
    let got2 = g.found_and_note(0, &vote2,
        crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority);
    assert!(!got2.contains("unasked"), "the vote's won and her banner still won't fly: {}", got2);
}
