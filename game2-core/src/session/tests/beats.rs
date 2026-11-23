use crate::session::*;
use crate::game::desires::{self, Problem};
use crate::sim::psychology::{AttachmentHistory, BeatKind, BeatMemory, Episode, EpisodeKind};

/// A grown villager - beats don't fire on kids
fn grown(g: &Game, skip: usize) -> Entity {
    g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .nth(skip).unwrap()
}

/// A villager the player knows, carrying a fresh bereavement for `dead`
fn bereaved(g: &mut Game, v: Entity, dead: Entity) {
    g.world.add_component(v, crate::game::player::Rapport::for_player(g.player, 0.5));
    desires::add_grievance(&mut g.world, v, Problem::Bereaved, g.tick + 60);
    g.world.add_component(v, AttachmentHistory {
        anxiety_episodes: 0, warmth_episodes: 0,
        episodes: vec![Episode { with: dead, kind: EpisodeKind::Loss }],
    });
}

fn beats_of(g: &Game, v: Entity, kind: BeatKind) -> usize {
    g.world.get_component::<BeatMemory>(v)
        .map(|m| m.of_kind.iter().filter(|(k, _)| *k == kind).count()).unwrap_or(0)
}

#[test]
fn test_the_bereaved_mourns_aloud_once_a_week() {
    let mut g = Game::new();
    let dead = grown(&g, 9);
    g.world.add_component(dead, villager::Dead);
    let v = grown(&g, 0);
    bereaved(&mut g, v, dead);
    let talks0 = g.talk_log.len();

    g.beat_tick(g.tick);
    let mem = g.world.get_component::<BeatMemory>(v).expect("the beat marked them");
    assert!(mem.of_kind.contains(&(BeatKind::Loss, g.tick)));
    assert_eq!(mem.lost, vec![dead], "the same dead won't beat twice");
    assert!(g.talk_log.len() > talks0, "the moment goes on their record");
    let name = g.name_of(dead);
    assert!(g.talk_log.last().unwrap().lines.iter().any(|l| l.text.contains(&name)),
        "the line names the one they lost: {:?}", g.talk_log.last().unwrap().lines);

    // the cap holds: the next day nominates again but the week's spent
    g.beat_tick(g.tick + 1);
    assert_eq!(beats_of(&g, v, BeatKind::Loss), 1,
        "at most one major beat per villager per in-game week");
    // and past the cap the SAME loss still doesn't beat a second time
    g.beat_tick(g.tick + 9);
    assert_eq!(beats_of(&g, v, BeatKind::Loss), 1);
}

#[test]
fn test_two_mourners_beat_in_the_same_week() {
    let mut g = Game::new();
    let dead = grown(&g, 9);
    g.world.add_component(dead, villager::Dead);
    let (a, b) = (grown(&g, 0), grown(&g, 1));
    bereaved(&mut g, a, dead);
    bereaved(&mut g, b, dead);
    g.beat_tick(g.tick);
    // the cap's per person: both may carry their grief aloud this week
    assert_eq!(beats_of(&g, a, BeatKind::Loss), 1);
    assert_eq!(beats_of(&g, b, BeatKind::Loss), 1);
}

#[test]
fn test_the_first_declaration_names_the_trouble() {
    let mut g = Game::new();
    let v = grown(&g, 0);
    g.world.add_component(v, crate::game::player::Rapport::for_player(g.player, 0.5));
    // the player's ear, lately: a talk or a meeting they were at
    let mut tc = crate::game::player::TalkCount::default();
    tc.last_day_by.insert(g.player, g.tick);
    g.world.add_component(v, tc);
    if let Some(l) = g.world.get_component_mut::<desires::InnerLife>(v) {
        l.grievances.clear();
    }
    desires::add_grievance(&mut g.world, v, Problem::Taxed, g.tick + 400);

    g.beat_tick(g.tick);
    assert_eq!(beats_of(&g, v, BeatKind::FirstDeclaration), 1);
    let line = g.talk_log.last().unwrap().lines.last().unwrap().text.clone();
    assert!(line.contains("the tax-man's tally"), "the trouble's named: {line}");

    // a first declaration is once per trouble - the week after, the
    // same grievance stays held (and their cap's still cooling anyway)
    g.beat_tick(g.tick + 9);
    assert_eq!(beats_of(&g, v, BeatKind::FirstDeclaration), 1);
}

#[test]
fn test_a_strangers_beats_dont_reach_the_player() {
    let mut g = Game::new();
    let dead = grown(&g, 9);
    g.world.add_component(dead, villager::Dead);
    let v = grown(&g, 0);
    // bereaved but unknown to the player: no rapport, no bond, no talk
    desires::add_grievance(&mut g.world, v, Problem::Bereaved, g.tick + 60);
    g.world.add_component(v, AttachmentHistory {
        anxiety_episodes: 0, warmth_episodes: 0,
        episodes: vec![Episode { with: dead, kind: EpisodeKind::Loss }],
    });
    g.beat_tick(g.tick);
    assert!(g.world.get_component::<BeatMemory>(v).is_none(),
        "a stranger's beat is noise the log doesn't want");
}
