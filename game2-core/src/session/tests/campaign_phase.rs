use crate::session::*;

#[test]
fn test_the_campaign_keeps_its_seasons() {
    use crate::game::campaign::CampaignPhase;
    assert_eq!(CampaignPhase::Listening.step(), CampaignPhase::IssueCut);
    assert_eq!(CampaignPhase::IssueCut.step(), CampaignPhase::Research);
    assert_eq!(CampaignPhase::Research.step(), CampaignPhase::Action);
    assert_eq!(CampaignPhase::Action.step(), CampaignPhase::Evaluate);
    assert_eq!(CampaignPhase::Evaluate.step(), CampaignPhase::Listening);
}

#[test]
fn test_a_campaign_moves_when_its_yield_lands() {
    let mut g = Game::new();
    let org = g.org;
    let ce = crate::game::campaign::found_campaign(&mut g.world, org,
        &crate::game::campaign::IssueVote { winner: "the bread".into(), ballots: vec![] },
        crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority, 1);
    g.world.get_component_mut::<crate::game::campaign::Campaign>(ce)
        .map(|c| c.target = Some(g.villagers[0]));
    // listening until a target's named
    g.phase_tick(50);
    let phase = g.world.get_component::<crate::game::campaign::Campaign>(ce)
        .map(|c| c.phase).unwrap();
    assert_eq!(phase, crate::game::campaign::CampaignPhase::IssueCut,
        "a named target never cut the issue");
    // a dossier moves it to research
    g.world.get_component_mut::<crate::game::campaign::Campaign>(ce)
        .map(|c| c.dossier = Some(crate::game::campaign::Dossier {
            target: g.villagers[0], stance: "Oppose", stake: "the tax".into() }));
    g.phase_tick(51);
    assert_eq!(g.world.get_component::<crate::game::campaign::Campaign>(ce)
        .map(|c| c.phase).unwrap(), crate::game::campaign::CampaignPhase::Research,
        "the dossier never moved it");
}
