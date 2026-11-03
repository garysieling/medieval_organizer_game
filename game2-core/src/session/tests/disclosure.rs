use crate::session::*;

#[test]
fn test_a_truth_ripens_and_comes_out() {
    let mut g = Game::new();
    let cleric = g.villagers[5];
    g.translator = Some(cleric);
    g.discovery_tick(20);
    assert!(g.pending_discovery.is_some(), "a truth sits unsaid");
    g.discovery_tick(50); // past the fortnight
    assert!(g.disclosure_made, "the word's out");
    assert!(g.log.iter().any(|l| l.contains("whispers it about")),
        "it comes out in a whisper");
    // weak org: the threatened move - blame lands on the discoverer
    let blame = desires::problems_of(&g.world, cleric, 50)
        .iter().filter(|p| **p == desires::Problem::Blamed).count();
    assert!(blame >= 1, "the ladder's there for a man who talks treason");
}

#[test]
fn test_a_strong_org_stands_with_its_own() {
    let mut g = Game::new();
    let cleric = g.villagers[5];
    g.translator = Some(cleric);
    g.lord_esteem = 0.5;
    g.pending_discovery = Some((cleric, "the lord's fifth is really a quarter", 20));
    g.discovery_tick(50);
    assert!(g.disclosure_made);
    let blame = desires::problems_of(&g.world, cleric, 50)
        .iter().filter(|p| **p == desires::Problem::Blamed).count();
    assert_eq!(blame, 0, "the village stands with him");
    assert!(g.log.iter().any(|l| l.contains("stands with")));
}
