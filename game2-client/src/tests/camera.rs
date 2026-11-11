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
    let mut fresh = FollowCam::default();
    assert_eq!(camera_follow(&m, far, 1.0, &mut fresh), None);
    // but once they've walked off while moving, the camera goes back -
    // clean off the view is the one place a snap's still allowed
    let mut seen = FollowCam { seen: Some(((pos.0 - 0.4, pos.1), 1.0)), ease: None };
    assert_eq!(camera_follow(&m, far, 1.0, &mut seen), Some(camera_on(pos.0, pos.1)),
        "walking off the map brings the camera back to them");
    // stood still since: a look elsewhere is theirs to keep
    let mut still = FollowCam { seen: Some((pos, 1.0)), ease: None };
    assert_eq!(camera_follow(&m, far, 1.0, &mut still), None);
    // and the middle stretch never asks for anything
    let near = camera_on(pos.0, pos.1);
    let mut seen2 = FollowCam { seen: Some(((pos.0 - 0.4, pos.1), 1.0)), ease: None };
    assert_eq!(camera_follow(&m, near, 1.0, &mut seen2), None);
    // zooming them out of sight while they stand still counts as moving:
    // the camera's parked off to one side, they were in view at zoom 1
    let (cx, cy) = camera_on(pos.0, pos.1);
    let mut zoomed = FollowCam { seen: Some((pos, 1.0)), ease: None };
    assert_eq!(camera_follow(&m, (cx + 300, cy), 3.0, &mut zoomed),
        Some(camera_on(pos.0, pos.1)), "a zoom that loses them brings the camera back");
}

/// Walking to the frame's soft edge eases the camera toward a point a
/// few tiles ahead of their walk - never a hard cut, and once they're
/// back inside the box the camera's theirs again
#[test]
fn test_the_camera_eases_ahead_of_a_walk_at_the_soft_edge() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    let pos = m.people.iter().find(|p| p.kind == PKind::Player).expect("the seat has a player").pos;
    // parked so they're on the view but past the soft box's right edge
    let cam = camera_on(pos.0 - 6.0, pos.1 + 6.0);
    let (sx, sy) = iso_to_screen_cam(pos.0, pos.1, cam.0, cam.1, 1.0);
    assert!(sx > (MAP_W as f32 * 0.7) as i32 && sx < MAP_W as i32
        && sy > (H as f32 * 0.3) as i32 && sy < (H as f32 * 0.7) as i32,
        "the test parks them at the edge, not off the view: {sx},{sy}");
    // they were a step back: walking on, and the camera answers by easing
    let mut st = FollowCam { seen: Some(((pos.0 - 0.4, pos.1), 1.0)), ease: None };
    let next = camera_follow(&m, cam, 1.0, &mut st).expect("the edge crossing asks the camera");
    assert_ne!(next, cam, "the camera moves");
    assert_ne!(next, camera_on(pos.0, pos.1), "it eases - no hard cut to centre");
    // toward a point ahead of their walk, so they run into the frame
    let ahead = camera_on(pos.0 + 3.0, pos.1);
    let d0 = (((cam.0 - ahead.0).pow(2) + (cam.1 - ahead.1).pow(2)) as f32).sqrt();
    let d1 = (((next.0 - ahead.0).pow(2) + (next.1 - ahead.1).pow(2)) as f32).sqrt();
    assert!(d1 < d0, "it closes on the point ahead: {d1} < {d0}");
    // stood inside the box after: the camera's where they left it
    let near = camera_on(pos.0, pos.1);
    let mut st2 = FollowCam { seen: Some(((pos.0 - 0.4, pos.1), 1.0)), ease: None };
    assert_eq!(camera_follow(&m, near, 1.0, &mut st2), None,
        "inside the soft box the camera never fights a pan");
    // and a pan made mid-walk isn't yanked back next frame
    let mut st3 = FollowCam { seen: Some(((pos.0 - 0.4, pos.1), 1.0)), ease: Some((0.0, 0.0)) };
    let resumed = camera_follow(&m, cam, 1.0, &mut st3).expect("walking still asks");
    let from_cam = (cam.0 as f32 + (ahead.0 as f32 - cam.0 as f32) * 0.15,
        cam.1 as f32 + (ahead.1 as f32 - cam.1 as f32) * 0.15);
    assert!(((resumed.0 as f32 - from_cam.0).abs() < 2.0
        && (resumed.1 as f32 - from_cam.1).abs() < 2.0),
        "the ease seats where the camera is, not where it was left");
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
