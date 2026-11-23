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
    PendingQuest { giver, kind, target, pickup, expires: 100 }
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        expires: g.tick + 10 });
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
    let (cx, cy) = g.sites[0].centre;
    assert_eq!(g.ui_model().quest_target, Some((cx as f32, cy as f32)),
        "the parcel's marker is the far village's centre");
    // a fetch still under way points at the thing, not the giver
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Egg, target: QuestTarget::Villager(giver),
        stage: QuestStage::ToFetch, pickup: Some((g.chicken_at.0, g.chicken_at.1)),
        expires: g.tick + 10 });
    assert_eq!(g.ui_model().quest_target, Some((g.chicken_at.0, g.chicken_at.1)),
        "the fetch's marker is where the thing lies");
    // carried back to a person, it tracks them
    let op = *g.world.get_component::<Pos>(other).unwrap();
    g.seats[0].quest = Some(PlayerQuest {
        giver, kind: QuestKind::Note, target: QuestTarget::Villager(other),
        stage: QuestStage::ToCarry, pickup: None, expires: g.tick + 10 });
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
        expires: g.tick + 10 });
    let mut rng = Rng(7);
    g.quest_dawn(g.tick, &mut rng);
    assert!(g.seats[0].quest.is_some(),
        "the thing's still fetchable - dawn shouldn't call it a dead end");
}
