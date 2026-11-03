use crate::session::*;

#[test]
fn test_every_button_hint_is_its_bound_key_and_none_move_the_player() {
    assert_eq!(action_buttons().len(), ACTION_KEYS.len(), "every action button needs a key");
    for (b, ch) in action_buttons().iter().zip(ACTION_KEYS.iter()) {
        assert_eq!(b.key, *ch, "{} shows key {}", b.label, b.key);
        assert!(!MOVE_KEYS.contains(ch), "{} uses {}, which moves the player", b.label, ch);
    }
}
