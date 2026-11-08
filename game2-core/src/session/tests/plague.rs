use crate::session::*;
use crate::engine::ecs::System;
use crate::sim::illness::IllnessSystem;

/// The village's own able hands - country folk and children aren't
/// the workforce the wages answer to
fn village_adults(g: &Game) -> Vec<Entity> {
    g.villagers.iter().copied().filter(|&v|
        g.world.get_component::<Kid>(v).is_none()
        && g.world.get_component::<country::Outlander>(v).is_none()
        && !villager::is_dead(&g.world, v)).collect()
}

fn has_kind(g: &Game, who: Entity, kind: DiseaseKind) -> bool {
    g.world.query::<Disease>().iter().any(|&d|
        g.world.get_component::<Disease>(d)
            .map(|x| x.patient == who && x.kind == kind).unwrap_or(false))
}

#[test]
fn the_pestilence_is_a_contact_disease_that_keeps_no_season() {
    assert_eq!(DiseaseKind::Plague.name(), "the pestilence");
    assert_eq!(DiseaseKind::Plague.route(), crate::sim::illness::Route::Contact);
    assert!(DiseaseKind::Plague.contagious());
    // it walks the roads, not the calendar - every day is its season
    for day in [0u32, 90, 200, 300] {
        assert!(DiseaseKind::Plague.in_season(day));
    }
    // and beating it teaches the body nearly nothing
    assert!(DiseaseKind::Plague.immunity_gain() < 0.1);
    assert!(DiseaseKind::Plague.relapse().is_none());
}

#[test]
fn the_seed_finds_a_traveler_and_the_log_says_so() {
    let mut g = Game::new();
    let who = g.seed_plague(50).expect("a fresh country has carriers on the road");
    // a site carrier or a post rider took it - never a housebound farmer
    let travels = g.sites.iter().any(|s| s.carrier == Some(who))
        || g.couriers.iter().any(|c| c.e == who);
    assert!(travels, "the pestilence should ride somebody who actually walks the roads");
    assert!(has_kind(&g, who, DiseaseKind::Plague));
    assert!(g.log.iter().any(|l| l.contains("pestilence")),
        "the log should say the pestilence came up the road");
}

#[test]
fn the_seed_is_deterministic() {
    let mut a = Game::new();
    let mut b = Game::new();
    assert_eq!(a.seed_plague(50), b.seed_plague(50));
}

#[test]
fn a_carrier_sheds_it_wherever_they_stop() {
    let mut g = Game::new();
    let carrier = g.sites.iter().find_map(|s| s.carrier).expect("sites have carriers");
    let d = g.world.create_entity();
    g.world.add_component(d, Disease::new(DiseaseKind::Plague, carrier, 0.7));
    // a village hand stands a tile from where the carrier's rounds leave him
    let near = village_adults(&g).into_iter().next().expect("the village has adults");
    let cp = g.world.get_component::<Pos>(carrier).copied().unwrap();
    g.world.add_component(near, Pos { x: cp.x + 1.0, y: cp.y });
    // four days' silent brewing, then the dose builds tick by tick
    for _ in 0..30 { IllnessSystem.run(&mut g.world); }
    assert!(has_kind(&g, near, DiseaseKind::Plague),
        "whoever stands near the carrier catches what he carries");
}

/// One wave through a crowd: returns how many of them (carrier aside)
/// the pestilence killed
fn wave_toll(sev: f32) -> (usize, usize) {
    let mut g = Game::new();
    let carrier = g.sites.iter().find_map(|s| s.carrier).expect("sites have carriers");
    let d = g.world.create_entity();
    g.world.add_component(d, Disease::new(DiseaseKind::Plague, carrier, sev));
    // a market-day crowd round where the carrier stands
    let cp = g.world.get_component::<Pos>(carrier).copied().unwrap();
    let crowd: Vec<Entity> = village_adults(&g).into_iter().take(8).collect();
    for (i, &v) in crowd.iter().enumerate() {
        g.world.add_component(v, Pos { x: cp.x + (i % 3) as f32, y: cp.y + (i / 3) as f32 });
    }
    for _ in 0..600 { IllnessSystem.run(&mut g.world); }
    let dead = crowd.iter().filter(|&&v|
        villager::is_dead(&g.world, v)
            || g.world.get_component::<Health>(v).map(|h| h.condition <= 0.0).unwrap_or(false)).count();
    (dead, crowd.len())
}

#[test]
fn it_takes_a_healthy_share_of_the_contacts_not_all_of_them() {
    // the same crowd, two waves: the fierce strain fells most of them,
    // the fading one mostly spares them - a pestilence's toll is real
    // but never the whole village at once
    let (fierce_dead, n) = wave_toll(1.3);
    assert!(fierce_dead >= n / 2, "a fierce wave should fell about half the crowd or more, got {}/{}", fierce_dead, n);
    let (fading_dead, _) = wave_toll(1.1);
    assert!(fading_dead <= 2, "a fading wave should spare nearly the whole crowd, got {}/{}", fading_dead, n);
}

#[test]
fn wages_rise_when_the_plague_takes_the_hands() {
    let mut g = Game::new();
    let base = g.wage_mult();
    assert!(base <= 1.001, "a healthy village pays the going rate, got {}", base);
    // the pestilence's tally: the dead and the burning can't work
    let adults = village_adults(&g);
    for &v in adults.iter().take(adults.len() / 2) {
        if let Some(h) = g.world.get_component_mut::<Health>(v) { h.condition = 0.0; }
    }
    for &v in adults.iter().skip(adults.len() / 2).take(4) {
        let d = g.world.create_entity();
        let mut dis = Disease::new(DiseaseKind::Plague, v, 0.7);
        dis.stage = Stage::Moderate;
        dis.incubation = 0;
        g.world.add_component(d, dis);
    }
    let hurt = g.wage_mult();
    assert!(hurt > base, "half the hands gone, the living should command more: {} -> {}", base, hurt);
}
