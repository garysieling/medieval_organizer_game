use crate::session::*;

/// An office at every town and castle, a rider to keep it - and word
/// for the country goes with him, not the village's own week
#[test]
fn test_the_post_rides_and_the_village_keeps_its_own_week() {
    let mut g = Game::new();
    // offices at the settled places, riders at their doors
    let offices = g.post_offices.len();
    assert!(offices >= 2, "no offices raised");
    assert_eq!(g.couriers.len(), offices, "a rider to an office");
    // a villager writes to a townman: the post stamps it and the fee moves
    let from = g.villagers[0];
    let to = g.villagers.iter().copied()
        .find(|&v| g.site_of(v).is_some()).expect("no countryman");
    let si = g.site_of(to).unwrap();
    if let Some(p) = g.world.get_component_mut::<Purse>(from) { p.gold = 10.0; }
    let gold0 = g.world.get_component::<Purse>(from).map(|p| p.gold);
    g.letters.push(Letter { from, to, office: None, courier: None, due: 1 });
    g.post_tick(1);
    let l = g.letters[0];
    assert!(l.office.is_some(), "no office stamped it");
    let gold1 = g.world.get_component::<Purse>(from).map(|p| p.gold);
    assert_eq!(gold0.map(|g0| g0 - 1.0), gold1, "the penny didn't move");
    // and a rider takes the road: his Pos leaves the office door
    let ci = g.couriers.iter().position(|c| c.office == l.office.unwrap())
        .expect("no rider for that office");
    let e = g.couriers[ci].e;
    let p0 = g.world.get_component::<Pos>(e).copied().unwrap();
    for t in 2..8 { g.post_tick(t); }
    assert!(g.couriers.iter().any(|c| c.on_road), "nobody rode out");
    // walk him: the road moves the courier, not just a tick counter
    for _ in 0..400 { g.update(); }
    let p1 = g.world.get_component::<Pos>(e).copied().unwrap();
    let moved = (p1.x - p0.x).abs() + (p1.y - p0.y).abs();
    assert!(moved > 1.0 || g.letters.is_empty() || g.letters.iter().all(|l| l.courier.is_some()),
        "the rider never left: moved {}, si {}", moved, si);
}

/// Village-internal word keeps its abstract week; a letter to the dead
/// comes back on the next leg
#[test]
fn test_the_village_post_keeps_its_week_and_the_dead_send_word_back() {
    let mut g = Game::new();
    let from = g.villagers[0];
    let to = g.villagers[1];
    g.letters.push(Letter { from, to, office: None, courier: None, due: 5 });
    g.post_tick(1);
    assert!(g.letters[0].office.is_none(), "the post took a village letter");
    // the dead: the word comes back to the writer
    let dead = g.villagers[2];
    let si = g.post_offices[0].0;
    g.world.add_component(dead, country::Outlander { site: si });
    g.world.add_component(dead, villager::Dead);
    g.letters.push(Letter { from, to: dead, office: None, courier: None, due: 1 });
    g.post_tick(1);
    let l = g.letters.iter().find(|l| l.to == dead).copied().expect("no letter");
    assert!(l.office.is_some(), "the country letter wasn't stamped");
    // the courier carries it; when he lands at the site it turns for home
    let ci = g.couriers.iter().position(|c| c.office == l.office.unwrap()).unwrap();
    let e = g.couriers[ci].e;
    let dep = g.couriers[ci].depart;
    g.post_tick(dep); // his day: the bag loads and the road takes him
    assert_eq!(g.letters.iter().find(|l| l.to == dead).unwrap().courier, Some(e),
        "the letter never made the bag");
    g.arrive_post(e, si);
    assert!(g.letters.iter().any(|l| l.from == dead && l.to == from),
        "the word never came back");
    assert!(g.log.iter().any(|s| s.contains("comes back")), "{:?}", g.log.last());
}
