//! Campaign decision content: the campaign-arc library, the source
//! patterns both libraries borrow, and the authoring helpers. The
//! house-meeting library lives in `campaign_meeting_beats`; the model and
//! its audit in `campaign_dialogue`.
//!
//! Authoring rules (wiki/software-design/campaign-dialogue-trees-plan.md):
//! - Every phase keeps at least one criteria-free beat - the fallback.
//! - A beat is a situation put to the room; its moves are what the room
//!   can do about it. Weights are relative priors, not history's odds.
//! - The village voice: short, concrete, period words; no modern terms.
//!   Source tags carry the history; the lines never lecture.

use crate::game::campaign_dialogue::{Beat, Evidence, Hook, Move, Next, Phase, SourcePattern, Voice};
use crate::game::conversation::Criterion;
use crate::game::conversation::Criterion::*;
use Hook::*;
use Phase::*;
use Voice::*;

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

// ---------------------------------------------------------------------
// Campaign arcs: 50 beats

pub const ARC_BEATS: &[Beat] = &[
    // --- Target (5) ---
    Beat { id: "ca_target_generic", phase: Target, criteria: &[], voice: Driver,
        lines: &["Who can give us what we want on {issue}?", "Whose word ends {issue}? That's who we go to."],
        moves: &[
            mv("steward", 1.0, Member, &["The steward. He keeps the rolls.", "Not the lord - his steward does the doing."], Nothing, to(Build)),
            mv("lord", 0.6, Member, &["Go to the top. The lord himself.", "Why ask the hand when ye can ask the head?"], Nothing, to(Build)),
        ],
        sources: &["salt_march_demand"] },
    Beat { id: "ca_target_foe", phase: Target, criteria: &[Has("framing:foe")], voice: Driver,
        lines: &["We know whose hand is in it. We go at him.", "He's the one. We make it his trouble."],
        moves: &[
            mv("him", 1.0, Room, &["Him, then.", "Aye, him and no other."], Nothing, to(Build)),
            mv("his_master", 0.5, Member, &["He answers to someone. Go round him.", "Squeeze his master and he'll bend."], Nothing, to(Build)),
        ],
        sources: &["land_war_boycott"] },
    Beat { id: "ca_target_weak", phase: Target, criteria: &[Has("arc:weak")], voice: Driver,
        lines: &["We're few. Pick a mark we can reach.", "Something near. Something we can win this season."],
        moves: &[
            mv("small", 1.0, Room, &["The weir gate. Small, but ours.", "The toll on the footbridge. That we can do."], Nothing, to(Build)),
            mv("too_small", 0.4, Member, &["Win a footbridge and lose the war.", "Small marks make small folk of us."], Nothing, to(Build)),
        ],
        sources: &["salt_march_demand"] },
    Beat { id: "ca_target_demand", phase: Target, criteria: &[], voice: Member,
        lines: &["What do we ask? One thing they can grant by Lammas.", "Put the ask in a sentence, or we've none."],
        moves: &[
            mv("specific", 1.0, Driver, &["Take the toll off the mill. Nothing more.", "Reopen the common to our sheep. That's all."], Nothing, to(Build)),
            mv("broad", 0.5, Member, &["Ask for the whole of it, or we'll never get it.", "All the old dues gone, or nothing."], Nothing, to(Build)),
            mv("accountable", 0.4, Member, &["Ask that they answer to us each quarter-day.", "Not a gift - a reckoning, every season."], Nothing, to(Build)),
        ],
        sources: &["grievance_articles"] },
    Beat { id: "ca_target_duty", phase: Target, criteria: &[Has("framing:duty")], voice: Driver,
        lines: &["The abbot preaches charity. Hold him to his own sermon.", "They say it's God's will to care for the poor. Let them show it."],
        moves: &[
            mv("hold_him", 1.0, Room, &["To the abbey, then.", "Let him hear his own words back."], Nothing, to(Build)),
            mv("debt_release", 0.5, Member, &["Ask for the old release - debts forgiven in the seventh year.", "The book says debts are let go. Ask for that."], Nothing, to(Build)),
        ],
        sources: &["jubilee_release"] },

    // --- Build (5) ---
    Beat { id: "ca_build_generic", phase: Build, criteria: &[], voice: Driver,
        lines: &["Before we ask, let's know who's with us.", "First we count friends. Then we ask."],
        moves: &[
            mv("doors", 1.0, Member, &["Door to door, then. Every house.", "I'll walk the lanes this week."], Nothing, to(Test)),
            mv("feast", 0.6, Member, &["A shared supper. Folk talk with bread in them.", "Bring a pot, bring a neighbor."], Nothing, to(Test)),
        ],
        sources: &["abolition_petitions"] },
    Beat { id: "ca_build_school", phase: Build, criteria: &[], voice: Member,
        lines: &["Teach the young ones to speak in the hall, so it's not always us.", "If we're taken, who speaks? Teach others now."],
        moves: &[
            mv("teach", 1.0, Driver, &["Each of us teaches one.", "Evenings, by the fire. Who can read teaches who can't."], Nothing, to(Test)),
            mv("no_time", 0.4, Member, &["No time for schooling. The fight's now.", "Teach later. Win first."], Nothing, to(Test)),
        ],
        sources: &["citizenship_schools"] },
    Beat { id: "ca_build_tally", phase: Build, criteria: &[Has("seat:note_taker")], voice: NoteTaker,
        lines: &["I've the names. Forty marks on the tally.", "Here's the count, house by house."],
        moves: &[
            mv("enough", 1.0, Driver, &["Forty's enough to try.", "Then we test them."], Nothing, to(Test)),
            mv("record_it", 0.5, NoteTaker, &["I'll keep the list hid and write the count in the book.", "Names stay with me. The number goes on record."], Resolve, to(Test)),
        ],
        sources: &["abolition_petitions", "montaillou_register"] },
    Beat { id: "ca_build_weak", phase: Build, criteria: &[Has("arc:weak")], voice: Member,
        lines: &["Nobody'll come. They're afraid.", "I knocked on twenty doors. Two opened."],
        moves: &[
            mv("smaller_asks", 1.0, Driver, &["Ask less of them. A ribbon, not a march.", "Something they can do without being seen."], Nothing, to(Test)),
            mv("give_up", 0.3, Member, &["Then it's done before it started.", "Leave it. Wait for a worse year."], Nothing, to(Learn)),
        ],
        sources: &["waldensian_cells"] },
    Beat { id: "ca_build_split", phase: Build, criteria: &[Has("arc:split")], voice: Dissenter,
        lines: &["Half of us want the mill, half the tithe. Choose, or lose both.", "We're pulling two ways. It'll tear."],
        moves: &[
            mv("choose", 1.0, Driver, &["The mill first. The tithe after, I swear it.", "One, then the other. In that order."], Nothing, to(Test)),
            mv("both", 0.4, Driver, &["Both. Together we're more.", "Ask both and let them pick which to give."], Nothing, to(Test)),
            mv("walk", 0.4, Dissenter, &["Then the tithe folk go their own way.", "Ye chose. We'll see who comes next time."], Dissent, to(Test)),
        ],
        sources: &["hussite_compacts"] },

    // --- Test (5) ---
    Beat { id: "ca_test_generic", phase: Test, criteria: &[], voice: Driver,
        lines: &["A small thing first: everyone wears a green ribbon on market day.", "We test ourselves before we test them."],
        moves: &[
            mv("many", 1.0, Room, &["Green everywhere on market day.", "Even the alewife wore one."], Nothing, to(Escalate)),
            mv("few", 0.6, Member, &["A handful of ribbons. Not enough.", "More stayed home than came out."], Nothing, to(Build)),
        ],
        sources: &["labor_strike"] },
    Beat { id: "ca_test_petition", phase: Test, criteria: &[], voice: Member,
        lines: &["Sign the list - a mark will do.", "Every mark is a promise. Who'll make one?"],
        moves: &[
            mv("filled", 1.0, Room, &["Two pages of marks.", "The list runs off the edge."], Nothing, to(Escalate)),
            mv("thin", 0.5, Member, &["Half a page, and some of those children.", "Fewer marks than mouths."], Nothing, to(Build)),
        ],
        sources: &["abolition_petitions", "grievance_articles"] },
    Beat { id: "ca_test_strong", phase: Test, criteria: &[Has("arc:strong")], voice: Driver,
        lines: &["They came. All of them.", "Every house turned out."],
        moves: &[
            mv("onward", 1.0, Room, &["Then on.", "Now they'll listen."], Nothing, to(Escalate)),
            mv("bargain_now", 0.5, Member, &["Strong now - ask now, before it fades.", "Take this to the hall while it's warm."], Nothing, to(Negotiate)),
        ],
        sources: &["labor_strike"] },
    Beat { id: "ca_test_consent", phase: Test, criteria: &[Has("arc:consent")], voice: Driver,
        lines: &["We agreed this together. Now we keep to it.", "Every hearth said yes. Now every hearth shows it."],
        moves: &[
            mv("kept", 1.0, Room, &["They keep it. All of them.", "Not one hearth breaks."], Nothing, to(Escalate)),
            mv("one_breaks", 0.3, Member, &["The cooper broke it already.", "One went back on it by noon."], Nothing, to(Build)),
        ],
        sources: &["commune_oath"] },
    Beat { id: "ca_test_weak", phase: Test, criteria: &[Has("arc:weak")], voice: Member,
        lines: &["Three ribbons. Three.", "I counted on my fingers, and had fingers left."],
        moves: &[
            mv("rebuild", 1.0, Driver, &["Back to the doors, then.", "We built too thin. Build again."], Nothing, to(Build)),
            mv("why", 0.5, Driver, &["Let's ask why before we try again.", "Something put them off. Find it."], Nothing, to(Learn)),
        ],
        sources: &["waldensian_cells"] },

    // --- Escalate (5) ---
    Beat { id: "ca_esc_generic", phase: Escalate, criteria: &[], voice: Driver,
        lines: &["Asking hasn't moved them. Time to make them feel it.", "Polite didn't work. What next?"],
        moves: &[
            mv("down_tools", 1.0, Member, &["Down tools. No reaping till they answer.", "Let the harvest stand till they hear us."], Nothing, to(Counter)),
            mv("march", 0.8, Member, &["March to the hall gate. All of us.", "Stand in the square where they must look."], Nothing, to(Counter)),
            mv("ask_again", 0.4, Member, &["One more asking, formal, before we push.", "Give them one more door."], Nothing, to(Negotiate)),
        ],
        sources: &["labor_strike"] },
    Beat { id: "ca_esc_boycott", phase: Escalate, criteria: &[], voice: Member,
        lines: &["Nobody grinds at his mill. Not one sack.", "Not a word, not a coin, not a sack for him."],
        moves: &[
            mv("shun", 1.0, Room, &["Nobody grinds. Hand querns come down from the lofts.", "His wheel turns empty."], Nothing, to(Counter)),
            mv("breakers", 0.5, Member, &["The Hobbs family still grind there.", "Two houses broke it. Everyone saw."], Nothing, to(Counter)),
        ],
        sources: &["land_war_boycott"] },
    Beat { id: "ca_esc_strong", phase: Escalate, criteria: &[Has("arc:strong")], voice: Driver,
        lines: &["We could shut the market. Shall we?", "We've the numbers to close the fair."],
        moves: &[
            mv("shut_it", 1.0, Room, &["Shut it.", "Not a stall opens."], Nothing, to(Counter)),
            mv("restraint", 0.5, Member, &["Show them we could, then ask.", "A threat they believe is worth more than a fair we close."], Nothing, to(Negotiate)),
        ],
        sources: &["labor_strike", "section_504_coalition"] },
    Beat { id: "ca_esc_consent", phase: Escalate, criteria: &[Has("arc:consent")], voice: Driver,
        lines: &["Every house agreed. Down tools at dawn.", "We swore it together. Tomorrow we keep it."],
        moves: &[
            mv("dawn", 1.0, Room, &["At dawn, every hearth.", "Sworn and kept."], Nothing, to(Counter)),
            mv("doubt", 0.3, Member, &["Some are wavering in the night.", "Sworn tonight, sorry by morning."], Nothing, to(Counter)),
        ],
        sources: &["commune_oath"] },
    Beat { id: "ca_esc_sit_in", phase: Escalate, criteria: &[Has("framing:duty")], voice: Driver,
        lines: &["We'll kneel at the church door till the abbot hears us.", "We sit in the abbey yard and pray. Day and night."],
        moves: &[
            mv("kneel", 1.0, Room, &["They kneel. The bells ring over them.", "Folk bring them bread and blankets."], Nothing, to(Counter)),
            mv("hear_first", 0.5, Member, &["Ask him first. Kneel if he won't.", "Let him refuse before we shame him."], Nothing, to(Negotiate)),
        ],
        sources: &["section_504_coalition"] },

    // --- Counter (5) ---
    Beat { id: "ca_counter_generic", phase: Counter, criteria: &[], voice: Opponent,
        lines: &["The steward says any who march lose their strips.", "Word from the hall: keep on, and the rents go up."],
        moves: &[
            mv("hold", 1.0, Room, &["We hold.", "Let him take them all, then. Who'll work them?"], Nothing, to(Escalate)),
            mv("bend", 0.6, Driver, &["We'll talk. But we don't scatter.", "Ask what he'd settle for."], Nothing, to(Negotiate)),
            mv("scatter", 0.3, Room, &["Some go home and bolt the door.", "The crowd thins by evening."], Nothing, to(Learn)),
        ],
        sources: &["cointelpro_divide"] },
    Beat { id: "ca_counter_crackdown", phase: Counter, criteria: &[Has("arc:crackdown")], voice: Opponent,
        lines: &["Men-at-arms in the square. Disperse or be dispersed.", "The knights ride in. Go home."],
        moves: &[
            mv("stand", 0.7, Room, &["They stand, hands empty. Folk at the windows watch.", "Nobody runs. Nobody strikes back."], Nothing, to(Escalate)),
            mv("scatter", 1.0, Room, &["They scatter down the lanes.", "Shutters slam all down the street."], Nothing, to(Learn)),
        ],
        sources: &["repression_backfire"] },
    Beat { id: "ca_counter_prisoners", phase: Counter, criteria: &[Has("arc:prisoners")], voice: Member,
        lines: &["They've taken Wat and the smith's boy.", "Two of ours in the castle cells."],
        moves: &[
            mv("letters", 1.0, Driver, &["Letters, then. Every day till they're home.", "We write. The whole village writes."], Nothing, to(Negotiate)),
            mv("vigil", 0.6, Room, &["A vigil at the gate, every night.", "Candles at the castle wall."], Nothing, to(Escalate)),
        ],
        sources: &["outlaw_ballads"] },
    Beat { id: "ca_counter_split", phase: Counter, criteria: &[Has("arc:split")], voice: Opponent,
        lines: &["The steward offers the millers a fair price - only the millers.", "A deal for some of you. The rest can whistle."],
        moves: &[
            mv("hold_together", 1.0, Driver, &["All of us or none of us.", "We don't sell our neighbors for a fair price."], Nothing, to(Escalate)),
            mv("take_it", 0.6, Dissenter, &["A fair price is a fair price. I'm taking it.", "Mine's settled. Yours is yours."], Dissent, to(Negotiate)),
        ],
        sources: &["hussite_compacts", "cointelpro_divide"] },
    Beat { id: "ca_counter_informer", phase: Counter, criteria: &[], voice: Member,
        lines: &["Someone told the hall our plan before we'd finished saying it.", "They knew. They knew before we did."],
        moves: &[
            mv("smaller_circles", 1.0, Driver, &["Smaller circles, then. Each knows only their part.", "Plans in threes. No more."], Nothing, to(Build)),
            mv("press_on", 0.5, Driver, &["Let them know. We do it anyway.", "Nothing hid, nothing to find."], Nothing, to(Escalate)),
        ],
        sources: &["waldensian_cells", "montaillou_register"] },

    // --- Negotiate (5) ---
    Beat { id: "ca_neg_generic", phase: Negotiate, criteria: &[], voice: Opponent,
        lines: &["The lord will hear one of you. One.", "Send someone up to the hall. Alone."],
        moves: &[
            mv("send_three", 1.0, Driver, &["Three go, or none.", "We send three, so no one's bought alone."], Nothing, to(Outcome)),
            mv("send_one", 0.5, Driver, &["I'll go. Wait for me.", "One, then. I'll carry all of you."], Nothing, to(Outcome)),
            mv("refuse", 0.3, Room, &["We'll talk in the square, where all can hear.", "No back rooms."], Nothing, to(Escalate)),
        ],
        sources: &["hussite_compacts"] },
    Beat { id: "ca_neg_offer", phase: Negotiate, criteria: &[], voice: Opponent,
        lines: &["Half the toll, and ye go home quiet.", "Half now. The rest we'll think on."],
        moves: &[
            mv("take", 0.8, Driver, &["Half, in writing. Today.", "We take it - and we'll be back for the rest."], Nothing, to(Outcome)),
            mv("counter", 1.0, Driver, &["The whole toll, or three-quarters and the common.", "Not half. Make it more."], Nothing, to(Outcome)),
            mv("refuse", 0.4, Room, &["No. All of it.", "Half is still a toll."], Nothing, to(Escalate)),
        ],
        sources: &["remenca_redemption"] },
    Beat { id: "ca_neg_strong", phase: Negotiate, criteria: &[Has("arc:strong")], voice: Driver,
        lines: &["They need this ended more than we do. Ask for all of it.", "The fields stand unreaped. Their need is louder than ours."],
        moves: &[
            mv("all", 1.0, Room, &["All of it.", "Every last penny of it."], Nothing, to(Outcome)),
            mv("not_too_far", 0.4, Member, &["Push too hard and they'll bring soldiers.", "Leave them a way out, or they'll make one."], Nothing, to(Outcome)),
        ],
        sources: &["labor_strike"] },
    Beat { id: "ca_neg_consent", phase: Negotiate, criteria: &[Has("arc:consent")], voice: Driver,
        lines: &["Whatever we say in the hall, we agreed it first here.", "I carry what we settled. No more, no less."],
        moves: &[
            mv("trust", 1.0, Room, &["Carry it, then.", "Go. We're behind thee."], Nothing, to(Outcome)),
            mv("check_back", 0.5, Member, &["And if they offer something new, come back before thou answers.", "Bring any new word back to us first."], Nothing, to(Outcome)),
        ],
        sources: &["commune_oath", "polletta_decision_rule"] },
    Beat { id: "ca_neg_noted", phase: Negotiate, criteria: &[Has("seat:note_taker")], voice: NoteTaker,
        lines: &["Write what they promise, and have their clerk mark it too.", "Two copies. Theirs and ours."],
        moves: &[
            mv("written", 1.0, NoteTaker, &["Both marked. Ours goes in the chest.", "Signed, sealed, and copied."], Resolve, to(Outcome)),
            mv("refuse_ink", 0.5, Opponent, &["The lord's word is enough. No ink.", "We don't sign for peasants."], Nothing, to(Outcome)),
        ],
        sources: &["grievance_articles", "ufw_enforcement"] },

    // --- Outcome (6) ---
    Beat { id: "ca_out_generic", phase: Outcome, criteria: &[], voice: Driver,
        lines: &["So. That's their answer.", "We have their word, for what it's worth."],
        moves: &[
            mv("weigh", 1.0, Room, &["What do we make of it?", "Folk talk it over half the night."], Nothing, to(Learn)),
            mv("next", 0.5, Member, &["Whatever it is, we're not done.", "There's always the next one."], Nothing, to(Renew)),
        ],
        sources: &["remenca_redemption"] },
    Beat { id: "ca_out_won", phase: Outcome, criteria: &[Has("arc:won")], voice: Driver,
        lines: &["They've granted it. We've won on {issue}.", "We won. It's given."],
        moves: &[
            mv("watch", 1.0, Driver, &["Now we watch they keep it.", "A grant's only good while it's kept."], Nothing, to(Enforce)),
            mv("feast", 0.8, Room, &["A feast in the green!", "Ale for every hearth."], Nothing, to(Enforce)),
            mv("next_fight", 0.3, Member, &["Good. Now mine.", "One down. What's next?"], Nothing, to(Renew)),
        ],
        sources: &["ufw_enforcement"] },
    Beat { id: "ca_out_lost", phase: Outcome, criteria: &[Has("arc:lost")], voice: Driver,
        lines: &["They've refused us flat.", "Nothing. Not a penny."],
        moves: &[
            mv("learn", 1.0, Driver, &["Then we learn why.", "Lost this time. Let's see what broke."], Nothing, to(Learn)),
            mv("bitter", 0.4, Member, &["I said this would happen. Nobody listened.", "All that for nothing. I'm done."], Dissent, to(Learn)),
        ],
        sources: &["waldensian_cells"] },
    Beat { id: "ca_out_partial", phase: Outcome, criteria: &[Has("arc:partial")], voice: Driver,
        lines: &["Half of it. Not nothing.", "They gave some. Not all."],
        moves: &[
            mv("hold_half", 1.0, Driver, &["Hold what we won. Then go for the rest.", "Keep the half safe first."], Nothing, to(Enforce)),
            mv("push_on", 0.6, Member, &["Half is a start, not an end.", "They bent once. They'll bend again."], Nothing, to(Renew)),
            mv("sold_out", 0.3, Member, &["Half is a sell-out.", "We settled for crumbs."], Dissent, to(Learn)),
        ],
        sources: &["remenca_redemption"] },
    Beat { id: "ca_out_delayed", phase: Outcome, criteria: &[], voice: Opponent,
        lines: &["The lord will consider it after harvest.", "It goes before the manor court. In time."],
        moves: &[
            mv("wait", 0.6, Room, &["After harvest, then. We'll remember.", "We'll be waiting."], Nothing, to(Learn)),
            mv("press", 1.0, Driver, &["After harvest is never. We press now.", "Delay is a no in a fine coat."], Nothing, to(Renew)),
        ],
        sources: &["hussite_compacts"] },
    Beat { id: "ca_out_won_split", phase: Outcome, criteria: &[Has("arc:won"), Has("arc:split")], voice: Dissenter,
        lines: &["Ye won yours. Mine's still owed.", "Fine for the mill folk. The rest of us?"],
        moves: &[
            mv("thine_next", 1.0, Driver, &["Thine's next. On my word.", "We go for thine now, with all of us."], Nothing, to(Renew)),
            mv("still_owed", 0.5, Dissenter, &["I'll believe it when I see it.", "Words. Again."], Dissent, to(Enforce)),
        ],
        sources: &["hussite_compacts"] },

    // --- Enforce (5) ---
    Beat { id: "ca_enf_generic", phase: Enforce, criteria: &[], voice: Member,
        lines: &["Is he keeping his word?", "Has anyone checked the toll this week?"],
        moves: &[
            mv("kept", 1.0, Member, &["So far. I go by the mill every morning.", "Kept, aye. For now."], Nothing, to(Renew)),
            mv("slipping", 0.5, Member, &["He's slipping. A penny here, a penny there.", "The old toll creeps back."], Nothing, to(Counter)),
        ],
        sources: &["ufw_enforcement"] },
    Beat { id: "ca_enf_promise_broken", phase: Enforce, criteria: &[Has("arc:promise_broken")], voice: Member,
        lines: &["The toll's back on, like it never came off.", "They took it back. Every bit."],
        moves: &[
            mv("again", 1.0, Driver, &["Then we go again - and this time we hold the paper.", "Back to the square. We know the way."], Nothing, to(Counter)),
            mv("weary", 0.4, Room, &["Tired faces. Some won't go twice.", "Half the room shakes its head."], Nothing, to(Learn)),
        ],
        sources: &["reconstruction_redemption"] },
    Beat { id: "ca_enf_noted", phase: Enforce, criteria: &[Has("seat:note_taker")], voice: NoteTaker,
        lines: &["I have it written in their clerk's hand.", "The paper's in the chest. Shall I bring it?"],
        moves: &[
            mv("bring_it", 1.0, NoteTaker, &["Read it at the hall gate, loud.", "I'll read it to the steward's face."], Resolve, to(Renew)),
            mv("paper_nothing", 0.4, Member, &["Paper's nothing if they don't fear us.", "Ink doesn't make knights blush."], Nothing, to(Counter)),
        ],
        sources: &["ufw_enforcement", "guild_ordinance"] },
    Beat { id: "ca_enf_won", phase: Enforce, criteria: &[Has("arc:won")], voice: Driver,
        lines: &["A win's not kept by itself. Who'll watch the mill?", "We need eyes on it, every week."],
        moves: &[
            mv("watcher", 1.0, Member, &["I'll watch it. I live by there.", "Me and my sister, turn about."], Nothing, to(Renew)),
            mv("rest", 0.5, Room, &["We won. Let it rest.", "Nobody wants to think on it now."], Nothing, to(Learn)),
        ],
        sources: &["reconstruction_redemption", "ufw_enforcement"] },
    Beat { id: "ca_enf_crackdown", phase: Enforce, criteria: &[Has("arc:crackdown")], voice: Opponent,
        lines: &["Now they're taking back what was granted, one house at a time.", "The bailiff comes at night, house by house."],
        moves: &[
            mv("stand", 1.0, Driver, &["Then every door opens when he knocks - all of us.", "He comes to one, he finds twenty."], Nothing, to(Counter)),
            mv("quiet", 0.5, Room, &["Folk keep quiet and hope.", "Nobody speaks of it in the lanes."], Nothing, to(Learn)),
        ],
        sources: &["reconstruction_redemption"] },

    // --- Learn (5) ---
    Beat { id: "ca_learn_generic", phase: Learn, criteria: &[], voice: Driver,
        lines: &["What did we learn?", "Let's speak it plain - what went right, what went wrong."],
        moves: &[
            mv("onward", 1.0, Member, &["We're stronger than we were.", "We know the steward now. That's worth something."], Nothing, to(Renew)),
            mv("rebuild", 0.6, Member, &["We need more of us.", "Too few, too fast."], Nothing, to(Build)),
        ],
        sources: &["citizenship_schools"] },
    Beat { id: "ca_learn_lost", phase: Learn, criteria: &[Has("arc:lost")], voice: Member,
        lines: &["We lost. We're still here.", "They won this time. We're not gone."],
        moves: &[
            mv("still_here", 1.0, Room, &["Still here.", "Aye. Still here."], Nothing, to(Renew)),
            mv("quieter", 0.6, Driver, &["Quieter, then. Smaller, and longer.", "Underground for a while."], Nothing, to(Build)),
        ],
        sources: &["waldensian_cells", "exodus_assembly"] },
    Beat { id: "ca_learn_crackdown", phase: Learn, criteria: &[Has("arc:crackdown")], voice: Member,
        lines: &["They showed their hand. The folk on the fence saw it.", "Knights on peaceful folk. People talk."],
        moves: &[
            mv("turned", 0.8, Driver, &["Some who laughed at us before are asking now.", "The miller's wife joined. After that."], Nothing, to(Renew)),
            mv("cowed", 1.0, Member, &["Or it frightened them off for good.", "Nobody new came. They're scared."], Nothing, to(Build)),
        ],
        sources: &["repression_backfire"] },
    Beat { id: "ca_learn_weak", phase: Learn, criteria: &[Has("arc:weak")], voice: Driver,
        lines: &["We asked too big for our numbers.", "We reached past our arms."],
        moves: &[
            mv("build", 1.0, Room, &["Build first, then.", "More hands before more asks."], Nothing, to(Build)),
            mv("smaller", 0.6, Member, &["Smaller asks, more often.", "A string of little wins."], Nothing, to(Renew)),
        ],
        sources: &["salt_march_demand"] },
    Beat { id: "ca_learn_won", phase: Learn, criteria: &[Has("arc:won")], voice: Member,
        lines: &["What won it? Write it down, so the young ones know.", "Tell it to the children. How we did it."],
        moves: &[
            mv("tell", 1.0, Driver, &["We held together. That's all of it.", "Nobody broke. That's the whole tale."], Nothing, to(Renew)),
            mv("luck", 0.4, Member, &["Half of it was luck, and we should say so.", "The lord needed coin. We were lucky."], Nothing, to(Renew)),
        ],
        sources: &["citizenship_schools"] },

    // --- Renew (4) ---
    Beat { id: "ca_renew_generic", phase: Renew, criteria: &[], voice: Driver,
        lines: &["What's next?", "Where do we go from here?"],
        moves: &[
            mv("new_target", 1.0, Member, &["The commons. They've been fencing it.", "The tithe barn. Next."], Nothing, to(Target)),
            mv("rest", 0.6, Room, &["Rest a season. Then.", "Let the fields have us for a while."], Nothing, END),
        ],
        sources: &["citizenship_schools"] },
    Beat { id: "ca_renew_won", phase: Renew, criteria: &[Has("arc:won")], voice: Member,
        lines: &["The weavers saw us win. They want help.", "Other villages are asking how."],
        moves: &[
            mv("help", 1.0, Driver, &["Then we go help them.", "Teach them what we learned."], Nothing, to(Target)),
            mv("ours_first", 0.5, Member, &["Our own first. Keep what we've won.", "Not yet. We're still tender."], Nothing, END),
        ],
        sources: &["section_504_coalition"] },
    Beat { id: "ca_renew_lost", phase: Renew, criteria: &[Has("arc:lost")], voice: Driver,
        lines: &["Same fight, new way.", "We don't drop it. We change how."],
        moves: &[
            mv("again", 1.0, Room, &["Again, then.", "Again, and smarter."], Nothing, to(Target)),
            mv("wait", 0.5, Member, &["Not this year.", "Let the anger cool first."], Nothing, END),
        ],
        sources: &["exodus_assembly"] },
    Beat { id: "ca_renew_succession", phase: Renew, criteria: &[], voice: Member,
        lines: &["Thou'rt tired. Let someone else lead the next one.", "New hands on the plough, this time."],
        moves: &[
            mv("hand_over", 1.0, Driver, &["Aye. I'll stand behind, for once.", "Take it. I'll show thee what I know."], Nothing, to(Target)),
            mv("not_yet", 0.4, Driver, &["One more fight. Then.", "Not yet. Soon."], Nothing, END),
        ],
        sources: &["citizenship_schools"] },
];
