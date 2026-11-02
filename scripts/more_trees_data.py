# Part 2: Sickness, Grief, Clergy, Rivalry, Hunger, Cold Hearth, Drunk, Kin, Friend, Stranger

MORE_DATA = [
    # -----------------------------------------------------------------
    # Sickness
    # -----------------------------------------------------------------
    {
        "name": "SICK_THANKS",
        "act": "Remark",
        "concepts": ["concern:sickness", "aid"],
        "calls": [
            "Thou'rt kind, {other}. The saints remember a neighborly hand.",
            "My thanks. A warm bowl does more than herbs.",
            "Bless thee for thinking of our small house.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Think nothing of it. We look after our own.",
                    "Rest quiet by the coals. I'll check on thee tomorrow.",
                    "The village stands together or it falls. Fare well.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SICK_STUNG",
        "act": "Remark",
        "concepts": ["concern:sickness"],
        "calls": [
            "It is not the black pestilence, keep thy wits about thee.",
            "As thou wilt, then - step wide if thou must.",
            "A little cough won't kill an honest worker.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 0.8, "bias": [("Kind", 1.5)],
                "act": "Agree", "concepts": ["concern:sickness"],
                "lines": [
                    "Forgive me - fear makes cowards of us all.",
                    "Thou hast right of it. I spoke too quick.",
                    "No offence meant, {other}. We are all on edge.",
                ],
                "then": "SICK_THANKS", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Hotheaded", 3.0)],
                "act": "PushBack", "concepts": ["concern:sickness"],
                "lines": [
                    "I'll not catch my death for thy pride.",
                    "Breathe elsewhere then, knave.",
                    "Stay in thy own yard if thou'rt frightened.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "SICK_ANSWER",
        "act": "Answer",
        "concepts": ["concern:sickness"],
        "calls": [
            "Aye, since the heavy frost. The fever burns at dusk and chills at dawn.",
            "They say the herb-wife's willow draught brings the sweat out.",
            "My lungs feel like wet wool on a rainy morning.",
            "Half the north lane is coughing into their sleeves.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [("Kind", 1.5)],
                "act": "Agree", "concepts": ["concern:sickness"],
                "lines": [
                    "The saints speed thy mending, {other}.",
                    "Drink hot boiled water with a scrape of honey.",
                    "May the affliction pass thee lightly.",
                ],
                "then": None, "end": None
            },
            {
                "criteria": [], "weight": 0.8, "bias": [("Kind", 2.0), ("Pious", 1.5)],
                "act": "Remark", "concepts": ["concern:sickness", "aid"],
                "lines": [
                    "I will send broth round when the kettle boils.",
                    "I'll ask Father Anselm to light a candle for thy recovery.",
                    "Sit by the hearth; I'll fetch firewood if thou hast none.",
                ],
                "then": "SICK_THANKS", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Stingy", 2.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I must be away before the air catches me too.",
                    "Mind thy cup and bowl. Good day.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "SICK_EXCUSE",
        "act": "Answer",
        "concepts": ["concern:sickness"],
        "calls": [
            "I'd rest in the straw if the chores would do themselves.",
            "There is no one else to fetch water from the river.",
            "The calves won't feed themselves, fever or no.",
            "The lord's rent waits for no man's cough.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["concern:sickness"],
                "lines": [
                    "We labor or we go hungry, that's the hard truth of it.",
                    "Aye, the dirt cares nothing for an aching back.",
                    "A poor man's bed is a luxury he can rarely afford.",
                ],
                "then": None, "end": None
            },
            {
                "criteria": [], "weight": 0.8, "bias": [("Kind", 2.0)],
                "act": "Remark", "concepts": ["concern:sickness", "aid"],
                "lines": [
                    "Let me carry thy pail to the well, {other}.",
                    "Go inside; I'll see to thy livestock this evening.",
                    "I have barley meal to spare for thy kettle.",
                ],
                "then": "SICK_THANKS", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Rest when thou canst, {other}. Fair day.",
                    "God grant thee strength for the day's toil.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SICK_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:sickness"],
        "calls": [
            "*cough* Good morrow, {other}.",
            "Forgive my rough voice - this chest cold won't quit.",
            "Morrow, {other}. Don't stand too close in the breeze.",
            "A heavy morning, neighbor. My limbs are like lead.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.2, "bias": [("Kind", 2.0)],
                "act": "Agree", "concepts": ["concern:sickness"],
                "lines": [
                    "Poor soul - get thee back into the warm cottage.",
                    "Thou shouldst be under woolen blankets, not out here.",
                    "Thou lookest pale as a winter moon, {other}.",
                ],
                "then": "SICK_EXCUSE", "end": None
            },
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Question", "concepts": ["concern:sickness"],
                "lines": [
                    "Is it the ague that took the miller's lad?",
                    "How long has this fever had hold of thee?",
                    "Hast thou seen the herb-wife for a draught?",
                ],
                "then": "SICK_ANSWER", "end": None
            },
            {
                "criteria": [("Lacks(\"self:status:sick\")")], "weight": 0.5, "bias": [("Proud", 1.5), ("Stingy", 1.5)],
                "act": "PushBack", "concepts": ["concern:sickness"],
                "lines": [
                    "Keep thy distance, neighbor. I have no wish to catch it.",
                    "I will hear thee from across the ditch.",
                    "Cover thy mouth when thou speakest in the wind.",
                ],
                "then": "SICK_STUNG", "end": None
            }
        ]
    },
    {
        "name": "VISIT_PROMISE",
        "act": "Remark",
        "concepts": ["aid"],
        "calls": [
            "It is done - look for it by sundown.",
            "I'll have my girl bring the loaf before dark.",
            "Rest easy; the parish will not let thee freeze.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Thou'rt a true neighbor, {other}.",
                    "The blessing of Heaven on thy roof.",
                    "I'll repay the kindness when my legs mend.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "VISIT_CARE",
        "act": "Question",
        "concepts": ["concern:sickness", "aid"],
        "calls": [
            "Is there aught thou needest brought to the cottage?",
            "Can I fetch water or chop kindling for thy hearth?",
            "Dost thou have grain left in the bin?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["aid", "concern:food"],
                "lines": [
                    "A little bread or salt, if thou canst spare it.",
                    "Some dry firewood would keep the chill out of our bones.",
                    "A jug of fresh well water would be a godsend.",
                ],
                "then": "VISIT_PROMISE", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [("Pious", 1.5)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Just a prayer at the church altar, {other}.",
                    "We have enough for today. Pray for my children.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "VISIT_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:sickness"],
        "calls": [
            "How dost thou fare today, {other}?",
            "I heard the sickness took thy household - how is it?",
            "Peace to this cottage, {other}. How goes the fever?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:sickness"],
                "lines": [
                    "Poorly, if thou'lt have the plain truth.",
                    "Weak as a lamb, but still drawing breath.",
                    "The night was hard, but the morning brings a little ease.",
                ],
                "then": "VISIT_CARE", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [("Proud", 2.0)],
                "act": "PushBack", "concepts": ["concern:sickness"],
                "lines": [
                    "It will pass. We do not need fussing over.",
                    "I have weathered worse than a common cough.",
                ],
                "then": "VISIT_CARE", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # Grief & Bereavement
    # -----------------------------------------------------------------
    {
        "name": "GRIEF_CLOSE",
        "act": "Remark",
        "concepts": ["concern:grief"],
        "calls": [
            "Thank thee for listening, {other} - it lightens the heart.",
            "It does a body good to speak his name aloud.",
            "The house feels less empty when a neighbor sits a spell.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Any hour, {other}. My door is always unlatched.",
                    "We do not forget those who walked this lane with us.",
                    "May holy angels watch over thy sleep tonight.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "GRIEF_ASK",
        "act": "Answer",
        "concepts": ["concern:grief", "aid"],
        "calls": [
            "A loaf of bread would not go amiss, {other}.",
            "A few split logs, if thou hast any seasoned oak.",
            "Someone to watch the sheep for an afternoon.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["aid", "farewell"],
                "lines": [
                    "It is done. Look for my boy with the basket by dusk.",
                    "I will bring it myself before sunset, friend.",
                    "Leave the sheep to me; mourn in peace.",
                ],
                "then": None, "end": "Farewell"
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Stingy", 3.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I will see what can be spared from our cellar.",
                    "Times are tight, but I'll see what I can find.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "GRIEF_OPENS",
        "act": "Remark",
        "concepts": ["concern:grief"],
        "calls": [
            "I keep setting his wooden spoon at table.",
            "She loved the smell of the damp earth after rain.",
            "Every corner of the cottage remembers his footsteps.",
            "Forty winters together, and now only the silence.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.2, "bias": [("Kind", 1.5)],
                "act": "Agree", "concepts": ["concern:grief"],
                "lines": [
                    "Tell me of him - good memories are a shield against darkness.",
                    "She was loved by all who knew her in the village.",
                    "Their toil shaped this parish; their memory will outlive us.",
                ],
                "then": "GRIEF_CLOSE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I'll not weary thee with more words. God keep thee.",
                    "Rest thy sorrow in prayer, {other}. Fare well.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "GRIEF_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:grief"],
        "calls": [
            "{other}. Forgive me - the house is very still since he passed.",
            "Morrow, {other}. The bell tolled, and my hearth is dark.",
            "Well met, friend. The days feel longer than they used to.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.4, "bias": [("Kind", 2.0), ("Pious", 1.5)],
                "act": "Agree", "concepts": ["concern:grief"],
                "lines": [
                    "He was an honest soul. The whole village mourns him.",
                    "The saints receive her spirit in eternal peace.",
                    "My heart aches for thee, neighbor.",
                ],
                "then": "GRIEF_OPENS", "end": None
            },
            {
                "criteria": [], "weight": 0.7, "bias": [("Kind", 1.3)],
                "act": "Question", "concepts": ["concern:grief", "aid"],
                "lines": [
                    "Is there any chore I can take off thy shoulders?",
                    "Hast thou eaten a warm bite today, {other}?",
                ],
                "then": "GRIEF_ASK", "end": None
            }
        ]
    },
    {
        "name": "CONDOLE_OFFER",
        "act": "Remark",
        "concepts": ["concern:grief", "aid"],
        "calls": [
            "If thou needest aught at all, say but the word.",
            "We have flour and eggs; let us share with thy house.",
            "My sons can plough thy furrow until thou'rt ready.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Thou'rt a true soul, {other}. I will remember it.",
                    "Thy kindness brings tears to an old eye. God bless thee.",
                ],
                "then": None, "end": "Farewell"
            },
            {
                "criteria": [], "weight": 0.4, "bias": [("Proud", 2.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "We will manage on our own, but thank thee kindly.",
                    "I will not burden thy house with my sorrow.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CONDOLE_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:grief"],
        "calls": [
            "I was deeply grieved to hear of thy loss, {other}.",
            "The village stands beside thee in this dark trial.",
            "God's mercy comfort thee in thy sorrow, neighbor.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Agree", "concepts": ["concern:grief"],
                "lines": [
                    "Thou'rt good to come by, {other}. It means much.",
                    "Thank thee for thy prayers and thoughts.",
                    "The road is lonely now, but kind words help.",
                ],
                "then": "CONDOLE_OFFER", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 2.0)],
                "act": "PushBack", "concepts": ["concern:grief"],
                "lines": [
                    "We endure what Heaven sends us.",
                    "Tears do not bring back the dead.",
                ],
                "then": "CONDOLE_OFFER", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # Clergy Blessings & Pastoral Care
    # -----------------------------------------------------------------
    {
        "name": "CLERIC_COMFORT",
        "act": "Remark",
        "concepts": ["concern:faith"],
        "calls": [
            "The saints watch over thy cottage. I will come by with holy water and draughts.",
            "Rest by the hearth, child - I'll light a candle and pray for thy household.",
            "Cast thy burdens before the altar; the Lord does not forget the meek.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Thank thee, father - thy blessing brings comfort.",
                    "The Lord keep thee, father, and preserve thy voice.",
                    "I will be at the church when the bell sounds.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CLERIC_PRAYER",
        "act": "Question",
        "concepts": ["concern:faith", "concern:grief", "concern:sickness"],
        "calls": [
            "The Lord comfort thee, child. Tell me of thy affliction.",
            "Speak, child - my prayers and candles are with thee.",
            "Open thy heart; what sorrow weighs upon thee?",
        ],
        "replies": [
            {
                "criteria": [("Has(\"self:status:sick\")")], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:sickness"],
                "lines": [
                    "It is the little one, father - she burns with fever.",
                    "This cough won't leave me, and our store of grain is thin.",
                ],
                "then": "CLERIC_COMFORT", "end": None
            },
            {
                "criteria": [("Has(\"self:status:grieving\")")], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:grief"],
                "lines": [
                    "We buried my brother last week, and grief hollows my chest.",
                ],
                "then": "CLERIC_COMFORT", "end": None
            },
            {
                "criteria": [], "weight": 0.4, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I must be on my way, {other}. Fare well.",
                    "The sun climbs high; chores call me. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CLERIC_PARISH",
        "act": "Remark",
        "concepts": ["concern:faith"],
        "calls": [
            "How fares thy house in these lean weeks, child?",
            "Dost thou find peace in thy daily labor, child?",
            "Remember that honest work in the field is a holy prayer.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [("Pious", 1.5)],
                "act": "Agree", "concepts": ["concern:faith"],
                "lines": [
                    "We keep as best we can, father. The toil is hard.",
                    "The season has been lean, father, but faith sustains us.",
                    "I try to walk uprightly before God and my neighbors.",
                ],
                "then": None, "end": None
            },
            {
                "criteria": [], "weight": 0.7, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I will see thee at the Sunday bell, father.",
                    "Pray for our harvest, father. Fair day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CLERIC_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:faith"],
        "calls": [
            "Blessings on thee, {other}.",
            "The peace of the saints be with thee, child.",
            "God's grace light upon thy steps, {other}.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.2, "bias": [("Pious", 2.0)],
                "act": "Greet", "concepts": ["concern:faith"],
                "lines": [
                    "And on thee, father.",
                    "Blessings on thy holy house as well.",
                    "Peace be with thee, good priest.",
                ],
                "then": "CLERIC_PARISH", "end": None
            },
            {
                "criteria": [("Has(\"self:status:sick\")")], "weight": 1.0, "bias": [],
                "act": "Question", "concepts": ["concern:faith", "concern:sickness"],
                "lines": [
                    "A prayer for me, father - this fever won't quit.",
                    "Father, hear my plea; the sickness grips my throat.",
                ],
                "then": "CLERIC_PRAYER", "end": None
            },
            {
                "criteria": [("Has(\"self:status:grieving\")")], "weight": 1.0, "bias": [],
                "act": "Question", "concepts": ["concern:faith", "concern:grief"],
                "lines": [
                    "Pray for him, father - he was taken so sudden.",
                    "A blessing for my sorrowing cottage, father.",
                ],
                "then": "CLERIC_PRAYER", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 1.5)],
                "act": "Remark", "concepts": ["concern:faith"],
                "lines": [
                    "Morrow, father.",
                    "A fair day to thee, priest.",
                ],
                "then": "CLERIC_PARISH", "end": None
            }
        ]
    },
    {
        "name": "FATHER_THANKS",
        "act": "Remark",
        "concepts": ["concern:faith"],
        "calls": [
            "Thank thee, father - that lifts a great stone from my mind.",
            "Thy counsel is wise, father. We will trust in Providence.",
            "The church has been our refuge through many a cold year.",
            "Thy blessing gives me heart to face the coming week, father.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Go in peace, {other}, and love thy neighbors.",
                    "The Lord preserve thy hearth and thy children.",
                    "Walk in the light, child. Fare well.",
                    "My prayers go with thee, {other}.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "FATHER_ASKS",
        "act": "Question",
        "concepts": ["concern:faith"],
        "calls": [
            "Must the tithe stand full when the granary is half empty, father?",
            "Can the church spare a peck of grain for the hungry children, father?",
            "Will the parish altar provide candles for our sick neighbor, father?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:faith"],
                "lines": [
                    "The church will share its grain, child - none shall starve at the door.",
                    "The tithe stands, but I will speak for thy house at the manor court.",
                    "Holy candles and broth shall be brought to their bedside this very day.",
                ],
                "then": "FATHER_THANKS", "end": None
            }
        ]
    },
    {
        "name": "FATHER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:faith"],
        "calls": [
            "Good morrow, father.",
            "A word with thee, father, if thou canst spare a moment.",
            "Greetings, holy father - I seek thy pastoral ear.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [("Kind", 1.3)],
                "act": "Greet", "concepts": ["concern:faith"],
                "lines": [
                    "The saints keep thee, {other}. What weighs on thee?",
                    "Always, child. Speak freely in the sight of God.",
                    "Peace to thee, {other}. How may I help thy soul?",
                ],
                "then": "FATHER_ASKS", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # Feud & Rivalry
    # -----------------------------------------------------------------
    {
        "name": "RIVAL_END",
        "act": "Remark",
        "concepts": ["feud"],
        "calls": [
            "We are done here.",
            "I have looked upon thy face long enough.",
            "Take thy shadow off my land.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Aye. Done.",
                    "Until we meet at the boundary ditch.",
                    "Gladly. The sight of thee sours my stomach.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "RIVAL_BITE",
        "act": "Remark",
        "concepts": ["feud"],
        "calls": [
            "Make me.",
            "Come closer and say that to my beard.",
            "Thou'rt all mouth and no marrow.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 0.8, "bias": [],
                "act": "Remark", "concepts": ["feud"],
                "lines": [
                    "Not worth the breath or the bruise.",
                    "I'll settle this before the magistrate, not in the dirt.",
                ],
                "then": "RIVAL_END", "end": None
            },
            {
                "criteria": [], "weight": 0.6, "bias": [("Hotheaded", 4.0)],
                "act": "PushBack", "concepts": ["feud"],
                "lines": [
                    "Cur! Thou'lt regret those words.",
                    "Keep thy fists to thyself before I break them.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
    {
        "name": "RIVAL_SNIPE",
        "act": "Remark",
        "concepts": ["feud"],
        "calls": [
            "Heard about thy boundary fence. Shoddy timber, that.",
            "Thy lot still owes mine three groats, whatever lies thou tellest.",
            "Keep thy lean cows out of our clover pasture.",
            "I saw where thy cart rutted the lane by my ditch.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "PushBack", "concepts": ["feud"],
                "lines": [
                    "My fence stands straighter than thy crooked word.",
                    "We paid that debt at Michaelmas, as the reeve well knows.",
                    "Thy pasture is mostly thistles anyway.",
                ],
                "then": "RIVAL_END", "end": None
            },
            {
                "criteria": [], "weight": 0.8, "bias": [("Hotheaded", 3.0)],
                "act": "PushBack", "concepts": ["feud"],
                "lines": [
                    "Say that again, if thou darest.",
                    "Thou hast a treacherous tongue in thy skull.",
                ],
                "then": "RIVAL_BITE", "end": None
            }
        ]
    },
    {
        "name": "RIVAL_GREET",
        "act": "Greet",
        "concepts": ["greeting", "feud"],
        "calls": [
            "You.",
            "Look what the crows dropped on the highway.",
            "{other}. Still walking upright, I see.",
            "What business brings thy ugly mug across my path?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["feud"],
                "lines": [
                    "{other}.",
                    "I walk where I please in this parish.",
                    "Save thy bile for someone who fears it.",
                ],
                "then": "RIVAL_SNIPE", "end": None
            },
            {
                "criteria": [], "weight": 0.8, "bias": [("Hotheaded", 3.0)],
                "act": "PushBack", "concepts": ["feud"],
                "lines": [
                    "Still breathing, I see. A great pity for the town.",
                    "The years have only made thee sourer.",
                ],
                "then": "RIVAL_BITE", "end": None
            },
            {
                "criteria": [], "weight": 0.5, "bias": [("Proud", 2.0)],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "I have nothing to say to thee, scoundrel.",
                    "Out of my way. I don't parley with knaves.",
                ],
                "then": None, "end": "WalksOff"
            }
        ]
    },
]
print(f"Loaded {len(MORE_DATA)} more exchanges.")
