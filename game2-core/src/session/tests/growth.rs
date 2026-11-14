use crate::session::*;

#[test]
fn test_a_full_house_gets_a_child() {
    let mut g = Game::new();
    // a house with two grown folk: room fills out from there
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b).map(|bb| bb.kind == BuildingKind::Home).unwrap_or(false)).unwrap();
    let two: Vec<Entity> = g.villagers[..2].to_vec();
    g.world.get_component_mut::<Building>(home).unwrap().occupants = two;
    let n = g.villagers.len();
    g.maybe_birth(40); // may or may not, it's a roll - keep calling until the roll lands
    for t in (40..400).step_by(40) { if g.villagers.len() > n { break; } g.maybe_birth(t); }
    assert!(g.villagers.len() > n, "a full house gets a child");
    let baby = *g.villagers.last().unwrap();
    assert!(g.world.get_component::<Kid>(baby).is_some());
    assert!(g.world.get_component::<KidGrowth>(baby).is_some());
    assert!(g.log.iter().any(|l| l.contains("born")), "{:?}", g.log.last());
}

/// Births feel the larder twice over: the village table's own census
/// thins the roll, and a hungry belly in the house stops it outright
fn births_over(g: &mut Game, calls: u64) -> usize {
    let n = g.villagers.len();
    for t in 0..calls { g.maybe_birth(t * 40); }
    g.villagers.len() - n
}

fn empty_the_table(g: &mut Game) {
    for f in std::iter::once(g.market_food).chain(g.crop_fields.iter().copied()) {
        g.world.get_component_mut::<villager::FoodSource>(f)
            .map(|f| f.servings = 0);
    }
}

#[test]
fn test_hungry_folk_raise_no_children() {
    let mut g = Game::new();
    // every belly empty: no house rolls, no matter how the table stands
    for v in g.villagers.clone() {
        g.world.get_component_mut::<Hunger>(v).map(|h| h.value = 10.0);
    }
    assert_eq!(births_over(&mut g, 200), 0,
        "starving couples don't conceive - no crib fills on an empty belly");
}

#[test]
fn test_a_bare_larder_thins_the_nursery() {
    // the roll itself reads the table: stocked, the usual one-in-three;
    // bare, one-in-twelve
    let mut fed = Game::new();
    let mut bare = Game::new();
    assert_eq!(fed.birth_odds(), 3, "a stocked table keeps the usual clip");
    empty_the_table(&mut bare);
    assert_eq!(bare.birth_odds(), 12, "a bare table thins the roll fourfold");
    // and over the same days the bare village births fewer
    let fed_births = births_over(&mut fed, 30);
    let bare_births = births_over(&mut bare, 30);
    assert!(fed_births > 0, "a fed village fills its cribs");
    assert!(bare_births < fed_births,
        "a famine thins the nursery: {} bare vs {} fed", bare_births, fed_births);
}

#[test]
fn test_a_stranger_settles_at_the_gate() {
    let mut g = Game::new();
    let n = g.villagers.len();
    g.maybe_migrant(30);
    assert_eq!(g.villagers.len(), n + 1);
    let m = *g.villagers.last().unwrap();
    // trade in hand, and a roof if one's going (a bed want otherwise)
    assert!(g.world.get_component::<Craft>(m).is_some());
    let housed = household::home_of(&g.world, m).is_some();
    let wants_bed = desires::problems_of(&g.world, m, g.tick).contains(&desires::Problem::NoHome);
    assert!(housed || wants_bed, "a spare bed or the want of one");
    assert!(g.log.iter().any(|l| l.contains("settles")), "{:?}", g.log.last());
}
