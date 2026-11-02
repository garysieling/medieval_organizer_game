//! Campaign decision content shared by both libraries: the source
//! patterns they borrow and the authoring helpers. The house-meeting
//! library lives in `campaign_meeting_beats`, the campaign-arc library in
//! `campaign_arc_beats`; the model and its audit in `campaign_dialogue`.
//!
//! Authoring rules (wiki/software-design/campaign-dialogue-trees-plan.md):
//! - Every phase keeps at least one criteria-free beat - the fallback.
//! - A beat is a situation put to the room; its moves are what the room
//!   can do about it. Weights are relative priors, not history's odds.
//! - The village voice: short, concrete, period words; no modern terms.
//!   Source tags carry the history; the lines never lecture.

use crate::game::campaign_dialogue::{Beat, Evidence, Hook, Move, Next, Phase, SourcePattern, Voice};
use crate::game::conversation::Criterion;

pub(crate) const fn to(p: Phase) -> Next { Next::Phase(p) }
pub(crate) const END: Next = Next::Close;

/// A move with no criteria or bias of its own; struct-update it to add them
pub(crate) const fn mv(id: &'static str, weight: f32, voice: Voice, lines: &'static [&'static str], hook: Hook, next: Next) -> Move {
    Move { id, criteria: &[], weight, bias: &[], voice, lines, hook, next }
}

/// A beat, positionally: id, phase, criteria, voice, lines, moves, sources
pub(crate) const fn bt(id: &'static str, phase: Phase, criteria: &'static [Criterion], voice: Voice,
                       lines: &'static [&'static str], moves: &'static [Move], sources: &'static [&'static str]) -> Beat {
    Beat { id, phase, criteria, voice, lines, moves, sources }
}

pub const SOURCE_PATTERNS: &[SourcePattern] = &[
    SourcePattern { tag: "exodus_assembly", origin: "Exodus tradition", evidence: Evidence::Sacred,
        mechanism: "a told story of leaving bondage together binds a room to a common road" },
    SourcePattern { tag: "jubilee_release", origin: "Levitical jubilee and sabbath-year release", evidence: Evidence::Sacred,
        mechanism: "a moral claim on the powerful made in their own scripture" },
    SourcePattern { tag: "athenian_assembly", origin: "Athenian ekklesia, 5th-4th c. BC", evidence: Evidence::Documented,
        mechanism: "who may speak, in what order, and a public show of hands" },
    SourcePattern { tag: "guild_ordinance", origin: "medieval craft guild ordinances and courts", evidence: Evidence::Documented,
        mechanism: "an officer keeps order and a written rule binds members to what they agreed" },
    SourcePattern { tag: "commune_oath", origin: "medieval sworn communes (coniuratio)", evidence: Evidence::Documented,
        mechanism: "a mutual oath turns many households into one party that acts together" },
    SourcePattern { tag: "remenca_redemption", origin: "Remença peasants, Catalonia 1462-1486", evidence: Evidence::Documented,
        mechanism: "persistence plus bargaining wins a partial, paid-for release - not a clean victory" },
    SourcePattern { tag: "hussite_compacts", origin: "Hussite wars and the Compacts of Basel, 1415-1436", evidence: Evidence::Documented,
        mechanism: "a negotiated settlement offered to moderates splits them from the militants" },
    SourcePattern { tag: "waldensian_cells", origin: "Waldensians, 1170s onward", evidence: Evidence::Documented,
        mechanism: "small trusted circles outlast a hunt that crushes open assemblies" },
    SourcePattern { tag: "montaillou_register", origin: "Fournier inquisition register, Montaillou 1318-1325", evidence: Evidence::Documented,
        mechanism: "names spoken and written can be gathered by those who hunt you" },
    SourcePattern { tag: "grievance_articles", origin: "peasant grievance lists, e.g. the Twelve Articles of 1525", evidence: Evidence::Documented,
        mechanism: "a written list of wrongs and asks makes demands specific and checkable" },
    SourcePattern { tag: "abolition_petitions", origin: "abolitionist petition drives, 1830s", evidence: Evidence::Documented,
        mechanism: "gathering signatures recruits and counts supporters as much as it asks" },
    SourcePattern { tag: "labor_strike", origin: "labor movement, 1880s-1950s", evidence: Evidence::Documented,
        mechanism: "withholding work tests the organization and shows the target its cost" },
    SourcePattern { tag: "land_war_boycott", origin: "Irish Land War, 1879-1882", evidence: Evidence::Documented,
        mechanism: "shunning isolates the agent and holds neighbors to the common line" },
    SourcePattern { tag: "montgomery_mass_meeting", origin: "Montgomery bus boycott, 1955-1956", evidence: Evidence::Documented,
        mechanism: "regular mass meetings and testimony sustain a long, costly refusal" },
    SourcePattern { tag: "salt_march_demand", origin: "Salt March, 1930", evidence: Evidence::Documented,
        mechanism: "a small demand everyone feels dramatizes a larger injustice" },
    SourcePattern { tag: "section_504_coalition", origin: "Section 504 sit-in, 1977", evidence: Evidence::Documented,
        mechanism: "allies with their own reasons keep an occupation fed and visible" },
    SourcePattern { tag: "freeman_structurelessness", origin: "Freeman, 'The Tyranny of Structurelessness', 1970", evidence: Evidence::Documented,
        mechanism: "where no one is chosen to lead, the best-connected lead unaccountably" },
    SourcePattern { tag: "polletta_decision_rule", origin: "Polletta, 'Freedom Is an Endless Meeting', 2002", evidence: Evidence::Documented,
        mechanism: "majority settles fast but leaves a minority; consensus is slow but binds" },
    SourcePattern { tag: "snow_frame_alignment", origin: "Snow et al., frame alignment, 1986", evidence: Evidence::Documented,
        mechanism: "how a cause is put decides who hears themselves in it" },
    SourcePattern { tag: "citizenship_schools", origin: "Highlander and the Citizenship Schools, 1950s-60s", evidence: Evidence::Documented,
        mechanism: "teaching ordinary members to lead multiplies leaders and outlives any one" },
    SourcePattern { tag: "cointelpro_divide", origin: "COINTELPRO, 1956-1971", evidence: Evidence::Documented,
        mechanism: "the other side feeds rumor and offers some members a separate peace" },
    SourcePattern { tag: "reconstruction_redemption", origin: "Reconstruction and its reversal, 1865-1877", evidence: Evidence::Documented,
        mechanism: "a won right left unguarded is clawed back piece by piece" },
    SourcePattern { tag: "ufw_enforcement", origin: "United Farm Workers contract enforcement, 1970s", evidence: Evidence::Documented,
        mechanism: "a win holds only while someone watches and can call folk out again" },
    SourcePattern { tag: "repression_backfire", origin: "'political jiu-jitsu' accounts of crackdowns", evidence: Evidence::Contested,
        mechanism: "open force against the peaceful can turn onlookers - but often it simply works" },
    SourcePattern { tag: "nehemiah_assembly", origin: "Nehemiah 5, the great assembly against usury", evidence: Evidence::Sacred,
        mechanism: "a public assembly shames creditors into releasing pledges and swearing to it" },
    SourcePattern { tag: "acts_common_purse", origin: "Acts 2 and 4, the common purse", evidence: Evidence::Sacred,
        mechanism: "need shared out from a common store holds a group through scarcity" },
    SourcePattern { tag: "village_bylaws", origin: "English manorial bylaws and village assemblies, 13th-15th c.", evidence: Evidence::Documented,
        mechanism: "the village sets and polices its own field rules, fines and gleaning" },
    SourcePattern { tag: "peasants_revolt_1381", origin: "English Rising of 1381", evidence: Evidence::Documented,
        mechanism: "charters granted under pressure are revoked once the crowd disperses" },
    SourcePattern { tag: "ciompi_revolt", origin: "Ciompi wool-workers, Florence 1378", evidence: Evidence::Documented,
        mechanism: "the unrepresented win a guild of their own, then lose it to divisions within" },
    SourcePattern { tag: "lollard_conventicle", origin: "Lollard household conventicles, 1380s-1520s", evidence: Evidence::Documented,
        mechanism: "reading and talking in a trusted kitchen keeps a cause alive under suspicion" },
    SourcePattern { tag: "beguine_houses", origin: "Beguine communities, 13th c. onward", evidence: Evidence::Documented,
        mechanism: "women pool work and lodging outside the usual authorities and govern it themselves" },
    SourcePattern { tag: "quaker_business_meeting", origin: "Quaker meeting for business, 17th c. onward", evidence: Evidence::Documented,
        mechanism: "silence and a clerk's minute find the sense of the meeting rather than a count" },
    SourcePattern { tag: "rochdale_rules", origin: "Rochdale Pioneers, 1844", evidence: Evidence::Documented,
        mechanism: "plain written rules on shares and accounts keep a mutual store honest" },
    SourcePattern { tag: "group_centered_leadership", origin: "Ella Baker and SNCC, 1960s", evidence: Evidence::Documented,
        mechanism: "the organizer draws out local leaders rather than speaking for them" },
    SourcePattern { tag: "cut_issue", origin: "Alinsky-school organizing, 1940s onward", evidence: Evidence::Documented,
        mechanism: "a problem is cut into an issue: specific, winnable, with someone who can say yes" },
    SourcePattern { tag: "tell_legend", origin: "William Tell legend", evidence: Evidence::Legendary,
        mechanism: "a remembered refusal to bow lends a room the nerve to refuse together" },
    SourcePattern { tag: "jacquerie", origin: "Jacquerie, France 1358", evidence: Evidence::Contested,
        mechanism: "rage without plan or allies is crushed - and the chroniclers were the victors" },
    SourcePattern { tag: "outlaw_ballads", origin: "Robin Hood and greenwood ballads", evidence: Evidence::Legendary,
        mechanism: "a told story of the taken brought home lends nerve to those left behind" },
];
