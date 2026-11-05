use game2_core::session::{edge_scroll, EDGE_MARGIN, EDGE_SPEED, MAP_W, H, UI_X};

#[test]
fn test_the_map_view_edges_drift_and_the_middle_still_does() {
    // deep in a rim the drift is quickest; halfway into the margin,
    // half of that; the middle of the view doesn't move at all
    let (lx, _) = edge_scroll(2, H as i32 / 2);
    assert!(lx > EDGE_SPEED * 0.8, "near the left rim it should scroll hard, got {lx}");
    let (mx, _) = edge_scroll(EDGE_MARGIN / 2, H as i32 / 2);
    assert!((mx - EDGE_SPEED / 2.0).abs() < 0.01, "halfway in it should be half speed, got {mx}");
    assert_eq!(edge_scroll(MAP_W as i32 / 2, H as i32 / 2), (0.0, 0.0));
}

#[test]
fn test_the_drift_pushes_the_way_the_arrow_keys_do() {
    // +x pans the view left, -x right; +y up, -y down
    assert!(edge_scroll(0, H as i32 / 2).0 > 0.0);
    assert!(edge_scroll(MAP_W as i32 - 1, H as i32 / 2).0 < 0.0);
    assert!(edge_scroll(MAP_W as i32 / 2, 0).1 > 0.0);
    assert!(edge_scroll(MAP_W as i32 / 2, H as i32 - 1).1 < 0.0);
}

#[test]
fn test_the_side_panel_is_not_the_maps_edge() {
    // the UI column and find box sit past MAP_W: hovering them must not
    // scroll the map underneath
    assert_eq!(edge_scroll(UI_X as i32 + 5, 100), (0.0, 0.0));
    assert_eq!(edge_scroll(UI_X as i32 + 5, 800), (0.0, 0.0));
}
