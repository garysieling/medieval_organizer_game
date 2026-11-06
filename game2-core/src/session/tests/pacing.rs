//! How fast the calendar runs: a day is ticks_per_frame frames, and the
//! watch speed is deliberately slow - the village has time to be seen
use crate::session::*;

#[test]
fn test_a_day_takes_its_time_on_the_watch() {
    let g = Game::new();
    // ~21s at 60fps: slow enough to watch folk go about their day
    assert_eq!(g.ticks_per_frame(), 21 * FPS);
    // turbo is the same pace, just quicker - still a real day
    let mut fast = Game::new();
    fast.speed = 4;
    assert_eq!(fast.ticks_per_frame(), 15 * FPS / 4);
    // and the meeting's half pace still applies on top
    fast.speed = 1;
    assert_eq!(fast.ticks_per_frame(), 21 * FPS);
}
