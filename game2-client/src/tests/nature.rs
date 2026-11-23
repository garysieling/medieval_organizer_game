use crate::*;
use game2_core::sim::psychology::AttachmentStyle;

#[test]
fn test_card_describes_nature_in_plain_words_that_fit() {
    use AttachmentStyle::*;
    for st in [Secure, Anxious, Avoidant, Disorganized] {
        let word = nature_word(st);
        assert!(!word.contains(&format!("{:?}", st)), "{} still uses the jargon", word);
    }
    // every row the card emits - Nature's value wrap included - stays on
    // the plate: drawn at x+6, the card's right edge sits at 380
    let mut game = Game::new_seeded(MAP_SEED);
    game.selected_target = 0;
    let m = game.ui_model();
    let insp = m.inspector.as_ref().expect("a picked villager's card");
    assert!(insp.lines.iter().any(|l| l.text == "Nature:"), "no Nature row on the card");
    for l in &insp.lines {
        assert!(l.x + 6 + text_width(&l.text, 1) <= 380, "row runs off the card: {:?}", l.text);
    }
}
