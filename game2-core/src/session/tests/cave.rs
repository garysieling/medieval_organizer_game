use crate::session::*;

#[test]
fn test_the_monochrome_under_sits_a_few_tiles_round_the_mouth() {
    let g = Game::new();
    let Some((mx, my)) = g.cave_pos else { panic!("no cave mouth") };
    let Some(((ax, ay), (bx, by))) = g.cave_zone else { panic!("no cave zone") };
    // a small dark: a few tiles each way, the mouth inside its own bounds
    assert!(ax <= mx && mx <= bx && ay <= my && my <= by,
        "the mouth {:?} falls outside the zone", (mx, my));
    assert!((bx - ax) <= 8 && (by - ay) <= 8, "the zone sprawls");
}

#[test]
fn test_the_zone_projects_to_the_wire_for_the_tint_pass() {
    let g = Game::new();
    let m = g.ui_model();
    assert!(m.cave_zone.is_some(), "the zone never reached the model");
}
