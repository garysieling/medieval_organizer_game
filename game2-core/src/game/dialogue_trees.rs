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
    Reply, Speaker,
};
use crate::game::traits::Trait;
use crate::game::traits::Trait::*;

const SMALL_FADE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["As thou sayest, friend.", "True enough - the fields don't gossip.", "Aye, words won't fill the storehouse.", "We have spoken our piece for today."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Fair day to thee, {other}.", "Go well, {other}, and mind the mud.", "The saints keep thy roof whole.", "Fare thee well until the morrow."],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I take my leave.", "Enough chatter. Good day.", "My day is waiting.", "I have no leisure to stand about, {other}.", "My chores call me away."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SMALL_GOSSIP_MORE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip"],
    calls: &["They do say the reeve counts the sheep twice over.", "The herb-wife was seen out past the abbey wall at dusk.", "Old Wulfric swore he saw an owl perched on the tavern roof.", "A stranger with two laden mules passed by the weir at noon."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["gossip"],
            lines: &["Well, that is talk enough for one morning.", "Ha! I'll believe the half of it and no more.", "Tavern tales grow taller with each ale drawn.", "Every tongue in the parish wags before sunset."],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            lines: &["Better to tend thy own hearth than watch the road.", "I have heard that lie three times this week.", "Folk will invent dragons where there's only sheep.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_GOSSIP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip", "smalltalk"],
    calls: &["Didst thou hear who was seen down by the mill?", "They say the miller took an extra scoop of meal again.", "The peddler's cart lost a wheel at the bridge.", "A rider from the castle came through before dawn."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Remark, concepts: &["gossip"],
            lines: &["Thou dost not say it! What happened then?", "Aye? And what else did folk see?", "Tell on, neighbor - my ears are open.", "I knew that bridge would give way sooner or later."],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            lines: &["Idle tongues, that. It breeds mischief.", "I'll not repeat marketplace whispers.", "Let the miller and the rider sort their own affairs.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["A tale for another hour - I must away.", "The sun climbs high; I have chores to finish.", "Walk with God, {other}, I cannot linger.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_SEASON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "harvest"],
    calls: &["We must thatch the hayrick before the clouds burst.", "The kitchen gardens are dry as a bone this month.", "Rooks are gathering thick in the elm trees.", "The ditch along the south pasture needs clearing."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["harvest"],
            lines: &["Aye, work never sleeps in this valley.", "A dry spade makes for a hungry winter.", "Every hand will be needed when the grain ripens.", "I'll lend a pitchfork if thou hast timber to spare."],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Lazy, 2.5)],
            act: Act::PushBack, concepts: &["harvest"],
            lines: &["The hay will wait until tomorrow, surely.", "Let the rain fall; my bones need a day's rest.", "There's always another ditch to dig.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_FARES: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["family", "smalltalk"],
    calls: &["Well enough, thank thee, {other}.", "We rub along as ever, by the saints' mercy.", "The little ones keep me running from dawn to dusk.", "My knees ache with the damp, but I still walk."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["family"],
            lines: &["Good to hear it - long may health stay under thy roof.", "Aye, children grow like weeds in the summer ditch.", "A full hearth is a blessing in hard times.", "May the coming season be kind to thy cottage."],
            then: Some(SMALL_SEASON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Give my best to thy household. Fare well.", "God keep thy folk in peace, {other}.", "I must see to my own broth. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_VOLLEY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Even so, we will manage as we always do.", "Aye, the year will turn and bring what it brings.", "A steady hand and a patient mule see it through.", "No use fretting over skies we cannot steer."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            lines: &["That it will - the store is not empty yet.", "Aye, we have lived through harder winters than this.", "True words; worry never baked a single loaf.", "So long as the well stays sweet, we'll thrive."],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll not keep thee standing in the wind. Go well.", "On thy way then, and mine. Fair day.", "Until the church bell rings, neighbor.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "weather"],
    calls: &["Looks like rain again, doesn't it?", "Cold wind off the moor this morning.", "Gray clouds gathering over the western hill.", "The morning dew was heavy on the clover."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            lines: &["It will be a wet week, mark me.", "The thatch won't thank it if it pours.", "Aye, and the mud will take thy boots right off.", "Good for the barley, but bad for the road."],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["weather"],
            lines: &["Rain? It is the driest week in memory.", "Thou seest a tempest in every puff of smoke.", "My joints say dry weather, whatever the sky says.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Gregarious, 2.0)],
            act: Act::Question, concepts: &["family", "smalltalk"],
            lines: &["And how fares thy folk in this cold snap?", "Tell me true, {other} - how goes it with thee?", "Has thy brother returned from the market town?", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            then: Some(SMALL_FARES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Well, I had best be about my chores.", "I should get on before the bell strikes.", "Fair day to thee, {other}.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &["Good morrow to thee, {other}.", "Well met, neighbor {other}.", "A fair day to thee and thine, {other}.", "God's peace on this path, {other}.", "Hail, {other} - traveling far today?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            lines: &["And to thee, {other}.", "Well met indeed, friend.", "The saints keep thee, {other}.", "Good day and full granaries to thee."],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::Remark, concepts: &["greeting"],
            lines: &["Morrow.", "Aye. What is it, {other}?", "I am here, as thou seest.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[Lacks("pair:relation:friend"), Lacks("pair:relation:kin")], weight: 0.25, bias: &[(Proud, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I have no time for idle talk today.", "Work is waiting at the barn, {other}.", "I walk on business, neighbor.", "I have no leisure to stand about, {other}.", "My chores call me away."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SICK_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness", "aid"],
    calls: &["Thou'rt kind, {other}. The saints remember a neighborly hand.", "My thanks. A warm bowl does more than herbs.", "Bless thee for thinking of our small house.", "Does the fever ease when the sun warms the cottage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Think nothing of it. We look after our own.", "Rest quiet by the coals. I'll check on thee tomorrow.", "The village stands together or it falls. Fare well.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_STUNG: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness"],
    calls: &["It is not the black pestilence, keep thy wits about thee.", "As thou wilt, then - step wide if thou must.", "A little cough won't kill an honest worker.", "Does the fever ease when the sun warms the cottage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["Forgive me - fear makes cowards of us all.", "Thou hast right of it. I spoke too quick.", "No offence meant, {other}. We are all on edge.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            lines: &["A fine friend thou art when someone's throat burns.", "Breathe elsewhere then, knave.", "Stay in thy own yard if thou'rt frightened.", "I have no leisure to stand about, {other}.", "My chores call me away."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SICK_ANSWER: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["Aye, since the heavy frost. The fever burns at dusk and chills at dawn.", "They say the herb-wife's willow draught brings the sweat out.", "My lungs feel like wet wool on a rainy morning.", "Half the north lane is coughing into their sleeves."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["The saints speed thy mending, {other}.", "Drink hot boiled water with a scrape of honey.", "May the affliction pass thee lightly.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            lines: &["I will send broth round when the kettle boils.", "I'll ask Father Anselm to light a candle for thy recovery.", "Sit by the hearth; I'll fetch firewood if thou hast none.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be away before the air catches me too.", "Mind thy cup and bowl. Good day.", "I have no leisure to stand about, {other}.", "My chores call me away.", "I must walk on; fair day."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SICK_EXCUSE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["I'd rest in the straw if the chores would do themselves.", "There is no one else to fetch water from the river.", "The calves won't feed themselves, fever or no.", "The lord's rent waits for no man's cough."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["We labor or we go hungry, that's the hard truth of it.", "Aye, the dirt cares nothing for an aching back.", "A poor man's bed is a luxury he can rarely afford.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Kind, 2.0)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            lines: &["Let me carry thy pail to the well, {other}.", "Go inside; I'll see to thy livestock this evening.", "I have barley meal to spare for thy kettle.", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Rest when thou canst, {other}. Fair day.", "God grant thee strength for the day's toil.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["*cough* Good morrow, {other}.", "Forgive my rough voice - this chest cold won't quit.", "Morrow, {other}. Don't stand too close in the breeze.", "A heavy morning, neighbor. My limbs are like lead."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["Poor soul - get thee back into the warm cottage.", "Thou shouldst be under woolen blankets, not out here.", "Thou lookest pale as a winter moon, {other}.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(SICK_EXCUSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:sickness"],
            lines: &["Is it the ague that took the miller's lad?", "How long has this fever had hold of thee?", "Hast thou seen the herb-wife for a draught?", "Aye, words have weight in a small village.", "Time will tell what comes of this."],
            then: Some(SICK_ANSWER), end: None,
        },
        Reply {
            criteria: &[Lacks("self:status:sick")], weight: 0.50, bias: &[(Proud, 1.5), (Stingy, 1.5)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            lines: &["Keep thy distance, neighbor. I have no wish to catch it.", "I will hear thee from across the ditch.", "Cover thy mouth when thou speakest in the wind.", "Nay, I see it differently, mark me.", "That is not the way the elders told it."],
            then: Some(SICK_STUNG), end: None,
        },
    ],
};

const VISIT_PROMISE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid"],
    calls: &["It is done - look for it by sundown.", "I'll have my girl bring the loaf before dark.", "Rest easy; the parish will not let thee freeze.", "The hours slip by as the sun climbs.", "A good day's toil makes the evening bread sweet."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thou'rt a true neighbor, {other}.", "The blessing of Heaven on thy roof.", "I'll repay the kindness when my legs mend.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_CARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:sickness", "aid"],
    calls: &["Is there aught thou needest brought to the cottage?", "Can I fetch water or chop kindling for thy hearth?", "Dost thou have grain left in the bin?", "Does the fever ease when the sun warms the cottage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["aid", "concern:food"],
            lines: &["A little bread or salt, if thou canst spare it.", "Some dry firewood would keep the chill out of our bones.", "A jug of fresh well water would be a godsend.", "The sweat broke at dawn and the fever feels a little lighter."],
            then: Some(VISIT_PROMISE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Pious, 1.5)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Just a prayer at the church altar, {other}.", "We have enough for today. Pray for my children.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["How dost thou fare today, {other}?", "I heard the sickness took thy household - how is it?", "Peace to this cottage, {other}. How goes the fever?", "Does the fever ease when the sun warms the cottage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:sickness"],
            lines: &["Poorly, if thou'lt have the plain truth.", "Weak as a lamb, but still drawing breath.", "The night was hard, but the morning brings a little ease.", "The sweat broke at dawn and the fever feels a little lighter."],
            then: Some(VISIT_CARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            lines: &["It will pass. We do not need fussing over.", "I have weathered worse than a common cough.", "Nay, I see it differently, mark me.", "That is not the way the elders told it.", "Thou'rt jumping to conclusions like a startled hare."],
            then: Some(VISIT_CARE), end: None,
        },
    ],
};

const GRIEF_CLOSE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["Thank thee for listening, {other} - it lightens the heart.", "It does a body good to speak his name aloud.", "The house feels less empty when a neighbor sits a spell.", "Does the quiet cottage weigh heavy upon thy spirit?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Any hour, {other}. My door is always unlatched.", "We do not forget those who walked this lane with us.", "May holy angels watch over thy sleep tonight.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_ASK: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:grief", "aid"],
    calls: &["A loaf of bread would not go amiss, {other}.", "A few split logs, if thou hast any seasoned oak.", "Someone to watch the sheep for an afternoon.", "Does the quiet cottage weigh heavy upon thy spirit?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["aid", "farewell"],
            lines: &["It is done. Look for my boy with the basket by dusk.", "I will bring it myself before sunset, friend.", "Leave the sheep to me; mourn in peace.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Stingy, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I will see what can be spared from our cellar.", "Times are tight, but I'll see what I can find.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_OPENS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["I keep setting his wooden spoon at table.", "She loved the smell of the damp earth after rain.", "Every corner of the cottage remembers his footsteps.", "Forty winters together, and now only the silence."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            lines: &["Tell me of him - good memories are a shield against darkness.", "She was loved by all who knew her in the village.", "Their toil shaped this parish; their memory will outlive us.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(GRIEF_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll not weary thee with more words. God keep thee.", "Rest thy sorrow in prayer, {other}. Fare well.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["{other}. Forgive me - the house is very still since he passed.", "Morrow, {other}. The bell tolled, and my hearth is dark.", "Well met, friend. The days feel longer than they used to.", "Does the quiet cottage weigh heavy upon thy spirit?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.40, bias: &[(Kind, 2.0), (Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            lines: &["He was an honest soul. The whole village mourns him.", "The saints receive her spirit in eternal peace.", "My heart aches for thee, neighbor.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(GRIEF_OPENS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[(Kind, 1.3)],
            act: Act::Question, concepts: &["concern:grief", "aid"],
            lines: &["Is there any chore I can take off thy shoulders?", "Hast thou eaten a warm bite today, {other}?", "Aye, words have weight in a small village.", "Time will tell what comes of this.", "We shall see how the wind blows tomorrow."],
            then: Some(GRIEF_ASK), end: None,
        },
    ],
};

const CONDOLE_OFFER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief", "aid"],
    calls: &["If thou needest aught at all, say but the word.", "We have flour and eggs; let us share with thy house.", "My sons can plough thy furrow until thou'rt ready.", "Does the quiet cottage weigh heavy upon thy spirit?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thou'rt a true soul, {other}. I will remember it.", "Thy kindness brings tears to an old eye. God bless thee.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["We will manage on our own, but thank thee kindly.", "I will not burden thy house with my sorrow.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CONDOLE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["I was deeply grieved to hear of thy loss, {other}.", "The village stands beside thee in this dark trial.", "God's mercy comfort thee in thy sorrow, neighbor.", "Does the quiet cottage weigh heavy upon thy spirit?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:grief"],
            lines: &["Thou'rt good to come by, {other}. It means much.", "Thank thee for thy prayers and thoughts.", "The road is lonely now, but kind words help.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(CONDOLE_OFFER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["concern:grief"],
            lines: &["We endure what Heaven sends us.", "Tears do not bring back the dead.", "Nay, I see it differently, mark me.", "That is not the way the elders told it.", "Thou'rt jumping to conclusions like a startled hare."],
            then: Some(CONDOLE_OFFER), end: None,
        },
    ],
};

const CLERIC_COMFORT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["The saints watch over thy cottage. I will come by with holy water and draughts.", "Rest by the hearth, child - I'll light a candle and pray for thy household.", "Cast thy burdens before the altar; the Lord does not forget the meek.", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thank thee, father - thy blessing brings comfort.", "The Lord keep thee, father, and preserve thy voice.", "I will be at the church when the bell sounds.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PRAYER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith", "concern:grief", "concern:sickness"],
    calls: &["The Lord comfort thee, child. Tell me of thy affliction.", "Speak, child - my prayers and candles are with thee.", "Open thy heart; what sorrow weighs upon thee?", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:sickness", "concern:grief"],
            lines: &["It is the little one, father - she burns with fever.", "This cough won't leave me, and our store of grain is thin.", "We buried my brother last week, and grief hollows my chest.", "We do what we can with the strength Heaven gives.", "The truth of it is plain for any eye to see."],
            then: Some(CLERIC_COMFORT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PARISH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["How fares thy house in these lean weeks, child?", "Dost thou find peace in thy daily labor, child?", "Remember that honest work in the field is a holy prayer.", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:faith"],
            lines: &["We keep as best we can, father. The toil is hard.", "The season has been lean, father, but faith sustains us.", "I try to walk uprightly before God and my neighbors.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: None, end: None,
        },
        Reply {
            criteria: &[], weight: 0.70, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I will see thee at the Sunday bell, father.", "Pray for our harvest, father. Fair day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Blessings on thee, {other}.", "The peace of the saints be with thee, child.", "God's grace light upon thy steps, {other}.", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Pious, 2.0)],
            act: Act::Greet, concepts: &["concern:faith"],
            lines: &["And on thee, father.", "Blessings on thy holy house as well.", "Peace be with thee, good priest.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(CLERIC_PARISH), end: None,
        },
        Reply {
            criteria: &[Has("self:status:sick")], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:sickness"],
            lines: &["A prayer for me, father - this fever won't quit.", "Father, hear my plea; the sickness grips my throat.", "Aye, words have weight in a small village.", "Time will tell what comes of this.", "We shall see how the wind blows tomorrow."],
            then: Some(CLERIC_PRAYER), end: None,
        },
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 1.00, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:grief"],
            lines: &["Pray for him, father - he was taken so sudden.", "A blessing for my sorrowing cottage, father.", "Aye, words have weight in a small village.", "Time will tell what comes of this.", "We shall see how the wind blows tomorrow."],
            then: Some(CLERIC_PRAYER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 1.5)],
            act: Act::Remark, concepts: &["concern:faith"],
            lines: &["Morrow, father.", "A fair day to thee, priest.", "Aye, words have weight in a small village.", "Time will tell what comes of this.", "We shall see how the wind blows tomorrow."],
            then: Some(CLERIC_PARISH), end: None,
        },
    ],
};

const FATHER_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["Thank thee, father - that lifts a great stone from my mind.", "Thy counsel is wise, father. We will trust in Providence.", "The church has been our refuge through many a cold year.", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Go in peace, {other}, and love thy neighbors.", "The Lord preserve thy hearth and thy children.", "Walk in the light, child. Fare well.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_ASKS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["Must the tithe stand full when the granary is half empty, father?", "Can the church spare a peck of grain for the hungry children, father?", "Will the parish altar provide candles for our sick neighbor, father?", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:faith"],
            lines: &["The church will share its grain, child - none shall starve at the door.", "The tithe stands, but I will speak for thy house at the manor court.", "Holy candles and broth shall be brought to their bedside this very day.", "We do what we can with the strength Heaven gives.", "The truth of it is plain for any eye to see."],
            then: Some(FATHER_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Good morrow, father.", "A word with thee, father, if thou canst spare a moment.", "Greetings, holy father - I seek thy pastoral ear.", "Good morrow, father - canst thou spare a moment for pastoral counsel?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Kind, 1.3)],
            act: Act::Greet, concepts: &["concern:faith"],
            lines: &["The saints keep thee, {other}. What weighs on thee?", "Always, child. Speak freely in the sight of God.", "Peace to thee, {other}. How may I help thy soul?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(FATHER_ASKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const RIVAL_END: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["We are done here.", "I have looked upon thy face long enough.", "Take thy shadow off my land.", "Dost thou still claim the ditch on my side of the hedge?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Aye. Done.", "Until we meet at the boundary ditch.", "Gladly. The sight of thee sours my stomach.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const RIVAL_BITE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Make me.", "Come closer and say that to my beard.", "Thou'rt all mouth and no marrow.", "Dost thou still claim the ditch on my side of the hedge?"],
    replies: &[
        Reply {
            criteria: &[], weight: 0.80, bias: &[],
            act: Act::Remark, concepts: &["feud"],
            lines: &["Not worth the breath or the bruise.", "I'll settle this before the magistrate, not in the dirt.", "Get out of my sight, {other}.", "I am done looking at thee.", "Walk away before I lose my patience."],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Hotheaded, 4.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Cur! Thou'lt regret those words.", "Keep thy fists to thyself before I break them.", "Get out of my sight, {other}.", "I am done looking at thee.", "Walk away before I lose my patience."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const RIVAL_SNIPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Heard about thy boundary fence. Shoddy timber, that.", "Thy lot still owes mine three groats, whatever lies thou tellest.", "Keep thy lean cows out of our clover pasture.", "I saw where thy cart rutted the lane by my ditch."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["My fence stands straighter than thy crooked word.", "We paid that debt at Michaelmas, as the reeve well knows.", "Thy pasture is mostly thistles anyway.", "Get out of my sight, {other}.", "I am done looking at thee."],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Say that again, if thou darest.", "Thou hast a treacherous tongue in thy skull.", "Get out of my sight, {other}.", "I am done looking at thee.", "Walk away before I lose my patience."],
            then: Some(RIVAL_BITE), end: None,
        },
    ],
};

const RIVAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "feud"],
    calls: &["You.", "Look what the crows dropped on the highway.", "{other}. Still walking upright, I see.", "What business brings thy ugly mug across my path?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["feud"],
            lines: &["{other}.", "I walk where I please in this parish.", "Save thy bile for someone who fears it.", "Get out of my sight, {other}.", "I am done looking at thee."],
            then: Some(RIVAL_SNIPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.80, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Still breathing, I see. A great pity for the town.", "The years have only made thee sourer.", "Get out of my sight, {other}.", "I am done looking at thee.", "Walk away before I lose my patience."],
            then: Some(RIVAL_BITE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I have nothing to say to thee, scoundrel.", "Out of my way. I don't parley with knaves.", "I have no leisure to stand about, {other}.", "My chores call me away.", "I must walk on; fair day."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const HUNGER_HOPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:food"],
    calls: &["If the rye crop ripens before the frost, we'll see another spring.", "They say the monastery may open its tithe barn if hunger deepens.", "We have boiled nettles and wild leeks before, and survived.", "Hast thou a handful of barley meal to spare for the pot?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Aye, hold fast. Better days will follow.", "The earth will feed us yet. Go gently, {other}.", "May tomorrow bring a fuller kettle.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const HUNGER_SHARE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:food", "aid"],
    calls: &["We boiled a pot of turnip tops and barley chaff this noon.", "My store bin has nothing but dust and two dried apples.", "The children went to sleep crying with empty bellies last night.", "Hast thou a handful of barley meal to spare for the pot?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.20, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:food", "aid"],
            lines: &["Take half my loaf, neighbor. No child should hunger in this village.", "Come by our hearth; there is pottage enough for two more bowls.", "We have a sack of roots in the cellar. Let us divide it.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(HUNGER_HOPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[(Stingy, 3.0)],
            act: Act::PushBack, concepts: &["concern:food"],
            lines: &["Everyone's bin is empty; don't look to my door for bread.", "We each must stretch our own store as best we can.", "I have no leisure to stand about, {other}.", "My chores call me away.", "I must walk on; fair day."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const HUNGER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:food"],
    calls: &["Good morrow, {other}. A hollow belly makes for a slow step.", "Well met, friend, though the times are thin and hard.", "Greetings, {other}. Hast thou found anything to put in the pot?", "Hast thou a handful of barley meal to spare for the pot?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:food"],
            lines: &["Morrow, {other}. The porridge is thin as ditchwater this week.", "Aye, we scrape the bottom of the flour chest.", "The storehouse has dwindled down to crumbs.", "Take half this barley loaf; we must look after each other."],
            then: Some(HUNGER_SHARE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["We tighten our belts and carry on.", "Talk won't fill the larder. I must seek wild greens.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const COLD_FIRE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:fuel"],
    calls: &["The kindling caught! Sweet smoke rises up the chimney at last.", "Blow on the embers gently - the dry twigs are taking flame.", "Ah, blessed warmth! My stiff fingers can move again.", "Can we gather dry brushwood together in the common woods?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Bank the coals with ashes so they last till dawn.", "May thy hearth keep hot through the bitter night.", "The fire is life in winter. Fare thee well.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const COLD_GATHER: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:fuel", "aid"],
    calls: &["Will thou come to the common wood to cut deadwood and kindling?", "Can I borrow a shovel to dig peat from the dry bog?", "Hast thou glowing embers in an iron pot to rekindle our cold stones?", "Can we gather dry brushwood together in the common woods?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:fuel"],
            lines: &["Aye, I'll bring my hatchet. Two bundles of dry boughs will do.", "Take these live embers from my hearth; shield them from the wind.", "We stacked dry turf behind the byre; take what thou canst carry.", "Take these live embers in an iron pot to start thy fire."],
            then: Some(COLD_FIRE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Stingy, 2.0)],
            act: Act::PushBack, concepts: &["concern:fuel"],
            lines: &["The bailiff watches the common wood; he will fine us for gathering boughs.", "My own hearth barely has coals to warm an egg.", "I have no leisure to stand about, {other}.", "My chores call me away.", "I must walk on; fair day."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const COLD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:fuel"],
    calls: &["Brrr! Good morrow, {other}. The rime frost is inside the thatch.", "Well met, neighbor. My breath freezes right before my nose.", "Greetings, {other}. The wind blows straight through the cottage wattles.", "Can we gather dry brushwood together in the common woods?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Agree, concepts: &["concern:fuel"],
            lines: &["Our hearth went cold before midnight; the stone is like ice.", "We have burned through the last dry bundle of brushwood.", "Without dry turf, the sheepfold is warmer than the cottage.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(COLD_GATHER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.60, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must chop timber before the dark settles in.", "Keep moving or the cold will numb thy toes. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const DRUNK_SOBER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["A bucket of icy well water over the skull will clear the cobwebs.", "The plough handles won't hold themselves, headache or no.", "Bread and pickled onions, that's what the stomach needs.", "The hours slip by as the sun climbs.", "A good day's toil makes the evening bread sweet."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Walk straight, {other}, or the reeve will notice.", "No more brown ale for thee until Sunday! Fare well.", "May the sun be gentle on thy poor eyes today.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const DRUNK_RECALL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Did I truly challenge the smith to a wrestling match at midnight?", "They say we sang every verse of the Harvest ballad three times over.", "My purse is six silver pennies lighter than when dusk fell.", "The hours slip by as the sun climbs.", "A good day's toil makes the evening bread sweet."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[(Gregarious, 2.0)],
            act: Act::Agree, concepts: &["smalltalk"],
            lines: &["Thou didst indeed! And fell backwards over the ale barrel!", "Aye, and the fiddler laughed so hard his bow went flying.", "It was a merry night, though the morning pays the reckoning.", "Aye, there is truth in what thou sayest.", "Thou hast spoken the plain heart of the matter."],
            then: Some(DRUNK_SOBER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Pious, 2.0)],
            act: Act::PushBack, concepts: &["smalltalk"],
            lines: &["Thou made an ungodly spectacle of thyself.", "A shameful racket; the priest shook his head at the doorway.", "Nay, I see it differently, mark me.", "That is not the way the elders told it.", "Thou'rt jumping to conclusions like a startled hare."],
            then: Some(DRUNK_SOBER), end: None,
        },
    ],
};

const DRUNK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "smalltalk"],
    calls: &["Oof... good morrow, {other}. Don't shout, I pray thee.", "Well met, friend. My head is hammering like the smith's anvil.", "Ugh, the sunlight is blinding today, {other}.", "Good morrow to thee, {other}.", "Well met on this lane, neighbor."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["smalltalk"],
            lines: &["Ha! The tavern-keeper's ale had a bite to it, didn't it?", "Thou lookest like a corpse dug up from the wet turf.", "I warned thee that dark brew was brewed for giants.", "We do what we can with the strength Heaven gives.", "The truth of it is plain for any eye to see."],
            then: Some(DRUNK_RECALL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Sleep it off in the hayloft, {other}.", "Go dunk thy head in the horse trough. Fair day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again.", "God's peace be upon thy cottage, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_AID: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid", "family"],
    calls: &["Blood is thicker than river mud, {other}.", "Whatever befalls the village, our clan stands shoulder to shoulder.", "Our grandfather laid the hearthstones in both our cottages.", "The hours slip by as the sun climbs.", "A good day's toil makes the evening bread sweet."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Always, kin. The family latch is never barred to thee.", "God keep our house and hearth safe from storm.", "See thee at Sunday meal, {other}.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_FAMILY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["family"],
    calls: &["How fares mother's knee in this damp spell?", "Has our cousin returned with the pack horse from the fair?", "Dost thou need an extra hand to dig the potato trench?", "How fares our sister's household down by the green?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["family"],
            lines: &["She sits by the fire with warm sheep's grease; it eases the pain.", "Aye, he brought home two bags of salt and a fine sickle.", "Come by with a spade after the noon bell, brother.", "Our kin will stand beside thee through good season and bad."],
            then: Some(KIN_AID), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const KIN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "family"],
    calls: &["Well met, cousin {other}!", "Good morrow, kin! How fares the old cottage?", "Greetings, brother! It does my heart good to see thy face.", "How fares our sister's household down by the green?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            lines: &["And to thee, kin! The roof still holds.", "Well met, dear cousin. Thou'rt looking hale and hearty.", "God bless thee, kin. What news from thy side of the lane?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(KIN_FAMILY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_PART: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["friendship"],
    calls: &["A good friend is worth more than a pouch of silver.", "We have shared crusts and sweat for twenty years now.", "Thy word has never failed me, {other}.", "Dost thou remember the winter we hauled the great oak?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Nor thine me, old friend. Walk safe.", "We will share a cup of cider when the work is done.", "Fair day, {other} - peace on thy household.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_TRUST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["friendship"],
    calls: &["Dost thou recall the summer the mill dam broke?", "Can I count on thy team when we haul the heavy timber?", "Hast thou heard any whispers about the new taxes?", "Dost thou remember the winter we hauled the great oak?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["friendship"],
            lines: &["Aye, we were waist-deep in mud until midnight saving the sacks!", "My oxen are thine whenever thou needest them, friend.", "The bailiff spoke of three pennies per hide, but we'll contest it.", "Thy word is better than gold; count on my team anytime."],
            then: Some(FRIEND_PART), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FRIEND_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "friendship"],
    calls: &["A hearty good morrow to thee, {other}!", "Well met, true friend! The sun shines brighter when thou'rt near.", "Greetings, old comrade! How goes the day's labor?", "Dost thou remember the winter we hauled the great oak?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["friendship"],
            lines: &["And to thee, {other}! It gladdens my soul to see thee.", "Well met indeed, friend. The chores are lighter for thy smile.", "Good day, partner! We'll weather whatever comes.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(FRIEND_TRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_PASS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["stranger"],
    calls: &["Keep to the beaten track; the marshes to the east are treacherous.", "The alehouse by the bridge offers straw beds if thou hast coin.", "The parish watch keeps guard after the curfew bell.", "What brings thy footsteps into our quiet parish?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["My thanks for the counsel, villager. I pass in peace.", "Safe travels to thee as well. Good day.", "The highway calls; farewell.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_PATH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["stranger"],
    calls: &["Where does this trail lead past the ridge, traveler?", "What town dost thou hail from, stranger?", "Is the road north safe from cutpurses and wolves?", "What brings thy footsteps into our quiet parish?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["stranger"],
            lines: &["It leads down to the cathedral town and the salt wharves.", "I come from the valley beyond the high moors.", "The road is quiet by day, but light a torch if thou walkest at dusk.", "The road to the cathedral town follows the river south."],
            then: Some(STRANGER_PASS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const STRANGER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "stranger"],
    calls: &["Halt there, traveler. What brings thee through our parish?", "Good day, stranger. Thou'rt far from the king's highway.", "Peace be upon thy journey, traveler. Who art thou?", "What brings thy footsteps into our quiet parish?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["stranger"],
            lines: &["Greetings, villager. I mean no harm on these lanes.", "Well met. I am but a traveler seeking the ford across the water.", "Peace to thy fields, friend. I look for the nearest smithy.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(STRANGER_PATH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.50, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["stranger"],
            lines: &["My business is my own, goodman.", "The king's highway is open to every free soul.", "Nay, I see it differently, mark me.", "That is not the way the elders told it.", "Thou'rt jumping to conclusions like a startled hare."],
            then: Some(STRANGER_PASS), end: None,
        },
    ],
};

const SMITH_IRON: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:smith"],
    calls: &["Good Spanish iron rings clear; bad slag thuds like lead.", "A hammer needs an eye that sees the bend before it strikes.", "The quench tub hisses when the blade has its proper temper.", "Is the hearth hot enough to weld the iron tyre?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Keep the fire roaring, smith. Good day.", "May thy hammer never miss its mark. Fair day.", "I will return for the coulter at sunset.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMITH_WORK: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:smith"],
    calls: &["Canst thou forge three dozen horseshoe nails by tomorrow?", "My plough coulter struck a buried boulder and snapped clean.", "Dost thou have charcoal enough to heat the heavy anvil?", "Is the hearth hot enough to weld the iron tyre?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:smith"],
            lines: &["Bring me two groats of coal and the nails will be ready by dawn.", "Lay the broken iron on the bench; I'll weld the seam.", "Aye, the charcoal burner delivered two sacks from the pit yesterday.", "The white heat is ready; bring the iron to the anvil."],
            then: Some(SMITH_IRON), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMITH_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:smith"],
    calls: &["Good morrow, smith! The sparks are flying bright today.", "Well met, master smith! The sound of thy forge wakes the village.", "Hail to the forge, {other}! Sweating by the bellows already?", "Is the hearth hot enough to weld the iron tyre?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:smith"],
            lines: &["Morrow, {other}! The coals are white-hot and hungry for iron.", "Well met! Step inside out of the wind, mind the sparks.", "Aye, iron waits for no man's leisure. What needest thou?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(SMITH_WORK), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_TOLL: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:miller"],
    calls: &["One scoop in twenty is the customary multure, no more, no less.", "The millstones were freshly dressed with the chisel last Pentecost.", "Fine white flour for white loaves, coarse bran for the byre.", "Will the mill race freeze if the frost sharpens tonight?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Keep the wheel turning, miller. Fair day.", "Mind thou dost not spill my meal on the floorboards.", "Until harvest time, master of the water.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_GRIST: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:miller"],
    calls: &["Is the river running high enough to turn the big wheel?", "Canst thou grind my two sacks of barley before noon?", "Did the damp get into the meal hopper after the downpour?", "Will the mill race freeze if the frost sharpens tonight?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:miller"],
            lines: &["The millpond is full to the weir boards; she turns swift today.", "Stack thy sacks by the scale; thy meal will be ground by mid-day.", "The oak bins are dry as bone; not a drop touched the flour.", "The river turns the wheel fast; the meal pours sweet."],
            then: Some(MILLER_TOLL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MILLER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:miller"],
    calls: &["Good morrow, miller! The roar of thy weir echoes down the valley.", "Well met, {other}! Flour dusts thy coat like fresh snow.", "Hail to the mill, {other}! Is the grist running smooth?", "Will the mill race freeze if the frost sharpens tonight?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:miller"],
            lines: &["Morrow, neighbor! The stones rumble and the water runs free.", "Well met! Unload thy cart; the hopper is ready.", "Aye, dust in my hair and bread in my belly! What grist bringest thou?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(MILLER_GRIST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_CRUST: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:baker"],
    calls: &["A well-swept oven floor gives the sweetest crust in the parish.", "Yeast from the brewer's ale makes the dough rise twice as high.", "Golden loaves on the wooden board - the glory of the harvest.", "Are the loaves golden brown on the oven bricks?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["The smell of thy loaves is a blessing to the lane. Fare well.", "May thy oven never grow cold, baker.", "I take my warm crust home to the children. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_BREAD: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:baker"],
    calls: &["Are the morning rye loaves ready to pull with the peel?", "Canst thou bake my barley dough in thy community oven?", "Hast thou any day-old trenchers left for a penny?", "Are the loaves golden brown on the oven bricks?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:baker"],
            lines: &["Aye, hot from the bricks and brown as a hazelnut!", "Bring thy dough before noon; there is room beside the landlord's pies.", "Take four trenchers; they are crisp and good for soaking broth.", "The crust is crisp and hot from the stone floor."],
            then: Some(BAKER_CRUST), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BAKER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:baker"],
    calls: &["Good morrow, baker! Thy chimney breathes the sweetest air in town.", "Well met, {other}! The smell of fresh crust drew me from the lane.", "Hail, master of the peel! Has the first batch risen?", "Are the loaves golden brown on the oven bricks?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:baker"],
            lines: &["Morrow, {other}! Flour on my hands and fire in the hearth.", "Well met! The oven is blazing hot and the loaves are baking.", "A warm welcome to thee! Step in and feel the heat.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(BAKER_BREAD), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_CLOTH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:weaver"],
    calls: &["Tight warp and even weft make a cloth that sheds the rain.", "Madder root gives a brave red, but woad blues last thirty years.", "A warm woolen blanket keeps the winter chills from the cradle.", "How many yards of wool cloth come off the loom today?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy shuttle fly true and never break a thread.", "I will return when the piece comes off the frame. Fare well.", "Good cloth is true wealth. Good day, weaver.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_FLEECE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:weaver"],
    calls: &["Did the sheep shearers bring in the upland wool yet?", "Canst thou weave a yard of heavy russet for my work kirtle?", "Is the spun yarn dry enough to wind onto the spools?", "How many yards of wool cloth come off the loom today?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:weaver"],
            lines: &["Ten sacks of greasy fleece sit by the washing trough.", "I'll set the warp tomorrow; russet is sturdy and warm for field work.", "Aye, the carders combed it fine and the spindles are humming.", "The shuttle flies true and the cloth grows stout."],
            then: Some(WEAVER_CLOTH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const WEAVER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:weaver"],
    calls: &["Good morrow, weaver! The click-clack of thy loom is steady today.", "Well met, {other}! Surrounded by skeins of dyed wool, I see.", "Hail to the loom, neighbor! How runs the shuttle this morning?", "How many yards of wool cloth come off the loom today?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:weaver"],
            lines: &["Morrow, {other}! Counting the threads from dawn till twilight.", "Well met! The treadles are flying and the cloth grows fast.", "Greetings! A weaver's work is never done while sheep have wool.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(WEAVER_FLEECE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_PEGS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:carpenter"],
    calls: &["Heart of oak and seasoned ash will outlast three generations.", "An auger hole drilled straight needs only a split hazel peg to hold.", "Cedar smells sweet in a chest, but elm makes the stoutest wheel hubs.", "Canst thou hew a sturdy axle-tree for my cart?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy adze stay sharp, carpenter. Fare well.", "I will look for the cart axle on Friday. Good day.", "Stout timber makes a sturdy home. Walk in peace.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_BEAMS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:carpenter"],
    calls: &["Canst thou mortise a tie-beam for the cottage roof ridge?", "Didst thou square the timber logs felled in the lord's wood?", "Dost thou have spare dry pegs to secure the wattle hurdle?", "Canst thou hew a sturdy axle-tree for my cart?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:carpenter"],
            lines: &["I have seasoned oak ready on the trestles; it will bear the rafters.", "The broadaxe squared two dozen beams; they are stacked to cure.", "Take a handful of dried oak pins from the tub by the bench.", "The oak timber is straight-grained and cured dry."],
            then: Some(CARPENTER_PEGS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CARPENTER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:carpenter"],
    calls: &["Good morrow, carpenter! Shavings and sawdust fly thick today.", "Well met, {other}! The scent of fresh-cut pine is in the lane.", "Hail to the woodwright! Has the new timber arrived from the coppice?", "Canst thou hew a sturdy axle-tree for my cart?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:carpenter"],
            lines: &["Morrow, neighbor! Measuring twice and cutting once, as ever.", "Well met! The saw pit was busy since the dawn bell.", "Greetings! A stout tree fell yesterday; fine timber for building.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(CARPENTER_BEAMS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_LIME: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:mason"],
    calls: &["Hot slaked lime and sharp sand bond tighter than the quarry itself.", "A drystone foundation keeps the rot out of the lowest timbers.", "Granite breaks the chisel, but limestone yields to an honest stroke.", "Will the limestone mortar hold against the river damp?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy plumb-line hang true, mason. Fair day.", "Stone stands when all else fades. Farewell.", "I will bring the ox-cart for the rubble tomorrow.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_WALL: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:mason"],
    calls: &["Canst thou dress ashlar stones for the new church window?", "Did the quarry deliver enough rubble stone to rebuild the wall?", "Will the lime mortar set before the autumn rains begin?", "Will the limestone mortar hold against the river damp?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:mason"],
            lines: &["I am chiseling the jambs now; fine freestone from the quarry.", "Two heavy wagonloads came down the hill this morning.", "It will cure rock-hard if we keep the frost off it for a fortnight.", "The lime mortar sets hard as flint between the stones."],
            then: Some(MASON_LIME), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MASON_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:mason"],
    calls: &["Good morrow, mason! The ringing of thy hammer on stone echoes far.", "Well met, {other}! White rock dust on thy leather apron.", "Hail to the stone-cutter! Setting foundations today?", "Will the limestone mortar hold against the river damp?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:mason"],
            lines: &["Morrow, {other}! Stone by stone the hall rises from the earth.", "Well met! Testing the level and squaring the corners.", "Greetings! A heavy trade, but a wall built right outlasts empires.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(MASON_WALL), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_PASTURE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:shepherd"],
    calls: &["The upland turf is sweet and short; the ewes grow fat on it.", "The sheepfold dung will enrich the barley fields when autumn comes.", "A good dog is worth three hired hands on the open hills.", "Did the herd dog drive the stray lambs from the briars?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Keep thy eye on the brushwood, shepherd. Fair day.", "May the flock prosper and the wolves stay distant.", "Good grazing to thy flock, neighbor.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_EWES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:shepherd"],
    calls: &["How many twin lambs dropped in the fold last night?", "Did the wolf from the high ridge trouble the sheepfold?", "Is it time to drive the flock down to the river wash?", "Did the herd dog drive the stray lambs from the briars?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:shepherd"],
            lines: &["Six fine lambs, all suckling strong by their mothers.", "The hound barked at midnight and drove the beast away.", "By next full moon the fleece will be clean for the shearers.", "The ewes are content and the lambs suckle strong."],
            then: Some(SHEPHERD_PASTURE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SHEPHERD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:shepherd"],
    calls: &["Good morrow, shepherd! Bleating ewes fill the morning air.", "Well met, {other}! Cloaked in wool and leaning on thy crook.", "Hail, drover of the flock! How fare the sheep on the commons?", "Did the herd dog drive the stray lambs from the briars?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:shepherd"],
            lines: &["Morrow, neighbor! Watching for hawks and counting tails.", "Well met! The bells on the wethers make a pleasant music.", "Greetings! Wind on the hill, but the ewes are peaceful.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(SHEPHERD_EWES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_CUP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:brewer"],
    calls: &["Clear spring water and sound malt make an ale fit for a baron.", "An ale-stake hung with green boughs means a fresh barrel is tapped.", "A tankard at sundown washes the harvest dust from the throat.", "Has the barley mash cooled enough to pitch the yeast?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy vats never sour, brewer. Fair day.", "I will bring my pewter mug at sundown. Farewell.", "A good brew makes neighbors into brothers. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_MASH: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:brewer"],
    calls: &["Is the new brewing of October ale ready for the tap?", "Did the maltsters bring in the sprouted barley from the floor?", "Dost thou have small beer to sell for the field workers' pails?", "Has the barley mash cooled enough to pitch the yeast?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:brewer"],
            lines: &["It has fermented sweet and strong; we broach the cask today.", "Three carts of roasted malt came down from the kiln yesterday.", "Aye, fill thy pails; small beer keeps the reapers from fainting.", "The ale has cleared and the barrel is broached sweet."],
            then: Some(BREWER_CUP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BREWER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:brewer"],
    calls: &["Good morrow, brewer! Warm steam rises over thy brew-house.", "Well met, {other}! The rich scent of mash drifts through the lane.", "Hail to the tapster! Tapping a fresh barrel this morning?", "Has the barley mash cooled enough to pitch the yeast?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:brewer"],
            lines: &["Morrow, {other}! Stirring the copper vat since before cockcrow.", "Well met! The yeast is working its miracle in the vats.", "Greetings! A thirsty throat finds a ready welcome here.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(BREWER_MASH), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_SWARM: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:beekeeper"],
    calls: &["Sweet yellow wax makes the church tapers that burn without smoke.", "The clover blossoms give the clearest honey in the valley.", "Bees love a quiet tongue and a gentle step among the skeps.", "Are the worker bees flying strong to the heather hills?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy hives thrive through the winter, beekeeper. Fair day.", "I will buy a jar of comb at Michaelmas. Farewell.", "The little workers teach us all to labor together. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_HONEY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:beekeeper"],
    calls: &["Did the swarm from the orchard settle in the new straw skep?", "Can I buy a pot of honey to soothe my child's raw throat?", "Hast thou cakes of beeswax ready for the shoemaker's thread?", "Are the worker bees flying strong to the heather hills?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:beekeeper"],
            lines: &["Aye, we beat the brass pan and hived them safely under the hedge.", "Take this jar of dark thyme honey; it mends a cough overnight.", "Three cakes of yellow wax, filtered clean and smelling of clover.", "The skeps are heavy with comb and clear summer honey."],
            then: Some(BEEKEEPER_SWARM), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const BEEKEEPER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:beekeeper"],
    calls: &["Good morrow, beekeeper! The apiary hums like an organ in the sunshine.", "Well met, {other}! Straw skeps standing tidy in the garden rows.", "Hail, master of the hives! Smoking the bees this morning?", "Are the worker bees flying strong to the heather hills?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:beekeeper"],
            lines: &["Morrow, {other}! The bees are flying thick to the heather hills.", "Well met! Sweet honey on the comb and wax for the monastery.", "Greetings! Walk soft, neighbor; the queen is in her prime.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(BEEKEEPER_HONEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_SALVE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:herb_wife"],
    calls: &["Dried elder flowers and mint leaves keep the ague away in winter.", "Yarrow leaves stop a blood-cut faster than cobwebs.", "The roots must be gathered in the dark of the moon for full virtue.", "What root will soothe a racking cough in the chest?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thy knowledge has saved many a soul in this town. Fare well.", "I will brew the tea as thou biddest. Good day.", "May the earth yield its medicine to thee. Farewell.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_DRAUGHT: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:herb_wife"],
    calls: &["Hast thou a salve for stiff knees and swollen knuckles?", "Canst thou brew a draught for the fever that shakes the child?", "What herb will keep the pests away from our flour bin?", "What root will soothe a racking cough in the chest?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:herb_wife"],
            lines: &["Rub this grease rendered with comfrey and marigold twice a day.", "Take this dried willow bark; steep it in hot water and give three sips.", "Hang dried tansy and wormwood above the bin; no weevil will touch it.", "Boil the willow bark in rain water and drink it warm."],
            then: Some(HERB_SALVE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const HERB_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:herb_wife"],
    calls: &["Good morrow, wise woman {other}! Bunches of herbs hang drying everywhere.", "Well met, herb-wife! Thy basket is full of green leaves from the fen.", "Hail, healer of the sick! Gathering roots in the dew?", "What root will soothe a racking cough in the chest?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:herb_wife"],
            lines: &["Morrow, {other}! The earth gives her secrets to those who look.", "Well met! Stinging nettle and plantain leaves gathered at sunrise.", "Greetings, neighbor! What ails thee or thy folk today?", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(HERB_DRAUGHT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_BARTER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:merchant"],
    calls: &["A good trade leaves both men smiling when hands are struck.", "Silver coin or salted hams - sound goods are always welcome.", "The road is long, but fair trade makes every mile worthwhile.", "What exchange dost thou ask for a bag of sea salt?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Fair journey on the highway, trader. Good day.", "May thy pack horses stay sure-footed on the ridge.", "Until the spring fair, merchant. Fare well.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_WARES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:merchant"],
    calls: &["Dost thou carry steel sewing needles and linen thread from the city?", "What price dost thou ask for a pound of sea salt?", "Hast thou any ribbons of silk or spice from overseas?", "What exchange dost thou ask for a bag of sea salt?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:merchant"],
            lines: &["Fine iron needles in a bone case; two eggs or a penny each.", "White sea salt from the coast; three groats for a full measure.", "Scarlet ribbon from Flanders and a pinch of black pepper to trade.", "Two silver pennies or a salted ham buys the measure."],
            then: Some(MERCHANT_BARTER), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const MERCHANT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:merchant"],
    calls: &["Good morrow, trader! Thy pack mules are laden heavy from the highway.", "Well met, merchant! What exotic wares bringest thou to our valley?", "Hail to the traveling merchant! How stood the roads through the forest?", "What exchange dost thou ask for a bag of sea salt?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:merchant"],
            lines: &["Morrow, good folk! Goods from the sea ports and goods from the hills.", "Well met! Unbuckling the leather packs; step up and see the wares.", "Greetings! Mud on my boots, but treasures in my packs for honest barter.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(MERCHANT_WARES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_PETITION: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:rents"],
    calls: &["The manor must maintain the bridge if the peasants are to bring in the harvest.", "A good lord shears his sheep but does not flay the skin.", "The court rolls show every customary right since the grandfather's time.", "Will the lord's bailiff hear our plea concerning the hail damage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["We will see what the manor court decides on Tuesday. Good day.", "May justice be remembered at the high table. Fare well.", "I take my leave of the manor gate. Fair day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_LEASES: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:rents"],
    calls: &["Will the rent in silver be abated because the hail ruined the oats?", "Must our cottage provide three days of boon-ploughing this season?", "When will the lord's bailiff hear our plea concerning the common ditch?", "Will the lord's bailiff hear our plea concerning the hail damage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["concern:rents"],
            lines: &["The steward will inspect the damaged fields before the tally is set.", "Two days of plough-service are due by custom, as the parchment records.", "The court sits under the great oak on Thursday morning; be present.", "The manor court must recognize the customary rights of our lane."],
            then: Some(LORD_PETITION), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const LORD_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:rents"],
    calls: &["Good morrow to the manor's steward. We come on village business.", "Well met, sir. We seek speech concerning the leases and tallies.", "Greetings, lord's man. The tenants have gathered to speak.", "Will the lord's bailiff hear our plea concerning the hail damage?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["concern:rents"],
            lines: &["Morrow, tenants. The tally sticks are on the table; state thy grievance.", "Well met, villagers. The lord listens to lawful petitions.", "Speak thy mind plainly, goodmen; the court roll is open.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(LORD_LEASES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_NETS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:fisher"],
    calls: &["Trout leap in the shallows when the evening mayflies hatch.", "Wicker eel pots set in the weir reeds yield a fat catch by dawn.", "A fisherman knows every deep hole and eddy between here and the bridge.", "Did the willow fish-pots take any eels at the weir?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy nets come up heavy with silver fish. Fair day.", "Mind the river current by the mill dam. Farewell.", "I will bring an eel pie to the festival. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_CURRENT: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:fisher"],
    calls: &["Is the river water clear or muddy after the hill rains?", "Did the fish trap take any pike in the reed bed today?", "Can I buy three fresh trout for the Friday supper?", "Did the willow fish-pots take any eels at the weir?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:fisher"],
            lines: &["The current runs green and sweet; the fish bite eager.", "A stout pike and two dozen silver dace lie in the willow creel.", "Take these three trout fresh from the river; three pennies or a loaf.", "Three fat river trout and an eel fresh from the wicker trap."],
            then: Some(FISHER_NETS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FISHER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:fisher"],
    calls: &["Good morrow, riverman! Thy boots are wet to the knee.", "Well met, {other}! Mending thy linen gill-nets on the bank?", "Hail, fisher of the weir! Any fish in the basket today?", "Did the willow fish-pots take any eels at the weir?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:fisher"],
            lines: &["Morrow, {other}! The river is awake and cold as ice.", "Well met! Tying knots and checking the willow osiers.", "Greetings! River water in my veins and fresh catch in the creel.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(FISHER_CURRENT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_COALS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["trade:charcoal"],
    calls: &["Slow heat with no open flame turns dense oak into pure black coal.", "Black dust in the beard and lungs, but the smith's forge cannot live without us.", "A charcoal stack needs watching through three nights of vigil.", "How burns the earth-covered pit mound in the clearing?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May thy stacks burn true without flaring. Fair day.", "I will tell the smith his coals are ready. Farewell.", "Watch the sparks in the dry brushwood, collier. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_SMOKE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["trade:charcoal"],
    calls: &["Is the earth-covered mound ready to open and rake?", "Canst thou spare a sack of charcoal for the tavern braziers?", "Did the night wind blow the turf cover off the kiln?", "How burns the earth-covered pit mound in the clearing?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["trade:charcoal"],
            lines: &["The blue smoke has turned clear; the coal is cooled and cured.", "Two sacks of hard black charcoal wait by the pit hut.", "The sods held fast; not a spark escaped into the trees.", "The mound is raked and two sacks of hard coal are cooled."],
            then: Some(CHARCOAL_COALS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CHARCOAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "trade:charcoal"],
    calls: &["Good morrow, charcoal burner! Smoke drifts thick from the deep wood.", "Well met, {other}! Black as an imp from tending the pits.", "Hail, master of the kiln! How burns the timber mound?", "How burns the earth-covered pit mound in the clearing?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["trade:charcoal"],
            lines: &["Morrow, {other}! Watching the slow smoke day and night.", "Well met! Soot on my face, but clean fuel for the iron forge.", "Greetings! The mound is smoking sweet under its turf blanket.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(CHARCOAL_SMOKE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_BLESS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["elder"],
    calls: &["Lords come and go, but the plough and the furrow remain.", "Listen to the land; it was here before us and will keep our graves.", "Kindness to thy neighbor is the best hedge against misfortune.", "Dost thou recall how the great freeze broke forty years ago?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thy words are gold to my ears, elder. Fair day.", "May God grant thee peace in thy old age. Farewell.", "We will heed thy counsel at the village meeting. Good day.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_WINTERS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["elder"],
    calls: &["Dost thou recall the great famine fifty winters ago, grandfather?", "Where did the old village boundary run before the ditch was dug?", "What signs tell us when the frost will finally break?", "Dost thou recall how the great freeze broke forty years ago?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["elder"],
            lines: &["Aye, we ate wild acorns and shared what porridge there was, and survived.", "The boundary ran straight from the standing oak to the weeping willow.", "When the dandelions flower on the ditch bank, the frost is broken for good.", "Patience and honest toil have seen eighty winters come and go."],
            then: Some(ELDER_BLESS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const ELDER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "elder"],
    calls: &["Good morrow to thee, venerable {other}.", "Well met, grandfather! Sitting in the sunny doorway to warm thy joints?", "Greetings, mother of the village! How fares thy health today?", "Dost thou recall how the great freeze broke forty years ago?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["elder"],
            lines: &["Morrow, young soul. The sun feels good on eighty winters of bones.", "Well met, child! Another day Heaven grants to watch the fields.", "Greetings! My step is slow, but my memory is long and clear.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(ELDER_WINTERS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_GIFT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["family"],
    calls: &["May the little one grow tall as an ash and hardy as an oak.", "A baby's cry in the cottage banishes the winter gloom.", "The parish grows by another soul; blessed be the hearth.", "How fares our sister's household down by the green?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["May the saints guard the cradle day and night. Fair day.", "I will bring goat's milk for the mother. Farewell.", "Joy to thy house, {other}! God bless the babe.", "May the road be kind under thy boots.", "Fare thee well until our paths cross again."],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_JOY: Exchange = Exchange {
    act: Act::Question,
    concepts: &["family"],
    calls: &["Is the little one sleeping peacefully in the wicker cradle?", "Has the parish priest set the day for the holy baptism?", "Does the mother have warm elderberry cordial to restore her strength?", "How fares our sister's household down by the green?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Answer, concepts: &["family"],
            lines: &["Aye, wrapped in soft lamb's wool, dreaming sweet dreams.", "On Sunday morning when the great church bell tolls.", "The neighbors brought honey broth and fresh eggs for her recovery.", "Our kin will stand beside thee through good season and bad."],
            then: Some(NEWBORN_GIFT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

const NEWBORN_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "family"],
    calls: &["Joy to thy household, {other}! I heard a newborn babe has arrived!", "Good morrow, new mother! How fares the precious child?", "Blessings on the cradle, {other}! What a joyous day for our lane!", "How fares our sister's household down by the green?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.00, bias: &[],
            act: Act::Greet, concepts: &["family"],
            lines: &["Morrow, {other}! A healthy babe, praise be to Heaven!", "Well met! Exhausted, but my heart is overflowing with gratitude.", "Greetings! A little girl with eyes bright as the morning sky.", "God bless thy steps this day, {other}.", "Well met, friend, under the open sky."],
            then: Some(NEWBORN_JOY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.40, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I must be on my way, {other}. Fare well.", "The sun climbs high; chores call me. Good day.", "We will speak more of this later. God keep thee.", "Until next time, neighbor."],
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
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
        lines: &["We buried two souls this season; the churchyard grows crowded."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
        lines: &["The sorrow in our cottages weighs heavier than the taxes."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
        lines: &["A grieving house should be supported with food and wood by neighbors."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
        lines: &["Let the parish pray together for those who have gone before us."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
        lines: &["The tolling bell reminds every family that life is fragile."],
    },
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief"], bias: &[],
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
        act: Act::Agree, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons"], bias: &[],
        lines: &["Aye, there is the truth of it.", "{other} has the right of it.", "It is so - my house has felt the same.", "Well spoken, neighbor; we stand with thee."],
    },
    MeetingLine {
        act: Act::PushBack, concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest", "concern:fuel", "concern:bridge", "concern:commons"], bias: &[],
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
    pool.extend_from_slice(MEET_HARVEST);
    pool.extend_from_slice(MEET_COMMONS);
    pool.extend_from_slice(MEET_BRIDGE);
    pool.extend_from_slice(MEET_COMMON);
    pool
}

/// A worked-example agenda for a house meeting
pub const MEET_PROMPT: MeetingPrompt = MeetingPrompt {
    text: "What ails the village most, friends? Speak true.",
    concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:fuel", "concern:bridge", "concern:harvest", "concern:commons"],
};

/// The full dialogue library: 30 circumstance and vocation trees plus smalltalk tail
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
        DialogueTree { name: "hunger_lean_times", criteria: &[Has("any:status:hungry")],
            root: &HUNGER_GREET, rejoin: Some(&HUNGER_SHARE) },
        DialogueTree { name: "cold_hearth_fuel", criteria: &[Has("any:status:cold")],
            root: &COLD_GREET, rejoin: Some(&COLD_GATHER) },
        DialogueTree { name: "morning_after_drunk", criteria: &[Has("any:status:drunk")],
            root: &DRUNK_GREET, rejoin: Some(&DRUNK_RECALL) },
        DialogueTree { name: "kin_warmth", criteria: &[Has("pair:relation:kin")],
            root: &KIN_GREET, rejoin: Some(&KIN_FAMILY) },
        DialogueTree { name: "friendship_bonds", criteria: &[Has("pair:relation:friend")],
            root: &FRIEND_GREET, rejoin: Some(&FRIEND_TRUST) },
        DialogueTree { name: "stranger_road", criteria: &[Has("pair:relation:stranger")],
            root: &STRANGER_GREET, rejoin: Some(&STRANGER_PATH) },
        DialogueTree { name: "smith_forge", criteria: &[Has("any:status:smith")],
            root: &SMITH_GREET, rejoin: Some(&SMITH_WORK) },
        DialogueTree { name: "miller_grain", criteria: &[Has("any:status:miller")],
            root: &MILLER_GREET, rejoin: Some(&MILLER_GRIST) },
        DialogueTree { name: "baker_hearth", criteria: &[Has("any:status:baker")],
            root: &BAKER_GREET, rejoin: Some(&BAKER_BREAD) },
        DialogueTree { name: "weaver_wool", criteria: &[Has("any:status:weaver")],
            root: &WEAVER_GREET, rejoin: Some(&WEAVER_FLEECE) },
        DialogueTree { name: "carpenter_wood", criteria: &[Has("any:status:carpenter")],
            root: &CARPENTER_GREET, rejoin: Some(&CARPENTER_BEAMS) },
        DialogueTree { name: "mason_stone", criteria: &[Has("any:status:mason")],
            root: &MASON_GREET, rejoin: Some(&MASON_WALL) },
        DialogueTree { name: "shepherd_fold", criteria: &[Has("any:status:shepherd")],
            root: &SHEPHERD_GREET, rejoin: Some(&SHEPHERD_EWES) },
        DialogueTree { name: "brewer_ale", criteria: &[Has("any:status:brewer")],
            root: &BREWER_GREET, rejoin: Some(&BREWER_MASH) },
        DialogueTree { name: "beekeeper_hive", criteria: &[Has("any:status:beekeeper")],
            root: &BEEKEEPER_GREET, rejoin: Some(&BEEKEEPER_HONEY) },
        DialogueTree { name: "herb_apothecary", criteria: &[Has("any:status:herb_wife")],
            root: &HERB_GREET, rejoin: Some(&HERB_DRAUGHT) },
        DialogueTree { name: "merchant_trade", criteria: &[Has("any:ideology:merchant")],
            root: &MERCHANT_GREET, rejoin: Some(&MERCHANT_WARES) },
        DialogueTree { name: "lord_manor", criteria: &[Has("any:ideology:lord")],
            root: &LORD_GREET, rejoin: Some(&LORD_LEASES) },
        DialogueTree { name: "fisher_river", criteria: &[Has("any:status:fisher")],
            root: &FISHER_GREET, rejoin: Some(&FISHER_CURRENT) },
        DialogueTree { name: "charcoal_pit", criteria: &[Has("any:status:charcoal_burner")],
            root: &CHARCOAL_GREET, rejoin: Some(&CHARCOAL_SMOKE) },
        DialogueTree { name: "elder_wisdom", criteria: &[Has("any:status:elder")],
            root: &ELDER_GREET, rejoin: Some(&ELDER_WINTERS) },
        DialogueTree { name: "newborn_cradle", criteria: &[Has("any:status:newborn")],
            root: &NEWBORN_GREET, rejoin: Some(&NEWBORN_JOY) },
        DialogueTree { name: "village_smalltalk", criteria: &[],
            root: &SMALL_GREET, rejoin: Some(&SMALL_CHAT) },
    ]
}

/// A one-line backstory from a speaker's profile
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
