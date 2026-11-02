//! Campaign decision content: the Stage 1 tranche of 50 house-meeting
//! beats and 50 campaign-arc beats, plus the source patterns they borrow.
//! The model and its audit live in `campaign_dialogue`.
//!
//! Authoring rules (wiki/software-design/campaign-dialogue-trees-plan.md):
//! - Every phase keeps at least one criteria-free beat - the fallback.
//! - A beat is a situation put to the room; its moves are what the room
//!   can do about it. Weights are relative priors, not history's odds.
//! - The village voice: short, concrete, period words; no modern terms.
//!   Source tags carry the history; the lines never lecture.

use crate::game::campaign::Framing;
use crate::game::campaign_dialogue::{Beat, Evidence, Hook, Move, Next, Phase, SourcePattern, Voice};
use crate::game::conversation::Criterion::*;
use Hook::*;
use Phase::*;
use Voice::*;

const fn to(p: Phase) -> Next { Next::Phase(p) }
const END: Next = Next::Close;

/// A move with no criteria or bias of its own; struct-update it to add them
const fn mv(id: &'static str, weight: f32, voice: Voice, lines: &'static [&'static str], hook: Hook, next: Next) -> Move {
    Move { id, criteria: &[], weight, bias: &[], voice, lines, hook, next }
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
    SourcePattern { tag: "outlaw_ballads", origin: "Robin Hood and greenwood ballads", evidence: Evidence::Legendary,
        mechanism: "a told story of the taken brought home lends nerve to those left behind" },
];

// ---------------------------------------------------------------------
// House meetings: 50 beats

pub const MEETING_BEATS: &[Beat] = &[
    // --- Agenda (6) ---
    Beat { id: "hm_agenda_generic", phase: Agenda, criteria: &[], voice: Driver,
        lines: &["We're met about {issue}. What comes first?", "Sit, all of you. {issue} brings us here - where do we start?"],
        moves: &[
            mv("hear_room", 1.0, Room, &["Let folk say their piece before aught is settled.", "Hear the room first, then plan."], Nothing, to(Floor)),
            mv("worst_first", 0.8, Member, &["Let them who bear it worst speak first.", "Start with the houses hit hardest."], Nothing, to(Testimony)),
            Move { bias: &[("history:seasoned", 1.6)],
                ..mv("straight_to_it", 0.6, Member, &["We all know the trouble. Put a remedy to us.", "Enough telling. What do we do?"], Nothing, to(Proposal)) },
        ],
        sources: &["athenian_assembly"] },
    Beat { id: "hm_agenda_first_meeting", phase: Agenda, criteria: &[Has("history:first")], voice: Driver,
        lines: &["This is our first sitting on {issue}. Let's agree how we'll talk.", "We've never met like this before. A word on how we do it."],
        moves: &[
            mv("one_at_a_time", 1.0, Room, &["One at a time, and no shouting.", "Each speaks once before anyone speaks twice."], Nothing, to(Floor)),
            mv("who_keeps_order", 0.6, Member, &["Who keeps the order? Somebody must.", "Pick someone to say whose turn it is."], Nothing, to(Floor)),
            mv("just_begin", 0.5, Member, &["Rules can wait. My roof can't.", "We'll learn the manner of it by doing."], Nothing, to(Testimony)),
        ],
        sources: &["exodus_assembly", "guild_ordinance"] },
    Beat { id: "hm_agenda_drifted", phase: Agenda, criteria: &[Has("room:drifted")], voice: Driver,
        lines: &["Nobody called this, but here we sit. Shall I begin?", "Folk came because I asked them. I'll start, if none object."],
        moves: &[
            mv("go_on", 1.0, Room, &["Go on, then. Thou hast all our ears.", "Aye, thou'rt as good as any to start."], Nothing, to(Floor)),
            Move { bias: &[("seat:chair", 1.5)],
                ..mv("who_made_thee", 0.5, Member, &["Who made thee chair? Let them we chose speak.", "We never gave thee the floor to give."], Dissent, to(Objection)) },
        ],
        sources: &["freeman_structurelessness"] },
    Beat { id: "hm_agenda_chair", phase: Agenda, criteria: &[Has("seat:chair")], voice: Chair,
        lines: &["The chair has the order of it: {issue} first, then all else.", "By the chair's list - {issue}, then any other business."],
        moves: &[
            mv("as_listed", 1.0, Room, &["As the chair says.", "Fair enough. Go in order."], Nothing, to(Testimony)),
            mv("open_it_first", 0.5, Member, &["Open the floor a moment first. Some have news.", "Before the list - let the quiet ones speak."], Nothing, to(Floor)),
        ],
        sources: &["guild_ordinance"] },
    Beat { id: "hm_agenda_absent_dissent", phase: Agenda, criteria: &[Has("room:dissent_absent")], voice: Driver,
        lines: &["Some who voted for another fight aren't here tonight.", "There are empty stools - and I know whose."],
        moves: &[
            mv("send_word", 0.8, Member, &["Send word to them after. Don't decide behind their backs.", "Carry what we say to their doors."], Nothing, to(Floor)),
            mv("proceed", 1.0, Driver, &["We go on with who's here. They know where we meet.", "The door was open. We can't wait on them."], Nothing, to(Proposal)),
        ],
        sources: &["land_war_boycott"] },
    Beat { id: "hm_agenda_seasoned", phase: Agenda, criteria: &[Has("history:seasoned")], voice: Driver,
        lines: &["We've met on {issue} many a time. What's changed since last?", "Another sitting. Tell me what's moved."],
        moves: &[
            mv("report", 1.0, Member, &["Some things. Let me tell it.", "More than thou'd think. Hear this."], Nothing, to(Testimony)),
            mv("nothing_changed", 0.6, Member, &["Naught has changed, and that's the trouble.", "Same as last time. Same as the time before."], Nothing, to(Objection)),
        ],
        sources: &["montgomery_mass_meeting"] },

    // --- Floor (6) ---
    Beat { id: "hm_floor_generic", phase: Floor, criteria: &[], voice: Driver,
        lines: &["Who'll speak first?", "The floor is open. Who has something?"],
        moves: &[
            mv("one_rises", 1.0, Member, &["I will. I've held it in long enough.", "Me. Mine's short."], Nothing, to(Testimony)),
            mv("silence", 0.5, Driver, &["None speak. Then I'll put a thought to you.", "Quiet as a church. I'll start us off."], Nothing, to(Proposal)),
            Move { bias: &[("room:drifted", 1.6)],
                ..mv("loudest", 0.4, Member, &["I'll speak, and I'm against the lot of it.", "Somebody has to say it - this is foolish."], Nothing, to(Objection)) },
        ],
        sources: &["athenian_assembly"] },
    Beat { id: "hm_floor_thin", phase: Floor, criteria: &[Has("room:thin")], voice: Driver,
        lines: &["There's few of us tonight. Speak plain, then.", "A small room. No need for speeches."],
        moves: &[
            mv("plain", 1.0, Member, &["Plain it is. It's getting worse.", "I'll be quick: we're hurting."], Nothing, to(Testimony)),
            Move { bias: &[("rule:consensus", 1.5)],
                ..mv("too_few", 0.5, Member, &["So few - should we decide for the rest at all?", "Let's not settle aught with half the village abed."], Nothing, END) },
        ],
        sources: &["polletta_decision_rule"] },
    Beat { id: "hm_floor_full", phase: Floor, criteria: &[Has("room:full")], voice: Driver,
        lines: &["The room's full. Speak short, or we'll be here till cockcrow.", "Look at all of you. Keep it brief, each one."],
        moves: &[
            mv("hands_up", 1.0, Room, &["A forest of hands goes up.", "Half the room wants the floor at once."], Nothing, to(Testimony)),
            mv("shouting", 0.4, Member, &["Over the din: some of us want action, not talk!", "Shouting from the back: get on with it!"], Nothing, to(Objection)),
        ],
        sources: &["montgomery_mass_meeting"] },
    Beat { id: "hm_floor_chair", phase: Floor, criteria: &[Has("seat:chair")], voice: Chair,
        lines: &["The chair calls first on those who've not spoken before.", "New voices first. Thou in the corner?"],
        moves: &[
            mv("quiet_one", 1.0, Member, &["I... aye. I'll try.", "I never thought to be asked. Here's how it is."], Nothing, to(Testimony)),
            mv("declines", 0.5, Member, &["Not me. Let someone with a stronger voice.", "I'll listen tonight, if it's all the same."], Nothing, to(Proposal)),
        ],
        sources: &["guild_ordinance", "citizenship_schools"] },
    Beat { id: "hm_floor_drifted", phase: Floor, criteria: &[Has("room:drifted")], voice: Driver,
        lines: &["Folk here look to me. I'll say who speaks.", "I'll call on folk - I know who's got something."],
        moves: &[
            mv("let_it", 1.0, Room, &["Nobody minds. Easier that way.", "Heads nod. It's always been so."], Nothing, to(Proposal)),
            mv("push_back", 0.6, Member, &["Thou'lt call thy friends and skip the rest.", "And who calls on thee to sit down?"], Dissent, to(Objection)),
        ],
        sources: &["freeman_structurelessness"] },
    Beat { id: "hm_floor_consensus", phase: Floor, criteria: &[Has("rule:consensus")], voice: Driver,
        lines: &["We go round the circle. Nobody's passed over.", "Round the hearth, each in turn."],
        moves: &[
            mv("round", 1.0, Room, &["Round it goes, one after another.", "Each says a little, and it adds up."], Nothing, to(Testimony)),
            mv("slow", 0.5, Member, &["This'll take all night.", "By the time it comes round I'll have forgot mine."], Nothing, to(Proposal)),
        ],
        sources: &["polletta_decision_rule"] },

    // --- Testimony (6) ---
    Beat { id: "hm_testimony_generic", phase: Testimony, criteria: &[], voice: Member,
        lines: &["Let me tell how {issue} falls on my house.", "I'll say what it's cost us, plain as I can."],
        moves: &[
            mv("moved", 1.0, Room, &["The room goes still. Then: what do we do?", "A low murmur. Nobody doubts it."], Nothing, to(Proposal)),
            mv("me_too", 0.8, Member, &["Same at mine. Worse, maybe.", "Thou'rt not alone. Ask the far end of the lane."], Nothing, to(Coalition)),
            mv("doubt", 0.4, Member, &["That's thy trouble, not the whole village's.", "Hard luck, but it's no cause for all of us."], Nothing, to(Objection)),
        ],
        sources: &["montgomery_mass_meeting"] },
    Beat { id: "hm_testimony_dissent", phase: Testimony, criteria: &[Has("room:dissent_heard")], voice: Dissenter,
        lines: &["I voted for another fight. Hear why before ye ask me to march.", "My trouble lost the vote. It didn't go away."],
        moves: &[
            mv("heard", 1.0, Driver, &["We hear thee. Thy trouble goes on the list.", "Fair. It's not forgot."], Nothing, to(Proposal)),
            Move { bias: &[("rule:majority", 1.4)],
                ..mv("unsatisfied", 0.5, Dissenter, &["Listening's cheap. I'm still not with you.", "Nodding at me isn't the same as helping."], Dissent, to(Objection)) },
            Move { bias: &[("framing:foe", 1.5)],
                ..mv("reframe_owed", 0.4, Driver, &["Then let's not put it as a hanging. It's what's owed to all of us.", "Put it as a debt the village is owed, and it's thine as much as ours."], Reframe(Framing::MoralDuty), to(Proposal)) },
        ],
        sources: &["snow_frame_alignment", "polletta_decision_rule"] },
    Beat { id: "hm_testimony_foe", phase: Testimony, criteria: &[Has("framing:foe")], voice: Member,
        lines: &["Name the man. Everyone knows who's behind {issue}.", "We all know whose purse fattens on {issue}."],
        moves: &[
            mv("name_him", 1.0, Room, &["The name goes round the room in a whisper.", "Nobody says it loud, but every head nods."], Nothing, to(Proposal)),
            mv("careful", 0.6, Member, &["Say it loud and he'll know our names by morning.", "Walls have ears, and some ears drink at the hall."], Nothing, to(Objection)),
            mv("not_one_man", 0.3, Driver, &["It's not one man. We all go short together.", "Hang him and the next one's the same. It's all of us carrying it."], Reframe(Framing::SharedGrievance), to(Proposal)),
        ],
        sources: &["snow_frame_alignment", "montaillou_register"] },
    Beat { id: "hm_testimony_duty", phase: Testimony, criteria: &[Has("framing:duty")], voice: Member,
        lines: &["The priest says we owe each other. Then why does {issue} go unanswered?", "We're told charity's a duty. Whose?"],
        moves: &[
            mv("duty", 1.0, Room, &["Aye. Hold them to their own sermon.", "If it's owed, let it be paid."], Nothing, to(Proposal)),
            mv("belly", 0.5, Member, &["Fine words. My children want bread.", "Duty never thatched a roof."], Nothing, to(Objection)),
            mv("shared", 0.4, Driver, &["Then put it plain: we're all hungry, and we'll share it out.", "Leave the pulpit out. We go short together."], Reframe(Framing::SharedGrievance), to(Proposal)),
        ],
        sources: &["jubilee_release", "snow_frame_alignment"] },
    Beat { id: "hm_testimony_grievance", phase: Testimony, criteria: &[Has("framing:grievance")], voice: Member,
        lines: &["We all carry {issue}. Let's count how many houses.", "Hands up, every house that's felt it."],
        moves: &[
            mv("count", 1.0, Room, &["Most hands go up. Some slowly.", "Nearly every hand. Even the miller's wife."], Nothing, to(Proposal)),
            mv("somebody_profits", 0.5, Member, &["It's no weather. Somebody profits by it.", "Shared? Then why's one house fatter for it?"], Reframe(Framing::CommonFoe), to(Proposal)),
        ],
        sources: &["exodus_assembly"] },
    Beat { id: "hm_testimony_first", phase: Testimony, criteria: &[Has("history:first")], voice: Member,
        lines: &["I've never said this aloud before.", "I don't know how to say this in front of folk."],
        moves: &[
            mv("support", 1.0, Room, &["Go on. We're with thee.", "Take thy time. None here will laugh."], Nothing, to(Coalition)),
            mv("hush", 0.4, Member, &["Careful whose ears are at the window.", "Lower, friend. Not everyone here is a friend."], Nothing, to(Objection)),
        ],
        sources: &["montgomery_mass_meeting", "waldensian_cells"] },

    // --- Proposal (6) ---
    Beat { id: "hm_proposal_generic", phase: Proposal, criteria: &[], voice: Driver,
        lines: &["Here's what I'd have us do about {issue}.", "I'll put a plan to the room."],
        moves: &[
            mv("nods", 1.0, Room, &["Nods around the hearth.", "That'll do. Put it to us."], Nothing, to(Decision)),
            mv("mend_it", 0.7, Member, &["Good, but it wants changing.", "Near enough. One thing, though."], Nothing, to(Amendment)),
            mv("against", 0.4, Member, &["I'll not have it.", "That's a fool's plan and I'll say why."], Nothing, to(Objection)),
        ],
        sources: &["athenian_assembly"] },
    Beat { id: "hm_proposal_petition", phase: Proposal, criteria: &[], voice: Member,
        lines: &["Let's write it down - every name, every wrong - and carry it to the hall.", "A list. Our marks and our grievances, in ink."],
        moves: &[
            mv("aye", 1.0, Room, &["Aye, a list. Let them read it.", "In ink, so they can't say they never heard."], Nothing, to(Decision)),
            mv("who_writes", 0.6, Member, &["Who among us has letters?", "The clerk won't write it for us."], Nothing, to(Amendment)),
            mv("names_danger", 0.4, Member, &["Our names on a list? In their hands?", "A list of names is a gift to the hall."], Nothing, to(Objection)),
        ],
        sources: &["grievance_articles", "montaillou_register"] },
    Beat { id: "hm_proposal_small_demand", phase: Proposal, criteria: &[], voice: Driver,
        lines: &["Ask for one thing we can win, not the whole of it.", "Start with what they can't refuse without shame."],
        moves: &[
            mv("agree", 1.0, Room, &["One thing. Aye.", "Small and sure. Good."], Nothing, to(Decision)),
            mv("broader", 0.6, Member, &["Win a crumb and they'll think us satisfied.", "Ask small and we'll get smaller."], Nothing, to(Amendment)),
        ],
        sources: &["salt_march_demand"] },
    Beat { id: "hm_proposal_seasoned", phase: Proposal, criteria: &[Has("history:seasoned")], voice: Driver,
        lines: &["We've talked enough. It's time to act on {issue}.", "No more sittings without a deed after."],
        moves: &[
            mv("act", 1.0, Room, &["Past time.", "Aye. Talking's worn thin."], Nothing, to(Decision)),
            mv("not_yet", 0.5, Member, &["We're not strong enough. Build first.", "Act now and we'll be few in the square."], Nothing, to(Amendment)),
        ],
        sources: &["labor_strike"] },
    Beat { id: "hm_proposal_thin", phase: Proposal, criteria: &[Has("room:thin")], voice: Driver,
        lines: &["We're few, so let's ask small and ask together.", "Two or three of us can still carry one ask."],
        moves: &[
            mv("together", 1.0, Member, &["Together, then. I'll stand by thee.", "Few is more than none."], Nothing, to(Decision)),
            mv("wait_for_more", 0.6, Member, &["Let's wait till more come. Next time.", "Not with so few. Not yet."], Nothing, END),
        ],
        sources: &["salt_march_demand"] },
    Beat { id: "hm_proposal_noted", phase: Proposal, criteria: &[Has("room:noted")], voice: NoteTaker,
        lines: &["Last time we said we'd do this. I have it written. Did we?", "It's in the notes from last sitting. Who kept to it?"],
        moves: &[
            mv("shamed", 1.0, Room, &["We didn't. Let's do it now, then.", "Sheepish looks. Then: aye, this time."], Nothing, to(Decision)),
            mv("changed", 0.6, Driver, &["Things changed. The plan wants mending.", "We learned since. Let's alter it."], Nothing, to(Amendment)),
        ],
        sources: &["guild_ordinance"] },

    // --- Amendment (5) ---
    Beat { id: "hm_amend_generic", phase: Amendment, criteria: &[], voice: Member,
        lines: &["I'd change one word in it.", "Keep the plan, but not the way it's put."],
        moves: &[
            mv("accepted", 1.0, Room, &["Changed, then. Better for it.", "Fair. Mend it so."], Nothing, to(Decision)),
            mv("stands", 0.6, Driver, &["The plan stands as put.", "No. Leave it be, we'll muddle it."], Nothing, to(Decision)),
            mv("spiral", 0.3, Member, &["And if we change that, we must change this...", "Then the whole thing's wrong, isn't it?"], Nothing, to(Objection)),
        ],
        sources: &["guild_ordinance"] },
    Beat { id: "hm_amend_consensus", phase: Amendment, criteria: &[Has("rule:consensus")], voice: Driver,
        lines: &["Who cannot live with this? Speak now, so we can mend it.", "If it sticks in anyone's throat, say so before we settle."],
        moves: &[
            mv("mended", 1.0, Member, &["Shift the day and I can live with it.", "Leave my strip out of it and I'm content."], Nothing, to(Decision)),
            mv("block", 0.4, Member, &["I can't. Not like this.", "No mending will make me swallow it."], Dissent, to(Objection)),
        ],
        sources: &["polletta_decision_rule"] },
    Beat { id: "hm_amend_dissent", phase: Amendment, criteria: &[Has("room:dissent_heard")], voice: Dissenter,
        lines: &["Fold my trouble into it, and I'll stand with you.", "Put a word in for mine, and ye have me."],
        moves: &[
            mv("folded", 1.0, Driver, &["Done. It goes in.", "Aye, there's room for thine."], Nothing, to(Decision)),
            mv("refused", 0.5, Dissenter, &["One fight at a time, is it? Then count me out.", "So mine waits again. I've waited enough."], Dissent, to(Decision)),
        ],
        sources: &["snow_frame_alignment"] },
    Beat { id: "hm_amend_soften", phase: Amendment, criteria: &[], voice: Member,
        lines: &["Ask it softer. Give them a way to say yes and keep face.", "Leave them a door to walk through."],
        moves: &[
            mv("soften", 1.0, Room, &["Softer, then. Honey for the hall.", "A door for them. Clever."], Nothing, to(Decision)),
            mv("no_soft", 0.6, Member, &["Soft words got us nothing last year.", "They'll take the door and slam it on us."], Nothing, to(Decision)),
        ],
        sources: &["hussite_compacts"] },
    Beat { id: "hm_amend_foe_to_duty", phase: Amendment, criteria: &[Has("framing:foe")], voice: Member,
        lines: &["Don't make it a hanging matter. Put it as what's owed.", "Less blame, more debt. Folk will follow a debt."],
        moves: &[
            mv("reframe", 0.6, Room, &["As what's owed, then.", "A debt. Aye, that sits better."], Reframe(Framing::MoralDuty), to(Decision)),
            mv("keep_foe", 1.0, Driver, &["No. He did it, and folk should know he did.", "Soften it and he walks free of it."], Nothing, to(Decision)),
        ],
        sources: &["snow_frame_alignment", "jubilee_release"] },

    // --- Objection (6) ---
    Beat { id: "hm_objection_generic", phase: Objection, criteria: &[], voice: Member,
        lines: &["I'm against it, and I'll say why.", "Hold. I've a word against this."],
        moves: &[
            Move { bias: &[("rule:consensus", 2.0)],
                ..mv("talk_it_out", 1.0, Driver, &["Then help us mend it.", "Say what would bring thee round."], Nothing, to(Amendment)) },
            Move { bias: &[("rule:majority", 2.0)],
                ..mv("call_question", 1.0, Driver, &["Heard. Let's put it to the room.", "Thy word's on the record. Hands now."], Nothing, to(Decision)) },
            mv("let_it_lie", 0.4, Room, &["Let it lie a week. Tempers are up.", "Sleep on it. All of us."], Nothing, END),
        ],
        sources: &["polletta_decision_rule"] },
    Beat { id: "hm_objection_fear", phase: Objection, criteria: &[], voice: Member,
        lines: &["If we do this, who loses their place first? Not thee.", "Who pays when the hall hits back?"],
        moves: &[
            mv("together", 1.0, Driver, &["Then we go together, or not at all.", "Nobody stands alone. That's the whole point."], Nothing, to(Coalition)),
            mv("fear_wins", 0.5, Room, &["Uneasy silence. Nobody will say it first.", "Eyes on the floor. The fear has the room."], Nothing, END),
        ],
        sources: &["commune_oath"] },
    Beat { id: "hm_objection_drifted", phase: Objection, criteria: &[Has("room:drifted")], voice: Member,
        lines: &["Who decided this? I don't remember a vote.", "When did we agree to that? I was here the whole time."],
        moves: &[
            mv("vote", 1.0, Driver, &["Fair. We'll vote it now.", "Then let's have hands on it, proper."], Nothing, to(Decision)),
            mv("seat_someone", 0.6, Member, &["Then seat someone to keep order, so it's not who shouts loudest.", "Choose a chair. Out loud, this time."], Nothing, to(Decision)),
        ],
        sources: &["freeman_structurelessness"] },
    Beat { id: "hm_objection_dissent", phase: Objection, criteria: &[Has("room:dissent_heard")], voice: Dissenter,
        lines: &["I lost the vote. I didn't lose my tongue.", "Ye won. That doesn't make ye right."],
        moves: &[
            mv("heard_out", 1.0, Room, &["Fair. Say thy piece and we'll weigh it.", "Speak. We'll mend what we can."], Nothing, to(Amendment)),
            mv("walks", 0.4, Dissenter, &["I'll stay. But I'll not pretend to be glad of it.", "Do it without me, then."], Dissent, to(Decision)),
        ],
        sources: &["polletta_decision_rule"] },
    Beat { id: "hm_objection_thin", phase: Objection, criteria: &[Has("room:thin")], voice: Member,
        lines: &["There's not enough of us to carry this.", "Look around. Who'll stand with us?"],
        moves: &[
            mv("recruit", 1.0, Driver, &["Then we bring more. Each of us brings one.", "We fill this room before we fill the square."], Nothing, to(Coalition)),
            mv("anyway", 0.5, Member, &["Few is enough if few are steady.", "Do it anyway. Others will follow if we start."], Nothing, to(Decision)),
        ],
        sources: &["abolition_petitions"] },
    Beat { id: "hm_objection_seasoned", phase: Objection, criteria: &[Has("history:seasoned")], voice: Member,
        lines: &["We tried this before and it went bad.", "Same plan as last spring. Same end, I'd wager."],
        moves: &[
            mv("learned", 1.0, Driver, &["Aye, and now we know where it broke.", "We do it different this time - we learned."], Nothing, to(Amendment)),
            mv("agreed", 0.4, Room, &["Fair point. Let's think again.", "Grumbles of agreement. It's set aside."], Nothing, END),
        ],
        sources: &["citizenship_schools"] },

    // --- Coalition (5) ---
    Beat { id: "hm_coalition_generic", phase: Coalition, criteria: &[], voice: Member,
        lines: &["The weavers have the same trouble. Bring them in.", "We're not the only lane hurting. Ask the others."],
        moves: &[
            mv("aye", 1.0, Room, &["More hands, lighter work.", "Ask them. What's the harm?"], Nothing, to(Decision)),
            mv("their_own", 0.5, Member, &["They'll want their own fight first.", "They'll bring their quarrel and drop ours."], Nothing, to(Amendment)),
        ],
        sources: &["guild_ordinance", "section_504_coalition"] },
    Beat { id: "hm_coalition_clergy", phase: Coalition, criteria: &[], voice: Member,
        lines: &["If the priest would stand with us, the hall would think twice.", "Get a cassock at the front and they'll not dare."],
        moves: &[
            mv("ask_him", 1.0, Driver, &["I'll ask him after mass.", "Worth the asking."], Nothing, to(Decision)),
            mv("dines_at_hall", 0.6, Member, &["The priest dines at the hall.", "He'll bless them, not us."], Nothing, to(Decision)),
        ],
        sources: &["section_504_coalition"] },
    Beat { id: "hm_coalition_full", phase: Coalition, criteria: &[Has("room:full")], voice: Driver,
        lines: &["Look at this room. We're more than they think.", "Count heads. Then tell me we're weak."],
        moves: &[
            mv("count", 1.0, Room, &["A cheer, quickly hushed.", "Folk look round, and stand a little straighter."], Nothing, to(Decision)),
            mv("crowd_not_army", 0.4, Member, &["A full room isn't a steady one.", "Crowds come. Crowds go home."], Nothing, to(Amendment)),
        ],
        sources: &["montgomery_mass_meeting"] },
    Beat { id: "hm_coalition_absent", phase: Coalition, criteria: &[Has("room:dissent_absent")], voice: Driver,
        lines: &["Those who stayed home - someone must walk to their doors.", "The empty stools. Who'll fetch them back?"],
        moves: &[
            mv("i_will", 1.0, Member, &["I'll go. I know them.", "Leave them to me. They'll listen to kin."], Nothing, to(Decision)),
            mv("let_them_be", 0.5, Member, &["Let them be. They chose.", "Chase them and they'll dig in."], Nothing, to(Decision)),
        ],
        sources: &["land_war_boycott"] },
    Beat { id: "hm_coalition_consensus", phase: Coalition, criteria: &[Has("rule:consensus")], voice: Member,
        lines: &["Let each household send one voice, so none speak for others.", "One voice a hearth. Nobody carries two."],
        moves: &[
            mv("aye", 1.0, Room, &["Fair and even.", "One a hearth. Aye."], Nothing, to(Decision)),
            mv("slow", 0.5, Member, &["That'll take till Michaelmas.", "Fair, and slow as a plough ox."], Nothing, to(Decision)),
        ],
        sources: &["section_504_coalition", "commune_oath"] },

    // --- Decision (6) ---
    Beat { id: "hm_decision_generic", phase: Decision, criteria: &[], voice: Driver,
        lines: &["Are we agreed?", "Then it's put. Hands, if ye'll have it."],
        moves: &[
            mv("carried", 1.0, Room, &["Carried.", "Agreed, near enough all."], Nothing, to(FollowThrough)),
            mv("split", 0.4, Room, &["Hands half up, half down.", "Too close to call. Nobody's happy."], Nothing, END),
            mv("tabled", 0.3, Room, &["Not tonight. Bring it back.", "Put off. Too many doubts."], Nothing, END),
        ],
        sources: &["athenian_assembly"] },
    Beat { id: "hm_decision_noted", phase: Decision, criteria: &[Has("room:noted")], voice: NoteTaker,
        lines: &["Say it plain, so I can write it.", "Word for word, so it holds tomorrow."],
        moves: &[
            mv("written", 1.0, NoteTaker, &["Written. None can say otherwise now.", "There. In ink, with the day beside it."], Nothing, to(FollowThrough)),
            mv("muddle", 0.4, Room, &["Three folk say it three ways.", "Nobody can agree what they agreed."], Nothing, to(Amendment)),
        ],
        sources: &["guild_ordinance"] },
    Beat { id: "hm_decision_majority", phase: Decision, criteria: &[Has("rule:majority")], voice: Driver,
        lines: &["Show of hands. Most carries it.", "Count them. The greater part decides."],
        moves: &[
            mv("carried", 1.0, Room, &["Carried, and plainly.", "A good majority."], Nothing, to(FollowThrough)),
            mv("narrow", 0.4, Member, &["Carried by two, and the rest of us sour.", "Two hands made it. I'm not one."], Dissent, to(FollowThrough)),
        ],
        sources: &["athenian_assembly", "polletta_decision_rule"] },
    Beat { id: "hm_decision_consensus", phase: Decision, criteria: &[Has("rule:consensus")], voice: Driver,
        lines: &["Does anyone stand aside? Does anyone block?", "Last chance. Any who cannot bear it?"],
        moves: &[
            mv("consent", 1.0, Room, &["No one blocks.", "Silence, and nods all round."], Nothing, to(FollowThrough)),
            mv("aside", 0.5, Member, &["I'll stand aside, not block.", "Not for me, but I'll not stop you."], Nothing, to(FollowThrough)),
            mv("block", 0.3, Member, &["I block it.", "No. I'll not consent."], Dissent, to(Amendment)),
        ],
        sources: &["polletta_decision_rule", "commune_oath"] },
    Beat { id: "hm_decision_consensus_noted", phase: Decision, criteria: &[Has("rule:consensus"), Has("room:noted")], voice: NoteTaker,
        lines: &["No one blocks. I'll write it as all of us.", "Every hearth consents - I'll mark it so."],
        moves: &[
            mv("write", 1.0, NoteTaker, &["Written as one voice.", "Down it goes: all of us, this day."], Nothing, to(FollowThrough)),
            mv("wait", 0.3, Member, &["Not yet - one more word first.", "Hold the quill. I've a doubt."], Nothing, to(Amendment)),
        ],
        sources: &["polletta_decision_rule", "guild_ordinance"] },
    Beat { id: "hm_decision_drifted", phase: Decision, criteria: &[Has("room:drifted")], voice: Driver,
        lines: &["I say we do it. Who's with me?", "I'm doing it. Come if ye will."],
        moves: &[
            mv("follow", 1.0, Room, &["Most follow. They always do.", "Hands go up for them, not for the plan."], Nothing, to(FollowThrough)),
            mv("thy_say", 0.5, Member, &["That's thy say. Where's ours?", "I'll not follow a vote nobody took."], Dissent, END),
        ],
        sources: &["freeman_structurelessness"] },

    // --- Follow-through (4) ---
    Beat { id: "hm_follow_generic", phase: FollowThrough, criteria: &[], voice: Driver,
        lines: &["Who does what, and by when?", "Settled. Now - whose hands?"],
        moves: &[
            mv("volunteers", 1.0, Member, &["I'll take it to the far lane.", "Give me the tallying."], Nothing, END),
            mv("nobody", 0.4, Room, &["Silence. Folk look at their boots.", "Everyone's busy, of a sudden."], Nothing, END),
        ],
        sources: &["guild_ordinance"] },
    Beat { id: "hm_follow_noted", phase: FollowThrough, criteria: &[Has("room:noted")], voice: NoteTaker,
        lines: &["I'll read back who promised what.", "Before ye go - here's what each of you said ye'd do."],
        moves: &[
            mv("read", 1.0, NoteTaker, &["Each name, each task. Read and marked.", "All on the page. I'll ask after it next time."], Nothing, END),
            mv("groans", 0.4, Room, &["Groans. Nobody likes hearing it twice.", "A few wince at their own names."], Nothing, END),
        ],
        sources: &["guild_ordinance"] },
    Beat { id: "hm_follow_coordinator", phase: FollowThrough, criteria: &[Has("seat:coordinator")], voice: Driver,
        lines: &["I'll walk to every door before we meet again.", "Every house will hear of this by Sunday."],
        moves: &[
            mv("thanks", 1.0, Room, &["Thanks all round.", "Good. Thou'rt the legs of this."], Nothing, END),
            mv("help", 0.6, Member, &["I'll take the north end. Thou take the mill side.", "Two of us go faster."], Nothing, END),
        ],
        sources: &["abolition_petitions"] },
    Beat { id: "hm_follow_first", phase: FollowThrough, criteria: &[Has("history:first")], voice: Driver,
        lines: &["Same night, same hearth, next time?", "We meet again. Let's say when."],
        moves: &[
            mv("aye", 1.0, Room, &["Same night.", "Aye - and bring a neighbor."], Nothing, END),
            mv("bigger", 0.5, Member, &["Somewhere bigger. We'll be more.", "The barn, next time. This won't hold us."], Nothing, END),
        ],
        sources: &["commune_oath"] },
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
