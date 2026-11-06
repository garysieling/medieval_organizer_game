//! The walk cycle follows ground covered, not wall-clock frames: a
//! strolling amble and a runner stride at their own pace.

use crate::*;

#[test]
fn the_walk_cycle_follows_ground_covered() {
    let view = View::new(Art::load_fs("../assets", 0));
    // stood still: the phase holds
    let t0 = view.walk_t(7, 10.0, 10.0, 4);
    assert_eq!(t0, view.walk_t(7, 10.0, 10.0, 4));
    // a step forward advances it
    assert!(view.walk_t(7, 10.05, 10.0, 4) > t0);
    // and a longer step advances it further
    let a = view.walk_t(7, 10.10, 10.0, 4);
    let b = view.walk_t(7, 10.20, 10.0, 4);
    assert!(b > a);
    // each entity keeps its own beat: two walkers a tile apart, moved
    // the same, don't land on the same frame
    let _ = view.walk_t(8, 20.0, 20.0, 4);
    let _ = view.walk_t(9, 20.0, 20.0, 4);
    let wa = view.walk_t(8, 20.05, 20.0, 4);
    let wb = view.walk_t(9, 20.05, 20.0, 4);
    assert_ne!(wa, wb);
}
