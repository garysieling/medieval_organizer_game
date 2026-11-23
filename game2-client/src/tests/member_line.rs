use crate::*;

#[test]
fn test_member_line_reads_and_fits_the_card() {
    assert_eq!(members_line(&[]), "nobody yet - recruit!");
    assert_eq!(members_line(&["Dagny", "Bram"]), "Dagny, Bram");
    // past the width it stops at a whole name instead of mid-word
    let long = members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza", "Wulfric"]);
    assert!(long.ends_with(" ..") && long.len() <= MEMBERS_CHARS, "{}", long);
    // the roll draws on its own row under the label, ending inside the panel
    for line in [members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza"]), long] {
        assert!(UI_X as i32 + 18 + text_width(&line, 1) <= UI_X as i32 + 330,
            "{} runs off the card", line);
    }
}
