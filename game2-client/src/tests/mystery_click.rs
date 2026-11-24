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
    let mz = MysteryV {
        title: "who keeps leaving gifts".into(), heard: 3, total: 5,
        dims: vec![],
        asker_name: "Matilda".into(), asker: Some(7), asker_at: Some((10.0, 20.0)),
    };
    let m = mystery_model(Some(mz.clone()));
    // the name's own pixels answer - mystery_hit reads the same span the
    // draw measures, on a LINE_H pitch from y 90
    let (row, x0, x1) = crate::view::mystery_name_span(&mz).unwrap();
    let y = 90 + row as i32 * LINE_H + 2;
    assert_eq!(view.mystery_hit(&m, x0 + 2, y), Some(7), "the name answers");
    assert_eq!(view.mystery_hit(&m, x1 - 1, y), Some(7), "its last cell too");
    assert_eq!(view.mystery_hit(&m, UI_X as i32 + 150, y), None,
        "the rest of the line is inert");
    assert_eq!(view.mystery_hit(&m, x0 + 2, 90 + 3 * LINE_H + 2), None,
        "a nameless row hits nothing");
    assert_eq!(view.mystery_hit(&m, 40, y), None, "off the panel hits nothing");
}

#[test]
fn test_a_legacy_or_askerless_mystery_has_no_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    // a bare-string entry from an older server: the line draws, nothing clicks
    let m = mystery_model(Some(MysteryV {
        title: String::new(), heard: 0, total: 0, dims: vec![],
        asker_name: "Mystery: whatever - 1/5 whispers (Matilda wants to know)".into(),
        asker: None, asker_at: None,
    }));
    assert_eq!(view.mystery_hit(&m, UI_X as i32 + 200, 90), None);
    // and a structured one whose asker has left the roll
    let m2 = mystery_model(Some(MysteryV {
        title: "who".into(), heard: 1, total: 5, dims: vec![],
        asker_name: "Matilda".into(), asker: None, asker_at: None,
    }));
    assert_eq!(view.mystery_hit(&m2, UI_X as i32 + 200, 90), None);
    // no mystery at all
    let m3 = mystery_model(None);
    assert_eq!(view.mystery_hit(&m3, UI_X as i32 + 200, 90), None);
}
