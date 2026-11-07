use crate::*;

/// The collision band a plain name label always had: glyph plus gap
const NAME_H: i32 = GLYPH_H + 3;

#[test]
fn test_overlapping_labels_are_stacked_apart() {
    // three names on the same spot (villagers sharing the market tile)
    let ys = place_labels(&[(100, 200, 36, NAME_H), (104, 202, 30, NAME_H), (98, 199, 30, NAME_H)]);
    for a in 0..3 { for b in (a + 1)..3 {
        assert!((ys[a] - ys[b]).abs() >= NAME_H, "labels {} and {} still overlap: {:?}", a, b, ys);
    }}
}

#[test]
fn test_separate_labels_stay_where_they_are() {
    let ys = place_labels(&[(10, 100, 30, NAME_H), (200, 100, 30, NAME_H), (10, 300, 30, NAME_H)]);
    assert_eq!(ys, vec![100, 100, 300]);
}

#[test]
fn test_front_label_keeps_its_spot() {
    // the lower label (character in front) stays put; the other moves up
    let ys = place_labels(&[(100, 200, 36, NAME_H), (100, 205, 36, NAME_H)]);
    assert_eq!(ys[1], 205);
    assert!(ys[0] < 200);
}

#[test]
fn test_a_tag_taller_than_a_name_still_clears() {
    // a hearted name's tag is taller: it must move far enough up that
    // its hearts can't sit on the neighbour's name
    let zone = HEART_H + 1;
    let tall = GLYPH_H + 3 + zone;
    let ys = place_labels(&[(100, 200, 36, NAME_H), (100, 205 - zone, 39, tall)]);
    // the tall tag's bottom (y + tall) must clear the plain name's top
    assert!(ys[1] + tall <= ys[0] || ys[0] + NAME_H <= ys[1],
        "the tags still overlap: {:?}", ys);
}
