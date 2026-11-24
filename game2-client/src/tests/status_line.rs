use crate::*;

#[test]
fn test_status_line_names_things_and_fits() {
    assert_eq!(org_line("The Harvesters", None, 5), "Harvesters 5");
    assert_eq!(org_line("The Harvesters", Some("you"), 5), "Harvesters 5 led by you");
    assert_eq!(purse_line(0.0, Some(3)), "0g | Merchant Guild 3");
    assert_eq!(purse_line(100.0, None), "100g | no guild yet");
    // a long successor's name clips before it can hide the count
    let long = org_line("The Harvesters", Some("Winifred"), 5);
    assert!(long.len() <= 29, "the line must hold its margin: {:?}", long);
    // the widest the header rows get still fits the panel
    let widest = purse_line(9999.0, Some(99));
    assert!(UI_X as i32 + 10 + text_width(&widest, 1) <= W as i32 - 6, "{} runs off the panel", widest);
    assert!(UI_X as i32 + 10 + text_width(&org_line("The Harvesters", Some("Winifred"), 5), 1) <= W as i32 - 6);
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
    // the panel wraps the line into the two rows the draw path gives it
    for l in wrap_px(&line, 300) {
        assert!(UI_X as i32 + 10 + text_width(&l, 1) <= W as i32 - 6,
            "the powers' words outrun the sidebar: {:?}", l);
    }
    assert!(line.contains("grant") && line.contains("knights") && line.contains("ban"),
        "a power went missing: {:?}", line);
}
