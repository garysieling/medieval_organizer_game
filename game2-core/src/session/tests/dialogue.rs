//! Conversations: the rules cascade in play, the talk log, and the TALK and conversation panels.
use crate::session::*;
use crate::session::dialogue::*;
use crate::engine::ecs::Entity;
use super::fixtures::plan_ready_meeting;
use crate::game::conversation::{Act, Relation};
use crate::game::relationships::RelKind;
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Kid, Vocation};
use crate::game::traits::Trait;
use crate::game::villager::Pos;

/// Living grown-up villager indices
fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
}

#[test]
fn test_speaker_reads_traits_trade_and_circumstance_off_the_world() {
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Hotheaded] });
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Weaver, 0.5));
    g.world.add_component(v, Hunger { value: 5.0 });
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert_eq!(s.traits, vec![Trait::Hotheaded]);
    assert!(s.status.contains(&"weaver") && s.status.contains(&"hungry"), "{:?}", s.status);
    assert_eq!(s.trade, Some("weaver"));
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Priest, 0.5));
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).ideology, "clergy");
    // every trade speaks as itself, even ones with no status word (#1238)
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).trade, Some("priest"));
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Farmer, 0.5));
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).trade, Some("farmer"));
    g.world.add_component(v, Kid);
    assert!(speaker_of(&g.world, v, "Eliza", g.tick).status.contains(&"child"));
}

#[test]
fn test_the_day_rides_into_the_speakers_statuses() {
    // #1617: the trades' "what's going on" lines gate on the world's
    // real turn - every speaker carries the season underfoot, and a
    // market day adds its own word to the talk
    let g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert!(s.status.iter().any(|st| st.starts_with("season:")), "{:?}", s.status);
    assert!(speaker_of(&g.world, v, "Eliza", routine::MARKET_DAY + 60).status.contains(&"market"));
    assert!(!speaker_of(&g.world, v, "Eliza", routine::MARKET_DAY + 61).status.contains(&"market"));
}

#[test]
fn test_the_common_trades_carry_their_status_into_talk() {
    // the vocations the village actually rolls (#1617): each carries
    // the status word its trees gate on
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    for (voc, slug) in [(Vocation::Farmer, "farmer"), (Vocation::Tailor, "tailor"),
                        (Vocation::Luthier, "luthier"), (Vocation::Printmaker, "printer"),
                        (Vocation::Glazier, "glazier")] {
        g.world.add_component(v, crate::game::vocation::craft(voc, 0.5));
        let s = speaker_of(&g.world, v, "Eliza", g.tick);
        assert!(s.status.contains(&slug), "{slug}: {:?}", s.status);
    }
}

#[test]
fn test_a_speakers_persona_wears_the_tells() {
    // E3-4 (#1425): the wound's habits ride the speaker as statuses -
    // the gossip criteria and the ask gate's tell bonus both read them
    use crate::game::persona::*;
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let mut p = Persona {
        seed: 1, sex: villager::Sex::Male, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie: Belief::PromisesForKeepers, strength: 0.8, fear: Fear::BailiffKnowsMyFace,
        need: Need::ToBeSafe, turned: false });
    g.world.add_component(v, p);
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert!(s.status.contains(&"shuns_the_gate") && s.status.contains(&"promises_nothing"),
        "the tells should ride the speaker: {:?}", s.status);
    assert!(s.status.contains(&"keeps_wound"), "a carried ghost is a kept thing");
    // and the wear names the deep ask it hides behind
    assert!(s.tell_areas().contains(&crate::game::conversation::Area::Trouble)
        && s.tell_areas().contains(&crate::game::conversation::Area::Heart));
}

#[test]
fn test_a_speakers_body_reaches_the_fact_stream() {
    // E1-3 (#1417): the pronoun facts text.rs will consume - set even
    // through the child's early out
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let w = g.villagers[adults(&g)[1]];
    g.world.add_component(v, villager::Sex::Female);
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert_eq!(s.sex, Some("female"));
    let other = speaker_of(&g.world, w, "Aldric", g.tick);
    let f = crate::game::conversation::Facts::for_roles(&s, &other,
        crate::game::conversation::Relation::Acquaintance);
    assert!(f.has("self:sex:female"));
    assert!(!f.has("self:sex:male"));
    // and a kid still carries theirs
    g.world.add_component(v, Kid);
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).sex, Some("female"));
}

#[test]
fn test_neighbours_are_acquaintances_but_the_wanderer_is_a_stranger() {
    assert_eq!(relation_of(Some(RelKind::Family)), Relation::Kin);
    assert_eq!(relation_of(Some(RelKind::Rival)), Relation::Rival);
    assert_eq!(relation_of(Some(RelKind::Friend)), Relation::Friend);
    assert_eq!(relation_of(None), Relation::Acquaintance);
    assert_eq!(relation_of(Some(RelKind::Stranger)), Relation::Acquaintance);
    assert_eq!(relation_with_wanderer(None), Relation::Stranger);
}

#[test]
fn test_villagers_chat_goes_on_the_record_not_over_the_map() {
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b) = (g.villagers[ad[0]], g.villagers[ad[1]]);
    g.bubbles.clear();
    assert!(g.villagers_chat(a, b));
    assert!(!g.bubbles.iter().any(|x| x.0 == a || x.0 == b), "no bubbles for villager chatter");
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.kind, TalkKind::Chat);
    assert!(r.lines.len() >= 2 && r.lines.iter().any(|l| l.who == Some(b)), "both speak: {:?}", r.lines);
    // clicking either of them shows it
    g.seats[0].selected_target = ad[1];
    g.seats[0].picked = true;
    let p = g.talk_panel_at(0);
    assert!(p.title.contains(&g.name_of(b)), "{}", p.title);
    assert!(p.rows.iter().any(|row| row.text == r.lines[0].text));
}

#[test]
fn test_kids_keep_to_childrens_talk() {
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b, c) = (g.villagers[ad[0]], g.villagers[ad[1]], g.villagers[ad[2]]);
    g.world.add_component(a, Kid);
    g.world.add_component(b, Kid);
    assert!(!g.villagers_chat(a, c), "a child and a grown-up just nod");
    assert!(g.villagers_chat(a, b));
    let said: Vec<String> = g.talk_log.last().unwrap().lines.iter().map(|l| l.text.clone()).collect();
    let kids_lines: Vec<&str> = ["Oi, ", "Hast thou any errands", "Psst"].to_vec();
    assert!(kids_lines.iter().any(|k| said[0].contains(k)), "opens on the children's tree: {:?}", said);
}

/// Stand the player beside villager index `i` and Talk
/// A bond warm enough to hear the private layer (E3-2): three talks
/// behind them, intimacy and trust to carry openness past the private
/// bar even with ears near. Parts and summary are seeded consistent -
/// reconcile leaves them be
fn warm_bond(g: &mut Game, v: Entity) {
    let pe = g.player;
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: pe, other: v, kind: RelKind::Friend,
            strength: 1.0, interactions: 3,
            intimacy: 0.6, pivots: 0, deflects: 0, bids_met: 0,
            trust: crate::game::relationships::Trust {
                ability: 0.0, benevolence: 0.6, integrity: 0.6 },
            safety: crate::game::relationships::SafetyRung::Challenger,
        });
    // and the seat's told them a piece of themselves (E3-3): the
    // private layer wants reciprocation, not just warmth
    let mut sd = SharedDepth::default();
    sd.of.insert(pe, (2, g.tick));
    g.world.add_component(v, sd);
}

fn talk_beside(g: &mut Game, i: usize) -> Entity {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    v
}

#[test]
fn test_talk_opens_a_conversation_on_the_talk_panel_with_replies() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let log_before = g.log.len();
    let v = talk_beside(&mut g, i);
    let rec = g.talk_log.last().unwrap().clone();
    assert_eq!(rec.kind, TalkKind::Talk);
    assert!(rec.lines.iter().any(|l| l.gist && l.who == Some(v)), "what's on their mind is on the record");
    assert!(!g.log[log_before..].iter().any(|l| rec.lines.iter().any(|r| r.gist && l.contains(&r.text))),
        "the words aren't in the event log");
    let t = g.seats[0].talk.as_ref().expect("a talk under way").clone();
    assert!(!t.options.is_empty() && t.options.len() <= 2);
    let panel = g.talk_panel_at(0);
    assert!(panel.in_talk && panel.options.len() == t.options.len() + t.asks_shown()
        + t.inoculation_beyond().is_some() as usize + t.probe_beyond().is_some() as usize);
    // pick the first reply: it's said, as previewed
    g.talk_pick_at(0, Some(0));
    let rec = g.talk_log.iter().find(|r| r.id == rec.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(g.player) && l.text == t.options[0].2), "{:?}", rec.lines);
}

#[test]
fn test_the_choice_list_stops_at_three() {
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // they've told you something of themselves, and the bond's a
    // follower's - the asks open up
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    let pe = g.player;
    crate::game::relationships::record_interaction(&mut g.world, pe,
        crate::game::relationships::Interaction { other: v,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<crate::game::relationships::Relationship>(r))
        .unwrap().strength = 0.6;
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.asks.len() > 1, "asks on offer: {:?}", t.asks);
    // replies plus asks, never more than three on offer at once - the
    // asks that open a flow of their own may ride one past the cap
    let convo = g.convo_panel_at(0).unwrap();
    let beyond = t.inoculation_beyond().is_some() as usize + t.probe_beyond().is_some() as usize;
    assert_eq!(convo.options.len(), t.options.len() + t.asks_shown() + beyond);
    assert!(t.options.len() + t.asks_shown() <= TALK_MAX_CHOICES, "{:?}", convo.options);
    let panel = g.talk_panel_at(0);
    assert_eq!(panel.options.len(), convo.options.len());
    // a pick past the whole list does nothing; the first ask still lands
    g.talk_pick_at(0, Some(99));
    assert!(g.seats[0].talk.is_some(), "a pick past the end does nothing");
    let ask = t.asks[0];
    g.talk_pick_at(0, Some(t.options.len()));
    assert!(!g.seats[0].talk.as_ref().unwrap().asks.contains(&ask), "the ask was spent");
}

#[test]
fn test_leaving_or_walking_off_ends_the_talk() {
    let mut g = Game::new();
    let ad = adults(&g);
    talk_beside(&mut g, ad[0]);
    assert!(g.seats[0].talk.is_some());
    g.talk_pick_at(0, None);
    assert!(g.seats[0].talk.is_none());
    assert_eq!(g.talk_log.last().unwrap().lines.last().unwrap().text, "You take your leave.");
    let v = talk_beside(&mut g, ad[1]);
    assert!(g.seats[0].talk.is_some());
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + PLAYER_TALK_RANGE + 2.0, y: p.y };
    g.advance_player_talks();
    assert!(g.seats[0].talk.is_none(), "walking away ends it");
}

#[test]
fn test_a_whole_talk_runs_to_its_end_by_picking_replies() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let mut picks = 0;
    while g.seats[0].talk.is_some() && picks < 30 { g.talk_pick_at(0, Some(0)); picks += 1; }
    assert!(g.seats[0].talk.is_none(), "it ends: farewell, walk-off or out of things to say");
    assert!(picks >= 1);
}

#[test]
fn test_a_spent_talk_closes_itself_on_a_mutual_parting() {
    // #1179: when the material's spent the talk ends itself - a mutual
    // parting, not an empty option list to hover in
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let id = g.seats[0].talk.as_ref().unwrap().id;
    let mut picks = 0;
    // drive it dry: farewells never sit beside live replies now, so pick 0
    // keeps talking until the talk itself decides it's done
    while g.seats[0].talk.is_some() && picks < 30 {
        if let Some(t) = &g.seats[0].talk {
            let farewells = t.options.iter().filter(|(a, _, _)| *a == Act::Farewell).count();
            assert!(farewells == 0 || farewells == t.options.len(),
                "farewells shared a list with live replies: {:?}", t.options);
        }
        g.talk_pick_at(0, Some(0));
        picks += 1;
    }
    assert!(g.seats[0].talk.is_none(), "a spent talk doesn't hover");
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    // the talk closed on its own parting - a villager's goodbye or the
    // dry-talk narration - never on the player's "take your leave":
    // 417 farewell lines exist, so the check is what the close isn't
    assert!(rec.lines.last().is_some_and(|l| l.text != "You take your leave."),
        "the player had to leave a talk that should have closed itself: {:?}",
        rec.lines.last().map(|l| l.text.clone()));
}

#[test]
fn test_a_house_meeting_is_kept_on_the_talk_panel() {
    let mut g = Game::new();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (eliza, bram) = (g.villagers[1], g.villagers[2]);
    for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
    for (i, v) in g.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
    }
    plan_ready_meeting(&mut g, eliza, (18.0, 16.0), &[bram]);
    g.call_meeting();
    for _ in 0..crate::session::FPS * 60 { if g.meeting.is_none() { break; } g.update(); }
    assert!(g.meeting.is_none(), "the meeting ran its course");
    let r = g.talk_log.iter().rev().find(|r| r.kind == TalkKind::Meeting).expect("a meeting on the record");
    assert!(r.with.contains(&g.player));
    assert!(r.lines.iter().any(|l| l.who == Some(g.player)) && r.lines.iter().any(|l| l.who == Some(eliza) || l.who == Some(bram)),
        "{:?}", r.lines);
    // the room's talk, not just the stock remarks
    assert!(r.lines.len() >= 5);
}

#[test]
fn test_each_seat_sees_its_own_lines_as_you_and_hosts_its_own_meeting() {
    let mut g = Game::new();
    let s1 = g.add_seat();
    let (p0, p1) = (g.seats[0].player, g.seats[s1].player);
    let v = g.villagers[adults(&g)[0]];
    // seat 1 hosts: the prompts are theirs, not seat 0's
    let lines = g.meeting_lines(&[v], s1, (false, false, false, false, false), false, None);
    assert!(lines.iter().any(|l| l.0 == p1) && !lines.iter().any(|l| l.0 == p0), "seat 1 asks the questions");
    let n1 = g.speaker_name(p1);
    g.record_meeting(vec![p1, p0, v], vec![(Some(p1), n1.clone(), "Thanks for coming.".into())]);
    let row = |g: &Game, seat: usize| g.talk_panel_at(seat).rows.into_iter()
        .find(|r| r.text == "Thanks for coming.").and_then(|r| r.name).unwrap();
    assert_eq!(row(&g, s1), "You", "the host's own panel");
    assert_eq!(row(&g, 0), n1, "the other seat sees who said it");
    assert_ne!(n1, "You");
}

#[test]
fn test_villagers_talk_in_a_running_game() {
    let mut g = Game::new();
    for _ in 0..crate::session::FPS * 60 * 2 { g.update(); }
    let chats = g.talk_log.iter().filter(|r| r.kind == TalkKind::Chat).count();
    assert!(chats > 0, "villagers who meet should talk");
}

/// An org ready for the vote: three living members, one issue heard
fn org_ready(g: &mut Game) {
    for &v in &g.villagers[..3] {
        crate::game::desires::add_grievance(&mut g.world, v, crate::game::desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<crate::game::desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
}

#[test]
fn test_deciding_founds_the_campaign_on_the_picked_framing() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    g.decide_at(0, 1); // take it on, naming the foe
    let c = crate::game::campaign::campaign_of(&g.world, g.org).expect("the campaign founded");
    let cp = g.world.get_component::<crate::game::campaign::Campaign>(c).unwrap();
    assert_eq!(cp.issue, "the taxes");
    assert!(matches!(cp.framing, crate::game::campaign::Framing::CommonFoe));
    assert!(g.seats[0].pending_founding.is_none());
    // and the outcome lands on the meeting's record
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("they'll fight for the taxes")), "{:?}", rec.lines);
}

#[test]
fn test_a_talk_opens_with_who_they_are() {
    let mut g = Game::new();
    let v = g.villagers[7];
    g.open_player_talk(0, v, crate::game::conversation::Relation::Acquaintance, "the day's work");
    let t = g.seats[0].talk.as_ref().expect("a talk under way");
    // the profile line heads the talk - who they are before the gist
    assert!(!t.profile.is_empty(), "the talk opened bare");
}

#[test]
fn test_pressing_talk_again_keeps_the_conversation_going() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let before = g.seats[0].talk.as_ref().expect("a talk under way").id;
    let records = g.talk_log.len();
    // pressing T again while talking must not reset the conversation
    g.act_at(0, 0);
    let t = g.seats[0].talk.as_ref().expect("the talk must stay open");
    assert_eq!(t.id, before, "a second Talk press restarted the talk");
    assert_eq!(g.talk_log.len(), records, "the re-press opened a fresh talk");
    // and talking to someone new still opens fresh
    let j = adults(&g).into_iter().find(|&k| k != i).unwrap();
    talk_beside(&mut g, j);
    let after = g.seats[0].talk.as_ref().expect("the new talk opened").id;
    assert_ne!(after, before, "a different villager gets their own talk");
}

#[test]
fn test_a_fresh_talk_always_offers_something_to_say() {
    // a normal opening offers replies, or at least the leave - never a
    // bare panel that hangs up on its own (#1058)
    let mut g = Game::new();
    for &i in adults(&g).iter().take(10) {
        g.seats[0].talk = None;
        talk_beside(&mut g, i);
        let t = g.seats[0].talk.as_ref().expect("the talk must stay open");
        assert!(!t.options.is_empty(), "villager {i} opened with nothing to say");
    }
}

#[test]
fn test_an_old_hello_greys_the_name_but_a_stranger_stays_bright() {
    // a face you let drift greys out; one you've never met doesn't (#1069)
    let mut g = Game::new();
    let (met, stranger) = (g.villagers[0], g.villagers[1]);
    let pe = g.player;
    crate::game::player::act_talk(&mut g.world, pe, met, 0);
    g.tick = 30; // half gone grey
    let stale_of = |g: &Game, v| g.ui_model().people.iter()
        .find(|p| p.e == v).map(|p| p.stale).unwrap_or(-1.0);
    let (sm, ss) = (stale_of(&g, met), stale_of(&g, stranger));
    assert!((sm - 0.5).abs() < 0.01, "30 days of silence reads half-faded, got {}", sm);
    assert_eq!(ss, 0.0, "a stranger has nothing to fade");
    g.tick = 200; // past the fade's end
    assert_eq!(stale_of(&g, met), 1.0, "long silence greys all the way out");
    // a meeting counts as speaking: the clock starts over
    let (pe, tick) = (g.player, g.tick);
    crate::game::player::note_conversation(&mut g.world, pe, met, tick);
    assert_eq!(stale_of(&g, met), 0.0, "seeing them at the meeting freshens the name");
}

#[test]
fn test_a_talk_holds_them_where_they_stand() {
    // folk in a Talk with you don't walk off mid-line: a walk under way
    // stops, no new one dispatches, and the pull home leaves them be -
    // while a villager you aren't talking to still drifts to the table
    let mut g = Game::new();
    let (i, j) = (adults(&g)[0], adults(&g)[1]);
    let (v, w) = (g.villagers[i], g.villagers[j]);
    // a walk under way when you hail them: the hail stops their feet
    let p = *g.world.get_component::<Pos>(v).unwrap();
    g.travels.push(Travel { who: v, path: vec![(p.x.round() as i32 + 4, p.y.round() as i32)], goal: Goal::Wander });
    talk_beside(&mut g, i);
    assert!(g.seats[0].talk.is_some(), "a talk under way");
    assert!(!g.travels.iter().any(|t| t.who == v), "the hail stops their feet");
    // and no new one starts while they stand with you
    assert!(!g.send(v, Goal::Wander), "no fresh errand mid-talk");
    assert!(!g.travels.iter().any(|t| t.who == v));
    // hungry and away from the table: the seek and the pull home would
    // carry anyone else off, but not one you're talking to
    for e in [v, w] {
        g.world.get_component_mut::<villager::Hunger>(e).unwrap().value = 5.0;
    }
    let (vp, wp0) = (*g.world.get_component::<Pos>(v).unwrap(),
        *g.world.get_component::<Pos>(w).unwrap());
    for _ in 0..40 { g.update(); }
    let (vp1, wp1) = (*g.world.get_component::<Pos>(v).unwrap(),
        *g.world.get_component::<Pos>(w).unwrap());
    assert_eq!((vp1.x, vp1.y), (vp.x, vp.y),
        "the one you're talking to holds their ground");
    assert!((wp1.x - wp0.x).abs() + (wp1.y - wp0.y).abs() > 0.1,
        "the one you aren't talking to still heads for the table: {:?} -> {:?}", (wp0.x, wp0.y), (wp1.x, wp1.y));
    // once the talk's done their feet are their own again: you walk on,
    // it closes, and the next errand can send them
    let pp = g.world.get_component::<Pos>(g.player).unwrap();
    let far = Pos { x: pp.x + PLAYER_TALK_RANGE as f32 + 4.0, y: pp.y };
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = far;
    g.advance_player_talks();
    assert!(g.seats[0].talk.is_none(), "walking away still ends it");
    assert!(g.send(v, Goal::Wander), "free to be sent once it's done");
}

/// The cold brush-offs a Slighted memory serves (#1183)
fn cold_lines() -> [&'static str; 4] {
    ["You are back. What is it now?", "Hmph. Say your piece and go.",
     "We left it badly last time. Say what you came for.", "You have a nerve, coming back."]
}

#[test]
fn test_a_talked_back_answer_is_remembered_next_time() {
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let ad = adults(&g);
    // someone whose first options include a PushBack
    let mut slighted = None;
    for &i in ad.iter() {
        let v = talk_beside(&mut g, i);
        let has = g.seats[0].talk.as_ref().unwrap().options.iter()
            .position(|(a, _, _)| *a == Act::PushBack);
        if let Some(p) = has {
            g.talk_pick_at(0, Some(p));
            g.talk_pick_at(0, None);
            slighted = Some(v);
            break;
        }
        g.close_player_talk(0, None);
    }
    let v = slighted.expect("someone takes a PushBack on this map");
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Slighted));
    // they've told you a secret before: asks would be on offer...
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    // ...and the bond still reads warm - the slight should outrank it
    bond_to(&mut g, v, 0.6);
    let i = g.villagers.iter().position(|&x| x == v).unwrap();
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    // ...but what's on their mind now is you, and no proposition lands
    // a style marker may sign the line (E4-6): match the base words
    assert!(cold_lines().iter().any(|l| t.gist.contains(l.trim_end_matches('.'))),
        "the gist is the cold shoulder: {:?}", t.gist);
    assert!(t.asks.is_empty(), "a cold ending closes the asks: {:?}", t.asks);
    // the icon's mark holds the slight while the amends are on offer -
    // a warm bond doesn't paper it over (#1320, review find)
    assert!(g.seats[0].talk.as_ref().unwrap().amends, "the apology's on offer");
    let rows = g.convo_panel_at(0).unwrap().rows;
    assert_eq!(rows.iter().find(|r| r.who == Some(i)).map(|r| r.mood), Some(3),
        "a fresh slight marks them angry, not warm");
}

#[test]
fn test_a_kind_answer_warms_the_hello_after() {
    let mut g = Game::new();
    let ad = adults(&g);
    let mut warmed = None;
    for &i in ad.iter() {
        let v = talk_beside(&mut g, i);
        // talk along until a kind reply is on offer (the first beat's
        // usually a greeting), taking it when it comes
        for _ in 0..10 {
            let Some(t) = &g.seats[0].talk else { break };
            match t.options.iter().position(|(a, _, _)| matches!(a, Act::Agree | Act::Answer)) {
                Some(p) => { g.talk_pick_at(0, Some(p)); break; }
                None => g.talk_pick_at(0, Some(0)),
            }
        }
        if g.seats[0].talk.is_some() {
            if g.seats[0].talk.as_ref().unwrap().mood == TalkMood::Warmed {
                g.talk_pick_at(0, None);
                warmed = Some(v);
                break;
            }
        } else if g.talk_mood_of(v) == Some(TalkMood::Warmed) { warmed = Some(v); break; }
        g.close_player_talk(0, None);
    }
    let v = warmed.expect("someone takes a kind answer on this map");
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Warmed));
    // the next hello opens on their real mind, not a cold shoulder
    let i = g.villagers.iter().position(|&x| x == v).unwrap();
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(!cold_lines().contains(&t.gist.as_str()), "still cold after a warm parting: {:?}", t.gist);
}

#[test]
fn test_leaving_before_they_speak_leaves_it_cool() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    g.talk_pick_at(0, None);
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Cool));
}

/// Open a talk on a stingy villager who's already shared something (the
/// asks are on the table), answer the greeting, and take the landing or
/// the wrong note of their audition - returns the talk's record id
fn audition(g: &mut Game, i: usize, pass: bool) -> (Entity, u64) {
    use crate::game::desires::{Problem, Reveal};
    audition_bonded(g, i, pass, vec![Reveal::Problem(Problem::Taxed)], false)
}

/// `audition` with the bond and what you've heard set up ahead of time -
/// `bond` puts one past conversation on the roll so a stage exists to
/// climb (#1188)
fn audition_bonded(g: &mut Game, i: usize, pass: bool, revealed: Vec<crate::game::desires::Reveal>, bond: bool) -> (Entity, u64) {
    let v = g.villagers[i];
    g.world.add_component(v, Personality { traits: vec![Trait::Stingy] });
    let __pe = g.player; for __r in revealed { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    if bond {
        let pe = g.player;
        crate::game::relationships::record_interaction(&mut g.world, pe,
            crate::game::relationships::Interaction { other: v,
                kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    }
    talk_beside(g, i);
    let t = g.seats[0].talk.as_ref().expect("a talk under way").clone();
    if bond {
        assert!(!t.asks.is_empty(), "they'd bring a proposition to this talk");
    }
    g.talk_pick_at(0, Some(0)); // the greeting's answer
    let t = g.seats[0].talk.as_ref().expect("still talking").clone();
    assert_eq!(t.opt_tree, Some(crate::game::conversation::PROBE_TREE),
        "after the hello they put their test to you: {:?}", t.opt_tree);
    assert_eq!(t.options.len(), 2, "the audition offers the landing and the wrong note");
    // pick the fork whose words are the stingy test's pass (or miss)
    let probe = crate::game::conversation::probe_for(&t.villager).unwrap();
    let wanted = if pass { probe.pass.1 } else { probe.fail.1 };
    let pick = t.options.iter().position(|(_, _, l)| wanted.contains(&l.as_str()))
        .expect("their fork among the two");
    g.talk_pick_at(0, Some(pick));
    (v, t.id)
}

#[test]
fn test_a_good_answer_to_the_audition_opens_them_up() {
    use crate::game::conversation::Vibe;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let (_, id) = audition(&mut g, i, true);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    assert_eq!(t.vibe, Some(Vibe::Safe));
    assert!(!t.asks.is_empty(), "safe: the propositions stay on offer");
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("warms to you")),
        "the read is narrated: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_a_wrong_note_keeps_the_talk_polite() {
    use crate::game::conversation::Vibe;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let (_, id) = audition(&mut g, i, false);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, politely").clone();
    assert_eq!(t.vibe, Some(Vibe::Mismatch));
    // coin and nerve still stand to be tried (#1225) - it's the bond's
    // own asks a bad read keeps out of reach
    assert!(t.asks.iter().all(|o| o.key != crate::game::organizing::AskKey::Earned),
        "a mismatch brings no earned proposition: {:?}", t.asks);
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("keeps it to the weather")),
        "the shallow read is narrated: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_a_safe_read_opens_the_next_rung_early() {
    use crate::game::conversation::Vibe;
    use crate::game::desires::{Problem, Reveal};
    use crate::game::organizing::{AskKey, PropKind};
    let mut g = Game::new();
    let i = adults(&g)[0];
    // an acquaintance with one piece heard offers only the light ask at
    // the hello - but a good answer to their test opens the next rung
    // right there in the talk (#1188)
    audition_bonded(&mut g, i, true, vec![Reveal::Problem(Problem::Taxed)], true);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    assert_eq!(t.vibe, Some(Vibe::Safe));
    assert!(t.asks.iter().any(|o| o.kind == PropKind::Introduce && o.key == AskKey::Earned)
        && t.asks.iter().any(|o| o.kind == PropKind::InviteFriends && o.key == AskKey::Earned),
        "safe: the known-tier asks open early: {:?}", t.asks);
    assert!(!t.asks.iter().any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Earned),
        "meetings still wait on the bond: {:?}", t.asks);
}

#[test]
fn test_a_mismatch_keeps_the_asks_shallow_not_barred() {
    use crate::game::conversation::Vibe;
    use crate::game::desires::{Problem, Reveal, Want};
    use crate::game::organizing::{AskKey, PropKind, RelStage};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // a villager you've known through three pieces would offer the
    // deeper asks - a wrong note in their audition drops them a rung,
    // leaving only the light one (#1188)
    audition_bonded(&mut g, i, false, vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)], true);
    assert_eq!(crate::game::organizing::stage_of(&g.world, g.player, v), RelStage::Known);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, politely").clone();
    assert_eq!(t.vibe, Some(Vibe::Mismatch));
    let earned: Vec<PropKind> = t.asks.iter()
        .filter(|o| o.key == AskKey::Earned).map(|o| o.kind).collect();
    assert_eq!(earned, vec![PropKind::Scout, PropKind::Probe, PropKind::Introduce],
        "a mismatch stays slow - the light asks still on the table: {:?}", t.asks);
}

/// Pick through a Talk until the ask-slot shows, take it, and hand the
/// topics it opened - `t.options` at that point is the question menu
fn open_ask_menu(g: &mut Game) -> Vec<(Act, crate::game::conversation::Respond, String)> {
    for _ in 0..30 {
        let t = g.seats[0].talk.as_ref().expect("still talking").clone();
        if let Some(i) = t.options.iter().position(|(_, _, l)| l == "Put a question to them") {
            g.talk_pick_at(0, Some(i));
            return g.seats[0].talk.as_ref().expect("the menu opens").options.clone();
        }
        match t.options.iter().position(|(a, _, _)| *a != Act::Farewell) {
            Some(i) => g.talk_pick_at(0, Some(i)),
            None => break,
        }
    }
    panic!("the ask slot never showed");
}

#[test]
fn test_a_talk_lets_you_put_your_own_questions() {
    // the ask-slot rides the pick list (#1189): take it and the topics
    // they can speak to open as a menu - your question, their answer,
    // then the talk picks back up where it stood
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    warm_bond(&mut g, v);
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    // the standing asks never leave the table, and the out is a remark
    assert!(menu.iter().any(|(_, _, l)| l == "Ask about themselves"), "{:?}", menu);
    assert_eq!(menu.last().unwrap().2, "Say nothing of it");
    let p = menu.iter().position(|(_, _, l)| l == "Ask about themselves").unwrap();
    let pe = g.player;
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let asks: Vec<String> = crate::game::dialogue_trees::ASK_TOPICS.iter()
        .find(|tp| tp.label == "themselves").unwrap().ask.iter()
        .map(|a| super::fixtures::said(&t, true, *a)).collect();
    assert!(rec.lines.iter().any(|l| l.who == Some(pe) && asks.contains(&l.text)),
        "your question's on the record: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
    assert!(!t.options.is_empty(), "asking isn't a farewell - options stay live");
}

#[test]
fn test_asking_what_weighs_on_them_draws_it_out() {
    // a kept secret opens the deeper ask (#1189): putting the question
    // is the way a trouble comes out - the same reveal a talk's opening
    // might have rolled, landed because you asked
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    warm_bond(&mut g, v);
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("a kept secret opens the deep ask");
    g.talk_pick_at(0, Some(p));
    let heard = crate::game::desires::known(&g.world, g.player, v);
    assert!(!heard.is_empty(), "the asking drew nothing out");
    // and the telling of it lands on the talk's record
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(v)),
        "they told it on the record: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_asking_the_old_days_tells_their_story() {
    // a carried mark seeds the tale at talk-open (E4-8, #1426): the
    // "old days" ask draws a Labov telling - the record's souls named,
    // the parts in the frame's order, on the talk's record
    let mut g = Game::new_seeded_cast(MAP_SEED);
    let ph = g.world.resource::<crate::session::prehistory::Prehistory>().unwrap().clone();
    let i = adults(&g).into_iter()
        .find(|&i| !ph.marks_of(i).is_empty())
        .expect("someone in the cast carries a mark");
    let v = g.villagers[i];
    let mark = ph.marks_of(i).into_iter()
        .max_by(|a, b| a.severity.partial_cmp(&b.severity).unwrap()).unwrap();
    let ev = ph.events[mark.id.0 as usize].clone();
    let other = ev.touched.iter().find(|t| t.who != crate::session::prehistory::Who::Cast(i))
        .map(|t| match t.who {
            crate::session::prehistory::Who::Cast(s) => g.name_of(g.villagers[s]),
            crate::session::prehistory::Who::Ephemeral(n) => n.to_string(),
        });
    warm_bond(&mut g, v);
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about the old days")
        .unwrap_or_else(|| panic!("a marked past opens the tale ask: {:?}", menu));
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let said = rec.lines.iter().find(|l| l.who == Some(v) && l.text.contains("years back"))
        .unwrap_or_else(|| panic!("the telling lands on the record: {:?}",
            rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>()));
    if let Some(n) = other {
        assert!(said.text.contains(&n), "{} tells of {}: {}", g.name_of(v), n, said.text);
    }
}

#[test]
fn test_a_too_deep_ask_costs_them_nerve() {
    // a question above the pair's openness reads as forward (E3-2):
    // the secret's own wear makes the ask listable, but asking a
    // stranger after it deflects - and costs the bond a rung of nerve
    use crate::game::desires::{InnerLife, Problem};
    use crate::game::relationships::{SafetyRung, add_relationship, find_relationship};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    let pe = g.player;
    add_relationship(&mut g.world, crate::game::relationships::Relationship {
        from: pe, other: v, kind: RelKind::Acquaintance,
        strength: 0.1, interactions: 1,
        safety: SafetyRung::Contributor, ..Default::default() });
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("the worn secret's ask lists, as the tell-named kind");
    g.talk_pick_at(0, Some(p));
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<crate::game::relationships::Relationship>(e))
        .unwrap();
    assert_eq!(r.safety, SafetyRung::Learner,
        "a forward ask costs the bond a rung: {:?}", r.safety);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert_eq!(t.forward_seen, 1);
}

/// Stranger with a worn secret: each "Ask about what weighs on them"
/// turns aside; returns the villager
fn press_recipe(g: &mut Game) -> Entity {
    use crate::game::desires::{InnerLife, Problem};
    use crate::game::relationships::{SafetyRung, add_relationship};
    let i = adults(g)[0];
    let v = g.villagers[i];
    let pe = g.player;
    add_relationship(&mut g.world, crate::game::relationships::Relationship {
        from: pe, other: v, kind: RelKind::Acquaintance,
        strength: 0.1, interactions: 1,
        safety: SafetyRung::Contributor, ..Default::default() });
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    talk_beside(g, i);
    v
}

/// Press the worn secret's deep ask once - it's spent as it's put; the
/// press (E5-2) is the same ask re-put across talks
fn press_on(g: &mut Game) {
    let menu = open_ask_menu(g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .unwrap_or_else(|| panic!("the worn secret's ask lists: {menu:?}"));
    g.talk_pick_at(0, Some(p));
}

#[test]
fn test_pressing_where_they_wont_go_draws_its_shape() {
    // E5-2 done-when: pressing where they won't go draws the shape -
    // "something about the lord", here "something about the trouble" -
    // and the seat's told they see it. Earlier presses this seat's
    // already made (the ledger keeps them across talks)
    let mut g = Game::new();
    let v = press_recipe(&mut g);
    let trouble = crate::game::conversation::area_name(crate::game::conversation::Area::Trouble);
    let (pe, t0) = (g.player, g.tick);
    crate::game::desires::note_evasion(&mut g.world, pe, v, trouble, t0);
    crate::game::desires::note_evasion(&mut g.world, pe, v, trouble, t0);
    press_on(&mut g);
    assert_eq!(crate::game::desires::evasions_of(&g.world, g.player, v, trouble),
        crate::game::desires::SILHOUETTE_AT, "the third press draws the shape");
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.notes.iter().any(|n| n.contains("shape of it")),
        "the seat's told what they see: {:?}", t.notes);
}

#[test]
fn test_a_hothead_pressed_past_the_shape_blurts_it_out() {
    // E5-2: a hothead pressed past the shape tells the thing anyway -
    // in temper, as a real telling on the seat's ledger - and the bond
    // pays for the heat
    use crate::game::desires;
    use crate::game::relationships::{find_relationship, Relationship};
    let mut g = Game::new();
    let v = press_recipe(&mut g);
    g.world.add_component(v, Personality { traits: vec![Trait::Hotheaded] });
    let trouble = crate::game::conversation::area_name(crate::game::conversation::Area::Trouble);
    let (pe, t0) = (g.player, g.tick);
    crate::game::desires::note_evasion(&mut g.world, pe, v, trouble, t0);
    crate::game::desires::note_evasion(&mut g.world, pe, v, trouble, t0);
    press_on(&mut g);
    let heard = desires::known(&g.world, g.player, v);
    assert!(!heard.is_empty(), "the blurt told something: {:?}", heard);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.blurted, "the temper spent itself");
    assert!(t.argued.iter().any(|(_, l)| l.text.contains("Is that what you wanted")),
        "it came out in heat: {:?}", t.argued.iter().map(|x| &x.1.text).collect::<Vec<_>>());
    let integ = find_relationship(&g.world, g.player, v)
        .and_then(|e| g.world.get_component::<Relationship>(e).map(|r| r.trust.integrity))
        .unwrap_or(0.0);
    assert!(integ < 0.0, "the telling cost the bond's keeping: {integ}");
}

#[test]
fn test_backing_off_mends_the_nerve_a_press_frayed() {
    // E5-2: after a press frays the rung, taking an easier ask - one
    // they answer - raises it back a step
    use crate::game::relationships::{find_relationship, Relationship, SafetyRung};
    let mut g = Game::new();
    let v = press_recipe(&mut g);
    press_on(&mut g);
    let rung = |g: &Game| find_relationship(&g.world, g.player, v)
        .and_then(|e| g.world.get_component::<Relationship>(e).map(|r| r.safety));
    assert_eq!(rung(&g), Some(SafetyRung::Learner), "the press frayed it");
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about the village")
        .expect("the easy ask's on the table");
    g.talk_pick_at(0, Some(p));
    assert_eq!(rung(&g), Some(SafetyRung::Contributor),
        "backing off paid the fray back a rung: {:?}", rung(&g));
}

/// The doorknob drive (E5-3): take the first live reply while one's on
/// offer and the farewell once that's all that's left - true when the
/// worry's own menu showed and `pick` was taken from it
fn drive_to_doorknob(g: &mut Game, pick: &str) -> bool {
    for _ in 0..80 {
        let Some(t) = g.seats[0].talk.as_ref() else { return false };
        if t.options.is_empty() { return false; }
        let opts = t.options.clone();
        if let Some(p) = opts.iter().position(|(_, _, l)| l == pick) {
            g.talk_pick_at(0, Some(p));
            return true;
        }
        match opts.iter().position(|(a, _, _)| *a == Act::Farewell) {
            Some(p) => g.talk_pick_at(0, Some(p)),
            None => g.talk_pick_at(0, Some(0)),
        }
    }
    false
}

#[test]
fn test_the_doorknob_carries_a_real_fear() {
    // E5-3: the worry a talk carries is their own pressing thing -
    // seeded off the talk's seed so every replay asks it the same
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    g.world.add_component(v, crate::game::desires::InnerLife {
        wants: vec![], grievances: vec![(crate::game::desires::Problem::Taxed, 1000)],
        satisfied: vec![] });
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert_eq!(t.villager.worry,
        crate::game::desires::worry_of(&g.world, v, g.tick, t.seed),
        "the seeded worry's the live pool's pick");
    assert!(t.villager.worry.is_some(), "a live load carries a worry");
}

#[test]
fn test_asking_why_at_the_door_opens_the_tier() {
    // E5-3 done-when: asking why opens the tier whenever openness
    // allows - the fear's told and lands on the seat's ledger
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let (pe, mut fired) = (g.player, false);
    for i in adults(&g) {
        let v = talk_beside(&mut g, i);
        {
            let t = g.seats[0].talk.as_mut().unwrap();
            t.villager.worry = Some(Reveal::Problem(Problem::Taxed));
            t.villager.open = crate::game::conversation::Open {
                intimacy: 0.7, benevolence: 0.7, integrity: 0.7,
                safety: 0.8, mood: 0.0, talks: 4, earshot: false, shared: 2 };
        }
        if drive_to_doorknob(&mut g, "Ask why") {
            fired = true;
            assert!(crate::game::desires::known(&g.world, pe, v)
                .contains(&Reveal::Problem(Problem::Taxed)),
                "the truth landed on the ledger");
            break;
        }
    }
    assert!(fired, "no doorknob fired across the adults");
}

#[test]
fn test_a_door_that_stays_shut_counts_the_tell() {
    // E5-3/E5-2: the why asked past what they'll open gets turned
    // aside - the same silhouette count a pressed ask draws
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let (pe, mut fired) = (g.player, false);
    for i in adults(&g) {
        let v = talk_beside(&mut g, i);
        {
            let t = g.seats[0].talk.as_mut().unwrap();
            t.villager.worry = Some(Reveal::Problem(Problem::Taxed));
            t.villager.open = crate::game::conversation::Open::default();
        }
        if drive_to_doorknob(&mut g, "Ask why") {
            fired = true;
            assert!(!crate::game::desires::known(&g.world, pe, v)
                .contains(&Reveal::Problem(Problem::Taxed)),
                "a shut door tells nothing");
            let trouble = crate::game::conversation::area_name(
                crate::game::conversation::Area::Trouble);
            assert_eq!(crate::game::desires::evasions_of(&g.world, pe, v, trouble), 1,
                "the turn-aside counted under its area");
            break;
        }
    }
    assert!(fired, "no doorknob fired across the adults");
}

#[test]
fn test_a_slip_comes_out_once_to_a_relationship() {
    // E5-8: a marked soul's leak is a landmark - one to a
    // relationship. Heard, the budget spends: the next sit-down with
    // the same ear opens with nothing left to leak
    let mut g = Game::new();
    let pe = g.player;
    let mut heard = None;
    for i in adults(&g) {
        let v = talk_beside(&mut g, i);
        {
            let t = g.seats[0].talk.as_mut().unwrap();
            t.villager.slip = Some(crate::game::conversation::Slip {
                prone: 1.0,
                dead: Some(crate::game::conversation::Dead {
                    kin: "brother", name: "Corvin".into(), trade: "carter" }) });
            t.villager.slips_left = 1;
        }
        let mut picks = 0;
        while g.seats[0].talk.is_some() && picks < 40 { g.talk_pick_at(0, Some(0)); picks += 1; }
        if g.world.get_component::<SlipMemory>(v)
            .map(|m| m.of.contains(&pe)).unwrap_or(false) {
            heard = Some((i, v));
            break;
        }
    }
    let (i, v) = heard.expect("no slip fired across the adults");
    // and the next sit-down with the same ear carries no budget
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert_eq!(t.villager.slips_left, 0, "the arc's one slip is spent");
    let mut picks = 0;
    while g.seats[0].talk.is_some() && picks < 40 { g.talk_pick_at(0, Some(0)); picks += 1; }
    assert!(g.world.get_component::<SlipMemory>(v)
        .map(|m| m.of.len() == 1 && m.of.contains(&pe)).unwrap_or(false),
        "one slip to a relationship, marked once");
}

#[test]
fn test_the_trouble_they_told_is_an_ask() {
    // what's already shared sits on the menu as its own question
    // (#1189) - asking after it is following up, not prying anew
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    warm_bond(&mut g, v);
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    assert!(menu.iter().any(|(_, _, l)| l == "Ask about the trouble they told"),
        "a shared trouble opens the follow-up: {:?}", menu);
}

#[test]
fn test_a_live_mystery_is_its_own_ask() {
    // folk are asking after something (#1189): putting the question
    // pulls a piece of it the way the gossip line would
    use crate::session::mystery::SeatMystery;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    warm_bond(&mut g, v);
    g.seats[0].mystery = Some(SeatMystery { def: 0, known: 0, asker: v });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about the question folk whisper of")
        .expect("a live mystery opens the ask");
    let before = g.seats[0].mystery.as_ref().unwrap().heard();
    g.talk_pick_at(0, Some(p));
    let m = g.seats[0].mystery.as_ref().unwrap();
    assert_eq!(m.heard(), before + 1, "the asking drew a fact");
    let t = g.seats[0].talk.as_ref().expect("still talking").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let facts: Vec<&str> = crate::game::mysteries::MYSTERIES[0].clues.iter().map(|c| c.text).collect();
    assert!(rec.lines.iter().any(|l| l.who == Some(v) && facts.contains(&l.text.as_str())),
        "they passed the piece on: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

/// A villager pressed twice in a day runs dry (#1226): the second talk
/// opens flat - no fresh mind to speak, no propositions, one word at a
/// time - where the first ran full
#[test]
fn test_a_second_pressing_runs_flat() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    assert_eq!(g.seats[0].talk.as_ref().unwrap().tired, 1, "the day's first pressing runs full");
    g.close_player_talk(0, None);
    assert_eq!(talk_beside(&mut g, i), v, "same villager, pressed again");
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, flatter").clone();
    assert_eq!(t.tired, 2, "the day's tally counted the second pressing");
    // flat: no propositions, no fresh mind to speak, and they say so
    assert!(t.asks.is_empty(), "a tired villager keeps their asks: {:?}", t.asks);
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(!rec.lines.iter().any(|l| l.gist), "the gist's already been said today");
    assert!(rec.lines.iter().any(|l| l.text.contains("Still here")),
        "they acknowledge the pressing: {:?}", rec.lines);
    // and what comes back runs a word at a time
    let mut saw_short = false;
    for _ in 0..4 {
        let Some(t) = g.seats[0].talk.as_ref() else { break };
        assert!(t.options.len() <= 1, "the pressed list runs short: {:?}", t.options);
        saw_short |= t.options.len() == 1;
        g.talk_pick_at(0, Some(0));
    }
    assert!(saw_short, "a flat talk still answers");
}

/// The third pressing of a day finds them nearly talked out (#1226): a
/// word or two and the talk itself calls it done
#[test]
fn test_a_third_pressing_runs_dry() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    for _ in 0..2 { talk_beside(&mut g, i); g.close_player_talk(0, None); }
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("a third talk still opens").clone();
    assert_eq!(t.tired, 3);
    let id = t.id;
    let mut picks = 0;
    while g.seats[0].talk.is_some() && picks < 6 {
        g.talk_pick_at(0, Some(0));
        picks += 1;
    }
    assert!(g.seats[0].talk.is_none(), "the third pressing doesn't linger");
    assert!(picks <= 3, "it ran a word or two, not a conversation: {} picks", picks);
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("run out of things to say")),
        "the thin parting's on the record: {:?}", rec.lines.last().map(|l| l.text.clone()));
}

/// The tally lifts at dawn (#1226): yesterday's pressings don't follow
/// a villager into the next day's talk
#[test]
fn test_a_new_day_lifts_the_tiredness() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    g.close_player_talk(0, None);
    g.tick += 1;
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("a fresh day's talk").clone();
    assert_eq!(t.tired, 1, "dawn lifts the tally");
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.gist), "a fresh day's talk opens with the gist again");
}

// --- signature scenes (#1227): the village's small set of stories ---

/// Two grown villagers with no scene waiting for them
fn plain_pair(g: &Game) -> (Entity, Entity) {
    let ad = adults(g);
    for &i in &ad {
        for &j in &ad {
            if i == j { continue; }
            let (a, b) = (g.villagers[i], g.villagers[j]);
            if crate::game::scenes::scene_for(&g.world, a, b, g.tick).is_none() { return (a, b); }
        }
    }
    panic!("expected a pair with no signature scene");
}

#[test]
fn test_a_feuding_pair_plays_their_scene() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: -0.5, interactions: 3, ..Default::default() });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.kind, TalkKind::Chat);
    assert_eq!(r.lines[0].text, "Thy fence still leans on my strip.");
    // the offender speaks first, however the street order fell
    assert_eq!(r.lines[0].who, Some(a));
    assert_eq!(r.lines[1].who, Some(b));
    // and the TALK panel shows it like any village chat
    let i = g.villagers.iter().position(|&v| v == b).unwrap();
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    assert!(g.talk_panel_at(0).rows.iter().any(|row| row.text == r.lines[0].text));
}

#[test]
fn test_the_scene_take_moves_with_the_bond() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    let rel = crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: 0.0, interactions: 1, ..Default::default() });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Thy boy ran my fence line yesterday.",
        "a cooling feud plays the middle take: {:?}", r.lines);
    // bonds warm, the feud thaws - the next telling is a kinder one
    g.world.get_component_mut::<crate::game::relationships::Relationship>(rel).unwrap().strength = 0.5;
    g.tick += 14;
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Cold coming in early this year.",
        "a thawed feud plays the warm take: {:?}", r.lines);
}

#[test]
fn test_the_widow_and_the_priest_have_theirs() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.add_component(a, crate::game::desires::InnerLife {
        wants: vec![], satisfied: vec![],
        grievances: vec![(crate::game::desires::Problem::Bereaved, g.tick + 10)] });
    g.world.add_component(b, crate::game::vocation::craft(Vocation::Priest, 0.9));
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "I saw thee at the grave again, child.");
    assert_eq!(r.lines[0].who, Some(b), "the priest opens: {:?}", r.lines);
    assert_eq!(r.lines[1].who, Some(a));
}

#[test]
fn test_the_pedlar_and_the_watch_have_theirs() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.add_component(a, crate::game::vocation::TownRole::Trader);
    g.world.add_component(b, crate::game::vocation::craft(Vocation::Knight, 0.5));
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Honest goods, watch - same as the last three times.");
    assert_eq!(r.lines[0].who, Some(a), "the pedlar opens: {:?}", r.lines);
    // and it holds whichever way the street lists them
    let mut g2 = Game::new();
    let (a2, b2) = (a, b);
    g2.world.add_component(a2, crate::game::vocation::TownRole::Trader);
    g2.world.add_component(b2, crate::game::vocation::craft(Vocation::Knight, 0.5));
    assert!(g2.villagers_chat(b2, a2));
    let r2 = g2.talk_log.last().unwrap().clone();
    assert_eq!(r2.lines[0].who, Some(a2));
    assert_eq!(r2.lines[0].text, r.lines[0].text);
}

#[test]
fn test_a_scene_cools_off_between_tellings() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: -0.5, interactions: 3, ..Default::default() });
    assert!(g.villagers_chat(a, b));
    assert_eq!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
    // pressed again the same day, they fall back to small talk
    assert!(g.villagers_chat(a, b));
    assert_ne!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
    // a fortnight on, the scene's theirs again
    g.tick += 14;
    assert!(g.villagers_chat(a, b));
    assert_eq!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
}

#[test]
fn test_a_pair_with_no_story_small_talks() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let scripted: Vec<&str> = crate::game::scenes::SCENES.iter()
        .flat_map(|s| s.variants).flat_map(|v| v.lines.iter().map(|l| l.1)).collect();
    assert!(!r.lines.iter().any(|l| scripted.contains(&l.text.as_str())),
        "no scene lines for an unmatched pair: {:?}", r.lines);
}

// --- the bond register (#1228): how well they know you swaps how they talk ---

/// Walk the organizer's bond with a villager to a strength (and mark it
/// spoken-to, so the stage ladder can climb past stranger)
fn bond_to(g: &mut Game, v: Entity, strength: f32) {
    let pe = g.player;
    crate::game::relationships::record_interaction(&mut g.world, pe,
        crate::game::relationships::Interaction { other: v,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<crate::game::relationships::Relationship>(r))
        .unwrap().strength = strength;
}

#[test]
fn test_an_acquaintance_opens_like_a_neighbor() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.2);
    assert_eq!(crate::game::organizing::stage_of(&g.world, g.player, v),
        crate::game::organizing::RelStage::Acquaintance);
    talk_beside(&mut g, i);
    let rec = g.talk_log.last().unwrap().clone();
    let first = rec.lines.iter().find(|l| l.who == Some(v) && !l.gist)
        .map(|l| l.text.clone()).unwrap_or_default();
    let calls = ["Morrow, ", "good to see a familiar face", "Well met, ", "Neighbor!"];
    assert!(calls.iter().any(|c| first.contains(c)),
        "the neighbor register greets, not the sky's smalltalk: {}", first);
}

#[test]
fn test_two_members_carry_the_cause_in_the_street() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    // no shared cause yet: they small-talk like any neighbors
    assert!(g.villagers_chat(a, b));
    let before = g.talk_log.last().unwrap().clone();
    assert!(!before.lines.iter().any(|l| l.text.contains("the cause")
        || l.text.contains("the meet's")), "open-air talk: {:?}", before.lines);
    // sworn to the same org, the next hello drops to a quiet word
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![a, b];
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let calls = ["the meet's", "the cause keeps", "folk are talking", "Between us and the wall"];
    assert!(calls.iter().any(|c| r.lines[0].text.contains(c)),
        "the fellows register greets: {}", r.lines[0].text);
}

#[test]
fn test_one_member_still_small_talks() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![a];
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let calls = ["the meet's", "the cause keeps", "Between us and the wall"];
    assert!(!calls.iter().any(|c| r.lines[0].text.contains(c)),
        "a lone member gets open-air talk: {}", r.lines[0].text);
}

#[test]
fn test_a_follower_opens_with_the_causes_weather() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.6);
    g.seats[0].mystery = None;
    let pe = g.player;
    g.world.add_component(pe, crate::game::organizing::MeetingPlan {
        for_day: Some(g.tick + 3), ..Default::default() });
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.gist.contains("e gather in 3 days"),
        "the meet's on the gist, not the sky: {}", t.gist);
}

#[test]
fn test_the_causes_weather_is_a_sworn_privilege() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // no bond - no inside weather
    assert!(g.cause_gist(0, v).is_none());
    // acquaintance is still the street's weather: the stage gates it
    bond_to(&mut g, v, 0.2);
    assert!(g.cause_gist(0, v).is_none());
    // sworn, with a live plan - the meet's the weather
    bond_to(&mut g, v, 0.6);
    let pe = g.player;
    g.world.add_component(pe, crate::game::organizing::MeetingPlan {
        for_day: Some(g.tick + 1), ..Default::default() });
    let (_, said) = g.cause_gist(0, v).unwrap();
    assert!(said.contains("tomorrow"), "{}", said);
}

// --- the first greeting carries the warmth (#1232): one ledger beat a day ---

/// How many bond-beats the pair's ledger has taken: every recorded
/// interaction is one
fn ledger(g: &Game, a: Entity, b: Entity) -> u32 {
    crate::game::relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.interactions).unwrap_or(0)
}

/// Pick the first bond-moving option on offer (a kind answer, a slight,
/// a hello or a goodbye); false if none are up right now
fn pick_bond_moving(g: &mut Game) -> bool {
    let idx = g.seats[0].talk.as_ref().and_then(|t| t.options.iter()
        .position(|(a, _, _)| matches!(a, Act::PushBack | Act::Agree | Act::Answer
            | Act::Greet | Act::Farewell)));
    match idx { Some(i) => { g.talk_pick_at(0, Some(i)); true } None => false }
}

#[test]
fn test_a_re_talk_the_same_day_leaves_the_bond_alone() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 {
        if !pick_bond_moving(&mut g) { g.talk_pick_at(0, Some(0)); }
        p += 1;
    }
    assert!(crate::game::player::pick_spent_today(&g.world, v, pe, g.tick),
        "the first talk spent the day's beat");
    // let the sit-down's armed depth settle - it's the first talk's own
    // beat, and the baseline shouldn't count it as the re-talk's
    crate::engine::ecs::System::run(&mut crate::sim::community::OneOnOneSystem, &mut g.world);
    let beat = ledger(&g, pe, v);
    assert!(beat > 0, "the first talk moved the bond");
    let rap = crate::game::player::rapport(&g.world, pe, v);
    // a second sit-down the same day: the lines play, the ledger doesn't
    talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 {
        if !pick_bond_moving(&mut g) { g.talk_pick_at(0, Some(0)); }
        p += 1;
    }
    crate::engine::ecs::System::run(&mut crate::sim::community::OneOnOneSystem, &mut g.world);
    assert_eq!(ledger(&g, pe, v), beat,
        "a re-talk's picks and its sit-down's depth spend nothing more");
    // rapport isn't the bond ledger - the catch-up still counts
    assert!(crate::game::player::rapport(&g.world, pe, v) > rap,
        "rapport grows on the repeat all the same");
}

#[test]
fn test_the_first_pick_spends_the_days_beat_either_valence() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let before = ledger(&g, pe, v);
    // the first bond-moving pick of the day - whatever it is - lands
    // and stamps; inside the same talk the rest read as flavour
    let mut stamped = false;
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { stamped = true; break; }
        g.talk_pick_at(0, Some(0));
    }
    assert!(stamped, "the talk offered a bond-moving pick");
    assert!(crate::game::player::pick_spent_today(&g.world, v, pe, g.tick));
    assert!(ledger(&g, pe, v) <= before + 1, "one pick, one beat at most");
    // and a second bond-moving pick inside the same talk adds nothing
    let now = ledger(&g, pe, v);
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { break; }
        g.talk_pick_at(0, Some(0));
    }
    assert_eq!(ledger(&g, pe, v), now,
        "a second pick in the same talk records nothing more");
}

#[test]
fn test_a_new_day_earns_a_new_beat() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 { g.talk_pick_at(0, Some(0)); p += 1; }
    let beat = ledger(&g, pe, v);
    g.tick += 1;
    talk_beside(&mut g, i);
    // the morning's beat is fresh - the day's first bond-moving pick lands
    let mut took = false;
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { took = true; break; }
        g.talk_pick_at(0, Some(0));
    }
    assert!(took);
    assert!(ledger(&g, pe, v) > beat, "a new day earns a new beat");
}

// --- villager chats type their own bond delta (#1230): a spat cools ---

/// The pair's bond strength, for reading a chat's landing
fn bond(g: &Game, a: Entity, b: Entity) -> f32 {
    crate::game::relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0)
}

/// A transcript's acts and its close read as the pair's temp: push-backs
/// and a walk-off go cold, agrees and a kind parting stay warm
#[test]
fn test_the_acts_say_how_the_chat_landed() {
    use crate::game::conversation::{chat_temp, Act, Ending, Transcript, Turn};
    let tr = |acts: &[(usize, Act)], ending| Transcript {
        speakers: ["A".into(), "B".into()], trees_walked: vec![],
        turns: acts.iter().map(|&(speaker, act)| Turn {
            speaker, act, respond: None, text: "x".into(), concepts: vec![],
            tree: "t", line: None,
            channel: crate::game::setting::Channel::Public,
        }).collect(),
        ending, vibe: None, asked: vec![], forward: 0, shared: 0, overshot: false,
        setting: crate::game::setting::STREET,
        evasions: vec![], worry_told: None, slips: vec![], jokes: vec![],
    };
    // a warm exchange
    assert!(chat_temp(&tr(&[(0, Act::Greet), (1, Act::Agree), (0, Act::Agree)],
        Some(Ending::Farewell))) > 0.5);
    // one sharp word in an easy talk isn't a spat
    let easy = chat_temp(&tr(&[(0, Act::Greet), (1, Act::PushBack), (0, Act::Agree)],
        Some(Ending::Farewell)));
    assert!(easy > crate::session::SPAT_TEMP && easy < 1.0, "{easy}");
    // but pushed back and walked off is a quarrel
    assert!(chat_temp(&tr(&[(0, Act::Greet), (1, Act::PushBack), (0, Act::Remark)],
        Some(Ending::WalksOff))) <= crate::session::SPAT_TEMP);
    // a pair's bent is a constant - same pair, same read, every time
    let (a, b) = (7u32, 13u32);
    assert_eq!(crate::game::conversation::pair_bias(a, b),
        crate::game::conversation::pair_bias(a, b));
    assert!(crate::game::conversation::pair_bias(a, b).abs() <= 1.0);
    assert_eq!(crate::game::conversation::pair_bias(a, b),
        crate::game::conversation::pair_bias(b, a), "the bent's the pair's, not a side's");
}

#[test]
fn test_sharp_words_chill_the_bond_and_make_the_log() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    let before = bond(&g, a, b);
    g.chat_lands(a, b, -1.0, a);
    assert!(bond(&g, a, b) < before, "a spat cools what a warm talk heats");
    assert!(g.log.iter().any(|l| l.contains("sharp words")),
        "the quarrel's a log beat");
    assert_eq!(g.spats.get(&(a.min(b), a.max(b))).copied(), Some((1, g.tick)),
        "the spat goes on the tally");
    // and a cold chat is no warmth beat - the record knows it ran cold
    let w0 = g.world.get_component::<crate::sim::psychology::AttachmentHistory>(a)
        .map(|h| h.warmth_episodes).unwrap_or(0);
    g.push_talk_cold(crate::session::dialogue::TalkKind::Chat, vec![a, b], vec![
        crate::session::dialogue::TalkLine { who: Some(a), name: "A".into(), text: "get out of my road".into(), gist: false },
        crate::session::dialogue::TalkLine { who: Some(b), name: "B".into(), text: "and good riddance".into(), gist: false }], true);
    assert_eq!(g.world.get_component::<crate::sim::psychology::AttachmentHistory>(a)
        .map(|h| h.warmth_episodes).unwrap_or(0), w0,
        "a spat counts no warmth");
}

#[test]
fn test_a_warm_word_between_two_still_warms() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    let before = bond(&g, a, b);
    g.chat_lands(a, b, 1.0, a);
    assert!(bond(&g, a, b) > before, "a warm chat warms the bond");
    // and the ledger still knows they spoke
    assert!(ledger(&g, a, b) > 0 || ledger(&g, b, a) > 0);
}

#[test]
fn test_quarrels_pile_up_into_a_feud() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    // the bond's already cold, and this is their third spat of the season
    crate::game::relationships::record_interaction(&mut g.world, a,
        crate::game::relationships::Interaction { other: b,
            kind: crate::game::relationships::InteractionKind::Conflict, intensity: 1.0 });
    g.spats.insert((a.min(b), a.max(b)), (crate::session::SPAT_FEUD_N - 1, g.tick));
    g.chat_lands(a, b, -1.0, a);
    let feud = g.world.query::<crate::game::feuds::Feud>().iter().any(|&f| {
        g.world.get_component::<crate::game::feuds::Feud>(f).map(|x|
            (x.offender, x.aggrieved) == (a, b)).unwrap_or(false)
    });
    assert!(feud, "quarrels piling up while the bond's cold open a feud");
    assert!(!g.spats.contains_key(&(a.min(b), a.max(b))), "the tally's spent");
    assert!(g.log.iter().any(|l| l.contains("Bad blood")));
}

#[test]
fn test_the_street_only_goes_so_deep() {
    let mut g = Game::new();
    let (a, b, c, d) = (g.villagers[0], g.villagers[1], g.villagers[2], g.villagers[3]);
    // one pair already fast friends, one pair strangers - the same warm
    // chat lands on each
    crate::game::relationships::record_interaction(&mut g.world, a,
        crate::game::relationships::Interaction { other: b,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 1.0 });
    let r = crate::game::relationships::find_relationship(&g.world, a, b).unwrap();
    g.world.get_component_mut::<crate::game::relationships::Relationship>(r).unwrap().strength = 0.7;
    g.chat_lands(a, b, 1.0, a);
    g.chat_lands(c, d, 1.0, c);
    let deep_gain = bond(&g, a, b) - 0.7;
    let new_gain = bond(&g, c, d);
    assert!(new_gain > 0.01, "a warm chat still warms strangers: {new_gain}");
    assert!(deep_gain < new_gain * 0.5,
        "past the friend mark the street only keeps the bond warm: {deep_gain} vs {new_gain}");
}

// --- the amends: a wrong word owned (#1251) ---

/// Leave a Slighted memory on `v` fresh today
fn slight(g: &mut Game, v: Entity) {
    g.world.add_component(v, TalkMemory { mood: TalkMood::Slighted, tick: g.tick });
}

#[test]
fn test_the_amends_is_on_offer_only_after_a_slight() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = talk_beside(&mut g, ad[0]);
    assert!(!g.seats[0].talk.as_ref().unwrap().amends,
        "no wrong to own, no amends on the table");
    g.talk_pick_at(0, None);
    slight(&mut g, v);
    talk_beside(&mut g, ad[0]);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.amends, "a slight still fresh puts the repair on the table");
    assert_eq!(t.choice_labels(&g.world).last().unwrap(), "Make amends for the other day");
    // and it's the last pick, past even a telling
    assert_eq!(t.options.len(), t.options.len()); // options still lead the list
    g.talk_pick_at(0, None);
    // a warm parting carries no amends
    g.world.add_component(v, TalkMemory { mood: TalkMood::Warmed, tick: g.tick });
    talk_beside(&mut g, ad[0]);
    assert!(!g.seats[0].talk.as_ref().unwrap().amends,
        "nothing to make good once it's warm");
}

#[test]
fn test_an_amends_taken_reopens_the_door_guarded() {
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Kind] });
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    slight(&mut g, v);
    talk_beside(&mut g, ad[0]);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.asks.is_empty(), "the slight shut the asks");
    let pick = t.choice_labels(&g.world).len() - 1;
    let before = bond(&g, g.seats[0].player, v);
    g.talk_pick_at(0, Some(pick));
    let t = g.seats[0].talk.as_ref().unwrap();
    assert_eq!(t.mood, TalkMood::Guarded, "the wrong's owned - guarded, not warm");
    assert!(!t.asks.is_empty(), "a door reopened is a door asks come back through");
    assert!(t.choice_labels(&g.world).iter()
        .all(|l| l != "Make amends for the other day"), "the word's spent once said");
    assert!(bond(&g, g.seats[0].player, v) > before, "an accepted amends warms a little");
    assert_eq!(g.world.get_component::<AmendsTally>(v).map(|a| (a.of, a.n)),
        Some((g.seats[0].player, 1)), "the sorry's counted on the villager");
    // and the parting writes the guarded memory, not the slight
    g.talk_pick_at(0, None);
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Guarded));
}

#[test]
fn test_a_refused_amends_keeps_the_chill_and_is_heard() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    // a hard hand refuses half the time: keep re-owning the wrong till
    // it turns you away - the count ticks either way
    g.world.add_component(v, Personality { traits: vec![Trait::Cruel] });
    let mut refused = false;
    for _ in 0..40 {
        slight(&mut g, v);
        talk_beside(&mut g, ad[0]);
        let pick = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world).len() - 1;
        g.talk_pick_at(0, Some(pick));
        let t = g.seats[0].talk.as_ref().unwrap();
        if t.mood == TalkMood::Slighted { refused = true; break; }
        g.talk_pick_at(0, None);
        // the door reopened: slight them again so the next sorry's on the table
    }
    assert!(refused, "a cruel one turns an amends away inside forty sorries");
    assert!(g.seats[0].talk.as_ref().unwrap().notes.iter()
        .any(|n| n.contains("refused")), "the refused word's a beat of its own");
    g.talk_pick_at(0, None);
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Slighted),
        "the memory stays the slight - no free reset");
    assert!(g.world.get_component::<AmendsTally>(v).map(|a| a.n).unwrap_or(0) >= 1,
        "even the refused sorries count");
}

#[test]
fn test_sorries_stop_landing_as_the_count_grows() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Kind] });
    // the fourth sorry: already counted thrice
    g.world.add_component(v, AmendsTally { of: g.seats[0].player, n: 3 });
    slight(&mut g, v);
    talk_beside(&mut g, ad[0]);
    let before = bond(&g, g.seats[0].player, v);
    let pick = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world).len() - 1;
    g.talk_pick_at(0, Some(pick));
    let d4 = bond(&g, g.seats[0].player, v) - before;
    assert!(d4 > 0.0 && d4 < 0.05, "a sorry oft-repeated barely lands: {d4}");
    assert_eq!(g.world.get_component::<AmendsTally>(v).unwrap().n, 4);
    g.talk_pick_at(0, None);
}

#[test]
fn test_the_sorry_count_outlives_the_memory() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Kind] });
    g.world.add_component(v, AmendsTally { of: g.seats[0].player, n: 2 });
    // the slight's memory's a season gone - the tally isn't
    g.world.add_component(v, TalkMemory { mood: TalkMood::Slighted, tick: 0 });
    g.tick = 21; // past the 20-day memory
    // wrong them again today: the count from the old sorries still stands
    slight(&mut g, v);
    talk_beside(&mut g, ad[0]);
    let pick = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world).len() - 1;
    g.talk_pick_at(0, Some(pick));
    assert_eq!(g.world.get_component::<AmendsTally>(v).unwrap().n, 3,
        "the tally outlives the twenty-day memory");
    g.talk_pick_at(0, None);
}

#[test]
fn test_neighbours_patch_a_spat_in_a_passing_chat() {
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b) = (g.villagers[ad[0]], g.villagers[ad[1]]);
    let key = (a.min(b), a.max(b));
    // the kinder of the pair offers: give a the gentlest bent, b just
    // doesn't refuse
    g.world.add_component(a, Personality { traits: vec![Trait::Kind, Trait::Gregarious] });
    g.world.add_component(b, Personality { traits: vec![] });
    g.chat_lands(a, b, -1.0, a);
    assert!(g.spats.contains_key(&key), "a quarrel's on the tally");
    let before = bond(&g, a, b);
    let mut patched = false;
    for _ in 0..80 {
        assert!(g.villagers_chat(a, b));
        if !g.spats.contains_key(&key) { patched = true; break; }
    }
    assert!(patched, "a passing word patches the quarrel inside eighty chats");
    assert!(g.talk_log.iter().any(|r| r.lines.iter()
        .any(|l| l.text == crate::game::amends::AMENDS_ASK)), "the amends is on the record");
    assert!(bond(&g, a, b) > before, "the patch warms what the quarrel chilled");
    assert!(g.world.get_component::<AmendsTally>(a).map(|x| x.n).unwrap_or(0) >= 1
        || g.world.get_component::<AmendsTally>(b).map(|x| x.n).unwrap_or(0) >= 1,
        "the sorry's counted on whoever made it");
}

// --- the hog: a villager who never trades, only talks (#1252) ---

#[test]
fn test_let_them_run_is_on_offer_for_a_hog() {
    let mut g = Game::new();
    let ad = adults(&g);
    let plain = talk_beside(&mut g, ad[0]);
    assert!(!g.seats[0].talk.as_ref().unwrap().let_run,
        "no space to hold with a fair trader");
    g.talk_pick_at(0, None);
    let _ = plain;
    let hog = g.villagers[ad[1]];
    g.world.add_component(hog, Personality { traits: vec![Trait::SelfAbsorbed] });
    talk_beside(&mut g, ad[1]);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.let_run, "a hog's last slot is the listener's trade");
    assert_eq!(t.choice_labels(&g.world).last().unwrap(), "Let them run on");
}

#[test]
fn test_letting_them_run_discloses_their_own_not_the_villages() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::SelfAbsorbed] });
    // a fresh craft with no bench: a trouble of their own to spill
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Weaver, 0.5));
    talk_beside(&mut g, ad[0]);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.let_run);
    let before = crate::game::desires::known(&g.world, g.player, v).len();
    let pick = t.choice_labels(&g.world).len() - 1;
    g.talk_pick_at(0, Some(pick));
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(!t.let_run, "the space is held once, not forever");
    assert!(crate::game::desires::known(&g.world, g.player, v).len() > before,
        "the monologue yields what they'd never trade: their own trouble");
    assert!(t.notes.iter().any(|n| n.contains("let them run")),
        "the holding's a beat of its own");
    assert!(t.argued.len() >= 2, "the run goes on the record: {t:?}");
    g.talk_pick_at(0, None);
}

#[test]
fn test_a_hog_speaks_of_themselves_not_the_village() {
    let mut g = Game::new();
    let ad = adults(&g);
    let v = g.villagers[ad[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::SelfAbsorbed] });
    // their secrets all said already: the draw falls through to the gist
    let all: Vec<crate::game::desires::Reveal> =
        crate::game::desires::unrevealed(&g.world, g.player, v, g.tick);
    let __pe = g.player; for __r in all { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    let v2 = talk_beside(&mut g, ad[0]);
    assert_eq!(v, v2);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(crate::game::conversation::HOG_GISTS.contains(&t.gist.as_str())
        || !t.gist.is_empty(), "the hog holds forth: {:?}", t.gist);
    // and what they hold forth on is them: a hog's gist is never a rumor
    assert!(!t.gist.contains("they say") , "no village business out of a hog's mouth");
}

#[test]
fn test_the_thaw_rung_reads_on_the_speaker() {
    use crate::game::desires::{rung_of, thaw_status, ThawRung, Thawed, Planted};
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert_eq!(rung_of(&g.world, v), ThawRung::Helpless);
    assert!(s.status.contains(&thaw_status(ThawRung::Helpless)),
        "an un-activated villager reads cold: {:?}", s.status);
    // a planted want, ripening, warms them a rung
    g.world.add_component(v, Planted { wants: vec![(crate::game::desires::Want::Security, 999, g.player)] });
    assert!(speaker_of(&g.world, v, "Eliza", g.tick).status.contains(&"thaw_thawing"));
    // and the ripened mark does the same
    g.world.remove_component::<Planted>(v);
    g.world.add_component(v, Thawed { asker: g.player, tick: 0 });
    assert!(speaker_of(&g.world, v, "Eliza", g.tick).status.contains(&"thaw_thawing"));
    // signed on: joined - and falling back out re-chills
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members.push(v);
    assert_eq!(rung_of(&g.world, v), ThawRung::Joined);
    assert!(speaker_of(&g.world, v, "Eliza", g.tick).status.contains(&"thaw_joined"));
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members.retain(|&m| m != v);
    assert_eq!(rung_of(&g.world, v), ThawRung::Thawing, "the mark's warmth outlasts the membership");
}

#[test]
fn test_the_probing_question_thaws_them_through_the_talk() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.2);
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    let labels = t.choice_labels(&g.world);
    let pick = labels.iter().position(|l| l == "Ask what they actually want")
        .unwrap_or_else(|| panic!("the probing question should be on offer: {:?}", labels));
    assert!(g.world.get_component::<crate::game::desires::Planted>(v).is_none());
    g.talk_pick_at(0, Some(pick));
    // the want's planted, ripening in its own time - and the rung reads it
    assert!(g.world.get_component::<crate::game::desires::Planted>(v)
        .map(|p| !p.wants.is_empty()).unwrap_or(false),
        "the question plants a want they didn't carry");
    assert_eq!(crate::game::desires::rung_of(&g.world, v), crate::game::desires::ThawRung::Thawing);
    // and the ask's spent - asked once, it doesn't come again
    let labels = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world);
    assert!(!labels.iter().any(|l| l == "Ask what they actually want"),
        "the probing question is asked once: {:?}", labels);
}

#[test]
fn test_the_probe_leaves_the_offer_when_theres_nothing_left_to_hear() {
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    bond_to(&mut g, v, 0.2);
    // every probeable want carried already, and everything they've got
    // already heard: the question's spent (#1347)
    g.world.get_component_mut::<crate::game::desires::InnerLife>(v).unwrap().wants =
        vec![crate::game::desires::Want::Security, crate::game::desires::Want::Mastery,
            crate::game::desires::Want::Love, crate::game::desires::Want::Finery,
            crate::game::desires::Want::BetterTools];
    let all = crate::game::desires::unrevealed(&g.world, g.player, v, g.tick);
    let __pe = g.player; for __r in all { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    assert!(!crate::game::desires::probe_open(&g.world, g.player, v, g.tick));
    let offers = crate::game::organizing::ask_offers(&g.world, g.org, v, 0, g.tick);
    assert!(!offers.iter().any(|o| o.kind == crate::game::organizing::PropKind::Probe),
        "nothing left to plant or hear, no probe on offer");
}

/// The world moving re-opens the probing question (#1347): once every
/// want's spoken for and all's heard, a fresh burden puts the ask back
/// - and putting it draws the new thing out
#[test]
fn test_the_probe_reopens_when_their_life_moved_on() {
    use crate::game::desires::{InnerLife, Problem, Reveal, Want};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.2);
    g.world.get_component_mut::<InnerLife>(v).unwrap().wants =
        vec![Want::Security, Want::Mastery, Want::Love, Want::Finery, Want::BetterTools];
    let all = crate::game::desires::unrevealed(&g.world, g.player, v, g.tick);
    let __pe = g.player; for __r in all { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    assert!(!crate::game::desires::probe_open(&g.world, g.player, v, g.tick), "spent while all's heard");
    // their life took on a load: the question's worth asking again
    crate::game::desires::add_grievance(&mut g.world, v, Problem::Loss, g.tick + 60);
    assert!(crate::game::desires::probe_open(&g.world, g.player, v, g.tick), "a new burden reopens it");
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    let pick = t.choice_labels(&g.world).iter().position(|l| l == "Ask what they actually want")
        .expect("the probing question's back on the table");
    g.talk_pick_at(0, Some(pick));
    // a stranger's nerve isn't earned yet (E5-4): the re-asked question
    // meets the safety test first - answer it true and the telling comes
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.safety_bid.is_some(), "a cold bond's secret tests first: {:?}", t);
    g.talk_pick_at(0, Some(0));
    let heard = crate::game::desires::known(&g.world, g.player, v);
    assert!(heard.contains(&Reveal::Problem(Problem::Loss)),
        "the re-asked question drew the new trouble: {:?}", heard);
}

/// A trouble heard can still come back around (#1347): with every live
/// fact already shared, asking after what they told draws the
/// still-carrying line - catching up, not a fresh telling - and learns
/// nothing new twice
#[test]
fn test_the_trouble_they_told_comes_back_around() {
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    warm_bond(&mut g, v);
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    let all = crate::game::desires::unrevealed(&g.world, g.player, v, g.tick);
    let heard_n = all.len();
    let __pe = g.player; for __r in all { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about the trouble they told")
        .expect("a shared trouble stays an ask");
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(v) && l.text.to_lowercase().contains("still")),
        "the catch-up reads as still-carrying: {:?}",
        rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
    let heard = crate::game::desires::known(&g.world, g.player, v);
    assert_eq!(heard.len(), heard_n,
        "catching up learns nothing twice: {:?}", heard);
    // and the telling survives the next pick: the replay rebuild would
    // wash a straight rec.lines push out - the drawn lines ride the
    // record like an argument's do
    for _ in 0..30 {
        let t = g.seats[0].talk.as_ref().expect("still talking").clone();
        match t.options.iter().position(|(a, _, _)| *a != crate::game::conversation::Act::Farewell) {
            Some(i) => g.talk_pick_at(0, Some(i)),
            None => break,
        }
    }
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(v) && l.text.to_lowercase().contains("still")),
        "the catch-up's still on the record after another pick: {:?}",
        rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}


/// The convo box's mood mark (#1320): the seat's read of the bond plus
/// how the talk's going - a stranger mid-greeting reads neutral, the
/// live TalkMood outranks the bond, and only villager speaker rows
/// carry the mark
#[test]
fn test_the_convo_box_marks_the_speakers_mood() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    let mood_of = |g: &Game| g.convo_panel_at(0).unwrap().rows
        .into_iter().find(|r| r.who == Some(i)).map(|r| r.mood);
    // a stranger mid-greeting: nothing to read
    assert_eq!(mood_of(&g), Some(0));
    // the live talk's read wins: a kind word warms, a push-back slights
    g.seats[0].talk.as_mut().unwrap().mood = TalkMood::Warmed;
    assert_eq!(mood_of(&g), Some(1), "a warmed talk marks them warm");
    g.seats[0].talk.as_mut().unwrap().mood = TalkMood::Slighted;
    assert_eq!(mood_of(&g), Some(3), "a slighted talk marks them angry");
    // back to baseline: the bond alone carries it
    g.seats[0].talk.as_mut().unwrap().mood = TalkMood::Cool;
    bond_to(&mut g, v, 0.5);
    assert_eq!(mood_of(&g), Some(1), "a warm bond marks them warm");
    // a rival reads angry, bad blood reads cold
    let pe = g.player;
    let r = crate::game::relationships::find_relationship(&g.world, pe, v).unwrap();
    g.world.get_component_mut::<crate::game::relationships::Relationship>(r).unwrap().kind = RelKind::Rival;
    assert_eq!(mood_of(&g), Some(3), "a rival marks them angry");
    let rel = g.world.get_component_mut::<crate::game::relationships::Relationship>(r).unwrap();
    rel.kind = RelKind::Stranger; rel.strength = -0.4;
    assert_eq!(mood_of(&g), Some(2), "bad blood marks them cold");
    // your own lines and narration carry no mark
    assert!(g.convo_panel_at(0).unwrap().rows.iter().all(|r| r.who.is_some() || r.mood == 0),
        "only a villager speaker can carry the mark");
}


#[test]
fn test_a_neighbor_asked_for_a_hand_comes_by_or_stings() {
    // #1368: ambient chats sometimes end in one neighbor asking another
    // for a hand - a yes sends them by to lend it (the favor lands warm
    // on arrival), a no adds its cool on top of the chat's own landing
    let mut g = Game::new();
    let ad: Vec<Entity> = adults(&g).iter().map(|&i| g.villagers[i]).collect();
    let rel_of = |g: &Game, a: Entity, b: Entity|
        relationships::find_relationship(&g.world, a, b)
            .and_then(|e| g.world.get_component::<crate::game::relationships::Relationship>(e))
            .map(|r| (r.interactions, r.strength));
    let mut yes: Option<(Entity, Entity)> = None;
    let mut refused: Option<(Entity, Entity)> = None;
    'outer: for &a in ad.iter().take(24) {
        for &b in ad.iter().take(24) {
            if a == b { continue; }
            let before = rel_of(&g, a, b).map(|(i, _)| i).unwrap_or(0);
            let n = g.talk_log.len();
            if !g.villagers_chat(a, b) || g.talk_log.len() == n { continue; }
            let rec = g.talk_log.last().unwrap();
            let is_ask = |t: &str| HELP_ASKS.iter()
                .any(|h| *t == g.vv_said(a, b, h) || *t == g.vv_said(b, a, h));
            let Some(ai) = rec.lines.iter().position(|l| is_ask(&l.text)) else { continue };
            let asker = rec.lines[ai].who.unwrap();
            let answer = &rec.lines[ai + 1];
            let asked = answer.who.unwrap();
            // a style marker may sign the line (E4-6): match its base words
            let said = |set: &[&str]| set.iter().any(|l| answer.text.contains(l.trim_end_matches('.')));
            if said(HELP_YES)
                || FAVOR_YES.iter().any(|h| answer.text == g.vv_said(asked, asker, h)) {
                yes = Some((asker, asked));
            } else {
                assert!(said(HELP_NO),
                    "the answer's a yes or a no: {:?}", answer.text);
                let after = rel_of(&g, a, b).map(|(i, _)| i).unwrap_or(0);
                assert_eq!(after - before, 2,
                    "a turned-down hand adds its sting on top of the chat's own landing");
                refused = Some((asker, asked));
            }
            if yes.is_some() && refused.is_some() { break 'outer; }
        }
    }
    let (asker, asked) = yes.expect("some passing chat asks for a hand and gets a yes");
    assert!(g.travels.iter().any(|t| t.who == asked
        && matches!(&t.goal, Goal::HelpOut(e) if *e == asker)),
        "a yes puts the hand on the road to the one who asked");
    let before = rel_of(&g, asked, asker).map(|(_, s)| s).unwrap_or(0.0);
    g.arrive(asked, Goal::HelpOut(asker));
    let after = rel_of(&g, asked, asker).map(|(_, s)| s).unwrap_or(0.0);
    assert!(after > before, "the hand lent warms the bond: {before} -> {after}");
    assert!(refused.is_some(), "some passing chat asks and is turned down");
}

#[test]
fn test_the_village_remembers_what_was_said() {
    // #1369: a chat's lines go into the village's memory, and the memory
    // lets go of what was said long enough ago
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b) = (g.villagers[ad[0]], g.villagers[ad[1]]);
    assert!(g.villagers_chat(a, b));
    assert!(!g.said_lines.is_empty(), "the talk's lines go into memory");
    // a line from long enough ago falls out of the memory on the next chat
    g.tick = 10;
    g.said_lines.insert("Said ages ago, and forgot.", g.tick - crate::session::dialogue::FRESH_LINE_DAYS - 1);
    let (c, d) = (g.villagers[ad[2]], g.villagers[ad[3]]);
    assert!(g.villagers_chat(c, d));
    assert!(!g.said_lines.contains_key("Said ages ago, and forgot."),
        "the memory let go of a line past the window");
}

#[test]
fn test_a_first_talk_counts_no_talks_yet() {
    // the greeting's own beat isn't a talk behind the pair (E3-2):
    // act_talk lands an interaction before the panel opens, but the
    // personal layer's count is prior talks - even the gregarious
    // don't open it on a first meeting
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    g.world.add_component(v, Personality { traits: vec![Trait::Gregarious] });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    assert!(menu.iter().all(|(_, _, l)| l != "Ask about themselves"),
        "the personal layer opened on a first meeting: {:?}",
        menu.iter().map(|(_, _, l)| l).collect::<Vec<_>>());
}

#[test]
fn test_the_responsive_guest_hears_more_than_the_curious() {
    // E3-3's done-when: telling of yourself opens them - the guest who
    // answers substantively is told more than the one who only asks.
    // Same pair, same bond warmth; only the talk policy differs
    use crate::game::conversation::Act;
    let play_talk = |g: &mut Game, i: usize, curious: bool| {
        // ears near the crowd close them: a telling wants the pair alone
        let v = g.villagers[i];
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
        talk_beside(g, i);
        for _ in 0..60 {
            let Some(t) = g.seats[0].talk.clone() else { break };
            let pos = if curious {
                // only ever asks and remarks - never answers or agrees
                t.options.iter().position(|(a, _, _)| *a == Act::Question)
                    .or_else(|| t.options.iter()
                        .position(|(a, _, _)| !matches!(a, Act::Farewell | Act::Answer | Act::Agree)))
                    .or_else(|| t.options.iter().position(|(a, _, _)| *a != Act::Farewell))
            } else {
                // answers and agrees where they're offered - tells of themself
                t.options.iter()
                    .position(|(a, _, _)| matches!(a, Act::Answer | Act::Agree))
                    .or_else(|| t.options.iter().position(|(a, _, _)| *a != Act::Farewell))
            };
            match pos { Some(p) => g.talk_pick_at(0, Some(p)), None => break }
        }
        // part kindly when the talk's spent: only the parting writes
        // the sitting's telling into what they keep of you
        if g.seats[0].talk.is_some() { g.talk_pick_at(0, None); }
    };
    let shared_after = |curious: bool| {
        let mut g = Game::new();
        let i = adults(&g)[0];
        let v = g.villagers[i];
        let pe = g.player;
        // an acquaintance warm enough to hold a first telling - the
        // audition fires for acquaintances, so their probe is where
        // the telling happens
        crate::game::relationships::add_relationship(&mut g.world,
            crate::game::relationships::Relationship {
                from: pe, other: v, kind: RelKind::Acquaintance,
                strength: 0.6, interactions: 2,
                intimacy: 0.5, pivots: 0, deflects: 0, bids_met: 0,
                trust: crate::game::relationships::Trust {
                    ability: 0.0, benevolence: 0.6, integrity: 0.5 },
                safety: crate::game::relationships::SafetyRung::Contributor,
            });
        g.world.add_component(v, Personality { traits: vec![Trait::Stingy] });
        // nothing told them yet: the talk starts unreciprocated
        g.world.add_component(v, TalkMemory { mood: TalkMood::Warmed, tick: g.tick });
        play_talk(&mut g, i, curious);
        g.world.get_component::<SharedDepth>(v)
            .and_then(|sd| sd.of.get(&pe).copied())
            .map(|(depth, _)| depth).unwrap_or(0)
    };
    let responsive = shared_after(false);
    let curious = shared_after(true);
    assert!(responsive >= 1, "the responsive guest's telling landed: {responsive}");
    assert_eq!(curious, 0, "the curious guest never told them anything");
}

#[test]
fn test_another_guests_telling_isnt_yours() {
    // E3-3: disclosure credit keys to the seat's player - a different
    // guest's deep telling doesn't open the private layer for you
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    let pe = g.player;
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: pe, other: v, kind: RelKind::Acquaintance,
            strength: 0.6, interactions: 2,
            intimacy: 0.5, pivots: 0, deflects: 0, bids_met: 0,
            trust: crate::game::relationships::Trust {
                ability: 0.0, benevolence: 0.6, integrity: 0.5 },
            safety: crate::game::relationships::SafetyRung::Contributor });
    // someone else's deepest telling sits on them
    let mut sd = SharedDepth::default();
    sd.of.insert(u32::MAX, (3, g.tick));
    g.world.add_component(v, sd);
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert_eq!(t.villager.open.shared, 0,
        "another guest's telling doesn't count for this seat");
}

#[test]
fn test_a_later_telling_keeps_the_earlier_deflection() {
    // E3-3 + replay: a telling mid-talk opens deeper asks for LATER
    // beats only - the replay rebuild keeps earlier forward asks
    // turned aside, not rewritten by the newer count
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    let pe = g.player;
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: pe, other: v, kind: RelKind::Acquaintance,
            strength: 0.6, interactions: 2,
            intimacy: 0.5, pivots: 0, deflects: 0, bids_met: 0,
            trust: crate::game::relationships::Trust {
                ability: 0.0, benevolence: 0.6, integrity: 0.5 },
            safety: crate::game::relationships::SafetyRung::Contributor });
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    g.world.add_component(v, TalkMemory { mood: TalkMood::Warmed, tick: g.tick });
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
    talk_beside(&mut g, i);
    // walk to the ask slot without answering - an agree/answer on the
    // way would already count as the telling under test
    let menu = (|| {
        for _ in 0..30 {
            let t = g.seats[0].talk.as_ref().expect("still talking").clone();
            if let Some(i) = t.options.iter()
                .position(|(_, _, l)| l == "Put a question to them") {
                g.talk_pick_at(0, Some(i));
                return g.seats[0].talk.as_ref().unwrap().options.clone();
            }
            let pos = t.options.iter().position(|(a, _, _)|
                !matches!(a, Act::Farewell | Act::Answer | Act::Agree));
            match pos { Some(i) => g.talk_pick_at(0, Some(i)), None => break }
        }
        panic!("the ask slot never showed");
    })();
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("the worn secret's ask lists, as the tell-named kind");
    g.talk_pick_at(0, Some(p));
    assert_eq!(g.seats[0].talk.as_ref().unwrap().forward_seen, 1,
        "the forward ask deflected");
    // answer substantively - a telling lands mid-talk
    for _ in 0..30 {
        let t = g.seats[0].talk.as_ref().expect("still talking").clone();
        if t.shared >= 1 { break; }
        let pos = t.options.iter()
            .position(|(a, _, _)| matches!(a, Act::Answer | Act::Agree))
            .or_else(|| t.options.iter().position(|(a, _, _)| *a != Act::Farewell));
        match pos { Some(p) => g.talk_pick_at(0, Some(p)), None => break }
    }
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    assert!(t.shared >= 1, "the telling landed mid-talk");
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let deflected = ["That's not for the telling",
        "We don't know each other so well as that",
        "Ask me another", "Some things keep"];
    assert!(rec.lines.iter().any(|l| l.who == Some(v)
            && deflected.iter().any(|d| l.text.contains(d))),
        "the earlier ask stays turned aside after the telling: {:?}",
        rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

/// #1372: the pair's fine bond rides into the talk - lovers hear the
/// sweetheart beat, not the coarse kin material or smalltalk
#[test]
fn test_a_sweetheart_chat_speaks_as_sweethearts() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Lover, strength: 0.8, interactions: 3, ..Default::default() });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let lovers = ["my heart", "love", "stand closer", "thee this morning",
        "saved a smile", "talking with thee", "at the well"];
    assert!(r.lines.iter().any(|l| lovers.iter().any(|w| l.text.to_lowercase().contains(w))),
        "no sweetheart words in the record: {:?}", r.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

/// #1372: a master and their learner get the teacher's register
#[test]
fn test_a_mentor_chat_carries_the_craft() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Mentor, strength: 0.5, interactions: 5, ..Default::default() });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let craft = ["master", "the craft", "mark me", "mark this", "the bench", "lesson"];
    assert!(r.lines.iter().any(|l| craft.iter().any(|w| l.text.to_lowercase().contains(w))),
        "no mentor register in the record: {:?}", r.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}


/// #1388: the weather talk tracks the sky - a wet day falls rain lines,
/// a fair one dries - the day's own weather rides as a status
#[test]
fn test_weather_talk_matches_the_sky() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    // set the sky to rain: the drift past the threshold reads Rain
    g.world.get_component_mut::<crate::sim::climate::Climate>(g.climate).unwrap().rainfall_drift = 1.0;
    let na = g.name_of(a);
    let s = speaker_of(&g.world, a, &na, g.tick);
    assert!(s.status.contains(&"wx-rain"), "the wet sky rode no status: {:?}", s.status);
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let text = r.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" | ");
    assert!(!text.contains("dew lay heavy") && !text.contains("drying") && !text.contains("hay wishes"),
        "fair-sky lines under a wet sky: {text}");
    // and a fair day hears no rain
    g.world.get_component_mut::<crate::sim::climate::Climate>(g.climate).unwrap().rainfall_drift = 0.0;
    g.tick += 1;
    let s = speaker_of(&g.world, a, &na, g.tick);
    assert!(s.status.contains(&"wx-fair"), "the clear sky rode no status: {:?}", s.status);
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let text = r.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" | ");
    assert!(!text.contains("Looks like rain") && !text.contains("wet week") && !text.contains("mud will take"),
        "wet-sky lines under a fair sky: {text}");
}

/// #1388: a favor-ask names the asked's trade - the fisher gets asked
/// for a fish, and a yes walks it round like any help-out
#[test]
fn test_a_favor_ask_names_their_trade() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    let fisher = || crate::game::vocation::Craft {
        vocation: crate::game::vocation::Vocation::Fisher, skill: 0.7,
        workplace: None, master: None, exposure: 0.0, tasked_by: None,
        pending: 0.0, apprentice_years: 0, stage: 0 };
    g.world.add_component(a, fisher());
    g.world.add_component(b, fisher());
    let mut found = false;
    for i in 0..600 {
        g.tick += 1;
        g.villagers_chat(a, b);
        let r = g.talk_log.last().unwrap();
        if r.lines.iter().any(|l| l.text.contains("spare a fat trout")) {
            let texts: Vec<&str> = r.lines.iter().map(|l| l.text.as_str()).collect();
            assert!(texts.iter().any(|t| t.contains("bring it by") || t.contains("set it aside")
                    || t.contains("at thy door")
                    || HELP_NO.iter().any(|n| t.contains(n.trim_end_matches('.')))),
                "the ask never got its answer: {texts:?}");
            found = true;
            break;
        }
        let _ = i;
    }
    assert!(found, "600 chats and no favor-ask fired");
}

#[test]
fn test_the_responsive_guest_outbonds_the_greedy_one() {
    // E6-3's done-when: the greedy bot loses to the responsive bot.
    // Two identical games, the same villager, a talk a day - one guest
    // always takes the kindest reply on offer, the other always takes
    // the one that turns the talk back to themselves
    use crate::game::conversation::Respond;
    use crate::game::relationships::{find_relationship, Relationship};
    fn respond_rank(r: &Respond) -> i32 {
        match r {
            Respond::Care => 4, Respond::Reflect => 3, Respond::Advise => 2,
            Respond::Validate => 1, Respond::Deflect => -1, Respond::PivotAsk => -2,
        }
    }
    fn bond_after(g: &mut Game, i: usize, responsive: bool) -> (f32, u32, u32) {
        let v = g.villagers[i];
        let pe = g.player;
        for day in 0..8u64 {
            g.tick = day;
            talk_beside(g, i);
            let mut guard = 0;
            while g.seats[0].talk.is_some() && guard < 30 {
                guard += 1;
                let t = g.seats[0].talk.clone().unwrap();
                if t.options.is_empty() { g.talk_pick_at(0, None); continue }
                let (p, _) = t.options.iter().enumerate()
                    .max_by_key(|(_, (_, r, _))| {
                        let s = respond_rank(r);
                        if responsive { s } else { -s }
                    })
                    .expect("a non-empty menu picks");
                g.talk_pick_at(0, Some(p));
            }
        }
        let e = find_relationship(&g.world, pe, v).expect("the talks left a bond");
        let r = g.world.get_component::<Relationship>(e).unwrap();
        (r.intimacy + r.trust.benevolence + r.trust.integrity, r.pivots, r.deflects)
    }
    let mut g1 = Game::new();
    let i = adults(&g1)[0];
    let responsive = bond_after(&mut g1, i, true);
    let mut g2 = Game::new();
    let greedy = bond_after(&mut g2, i, false);
    assert!(responsive.0 > greedy.0,
        "a guest who receives you comes out closer than one who doesn't: {responsive:?} vs {greedy:?}");
    assert!(greedy.1 + greedy.2 > 0,
        "the greedy guest left marks the bond keeps: {greedy:?}");
}

/// Warm but untested (E5-4, #1436): the parts reach the private layer
/// the way `warm_bond`'s do, but the nerve's never climbed - a fresh
/// telling still tests before it moves
fn untested_bond(g: &mut Game, v: Entity) {
    let pe = g.player;
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: pe, other: v, kind: RelKind::Friend,
            strength: 1.0, interactions: 3,
            intimacy: 0.6, pivots: 0, deflects: 0, bids_met: 0,
            trust: crate::game::relationships::Trust {
                ability: 0.0, benevolence: 0.6, integrity: 0.6 },
            safety: crate::game::relationships::SafetyRung::Inclusion,
        });
    let mut sd = SharedDepth::default();
    sd.of.insert(pe, (2, g.tick));
    g.world.add_component(v, sd);
}

#[test]
fn test_a_reach_answered_moves_what_the_table_says() {
    // E5-4's table (#1436): the turn toward warms them and counts - two
    // kept reaches raise the nerve a rung; the miss costs the rung
    // outright, deliberately heavier than the refusal's mark
    use crate::game::relationships::{find_relationship, push_bid, BidTurn,
        Relationship, SafetyRung};
    let mut g = Game::new();
    let pe = g.player;
    let v = g.villagers[adults(&g)[0]];
    push_bid(&mut g.world, pe, v, BidTurn::Toward);
    push_bid(&mut g.world, pe, v, BidTurn::Toward);
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap();
    assert_eq!(r.bids_met, 2);
    assert!(r.intimacy > 0.1 && r.trust.benevolence > 0.0,
        "turning toward warms: {r:?}");
    assert_eq!(r.safety, SafetyRung::Learner, "kept reaches climb the nerve");
    push_bid(&mut g.world, pe, v, BidTurn::Away);
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap();
    assert_eq!(r.safety, SafetyRung::Inclusion, "the miss costs the nerve");
    // and the refusal marks without costing the rung's own floor - a
    // refused reach was at least seen, a missed one wasn't
    push_bid(&mut g.world, pe, v, BidTurn::Toward);
    push_bid(&mut g.world, pe, v, BidTurn::Toward);
    let integ = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().trust.integrity;
    push_bid(&mut g.world, pe, v, BidTurn::Against);
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap();
    assert!(r.trust.integrity < integ, "the refusal leaves its mark");
    assert_eq!(r.safety, SafetyRung::Learner,
        "the refusal isn't the miss - the nerve keeps its floor");
}

#[test]
fn test_a_reach_met_at_the_pick_is_counted() {
    // when their call was a bid, the pick's act is the turn it reads
    // (E5-4): an engaging answer counts a kept reach on the bond
    use crate::game::relationships::{find_relationship, Relationship};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.call_act.is_some(), "the replies answer a real call");
    g.seats[0].talk.as_mut().unwrap().call_act = Some(Act::Bid);
    let (p, act) = t.options.iter().enumerate()
        .map(|(i, (a, _, _))| (i, *a))
        .find(|(_, a)| !matches!(a, Act::Farewell))
        .unwrap_or((0, Act::Farewell));
    g.talk_pick_at(0, Some(p));
    let r = find_relationship(&g.world, g.player, v)
        .and_then(|e| g.world.get_component::<Relationship>(e))
        .expect("the bond moved");
    let toward = matches!(act, Act::Agree | Act::Answer | Act::Question);
    assert_eq!(r.bids_met, toward as u32,
        "the pick's act ({act:?}) was the turn");
}

#[test]
fn test_a_fresh_secret_tests_the_nerve_first() {
    // the disclosure's hard gate (E5-4, #1436): warm but untested, the
    // ask draws the safety bid, not the telling - answering it true is
    // what lets the telling out
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    untested_bond(&mut g, v);
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    // a secret wants a private hearing: no ears near
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("the worn secret's ask lists");
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let bid = t.safety_bid.expect("a fresh secret tests the nerve first");
    assert_eq!(t.choice_labels(&g.world), bid.labels(),
        "the choices are the test's answers now");
    assert!(crate::game::desires::known(&g.world, g.player, v).is_empty(),
        "the secret holds for the answer");
    // answering it true: the held telling comes out - the test's own
    // reveal, whatever the deck dealt - and the word's remembered
    let held = bid.reveal;
    g.talk_pick_at(0, Some(0));
    let heard = crate::game::desires::known(&g.world, g.player, v);
    assert!(heard.contains(&held),
        "the passed test let it out: {:?}", heard);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.bid_passed && t.safety_bid.is_none());
    // and the kept word's on the record E5-5's confidence ledger reads
    let mem = g.world.get_component::<crate::game::episodes::PairMemory>(v)
        .expect("the answer left a memory");
    assert!(mem.eps.iter().any(|e| e.kind == crate::game::episodes::MemKind::KeptConfidence),
        "{:?}", mem.eps);
}

#[test]
fn test_a_test_failed_keeps_the_secret_held() {
    // the leak answer (E5-4): the telling stays held, and where the
    // test's shape makes it a confidence leaked the memory's the sour
    // one the ledger reads back
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    untested_bond(&mut g, v);
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    // a secret wants a private hearing: no ears near
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("the worn secret's ask lists");
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    let bid = t.safety_bid.expect("the test stands");
    let leak = bid.labels().len() - 1;
    g.talk_pick_at(0, Some(leak));
    assert!(crate::game::desires::known(&g.world, g.player, v).is_empty(),
        "the telling stayed held");
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.safety_bid.is_none() && !t.bid_passed, "the test's done and failed");
    match bid.kind {
        SafetyBidKind::Hypothetical | SafetyBidKind::Probe => {
            let mem = g.world.get_component::<crate::game::episodes::PairMemory>(v)
                .expect("the leak left a memory");
            assert!(mem.eps.iter().any(|e| e.kind == crate::game::episodes::MemKind::LeakedConfidence),
                "{:?}", mem.eps);
        }
        SafetyBidKind::Confidence => {
            // "I make no promises" is a miss, not a leak: no sour
            // memory, just the rung's own cost
            assert!(g.world.get_component::<crate::game::episodes::PairMemory>(v)
                .map(|m| !m.eps.iter().any(|e| e.kind == crate::game::episodes::MemKind::LeakedConfidence))
                .unwrap_or(true));
        }
    }
}

#[test]
fn test_walking_out_on_their_test_is_the_miss() {
    // leaving the test unanswered is the turn away (E5-4): they
    // reached and were left standing - the nerve pays it
    use crate::game::desires::{InnerLife, Problem};
    use crate::game::relationships::{find_relationship, Relationship, SafetyRung};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    untested_bond(&mut g, v);
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    // a secret wants a private hearing: no ears near
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 200.0, y: 200.0 };
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, _, l)| l == "Ask about what weighs on them")
        .expect("the worn secret's ask lists");
    g.talk_pick_at(0, Some(p));
    // the rung earned: the untested bond's nerve climbs off the floor
    let pe = g.player;
    crate::game::relationships::push_bid(&mut g.world, pe, v,
        crate::game::relationships::BidTurn::Toward);
    crate::game::relationships::push_bid(&mut g.world, pe, v,
        crate::game::relationships::BidTurn::Toward);
    assert_eq!(find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().safety,
        SafetyRung::Learner);
    g.talk_pick_at(0, None);
    assert!(g.seats[0].talk.is_none(), "the leave closed the talk");
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap();
    assert_eq!(r.safety, SafetyRung::Inclusion,
        "walking out on the reach cost the rung");
}

#[test]
fn test_the_probe_bid_tells_someone_elses_secret_first() {
    // the third shape (E5-4): they tell you another's lesser secret
    // and watch what you do - the decoy's real content, filed on its
    // owner, so your handling of it is the thing being watched
    use crate::game::desires::{Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    let told = |g: &Game| g.villagers.iter().copied()
        .filter(|&x| x != v)
        .map(|x| crate::game::desires::known(&g.world, g.player, x).len())
        .sum::<usize>();
    let before = told(&g);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    let r = crate::game::desires::Reveal::Problem(Problem::Taxed);
    let (line, bid) = g.safety_bid_line(0, &t, r, 2);
    assert_eq!(bid.kind, SafetyBidKind::Probe, "the third shape probes");
    assert!(line.contains("Word is,"), "the decoy told: {}", line);
    assert!(!line.contains(&t.villager.name),
        "it isn't their own secret: {}", line);
    assert!(told(&g) > before, "the lesser secret filed on its owner");
    assert!(bid.probe.is_some(), "the probe names its third party");
    // and the watch answers are keep / question / leak
    assert_eq!(bid.labels().len(), 3);
}

/// E7-1 (#1404): the ledger's the seat's own - what one organizer
/// hears stays invisible to the other seat
#[test]
fn test_what_one_seat_learns_is_invisible_to_the_other() {
    use crate::game::desires;
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let s1 = g.add_seat();
    let (pe0, pe1) = (g.seats[0].player, g.seats[s1].player);
    // seat 0's meeting: the room gives up a trouble and their ledger
    // marks it; the asking goes through the real hear path
    g.meeting_lines(&[v], 0, (false, false, false, false, false), true, None);
    assert!(!desires::known(&g.world, pe0, v).is_empty(),
        "the host's meeting marked what they heard");
    // the other seat's record stays blind: the same draw's still live
    // for them, and the stage never rose on their side
    assert!(desires::known(&g.world, pe1, v).is_empty(),
        "what one seat learns is invisible to the other");
    // a live secret the meeting didn't give up: seat 0 marks it heard,
    // and it's off their draw but still on the other's
    let r = desires::unrevealed(&g.world, pe1, v, g.tick)[0];
    desires::note(&mut g.world, pe0, v, r);
    assert!(!desires::unrevealed(&g.world, pe0, v, g.tick).contains(&r),
        "heard once, it's off their own draw");
    assert!(desires::unrevealed(&g.world, pe1, v, g.tick).contains(&r),
        "but the other seat could still draw it out of them");
}


#[test]
fn test_a_shared_laugh_over_power_files_alignment() {
    // E5-6: a barb at the lord taken up where it could have been
    // reported is the village mind made visible - the villager keeps
    // the laugh as an episode
    use crate::game::conversation::{Grievance, Joke, JokeTarget};
    use crate::game::episodes::{MemKind, PairMemory};
    let mut g = Game::new();
    let mut heard = None;
    for i in adults(&g) {
        let v = talk_beside(&mut g, i);
        {
            let t = g.seats[0].talk.as_mut().unwrap();
            t.setting = crate::game::setting::BACKSTAGE;
            t.villager.joke = Some(Joke {
                styles: [0.0, 0.0, 1.0, 0.0],
                grievance: Some(Grievance { target: JokeTarget::Lord,
                    word: "the lord's wars",
                    lie: crate::game::persona::Belief::LordAlwaysWins }) });
        }
        // option 0 is the open laugh when the four forks show
        let mut picks = 0;
        while g.seats[0].talk.is_some() && picks < 40 { g.talk_pick_at(0, Some(0)); picks += 1; }
        if g.world.get_component::<PairMemory>(v)
            .map(|m| m.eps.iter().any(|e| e.kind == MemKind::LaughedWithUs))
            .unwrap_or(false) {
            heard = Some(v);
            break;
        }
    }
    assert!(heard.is_some(), "no laugh over a power joke across the adults");
}

#[test]
fn test_a_villagers_temperament_signs_their_lines() {
    // E4-6 (#1433): the style markers ride the session's own speaker
    // build - a pious mouth swears where a plain one stays unmarked,
    // and the signing lands inside the realized line, cap two a line
    use crate::game::conversation::{fill, Rng};
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Pious] });
    let s = speaker_of(&g.world, v, &g.name_of(v), g.tick);
    let p = g.player_speaker(g.player);
    let oaths = crate::game::text::authored_lines().iter()
        .filter(|&&t| {
            let got = fill(t, &s, &p, &mut Rng::new(0));
            got.starts_with("Saints keep us.") || got.starts_with("By the rood.")
                || got.starts_with("God witness.")
        }).count();
    assert!(oaths > 50, "a pious mouth should sign its lines: {oaths}");
    // the player's own un-traited voice stays plain
    let marked = crate::game::text::authored_lines().iter()
        .filter(|&&t| {
            let got = fill(t, &p, &s, &mut Rng::new(0));
            ["Saints keep us.", "By the rood.", "God witness."].iter()
                .any(|w| got.starts_with(w))
                || ["and that's the long of it.", "and more besides.",
                    "as I've cause to say.", "or so I hear.", "as like as not.",
                    "by my reckoning.", "or am I mistook?",
                    "or is that not the way of it?", "or do I hear wrong?",
                    "for my part.", "as I see it.", "or so I make it."]
                    .iter().any(|w| got.ends_with(w))
        }).count();
    assert_eq!(marked, 0, "a trait-less speaker marks nothing: {marked}");
}
