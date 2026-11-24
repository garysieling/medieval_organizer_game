//! The odd jobs villagers ask the Wanderer to run: taken in the Talk
//! they open with, the thing fetched where it lies, handed over by
//! talking to whoever it's for - or by standing on the far village's
//! ground
use crate::session::*;
use crate::session::quests::{PendingQuest, PlayerQuest, QuestKind, QuestStage, QuestTarget};
use crate::game::livelihood::Purse;
use crate::game::player::PlayerCharacter;

/// A grown villager of the village who'll ask after the Wanderer
fn askers(g: &Game) -> Vec<Entity> {
    g.villagers.iter().copied().filter(|&v| {
        g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !crate::game::villager::is_dead(&g.world, v)
            && g.world.get_component::<crate::session::country::Outlander>(v).is_none()
    }).collect()
}

fn pending(giver: Entity, kind: QuestKind, target: QuestTarget, pickup: Option<(f32, f32)>) -> PendingQuest {
    PendingQuest { giver, kind, target, pickup, expires: 100, errand: None }
}

/// A job going begging opens the Talk it's taken in: the offer leads
/// the picks, your yes runs the same accept the card's button did, and
/// the exchange lands on the transcript (#1336)
#[test]
fn test_a_begging_job_opens_the_talk_and_its_pick_takes_it() {
    let mut g = Game::new();
    // the opening job's been heard out - the slot's free for a begging one
    g.seats[0].quest = None;
    let folk = askers(&g);
    assert!(folk.len() >= 3, "enough folk to ask");
    let (giver, other) = (folk[0], folk[1]);
    g.pending_quests.push(pending(giver, QuestKind::Note, QuestTarget::Villager(other), None));
    // beside them, Talk opens on the offer: it leads the picks
    let gi = g.villagers.iter().position(|&v| v == giver).unwrap();
    let gp = *g.world.get_component::<Pos>(giver).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = gp.x; p.y = gp.y; }
    g.talk_to_at(0, gi);
    let t = g.seats[0].talk.as_ref().expect("the talk's open");
    assert!(t.job, "the offer latched onto the talk");
    let labels = t.choice_labels(&g.world);
    assert_eq!(labels.first().map(String::as_str), Some("About that job - I'll take it."),
        "the take-it pick leads: {:?}", labels);
    // the pick takes it: quest's the seat's, offer's off the list
    g.talk_pick_at(0, Some(0));
    assert!(g.seats[0].quest.is_some(), "the job should be under way");
    assert!(g.pending_quests.is_empty(), "and it's off the begging list");
    assert!(!g.seats[0].talk.as_ref().map(|t| t.job).unwrap_or(false),
        "the pick's spent - it leads no more");
    // the exchange's on the record once the talk's let go
    g.talk_pick_at(0, None);
    let lines: Vec<&str> = g.talk_log.last().expect("the talk's recorded")
        .lines.iter().map(|l| l.text.as_str()).collect();
    assert!(lines.iter().any(|l| l.contains("take it")),
        "your yes is on the record: {:?}", lines);
    assert!(lines.iter().any(|l| l.contains("note")),
        "their ask is on the record: {:?}", lines);
    // and the panel names the job as ever
    let m = g.ui_model();
    let line = m.panel.quest.expect("the panel should name the job");
    assert!(line.contains("note") && line.contains(&g.name_of(other)), "the line names the thing and the one it's for: {}", line);
}

/// The job pick only rides while there's an offer to take and hands
/// free to take it - otherwise the talk opens as ever
#[test]
fn test_the_job_pick_needs_a_live_offer_and_free_hands() {
    let mut g = Game::new();
    g.seats[0].quest = None;
    let folk = askers(&g);
    assert!(folk.len() >= 3, "enough folk to ask");
    let giver = folk[0];
    let gi = g.villagers.iter().position(|&v| v == giver).unwrap();
    let gp = *g.world.get_component::<Pos>(giver).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = gp.x; p.y = gp.y; }
    // no offer on the table: the talk's an ordinary one
    g.talk_to_at(0, gi);
    assert!(!g.seats[0].talk.as_ref().unwrap().job, "nothing to take, no job pick");
    g.talk_pick_at(0, None);
    // hands full: a live offer stays off the picks
    g.pending_quests.push(pending(giver, QuestKind::Egg, QuestTarget::Villager(giver),
        Some((g.chicken_at.0, g.chicken_at.1))));
    g.seats[0].quest = Some(PlayerQuest {
        giver: folk[1], kind: QuestKind::Note, target: QuestTarget::Villager(folk[2]),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    g.talk_to_at(0, gi);
    assert!(!g.seats[0].talk.as_ref().unwrap().job,
        "no free hands, no job pick");
}

#[test]
fn test_a_note_is_taken_to_whoever_it_is_for() {
    let mut g = Game::new();
    let folk = askers(&g);
    let (giver, other) = (folk[0], folk[1]);
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Note, target: QuestTarget::Villager(other),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    let gold_before = g.world.get_component::<Purse>(g.player).map(|p| p.gold).unwrap_or(0.0);
    let res_before = g.world.get_component::<PlayerCharacter>(g.player).map(|p| p.resources).unwrap_or(0.0);
    let who = g.villagers.iter().position(|&v| v == other).unwrap();
    // the real path: picked and beside them, Talk hands it over - and the
    // pay has to survive act_at's purse sync, which reads resources
    g.seats[0].picked = true;
    g.seats[0].selected_target = who;
    let op = *g.world.get_component::<Pos>(other).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = op.x; p.y = op.y; }
    g.act_at(0, 0);
    assert!(g.seats[0].quest.is_none(), "handing it over ends the job");
    let gold_after = g.world.get_component::<Purse>(g.player).map(|p| p.gold).unwrap_or(0.0);
    let res_after = g.world.get_component::<PlayerCharacter>(g.player).map(|p| p.resources).unwrap_or(0.0);
    assert!(gold_after > gold_before && res_after > res_before,
        "the pay lands in both purses: {:.1}->{:.1} gold, {:.1}->{:.1} resources",
        gold_before, gold_after, res_before, res_after);
    assert!(g.log.iter().any(|l| l.contains("reaches") || l.contains("remembers")),
        "the log says it got there: {:?}", g.log.iter().rev().take(4).collect::<Vec<_>>());
}

#[test]
fn test_an_egg_is_fetched_then_brought_back() {
    let mut g = Game::new();
    let folk = askers(&g);
    let giver = folk[0];
    // the egg lies by the hen run: stand on it and it's in hand
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Egg, target: QuestTarget::Villager(giver),
        stage: QuestStage::ToFetch, pickup: Some((g.chicken_at.0, g.chicken_at.1)),
        expires: g.tick + 10, errand: None });
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) {
        p.x = g.chicken_at.0; p.y = g.chicken_at.1;
    }
    g.update();
    assert_eq!(g.seats[0].quest.as_ref().map(|q| q.stage), Some(QuestStage::ToCarry),
        "standing on it picks it up");
    // and it's the giver's until it's talked back to them
    let who = g.villagers.iter().position(|&v| v == giver).unwrap();
    g.talk_to_at(0, who);
    assert!(g.seats[0].quest.is_none(), "brought back, the job's done");
}

#[test]
fn test_a_parcel_is_carried_out_to_the_far_village() {
    let mut g = Game::new();
    assert!(!g.sites.is_empty(), "the country has villages to send things to");
    let folk = askers(&g);
    let giver = folk[0];
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Parcel, target: QuestTarget::Site(0),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    let res_before = g.world.get_component::<PlayerCharacter>(g.player).map(|p| p.resources).unwrap_or(0.0);
    // stand on the far village's ground: that's the handover
    let (cx, cy) = g.sites[0].centre;
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) {
        p.x = cx as f32; p.y = cy as f32;
    }
    g.update();
    assert!(g.seats[0].quest.is_none(), "standing on their ground delivers it");
    let res_after = g.world.get_component::<PlayerCharacter>(g.player).map(|p| p.resources).unwrap_or(0.0);
    assert!(res_after > res_before, "the parcel's pay lands in resources too");
    assert!(g.log.iter().any(|l| l.contains("reaches")),
        "the log says the parcel landed: {:?}", g.log.iter().rev().take(4).collect::<Vec<_>>());
}

/// One job at a time: a second ask can't be taken while the first's
/// still under way - the offer waits
#[test]
fn test_only_one_job_at_a_time() {
    let mut g = Game::new();
    let folk = askers(&g);
    assert!(folk.len() >= 3, "enough folk to ask");
    let (giver, other) = (folk[0], folk[1]);
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Note, target: QuestTarget::Villager(other),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    // another folk's job going begging, picked and in earshot
    g.pending_quests.push(pending(folk[2], QuestKind::Egg, QuestTarget::Villager(folk[2]),
        Some((g.chicken_at.0, g.chicken_at.1))));
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == folk[2]).unwrap();
    let msg = g.accept_quest_at(0);
    assert!(msg.contains("already got a job"), "the refusal says why: {}", msg);
    let q = g.seats[0].quest.as_ref().expect("the first job's still under way");
    assert_eq!(q.giver, giver, "and it wasn't swapped out");
    assert_eq!(g.pending_quests.len(), 1, "the second ask still waits");
}

/// Doing a job for someone bumps them to known: the deed itself is a
/// piece of what you know of them, so even a giver with little to tell
/// reads as known once it's handed over
#[test]
fn test_a_finished_job_makes_them_known() {
    let mut g = Game::new();
    let folk = askers(&g);
    let (giver, other) = (folk[0], folk[1]);
    // a stranger to the seat before the job: no bond, nothing heard
    assert_eq!(crate::game::organizing::stage_of(&g.world, g.player, giver),
        crate::game::organizing::RelStage::Stranger);
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Note, target: QuestTarget::Villager(other),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    let who = g.villagers.iter().position(|&v| v == other).unwrap();
    g.seats[0].picked = true;
    g.seats[0].selected_target = who;
    let op = *g.world.get_component::<Pos>(other).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = op.x; p.y = op.y; }
    g.act_at(0, 0);
    assert!(g.seats[0].quest.is_none(), "handing it over ends the job");
    let stage = crate::game::organizing::stage_of(&g.world, g.player, giver);
    assert_eq!(stage, crate::game::organizing::RelStage::Known,
        "a job done for them reads as known, got {:?}", stage);
    // and the giver's card says so
    g.seats[0].picked = true;
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == giver).unwrap();
    let m = g.ui_model();
    let card = m.inspector.expect("the picked villager's card");
    assert!(card.lines.iter().any(|l| l.text.contains("you know them")),
        "the card names the stage: {:?}", card.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

/// The model carries where the job ends so the client's marker can
/// point the way: the far village's centre for a parcel, the thing's
/// spot while it's a fetch, the one it's for when it goes to a person
#[test]
fn test_the_model_names_where_the_job_ends() {
    let mut g = Game::new();
    g.seats[0].quest = None;
    assert_eq!(g.ui_model().quest_target, None, "no job, no marker");
    let folk = askers(&g);
    let (giver, other) = (folk[0], folk[1]);
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Parcel, target: QuestTarget::Site(0),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    let (cx, cy) = g.sites[0].centre;
    assert_eq!(g.ui_model().quest_target, Some((cx as f32, cy as f32)),
        "the parcel's marker is the far village's centre");
    // a fetch still under way points at the thing, not the giver
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Egg, target: QuestTarget::Villager(giver),
        stage: QuestStage::ToFetch, pickup: Some((g.chicken_at.0, g.chicken_at.1)),
        expires: g.tick + 10, errand: None });
    assert_eq!(g.ui_model().quest_target, Some((g.chicken_at.0, g.chicken_at.1)),
        "the fetch's marker is where the thing lies");
    // carried back to a person, it tracks them
    let op = *g.world.get_component::<Pos>(other).unwrap();
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Note, target: QuestTarget::Villager(other),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10, errand: None });
    assert_eq!(g.ui_model().quest_target, Some((op.x, op.y)),
        "the note's marker follows who it's for");
}

/// A fetch spot beside an occupied tile still counts: the pickup is the
/// passable tile's centre, and dawn's check has to find that tile, not
/// the blocked one the centre rounds up onto
#[test]
fn test_a_fetch_beside_a_blocked_tile_isnt_cancelled() {
    let mut g = Game::new();
    let folk = askers(&g);
    let giver = folk[0];
    // a stone's passable neighbour, if the boulders have one
    let spot = g.stones.iter().copied()
        .flat_map(|(sx, sy)| [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]
            .map(move |(dx, dy)| (sx + dx, sy + dy)))
        .find(|&(tx, ty)| g.map.passable(tx, ty));
    let Some((tx, ty)) = spot else { return };
    // the pickup sits at the tile's centre, as near_passable leaves it
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Stone, target: QuestTarget::Villager(giver),
        stage: QuestStage::ToFetch, pickup: Some((tx as f32 + 0.5, ty as f32 + 0.5)),
        expires: g.tick + 10, errand: None });
    let mut rng = Rng(7);
    g.quest_dawn(g.tick, &mut rng);
    assert!(g.seats[0].quest.is_some(),
        "the thing's still fetchable - dawn shouldn't call it a dead end");
}

// ---------------------------------------------------------------- E5-7

/// The ghost's own ask (E5-7, #1438): the strongest wound picks the
/// errand's shape - grief lays a loaf at the stones, debt fears ask who
/// keeps the roll, the hidden thing goes to the manor's sight, the
/// estranged get word carried, and the secret's a bundle held
#[test]
fn test_the_ghost_picks_the_errands_shape() {
    use crate::game::persona::*;
    let mut g = Game::new();
    let folk = askers(&g);
    let persona = |kind, strength| {
        let mut p = Persona {
            seed: 1, sex: villager::Sex::Male, archetype: Archetype::OldGuard,
            temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
            story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
            hidden: Landmark::Skill("y"), contradiction: None,
            idiolect: crate::game::idiolect::Idiolect {
                phrase: "x", habit: crate::game::idiolect::Habit::Echo },
        };
        p.ghosts.push(Ghost { event: EventId(0), kind, hook: None,
            lie: Belief::LordAlwaysWins, strength, fear: Fear::BailiffKnowsMyFace,
            need: Need::ToBeSafe, turned: false });
        p
    };
    let cases = [
        (EventKind::Fire, QuestKind::Deliver),
        (EventKind::DeathDuty, QuestKind::Deliver),
        (EventKind::MillerDebt, QuestKind::Ask),
        (EventKind::Levy, QuestKind::Ask),
        (EventKind::Eviction, QuestKind::FindOut),
        (EventKind::BrokenBetrothal, QuestKind::CarryWord),
        (EventKind::OldFaith, QuestKind::Keep),
    ];
    for (i, (kind, want)) in cases.iter().enumerate() {
        g.world.add_component(folk[i], persona(*kind, 0.8));
        let got = g.errand_offer(folk[i]);
        assert_eq!(got.map(|(k, _, _)| k), Some(*want),
            "{kind:?} should ask {want:?}, got {got:?}");
    }
    // the strongest wound wins the shape
    let mut p = persona(EventKind::Fire, 0.4);
    p.ghosts.push(Ghost { event: EventId(1), kind: EventKind::MillerDebt, hook: None,
        lie: Belief::LordAlwaysWins, strength: 0.9, fear: Fear::CollectorAtTheDoor,
        need: Need::ToSetItRight, turned: false });
    g.world.add_component(folk[7], p);
    assert_eq!(g.errand_offer(folk[7]).map(|(k, _, _)| k), Some(QuestKind::Ask),
        "the stronger ghost asks");
    // and no ghost means the plain jobs roll for them instead
    assert_eq!(g.errand_offer(folk[8]), None, "the unmarked get no errand");
}

/// E5-7's done-when: a completed errand can confirm a ledger fact - the
/// debt heard of becomes the debt seen in the roll
#[test]
fn test_a_finished_ask_confirms_what_was_heard() {
    let mut g = Game::new();
    let folk = askers(&g);
    let (giver, keeper) = (folk[0], folk[1]);
    let pe = g.player;
    // the seat heard of the debt as gossip - it stands at a rumour's weight
    crate::game::desires::record(&mut g.world, pe, giver, "ghost:miller_debt".into(),
        None, crate::game::desires::Source::Gossip { speaker: keeper }, g.tick);
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Ask, target: QuestTarget::Villager(keeper),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10,
        errand: Some(crate::session::quests::Errand { confirm: Some("miller_debt"), due: 0 }) });
    let who = g.villagers.iter().position(|&v| v == keeper).unwrap();
    g.seats[0].picked = true;
    g.seats[0].selected_target = who;
    let op = *g.world.get_component::<Pos>(keeper).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = op.x; p.y = op.y; }
    g.act_at(0, 0);
    assert!(g.seats[0].quest.is_none(), "the question's asked");
    let en = crate::game::desires::entries(&g.world, pe, giver)
        .into_iter().find(|e| e.fact == "ghost:miller_debt").expect("the fact stands");
    assert_eq!(en.source, crate::game::desires::Source::Witnessed,
        "the answer's a seen thing now, not hearsay");
    assert!(en.confidence >= 0.9, "the roll's word sits high: {:.2}", en.confidence);
}

/// The sighted roll confirms just the same: a Find out finished at the
/// castle writes the ghost-fact as witnessed even when nothing was heard
#[test]
fn test_a_sighted_roll_writes_the_fact_witnessed() {
    let mut g = Game::new();
    let castle = g.sites.iter().position(|s|
        s.kind == crate::session::country::SiteKind::Castle)
        .or(if g.sites.is_empty() { None } else { Some(0) }).expect("a site stands");
    let folk = askers(&g);
    let giver = folk[0];
    let pe = g.player;
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::FindOut, target: QuestTarget::Site(castle),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10,
        errand: Some(crate::session::quests::Errand { confirm: Some("eviction"), due: 0 }) });
    let (cx, cy) = g.sites[castle].centre;
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = cx as f32; p.y = cy as f32; }
    g.update();
    assert!(g.seats[0].quest.is_none(), "the roll was seen");
    let en = crate::game::desires::entries(&g.world, pe, giver)
        .into_iter().find(|e| e.fact == "ghost:eviction").expect("the sight wrote it");
    assert_eq!(en.source, crate::game::desires::Source::Witnessed);
}

/// One open errand a villager (E5-7): while the seat still runs theirs,
/// dawn's offers pass them by
#[test]
fn test_one_open_errand_a_villager() {
    let mut g = Game::new();
    let folk = askers(&g);
    let busy = folk[0];
    // everyone eligible, one already mid-job for the seat
    let pe = g.player;
    for &v in &folk {
        let mut rap = g.world.get_component::<crate::game::player::Rapport>(v)
            .cloned().unwrap_or_default();
        rap.by_player.insert(pe, 0.5);
        g.world.add_component(v, rap);
    }
    g.seats[0].quest = Some(PlayerQuest {
        giver: busy, kind: QuestKind::Keep, target: QuestTarget::Villager(busy),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10,
        errand: Some(crate::session::quests::Errand { confirm: None, due: g.tick + 4 }) });
    for s in 0..40 {
        let mut rng = Rng(s);
        g.quest_dawn(g.tick, &mut rng);
        assert!(g.pending_quests.iter().all(|q| q.giver != busy),
            "seed {s}: they don't beg while their errand runs");
    }
}

/// A Keep isn't handed back early (E5-7): held to its day, the bundle
/// returns - and the keeping proves integrity, not kindness
#[test]
fn test_a_held_bundle_comes_back_on_its_day() {
    let mut g = Game::new();
    let folk = askers(&g);
    let giver = folk[0];
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Keep, target: QuestTarget::Villager(giver),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10,
        errand: Some(crate::session::quests::Errand { confirm: None, due: g.tick + 3 }) });
    // before the day: they take nothing back
    assert!(g.quest_talk_done(0, giver).is_none(), "the bundle's still hid");
    assert!(g.seats[0].quest.is_some());
    g.tick += 3;
    assert!(g.quest_talk_done(0, giver).is_some(), "its day came - it goes back");
    assert!(g.seats[0].quest.is_none());
    // the keeping proved integrity: the bond's face that counts faith moved
    let bond = crate::game::relationships::find_relationship(&g.world, g.player, giver)
        .expect("the job made a bond");
    let rel = g.world.get_component::<crate::game::relationships::Relationship>(bond).unwrap();
    assert!(rel.trust.integrity > 0.1,
        "the kept bundle reads as faith kept: {:.2}", rel.trust.integrity);
}

/// A shaped errand's trust press is real warmth (review on #1558):
/// pushed after the aid's own push, it must raise the bond's total -
/// recompose would read it as overshoot and scale the parts back
#[test]
fn test_an_errands_press_lifts_the_bonds_warmth() {
    let mut g = Game::new();
    let folk = askers(&g);
    let (kept, plain) = (folk[0], folk[1]);
    g.quest_bond(0, kept, QuestKind::Keep);
    g.quest_bond(0, plain, QuestKind::Talk);
    let rel_of = |g: &Game, v| {
        let b = crate::game::relationships::find_relationship(&g.world, g.player, v)
            .expect("the job made a bond");
        let r = g.world.get_component::<crate::game::relationships::Relationship>(b).unwrap();
        (r.strength, r.intimacy, r.trust.ability, r.trust.benevolence, r.trust.integrity)
    };
    let (rk, rp) = (rel_of(&g, kept), rel_of(&g, plain));
    // the keeping proves its face and warms past a bare job done
    assert!(rk.4 > rp.4 + 0.1,
        "the press grew its face: {:.3} vs {:.3}", rk.4, rp.4);
    assert!(rk.0 > rp.0 + 0.1,
        "the press lifted the total: {:.3} vs {:.3}", rk.0, rp.0);
    // and it shaved nothing it didn't grow
    assert!((rk.3 - rp.3).abs() < 1e-3
        && (rk.2 - rp.2).abs() < 1e-3
        && (rk.1 - rp.1).abs() < 1e-3,
        "the press left the other faces alone: {rk:?} vs {rp:?}");
}

/// A bundle's days run from the handover (review on #1558): a posting
/// that sat on the board still gets its full four days to hold
#[test]
fn test_a_bundle_taken_late_holds_its_own_days() {
    let mut g = Game::new();
    let folk = askers(&g);
    let giver = folk[0];
    // the offer went up days ago - its due was set at the posting
    g.pending_quests.push(crate::session::quests::PendingQuest {
        giver, kind: QuestKind::Keep, target: QuestTarget::Villager(giver),
        pickup: None, expires: g.tick + 6,
        errand: Some(crate::session::quests::Errand { confirm: None, due: g.tick }) });
    let q = g.take_pending_quest(0, 0);
    assert_eq!(q.errand.map(|e| e.due), Some(g.tick + 4),
        "the keeping's days run from the taking, not the posting");
}

/// The roll question skips who's off it (review on #1558): a dead or
/// taken collector can't be sent to answer for the debt
#[test]
fn test_the_roll_question_skips_the_dead() {
    use crate::game::persona::*;
    let mut g = Game::new();
    let folk = askers(&g);
    let giver = folk[0];
    let mut p = Persona {
        seed: 1, sex: villager::Sex::Male, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::MillerDebt,
        hook: None, lie: Belief::LordAlwaysWins, strength: 0.9,
        fear: Fear::BailiffKnowsMyFace, need: Need::ToBeSafe, turned: false });
    g.world.add_component(giver, p);
    // the collector's gone - whoever the ask lands on must be answerable
    if let Some(dead) = g.villagers.iter().copied().find(|&v|
        crate::game::livelihood::has_role(&g.world, v,
            crate::game::vocation::TownRole::TaxCollector)) {
        g.world.add_component(dead, villager::Dead);
        if let Some((_, QuestTarget::Villager(t), _)) = g.errand_offer(giver) {
            assert!(t != dead, "the ask can't go to the dead collector");
            assert!(!crate::game::dungeon::unavailable(&g.world, t),
                "the ask lands on someone who can answer");
        }
    }
}
