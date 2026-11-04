use crate::session::*;
use crate::client::*;

#[test]
fn test_fullness_reads_the_right_way_round() {
    // Hunger stores fullness: a low number is a hungry villager
    assert_eq!(fullness_word(95.0), "full");
    assert_eq!(fullness_word(24.0), "hungry");
    assert_eq!(fullness_word(5.0), "starving");
    assert_eq!(meter_colour(0.24), (230, 80, 70), "a hungry villager's bar should be red");
    assert_eq!(meter_colour(0.95), (90, 200, 90));
}

#[test]
fn test_activity_in_plain_words() {
    assert_eq!(activity_text(Activity::SeekingFood), "looking for food");
    assert_eq!(activity_text(Activity::Idle), "idle");
}
