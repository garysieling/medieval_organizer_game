//! The walk cycle follows ground covered, not wall-clock frames: a
//! strolling amble and a runner stride at their own pace.

use crate::*;

#[test]
fn the_walk_cycle_follows_ground_covered() {
    let view = View::new(Art::load_fs("../assets", 0));
    // stood still: the phase holds
    let t0 = view.walk_t(7, 10.0, 10.0, 4, 100);
    assert_eq!(t0, view.walk_t(7, 10.0, 10.0, 4, 100));
    // a step forward advances it
    assert!(view.walk_t(7, 10.05, 10.0, 4, 101) > t0);
    // and a longer step advances it further
    let a = view.walk_t(7, 10.10, 10.0, 4, 102);
    let b = view.walk_t(7, 10.20, 10.0, 4, 103);
    assert!(b > a);
    // each entity keeps its own beat: two walkers moved the same
    // don't land on the same frame
    let _ = view.walk_t(8, 20.0, 20.0, 4, 100);
    let _ = view.walk_t(9, 20.0, 20.0, 4, 100);
    let wa = view.walk_t(8, 20.05, 20.0, 4, 101);
    let wb = view.walk_t(9, 20.05, 20.0, 4, 101);
    assert_ne!(wa, wb);
}

#[test]
fn skipped_frames_still_count_and_jumps_dont() {
    let view = View::new(Art::load_fs("../assets", 0));
    // a recording renders every fifth sim frame: five frames' walk is
    // ordinary ground, not a clampable step
    let seed = view.walk_t(3, 5.0, 5.0, 4, 0);
    assert!(view.walk_t(3, 5.2, 5.0, 4, 5) > seed);
    // a jump across the map in a single frame is a teleport: the legs
    // don't spin for it
    let tp = view.walk_t(4, 50.0, 50.0, 4, 0);
    assert_eq!(tp, view.walk_t(4, 55.0, 50.0, 4, 1));
}
