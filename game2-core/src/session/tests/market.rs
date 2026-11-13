use crate::session::*;
use crate::session::routine::{Mode, Routine, MARKET_DAY, STALL_TRADER};
use crate::game::villager::Pos;

fn walk_frames(g: &mut Game, n: u32) { for _ in 0..n { g.update(); } }

#[test]
fn test_market_day_puts_up_stalls_for_the_view() {
    let mut g = Game::new();
    while g.tick < MARKET_DAY { g.update(); }
    let n = g.ticks_per_frame() - 2; walk_frames(&mut g, n);
    let sellers = g.villagers.iter().filter(|&&v| g.world.get_component::<Routine>(v)
        .is_some_and(|r| r.mode == Mode::Selling && r.day == MARKET_DAY)).count();
    let stalls = g.stalls_standing();
    assert!(sellers > 0, "someone sells on market day");
    assert!(!stalls.is_empty(), "the sellers who reached their pitch get a stall ({} selling)", sellers);
    // each stall stands just in front of a seller's pitch, on the side
    // facing the square
    let mp = *g.world.get_component::<Pos>(g.market_food).unwrap();
    let d = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    for &(x, y, _) in &stalls {
        let pitch = g.villagers.iter().filter_map(|&v| g.world.get_component::<Routine>(v))
            .filter(|r| r.mode == Mode::Selling && r.day == MARKET_DAY)
            .map(|r| r.anchor).find(|&a| (d(a, (x, y)) - 0.6).abs() < 0.01);
        let Some(a) = pitch else { panic!("stall at ({:.1},{:.1}) isn't at a seller's pitch", x, y) };
        assert!(d((x, y), (mp.x, mp.y)) < d(a, (mp.x, mp.y)), "stall at ({:.1},{:.1}) faces away from the square", x, y);
    }
}

#[test]
fn test_the_pedlars_booth_comes_down_when_he_stops_selling() {
    let mut g = Game::new();
    g.spawn_pedlar();
    for _ in 0..400 {
        if g.pedlar.as_ref().map(|p| p.selling_until.is_some()).unwrap_or(false) { break; }
        g.advance_pedlar();
    }
    let until = g.pedlar.as_ref().and_then(|m| m.selling_until).expect("he pitches");
    assert!(g.stalls_standing().iter().any(|s| s.2 == STALL_TRADER), "his booth stands while he sells");
    // past his last selling day he's packing up: the booth is gone, though
    // his stall entity lasts until he's off the map
    g.tick = until + 1;
    assert!(g.pedlar.as_ref().and_then(|m| m.stall).is_some());
    assert!(!g.stalls_standing().iter().any(|s| s.2 == STALL_TRADER), "the booth comes down when he leaves");
}

#[test]
fn test_a_clicked_shop_sells_from_a_walk_over() {
    let mut g = Game::new();
    // a stocked shop with someone minding the till, and far enough from
    // where the player stands that the buy wants a walk
    let pe = g.seats[0].player;
    let (px, py) = g.player_pos_at(0);
    let bi = (0..g.building_entities.len()).find(|&i| {
        let b = g.building_entities[i];
        let far = g.world.get_component::<Pos>(b)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() > 8.0).unwrap_or(false);
        far && g.world.get_component::<Building>(b)
            .map(|bb| bb.worker.is_some()
                && !crate::game::household::on_offer_at(&g.world, bb).is_empty())
            .unwrap_or(false)
    }).expect("a far shop with an offer on the shelf");
    let b = g.building_entities[bi];
    g.seats[0].selected_building = Some(bi);
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Buy)
        .expect("a clicked shop with an offer puts up Buy");
    // far off: the click starts the walk, not the trade
    g.ctx_act_at(0, i);
    assert!(g.seats[0].buy_walk.is_some(), "the click starts the walk over");
    // beside the shelf: the walk lands one unit, and the purse feels it
    let bp = g.world.get_component::<Pos>(b).copied().unwrap();
    { let p = g.world.get_component_mut::<Pos>(pe).unwrap(); p.x = bp.x; p.y = bp.y; }
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 100.0;
    let (g0, _) = {
        let bb = g.world.get_component::<Building>(b).unwrap();
        let o = crate::game::household::on_offer_at(&g.world, bb);
        (o[0].0.clone(), o[0].2)
    };
    let gold0 = g.world.get_component::<Purse>(pe).unwrap().gold;
    g.advance_buy_walk_at(0);
    assert!(g.seats[0].buy_walk.is_none(), "the walk's done once the trade lands");
    assert!(g.world.get_component::<Purse>(pe).unwrap().gold < gold0, "the buy took coin");
    assert!(*g.world.get_component::<Satchel>(pe).unwrap().goods.get(&g0).unwrap_or(&0) > 0,
        "the bag holds the {}", g0);
}

#[test]
fn test_a_clicked_building_with_no_till_offers_no_buy() {
    let mut g = Game::new();
    // stock but no one minding it: the shelf shows "for offer" on the
    // card yet nobody can take your coin
    let bi = (0..g.building_entities.len()).find(|&i| {
        let b = g.building_entities[i];
        g.world.get_component::<Building>(b)
            .map(|bb| bb.worker.is_none()
                && !crate::game::household::on_offer_at(&g.world, bb).is_empty())
            .unwrap_or(false)
    });
    if let Some(bi) = bi {
        g.seats[0].selected_building = Some(bi);
        g.update_ctx_actions_at(0);
        assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Buy),
            "no keeper, no till - Buy stays off");
    }
}
