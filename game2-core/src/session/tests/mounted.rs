use crate::game::villager::{Mounted, Pos};
use crate::game::vocation::{Craft, Vocation};
use crate::session::*;

/// The first knight on the map with a place to stand (a castle's
/// garrison or a village blade)
fn find_knight(g: &Game) -> Entity {
    g.villagers
        .iter()
        .copied()
        .find(|&v| {
            g.world
                .get_component::<Craft>(v)
                .map(|c| c.vocation == Vocation::Knight)
                .unwrap_or(false)
                && g.world.get_component::<Pos>(v).is_some()
        })
        .expect("the country fields knights")
}

fn find_vocation(g: &Game, voc: Vocation) -> Entity {
    g.villagers
        .iter()
        .copied()
        .find(|&v| {
            g.world
                .get_component::<Craft>(v)
                .map(|c| c.vocation == voc)
                .unwrap_or(false)
                && g.world.get_component::<Pos>(v).is_some()
        })
        .unwrap_or_else(|| panic!("no {:?} in the country", voc))
}

/// A village doorstep to ride to
fn village_tile(g: &Game) -> (i32, i32) {
    g.villagers
        .iter()
        .find(|&&v| g.site_of(v).is_none())
        .and_then(|&v| g.world.get_component::<Pos>(v))
        .map(|p| (p.x.round() as i32, p.y.round() as i32))
        .expect("a village doorstep")
}

#[test]
fn test_a_knight_takes_the_mount_for_the_long_road_and_leaves_it_at_the_end() {
    let mut g = Game::new_seeded(7);
    let k = find_knight(&g);
    let to = village_tile(&g);
    assert!(
        g.send_far(k, to, Goal::Wander),
        "no road rides to the village"
    );
    assert!(
        g.world.get_component::<Mounted>(k).is_some(),
        "the knight rides bare"
    );
    // and the public model says so, for the client to draw the mount
    let m = g.ui_model();
    assert!(
        m.people.iter().any(|p| p.e == k && p.mounted),
        "the model never says he's riding"
    );
    for _ in 0..20000 {
        g.advance_travels();
        if !g.travels.iter().any(|t| t.who == k) {
            break;
        }
    }
    assert!(g.travels.iter().all(|t| t.who != k), "the ride never ended");
    assert!(
        g.world.get_component::<Mounted>(k).is_none(),
        "still in the saddle at the door"
    );
}

#[test]
fn test_the_mount_nearly_doubles_the_stride() {
    // two copies of one seed: the same knight rides in one and walks
    // in the other - everything but the mount matches
    let mut rider = Game::new_seeded(7);
    let mut walker = Game::new_seeded(7);
    let k = find_knight(&rider);
    let to = village_tile(&rider);
    assert!(rider.send_far(k, to, Goal::Wander));
    assert!(walker.send_far(k, to, Goal::Wander));
    walker.world.remove_component::<Mounted>(k);
    let at = |g: &Game| {
        let p = g.world.get_component::<Pos>(k).unwrap();
        (p.x, p.y)
    };
    let (rx0, ry0) = at(&rider);
    let (wx0, wy0) = at(&walker);
    rider.advance_travels();
    walker.advance_travels();
    let (rx1, ry1) = at(&rider);
    let (wx1, wy1) = at(&walker);
    let rd = ((rx1 - rx0).powi(2) + (ry1 - ry0).powi(2)).sqrt();
    let wd = ((wx1 - wx0).powi(2) + (wy1 - wy0).powi(2)).sqrt();
    assert!(wd > 0.0, "the walker never stepped");
    assert!(
        rd > wd * 1.8 && rd < wd * 2.2,
        "the mount should ride at RIDE_PACE: {:.3} vs {:.3}",
        rd,
        wd
    );
}

#[test]
fn test_only_a_knight_on_a_country_road_takes_the_mount() {
    let mut g = Game::new_seeded(7);
    let to = village_tile(&g);
    // a farmer's long ride stays foot - the horse is the knight's
    let f = find_vocation(&g, Vocation::Farmer);
    assert!(g.send_far(f, to, Goal::Wander));
    assert!(
        g.world.get_component::<Mounted>(f).is_none(),
        "a farmer took a horse"
    );
    // and a knight's village errand stays foot too: the mount is for
    // the long roads, not the walk to a neighbour's door
    let k = find_knight(&g);
    // a neighbour under the same roof, the knight set down beside them:
    // the walk stays a doorstep's remove
    let site = g.site_of(k);
    let other = g
        .villagers
        .iter()
        .copied()
        .find(|&v| v != k && g.site_of(v) == site && g.world.get_component::<Pos>(v).is_some())
        .expect("a neighbour to visit");
    let op = *g.world.get_component::<Pos>(other).unwrap();
    let spot = [
        (2.0, 0.0),
        (-2.0, 0.0),
        (0.0, 2.0),
        (0.0, -2.0),
        (3.0, 0.0),
        (0.0, 3.0),
    ]
    .iter()
    .map(|&(dx, dy)| (op.x + dx, op.y + dy))
    .find(|&(x, y)| g.map.passable(x.round() as i32, y.round() as i32))
    .expect("no clear tile beside the neighbour");
    g.world.get_component_mut::<Pos>(k).unwrap().x = spot.0;
    g.world.get_component_mut::<Pos>(k).unwrap().y = spot.1;
    assert!(g.send(k, Goal::TalkTo(other)), "no path to the neighbour");
    assert!(
        g.world.get_component::<Mounted>(k).is_none(),
        "mounted for a doorstep"
    );
    // and the player never rides at all
    let p = g.seats[0].player;
    if g.world.get_component::<Pos>(p).is_some() && g.send_far(p, to, Goal::Wander) {
        assert!(
            g.world.get_component::<Mounted>(p).is_none(),
            "the player took a horse"
        );
    }
}
