use crate::session::*;

/// The opening victim: one villager marked by a seeded injustice, warm
/// enough to open up, with every seat's first job being to hear them out
#[test]
fn test_every_seed_marks_exactly_one_victim() {
    for seed in [MAP_SEED, 7, 42, 1337] {
        let g = Game::new_seeded(seed);
        let (v, wrong) = g.intro_victim.expect("no opening victim picked");
        // a real grievance pinned on them - the same wrong the pick recorded
        let life = g.world.get_component::<desires::InnerLife>(v).expect("no inner life");
        assert!(life.grievances.iter().any(|&(p, u)| p == wrong && u == u64::MAX),
            "seed {seed}: the victim doesn't carry {:?}", wrong);
        // and they're somebody the village could actually ask after:
        // grown, living, one of the village's own
        assert!(g.world.get_component::<crate::game::vocation::Kid>(v).is_none());
        assert!(!crate::game::villager::is_dead(&g.world, v));
        assert!(g.world.get_component::<country::Outlander>(v).is_none());
        // warm enough to open up - above the ask line, not best-friends
        let rap = crate::game::player::rapport(&g.world, g.seats[0].player, v);
        assert!(rap >= 0.2, "seed {seed}: rapport {rap} is stranger-than-stranger");
        assert!(rap < 0.9, "seed {seed}: {rap} - manufactured best-friends isn't warmth, it's a lie");
        // and every seat's first job is the same shared one
        for s in &g.seats {
            let q = s.quest.as_ref().expect("a seat with no opening job");
            assert_eq!(q.kind, quests::QuestKind::Talk);
            assert_eq!(q.giver, v);
            assert_eq!(q.target, quests::QuestTarget::Villager(v));
        }
        // the panel says what the job is
        let line = g.ui_model().panel.quest.expect("the job's not on the panel");
        assert!(line.contains("hear") && line.contains(&g.name_of(v)), "{line}");
        // and the intro card names them with their wrong
        let facts = g.intro_facts();
        assert!(facts.iter().any(|f| f.contains(&g.name_of(v)) && f.contains("listener")),
            "the intro never names them: {facts:?}");
    }
}

/// The same seed deals the same victim - the pick can't shift the village
#[test]
fn test_the_victim_pick_is_deterministic() {
    for seed in [MAP_SEED, 42] {
        let a = Game::new_seeded(seed);
        let b = Game::new_seeded(seed);
        assert_eq!(a.intro_victim, b.intro_victim, "seed {seed} dealt two victims");
    }
}

/// Hearing them out ends the job: no coin for an ear, but the bond's
/// paid in knowing them
#[test]
fn test_talking_to_them_hears_the_trouble_out() {
    let mut g = Game::new();
    let (v, _) = g.intro_victim.unwrap();
    let pe = g.seats[0].player;
    let gold_before = g.world.get_component::<crate::game::livelihood::Purse>(pe)
        .map(|p| p.gold).unwrap_or(0.0);
    let done = g.quest_talk_done(0, v).expect("talking to them didn't finish the job");
    assert!(done.contains("feels heard"), "{done}");
    assert!(g.seats[0].quest.is_none(), "the job's still on the books");
    let gold_after = g.world.get_component::<crate::game::livelihood::Purse>(pe)
        .map(|p| p.gold).unwrap_or(0.0);
    assert_eq!(gold_before, gold_after, "an ear shouldn't pay coin");
    // and the panel's quiet again
    assert!(g.ui_model().panel.quest.is_none());
}

/// Somebody else's ear doesn't count - the job's theirs specifically
#[test]
fn test_only_the_victim_lifts_the_trouble() {
    let mut g = Game::new();
    let (v, _) = g.intro_victim.unwrap();
    let other = g.villagers.iter().copied()
        .find(|&x| x != v && g.world.get_component::<desires::InnerLife>(x).is_some())
        .expect("no other villager to try");
    assert!(g.quest_talk_done(0, other).is_none(), "a stranger's ear finished it");
    assert!(g.seats[0].quest.is_some(), "the job lapsed on the wrong ears");
    // talking past them first doesn't spend it either
    assert!(g.quest_talk_done(0, v).is_some());
}

/// If they die before they're heard, the job lapses like any other
#[test]
fn test_a_victim_who_dies_leaves_the_job_unanswered() {
    let mut g = Game::new();
    let (v, _) = g.intro_victim.unwrap();
    // the grave takes them before dawn's bookkeeping
    g.world.add_component(v, crate::game::villager::Dead);
    g.tick += 1;
    let mut rng = Rng(g.seed ^ 0x9E37);
    g.quest_dawn(g.tick, &mut rng);
    assert!(g.seats[0].quest.is_none(), "a dead man's job still stands");
}

/// An agent who takes the victim's own reins can't be asked to hear
/// themselves out - the job's for a listener, and they have the trouble
#[test]
fn test_an_agent_driving_the_victim_gets_no_talk_job() {
    let mut g = Game::new();
    let (v, _) = g.intro_victim.unwrap();
    let seat = g.add_agent_seat(v);
    assert!(g.seats[seat].quest.is_none(),
        "the victim was asked to hear their own trouble out");
    // the Wanderer's job is untouched - somebody still owes them an ear
    assert_eq!(g.seats[0].quest.as_ref().map(|q| q.kind), Some(quests::QuestKind::Talk));
}

/// And an agent's own chat counts: walking to the victim and talking
/// hands their opening job over, same as the Wanderer's verb
#[test]
fn test_an_agent_hears_the_trouble_out_by_talking() {
    let mut g = Game::new();
    let (v, _) = g.intro_victim.unwrap();
    let who = g.villagers.iter().copied().find(|&x| x != v
        && g.world.get_component::<crate::game::vocation::Kid>(x).is_none()
        && g.world.get_component::<country::Outlander>(x).is_none()).unwrap();
    let seat = g.add_agent_seat(who);
    assert_eq!(g.seats[seat].quest.as_ref().map(|q| q.kind), Some(quests::QuestKind::Talk));
    // beside them already, the walk's a formality - the chat is the job
    let at = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(who).unwrap() = at;
    g.agent_command(seat, AgentCmd::TalkTo { e: v }).unwrap();
    for _ in 0..30 {
        g.update();
        if g.seats[seat].quest.is_none() { break; }
    }
    assert!(g.seats[seat].quest.is_none(), "the agent's chat didn't hand the job over");
}
