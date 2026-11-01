//! The worked-example dialogue library: a handful of trees that show the
//! framework's pieces - a generic small-talk tail that always matches,
//! and circumstance trees (sickness, grief, a cleric, a rivalry) that
//! outrank it when their facts hold.
//!
//! Authoring rule of thumb: the initiator speaks every `calls`, so write
//! calls in the lead's voice; the answerer speaks `lines`, and which fork
//! they take decides the next beat. A reply's criteria are how
//! circumstance *adds* a path - the sick get a "pray for me" fork a well
//! person never sees.

use crate::game::conversation::{
    Act, Criterion::*, DialogueTree, Ending::*, Exchange, MeetingLine, MeetingPrompt,
    Reply, Speaker,
};
use crate::game::traits::Trait;
use crate::game::traits::Trait::*;

// =====================================================================
// The generic tail: village small talk. No criteria - it always matches
// and always sorts last, the doldrums every specific tree drains into.
// `rejoin` lets a conversation that fell through to it skip re-greeting.
// =====================================================================

const SMALL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting"],
    calls: &[
        "Good morrow to thee, {other}.",
        "Well met, {other}.",
        "A fair day to thee, {other}.",
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Greet, concepts: &["greeting"],
            lines: &["And to thee, {other}.", "Well met indeed.", "The saints keep thee, {other}."],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[], weight: 0.35, bias: &[(Hotheaded, 3.0), (Proud, 2.0)],
            act: Act::Remark, concepts: &["greeting"],
            lines: &["Morrow.", "Aye. What is it?"],
            then: Some(SMALL_CHAT), end: None,
        },
        Reply {
            criteria: &[Lacks("pair:relation:friend"), Lacks("pair:relation:kin")],
            weight: 0.2, bias: &[(Proud, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I've no time for idle talk.", "Work's waiting, {other}."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SMALL_CHAT: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk", "weather"],
    calls: &[
        "Looks like rain again, doesn't it?",
        "Sky's black over the moor - rain by noon, I'd say.",
        "Smell that? There's rain coming.",
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["weather"],
            lines: &["It'll be a wet week, mark me.", "The thatch won't thank it.", "Aye, and the mud'll take thy boots."],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.4, bias: &[(Hotheaded, 2.0)],
            act: Act::PushBack, concepts: &["weather"],
            lines: &["Rain? It's the driest year in memory.", "Thou seest storms in a clear sky."],
            then: Some(SMALL_VOLLEY), end: None,
        },
        Reply {
            criteria: &[], weight: 0.7, bias: &[(Gregarious, 2.0)],
            act: Act::Question, concepts: &["family", "smalltalk"],
            lines: &["And how fares thy folk?", "Tell me true - how goes it with thee?"],
            then: Some(SMALL_FARES), end: None,
        },
        Reply {
            criteria: &[], weight: 0.35, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Well, I'd best be about my day.", "I should get on. Fair day to thee."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_VOLLEY: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["Even so, we'll manage.", "Aye, the year'll turn."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["smalltalk"],
            lines: &["That it will - the store's not empty yet.", "Aye, we've seen worse."],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.7, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll not keep thee. Go well.", "On thy way then, and mine."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_FARES: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["family", "smalltalk"],
    calls: &["Well enough, thank thee.", "We rub along, as ever.", "The little ones keep me busy."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["family"],
            lines: &["Good to hear it.", "Long may it last."],
            then: Some(SMALL_GOSSIP), end: None,
        },
        Reply {
            criteria: &[], weight: 0.6, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Give them my best. Fare well."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GOSSIP: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip", "smalltalk"],
    calls: &[
        "Didst thou hear who was seen down by the mill?",
        "They say the miller's a knave and no mistake.",
        "Folk saw the reeve's cart loaded past dark.",
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[(Gregarious, 2.0)],
            act: Act::Remark, concepts: &["gossip"],
            lines: &["Thou dost not say it! Go on.", "Aye? And what else?"],
            then: Some(SMALL_GOSSIP_MORE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[(Proud, 2.0)],
            act: Act::PushBack, concepts: &["gossip"],
            lines: &["Idle tongues, that.", "I'll not repeat market talk."],
            then: Some(SMALL_FADE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.4, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["A tale for another day - I must away."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SMALL_GOSSIP_MORE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["gossip"],
    calls: &["They do say the reeve counts twice.", "And the herb-wife was seen with the tinker, so it's told."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["gossip"],
            lines: &["Well, that's talk enough for one day.", "Ha! I'll believe the half of it."],
            then: Some(SMALL_FADE), end: None,
        },
    ],
};

const SMALL_FADE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["smalltalk"],
    calls: &["As thou sayest.", "True enough - the fields don't gossip."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Fair day to thee, {other}.", "Go well, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

// =====================================================================
// The sickness: the sick steer their own talk; the well who call on them
// steer a different tree. Replies the well make recoil (PushBack) or
// sympathise by temperament.
// =====================================================================

const SICK_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &[
        "*cough* Good morrow, {other}.",
        "Forgive the cough - it won't quit.",
        "Morrow, {other}. Don't stand too close.",
    ],
    replies: &[
        Reply {
            criteria: &[], weight: 1.2, bias: &[(Kind, 2.0)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["Poor thing - get thee to bed.", "Thou shouldst be by the fire, not out here."],
            then: Some(SICK_EXCUSE), end: None,
        },
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Question, concepts: &["concern:sickness"],
            lines: &["Is it the sickness going round?", "Half the village is coughing - is it that?"],
            then: Some(SICK_ANSWER), end: None,
        },
        Reply {
            criteria: &[Lacks("self:status:sick")], weight: 0.5, bias: &[(Proud, 1.5), (Stingy, 1.5)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            lines: &["Keep thy distance, friend.", "I'll hear thee from here."],
            then: Some(SICK_STUNG), end: None,
        },
    ],
};

const SICK_EXCUSE: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["I'd rest if the work'd wait.", "There's no one else to fetch the water."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["We rub along or we don't eat, that's the truth of it."],
            then: None, end: None, // bottoms out -> falls through to small talk
        },
        Reply {
            criteria: &[], weight: 0.7, bias: &[(Kind, 2.0)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            lines: &["I'll send soup round this evening."],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Rest well, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_ANSWER: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:sickness"],
    calls: &["Aye, since the last rain. The fever comes and goes.", "They say the herb-wife's draught helps."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["The saints speed thy mending.", "May it pass thee lightly."],
            then: None, end: None, // drifts to small talk
        },
        Reply {
            criteria: &[], weight: 0.8, bias: &[(Kind, 2.0), (Pious, 1.3)],
            act: Act::Remark, concepts: &["concern:sickness", "aid"],
            lines: &["I'll fetch the herb-wife round.", "Say a prayer and I'll bring broth."],
            then: Some(SICK_THANKS), end: None,
        },
        Reply {
            criteria: &[], weight: 0.4, bias: &[(Stingy, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll be off before it takes me too."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const SICK_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness", "aid"],
    calls: &["Thou'rt kind. The saints remember it.", "My thanks, {other}."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Think nothing of it.", "We look after our own."],
            then: None, end: Some(Farewell),
        },
    ],
};

const SICK_STUNG: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:sickness"],
    calls: &["It's not the plague, keep thy wig on.", "As thou wilt, then."],
    replies: &[
        Reply {
            criteria: &[], weight: 0.8, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:sickness"],
            lines: &["Forgive me - fear makes cowards of us all."],
            then: Some(SICK_ANSWER), end: None, // the talk recovers
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["concern:sickness"],
            lines: &["Cough on someone else, then."],
            then: None, end: Some(WalksOff),
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Another time, then."],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:sickness"],
    calls: &["How dost thou fare, {other}?", "I heard the sickness took thee - how is it?"],
    replies: VISIT_ANSWERS,
};

const VISIT_ANSWERS: &[Reply] = &[
    Reply {
        criteria: &[], weight: 1.0, bias: &[],
        act: Act::Answer, concepts: &["concern:sickness"],
        lines: &["Poorly, if thou'lt have the truth.", "Weak as a kitten, but here I am."],
        then: Some(VISIT_CARE), end: None,
    },
    Reply {
        criteria: &[], weight: 0.6, bias: &[(Proud, 2.0)],
        act: Act::PushBack, concepts: &["concern:sickness"],
        lines: &["It'll pass. Don't fuss."],
        then: Some(VISIT_CARE), end: None,
    },
];

// Joined mid-talk: already greeted, so straight to the sickness
const VISIT_REJOIN: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:sickness"],
    calls: &["But thy sickness, {other} - how is it, truly?", "And the cough - is it any better?"],
    replies: VISIT_ANSWERS,
};

const VISIT_CARE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:sickness", "aid"],
    calls: &["Is there aught thou needest?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Answer, concepts: &["aid", "concern:food"],
            lines: &["A little bread, if thou canst spare it."],
            then: Some(VISIT_PROMISE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[(Pious, 1.5)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Just a prayer, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const VISIT_PROMISE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["aid"],
    calls: &["It's done - look for it by dark."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thou'rt a true neighbor."],
            then: None, end: Some(Farewell),
        },
    ],
};

// =====================================================================
// Grief: the mourner's own tree when they lead, the caller's when they do.
// =====================================================================

const GRIEF_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &[
        "{other}. Forgive me - the house is quiet since he passed.",
        "Morrow, {other}. It is a hard week.",
    ],
    replies: GRIEF_ANSWERS,
};

const GRIEF_ANSWERS: &[Reply] = &[
    Reply {
        criteria: &[], weight: 1.4, bias: &[(Kind, 2.0), (Pious, 1.3)],
        act: Act::Agree, concepts: &["concern:grief"],
        lines: &["He was a good man. I'm sorry for it.", "The saints rest him."],
        then: Some(GRIEF_OPENS), end: None,
    },
    Reply {
        criteria: &[], weight: 0.6, bias: &[],
        act: Act::Remark, concepts: &["concern:grief"],
        lines: &["I'm - I'm sorry. I've not the words."],
        then: Some(GRIEF_OPENS), end: None,
    },
    Reply {
        criteria: &[], weight: 0.7, bias: &[(Kind, 1.3)],
        act: Act::Question, concepts: &["concern:grief", "aid"],
        lines: &["Is there aught thou needest fetched?"],
        then: Some(GRIEF_ASK), end: None,
    },
];

// Joined mid-talk: no second good-morrow
const GRIEF_REJOIN: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["And the house is so quiet since he passed, {other}.", "It's been a hard week, {other}, since we buried him."],
    replies: GRIEF_ANSWERS,
};

const GRIEF_OPENS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["I keep setting his place at table.", "He'd have liked the rain this morning."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.2, bias: &[(Kind, 1.5)],
            act: Act::Agree, concepts: &["concern:grief"],
            lines: &["Tell me of him.", "He'll be missed, truly."],
            then: Some(GRIEF_CLOSE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll not weary thee. God keep thee."],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_CLOSE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["Thank thee for listening - it helps, it does."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Any time, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

const GRIEF_ASK: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:grief", "aid"],
    calls: &["A little bread'd not go amiss.", "Wood, if thou canst spare it."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["aid", "farewell"],
            lines: &["It's done. Look for it by dark."],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.4, bias: &[(Stingy, 3.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll see what's to spare."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CONDOLE_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:grief"],
    calls: &["I was sorry to hear of thy loss, {other}.", "The village grieves with thee."],
    replies: CONDOLE_ANSWERS,
};

const CONDOLE_ANSWERS: &[Reply] = &[
    Reply {
        criteria: &[], weight: 1.0, bias: &[],
        act: Act::Agree, concepts: &["concern:grief"],
        lines: &["Thou'rt kind to say it.", "It means much, {other}."],
        then: Some(CONDOLE_OFFER), end: None,
    },
    Reply {
        criteria: &[], weight: 0.5, bias: &[(Proud, 2.0)],
        act: Act::PushBack, concepts: &["concern:grief"],
        lines: &["We manage."],
        then: Some(CONDOLE_OFFER), end: None,
    },
];

// Joined mid-talk: no fresh greeting
const CONDOLE_REJOIN: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief"],
    calls: &["I was sorry to hear of thy loss, {other}.", "And I'm sorry for thy loss, {other} - truly."],
    replies: CONDOLE_ANSWERS,
};

const CONDOLE_OFFER: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:grief", "aid"],
    calls: &["If thou needest aught, say the word."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Thou'rt good - I will."],
            then: None, end: Some(Farewell),
        },
        Reply {
            criteria: &[], weight: 0.4, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I won't trouble thee."],
            then: None, end: Some(Farewell),
        },
    ],
};

// =====================================================================
// The cleric: greeting a man of the cloth runs different to greeting a
// laborer - and the sick and grieving gain a "pray for me" fork nobody
// else sees.
// =====================================================================

const CLERIC_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Blessings on thee, {other}.", "The peace of the saints be with thee."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.2, bias: &[(Pious, 2.0)],
            act: Act::Greet, concepts: &["concern:faith"],
            lines: &["And on thee, father.", "Blessings on thy house as well."],
            then: Some(CLERIC_PARISH), end: None,
        },
        // Circumstance adds a path: only a sick answerer begs a prayer
        Reply {
            criteria: &[Has("self:status:sick")], weight: 0.9, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:sickness"],
            lines: &["A prayer for me, father - this cough won't quit.", "Father, pray for my house."],
            then: Some(CLERIC_PRAYS_SICK), end: None,
        },
        Reply {
            criteria: &[Has("self:status:grieving")], weight: 0.9, bias: &[],
            act: Act::Question, concepts: &["concern:faith", "concern:grief"],
            lines: &["Pray for him, father - he was taken sudden."],
            then: Some(CLERIC_PRAYS_GRIEF), end: None,
        },
        Reply {
            criteria: &[], weight: 0.6, bias: &[(Proud, 1.5)],
            act: Act::Remark, concepts: &["concern:faith"],
            lines: &["Morrow, father."],
            then: Some(CLERIC_PARISH), end: None,
        },
    ],
};

const CLERIC_PARISH: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["How fares thy house, child?", "Dost thou find peace in thy labor, child?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[(Pious, 1.5)],
            act: Act::Agree, concepts: &["concern:faith"],
            lines: &["We keep as we can, father.", "The season's been lean, father."],
            then: None, end: None, // drifts to small talk
        },
        Reply {
            criteria: &[], weight: 0.7, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I'll see thee at the bell, father.", "Pray for us, father."],
            then: None, end: Some(Farewell),
        },
    ],
};

// The cleric answers the plea - the parishioner's lines sit in the
// replies, the cleric's in the calls
const CLERIC_PRAYS_SICK: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:faith", "concern:sickness"],
    calls: &["I'll pray for thy mending, child - and send the herb-wife round.", "Rest and prayers, child - I'll come by."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[(Pious, 1.5), (Kind, 1.3)],
            act: Act::Farewell, concepts: &["concern:faith", "farewell"],
            lines: &["Thank thee, father.", "Bless thee, father - that's a comfort."],
            then: None, end: Some(Farewell),
        },
    ],
};

const CLERIC_PRAYS_GRIEF: Exchange = Exchange {
    act: Act::Answer,
    concepts: &["concern:faith", "concern:grief"],
    calls: &["I'll say a mass for him. He rests easy by the yew.", "The saints have him now, child. I'll light a candle."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[(Pious, 1.5), (Kind, 1.3)],
            act: Act::Farewell, concepts: &["concern:faith", "farewell"],
            lines: &["Thank thee, father.", "Bless thee, father - he'd have liked that."],
            then: None, end: Some(Farewell),
        },
    ],
};

const FATHER_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "concern:faith"],
    calls: &["Good morrow, father.", "A word with thee, father, if thou'lt spare it."],
    replies: FATHER_HEARS,
};

// Which way the father bids them speak picks the question, so each
// question meets its own answer
const FATHER_HEARS: &[Reply] = &[
    Reply {
        criteria: &[], weight: 1.0, bias: &[(Kind, 1.3)],
        act: Act::Answer, concepts: &["concern:faith"],
        lines: &["The saints keep thee, {other}. What is it?"],
        then: Some(FATHER_ASKS_DREAMS), end: None,
    },
    Reply {
        criteria: &[], weight: 1.0, bias: &[(Kind, 1.3)],
        act: Act::Answer, concepts: &["concern:faith"],
        lines: &["Always, child. Speak."],
        then: Some(FATHER_ASKS_TITHE), end: None,
    },
];

// Joined mid-talk: no second good-morrow
const FATHER_REJOIN: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["While I have thee, father - a question.", "Father, might I ask thee something?"],
    replies: FATHER_HEARS,
};

const FATHER_ASKS_DREAMS: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["Is it true the dead walk in dreams to warn us?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Answer, concepts: &["concern:faith"],
            lines: &["The dreams of the grieving are their own counsel, child."],
            then: Some(FATHER_THANKS), end: None,
        },
    ],
};

const FATHER_ASKS_TITHE: Exchange = Exchange {
    act: Act::Question,
    concepts: &["concern:faith"],
    calls: &["Must the miller's tithe stand when the mill's half idle?"],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Answer, concepts: &["concern:faith"],
            lines: &["The tithe stands - but I'll speak for thee at the manor."],
            then: Some(FATHER_THANKS), end: None,
        },
    ],
};

const FATHER_THANKS: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["concern:faith"],
    calls: &["Thank thee, father - that's a weight off."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Go in peace, {other}."],
            then: None, end: Some(Farewell),
        },
    ],
};

// =====================================================================
// Old scores: a rivalry makes every fork sharper, and a hothead can end
// it in a walk-off.
// =====================================================================

const RIVAL_GREET: Exchange = Exchange {
    act: Act::Greet,
    concepts: &["greeting", "feud"],
    calls: &["You.", "Look what the cat wouldn't drag in.", "{other}. Still about, I see."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Greet, concepts: &["feud"],
            lines: &["{other}."],
            then: Some(RIVAL_SNIPE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.8, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Still breathing, I see. Pity.", "The years haven't improved thee."],
            then: Some(RIVAL_BITE), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[(Proud, 2.0)],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["I've nothing to say to thee."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const RIVAL_SNIPE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Heard about thy fence. Shoddy work, that.", "Thy lot still owes mine, whatever thou sayest."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["My fence stands better than thy word."],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.7, bias: &[(Hotheaded, 3.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Say that again."],
            then: Some(RIVAL_BITE), end: None,
        },
    ],
};

const RIVAL_BITE: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["Make me."],
    replies: &[
        Reply {
            criteria: &[], weight: 0.8, bias: &[],
            act: Act::Remark, concepts: &["feud"],
            lines: &["Not worth the breath."],
            then: Some(RIVAL_END), end: None,
        },
        Reply {
            criteria: &[], weight: 0.5, bias: &[(Hotheaded, 4.0)],
            act: Act::PushBack, concepts: &["feud"],
            lines: &["Cur."],
            then: None, end: Some(WalksOff),
        },
    ],
};

const RIVAL_END: Exchange = Exchange {
    act: Act::Remark,
    concepts: &["feud"],
    calls: &["We're done here."],
    replies: &[
        Reply {
            criteria: &[], weight: 1.0, bias: &[],
            act: Act::Farewell, concepts: &["farewell"],
            lines: &["Aye. Done."],
            then: None, end: Some(Farewell),
        },
    ],
};

/// The example library, ordered most-likely-specific first; the cascade
/// sorts by criterion count and keeps this order on ties. One's own lot
/// leads, then one's calling (a priest calling on the sick speaks as a
/// priest - his tree carries their "pray for me" forks), then the other's
/// lot.
pub fn village_library() -> Vec<DialogueTree> {
    vec![
        DialogueTree { name: "the_sickness", criteria: &[Has("self:status:sick")],
            root: &SICK_GREET, rejoin: None },
        DialogueTree { name: "a_death_in_the_house", criteria: &[Has("self:status:grieving")],
            root: &GRIEF_GREET, rejoin: Some(&GRIEF_REJOIN) },
        DialogueTree { name: "the_cleric_blesses", criteria: &[Has("self:ideology:clergy")],
            root: &CLERIC_GREET, rejoin: Some(&CLERIC_PARISH) },
        DialogueTree { name: "visiting_the_sick", criteria: &[Has("other:status:sick")],
            root: &VISIT_GREET, rejoin: Some(&VISIT_REJOIN) },
        DialogueTree { name: "condolence_call", criteria: &[Has("other:status:grieving")],
            root: &CONDOLE_GREET, rejoin: Some(&CONDOLE_REJOIN) },
        DialogueTree { name: "asking_the_father", criteria: &[Has("other:ideology:clergy")],
            root: &FATHER_GREET, rejoin: Some(&FATHER_REJOIN) },
        DialogueTree { name: "old_scores", criteria: &[Has("pair:relation:rival")],
            root: &RIVAL_GREET, rejoin: Some(&RIVAL_SNIPE) },
        DialogueTree { name: "village_smalltalk", criteria: &[],
            root: &SMALL_GREET, rejoin: Some(&SMALL_CHAT) },
    ]
}

// =====================================================================
// House meetings: each attendee brings a pool of lines tagged with the
// concerns they actually have; the meeting threads on the tags.
// =====================================================================

/// Concern lines shared by everyone at a meeting, whatever their lot
const MEET_COMMON: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Agree,
        concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest"],
        bias: &[],
        lines: &["Aye, there's the truth of it.", "{other} has the right of it.", "It's so - I've thought the same."],
    },
    MeetingLine {
        act: Act::PushBack,
        concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest"],
        bias: &[(Hotheaded, 2.5)],
        lines: &["Talk's cheap - who'll do aught about it?", "And what wouldst thou have us do, {other}?"],
    },
    MeetingLine {
        act: Act::Remark,
        concepts: &["concern:harvest"],
        bias: &[],
        lines: &["If the harvest is good, we'll get through the winter.", "We'll need every hand at harvest."],
    },
];

const MEET_FOOD: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:food"], bias: &[],
        lines: &["The store won't last the month.", "My children go to bed hungry.", "We need to share what grain there is."],
    },
];
const MEET_SICKNESS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:sickness"], bias: &[],
        lines: &["Half the village is coughing.", "Who'll work the fields if we're all sick?", "We should look after the sick together."],
    },
];
const MEET_WAGES: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:wages"], bias: &[],
        lines: &["A day's work hardly buys a day's bread.", "The miller takes more every season.", "If we asked together, they'd have to listen."],
    },
];
const MEET_ROADS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:roads"], bias: &[],
        lines: &["The roads aren't safe to travel.", "No trader comes this way any more.", "We could watch the roads in turns."],
    },
];
const MEET_FAITH: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:faith"], bias: &[(Pious, 1.5)],
        lines: &["We need something to hold on to.", "The church is the only thing that brings us together.", "Let's keep faith with each other."],
    },
];
const MEET_RENTS: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:rents"], bias: &[],
        lines: &["The rents go up every season.", "I can't pay what the lord asks.", "One of us alone can't argue the rent."],
    },
];
const MEET_GRIEF: &[MeetingLine] = &[
    MeetingLine {
        act: Act::Remark, concepts: &["concern:grief", "concern:sickness"], bias: &[],
        lines: &["We buried my Tom last week - the sickness took him.", "There's a cold place at my table now."],
    },
];

/// The lines `speaker` could bring to a house meeting: what they actually
/// live with leads, the common stock pads the rest.
pub fn meeting_pool(speaker: &Speaker) -> Vec<MeetingLine> {
    let mut pool: Vec<MeetingLine> = vec![];
    if speaker.status.contains(&"sick") { pool.extend_from_slice(MEET_SICKNESS); }
    if speaker.status.contains(&"hungry") { pool.extend_from_slice(MEET_FOOD); }
    if speaker.status.contains(&"grieving") { pool.extend_from_slice(MEET_GRIEF); }
    pool.extend_from_slice(match speaker.ideology {
        "clergy" => MEET_FAITH,
        "merchant" => MEET_ROADS,
        "lord" => MEET_RENTS,
        _ => MEET_WAGES,
    });
    pool.extend_from_slice(MEET_COMMON);
    pool
}

/// A worked-example agenda for a house meeting
pub const MEET_PROMPT: MeetingPrompt = MeetingPrompt {
    text: "What ails the village most, friends? Speak true.",
    concepts: &["concern:food", "concern:sickness", "concern:wages", "concern:roads", "concern:faith", "concern:rents", "concern:harvest"],
};

/// A one-line backstory from a speaker's profile - enough for a grader to
/// check the talk against who they are.
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
            other => format!("Lately: {}.", other),
        });
    }
    parts.join(" ")
}
