use crate::session::*;
use crate::game::relationships::RelKind;
use super::fixtures::plan_ready_meeting;

#[test]
fn test_zoom_keeps_the_view_centre_still() {
    // whatever sits at the middle of the map view stays there as you zoom
    let (cx, cy) = (ZOOM_PIVOT.0 as i32, ZOOM_PIVOT.1 as i32);
    let (tx, ty) = (11.4f32, 11.4f32); // a tile near the view centre at zoom 1, camera at rest
    let at1 = iso_to_screen_cam(tx, ty, 0, 0, 1.0);
    for z in [0.5, 1.5, 2.0] {
        let at = iso_to_screen_cam(tx, ty, 0, 0, z);
        let (d1, dz) = ((at1.0 - cx).abs() + (at1.1 - cy).abs(), (at.0 - cx).abs() + (at.1 - cy).abs());
        assert!(dz as f32 <= d1 as f32 * z.max(1.0) + 2.0, "zoom {} moved the centre tile from {:?} to {:?}", z, at1, at);
    }
}

/// The camera's civic-event ask: a march the seat called reports its
/// column's live head; a house meeting it called reports its venue
#[test]
fn test_the_seat_s_events_reach_the_camera() {
    let mut g = Game::new();
    // the fight the march answers, same founding the formation tests set
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g.seats[0].ctx_actions = vec![CtxAct::March];
    g.ctx_act_at(0, 0);
    assert!(g.formation.is_some(), "no column formed");
    let ev = g.ui_model().player_event.expect("the march is the event");
    assert_eq!(ev.kind, 0, "a march");
    assert!(ev.moving, "the column keeps walking");
    assert!(ev.extent > 1.0, "a column is wider than a doorstep: {}", ev.extent);
    let epoch = ev.epoch;
    // the same march keeps its epoch while it runs - the client jumps
    // once, not every frame
    g.update();
    assert_eq!(g.ui_model().player_event.unwrap().epoch, epoch);
    // and the column's head is the pos it tracks
    let head = g.formation.as_ref().unwrap().members[0];
    let p = g.world.get_component::<Pos>(head).copied().unwrap();
    let ev = g.ui_model().player_event.unwrap();
    assert!((ev.pos.0 - p.x).abs() < 0.01 && (ev.pos.1 - p.y).abs() < 0.01,
        "the event tracks the head: {:?} vs {:?}", ev.pos, (p.x, p.y));
}

#[test]
fn test_a_house_meeting_the_seat_called_is_the_event() {
    let mut game = Game::new();
    let (eliza, bram) = (game.villagers[1], game.villagers[2]);
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] {
        *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y };
    }
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
    }
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship { from: game.player, other: eliza,
        kind: RelKind::Friend, strength: 0.7, interactions: 6 });
    plan_ready_meeting(&mut game, eliza, (18.0, 16.0), &[bram]);
    game.call_meeting();
    assert!(game.meeting.is_some(), "the meeting never got called");
    let m = game.meeting.as_ref().unwrap();
    let ev = game.ui_model().player_event.expect("the meeting is the event");
    assert_eq!(ev.kind, 1, "a meeting");
    assert!(!ev.moving, "the venue stands still");
    assert_eq!(ev.pos, (m.venue.0 as f32, m.venue.1 as f32), "the venue is the point");
    assert_eq!(ev.epoch, m.born * 4 + 1, "the meeting's own stamp");
}
