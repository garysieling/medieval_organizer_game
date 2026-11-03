use crate::session::*;

#[test]
fn test_overlapping_labels_are_stacked_apart() {
    // three names on the same spot (villagers sharing the market tile)
    let ys = place_labels(&[(100, 200, 36), (104, 202, 30), (98, 199, 30)]);
    for a in 0..3 { for b in (a + 1)..3 {
        assert!((ys[a] - ys[b]).abs() >= GLYPH_H + 3, "labels {} and {} still overlap: {:?}", a, b, ys);
    }}
}

#[test]
fn test_separate_labels_stay_where_they_are() {
    let ys = place_labels(&[(10, 100, 30), (200, 100, 30), (10, 300, 30)]);
    assert_eq!(ys, vec![100, 100, 300]);
}

#[test]
fn test_front_label_keeps_its_spot() {
    // the lower label (character in front) stays put; the other moves up
    let ys = place_labels(&[(100, 200, 36), (100, 205, 36)]);
    assert_eq!(ys[1], 205);
    assert!(ys[0] < 200);
}
