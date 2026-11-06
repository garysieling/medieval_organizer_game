use crate::session::*;

#[test]
fn test_the_guild_forms_of_real_traders_and_shields_its_own() {
    let mut g = Game::new();
    // a day turns; the traders band together
    stage::run_days(&mut g, 2);
    let guild = g.world.query::<crate::game::guilds::Guild>().first().copied()
        .expect("the guild never formed");
    let members = g.world.get_component::<crate::game::guilds::Guild>(guild)
        .unwrap().members.clone();
    assert!(!members.is_empty(), "the guild holds nobody");
    // the members are villagers, not phantoms
    assert!(members.iter().all(|m| g.villagers.contains(m)),
        "the guild holds strangers: {:?}", members);
    // and each carries the card's badge
    let i = g.villagers.iter().position(|v| *v == members[0]).unwrap();
    g.seats[0].selected_target = i;
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("of the guild")),
        "the badge's unsaid: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}
