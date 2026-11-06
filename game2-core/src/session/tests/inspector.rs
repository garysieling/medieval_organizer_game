use crate::session::*;

#[test]
fn test_activity_in_plain_words() {
    assert_eq!(activity_text(Activity::SeekingFood), "looking for food");
    assert_eq!(activity_text(Activity::Idle), "idle");
}

const TRAIT_WORDS: [&str; 8] = ["lazy", "ambitious", "kind", "stingy", "pious", "hot-headed", "gregarious", "proud"];

fn bio_of(g: &mut Game, i: usize) -> String {
    g.seats[0].selected_target = i;
    g.villager_inspector_at(0).lines.iter()
        .find(|l| l.y == 454).map(|l| l.text.clone()).unwrap_or_default()
}

/// The card's biography line: the limp is plain to see, but temperament
/// and kin take a bond known closely enough to have been told
#[test]
fn test_the_card_tells_the_biography_you_know() {
    let mut g = Game::new();
    // villagers[7] carries "a lame leg" from the setup
    let v = g.villagers[7];
    let bio = bio_of(&mut g, 7);
    assert!(bio.contains("a lame leg"), "the limp is plain to see: {}", bio);
    assert!(!TRAIT_WORDS.iter().any(|w| bio.contains(w)),
        "a stranger doesn't read their temperament: {}", bio);
    // a follower's bond is close enough to have been told things
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    let bio = bio_of(&mut g, 7);
    let p = g.world.get_component::<crate::game::traits::Personality>(v).unwrap();
    let told = p.traits.iter().any(|t| {
        let w = match t {
            crate::game::traits::Trait::Lazy => "lazy",
            crate::game::traits::Trait::Ambitious => "ambitious",
            crate::game::traits::Trait::Kind => "kind",
            crate::game::traits::Trait::Stingy => "stingy",
            crate::game::traits::Trait::Pious => "pious",
            crate::game::traits::Trait::Hotheaded => "hot-headed",
            crate::game::traits::Trait::Gregarious => "gregarious",
            crate::game::traits::Trait::Proud => "proud",
        };
        bio.contains(w)
    });
    assert!(told, "known folk show their temperament: {}", bio);
    // and the word carries what it does to their day
    let hinted = p.traits.iter().any(|t| {
        let h = match t {
            crate::game::traits::Trait::Lazy => "rests more",
            crate::game::traits::Trait::Ambitious => "works harder",
            crate::game::traits::Trait::Kind => "a warm word",
            crate::game::traits::Trait::Stingy => "tight-fisted",
            crate::game::traits::Trait::Pious => "keeps the faith",
            crate::game::traits::Trait::Hotheaded => "quick to anger",
            crate::game::traits::Trait::Gregarious => "strolls the village",
            crate::game::traits::Trait::Proud => "a proud bearing",
        };
        bio.contains(h)
    });
    assert!(hinted, "a trait should say what it does: {}", bio);
    // and kin by name, once the family tie is there to see
    let k = g.villagers[8];
    let krel = g.world.create_entity();
    g.world.add_component(krel, crate::game::relationships::Relationship {
        from: v, other: k,
        kind: crate::game::relationships::RelKind::Family,
        strength: 0.9, interactions: 0 });
    let bio = bio_of(&mut g, 7);
    assert!(bio.contains("kin:"), "family shows once known: {}", bio);
}

#[test]
fn test_the_card_names_the_rung() {
    let mut g = Game::new();
    let v = g.villagers[7];
    // a master on the bench, by skill
    g.world.get_component_mut::<crate::game::vocation::Craft>(v).unwrap().skill = 0.95;
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    assert!(ins.lines.iter().any(|l| l.text.contains("Master")),
        "the rung never showed: {:?}", ins.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}
