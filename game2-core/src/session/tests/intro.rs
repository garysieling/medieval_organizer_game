use crate::session::*;

#[test]
fn test_the_card_tells_true_things() {
    let g = Game::new();
    let facts = g.intro_facts();
    assert!(facts.len() >= 3, "a thin card: {:?}", facts);
    // both market towns and both keeps are named on it - all real sites
    for s in &g.sites {
        if matches!(s.kind, country::SiteKind::Town | country::SiteKind::Castle) {
            assert!(facts.iter().any(|f| f.contains(s.name)), "{} went unmentioned", s.name);
        }
    }
    // and a face's name on it is somebody real
    let names: Vec<&str> = (0..g.villagers.len()).map(vill_name).collect();
    assert!(facts.iter().any(|f| names.iter().any(|n| f.contains(n))),
        "no villager made the card: {:?}", facts);
}

#[test]
fn test_the_seed_picks_the_facts() {
    let a = Game::new().intro_facts();
    assert_eq!(a, Game::new().intro_facts(), "the same seed dealt a different card");
    let differ = (1..10).any(|s| Game::new_seeded(s).intro_facts() != a);
    assert!(differ, "every seed dealt the same card");
}

#[test]
fn test_the_card_and_the_camera_agree_on_the_face() {
    let g = Game::new();
    let i = g.intro_face().expect("no face named");
    // the face's name is on the card - the same pick, not a second roll
    let facts = g.intro_facts();
    assert!(facts.iter().any(|f| f.contains(vill_name(i))),
        "the card doesn't name the focused face: {:?}", facts);
    // and it's a living villager, not a grave or a country stranger
    let v = g.villagers[i];
    assert!(!villager::is_dead(&g.world, v));
    assert!(g.world.get_component::<country::Outlander>(v).is_none());
}

#[test]
fn test_settling_points_the_camera_at_the_named_face() {
    let mut g = Game::new();
    let i = g.intro_face().expect("no face named");
    let pos = g.world.get_component::<villager::Pos>(g.villagers[i]).map(|p| (p.x, p.y));
    // keeping the wanderer settles the seat - and the camera answers
    // the card: the seat's focus is the named face's spot
    assert!(g.decline_pick_at(0));
    let focus = g.seats[0].focus.expect("no focus asked");
    assert_eq!(Some((focus.0, focus.1)), pos);
    // a second decline doesn't replay the ask
    let n = focus.2;
    assert!(g.decline_pick_at(0));
    assert_eq!(g.seats[0].focus.unwrap().2, n);
}
