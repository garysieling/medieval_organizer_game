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
fn test_a_clicked_shop_walks_you_to_its_keeper() {
    let mut g = Game::new();
    g.pending_quests.clear(); // no job offer to crowd the till's picks
    // a stocked shop with someone minding the till, far enough from the
    // player that the click wants a walk, and nobody's sprite on the
    // click point
    let pe = g.seats[0].player;
    let (px, py) = g.player_pos_at(0);
    // figure box_d at a click point, matching click_map_at's hit test
    let fig_d = |g: &Game, mx: i32, my: i32| g.villagers.iter()
        .filter_map(|&v| g.world.get_component::<Pos>(v)
            .map(|p| { let (vx, vy) = iso_to_screen_cam(p.x, p.y, 0, 0, 1.0);
                let dx = ((mx - vx).abs() - 18).max(0) as f32;
                let dy = (if my <= vy { (vy - my) - 40 } else { (my - vy) - 12 }).max(0) as f32;
                (dx * dx + dy * dy).sqrt() }))
        .fold(f32::MAX, f32::min);
    // a stocked shop with someone minding the till, far enough from the
    // player that the click wants a walk - and a point inside its hit
    // radius that no figure's box covers (the keeper stands at the till,
    // so the click aims off-centre)
    let (bi, mx, my) = (0..g.buildings.len()).filter_map(|i| {
        let b = *g.building_entities.get(i)?;
        let far = g.world.get_component::<Pos>(b)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() > 8.0).unwrap_or(false);
        let sells = g.world.get_component::<Building>(b)
            .map(|bb| bb.worker.is_some()
                && !crate::game::household::on_offer_at(&g.world, bb).is_empty())
            .unwrap_or(false);
        if !far || !sells { return None; }
        let (bx, by, _) = g.buildings[i];
        let (sx, sy) = iso_to_screen_cam(bx, by, 0, 0, 1.0);
        for dx in (-38..=38).step_by(4) { for dy in (-16..=8).step_by(4) {
            let (mx, my) = (sx + dx, sy - 14 + dy);
            let bd = ((mx - sx).pow(2) + (my - (sy - 14)).pow(2)) as f32;
            if bd.sqrt() < 42.0 && fig_d(&g, mx, my) >= 4.0 { return Some((i, mx, my)); }
        }}
        None
    }).next().expect("a far shop with an offer on the shelf and a clear click point");
    g.click_map(mx, my, 0, 0, 1.0);
    let worker = g.world.get_component::<Building>(g.building_entities[bi]).unwrap().worker.unwrap();
    let wi = g.villagers.iter().position(|&v| v == worker).unwrap();
    assert!(g.seats[0].picked && g.seats[0].selected_target == wi,
        "the click picks the keeper, not the stones");
    assert!(g.seats[0].approach.is_some(), "a far keeper starts the walk over");
    assert!(g.seats[0].talk.is_none(), "no words pass until you're beside them");
    // beside them: the walk lands the talk, and their offer rides its picks
    let wp = g.world.get_component::<Pos>(worker).copied().unwrap();
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: wp.x + 1.0, y: wp.y };
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 100.0;
    g.advance_approach_at(0);
    let b = g.building_entities[bi];
    let good = { let bb = g.world.get_component::<Building>(b).unwrap();
        crate::game::household::on_offer_at(&g.world, bb)[0].0.clone() };
    let t = g.seats[0].talk.as_ref().expect("arriving beside them opens the talk");
    assert_eq!(t.who, worker, "the talk's with the keeper");
    assert!(!t.wares.is_empty() && t.wares[0] == good, "the till's offer leads the picks");
    g.talk_pick_at(0, Some(0));
    assert!(g.world.get_component::<Purse>(pe).unwrap().gold < 100.0, "the buy took coin");
    assert!(*g.world.get_component::<Satchel>(pe).unwrap().goods.get(&good).unwrap_or(&0) > 0,
        "the bag holds the {}", good);
}

#[test]
fn test_a_clicked_building_with_no_till_stays_a_building() {
    let mut g = Game::new();
    // stock but no one minding it: the click shows what the place is -
    // nobody walks over, nobody opens their mouth
    let bi = (0..g.buildings.len()).find(|&i| {
        let Some(&b) = g.building_entities.get(i) else { return false };
        g.world.get_component::<Building>(b)
            .map(|bb| bb.worker.is_none()
                && !crate::game::household::on_offer_at(&g.world, bb).is_empty())
            .unwrap_or(false)
    });
    if let Some(bi) = bi {
        let (bx, by, _) = g.buildings[bi];
        let (sx, sy) = iso_to_screen_cam(bx, by, 0, 0, 1.0);
        g.click_map(sx, sy - 14, 0, 0, 1.0);
        assert_eq!(g.seats[0].selected_building, Some(bi), "the building answers the click");
        assert!(g.seats[0].approach.is_none() && g.seats[0].talk.is_none(),
            "no keeper to walk to or talk with");
    }
}
