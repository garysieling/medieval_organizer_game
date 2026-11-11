use crate::*;

/// The camera leans after a character who wanders out of the view's
/// comfort box - and only while they move, so a deliberate look
/// elsewhere stays
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
    // but once they've walked on while moving, the camera leans back
    // after them - a share of the way, easing, never a snap
    let mut seen = Some(((pos.0 - 0.4, pos.1), 1.0));
    let next = camera_follow(&m, far, 1.0, &mut seen)
        .expect("walking on asks the camera after them");
    let (tx, ty) = camera_on(pos.0, pos.1);
    let before = ((far.0 - tx).pow(2) + (far.1 - ty).pow(2)) as f32;
    let after = ((next.0 - tx).pow(2) + (next.1 - ty).pow(2)) as f32;
    assert!(after < before, "the lean closes on them");
    assert_ne!(next, (tx, ty), "a share of the gap, not the snap");
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
    let znext = camera_follow(&m, (cx + 300, cy), 3.0, &mut zoomed)
        .expect("a zoom that loses them leans the camera back");
    let zbefore = ((cx + 300 - tx).pow(2) + (cy - ty).pow(2)) as f32;
    let zafter = ((znext.0 - tx).pow(2) + (znext.1 - ty).pow(2)) as f32;
    assert!(zafter < zbefore, "the zoomed-out lean closes on them");
}

/// The comfort box: inside the view's middle they wander free; at the
/// rim the camera starts leaning - and the lean opens the way their
/// stride points, not just back to centre
#[test]
fn test_the_comfort_box_opens_the_way_theyre_going() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    let p = m.people.iter().find(|p| p.kind == PKind::Player).expect("the seat has a player");
    let pos = p.pos;
    // just inside the box's x rim, walking out of it: the camera leans
    let rim = camera_on(pos.0 - 8.0, pos.1);
    let mut walking = Some(((pos.0 - 0.3, pos.1), 1.0));
    let next = camera_follow(&m, rim, 1.0, &mut walking)
        .expect("a stride out of the box asks the camera after them");
    // the lead's down their stride: walking +x, the camera's aim sits
    // past them in the frame (cam.x runs the other way on +x tiles),
    // and the lean heads after it - not merely back to them
    let on_them = camera_on(pos.0, pos.1);
    let lead = camera_on(pos.0 + 0.3 * 45.0, pos.1);
    assert!(lead.0 < on_them.0, "a +x stride's lead looks past them");
    assert!(next.0 < rim.0, "the lean heads after it");
    // and dead centre, the same stride asks nothing at all
    let mut centre = Some(((pos.0 - 0.3, pos.1), 1.0));
    assert_eq!(camera_follow(&m, camera_on(pos.0, pos.1), 1.0, &mut centre), None);
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
