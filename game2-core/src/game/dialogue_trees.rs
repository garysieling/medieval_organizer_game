//! Expanded dialogue library: wide, shallow set of 1000+ contextually grounded
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
    MeetingRites, Probe, Reply, Speaker, Voice,
};
use crate::game::traits::Trait;
use crate::game::traits::Trait::*;

/// Stock phrases that are the village's common property: anyone may say
/// them any day, so the freshness memory (#1369) doesn't count them
pub const STOCK_LINES: &[&'static str] = &[
    "As {2:subj} {2:say}, friend.",
    "True enough - the fields don't gossip.",
    "Aye, there is truth in what {2:subj} {2:say}.",
    "{2:subj} {2:have} spoken the plain heart of the matter.",
];

const SMALL_FADE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["As {2:subj} {2:say}, friend.", "True enough - the fields don't gossip.", "Aye, words won't fill the storehouse.", "We have spoken our piece for today."],
    voice: &[
        // the village's own weather (#1696): under a parish weight the
        // fade reads lean - the axis-specific registers live up in
        // the_hard_times; the tails just stop sounding fat
        Voice { criteria: &[Has("village:hard")], lines: &[
            "Aye - we bear it the same as all the rest.",
            "Words won't fill what's empty - but aye, as {2:subj} {2:say}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair day to {2:obj}, {other} - 'twas good to pass the time.", "Go well, {other}, and mind the mud.", "The saints keep {2:poss} roof whole.", "Fare {2:obj} well until the morrow."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["{2:poss} servant, {addr}. Good day.", "God keep the manor and {2:obj} with it, {addr}."] },
                Voice { criteria: &[Has("other:ideology:merchant")], lines: &["Fair profit to {2:obj}, trader.", "May the road pay {2:poss} way, merchant."] },
                Voice { criteria: &[Has("self:ideology:lord")], lines: &["Walk on, goodman - mind the hedges.", "Aye, good day to {2:obj}."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Fair day, {other} - the ale waits for us both someday.", "Go well, my friend - come by the house when {2:subj} {2:can}."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["God keep our house - and {2:obj} in it, {other}.", "Fair day, kin - give my love to {2:poss} hearth."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Fair road to {2:obj}, stranger.", "Go well, traveler - the saints watch {2:poss} path."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Away then, {other} - the day improves already.", "Fair day, {other} - such as it is, with {2:obj} in it."] },
            ],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I take my leave.", "Enough chatter. Good day.", "My day is waiting.", "I have no leisure to stand about, {other}.", "I must walk on; fair day."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Enough. Good day, {other}.", "I take my leave - gladly."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GOSSIP_MORE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip"],
    calls: &["They do say the reeve counts the sheep twice over.", "The herb-wife was seen out past the abbey wall at dusk.", "Old Wulfric swore he saw an owl perched on the tavern roof.", "A stranger with two laden mules passed by the weir at noon."],
    voice: &[
        // the parish's own weather is the talk under a weight (#1696)
        Voice { criteria: &[Has("village:hard")], lines: &[
            "Every door's got its own trouble to tell of, this stretch.",
            "They do say half the parish's whispering the same worry."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["gossip"],
            respond: None,
            lines: &["Well, that is talk enough for one morning.", "Ha! I'll believe the half of it and no more.", "Tavern tales grow taller with each ale drawn.", "Every tongue in the parish wags before sunset."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // the rare leak (E5-1, #1430): a tense slips past the guard
        // before they catch it - the once-per-arc cap lives with the
        // arc runner (E5-8)
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 0.30, bias: &[],
            act: Act::Slip, concepts: &["gossip", "slip"],
            respond: None,
            lines: &["My brother is... was a carter.", "We kept bees, my - we don't speak of it now.", "The harvest sang that year, when - it doesn't matter."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            respond: None,
            lines: &["Better to tend {2:poss} own hearth than watch the road.", "I have heard that lie three times this week.", "Folk will invent dragons where there's only sheep.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_WORK: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "work"],
    calls: &[
        "How goes it at {other_trade}, {other}?",
        "I hear the work keeps {2:obj} busy at {other_trade}.",
        "Hard at it down at {other_trade}, are we?",
        "Busy hands at {other_trade} this season, I'd wager.",
        "{other} - how goes {other_trade} with {2:obj} this season?",
    ],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["How goes {other_trade}, friend? {2:poss} hands never idle.", "I passed {other_trade} and thought of {2:obj}, {other}."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our kin's trade serves {2:obj} well at {other_trade}, aye?"] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Still grinding at {other_trade}, {other}? Some things never change.", "They say {other_trade} has seen better hands than {2:own}."] },
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["My steward spoke well of {other_trade}. Keep at it.", "The manor has need of good work at {other_trade}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 1.5)],
            act: Act::Answer, concepts: &["work"],
            respond: None,
            lines: &["Well enough - {trade} keeps my hands full and my belly fuller.", "It goes, by the saints' grace. {trade} is kind to a steady hand.", "Fair season for it - {trade} has not wanted for custom.", "Slow mornings, honest evenings - that is {trade} for {2:obj}."],
            voice: &[
                // the two transcripts (E5-9, #1446): backstage the
                // work's grievance comes out; in public the answer's
                // loyal enough for the law's ear
                Voice { criteria: &[Has("pair:channel:backstage")], lines: &[
                    "The lord's field eats my best hours, {other} - {trade} gets what's left.",
                    "Work's work - but the reeve counts my sacks twice over, that I'd tell {2:obj} here.",
                    "It goes - though half of {trade} belongs to the manor before my own table sees it.",
                ] },
                Voice { criteria: &[Has("pair:channel:public")], lines: &[
                    "Honest work for honest pay at {trade} - can't complain, and wouldn't.",
                    "The lord's acres are kind to a steady hand, {addr} - I'll say it anywhere.",
                ] },
                Voice { criteria: &[Has("self:status:hungry")], lines: &["Thin fare at {trade}, truth be told - but it goes."] },
                Voice { criteria: &[Has("self:status:newborn")], lines: &["Busy as it's ever been - {trade} and a new babe both."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["The work goes, kin - mother would be proud of {trade}."] },
            ],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Proud, 1.5)],
            act: Act::Question, concepts: &["work"],
            respond: None,
            lines: &["Well enough, aye. And how does {other_trade} treat {2:obj}?", "Never better. And {2:self} - is {other_trade} kind to {2:obj}?"],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["work"],
            respond: None,
            lines: &["{trade} is my affair, and it fares as it fares.", "Ask the reeve about {trade}, not me. I keep my own count."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["My {trade} needs no commentary from {2:obj}, {other}."] },
            ],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_GOSSIP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip", "smalltalk"],
    calls: &["{2:did} {2:subj} hear who was seen down by the mill?", "They say the miller took an extra scoop of meal again.", "The peddler's cart lost a wheel at the bridge.", "A rider from the castle came through before dawn."],
    voice: &[
        Voice { criteria: &[Has("village:hard")], lines: &[
            "{2:did} {2:subj} hear what's come over the parish this week?",
            "Folk have more than the miller's scoop to whisper of these days."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Remark, concepts: &["gossip"],
            respond: None,
            lines: &["{2:subj} {2:do} not say it! What happened then?", "Aye? And what else did folk see?", "Tell on, neighbor - my ears are open.", "I knew that bridge would give way sooner or later."],
            voice: &[
                // the two transcripts (E5-9, #1446): backstage the
                // talk names power plainly; in public the same ears
                // hear deference - "the lord's fair, as lords go"
                Voice { criteria: &[Has("pair:channel:backstage")], lines: &[
                    "The lord's share grows a little fatter every year, and nobody says it plain. Here, I'll say it.",
                    "The reeve's scales lean a half-pound to the manor's side - we all pay for the tilt.",
                    "They call it God's tithe. It's the manor's appetite, is what it is.",
                    "His lordship counts our sacks twice - once for the barn, once for his own.",
                ] },
                Voice { criteria: &[Has("pair:channel:public")], lines: &[
                    "The lord's a fair master, as lords go - I'll say that anywhere.",
                    "The reeve's hard but the law's the law - keeps us all honest.",
                    "God keep the lord and his men. They keep the peace, they say.",
                    "Aye, the tithe's the tithe. It was ever so, and ever will be.",
                ] },
            ],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            respond: None,
            lines: &["Idle tongues, that. It breeds mischief.", "I'll not repeat marketplace whispers.", "Let the miller and the rider sort their own affairs.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // jokes ride gossip (E5-1, #1430): the everyday set bonds and
        // copes; the ones that name power only come where it's safe to
        // say them - the style/target matrix is E5-6's
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Gregarious, 1.6)],
            act: Act::Joke, concepts: &["gossip", "joke"],
            respond: None,
            lines: &["I told the rain to stop and it came down the harder. So much for commanding the weather.", "My wife says I snore like the mill at full grind. I tell her the mill learned it from me.", "The cat catches nothing and eats like a deacon. We've a prior in the house."],
            voice: &[
                // E5-9 (#1446): backstage is where power's name is safe
                // to laugh at - the channel, not the kin count, is the
                // honest gate the comment below meant
                Voice { criteria: &[Has("pair:channel:backstage")], lines: &[
                    "The tax man weighed my goose twice - once for the count, once for his supper.",
                    "The reeve says the village is his family. Then let him eat our bread - there's little enough.",
                    "The lord's man writes everything down, they say - shame he can't read what he's owed us.",
                ] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["The reeve's horse eats better than my children. Mind, it works harder.", "The priest's sermon ran so long the candles asked for mercy."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["The lord's feast had more ale than harvest - I'll say it to {2:obj} and no further.", "The reeve counts our sheep twice over - practices, I hear, on his own children."] },
                Voice { criteria: &[Has("self:status:drunk")], lines: &["Ale's the only thing that listens to me - and even it makes me pay."] },
            ],
            // a laugh earned a reach (E5-4): the bid asks the answerer
            // to turn toward, away or against
            then: Some(SMALL_BID), end: None,
        },
        // a visible turn-aside (E5-1): the refused reach reads as a
        // wall, not a silence - temperament flavors it in E5-2
        Reply {
            criteria: &[], weight: 0.20, bias: &[(Proud, 1.5)],
            act: Act::Evade, concepts: &["gossip", "evade"],
            respond: None,
            lines: crate::game::conversation::DEFLECTED,
            voice: &[
                // E5-9 (#1446): a public deflection names the public -
                // the transcript's kept where the law can't hear
                Voice { criteria: &[Has("pair:channel:public")], lines: &[
                    "Careful - walls have ears, {addr}.",
                    "That's kitchen talk, not street talk.",
                    "Hush - some things are best said where the law can't hear.",
                ] },
            ],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["A tale for another hour - I must away.", "The sun climbs high; I have chores to finish.", "Walk with God, {other}, I cannot linger.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// A reach for attention mid-tail (E5-4, #1436): they reach rather
/// than remark, and the answer is the turn toward, away or against the
/// design's table weighs - the engaged reply takes it up, the flat
/// remark's the half-turn, the walk-off's the miss, the snap's the
/// refusal
const SMALL_BID: Exchange = Exchange {
    act: Act::Bid,
    concepts: &["bid", "smalltalk"],
    calls: &["Look, the swallows are back.", "Walk with me a moment, {other} - the light's good this hour.", "{2:can} {2:subj} keep a thing quiet, just between us?", "Mind how the mist sits on the lower field. I never tire of it."],
    voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Kind, 1.5), (Gregarious, 1.4)],
            act: Act::Agree, concepts: &["bid"],
            respond: None,
            lines: &["Aye, so they are - I'd not have seen it.", "Walk with {2:obj} a moment, gladly.", "That's a fine sight to be shown, {other}."],
            voice: &[],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.45, bias: &[],
            act: Act::Remark, concepts: &["bid"],
            respond: None,
            lines: &["They come back every year, swallows.", "Hm. There's work waiting yet.", "Aye, the light's the light."],
            voice: &[],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.25, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I've other things to tend, {other}.", "Another day, friend - the day's not waiting."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.15, bias: &[(Hotheaded, 2.0), (Proud, 1.5)],
            act: Act::PushBack, concepts: &["bid"],
            respond: None,
            lines: &["Birds and idle talk - I've a day's work in me yet.", "Keep {2:poss} swallows. I've troubles enough."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_SEASON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "harvest"],
    calls: &["We must thatch the hayrick before the clouds burst.", "The kitchen gardens are dry as a bone this month.", "Rooks are gathering thick in the elm trees.", "The ditch along the south pasture needs clearing."],
    voice: &[
        Voice { criteria: &[Has("village:hard")], lines: &[
            "The fields can wait - there's weightier wants than ditches this week.",
            "Work all the day, and it doesn't touch what ails us."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["harvest"],
            respond: None,
            lines: &["Aye, work never sleeps in this valley.", "A dry spade makes for a hungry winter.", "Every hand will be needed when the grain ripens.", "I'll lend a pitchfork if {2:subj} {2:have} timber to spare."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Lazy, 2.5)],
            act: Act::PushBack, concepts: &["harvest"],
            respond: None,
            lines: &["The hay will wait until tomorrow, surely.", "Let the rain fall; my bones need a day's rest.", "There's always another ditch to dig.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_FARES: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["family", "smalltalk"],
    calls: &["Well enough, thank {2:obj}, {other}.", "We rub along as ever, by the saints' mercy.", "The little ones keep me running from dawn to dusk.", "My knees ache with the damp, but I still walk."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["family"],
            respond: None,
            lines: &["Good to hear it - long may health stay under {2:poss} roof.", "Aye, children grow like weeds in the summer ditch.", "A full hearth is a blessing in hard times.", "May the coming season be kind to {2:poss} cottage."],
            voice: &[],
            then: Some(SMALL_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Give my best to {2:poss} household. Fare well.", "God keep {2:poss} folk in peace, {other}.", "I must see to my own broth. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The kin abroad (#1707): "has your brother returned" earns a real
/// answer - the market town's named because the speaker's folk deal
/// there; lean years keep the cart away longer
const SMALL_KIN: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["family", "smalltalk"],
    calls: &["Not yet - the cart's looked for back from {town} come the week's end.", "He's there still, buying salt - home by holy day, God willing.", "Aye, home since Tuesday, and full of {town} talk."],
    voice: &[
        Voice { criteria: &[Has("village:hard")], lines: &["Not yet - thin pickings in {town} keep him longer than he'd like.", "Still there - the roads are hard this year, and harder coming home."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["family"],
            respond: None,
            lines: &["The saints keep him on the road, {other}.", "A full cart home is worth the waiting.", "May he come home safe and heavier-pursed than he went."],
            voice: &[],
            then: Some(SMALL_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Give my best to {2:poss} household. Fare well.", "God keep {2:poss} folk in peace, {other}.", "I must see to my own broth. Good day."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_VOLLEY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Even so, we will manage as we always do.", "Aye, the year will turn and bring what it brings.", "My bones say the rain will hold off till evening.", "No use fretting over skies we cannot steer."],
    voice: &[
        Voice { criteria: &[Has("village:hard")], lines: &[
            "We bear what's laid on us - what else is there?",
            "Lean years teach lean talk, {other}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["That they will - the store is not full yet.", "Aye, we have lived through harder winters than this.", "True words; worry never baked a single loaf.", "So long as the well stays sweet, we'll thrive."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        // the reveal acts (E5-1, #1430): a burden carried slips out as a
        // question, a reach, or a job asked - mid-talk for now; the
        // door-side placing is E5-3's, the errand's content E5-7's
        Reply {
            criteria: &[Has("self:status:hungry")], weight: 0.55, bias: &[],
            act: Act::Worry, concepts: &["worry", "fear"],
            respond: None,
            lines: &["They say the levy rides again come spring - is it true, {other}?", "Will the frost take the late sowings? The store's so thin.", "Do the tithe-men read letters, {other}? I heard they do."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 0.55, bias: &[],
            act: Act::Worry, concepts: &["worry", "fear"],
            respond: None,
            lines: &["Do the dead sleep, {other}? Or do they wait on us?", "Is the grave the end of it? Folk by the stones say otherwise.", "I dream of the bell some nights - does it mean aught?"],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[Has("self:status:sick")], weight: 0.55, bias: &[],
            act: Act::Worry, concepts: &["worry", "fear"],
            respond: None,
            lines: &["This cough that's going round - does it take the old ones, {other}?", "Will the physic come dear this year? My chest's not right.", "Is the well still sweet? I heard it turned a baby ill."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.45, bias: &[(Kind, 1.6), (Gregarious, 1.5)],
            act: Act::Bid, concepts: &["bid", "smalltalk"],
            respond: None,
            lines: &["Look, the swallows are back.", "Walk with me to the well a moment, {other}.", "It's good to see {2:obj} about, {other}.", "Mind how the light sits on the church roof this hour.", "Can I tell {2:obj} a thing, quiet like?"],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.30, bias: &[],
            act: Act::Errand, concepts: &["errand", "favor"],
            respond: None,
            lines: &["Leave this loaf at the cairn on the hill - and don't say who from.", "Carry a word to the hermit for me, quiet like. There's a loaf in it.", "Mind the stall an hour come market day - a loaf and my thanks for it."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll not keep {2:obj} standing in the wind. Go well.", "On {2:poss} way then, and mine. Fair day.", "Until the church bell rings, neighbor.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The village smalltalk beat - the polite tail a failed audition is
/// left on (#1187), and the generic rejoin of the smalltalk tree
pub const SMALL_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "weather"],
    calls: &["Cold wind off the moor this morning.", "The sky does what it does, {other}."],
    voice: &[
        // the sky overhead (#1388): a wet sky falls rain lines, a fair
        // one drying lines - the day's own weather rides as wx-*
        Voice { criteria: &[Has("self:status:wx-rain")], lines: &["Looks like rain again, doesn't it?", "Gray clouds gathering over the western hill.", "The rain's been at it since matins, hasn't it?"] },
        Voice { criteria: &[Has("self:status:wx-snow")], lines: &["The frost sat late on the furrows this morning.", "Snow again - the lane's white to the gate.", "It came down in the night; drifts against the byre."] },
        Voice { criteria: &[Has("self:status:wx-fog")], lines: &["A gray dawn, but the day's labor won't wait for it.", "The haar's rolled in thick - the steeple's gone clean out of sight.", "Pale and close this morning - can't see past the hedge."] },
        Voice { criteria: &[Has("self:status:wx-fair")], lines: &["Aye, the dew lay heavy on the clover this morning.", "Bright enough for drying, thank God.", "A fine morning - the kind the hay wishes for."] },
        // helplessness as a delivery mode (#1269): the same issue lands
        // resigned while they're cold, and warms rung by rung - these
        // lead so they out-read the plain status voices below
        Voice { criteria: &[Has("self:status:thaw_helpless"), Has("self:status:hungry")], lines: &["The pot's been bare a week, {other}. Nothing for it but thinner.", "An empty belly's just the way of it, isn't it? What's to be done."] },
        Voice { criteria: &[Has("self:status:thaw_helpless"), Has("self:status:sick")], lines: &["This cough'll see me out, I reckon - physic's for folk who can pay.", "Sick or well, the work's the same and the wage's the same."] },
        Voice { criteria: &[Has("self:status:thaw_helpless"), Has("self:status:cold")], lines: &["The hearth's cold and it'll stay cold. That's the size of it."] },
        Voice { criteria: &[Has("self:status:thaw_helpless"), Has("self:status:grieving")], lines: &["Grief's a tenant that pays no rent, {other}. It leaves when it leaves."] },
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Gray skies, gray days - it's the weather of us, {other}, and nothing moves it.", "What's the use of worry? The world's set how it's set.", "Rain or shine, it's all one to the likes of us."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Cold wind - though I've a question still turning over in me, {other}.", "Gray clouds - somebody asked what I wanted, the other day. I'd never thought to ask."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Cold wind - good thing we're not weathering it alone anymore."] },
        Voice { criteria: &[Has("self:status:thaw_acting")], lines: &["Gray skies - the sort we've work afoot on, {other}."] },
        Voice { criteria: &[Has("self:status:hungry")], lines: &["A thin soup and a thinner week, {other}.", "The sky can do what it likes - it's the empty pot that weighs on me."] },
        Voice { criteria: &[Has("self:status:grieving")], lines: &["The mornings are long since the burying, {other}.", "Gray clouds, gray days - I scarce mark the weather now."] },
        // the old wounds they carry, hinted long before they're named (#1243);
        // a status reads as `self:status:<status>`
        Voice { criteria: &[Has("self:status:hook:brother_soldier")], lines: &["Gray sky over the levy road - I watch that road more than I should, {other}."] },
        Voice { criteria: &[Has("self:status:hook:brother_fate")], lines: &["They say the levy came home from the north. Not all of it, {other}."] },
        Voice { criteria: &[Has("self:status:hook:miller_debt")], lines: &["I take the long way round the mill these days, {other}. Mud's better that side."] },
        Voice { criteria: &[Has("self:status:hook:old_faith")], lines: &["The stones on the hill look lonely in this light, don't they?"] },
        Voice { criteria: &[Has("self:status:hook:child_buried")], lines: &["There's a small grave by the yew I keep weeded. Rain or no."] },
        Voice { criteria: &[Has("self:status:hook:burnt_barn")], lines: &["Dry weather. I can't abide the smell of smoke, not since."] },
        Voice { criteria: &[Has("self:status:hook:blamed_neighbor")], lines: &["Hob looks at me across the lane like he knows something, {other}."] },
        Voice { criteria: &[Has("self:status:hook:broken_troth")], lines: &["The road to the next valley - I've not walked it in years."] },
        // the village's own weather (#1696): under a parish weight the
        // polite tail keeps the thread - the sky's the same sky, but it's
        // read through empty stores and fresh graves. These sit behind a
        // speaker's own states, ahead of the plain registers
        Voice { criteria: &[Has("village:famine")], lines: &["The sky can do what it likes - it's the empty pile that weighs on us.", "Fair skies or foul, the market's crumbs either way."] },
        Voice { criteria: &[Has("village:fever")], lines: &["Fair skies, and half the doors barred for the fever. Strange times."] },
        Voice { criteria: &[Has("village:winter")], lines: &["The frost sits late on the furrows again - the season won't let go."] },
        Voice { criteria: &[Has("village:unrest")], lines: &["Bright enough for drying - though folk have more than weather on their minds."] },
        Voice { criteria: &[Has("village:bereft")], lines: &["A gray morning for a gray parish, {other}."] },
        Voice { criteria: &[Has("self:status:cold")], lines: &["The hearth's been out a week - a cold wind feels personal now."] },
        Voice { criteria: &[Has("self:status:drunk")], lines: &["Is it raining, or is that the ale talking? *hic*"] },
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["The sky owes my wheat a fortnight of sun.", "The weather keeps my stewards busy, I'll warrant."] },
        Voice { criteria: &[Has("self:ideology:merchant")], lines: &["Rain means mud, and mud means slow wagons.", "Grey skies never stopped a good price."] },
        // the fine bond out-reads the coarse one (#1372): a sweetheart
        // hears the weather as a sweetheart
        Voice { criteria: &[Has("pair:kind:lover")], lines: &["Cold wind - stand closer, love, and share my cloak.", "Gray sky or no, the world's brighter where {2:subj} {2:stand}."] },
        Voice { criteria: &[Has("pair:kind:family")], lines: &["Rain's coming - mother's joints will know it first.", "Gray clouds - same sky that watched us grow up side by side."] },
        Voice { criteria: &[Has("pair:kind:mentor")], lines: &["Watch the sky awhile - weather-reading is half the craft, {other}.", "Cold mornings teach what warm ones cannot, mark me."] },
        Voice { criteria: &[Has("pair:kind:apprentice")], lines: &["Cold wind - I'll fetch {2:poss} cloak, master, before we start.", "Gray skies - is it true the old masters read rain in the smoke?"] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Gray clouds again - {2:subj} and I have worked under worse, {other}.", "Cold wind this morning - good thing {2:poss} tales are warm."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Rain's coming - mother will want the washing in.", "A gray sky over our lane again, {other}."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["The moor wind blows cold on travelers, stranger.", "Gray clouds - ill weather for the road, I'm told."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["The sky's as sour as {2:poss} company, {other}.", "Cold wind - fitting weather for {2:obj} to be abroad in."] },
    ],
    replies: &[
        // the dedicated pool (#1269): an un-activated speaker's answer
        // to any smalltalk is resignation - it out-draws the ordinary
        // agree for them, and never comes for the warm
        Reply {
            criteria: &[Has("self:status:thaw_helpless")], weight: 2.00, bias: &[],
            act: Act::Agree, concepts: &["weather", "smalltalk"],
            respond: None,
            lines: &["Aye. What's to be done but bear it.", "It'll rain or it won't - nothing we say moves the sky.", "That it will. It always is what it is."],
            voice: &[
                Voice { criteria: &[Has("self:status:hungry")], lines: &["Aye, wet or dry, the pot's still bare."] },
                Voice { criteria: &[Has("self:status:sick")], lines: &["Aye. Rain's the least of what ails me."] },
            ],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            respond: None,
            lines: &["It will be a wet week, mark me.", "The thatch won't thank it if it pours.", "Aye, and the mud will take {2:poss} boots right off.", "Good for the barley, but bad for the road."],
            voice: &[
                Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Aye - though maybe there's more to be done than talk of it."] },
                Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Aye - and we've more than weather-talk between us now."] },
                Voice { criteria: &[Has("self:status:thaw_acting")], lines: &["Aye - the mud's where the work is, mind."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Aye - we'll weather it together, like always.", "Wet week or dry, good to stand a while with {2:obj}."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Aye - our roof's held worse, thanks to our father's hands.", "The mud'll take our boots, same as when we were little."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, rain - and I'll not lend {2:obj} my cloak for it.", "The thatch won't thank it, nor will I thank {2:obj} for saying so."] },
            ],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-snow")], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            respond: None,
            lines: &["Aye, deep snow keeps us close to the hearth.", "The drifts will have us digging till Candlemas.", "Aye - good snow keeps the root cellar cold."],
            voice: &[],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-fog")], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            respond: None,
            lines: &["Aye, thick as wool out there.", "Can't see the ford for it, mark me.", "Aye - a morning for staying by the fire."],
            voice: &[],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-fair")], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            respond: None,
            lines: &["Aye, a day made for the field.", "Good drying weather, praise be.", "Aye - a sky worth working under."],
            voice: &[],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 0.40, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["weather"],
            respond: None,
            lines: &["Rain? It is the driest week in memory.", "{2:subj} {2:see} a tempest in every puff of smoke.", "My joints say dry weather, whatever the sky says.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Rain? Trust {2:obj} to call the sky wrong, {other}.", "{2:subj} {2:would} argue with the weather itself - and lose."] },
            ],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Gregarious, 2.0)],
            act: Act::Question, concepts: &["family", "smalltalk"],
            respond: None,
            lines: &["And how fares {2:poss} folk in this cold snap?", "Tell me true, {other} - how goes it with {2:obj}?", "Do {2:poss} folk keep well through the snap?"],
            voice: &[],
            then: Some(SMALL_FARES), end: None,
        },
        // the kin abroad (#1707): only asked of a house whose folk run
        // the roads, and the town's named - the answer answers it
        Reply {
            criteria: &[Has("other:status:kin_abroad")], weight: 0.80, bias: &[(Gregarious, 2.0)],
            act: Act::Question, concepts: &["family", "smalltalk"],
            respond: None,
            lines: &["Has {2:poss} brother returned from {town}?", "Any word of {2:poss} kin at {town}, {other}?", "Does the road bring {2:poss} brother back from {town} soon?"],
            voice: &[],
            then: Some(SMALL_KIN), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Ambitious, 1.5)],
            act: Act::Question, concepts: &["work", "smalltalk"],
            respond: None,
            lines: &["Enough sky-talk - how goes the work these days?", "Weather aside, {other} - how does the season treat {2:obj}?", "And the day's labor - does it treat {2:obj} kindly?"],
            voice: &[],
            then: Some(SMALL_WORK), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Well, I had best be about my chores.", "I should get on before the bell strikes.", "Fair day to {2:obj}, {other}.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &["Good morrow to {2:obj}, {other}.", "Well met, neighbor {other}.", "A fair day to {2:obj} and {2:own}, {other}.", "God's peace on this path, {other}.", "Hail, {other} - traveling far today?"],
    voice: &[
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["Morrow, {other} - {2:poss} strip looks well enough this year.", "Aye, neighbor - the morning serves us all."] },
        Voice { criteria: &[Has("self:ideology:merchant")], lines: &["Morrow, {other} - brisk trade to {2:obj}.", "Fair day! The shelves are full and the till's been kind."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Well met, {other} - the day brightens for {2:poss} company.", "Ah, {other}! I hoped the road would bring {2:obj}."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - the house still smells of {2:poss} baking.", "{other}! Good blood on the road today."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, traveler.", "A fair day, stranger - if friend {2:subj} be."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other}.", "Fair morning to {2:obj}, {other}."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["{other}. {2:subj} {2:be} out early.", "Morrow, {other}. Mind where {2:subj} {2:walk}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            respond: None,
            lines: &["And to {2:obj}, {other}.", "Well met indeed, friend.", "The saints keep {2:obj}, {other}.", "Good day and full granaries to {2:obj}."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["And to {2:obj}, {addr}.", "God keep {2:obj} and the manor, {addr}."] },
                Voice { criteria: &[Has("other:ideology:merchant")], lines: &["And fair coin to {2:obj}, trader.", "Well met, merchant - the road treats {2:obj} kindly."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["And a brighter one to {2:obj}, {other}.", "Well met, my friend - always."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["To {2:obj} and all our house, {other}.", "Morrow, kin - the latch is never barred to {2:obj}."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["And to {2:obj}, stranger.", "Fair met, though I know {2:obj} not well."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["And to {2:obj}.", "Good day to {2:obj}."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, well met - for some.", "I see {2:obj}, {other}."] },
            ],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["Morrow.", "Aye. What is it, {other}?", "I am here, as {2:subj} {2:see}.", "Grace and good morrow to {2:obj}, {other}.", "Time will tell what comes of this."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Speak {2:poss} piece and be done.", "Out with it - I've no patience for {2:obj} today."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Aye. What {2:do} {2:subj} want of me?", "I am here, stranger - as {2:subj} {2:see}."] },
            ],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[Lacks("pair:relation:friend"), Lacks("pair:relation:kin")], weight: 0.25, bias: &[(Proud, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I have no time for idle talk today.", "Work is waiting at the barn, {other}.", "I walk on business, neighbor.", "I have no leisure to stand about, {other}.", "I must walk on; fair day."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I've no time for {2:obj} today.", "Walk on, {other}."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["I walk on business, stranger.", "I've no leisure for a stranger's talk today."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

/// What a settled soul's name earns on the road (#950): the virtues are
/// never a score anyone reads, but a marked life gets greeted by what
/// the village knows of it. These fire late in the cascade - a trade, a
/// circumstance or a bond always speaks first - and pass the talk back
/// to the common smalltalk once the name's been said
const VIRTUE_GOOD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &["Well met, {other} - {2:poss} table's never bare to a neighbor, they say.", "God's peace, {other}. Half the village owes {2:obj} a meal, by the telling.", "Ah, {other} - the soul that asks after folk's pots before their own.", "Fair day, {other}. {2:poss} good name walks ahead of {2:obj}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            respond: None,
            lines: &["{2:subj} {2:be} kind to say so, {other}.", "A neighbor's a neighbor - the saints ask no more.", "Well met, friend. The talk's kinder than the truth.", "And a fair day to {2:obj} for saying it."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.45, bias: &[(Proud, 2.5)],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["Aye, folk say what folk say.", "Kind words cost nothing, {other}.", "The road's full of talk; I keep my own ledger.", "Well enough."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
    ],
};

const VIRTUE_DEVOUT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["God's peace, {other} - the bell finds {2:obj} on {2:poss} knees more oft than not, they say.", "Well met, {other}. {2:poss} faith's a lamp the whole parish sees.", "Fair day, {other} - fresh from the service, like as not.", "The saints keep {2:obj}, {other}, as {2:subj} {2:keep} them."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting", "concern:faith"],
            respond: None,
            lines: &["And His peace on {2:obj}, {other}.", "The day's not whole without the words, friend.", "Kind of {2:obj} to mark it. Well met.", "The saints watch us all alike - I only bow the sooner."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.5)],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["A soul's business is its own, {other}.", "Aye. Well, the day's the day's.", "The church keeps my hours; I keep my own counsel.", "Well met."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
    ],
};

const VIRTUE_COLD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &["{other}. {2:subj} {2:be} about, then.", "Well met, {other} - if well it be.", "Morrow, {other}. {2:poss} gate was shut to the bailiff's widow last week, I hear.", "Ah. {other}. Keep {2:poss} weather to {2:self}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["Is there aught {2:subj} {2:need}, {other}?", "The day is what it is.", "Aye, well met then.", "Speak {2:poss} piece and have done, friend."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.55, bias: &[(Hotheaded, 2.5), (Proud, 2.0)],
            act: Act::PushBack, concepts: &["greeting"],
            respond: None,
            lines: &["My gate's my own affair, {other}.", "I owe the whole village nothing, mark me.", "Cold? I've a fire like any man's.", "Wag {2:poss} tongue elsewhere, {other}."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
    ],
};

const SICK_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness", "aid"],
    calls: &["{2:subj} {2:be} kind, {other}. The saints remember a neighborly hand.", "My thanks. A warm bowl does more than herbs.", "Bless {2:obj} for thinking of our small house.", "Rest by the hearth and let the fever break in peace."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Think nothing of it. We look after our own.", "Rest quiet by the coals. I'll check on {2:obj} tomorrow.", "The village stands together or it falls. Fare well.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_STUNG: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness"],
    calls: &["It is not the black pestilence, keep {2:poss} wits about {2:obj}.", "As {2:subj} {2:will}, then - step wide if {2:subj} {2:must}.", "A little cough won't kill an honest worker.", "Rest by the hearth and let the fever break in peace."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Forgive me - fear makes cowards of us all.", "{2:subj} {2:have} right of it. I spoke too quick.", "No offence meant, {other}. We are all on edge.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            respond: None,
            lines: &["I'll not catch my death for {2:poss} pride.", "Breathe elsewhere then, knave.", "Stay in {2:poss} own yard if {2:subj} {2:be} frightened."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SICK_ANSWER: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["Aye, since the heavy frost. The fever burns at dusk and chills at dawn.", "They say the herb-wife's willow draught brings the sweat out.", "My lungs feel like wet wool on a rainy morning.", "Half the north lane is coughing into their sleeves."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["The saints speed {2:poss} mending, {other}.", "Drink hot boiled water with a scrape of honey.", "May the affliction pass {2:obj} lightly.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            respond: None,
            lines: &["I will send broth round when the kettle boils.", "I'll ask Father Anselm to light a candle for {2:poss} recovery.", "Sit by the hearth; I'll fetch firewood if {2:subj} {2:have} none.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be away before the air catches me too.", "Mind {2:poss} cup and bowl. Good day.", "I have no leisure to stand about, {other}.", "I must walk on; fair day."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_EXCUSE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["I'd rest in the straw if the chores would do themselves.", "There is no one else to fetch water from the river.", "The calves won't feed themselves, fever or no.", "The lord's rent waits for no man's cough."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["I'd rest, {other}, but the chores won't do themselves - though I know {2:subj} {2:would} lend a hand.", "The calves won't feed themselves, friend - fever or no."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["The calves won't feed themselves - though our kin have offered, bless them.", "There's none else to fetch the water - though I know our house would send it."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["There is no one else to fetch water from the river, stranger.", "The rent waits for no man's cough."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["The calves won't feed themselves - and I'd not ask {2:obj} for help, {other}.", "I'd rest, but who'd do it? Not {2:obj}, I reckon."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["We labor or we go hungry, that's the hard truth of it.", "Aye, the dirt cares nothing for an aching back.", "A poor man's bed is a luxury he can rarely afford.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Aye, and I'd carry half {2:poss} load if {2:subj} {2:would} let me.", "True enough - but a friend's help makes the toil shorter."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Aye, but our house will see {2:obj} through it.", "Blood doesn't watch blood labor alone for long."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, work or starve - {2:subj} {2:know} it as well as I.", "The dirt cares nothing - for either of us."] },
            ],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            respond: None,
            lines: &["Let me carry {2:poss} pail to the well, {other}.", "Go inside; I'll see to {2:poss} livestock this evening.", "I have barley meal to spare for {2:poss} kettle.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Say the word and the chore's done, {other} - {2:subj} {2:know} it.", "I'll see to {2:poss} beasts this evening; rest easy."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our house has barley to spare - blood takes care of blood.", "Go in, {other}; {2:poss} cousins will see to the yard."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I'll carry {2:poss} pail - once - and I'd not have it spoke of.", "{2:poss} beasts I'll see to; {2:obj}, I'll suffer at a distance."] },
            ],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Rest when {2:subj} {2:can}, {other}. Fair day.", "God grant {2:obj} strength for the day's toil.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Rest well, {other} - I'll look in on {2:obj} at dusk."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Rest when {2:subj} {2:can} - or don't; it's no care of mine.", "God grant {2:obj} strength - and keep {2:obj} far from me."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["*cough* Good morrow, {other}.", "Forgive my rough voice - this chest cold won't quit.", "Morrow, {other}. Don't stand too close in the breeze.", "A heavy morning, neighbor. My limbs are like lead."],
    voice: &[
        // the rung it's coughed from (#1269): sickness is fate to the
        // cold, a thing that could be looked to once they're warm
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["*cough* Morrow. It'll run its course or it won't - nothing for it either way.", "Aye, the cough again. Sick's what poor folk are, {other}."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["*cough* Morrow, {other} - a foul chest, though folk have begun asking what I'd want done."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["*cough* Morrow, {other} - the physic dear, but we've people on it now."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["*cough* Ah, {other} - forgive the croak. Stand off, mind; I'd not share this with a friend.", "Morrow, {other}. A wretched cold - but warmer for seeing {2:obj}."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["*cough* Morrow, {other}. Tell our mother I still stand upright.", "{other} - aye, it's me, coughing like a broken bellows. Keep out of the draft."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["*cough* Morrow, traveler. Best keep to the far side of the path.", "A sick man's greeting, stranger - from a distance."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["*cough* Fair morning, {other}. Mind the breeze between us.", "Morrow, {other} - a foul chest this week."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["*cough* {other}. Keep to {2:poss} side - I'd not give {2:obj} even my sickness.", "Morrow, {other}. Step wide; even my fever wants no part of {2:obj}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Poor soul - get {2:obj} back into the warm cottage.", "{2:subj} {2:should} be under woolen blankets, not out here.", "{2:subj} {2:look} pale as a winter moon, {other}.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["Poor soul - get {2:obj} back to the manor's warm hearth, {addr}.", "{2:subj} {2:should} be under woolen blankets, {addr} - {2:poss} folk will fret."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Get {2:obj} back under the blankets, {other} - I'll not watch {2:obj} worsen.", "Poor soul. Tell me what {2:poss} house needs and it's done."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["{2:subj} {2:be} blood - I'll not watch {2:obj} shake. Home with {2:obj}.", "Back to bed, {other}. I'll fetch the broth myself."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["{2:subj} {2:should} not be abroad in this state, stranger.", "Keep {2:poss} strength - and {2:poss} distance."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["{2:subj} {2:look} pale, {other}. Best be indoors.", "A rough fever, by the look of {2:obj}."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, keep {2:poss} distance - but I'll not pretend a pity I don't feel.", "Even I'd not wish this on {2:obj}, {other}. Perhaps."] },
            ],
            then: Some(SICK_EXCUSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Is it the ague that took the miller's lad?", "How long has this fever had hold of {2:obj}?", "{2:have} {2:subj} seen the herb-wife for a draught?"],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["How long has the fever held {2:obj}, {addr}?", "{2:have} {2:subj} seen the herb-wife for a draught, {addr}?"] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["How long has it held {2:obj}, {other}? Speak plain with me.", "Is it the miller's lad's fever? {2:subj} {2:know} I'll help."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["What does the herb-wife say? Our house will hear of it."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["How long since it took hold?", "Is it catching, this fever of {2:own}?"] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["How long {2:have} {2:subj} dragged that about, {other}?", "Is it the fever, or merely {2:poss} face?"] },
            ],
            then: Some(SICK_ANSWER), end: None,
        },
        Reply {
            criteria: &[Lacks("self:status:sick")], weight: 0.50, bias: &[(Proud, 1.5), (Stingy, 1.5)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Keep {2:poss} distance, neighbor. I have no wish to catch it.", "I will hear {2:obj} from across the ditch.", "Cover {2:poss} mouth when {2:subj} {2:speak} in the wind.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["Keep {2:poss} distance, {addr} - I have no wish to catch it.", "I'll hear {2:obj} from across the ditch, begging {addr}'s pardon."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Keep {2:poss} plague and {2:poss} distance both.", "Breathe elsewhere, {other} - as ever."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Keep {2:poss} distance, stranger - I want no new sickness.", "I'll hear {2:obj} from across the ditch, and no nearer."] },
            ],
            then: Some(SICK_STUNG), end: None,
        },
    ],
};

/// The visitor's own close (E5-1): the sick one's answers deepen
/// before it ends - aid content on the way out
const VISIT_CLOSE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid", "concern:sickness"],
    calls: &["I'll look in again on the morrow, rest now.", "The parish keeps its own, {other} - sleep easy."],
    voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Rest well, {other}.", "Then I'll see {2:obj} at first light."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_PROMISE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid"],
    calls: &["It is done - look for it by sundown.", "I'll have my girl bring the loaf before dark.", "Rest easy; the parish will not let {2:obj} freeze.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[(Kind, 1.4)],
            act: Act::Agree, concepts: &["aid"],
            respond: None,
            lines: &["Aye, that would sit kindly, {other}.", "The house thanks {2:obj} for it."],
            voice: &[],
            then: Some(VISIT_CLOSE), end: None,
        },
        // the sickbed worry (E5-1): the fear asks out loud what the
        // fever's been whispering
        Reply {
            criteria: &[Has("self:status:sick")], weight: 0.45, bias: &[],
            act: Act::Worry, concepts: &["worry", "concern:sickness"],
            respond: None,
            lines: &["This cough that's going round - does it take the old ones, {other}?", "Will the physic come dear this year? My chest's not right.", "Is the well still sweet? I heard it turned a baby ill."],
            voice: &[],
            then: Some(VISIT_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:subj} {2:be} a true neighbor, {other}.", "The blessing of Heaven on {2:poss} roof.", "I'll repay the kindness when my legs mend.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_CARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:sickness", "aid"],
    calls: &["Is there aught {2:subj} {2:need} brought to the cottage?", "Can I fetch water or chop kindling for {2:poss} hearth?", "{2:do} {2:subj} have grain left in the bin?", "Does the fever ease when the sun warms the cottage?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["aid", "concern:food"],
            respond: None,
            lines: &["A little bread or salt, if {2:subj} {2:can} spare it.", "Some dry firewood would keep the chill out of our bones.", "A jug of fresh well water would be a godsend.", "The sweat broke at dawn and the fever feels a little lighter."],
            voice: &[],
            then: Some(VISIT_PROMISE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Pious, 1.5)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Just a prayer at the church altar, {other}.", "We have enough for today. Pray for my children.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["How {2:do} {2:subj} fare today, {other}?", "I heard the sickness took {2:poss} household - how is it?", "Peace to this cottage, {other}. How goes the fever?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Poorly, if {2:subj} {2:will} have the plain truth.", "Weak as a lamb, but still drawing breath.", "The night was hard, but the morning brings a little ease.", "The sweat broke at dawn and the fever feels a little lighter."],
            voice: &[],
            then: Some(VISIT_CARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            respond: None,
            lines: &["It will pass. We do not need fussing over.", "I have weathered worse than a common cough.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(VISIT_CARE), end: None,
        },
    ],
};

const GRIEF_CLOSE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["Thank {2:obj} for listening, {other} - it lightens the heart.", "It does a body good to speak their name aloud.", "The house feels less empty when a neighbor sits a spell.", "The sorrow will ease as the seasons turn and heal."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[(Kind, 1.4)],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["We'll see it through, {other} - the village does.", "Gently does it, friend. Day by day."],
            voice: &[],
            then: None, end: None,
        },
        // the consoler's own reach (E5-1): the bid asks them to stay a
        // little longer than the words do
        Reply {
            criteria: &[], weight: 0.35, bias: &[(Kind, 1.6)],
            act: Act::Bid, concepts: &["bid", "concern:grief"],
            respond: None,
            lines: &["Sit a while longer - the kettle's still warm.", "Walk with me to the well a moment, {other}."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Any hour, {other}. My door is always unlatched.", "We do not forget those who walked this lane with us.", "May holy angels watch over {2:poss} sleep tonight.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_ASK: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:grief", "aid"],
    calls: &["A loaf of bread would not go amiss, {other}.", "A few split logs, if {2:subj} {2:have} any seasoned oak.", "Someone to watch the sheep for an afternoon."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["aid", "farewell"],
            respond: None,
            lines: &["It is done. Look for my boy with the basket by dusk.", "I will bring it myself before sunset, friend.", "Leave the sheep to me; mourn in peace.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I will see what can be spared from our cellar.", "Times are tight, but I'll see what I can find.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_OPENS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["I keep setting {his:dead} wooden spoon at table.", "{he:dead} loved the smell of the damp earth after rain.", "Every corner of the cottage remembers {his:dead} footsteps.", "Forty winters together, and now only the silence."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["Tell me of {him:dead} - good memories are a shield against darkness.", "{he:dead} was loved by all who knew {him:dead} in the village.", "{his:dead} toil shaped this parish; {his:dead} memory will outlive us.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(GRIEF_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll not weary {2:obj} with more words. God keep {2:obj}.", "Rest {2:poss} sorrow in prayer, {other}. Fare well.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["{other}. Forgive me - the house is very still since {he:dead} passed.", "Morrow, {other}. The bell tolled, and my hearth is dark.", "Well met, friend. The days feel longer than they used to.", "A peaceful day to {2:obj}, {other}."],
    voice: &[
        // grief's the one wound the rung reads differently (#1269):
        // cold it's endured, warm it's carried
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Morrow. The house is still, and that's just how it is now - no help for it.", "{other}. The bell tolled, and we go on. There's nothing else to do."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["{other}. The house is still - but somebody asked me what I wanted, and I've been thinking on them since."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["{other}. The house is still - but the cause's folk have sat with me in it."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["{other}. Forgive me - the house is still since they passed, and {2:poss} face is a kindness.", "Well met, dear friend. The days feel longer now."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["{other}. Blood knows blood - the house is empty without them.", "Morrow, {other}. Our family's table has an empty bench."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Forgive a grieving house its silence.", "A heavy morning - we've buried one of ours."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other}. The bell tolled for my house this week.", "A gray day, {other}. The hearth is dark."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["{other}. Even {2:subj} {2:will} know the house has gone quiet.", "Morrow, {other}. Death levels more than our quarrel."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.40, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["They were an honest soul. The whole village mourns them.", "The saints receive their spirit in eternal peace.", "My heart aches for {2:obj}, neighbor.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["They were the best of us, {other} - I'll miss them beside {2:obj}.", "My heart breaks with {2:own}, {other}. Always."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our blood mourns together, {other}. The saints keep them.", "They were ours - and we carry them now."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["I did not know them, stranger, but I grieve {2:poss} grief.", "A loss is a loss - the saints keep their soul."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["An honest soul gone. The village mourns with {2:obj}.", "The saints receive them kindly."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["We'd our quarrels, {other} - but the grave settles them.", "I'll not pretend tears, {other} - but I'll not mock the dead."] },
            ],
            then: Some(GRIEF_OPENS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Kind, 1.3)],
            act: Act::Question, concepts: &["concern:grief", "aid"],
            respond: None,
            lines: &["Is there any chore I can take off {2:poss} shoulders?", "{2:have} {2:subj} eaten a warm bite today, {other}?", "Shall I sit with {2:obj} a while, {other}?"],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["What can I take off {2:poss} hands, {other}? Name it.", "Shall I sit with {2:obj}? {2:subj} needn't be alone."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["What does the house need? Say it and it's done."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Is there aught a stranger can do?", "{2:do} {2:subj} have folk to help {2:obj}?"] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I'll not offer charity {2:subj} {2:would} refuse - but is {2:poss} house seen to?", "Is anyone feeding {2:obj}? Even I would not see {2:obj} starve."] },
            ],
            then: Some(GRIEF_ASK), end: None,
        },
    ],
};

/// The condoler's own close (E5-1): the rejoin's answers deepen
/// before it ends - it keeps grief content on the way out
const CONDOLE_CLOSE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief", "aid"],
    calls: &["I'll sit with {2:obj} a while yet, if it helps.", "The house won't grieve alone, {other}.", "Send for me at any hour - the latch lifts easy."],
    voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:poss} coming did the house good, {other}.", "Go gently, {other}. The door's always open to {2:obj}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CONDOLE_OFFER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief", "aid"],
    calls: &["If {2:subj} {2:need} aught at all, say but the word.", "We have flour and eggs; let us share with {2:poss} house.", "My sons can plough {2:poss} furrow until {2:subj} {2:be} ready.", "The sorrow will ease as the seasons turn and heal."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[(Kind, 1.4)],
            act: Act::Agree, concepts: &["concern:grief", "aid"],
            respond: None,
            lines: &["Sit then, {other} - the fire's low but there's room.", "Kindly said, {other}. Stay the hour."],
            voice: &[],
            then: Some(CONDOLE_CLOSE), end: None,
        },
        // a door-side fear (E5-1, #1430): grief carries its questions -
        // the deep placements come with E5-3
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 0.50, bias: &[],
            act: Act::Worry, concepts: &["worry", "concern:grief"],
            respond: None,
            lines: &["Do the dead sleep, {other}? Or do they wait on us?", "Is the grave the end of it? Folk by the stones say otherwise.", "I dream of the bell some nights - does it mean aught?"],
            voice: &[],
            then: Some(CONDOLE_CLOSE), end: None,
        },
        // the rare leak (E5-1): consolation presses past the guard -
        // the once-per-arc cap lives with the arc runner (E5-8)
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 0.35, bias: &[],
            act: Act::Slip, concepts: &["slip", "concern:grief"],
            respond: None,
            lines: &["My brother is... was a carter.", "We kept bees, my - we don't speak of it now.", "The harvest sang that year, when - it doesn't matter."],
            voice: &[],
            then: Some(CONDOLE_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:subj} {2:be} a true soul, {other}. I will remember it.", "{2:poss} kindness brings tears to an old eye. God bless {2:obj}.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["We will manage on our own, but thank {2:obj} kindly.", "I will not burden {2:poss} house with my sorrow.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CONDOLE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["I was deeply grieved to hear of {2:poss} loss, {other}.", "The village stands beside {2:obj} in this dark trial.", "God's mercy comfort {2:obj} in {2:poss} sorrow, neighbor.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["{2:subj} {2:be} good to come by, {other}. It means much.", "Thank {2:obj} for {2:poss} prayers and thoughts.", "The road is lonely now, but kind words help.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(CONDOLE_OFFER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:grief"],
            respond: None,
            lines: &["We endure what Heaven sends us.", "Tears do not bring back the dead.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(CONDOLE_OFFER), end: None,
        },
    ],
};

const CLERIC_COMFORT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["The saints watch over {2:poss} cottage. I will come by with holy water and draughts.", "Rest by the hearth, child - I'll light a candle and pray for {2:poss} household.", "Cast {2:poss} burdens before the altar; the Lord does not forget the meek.", "The church altar is always open to the weary soul."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thank {2:obj}, father - {2:poss} blessing brings comfort.", "The Lord keep {2:obj}, father, and preserve {2:poss} voice.", "I will be at the church when the bell sounds.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PRAYER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith", "concern:grief", "concern:sickness"],
    calls: &["The Lord comfort {2:obj}, child. Tell me of {2:poss} affliction.", "Speak, child - my prayers and candles are with {2:obj}.", "Open {2:poss} heart; what sorrow weighs upon {2:obj}?", "How {2:do} {2:subj} find the road today, {other}?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:sick")], weight: 1.0, bias: &[],
            act: Act::Answer, concepts: &["concern:sickness"],
            respond: None,
            lines: &[
                "It's the little one mostly - she burns with fever.",
                "This cough won't leave me, father, and the store is thin.",
            ],
            voice: &[],
            then: Some(CLERIC_COMFORT), end: None,
        },
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 1.0, bias: &[],
            act: Act::Answer, concepts: &["concern:grief"],
            respond: None,
            lines: &[
                "He went in the night, father, and I never said farewell.",
                "The house is so quiet, father. I can't bear the quiet.",
            ],
            voice: &[],
            then: Some(CLERIC_COMFORT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PARISH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["How fares {2:poss} house in these lean weeks, child?", "{2:do} {2:subj} find peace in {2:poss} daily labor, child?", "Remember that honest work in the field is a holy prayer.", "The church altar is always open to the weary soul."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:faith"],
            respond: None,
            lines: &["We keep as best we can, father. The toil is hard.", "The season has been lean, father, but faith sustains us.", "I try to walk uprightly before God and my neighbors.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I will see {2:obj} at the Sunday bell, father.", "Keep our fields in {2:poss} prayers, father. Fair day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Blessings on {2:obj}, {other}.", "The peace of the saints be with {2:obj}, child.", "God's grace light upon {2:poss} steps, {other}.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Pious, 2.0)],
            act: Act::Greet, concepts: &["concern:faith"],
            respond: None,
            lines: &["And on {2:obj}, father.", "Blessings on {2:poss} holy house as well.", "Peace be with {2:obj}, good priest.", "God bless {2:poss} steps this day, {other}.", "A blessed morning to {2:obj}, father."],
            voice: &[],
            then: Some(CLERIC_PARISH), end: None,
        },
        Reply {
            criteria: &[Has("self:status:sick")], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:sickness"],
            respond: None,
            lines: &["A prayer for me, father - this fever won't quit.", "Father, hear my plea; the sickness grips my throat.", "{2:will} {2:subj} hear my confession ere the fever takes me?"],
            voice: &[],
            then: Some(CLERIC_PRAYER), end: None,
        },
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:grief"],
            respond: None,
            lines: &["Pray for them, father - they were taken so sudden.", "A blessing for my sorrowing cottage, father.", "Father, where do the dead wait for us?"],
            voice: &[],
            then: Some(CLERIC_PRAYER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 1.5)],
            act: Act::Remark, concepts: &["concern:faith"],
            respond: None,
            lines: &["Morrow, father.", "A fair day to {2:obj}, priest.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[],
            then: Some(CLERIC_PARISH), end: None,
        },
    ],
};

const FATHER_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["Thank {2:obj}, father - that lifts a great stone from my mind.", "{2:poss} counsel is wise, father. We will trust in Providence.", "The church has been our refuge through many a cold year.", "{2:poss} blessing gives me heart to face the coming week, father."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[],
            act: Act::Agree, concepts: &["concern:faith"],
            respond: None,
            lines: &["Blessings on {2:poss} house, child.", "The parish is blessed in souls like {2:own}."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // a job asked of the priest (E5-1): the errand's ghost-tied
        // shapes come with E5-7
        Reply {
            criteria: &[], weight: 0.35, bias: &[],
            act: Act::Errand, concepts: &["errand", "concern:faith"],
            respond: None,
            lines: &["Carry a word to the hermit for me, quiet like. There's a loaf in it.", "Would the parish take a letter to the bishop's man? I'll see the wax paid for."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Go in peace, {other}, and love {2:poss} neighbors.", "The Lord preserve {2:poss} hearth and {2:poss} children.", "Walk in the light, child. Fare well.", "My prayers go with {2:obj}, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_ASKS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["Must the tithe stand full when the granary is half empty, father?", "Can the church spare a peck of grain for the hungry children, father?", "Will the parish altar provide candles for our sick neighbor, father?", "How {2:do} {2:subj} find the road today, {other}?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:faith"],
            respond: None,
            lines: &["The church will share its grain, child - none shall starve at the door.", "The tithe stands, but I will speak for {2:poss} house at the manor court.", "Holy candles and broth shall be brought to their bedside this very day.", "We do what we can with the strength Heaven gives."],
            voice: &[],
            then: Some(FATHER_THANKS), end: None,
        },
        // the confessional's wall (E5-1, #1430): some asks a father
        // turns aside - the tier-driven evasions come with E5-2
        Reply {
            criteria: &[], weight: 0.30, bias: &[(Proud, 1.5)],
            act: Act::Evade, concepts: &["evade", "concern:faith"],
            respond: None,
            lines: crate::game::conversation::DEFLECTED,
            voice: &[],
            then: Some(FATHER_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Good morrow, father.", "A word with {2:obj}, father, if {2:subj} {2:can} spare a moment.", "Greetings, holy father - I seek {2:poss} pastoral ear.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Kind, 1.3)],
            act: Act::Greet, concepts: &["concern:faith"],
            respond: None,
            lines: &["The saints keep {2:obj}, {other}. What weighs on {2:obj}?", "Always, child. Speak freely in the sight of God.", "Peace to {2:obj}, {other}. How may I help {2:poss} soul?", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(FATHER_ASKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const RIVAL_END: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["We are done here.", "I have looked upon {2:poss} face long enough.", "Take {2:poss} shadow off my land.", "I have no words to waste on a crooked knave."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Aye. Done.", "Until we meet at the boundary ditch.", "Gladly. The sight of {2:obj} sours my stomach.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const RIVAL_BITE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Make me.", "Come closer and say that to my beard.", "{2:subj} {2:be} all mouth and no marrow.", "I have no words to waste on a crooked knave."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[],
            act: Act::Remark, concepts: &["feud"],
            respond: None,
            lines: &["Not worth the breath or the bruise.", "I'll settle this before the magistrate, not in the dirt.", "This quarrel's older than {2:poss} fence, and twice as rotten."],
            voice: &[],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Hotheaded, 4.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Cur! {2:subj} {2:will} regret those words.", "Keep {2:poss} fists to {2:self} before I break them.", "Get out of my sight, {other}.", "Take {2:poss} shadow off my path."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const RIVAL_SNIPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Heard about {2:poss} boundary fence. Shoddy timber, that.", "{2:poss} lot still owes mine three groats, whatever lies {2:subj} {2:tell}.", "Keep {2:poss} lean cows out of our clover pasture.", "I saw where {2:poss} cart rutted the lane by my ditch."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["My fence stands straighter than {2:poss} crooked word.", "We paid that debt at Michaelmas, as the reeve well knows.", "{2:poss} pasture is mostly thistles anyway."],
            voice: &[],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Say that again, if {2:subj} {2:dare}.", "{2:subj} {2:have} a treacherous tongue in {2:poss} skull.", "The whole parish knows the measure of {2:poss} word."],
            voice: &[],
            then: Some(RIVAL_BITE), end: None,
        },
        // the cold rival turns it aside rather than bite (E5-1)
        Reply {
            criteria: &[], weight: 0.25, bias: &[(Proud, 1.5)],
            act: Act::Evade, concepts: &["evade", "feud"],
            respond: None,
            lines: crate::game::conversation::DEFLECTED,
            voice: &[],
            then: Some(RIVAL_END), end: None,
        },
    ],
};

const RIVAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "feud"],
    calls: &["{2:subj}.", "Look what the crows dropped on the highway.", "{other}. Still walking upright, I see.", "What business brings {2:poss} ugly mug across my path?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["feud"],
            respond: None,
            lines: &["{other}.", "I walk where I please in this parish.", "Save {2:poss} bile for someone who fears it."],
            voice: &[],
            then: Some(RIVAL_SNIPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Still breathing, I see. A great pity for the town.", "The years have only made {2:obj} sourer.", "The parish got poorer the day {2:subj} {2:drew} breath."],
            voice: &[],
            then: Some(RIVAL_BITE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["farewell"],
            respond: None,
            lines: &["I have nothing to say to {2:obj}, scoundrel.", "Out of my way. I don't parley with knaves.", "Begone before my temper answers for me."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const HUNGER_HOPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:food"],
    calls: &["If the rye crop ripens before the frost, we'll see another spring.", "They say the monastery may open its tithe barn if hunger deepens.", "We have boiled nettles and wild leeks before, and survived.", "A shared loaf tastes twice as sweet in hard times."],
    voice: &[
        // even the hope beat sours while they're cold (#1269)
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Hope, aye - we've had that before, and thinner soup after it.", "The rye'll ripen or it won't. It's never asked us which."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["If the rye ripens - and if folk stood together, maybe it needn't fall to luck."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["The tithe barn opens for those who ask together, I hear."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[],
            act: Act::Agree, concepts: &["concern:food"],
            respond: None,
            lines: &["Aye, hold fast - the year's not written yet.", "Here's to a fat harvest, neighbor."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // the thin joke is how the lean ones cope (E5-1): power is
        // only fair game where it's safe to say - E5-6 fills the matrix
        Reply {
            criteria: &[], weight: 0.30, bias: &[(Gregarious, 1.6)],
            act: Act::Joke, concepts: &["concern:food", "joke"],
            respond: None,
            lines: &["I've started watering the soup twice - the pot's never looked fuller.", "My belt's a sundial now - every notch marks a leaner week."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["The reeve's horse eats better than my children. Mind, it works harder."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["The lord's feast had more ale than harvest - I'll say it to {2:obj} and no further."] },
            ],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Aye, hold fast. Better days will follow.", "The earth will feed us yet. Go gently, {other}.", "May tomorrow bring a fuller kettle.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HUNGER_SHARE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:food", "aid"],
    calls: &["We boiled a pot of turnip tops and barley chaff this noon.", "My store bin has nothing but dust and two dried apples.", "The children went to sleep crying with empty bellies last night.", "A shared loaf tastes twice as sweet in hard times."],
    voice: &[
        // the same trouble, told by where they stand (#1269): cold it's
        // fate; warming it's a thing that might yet move
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["The bin's dust, and there's nothing for it. There never is.", "Hungry's what we are, {other}. It's not a thing to fix - it's a thing to be."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["The bin's near bare - but somebody asked what I'd want, if I could want. Turns me over still.", "Thin pickings here - though I've started wondering if it must stay so."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["The bin's thin - but empty pots are what the meeting's for, {other}."] },
        Voice { criteria: &[Has("self:status:thaw_acting")], lines: &["Thin here, aye - and it's my own task on the plan to see it thicker."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:food", "aid"],
            respond: None,
            lines: &["Take half my loaf, neighbor. No child should hunger in this village.", "Come by our hearth; there is pottage enough for two more bowls.", "We have a sack of roots in the cellar. Let us divide it.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(HUNGER_HOPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Stingy, 3.0)],
            act: Act::PushBack, concepts: &["concern:food"],
            respond: None,
            lines: &["Everyone's bin is empty; don't look to my door for bread.", "We each must stretch our own store as best we can.", "My own children gnaw their crusts; I have nothing spare."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const HUNGER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:food"],
    calls: &["Good morrow, {other}. A hollow belly makes for a slow step.", "Well met, friend, though the times are thin and hard.", "Greetings, {other}. {2:have} {2:subj} found anything to put in the pot?", "A peaceful day to {2:obj}, {other}."],
    voice: &[
        // the rung the greeting's spoken from (#1269): a cold greeting
        // is flat, a warm one looks for what could move
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Morrow, {other}. Another hungry one - same as all the rest.", "Aye, morrow. Thin again, like every day these years."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Morrow, {other} - the pot's thin, though I've a question still turning over in me."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Morrow, {other} - lean times, but we've folk looking at it together now."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Morrow, {other} - a hollow belly, but glad company.", "Well met, {other} - we've both known thinner tables. Barely."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - is there bread at {2:poss} hearth? Ours runs thin.", "{other} - kin shouldn't have to ask, but {2:have} {2:subj} grain?"] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Thin pickings along this road.", "A hungry greeting, traveler - the fields gave little this year."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other} - lean times on every lane.", "Fair morning, {other}. The pot stays empty these days."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Morrow, {other}. Hunger bites even the unworthy.", "{other}. If {2:poss} larder's full, I'd not believe {2:obj} anyway."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:food"],
            respond: None,
            lines: &["Morrow, {other}. The porridge is thin as ditchwater this week.", "Aye, we scrape the bottom of the flour chest.", "The storehouse has dwindled down to crumbs.", "Take half this barley loaf; we must look after each other."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Thin porridge for us both, {other} - but half my loaf is {2:own}.", "Crumbs at our house too - come share what little there is."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["The bin's near empty, {other} - but blood shares what it has.", "Our house eats thin too; come by and we'll stretch it."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["We scrape what we can, like all here.", "The storehouse holds little for strangers' asking."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Thin enough, {other} - though I'd not expect {2:obj} to spare a crumb.", "We manage - no thanks to the likes of {2:obj}."] },
            ],
            then: Some(HUNGER_SHARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["We tighten our belts and carry on.", "Talk won't fill the larder. I must seek wild greens.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Come by the house tonight, {other} - we'll make it stretch."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Talk fills no bellies - least of all {2:own}.", "I must seek greens. Keep {2:poss} hands off my patch."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The parish's weather (#1696): while the village stands under a
/// weight - the stores out, the fever in the beds, the lord's squeeze,
/// fresh graves, the long snow - a talk opens on it ahead of the
/// shop-talk. One tree serves every axis: the `village:*` facts pick
/// which register the calls and answers are heard in, so a famine and
/// a fever don't share a mouth
const HARD_TIMES_SHARE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["village", "pressure"],
    calls: &["We bear it the same as all the rest, {other} - what else is there?",
        "Day by day, {other}. That's all anyone's doing."],
    voice: &[
        Voice { criteria: &[Has("village:famine")], lines: &[
            "We stretch what's left - chaff bread, thin pottage. It holds, barely.",
            "The children sleep hungry. I tell them the fasting saints did the same."] },
        Voice { criteria: &[Has("village:fever")], lines: &[
            "We air the bedding and keep the sick apart - it's all there is to do.",
            "We burn what the sick touched and pray the rest of us stand."] },
        Voice { criteria: &[Has("village:winter")], lines: &[
            "We burn the furniture's scrap before the good beams - the cold doesn't care.",
            "We hole up close and count the woodpile twice a day."] },
        Voice { criteria: &[Has("village:unrest")], lines: &[
            "We count what's owed and what's been took, {other} - the sums don't sit right.",
            "We talk of it low - but we talk of it."] },
        Voice { criteria: &[Has("village:bereft")], lines: &[
            "We keep the candles and say their names - what else is there?",
            "The empty beds don't fill themselves. We visit who's left."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["village", "pressure"],
            respond: None,
            lines: &["Aye - and it'll pass, as all things pass.", "Aye. That's the size of it, {other}."],
            voice: &[
                Voice { criteria: &[Has("village:famine")], lines: &["The first green shoots can't come soon enough."] },
                Voice { criteria: &[Has("village:unrest")], lines: &["Not forever, {other} - not if folk say it plain."] },
                Voice { criteria: &[Has("village:bereft")], lines: &["God keep their souls - and ours while He's at it."] },
            ],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Go well, {other} - mind what's left of the week.", "I should be about it. Keep {2:poss} door warm, {other}.", "May the road be kind under {2:poss} boots."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HARD_TIMES_GREET: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["village", "pressure"],
    calls: &["These are thin days, {other}.",
        "Hard times on the parish - {2:have} {2:subj} felt it?"],
    voice: &[
        Voice { criteria: &[Has("village:famine")], lines: &[
            "The pile's near out, {other} - my pot's been water and prayer this week.",
            "Bread's a memory already. What's left at the market wouldn't feed a sparrow.",
            "The fields gave nothing, and now the store gives nothing. Thin days, {other}."] },
        Voice { criteria: &[Has("village:fever")], lines: &[
            "The sweat's in three beds and counting - God keep it from {2:poss} door.",
            "Every other door's shut these days, {other}. The fever takes who it will."] },
        Voice { criteria: &[Has("village:winter")], lines: &[
            "The cold's settled for good - it'll take the weak before it's done, {other}.",
            "Snow past the sill and the year's hardly turned. Hard season, this."] },
        Voice { criteria: &[Has("village:unrest")], lines: &[
            "Every door's got a grievance these days, {other} - the parish's patience wears thin.",
            "The lord's tally grows and the bread shrinks. Something will give, mark me."] },
        Voice { criteria: &[Has("village:bereft")], lines: &[
            "Fresh earth in the churchyard, {other} - the bell's been at it this week.",
            "Too many empty chairs of late. The parish grieves."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["village", "pressure"],
            respond: None,
            lines: &["Aye - thin days make thin talk. But we bear them together."],
            voice: &[
                Voice { criteria: &[Has("village:famine")], lines: &[
                    "Aye - the children crying keeps me up more than the cold.",
                    "Aye. We boiled chaff this noon; tomorrow's thinner."] },
                Voice { criteria: &[Has("village:fever")], lines: &[
                    "Aye - burn the bedding, they say. I keep my door barred.",
                    "Aye - and pray it passes our house by, {other}."] },
                Voice { criteria: &[Has("village:winter")], lines: &[
                    "Aye - the woodpile's thin as the larder.",
                    "Aye - and the lambing's still to come in it."] },
                Voice { criteria: &[Has("village:unrest")], lines: &[
                    "Aye - taxed, blamed, shorted. Folk are talking of it, {other}.",
                    "Aye - I've a tally of my own that needs saying."] },
                Voice { criteria: &[Has("village:bereft")], lines: &[
                    "Aye - three candles lit on the lane. God rest them.",
                    "Aye - the churchyard's fuller than the pews this week."] },
            ],
            then: Some(HARD_TIMES_SHARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Gregarious, 1.5)],
            act: Act::Question, concepts: &["village", "pressure"],
            respond: None,
            lines: &["Aye - and how does {2:poss} house weather it?"],
            voice: &[],
            then: Some(HARD_TIMES_SHARE), end: None,
        },
    ],
};

const COLD_FIRE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:fuel"],
    calls: &["The kindling caught! Sweet smoke rises up the chimney at last.", "Blow on the embers gently - the dry twigs are taking flame.", "Ah, blessed warmth! My stiff fingers can move again.", "Dry kindling keeps the bitter frost outside the thatch."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Bank the coals with ashes so they last till dawn.", "May {2:poss} hearth keep hot through the bitter night.", "The fire is life in winter. Fare {2:obj} well.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const COLD_GATHER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:fuel", "aid"],
    calls: &["{2:will} {2:subj} come to the common wood to cut deadwood and kindling?", "Can I borrow a shovel to dig peat from the dry bog?", "{2:have} {2:subj} glowing embers in an iron pot to rekindle our cold stones?", "Can we gather dry brushwood together in the common woods?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:fuel"],
            respond: None,
            lines: &["Aye, I'll bring my hatchet. Two bundles of dry boughs will do.", "Take these live embers from my hearth; shield them from the wind.", "We stacked dry turf behind the byre; take what {2:subj} {2:can} carry.", "Take these live embers in an iron pot to start {2:poss} fire."],
            voice: &[],
            then: Some(COLD_FIRE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["concern:fuel"],
            respond: None,
            lines: &["The bailiff watches the common wood; he will fine us for gathering boughs.", "My own hearth barely has coals to warm an egg.", "Gather on the lord's land if {2:subj} {2:dare} - not from me."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const COLD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:fuel"],
    calls: &["Brrr! Good morrow, {other}. The rime frost is inside the thatch.", "Well met, neighbor. My breath freezes right before my nose.", "Greetings, {other}. The wind blows straight through the cottage wattles.", "A peaceful day to {2:obj}, {other}."],
    voice: &[
        // cold hands, colder prospects (#1269): the same frozen hearth
        // is fate to the helpless and a shared errand to the warm
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Brrr - another frozen morning, {other}. The hearth stays out; it always does."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Brrr! Morrow, {other} - cold, though a body's asked me what I'd do about it."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Brrr! Morrow, {other} - the wood's shared out at the meeting now, mind."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:fuel"],
            respond: None,
            lines: &["Our hearth went cold before midnight; the stone is like ice.", "We have burned through the last dry bundle of brushwood.", "Without dry turf, the sheepfold is warmer than the cottage.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(COLD_GATHER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must chop timber before the dark settles in.", "Keep moving or the cold will numb {2:poss} toes. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const DRUNK_SOBER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["A bucket of icy well water over the skull will clear the cobwebs.", "The plough handles won't hold themselves, headache or no.", "Bread and pickled onions, that's what the stomach needs.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Walk straight, {other}, or the reeve will notice.", "No more brown ale for {2:obj} until Sunday! Fare well.", "May the sun be gentle on {2:poss} poor eyes today.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const DRUNK_RECALL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Did I truly challenge the smith to a wrestling match at midnight?", "They say we sang every verse of the Harvest ballad three times over.", "My purse is six silver pennies lighter than when dusk fell.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        // a reach and a laugh back at the drunk's recall (E5-1, #1430)
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Kind, 1.5), (Gregarious, 1.5)],
            act: Act::Bid, concepts: &["bid", "smalltalk"],
            respond: None,
            lines: &["Look, the swallows are back.", "It's good to see {2:obj} about, {other}.", "Walk with me to the well a moment, {other}."],
            voice: &[],
            then: Some(DRUNK_SOBER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.35, bias: &[(Gregarious, 1.6)],
            act: Act::Joke, concepts: &["joke", "smalltalk"],
            respond: None,
            lines: &["They'll sing of that night till harvest - mostly the fiddler's part.", "I told the rain to stop and it came down the harder. So much for commanding the weather."],
            voice: &[
                Voice { criteria: &[Has("self:status:drunk")], lines: &["Ale's the only thing that listens to me - and even it makes me pay."] },
            ],
            then: Some(DRUNK_SOBER), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["{2:subj} {2:did} indeed! And fell backwards over the ale barrel!", "Aye, and the fiddler laughed so hard his bow went flying.", "It was a merry night, though the morning pays the reckoning.", "Aye, there is truth in what {2:subj} {2:say}.", "{2:subj} {2:have} spoken the plain heart of the matter."],
            voice: &[],
            then: Some(DRUNK_SOBER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Pious, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            respond: None,
            lines: &["{2:subj} made an ungodly spectacle of {2:self}.", "A shameful racket; the priest shook his head at the doorway.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(DRUNK_SOBER), end: None,
        },
    ],
};

const DRUNK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &["Oof... good morrow, {other}. Don't shout, I pray {2:obj}.", "Well met, friend. My head is hammering like the smith's anvil.", "Ugh, the sunlight is blinding today, {other}.", "A peaceful day to {2:obj}, {other}."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Oof - morrow, {other}. Don't laugh; my head's an anvil today.", "Well met, {other} - {2:subj} {2:would} have warned me off that third cup, I know."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Oof - morrow, {other}. Don't tell our mother the state of me.", "{other}! Aye, it's me - the tavern won last night."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Oof - morrow, stranger. Mind the volume, I pray.", "A rough greeting, traveler - the ale had a bite."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Oof. Fair morning, {other} - the light's too loud today.", "Morrow, {other}. A rough night at the tavern."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Oof - {other}. Say one word about last night and we're done.", "Morrow, {other}. Laugh and I'll remember it."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["smalltalk"],
            respond: None,
            lines: &["Ha! The tavern-keeper's ale had a bite to it, didn't it?", "{2:subj} {2:look} like a corpse dug up from the wet turf.", "I warned {2:obj} that dark brew was brewed for giants.", "We do what we can with the strength Heaven gives."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Ha! I told {2:obj} that dark brew was brewed for giants, {other}.", "{2:subj} {2:look} like death's cousin - and I should know, I feel the same."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our grandfather drank the same way - the shame's in the blood.", "Mother warned the whole house about that ale."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["The ale bites the unready, stranger.", "{2:subj} {2:look} like the grave let {2:obj} out early."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["{2:subj} {2:look} a wreck, {other} - I'd pay to see it again.", "The ale had more spine than {2:obj}, I see."] },
            ],
            then: Some(DRUNK_RECALL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Sleep it off in the hayloft, {other}.", "Go dunk {2:poss} head in the horse trough. Fair day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Sleep it off, {other} - I'll cover for {2:obj} at the work.", "Rest {2:poss} poor head, friend."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Sleep it off, {other} - somewhere out of my sight.", "Go soak {2:poss} head - and stay gone."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_AID: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid", "family"],
    calls: &["Blood is thicker than river mud, {other}.", "Whatever befalls the village, our clan stands shoulder to shoulder.", "Our grandfather laid the hearthstones in both our cottages.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[],
            act: Act::Agree, concepts: &["family"],
            respond: None,
            lines: &["Kindly, kin - I'll not forget it.", "Blood looks to blood, and here we are."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // kin ask kin (E5-1): the errand's ghost-tied shapes come with
        // E5-7 - this is the family's ordinary ask
        Reply {
            criteria: &[], weight: 0.35, bias: &[],
            act: Act::Errand, concepts: &["errand", "family"],
            respond: None,
            lines: &["Carry a word to the hermit for me, quiet like. There's a loaf in it.", "Mind the stall an hour come market day - a loaf and my thanks for it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Always, kin. The family latch is never barred to {2:obj}.", "God keep our house and hearth safe from storm.", "See {2:obj} at Sunday meal, {other}.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_FAMILY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["family"],
    calls: &["How fares mother's knee in this damp spell?", "Has our cousin returned with the pack horse from the fair?", "{2:do} {2:subj} need an extra hand to dig the potato trench?", "How fares our sister's household down by the green?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["family"],
            respond: None,
            lines: &["She sits by the fire with warm sheep's grease; it eases the pain.", "Aye, he brought home two bags of salt and a fine sickle.", "Come by with a spade after the noon bell, brother.", "Our kin will stand beside {2:obj} through good season and bad."],
            voice: &[],
            then: Some(KIN_AID), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "family"],
    calls: &["Well met, cousin {other}!", "Good morrow, kin! How fares the old cottage?", "Greetings, brother! It does my heart good to see {2:poss} face.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            respond: None,
            lines: &["And to {2:obj}, kin! The roof still holds.", "Well met, dear cousin. {2:subj} {2:be} looking hale and hearty.", "God bless {2:obj}, kin. What news from {2:poss} side of the lane?", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(KIN_FAMILY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_PART: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["friendship"],
    calls: &["A good friend is worth more than a pouch of silver.", "We have shared crusts and sweat for twenty years now.", "{2:poss} word has never failed me, {other}.", "True fellowship is the richest coin in the parish."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Nor {2:obj}, me old friend. Walk safe.", "We will share a cup of cider when the work is done.", "Fair day, {other} - peace on {2:poss} household.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_TRUST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["friendship"],
    calls: &["{2:do} {2:subj} recall the summer the mill dam broke?", "Can I count on {2:poss} team when we haul the heavy timber?", "{2:have} {2:subj} heard any whispers about the new taxes?", "Can I count on {2:poss} strong arm when we raise the barn?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["friendship"],
            respond: None,
            lines: &["Aye, we were waist-deep in mud until midnight saving the sacks!", "My oxen are {2:own} whenever {2:subj} {2:need} them, friend.", "The bailiff spoke of three pennies per hide, but we'll contest it.", "{2:poss} word is better than gold; count on my team anytime."],
            voice: &[],
            then: Some(FRIEND_PART), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// Sweethearts get a beat of their own (#1372): the coarse Kin fold
/// never told a lover from a cousin
const LOVER_BEAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:lover"],
    calls: &["My heart sits lighter having seen {2:obj} this morning.", "I saved a smile for {2:obj} since yesterday - here it is.", "The lane could burn down and I'd not mark it, talking with {2:obj}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:lover"],
            respond: None,
            lines: &["And mine at seeing {2:obj} - walk with me a while.", "{2:poss} words are warmer than the hearth, love.", "As {2:subj} {2:say} - the day is kinder for {2:obj}."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::PushBack, concepts: &["bond:lover"],
            respond: None,
            lines: &["Sweet words - and {2:did} {2:subj} say the same to the miller's daughter?", "{2:subj} {2:be} bold this morning - what {2:do} {2:subj} want of me?"],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Till this evening, my heart.", "Go well - and think of me at the well, love."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// Blood kin, not just any kin (#1372): family news, family claims
const KIN_BLOOD: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:family"],
    calls: &["Mother sent word - she prays for {2:poss} cough every bell.", "Blood is blood - what little we have is {2:own} to share.", "{2:poss} grandfather and mine broke the same field, remember."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:family"],
            respond: None,
            lines: &["Kin looks after kin - now and always.", "Tell her I mend, and she should spare her knees.", "Our houses stand closer than these lanes show."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Give my love to {2:poss} mother when {2:subj} {2:see} her.", "Go well, cousin - the house is always open to {2:obj}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// A mentor's counsel (#1372): the teacher's register, not smalltalk
const MENTOR_COUNSEL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:mentor"],
    calls: &["Mark this well - the work outlives the wage.", "Patience, lad - the craft reveals itself to those who wait.", "I was green once, same as {2:obj} - the hand learns what the eye cannot."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:mentor"],
            respond: None,
            lines: &["{2:poss} words are worth more than a day's wage, master.", "Aye, I mark it - as {2:subj} {2:have} taught me.", "Every lesson {2:subj} {2:give} me I keep by heart."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I go to practice what {2:subj} {2:said}. Fair day, master.", "By {2:poss} leave, master - the bench calls."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The apprentice's eager report (#1372): the learner's register
const APPRENTICE_EAGER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:apprentice"],
    calls: &["I set the joints as {2:subj} showed me - {2:will} {2:subj} look?", "The work goes quicker now - {2:poss} lessons hold.", "Tell me again how the old ones did it - I want it right."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:apprentice"],
            respond: None,
            lines: &["Well enough - keep {2:poss} hand steady and {2:poss} eyes open.", "{2:subj} {2:learn} faster than I did at {2:poss} age.", "Eager hands make light work - go on."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Back to the bench with {2:obj} - I'll look in before the bell.", "Go on then, and mind the edge as I showed {2:obj}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "friendship"],
    calls: &["A hearty good morrow to {2:obj}, {other}!", "Well met, true friend! The sun shines brighter when {2:subj} {2:be} near.", "Greetings, old comrade! How goes the day's labor?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["friendship"],
            respond: None,
            lines: &["And to {2:obj}, {other}! It gladdens my soul to see {2:obj}.", "Well met indeed, friend. The chores are lighter for {2:poss} smile.", "Good day, partner! We'll weather whatever comes.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(FRIEND_TRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_PASS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["stranger"],
    calls: &["Keep to the beaten track; the marshes to the east are treacherous.", "The alehouse by the bridge offers straw beds if {2:subj} {2:have} coin.", "The parish watch keeps guard after the curfew bell.", "The road is wide enough for all peaceful travelers."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.90, bias: &[],
            act: Act::Agree, concepts: &["stranger"],
            respond: None,
            lines: &["A fair road and fair talk, traveler.", "Then the parish wishes {2:obj} well of it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        // anyone carries a worry (E5-1): the stranger's the one who
        // knows the roads a fear travels on
        Reply {
            criteria: &[], weight: 0.35, bias: &[],
            act: Act::Worry, concepts: &["worry", "stranger"],
            respond: None,
            lines: &["Is the north road safe still? I've kin on it.", "They say the levy rides again come spring - is it true, {other}?"],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["My thanks for the counsel, villager. I pass in peace.", "Safe travels to {2:obj} as well. Good day.", "The highway calls; farewell.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_PATH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["stranger"],
    calls: &["Where does this trail lead past the ridge, traveler?", "What town {2:do} {2:subj} hail from, stranger?", "Is the road north safe from cutpurses and wolves?", "What brings {2:poss} footsteps into our quiet parish?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["stranger"],
            respond: None,
            lines: &["It leads down to the cathedral town and the salt wharves.", "I come from the valley beyond the high moors.", "The road is quiet by day, but light a torch if {2:subj} {2:walk} at dusk.", "The road to the cathedral town follows the river south."],
            voice: &[],
            then: Some(STRANGER_PASS), end: None,
        },
        // a stranger owes no answers (E5-1): the visible turn-aside
        Reply {
            criteria: &[], weight: 0.30, bias: &[(Proud, 1.5)],
            act: Act::Evade, concepts: &["evade", "stranger"],
            respond: None,
            lines: crate::game::conversation::DEFLECTED,
            voice: &[],
            then: Some(STRANGER_PASS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "stranger"],
    calls: &["Halt there, traveler. What brings {2:obj} through our parish?", "Good day, stranger. {2:subj} {2:be} far from the king's highway.", "Peace be upon {2:poss} journey, traveler. Who {2:be} {2:subj}?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["stranger"],
            respond: None,
            lines: &["Greetings, villager. I mean no harm on these lanes.", "Well met. I am but a traveler seeking the ford across the water.", "Peace to {2:poss} fields, friend. I look for the nearest smithy.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(STRANGER_PATH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["stranger"],
            respond: None,
            lines: &["My business is my own, goodman.", "The king's highway is open to every free soul.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(STRANGER_PASS), end: None,
        },
    ],
};

const SMITH_IRON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:smith"],
    calls: &["Good Spanish iron rings clear; bad slag thuds like lead.", "A hammer needs an eye that sees the bend before it strikes.", "The quench tub hisses when the blade has its proper temper.", "The hammer makes the iron obey, neighbor."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Keep the fire roaring, smith. Good day.", "May {2:poss} hammer never miss its mark. Fair day.", "I will return for the coulter at sunset.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMITH_WORK: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:smith"],
    calls: &["{2:can} {2:subj} forge three dozen horseshoe nails by tomorrow?", "My plough coulter struck a buried boulder and snapped clean.", "{2:do} {2:subj} have charcoal enough to heat the heavy anvil?", "Is the hearth hot enough to weld the iron tyre?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:wx-snow")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:smith"],
            respond: None,
            lines: &["The snow's the best customer the forge has - doors warp, hinges split, and everyone remembers the smith.", "Aye, hot work in cold weather; the whole lane's mending lands on my anvil.", "This frost's cracked two cart-wheels already; the queue runs past the door."],
            voice: &[],
            then: Some(SMITH_IRON), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:smith"],
            respond: None,
            lines: &["Bring me two groats of coal and the nails will be ready by dawn.", "Lay the broken iron on the bench; I'll weld the seam.", "Aye, the charcoal burner delivered two sacks from the pit yesterday.", "The white heat is ready; bring the iron to the anvil."],
            voice: &[],
            then: Some(SMITH_IRON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMITH_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:smith"],
    calls: &["Good morrow, smith! The sparks are flying bright today.", "Well met, master smith! The sound of {2:poss} forge wakes the village.", "Hail to the forge, {other}! Sweating by the bellows already?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:smith"],
            respond: None,
            lines: &["Morrow, {other}! The coals are white-hot and hungry for iron.", "Well met! Step inside out of the wind, mind the sparks.", "Aye, iron waits for no man's leisure. What {2:need} {2:subj}?", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(SMITH_WORK), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_TOLL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:miller"],
    calls: &["One scoop in twenty is the customary multure, no more, no less.", "The millstones were freshly dressed with the chisel last Pentecost.", "Fine white flour for white loaves, coarse bran for the byre.", "The millpond water never turns the wheel twice."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Keep the wheel turning, miller. Fair day.", "Mind {2:subj} {2:do} not spill my meal on the floorboards.", "Until harvest time, master of the water.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_GRIST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:miller"],
    calls: &["Is the river running high enough to turn the big wheel?", "{2:can} {2:subj} grind my two sacks of barley before noon?", "Did the damp get into the meal hopper after the downpour?", "Will the mill race freeze if the frost sharpens tonight?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:miller"],
            respond: None,
            lines: &["The millpond is full to the weir boards; she turns swift today.", "Stack {2:poss} sacks by the scale; {2:poss} meal will be ground by mid-day.", "The oak bins are dry as bone; not a drop touched the flour.", "The river turns the wheel fast; the meal pours sweet."],
            voice: &[],
            then: Some(MILLER_TOLL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:miller"],
    calls: &["Good morrow, miller! The roar of {2:poss} weir echoes down the valley.", "Well met, {other}! Flour dusts {2:poss} coat like fresh snow.", "Hail to the mill, {other}! Is the grist running smooth?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:miller"],
            respond: None,
            lines: &["Morrow, neighbor! The stones rumble and the water runs free.", "Well met! Unload {2:poss} cart; the hopper is ready.", "Aye, dust in my hair and bread in my belly! What grist {2:bring} {2:subj}?", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(MILLER_GRIST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_CRUST: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:baker"],
    calls: &["A well-swept oven floor gives the sweetest crust in the parish.", "Yeast from the brewer's ale makes the dough rise twice as high.", "Golden loaves on the wooden board - the glory of the harvest.", "Flour and salt and yeast - the holy trinity of the table."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["The smell of {2:poss} loaves is a blessing to the lane. Fare well.", "May {2:poss} oven never grow cold, baker.", "I take my warm crust home to the children. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_BREAD: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:baker"],
    calls: &["Are the morning rye loaves ready to pull with the peel?", "{2:can} {2:subj} bake my barley dough in {2:poss} community oven?", "{2:have} {2:subj} any day-old trenchers left for a penny?", "Are the loaves golden brown on the oven bricks?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:selling")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:baker"],
            respond: None,
            lines: &["The stall's out by the cross - the loaves went before the second bell.", "Market day's the oven's Sunday; the batch never rests and the queue never shrinks.", "Sold the morning's bake already - the market crowd clears a board faster than any feast."],
            voice: &[],
            then: Some(BAKER_CRUST), end: None,
        },
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:baker"],
            respond: None,
            lines: &["Market day's the oven's Sunday; the batch never rests.", "The bakehouse smells the crowd before the bells do."],
            voice: &[],
            then: Some(BAKER_CRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:baker"],
            respond: None,
            lines: &["Aye, hot from the bricks and brown as a hazelnut!", "Bring {2:poss} dough before noon; there is room beside the landlord's pies.", "Take four trenchers; they are crisp and good for soaking broth.", "The crust is crisp and hot from the stone floor."],
            voice: &[],
            then: Some(BAKER_CRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:baker"],
    calls: &["Good morrow, baker! {2:poss} chimney breathes the sweetest air in town.", "Well met, {other}! The smell of fresh crust drew me from the lane.", "Hail, master of the peel! Has the first batch risen?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:baker"],
            respond: None,
            lines: &["Morrow, {other}! Flour on my hands and fire in the hearth.", "Well met! The oven is blazing hot and the loaves are baking.", "A warm welcome to {2:obj}! Step in and feel the heat.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(BAKER_BREAD), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_CLOTH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:weaver"],
    calls: &["Tight warp and even weft make a cloth that sheds the rain.", "Madder root gives a brave red, but woad blues last thirty years.", "A warm woolen blanket keeps the winter chills from the cradle.", "The pattern grows one thread at a time on the loom."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} shuttle fly true and never break a thread.", "I will return when the piece comes off the frame. Fare well.", "Good cloth is true wealth. Good day, weaver.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_FLEECE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:weaver"],
    calls: &["Did the sheep shearers bring in the upland wool yet?", "{2:can} {2:subj} weave a yard of heavy russet for my work kirtle?", "Is the spun yarn dry enough to wind onto the spools?", "How many yards of wool cloth come off the loom today?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:selling")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:weaver"],
            respond: None,
            lines: &["There's a bolt of fustian on my stall and three lengths of woollen besides.", "Market day's the loom's market too - the weaver sells what the shepherd shears.", "I've cloth at the cross and buyers at the door; a good day for thread."],
            voice: &[],
            then: Some(WEAVER_CLOTH), end: None,
        },
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:weaver"],
            respond: None,
            lines: &["The market crowd's all dressed in somebody's winter.", "Every sleeve on the square was somebody's loomwork."],
            voice: &[],
            then: Some(WEAVER_CLOTH), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:weaver"],
            respond: None,
            lines: &["Ten sacks of greasy fleece sit by the washing trough.", "I'll set the warp tomorrow; russet is sturdy and warm for field work.", "Aye, the carders combed it fine and the spindles are humming.", "The shuttle flies true and the cloth grows stout."],
            voice: &[],
            then: Some(WEAVER_CLOTH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:weaver"],
    calls: &["Good morrow, weaver! The click-clack of {2:poss} loom is steady today.", "Well met, {other}! Surrounded by skeins of dyed wool, I see.", "Hail to the loom, neighbor! How runs the shuttle this morning?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:weaver"],
            respond: None,
            lines: &["Morrow, {other}! Counting the threads from dawn till twilight.", "Well met! The treadles are flying and the cloth grows fast.", "Greetings! A weaver's work is never done while sheep have wool.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(WEAVER_FLEECE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_PEGS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:carpenter"],
    calls: &["Heart of oak and seasoned ash will outlast three generations.", "An auger hole drilled straight needs only a split hazel peg to hold.", "Cedar smells sweet in a chest, but elm makes the stoutest wheel hubs.", "True timber bends to no man's haste."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} adze stay sharp, carpenter. Fare well.", "I will look for the cart axle on Friday. Good day.", "Stout timber makes a sturdy home. Walk in peace.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_BEAMS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:carpenter"],
    calls: &["{2:can} {2:subj} mortise a tie-beam for the cottage roof ridge?", "{2:did} {2:subj} square the timber logs felled in the lord's wood?", "{2:do} {2:subj} have spare dry pegs to secure the wattle hurdle?", "{2:can} {2:subj} hew a sturdy axle-tree for my cart?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:wx-snow")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:carpenter"],
            respond: None,
            lines: &["Snow's a carpenter's payday - every fence and shed remembers it's wood.", "The frost split the tithe barn's rafter; I've a truss to reset come morning.", "Cold split the warden's gate clean through - wood talks loud in the freeze."],
            voice: &[],
            then: Some(CARPENTER_PEGS), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:carpenter"],
            respond: None,
            lines: &["I have seasoned oak ready on the trestles; it will bear the rafters.", "The broadaxe squared two dozen beams; they are stacked to cure.", "Take a handful of dried oak pins from the tub by the bench.", "The oak timber is straight-grained and cured dry."],
            voice: &[],
            then: Some(CARPENTER_PEGS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:carpenter"],
    calls: &["Good morrow, carpenter! Shavings and sawdust fly thick today.", "Well met, {other}! The scent of fresh-cut pine is in the lane.", "Hail to the woodwright! Has the new timber arrived from the coppice?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:carpenter"],
            respond: None,
            lines: &["Morrow, neighbor! Measuring twice and cutting once, as ever.", "Well met! The saw pit was busy since the dawn bell.", "Greetings! A stout tree fell yesterday; fine timber for building.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(CARPENTER_BEAMS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_LIME: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:mason"],
    calls: &["Hot slaked lime and sharp sand bond tighter than the quarry itself.", "A drystone foundation keeps the rot out of the lowest timbers.", "Granite breaks the chisel, but limestone yields to an honest stroke.", "Three cartloads of lime came down the hill at daybreak."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} plumb-line hang true, mason. Fair day.", "Stone stands when all else fades. Farewell.", "I will bring the ox-cart for the rubble tomorrow.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_WALL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:mason"],
    calls: &["{2:can} {2:subj} dress ashlar stones for the new church window?", "Did the quarry deliver enough rubble stone to rebuild the wall?", "Will the lime mortar set before the autumn rains begin?", "Will the limestone mortar hold against the river damp?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:mason"],
            respond: None,
            lines: &["Rain lays the mortar slow; a wet day's a measuring day.", "The wall's sheeted and waiting - mortar won't set under a dripping sky.", "Wet weather stops the trowel; the stone keeps, the lime doesn't."],
            voice: &[],
            then: Some(MASON_LIME), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:mason"],
            respond: None,
            lines: &["I am chiseling the jambs now; fine freestone from the quarry.", "Two heavy wagonloads came down the hill this morning.", "It will cure rock-hard if we keep the frost off it for a fortnight.", "The lime mortar sets hard as flint between the stones."],
            voice: &[],
            then: Some(MASON_LIME), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:mason"],
    calls: &["Good morrow, mason! The ringing of {2:poss} hammer on stone echoes far.", "Well met, {other}! White rock dust on {2:poss} leather apron.", "Hail to the stone-cutter! Setting foundations today?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:mason"],
            respond: None,
            lines: &["Morrow, {other}! Stone by stone the hall rises from the earth.", "Well met! Testing the level and squaring the corners.", "Greetings! A heavy trade, but a wall built right outlasts empires.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(MASON_WALL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_PASTURE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:shepherd"],
    calls: &["The upland turf is sweet and short; the ewes grow fat on it.", "The sheepfold dung will enrich the barley fields when autumn comes.", "A good dog is worth three hired hands on the open hills.", "The hound barked at midnight and drove the beast away."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Keep {2:poss} eye on the brushwood, shepherd. Fair day.", "May the flock prosper and the wolves stay distant.", "Good grazing to {2:poss} flock, neighbor.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_EWES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:shepherd"],
    calls: &["How many twin lambs dropped in the fold last night?", "Did the wolf from the high ridge trouble the sheepfold?", "Is it time to drive the flock down to the river wash?", "Did the herd dog drive the stray lambs from the briars?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:spring")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:shepherd"],
            respond: None,
            lines: &["The lambs dropped early this year - the commons are thick with them already.", "Lambing's near done; two ewes still heavy and a fox about the fold.", "Spring's the busy watch; every night I count the flock twice over."],
            voice: &[],
            then: Some(SHEPHERD_PASTURE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:shepherd"],
            respond: None,
            lines: &["Six fine lambs, all suckling strong by their mothers.", "Aye, the hound fetched the strays from the briars by daybreak.", "By next full moon the fleece will be clean for the shearers.", "The ewes are content and the lambs suckle strong."],
            voice: &[],
            then: Some(SHEPHERD_PASTURE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:shepherd"],
    calls: &["Good morrow, shepherd! Bleating ewes fill the morning air.", "Well met, {other}! Cloaked in wool and leaning on {2:poss} crook.", "Hail, drover of the flock! How fare the sheep on the commons?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:shepherd"],
            respond: None,
            lines: &["Morrow, neighbor! Watching for hawks and counting tails.", "Well met! The bells on the wethers make a pleasant music.", "Greetings! Wind on the hill, but the ewes are peaceful.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(SHEPHERD_EWES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_CUP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:brewer"],
    calls: &["Clear spring water and sound malt make an ale fit for a baron.", "An ale-stake hung with green boughs means a fresh barrel is tapped.", "A tankard at sundown washes the harvest dust from the throat.", "A foaming mug cheers the tired ploughman at sundown."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} vats never sour, brewer. Fair day.", "I will bring my pewter mug at sundown. Farewell.", "A good brew makes neighbors into brothers. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_MASH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:brewer"],
    calls: &["Is the new brewing of October ale ready for the tap?", "Did the maltsters bring in the sprouted barley from the floor?", "{2:do} {2:subj} have small beer to sell for the field workers' pails?", "Has the barley mash cooled enough to pitch the yeast?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:autumn")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:brewer"],
            respond: None,
            lines: &["The October ale is tapped - the year's brewing is the year's best.", "Harvest ale and October ale both broached this week; the vats earn their keep.", "The autumn mash is the strong one - the village will keep Christmas on it."],
            voice: &[],
            then: Some(BREWER_CUP), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:brewer"],
            respond: None,
            lines: &["It has fermented sweet and strong; we broach the cask today.", "Three carts of roasted malt came down from the kiln yesterday.", "Aye, fill {2:poss} pails; small beer keeps the reapers from fainting.", "The ale has cleared and the barrel is broached sweet."],
            voice: &[],
            then: Some(BREWER_CUP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:brewer"],
    calls: &["Good morrow, brewer! Warm steam rises over {2:poss} brew-house.", "Well met, {other}! The rich scent of mash drifts through the lane.", "Hail to the tapster! Tapping a fresh barrel this morning?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:brewer"],
            respond: None,
            lines: &["Morrow, {other}! Stirring the copper vat since before cockcrow.", "Well met! The yeast is working its miracle in the vats.", "Greetings! A thirsty throat finds a ready welcome here.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(BREWER_MASH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAVERNER_RECKONING: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:taverner"],
    calls: &["A man's reckoning on the slate grows longer than his memory of it.", "Two measures on the slate and a third if {2:subj} {2:sing} for it.", "The bench by the fire is the warmest seat in the parish, and it costs nothing.", "I hear every quarrel in this village twice - once drunk and once sober."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Chalk it up and I will square it at Michaelmas. Fare well.", "{2:subj} {2:keep} an honest house, taverner. Good day.", "Keep the bench warm; I will be back before dark.", "May {2:poss} slate be short and {2:poss} barrel long. Farewell.", "God's peace on {2:poss} threshold, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAVERNER_BARREL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:taverner"],
    calls: &["What is on tap tonight - the alewife's small ale or the brewer's October?", "{2:did} {2:subj} lay in a firkin off the vintner before the frost?", "Is there a bed to be had, or must I lie in the straw with the carters?", "What does a measure cost a man who has walked twelve miles for it?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:taverner"],
            respond: None,
            lines: &["Market day fills the bench by noon and empties the barrel by dusk.", "The market crowd drinks double what the village does - I've two firkins standing by.", "Every stall-holder drinks on my slate today; I'll be chalking till dark."],
            voice: &[],
            then: Some(TAVERNER_RECKONING), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:taverner"],
            respond: None,
            lines: &["Small ale from the alehouse this morning, and it will be sour by Friday - drink it now.", "I bought the brewer out at three farthings the measure and I pour it at a penny. That is the whole of my trade.", "There is a tick of clean straw above the kitchen and a bolt on the door. Sixpence.", "Four pence, and the fire is free, and so is everything {2:subj} {2:will} hear beside it."],
            voice: &[],
            then: Some(TAVERNER_RECKONING), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAVERNER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:taverner"],
    calls: &["Good morrow, taverner! The ale-stake is out, so there is something in the barrel.", "Well met, {other}! A pint of the good dark, if the barrel holds.", "Hail the keeper of the one table in this village. Is the fire lit?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:taverner"],
            respond: None,
            lines: &["Morrow, {other}! Sit where {2:subj} {2:like}, the far end is dry.", "Well met! I brew nothing and I sell everything - come in.", "Greetings! A thirsty throat and a civil tongue are all this house asks.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(TAVERNER_BARREL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_SWARM: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:beekeeper"],
    calls: &["Sweet yellow wax makes the church tapers that burn without smoke.", "A swarm in May is worth a load of hay.", "Bees love a quiet tongue and a gentle step among the skeps.", "The wax is sweet and the honey flows clear."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} hives thrive through the winter, beekeeper. Fair day.", "I will buy a jar of comb at Michaelmas. Farewell.", "The little workers teach us all to labor together. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_HONEY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:beekeeper"],
    calls: &["Did the swarm from the orchard settle in the new straw skep?", "Can I buy a pot of honey to soothe my child's raw throat?", "{2:have} {2:subj} cakes of beeswax ready for the shoemaker's thread?", "Are the worker bees flying strong to the heather hills?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:summer")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:beekeeper"],
            respond: None,
            lines: &["The skeps are roaring - a strong summer makes a rich comb.", "Honey's heavy this month; the meadow's loud enough to walk on.", "High summer's the harvest here - the combs are near too fat to lift."],
            voice: &[],
            then: Some(BEEKEEPER_SWARM), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:beekeeper"],
            respond: None,
            lines: &["Aye, we beat the brass pan and hived them safely under the hedge.", "Take this jar of dark clover honey; it mends a cough overnight.", "Three cakes of yellow wax, filtered clean and smelling of clover.", "Aye, a pot for a penny - it will soothe her before the week's out."],
            voice: &[],
            then: Some(BEEKEEPER_SWARM), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:beekeeper"],
    calls: &["Good morrow, beekeeper! The apiary hums like an organ in the sunshine.", "Well met, {other}! Straw skeps standing tidy in the garden rows.", "Hail, master of the hives! Smoking the bees this morning?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:beekeeper"],
            respond: None,
            lines: &["Morrow, {other}! The bees are flying thick to the heather hills.", "Well met! Sweet honey on the comb and wax for the monastery.", "Greetings! Walk soft, neighbor; the queen is in her prime.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(BEEKEEPER_HONEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_SALVE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:herb_wife"],
    calls: &["Dried elder flowers and mint leaves keep the ague away in winter.", "Yarrow leaves stop a blood-cut faster than cobwebs.", "The roots must be gathered in the dark of the moon for full virtue.", "Nature holds a remedy for every mortal ache."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:poss} knowledge has saved many a soul in this town. Fare well.", "I will brew the tea as {2:subj} {2:bid}. Good day.", "May the earth yield its medicine to {2:obj}. Farewell.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_DRAUGHT: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:herb_wife"],
    calls: &["{2:have} {2:subj} a salve for stiff knees and swollen knuckles?", "{2:can} {2:subj} brew a draught for the fever that shakes the child?", "What herb will keep the pests away from our flour bin?", "What root will soothe a racking cough in the chest?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:spring")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:herb_wife"],
            respond: None,
            lines: &["The first shoots are up - selfheal and sorrel, spring's earliest medicine.", "Spring brings the picking; the stillroom's hung with green already.", "The hedge gave me a basket of new growth this morning - winter's stores are near spent."],
            voice: &[],
            then: Some(HERB_SALVE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:herb_wife"],
            respond: None,
            lines: &["Rub this grease rendered with comfrey and marigold twice a day.", "Take this dried willow bark; steep it in hot water and give three sips.", "Hang dried tansy and wormwood above the bin; no weevil will touch it.", "Boil the willow bark in rain water and drink it warm."],
            voice: &[],
            then: Some(HERB_SALVE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:herb_wife"],
    calls: &["Good morrow, wise woman {other}! Bunches of herbs hang drying everywhere.", "Well met, herb-wife! {2:poss} basket is full of green leaves from the fen.", "Hail, healer of the sick! Gathering roots in the dew?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:herb_wife"],
            respond: None,
            lines: &["Morrow, {other}! The earth gives her secrets to those who look.", "Well met! Stinging nettle and plantain leaves gathered at sunrise.", "Greetings, neighbor! What ails {2:obj} or {2:poss} folk today?", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(HERB_DRAUGHT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_BARTER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:merchant"],
    calls: &["A good trade leaves both men smiling when hands are struck.", "Silver coin or salted hams - sound goods are always welcome.", "The tally keeps honest count, and honest count brings folk back.", "The scales do not lie - a scruple light and the whole village remembers."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair trade at the counter, trader. Good day.", "May {2:poss} shelves never stand empty of stock.", "Until the spring fair, merchant. Fare well.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_WARES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:merchant"],
    calls: &["{2:do} {2:subj} have steel needles and linen thread on {2:poss} shelf?", "What price for a pound of good salt this week?", "{2:have} {2:subj} ribbons or a pinch of spice left from the fair?", "What exchange {2:do} {2:subj} take for a bag of salt?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:merchant"],
            respond: None,
            lines: &["Fine iron needles in a bone case; two eggs or a penny each.", "Good salt by the measure; three groats and it's {2:own}.", "Ribbon off the fair cart, and a pinch of pepper to spare.", "Two silver pennies or a salted ham buys the measure."],
            voice: &[],
            then: Some(MERCHANT_BARTER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:merchant"],
    calls: &["Good morrow, trader! What does the stall hold today?", "Well met, {addr}! What sits behind the counter this week?", "Fair day to the stall-keeper! How stands the tally this season?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:merchant"],
            respond: None,
            lines: &["Morrow, good folk! Stock from the town fair and goods from the parish.", "Well met! Lifting the counter - step up and see the stock.", "Greetings! Dust on the shelves, but fair goods behind the counter.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(MERCHANT_WARES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEDGER_ACCOUNTS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:merchant"],
    calls: &["A bushel on credit is a bushel at interest, mark me.", "The ledger forgets nothing - kindness least of all.", "Coin talks where the tongue falls short, {other}.", "The price of wool has climbed a penny since the spring fair."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:merchant"],
            respond: None,
            lines: &["Aye, and debt is a chain that rattles at night.", "{2:subj} {2:speak} the trader's own gospel.", "True enough - a debt unpaid is a friend unmade."],
            voice: &[],
            then: Some(MERCHANT_BARTER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair profit to {2:poss} ledger, merchant. Good day.", "I'll mind my own accounts, thank {2:obj} kindly.", "Until the next fair, trader."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEDGER_ROADS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:merchant"],
    calls: &["What news of the wool price in the towns, {other}?", "{2:know} {2:subj} a steady hand for the market day hauling?", "What exchange does the village offer for good salt this season?", "{2:have} {2:subj} goods to trade, or only gossip?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:merchant"],
            respond: None,
            lines: &["The fleece went for three pence the stone at the last market.", "My cousin's boy drives a steady cart for a fair wage.", "Salt fetches well here; the winter salting never fails to call for it.", "Barley and honest talk, trader - that is our stock in trade."],
            voice: &[],
            then: Some(LEDGER_ACCOUNTS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, trader. Fare well.", "My purse is thin - another market, perhaps.", "Until next time, merchant."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEDGER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:merchant"],
    calls: &["Morrow, {other} - the ledger never sleeps, eh?", "Fair day! Prices move like the river these weeks.", "Well met, {other} - trade is brisk when the roads are kind.", "Greetings! Every handshake's a tally waiting to be struck."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:merchant"],
            respond: None,
            lines: &["And fair coin to {2:poss} purse, merchant.", "Well met, trader - the village keeps its tally with {2:obj}.", "Morrow - what does market day ask of us this season?"],
            voice: &[],
            then: Some(LEDGER_ROADS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, merchant. Fare well.", "Chores before commerce, I'm afraid.", "Until next time, trader."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_PETITION: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:rents"],
    calls: &["The manor must maintain the bridge if the peasants are to bring in the harvest.", "A good lord shears his sheep but does not flay the skin.", "The court rolls show every customary right since the grandfather's time.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["We will see what the manor court decides on Tuesday. Good day.", "May justice be remembered at the high table. Fare well.", "I take my leave of the manor gate. Fair day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_LEASES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:rents"],
    calls: &["Will the rent in silver be abated because the hail ruined the oats?", "Must our cottage provide three days of boon-ploughing this season?", "Will the bailiff hear the widow's plea for her late husband's strip?", "Will the lord's bailiff hear our plea concerning the hail damage?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:rents"],
            respond: None,
            lines: &["The steward will inspect the damaged fields before the tally is set.", "Two days of plough-service are due by custom, as the parchment records.", "The court sits under the great oak on Thursday morning; be present.", "The bailiff hears all pleas when the court sits under the oak."],
            voice: &[],
            then: Some(LORD_PETITION), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:rents"],
    calls: &["Good morrow to the manor's steward. We come on village business.", "Well met, sir. We seek speech concerning the leases and tallies.", "Greetings, lord's man. The tenants have gathered to speak.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["concern:rents"],
            respond: None,
            lines: &["Morrow, tenants. The tally sticks are on the table; state {2:poss} grievance.", "Well met, villagers. The lord listens to lawful petitions.", "Speak {2:poss} mind plainly, goodmen; the court roll is open.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(LORD_LEASES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEVEE_SEASON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:rents", "concern:crops"],
    calls: &["The steward keeps count of every sheaf from {2:poss} strip, goodman.", "My bailiff tells me the common ditch wants scouring before the rains.", "A good tenant renders his boon-days before the tally is read.", "The deer have been at the lower field again - keep {2:poss} dogs close."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:rents"],
            respond: None,
            lines: &["I'll see to it before the week's end, {addr}.", "As {2:subj} {2:say}, {addr}.", "The ditch will be cleared by the quarter day, {addr}."],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.30, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:rents"],
            respond: None,
            lines: &["The rents bite deeper than the frost, begging {addr}'s pardon.", "Custom says two days, {addr} - not three.", "My hands are cracked enough without fresh burdens, {addr}."],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:poss} servant, {addr}. Good day.", "I must be about the fields, {addr}.", "God keep {2:obj} and {2:poss} house, {addr}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEVEE_LAW: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:rents"],
    calls: &["The manor court sits under the oak on Thursday - bring {2:poss} witnesses.", "Custom binds us both, goodman - {2:poss} strip, my tallage.", "The rolls are kept fair; what {2:poss} father owed, {2:subj} {2:owe}, no more.", "A lord's word is the law of the lane, goodman."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll stand at the oak on Thursday, {addr}.", "Custom holds, {addr} - my strip, {2:poss} tallage.", "{2:poss} servant, {addr}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEVEE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:rents"],
    calls: &["Aye, goodman - out with the dawn, I see.", "Morrow, {other}. {2:poss} hedge wants mending on the north side.", "Well met, tenant - the manor hears its people."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["concern:rents"],
            respond: None,
            lines: &["{addr}. We came to be heard.", "Aye, {addr} - good morrow to the manor.", "At {2:poss} service, {addr}."],
            voice: &[],
            then: Some(LEVEE_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Question, concepts: &["concern:rents", "aid"],
            respond: None,
            lines: &["{addr} - might the hail damage weigh on the tallage?", "A word, {addr} - the oats failed on the lower strip.", "Will the court hear our plea about the common ditch, {addr}?", "Will the court grant us gleaning after the lord's own harvest, {addr}?", "A word, {addr} - the mill queue starves us; might a second stone be licensed?", "{addr} - may my boy take dead wood from the manor copse this winter?", "Will the manor take our rent in rye this year, {addr}? The silver's short.", "A word, {addr} - the boon-works fall on haymaking week."],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.30, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:poss} servant, {addr}.", "The manor's hour is short, {addr}.", "God keep {2:obj}, {addr}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_NETS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:fisher"],
    calls: &["Trout leap in the shallows when the evening mayflies hatch.", "Wicker eel pots set in the weir reeds yield a fat catch by dawn.", "A fisherman knows every deep hole and eddy between here and the bridge.", "Patience brings more trout than a hasty net."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} nets come up heavy with silver fish. Fair day.", "Mind the river current by the mill dam. Farewell.", "I will bring an eel pie to the festival. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_CURRENT: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:fisher"],
    calls: &["Is the river water clear or muddy after the hill rains?", "Did the fish trap take any pike in the reed bed today?", "Can I buy three fresh trout for the Friday supper?", "Did the willow fish-pots take any eels at the weir?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:fisher"],
            respond: None,
            lines: &["The river's swollen and the trout run - rain makes a fisher's morning.", "The weir's roaring and the eel-pots ride low; a wet day's a fat one.", "Rain on the water hides the fisher from the fish - best day's work there is."],
            voice: &[],
            then: Some(FISHER_NETS), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:fisher"],
            respond: None,
            lines: &["The current runs green and sweet; the fish bite eager.", "A stout pike and two dozen silver dace lie in the willow creel.", "Take these three trout fresh from the river; three pennies or a loaf.", "Three fat river trout and an eel fresh from the wicker trap."],
            voice: &[],
            then: Some(FISHER_NETS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:fisher"],
    calls: &["Good morrow, riverman! {2:poss} boots are wet to the knee.", "Well met, {other}! Mending {2:poss} linen gill-nets on the bank?", "Hail, fisher of the weir! Any fish in the basket today?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:fisher"],
            respond: None,
            lines: &["Morrow, {other}! The river is awake and cold as ice.", "Well met! Tying knots and checking the willow osiers.", "Greetings! River water in my veins and fresh catch in the creel.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(FISHER_CURRENT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_COALS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:charcoal"],
    calls: &["Slow heat with no open flame turns dense oak into pure black coal.", "Black dust in the beard and lungs, but the smith's forge cannot live without us.", "A charcoal stack needs watching through three nights of vigil.", "Slow burning makes the charcoal hard and sound."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} stacks burn true without flaring. Fair day.", "I will tell the smith his coals are ready. Farewell.", "Watch the sparks in the dry brushwood, collier. Good day.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_SMOKE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:charcoal"],
    calls: &["Is the earth-covered mound ready to open and rake?", "{2:can} {2:subj} spare a sack of charcoal for the tavern braziers?", "Did the night wind blow the turf cover off the kiln?", "How burns the earth-covered pit mound in the clearing?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:charcoal"],
            respond: None,
            lines: &["The blue smoke has turned clear; the coal is cooled and cured.", "Two sacks of hard black charcoal wait by the pit hut.", "The sods held fast; not a spark escaped into the trees.", "The mound is raked and two sacks of hard coal are cooled."],
            voice: &[],
            then: Some(CHARCOAL_COALS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:charcoal"],
    calls: &["Good morrow, charcoal burner! Smoke drifts thick from the deep wood.", "Well met, {other}! Black as an imp from tending the pits.", "Hail, master of the kiln! How burns the timber mound?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:charcoal"],
            respond: None,
            lines: &["Morrow, {other}! Watching the slow smoke day and night.", "Well met! Soot on my face, but clean fuel for the iron forge.", "Greetings! The mound is smoking sweet under its turf blanket.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(CHARCOAL_SMOKE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FARMER_TOLL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:farmer"],
    calls: &["A field's a ledger that only pays once a year.", "The lark's the only foreman a ploughman answers to.", "Stones never run out in a field; they only sink and rise again.", "A straight furrow is a prayer said with a plough."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} acres bear well, {other}. Good day.", "Fair day - back to the furrows with me.", "God speed {2:poss} hands to the harvest.", "Fare well, neighbor; the field won't wait.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FARMER_SOW: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:farmer"],
    calls: &["Is the seed corn in the ground, or still standing in the barn?", "How does the wheat come - thick or thin?", "{2:do} {2:subj} read the sky much over a field of beans?", "What does {2:poss} ground ask of {2:obj} today?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:spring")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:farmer"],
            respond: None,
            lines: &["The last of the seed went in the north strip this week; pray for a soft rain, not a washing one.", "Oats in the ground and the plough stood in the barn - spring's half spent and the year's half made.", "All in now. It grows or it doesn't; a farmer's spring is mostly waiting."],
            voice: &[],
            then: Some(FARMER_TOLL), end: None,
        },
        Reply {
            criteria: &[Has("self:status:season:autumn")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:farmer"],
            respond: None,
            lines: &["Stooks in the high field and half the wheat to carry yet - the harvest never lands all at once.", "The carts run till the light gives out; it's the good tired of the year.", "A third of the sheaves are in the barn. If the sky holds, we're made for the winter."],
            voice: &[],
            then: Some(FARMER_TOLL), end: None,
        },
        Reply {
            criteria: &[Has("self:status:selling")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:farmer"],
            respond: None,
            lines: &["The mare's hitched and the sacks are up - my stall's by the north side of the cross today.", "A cart of barley's my price today, if the miller's thumb keeps off the scale.", "I've grain and a few fat capons on the square; the field's saved the day for me."],
            voice: &[],
            then: Some(FARMER_TOLL), end: None,
        },
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:farmer"],
            respond: None,
            lines: &["The market crowd needs feeding same as the village - the field grows for both.", "Half the square's dinner walked or grew off my strips this year."],
            voice: &[],
            then: Some(FARMER_TOLL), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:farmer"],
            respond: None,
            lines: &["The field's half ploughed and the share wants a smith's hand before dark.", "Beans on the low strip, wheat on the high - same as my grandsire had it.", "The ground's kind this year, God willing it stays kind till harvest.", "Weeds out of the young corn this week; the hocke's a dull trade and a true one."],
            voice: &[],
            then: Some(FARMER_TOLL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FARMER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:farmer"],
    calls: &["Good morrow, {other}! The smell of the earth's on {2:poss} hands.", "Well met! Fresh from the furrows, or off to them?", "Hail, tiller of the soil! How do the strips stand this day?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:farmer"],
            respond: None,
            lines: &["Morrow, {other}! The plough's been out since first light.", "Well met! Two strips sown and the lark's up over the wheat field.", "God bless {2:poss} steps. The furrows run straight today.", "Well met - the muck's carted and the near field lies open for seed.", "A fair day to {2:obj}; the year's half made if the sky holds."],
            voice: &[],
            then: Some(FARMER_SOW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAILOR_FITTING: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:tailor"],
    calls: &["Cloth keeps its counsel until the shears speak first.", "A seam sewn straight holds; one sewn sullen gapes by Lent.", "The fitting's the fight; the sewing's the peace.", "Fustian for the field and wool for the frost - velvet's for towns."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} shears stay honest, tailor. Fare well.", "Good day - send word when the loom's next piece lands.", "God speed the needle and the hand that drives it.", "Keep the seams straight, friend. Farewell.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAILOR_SEAM: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:tailor"],
    calls: &["What garment lies on {2:poss} board today?", "Is the warden's gown cut yet, or still folded?", "{2:do} {2:subj} sew the festival banners this year?", "What does the needle ask of {2:obj} this week?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:selling")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:tailor"],
            respond: None,
            lines: &["Stockings and a ribbon or two on my stall - the bespoke work waits at home.", "Market day's the haggler's holiday; I sell the smalls and take the gown orders there.", "I've gloves and hose at the cross today - smalls for the crowd, fittings by appointment."],
            voice: &[],
            then: Some(TAILOR_FITTING), end: None,
        },
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:tailor"],
            respond: None,
            lines: &["The crowd's all seams and hems - a tailor reads a market like a pattern book.", "The fitting book fills on market day; the board work comes after."],
            voice: &[],
            then: Some(TAILOR_FITTING), end: None,
        },
        Reply {
            criteria: &[Has("self:status:season:winter")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:tailor"],
            respond: None,
            lines: &["Winter's my harvest - the whole village wants thicker hose before the frost bites.", "Every door wants a lined cloak this month; I sew till my fingers split.", "The cold's a tailor's friend; wool work doubles when the wind turns."],
            voice: &[],
            then: Some(TAILOR_FITTING), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:tailor"],
            respond: None,
            lines: &["A wedding gown for the weaver's girl - the seams are half-basted.", "Two pairs of hose and a livery jacket; the steward pays slow but he pays.", "Cloth is money lying folded - a bad cut costs a week's wage.", "The board's covered - a gown, a funeral cloak and mending besides."],
            voice: &[],
            then: Some(TAILOR_FITTING), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAILOR_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:tailor"],
    calls: &["Good morrow, tailor! Thread and shears keep {2:obj} busy?", "Well met, {other}! Cutting a gown today or sewing one?", "Hail! Half the village wears {2:poss} work on its back.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:tailor"],
            respond: None,
            lines: &["Morrow! Chalk down three lengths of fustian before breakfast.", "Well met - the shears are sharp and every yard's spoken for.", "God bless {2:poss} steps; the day's a hose and a half already.", "Well met, friend - the thimble's warm and the seam's running true.", "A fair morning - I'm basting sleeves till the light goes."],
            voice: &[],
            then: Some(TAILOR_SEAM), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LUTHIER_SONG: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:luthier"],
    calls: &["A lute is a slow-built thing that dies in a moment's damp.", "Every instrument remembers the hand that made it.", "The wood chooses its sound; the luthier only finds it.", "A cracked bell plate's a sad thing; a cracked spruce belly's a dead one."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} strings never snap in the frost. Fare well.", "Good day - when the fiddle's done, bring it to the feast.", "God speed the scraping and the singing, friend.", "Keep the strings true, {other}. Farewell.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LUTHIER_WOOD: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:luthier"],
    calls: &["What sings on {2:poss} bench today?", "Is the new lute strung, or does the wood still wait?", "Does the wood tell {2:obj} when it's ready?", "Where does {2:poss} spruce come from, if the asking's no offence?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:winter")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:luthier"],
            respond: None,
            lines: &["The shop stove keeps the glue-pot soft - a hard frost kills a joint as sure as a drop does.", "Winter's scraping weather; the wood sits tight and the glue holds true.", "Cold shop, warm stove - the instruments huddle close as the parish."],
            voice: &[],
            then: Some(LUTHIER_SONG), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-fair")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:luthier"],
            respond: None,
            lines: &["A dry day opens the soundboards - this air glues tight and rings true.", "Fair weather's a luthier's friend; the varnish'll cure by vespers.", "The wood's content today - dry boards cut clean."],
            voice: &[],
            then: Some(LUTHIER_SONG), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:luthier"],
            respond: None,
            lines: &["A rebec for the warden's minstrel, strung in the old tuning.", "The spruce was felled nine winters back; it was ready before the player was.", "Gut strings and an honest belly - the rest is patience.", "The fiddle's glued and the pegs are fit; it only wants its strings and a player."],
            voice: &[],
            then: Some(LUTHIER_SONG), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LUTHIER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:luthier"],
    calls: &["Good morrow, {other}! The sweet sawdust on {2:poss} sleeve says the bench is alive.", "Well met! Working a belly of spruce this morning?", "Hail! Has {2:poss} bench got a lute on it or a rebec?", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:luthier"],
            respond: None,
            lines: &["Morrow! Three soundboards scraped since cockcrow and a fourth gone thin.", "Well met - the gut strings are steeping and the varnish smells grand.", "God keep {2:obj}, neighbor; the wood's seasoning quiet today.", "Well met - a quiet morning for cutting pegs.", "A fair day to {2:obj}; the strings are steeping sweet."],
            voice: &[],
            then: Some(LUTHIER_WOOD), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRINTER_SHEET: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:printer"],
    calls: &["A press is a forge that prints words instead of horseshoes.", "Paper remembers every mistake the ink told it.", "The town's whole memory is a stack of drying sheets.", "A loose letter's a lost hour; a smeared sheet's a lost day."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} ink never smear a wedding notice. Fare well.", "Good day - send a sheet when the harvest tally's printed.", "God speed the bar and the arm that pulls it.", "Keep the letters true, {other}. Good day.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRINTER_FORME: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:printer"],
    calls: &["What leaves the press today?", "Is the type set for the warden's notice?", "Do the letters fight {2:obj}, or lie down meek?", "How many sheets before the light gives out?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:printer"],
            respond: None,
            lines: &["Market bills, a penny the pair - every stall wants its prices inked these days.", "The market handbills went off with the crier this morning.", "Half the square's signage was on my press yesterday; market day's the printer's Sunday."],
            voice: &[],
            then: Some(PRINTER_SHEET), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:printer"],
            respond: None,
            lines: &["Damp air swells the paper; today's sheets will dry crooked whatever I do.", "The rain's in the paper and the ink runs - a wet day's a wasted pull.", "Wet day's wasted work; the paper won't take ink square till it dries."],
            voice: &[],
            then: Some(PRINTER_SHEET), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:printer"],
            respond: None,
            lines: &["A cart's worth of almanacs bound for the next town, God keep the carrier.", "The forme's tight for once - the letters lie like sheep in a fold.", "Ink, letters, a strong arm on the bar - the rest is patience and swearing.", "A hundred sheets of the warden's edict; the spelling's his own."],
            voice: &[],
            then: Some(PRINTER_SHEET), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRINTER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:printer"],
    calls: &["Good morrow, printer! The ink's up to {2:poss} elbows again.", "Well met, {other}! What's on the press today - letters or leaflets?", "Hail! The smell of lampblack means fresh sheets.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:printer"],
            respond: None,
            lines: &["Morrow! The press ran before the bells - a whole alphabet backwards till it lands right.", "Well met - the type's set for the market notice, bar two letters I've had to whittle.", "God bless {2:poss} steps; the ink-pot's never empty nor the day long enough.", "Well met - the forme's tight and the paper's square for once.", "A fair morning; the shop already smells of lampblack."],
            voice: &[],
            then: Some(PRINTER_FORME), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GLAZIER_PANES: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:glazier"],
    calls: &["A window is a wall that learned to pray.", "Glass takes the weather's side; it only puts a face on it.", "Lead is the thread and light is the cloth.", "A sound pane's invisible; every crack's a signature."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} panes outlast the storm. Fare well.", "Good day - when the chancel's done, tell the priest to keep a candle by it.", "God speed the solder and the steady hand.", "Keep the light coming, glazier. Good day.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GLAZIER_LEAD: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:glazier"],
    calls: &["Is the church window all in, or still leaded halfway?", "What glass is on the bench today?", "Do the quarrels cut clean, or bite back?", "{2:do} {2:subj} take small work - a cracked dairy pane?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[Has("self:status:season:winter")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:glazier"],
            respond: None,
            lines: &["Half the village wants a shutter turned to glass before the cold - I can't cut quarrels fast enough.", "Winter's the glazier's harvest; every pane's a prayer against the wind.", "The frost brings the work to me - every crack in the parish wants leading."],
            voice: &[],
            then: Some(GLAZIER_PANES), end: None,
        },
        Reply {
            criteria: &[Has("self:status:wx-rain")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:glazier"],
            respond: None,
            lines: &["A wet pane shows every crack I left; the rain's my strictest customer.", "This rain'll find the chapel's east seam; I'll be up the ladder when it clears.", "The leaks report themselves today; rain's a better inspector than the warden."],
            voice: &[],
            then: Some(GLAZIER_PANES), end: None,
        },
        Reply {
            criteria: &[Has("self:status:selling")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:glazier"],
            respond: None,
            lines: &["Lanterns and small bottles at the stall - the windows keep the door, the glassware pays the bread.", "I've vials and a small mirror or two on the stall today.", "The stall's out - glassware for the crowd, leaded work by the eye."],
            voice: &[],
            then: Some(GLAZIER_PANES), end: None,
        },
        Reply {
            criteria: &[Has("self:status:market")], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:glazier"],
            respond: None,
            lines: &["Every lantern on the square was somebody's furnace day.", "Market folk look at glass the way the lord looks at land."],
            voice: &[],
            then: Some(GLAZIER_PANES), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:glazier"],
            respond: None,
            lines: &["The east window's rose is half-leaded; the rest's still flour lines on the tracing board.", "Green glass for the warden's study - he wants the light but not the weather.", "Glass breaks honest; a bad solder sneaks loose by Candlemas.", "A roundel for the tavern's new room, and three small quarrels for the dairy."],
            voice: &[],
            then: Some(GLAZIER_PANES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GLAZIER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:glazier"],
    calls: &["Good morrow, glazier! There's lead dust on {2:poss} cuffs.", "Well met, {other}! Cutting quarrels today or soldering cames?", "Hail! The church windows keep {2:obj} fed, I'll wager.", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:glazier"],
            respond: None,
            lines: &["Morrow! Two panes set in the chancel and a third broke on the bench - glass is glass.", "Well met - the leads are soft and the glass is hard; my whole day's between them.", "God keep {2:obj}, friend; the light comes through my shop crooked but it comes.", "Well met - tracing a rose for the east window till the light goes.", "A fair day to {2:obj}; the solder pot's hot already."],
            voice: &[],
            then: Some(GLAZIER_LEAD), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

// The trades speaking for themselves (#1617): the vocation-holder is
// the CALLER here, so these ride talks they lead - the player's Talk
// included. Each one opens with what the work's actually doing (a
// day's account, never an "I am a smith" explainer), and the two-crit
// variants below key whole trees to the market and the season.

const PLOUGH_SEASON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:farmer"],
    calls: &["Every year teaches a field something new and a farmer something old.", "The harvest's a promise the weather signs or doesn't.", "A good year's made in March and carried in September.", "The lark knows the field better than the ploughman ever will."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May the year's weather take {2:poss} side, farmer. Good day.", "Fair day - the village eats well by {2:poss} hands.", "God speed {2:poss} plough and the beast that pulls it.", "Fare well, {other}; give the low strip my sympathy."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PLOUGH_FIELDS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:farmer"],
    calls: &["A third of the crop's the weather's doing; the rest's sweat and stubbornness.", "The low strip drowns every March and grows sweet every June - land's like people, it has its moods.", "I've wheat on the high strip that'd shame the manor's own, and a ditch on the low that'd shame me.", "The ground don't lie: what's sown in temper comes up thin."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:farmer"],
            respond: None,
            lines: &["Aye, the land keeps honest accounts.", "True said - the furrow tells on the farmer.", "That's the truth of it; the field remembers every kindness.", "Well said - the earth has a long memory."],
            voice: &[],
            then: Some(PLOUGH_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PLOUGH_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:farmer"],
    calls: &["Morrow, {other} - the north strip took the last of the seed this week.", "Well met! The plough's been walking since first light.", "Fair day! The furrows run straight and the lark's up early.", "Morrow - the field's quieter company than the square, but it answers better."],
    voice: &[Voice { criteria: &[Has("self:status:season:spring")], lines: &["Morrow, {other} - the last seed went in the north strip this week; pray a soft rain, not a washing one.", "Well met! Spring's half spent and the year's half made.", "Fair day - the lambs are up on the commons and my oats are in the ground.", "Morrow - the field's all in; a farmer's spring is mostly waiting."] }],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:farmer"],
            respond: None,
            lines: &["And a straight furrow to {2:poss} plough, {other}.", "Morrow - the whole village eats what {2:poss} field grows.", "Well met - hard luck to {2:poss} weeds, good luck to {2:poss} corn.", "A fair day to the field's own man."],
            voice: &[],
            then: Some(PLOUGH_FIELDS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const OVEN_CRUST: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:baker"],
    calls: &["A loaf's weight is a baker's word.", "The oven asks for wood; the wood asks for January's felling.", "Half the village's quarrels start in a queue for bread.", "The hungry smell bread from three streets away."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} oven never run cold. Fare well.", "Good day - save me a loaf if the queue's kind.", "God speed {2:poss} loaves and the hands that shape them.", "Keep the bake sweet, baker. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const OVEN_BATCH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:baker"],
    calls: &["The dough rose slow today - the weather was in it, like everything else.", "A bake's a small miracle: flour, water, salt, and four hours of faith.", "The crust tells the truth about the fire every single day.", "Burnt one tray this morning - the ash got in with the kindling."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:baker"],
            respond: None,
            lines: &["The village rises when {2:poss} oven does.", "A bad bake's a hard day for all of us - but today's smelled fine.", "True - the crust never flatters.", "Aye - even the priest can't hide a burnt loaf."],
            voice: &[],
            then: Some(OVEN_CRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const OVEN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:baker"],
    calls: &["Morrow, {other} - the oven's been roaring since the small hours.", "Well met! The batch is out and the second's rising.", "Fair day! The whole lane smells of my oven this morning.", "Morrow - flour to the elbows and loaves on every shelf."],
    voice: &[
        Voice { criteria: &[Has("self:status:selling")], lines: &["Morrow, {other} - the stall's out by the cross and the loaves went before the second bell.", "Well met! I've sold more than I've said good-morrow to this morning."] },
        Voice { criteria: &[Has("self:status:market")], lines: &["Well met! Market day's the oven's Sunday - the batch never rests.", "Morrow - the square's already smelling the morning's bake."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:baker"],
            respond: None,
            lines: &["And a hot oven to {2:obj}, baker.", "Morrow - {2:poss} loaves carry the village's mornings.", "Well met - nothing smells like a fresh bakehouse.", "A fair day to the one who feeds us all."],
            voice: &[],
            then: Some(OVEN_BATCH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHISEL_QUARRY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:mason"],
    calls: &["Stone keeps its own time; the mason only borrows it.", "A chipped block teaches more than a whole wall of luck.", "The quarry gives, the frost takes - the trade's a treaty between.", "Every village was a pile of stones once; a mason made it an address."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} plumb line never lie. Fare well.", "Good day - the wall'll stand long past us both.", "God speed the mallet and the sure eye.", "Keep the walls true, mason. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHISEL_STONE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:mason"],
    calls: &["The church porch took three loads from the quarry this week; the stone kept the hill's own face.", "A wall's a long argument between the mason and gravity - today the wall's winning.", "The lime's still hot in the pit; mortar's a week away from ready.", "Two cottars' chimneys were smoking backward - I've their flues to reset."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:mason"],
            respond: None,
            lines: &["A wall well built outlives the quarrel it was raised in.", "True - the stone remembers every lazy blow.", "Well said - even the church took a mason's patience first.", "Aye, and a straight wall's a straighter conscience."],
            voice: &[],
            then: Some(CHISEL_QUARRY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHISEL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:mason"],
    calls: &["Morrow, {other} - the yard's got a wall half-risen and a chisel half-blunt.", "Well met! The stone's talking back today.", "Fair day - the church porch keeps me in work, God be thanked.", "Morrow - the mortar's mixed and the blocks lie straight on the ground."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:mason"],
            respond: None,
            lines: &["And a true line to {2:poss} level, mason.", "Morrow - half the village stands on {2:poss} work.", "Well met - stone's a stubborn trade and an honest one.", "A fair day to the man who builds what outlasts us."],
            voice: &[],
            then: Some(CHISEL_STONE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEEDLE_FITTING: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:tailor"],
    calls: &["A gown's a promise that hangs on a shoulder.", "The shears only lie once; the fitting tells it every time.", "A tailor's mirror and a tailor's tongue must both tell the truth.", "The village wears its story on its sleeves - I only sew the sleeves."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} needle keep its temper. Fare well.", "Good day - send word when the banner's sewn.", "God speed the shears and the steady wrist.", "Keep us dressed, tailor. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEEDLE_SEAM: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:tailor"],
    calls: &["The warden's gown took its first fitting yesterday - the man stands like a clothes-press.", "A wedding gown's half done for the weaver's girl; the lace came down the road Tuesday.", "I've mending enough to outlast the week - every door in the lane sends a sleeve.", "Cloth's dear this season; a tailor counts threads like a steward counts pennies."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:tailor"],
            respond: None,
            lines: &["A straight seam's a small miracle, says I.", "True - the village would look a sight in sackcloth.", "Well said - {2:poss} work shows in every procession.", "Aye, and a good fit's a quiet boast."],
            voice: &[],
            then: Some(NEEDLE_FITTING), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEEDLE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:tailor"],
    calls: &["Morrow, {other} - chalk on my knuckles and three gowns cut already.", "Well met! The shears have been honest this morning.", "Fair day - the board's covered in fustian and promise.", "Morrow - a tailor's day starts before the seams are chalked."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:tailor"],
            respond: None,
            lines: &["And a true seam to {2:poss} needle, tailor.", "Morrow - {2:poss} stitches keep the whole village decent.", "Well met - a good tailor's worth two sermons.", "A fair day to the keeper of the village's back."],
            voice: &[],
            then: Some(NEEDLE_SEAM), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ANVIL_HEAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:smith"],
    calls: &["Iron remembers every blow; so does the arm that struck it.", "A cold forge is a smith's confession; mine hasn't confessed in years.", "The white heat forgives nothing - it shows the iron as it is.", "Temper the metal, not the man - though both get tested daily."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} welds hold fast, smith. Fare well.", "Good day - the hinges can wait a week for a good weld.", "God speed the hammer and the sure hand.", "Keep the forge hot, friend. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ANVIL_IRON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:smith"],
    calls: &["Three dozen nails, a coulter tip and a cracked hinge already - the day's half wrought.", "The charcoal burner's last sack runs low; the forge eats black gold.", "A ploughshare came in snapped clean - the field fought hard this spring.", "Every door in the lane owes me a hinge; some owe me two."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:smith"],
            respond: None,
            lines: &["The forge's work holds the village together, aye.", "True - every good tool's a smith's signature.", "Well said - iron keeps honest accounts too.", "Aye, and the anvil never flatters."],
            voice: &[],
            then: Some(ANVIL_HEAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ANVIL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:smith"],
    calls: &["Morrow, {other} - the coals were white before the bells.", "Well met! The anvil's been singing since first light.", "Fair day - the forge smells of scorched hoof and hot iron, as it should.", "Morrow - mind the sparks on {2:poss} way through."],
    voice: &[Voice { criteria: &[Has("self:status:market")], lines: &["Morrow, {other} - every cart in the square's shed a nail; I've a queue of mending past the door.", "Well met! The market's good for my trade and worse for my ears.", "Fair day - the coals ran white early; market days don't wait.", "Morrow - half the stall-holders owe me a hook or a hinge."] }],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:smith"],
            respond: None,
            lines: &["And white coals to {2:poss} hearth, smith.", "Morrow - the whole village hears {2:poss} hammer and takes heart.", "Well met - keep the iron honest, as ever.", "A fair day to the strongest arm in the parish."],
            voice: &[],
            then: Some(ANVIL_IRON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BELLY_STRING: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:luthier"],
    calls: &["The string sings what the wood allows - the luthier only introduces them.", "Every fiddle remembers the hand that carved it.", "Wood is patient work; strings are honest work; varnish is faith.", "A snapped string's a small grief; a cracked belly's a dead one."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} varnish cure true. Fare well.", "Good day - the feast'll sing for {2:poss} bench.", "God speed the knife and the listening ear.", "Keep the strings singing, luthier. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BELLY_WOOD: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:luthier"],
    calls: &["A rebec's half-carved for the warden's minstrel; the scroll came sweet off the knife.", "The spruce for the new lute was felled nine winters back - it was ready before its player was.", "I've a fiddle to re-glue and a harp to re-pin; both weep if the glue's rushed.", "The varnish stinks of pine resin and patience; a week more and it cures."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:luthier"],
            respond: None,
            lines: &["A good instrument outlives its maker's name.", "True - the wood waits longer than the player ever does.", "Well said - the feast's music has {2:poss} fingerprints on it.", "Aye, and a sweet belly's worth a sharp knife."],
            voice: &[],
            then: Some(BELLY_STRING), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BELLY_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:luthier"],
    calls: &["Morrow, {other} - the spruce shavings are knee-deep and the varnish pot's warm.", "Well met! A new belly's been scraped since cockcrow.", "Fair day - the workshop's quiet as a chapel and twice as patient.", "Morrow - gut strings steeping, wood seasoning, all in good order."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:luthier"],
            respond: None,
            lines: &["And a true tone to {2:poss} strings, luthier.", "Morrow - the village's music starts on {2:poss} bench.", "Well met - a maker of instruments makes the feast itself.", "A fair day to the man who teaches wood to sing."],
            voice: &[],
            then: Some(BELLY_WOOD), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRESS_SHEET: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:printer"],
    calls: &["A press is a forge that prints words instead of horseshoes.", "Paper remembers every mistake the ink told it.", "The town's whole memory is a stack of drying sheets.", "A loose letter's a lost hour; a smeared sheet's a lost day."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} type never whittle crooked. Fare well.", "Good day - keep the sheets honest, printer.", "God speed the bar and the patient arm.", "Keep the town readable, {other}. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRESS_FORME: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:printer"],
    calls: &["The warden's edict sheets are half-pulled; the spelling's his own and so's the blame.", "A forme of almanacs for the next town's carrier - the letters fought me all morning.", "Two sorts broke in the chase this week; a whittled letter holds the line meanwhile.", "The paper's damp from last night's rain; every sheet'll dry crooked, complain as they may."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:printer"],
            respond: None,
            lines: &["A sheet printed true's worth a week of talk.", "True - the town reads what {2:poss} press allows.", "Well said - ink's a longer memory than gossip.", "Aye, and a straight column's a straight story."],
            voice: &[],
            then: Some(PRESS_SHEET), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const PRESS_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:printer"],
    calls: &["Morrow, {other} - the press ran before the bells and the ink's already arguing.", "Well met! A hundred letters backwards on the forme, all waiting to land right.", "Fair day - the lampblack's fresh and the paper's square.", "Morrow - the type's set, the bar's pulled twice, and the day's begun."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:printer"],
            respond: None,
            lines: &["And a clean pull to {2:poss} bar, printer.", "Morrow - the town's memory starts in {2:poss} shop.", "Well met - a press is a sermon {2:subj} {2:can} read twice.", "A fair day to the man who inks the truth."],
            voice: &[],
            then: Some(PRESS_FORME), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CAME_LIGHT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:glazier"],
    calls: &["A window is a wall that learned to pray.", "Glass takes the weather's side; it only puts a face on it.", "Lead is the thread and light is the cloth.", "Every pane's a treaty between the house and the sky."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May {2:poss} solder hold through the storms. Fare well.", "Good day - keep the light coming in, glazier.", "God speed the cutter and the steady hand.", "Keep the windows true, {other}. Fare well."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CAME_GLASS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:glazier"],
    calls: &["The east window's rose took its cames yesterday; the light comes through it red as wine.", "Three quarrels and a roundel went out this week - small work keeps the bread honest.", "A storm took the chapel's north pane last frost; the leads held, the glass didn't.", "The warden wants green glass for his study - light, he says, but not weather."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["trade:glazier"],
            respond: None,
            lines: &["A good pane's invisible until it cracks - like most honest work.", "True - the village prays through {2:poss} glass.", "Well said - even the rain reads better through a window.", "Aye, and the chancel'll shine for it."],
            voice: &[],
            then: Some(CAME_LIGHT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CAME_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:glazier"],
    calls: &["Morrow, {other} - two panes set this morning and a third broke honest.", "Well met! The cames are soft and the glass is hard, and the day's between them.", "Fair day - the chancel's rose is half-leaded and half-hoped.", "Morrow - the tracing board's flour-fresh and the cutter's sharp."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:glazier"],
            respond: None,
            lines: &["And clear panes to {2:poss} windows, glazier.", "Morrow - {2:poss} glass is how the whole village reads the sky.", "Well met - a window's the best sermon, and {2:subj} {2:preach} it.", "A fair day to the man who frames the light."],
            voice: &[],
            then: Some(CAME_GLASS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_BLESS: Exchange = Exchange {
    act: Act::Agree,
    concepts: &["elder"],
    calls: &["{2:poss} words are gold to my ears, elder. My thanks.", "We will heed {2:poss} counsel at the village meeting, elder."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Lords come and go, but the plough and the furrow remain. Fair day, {other}.", "Listen to the land, {other}; it was here before us and will keep our graves.", "Kindness to {2:poss} neighbor is the best hedge against misfortune. Good day.", "Seasons cycle and return as they have always done. May the road be kind under {2:poss} boots.", "May God grant {2:obj} peace in {2:poss} old age. Farewell.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

// The caller-side mirror of ELDER_GREET/WINTERS/BLESS (#1693): the
// player Talk replay always seats the villager as caller (self), so
// an elder villager never reaches the listener-side `other:status:elder`
// tree above. These three carry the same lines, calls and replies
// swapped, for when the elder is the one who opens the exchange -
// ambient elder-initiated chats and every player Talk with an elder.
const ELDER_BLESS_SELF: Exchange = Exchange {
    act: Act::Farewell,
    concepts: &["elder"],
    calls: &["Lords may come and lords may go, but the plough and the furrow remain, {other}.", "Mark the land well; it stood before us, and it will keep our graves after.", "Kindness to {2:poss} neighbor is still the best hedge against hard fortune.", "The seasons turn and return as ever they have - go on now, with my blessing.", "May the Lord grant {2:obj} peace in {2:poss} own old age, when it comes.", "God's peace rest upon {2:poss} cottage, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["{2:poss} words are worth more than gold to me, elder. My thanks, and fair day.", "We shall carry {2:poss} counsel to the village meeting, and gladly, elder."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_WINTERS_SELF: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["elder"],
    calls: &["Aye, acorns and thin porridge saw us through the great hunger, and we lived on.", "The old boundary ran clean from the standing oak down to the weeping willow.", "When dandelions bloom along the ditch bank, {2:subj} {2:may} trust the frost has broken.", "Patience and honest toil - that's seen near eighty winters come and go for me."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["elder"],
            respond: None,
            lines: &["{2:do} {2:subj} truly recall the great famine, fifty winters gone, grandfather?", "Where did the old boundary run, before ever the ditch was dug?", "What sign tells {2:obj} the frost will finally break?", "{2:do} {2:subj} recall how the great freeze finally broke, forty years back?"],
            voice: &[],
            then: Some(ELDER_BLESS_SELF), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_GREET_SELF: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "elder"],
    calls: &["Morrow, young soul - good to feel the sun on these old bones.", "Well met, little one! Heaven grants me yet another day to mind the fields.", "Ah, good day - my step is slow, but my memory still runs clear and long.", "God keep {2:poss} road safe this day, {other}.", "Well met indeed, friend - a fine sky we share this morning."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Morrow there, {other} - slow of step, but these old ears still catch everything of {2:obj}.", "Well met, little one - I know {2:poss} name and all {2:poss} doings, and gladly."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, young one - our family's roots run straight through {2:obj}.", "Bless {2:obj}, child of this house. Sit a while and tell me its news."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, wanderer. These grey ears still take to new voices well enough.", "Well met, traveler - rest here, if the road's run long."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, {other} - I've outlasted better rivals than {2:obj} in my day.", "Morrow, {other}. These long years teach patience, even toward {2:obj}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["elder"],
            respond: None,
            lines: &["A fair morrow to {2:obj}, venerable {other}.", "Well met, grandfather - sat in the sun to ease {2:poss} old joints?", "Greetings, mother of this village. How does {2:poss} health hold up today?", "A peaceful morrow to {2:obj}, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["A fair morrow, {other} - eighty winters gone, and still the sharpest tongue on this lane.", "Well met, old friend! Still warming those joints in the sun, I see."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - the oldest lamp of our family, burning on still.", "Grandmother! I do hope {2:poss} sunny doorway's kept a seat for me."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["A morrow to {2:obj}, elder - the road gives grey hair its due.", "A fine morrow to {2:obj}, mother of this village."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["A fair morrow to {2:obj}, venerable {other}.", "Well met, elder - how does {2:poss} health hold up?"] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Morrow, {other} - eighty winters gone, and {2:subj} still {2:rise} to spite me yet.", "Well met, elder. Even rivals must bow to the winters on {2:poss} head."] },
            ],
            then: Some(ELDER_WINTERS_SELF), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on, {other}. Fare well.", "Chores call; good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_WINTERS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["elder"],
    calls: &["{2:do} {2:subj} recall the great famine fifty winters ago, grandfather?", "Where did the old village boundary run before the ditch was dug?", "What signs tell us when the frost will finally break?", "{2:do} {2:subj} recall how the great freeze broke forty years ago?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["elder"],
            respond: None,
            lines: &["Aye, we ate wild acorns and shared what porridge there was, and survived.", "The boundary ran straight from the standing oak to the weeping willow.", "When the dandelions flower on the ditch bank, the frost is broken for good.", "Patience and honest toil have seen eighty winters come and go."],
            voice: &[],
            then: Some(ELDER_BLESS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "elder"],
    calls: &["Good morrow to {2:obj}, venerable {other}.", "Well met, grandfather! Sitting in the sunny doorway to warm {2:poss} joints?", "Greetings, mother of the village! How fares {2:poss} health today?", "A peaceful day to {2:obj}, {other}."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Good morrow, {other} - eighty winters and still the sharpest tongue on the lane.", "Well met, old friend! Still warming those joints in the sun?"] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - the family's oldest lamp still burning.", "Grandmother! {2:poss} sunny doorway kept a seat for me, I hope."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, elder. The road respects grey hair.", "A fair day to {2:obj}, mother of the village."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Good morrow to {2:obj}, venerable {other}.", "Well met, elder - how fares {2:poss} health?"] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Morrow, {other} - eighty winters and {2:subj} still {2:rise} to spite me.", "Well met, elder. Even rivals bow to the winters on {2:poss} head."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["elder"],
            respond: None,
            lines: &["Morrow, young soul. The sun feels good on eighty winters of bones.", "Well met, child! Another day Heaven grants to watch the fields.", "Greetings! My step is slow, but my memory is long and clear.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Morrow, {other} - my step is slow but my ears miss nothing, especially from {2:obj}.", "Well met, child - I know {2:poss} name and all {2:poss} doings, fondly."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, little one - the family's roots run through {2:obj}.", "Bless {2:obj}, kin. Come sit and tell me the house's news."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Grey ears still catch new voices.", "Well met, traveler - sit, if the road is long."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, {other} - I've buried better rivals than {2:obj}.", "Morrow, {other}. Age teaches patience, even with {2:obj}."] },
            ],
            then: Some(ELDER_WINTERS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["The sun climbs; come sit again soon, {other}."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Away then, {other} - my sun's too short for {2:obj}.", "Go on - an old woman's time isn't {2:own} to waste."] },
            ],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_GIFT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["family"],
    calls: &["May the little one grow tall as an ash and hardy as an oak.", "A baby's cry in the cottage banishes the winter gloom.", "The parish grows by another soul; blessed be the hearth.", "A united house can withstand any tempest."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["May the saints guard the cradle day and night. Fair day.", "I will bring goat's milk for the mother. Farewell.", "Joy to {2:poss} house, {other}! God bless the babe.", "May the road be kind under {2:poss} boots.", "God's peace be upon {2:poss} cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_JOY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["family"],
    calls: &["Is the little one sleeping peacefully in the wicker cradle?", "Has the parish priest set the day for the holy baptism?", "Does the mother have warm elderberry cordial to restore her strength?", "How fares our sister's household down by the green?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["family"],
            respond: None,
            lines: &["Aye, wrapped in soft lamb's wool, dreaming sweet dreams.", "On Sunday morning when the great church bell tolls.", "The neighbors brought honey broth and fresh eggs for her recovery.", "Our kin will stand beside {2:obj} through good season and bad."],
            voice: &[],
            then: Some(NEWBORN_GIFT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "family"],
    calls: &["Joy to {2:poss} household, {other}! I heard a newborn babe has arrived!", "Good morrow, new mother! How fares the precious child?", "Blessings on the cradle, {other}! What a joyous day for our lane!", "A peaceful day to {2:obj}, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            respond: None,
            lines: &["Morrow, {other}! A healthy babe, praise be to Heaven!", "Well met! Exhausted, but my heart is overflowing with gratitude.", "Greetings! A little girl with eyes bright as the morning sky.", "God bless {2:poss} steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[],
            then: Some(NEWBORN_JOY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "Until next time, neighbor."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MEET_FOOD: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["The communal grain store won't last through the freeze."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["My children go to bed with empty bellies and cold feet."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["We need to share what grain there is before rot sets in."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["A bushel of barley costs twice what it did at harvest."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["The flour bin is down to chaff and dust."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["If we do not ration the grain together, the weakest will starve."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["The miller takes too large a share while cottages go hungry."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["Wild roots and boiled leeks cannot sustain the whole winter."],
    },
];

const MEET_SICKNESS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["Half the north lane is coughing with burning fevers."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["Who will plough the spring furrow if we all lie sick?"],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["We should look after the sick together and bring them broth."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["The ague spreads from house to house in the cold fog."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["The herb-wife needs clean well water and willow bark for remedies."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["No family should be left alone to suffer when fever strikes."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["Keep the infected warm and let neighbors tend their cattle."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["We buried two elders this week; we must protect the living."],
    },
];

const MEET_WAGES: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["A full day's toil in the fields hardly buys half a day's bread."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["The miller's cut increases every single season."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["If we speak with one voice to the bailiff, they must listen."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["Our sweat builds the lord's barn, yet our own thatch leaks."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["Three pennies for a week of reaping is daylight robbery."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["Stand together on harvest morning, or take starvation wages alone."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["The customary wage of our fathers has been whittled to nothing."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["We ask for fair payment for an honest day's labor."],
    },
];

const MEET_ROADS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["The king's highway is riddled with deep ruts and broken stone."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["No pack merchant will bring salt if the bridge timbers rot."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["We could take turns watching the road and repairing the ditch."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["Wolves come close to the highway when darkness falls."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["A wagon broke an axle in the mud hole by the south boundary."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["Safe roads bring trade and silver into every cottage in town."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["If the parish does not mend the culverts, the autumn floods will wash the lane away."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["We need two stout watchmen with lanterns by the crossways."],
    },
];

const MEET_FAITH: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["We need faith and fellowship to hold our spirits upright."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["The church roof leaks over the altar; rain drips on the holy books."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["Let us keep faith with God and be true neighbors to each other."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["The bell tower timbers are weathered and need fresh oak beams."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["Sunday worship is the one hour the whole valley gathers in peace."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["A parish that forgets the poor has forgotten its holy duty."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["Let every house bring one candle for the chapel altar."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["When despair darkens the cottage, prayer and community bring light."],
    },
];

const MEET_RENTS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["The rents increase every Michaelmas without mercy or reason."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["I cannot pay in silver what the lord's steward demands."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["One tenant alone cannot argue with the manor court."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["If the hail ruined the barley, the rent in kind should be reduced."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["They demand extra boon-days when our own harvest is rotting in the rain."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["We hold our land by ancient custom, not by the whim of a greedy bailiff."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["Let us present a joint petition bearing every householder's mark."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["No man should be evicted from the hearth his grandfather built."],
    },
];

const MEET_FUEL: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["The peat bog is frozen solid and our dry brushwood is spent."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["Without firewood, the children shiver through the frosty nights."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["We should clear deadfall together in the common woods on Saturday."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["Cold hearths breed the sickness faster than foul water."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["Let those with seasoned oak share kindling with the widows."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["The bailiff threatens fines if we gather fallen boughs in the grove."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["A cottage without smoke from its chimney is a dying cottage."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["We need dry turf and split logs stored under cover before snow falls."],
    },
];

const MEET_GRIEF: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["We buried two souls this season; the churchyard grows crowded."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["The sorrow in our cottages weighs heavier than the taxes."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["A grieving house should be supported with food and wood by neighbors."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["Let the parish pray together for those who have gone before us."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["The tolling bell reminds every family that life is fragile."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["When a hearth is left cold by death, we must rekindle hope together."],
    },
];

const MEET_BRIDGE: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["The weir boards are cracked and the mill race threatens to overflow."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["The wooden footbridge shakes whenever an ox cart crosses."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["If the bridge collapses, our children cannot reach the chapel."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["We need three stout oak stringers to replace the rotten spans."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["The river bank is caving into the water by the washing steps."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["Every able-bodied hand must give a day's work to rebuild the pier."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["Stone piers and lime mortar would make a bridge to last for centuries."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:bridge"], bias: &[],
        lines: &["Let the carpenter cut the pegs and the village haul the timber."],
    },
];

const MEET_HARVEST: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["If the harvest is bountiful, the whole parish will thrive."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["We will need every scythe and sickle when the wheat turns gold."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["Let us bring the crops in together before the autumn tempests break."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["The gleanings of the fields must be left for the poorest widows."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["A shared threshing floor makes the work three times swifter."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["May the Lord bless the seed in the furrow and give us full sheaves."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["We must clean the communal granary before the new grain arrives."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["Celebration tastes sweeter when all have labored side by side."],
    },
];

const MEET_COMMONS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["The common pasture is overgrazed; we must stint the number of sheep."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["The ditch enclosing the common wood has been unlawfully fenced off."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["Our ancient right of turbary and gathering brushwood must be defended."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["Every cottager has a right to graze two cows on the village green."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["The bailiff's cattle are eating the sweet grass reserved for the poor."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["If we lose the commons, our way of life is broken forever."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["Let us walk the boundary markers on Rogation days as our fathers did."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["The common land belongs to the whole parish, not to a single estate."],
    },
];

const MEET_COMMON: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Agree, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"], bias: &[],
        lines: &["Aye, there is the truth of it.", "{other} has the right of it.", "It is so - my house has felt the same.", "Well spoken, neighbor; we stand with {2:obj}."],
    },
    MeetingLine {
        act: Act::PushBack, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"], bias: &[],
        lines: &["Talk is cheap - who will lift a hand to do aught about it?", "And what {2:would} {2:subj} have the village do, {other}?", "We have argued this before and accomplished nothing."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["If the harvest is sound, we will weather the winter.", "We'll need every willing pair of hands in the valley."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["Safe roads bring trade and bread into every cottage.", "The ditch by the bridge must be dug out."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:fuel"], bias: &[],
        lines: &["Cold stones freeze the spirit; we must gather kindling together.", "The common wood has brushwood enough for all if shared."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["The parish mourns with those who have lost kin.", "A neighbor's visit and warm broth bring comfort to an empty room."],
    },
];

/// The hog's own lines (#1252): whatever the matter, the answer is a
/// story about them. Every concern concept so they can take any floor
const MEET_SELF: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"],
        bias: &[(Trait::SelfAbsorbed, 2.0)],
        lines: &["This puts me in mind of my own case, which is not so different - and I'll tell it, if the floor allows.",
            "If I may - as one who has seen more of this than most, and borne more of it besides."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food", "concern:wages", "concern:harvest", "concern:commons"],
        bias: &[(Trait::SelfAbsorbed, 2.0)],
        lines: &["My own table's felt it, I promise {2:obj} - nobody here keeps a table like mine, for good or ill.",
            "When I was younger I'd have settled this myself and thought nothing of it. Ask me how."],
    },
];

/// The lines `speaker` brings to a house meeting: what they actually live with
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
    // the trade's own counsel, on its own ground (#1238)
    pool.extend(MEET_COUNSEL.iter().filter(|(t, _)| speaker.trade == Some(*t)).map(|(_, l)| l.clone()));
    pool.extend_from_slice(MEET_HARVEST);
    pool.extend_from_slice(MEET_COMMONS);
    pool.extend_from_slice(MEET_BRIDGE);
    pool.extend_from_slice(MEET_COMMON);
    // the hog's own material rides on top (#1252)
    if speaker.traits.contains(&Trait::SelfAbsorbed) { pool.extend_from_slice(MEET_SELF); }
    pool
}

/// A trade's authority on the matter it owns (#1238): the farmer on the
/// harvest, the mason on the bridge - words the other trades don't have
pub const MEET_COUNSEL: &[(&str, MeetingLine)] = &[
    ("farmer", MeetingLine { act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["I've cut thirty harvests - the field won't give what it hasn't got. Rest the low strip a year.",
            "I've walked that ground since I could hold a sickle. Sow it late, or the frost takes it."] }),
    ("farmer", MeetingLine { act: Act::Remark, concepts: &["concern:commons"], bias: &[],
        lines: &["My plough's turned every furrow by the common. Fence it and we all go short."] }),
    ("shepherd", MeetingLine { act: Act::Remark, concepts: &["concern:commons", "concern:harvest"], bias: &[],
        lines: &["I've kept the fold twenty winters. The flock needs the common grazing, or there's no wool and no milk.",
            "Ask the sheep - the hill grass is thin. Move them down in the wet months."] }),
    ("beekeeper", MeetingLine { act: Act::Remark, concepts: &["concern:harvest"], bias: &[],
        lines: &["The skeps tell the season before the fields do. Mark the clover - it's late this year."] }),
    ("baker", MeetingLine { act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["I've fired that oven every dawn for years. Give me the flour and I'll feed the street.",
            "Bread's only as good as the grain that comes to my door. Mend the mill and the loaves follow."] }),
    ("alewife", MeetingLine { act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["Small ale's fed this village longer than the church has stood. Spare the barley for the vat."] }),
    ("brewer", MeetingLine { act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["I know grain the way a priest knows psalms. Hold back a sack and brew it - it keeps when bread won't."] }),
    ("taverner", MeetingLine { act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["I feed the whole road at my tables. Pool the stores and I'll stretch them further than any of {2:obj}."] }),
    ("smith", MeetingLine { act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["I've worked iron since I could lift the hammer. A day's pay doesn't buy the charcoal for a day's work."] }),
    ("mason", MeetingLine { act: Act::Remark, concepts: &["concern:bridge", "concern:wages"], bias: &[],
        lines: &["I've laid stone thirty years - that pier's been going since the flood. Mend it before the spring melt.",
            "A mason's wage is set by men who've never lifted a block. Name the price and we'll hold to it."] }),
    ("cooper", MeetingLine { act: Act::Remark, concepts: &["concern:bridge", "concern:wages"], bias: &[],
        lines: &["I know wood - those planks are soft to the heart. Replace them now, not one at a time."] }),
    ("shoemaker", MeetingLine { act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["I've shod half this village. A pair takes me three days; the price they pay is one."] }),
    ("weaver", MeetingLine { act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["The loom doesn't lie - an ell of cloth is a day of my life. They pay for half of it."] }),
    ("tailor", MeetingLine { act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["Every seam in this room is my work or my master's. Our needles feed us less each year."] }),
    ("miner", MeetingLine { act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["I've been down the pit since my father was. The ore's worth ten times what they pay us to cut it."] }),
    ("priest", MeetingLine { act: Act::Remark, concepts: &["concern:faith"], bias: &[],
        lines: &["I've said the office at that altar many years. The faith isn't failing - the bell is, and the roof.",
            "I've buried and married most of {2:obj}. The church will stand with {2:obj}, if {2:subj} {2:will} come back to it."] }),
];

/// A worked-example agenda for a house meeting
pub const MEET_PROMPT: MeetingPrompt = MeetingPrompt {
    text: "What ails the village most, friends? Speak true.",
    concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:fuel", "concern:bridge", "concern:harvest", "concern:commons", "concern:grief"],
};

/// The probing question (#1267): not the village's trouble but the
/// person's own - what they want, and who they're doing it for
pub const MEET_PROBE: MeetingPrompt = MeetingPrompt {
    text: "Leave the village's troubles a moment. What {2:do} {2:subj} actually want - and who {2:be} {2:subj} doing it for?",
    concepts: &["probe:want", "probe:love"],
};

/// What the room says to the probe when nobody's naming names
pub const MEET_PROBE_LINES: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["probe:want"], bias: &[],
        lines: &["To feel I did right by them, when it's all done.", "Not to be afraid of the knock at the door.",
            "To be more than the work my hands do.", "That my children won't bow as low as I do."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["probe:love"], bias: &[],
        lines: &["For my own - who else would I do it for?", "For the ones at my table. That's the whole of it.",
            "For the ones who can't stand up for themselves."],
    },
    MeetingLine {
        act: Act::Agree, concepts: &["probe:want", "probe:love"], bias: &[],
        lines: &["Aye - {other} says what I'd not have said aloud.", "That's it. That's mine too."],
    },
    MeetingLine {
        act: Act::PushBack, concepts: &["probe:want", "probe:love"], bias: &[],
        lines: &["Wants are a luxury, {other}. The rent isn't.", "Feelings won't fill the granary."],
    },
];

/// A trusting answer to the probe: they name who they do it for
/// (`{name}`) - the room hears the bond (#1267)
pub const PROBE_NAMES: &[&str] = &[
    "For {name}. If it comes right for them, it's come right for me.",
    "{name}. It's always been {name}.",
    "Truth? For {name}. I'd not say it anywhere but here.",
];

// The rite that frames a meeting (#1185): a devout attendee's prayer
// opens it - old-ways folk get their own words - a gregarious one offers
// a verse, a believer gives the benediction, and when the room offers
// nobody the organizer does the plain word
const RITE_PRAYER: &[&str] = &[
    "God keep this house and all who speak plain in it.",
    "Lord, give us patience with one another and courage for the truth.",
    "Bless this hearth, and every hand that keeps it.",
    "May the words said here do more good than harm. Amen.",
];
const RITE_OLD: &[&str] = &[
    "The stones hear us. Speak true.",
    "By the elder tree and the ones who went before - we begin.",
    "What is said here, the land keeps. Let it be worth keeping.",
];
const RITE_VERSE: &[&str] = &[
    "A verse for the room: may our wants be few and our voices many.",
    "Here's to the roof that holds us all, and the tongues that dare to speak.",
    "Sit close, speak plain - the night is long and the fire is warm.",
];
const RITE_PLAIN: &[&str] = &[
    "Well then. Let's begin.",
    "We're gathered. Say what needs saying.",
];
const CLOSE_BLESSING: &[&str] = &[
    "Go in peace - and keep what was said here.",
    "God go with {2:obj} all, and bring {2:obj} back next time.",
    "May what we spoke of take root. Amen, and good night.",
];
const CLOSE_OLD: &[&str] = &[
    "The stones watch {2:obj} home.",
    "Keep the old ways, and keep each other.",
];
const CLOSE_PLAIN: &[&str] = &[
    "That's the whole of it. Go well.",
    "Home safe, all of {2:obj} - and thank {2:obj} for coming.",
];
// the organizer owns the thread: when the floor drifts, they call it
// back to the matter (#1186)
const PULL_BACKS: &[&str] = &[
    "Back to the matter at hand, friends.",
    "That's for another fire - what ails us most?",
    "Hold - we've wandered. Speak to the matter.",
];

/// The framing lines `run_meeting` opens and closes with
pub const MEET_RITES: MeetingRites = MeetingRites {
    prayers: RITE_PRAYER,
    old_rites: RITE_OLD,
    verses: RITE_VERSE,
    plain_opens: RITE_PLAIN,
    blessings: CLOSE_BLESSING,
    old_closes: CLOSE_OLD,
    plain_closes: CLOSE_PLAIN,
    pull_backs: PULL_BACKS,
};

// The audition (#1187): early in a talk with someone new, the caller
// probes whether this one's safe - and the test IS their trait. A good
// answer takes the talk deeper; a wrong note leaves it polite and
// shallow. Small stakes, not a grudge.
pub const PROBE_TESTS: &[Probe] = &[
    // the stingy test generosity: a free hand's a warm neighbor - or a fool
    Probe {
        who: Trait::Stingy,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "They say a free hand makes warm neighbors. What's {2:poss} mind on it?",
            "Some folk give till it hurts. Thin soup for the giver, say I - and {2:subj}?",
        ],
        pass: (Act::Agree, &[
            "An open hand finds open doors, say I.",
            "What's spent on neighbors comes back doubled in the end.",
            "A loaf shared is a friend made - I'd not grudge it.",
        ]),
        fail: (Act::Remark, &[
            "Charity begins at home, and mostly stays there.",
            "A wise purse keeps its own counsel, say I.",
            "Folk who give away their bread soon beg for mine.",
        ]),
    },
    // the pious test faith: does this one keep the word?
    Probe {
        who: Trait::Pious,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "The bells call us all the same. {2:do} keep the faith?",
            "Some say the old prayers are only words. And {2:subj}?",
        ],
        pass: (Act::Agree, &[
            "I keep the faith as my mother taught me, and gladly.",
            "The Word's a lamp in dark times - I'd not put it out.",
            "Prayer and work, friend - that's the whole of it.",
        ]),
        fail: (Act::Remark, &[
            "The priest talks much and mends little, say I.",
            "I bow when the neighbors bow. It keeps the peace.",
            "The saints help them as help themselves, I've heard.",
        ]),
    },
    // the hotheaded test whether you'll stand up to them: a prod to see
    // if there's iron in you - the right answer is the push-back
    Probe {
        who: Trait::Hotheaded,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "The miller had the right of it in that quarrel - {2:subj} {2:will} agree, surely?",
            "I'd have thumped the steward myself. {2:subj} {2:would} have done the same, no doubt?",
        ],
        pass: (Act::PushBack, &[
            "I say no such thing. The miller was half to blame and more.",
            "Not I. A quick fist makes a slow friend.",
            "{2:subj} {2:will} not put words in my mouth, friend.",
        ]),
        fail: (Act::Agree, &[
            "Aye... I suppose {2:subj} {2:be} right about that.",
            "As {2:subj} {2:say}, no doubt it's so.",
            "I'll not argue it, if it pleases {2:obj}.",
        ]),
    },
    // the proud test whether you'll treat them as they reckon themselves
    Probe {
        who: Trait::Proud,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "Folk here know to speak fair to me. {2:subj} {2:will} have heard as much?",
            "My family held this land when {2:own} were sowing wild oats. {2:do} know it?",
        ],
        pass: (Act::Agree, &[
            "Aye, and I've heard nothing but good of {2:poss} house.",
            "An old name's worth the hearing, say I.",
            "I've heard, and I mind my manners accordingly.",
        ]),
        fail: (Act::Remark, &[
            "Old names plough no fields, friend.",
            "We all muck the same stalls here, high or low.",
            "Can't say the name reached me - I keep to my own lane.",
        ]),
    },
    // the gregarious test whether you'll lean in: a gossip bid
    Probe {
        who: Trait::Gregarious,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "{2:have} heard what they say of the reeve's new hound?",
            "There's talk going round the wells this week - heard it?",
        ],
        pass: (Act::Agree, &[
            "Not I - do tell, then. I love a good tale.",
            "Come, I've an ear for it - out with it.",
            "If it's fit to tell, I'm fit to hear it.",
        ]),
        fail: (Act::Remark, &[
            "I keep my ears for my own business, friend.",
            "Wells talk empties a mind faster than a sieve.",
            "I'd sooner hear the weather than the gossip.",
        ]),
    },
    // the ambitious test whether you've any drive: a body should want more
    Probe {
        who: Trait::Ambitious,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "A body's meant for more than the furrow, eh? Or is that {2:poss} lot?",
            "I mean to be more than I was born. And {2:subj} - content where {2:subj} {2:stand}?",
        ],
        pass: (Act::Agree, &[
            "More, aye - I've plans of my own, given half a chance.",
            "I mean to rise same as {2:obj}, if the saints allow it.",
            "The furrow feeds me; it doesn't define me.",
        ]),
        fail: (Act::Remark, &[
            "A full belly and a dry roof - what more's there to want?",
            "Ambition's a hungry hound. Mine sleeps.",
            "The furrow's honest work enough for me.",
        ]),
    },
    // the kind test whether there's warmth in you: a small confidence
    Probe {
        who: Trait::Kind,
        act: Act::Remark,
        concepts: &["audition"],
        calls: &[
            "These damp mornings set my knees aching something awful.",
            "The littlest one kept us up all night with the coughing, truth be told.",
        ],
        pass: (Act::Answer, &[
            "I'm sorry for it - a warm stone at the joint helps, they say.",
            "Poor mite - I'll ask after them tomorrow, if {2:subj} {2:will} allow.",
            "That's a hard row. Can I carry aught for {2:obj}?",
        ]),
        fail: (Act::Remark, &[
            "Knees ache. Such is age, friend.",
            "Children cough. They grow out of it.",
            "We've all our troubles, mine no lighter than {2:own}.",
        ]),
    },
    // the lazy test whether you'll be easy company: no hustle-preaching
    Probe {
        who: Trait::Lazy,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "Hard work's for them as never learned better, eh?",
            "I say a body earns their rest. {2:subj} {2:will} not disagree?",
        ],
        pass: (Act::Agree, &[
            "The field'll keep till tomorrow, say I.",
            "A wise back knows when to unbend.",
            "Rest's a harvest too, of a kind.",
        ]),
        fail: (Act::Remark, &[
            "Idle hands, friend. The frost doesn't wait.",
            "Work's what keeps a body honest.",
            "The saints gave us six days for a reason.",
        ]),
    },
];

/// A thing you can ask after mid-talk (#1189): the personal ones are
/// always on the table, the situational ones open by who they are -
/// their trade, their trouble, the road they came in on - and some
/// (`draws`) pull a real secret out when you put the question.
pub struct Topic {
    /// The menu words, as in "Ask about {label}"
    pub label: &'static str,
    /// The subject area the openness gate reads (E3-2, #1424)
    pub area: crate::game::conversation::Area,
    /// How deep it goes: 0 public, 1 personal (two talks), 2 private
    /// (openness .4 in the area), 3 core (.7, no ears) (E3-2, #1424)
    pub tier: u8,
    /// Statuses the asked-about speaker must carry (all of them)
    pub needs: &'static [&'static str],
    /// Or any one of these statuses, when `needs` is empty
    pub any: &'static [&'static str],
    /// Statuses that close the topic off
    pub not: &'static [&'static str],
    /// What you ask, in your mouth
    pub ask: &'static [&'static str],
    /// What they tell back
    pub tell: &'static [&'static str],
    /// The guarded register (#1710): a tell-named ask that only the
    /// seen tell carried past the bar gets this - an answer that keeps
    /// its depth for a truer ear
    pub hedged: &'static [&'static str],
    /// Thread tags the exchange carries
    pub concepts: &'static [&'static str],
    /// What asking it draws out, beyond the tell lines
    pub draws: Draw,
}

/// What putting a question can draw out (#1189): the plain topics just
/// get an answer; the deeper ones pull something real - a kept secret,
/// or a piece of the mystery folk are whispering about
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Draw {
    /// Just the answer
    No,
    /// A kept secret - the same reveal a talk's opening rolls for
    Secret,
    /// A fact of the seat's live mystery, if one's still unheard
    Mystery,
    /// Their story of a prehistory event, told in Labov's parts
    /// (E4-8, #1426): the answer is composed off the speaker's
    /// `story` seed, not picked from `tell`
    Story,
    /// A thing folk are saying (#1616): a real fact the speaker could
    /// know - a carried word about a third party, the sky, talk seen
    /// in the lanes - drawn session-side after the answer
    Gossip,
}

/// The topics a Talk's ask-slot can open (#1189). Personal questions
/// are always available; the rest appear when the state supports them -
/// a live status, a fact already shared, a secret still kept.
pub const ASK_TOPICS: &[Topic] = &[
    Topic {
        label: "the village",
        area: crate::game::conversation::Area::Village,
        tier: 0, needs: &[], any: &[], not: &["visitor"],
        // the village's own ear (#1616): the answer's a real thing folk
        // say - a carried word, the sky, talk seen in the lanes -
        // not one of the stock "folk fare as folk do" hedges
        ask: &["Tell me what folk are saying.", "What word of the village these days?", "How do folk fare here, to {2:poss} eyes?"],
        tell: &["Now, let me think what's going about...", "Ah, the talk - there's always talk.",
            "Folk do say things, if {2:subj} {2:will} hear them."],
        hedged: &[],
        concepts: &["village", "gossip"], draws: Draw::Gossip,
    },
    Topic {
        label: "the road",
        area: crate::game::conversation::Area::Road,
        tier: 1, needs: &["visitor"], any: &[], not: &[],
        ask: &["What word from the road?", "What news do the roads carry?"],
        tell: &["The roads are what they are - long, and thin of comfort.", "Word travels: the country's unsettled of late.",
            "I've seen worse lanes and better company."],
        hedged: &[],
        concepts: &["road", "news"], draws: Draw::No,
    },
    Topic {
        label: "their work",
        area: crate::game::conversation::Area::Work,
        tier: 0,
        needs: &[],
        any: &["smith", "mason", "shepherd", "beekeeper", "weaver", "fisher",
            "brewer", "baker", "taverner", "carpenter", "herb_wife"],
        not: &[],
        ask: &["How fares the work, friend?", "Tell me of {2:poss} craft - how goes it?"],
        tell: &["It keeps me, as honest work keeps any.", "Well enough - the hands know their business.",
            "Slower than I'd like this season, but it pays.", "A craft's a craft - it fills the day and the purse, mostly."],
        hedged: &[],
        concepts: &["work", "trade"], draws: Draw::No,
    },
    Topic {
        label: "the day's work",
        area: crate::game::conversation::Area::Work,
        tier: 0,
        needs: &[],
        any: &[],
        not: &["visitor", "smith", "mason", "shepherd", "beekeeper", "weaver", "fisher",
            "brewer", "baker", "taverner", "carpenter", "herb_wife", "child"],
        ask: &["How fares {2:poss} day's work?", "The day's labor - how does it treat {2:obj}?"],
        tell: &["A holding keeps a soul busy, one way and another.", "The fields are what they are - they want what they want.",
            "I take what work comes, and thank the saints for it."],
        hedged: &[],
        concepts: &["work"], draws: Draw::No,
    },
    // the kin abroad (#1707): the family ask names the real town their
    // folk run to, only of a house whose kin ride the roads
    Topic {
        label: "their family",
        area: crate::game::conversation::Area::Heart,
        tier: 1, needs: &["kin_abroad"], any: &[], not: &[],
        ask: &["How do {2:poss} folk fare - is there word of them?", "Any word of {2:poss} kin abroad, friend?"],
        tell: &["They rub along - my brother carts to {town} twice a season, saints keep him.",
            "A letter's looked for from {town}, any week now.", "They're scattered - one at {town}, the rest under this roof."],
        hedged: &[],
        concepts: &["family"], draws: Draw::No,
    },
    Topic {
        label: "their sickness",
        area: crate::game::conversation::Area::Body,
        tier: 1, needs: &["sick"], any: &[], not: &[],
        ask: &["How does the sickness fare with {2:obj}?", "{2:subj} {2:look} pale - is the illness past?"],
        tell: &["It lingers - worse some days than others.", "I mend, slowly. The herb-wife says patience.",
            "The cough keeps me nights, truth be told."],
        hedged: &[],
        concepts: &["sickness", "health"], draws: Draw::No,
    },
    Topic {
        label: "their hunger",
        area: crate::game::conversation::Area::Body,
        tier: 1, needs: &["hungry"], any: &[], not: &[],
        ask: &["When {2:did} {2:subj} last eat a full meal?", "Is bread short at {2:poss} table?"],
        tell: &["A belly remembers bread better than days.", "We make do - there's always less to make do with.",
            "The store's thin. The winter's long."],
        hedged: &[],
        concepts: &["hunger", "bread"], draws: Draw::No,
    },
    Topic {
        label: "their grief",
        area: crate::game::conversation::Area::Heart,
        tier: 2, needs: &["grieving"], any: &[], not: &[],
        ask: &["Who was it {2:subj} {2:lost}?", "The house sits heavy - who is it {2:subj} {2:miss}?"],
        tell: &["Someone the house still feels the lack of.", "I don't say the name much - it keeps.",
            "The chair by the fire's empty now. That's the whole of it."],
        hedged: &["A grief, aye - I'll not lay the whole of it out yet.",
            "Someone close. More than that's for a truer ear, friend."],
        concepts: &["grief", "family"], draws: Draw::No,
    },
    Topic {
        label: "the trouble they told",
        area: crate::game::conversation::Area::Trouble,
        tier: 1, needs: &["shared"], any: &[], not: &[],
        ask: &["That trouble {2:subj} {2:spoke} of - what weighs on {2:obj}?", "{2:subj} {2:said} something of it once - how goes it now?"],
        tell: &["Aye, it sits with me still.", "The same as was - the asking's kind, though.",
            "It hasn't eased, if that's the truth of it."],
        hedged: &[],
        concepts: &["trouble", "burden"], draws: Draw::Secret,
    },
    Topic {
        label: "what weighs on them",
        area: crate::game::conversation::Area::Trouble,
        tier: 2, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["Is aught weighing on {2:obj}? {2:subj} {2:will} tell me, if there is.", "Something sits behind {2:poss} eyes - what is it?"],
        tell: &["Ah. {2:subj} {2:be} sharp-eyed, then.", "I had hoped it didn't show.", "If anyone's to hear it, {2:subj} {2:be} as good as any."],
        hedged: &["Something sits, as {2:subj} saw - I'll not name it here.",
            "Aye, there's a weight. I'd keep the shape of it a while longer."],
        concepts: &["trouble", "secret"], draws: Draw::Secret,
    },
    Topic {
        label: "the question folk whisper of",
        area: crate::game::conversation::Area::Village,
        tier: 1, needs: &["mystery"], any: &[], not: &[],
        // the asks name the seat's live mystery - the asker can tell
        // which story they're chasing (#1608)
        ask: &["Folk whisper of {mystery} - what do they say?", "There's talk of {mystery} round the village - what's said of it?"],
        tell: &["Ah, {mystery} - {2:subj} {2:have} heard the asking too.", "Aye, {mystery} - folk do talk, and some of it's true.", "I've a piece of {mystery}, if {2:subj} {2:want} the hearing."],
        hedged: &[],
        concepts: &["mystery", "gossip"], draws: Draw::Mystery,
    },
    Topic {
        label: "themselves",
        area: crate::game::conversation::Area::Heart,
        tier: 1, needs: &[], any: &[], not: &[],
        ask: &["Tell me of {2:self} - how {2:did} {2:subj} come here?", "What's {2:poss} tale, then?"],
        tell: &["Born under this roof, like most of us.", "I've no great tale - the years just keep.",
            "Same story as the next one's, only mine.", "I came for the work and stayed for the quiet."],
        hedged: &[],
        concepts: &["self", "history"], draws: Draw::No,
    },
    // the tale itself (E4-8, #1426): a prehistory story told in Labov's
    // parts - the answer is composed off their `story` seed, not picked
    Topic {
        label: "the old days",
        area: crate::game::conversation::Area::Heart,
        tier: 1, needs: &["has_a_past"], any: &[], not: &["visitor"],
        ask: &["Tell me of the old days - what do {2:subj} keep of them?", "What years does {2:poss} memory still carry?"],
        tell: &["The old days keep what they keep.", "Long years back - the mind wanders there."],
        hedged: &[],
        concepts: &["self", "history", "past"], draws: Draw::Story,
    },
    // the deepest asks (E3-4, #1425): tier 3 - a kept secret tells in
    // the fact's own words when the carrier holds one, else these
    Topic {
        label: "the weight they carry",
        area: crate::game::conversation::Area::Heart,
        tier: 2, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["{2:subj} {2:carry} something, friend - what is it weighs on {2:obj}?"],
        tell: &["It's a weight, is all - I carry it as I can.",
            "Some things a body just keeps carrying."],
        hedged: &["A weight, as {2:subj} say - I'd not set it down in the lane.",
            "It rides with me. The telling of it's for a nearer ear."],
        concepts: &["grief", "burden"], draws: Draw::Secret,
    },
    Topic {
        label: "what they never told",
        area: crate::game::conversation::Area::Trouble,
        tier: 3, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["There's something {2:subj} {2:have} never told a soul - what is it?"],
        tell: &["{2:subj} {2:be} the first to ask it plain. I don't know that I can.",
            "That's not a thing I've said aloud."],
        hedged: &["Never told is never told - not yet, even to a kind ear.",
            "That's a door I'd keep shut a while longer."],
        concepts: &["trouble", "secret"], draws: Draw::Secret,
    },
    Topic {
        label: "the thing nearest the bone",
        area: crate::game::conversation::Area::Heart,
        tier: 3, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["What sits nearest the bone with {2:obj}, if {2:subj} {2:will} tell it?"],
        tell: &["The thing nearest the bone stays there, mostly.",
            "If I say it, it stops being only mine."],
        hedged: &["Near the bone it stays - a little less so, for the asking.",
            "{2:subj} {2:saw} it plain. I'd keep it wrapped a while still."],
        concepts: &["grief", "secret"], draws: Draw::Secret,
    },
];

// Two children meeting in the lane: games, dares and the things grown-ups
// mustn't hear. The village's other trees are grown-up talk - wages, rents,
// the harvest - so a pair of kids keeps to this one (session::dialogue
// cuts their talk off where it would fall through to the adult tail)
const KIDS_RUN: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["play"],
    calls: &["Race {2:obj} to the well and back!", "Last one to the mill pond is a mouldy turnip!", "Mother's calling. I'll be skinned if I'm late again."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Not fair, {2:subj} {2:had} a head start!", "Tomorrow, then. Same place.", "Run, {other}, run!"],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIDS_DARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["play"],
    calls: &["I dare {2:obj} to knock on the hermit's cave.", "{2:would} {2:subj} climb the church wall if I held {2:poss} boots?", "{2:have} {2:subj} seen the miller's new pup? It bit Tom."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Proud, 1.5), (Hotheaded, 1.5)],
            act: Act::Agree, concepts: &["play"],
            respond: None,
            lines: &["Easy! Watch me.", "Double dare {2:obj} back, then.", "Only if {2:subj} {2:go} first."],
            voice: &[],
            then: Some(KIDS_RUN), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Kind, 1.5), (Lazy, 1.5)],
            act: Act::PushBack, concepts: &["play"],
            respond: None,
            lines: &["Not likely - my father would tan my hide.", "{2:subj} {2:be} cracked in the head, {other}.", "I'm not scared. I just don't want to."],
            voice: &[],
            then: Some(KIDS_RUN), end: None,
        },
    ],
};

const KIDS_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "play"],
    calls: &["Oi, {other}! Come and see what I found.", "{other}! {2:have} {2:subj} any errands, or {2:can} {2:subj} play?", "Psst, {other} - over here."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 1.5)],
            act: Act::Greet, concepts: &["play"],
            respond: None,
            lines: &["What is it? Show me!", "I've an errand, but it can wait a bit.", "Is it a frog? Tell me it's a frog."],
            voice: &[],
            then: Some(KIDS_DARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Ambitious, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Can't - I'm running for the baker and he pays a penny.", "Later, {other}. I've chores."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The full dialogue library: 30 circumstance and vocation trees plus smalltalk tail
pub fn village_library() -> Vec<DialogueTree> {
    let mut lib = vec![
        // the sheep in the cradle (#1254): status-only trees, pushed by
        // the session while the vignette's live - first for the shortest
        // tie-break. The ewe trees never fire in villager-villager chat:
        // the statuses only exist on player-talk speakers
        DialogueTree { name: EWE_TREE,
            criteria: &[Has("self:status:ewe_secret"), Lacks("self:status:child")],
            root: &EWE_GREET, rejoin: None },
        DialogueTree { name: EWE_PRICE_TREE,
            criteria: &[Has("self:status:ewe_leverage"), Lacks("self:status:child")],
            root: &EWE_PRICE_GREET, rejoin: None },
        DialogueTree { name: EWE_VICTIM_TREE,
            criteria: &[Has("self:status:ewe_victim"), Lacks("self:status:child")],
            root: &EWE_VICTIM_GREET, rejoin: None },
        // the row in the street (#1257): either half mid-row answers
        // the mediator's opening
        DialogueTree { name: NOAH_ROW_TREE,
            criteria: &[Has("self:status:rowing"), Lacks("self:status:child")],
            root: &NOAH_GREET, rejoin: None },
        // the false traveller (#1258): three criteria outrank the
        // archetype trees - the motive status only exists while he's
        // about, so the wrong questions never fire on an honest pedlar
        DialogueTree { name: FT_SPY_TREE,
            criteria: &[Has("self:status:visitor"), Has("self:status:false-traveller"),
                Has("self:status:ft-spy")],
            root: &FT_SPY_GREET, rejoin: None },
        DialogueTree { name: FT_THIEF_TREE,
            criteria: &[Has("self:status:visitor"), Has("self:status:false-traveller"),
                Has("self:status:ft-thief")],
            root: &FT_THIEF_GREET, rejoin: None },
        DialogueTree { name: FT_RUN_TREE,
            criteria: &[Has("self:status:visitor"), Has("self:status:false-traveller"),
                Has("self:status:ft-runaway")],
            root: &FT_RUN_GREET, rejoin: None },
        // the road's visitors (#1168): the again-tree leads - at two
        // criteria it ties the kind/archetype trees and library order
        // makes the repeat the shorter talk
        DialogueTree { name: "visitor_again",
            criteria: &[Has("self:status:visitor"), Has("self:status:spoken")],
            root: &VIS_AGAIN_GREET, rejoin: None },
        // kind voices where the stakes are their own
        DialogueTree { name: "visitor_deserter",
            criteria: &[Has("self:status:visitor"), Has("self:status:deserter")],
            root: &VIS_DESERTER_GREET, rejoin: None },
        DialogueTree { name: "visitor_lepers",
            criteria: &[Has("self:status:visitor"), Has("self:status:leper-band")],
            root: &VIS_LEPER_GREET, rejoin: None },
        DialogueTree { name: "visitor_refugees",
            criteria: &[Has("self:status:visitor"), Has("self:status:refugee-family")],
            root: &VIS_REFUGEE_GREET, rejoin: None },
        DialogueTree { name: "visitor_heretic",
            criteria: &[Has("self:status:visitor"), Has("self:status:quiet-preacher")],
            root: &VIS_HERETIC_GREET, rejoin: None },
        DialogueTree { name: "visitor_recruiter",
            criteria: &[Has("self:status:visitor"), Has("self:status:recruiter")],
            root: &VIS_RECRUITER_GREET, rejoin: None },
        DialogueTree { name: "visitor_fortune_teller",
            criteria: &[Has("self:status:visitor"), Has("self:status:fortune-teller")],
            root: &VIS_FORTUNE_GREET, rejoin: None },
        DialogueTree { name: "visitor_beggar",
            criteria: &[Has("self:status:visitor"), Has("self:status:beggar")],
            root: &VIS_BEGGAR_GREET, rejoin: None },
        // the archetypes every traveller kind falls into
        DialogueTree { name: "visitor_trader",
            criteria: &[Has("self:status:visitor"), Has("self:status:arch-trader")],
            root: &VIS_TRADER_GREET, rejoin: None },
        DialogueTree { name: "visitor_craftsman",
            criteria: &[Has("self:status:visitor"), Has("self:status:arch-craftsman")],
            root: &VIS_CRAFT_GREET, rejoin: None },
        DialogueTree { name: "visitor_performer",
            criteria: &[Has("self:status:visitor"), Has("self:status:arch-performer")],
            root: &VIS_PERFORMER_GREET, rejoin: None },
        DialogueTree { name: "visitor_religious",
            criteria: &[Has("self:status:visitor"), Has("self:status:arch-religious")],
            root: &VIS_RELIGIOUS_GREET, rejoin: None },
        DialogueTree { name: "visitor_mystic",
            criteria: &[Has("self:status:visitor"), Has("self:status:arch-mystic")],
            root: &VIS_MYSTIC_GREET, rejoin: None },
        // the catch-all: every traveller talks, even the unmapped ones
        DialogueTree { name: "visitor_generic",
            criteria: &[Has("self:status:visitor")],
            root: &VIS_CALLER_GREET, rejoin: Some(&VIS_ROAD_NEWS) },
        DialogueTree { name: "children_at_play",
            criteria: &[Has("self:status:child"), Has("other:status:child")],
            root: &KIDS_GREET, rejoin: None },
        // A cleric calling on the sick or the bereaved outranks the lay
        // visit, so the "pray for me" forks are reachable
        DialogueTree { name: "the_cleric_visits_the_sick",
            criteria: &[Has("self:ideology:clergy"), Has("other:status:sick")],
            root: &CLERIC_GREET, rejoin: Some(&VISIT_CARE) },
        DialogueTree { name: "the_cleric_consoles",
            criteria: &[Has("self:ideology:clergy"), Has("other:status:grieving")],
            root: &CLERIC_GREET, rejoin: Some(&CONDOLE_OFFER) },
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
        // the fine kinds (#1372): a sweetheart, a cousin, a master and a
        // learner each get their own beat before the coarse kin/acquaintance
        // material - and before the generic talk
        DialogueTree { name: "sweetheart_beat", criteria: &[Has("pair:kind:lover")],
            root: &LOVER_BEAT, rejoin: Some(&LOVER_BEAT) },
        DialogueTree { name: "blood_kin", criteria: &[Has("pair:kind:family")],
            root: &KIN_BLOOD, rejoin: Some(&KIN_BLOOD) },
        DialogueTree { name: "mentor_counsel", criteria: &[Has("pair:kind:mentor")],
            root: &MENTOR_COUNSEL, rejoin: Some(&MENTOR_COUNSEL) },
        DialogueTree { name: "apprentice_eager", criteria: &[Has("pair:kind:apprentice")],
            root: &APPRENTICE_EAGER, rejoin: Some(&APPRENTICE_EAGER) },
        DialogueTree { name: "kin_warmth", criteria: &[Has("pair:relation:kin")],
            root: &KIN_GREET, rejoin: Some(&KIN_FAMILY) },
        DialogueTree { name: "friendship_bonds", criteria: &[Has("pair:relation:friend")],
            root: &FRIEND_GREET, rejoin: Some(&FRIEND_TRUST) },
        DialogueTree { name: "stranger_road", criteria: &[Has("pair:relation:stranger")],
            root: &STRANGER_GREET, rejoin: Some(&STRANGER_PATH) },
        // the village's own weather (#1696): what the whole parish lives
        // under outranks the shop-talk - a famine, a fever, fresh graves
        // color every talk in the street, whichever mouths carry them
        DialogueTree { name: "the_hard_times", criteria: &[Has("village:hard")],
            root: &HARD_TIMES_GREET, rejoin: Some(&HARD_TIMES_GREET) },
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
        DialogueTree { name: "taverner_bench", criteria: &[Has("other:status:taverner")],
            root: &TAVERNER_GREET, rejoin: Some(&TAVERNER_BARREL) },
        DialogueTree { name: "beekeeper_hive", criteria: &[Has("other:status:beekeeper")],
            root: &BEEKEEPER_GREET, rejoin: Some(&BEEKEEPER_HONEY) },
        DialogueTree { name: "herb_apothecary", criteria: &[Has("other:status:herb_wife")],
            root: &HERB_GREET, rejoin: Some(&HERB_DRAUGHT) },
        DialogueTree { name: "the_merchants_ledger", criteria: &[Has("self:ideology:merchant")],
            root: &LEDGER_GREET, rejoin: Some(&LEDGER_ACCOUNTS) },
        DialogueTree { name: "merchant_trade", criteria: &[Has("other:ideology:merchant")],
            root: &MERCHANT_GREET, rejoin: Some(&MERCHANT_WARES) },
        DialogueTree { name: "the_lords_levee", criteria: &[Has("self:ideology:lord")],
            root: &LEVEE_GREET, rejoin: Some(&LEVEE_LAW) },
        DialogueTree { name: "lord_manor", criteria: &[Has("other:ideology:lord")],
            root: &LORD_GREET, rejoin: Some(&LORD_LEASES) },
        DialogueTree { name: "fisher_river", criteria: &[Has("other:status:fisher")],
            root: &FISHER_GREET, rejoin: Some(&FISHER_CURRENT) },
        DialogueTree { name: "charcoal_pit", criteria: &[Has("other:status:charcoal_burner")],
            root: &CHARCOAL_GREET, rejoin: Some(&CHARCOAL_SMOKE) },
        DialogueTree { name: "farmer_fields", criteria: &[Has("other:status:farmer")],
            root: &FARMER_GREET, rejoin: Some(&FARMER_SOW) },
        DialogueTree { name: "tailor_stitch", criteria: &[Has("other:status:tailor")],
            root: &TAILOR_GREET, rejoin: Some(&TAILOR_SEAM) },
        DialogueTree { name: "luthier_wood", criteria: &[Has("other:status:luthier")],
            root: &LUTHIER_GREET, rejoin: Some(&LUTHIER_WOOD) },
        DialogueTree { name: "printer_press", criteria: &[Has("other:status:printer")],
            root: &PRINTER_GREET, rejoin: Some(&PRINTER_FORME) },
        DialogueTree { name: "glazier_panes", criteria: &[Has("other:status:glazier")],
            root: &GLAZIER_GREET, rejoin: Some(&GLAZIER_LEAD) },
        DialogueTree { name: "elder_wisdom", criteria: &[Has("other:status:elder")],
            root: &ELDER_GREET, rejoin: Some(&ELDER_WINTERS) },
        // the caller-side mirror (#1693): the player Talk replay always
        // seats the villager as self, so an elder villager only ever
        // reaches the elder tree through this criterion
        DialogueTree { name: "elder_wisdom_self", criteria: &[Has("self:status:elder")],
            root: &ELDER_GREET_SELF, rejoin: Some(&ELDER_WINTERS_SELF) },
        DialogueTree { name: "newborn_cradle", criteria: &[Has("self:status:newborn")],
            root: &NEWBORN_GREET, rejoin: Some(&NEWBORN_JOY) },
        // the bond ladder: how well they know the organizer changes how
        // they meet them - two criteria so it outranks friendship and the
        // one-criterion circumstances: the stage speaks first
        DialogueTree { name: "the_bond_acquainted",
            criteria: &[Has("self:status:acquainted"), Has("any:status:acquainted")],
            root: &ACQUAINTED_GREET, rejoin: Some(&NEIGHBOR_CHAT) },
        DialogueTree { name: "the_bond_known",
            criteria: &[Has("self:status:known"), Has("any:status:known")],
            root: &KNOWN_GREET, rejoin: Some(&BOND_CHAT) },
        DialogueTree { name: "the_bond_follower",
            criteria: &[Has("self:status:follower"), Has("any:status:follower")],
            root: &FOLLOWER_GREET, rejoin: Some(&BOND_CHAT) },
        DialogueTree { name: "the_bond_coleader",
            criteria: &[Has("self:status:coleader"), Has("any:status:coleader")],
            root: &COLEADER_GREET, rejoin: Some(&BOND_CHAT) },
        // two members of one cause carry its register in the street too
        // (#1228): both chairs must hold the fact, so the player's Talk -
        // where the organizer isn't a member - never lands here
        DialogueTree { name: "the_cause_fellows",
            criteria: &[Has("self:status:cause_member"), Has("other:status:cause_member")],
            root: &FELLOW_GREET, rejoin: Some(&FELLOW_CHAT) },
        // the trades lead their own talks too (#1617): when the holder's
        // the caller - the player's Talk included - the day at the bench
        // opens the exchange. One criterion each and declared last of the
        // voices, so a circumstance, a bond or an elder speaks first; the
        // day's own telling rides inside the greet's voice instead
        DialogueTree { name: "the_ploughman", criteria: &[Has("self:status:farmer")],
            root: &PLOUGH_GREET, rejoin: Some(&PLOUGH_FIELDS) },
        DialogueTree { name: "the_bakers_oven", criteria: &[Has("self:status:baker")],
            root: &OVEN_GREET, rejoin: Some(&OVEN_BATCH) },
        DialogueTree { name: "the_masons_yard", criteria: &[Has("self:status:mason")],
            root: &CHISEL_GREET, rejoin: Some(&CHISEL_STONE) },
        DialogueTree { name: "the_tailors_shears", criteria: &[Has("self:status:tailor")],
            root: &NEEDLE_GREET, rejoin: Some(&NEEDLE_SEAM) },
        DialogueTree { name: "the_smiths_anvil", criteria: &[Has("self:status:smith")],
            root: &ANVIL_GREET, rejoin: Some(&ANVIL_IRON) },
        DialogueTree { name: "the_luthiers_bench", criteria: &[Has("self:status:luthier")],
            root: &BELLY_GREET, rejoin: Some(&BELLY_WOOD) },
        DialogueTree { name: "the_printers_press", criteria: &[Has("self:status:printer")],
            root: &PRESS_GREET, rejoin: Some(&PRESS_FORME) },
        DialogueTree { name: "the_glaziers_lead", criteria: &[Has("self:status:glazier")],
            root: &CAME_GREET, rejoin: Some(&CAME_GLASS) },
        // a marked soul's name greets ahead of smalltalk (#950): single
        // criterion and declared late, so a trade, a circumstance or a
        // bond outranks it - the name only speaks when nothing else does
        DialogueTree { name: "the_good_name",
            criteria: &[Has("other:status:kind_soul")],
            root: &VIRTUE_GOOD_GREET, rejoin: Some(&SMALL_CHAT) },
        DialogueTree { name: "the_devout_soul",
            criteria: &[Has("other:status:devout")],
            root: &VIRTUE_DEVOUT_GREET, rejoin: Some(&SMALL_CHAT) },
        DialogueTree { name: "the_cold_one",
            criteria: &[Has("other:status:hard")],
            root: &VIRTUE_COLD_GREET, rejoin: Some(&SMALL_CHAT) },
        DialogueTree { name: "village_smalltalk", criteria: &[],
            root: &SMALL_GREET, rejoin: Some(&SMALL_CHAT) },
    ];
    // the wounds wear (E3-4, #1425): the slip and the neighbor's remark,
    // late in the library so a circumstance or a bond opens the talk
    lib.extend(crate::game::fact_content::tell_trees().iter().cloned());
    lib
}

/// The noun a speaker's work goes by in talk (`{trade}` / `{other_trade}`
/// slots, #1181): their craft's haunts if they have one, else whatever the
/// day's wage is for someone of their standing
pub fn trade_noun(s: &Speaker) -> &'static str {
    if let Some(tr) = s.status.iter().copied().chain(s.trade)
        .find(|st| matches!(*st,
        "smith" | "miller" | "baker" | "weaver" | "carpenter" | "mason"
        | "shepherd" | "brewer" | "taverner" | "beekeeper" | "herb_wife"
        | "fisher" | "charcoal_burner" | "farmer" | "tailor" | "luthier"
        | "printer" | "glazier" | "knight" | "priest" | "perfumer"
        | "painter" | "shoemaker" | "bookbinder" | "miner" | "vintner"
        | "distiller" | "cooper" | "alewife")) {
        return match tr {
            "smith" => "the forge",
            "miller" => "the mill",
            "baker" => "the oven",
            "weaver" => "the loom",
            "carpenter" => "the timberyard",
            "mason" => "the stone yard",
            "shepherd" => "the flock",
            "brewer" => "the vats",
            "taverner" => "the tavern",
            "beekeeper" => "the skeps",
            "herb_wife" => "the stillroom",
            "fisher" => "the weir",
            "farmer" => "the fields",
            "tailor" => "the tailor's",
            "luthier" => "the workshop",
            "printer" => "the press",
            "glazier" => "the glazier's",
            "knight" => "the lists",
            "priest" => "the parish",
            "perfumer" => "the scents",
            "painter" => "the panels",
            "shoemaker" => "the last",
            "bookbinder" => "the binding",
            "miner" => "the pit",
            "vintner" => "the vines",
            "distiller" => "the still",
            "cooper" => "the hoops",
            "alewife" => "the kettle",
            _ => "the mounds",
        };
    }
    match s.ideology {
        "clergy" => "the parish",
        "merchant" => "the ledger",
        "lord" => "the manor",
        _ => "the day's wage",
    }
}

/// The phrasing a profile fact lands in for this speaker (#1629): each
/// fact keeps a few wordings and the pick rides the speaker's name, so
/// two villagers sharing a trait don't share all of its phrasing
fn profile_line(pool: &[&'static str], name: &str, salt: u64) -> &'static str {
    let seed = name.bytes().fold(0u64, |a, b|
        a.wrapping_mul(0x0100_0000_01B3).wrapping_add(b as u64));
    pool[(crate::game::persona::draw(seed, salt) * pool.len() as f32) as usize % pool.len()]
}

/// A one-line backstory from a speaker's profile
pub fn backstory(s: &Speaker) -> String {
    let mut parts: Vec<String> = vec![];
    // the lead's their real standing: a craft named in their statuses
    // outranks the ideology's guess, and a child is a child
    let trade = s.status.iter().copied().find(|st| matches!(*st,
        "smith" | "miller" | "baker" | "weaver" | "carpenter" | "mason"
        | "shepherd" | "brewer" | "taverner" | "beekeeper" | "herb_wife"
        | "fisher" | "charcoal_burner" | "farmer" | "tailor" | "luthier"
        | "printer" | "glazier"));
    if s.status.iter().any(|st| *st == "child") {
        parts.push(profile_line(&[
            "{} is still a child.",
            "{} has yet to see a dozen summers.",
            "{} is young yet, and quicker for it."], &s.name, 1)
            .replace("{}", &s.name));
    } else if let Some(tr) = trade {
        parts.push(profile_line(&[
            "{} is the village {t}.",
            "The village {t} is {}.",
            "{} keeps the village's {t} trade."], &s.name, 2)
            .replace("{}", &s.name).replace("{t}", &tr.replace('_', " ")));
    } else {
        parts.push(profile_line(match s.ideology {
            "clergy" => &[
                "{} serves the village church.",
                "{} keeps the parish's book and bell.",
                "{} tends the village's prayers."] as &[&'static str],
            "merchant" => &[
                "{} trades on the road between villages.",
                "{}'s coin moves faster than their boots.",
                "{} carries wares up and down the road."],
            "lord" => &[
                "{} holds the manor's leases.",
                "{}'s rents come due on the quarter days.",
                "{} speaks for the manor hereabouts."],
            _ => &[
                "{} works the fields for a day's wage.",
                "{} takes the day's wage where it's offered.",
                "{} keeps a day's work where the fields want it."],
        }, &s.name, 3).replace("{}", &s.name));
    }
    for t in &s.traits {
        let pool: &[&'static str] = match t {
            Trait::Lazy => &[
                "They'd sooner watch the clouds than the plough.",
                "Work sits down beside them and they let it wait.",
                "A sunny bench argues better than a hoe."],
            Trait::Ambitious => &[
                "They mean to be more than they were born.",
                "Their eyes are always a rung up the ladder.",
                "They keep count of who's risen, and mean to pass them."],
            Trait::Kind => &[
                "They're the first with a loaf when a house is bare.",
                "A neighbor's empty shelf empties theirs first.",
                "They'd give the coat off their back and call it lending."],
            Trait::Stingy => &[
                "Their purse strings are knotted tight.",
                "A coin leaves their hand the way teeth leave a jaw.",
                "They count the crumbs twice before the second's swept."],
            Trait::Pious => &[
                "They never miss a service.",
                "The bell knows their step better than their own door does.",
                "Every saint's day finds them first at the door."],
            Trait::Hotheaded => &[
                "Their temper's shorter than a winter's day.",
                "A wrong word and the kettle's off.",
                "They burn fast and cool slow."],
            Trait::Gregarious => &[
                "They know every soul and every soul's business.",
                "No lane's gossip gets past them twice.",
                "They've a hello for every door and a question for every hello."],
            Trait::Proud => &[
                "They bow to no one.",
                "Their chin's set a half-inch higher than the rest of the room.",
                "They'd rather lose than ask."],
            Trait::Steady => &[
                "They'll finish the furrow before they curse the stone.",
                "The plough's line runs true under their hands, whatever the weather.",
                "Slow to start, impossible to stop."],
            Trait::Wary => &[
                "They weigh a stranger twice before a word.",
                "New faces get the back of their eye first.",
                "Trust comes to them the long way round."],
            Trait::Stolid => &[
                "They keep their own counsel, and keep it well.",
                "Their face tells you the weather, not what they're thinking.",
                "Ask twice and you'll still get the same short answer."],
            Trait::Cruel => &[
                "Their hand is heavy and their tongue heavier.",
                "The dog knows to give their path a wide berth.",
                "A small thing done mean gives them a good morning."],
            Trait::SelfAbsorbed => &[
                "They'll talk your ear off and never once ask after you.",
                "The world's a mirror they happen to live in.",
                "Every tale you start becomes a tale of theirs."],
        };
        parts.push(profile_line(pool, &s.name, *t as u64 + 10).into());
    }
    for st in &s.status {
        if *st == "child" { continue; }
        let salt = st.bytes().fold(40u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
        let pool: &[&'static str] = match *st {
            "sick" => &[
                "A cough's had them these two weeks.",
                "There's a wheeze in their step this season."],
            "hungry" => &[
                "The store ran out and the belly knows it.",
                "Their belt's gone a notch inward this winter."],
            "grieving" => &[
                "They buried one of their own last week.",
                "A fresh grave sits behind their eyes."],
            "drunk" => &[
                "Last night's cup is still on them.",
                "The tavern's tap still has their ear from yestereve."],
            "cold" => &[
                "Their hearth went out and fuel is spent.",
                "Their breath fogs indoors and the woodpile's bare."],
            "elder" => &[
                "Eighty winters of village memories are in their bones.",
                "They remember three lords back, and say the first was worse."],
            "newborn" => &[
                "A new babe was born to their house this month.",
                "There's a cradle by their hearth keeping everyone up."],
            // the soul's name, read not as a score but as what folk
            // have learned to expect of them (#950)
            "kind_soul" => &[
                "Their table's never bare to a neighbor.",
                "Folk name them when the pot runs low."],
            "upright" => &[
                "They deal straight where another man would bend.",
                "Their word's worth a seal, ask anyone."],
            "steady" => &[
                "Nothing much ruffles them.",
                "You could set a plough-line by their temper."],
            "shrewd" => &[
                "They weigh a thing thrice before they speak on it.",
                "They hear the price before they hear the tale."],
            "devout" => &[
                "They keep the holy days like a second trade.",
                "The fast days find them already fasting."],
            "hopeful" => &[
                "They expect the morning to bring good.",
                "Ask them and the season's always about to turn."],
            "bold" => &[
                "They step forward when the rest step back.",
                "First through a door nobody else wanted to open."],
            "hard" => &[
                "Their door stays shut when a neighbor's pot runs empty.",
                "A plea tends to die on their doorstep."],
            "smith" => &[
                "They work the village anvil and forge.",
                "The forge's smoke marks their door."],
            "miller" => &[
                "They run the watermill at the weir.",
                "The mill wheel's never far from their talk."],
            "baker" => &[
                "They tend the communal bread oven.",
                "Their mornings start with the oven's first fire."],
            "weaver" => &[
                "They weave russet cloth on the loom.",
                "The loom's clatter keeps their house's time."],
            "carpenter" => &[
                "They square timber and cut mortise joints.",
                "Their joints hold without a nail to brag of."],
            "mason" => &[
                "They cut freestone and build field walls.",
                "Their walls stand straighter than the lane."],
            "shepherd" => &[
                "They herd ewes and lambs on the common pasture.",
                "The flock knows their whistle on the common."],
            "brewer" => &[
                "They brew ale and tend the village vats.",
                "The village vats answer to their mash fork."],
            "taverner" => &[
                "They keep the tavern on the green and buy the drink trades out by the barrel.",
                "The tavern on the green keeps them, barrel and bench."],
            "beekeeper" => &[
                "They keep straw skeps in the clover meadow.",
                "The skeps hum for them in the clover meadow."],
            "herb_wife" => &[
                "They gather roots and brew fever draughts.",
                "Half the village's cures started in their basket."],
            "fisher" => &[
                "They set eel traps in the river current.",
                "The river pays them in eels."],
            "charcoal_burner" => &[
                "They watch the slow-burning earth mounds in the deep wood.",
                "The wood's smoke follows them home."],
            // the powers they run with (#969): the talk can name the pull,
            // the profile says it in words, not tokens
            "parish_faction" => &[
                "They keep close to the church folk.",
                "The church porch is where their counsel gathers."],
            "market_faction" => &[
                "They keep close to the market folk.",
                "The market stalls know them by name."],
            "guild_faction" => &[
                "They keep close to the guild's masters.",
                "The guild's tables set their company."],
            "kin_faction" => &[
                "Their kin's name opens doors here.",
                "Say their surname and doors come open."],
            "seeker" => &[
                "They're forever turning the big questions over.",
                "Ask them the hour and you'll get the meaning of the day."],
            "mystic" => &[
                "They speak of God like a neighbor they've met.",
                "They talk to the Almighty like he's at the fence."],
            "sage" => &[
                "Their faith seems to hold the whole wide world.",
                "Their belief has room for every doubt put to it."],
            // a status the talk keys on that says nothing about who they
            // are (bond stage, already-spoken, visitor plumbing) stays
            // out of the prose
            _ => continue,
        };
        parts.push(profile_line(pool, &s.name, salt).into());
    }
    parts.join(" ")
}

/// The bond ladder, spoken aloud: how a villager meets someone who has
/// The bond ladder's lowest rung (#1228): a face they've seen about is a
/// neighbor, not a stranger - familiar talk, a notch warmer than the
/// weather and a notch colder than knowing their mind. The rungs above:
/// heard their self-interest (known), earned their trust (follower), or
/// taken them on as a leader beside them (co-leader)
const ACQUAINTED_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &[
        "Morrow, {other} - a fine day to be out, isn't it?",
        "{other} - good to see a familiar face about.",
        "Well met, {other} - how does the morning find {2:obj}?",
        "Neighbor! Out early like the rest of us, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Fair enough, and to {2:obj}.", "Well enough - and {2:own}?", "Out and about early today."],
            voice: &[],
            then: Some(NEIGHBOR_CHAT), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            respond: None,
            lines: &["Morrow. I'd best keep moving.", "Aye. Good day to {2:obj}."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};

/// Where a neighbor's hello goes: lanes, roofs, wells - the small world
/// two folk share once they've nodded a few times
const NEIGHBOR_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &[
        "The lane's quieter since the rain, thank goodness.",
        "{2:poss} roof's holding better than mine in this wind, I'd wager.",
        "Folk say the well water runs sweet again.",
        "Keep an eye on the sky - there's weather in it yet.",
        "Saw {2:poss} lass helping at the press yesterday.",
    ],
    voice: &[
        // the parish's own weather, mid-chain (#1696): the lanes and
        // roofs talk reads lean while the village's under a weight
        Voice { criteria: &[Has("village:hard")], lines: &[
            "Thin days on every lane, {other} - {2:have} {2:subj} felt it?",
            "The lane's quiet - folk keep close when times run lean."] },
    ],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Aye, the village turns on small things.", "As do we all, neighbor.", "Good news, if it keeps."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};

/// Two members of one cause meet in the street (#1228): the register
/// drops to a quiet word - what's coming, kept close
const FELLOW_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "the_cause"],
    calls: &[
        "Quiet word, {other} - the meet's not far off now.",
        "{other} - the cause keeps, and so do we.",
        "Good to see {2:obj} about, {other} - folk are talking.",
        "Between us and the wall, {other} - soon, I think.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["the_cause"],
            respond: None,
            lines: &["Aye - low voices till then.", "The cause keeps us both.", "Say the word and I'm ready."],
            voice: &[],
            then: Some(FELLOW_CHAT), end: None },
    ],
};

const FELLOW_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["the_cause"],
    calls: &[
        "Folk at the well ask what's brewing - I keep it close.",
        "The others are ready when the word comes.",
        "I've kept my promise - has {2:own} held?",
        "Word travels - keep {2:poss} voice low round the square.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["the_cause"],
            respond: None,
            lines: &["Mine's held. It'll hold.", "Kept and quiet both.", "Low and steady - it'll hold."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Proud, 2.0)],
            act: Act::Remark, concepts: &["the_cause"],
            respond: None,
            lines: &["Let them guess - they'll know soon enough.", "The walls have ears, it's true."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};

const KNOWN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &[
        "Ah, {other} - {2:subj} {2:know} well what weighs on me by now.",
        "{2:subj} {2:ask} after my cares like an old friend, {other}.",
        "The stranger who asks twice is a stranger no longer, eh {other}?",
        "{other}! {2:subj} {2:be} coming to know my mind, I'll grant.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["And I mean to do right by what I've heard.", "I listen because it matters, friend.", "{2:poss} troubles are the village's troubles."],
            voice: &[],
            then: Some(BOND_CHAT), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            respond: None,
            lines: &["Knowing and doing are two different things.", "I'd not presume to know all of it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};

const FOLLOWER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &[
        "{other}! Good day - I was hoping I'd see {2:obj} about.",
        "There {2:subj} {2:be}, {other}. Word is {2:subj} {2:be} still talking to folk.",
        "A fair morning to {2:obj}, {other} - and to the work we share.",
        "I saved {2:obj} a thought or two since last we spoke, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Speak it, then - I came to hear.", "{2:subj} {2:be} not alone in it, remember.", "Tell me what folk are saying."],
            voice: &[],
            then: Some(BOND_CHAT), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Lazy, 2.0)],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["The work keeps, friend - tell me of {2:poss} day instead.", "Rest a moment first."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};

const COLEADER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &[
        "We stand in it together now, {other} - what news?",
        "{other}, good - I've a mind on what comes next.",
        "The others ask when we meet again, {other}.",
        "{2:subj} and I have work to plan, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Let's talk it through - what's on {2:poss} mind?", "Then let's settle the next step."],
            voice: &[],
            then: Some(BOND_CHAT), end: None },
    ],
};

/// Where a bond talk goes after the greet: what follows in their minds
const BOND_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "the_cause"],
    calls: &[
        "I still think on what was said of the Reeve.",
        "{2:subj} {2:will} call a meeting soon, aye? My kitchen's ready when the word comes.",
        "Folk ask me why I listen to {2:obj} - I tell them {2:subj} {2:listen} first.",
        "Say the word and I'll bring my cousin too.",
        "The talk goes further than me, mind - folk repeat what {2:subj} {2:ask} about.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["the_cause"],
            respond: None,
            lines: &["We take it up together, then.", "It'll be soon - keep the word to {2:self} till then.", "{2:subj} {2:will} not stand alone in it."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
        Reply { criteria: &[], weight: 0.40, bias: &[(Lazy, 2.0)],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["Patience - these things want doing right.", "Not yet, but the day's coming."],
            voice: &[],
            then: Some(SMALL_FADE), end: None },
    ],
};


// ---------- visitors on the road (#1168) ----------
// Every traveller kind gets a voice through its archetype tree; a few
// kinds carry their own (2-criteria, kind-slug) trees where the voice or
// the stakes are their own. The deal fork is always Act::Agree; a rebuff
// that can send them off is Act::PushBack - session code keys the
// mechanical outcome on those, so the forks stay data.

/// What follows the player's "what news of the road?": a traveller's
/// answer, then the talk runs out
const VIS_ROAD_NEWS: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["the_road", "smalltalk"],
    calls: &[
        "Mud, mostly. And a toll on every bridge that has a keeper.",
        "War talk in the north, they say - but they always say that.",
        "A fair behind us, a fair ahead - that's all a road is.",
        "Folk are hungry up the dale. Hungrier than here, I mean.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Safe road to {2:obj}, then.", "Walk soft - and thanks for the word."],
            voice: &[],
            then: None, end: Some(Farewell) },
        Reply { criteria: &[], weight: 0.4, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["Anything worth avoiding up the way?"],
            voice: &[],
            then: Some(VIS_ROAD_TAIL), end: None },
    ],
};

const VIS_ROAD_TAIL: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["the_road"],
    calls: &[
        "The ford's up with the rain - go the long way round.",
        "A press gang was about last month. Keep the lads close.",
        "Nothing a stout stick won't answer for.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Noted - and a good road to {2:obj}.", "I'll pass the word along."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

/// The visitor's own "done, then" after the deal lands (the session
/// writes the actual trade onto the talk's record)
const VIS_DEAL_DONE: Exchange = Exchange {
    act: Act::Agree,
    concepts: &["deal"],
    calls: &[
        "Done, then - and a fair bargain to both of us.",
        "{2:own}, and keep it well.",
        "The road thanks {2:obj}, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["A good day to {2:obj} - and to {2:poss} road.", "Fair dealing. God speed."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_AGAIN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &[
        "Back again, {other}? The road's no shorter.",
        "{2:subj} again. I've little new since last we spoke.",
        "Still standing here, {other} - as {2:subj} {2:see}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 3.0, bias: &[],
            act: Act::Agree, concepts: &["deal"],
            respond: None,
            lines: &["The same again, then - I'll take it."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Just passing - go well."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_TRADER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade"],
    calls: &[
        "Wares from far off, {other} - a fair price, and the road's own.",
        "Come see what the pack holds - I've travelled far for folk like {2:obj}.",
        "The village bench hasn't what I carry - a look costs nothing.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[],
            act: Act::Agree, concepts: &["deal", "trade"],
            respond: None,
            lines: &["Show me - I'll buy, if the price holds.", "Aye, the road's price then."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["What news on the road, friend?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Not today - my purse stays shut."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_CRAFT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "work"],
    calls: &[
        "A tradesman passing, {other} - honest work for honest coin.",
        "{2:subj} {2:have} need of my trade, or know someone who has - which is it?",
        "I do the work the village hasn't a hand for - a day, no more.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[],
            act: Act::Agree, concepts: &["deal", "work"],
            respond: None,
            lines: &["Aye - there's work here wants doing.", "Done - do the job and take the coin."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["What {2:have} {2:subj} seen on the road?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["We've no need - good day to {2:obj}."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_PERFORMER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "mirth"],
    calls: &[
        "A smile costs nothing, {other} - but a coin makes the show better!",
        "Gather round! Wonders for a penny, marvels for two!",
        "{2:subj} {2:have} not lived till {2:subj} {2:have} seen what my hands can do, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Gregarious, 2.0)],
            act: Act::Agree, concepts: &["deal", "mirth"],
            respond: None,
            lines: &["A coin for the hat - show us, then.", "Make us laugh, and it's {2:own}."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["Where does the road take {2:poss} show?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["No coin from me today - move along."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_RELIGIOUS_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "faith"],
    calls: &[
        "Peace on this house, {other} - and on all who dwell in it.",
        "The saints keep the traveller - {2:will} {2:subj} keep one tonight?",
        "A blessing on {2:poss} fields, friend - freely given, freely received.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Pious, 2.0)],
            act: Act::Agree, concepts: &["deal", "faith"],
            respond: None,
            lines: &["Take this - and a blessing back on {2:obj}.", "For the road, brother."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road", "faith"],
            respond: None,
            lines: &["Where does {2:poss} pilgrimage lead?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Go in peace - we've little to spare."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};





const VIS_MYSTIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "omens"],
    calls: &[
        "The stars told me a seeker would meet me here - is it {2:obj}, {other}?",
        "{2:poss} face, {other} - it carries a question. Cross my palm and I'll answer it.",
        "I deal in what the wise pretend not to know - have a care, but hear me.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Pious, 0.5)],
            act: Act::Agree, concepts: &["deal", "omens"],
            respond: None,
            lines: &["Read what {2:subj} {2:see}, then - here's the coin.", "Tell me my fortune, if {2:subj} {2:can}."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["omens"],
            respond: None,
            lines: &["What {2:do} {2:subj} see for this village?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 2.0)],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["The priest says {2:poss} kind brings ill luck - go."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_CALLER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "the_road"],
    calls: &[
        "A fair day for the road, {other} - God keep {2:poss} village.",
        "Just passing through, friend - the world's wide and I'm halfway across it.",
        "Well met, {other} - a kind face is rare on this road.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.5, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["Where {2:be} {2:subj} bound, traveller?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Safe road to {2:obj}, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

// ----- kind trees: voices and stakes that are their own -----

const VIS_DESERTER_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["war"],
    calls: &[
        "Three winters in the mud, and my captain sold our boots for brandy. I ran when the fever didn't.",
        "They hanged a boy of fourteen for lagging. I was next in line to lag.",
        "I saw the battle, {other}. Whatever they tell {2:obj} of it - it was worse.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Go quiet, then - and God speed {2:obj}."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_DESERTER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "war", "fear"],
    calls: &[
        "Keep {2:poss} voice down, {other} - there's a price on my head.",
        "I didn't run. I just stopped dying for men I'll never meet.",
        "{2:subj} {2:will} not turn me in, {2:will} {2:subj}? I only want to eat and sleep.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["deal", "mercy"],
            respond: None,
            lines: &["{2:subj} {2:will} be safe here a while - keep out of sight.", "Rest, then. I'll say nothing."],
            voice: &[],
            then: Some(VIS_DESERTER_TALE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["The watch should know {2:subj} {2:be} here - go, before I shout.", "{2:subj} {2:bring} danger on us all - be gone."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_LEPER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "want", "sickness"],
    calls: &[
        "Unclean! Keep {2:poss} distance, {other} - but spare a coin if God lives in {2:obj}.",
        "The bell announces us, but mercy needn't wait for it.",
        "We mean no harm - only bread, and to be seen as men.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 3.0)],
            act: Act::Agree, concepts: &["deal", "charity"],
            respond: None,
            lines: &["Here - food, and a coin with it.", "Stay where {2:subj} {2:be} - I'll bring something."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Stingy, 2.0), (Hotheaded, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["Out - {2:subj} {2:will} not plague this village.", "Back down the road, before I call folk out."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_REFUGEE_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["war", "the_road"],
    calls: &[
        "Three days north - the houses still burn, they say. We'd nothing left to bury.",
        "Where we came from doesn't matter now. Where we're going is a beggar's road.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["May the road be kinder to {2:obj} than it was.", "Go with God - and I'm sorry."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_REFUGEE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "want", "war"],
    calls: &[
        "We lost everything, {other} - the raiders took the village and the winter took the rest.",
        "A crust is all we ask. The little ones haven't eaten since yesterday.",
        "We were farmers once, like {2:obj}. The road owns us now.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 3.0), (Stingy, 0.3)],
            act: Act::Agree, concepts: &["deal", "charity"],
            respond: None,
            lines: &["Come - there's bread and a fire for the children.", "Take this - it's not much, but it's {2:own}."],
            voice: &[],
            then: Some(VIS_REFUGEE_TALE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["The village can't carry more mouths - move on."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_BEGGAR_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "want"],
    calls: &[
        "Spare a crust, {other} - the road's taken everything but my hunger.",
        "Alms, friend - I've not eaten since the last village but one.",
        "A man the wars forgot, {other} - a coin's a day's bread to me.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 3.0), (Stingy, 0.3)],
            act: Act::Agree, concepts: &["deal", "charity"],
            respond: None,
            lines: &["Here - a crust and a coin, and God keep {2:obj}.", "Take this - no man should beg for bread."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Stingy, 2.0), (Hotheaded, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["Begone - this village feeds its own poor.", "Down the road with {2:obj} - I've nothing spare."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_HERETIC_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["faith"],
    calls: &[
        "Why does a poor man tithe to a rich altar? Ask that, and {2:subj} {2:be} one of us already.",
        "The old ways kept this valley before the church did. Ask the stones, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll think on it - but quietly.", "Enough - I've heard what I came to hear."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_HERETIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "faith"],
    calls: &[
        "They preach one thing in the church, {other} - but who built the church?",
        "The truth walks with a staff, not a censer. Hear me, and judge.",
        "I ask questions the priest won't answer - {2:do} {2:subj} dare to listen?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Ambitious, 1.5), (Pious, 0.4)],
            act: Act::Agree, concepts: &["faith"],
            respond: None,
            lines: &["Say it, then - I'll judge for myself.", "I listen. Carefully."],
            voice: &[],
            then: Some(VIS_HERETIC_TALE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 2.0)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["{2:subj} {2:speak} poison - the priest should hear of {2:obj}.", "This village keeps the faith - be gone."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_RECRUITER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "war"],
    calls: &[
        "The King needs stout arms, {other} - two silver for signing, and meat every day!",
        "Honest service, steady pay - better than staring at a plough's arse-end.",
        "Who among {2:obj} wants more than this village can ever give? The bounty waits.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Ambitious, 2.5)],
            act: Act::Agree, concepts: &["deal", "war"],
            respond: None,
            lines: &["Two silver, {2:subj} {2:say}? I'm listening.", "Tell me more of the service."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["war"],
            respond: None,
            lines: &["Where's the fighting this year?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["{2:subj} {2:will} get no one here - move along, sergeant."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_FORTUNE_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["omens"],
    calls: &[
        "A dark cloud over the granary - keep {2:poss} stores dry and {2:poss} locks oiled.",
        "Someone here will rise, {other} - and the village will rise with them, or fall.",
        "I see a hard winter and a warm hearth - one must be earned before the other comes.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Keep {2:poss} omens - I'll make my own luck."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_FORTUNE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "omens"],
    calls: &[
        "Cross my palm with silver, {other} - the cards have been talking about {2:obj}.",
        "I saw {2:obj} coming before {2:subj} {2:did}. A penny tells {2:obj} what else I saw.",
        "The stars over this village shift oddly tonight - shall I read them?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[],
            act: Act::Agree, concepts: &["deal", "omens"],
            respond: None,
            lines: &["Read my fortune, then.", "Here's the coin - tell me true."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["omens"],
            respond: None,
            lines: &["What {2:do} {2:subj} see for the village?"],
            voice: &[],
            then: Some(VIS_FORTUNE_TALE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 1.5)],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I keep my fate to myself - good day."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

// ---- the sheep in the cradle (#1254) ----
// A stolen ewe folded into a cradle and dyed dark. The bluffing house
// answers with evasions until a kindness lifts the lid; the robbed one
// names the pot; and once the name's held, the squeeze sits in his talk.

/// The bluff tree's name - session hooks on it for the gift pick
pub const EWE_TREE: &str = "the_sheep_in_the_cradle";
/// The squeeze tree's name - session hooks on it for the hush pick
pub const EWE_PRICE_TREE: &str = "the_ewes_price";
/// The robbed one's tree's name - session hooks on it for the naming
pub const EWE_VICTIM_TREE: &str = "the_missing_ewe";
/// The pick text that lifts the lid - a kindness the house can't hold
pub const EWE_GIFT: &str = "I've brought a little something for the house.";
/// The pick text that spends the held name for hush money
pub const EWE_SQUEEZE: &str = "We both know what's in {2:poss} cradle.";
/// The pick text that gets the robbed one to name the thief
pub const EWE_WHOSE_POT: &str = "Whose pot, then? Say it.";

const EWE_CRACKED: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "guilt"],
    calls: &[
        "...she walked in herself, so help me. What's done is done.",
        "{2:subj} {2:saw} nothing. The house's luck is all it is.",
        "Take it up with the cradle, then - not with me.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll think on what I've seen."],
            voice: &[],
            then: None, end: Some(Farewell) },
        Reply { criteria: &[], weight: 0.8, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Good day to the house."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_STONEWALL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "bluff"],
    calls: &[
        "I've said all I'll say - the door's there.",
        "{2:subj} {2:will} have the watch called on {2:obj}, pushing at folk's doors so.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["gift", "ewe"],
            respond: None,
            lines: &["I've brought a little something for the house."],
            voice: &[],
            then: Some(EWE_CRACKED), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_DENY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "bluff"],
    calls: &[
        "No ewe here but the one in my soup.",
        "Folk talk. I mind my own kettle.",
        "Search if {2:subj} {2:like} - {2:subj} {2:will} find a babe asleep and nothing more.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["gift", "ewe"],
            respond: None,
            lines: &["I've brought a little something for the house."],
            voice: &[],
            then: Some(EWE_CRACKED), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["ewe", "accusation"],
            respond: None,
            lines: &["{2:subj} {2:be} lying, and we both know it.", "The dye's on {2:poss} hands, friend."],
            voice: &[],
            then: Some(EWE_STONEWALL), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_EVADE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "bluff"],
    calls: &[
        "Quiet house, aye - the babe keeps us up all night.",
        "Folk pass, folk talk - there's no harm in a closed door.",
        "The wife's not herself since the babe came. We keep to ourselves.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["gift", "ewe"],
            respond: None,
            lines: &["I've brought a little something for the house."],
            voice: &[],
            then: Some(EWE_CRACKED), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["ewe", "accusation"],
            respond: None,
            lines: &["Word is a ewe walked into this house last night."],
            voice: &[],
            then: Some(EWE_DENY), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["ewe", "bluff"],
    calls: &[
        "Hush - the babe sleeps. Say {2:poss} piece at the door.",
        "My wife's laid up - come another day.",
        "Whatever it is, make it short.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["gift", "ewe"],
            respond: None,
            lines: &["I've brought a little something for the house."],
            voice: &[],
            then: Some(EWE_CRACKED), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["ewe", "accusation"],
            respond: None,
            lines: &["Word is a ewe walked into this house last night."],
            voice: &[],
            then: Some(EWE_DENY), end: None },
        Reply { criteria: &[], weight: 0.8, bias: &[],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["Just passing - quiet house today.", "Fine weather for the fold."],
            voice: &[],
            then: Some(EWE_EVADE), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_VICTIM_NAMES: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["ewe", "theft"],
    calls: &[
        "Him - I saw the dye-stain on his hands.",
        "It walked into his house and never came out. Ask me again and I'll say it louder.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["ewe", "promise"],
            respond: None,
            lines: &["I'll look into it.", "I'll pay the house a visit."],
            voice: &[],
            then: None, end: Some(Farewell) },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Keep it quiet until I do."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_VICTIM_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["ewe", "theft", "loss"],
    calls: &[
        "{other}! My ewe's gone - gone from the fold, and I know whose pot it's in.",
        "A word, {other} - my best ewe's walked off, and not on her own legs.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["ewe", "theft"],
            respond: None,
            lines: &["Whose pot, then? Say it."],
            voice: &[],
            then: Some(EWE_VICTIM_NAMES), end: None },
        Reply { criteria: &[], weight: 0.8, bias: &[],
            act: Act::Agree, concepts: &["ewe", "promise"],
            respond: None,
            lines: &["We'll find her - I promise it."],
            voice: &[],
            then: Some(EWE_VICTIM_NAMES), end: None },
        Reply { criteria: &[], weight: 0.4, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I've no time for lost sheep today."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_PAYS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "hush"],
    calls: &[
        "Take it - and we never spoke.",
        "Here. It's all I've put by. Say nothing.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["We never spoke."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_TENSION: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "hush"],
    calls: &[
        "There's nothing to say. The house sleeps early.",
        "Mind {2:poss} own pots, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["ewe", "hush"],
            respond: None,
            lines: &["We both know what's in {2:poss} cradle."],
            voice: &[],
            then: Some(EWE_PAYS), end: None },
        Reply { criteria: &[], weight: 0.6, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_PRICE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["ewe", "hush"],
    calls: &[
        "{other}. What brings {2:obj} back here?",
        "Aye - what {2:do} {2:subj} want of me now?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["ewe", "hush"],
            respond: None,
            lines: &["We both know what's in {2:poss} cradle."],
            voice: &[],
            then: Some(EWE_PAYS), end: None },
        Reply { criteria: &[], weight: 0.8, bias: &[],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["Just checking the fold is quiet.", "Fine day for the flocks."],
            voice: &[],
            then: Some(EWE_TENSION), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

pub const NOAH_ROW_TREE: &str = "the_noah_row";
/// The mediator's word - the pick that talks one down (#1257)
pub const NOAH_PEACE: &str = "Peace - the whole street's watching.";
/// Backing the one you're talking to (#1257)
pub const NOAH_SIDE_THEM: &str = "{2:subj} {2:have} the right of it - hold {2:poss} line.";

const NOAH_LAMP: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["peace", "street"],
    calls: &[
        "...the whole street, aye. And me giving it a show.",
        "Huh. Let them find another supper, then.",
        "{2:subj} {2:be} not wrong. Not about the watching, anyway.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Take it home, now. The door's the place for it."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const NOAH_BACKED: Exchange = Exchange {
    act: Act::Agree,
    concepts: &["right", "row"],
    calls: &[
        "There - someone with sense! Tell THEM that.",
        "I knew it. I bloody knew I had the right of it.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Best keep walking - the row's not done."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const NOAH_GREET: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["row", "street"],
    calls: &[
        "What? The whole street's had its look already - say {2:poss} piece.",
        "I'm in no mood for callers, and that's the truth of it.",
        "If {2:subj} {2:have} come to gawp, the gawping's over there with the rest.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["peace", "street"],
            respond: None,
            lines: &[NOAH_PEACE],
            voice: &[],
            then: Some(NOAH_LAMP), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["side", "row"],
            respond: None,
            lines: &[NOAH_SIDE_THEM],
            voice: &[],
            then: Some(NOAH_BACKED), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

// the false traveller (#1258): a pedlar's pack that never sells - the
// motive trees key on the visitor+false-traveller+motive statuses the
// session pushes, so the wrong questions show up in his lines and
// nowhere else. The accusation puts the plays on the card; the card
// asks which lie he carries home.
/// The spy's tree's name - the session hooks its accusation pick
pub const FT_SPY_TREE: &str = "the_ears_in_the_pack";
/// The thief's tree's name
pub const FT_THIEF_TREE: &str = "the_full_larder";
/// The runaway's tree's name
pub const FT_RUN_TREE: &str = "the_quick_road";
/// The pick that calls the disguise: the plays go on the card
pub const FT_ACCUSE: &str = "{2:subj} {2:be} no pedlar. Who sent {2:obj}?";
/// The pick that calls the runaway's disguise
pub const FT_ACCUSE_RUN: &str = "Those aren't a tinker's questions. Who's after {2:obj}?";

const FT_SPARK: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["found", "sharp"],
    calls: &[
        "...sharp eyes. Sharper than a market's used to.",
        "A pedlar's wares are what a pedlar says they are - and what he carries home is his own.",
        "Careful with the naming, friend. Names ride home.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["We'll see what rides home."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const FT_SPY_TELL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["names", "asking"],
    calls: &[
        "Names, friend - who stands with whom when the doors are shut?",
        "And which houses meet nights? Good to know {2:poss} way, on the road.",
        "Who's aggrieved here, eh? The folk a lord should know about - for a price on a pot, mind.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "pedlar"],
            respond: None,
            lines: &[FT_ACCUSE],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to {2:poss} pack."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const FT_SPY_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["wares", "asking"],
    calls: &[
        "Pins and pots, friend - though first, tell me: who's about this village worth the naming?",
        "A pedlar's pack and a fair price - say, which houses keep their doors shut after dark?",
        "Odds and ends! - and tell me, friend, is there talk here? Talk a man should hear?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "pedlar"],
            respond: None,
            lines: &[FT_ACCUSE],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["What news off the road?", "Show me the pack, then."],
            voice: &[],
            then: Some(FT_SPY_TELL), end: None },
    ],
};

const FT_THIEF_TELL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["larder", "asking"],
    calls: &[
        "And the doors - which hold what? A pedlar likes to know where the buying's best.",
        "A fat larder, a full cradle, a purse left loose - which house has all three?",
        "Say there's a house worth a pedlar's whole stock - which door {2:would} {2:subj} point him at?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "pedlar"],
            respond: None,
            lines: &[FT_ACCUSE],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave {2:obj} to {2:poss} pack."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const FT_THIEF_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["wares", "asking"],
    calls: &[
        "Pots and pins, all a pedlar carries - which house here keeps the fullest larder?",
        "Fine village, this. Fat purses like quiet houses, do they not?",
        "Odds and ends! - and which door's got a purse worth the knocking, eh?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "pedlar"],
            respond: None,
            lines: &[FT_ACCUSE],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["What news off the road?", "Show me the pack, then."],
            voice: &[],
            then: Some(FT_THIEF_TELL), end: None },
    ],
};

const FT_RUN_TELL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["road", "castle", "asking"],
    calls: &[
        "The castle, friend - when do its riders pass, and which way?",
        "If a man wanted forgetting by a lord - which road {2:would} {2:subj} tell him?",
        "No lord's men about, are there? None that watch the road, anyway.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "tinker"],
            respond: None,
            lines: &[FT_ACCUSE_RUN],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Safe road to {2:obj}, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const FT_RUN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["wares", "road", "asking"],
    calls: &[
        "Tinker, me - mender of pans. Tell me quick: which road leaves this place fastest?",
        "Pans mended, kettles patched - and tell me, do the castle's men ride through often?",
        "Odds and ends, friend - and is there work for a pair of hands that's only passing?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["accuse", "tinker"],
            respond: None,
            lines: &[FT_ACCUSE_RUN],
            voice: &[],
            then: Some(FT_SPARK), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["What news off the road?", "Pans to mend, then? I've a pot."],
            voice: &[],
            then: Some(FT_RUN_TELL), end: None },
    ],
};
