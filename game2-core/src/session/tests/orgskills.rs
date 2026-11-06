use crate::session::*;
use crate::game::org_skills::*;

#[test]
fn test_the_craft_is_learned_by_doing_and_taught() {
    let mut g = Game::new();
    let pe = g.seats[0].player;
    // a 1-1 sat down to builds the 1-1 craft
    let v = g.villagers[0];
    let before = g.world.get_component::<Organizing>(pe).map(|o| o.get(OrgSkill::OneOnOne)).unwrap_or(0.0);
    crate::game::org_skills::practice(&mut g.world, pe, OrgSkill::OneOnOne, MEETING_PRACTICE);
    let after = g.world.get_component::<Organizing>(pe).map(|o| o.get(OrgSkill::OneOnOne)).unwrap_or(0.0);
    assert!(after > before, "no practice landed: {} -> {}", before, after);
    // and a skilled organizer's seat-naming teaches the craft down
    g.world.get_component_mut::<Organizing>(pe).unwrap().turnout = 0.6;
    let v0 = g.world.get_component::<Organizing>(v).map(|o| o.get(OrgSkill::Turnout)).unwrap_or(0.0);
    let taught = crate::game::org_skills::teach(&mut g.world, None, pe, v, OrgSkill::Turnout, 1);
    assert!(taught, "the seat's craft didn't teach");
    let v1 = g.world.get_component::<Organizing>(v).map(|o| o.get(OrgSkill::Turnout)).unwrap_or(0.0);
    assert!(v1 > v0, "the student didn't learn: {} -> {}", v0, v1);
    // nobody teaches past their own craft
    let low = g.villagers[2];
    let cant = crate::game::org_skills::teach(&mut g.world, None, low, v, OrgSkill::Turnout, 1);
    assert!(!cant, "an unskilled hand taught the craft");
}
