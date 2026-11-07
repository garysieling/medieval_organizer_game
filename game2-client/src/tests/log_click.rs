use crate::*;

/// A model carrying just the given log entries, nothing rolled off
fn log_model(entries: Vec<LogEntryV>) -> UiModel {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    m.log = entries;
    m.log_dropped = 0;
    m
}

fn entry(text: &str, at: Option<(f32, f32)>, who: Option<usize>) -> LogEntryV {
    LogEntryV { text: text.into(), at, who }
}

#[test]
fn test_a_log_line_with_a_spot_is_hit_where_it_is_drawn() {
    let view = View::new(Art::load_fs("../assets", 0));
    let m = log_model(vec![
        entry("t1: plain flavour", None, None),
        entry("t2: Bram died.", Some((3.0, 4.0)), Some(2)),
        entry("t3: fire on the meadow", Some((8.0, 9.0)), None),
    ]);
    // the LOG box's rows start at y 530 on a 12px pitch, x from UI_X+10;
    // each short line wraps to a single row
    let lx = UI_X as i32 + 10;
    assert_eq!(view.log_hit(&m, lx + 20, 516), None, "the LOG title hits nothing");
    assert_eq!(view.log_hit(&m, lx + 20, 530), None, "a plain line is inert");
    assert_eq!(view.log_hit(&m, lx + 20, 542), Some(1), "the death's line answers");
    assert_eq!(view.log_hit(&m, lx + 20, 554), Some(2), "the fire's line answers");
    assert_eq!(view.log_hit(&m, lx + 20, 566), None, "past the entries hits nothing");
    assert_eq!(view.log_hit(&m, 40, 542), None, "off the panel hits nothing");
}

#[test]
fn test_any_wrapped_row_of_a_log_line_counts() {
    let view = View::new(Art::load_fs("../assets", 0));
    // "t9:" wraps alone, then the 80-char word splits into 52+28: three
    // rows for the one entry - a click on any of them hits it
    let long = format!("t9: {}", "x".repeat(80));
    let m = log_model(vec![
        entry(&long, Some((1.0, 1.0)), None),
        entry("t10: a plain line", None, None),
    ]);
    let lx = UI_X as i32 + 10;
    assert_eq!(view.log_hit(&m, lx + 20, 530), Some(0), "the first wrapped row");
    assert_eq!(view.log_hit(&m, lx + 20, 542), Some(0), "and the second still hits it");
    assert_eq!(view.log_hit(&m, lx + 20, 554), Some(0), "and the third");
    assert_eq!(view.log_hit(&m, lx + 20, 566), None, "the plain line after is inert");
}

#[test]
fn test_the_earlier_lines_marker_is_not_a_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    let mut m = log_model(vec![entry("t1: only one line", Some((1.0, 1.0)), None)]);
    m.log_dropped = 40;
    // with lines rolled off, the entries lose their last row to the
    // "· · · N earlier lines" marker - and the marker itself is inert
    let lx = UI_X as i32 + 10;
    assert_eq!(view.log_hit(&m, lx + 20, 530), Some(0));
    assert_eq!(view.log_hit(&m, lx + 20, 542), None);
}
