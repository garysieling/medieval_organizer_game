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
