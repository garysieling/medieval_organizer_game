use crate::*;

#[test]
fn test_gossip_about_the_guild_is_not_coloured_as_a_guild_event() {
    assert_eq!(log_colour("t1: The Merchant Guild formed!"), (255, 215, 0));
    assert_ne!(log_colour("t120: Corvin tells Aldric - the guild merchants grow bold"), (255, 215, 0));
    assert_eq!(log_colour("Eliza joined!"), (120, 255, 120));
}
