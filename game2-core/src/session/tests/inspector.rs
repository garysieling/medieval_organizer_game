use crate::session::*;

#[test]
fn test_activity_in_plain_words() {
    assert_eq!(activity_text(Activity::SeekingFood), "looking for food");
    assert_eq!(activity_text(Activity::Idle), "idle");
}
