#!/usr/bin/env python3
"""Assembles all trees and meeting pools into game2-core/src/game/dialogue_trees.rs.
"""

import sys
import os
import re

sys.path.append(os.path.dirname(__file__))
from trees_data import DATA
from more_trees_data import MORE_DATA
from trade_data import TRADE_DATA
from vocations_data import VOCATIONS_DATA
from meeting_data import MEETING_POOLS
from emit_rust import augment_exchange, S, lines_slice, bias_slice, criteria_slice, concepts_slice, all_dialogue_lines

OUT_FILE = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "game2-core", "src", "game", "dialogue_trees.rs"))

header = """//! Expanded dialogue library: wide, shallow set of 1000+ contextually grounded
//! responses covering everyday smalltalk, trades, life circumstances, and house meetings.
//!
//! Research basis (wiki/literature-reviews/dialogue-systems.md):
//! - Ruskin (Valve): Criteria-based rule database where specificity leads and generic smalltalk
//!   forms the guaranteed fallback.
//! - Comme il Faut (McCoy et al.): Trait-weighted response distributions.
//! - Façade (Mateas & Stern): Discourse acts and concept-threaded meeting dynamics.
//! - Versu (Evans & Short): Quip exclusivity and graceful material exhaustion.

use crate::game::conversation::{
    Act, Criterion::*, DialogueTree, Ending::*, Exchange, MeetingLine, MeetingPrompt,
    Reply, Speaker,
};
use crate::game::traits::Trait;
use crate::game::traits::Trait::*;

"""

def render_ex(ex):
    out = []
    out.append(f"const {ex['name']}: Exchange = Exchange {{")
    out.append(f"    act: Act::{ex['act']},")
    out.append(f"    concepts: {concepts_slice(ex['concepts'])},")
    out.append(f"    calls: {lines_slice(ex['calls'])},")
    out.append("    replies: &[")
    for r in ex["replies"]:
        then_str = f"Some({r['then']})" if r.get("then") else "None"
        end_str = f"Some({r['end']})" if r.get("end") else "None"
        out.append("        Reply {")
        out.append(f"            criteria: {criteria_slice(r['criteria'])}, weight: {r['weight']:.2f}, bias: {bias_slice(r['bias'])},")
        out.append(f"            act: Act::{r['act']}, concepts: {concepts_slice(r['concepts'])},")
        out.append(f"            lines: {lines_slice(r['lines'])},")
        out.append(f"            then: {then_str}, end: {end_str},")
        out.append("        },")
    out.append("    ],")
    out.append("};")
    return "\n".join(out)

def main():
    all_raw_exchanges = DATA + MORE_DATA + TRADE_DATA + VOCATIONS_DATA
    for ex in all_raw_exchanges:
        augment_exchange(ex)

    rendered_exchanges = []
    for ex in all_raw_exchanges:
        rendered_exchanges.append(render_ex(ex))

    # Render meeting concern pools
    rendered_pools = []
    pool_var_names = []
    for var_name, concept, lines in MEETING_POOLS:
        pool_var_names.append((var_name, concept))
        rendered_pools.append(f"const {var_name}: &[MeetingLine] = &[")
        concepts = [concept]
        if concept == "concern:grief":
            concepts.append("concern:sickness")
        for line in lines:
            rendered_pools.append("    MeetingLine {")
            rendered_pools.append(f"        act: Act::Remark, concepts: {concepts_slice(concepts)}, bias: &[],")
            rendered_pools.append(f"        lines: {lines_slice([line])},")
            rendered_pools.append("    },")
        rendered_pools.append("];\n")

    # Common meeting lines
    meet_common_lines = [
        ("Agree", ["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"],
         ["Aye, there is the truth of it.", "{other} has the right of it.", "It is so - my house has felt the same.", "Well spoken, neighbor; we stand with thee."]),
        ("PushBack", ["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"],
         ["Talk is cheap - who will lift a hand to do aught about it?", "And what wouldst thou have the village do, {other}?", "We have argued this before and accomplished nothing."]),
        ("Remark", ["concern:harvest"],
         ["If the harvest is sound, we will weather the winter.", "We'll need every willing pair of hands in the valley."]),
        ("Remark", ["concern:roads"],
         ["Safe roads bring trade and bread into every cottage.", "The ditch by the bridge must be dug out."]),
        ("Remark", ["concern:fuel"],
         ["Cold stones freeze the spirit; we must gather kindling together.", "The common wood has brushwood enough for all if shared."]),
        ("Remark", ["concern:grief", "concern:sickness"],
         ["The parish mourns with those who have lost kin.", "A neighbor's visit and warm broth bring comfort to an empty room."]),
    ]
    common_pool_code = ["const MEET_COMMON: &[MeetingLine] = &["]
    for act, concepts, lines in meet_common_lines:
        common_pool_code.append("    MeetingLine {")
        common_pool_code.append(f"        act: Act::{act}, concepts: {concepts_slice(concepts)}, bias: &[],")
        common_pool_code.append(f"        lines: {lines_slice(lines)},")
        common_pool_code.append("    },")
    common_pool_code.append("];\n")

    # Render meeting_pool function
    meeting_pool_fn = """/// The lines `speaker` brings to a house meeting: what they actually live with
pub fn meeting_pool(speaker: &Speaker) -> Vec<MeetingLine> {
    let mut pool: Vec<MeetingLine> = vec![];
    if speaker.status.contains(&"sick") { pool.extend_from_slice(MEET_SICKNESS); }
    if speaker.status.contains(&"hungry") { pool.extend_from_slice(MEET_FOOD); }
    if speaker.status.contains(&"grieving") { pool.extend_from_slice(MEET_GRIEF); }
    if speaker.status.contains(&"cold") { pool.extend_from_slice(MEET_FUEL); }
    pool.extend_from_slice(match speaker.ideology {
        "clergy" => MEET_FAITH,
        "merchant" => MEET_ROADS,
        "lord" => MEET_RENTS,
        _ => MEET_WAGES,
    });
    pool.extend_from_slice(MEET_HARVEST);
    pool.extend_from_slice(MEET_COMMONS);
    pool.extend_from_slice(MEET_BRIDGE);
    pool.extend_from_slice(MEET_COMMON);
    pool
}
"""

    meet_prompt = """/// A worked-example agenda for a house meeting
pub const MEET_PROMPT: MeetingPrompt = MeetingPrompt {
    text: "What ails the village most, friends? Speak true.",
    concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:fuel", "concern:bridge", "concern:harvest", "concern:commons", "concern:grief"],
};
"""

    # Render village_library function
    library_fn = """/// The full dialogue library: 30 circumstance and vocation trees plus smalltalk tail
pub fn village_library() -> Vec<DialogueTree> {
    vec![
        DialogueTree { name: "the_sickness", criteria: &[Has("self:status:sick")],
            root: &SICK_GREET, rejoin: Some(&SICK_EXCUSE) },
        DialogueTree { name: "visiting_the_sick", criteria: &[Has("other:status:sick")],
            root: &VISIT_GREET, rejoin: Some(&VISIT_CARE) },
        DialogueTree { name: "a_death_in_the_house", criteria: &[Has("self:status:grieving")],
            root: &GRIEF_GREET, rejoin: Some(&GRIEF_OPENS) },
        DialogueTree { name: "condolence_call", criteria: &[Has("other:status:grieving")],
            root: &CONDOLE_GREET, rejoin: Some(&CONDOLE_OFFER) },
        DialogueTree { name: "the_cleric_blesses", criteria: &[Has("self:ideology:clergy")],
            root: &CLERIC_GREET, rejoin: Some(&CLERIC_PARISH) },
        DialogueTree { name: "asking_the_father", criteria: &[Has("other:ideology:clergy")],
            root: &FATHER_GREET, rejoin: Some(&FATHER_ASKS) },
        DialogueTree { name: "old_scores", criteria: &[Has("pair:relation:rival")],
            root: &RIVAL_GREET, rejoin: Some(&RIVAL_SNIPE) },
        DialogueTree { name: "hunger_lean_times", criteria: &[Has("self:status:hungry")],
            root: &HUNGER_GREET, rejoin: Some(&HUNGER_SHARE) },
        DialogueTree { name: "cold_hearth_fuel", criteria: &[Has("self:status:cold")],
            root: &COLD_GREET, rejoin: Some(&COLD_GATHER) },
        DialogueTree { name: "morning_after_drunk", criteria: &[Has("self:status:drunk")],
            root: &DRUNK_GREET, rejoin: Some(&DRUNK_RECALL) },
        DialogueTree { name: "kin_warmth", criteria: &[Has("pair:relation:kin")],
            root: &KIN_GREET, rejoin: Some(&KIN_FAMILY) },
        DialogueTree { name: "friendship_bonds", criteria: &[Has("pair:relation:friend")],
            root: &FRIEND_GREET, rejoin: Some(&FRIEND_TRUST) },
        DialogueTree { name: "stranger_road", criteria: &[Has("pair:relation:stranger")],
            root: &STRANGER_GREET, rejoin: Some(&STRANGER_PATH) },
        DialogueTree { name: "smith_forge", criteria: &[Has("other:status:smith")],
            root: &SMITH_GREET, rejoin: Some(&SMITH_WORK) },
        DialogueTree { name: "miller_grain", criteria: &[Has("other:status:miller")],
            root: &MILLER_GREET, rejoin: Some(&MILLER_GRIST) },
        DialogueTree { name: "baker_hearth", criteria: &[Has("other:status:baker")],
            root: &BAKER_GREET, rejoin: Some(&BAKER_BREAD) },
        DialogueTree { name: "weaver_wool", criteria: &[Has("other:status:weaver")],
            root: &WEAVER_GREET, rejoin: Some(&WEAVER_FLEECE) },
        DialogueTree { name: "carpenter_wood", criteria: &[Has("other:status:carpenter")],
            root: &CARPENTER_GREET, rejoin: Some(&CARPENTER_BEAMS) },
        DialogueTree { name: "mason_stone", criteria: &[Has("other:status:mason")],
            root: &MASON_GREET, rejoin: Some(&MASON_WALL) },
        DialogueTree { name: "shepherd_fold", criteria: &[Has("other:status:shepherd")],
            root: &SHEPHERD_GREET, rejoin: Some(&SHEPHERD_EWES) },
        DialogueTree { name: "brewer_ale", criteria: &[Has("other:status:brewer")],
            root: &BREWER_GREET, rejoin: Some(&BREWER_MASH) },
        DialogueTree { name: "beekeeper_hive", criteria: &[Has("other:status:beekeeper")],
            root: &BEEKEEPER_GREET, rejoin: Some(&BEEKEEPER_HONEY) },
        DialogueTree { name: "herb_apothecary", criteria: &[Has("other:status:herb_wife")],
            root: &HERB_GREET, rejoin: Some(&HERB_DRAUGHT) },
        DialogueTree { name: "merchant_trade", criteria: &[Has("other:ideology:merchant")],
            root: &MERCHANT_GREET, rejoin: Some(&MERCHANT_WARES) },
        DialogueTree { name: "lord_manor", criteria: &[Has("other:ideology:lord")],
            root: &LORD_GREET, rejoin: Some(&LORD_LEASES) },
        DialogueTree { name: "fisher_river", criteria: &[Has("other:status:fisher")],
            root: &FISHER_GREET, rejoin: Some(&FISHER_CURRENT) },
        DialogueTree { name: "charcoal_pit", criteria: &[Has("other:status:charcoal_burner")],
            root: &CHARCOAL_GREET, rejoin: Some(&CHARCOAL_SMOKE) },
        DialogueTree { name: "elder_wisdom", criteria: &[Has("self:status:elder")],
            root: &ELDER_GREET, rejoin: Some(&ELDER_WINTERS) },
        DialogueTree { name: "newborn_cradle", criteria: &[Has("self:status:newborn")],
            root: &NEWBORN_GREET, rejoin: Some(&NEWBORN_JOY) },
        DialogueTree { name: "village_smalltalk", criteria: &[],
            root: &SMALL_GREET, rejoin: Some(&SMALL_CHAT) },
    ]
}
"""

    backstory_fn = """/// A one-line backstory from a speaker's profile
pub fn backstory(s: &Speaker) -> String {
    let mut parts: Vec<String> = vec![];
    parts.push(match s.ideology {
        "clergy" => format!("{} serves the village church.", s.name),
        "merchant" => format!("{} trades on the road between villages.", s.name),
        "lord" => format!("{} holds the manor's leases.", s.name),
        _ => format!("{} works the fields for a day's wage.", s.name),
    });
    for t in &s.traits {
        parts.push(match t {
            Trait::Lazy => "They'd sooner watch the clouds than the plough.".into(),
            Trait::Ambitious => "They mean to be more than they were born.".into(),
            Trait::Kind => "They're the first with a loaf when a house is bare.".into(),
            Trait::Stingy => "Their purse strings are knotted tight.".into(),
            Trait::Pious => "They never miss a service.".into(),
            Trait::Hotheaded => "Their temper's shorter than a winter's day.".into(),
            Trait::Gregarious => "They know every soul and every soul's business.".into(),
            Trait::Proud => "They bow to no one.".into(),
        });
    }
    for st in &s.status {
        parts.push(match *st {
            "sick" => "A cough's had them these two weeks.".into(),
            "hungry" => "The store ran out and the belly knows it.".into(),
            "grieving" => "They buried their husband last week.".into(),
            "drunk" => "Last night's cup is still on them.".into(),
            "cold" => "Their hearth went out and fuel is spent.".into(),
            "elder" => "Eighty winters of village memories are in their bones.".into(),
            "newborn" => "A new babe was born to their house this month.".into(),
            "smith" => "They work the village anvil and forge.".into(),
            "miller" => "They run the watermill at the weir.".into(),
            "baker" => "They tend the communal bread oven.".into(),
            "weaver" => "They weave russet cloth on the loom.".into(),
            "carpenter" => "They square timber and cut mortise joints.".into(),
            "mason" => "They cut freestone and build field walls.".into(),
            "shepherd" => "They herd ewes and lambs on the common pasture.".into(),
            "brewer" => "They brew ale and tend the village vats.".into(),
            "beekeeper" => "They keep straw skeps in the clover meadow.".into(),
            "herb_wife" => "They gather roots and brew fever draughts.".into(),
            "fisher" => "They set eel traps in the river current.".into(),
            "charcoal_burner" => "They watch the slow-burning earth mounds in the deep wood.".into(),
            other => format!("Lately: {}.", other),
        });
    }
    parts.join(" ")
}
"""

    content = header + "\n\n".join(rendered_exchanges) + "\n\n" + "\n".join(rendered_pools) + "\n" + "\n".join(common_pool_code) + "\n" + meeting_pool_fn + "\n" + meet_prompt + "\n" + library_fn + "\n" + backstory_fn

    with open(OUT_FILE, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"Generated {OUT_FILE}")
    print(f"Total dialogue response lines generated: {len(all_dialogue_lines)}")

if __name__ == "__main__":
    main()
