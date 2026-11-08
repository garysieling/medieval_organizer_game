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

fn event(epoch: u64, x: f32, y: f32, extent: f32, moving: bool) -> EventFocusV {
    EventFocusV { epoch, kind: 0, pos: (x, y), extent, moving }
}

/// The event camera: a new event jumps and frames itself, a live one
/// tracks its centre, and a manual look holds it off for a spell
#[test]
fn test_the_event_camera_jumps_frames_and_yields() {
    let mut cam = EventCam::default();
    // no event: the camera is left alone
    assert_eq!(cam.step(None, (100, 100), 1.0), ((100, 100), 1.0));
    // a new event jumps to its centre and pulls the zoom out to fit
    let ev = event(1, 20.0, 20.0, 18.0, true);
    let ((cx, cy), z) = cam.step(Some(&ev), (100, 100), 1.0);
    assert_eq!((cx, cy), camera_on(20.0, 20.0), "a new event centres itself");
    assert!(z < 1.0, "a wide event zooms out to frame it: {z}");
    // the same event keeps its epoch: the camera tracks, eased - the
    // centre drifts toward the live pos, never a hard cut every frame
    let mut ev2 = ev;
    ev2.pos = (34.0, 12.0);
    let ((nx, ny), _) = cam.step(Some(&ev2), (cx, cy), z);
    let (tx, ty) = camera_on(34.0, 12.0);
    assert!((nx, ny) != (cx, cy), "a moving event's camera drifts toward it");
    let d0 = (((cx - tx).pow(2) + (cy - ty).pow(2)) as f32).sqrt();
    let d1 = (((nx - tx).pow(2) + (ny - ty).pow(2)) as f32).sqrt();
    assert!(d1 < d0, "it closes on the event: {d1} < {d0}");
    // a manual look holds the event camera off - pan wins for a spell
    cam.hold();
    let ((hx, hy), hz) = cam.step(Some(&ev2), (7, 8), 1.4);
    assert_eq!(((hx, hy), hz), ((7, 8), 1.4), "the player's look stands while held");
    // still held next frame: the ease re-seats where they left it
    let ((hx2, hy2), _) = cam.step(Some(&ev2), (9, 9), 1.4);
    assert_eq!((hx2, hy2), (9, 9));
    // and a NEW event always has the camera, hold or no hold
    let ev3 = event(2, 50.0, 50.0, 6.0, false);
    let ((jx, jy), _) = cam.step(Some(&ev3), (0, 0), 1.0);
    assert_eq!((jx, jy), camera_on(50.0, 50.0), "a new event jumps even mid-hold");
    // the event over: the camera is the player's again
    assert_eq!(cam.step(None, (3, 4), 0.9), ((3, 4), 0.9));
}

/// Zoom-to-fit only ever pulls back: a small gathering keeps the
/// player's zoom, a street-wide march pulls toward the floor
#[test]
fn test_event_zoom_only_ever_pulls_back() {
    assert_eq!(event_zoom(1.0, 1.0), 1.0, "a doorstep keeps the zoom");
    assert_eq!(event_zoom(1.0, 2.0), 2.0, "and never zooms in");
    let wide = event_zoom(40.0, 1.0);
    assert!(wide < 0.5 + f32::EPSILON || (wide - MIN_ZOOM).abs() < 0.01,
        "a street-wide event hits the floor: {wide}");
    let mid = event_zoom(12.0, 1.0);
    assert!(mid < 1.0 && mid > MIN_ZOOM, "a column pulls back but not to the floor: {mid}");
}
