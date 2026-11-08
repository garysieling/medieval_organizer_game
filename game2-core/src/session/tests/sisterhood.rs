use crate::session::*;

#[test]
fn test_an_abbess_house_keeps_the_hive() {
    let g = Game::new();
    let site = g.sites.iter().find(|s| s.sisterhood.is_some());
    assert!(site.is_some(), "no abbess kept a sisterhood");
    let org = site.unwrap().sisterhood.as_ref().unwrap();
    assert!(org.charter.contains("sisterhood"), "the charter didn't name the house");
    assert_eq!(org.leader, site.unwrap().lord.unwrap(), "the matriarch didn't lead");
    assert!(!org.members.is_empty(), "the hive's empty");
}
