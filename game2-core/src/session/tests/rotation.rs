use crate::session::*;

#[test]
fn test_the_fallow_restores_the_soil() {
    let mut g = Game::new();
    g.soil_quality = 0.4;
    g.rotation_tick(2 * 360 + 6); // a fallow year
    assert!(g.soil_quality > 0.4, "the fallow never restored: {}", g.soil_quality);
}

#[test]
fn test_the_crop_drains_the_soil() {
    let mut g = Game::new();
    g.soil_quality = 0.6;
    g.rotation_tick(6); // a cropping year
    assert!(g.soil_quality < 0.6, "the crop never drained: {}", g.soil_quality);
}

#[test]
fn test_the_springs_own_gap_reads_lean() {
    let mut g = Game::new();
    g.soil_quality = 0.3;
    g.rotation_tick(96); // the spring's own gap
    assert!(g.log.iter().any(|l| l.contains("the spring's own gap")),
        "the gap never logged");
}
