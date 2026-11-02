# Data definitions for all trees and exchanges.
# Each entry specifies const_name, act, concepts, calls, replies.

DATA = [
    # -----------------------------------------------------------------
    # 1. Generic Smalltalk (Universal tail)
    # -----------------------------------------------------------------
    {
        "name": "SMALL_FADE",
        "act": "Remark",
        "concepts": ["smalltalk"],
        "calls": [
            "As thou sayest, friend.",
            "True enough - the fields don't gossip.",
            "Aye, words won't fill the storehouse.",
            "We have spoken our piece for today.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Fair day to thee, {other}.",
                    "Go well, {other}, and mind the mud.",
                    "The saints keep thy roof whole.",
                    "Fare thee well until the morrow.",
                ],
                "then": None, "end": "Farewell"
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Proud", 2.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I take my leave.",
                    "Enough chatter. Good day.",
                    "My day is waiting.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "SMALL_GOSSIP_MORE",
        "act": "Remark",
        "concepts": ["gossip"],
        "calls": [
            "They do say the reeve counts the sheep twice over.",
            "The herb-wife was seen out past the abbey wall at dusk.",
            "Old Wulfric swore he saw an owl perched on the tavern roof.",
            "A stranger with two laden mules passed by the weir at noon.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["gossip"],
                "lines": [
                    "Well, that is talk enough for one morning.",
                    "Ha! I'll believe the half of it and no more.",
                    "Tavern tales grow taller with each ale drawn.",
                    "Every tongue in the parish wags before sunset.",
                ],
                "then": "SMALL_FADE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Hotheaded", 2.0)],
                "act": "PushBack", "concepts": ["gossip"],
                "lines": [
                    "Better to tend thy own hearth than watch the road.",
                    "I have heard that lie three times this week.",
                    "Folk will invent dragons where there's only sheep.",
                ],
                "then": "SMALL_FADE", "end": None
            }
        ]
    },
    {
        "name": "SMALL_GOSSIP",
        "act": "Remark",
        "concepts": ["gossip", "smalltalk"],
        "calls": [
            "Didst thou hear who was seen down by the mill?",
            "They say the miller took an extra scoop of meal again.",
            "The peddler's cart lost a wheel at the bridge.",
            "A rider from the castle came through before dawn.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [("Gregarious", 2.0)],
                "act": "Remark", "concepts": ["gossip"],
                "lines": [
                    "Thou dost not say it! What happened then?",
                    "Aye? And what else did folk see?",
                    "Tell on, neighbor - my ears are open.",
                    "I knew that bridge would give way sooner or later.",
                ],
                "then": "SMALL_GOSSIP_MORE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 2.0)],
                "act": "PushBack", "concepts": ["gossip"],
                "lines": [
                    "Idle tongues, that. It breeds mischief.",
                    "I'll not repeat marketplace whispers.",
                    "Let the miller and the rider sort their own affairs.",
                ],
                "then": "SMALL_FADE", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "A tale for another hour - I must away.",
                    "The sun climbs high; I have chores to finish.",
                    "Walk with God, {other}, I cannot linger.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SMALL_SEASON",
        "act": "Remark",
        "concepts": ["smalltalk", "harvest"],
        "calls": [
            "We must thatch the hayrick before the clouds burst.",
            "The kitchen gardens are dry as a bone this month.",
            "Rooks are gathering thick in the elm trees.",
            "The ditch along the south pasture needs clearing.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["harvest"],
                "lines": [
                    "Aye, work never sleeps in this valley.",
                    "A dry spade makes for a hungry winter.",
                    "Every hand will be needed when the grain ripens.",
                    "I'll lend a pitchfork if thou hast timber to spare.",
                ],
                "then": "SMALL_GOSSIP", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [("Lazy", 2.5)],
                "act": "PushBack", "concepts": ["harvest"],
                "lines": [
                    "The hay will wait until tomorrow, surely.",
                    "Let the rain fall; my bones need a day's rest.",
                    "There's always another ditch to dig.",
                ],
                "then": "SMALL_FADE", "end": None
            }
        ]
    },
    {
        "name": "SMALL_FARES",
        "act": "Answer",
        "concepts": ["family", "smalltalk"],
        "calls": [
            "Well enough, thank thee, {other}.",
            "We rub along as ever, by the saints' mercy.",
            "The little ones keep me running from dawn to dusk.",
            "My knees ache with the damp, but I still walk.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["family"],
                "lines": [
                    "Good to hear it - long may health stay under thy roof.",
                    "Aye, children grow like weeds in the summer ditch.",
                    "A full hearth is a blessing in hard times.",
                    "May the coming season be kind to thy cottage.",
                ],
                "then": "SMALL_SEASON", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Give my best to thy household. Fare well.",
                    "God keep thy folk in peace, {other}.",
                    "I must see to my own broth. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SMALL_VOLLEY",
        "act": "Remark",
        "concepts": ["smalltalk"],
        "calls": [
            "Even so, we will manage as we always do.",
            "Aye, the year will turn and bring what it brings.",
            "A steady hand and a patient mule see it through.",
            "No use fretting over skies we cannot steer.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["smalltalk"],
                "lines": [
                    "That it will - the store is not empty yet.",
                    "Aye, we have lived through harder winters than this.",
                    "True words; worry never baked a single loaf.",
                    "So long as the well stays sweet, we'll thrive.",
                ],
                "then": "SMALL_GOSSIP", "end": None
            },
            {
                "criteria": [], "weight": 0.7, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I'll not keep thee standing in the wind. Go well.",
                    "On thy way then, and mine. Fair day.",
                    "Until the church bell rings, neighbor.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SMALL_CHAT",
        "act": "Remark",
        "concepts": ["smalltalk", "weather"],
        "calls": [
            "Looks like rain again, doesn't it?",
            "Cold wind off the moor this morning.",
            "Gray clouds gathering over the western hill.",
            "The morning dew was heavy on the clover.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["weather"],
                "lines": [
                    "It will be a wet week, mark me.",
                    "The thatch won't thank it if it pours.",
                    "Aye, and the mud will take thy boots right off.",
                    "Good for the barley, but bad for the road.",
                ],
                "then": "SMALL_VOLLEY", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Hotheaded", 2.0)],
                "act": "PushBack", "concepts": ["weather"],
                "lines": [
                    "Rain? It is the driest week in memory.",
                    "Thou seest a tempest in every puff of smoke.",
                    "My joints say dry weather, whatever the sky says.",
                ],
                "then": "SMALL_VOLLEY", "end": None
            },
            {
                "criteria": [], "weight": 0.8, "bias": [("Gregarious", 2.0)],
                "act": "Question", "concepts": ["family", "smalltalk"],
                "lines": [
                    "And how fares thy folk in this cold snap?",
                    "Tell me true, {other} - how goes it with thee?",
                    "Has thy brother returned from the market town?",
                ],
                "then": "SMALL_FARES", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Well, I had best be about my chores.",
                    "I should get on before the bell strikes.",
                    "Fair day to thee, {other}.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SMALL_GREET",
        "act": "Greet",
        "concepts": ["greeting"],
        "calls": [
            "Good morrow to thee, {other}.",
            "Well met, neighbor {other}.",
            "A fair day to thee and thine, {other}.",
            "God's peace on this path, {other}.",
            "Hail, {other} - traveling far today?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["greeting"],
                "lines": [
                    "And to thee, {other}.",
                    "Well met indeed, friend.",
                    "The saints keep thee, {other}.",
                    "Good day and full granaries to thee.",
                ],
                "then": "SMALL_CHAT", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Hotheaded", 3.0), ("Proud", 2.0)],
                "act": "Remark", "concepts": ["greeting"],
                "lines": [
                    "Morrow.",
                    "Aye. What is it, {other}?",
                    "I am here, as thou seest.",
                ],
                "then": "SMALL_CHAT", "end": None
            },
            {
                "criteria": [("Lacks(\"pair:relation:friend\")"), ("Lacks(\"pair:relation:kin\")")],
                "weight": 0.25, "bias": [("Proud", 3.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I have no time for idle talk today.",
                    "Work is waiting at the barn, {other}.",
                    "I walk on business, neighbor.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
]
print(f"Loaded {len(DATA)} base exchanges.")
