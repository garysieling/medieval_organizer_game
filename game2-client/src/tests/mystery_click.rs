use crate::*;

fn mystery_model(mz: Option<MysteryV>) -> UiModel {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    m.panel.mystery = mz;
    m
}

#[test]
fn test_only_the_askers_name_is_a_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    let m = mystery_model(Some(MysteryV {
        title: "who keeps leaving gifts".into(), heard: 3, total: 5,
        asker_name: "Matilda".into(), asker: Some(7), asker_at: Some((10.0, 20.0)),
    }));
    // "Mystery: who keeps leaving gifts - 3/5 whispers (Matilda wants to
    // know)" wraps at 30 chars: row 1 is "gifts - 3/5 whispers (Matilda"
    // with the name at cols 22..29, drawn at UI_X+150 on an 11px pitch
    // from y 68 - the same rows mystery_hit measures
    let x = UI_X as i32 + 150;
    let name_x = x + 22 * GLYPH_W;
    assert_eq!(view.mystery_hit(&m, name_x + 2, 79), Some(7), "the name answers");
    assert_eq!(view.mystery_hit(&m, name_x + 6 * GLYPH_W - 2, 79), Some(7),
        "its last cell too");
    assert_eq!(view.mystery_hit(&m, x + 5 * GLYPH_W, 79), None,
        "the rest of the line is inert");
    assert_eq!(view.mystery_hit(&m, name_x + 2, 68), None, "row 0 holds no name");
    assert_eq!(view.mystery_hit(&m, name_x + 2, 91), None, "row 2 is past the name");
    assert_eq!(view.mystery_hit(&m, 40, 79), None, "off the panel hits nothing");
}

#[test]
fn test_a_legacy_or_askerless_mystery_has_no_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    // a bare-string entry from an older server: the line draws, nothing clicks
    let m = mystery_model(Some(MysteryV {
        title: String::new(), heard: 0, total: 0,
        asker_name: "Mystery: whatever - 1/5 whispers (Matilda wants to know)".into(),
        asker: None, asker_at: None,
    }));
    assert_eq!(view.mystery_hit(&m, UI_X as i32 + 200, 68), None);
    // and a structured one whose asker has left the roll
    let m2 = mystery_model(Some(MysteryV {
        title: "who".into(), heard: 1, total: 5,
        asker_name: "Matilda".into(), asker: None, asker_at: None,
    }));
    assert_eq!(view.mystery_hit(&m2, UI_X as i32 + 200, 68), None);
    // no mystery at all
    let m3 = mystery_model(None);
    assert_eq!(view.mystery_hit(&m3, UI_X as i32 + 200, 68), None);
}
