//! The log's named beats acted out (#1273): the harvest lift gathers
//! the org at the field, and the storm's pickers walk the strand -
//! the beat a log line names is a beat somebody walks.
use crate::session::*;
use crate::game::vocation::Kid;

#[test]
fn test_the_harvest_lift_gathers_the_org_at_the_field() {
    let mut g = Game::new();
    // grown members to share the lift - never a seat's own player
    let players: Vec<Entity> = g.seats.iter().map(|s| s.player).collect();
    let members: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<Kid>(v).is_none() && !players.contains(&v))
        .take(4).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = members.clone();
    // autumn, and the year's lift not yet taken
    g.world.get_component_mut::<crate::sim::climate::Climate>(g.climate).unwrap()
        .season = crate::sim::climate::Season::Autumn;
    g.harvest_year = u64::MAX;
    g.harvest_tick(g.tick);
    assert!(g.log.iter().any(|l| l.contains("takes the harvest in together")),
        "the beat still names itself in the log");
    // and the members are out in the fields for the day, ringed at the crop
    let staged: Vec<Entity> = members.iter().copied().filter(|&m|
        g.world.get_component::<routine::Routine>(m)
            .map(|r| r.spot == routine::Spot::Fields && r.mode == routine::Mode::Working
                && r.day == g.tick).unwrap_or(false)).collect();
    assert!(!staged.is_empty(), "somebody takes the field for the lift");
    // every slot's on the field's ground
    let field = g.crop_fields.iter()
        .find_map(|&f| g.world.get_component::<Pos>(f)).expect("a crop field");
    for &m in &staged {
        let r = g.world.get_component::<routine::Routine>(m).unwrap();
        assert!((r.anchor.0 - field.x).abs() + (r.anchor.1 - field.y).abs() < 6.0,
            "the lift keeps to the field: {:?}", r.anchor);
    }
}

#[test]
fn test_the_storms_leavings_walk_the_pickers_to_the_strand() {
    let mut g = Game::new();
    g.force_event(crate::session::events::EventId::StormWashUp);
    assert!(g.log.iter().any(|l| l.contains("a storm's leavings")),
        "the beat still names itself in the log");
    // the pickers' day is on the water: working routines at the strand
    let staged: Vec<(Entity, (f32, f32))> = g.villagers.iter().copied()
        .filter_map(|v| g.world.get_component::<routine::Routine>(v)
            .filter(|r| r.spot == routine::Spot::Creek && r.mode == routine::Mode::Working
                && r.day == g.tick)
            .map(|r| (v, r.anchor))).collect();
    assert!(!staged.is_empty(), "somebody walks the strand for the leavings");
    // every staged slot's a step off the water
    for &(v, a) in &staged {
        let (ax, ay) = (a.0.round() as i32, a.1.round() as i32);
        let lapping = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (dx, dy)))
            .any(|(dx, dy)| matches!(g.map.get(ax + dx, ay + dy),
                Some(crate::game::tilemap::Tile::Water) | Some(crate::game::tilemap::Tile::River)));
        assert!(lapping, "the strand's the shore: {:?} -> {:?}", v, a);
    }
}
