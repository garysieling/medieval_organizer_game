use crate::*;

#[test]
fn test_status_line_names_things_and_fits() {
    assert_eq!(status_line("The Harvesters", None, 5, 0.0, Some(3)), "Harvesters 5 | Gold 0 | Merchant Guild 3");
    assert_eq!(status_line("The Harvesters", None, 0, 100.0, None), "Harvesters 0 | Gold 100 | no guild yet");
    // a named head joins the line, and a short name always fits
    assert_eq!(status_line("The Harvesters", Some("you"), 5, 0.0, Some(3)),
        "Harvesters 5 led by you | Gold 0 | Merchant Guild 3");
    // a long successor's name clips before it can hide the guild count
    let long = status_line("The Harvesters", Some("Winifred"), 5, 9999.0, Some(99));
    assert!(long.len() <= 55, "the line must hold its margin: {:?}", long);
    assert!(long.contains("Guild 99"), "the guild count must survive: {:?}", long);
    // the widest it gets still fits the panel
    let widest = status_line("The Harvesters", None, 5, 9999.0, Some(99));
    assert!(UI_X as i32 + 10 + text_width(&widest, 1) <= W as i32 - 6, "{} runs off the panel", widest);
    assert!(UI_X as i32 + 70 + 120 + 6 + text_width("1.99/2.0 lead", 1) <= W as i32 - 4);
}

#[test]
fn test_the_powers_words_fit_the_sidebar() {
    // all three powers at once - the heaviest the line ever carries
    let words = vec![
        "grant: the taxes".to_string(),
        "2 knights".to_string(),
        "ban >2".to_string(),
    ];
    let line = org_status_line(&words);
    assert!(line.len() <= 55, "the powers' words outrun the sidebar: {:?}", line);
    assert!(line.contains("grant") && line.contains("knights") && line.contains("ban"),
        "a power went missing: {:?}", line);
}
