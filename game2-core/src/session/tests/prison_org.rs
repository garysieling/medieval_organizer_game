use crate::session::*;
use crate::game::dungeon::*;

/// Two souls taken on a charge, the cell has its people, the word goes
/// out and the keep answers
#[test]
fn test_the_cell_organizes_and_the_keep_answers() {
    let mut g = Game::new();
    for &v in &g.villagers[..2] {
        g.world.add_component(v, Imprisoned { since: 0, charge: "sedition".into() });
        g.world.add_component(v, ReleaseCampaign { pressure: 0.0 });
    }
    // the cell forms when the first soul lands
    g.cell_tick(0);
    assert_eq!(g.cells.len(), 1, "no cell formed");
    let org = g.cells[0].org;
    let o = g.world.get_component::<Organization>(org).expect("no org on the cell");
    assert_eq!(o.members.len(), 2, "the held aren't sworn in");
    // the weekly meeting picks an ask and puts it on the road
    for t in 1..60 { g.cell_tick(t); }
    assert!(g.log.iter().any(|l| l.contains("the cell meets")), "no meeting: {:?}", g.log);
    assert!(g.log.iter().any(|l| l.contains("keep answers the cell")), "no answer: {:?}", g.log);
    // and something moved: pressure on the sentence, or the kitchen
    let pushed = g.villagers[..2].iter().any(|&v|
        g.world.get_component::<ReleaseCampaign>(v).map(|r| r.pressure > 0.0).unwrap_or(false));
    let fed = g.cells[0].rations > 0.5;
    assert!(pushed || fed, "the answer changed nothing");
}

/// Thin rations breed the gripes; an ask for better bread raises them
#[test]
fn test_the_slop_sickens_and_the_table_asks_for_bread() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, Imprisoned { since: 0, charge: "sedition".into() });
    g.cell_tick(0);
    assert_eq!(g.cells.len(), 1);
    // the kitchen starves them: thin rations drive the ask first
    g.cells[0].rations = 0.2;
    let mut asked = false;
    for t in 1..60 {
        g.cell_tick(t);
        if g.cell_mail.iter().any(|l| l.demand == Demand::Rations) || asked {
            asked = true;
        }
    }
    assert!(asked || g.log.iter().any(|l| l.contains("slop")), "the table never asked for bread: {:?}", g.log);
}
