use crate::session::*;

#[test]
fn test_the_village_keeps_its_tune_in_the_model() {
    let g = Game::new();
    let m = g.ui_model();
    assert!(m.music.is_some(), "the tune never projected");
    let (mood, layers) = m.music.unwrap();
    assert!(!mood.is_empty() && !layers.is_empty(), "the tune's empty");
    // and its voices read medieval, not the techno bed it was
    assert!(layers.iter().all(|(n, _)| matches!(n.as_str(),
        "drone" | "pluck" | "choir" | "drum" | "bell")),
        "a techno layer leaked");
}

#[test]
fn test_a_cue_carries_where_it_fell() {
    let mut g = Game::new();
    g.sfx.push(SfxCue { kind: "bell".into(), x: 20.0, y: 30.0, intensity: 0.8 });
    let m = g.ui_model();
    assert!(m.sfx_v.iter().any(| (k, x, y, _i)| k == "bell" && *x == 20 && *y == 30),
        "the cue dropped its spot");
}
