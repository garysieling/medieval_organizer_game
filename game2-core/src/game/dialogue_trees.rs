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
    "As thou sayest, friend.",
    "True enough - the fields don't gossip.",
    "Aye, there is truth in what thou sayest.",
    "Thou hast spoken the plain heart of the matter.",
];

const SMALL_FADE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["As thou sayest, friend.", "True enough - the fields don't gossip.", "Aye, words won't fill the storehouse.", "We have spoken our piece for today."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair day to thee, {other} - 'twas good to pass the time.", "Go well, {other}, and mind the mud.", "The saints keep thy roof whole.", "Fare thee well until the morrow."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["Thy servant, m'lord. Good day.", "God keep the manor and thee with it, m'lord."] },
                Voice { criteria: &[Has("other:ideology:merchant")], lines: &["Fair profit to thee, trader.", "May the road pay thy way, merchant."] },
                Voice { criteria: &[Has("self:ideology:lord")], lines: &["Walk on, goodman - mind the hedges.", "Aye, good day to thee."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Fair day, {other} - the ale waits for us both someday.", "Go well, my friend - come by the house when thou canst."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["God keep our house - and thee in it, {other}.", "Fair day, kin - give my love to thy hearth."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Fair road to thee, stranger.", "Go well, traveler - the saints watch thy path."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Away then, {other} - the day improves already.", "Fair day, {other} - such as it is, with thee in it."] },
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
    calls: &["They do say the reeve counts the sheep twice over.", "The herb-wife was seen out past the abbey wall at dusk.", "Old Wulfric swore he saw an owl perched on the tavern roof.", "A stranger with two laden mules passed by the weir at noon."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["gossip"],
            respond: None,
            lines: &["Well, that is talk enough for one morning.", "Ha! I'll believe the half of it and no more.", "Tavern tales grow taller with each ale drawn.", "Every tongue in the parish wags before sunset."],
            voice: &[],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            respond: None,
            lines: &["Better to tend thy own hearth than watch the road.", "I have heard that lie three times this week.", "Folk will invent dragons where there's only sheep.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
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
        "I hear the work keeps thee busy at {other_trade}.",
        "Hard at it down at {other_trade}, are we?",
        "Busy hands at {other_trade} this season, I'd wager.",
        "{other} - how goes {other_trade} with thee this season?",
    ],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["How goes {other_trade}, friend? Thy hands never idle.", "I passed {other_trade} and thought of thee, {other}."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our kin's trade serves thee well at {other_trade}, aye?"] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Still grinding at {other_trade}, {other}? Some things never change.", "They say {other_trade} has seen better hands than thine."] },
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["My steward spoke well of {other_trade}. Keep at it.", "The manor has need of good work at {other_trade}."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 1.5)],
            act: Act::Answer, concepts: &["work"],
            respond: None,
            lines: &["Well enough - {trade} keeps my hands full and my belly fuller.", "It goes, by the saints' grace. {trade} is kind to a steady hand.", "Fair season for it - {trade} has not wanted for custom.", "Slow mornings, honest evenings - that is {trade} for thee."],
            voice: &[
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
            lines: &["Well enough, aye. And how does {other_trade} treat thee?", "Never better. And thyself - is {other_trade} kind to thee?"],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["work"],
            respond: None,
            lines: &["{trade} is my affair, and it fares as it fares.", "Ask the reeve about {trade}, not me. I keep my own count."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["My {trade} needs no commentary from thee, {other}."] },
            ],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_GOSSIP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip", "smalltalk"],
    calls: &["Didst thou hear who was seen down by the mill?", "They say the miller took an extra scoop of meal again.", "The peddler's cart lost a wheel at the bridge.", "A rider from the castle came through before dawn."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Remark, concepts: &["gossip"],
            respond: None,
            lines: &["Thou dost not say it! What happened then?", "Aye? And what else did folk see?", "Tell on, neighbor - my ears are open.", "I knew that bridge would give way sooner or later."],
            voice: &[],
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
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["A tale for another hour - I must away.", "The sun climbs high; I have chores to finish.", "Walk with God, {other}, I cannot linger.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_SEASON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "harvest"],
    calls: &["We must thatch the hayrick before the clouds burst.", "The kitchen gardens are dry as a bone this month.", "Rooks are gathering thick in the elm trees.", "The ditch along the south pasture needs clearing."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["harvest"],
            respond: None,
            lines: &["Aye, work never sleeps in this valley.", "A dry spade makes for a hungry winter.", "Every hand will be needed when the grain ripens.", "I'll lend a pitchfork if thou hast timber to spare."],
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
    calls: &["Well enough, thank thee, {other}.", "We rub along as ever, by the saints' mercy.", "The little ones keep me running from dawn to dusk.", "My knees ache with the damp, but I still walk."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["family"],
            respond: None,
            lines: &["Good to hear it - long may health stay under thy roof.", "Aye, children grow like weeds in the summer ditch.", "A full hearth is a blessing in hard times.", "May the coming season be kind to thy cottage."],
            voice: &[],
            then: Some(SMALL_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Give my best to thy household. Fare well.", "God keep thy folk in peace, {other}.", "I must see to my own broth. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_VOLLEY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Even so, we will manage as we always do.", "Aye, the year will turn and bring what it brings.", "My bones say the rain will hold off till evening.", "No use fretting over skies we cannot steer."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["That they will - the store is not full yet.", "Aye, we have lived through harder winters than this.", "True words; worry never baked a single loaf.", "So long as the well stays sweet, we'll thrive."],
            voice: &[],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll not keep thee standing in the wind. Go well.", "On thy way then, and mine. Fair day.", "Until the church bell rings, neighbor.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
    calls: &["Looks like rain again, doesn't it?", "Cold wind off the moor this morning.", "Gray clouds gathering over the western hill.", "Aye, the dew lay heavy on the clover this morning.", "A gray dawn, but the day's work won't wait for it.", "The frost sat late on the furrows this morning."],
    voice: &[
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
        Voice { criteria: &[Has("self:status:cold")], lines: &["The hearth's been out a week - a cold wind feels personal now."] },
        Voice { criteria: &[Has("self:status:drunk")], lines: &["Is it raining, or is that the ale talking? *hic*"] },
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["The sky owes my wheat a fortnight of sun.", "The weather keeps my stewards busy, I'll warrant."] },
        Voice { criteria: &[Has("self:ideology:merchant")], lines: &["Rain means mud, and mud means slow wagons.", "Grey skies never stopped a good price."] },
        // the fine bond out-reads the coarse one (#1372): a sweetheart
        // hears the weather as a sweetheart
        Voice { criteria: &[Has("pair:kind:lover")], lines: &["Cold wind - stand closer, love, and share my cloak.", "Gray sky or no, the world's brighter where thou standest."] },
        Voice { criteria: &[Has("pair:kind:family")], lines: &["Rain's coming - mother's joints will know it first.", "Gray clouds - same sky that watched us grow up side by side."] },
        Voice { criteria: &[Has("pair:kind:mentor")], lines: &["Watch the sky awhile - weather-reading is half the craft, {other}.", "Cold mornings teach what warm ones cannot, mark me."] },
        Voice { criteria: &[Has("pair:kind:apprentice")], lines: &["Cold wind - I'll fetch thy cloak, master, before we start.", "Gray skies - is it true the old masters read rain in the smoke?"] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Gray clouds again - thou and I have worked under worse, {other}.", "Cold wind this morning - good thing thy tales are warm."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Rain's coming - mother will want the washing in.", "A gray sky over our lane again, {other}."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["The moor wind blows cold on travelers, stranger.", "Gray clouds - ill weather for the road, I'm told."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["The sky's as sour as thy company, {other}.", "Cold wind - fitting weather for thee to be abroad in."] },
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
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            respond: None,
            lines: &["It will be a wet week, mark me.", "The thatch won't thank it if it pours.", "Aye, and the mud will take thy boots right off.", "Good for the barley, but bad for the road."],
            voice: &[
                Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Aye - though maybe there's more to be done than talk of it."] },
                Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Aye - and we've more than weather-talk between us now."] },
                Voice { criteria: &[Has("self:status:thaw_acting")], lines: &["Aye - the mud's where the work is, mind."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Aye - we'll weather it together, like always.", "Wet week or dry, good to stand a while with thee."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Aye - our roof's held worse, thanks to our father's hands.", "The mud'll take our boots, same as when we were little."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, rain - and I'll not lend thee my cloak for it.", "The thatch won't thank it, nor will I thank thee for saying so."] },
            ],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["weather"],
            respond: None,
            lines: &["Rain? It is the driest week in memory.", "Thou seest a tempest in every puff of smoke.", "My joints say dry weather, whatever the sky says.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Rain? Trust thee to call the sky wrong, {other}.", "Thou'dst argue with the weather itself - and lose."] },
            ],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Gregarious, 2.0)],
            act: Act::Question, concepts: &["family", "smalltalk"],
            respond: None,
            lines: &["And how fares thy folk in this cold snap?", "Tell me true, {other} - how goes it with thee?", "Has thy brother returned from the market town?"],
            voice: &[],
            then: Some(SMALL_FARES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Ambitious, 1.5)],
            act: Act::Question, concepts: &["work", "smalltalk"],
            respond: None,
            lines: &["Enough sky-talk - how goes the work these days?", "Weather aside, {other} - how does the season treat thee?", "And the day's work - does it treat thee kindly?"],
            voice: &[],
            then: Some(SMALL_WORK), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Well, I had best be about my chores.", "I should get on before the bell strikes.", "Fair day to thee, {other}.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &["Good morrow to thee, {other}.", "Well met, neighbor {other}.", "A fair day to thee and thine, {other}.", "God's peace on this path, {other}.", "Hail, {other} - traveling far today?"],
    voice: &[
        Voice { criteria: &[Has("self:ideology:lord")], lines: &["Morrow, {other} - thy strip looks well enough this year.", "Aye, neighbor - the morning serves us all."] },
        Voice { criteria: &[Has("self:ideology:merchant")], lines: &["Morrow, {other} - brisk trade to thee.", "Fair day! The road's been kind to the purse."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Well met, {other} - the day brightens for thy company.", "Ah, {other}! I hoped the road would bring thee."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - the house still smells of thy baking.", "{other}! Good blood on the road today."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, traveler.", "A fair day, stranger - if friend thou be."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other}.", "Fair morning to thee, {other}."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["{other}. Thou'rt out early.", "Morrow, {other}. Mind where thou walkest."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            respond: None,
            lines: &["And to thee, {other}.", "Well met indeed, friend.", "The saints keep thee, {other}.", "Good day and full granaries to thee."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["And to thee, m'lord.", "God keep thee and the manor, m'lord."] },
                Voice { criteria: &[Has("other:ideology:merchant")], lines: &["And fair coin to thee, trader.", "Well met, merchant - the road treats thee kindly."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["And a brighter one to thee, {other}.", "Well met, my friend - always."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["To thee and all our house, {other}.", "Morrow, kin - the latch is never barred to thee."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["And to thee, stranger.", "Fair met, though I know thee not well."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["And to thee.", "Good day to thee."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, well met - for some.", "I see thee, {other}."] },
            ],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["Morrow.", "Aye. What is it, {other}?", "I am here, as thou seest.", "Grace and good morrow to thee, {other}.", "Time will tell what comes of this."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Speak thy piece and be done.", "Out with it - I've no patience for thee today."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Aye. What dost thou want of me?", "I am here, stranger - as thou seest."] },
            ],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[Lacks("pair:relation:friend"), Lacks("pair:relation:kin")], weight: 0.25, bias: &[(Proud, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I have no time for idle talk today.", "Work is waiting at the barn, {other}.", "I walk on business, neighbor.", "I have no leisure to stand about, {other}.", "I must walk on; fair day."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I've no time for thee today.", "Walk on, {other}."] },
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
    calls: &["Well met, {other} - thy table's never bare to a neighbor, they say.", "God's peace, {other}. Half the village owes thee a meal, by the telling.", "Ah, {other} - the soul that asks after folk's pots before their own.", "Fair day, {other}. Thy good name walks ahead of thee."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            respond: None,
            lines: &["Thou'rt kind to say so, {other}.", "A neighbor's a neighbor - the saints ask no more.", "Well met, friend. The talk's kinder than the truth.", "And a fair day to thee for saying it."],
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
    calls: &["God's peace, {other} - the bell finds thee on thy knees more oft than not, they say.", "Well met, {other}. Thy faith's a lamp the whole parish sees.", "Fair day, {other} - fresh from the service, like as not.", "The saints keep thee, {other}, as thou keepest them."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting", "concern:faith"],
            respond: None,
            lines: &["And His peace on thee, {other}.", "The day's not whole without the words, friend.", "Kind of thee to mark it. Well met.", "The saints watch us all alike - I only bow the sooner."],
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
    calls: &["{other}. Thou'rt about, then.", "Well met, {other} - if well it be.", "Morrow, {other}. Thy gate was shut to the bailiff's widow last week, I hear.", "Ah. {other}. Keep thy weather to thyself."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Remark, concepts: &["greeting"],
            respond: None,
            lines: &["Is there aught thou needest, {other}?", "The day is what it is.", "Aye, well met then.", "Speak thy piece and have done, friend."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.55, bias: &[(Hotheaded, 2.5), (Proud, 2.0)],
            act: Act::PushBack, concepts: &["greeting"],
            respond: None,
            lines: &["My gate's my own affair, {other}.", "I owe the whole village nothing, mark me.", "Cold? I've a fire like any man's.", "Wag thy tongue elsewhere, {other}."],
            voice: &[],
            then: Some(SMALL_CHAT), end: None,
        },
    ],
};

const SICK_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness", "aid"],
    calls: &["Thou'rt kind, {other}. The saints remember a neighborly hand.", "My thanks. A warm bowl does more than herbs.", "Bless thee for thinking of our small house.", "Rest by the hearth and let the fever break in peace."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Think nothing of it. We look after our own.", "Rest quiet by the coals. I'll check on thee tomorrow.", "The village stands together or it falls. Fare well.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_STUNG: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness"],
    calls: &["It is not the black pestilence, keep thy wits about thee.", "As thou wilt, then - step wide if thou must.", "A little cough won't kill an honest worker.", "Rest by the hearth and let the fever break in peace."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Forgive me - fear makes cowards of us all.", "Thou hast right of it. I spoke too quick.", "No offence meant, {other}. We are all on edge.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            respond: None,
            lines: &["I'll not catch my death for thy pride.", "Breathe elsewhere then, knave.", "Stay in thy own yard if thou'rt frightened."],
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
            lines: &["The saints speed thy mending, {other}.", "Drink hot boiled water with a scrape of honey.", "May the affliction pass thee lightly.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            respond: None,
            lines: &["I will send broth round when the kettle boils.", "I'll ask Father Anselm to light a candle for thy recovery.", "Sit by the hearth; I'll fetch firewood if thou hast none.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must be away before the air catches me too.", "Mind thy cup and bowl. Good day.", "I have no leisure to stand about, {other}.", "I must walk on; fair day."],
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
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["I'd rest, {other}, but the chores won't do themselves - though I know thou'dst lend a hand.", "The calves won't feed themselves, friend - fever or no."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["The calves won't feed themselves - though our kin have offered, bless them.", "There's none else to fetch the water - though I know our house would send it."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["There is no one else to fetch water from the river, stranger.", "The rent waits for no man's cough."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["The calves won't feed themselves - and I'd not ask thee for help, {other}.", "I'd rest, but who'd do it? Not thee, I reckon."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["We labor or we go hungry, that's the hard truth of it.", "Aye, the dirt cares nothing for an aching back.", "A poor man's bed is a luxury he can rarely afford.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Aye, and I'd carry half thy load if thou'dst let me.", "True enough - but a friend's help makes the toil shorter."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Aye, but our house will see thee through it.", "Blood doesn't watch blood labor alone for long."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, work or starve - thou knowest it as well as I.", "The dirt cares nothing - for either of us."] },
            ],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            respond: None,
            lines: &["Let me carry thy pail to the well, {other}.", "Go inside; I'll see to thy livestock this evening.", "I have barley meal to spare for thy kettle.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Say the word and the chore's done, {other} - thou knowest it.", "I'll see to thy beasts this evening; rest easy."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our house has barley to spare - blood takes care of blood.", "Go in, {other}; thy cousins will see to the yard."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I'll carry thy pail - once - and I'd not have it spoke of.", "Thy beasts I'll see to; thee, I'll suffer at a distance."] },
            ],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Rest when thou canst, {other}. Fair day.", "God grant thee strength for the day's toil.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Rest well, {other} - I'll look in on thee at dusk."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Rest when thou canst - or don't; it's no care of mine.", "God grant thee strength - and keep thee far from me."] },
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
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["*cough* Ah, {other} - forgive the croak. Stand off, mind; I'd not share this with a friend.", "Morrow, {other}. A wretched cold - but warmer for seeing thee."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["*cough* Morrow, {other}. Tell our mother I still stand upright.", "{other} - aye, it's me, coughing like a broken bellows. Keep out of the draft."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["*cough* Morrow, traveler. Best keep to the far side of the path.", "A sick man's greeting, stranger - from a distance."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["*cough* Fair morning, {other}. Mind the breeze between us.", "Morrow, {other} - a foul chest this week."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["*cough* {other}. Keep to thy side - I'd not give thee even my sickness.", "Morrow, {other}. Step wide; even my fever wants no part of thee."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Poor soul - get thee back into the warm cottage.", "Thou shouldst be under woolen blankets, not out here.", "Thou lookest pale as a winter moon, {other}.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["Poor soul - get thee back to the manor's warm hearth, m'lord.", "Thou shouldst be under woolen blankets, m'lord - thy folk will fret."] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Get thee back under the blankets, {other} - I'll not watch thee worsen.", "Poor soul. Tell me what thy house needs and it's done."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Thou'rt blood - I'll not watch thee shake. Home with thee.", "Back to bed, {other}. I'll fetch the broth myself."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Thou shouldst not be abroad in this state, stranger.", "Keep thy strength - and thy distance."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Thou lookest pale, {other}. Best be indoors.", "A rough fever, by the look of thee."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, keep thy distance - but I'll not pretend a pity I don't feel.", "Even I'd not wish this on thee, {other}. Perhaps."] },
            ],
            then: Some(SICK_EXCUSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Is it the ague that took the miller's lad?", "How long has this fever had hold of thee?", "Hast thou seen the herb-wife for a draught?"],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["How long has the fever held thee, m'lord?", "Hast thou seen the herb-wife for a draught, m'lord?"] },
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["How long has it held thee, {other}? Speak plain with me.", "Is it the miller's lad's fever? Thou knowest I'll help."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["What does the herb-wife say? Our house will hear of it."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["How long since it took hold?", "Is it catching, this fever of thine?"] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["How long hast thou dragged that about, {other}?", "Is it the fever, or merely thy face?"] },
            ],
            then: Some(SICK_ANSWER), end: None,
        },
        Reply {
            criteria: &[Lacks("self:status:sick")], weight: 0.50, bias: &[(Proud, 1.5), (Stingy, 1.5)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Keep thy distance, neighbor. I have no wish to catch it.", "I will hear thee from across the ditch.", "Cover thy mouth when thou speakest in the wind.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[
                Voice { criteria: &[Has("other:ideology:lord")], lines: &["Keep thy distance, m'lord - I have no wish to catch it.", "I'll hear thee from across the ditch, begging m'lord's pardon."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Keep thy plague and thy distance both.", "Breathe elsewhere, {other} - as ever."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Keep thy distance, stranger - I want no new sickness.", "I'll hear thee from across the ditch, and no nearer."] },
            ],
            then: Some(SICK_STUNG), end: None,
        },
    ],
};

const VISIT_PROMISE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid"],
    calls: &["It is done - look for it by sundown.", "I'll have my girl bring the loaf before dark.", "Rest easy; the parish will not let thee freeze.", "We do what we must while daylight holds."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thou'rt a true neighbor, {other}.", "The blessing of Heaven on thy roof.", "I'll repay the kindness when my legs mend.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_CARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:sickness", "aid"],
    calls: &["Is there aught thou needest brought to the cottage?", "Can I fetch water or chop kindling for thy hearth?", "Dost thou have grain left in the bin?", "Does the fever ease when the sun warms the cottage?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["aid", "concern:food"],
            respond: None,
            lines: &["A little bread or salt, if thou canst spare it.", "Some dry firewood would keep the chill out of our bones.", "A jug of fresh well water would be a godsend.", "The sweat broke at dawn and the fever feels a little lighter."],
            voice: &[],
            then: Some(VISIT_PROMISE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Pious, 1.5)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Just a prayer at the church altar, {other}.", "We have enough for today. Pray for my children.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["How dost thou fare today, {other}?", "I heard the sickness took thy household - how is it?", "Peace to this cottage, {other}. How goes the fever?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:sickness"],
            respond: None,
            lines: &["Poorly, if thou'lt have the plain truth.", "Weak as a lamb, but still drawing breath.", "The night was hard, but the morning brings a little ease.", "The sweat broke at dawn and the fever feels a little lighter."],
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
    calls: &["Thank thee for listening, {other} - it lightens the heart.", "It does a body good to speak their name aloud.", "The house feels less empty when a neighbor sits a spell.", "The sorrow will ease as the seasons turn and heal."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Any hour, {other}. My door is always unlatched.", "We do not forget those who walked this lane with us.", "May holy angels watch over thy sleep tonight.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_ASK: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:grief", "aid"],
    calls: &["A loaf of bread would not go amiss, {other}.", "A few split logs, if thou hast any seasoned oak.", "Someone to watch the sheep for an afternoon."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["aid", "farewell"],
            respond: None,
            lines: &["It is done. Look for my boy with the basket by dusk.", "I will bring it myself before sunset, friend.", "Leave the sheep to me; mourn in peace.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I will see what can be spared from our cellar.", "Times are tight, but I'll see what I can find.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_OPENS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["I keep setting his wooden spoon at table.", "She loved the smell of the damp earth after rain.", "Every corner of the cottage remembers his footsteps.", "Forty winters together, and now only the silence."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["Tell me of him - good memories are a shield against darkness.", "She was loved by all who knew her in the village.", "Their toil shaped this parish; their memory will outlive us.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: Some(GRIEF_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll not weary thee with more words. God keep thee.", "Rest thy sorrow in prayer, {other}. Fare well.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["{other}. Forgive me - the house is very still since he passed.", "Morrow, {other}. The bell tolled, and my hearth is dark.", "Well met, friend. The days feel longer than they used to.", "A peaceful day to thee, {other}."],
    voice: &[
        // grief's the one wound the rung reads differently (#1269):
        // cold it's endured, warm it's carried
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Morrow. The house is still, and that's just how it is now - no help for it.", "{other}. The bell tolled, and we go on. There's nothing else to do."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["{other}. The house is still - but somebody asked me what I wanted, and I've been thinking on them since."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["{other}. The house is still - but the cause's folk have sat with me in it."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["{other}. Forgive me - the house is still since they passed, and thy face is a kindness.", "Well met, dear friend. The days feel longer now."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["{other}. Blood knows blood - the house is empty without them.", "Morrow, {other}. Our family's table has an empty bench."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Forgive a grieving house its silence.", "A heavy morning - we've buried one of ours."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other}. The bell tolled for my house this week.", "A gray day, {other}. The hearth is dark."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["{other}. Even thou'll know the house has gone quiet.", "Morrow, {other}. Death levels more than our quarrel."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.40, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["They were an honest soul. The whole village mourns them.", "The saints receive their spirit in eternal peace.", "My heart aches for thee, neighbor.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["They were the best of us, {other} - I'll miss them beside thee.", "My heart breaks with thine, {other}. Always."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our blood mourns together, {other}. The saints keep them.", "They were ours - and we carry them now."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["I did not know them, stranger, but I grieve thy grief.", "A loss is a loss - the saints keep their soul."] },
                Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["An honest soul gone. The village mourns with thee.", "The saints receive them kindly."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["We'd our quarrels, {other} - but the grave settles them.", "I'll not pretend tears, {other} - but I'll not mock the dead."] },
            ],
            then: Some(GRIEF_OPENS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Kind, 1.3)],
            act: Act::Question, concepts: &["concern:grief", "aid"],
            respond: None,
            lines: &["Is there any chore I can take off thy shoulders?", "Hast thou eaten a warm bite today, {other}?", "Shall I sit with thee a while, {other}?"],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["What can I take off thy hands, {other}? Name it.", "Shall I sit with thee? Thou needn't be alone."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["What does the house need? Say it and it's done."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Is there aught a stranger can do?", "Dost thou have folk to help thee?"] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["I'll not offer charity thou'dst refuse - but is thy house seen to?", "Is anyone feeding thee? Even I would not see thee starve."] },
            ],
            then: Some(GRIEF_ASK), end: None,
        },
    ],
};

const CONDOLE_OFFER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief", "aid"],
    calls: &["If thou needest aught at all, say but the word.", "We have flour and eggs; let us share with thy house.", "My sons can plough thy furrow until thou'rt ready.", "The sorrow will ease as the seasons turn and heal."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thou'rt a true soul, {other}. I will remember it.", "Thy kindness brings tears to an old eye. God bless thee.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["We will manage on our own, but thank thee kindly.", "I will not burden thy house with my sorrow.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CONDOLE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["I was deeply grieved to hear of thy loss, {other}.", "The village stands beside thee in this dark trial.", "God's mercy comfort thee in thy sorrow, neighbor.", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:grief"],
            respond: None,
            lines: &["Thou'rt good to come by, {other}. It means much.", "Thank thee for thy prayers and thoughts.", "The road is lonely now, but kind words help.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
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
    calls: &["The saints watch over thy cottage. I will come by with holy water and draughts.", "Rest by the hearth, child - I'll light a candle and pray for thy household.", "Cast thy burdens before the altar; the Lord does not forget the meek.", "The church altar is always open to the weary soul."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thank thee, father - thy blessing brings comfort.", "The Lord keep thee, father, and preserve thy voice.", "I will be at the church when the bell sounds.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PRAYER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith", "concern:grief", "concern:sickness"],
    calls: &["The Lord comfort thee, child. Tell me of thy affliction.", "Speak, child - my prayers and candles are with thee.", "Open thy heart; what sorrow weighs upon thee?", "How dost thou find the road today, {other}?"], voice: &[],
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
    calls: &["How fares thy house in these lean weeks, child?", "Dost thou find peace in thy daily labor, child?", "Remember that honest work in the field is a holy prayer.", "The church altar is always open to the weary soul."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:faith"],
            respond: None,
            lines: &["We keep as best we can, father. The toil is hard.", "The season has been lean, father, but faith sustains us.", "I try to walk uprightly before God and my neighbors.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I will see thee at the Sunday bell, father.", "Keep our fields in thy prayers, father. Fair day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Blessings on thee, {other}.", "The peace of the saints be with thee, child.", "God's grace light upon thy steps, {other}.", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Pious, 2.0)],
            act: Act::Greet, concepts: &["concern:faith"],
            respond: None,
            lines: &["And on thee, father.", "Blessings on thy holy house as well.", "Peace be with thee, good priest.", "God bless thy steps this day, {other}.", "A blessed morning to thee, father."],
            voice: &[],
            then: Some(CLERIC_PARISH), end: None,
        },
        Reply {
            criteria: &[Has("self:status:sick")], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:sickness"],
            respond: None,
            lines: &["A prayer for me, father - this fever won't quit.", "Father, hear my plea; the sickness grips my throat.", "Wilt thou hear my confession ere the fever takes me?"],
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
            lines: &["Morrow, father.", "A fair day to thee, priest.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            voice: &[],
            then: Some(CLERIC_PARISH), end: None,
        },
    ],
};

const FATHER_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["Thank thee, father - that lifts a great stone from my mind.", "Thy counsel is wise, father. We will trust in Providence.", "The church has been our refuge through many a cold year.", "Thy blessing gives me heart to face the coming week, father."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Go in peace, {other}, and love thy neighbors.", "The Lord preserve thy hearth and thy children.", "Walk in the light, child. Fare well.", "My prayers go with thee, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_ASKS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["Must the tithe stand full when the granary is half empty, father?", "Can the church spare a peck of grain for the hungry children, father?", "Will the parish altar provide candles for our sick neighbor, father?", "How dost thou find the road today, {other}?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:faith"],
            respond: None,
            lines: &["The church will share its grain, child - none shall starve at the door.", "The tithe stands, but I will speak for thy house at the manor court.", "Holy candles and broth shall be brought to their bedside this very day.", "We do what we can with the strength Heaven gives."],
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
    calls: &["Good morrow, father.", "A word with thee, father, if thou canst spare a moment.", "Greetings, holy father - I seek thy pastoral ear.", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Kind, 1.3)],
            act: Act::Greet, concepts: &["concern:faith"],
            respond: None,
            lines: &["The saints keep thee, {other}. What weighs on thee?", "Always, child. Speak freely in the sight of God.", "Peace to thee, {other}. How may I help thy soul?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["We are done here.", "I have looked upon thy face long enough.", "Take thy shadow off my land.", "I have no words to waste on a crooked knave."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Aye. Done.", "Until we meet at the boundary ditch.", "Gladly. The sight of thee sours my stomach.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const RIVAL_BITE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Make me.", "Come closer and say that to my beard.", "Thou'rt all mouth and no marrow.", "I have no words to waste on a crooked knave."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[],
            act: Act::Remark, concepts: &["feud"],
            respond: None,
            lines: &["Not worth the breath or the bruise.", "I'll settle this before the magistrate, not in the dirt.", "This quarrel's older than thy fence, and twice as rotten."],
            voice: &[],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Hotheaded, 4.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Cur! Thou'lt regret those words.", "Keep thy fists to thyself before I break them.", "Get out of my sight, {other}.", "Take thy shadow off my path."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const RIVAL_SNIPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Heard about thy boundary fence. Shoddy timber, that.", "Thy lot still owes mine three groats, whatever lies thou tellest.", "Keep thy lean cows out of our clover pasture.", "I saw where thy cart rutted the lane by my ditch."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["My fence stands straighter than thy crooked word.", "We paid that debt at Michaelmas, as the reeve well knows.", "Thy pasture is mostly thistles anyway."],
            voice: &[],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Say that again, if thou darest.", "Thou hast a treacherous tongue in thy skull.", "The whole parish knows the measure of thy word."],
            voice: &[],
            then: Some(RIVAL_BITE), end: None,
        },
    ],
};

const RIVAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "feud"],
    calls: &["You.", "Look what the crows dropped on the highway.", "{other}. Still walking upright, I see.", "What business brings thy ugly mug across my path?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["feud"],
            respond: None,
            lines: &["{other}.", "I walk where I please in this parish.", "Save thy bile for someone who fears it."],
            voice: &[],
            then: Some(RIVAL_SNIPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            respond: None,
            lines: &["Still breathing, I see. A great pity for the town.", "The years have only made thee sourer.", "The parish got poorer the day thou drew breath."],
            voice: &[],
            then: Some(RIVAL_BITE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["farewell"],
            respond: None,
            lines: &["I have nothing to say to thee, scoundrel.", "Out of my way. I don't parley with knaves.", "Begone before my temper answers for me."],
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
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Aye, hold fast. Better days will follow.", "The earth will feed us yet. Go gently, {other}.", "May tomorrow bring a fuller kettle.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
            lines: &["Take half my loaf, neighbor. No child should hunger in this village.", "Come by our hearth; there is pottage enough for two more bowls.", "We have a sack of roots in the cellar. Let us divide it.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
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
    calls: &["Good morrow, {other}. A hollow belly makes for a slow step.", "Well met, friend, though the times are thin and hard.", "Greetings, {other}. Hast thou found anything to put in the pot?", "A peaceful day to thee, {other}."],
    voice: &[
        // the rung the greeting's spoken from (#1269): a cold greeting
        // is flat, a warm one looks for what could move
        Voice { criteria: &[Has("self:status:thaw_helpless")], lines: &["Morrow, {other}. Another hungry one - same as all the rest.", "Aye, morrow. Thin again, like every day these years."] },
        Voice { criteria: &[Has("self:status:thaw_thawing")], lines: &["Morrow, {other} - the pot's thin, though I've a question still turning over in me."] },
        Voice { criteria: &[Has("self:status:thaw_joined")], lines: &["Morrow, {other} - lean times, but we've folk looking at it together now."] },
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Morrow, {other} - a hollow belly, but glad company.", "Well met, {other} - we've both known thinner tables. Barely."] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - is there bread at thy hearth? Ours runs thin.", "{other} - kin shouldn't have to ask, but hast thou grain?"] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Thin pickings along this road.", "A hungry greeting, traveler - the fields gave little this year."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Morrow, {other} - lean times on every lane.", "Fair morning, {other}. The pot stays empty these days."] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Morrow, {other}. Hunger bites even the unworthy.", "{other}. If thy larder's full, I'd not believe thee anyway."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:food"],
            respond: None,
            lines: &["Morrow, {other}. The porridge is thin as ditchwater this week.", "Aye, we scrape the bottom of the flour chest.", "The storehouse has dwindled down to crumbs.", "Take half this barley loaf; we must look after each other."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Thin porridge for us both, {other} - but half my loaf is thine.", "Crumbs at our house too - come share what little there is."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["The bin's near empty, {other} - but blood shares what it has.", "Our house eats thin too; come by and we'll stretch it."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["We scrape what we can, like all here.", "The storehouse holds little for strangers' asking."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Thin enough, {other} - though I'd not expect thee to spare a crumb.", "We manage - no thanks to the likes of thee."] },
            ],
            then: Some(HUNGER_SHARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["We tighten our belts and carry on.", "Talk won't fill the larder. I must seek wild greens.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Come by the house tonight, {other} - we'll make it stretch."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Talk fills no bellies - least of all thine.", "I must seek greens. Keep thy hands off my patch."] },
            ],
            then: None, end: Some(Farewell),
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
            lines: &["Bank the coals with ashes so they last till dawn.", "May thy hearth keep hot through the bitter night.", "The fire is life in winter. Fare thee well.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const COLD_GATHER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:fuel", "aid"],
    calls: &["Will thou come to the common wood to cut deadwood and kindling?", "Can I borrow a shovel to dig peat from the dry bog?", "Hast thou glowing embers in an iron pot to rekindle our cold stones?", "Can we gather dry brushwood together in the common woods?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:fuel"],
            respond: None,
            lines: &["Aye, I'll bring my hatchet. Two bundles of dry boughs will do.", "Take these live embers from my hearth; shield them from the wind.", "We stacked dry turf behind the byre; take what thou canst carry.", "Take these live embers in an iron pot to start thy fire."],
            voice: &[],
            then: Some(COLD_FIRE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["concern:fuel"],
            respond: None,
            lines: &["The bailiff watches the common wood; he will fine us for gathering boughs.", "My own hearth barely has coals to warm an egg.", "Gather on the lord's land if thou darest - not from me."],
            voice: &[],
            then: None, end: Some(WalksOff),
        },
    ],
};

const COLD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:fuel"],
    calls: &["Brrr! Good morrow, {other}. The rime frost is inside the thatch.", "Well met, neighbor. My breath freezes right before my nose.", "Greetings, {other}. The wind blows straight through the cottage wattles.", "A peaceful day to thee, {other}."],
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
            lines: &["Our hearth went cold before midnight; the stone is like ice.", "We have burned through the last dry bundle of brushwood.", "Without dry turf, the sheepfold is warmer than the cottage.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: Some(COLD_GATHER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I must chop timber before the dark settles in.", "Keep moving or the cold will numb thy toes. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
            lines: &["Walk straight, {other}, or the reeve will notice.", "No more brown ale for thee until Sunday! Fare well.", "May the sun be gentle on thy poor eyes today.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Thou didst indeed! And fell backwards over the ale barrel!", "Aye, and the fiddler laughed so hard his bow went flying.", "It was a merry night, though the morning pays the reckoning.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            voice: &[],
            then: Some(DRUNK_SOBER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Pious, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            respond: None,
            lines: &["Thou made an ungodly spectacle of thyself.", "A shameful racket; the priest shook his head at the doorway.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            voice: &[],
            then: Some(DRUNK_SOBER), end: None,
        },
    ],
};

const DRUNK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &["Oof... good morrow, {other}. Don't shout, I pray thee.", "Well met, friend. My head is hammering like the smith's anvil.", "Ugh, the sunlight is blinding today, {other}.", "A peaceful day to thee, {other}."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Oof - morrow, {other}. Don't laugh; my head's an anvil today.", "Well met, {other} - thou'dst have warned me off that third cup, I know."] },
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
            lines: &["Ha! The tavern-keeper's ale had a bite to it, didn't it?", "Thou lookest like a corpse dug up from the wet turf.", "I warned thee that dark brew was brewed for giants.", "We do what we can with the strength Heaven gives."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Ha! I told thee that dark brew was brewed for giants, {other}.", "Thou lookest like death's cousin - and I should know, I feel the same."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Our grandfather drank the same way - the shame's in the blood.", "Mother warned the whole house about that ale."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["The ale bites the unready, stranger.", "Thou lookest like the grave let thee out early."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Thou lookest a wreck, {other} - I'd pay to see it again.", "The ale had more spine than thee, I see."] },
            ],
            then: Some(DRUNK_RECALL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Sleep it off in the hayloft, {other}.", "Go dunk thy head in the horse trough. Fair day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Sleep it off, {other} - I'll cover for thee at the work.", "Rest thy poor head, friend."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Sleep it off, {other} - somewhere out of my sight.", "Go soak thy head - and stay gone."] },
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
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Always, kin. The family latch is never barred to thee.", "God keep our house and hearth safe from storm.", "See thee at Sunday meal, {other}.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_FAMILY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["family"],
    calls: &["How fares mother's knee in this damp spell?", "Has our cousin returned with the pack horse from the fair?", "Dost thou need an extra hand to dig the potato trench?", "How fares our sister's household down by the green?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["family"],
            respond: None,
            lines: &["She sits by the fire with warm sheep's grease; it eases the pain.", "Aye, he brought home two bags of salt and a fine sickle.", "Come by with a spade after the noon bell, brother.", "Our kin will stand beside thee through good season and bad."],
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
    calls: &["Well met, cousin {other}!", "Good morrow, kin! How fares the old cottage?", "Greetings, brother! It does my heart good to see thy face.", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            respond: None,
            lines: &["And to thee, kin! The roof still holds.", "Well met, dear cousin. Thou'rt looking hale and hearty.", "God bless thee, kin. What news from thy side of the lane?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["A good friend is worth more than a pouch of silver.", "We have shared crusts and sweat for twenty years now.", "Thy word has never failed me, {other}.", "True fellowship is the richest coin in the parish."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Nor thine me, old friend. Walk safe.", "We will share a cup of cider when the work is done.", "Fair day, {other} - peace on thy household.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_TRUST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["friendship"],
    calls: &["Dost thou recall the summer the mill dam broke?", "Can I count on thy team when we haul the heavy timber?", "Hast thou heard any whispers about the new taxes?", "Can I count on thy strong arm when we raise the barn?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["friendship"],
            respond: None,
            lines: &["Aye, we were waist-deep in mud until midnight saving the sacks!", "My oxen are thine whenever thou needest them, friend.", "The bailiff spoke of three pennies per hide, but we'll contest it.", "Thy word is better than gold; count on my team anytime."],
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
    calls: &["My heart sits lighter having seen thee this morning.", "I saved a smile for thee since yesterday - here it is.", "The lane could burn down and I'd not mark it, talking with thee."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:lover"],
            respond: None,
            lines: &["And mine at seeing thee - walk with me a while.", "Thy words are warmer than the hearth, love.", "As thou sayest - the day is kinder for thee."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::PushBack, concepts: &["bond:lover"],
            respond: None,
            lines: &["Sweet words - and didst thou say the same to the miller's daughter?", "Thou'rt bold this morning - what dost thou want of me?"],
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
    calls: &["Mother sent word - she prays for thy cough every bell.", "Blood is blood - what little we have is thine to share.", "Thy grandfather and mine broke the same field, remember."], voice: &[],
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
            lines: &["Give my love to thy mother when thou seest her.", "Go well, cousin - the house is always open to thee."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// A mentor's counsel (#1372): the teacher's register, not smalltalk
const MENTOR_COUNSEL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:mentor"],
    calls: &["Mark this well - the work outlives the wage.", "Patience, lad - the craft reveals itself to those who wait.", "I was green once, same as thee - the hand learns what the eye cannot."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:mentor"],
            respond: None,
            lines: &["Thy words are worth more than a day's wage, master.", "Aye, I mark it - as thou hast taught me.", "Every lesson thou givest me I keep by heart."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I go to practice what thou saidst. Fair day, master.", "By thy leave, master - the bench calls."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The apprentice's eager report (#1372): the learner's register
const APPRENTICE_EAGER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["bond:apprentice"],
    calls: &["I set the joints as thou showed me - wilt thou look?", "The work goes quicker now - thy lessons hold.", "Tell me again how the old ones did it - I want it right."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["bond:apprentice"],
            respond: None,
            lines: &["Well enough - keep thy hand steady and thy eyes open.", "Thou learnest faster than I did at thy age.", "Eager hands make light work - go on."],
            voice: &[],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Back to the bench with thee - I'll look in before the bell.", "Go on then, and mind the edge as I showed thee."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "friendship"],
    calls: &["A hearty good morrow to thee, {other}!", "Well met, true friend! The sun shines brighter when thou'rt near.", "Greetings, old comrade! How goes the day's labor?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["friendship"],
            respond: None,
            lines: &["And to thee, {other}! It gladdens my soul to see thee.", "Well met indeed, friend. The chores are lighter for thy smile.", "Good day, partner! We'll weather whatever comes.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["Keep to the beaten track; the marshes to the east are treacherous.", "The alehouse by the bridge offers straw beds if thou hast coin.", "The parish watch keeps guard after the curfew bell.", "The road is wide enough for all peaceful travelers."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["My thanks for the counsel, villager. I pass in peace.", "Safe travels to thee as well. Good day.", "The highway calls; farewell.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_PATH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["stranger"],
    calls: &["Where does this trail lead past the ridge, traveler?", "What town dost thou hail from, stranger?", "Is the road north safe from cutpurses and wolves?", "What brings thy footsteps into our quiet parish?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["stranger"],
            respond: None,
            lines: &["It leads down to the cathedral town and the salt wharves.", "I come from the valley beyond the high moors.", "The road is quiet by day, but light a torch if thou walkest at dusk.", "The road to the cathedral town follows the river south."],
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
    calls: &["Halt there, traveler. What brings thee through our parish?", "Good day, stranger. Thou'rt far from the king's highway.", "Peace be upon thy journey, traveler. Who art thou?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["stranger"],
            respond: None,
            lines: &["Greetings, villager. I mean no harm on these lanes.", "Well met. I am but a traveler seeking the ford across the water.", "Peace to thy fields, friend. I look for the nearest smithy.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["Keep the fire roaring, smith. Good day.", "May thy hammer never miss its mark. Fair day.", "I will return for the coulter at sunset.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMITH_WORK: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:smith"],
    calls: &["Canst thou forge three dozen horseshoe nails by tomorrow?", "My plough coulter struck a buried boulder and snapped clean.", "Dost thou have charcoal enough to heat the heavy anvil?", "Is the hearth hot enough to weld the iron tyre?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, smith! The sparks are flying bright today.", "Well met, master smith! The sound of thy forge wakes the village.", "Hail to the forge, {other}! Sweating by the bellows already?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:smith"],
            respond: None,
            lines: &["Morrow, {other}! The coals are white-hot and hungry for iron.", "Well met! Step inside out of the wind, mind the sparks.", "Aye, iron waits for no man's leisure. What needest thou?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["Keep the wheel turning, miller. Fair day.", "Mind thou dost not spill my meal on the floorboards.", "Until harvest time, master of the water.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_GRIST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:miller"],
    calls: &["Is the river running high enough to turn the big wheel?", "Canst thou grind my two sacks of barley before noon?", "Did the damp get into the meal hopper after the downpour?", "Will the mill race freeze if the frost sharpens tonight?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:miller"],
            respond: None,
            lines: &["The millpond is full to the weir boards; she turns swift today.", "Stack thy sacks by the scale; thy meal will be ground by mid-day.", "The oak bins are dry as bone; not a drop touched the flour.", "The river turns the wheel fast; the meal pours sweet."],
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
    calls: &["Good morrow, miller! The roar of thy weir echoes down the valley.", "Well met, {other}! Flour dusts thy coat like fresh snow.", "Hail to the mill, {other}! Is the grist running smooth?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:miller"],
            respond: None,
            lines: &["Morrow, neighbor! The stones rumble and the water runs free.", "Well met! Unload thy cart; the hopper is ready.", "Aye, dust in my hair and bread in my belly! What grist bringest thou?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["The smell of thy loaves is a blessing to the lane. Fare well.", "May thy oven never grow cold, baker.", "I take my warm crust home to the children. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_BREAD: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:baker"],
    calls: &["Are the morning rye loaves ready to pull with the peel?", "Canst thou bake my barley dough in thy community oven?", "Hast thou any day-old trenchers left for a penny?", "Are the loaves golden brown on the oven bricks?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:baker"],
            respond: None,
            lines: &["Aye, hot from the bricks and brown as a hazelnut!", "Bring thy dough before noon; there is room beside the landlord's pies.", "Take four trenchers; they are crisp and good for soaking broth.", "The crust is crisp and hot from the stone floor."],
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
    calls: &["Good morrow, baker! Thy chimney breathes the sweetest air in town.", "Well met, {other}! The smell of fresh crust drew me from the lane.", "Hail, master of the peel! Has the first batch risen?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:baker"],
            respond: None,
            lines: &["Morrow, {other}! Flour on my hands and fire in the hearth.", "Well met! The oven is blazing hot and the loaves are baking.", "A warm welcome to thee! Step in and feel the heat.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy shuttle fly true and never break a thread.", "I will return when the piece comes off the frame. Fare well.", "Good cloth is true wealth. Good day, weaver.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_FLEECE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:weaver"],
    calls: &["Did the sheep shearers bring in the upland wool yet?", "Canst thou weave a yard of heavy russet for my work kirtle?", "Is the spun yarn dry enough to wind onto the spools?", "How many yards of wool cloth come off the loom today?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, weaver! The click-clack of thy loom is steady today.", "Well met, {other}! Surrounded by skeins of dyed wool, I see.", "Hail to the loom, neighbor! How runs the shuttle this morning?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:weaver"],
            respond: None,
            lines: &["Morrow, {other}! Counting the threads from dawn till twilight.", "Well met! The treadles are flying and the cloth grows fast.", "Greetings! A weaver's work is never done while sheep have wool.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy adze stay sharp, carpenter. Fare well.", "I will look for the cart axle on Friday. Good day.", "Stout timber makes a sturdy home. Walk in peace.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_BEAMS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:carpenter"],
    calls: &["Canst thou mortise a tie-beam for the cottage roof ridge?", "Didst thou square the timber logs felled in the lord's wood?", "Dost thou have spare dry pegs to secure the wattle hurdle?", "Canst thou hew a sturdy axle-tree for my cart?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, carpenter! Shavings and sawdust fly thick today.", "Well met, {other}! The scent of fresh-cut pine is in the lane.", "Hail to the woodwright! Has the new timber arrived from the coppice?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:carpenter"],
            respond: None,
            lines: &["Morrow, neighbor! Measuring twice and cutting once, as ever.", "Well met! The saw pit was busy since the dawn bell.", "Greetings! A stout tree fell yesterday; fine timber for building.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy plumb-line hang true, mason. Fair day.", "Stone stands when all else fades. Farewell.", "I will bring the ox-cart for the rubble tomorrow.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_WALL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:mason"],
    calls: &["Canst thou dress ashlar stones for the new church window?", "Did the quarry deliver enough rubble stone to rebuild the wall?", "Will the lime mortar set before the autumn rains begin?", "Will the limestone mortar hold against the river damp?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, mason! The ringing of thy hammer on stone echoes far.", "Well met, {other}! White rock dust on thy leather apron.", "Hail to the stone-cutter! Setting foundations today?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:mason"],
            respond: None,
            lines: &["Morrow, {other}! Stone by stone the hall rises from the earth.", "Well met! Testing the level and squaring the corners.", "Greetings! A heavy trade, but a wall built right outlasts empires.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["Keep thy eye on the brushwood, shepherd. Fair day.", "May the flock prosper and the wolves stay distant.", "Good grazing to thy flock, neighbor.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
    calls: &["Good morrow, shepherd! Bleating ewes fill the morning air.", "Well met, {other}! Cloaked in wool and leaning on thy crook.", "Hail, drover of the flock! How fare the sheep on the commons?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:shepherd"],
            respond: None,
            lines: &["Morrow, neighbor! Watching for hawks and counting tails.", "Well met! The bells on the wethers make a pleasant music.", "Greetings! Wind on the hill, but the ewes are peaceful.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy vats never sour, brewer. Fair day.", "I will bring my pewter mug at sundown. Farewell.", "A good brew makes neighbors into brothers. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_MASH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:brewer"],
    calls: &["Is the new brewing of October ale ready for the tap?", "Did the maltsters bring in the sprouted barley from the floor?", "Dost thou have small beer to sell for the field workers' pails?", "Has the barley mash cooled enough to pitch the yeast?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:brewer"],
            respond: None,
            lines: &["It has fermented sweet and strong; we broach the cask today.", "Three carts of roasted malt came down from the kiln yesterday.", "Aye, fill thy pails; small beer keeps the reapers from fainting.", "The ale has cleared and the barrel is broached sweet."],
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
    calls: &["Good morrow, brewer! Warm steam rises over thy brew-house.", "Well met, {other}! The rich scent of mash drifts through the lane.", "Hail to the tapster! Tapping a fresh barrel this morning?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:brewer"],
            respond: None,
            lines: &["Morrow, {other}! Stirring the copper vat since before cockcrow.", "Well met! The yeast is working its miracle in the vats.", "Greetings! A thirsty throat finds a ready welcome here.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["A man's reckoning on the slate grows longer than his memory of it.", "Two measures on the slate and a third if thou singest for it.", "The bench by the fire is the warmest seat in the parish, and it costs nothing.", "I hear every quarrel in this village twice - once drunk and once sober."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Chalk it up and I will square it at Michaelmas. Fare well.", "Thou keepest an honest house, taverner. Good day.", "Keep the bench warm; I will be back before dark.", "May thy slate be short and thy barrel long. Farewell.", "God's peace on thy threshold, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const TAVERNER_BARREL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:taverner"],
    calls: &["What is on tap tonight - the alewife's small ale or the brewer's October?", "Didst thou lay in a firkin off the vintner before the frost?", "Is there a bed to be had, or must I lie in the straw with the carters?", "What does a measure cost a man who has walked twelve miles for it?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:taverner"],
            respond: None,
            lines: &["Small ale from the alehouse this morning, and it will be sour by Friday - drink it now.", "I bought the brewer out at three farthings the measure and I pour it at a penny. That is the whole of my trade.", "There is a tick of clean straw above the kitchen and a bolt on the door. Sixpence.", "Four pence, and the fire is free, and so is everything thou wilt hear beside it."],
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
    calls: &["Good morrow, taverner! The ale-stake is out, so there is something in the barrel.", "Well met, {other}! A pint of the good dark, if the barrel holds.", "Hail the keeper of the one table in this village. Is the fire lit?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:taverner"],
            respond: None,
            lines: &["Morrow, {other}! Sit where thou likest, the far end is dry.", "Well met! I brew nothing and I sell everything - come in.", "Greetings! A thirsty throat and a civil tongue are all this house asks.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy hives thrive through the winter, beekeeper. Fair day.", "I will buy a jar of comb at Michaelmas. Farewell.", "The little workers teach us all to labor together. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_HONEY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:beekeeper"],
    calls: &["Did the swarm from the orchard settle in the new straw skep?", "Can I buy a pot of honey to soothe my child's raw throat?", "Hast thou cakes of beeswax ready for the shoemaker's thread?", "Are the worker bees flying strong to the heather hills?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, beekeeper! The apiary hums like an organ in the sunshine.", "Well met, {other}! Straw skeps standing tidy in the garden rows.", "Hail, master of the hives! Smoking the bees this morning?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:beekeeper"],
            respond: None,
            lines: &["Morrow, {other}! The bees are flying thick to the heather hills.", "Well met! Sweet honey on the comb and wax for the monastery.", "Greetings! Walk soft, neighbor; the queen is in her prime.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["Thy knowledge has saved many a soul in this town. Fare well.", "I will brew the tea as thou biddest. Good day.", "May the earth yield its medicine to thee. Farewell.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_DRAUGHT: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:herb_wife"],
    calls: &["Hast thou a salve for stiff knees and swollen knuckles?", "Canst thou brew a draught for the fever that shakes the child?", "What herb will keep the pests away from our flour bin?", "What root will soothe a racking cough in the chest?"], voice: &[],
    replies: &[
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
    calls: &["Good morrow, wise woman {other}! Bunches of herbs hang drying everywhere.", "Well met, herb-wife! Thy basket is full of green leaves from the fen.", "Hail, healer of the sick! Gathering roots in the dew?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:herb_wife"],
            respond: None,
            lines: &["Morrow, {other}! The earth gives her secrets to those who look.", "Well met! Stinging nettle and plantain leaves gathered at sunrise.", "Greetings, neighbor! What ails thee or thy folk today?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["A good trade leaves both men smiling when hands are struck.", "Silver coin or salted hams - sound goods are always welcome.", "The road is long, but fair trade makes every mile worthwhile.", "The pack mules know every turn of the king's road."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair journey on the highway, trader. Good day.", "May thy pack horses stay sure-footed on the ridge.", "Until the spring fair, merchant. Fare well.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_WARES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:merchant"],
    calls: &["Dost thou carry steel sewing needles and linen thread from the city?", "What price dost thou ask for a pound of sea salt?", "Hast thou any ribbons of silk or spice from overseas?", "What exchange dost thou ask for a bag of sea salt?"], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:merchant"],
            respond: None,
            lines: &["Fine iron needles in a bone case; two eggs or a penny each.", "White sea salt from the coast; three groats for a full measure.", "Scarlet ribbon from Flanders and a pinch of black pepper to trade.", "Two silver pennies or a salted ham buys the measure."],
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
    calls: &["Good morrow, trader! Thy pack mules are laden heavy from the highway.", "Well met, merchant! What exotic wares bringest thou to our valley?", "Hail to the traveling merchant! How stood the roads through the forest?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:merchant"],
            respond: None,
            lines: &["Morrow, good folk! Goods from the sea ports and goods from the hills.", "Well met! Unbuckling the leather packs; step up and see the wares.", "Greetings! Mud on my boots, but treasures in my packs for honest barter.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["Aye, and debt is a chain that rattles at night.", "Thou speakest the trader's own gospel.", "True enough - a debt unpaid is a friend unmade."],
            voice: &[],
            then: Some(MERCHANT_BARTER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Fair profit to thy ledger, merchant. Good day.", "I'll mind my own accounts, thank thee kindly.", "Until the next fair, trader."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEDGER_ROADS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:merchant"],
    calls: &["What news of the wool price in the towns, {other}?", "Knowest thou a steady hand for the autumn caravan?", "What exchange does the village offer for good salt this season?", "Hast thou goods to trade, or only gossip?"], voice: &[],
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
            lines: &["And fair coin to thy purse, merchant.", "Well met, trader - the village watches the road for thy pack.", "Morrow - what does the wide world charge for news today?"],
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
            lines: &["We will see what the manor court decides on Tuesday. Good day.", "May justice be remembered at the high table. Fare well.", "I take my leave of the manor gate. Fair day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
    calls: &["Good morrow to the manor's steward. We come on village business.", "Well met, sir. We seek speech concerning the leases and tallies.", "Greetings, lord's man. The tenants have gathered to speak.", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["concern:rents"],
            respond: None,
            lines: &["Morrow, tenants. The tally sticks are on the table; state thy grievance.", "Well met, villagers. The lord listens to lawful petitions.", "Speak thy mind plainly, goodmen; the court roll is open.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
    calls: &["The steward keeps count of every sheaf from thy strip, goodman.", "My bailiff tells me the common ditch wants scouring before the rains.", "A good tenant renders his boon-days before the tally is read.", "The deer have been at the lower field again - keep thy dogs close."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:rents"],
            respond: None,
            lines: &["I'll see to it before the week's end, m'lord.", "As thou sayest, m'lord.", "The ditch will be cleared by the quarter day, m'lord."],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.30, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:rents"],
            respond: None,
            lines: &["The rents bite deeper than the frost, begging m'lord's pardon.", "Custom says two days, m'lord - not three.", "My hands are cracked enough without fresh burdens, m'lord."],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thy servant, m'lord. Good day.", "I must be about the fields, m'lord.", "God keep thee and thy house, m'lord."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEVEE_LAW: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:rents"],
    calls: &["The manor court sits under the oak on Thursday - bring thy witnesses.", "Custom binds us both, goodman - thy strip, my tallage.", "The rolls are kept fair; what thy father owed, thou owest, no more.", "A lord's word is the law of the lane, goodman."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["I'll stand at the oak on Thursday, m'lord.", "Custom holds, m'lord - my strip, thy tallage.", "Thy servant, m'lord - and my thanks for the hearing."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const LEVEE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:rents"],
    calls: &["Aye, goodman - out with the dawn, I see.", "Morrow, {other}. Thy hedge wants mending on the north side.", "Well met, tenant - the manor hears its people."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["concern:rents"],
            respond: None,
            lines: &["M'lord. An honor to be seen.", "Aye, m'lord - good morrow to the manor.", "At thy service, m'lord."],
            voice: &[],
            then: Some(LEVEE_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Question, concepts: &["concern:rents", "aid"],
            respond: None,
            lines: &["M'lord - might the hail damage weigh on the tallage?", "A word of grace, m'lord - the oats failed on the lower strip.", "Will the court hear our plea about the common ditch, m'lord?"],
            voice: &[],
            then: Some(LEVEE_LAW), end: None,
        },
        Reply {
            criteria: &[], weight: 0.30, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thy servant, m'lord.", "I dare not keep the manor waiting, m'lord.", "God keep thee, m'lord."],
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
            lines: &["May thy nets come up heavy with silver fish. Fair day.", "Mind the river current by the mill dam. Farewell.", "I will bring an eel pie to the festival. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
    calls: &["Good morrow, riverman! Thy boots are wet to the knee.", "Well met, {other}! Mending thy linen gill-nets on the bank?", "Hail, fisher of the weir! Any fish in the basket today?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:fisher"],
            respond: None,
            lines: &["Morrow, {other}! The river is awake and cold as ice.", "Well met! Tying knots and checking the willow osiers.", "Greetings! River water in my veins and fresh catch in the creel.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
            lines: &["May thy stacks burn true without flaring. Fair day.", "I will tell the smith his coals are ready. Farewell.", "Watch the sparks in the dry brushwood, collier. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_SMOKE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:charcoal"],
    calls: &["Is the earth-covered mound ready to open and rake?", "Canst thou spare a sack of charcoal for the tavern braziers?", "Did the night wind blow the turf cover off the kiln?", "How burns the earth-covered pit mound in the clearing?"], voice: &[],
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
    calls: &["Good morrow, charcoal burner! Smoke drifts thick from the deep wood.", "Well met, {other}! Black as an imp from tending the pits.", "Hail, master of the kiln! How burns the timber mound?", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:charcoal"],
            respond: None,
            lines: &["Morrow, {other}! Watching the slow smoke day and night.", "Well met! Soot on my face, but clean fuel for the iron forge.", "Greetings! The mound is smoking sweet under its turf blanket.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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

const ELDER_BLESS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["elder"],
    calls: &["Lords come and go, but the plough and the furrow remain.", "Listen to the land; it was here before us and will keep our graves.", "Kindness to thy neighbor is the best hedge against misfortune.", "Seasons cycle and return as they have always done."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Thy words are gold to my ears, elder. Fair day.", "May God grant thee peace in thy old age. Farewell.", "We will heed thy counsel at the village meeting. Good day.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_WINTERS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["elder"],
    calls: &["Dost thou recall the great famine fifty winters ago, grandfather?", "Where did the old village boundary run before the ditch was dug?", "What signs tell us when the frost will finally break?", "Dost thou recall how the great freeze broke forty years ago?"], voice: &[],
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
    calls: &["Good morrow to thee, venerable {other}.", "Well met, grandfather! Sitting in the sunny doorway to warm thy joints?", "Greetings, mother of the village! How fares thy health today?", "A peaceful day to thee, {other}."],
    voice: &[
        Voice { criteria: &[Has("pair:relation:friend")], lines: &["Good morrow, {other} - eighty winters and still the sharpest tongue on the lane.", "Well met, old friend! Still warming those joints in the sun?"] },
        Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, {other} - the family's oldest lamp still burning.", "Grandmother! Thy sunny doorway kept a seat for me, I hope."] },
        Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, elder. The road respects grey hair.", "A fair day to thee, mother of the village."] },
        Voice { criteria: &[Has("pair:relation:acquaintance")], lines: &["Good morrow to thee, venerable {other}.", "Well met, elder - how fares thy health?"] },
        Voice { criteria: &[Has("pair:relation:rival")], lines: &["Morrow, {other} - eighty winters and thou still risest to spite me.", "Well met, elder. Even rivals bow to the winters on thy head."] },
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["elder"],
            respond: None,
            lines: &["Morrow, young soul. The sun feels good on eighty winters of bones.", "Well met, child! Another day Heaven grants to watch the fields.", "Greetings! My step is slow, but my memory is long and clear.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            voice: &[
                Voice { criteria: &[Has("pair:relation:friend")], lines: &["Morrow, {other} - my step is slow but my ears miss nothing, especially from thee.", "Well met, child - I know thy name and all thy doings, fondly."] },
                Voice { criteria: &[Has("pair:relation:kin")], lines: &["Morrow, little one - the family's roots run through thee.", "Bless thee, kin. Come sit and tell me the house's news."] },
                Voice { criteria: &[Has("pair:relation:stranger")], lines: &["Morrow, stranger. Grey ears still catch new voices.", "Well met, traveler - sit, if the road is long."] },
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Aye, {other} - I've buried better rivals than thee.", "Morrow, {other}. Age teaches patience, even with thee."] },
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
                Voice { criteria: &[Has("pair:relation:rival")], lines: &["Away then, {other} - my sun's too short for thee.", "Go on - an old woman's time isn't thine to waste."] },
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
            lines: &["May the saints guard the cradle day and night. Fair day.", "I will bring goat's milk for the mother. Farewell.", "Joy to thy house, {other}! God bless the babe.", "May the road be kind under thy boots.", "God's peace be upon thy cottage, {other}."],
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
            lines: &["Aye, wrapped in soft lamb's wool, dreaming sweet dreams.", "On Sunday morning when the great church bell tolls.", "The neighbors brought honey broth and fresh eggs for her recovery.", "Our kin will stand beside thee through good season and bad."],
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
    calls: &["Joy to thy household, {other}! I heard a newborn babe has arrived!", "Good morrow, new mother! How fares the precious child?", "Blessings on the cradle, {other}! What a joyous day for our lane!", "A peaceful day to thee, {other}."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            respond: None,
            lines: &["Morrow, {other}! A healthy babe, praise be to Heaven!", "Well met! Exhausted, but my heart is overflowing with gratitude.", "Greetings! A little girl with eyes bright as the morning sky.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
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
        lines: &["Aye, there is the truth of it.", "{other} has the right of it.", "It is so - my house has felt the same.", "Well spoken, neighbor; we stand with thee."],
    },
    MeetingLine {
        act: Act::PushBack, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons", "concern:grief"], bias: &[],
        lines: &["Talk is cheap - who will lift a hand to do aught about it?", "And what wouldst thou have the village do, {other}?", "We have argued this before and accomplished nothing."],
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
        lines: &["My own table's felt it, I promise you - nobody here keeps a table like mine, for good or ill.",
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
        lines: &["I feed the whole road at my tables. Pool the stores and I'll stretch them further than any of you."] }),
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
            "I've buried and married most of you. The church will stand with you, if you'll come back to it."] }),
];

/// A worked-example agenda for a house meeting
pub const MEET_PROMPT: MeetingPrompt = MeetingPrompt {
    text: "What ails the village most, friends? Speak true.",
    concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:fuel", "concern:bridge", "concern:harvest", "concern:commons", "concern:grief"],
};

/// The probing question (#1267): not the village's trouble but the
/// person's own - what they want, and who they're doing it for
pub const MEET_PROBE: MeetingPrompt = MeetingPrompt {
    text: "Leave the village's troubles a moment. What do you actually want - and who are you doing it for?",
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
    "God go with you all, and bring you back next time.",
    "May what we spoke of take root. Amen, and good night.",
];
const CLOSE_OLD: &[&str] = &[
    "The stones watch you home.",
    "Keep the old ways, and keep each other.",
];
const CLOSE_PLAIN: &[&str] = &[
    "That's the whole of it. Go well.",
    "Home safe, all of you - and thank you for coming.",
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
            "They say a free hand makes warm neighbors. What's thy mind on it?",
            "Some folk give till it hurts. Thin soup for the giver, say I - and thou?",
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
            "The bells call us all the same. Dost keep the faith?",
            "Some say the old prayers are only words. And thou?",
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
            "The miller had the right of it in that quarrel - thou'lt agree, surely?",
            "I'd have thumped the steward myself. Thou'd have done the same, no doubt?",
        ],
        pass: (Act::PushBack, &[
            "I say no such thing. The miller was half to blame and more.",
            "Not I. A quick fist makes a slow friend.",
            "Thou'lt not put words in my mouth, friend.",
        ]),
        fail: (Act::Agree, &[
            "Aye... I suppose thou'rt right about that.",
            "As thou sayest, no doubt it's so.",
            "I'll not argue it, if it pleases thee.",
        ]),
    },
    // the proud test whether you'll treat them as they reckon themselves
    Probe {
        who: Trait::Proud,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "Folk here know to speak fair to me. Thou'lt have heard as much?",
            "My family held this land when thine were sowing wild oats. Dost know it?",
        ],
        pass: (Act::Agree, &[
            "Aye, and I've heard nothing but good of thy house.",
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
            "Hast heard what they say of the reeve's new hound?",
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
            "A body's meant for more than the furrow, eh? Or is that thy lot?",
            "I mean to be more than I was born. And thou - content where thou standest?",
        ],
        pass: (Act::Agree, &[
            "More, aye - I've plans of my own, given half a chance.",
            "I mean to rise same as thee, if the saints allow it.",
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
            "Poor mite - I'll ask after them tomorrow, if thou'lt allow.",
            "That's a hard row. Can I carry aught for thee?",
        ]),
        fail: (Act::Remark, &[
            "Knees ache. Such is age, friend.",
            "Children cough. They grow out of it.",
            "We've all our troubles, mine no lighter than thine.",
        ]),
    },
    // the lazy test whether you'll be easy company: no hustle-preaching
    Probe {
        who: Trait::Lazy,
        act: Act::Question,
        concepts: &["audition"],
        calls: &[
            "Hard work's for them as never learned better, eh?",
            "I say a body earns their rest. Thou'lt not disagree?",
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
}

/// The topics a Talk's ask-slot can open (#1189). Personal questions
/// are always available; the rest appear when the state supports them -
/// a live status, a fact already shared, a secret still kept.
pub const ASK_TOPICS: &[Topic] = &[
    Topic {
        label: "the village",
        area: crate::game::conversation::Area::Village,
        tier: 0, needs: &[], any: &[], not: &["visitor"],
        ask: &["What word of the village these days?", "How do folk fare here, to thy eyes?"],
        tell: &["Folk fare as folk do - some well, some not so.", "The village keeps its own counsel, mostly.",
            "There's grumbling at the well, but when isn't there.", "Quieter than it was, and that's no bad thing."],
        concepts: &["village", "gossip"], draws: Draw::No,
    },
    Topic {
        label: "the road",
        area: crate::game::conversation::Area::Road,
        tier: 1, needs: &["visitor"], any: &[], not: &[],
        ask: &["What word from the road?", "What news do the roads carry?"],
        tell: &["The roads are what they are - long, and thin of comfort.", "Word travels: the country's unsettled of late.",
            "I've seen worse lanes and better company."],
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
        ask: &["How fares the work, friend?", "Tell me of thy craft - how goes it?"],
        tell: &["It keeps me, as honest work keeps any.", "Well enough - the hands know their business.",
            "Slower than I'd like this season, but it pays.", "A craft's a craft - it fills the day and the purse, mostly."],
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
        ask: &["How fares thy day's work?", "The day's labor - how does it treat thee?"],
        tell: &["A holding keeps a soul busy, one way and another.", "The fields are what they are - they want what they want.",
            "I take what work comes, and thank the saints for it."],
        concepts: &["work"], draws: Draw::No,
    },
    Topic {
        label: "their sickness",
        area: crate::game::conversation::Area::Body,
        tier: 1, needs: &["sick"], any: &[], not: &[],
        ask: &["How does the sickness fare with thee?", "Thou lookest pale - is the illness past?"],
        tell: &["It lingers - worse some days than others.", "I mend, slowly. The herb-wife says patience.",
            "The cough keeps me nights, truth be told."],
        concepts: &["sickness", "health"], draws: Draw::No,
    },
    Topic {
        label: "their hunger",
        area: crate::game::conversation::Area::Body,
        tier: 1, needs: &["hungry"], any: &[], not: &[],
        ask: &["When did thou last eat a full meal?", "Is bread short at thy table?"],
        tell: &["A belly remembers bread better than days.", "We make do - there's always less to make do with.",
            "The store's thin. The winter's long."],
        concepts: &["hunger", "bread"], draws: Draw::No,
    },
    Topic {
        label: "their grief",
        area: crate::game::conversation::Area::Heart,
        tier: 2, needs: &["grieving"], any: &[], not: &[],
        ask: &["Who was it thou lost?", "The house sits heavy - who is it thou missest?"],
        tell: &["Someone the house still feels the lack of.", "I don't say the name much - it keeps.",
            "The chair by the fire's empty now. That's the whole of it."],
        concepts: &["grief", "family"], draws: Draw::No,
    },
    Topic {
        label: "the trouble they told",
        area: crate::game::conversation::Area::Trouble,
        tier: 1, needs: &["shared"], any: &[], not: &[],
        ask: &["That trouble thou spokest of - what weighs on thee?", "Thou saidst something of it once - how goes it now?"],
        tell: &["Aye, it sits with me still.", "The same as was - the asking's kind, though.",
            "It hasn't eased, if that's the truth of it."],
        concepts: &["trouble", "burden"], draws: Draw::Secret,
    },
    Topic {
        label: "what weighs on them",
        area: crate::game::conversation::Area::Trouble,
        tier: 2, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["Is aught weighing on thee? Thou'lt tell me, if there is.", "Something sits behind thy eyes - what is it?"],
        tell: &["Ah. Thou'rt sharp-eyed, then.", "I had hoped it didn't show.", "If anyone's to hear it, thou'rt as good as any."],
        concepts: &["trouble", "secret"], draws: Draw::Secret,
    },
    Topic {
        label: "the question folk whisper of",
        area: crate::game::conversation::Area::Village,
        tier: 1, needs: &["mystery"], any: &[], not: &[],
        ask: &["Folk whisper of something - what is it they ask?", "There's a question going round the village - what do folk say?"],
        tell: &["Ah, you've heard the asking too.", "Aye, folk do talk - and some of it's true.", "I've a piece of it, if you want the hearing."],
        concepts: &["mystery", "gossip"], draws: Draw::Mystery,
    },
    Topic {
        label: "themselves",
        area: crate::game::conversation::Area::Heart,
        tier: 1, needs: &[], any: &[], not: &[],
        ask: &["Tell me of thyself - how didst thou come here?", "What's thy tale, then?"],
        tell: &["Born under this roof, like most of us.", "I've no great tale - the years just keep.",
            "Same story as the next one's, only mine.", "I came for the work and stayed for the quiet."],
        concepts: &["self", "history"], draws: Draw::No,
    },
    // the deepest asks (E3-4, #1425): tier 3 - a kept secret tells in
    // the fact's own words when the carrier holds one, else these
    Topic {
        label: "the weight they carry",
        area: crate::game::conversation::Area::Heart,
        tier: 2, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["Thou carriest something, friend - what is it weighs on thee?"],
        tell: &["It's a weight, is all - I carry it as I can.",
            "Some things a body just keeps carrying."],
        concepts: &["grief", "burden"], draws: Draw::Secret,
    },
    Topic {
        label: "what they never told",
        area: crate::game::conversation::Area::Trouble,
        tier: 3, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["There's something thou hast never told a soul - what is it?"],
        tell: &["Thou'rt the first to ask it plain. I don't know that I can.",
            "That's not a thing I've said aloud."],
        concepts: &["trouble", "secret"], draws: Draw::Secret,
    },
    Topic {
        label: "the thing nearest the bone",
        area: crate::game::conversation::Area::Heart,
        tier: 3, needs: &[], any: &["keeps_secret", "keeps_wound"], not: &[],
        ask: &["What sits nearest the bone with thee, if thou'lt tell it?"],
        tell: &["The thing nearest the bone stays there, mostly.",
            "If I say it, it stops being only mine."],
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
    calls: &["Race thee to the well and back!", "Last one to the mill pond is a mouldy turnip!", "Mother's calling. I'll be skinned if I'm late again."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            respond: None,
            lines: &["Not fair, thou hadst a head start!", "Tomorrow, then. Same place.", "Run, {other}, run!"],
            voice: &[],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIDS_DARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["play"],
    calls: &["I dare thee to knock on the hermit's cave.", "Wouldst thou climb the church wall if I held thy boots?", "Hast thou seen the miller's new pup? It bit Tom."], voice: &[],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Proud, 1.5), (Hotheaded, 1.5)],
            act: Act::Agree, concepts: &["play"],
            respond: None,
            lines: &["Easy! Watch me.", "Double dare thee back, then.", "Only if thou goest first."],
            voice: &[],
            then: Some(KIDS_RUN), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Kind, 1.5), (Lazy, 1.5)],
            act: Act::PushBack, concepts: &["play"],
            respond: None,
            lines: &["Not likely - my father would tan my hide.", "Thou'rt cracked in the head, {other}.", "I'm not scared. I just don't want to."],
            voice: &[],
            then: Some(KIDS_RUN), end: None,
        },
    ],
};

const KIDS_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "play"],
    calls: &["Oi, {other}! Come and see what I found.", "{other}! Hast thou any errands, or canst thou play?", "Psst, {other} - over here."], voice: &[],
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
        DialogueTree { name: "elder_wisdom", criteria: &[Has("self:status:elder")],
            root: &ELDER_GREET, rejoin: Some(&ELDER_WINTERS) },
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
    if let Some(tr) = s.status.iter().copied().find(|st| matches!(*st,
        "smith" | "miller" | "baker" | "weaver" | "carpenter" | "mason"
        | "shepherd" | "brewer" | "taverner" | "beekeeper" | "herb_wife"
        | "fisher" | "charcoal_burner")) {
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
            _ => "the mounds",
        };
    }
    match s.ideology {
        "clergy" => "the parish",
        "merchant" => "the ledger",
        "lord" => "the manor",
        _ => "the day's work",
    }
}

/// A one-line backstory from a speaker's profile
pub fn backstory(s: &Speaker) -> String {
    let mut parts: Vec<String> = vec![];
    // the lead's their real standing: a craft named in their statuses
    // outranks the ideology's guess, and a child is a child
    let trade = s.status.iter().copied().find(|st| matches!(*st,
        "smith" | "miller" | "baker" | "weaver" | "carpenter" | "mason"
        | "shepherd" | "brewer" | "taverner" | "beekeeper" | "herb_wife"
        | "fisher" | "charcoal_burner"));
    if s.status.iter().any(|st| *st == "child") {
        parts.push(format!("{} is still a child.", s.name));
    } else if let Some(tr) = trade {
        parts.push(format!("{} is the village {}.", s.name, tr.replace('_', " ")));
    } else {
        parts.push(match s.ideology {
            "clergy" => format!("{} serves the village church.", s.name),
            "merchant" => format!("{} trades on the road between villages.", s.name),
            "lord" => format!("{} holds the manor's leases.", s.name),
            _ => format!("{} works the fields for a day's wage.", s.name),
        });
    }
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
            Trait::Steady => "They'll finish the furrow before they curse the stone.".into(),
            Trait::Wary => "They weigh a stranger twice before a word.".into(),
            Trait::Stolid => "They keep their own counsel, and keep it well.".into(),
            Trait::Cruel => "Their hand is heavy and their tongue heavier.".into(),
            Trait::SelfAbsorbed => "They'll talk your ear off and never once ask after yours.".into(),
        });
    }
    for st in &s.status {
        if *st == "child" { continue; }
        parts.push(match *st {
            "sick" => "A cough's had them these two weeks.".into(),
            "hungry" => "The store ran out and the belly knows it.".into(),
            "grieving" => "They buried one of their own last week.".into(),
            "drunk" => "Last night's cup is still on them.".into(),
            "cold" => "Their hearth went out and fuel is spent.".into(),
            "elder" => "Eighty winters of village memories are in their bones.".into(),
            "newborn" => "A new babe was born to their house this month.".into(),
            // the soul's name, read not as a score but as what folk
            // have learned to expect of them (#950)
            "kind_soul" => "Their table's never bare to a neighbor.".into(),
            "upright" => "They deal straight where another man would bend.".into(),
            "steady" => "Nothing much ruffles them.".into(),
            "shrewd" => "They weigh a thing thrice before they speak on it.".into(),
            "devout" => "They keep the holy days like a second trade.".into(),
            "hopeful" => "They expect the morning to bring good.".into(),
            "bold" => "They step forward when the rest step back.".into(),
            "hard" => "Their door stays shut when a neighbor's pot runs empty.".into(),
            "smith" => "They work the village anvil and forge.".into(),
            "miller" => "They run the watermill at the weir.".into(),
            "baker" => "They tend the communal bread oven.".into(),
            "weaver" => "They weave russet cloth on the loom.".into(),
            "carpenter" => "They square timber and cut mortise joints.".into(),
            "mason" => "They cut freestone and build field walls.".into(),
            "shepherd" => "They herd ewes and lambs on the common pasture.".into(),
            "brewer" => "They brew ale and tend the village vats.".into(),
            "taverner" => "They keep the tavern on the green and buy the drink trades out by the barrel.".into(),
            "beekeeper" => "They keep straw skeps in the clover meadow.".into(),
            "herb_wife" => "They gather roots and brew fever draughts.".into(),
            "fisher" => "They set eel traps in the river current.".into(),
            "charcoal_burner" => "They watch the slow-burning earth mounds in the deep wood.".into(),
            // the powers they run with (#969): the talk can name the pull,
            // the profile says it in words, not tokens
            "parish_faction" => "They keep close to the church folk.".into(),
            "market_faction" => "They keep close to the market folk.".into(),
            "guild_faction" => "They keep close to the guild's masters.".into(),
            "kin_faction" => "Their kin's name opens doors here.".into(),
            "seeker" => "They're forever turning the big questions over.".into(),
            "mystic" => "They speak of God like a neighbor they've met.".into(),
            "sage" => "Their faith seems to hold the whole wide world.".into(),
            // a status the talk keys on that says nothing about who they
            // are (bond stage, already-spoken, visitor plumbing) stays
            // out of the prose
            _ => continue,
        });
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
        "Well met, {other} - how does the morning find thee?",
        "Neighbor! Out early like the rest of us, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Fair enough, and to thee.", "Well enough - and thine?", "Out and about early today."],
            voice: &[],
            then: Some(NEIGHBOR_CHAT), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            respond: None,
            lines: &["Morrow. I'd best keep moving.", "Aye. Good day to thee."],
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
        "Thy roof's holding better than mine in this wind, I'd wager.",
        "Folk say the well water runs sweet again.",
        "Keep an eye on the sky - there's weather in it yet.",
        "Saw thy lass helping at the press yesterday.",
    ],
    voice: &[],
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
        "Good to see thee about, {other} - folk are talking.",
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
        "I've kept my promise - has thine held?",
        "Word travels - keep thy voice low round the square.",
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
        "Ah, {other} - thou knowest well what weighs on me by now.",
        "Thou askest after my cares like an old friend, {other}.",
        "The stranger who asks twice is a stranger no longer, eh {other}?",
        "{other}! Thou'rt coming to know my mind, I'll grant.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["And I mean to do right by what I've heard.", "I listen because it matters, friend.", "Thy troubles are the village's troubles."],
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
        "{other}! Good day - I was hoping I'd see thee about.",
        "There thou art, {other}. Word is thou'rt still talking to folk.",
        "A fair morning to thee, {other} - and to the work we share.",
        "I saved thee a thought or two since last we spoke, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Speak it, then - I came to hear.", "Thou'rt not alone in it, remember.", "Tell me what folk are saying."],
            voice: &[],
            then: Some(BOND_CHAT), end: None },
        Reply { criteria: &[], weight: 0.30, bias: &[(Lazy, 2.0)],
            act: Act::Remark, concepts: &["smalltalk"],
            respond: None,
            lines: &["The work keeps, friend - tell me of thy day instead.", "Rest a moment first."],
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
        "Thou and I have work to plan, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            respond: None,
            lines: &["Let's talk it through - what's on thy mind?", "Then let's settle the next step."],
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
        "Thou'lt call a meeting soon, aye? My kitchen's ready when the word comes.",
        "Folk ask me why I listen to thee - I tell them thou listenest first.",
        "Say the word and I'll bring my cousin too.",
        "The talk goes further than me, mind - folk repeat what thou askest about.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["the_cause"],
            respond: None,
            lines: &["We take it up together, then.", "It'll be soon - keep the word to thyself till then.", "Thou'lt not stand alone in it."],
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
            lines: &["Safe road to you, then.", "Walk soft - and thanks for the word."],
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
            lines: &["Noted - and a good road to you.", "I'll pass the word along."],
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
        "Yours, and keep it well.",
        "The road thanks you, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["A good day to you - and to your road.", "Fair dealing. God speed."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_AGAIN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &[
        "Back again, {other}? The road's no shorter.",
        "You again. I've little new since last we spoke.",
        "Still standing here, {other} - as you see.",
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
        "Come see what the pack holds - I've travelled far for folk like you.",
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
        "You've need of my trade, or know someone who has - which is it?",
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
            lines: &["What have you seen on the road?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["We've no need - good day to you."],
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
        "You've not lived till you've seen what my hands can do, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Gregarious, 2.0)],
            act: Act::Agree, concepts: &["deal", "mirth"],
            respond: None,
            lines: &["A coin for the hat - show us, then.", "Make us laugh, and it's yours."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["Where does the road take your show?"],
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
        "The saints keep the traveller - will you keep one tonight?",
        "A blessing on your fields, friend - freely given, freely received.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Pious, 2.0)],
            act: Act::Agree, concepts: &["deal", "faith"],
            respond: None,
            lines: &["Take this - and a blessing back on you.", "For the road, brother."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["the_road", "faith"],
            respond: None,
            lines: &["Where does your pilgrimage lead?"],
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
        "The stars told me a seeker would meet me here - is it you, {other}?",
        "Your face, {other} - it carries a question. Cross my palm and I'll answer it.",
        "I deal in what the wise pretend not to know - have a care, but hear me.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Pious, 0.5)],
            act: Act::Agree, concepts: &["deal", "omens"],
            respond: None,
            lines: &["Read what you see, then - here's the coin.", "Tell me my fortune, if you can."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["omens"],
            respond: None,
            lines: &["What do you see for this village?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 2.0)],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["The priest says your kind brings ill luck - go."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_CALLER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "the_road"],
    calls: &[
        "A fair day for the road, {other} - God keep your village.",
        "Just passing through, friend - the world's wide and I'm halfway across it.",
        "Well met, {other} - a kind face is rare on this road.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.5, bias: &[],
            act: Act::Question, concepts: &["the_road"],
            respond: None,
            lines: &["Where are you bound, traveller?"],
            voice: &[],
            then: Some(VIS_ROAD_NEWS), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Safe road to you, then."],
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
        "I saw the battle, {other}. Whatever they tell you of it - it was worse.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Go quiet, then - and God speed you."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_DESERTER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "war", "fear"],
    calls: &[
        "Keep your voice down, {other} - there's a price on my head.",
        "I didn't run. I just stopped dying for men I'll never meet.",
        "You'll not turn me in, will you? I only want to eat and sleep.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["deal", "mercy"],
            respond: None,
            lines: &["You'll be safe here a while - keep out of sight.", "Rest, then. I'll say nothing."],
            voice: &[],
            then: Some(VIS_DESERTER_TALE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Pious, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["The watch should know you're here - go, before I shout.", "You bring danger on us all - be gone."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_LEPER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "want", "sickness"],
    calls: &[
        "Unclean! Keep your distance, {other} - but spare a coin if God lives in you.",
        "The bell announces us, but mercy needn't wait for it.",
        "We mean no harm - only bread, and to be seen as men.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 3.0)],
            act: Act::Agree, concepts: &["deal", "charity"],
            respond: None,
            lines: &["Here - food, and a coin with it.", "Stay where you are - I'll bring something."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Stingy, 2.0), (Hotheaded, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["Out - you'll not plague this village.", "Back down the road, before I call folk out."],
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
            lines: &["May the road be kinder to you than it was.", "Go with God - and I'm sorry."],
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
        "We were farmers once, like you. The road owns us now.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Kind, 3.0), (Stingy, 0.3)],
            act: Act::Agree, concepts: &["deal", "charity"],
            respond: None,
            lines: &["Come - there's bread and a fire for the children.", "Take this - it's not much, but it's yours."],
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
            lines: &["Here - a crust and a coin, and God keep you.", "Take this - no man should beg for bread."],
            voice: &[],
            then: Some(VIS_DEAL_DONE), end: None },
        Reply { criteria: &[], weight: 1.0, bias: &[(Stingy, 2.0), (Hotheaded, 1.5)],
            act: Act::PushBack, concepts: &["rebuff"],
            respond: None,
            lines: &["Begone - this village feeds its own poor.", "Down the road with you - I've nothing spare."],
            voice: &[],
            then: None, end: Some(WalksOff) },
    ],
};

const VIS_HERETIC_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["faith"],
    calls: &[
        "Why does a poor man tithe to a rich altar? Ask that, and you're one of us already.",
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
        "I ask questions the priest won't answer - do you dare to listen?",
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
            lines: &["You speak poison - the priest should hear of you.", "This village keeps the faith - be gone."],
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
        "Who among you wants more than this village can ever give? The bounty waits.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 4.0, bias: &[(Ambitious, 2.5)],
            act: Act::Agree, concepts: &["deal", "war"],
            respond: None,
            lines: &["Two silver, you say? I'm listening.", "Tell me more of the service."],
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
            lines: &["You'll get no one here - move along, sergeant."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_FORTUNE_TALE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["omens"],
    calls: &[
        "A dark cloud over the granary - keep your stores dry and your locks oiled.",
        "Someone here will rise, {other} - and the village will rise with them, or fall.",
        "I see a hard winter and a warm hearth - one must be earned before the other comes.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["Keep your omens - I'll make my own luck."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const VIS_FORTUNE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "omens"],
    calls: &[
        "Cross my palm with silver, {other} - the cards have been talking about you.",
        "I saw you coming before you did. A penny tells you what else I saw.",
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
            lines: &["What do you see for the village?"],
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
pub const EWE_SQUEEZE: &str = "We both know what's in your cradle.";
/// The pick text that gets the robbed one to name the thief
pub const EWE_WHOSE_POT: &str = "Whose pot, then? Say it.";

const EWE_CRACKED: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["ewe", "guilt"],
    calls: &[
        "...she walked in herself, so help me. What's done is done.",
        "Ye saw nothing. The house's luck is all it is.",
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
        "You'll have the watch called on you, pushing at folk's doors so.",
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
            lines: &["I'll leave you to it, then."],
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
        "Search if you like - you'll find a babe asleep and nothing more.",
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
            lines: &["You're lying, and we both know it.", "The dye's on your hands, friend."],
            voice: &[],
            then: Some(EWE_STONEWALL), end: None },
        Reply { criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave you to it, then."],
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
            lines: &["I'll leave you to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["ewe", "bluff"],
    calls: &[
        "Hush - the babe sleeps. Say your piece at the door.",
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
            lines: &["I'll leave you to it, then."],
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
        "Mind your own pots, {other}.",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["ewe", "hush"],
            respond: None,
            lines: &["We both know what's in your cradle."],
            voice: &[],
            then: Some(EWE_PAYS), end: None },
        Reply { criteria: &[], weight: 0.6, bias: &[],
            act: Act::Farewell, concepts: &["parting"],
            respond: None,
            lines: &["I'll leave you to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

const EWE_PRICE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["ewe", "hush"],
    calls: &[
        "{other}. What brings you back here?",
        "Aye - what do you want of me now?",
    ],
    voice: &[],
    replies: &[
        Reply { criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["ewe", "hush"],
            respond: None,
            lines: &["We both know what's in your cradle."],
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
            lines: &["I'll leave you to it, then."],
            voice: &[],
            then: None, end: Some(Farewell) },
    ],
};

pub const NOAH_ROW_TREE: &str = "the_noah_row";
/// The mediator's word - the pick that talks one down (#1257)
pub const NOAH_PEACE: &str = "Peace - the whole street's watching.";
/// Backing the one you're talking to (#1257)
pub const NOAH_SIDE_THEM: &str = "You've the right of it - hold your line.";

const NOAH_LAMP: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["peace", "street"],
    calls: &[
        "...the whole street, aye. And me giving it a show.",
        "Huh. Let them find another supper, then.",
        "You're not wrong. Not about the watching, anyway.",
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
        "What? The whole street's had its look already - say your piece.",
        "I'm in no mood for callers, and that's the truth of it.",
        "If you've come to gawp, the gawping's over there with the rest.",
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
            lines: &["I'll leave you to it, then."],
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
pub const FT_ACCUSE: &str = "You're no pedlar. Who sent you?";
/// The pick that calls the runaway's disguise
pub const FT_ACCUSE_RUN: &str = "Those aren't a tinker's questions. Who's after you?";

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
        "And which houses meet nights? Good to know your way, on the road.",
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
            lines: &["I'll leave you to your pack."],
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
        "Say there's a house worth a pedlar's whole stock - which door would you point him at?",
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
            lines: &["I'll leave you to your pack."],
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
        "If a man wanted forgetting by a lord - which road would you tell him?",
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
            lines: &["Safe road to you, then."],
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
