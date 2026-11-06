//! The odd jobs villagers ask the Wanderer to run: taken from the card,
//! the thing fetched where it lies, handed over by talking to whoever
//! it's for - or by standing on the far village's ground
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

#[test]
fn test_a_begging_job_shows_on_the_card_and_can_be_taken() {
    let mut g = Game::new();
    let folk = askers(&g);
    assert!(folk.len() >= 3, "enough folk to ask");
    let (giver, other) = (folk[0], folk[1]);
    g.pending_quests.push(pending(giver, QuestKind::Note, QuestTarget::Villager(other), None));
    // picked, and close enough to hear the ask, the card offers to take it
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == giver).unwrap();
    let gp = *g.world.get_component::<Pos>(giver).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = gp.x; p.y = gp.y; }
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::TakeJob),
        "the job should be on the card: {:?}", g.seats[0].ctx_actions);
    // taken: it's the seat's to run, and the panel says what it is
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::TakeJob).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.seats[0].quest.is_some(), "the job should be under way");
    assert!(g.pending_quests.is_empty(), "and it's off the begging list");
    let m = g.ui_model();
    let line = m.panel.quest.expect("the panel should name the job");
    assert!(line.contains("note") && line.contains(&g.name_of(other)), "the line names the thing and the one it's for: {}", line);
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
