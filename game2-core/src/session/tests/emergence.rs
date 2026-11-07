use crate::session::*;
use crate::sim::emergence::Cell;

#[test]
fn test_the_wood_grows_burns_and_leans_the_fuel_price() {
    let mut g = Game::new();
    // the stand spreads over the days
    let t0 = g.forest.cells.iter().flatten().filter(|c| **c == Cell::Tree).count();
    for t in 1..40 { g.forest_tick(t); }
    let t1 = g.forest.cells.iter().flatten().filter(|c| **c == Cell::Tree).count();
    assert!(t1 > t0, "the stand never spread: {} -> {}", t0, t1);
    // a struck tree catches and burns out, and the log tells it
    let (x, y) = g.forest.cells.iter().enumerate()
        .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, c)| (x, y, c)))
        .find(|(_, _, c)| **c == Cell::Tree).map(|(x, y, _)| (x, y)).expect("no trees");
    g.forest.cells[y][x] = Cell::Burning;
    for t in 40..240 { g.forest_tick(t); }
    assert!(g.forest.cells.iter().flatten().all(|c| *c != Cell::Burning),
        "the fire never went out");
    assert!(g.log.iter().any(|l| l.contains("fire's out") || l.contains("fire breaks out")),
        "the fire went untold");
    // and a thinned canopy asks dear for a faggot
    if g.land.tree_cover.is_empty() {
        for i in 0..8 { g.land.tree_cover.insert((i, 0), 0.0); }
    }
    for v in g.land.tree_cover.values_mut() { *v = 0.0; }
    let w0 = crate::game::goods::worth_at(&g.world, "bavin");
    for t in 240..260 { g.forest_tick(t); }
    // a bare canopy leans the felt worth up
    assert!(crate::game::goods::worth_at(&g.world, "bavin") > w0 + 0.04,
        "a thin wood didn't lean fuel dear: {} -> {}", w0,
        crate::game::goods::worth_at(&g.world, "bavin"));
}
