use crate::session::*;
use crate::client::*;

#[test]
fn test_status_line_names_things_and_fits() {
    assert_eq!(status_line("The Harvesters", 5, 0.0, Some(3)), "Harvesters 5 | Gold 0 | Merchant Guild 3");
    assert_eq!(status_line("The Harvesters", 0, 100.0, None), "Harvesters 0 | Gold 100 | no guild yet");
    // the widest it gets still fits the panel
    let widest = status_line("The Harvesters", 5, 9999.0, Some(99));
    assert!(UI_X as i32 + 10 + text_width(&widest, 1) <= W as i32 - 6, "{} runs off the panel", widest);
    assert!(UI_X as i32 + 70 + 120 + 6 + text_width("1.99/2.0 lead", 1) <= W as i32 - 4);
}
