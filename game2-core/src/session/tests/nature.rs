use crate::session::*;

#[test]
fn test_card_describes_nature_in_plain_words_that_fit() {
    use AttachmentStyle::*;
    for st in [Secure, Anxious, Avoidant, Disorganized] {
        let line = format!("Nature: {}", nature_word(st));
        assert!(!line.contains(&format!("{:?}", st)), "{} still uses the jargon", line);
        assert!(UI_X as i32 + 60 + text_width(&line, 1) <= UI_X as i32 + 278, "{} runs off the card", line);
    }
}
