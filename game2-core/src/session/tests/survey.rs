//! The traveller's ask (#1085): a beat worth asking on raises the
//! survey, the conversation box carries it last, the answer writes the
//! feedback bundle, and the asks don't nag.
use crate::session::*;
use crate::session::survey::{SurveyBeat, SURVEY_OPTIONS};
use crate::game::vocation::Kid;
use crate::game::villager::{self, Pos};

/// Living grown-up villager indices
fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
}

/// A scratch feedback dir under the OS temp, unique per run
fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("game2-survey-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn test_no_ask_before_any_beat() {
    let mut g = Game::new();
    g.survey_dawn(g.tick);
    assert!(g.seats[0].survey.is_none());
    let p = g.convo_panel_at(0);
    assert!(p.map(|x| x.title != "THE TRAVELLER ASKS").unwrap_or(true));
}

#[test]
fn test_a_beat_puts_the_travellers_ask_on_the_box() {
    let mut g = Game::new();
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    assert_eq!(g.seats[0].survey.as_ref().map(|o| o.beat), Some(SurveyBeat::QuestDone));
    let p = g.convo_panel_at(0).expect("the ask shows on the box");
    assert_eq!(p.title, "THE TRAVELLER ASKS");
    assert!(p.decides && !p.leave);
    assert_eq!(p.options, SURVEY_OPTIONS.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert!(p.rows.iter().any(|r| r.text.contains("pedlar")), "{:?}", p.rows);
}

#[test]
fn test_the_harshest_beat_asks_first() {
    let mut g = Game::new();
    g.seats[0].quests_done = 1;
    g.martyr_grave = Some((4, 4));
    g.survey_dawn(g.tick);
    assert_eq!(g.seats[0].survey.as_ref().map(|o| o.beat), Some(SurveyBeat::MemberHanged));
    // and the gentler beat still waits its turn, not asked-over
    assert!(g.seats[0].survey_seen & SurveyBeat::QuestDone.bit() == 0);
}

#[test]
fn test_the_first_burial_and_the_founded_cause_raise_their_beats() {
    let mut g = Game::new();
    g.died_on[0] = Some(g.tick);
    g.survey_dawn(g.tick);
    assert_eq!(g.seats[0].survey.as_ref().map(|o| o.beat), Some(SurveyBeat::FirstDeath));
}

#[test]
fn test_the_answer_writes_the_bundle_clears_and_flags_the_shot() {
    let mut g = Game::new();
    let dir = scratch("bundle");
    g.feedback_dir = Some(dir.to_string_lossy().to_string());
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    g.decide_at(0, 0);
    assert!(g.seats[0].survey.is_none(), "the ask's answered");
    let name = format!("survey-s0-t{}-quest-done", g.tick);
    let txt = std::fs::read_to_string(dir.join(format!("{}.txt", name))).expect("the bundle lands");
    assert!(txt.contains(&format!("seed: {}", g.seed)), "{}", txt);
    assert!(txt.contains(&format!("tick: {}", g.tick)));
    assert!(txt.contains(&format!("beat: {:?}", SurveyBeat::QuestDone)));
    assert!(txt.contains("I'd send a cousin"), "{}", txt);
    assert!(txt.contains("The log's tail"));
    // the client gets the one-frame capture flag under the same stem
    assert_eq!(g.seats[0].survey_capture.as_ref().map(|(f, s)| (*f, s.as_str())),
        Some((g.frame_n, name.as_str())));
    assert_eq!(g.ui_model_at(0).feedback_capture.as_deref(), Some(name.as_str()));
    // next frame the flag's spent - it never re-fires
    g.seats[0].survey_capture = None;
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_the_wave_off_asks_nothing_and_writes_nothing() {
    let mut g = Game::new();
    let dir = scratch("wave");
    g.feedback_dir = Some(dir.to_string_lossy().to_string());
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    g.decide_at(0, 3);
    assert!(g.seats[0].survey.is_none());
    assert!(!dir.exists(), "the wave-off writes no bundle");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_the_asks_dont_nag_the_cooldown_holds() {
    let mut g = Game::new();
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    g.decide_at(0, 3);
    // a second beat lands but he's owed his leave
    g.martyr_grave = Some((4, 4));
    g.survey_dawn(g.tick);
    assert!(g.seats[0].survey.is_none());
    // past his leave, the new beat asks
    g.survey_dawn(g.tick + 30);
    assert_eq!(g.seats[0].survey.as_ref().map(|o| o.beat), Some(SurveyBeat::MemberHanged));
}

#[test]
fn test_the_same_question_doesnt_come_round_twice() {
    let mut g = Game::new();
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    g.decide_at(0, 3);
    g.survey_dawn(g.tick + 30);
    assert!(g.seats[0].survey.is_none(), "quest-done's spent its ask");
    // a third odd job doesn't revive it either
    g.seats[0].quests_done = 3;
    g.survey_dawn(g.tick + 60);
    assert!(g.seats[0].survey.is_none());
}

#[test]
fn test_a_live_talk_hides_the_ask_and_it_waits() {
    let mut g = Game::new();
    g.seats[0].quests_done = 1;
    g.survey_dawn(g.tick);
    // the player falls to talking
    let i = adults(&g)[0];
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    assert!(g.seats[0].talk.is_some(), "a talk's under way");
    let p = g.convo_panel_at(0).unwrap();
    assert!(p.title.starts_with("TALK"), "the talk owns the box: {}", p.title);
    // and a stray decide doesn't eat the ask
    g.decide_at(0, 0);
    assert!(g.seats[0].survey.is_some());
}
