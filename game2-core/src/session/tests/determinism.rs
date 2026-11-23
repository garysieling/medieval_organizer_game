//! One seed, one game: two runs of the same seed must agree tick for tick.
//! Replays, lockstep multiplayer and golden-frame tests all lean on it.
use crate::session::*;

/// Everything a run can differ in, as text: the omniscient probe, the
/// secret gauges, where everyone stands and the log
fn state_of(g: &Game) -> String {
    let pos: Vec<(f32, f32)> = g.villagers.iter()
        .map(|&v| g.world.get_component::<villager::Pos>(v).map(|p| (p.x, p.y)).unwrap_or((-1.0, -1.0)))
        .collect();
    format!("{}\n{:?}\n{:?}\n{:?}", serde_json::to_string(&g.probe()).unwrap(), g.secret_kpis(), pos, g.log)
}

/// First tick at which two runs of `seed` disagree, and the first differing line
fn first_divergence(seed: u64, ticks: u64) -> Option<(u64, String, String)> {
    // turbo+: the check is once a day either way, and 150 days of frames
    // is the suite's slowest test by far at walking pace
    let (mut a, mut b) = (Game::new_seeded(seed), Game::new_seeded(seed));
    a.speed = 16;
    b.speed = 16;
    while a.tick < ticks {
        let day = a.tick;
        a.update();
        b.update();
        assert_eq!(a.tick, b.tick, "the clocks run together");
        // compare once a day: the state's big, and a day is when most of it moves
        if a.tick == day { continue; }
        let (sa, sb) = (state_of(&a), state_of(&b));
        if sa != sb {
            let (la, lb) = sa.lines().zip(sb.lines()).find(|(x, y)| x != y).unwrap_or(("?", "?"));
            let at = la.char_indices().zip(lb.chars()).find(|((_, x), y)| x != y).map(|((i, _), _)| i).unwrap_or(0);
            let cut = |s: &str| s.chars().skip(at.saturating_sub(120)).take(260).collect::<String>();
            return Some((a.tick, cut(la), cut(lb)));
        }
    }
    None
}

#[test]
fn the_same_seed_plays_the_same_game() {
    if let Some((t, a, b)) = first_divergence(42, std::env::var("GAME2_DET_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(150)) {
        panic!("two runs of seed 42 diverge at tick {t}:\n  a: {a}\n  b: {b}");
    }
}

/// E1-2 (#1411): the seeded cast is deterministic too - same seed, same
/// villagers: same personas, dealt names and bodies, no duplicates
#[test]
fn a_seeded_cast_deals_the_same_people() {
    let a = Game::new_seeded_cast(77);
    let b = Game::new_seeded_cast(77);
    assert_eq!(a.villagers.len(), b.villagers.len());
    // every villager carries a persona and its dealt name
    for (i, (&va, &vb)) in a.villagers.iter().zip(b.villagers.iter()).enumerate() {
        let (pa, pb) = (a.world.get_component::<crate::game::persona::Persona>(va).unwrap(),
            b.world.get_component::<crate::game::persona::Persona>(vb).unwrap());
        assert_eq!(pa, pb, "slot {} persona", i);
        assert_eq!(a.names[i], b.names[i], "slot {} name", i);
        assert_eq!(a.world.get_component::<villager::Sex>(va),
                   b.world.get_component::<villager::Sex>(vb), "slot {} body", i);
    }
    // a different seed deals a different cast
    let c = Game::new_seeded_cast(78);
    let same = (0..a.villagers.len()).filter(|&i| a.names[i] == c.names[i]).count();
    assert!(same < a.villagers.len() / 2, "casts should differ: {}", same);
    // no two villagers share a name
    let mut seen = crate::engine::det::HashSet::default();
    for &n in a.names.iter().take(a.villagers.len()) {
        assert!(seen.insert(n), "duplicate name {}", n);
    }
    // the fixed cast is untouched: positional names as before
    let f = Game::new_seeded(77);
    assert!(!f.seeded_cast);
    assert_eq!(f.names[0], vill_name(0));
}

/// E1-3 (#1417): in a seeded cast names and bodies agree - every dealt
/// name comes out of the pool matching the persona's sex, and the Sex
/// component is the persona's own draw
#[test]
fn a_seeded_casts_names_agree_with_their_bodies() {
    let g = Game::new_seeded_cast(77);
    for (i, &v) in g.villagers.iter().enumerate() {
        let p = g.world.get_component::<crate::game::persona::Persona>(v).unwrap();
        let sex = g.world.get_component::<villager::Sex>(v).copied().unwrap();
        assert_eq!(sex, p.sex, "slot {} body isn't the persona's", i);
        let pool: &[&'static str] = match p.sex {
            villager::Sex::Female => &FEMALE_NAMES,
            villager::Sex::Male => &MALE_NAMES,
        };
        assert!(g.names[i] == "a newcomer" || pool.contains(&g.names[i]),
            "slot {}: {} ({:?}) named off the wrong pool", i, g.names[i], p.sex);
    }
}

/// E1-3 (#1417): the gendered pools partition the two authored rolls
/// exactly - every name dealt is drawn from NAMES or NEW_NAMES, once
#[test]
fn the_name_pools_cover_the_authored_rolls() {
    let mut all = crate::engine::det::HashSet::default();
    for &n in MALE_NAMES.iter().chain(FEMALE_NAMES.iter()) {
        assert!(all.insert(n), "{} sits in both pools", n);
    }
    for &n in NAMES.iter().chain(NEW_NAMES.iter()) {
        assert!(all.contains(n), "{} never reaches a pool", n);
    }
    assert_eq!(all.len(), NAMES.len() + NEW_NAMES.len());
}

/// E1-4 (#1412): in a seeded cast both personality components are the
/// persona - the Big Five is the axes verbatim and the discrete traits
/// come off the threshold table, so the views can never disagree
#[test]
fn a_seeded_casts_personalities_are_the_persona() {
    let g = Game::new_seeded_cast(77);
    for (i, &v) in g.villagers.iter().enumerate() {
        let p = g.world.get_component::<crate::game::persona::Persona>(v).unwrap();
        let (_, want) = p.personalities();
        let b5 = g.world.get_component::<crate::sim::psychology::Personality>(v)
            .unwrap_or_else(|| panic!("slot {i} carries no Big Five"));
        assert_eq!((b5.openness, b5.conscientiousness, b5.extraversion,
            b5.agreeableness, b5.neuroticism),
            (p.temperament[0], p.temperament[1], p.temperament[2],
             p.temperament[3], p.temperament[4]), "slot {i} Big Five isn't the axes");
        let t = g.world.get_component::<crate::game::traits::Personality>(v)
            .unwrap_or_else(|| panic!("slot {i} carries no traits"));
        assert_eq!(t.traits, want.traits, "slot {i} traits aren't the table's");
    }
}

/// E1-3 (#1417): a spawned villager's body is drawn, not the entity
/// id's parity - consecutive ids can share a sex now
#[test]
fn a_spawned_body_isnt_the_ids_parity() {
    let mut shared = 0;
    for e in 0..64u32 {
        if villager::Sex::from_seed(e) == villager::Sex::from_seed(e + 1) { shared += 1; }
    }
    assert!(shared > 0, "bodies still alternate with the id");
}
