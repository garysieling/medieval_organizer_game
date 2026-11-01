# Part 3: Hunger, Cold Hearth, Drunk, Kin, Friend, Stranger, Trades (Smith, Miller, Baker, Weaver, Carpenter, Mason, Shepherd, Brewer, Beekeeper, Herb, Merchant, Lord, Fisher, Charcoal), Elder, Newborn, and Meeting Pools.

TRADE_DATA = [
    # -----------------------------------------------------------------
    # Hunger & Lean Times
    # -----------------------------------------------------------------
    {
        "name": "HUNGER_HOPE",
        "act": "Remark",
        "concepts": ["concern:food"],
        "calls": [
            "If the rye crop ripens before the frost, we'll see another spring.",
            "They say the monastery may open its tithe barn if hunger deepens.",
            "We have boiled nettles and wild leeks before, and survived.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Aye, hold fast. Better days will follow.",
                    "The earth will feed us yet. Go gently, {other}.",
                    "May tomorrow bring a fuller kettle.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "HUNGER_SHARE",
        "act": "Remark",
        "concepts": ["concern:food", "aid"],
        "calls": [
            "We boiled a pot of turnip tops and barley chaff this noon.",
            "My store bin has nothing but dust and two dried apples.",
            "The children went to sleep crying with empty bellies last night.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.2, "bias": [("Kind", 2.0)],
                "act": "Agree", "concepts": ["concern:food", "aid"],
                "lines": [
                    "Take half my loaf, neighbor. No child should hunger in this village.",
                    "Come by our hearth; there is pottage enough for two more bowls.",
                    "We have a sack of roots in the cellar. Let us divide it.",
                ],
                "then": "HUNGER_HOPE", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [("Stingy", 3.0)],
                "act": "PushBack", "concepts": ["concern:food"],
                "lines": [
                    "Everyone's bin is empty; don't look to my door for bread.",
                    "We each must stretch our own store as best we can.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "HUNGER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:food"],
        "calls": [
            "Good morrow, {other}. A hollow belly makes for a slow step.",
            "Well met, friend, though the times are thin and hard.",
            "Greetings, {other}. Hast thou found anything to put in the pot?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:food"],
                "lines": [
                    "Morrow, {other}. The porridge is thin as ditchwater this week.",
                    "Aye, we scrape the bottom of the flour chest.",
                    "The storehouse has dwindled down to crumbs.",
                ],
                "then": "HUNGER_SHARE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 2.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "We tighten our belts and carry on.",
                    "Talk won't fill the larder. I must seek wild greens.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },

    # -----------------------------------------------------------------
    # Cold Hearth & Fuel (Safe terms: kindling, peat, brushwood, turf)
    # -----------------------------------------------------------------
    {
        "name": "COLD_FIRE",
        "act": "Remark",
        "concepts": ["concern:fuel"],
        "calls": [
            "The kindling caught! Sweet smoke rises up the chimney at last.",
            "Blow on the embers gently - the dry twigs are taking flame.",
            "Ah, blessed warmth! My stiff fingers can move again.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Bank the coals with ashes so they last till dawn.",
                    "May thy hearth keep hot through the bitter night.",
                    "The fire is life in winter. Fare thee well.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "COLD_GATHER",
        "act": "Question",
        "concepts": ["concern:fuel", "aid"],
        "calls": [
            "Will thou come to the common wood to cut deadwood and kindling?",
            "Can I borrow a shovel to dig peat from the dry bog?",
            "Hast thou glowing embers in an iron pot to rekindle our cold stones?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:fuel"],
                "lines": [
                    "Aye, I'll bring my hatchet. Two bundles of dry boughs will do.",
                    "Take these live embers from my hearth; shield them from the wind.",
                    "We stacked dry turf behind the byre; take what thou canst carry.",
                ],
                "then": "COLD_FIRE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Stingy", 2.0)],
                "act": "PushBack", "concepts": ["concern:fuel"],
                "lines": [
                    "The bailiff watches the common wood; he will fine us for gathering boughs.",
                    "My own hearth barely has coals to warm an egg.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "COLD_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:fuel"],
        "calls": [
            "Brrr! Good morrow, {other}. The rime frost is inside the thatch.",
            "Well met, neighbor. My breath freezes right before my nose.",
            "Greetings, {other}. The wind blows straight through the cottage wattles.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["concern:fuel"],
                "lines": [
                    "Our hearth went cold before midnight; the stone is like ice.",
                    "We have burned through the last dry bundle of brushwood.",
                    "Without dry turf, the sheepfold is warmer than the cottage.",
                ],
                "then": "COLD_GATHER", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I must chop timber before the dark settles in.",
                    "Keep moving or the cold will numb thy toes. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },

    # -----------------------------------------------------------------
    # Morning-after & Tavern Revelry
    # -----------------------------------------------------------------
    {
        "name": "DRUNK_SOBER",
        "act": "Remark",
        "concepts": ["smalltalk"],
        "calls": [
            "A bucket of icy well water over the skull will clear the cobwebs.",
            "The plough handles won't hold themselves, headache or no.",
            "Bread and pickled onions, that's what the stomach needs.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Walk straight, {other}, or the reeve will notice.",
                    "No more brown ale for thee until Sunday! Fare well.",
                    "May the sun be gentle on thy poor eyes today.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "DRUNK_RECALL",
        "act": "Remark",
        "concepts": ["smalltalk"],
        "calls": [
            "Did I truly challenge the smith to a wrestling match at midnight?",
            "They say we sang every verse of the Harvest ballad three times over.",
            "My purse is six silver pennies lighter than when dusk fell.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [("Gregarious", 2.0)],
                "act": "Agree", "concepts": ["smalltalk"],
                "lines": [
                    "Thou didst indeed! And fell backwards over the ale barrel!",
                    "Aye, and the fiddler laughed so hard his bow went flying.",
                    "It was a merry night, though the morning pays the reckoning.",
                ],
                "then": "DRUNK_SOBER", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Pious", 2.0)],
                "act": "PushBack", "concepts": ["smalltalk"],
                "lines": [
                    "Thou made an ungodly spectacle of thyself.",
                    "A shameful racket; the priest shook his head at the doorway.",
                ],
                "then": "DRUNK_SOBER", "end": None
            }
        ]
    },
    {
        "name": "DRUNK_GREET",
        "act": "Greet",
        "concepts": ["greeting", "smalltalk"],
        "calls": [
            "Oof... good morrow, {other}. Don't shout, I pray thee.",
            "Well met, friend. My head is hammering like the smith's anvil.",
            "Ugh, the sunlight is blinding today, {other}.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["smalltalk"],
                "lines": [
                    "Ha! The tavern-keeper's ale had a bite to it, didn't it?",
                    "Thou lookest like a corpse dug up from the wet turf.",
                    "I warned thee that dark brew was brewed for giants.",
                ],
                "then": "DRUNK_RECALL", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Sleep it off in the hayloft, {other}.",
                    "Go dunk thy head in the horse trough. Fair day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },

    # -----------------------------------------------------------------
    # Kinship & Family Warmth
    # -----------------------------------------------------------------
    {
        "name": "KIN_AID",
        "act": "Remark",
        "concepts": ["aid", "family"],
        "calls": [
            "Blood is thicker than river mud, {other}.",
            "Whatever befalls the village, our clan stands shoulder to shoulder.",
            "Our grandfather laid the hearthstones in both our cottages.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Always, kin. The family latch is never barred to thee.",
                    "God keep our house and hearth safe from storm.",
                    "See thee at Sunday meal, {other}.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "KIN_FAMILY",
        "act": "Question",
        "concepts": ["family"],
        "calls": [
            "How fares mother's knee in this damp spell?",
            "Has our cousin returned with the pack horse from the fair?",
            "Dost thou need an extra hand to dig the potato trench?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["family"],
                "lines": [
                    "She sits by the fire with warm sheep's grease; it eases the pain.",
                    "Aye, he brought home two bags of salt and a fine sickle.",
                    "Come by with a spade after the noon bell, brother.",
                ],
                "then": "KIN_AID", "end": None
            }
        ]
    },
    {
        "name": "KIN_GREET",
        "act": "Greet",
        "concepts": ["greeting", "family"],
        "calls": [
            "Well met, cousin {other}!",
            "Good morrow, kin! How fares the old cottage?",
            "Greetings, brother! It does my heart good to see thy face.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["family"],
                "lines": [
                    "And to thee, kin! The roof still holds.",
                    "Well met, dear cousin. Thou'rt looking hale and hearty.",
                    "God bless thee, kin. What news from thy side of the lane?",
                ],
                "then": "KIN_FAMILY", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # Friendship Bonds
    # -----------------------------------------------------------------
    {
        "name": "FRIEND_PART",
        "act": "Remark",
        "concepts": ["friendship"],
        "calls": [
            "A good friend is worth more than a pouch of silver.",
            "We have shared crusts and sweat for twenty years now.",
            "Thy word has never failed me, {other}.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Nor thine me, old friend. Walk safe.",
                    "We will share a cup of cider when the work is done.",
                    "Fair day, {other} - peace on thy household.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "FRIEND_TRUST",
        "act": "Question",
        "concepts": ["friendship"],
        "calls": [
            "Dost thou recall the summer the mill dam broke?",
            "Can I count on thy team when we haul the heavy timber?",
            "Hast thou heard any whispers about the new taxes?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["friendship"],
                "lines": [
                    "Aye, we were waist-deep in mud until midnight saving the sacks!",
                    "My oxen are thine whenever thou needest them, friend.",
                    "The bailiff spoke of three pennies per hide, but we'll contest it.",
                ],
                "then": "FRIEND_PART", "end": None
            }
        ]
    },
    {
        "name": "FRIEND_GREET",
        "act": "Greet",
        "concepts": ["greeting", "friendship"],
        "calls": [
            "A hearty good morrow to thee, {other}!",
            "Well met, true friend! The sun shines brighter when thou'rt near.",
            "Greetings, old comrade! How goes the day's labor?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["friendship"],
                "lines": [
                    "And to thee, {other}! It gladdens my soul to see thee.",
                    "Well met indeed, friend. The chores are lighter for thy smile.",
                    "Good day, partner! We'll weather whatever comes.",
                ],
                "then": "FRIEND_TRUST", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # Stranger on the Road
    # -----------------------------------------------------------------
    {
        "name": "STRANGER_PASS",
        "act": "Remark",
        "concepts": ["stranger"],
        "calls": [
            "Keep to the beaten track; the marshes to the east are treacherous.",
            "The alehouse by the bridge offers straw beds if thou hast coin.",
            "The parish watch keeps guard after the curfew bell.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "My thanks for the counsel, villager. I pass in peace.",
                    "Safe travels to thee as well. Good day.",
                    "The highway calls; farewell.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "STRANGER_PATH",
        "act": "Question",
        "concepts": ["stranger"],
        "calls": [
            "Where does this trail lead past the ridge, traveler?",
            "What town dost thou hail from, stranger?",
            "Is the road north safe from cutpurses and wolves?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["stranger"],
                "lines": [
                    "It leads down to the cathedral town and the salt wharves.",
                    "I come from the valley beyond the high moors.",
                    "The road is quiet by day, but light a torch if thou walkest at dusk.",
                ],
                "then": "STRANGER_PASS", "end": None
            }
        ]
    },
    {
        "name": "STRANGER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "stranger"],
        "calls": [
            "Halt there, traveler. What brings thee through our parish?",
            "Good day, stranger. Thou'rt far from the king's highway.",
            "Peace be upon thy journey, traveler. Who art thou?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["stranger"],
                "lines": [
                    "Greetings, villager. I mean no harm on these lanes.",
                    "Well met. I am but a traveler seeking the ford across the water.",
                    "Peace to thy fields, friend. I look for the nearest smithy.",
                ],
                "then": "STRANGER_PATH", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 2.0)],
                "act": "PushBack", "concepts": ["stranger"],
                "lines": [
                    "My business is my own, goodman.",
                    "The king's highway is open to every free soul.",
                ],
                "then": "STRANGER_PASS", "end": None
            }
        ]
    },
]
print(f"Loaded {len(TRADE_DATA)} trade and circumstance exchanges.")
