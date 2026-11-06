use crate::session::*;
use crate::session::country::SiteKind;
use crate::game::vocation::Kid;

#[test]
fn test_the_country_is_peopled_like_villages() {
    let g = Game::new_seeded(5);
    for (si, s) in g.sites.iter().enumerate() {
        let folk: Vec<Entity> = g.villagers.iter().copied()
            .filter(|&v| g.site_of(v) == Some(si)).collect();
        assert!(folk.len() >= 6, "{} holds only {} folk", s.name, folk.len());
        match s.kind {
            SiteKind::Town | SiteKind::Farm | SiteKind::Monastery => assert!(folk.iter().any(|&v|
                g.world.get_component::<Kid>(v).is_some()), "{} has children", s.name),
            SiteKind::Castle => {
                assert!(s.homes.len() >= 2, "{} has a hall beside the keep", s.name);
                assert!(folk.iter().all(|&v| g.world.get_component::<Kid>(v).is_none()),
                    "{} is a garrison, not a nursery", s.name);
            }
        }
        for &v in &folk {
            let i = g.villagers.iter().position(|x| *x == v).unwrap();
            assert_ne!(vill_name(i), "a newcomer", "{}'s folk all have names", s.name);
        }
    }
}
