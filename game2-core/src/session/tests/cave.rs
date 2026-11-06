use crate::session::*;
use crate::session::routine::{DayPlan, Routine, Spot};

#[test]
fn test_the_hermit_keeps_to_the_dark() {
    let g = Game::new();
    let Some(h) = g.hermit else { panic!("no hermit") };
    assert_eq!(g.world.get_component::<TownRole>(h), Some(&TownRole::Hermit));
    assert!(desires::problems_of(&g.world, h, 1).contains(&desires::Problem::TheDark),
        "the dark keeps them");
    // and the deep patch is there for the picking
    assert!(g.mush_patches.iter().any(|p| p.tile == (35, 33) && p.kind == MushKind::UnderCap),
        "the best mushrooms grow in the under");
}

#[test]
fn test_the_dark_keeps_the_hermit_home() {
    let mut g = Game::new();
    let h = g.hermit.expect("a recluse keeps the cave");
    let cave = g.cave_pos.expect("a cave at the map's edge");
    let mut rng = Rng(5);
    let plan = g.plan_day(h, 8, &mut rng);
    assert_eq!(plan, DayPlan::Free, "the cave IS his day");
    let r = g.world.get_component::<Routine>(h).expect("no routine");
    assert!(matches!(r.spot, Spot::Cave), "he's not bound for the cave: {:?}", r.spot);
    assert_eq!((r.anchor.0.round() as i32, r.anchor.1.round() as i32), cave, "not the cave");
    assert!(!desires::problems_of(&g.world, h, g.tick).contains(&desires::Problem::TheDark),
        "the dark never kept him");
}
