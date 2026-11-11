use crate::session::*;

#[test]
fn test_activity_in_plain_words() {
    assert_eq!(activity_text(Activity::SeekingFood), "looking for food");
    assert_eq!(activity_text(Activity::Idle), "idle");
}

const TRAIT_WORDS: [&str; 8] = ["lazy", "ambitious", "kind", "stingy", "pious", "hot-headed", "gregarious", "proud"];

/// The card's whole text, every row's words in one string - the
/// biography lives in the LIFE section now, told across wrapped lines
fn card_text(ins: &InspectorV) -> String {
    ins.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join(" ")
}

fn bio_of(g: &mut Game, i: usize) -> String {
    g.seats[0].selected_target = i;
    card_text(&g.villager_inspector_at(0))
}

/// The card's biography line: the limp is plain to see, but temperament
/// and kin take a bond known closely enough to have been told
#[test]
fn test_the_card_tells_the_biography_you_know() {
    let mut g = Game::new();
    // villagers[7] may or may not carry a difference from the setup -
    // make sure they've one so the card has something to say
    let v = g.villagers[7];
    if g.world.get_component::<villager::Condition>(v).is_none() {
        g.world.add_component(v, villager::Condition { kind: "a lame leg", until: u64::MAX });
    }
    let kind = g.world.get_component::<villager::Condition>(v).unwrap().kind;
    let bio = bio_of(&mut g, 7);
    assert!(bio.contains(kind), "the difference is plain to see: {}", bio);
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
    assert!(bio.contains("Kin:"), "family shows once known: {}", bio);
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

#[test]
fn test_a_child_s_card_names_their_leaning() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<Kid>(v).is_some()).unwrap();
    // keep the smith's errands busiest
    let smith = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<crate::game::vocation::Craft>(v)
            .map(|c| c.vocation == crate::game::vocation::Vocation::Toolmaker).unwrap_or(false)).unwrap();
    g.world.get_component_mut::<crate::game::vocation::KidGrowth>(v).unwrap()
        .errands.push((smith, 5));
    g.seats[0].selected_target = g.villagers.iter().position(|&x| x == v).unwrap();
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("runs for the toolmaker"), "the leaning never showed: {}", bio);
}

#[test]
fn test_the_card_names_who_they_lost() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let gone = g.villagers[9];
    // dead and grieved for
    g.world.add_component(gone, villager::Dead);
    g.world.get_component_mut::<AttachmentHistory>(v).unwrap()
        .episodes.push(crate::sim::psychology::Episode {
            with: gone, kind: crate::sim::psychology::EpisodeKind::Loss });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("grieves for"), "the loss never showed: {}", bio);
}

#[test]
fn test_the_card_names_the_tie_between_you() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.6, interactions: 3 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("your friend"), "the tie never showed: {}", bio);
}

#[test]
fn test_the_card_names_how_warm_they_are() {
    let mut g = Game::new();
    let v = g.villagers[7];
    crate::game::player::note_conversation(&mut g.world, g.seats[0].player, v);
    crate::game::player::note_conversation(&mut g.world, g.seats[0].player, v);
    crate::game::player::note_conversation(&mut g.world, g.seats[0].player, v);
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("warm to you") || bio.contains("close to you") || bio.contains("has your ear"),
        "the warmth never showed: {}", bio);
}

#[test]
fn test_the_card_names_their_cause_craft() {
    let mut g = Game::new();
    let v = g.villagers[7];
    g.world.add_component(v, crate::game::org_skills::Organizing {
        turnout: 0.7, ..Default::default() });
    // known, so the bio opens
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("brings them out"), "the cause-craft never showed: {}", bio);
}

#[test]
fn test_the_card_says_what_they_keep() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<crate::game::vocation::Craft>(v)
            .map(|c| c.vocation == crate::game::vocation::Vocation::Shepherd).unwrap_or(false)).unwrap();
    // known, so the bio opens
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = g.villagers.iter().position(|&x| x == v).unwrap();
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("keeps") && bio.contains("sheep"),
        "the fold never showed: {}", bio);
}

#[test]
fn test_the_card_names_their_pet() {
    let mut g = Game::new();
    let v = g.villagers[7];
    // a bonded dog
    let dog = crate::game::pets::spawn_pet(&mut g.world, "Fido",
        crate::game::pets::Species::Dog, crate::game::pets::PetRole::Companion);
    crate::game::pets::adopt(&mut g.world, dog, v);
    g.world.get_component_mut::<crate::game::pets::Pet>(dog).unwrap().bond_strength = 0.6;
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("keeps Fido the dog"), "the pet never showed: {}", bio);
}

#[test]
fn test_the_card_says_how_the_sickness_runs() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let d = g.world.create_entity();
    g.world.add_component(d, crate::sim::illness::Disease {
        kind: crate::sim::illness::DiseaseKind::Influenza, name: "flu".into(),
        patient: v, stage: crate::sim::illness::Stage::Severe,
        severity: 0.8, contagious: true, incubation: 0, recovering: 0 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("laid low") || bio.contains("taken ill") || bio.contains("near death"),
        "the stage never showed: {}", bio);
}

#[test]
fn test_the_card_names_a_weathered_one_and_a_sleeping_sickness() {
    let mut g = Game::new();
    let v = g.villagers[7];
    if let Some(h) = g.world.get_component_mut::<crate::sim::illness::Health>(v) {
        h.immunity = 0.7;
    }
    g.world.add_component(v, crate::sim::illness::Relapse {
        kind: crate::sim::illness::DiseaseKind::Ague, due_in: 200 });
    // the seed shows only once they've earned the villager's confidence
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("weathered"), "the hardiness never showed: {}", bio);
    assert!(bio.contains("sleeps in them"), "the seed never showed: {}", bio);
}

#[test]
fn test_the_card_says_the_frame_fails() {
    let mut g = Game::new();
    let v = g.villagers[7];
    if let Some(h) = g.world.get_component_mut::<crate::sim::illness::Health>(v) {
        h.condition = 0.2;
    }
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("failing"), "the frailty never showed: {}", bio);
}

#[test]
fn test_a_child_s_card_names_the_errand_they_re_on() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<Kid>(v).is_some()).unwrap();
    let smith = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<crate::game::vocation::Craft>(v)
            .map(|c| c.vocation == crate::game::vocation::Vocation::Toolmaker).unwrap_or(false)).unwrap();
    let from = g.building_entities[0];
    g.world.add_component(v, crate::game::livelihood::OddJob {
        hirer: smith, good: "ale".into(), qty: 2,
        from, to: from, pay: 1.5, loaded: false });
    g.seats[0].selected_target = g.villagers.iter().position(|&x| x == v).unwrap();
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    assert!(bio.contains("running ale"), "the errand never showed: {}", bio);
}

#[test]
fn test_the_card_names_their_temperament() {
    let mut g = Game::new();
    let v = g.villagers[7];
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let bio = card_text(&ins);
    for w in ["curious bent", "easily ruffled", "full of talk", "soft-hearted", "keeps good order", "even-tempered"] {
        if bio.contains(w) { return; }
    }
    panic!("no temperament word showed: {}", bio);
}

/// The page's edge: every row's text, wherever it sits, stays inside
/// it - the run-on line that once rode off across the map is gone
#[test]
fn test_the_card_keeps_its_rows_inside_the_page() {
    let mut g = Game::new();
    let v = g.villagers[7];
    // known to the seat, so the whole page is told - sections and all
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    for l in &ins.lines {
        // a meter sizes itself - its sentinel isn't text to measure
        if l.text.starts_with('\u{1}') { continue; }
        assert!(l.x + l.text.chars().count() as i32 * GLYPH_W <= CARD_RX,
            "row runs off the card: {}", l.text);
    }
    // a page's shape: the headers its sections read under
    for head in ["TRAITS", "LIFE"] {
        assert!(ins.lines.iter().any(|l| l.text == head), "the {} section never showed", head);
    }
    // no two rows share a slot - the old fixed rows let lines overlap
    let mut slots: Vec<(i32, i32)> = ins.lines.iter().map(|l| (l.x, l.y)).collect();
    slots.sort();
    slots.dedup();
    assert_eq!(slots.len(), ins.lines.len(), "rows share a slot");
}

/// The page's labels read like a properties sheet: the fields it asks
/// after are named rows, not a string of fragments
#[test]
fn test_the_card_reads_as_labelled_fields() {
    let mut g = Game::new();
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let text = ins.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join(" ");
    for label in ["Home:", "Nature:", "Today:"] {
        assert!(text.contains(label), "no {} row on the card: {}", label, text);
    }
}

#[test]
fn test_the_dead_s_card_names_their_mourners() {
    let mut g = Game::new();
    let dead = g.villagers[9];
    g.died_on[9] = Some(12);
    let m = g.villagers[7];
    g.world.get_component_mut::<crate::sim::psychology::AttachmentHistory>(m).unwrap()
        .episodes.push(crate::sim::psychology::Episode {
            with: dead, kind: crate::sim::psychology::EpisodeKind::Loss });
    g.seats[0].selected_target = 9;
    let ins = g.villager_inspector_at(0);
    assert!(ins.lines.iter().any(|l| l.text.contains("Mourned by")),
        "the mourner never showed: {:?}",
        ins.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}
