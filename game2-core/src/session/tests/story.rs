use crate::session::*;
use crate::sim::story::{self, EventKind};

fn note(g: &mut Game, i: usize, tick: u64, kind: EventKind, desc: &str) {
    let ev = story::StoryEvent { tick, kind, description: desc.into() };
    g.tales.entry(i).or_default().push(ev);
    if let Some(st) = story::detect_story(&g.tales[&i]) {
        if g.tales_told.insert(i) {
            g.log.push(format!("t{}: the village marks it - {}'s is a tale of {:?}.", tick, vill_name(i), st.theme));
        }
    }
}

#[test]
fn test_a_tale_of_survival_reads_on_the_card() {
    let mut g = Game::new();
    // a child: the card always shows their little biography
    let i = g.villagers.iter().position(|&v| g.world.get_component::<Kid>(v).is_some())
        .expect("no kid in the village");
    note(&mut g, i, 1, EventKind::Illness, "taken with the flux");
    note(&mut g, i, 3, EventKind::Recovery, "rose from the sickbed");
    note(&mut g, i, 5, EventKind::Bonding, "the neighbours sat with them");
    // the theme formed: the log named it once
    assert!(g.log.iter().any(|l| l.contains("Survival")), "the tale went unnamed: {:?}", g.log);
    // and the card carries the word
    g.seats[0].selected_target = i;
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("survival")),
        "the card hides the tale: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
    // and the chronicle's own words render
    let st = story::detect_story(&g.tales[&i]).unwrap();
    assert!(story::surface_story(&st).contains("theme: Survival"));
}

#[test]
fn test_the_card_carries_the_family_record() {
    let mut g = Game::new();
    note(&mut g, 1, 10, EventKind::Illness, "Aldric is taken with la grippe");
    note(&mut g, 1, 30, EventKind::Recovery, "Aldric recovered");
    g.seats[0].selected_target = 1;
    let insp = g.villager_inspector_at(0);
    let t = insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join(" ");
    assert!(t.contains("took ill") && t.contains("healed"), "the record's missing: {}", t);
}
