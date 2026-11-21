//! The watch's pose stills (#1279): a gate guard stands easy by
//! default, salutes a knight or the player passing under him, and
//! stands ready while there's danger in town - a levy on, a column
//! out, posted knights on a crackdown.

use crate::session::*;

fn guard_poses(g: &Game) -> Vec<u8> {
    g.ui_model().people.iter()
        .filter(|p| matches!(p.kind, PKind::Guard(_)))
        .map(|p| p.pose).collect()
}

#[test]
fn test_the_watch_stands_easy_by_default() {
    let mut g = Game::new();
    // the player far from every gate: nobody to salute
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().x = 1.0;
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().y = 1.0;
    let poses = guard_poses(&g);
    assert!(!poses.is_empty(), "the gates draw a watch");
    assert!(poses.iter().all(|&p| p == model::guard_pose::STANDING),
        "a quiet day: the watch stands easy - {:?}", poses);
}

#[test]
fn test_the_watch_salutes_the_player_passing() {
    let mut g = Game::new();
    let gate = g.guards[0];
    let p = g.seats[0].player;
    g.world.get_component_mut::<Pos>(p).unwrap().x = gate.0 + 1.0;
    g.world.get_component_mut::<Pos>(p).unwrap().y = gate.1;
    let m = g.ui_model();
    let saluting: Vec<u8> = m.people.iter()
        .filter(|pv| matches!(pv.kind, PKind::Guard(_)))
        .map(|pv| pv.pose).collect();
    assert_eq!(saluting[0], model::guard_pose::SALUTING,
        "the gate's man salutes the one passing under him - {:?}", saluting);
    // and a guard across town still just stands
    let far = m.people.iter()
        .filter(|pv| matches!(pv.kind, PKind::Guard(_)))
        .filter(|pv| (pv.pos.0 - gate.0).abs() + (pv.pos.1 - gate.1).abs() > 9.0)
        .map(|pv| pv.pose).collect::<Vec<_>>();
    assert!(far.iter().all(|&p| p == model::guard_pose::STANDING),
        "the far watch isn't stirred - {:?}", far);
}

#[test]
fn test_the_watch_stands_ready_while_the_levys_on() {
    let mut g = Game::new();
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().x = 1.0;
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().y = 1.0;
    g.imperial_project = Some(ImperialProject { until: g.tick + 10, rate: 6, eased_until: 0 });
    let poses = guard_poses(&g);
    assert!(!poses.is_empty());
    assert!(poses.iter().all(|&p| p == model::guard_pose::READY),
        "a levy on: the watch stands ready - {:?}", poses);
}

#[test]
fn test_posted_knights_always_read_ready() {
    let mut g = Game::new();
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().x = 1.0;
    g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap().y = 1.0;
    // the crackdown's knights stand in the streets on no other warning
    let (px, py) = g.player_pos_at(0);
    g.posted.push((g.seats[0].org, px + 20.0, py + 20.0));
    let poses = guard_poses(&g);
    assert_eq!(poses.len(), g.guards.len() + 1);
    assert_eq!(*poses.last().unwrap(), model::guard_pose::READY,
        "posted means posted - {:?}", poses);
    assert!(poses[..poses.len() - 1].iter().all(|&p| p == model::guard_pose::STANDING),
        "the gate watch still just stands - {:?}", poses);
}
