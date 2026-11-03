use crate::session::*;

#[test]
fn test_bubbles_stay_inside_the_map_view() {
    let text = "the lord's taxes are heavy";
    // near the right edge: pulled left so it doesn't spill into the panel
    let (bx, _, bw) = bubble_rect(MAP_W as i32 - 10, 300, 1.0, text, None).unwrap();
    assert!(bx + bw <= MAP_W as i32 - 4, "bubble runs into the panel: {}..{}", bx, bx + bw);
    // near the left and top edges: pushed back on screen
    let (bx, by, _) = bubble_rect(5, 20, 1.0, text, None).unwrap();
    assert!(bx >= 4 && by >= 4);
    // a speaker under the panel or off screen gets no bubble at all
    assert_eq!(bubble_rect(MAP_W as i32 + 40, 300, 1.0, text, None), None);
    assert_eq!(bubble_rect(-30, 300, 1.0, text, None), None);
}

#[test]
fn test_bubble_clears_the_name_label_at_any_zoom() {
    for z in [0.5f32, 1.0, 2.0] {
        let (_, by, _) = bubble_rect(300, 400, z, "hello", None).unwrap();
        let (_, label_y) = name_label_pos(300, label_anchor(400, z), "hello");
        assert!(by + BUBBLE_H <= label_y, "zoom {}: bubble (bottom {}) overlaps the name label (top {})", z, by + BUBBLE_H, label_y);
    }
}

#[test]
fn test_bubble_clears_a_label_nudged_up_by_the_crowd() {
    // place_labels can lift a crowded name up to 4 lines (40px); the
    // bubble used to sit 8px above the un-nudged spot, right on top of it
    let (_, home) = name_label_pos(300, label_anchor(400, 1.0), "Bram");
    for lift in [10, 20, 40] {
        let nudged = home - lift;
        let (_, by, _) = bubble_rect(300, 400, 1.0, "the lord's taxes are heavy", Some(nudged)).unwrap();
        // the bubble and its 4px tail end above the label
        assert!(by + BUBBLE_H + 4 <= nudged, "lift {}: bubble bottom {} covers the label at {}", lift, by + BUBBLE_H + 4, nudged);
    }
}
