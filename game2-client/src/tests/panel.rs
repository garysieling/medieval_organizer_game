use crate::*;

#[test]
fn test_gossip_about_the_guild_is_not_coloured_as_a_guild_event() {
    assert_eq!(log_colour("t1: The Merchant Guild formed!"), (255, 215, 0));
    assert_ne!(log_colour("t120: Corvin tells Aldric - the guild merchants grow bold"), (255, 215, 0));
    assert_eq!(log_colour("Eliza joined!"), (120, 255, 120));
}

#[test]
fn test_the_org_blocks_hide_until_the_causes_founded() {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    // before the cause: no members, no roll, no issues block
    m.panel.members.clear();
    m.panel.issues.clear();
    assert!(!org_started(&m), "the org blocks drew before the cause");
    // once the cause's founded: the roll carries, the blocks draw
    m.panel.members = vec!["Watt".into()];
    assert!(org_started(&m), "the org blocks hid after the cause");
}
