use crate::session::*;

fn deadly_patch(g: &Game) -> usize {
    g.mush_patches.iter().position(|p| p.deadly()).unwrap()
}

fn kind_patch(g: &Game, kind: MushKind) -> usize {
    g.mush_patches.iter().position(|p| p.kind == kind).unwrap()
}

/// The roll `shroom_take` makes for `who` on day `t`
fn read_roll(who: Entity, t: u64) -> u64 {
    (who as u64).wrapping_mul(2654435761).wrapping_add(t) % 1000
}

/// A day `who`'s eye reads true at `skill`
fn true_read_day(who: Entity, skill: f32) -> u64 {
    (0..50000).find(|&t| read_roll(who, t) < mycology::read_chance(skill)).unwrap()
}

/// A day `who`'s eye fails at `skill`, folk rule or no
fn misread_day(who: Entity, skill: f32, folk: bool) -> u64 {
    (0..50000).find(|&t| {
        let r = read_roll(who, t);
        r >= mycology::read_chance(skill) && (r % 10 < 3) == folk
    }).unwrap()
}

#[test]
fn test_the_wood_keeps_each_kind_to_its_ground() {
    let g = Game::new();
    assert!(g.mush_patches.len() >= 5, "the wood keeps its patches");
    assert!(g.mush_patches.iter().any(|p| p.deadly()), "some caps are death");
    assert!(g.mush_patches.iter().any(|p| !p.deadly()), "some are good");
    // the families, each true to its habitat
    assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::LibertyCap),
        "the pasture keeps the little nipple-tipped caps");
    assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::Galerina),
        "the little brown killer keeps them company");
    assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::FlyAgaric),
        "the fly agaric under birch");
    assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::UnderCap),
        "the under keeps its own");
}

#[test]
fn test_the_lookalike_pairs_sit_the_way_the_research_put_them() {
    // galerina is the little brown mushroom that looks like the
    // liberty cap - the textbook trap for the dream-seeker
    assert_eq!(MushKind::Galerina.spec().lookalike, MushKind::LibertyCap);
    assert_eq!(MushKind::LibertyCap.spec().lookalike, MushKind::Galerina);
    assert!(MushKind::Galerina.is_deadly());
    assert!(MushKind::LibertyCap.believed_edible());
    // the death-cap wears the field mushroom's face; the jack wears
    // the chanterelle's - one kills, one just turns the gut
    assert_eq!(MushKind::DeathCap.spec().lookalike, MushKind::FieldMushroom);
    assert_eq!(MushKind::Chanterelle.spec().lookalike, MushKind::JackOLantern);
    assert!(MushKind::DeathCap.is_deadly());
    assert!(!MushKind::JackOLantern.is_deadly());
}

#[test]
fn test_a_good_cap_fills_the_belly() {
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = kind_patch(&g, MushKind::Chanterelle);
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 30.0;
    g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
    g.tick = true_read_day(who, 1.0);
    g.arrive(who, Goal::ForageShroom(pi));
    let h = g.world.get_component::<Hunger>(who).unwrap().value;
    assert!(h > 30.0, "chanterelles fill the belly, got {:.0}", h);
}

#[test]
fn test_the_death_cap_keeps_its_secret_for_days() {
    // the amatoxin course: nothing, then the belly, a false mend,
    // then the liver - the delayed onset is the horror of it
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = deadly_patch(&g);
    g.world.get_component_mut::<Craft>(who).unwrap().skill = 0.0;
    g.tick = misread_day(who, 0.0, false);
    g.arrive(who, Goal::ForageShroom(pi));
    let diseases_of = |g: &Game| g.world.query::<Disease>().iter()
        .filter_map(|&d| g.world.get_component::<Disease>(d))
        .filter(|dd| dd.patient == who).map(|dd| (dd.name.clone(), dd.severity))
        .collect::<Vec<_>>();
    assert!(diseases_of(&g).is_empty(), "nothing at first - that's the horror of it");
    assert!(g.ailments.iter().any(|a| a.who == who && matches!(a.phase, AilPhase::ShroomBelly(_))),
        "the course is already under way");
    // the belly lands a day on, when the meal's half forgotten
    g.ailment_tick(g.tick + 1);
    assert!(diseases_of(&g).iter().any(|(n, _)| n == "a bad belly"), "the belly lands");
    assert!(g.lore_of(who).contains(&pi), "the belly teaches the patch");
    // the false mend - they seem to turn the corner
    g.ailment_tick(g.tick + 3);
    assert!(diseases_of(&g).iter().all(|(n, s)| n != "a bad belly" || *s < 0.2),
        "they seem to mend - they haven't");
    // then the liver gives out, days after the meal
    g.ailment_tick(g.tick + 7);
    assert!(diseases_of(&g).iter().any(|(n, _)| n == "the liver gives out"),
        "the liver gives out at the end");
}

#[test]
fn test_the_learned_leave_death_be() {
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = deadly_patch(&g);
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 30.0;
    g.learn_shroom(who, pi);
    g.arrive(who, Goal::ForageShroom(pi));
    let h = g.world.get_component::<Hunger>(who).unwrap().value;
    assert!(h <= 30.0, "the learned leave the death-cap standing");
    assert!(g.ailments.iter().all(|a| a.who != who), "no course begins for the learned");
}

#[test]
fn test_the_herb_wife_teaches_the_wood() {
    let mut g = Game::new();
    let her = g.herbologist.unwrap();
    let who = g.villagers.iter().copied()
        .find(|&v| v != her && !villager::is_dead(&g.world, v)).unwrap();
    let hp = g.world.get_component::<Pos>(her).copied().unwrap();
    let p = g.world.get_component_mut::<Pos>(who).unwrap();
    p.x = hp.x; p.y = hp.y;
    g.teach_shrooms(g.tick);
    assert!(g.lore_of(who).iter().any(|&pi| g.mush_patches[pi].deadly()),
        "a neighbour learns which cap is death");
}

#[test]
fn test_the_folk_rule_clears_a_death_cap() {
    // "a poisoned cap blackens silver" - the silver stayed bright,
    // the cap was death anyway: knowledge can be wrong
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = deadly_patch(&g);
    g.world.get_component_mut::<Craft>(who).unwrap().skill = 0.0;
    g.tick = misread_day(who, 0.0, true);
    g.arrive(who, Goal::ForageShroom(pi));
    assert!(g.ailments.iter().any(|a| a.who == who && matches!(a.phase, AilPhase::ShroomBelly(_))),
        "the silver stayed bright - they ate death anyway");
}

#[test]
fn test_the_liberty_cap_gives_visions() {
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = kind_patch(&g, MushKind::LibertyCap);
    g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
    let mut saw_joy = false;
    let mut saw_turn = false;
    let mut day = 0u64;
    for _ in 0..30 {
        day = (day + 1..day + 50000)
            .find(|&t| read_roll(who, t) < mycology::read_chance(1.0)).unwrap();
        g.tick = day;
        g.arrive(who, Goal::ForageShroom(pi));
        saw_joy |= desires::is_satisfied(&g.world, who, Want::Joy, g.tick);
        saw_turn |= g.world.query::<Disease>().iter().any(|&d|
            g.world.get_component::<Disease>(d)
                .map(|dd| dd.patient == who && dd.name == "a fearful turn").unwrap_or(false));
        if saw_joy && saw_turn { break; }
    }
    assert!(saw_joy, "the field speaks kindly");
    assert!(saw_turn, "now and then a dark turn");
}

#[test]
fn test_the_fly_agarics_dream_is_wild_not_lethal() {
    let mut g = Game::new();
    let who = g.villagers[4];
    let pi = kind_patch(&g, MushKind::FlyAgaric);
    g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
    g.tick = true_read_day(who, 1.0);
    g.arrive(who, Goal::ForageShroom(pi));
    assert!(g.world.query::<Disease>().iter().any(|&d|
        g.world.get_component::<Disease>(d)
            .map(|dd| dd.patient == who && dd.name == "the red cap's dream").unwrap_or(false)),
        "the red cap's dream, not its death");
    assert!(g.ailments.iter().all(|a| a.who != who), "no amatoxin course from the fly agaric");
}

#[test]
fn test_the_fire_blames_the_bread() {
    let mut g = Game::new();
    g.ergot(210);
    assert!(g.log.iter().any(|l| l.contains("in the bread")));
    // the fire takes the convulsive and the burning forms
    let n_fire = g.world.query::<Disease>().iter().filter(|&&d|
        g.world.get_component::<Disease>(d)
            .map(|dd| dd.name == "the convulsive fire" || dd.name == "the burning fire")
            .unwrap_or(false)).count();
    let want = 2 + if g.harvest_wetness(210) > 0.45 { 2 } else { 0 };
    assert_eq!(n_fire, want, "the wet year's toll");
    // somebody carries the blame - the baker if there is one
    let blamed = g.villagers.iter().copied().any(|v|
        desires::problems_of(&g.world, v, 210).contains(&desires::Problem::Blamed));
    assert!(blamed, "the village finds someone to blame");
    // the brothers of Saint Anthony make their rounds
    if g.church.is_some() {
        assert!(g.ailments.iter().any(|a| matches!(a.phase, AilPhase::AntonineCare)),
            "a hospice bed for the burned");
    }
}

#[test]
fn test_the_player_picks_a_cap() {
    let mut g = Game::new();
    let pi = kind_patch(&g, MushKind::Chanterelle);
    let t = g.mush_patches[pi].tile;
    let pe = g.seats[0].player;
    if let Some(p) = g.world.get_component_mut::<Pos>(pe) { p.x = t.0 as f32; p.y = t.1 as f32; }
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::PickCaps).expect("no pick-caps button");
    g.ctx_act_at(0, i);
    assert!(g.mush_patches[pi].next_ready > g.tick, "the patch was never picked");
}

#[test]
fn test_the_player_knows_a_killer_when_he_sees_one() {
    let mut g = Game::new();
    let pi = deadly_patch(&g);
    g.mush_patches[pi].next_ready = 0;
    let pe = g.seats[0].player;
    g.learn_shroom(pe, pi);
    let t = g.mush_patches[pi].tile;
    if let Some(p) = g.world.get_component_mut::<Pos>(pe) { p.x = t.0 as f32; p.y = t.1 as f32; }
    g.update_ctx_actions_at(0);
    // a patch the belly's learned isn't offered at all
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::PickCaps),
        "a known killer stays off the card: {:?}", g.seats[0].ctx_actions);
    // and a stale click still can't pick it - the verb re-checks
    g.seats[0].ctx_actions.push(CtxAct::PickCaps);
    let i = g.seats[0].ctx_actions.len() - 1;
    g.ctx_act_at(0, i);
    assert!(g.log.iter().any(|l| l.contains("death - you know it")), "the lore didn't speak");
    assert_eq!(g.mush_patches[pi].next_ready, 0, "he picked a killer");
}

#[test]
fn test_the_players_gut_turns_and_lifts() {
    let mut g = Game::new();
    let pi = deadly_patch(&g);
    let pe = g.seats[0].player;
    // he has to eat it first: an untrained eye on a day his roll fails
    if let Some(c) = g.world.get_component_mut::<Craft>(pe) { c.skill = 0.0; }
    let day = misread_day(pe, 0.0, false);
    g.shroom_take(pe, pi, g.mush_patches[pi].kind, day);
    let c = g.world.get_component::<villager::Condition>(pe)
        .expect("the belly never turned on him");
    assert_eq!(c.kind, "gut-poisoned");
    assert!(g.lore_of(pe).contains(&pi), "the lesson never landed");
    // and the bad belly lifts with its day
    g.tick = (c.until..c.until + 8).find(|&t| t % 7 == 2).unwrap();
    g.care_tick(g.tick);
    assert!(g.world.get_component::<villager::Condition>(pe).is_none(),
        "a lifted belly should come off");
}
