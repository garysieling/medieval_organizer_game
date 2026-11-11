//! The generated icon set (assets/rd/icons, #691): every HUD, event and
//! goods icon loads small and transparent, the log's event words pick
//! their mark, and a missing set leaves the text glyphs in place.
use crate::*;

#[test]
fn test_icons_load_small_and_transparent() {
    let art = Art::load_fs("../assets", 0);
    assert_eq!(art.icons.len(), ICON_SPRITES.len(),
        "every generated icon loads: missing {:?}",
        ICON_SPRITES.iter().filter(|n| art.icon(n).is_none()).collect::<Vec<_>>());
    for (name, s) in &art.icons {
        assert!(s.width <= 64 && s.height <= 64, "{} is {}x{} - icons draw 10-20px", name, s.width, s.height);
        assert!(s.pixels.chunks(4).any(|p| p[3] == 0), "{}: background not keyed out", name);
    }
}

#[test]
fn test_event_lines_pick_their_icon() {
    assert_eq!(event_icon("t120: a blood moon - the moon hangs red over the village."), Some("event-blood-moon"));
    assert_eq!(event_icon("t4: the aurora - the sky burns red and green."), Some("event-aurora"));
    assert_eq!(event_icon("t9: a meteor shower - the sky rains stars."), Some("event-meteor-shower"));
    assert_eq!(event_icon("t33: a lost child - she's not come in for supper."), Some("event-lost-child"));
    // the cellar's own thief reads cellar, not the plain thief mark
    assert_eq!(event_icon("t55: the cellar thief struck again."), Some("event-cellar-thief"));
    assert_eq!(event_icon("t55: a thief made off with the till."), Some("event-thief"));
    assert_eq!(event_icon("t1: the miller baked a fine loaf."), None);
}

#[test]
fn test_marks_fall_back_to_glyphs_without_icons() {
    // rd/icons missing: every lookup is None and the text marks stand
    let fetch = |rel: &str| -> Option<Vec<u8>> {
        if rel.starts_with("rd/icons") { return None; }
        std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
    };
    let art = Art::load(&fetch, 0);
    assert!(art.icons.is_empty());

    let mut buf = Buffer::new(120, 60);
    draw_mark(&mut buf, art.icon("icon-sick"), "+", (140, 230, 110), 20, 20);
    let found = (0..60usize).any(|y| (0..120usize).any(|x| {
        let i = (y * 120 + x) * 4;
        (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (140, 230, 110)
    }));
    assert!(found, "no icon and no glyph - the sick mark drew nothing");
}

#[test]
fn test_a_loaded_icon_draws_its_pixels() {
    let art = Art::load_fs("../assets", 0);
    let icon = art.icon("icon-sick").expect("icon-sick should load");
    let mut buf = Buffer::new(40, 40);
    assert!(draw_icon(&mut buf, Some(icon), 10, 10, 14));
    // something opaque landed in the box
    let drawn = (10..24usize).any(|y| (10..24usize).any(|x| {
        buf.pixels[(y * 40 + x) * 4 + 3] > 200
    }));
    assert!(drawn, "icon-sick drew no pixels");
}
