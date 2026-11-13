use crate::*;
use game2_core::engine::render::Buffer;

const BLUE: (u8, u8, u8) = (140, 205, 255);

fn blue_px(buf: &Buffer) -> Vec<(i32, i32)> {
    buf.pixels.chunks(4).enumerate()
        .filter(|(_, p)| (p[0], p[1], p[2]) == BLUE)
        .map(|(i, _)| ((i % buf.width) as i32, (i / buf.width) as i32))
        .collect()
}

#[test]
fn test_the_edge_arrow_points_the_way_out() {
    // a target off each side pins its arrow to the map's edge on that
    // side, tip aimed outward
    let (w, h) = (200i32, 120i32);
    for (sx, sy, edge) in [(-500, 60, "left"), (700, 60, "right"), (100, -500, "top"), (100, 700, "bottom")] {
        let mut buf = Buffer::new(w as usize, h as usize);
        draw_quest_arrow(&mut buf, sx, sy, w, h);
        let px = blue_px(&buf);
        assert!(!px.is_empty(), "{edge}: no arrow drawn");
        let near = px.iter().filter(|&&(x, y)| match edge {
            "left" => x <= 20, "right" => x >= w - 20,
            "top" => y <= 20, _ => y >= h - 20,
        }).count();
        assert_eq!(near, px.len(), "{edge}: arrow should pin to the {edge} edge, got {px:?}");
    }
}

#[test]
fn test_the_waypoint_dart_bobs_over_the_spot() {
    let mut buf = Buffer::new(60, 60);
    draw_quest_marker(&mut buf, 30, 40, 0);
    let px = blue_px(&buf);
    assert!(!px.is_empty(), "the dart painted nothing");
    let cy = px.iter().map(|&(_, y)| y).sum::<i32>() as f32 / px.len() as f32;
    assert!(cy < 40.0, "the dart hovers over the spot, not on it: {cy}");
}
