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
