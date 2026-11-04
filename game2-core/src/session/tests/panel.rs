use crate::session::*;
use crate::client::*;

#[test]
fn test_wrap_text_breaks_at_words() {
    let lines = wrap_text("t280: Bram tells Dagny - someone saw the lord's tax collector", 20);
    assert!(lines.iter().all(|l| l.chars().count() <= 20), "{:?}", lines);
    assert_eq!(lines.join(" "), "t280: Bram tells Dagny - someone saw the lord's tax collector");
    assert_eq!(lines[0], "t280: Bram tells");
}

#[test]
fn test_wrap_text_splits_overlong_words() {
    assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
    assert!(wrap_text("", 10).is_empty());
}

#[test]
fn test_gossip_about_the_guild_is_not_coloured_as_a_guild_event() {
    assert_eq!(log_colour("t1: The Merchant Guild formed!"), (255, 215, 0));
    assert_ne!(log_colour("t120: Corvin tells Aldric - the guild merchants grow bold"), (255, 215, 0));
    assert_eq!(log_colour("Eliza joined!"), (120, 255, 120));
}
