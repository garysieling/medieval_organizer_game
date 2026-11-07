use crate::*;

/// The camera follows a character who walks clean off the map view -
/// and only while they move, so a deliberate look elsewhere stays
#[test]
fn test_the_camera_follows_a_character_who_walks_off_the_map() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    let p = m.people.iter().find(|p| p.kind == PKind::Player).expect("the seat has a player");
    let pos = p.pos;
    // parked a long walk away: the character is clean off the view
    let far = camera_on(pos.0 + 90.0, pos.1 + 90.0);
    // first sight of them just takes their bearings - no yank
    let mut fresh = None;
    assert_eq!(camera_follow(&m, far, 1.0, &mut fresh), None);
    // but once they've walked off while moving, the camera goes back
    let mut seen = Some(((pos.0 - 0.4, pos.1), 1.0));
    assert_eq!(camera_follow(&m, far, 1.0, &mut seen), Some(camera_on(pos.0, pos.1)),
        "walking off the map brings the camera back to them");
    // stood still since: a look elsewhere is theirs to keep
    let mut still = Some((pos, 1.0));
    assert_eq!(camera_follow(&m, far, 1.0, &mut still), None);
    // and on-screen never asks for anything
    let near = camera_on(pos.0, pos.1);
    let mut seen2 = Some(((pos.0 - 0.4, pos.1), 1.0));
    assert_eq!(camera_follow(&m, near, 1.0, &mut seen2), None);
    // zooming them out of sight while they stand still counts as moving:
    // the camera's parked off to one side, they were in view at zoom 1
    let (cx, cy) = camera_on(pos.0, pos.1);
    let mut zoomed = Some((pos, 1.0));
    assert_eq!(camera_follow(&m, (cx + 300, cy), 3.0, &mut zoomed),
        Some(camera_on(pos.0, pos.1)), "a zoom that loses them brings the camera back");
}
