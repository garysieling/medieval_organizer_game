//! Conversation demo: showcase the conversation framework in action.
//!
//! Demonstrates:
//! 1. Specificity cascade: a sick villager talking to a neighbor (sickness tree leads,
//!    then falls through to generic smalltalk, then ends in farewell)
//! 2. Personality bias & rivalry: old scores between rivals (hotheaded pushback -> walk-off)
//! 3. Clergy interaction: parishioner asking the father for blessings/guidance
//! 4. Player choice turn: simulated player picking from 2 options or walking away
//! 5. House meeting: concept-threaded discussion where attendees reply to organizer
//!    or earlier statements

use game2_core::game::conversation::{
    converse, converse_with, run_meeting, Choice, Ending, MeetingLine, Relation, Rng, Speaker,
};
use game2_core::game::dialogue_trees::{
    backstory, meeting_pool, village_library, MEET_PROMPT,
};
use game2_core::game::traits::Trait;

fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(65));
    println!("  {}", title);
    println!("{}\n", "=".repeat(65));
}

fn main() {
    let library = village_library();

    // -----------------------------------------------------------------
    // 1. Specificity cascade: Sick villager & kind neighbor
    // -----------------------------------------------------------------
    print_separator("1. Specificity Cascade: Eliza (Sick) & Aldric (Kind neighbor)");
    let mut eliza = Speaker::new("Eliza");
    eliza.status.push("sick");
    eliza.traits.push(Trait::Gregarious);

    let mut aldric = Speaker::new("Aldric");
    aldric.traits.push(Trait::Kind);

    println!("Backstory - {}: {}", eliza.name, backstory(&eliza));
    println!("Backstory - {}: {}", aldric.name, backstory(&aldric));
    println!("Relation: Acquaintance\n");

    let mut rng = Rng::new(102);
    let tr = converse(&eliza, &aldric, Relation::Acquaintance, &library, &mut rng);
    println!("Trees walked in cascade: {:?}", tr.trees_walked);
    println!("Outcome: {:?}\n", tr.ending.unwrap_or(Ending::Exhausted));
    for turn in &tr.turns {
        println!("  {:10} [{:?} in {}]: {}", tr.speakers[turn.speaker], turn.act, turn.tree, turn.text);
    }

    // -----------------------------------------------------------------
    // 2. Old Scores: Rivalry with Hotheaded trait
    // -----------------------------------------------------------------
    print_separator("2. Old Scores: Wulfric (Rival) & Corvin (Hotheaded)");
    let mut wulfric = Speaker::new("Wulfric");
    wulfric.traits.push(Trait::Proud);

    let mut corvin = Speaker::new("Corvin");
    corvin.traits.push(Trait::Hotheaded);

    println!("Backstory - {}: {}", wulfric.name, backstory(&wulfric));
    println!("Backstory - {}: {}", corvin.name, backstory(&corvin));
    println!("Relation: Rival\n");

    let mut rng = Rng::new(202);
    let tr_rival = converse(&wulfric, &corvin, Relation::Rival, &library, &mut rng);
    println!("Trees walked: {:?}", tr_rival.trees_walked);
    println!("Outcome: {:?}\n", tr_rival.ending.unwrap_or(Ending::Exhausted));
    for turn in &tr_rival.turns {
        println!("  {:10} [{:?} in {}]: {}", tr_rival.speakers[turn.speaker], turn.act, turn.tree, turn.text);
    }

    // -----------------------------------------------------------------
    // 3. Clergy interaction: Anselm (Cleric) & Bram (Laborer)
    // -----------------------------------------------------------------
    print_separator("3. Clergy Blessing: Anselm (Clergy) & Bram (Laborer)");
    let mut anselm = Speaker::new("Anselm");
    anselm.ideology = "clergy";
    anselm.traits.push(Trait::Pious);
    anselm.traits.push(Trait::Kind);

    let mut bram = Speaker::new("Bram");
    bram.ideology = "laborer";
    bram.traits.push(Trait::Pious);

    println!("Backstory - {}: {}", anselm.name, backstory(&anselm));
    println!("Backstory - {}: {}", bram.name, backstory(&bram));
    println!("Relation: Acquaintance\n");

    let mut rng = Rng::new(303);
    let tr_cleric = converse(&anselm, &bram, Relation::Acquaintance, &library, &mut rng);
    println!("Trees walked: {:?}", tr_cleric.trees_walked);
    println!("Outcome: {:?}\n", tr_cleric.ending.unwrap_or(Ending::Exhausted));
    for turn in &tr_cleric.turns {
        println!("  {:10} [{:?} in {}]: {}", tr_cleric.speakers[turn.speaker], turn.act, turn.tree, turn.text);
    }

    // -----------------------------------------------------------------
    // 4. Interactive turn simulation (Player picking from 2 choices)
    // -----------------------------------------------------------------
    print_separator("4. Player Turn: Dagny greets Player (2 choices per turn)");
    let dagny = Speaker::new("Dagny");
    let player = Speaker::new("Player");
    let mut rng = Rng::new(404);

    let mut turn_count = 0;
    let tr_player = converse_with(
        &dagny,
        &player,
        Relation::Acquaintance,
        &library,
        1, // player is speaker 1 (responder)
        &mut |choices: &[Choice]| {
            turn_count += 1;
            println!("    [Player Prompt Turn {}]:", turn_count);
            for (i, c) in choices.iter().enumerate() {
                println!("      Option {}: [{:?}] \"{}\"", i + 1, c.act, c.preview);
            }
            if turn_count >= 3 {
                println!("      Option 3: [Leave] Walk away.");
                println!("    -> Player chooses to leave early.\n");
                None
            } else {
                println!("    -> Player chooses Option 1.\n");
                Some(0)
            }
        },
        &mut rng,
    );
    println!("Player conversation outcome: {:?}\n", tr_player.ending.unwrap_or(Ending::Exhausted));
    for turn in &tr_player.turns {
        println!("  {:10}: {}", tr_player.speakers[turn.speaker], turn.text);
    }

    // -----------------------------------------------------------------
    // 5. House Meeting: Threaded group discussion
    // -----------------------------------------------------------------
    print_separator("5. House Meeting: Reeve convenes 4 villagers on village concerns");
    let reeve = Speaker::new("Reeve");
    let mut m1 = Speaker::new("Eliza");
    m1.status.push("sick");
    m1.traits.push(Trait::Kind);

    let mut m2 = Speaker::new("Bram");
    m2.status.push("hungry");
    m2.traits.push(Trait::Hotheaded);

    let mut m3 = Speaker::new("Corvin");
    m3.ideology = "merchant";
    m3.traits.push(Trait::Ambitious);

    let mut m4 = Speaker::new("Anselm");
    m4.ideology = "clergy";
    m4.traits.push(Trait::Pious);

    let p1 = meeting_pool(&m1);
    let p2 = meeting_pool(&m2);
    let p3 = meeting_pool(&m3);
    let p4 = meeting_pool(&m4);
    let attendees = [&m1, &m2, &m3, &m4];
    let pools: Vec<&[MeetingLine]> = vec![&p1, &p2, &p3, &p4];

    let mut rng = Rng::new(505);
    let statements = run_meeting(&reeve, &attendees, &MEET_PROMPT, &pools, &mut rng);

    println!("House Meeting Transcript ({} statements):", statements.len());
    for (i, stmt) in statements.iter().enumerate() {
        let speaker_name = if stmt.speaker == usize::MAX {
            &reeve.name
        } else {
            &attendees[stmt.speaker].name
        };
        let reply_info = match stmt.replying_to {
            Some(target) => format!("(replying to #{})", target),
            None => "(agenda opening)".to_string(),
        };
        println!("  #{:<2} {:10} [{:?}] {:<18} concepts={:?}",
            i, speaker_name, stmt.act, reply_info, stmt.concepts);
        println!("      \"{}\"\n", stmt.text);
    }

    print_separator("Demo Complete");
}
