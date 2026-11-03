use crate::session::*;

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
