# Part 4: All 14 Vocations, Elder Wisdom, Newborn, and 10 Meeting Pools

VOCATIONS_DATA = [
    # -----------------------------------------------------------------
    # 1. Smith & Forge
    # -----------------------------------------------------------------
    {
        "name": "SMITH_IRON",
        "act": "Remark",
        "concepts": ["trade:smith"],
        "calls": [
            "Good Spanish iron rings clear; bad slag thuds like lead.",
            "A hammer needs an eye that sees the bend before it strikes.",
            "The quench tub hisses when the blade has its proper temper.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Keep the fire roaring, smith. Good day.",
                    "May thy hammer never miss its mark. Fair day.",
                    "I will return for the coulter at sunset.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SMITH_WORK",
        "act": "Question",
        "concepts": ["trade:smith"],
        "calls": [
            "Canst thou forge three dozen horseshoe nails by tomorrow?",
            "My plough coulter struck a buried boulder and snapped clean.",
            "Dost thou have charcoal enough to heat the heavy anvil?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:smith"],
                "lines": [
                    "Bring me two groats of coal and the nails will be ready by dawn.",
                    "Lay the broken iron on the bench; I'll weld the seam.",
                    "Aye, the charcoal burner delivered two sacks from the pit yesterday.",
                ],
                "then": "SMITH_IRON", "end": None
            }
        ]
    },
    {
        "name": "SMITH_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:smith"],
        "calls": [
            "Good morrow, smith! The sparks are flying bright today.",
            "Well met, master smith! The sound of thy forge wakes the village.",
            "Hail to the forge, {other}! Sweating by the bellows already?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:smith"],
                "lines": [
                    "Morrow, {other}! The coals are white-hot and hungry for iron.",
                    "Well met! Step inside out of the wind, mind the sparks.",
                    "Aye, iron waits for no man's leisure. What needest thou?",
                ],
                "then": "SMITH_WORK", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 2. Miller & Grain
    # -----------------------------------------------------------------
    {
        "name": "MILLER_TOLL",
        "act": "Remark",
        "concepts": ["trade:miller"],
        "calls": [
            "One scoop in twenty is the customary multure, no more, no less.",
            "The millstones were freshly dressed with the chisel last Pentecost.",
            "Fine white flour for white loaves, coarse bran for the byre.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Keep the wheel turning, miller. Fair day.",
                    "Mind thou dost not spill my meal on the floorboards.",
                    "Until harvest time, master of the water.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "MILLER_GRIST",
        "act": "Question",
        "concepts": ["trade:miller"],
        "calls": [
            "Is the river running high enough to turn the big wheel?",
            "Canst thou grind my two sacks of barley before noon?",
            "Did the damp get into the meal hopper after the downpour?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:miller"],
                "lines": [
                    "The millpond is full to the weir boards; she turns swift today.",
                    "Stack thy sacks by the scale; thy meal will be ground by mid-day.",
                    "The oak bins are dry as bone; not a drop touched the flour.",
                ],
                "then": "MILLER_TOLL", "end": None
            }
        ]
    },
    {
        "name": "MILLER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:miller"],
        "calls": [
            "Good morrow, miller! The roar of thy weir echoes down the valley.",
            "Well met, {other}! Flour dusts thy coat like fresh snow.",
            "Hail to the mill, {other}! Is the grist running smooth?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:miller"],
                "lines": [
                    "Morrow, neighbor! The stones rumble and the water runs free.",
                    "Well met! Unload thy cart; the hopper is ready.",
                    "Aye, dust in my hair and bread in my belly! What grist bringest thou?",
                ],
                "then": "MILLER_GRIST", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 3. Baker & Bread
    # -----------------------------------------------------------------
    {
        "name": "BAKER_CRUST",
        "act": "Remark",
        "concepts": ["trade:baker"],
        "calls": [
            "A well-swept oven floor gives the sweetest crust in the parish.",
            "Yeast from the brewer's ale makes the dough rise twice as high.",
            "Golden loaves on the wooden board - the glory of the harvest.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "The smell of thy loaves is a blessing to the lane. Fare well.",
                    "May thy oven never grow cold, baker.",
                    "I take my warm crust home to the children. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "BAKER_BREAD",
        "act": "Question",
        "concepts": ["trade:baker"],
        "calls": [
            "Are the morning rye loaves ready to pull with the peel?",
            "Canst thou bake my barley dough in thy community oven?",
            "Hast thou any day-old trenchers left for a penny?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:baker"],
                "lines": [
                    "Aye, hot from the bricks and brown as a hazelnut!",
                    "Bring thy dough before noon; there is room beside the landlord's pies.",
                    "Take four trenchers; they are crisp and good for soaking broth.",
                ],
                "then": "BAKER_CRUST", "end": None
            }
        ]
    },
    {
        "name": "BAKER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:baker"],
        "calls": [
            "Good morrow, baker! Thy chimney breathes the sweetest air in town.",
            "Well met, {other}! The smell of fresh crust drew me from the lane.",
            "Hail, master of the peel! Has the first batch risen?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:baker"],
                "lines": [
                    "Morrow, {other}! Flour on my hands and fire in the hearth.",
                    "Well met! The oven is blazing hot and the loaves are baking.",
                    "A warm welcome to thee! Step in and feel the heat.",
                ],
                "then": "BAKER_BREAD", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 4. Weaver & Cloth
    # -----------------------------------------------------------------
    {
        "name": "WEAVER_CLOTH",
        "act": "Remark",
        "concepts": ["trade:weaver"],
        "calls": [
            "Tight warp and even weft make a cloth that sheds the rain.",
            "Madder root gives a brave red, but woad blues last thirty years.",
            "A warm woolen blanket keeps the winter chills from the cradle.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy shuttle fly true and never break a thread.",
                    "I will return when the piece comes off the frame. Fare well.",
                    "Good cloth is true wealth. Good day, weaver.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "WEAVER_FLEECE",
        "act": "Question",
        "concepts": ["trade:weaver"],
        "calls": [
            "Did the sheep shearers bring in the upland wool yet?",
            "Canst thou weave a yard of heavy russet for my work kirtle?",
            "Is the spun yarn dry enough to wind onto the spools?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:weaver"],
                "lines": [
                    "Ten sacks of greasy fleece sit by the washing trough.",
                    "I'll set the warp tomorrow; russet is sturdy and warm for field work.",
                    "Aye, the carders combed it fine and the spindles are humming.",
                ],
                "then": "WEAVER_CLOTH", "end": None
            }
        ]
    },
    {
        "name": "WEAVER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:weaver"],
        "calls": [
            "Good morrow, weaver! The click-clack of thy loom is steady today.",
            "Well met, {other}! Surrounded by skeins of dyed wool, I see.",
            "Hail to the loom, neighbor! How runs the shuttle this morning?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:weaver"],
                "lines": [
                    "Morrow, {other}! Counting the threads from dawn till twilight.",
                    "Well met! The treadles are flying and the cloth grows fast.",
                    "Greetings! A weaver's work is never done while sheep have wool.",
                ],
                "then": "WEAVER_FLEECE", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 5. Carpenter & Wood
    # -----------------------------------------------------------------
    {
        "name": "CARPENTER_PEGS",
        "act": "Remark",
        "concepts": ["trade:carpenter"],
        "calls": [
            "Heart of oak and seasoned ash will outlast three generations.",
            "An auger hole drilled straight needs only a split hazel peg to hold.",
            "Cedar smells sweet in a chest, but elm makes the stoutest wheel hubs.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy adze stay sharp, carpenter. Fare well.",
                    "I will look for the cart axle on Friday. Good day.",
                    "Stout timber makes a sturdy home. Walk in peace.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CARPENTER_BEAMS",
        "act": "Question",
        "concepts": ["trade:carpenter"],
        "calls": [
            "Canst thou mortise a tie-beam for the cottage roof ridge?",
            "Didst thou square the timber logs felled in the lord's wood?",
            "Dost thou have spare dry pegs to secure the wattle hurdle?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:carpenter"],
                "lines": [
                    "I have seasoned oak ready on the trestles; it will bear the rafters.",
                    "The broadaxe squared two dozen beams; they are stacked to cure.",
                    "Take a handful of dried oak pins from the tub by the bench.",
                ],
                "then": "CARPENTER_PEGS", "end": None
            }
        ]
    },
    {
        "name": "CARPENTER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:carpenter"],
        "calls": [
            "Good morrow, carpenter! Shavings and sawdust fly thick today.",
            "Well met, {other}! The scent of fresh-cut pine is in the lane.",
            "Hail to the woodwright! Has the new timber arrived from the coppice?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:carpenter"],
                "lines": [
                    "Morrow, neighbor! Measuring twice and cutting once, as ever.",
                    "Well met! The saw pit was busy since the dawn bell.",
                    "Greetings! A stout tree fell yesterday; fine timber for building.",
                ],
                "then": "CARPENTER_BEAMS", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 6. Mason & Stone
    # -----------------------------------------------------------------
    {
        "name": "MASON_LIME",
        "act": "Remark",
        "concepts": ["trade:mason"],
        "calls": [
            "Hot slaked lime and sharp sand bond tighter than the quarry itself.",
            "A drystone foundation keeps the rot out of the lowest timbers.",
            "Granite breaks the chisel, but limestone yields to an honest stroke.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy plumb-line hang true, mason. Fair day.",
                    "Stone stands when all else fades. Farewell.",
                    "I will bring the ox-cart for the rubble tomorrow.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "MASON_WALL",
        "act": "Question",
        "concepts": ["trade:mason"],
        "calls": [
            "Canst thou dress ashlar stones for the new church window?",
            "Did the quarry deliver enough rubble stone to rebuild the wall?",
            "Will the lime mortar set before the autumn rains begin?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:mason"],
                "lines": [
                    "I am chiseling the jambs now; fine freestone from the quarry.",
                    "Two heavy wagonloads came down the hill this morning.",
                    "It will cure rock-hard if we keep the frost off it for a fortnight.",
                ],
                "then": "MASON_LIME", "end": None
            }
        ]
    },
    {
        "name": "MASON_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:mason"],
        "calls": [
            "Good morrow, mason! The ringing of thy hammer on stone echoes far.",
            "Well met, {other}! White rock dust on thy leather apron.",
            "Hail to the stone-cutter! Setting foundations today?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:mason"],
                "lines": [
                    "Morrow, {other}! Stone by stone the hall rises from the earth.",
                    "Well met! Testing the level and squaring the corners.",
                    "Greetings! A heavy trade, but a wall built right outlasts empires.",
                ],
                "then": "MASON_WALL", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 7. Shepherd & Fold
    # -----------------------------------------------------------------
    {
        "name": "SHEPHERD_PASTURE",
        "act": "Remark",
        "concepts": ["trade:shepherd"],
        "calls": [
            "The upland turf is sweet and short; the ewes grow fat on it.",
            "The sheepfold dung will enrich the barley fields when autumn comes.",
            "A good dog is worth three hired hands on the open hills.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Keep thy eye on the brushwood, shepherd. Fair day.",
                    "May the flock prosper and the wolves stay distant.",
                    "Good grazing to thy flock, neighbor.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "SHEPHERD_EWES",
        "act": "Question",
        "concepts": ["trade:shepherd"],
        "calls": [
            "How many twin lambs dropped in the fold last night?",
            "Did the wolf from the high ridge trouble the sheepfold?",
            "Is it time to drive the flock down to the river wash?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:shepherd"],
                "lines": [
                    "Six fine lambs, all suckling strong by their mothers.",
                    "The hound barked at midnight and drove the beast away.",
                    "By next full moon the fleece will be clean for the shearers.",
                ],
                "then": "SHEPHERD_PASTURE", "end": None
            }
        ]
    },
    {
        "name": "SHEPHERD_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:shepherd"],
        "calls": [
            "Good morrow, shepherd! Bleating ewes fill the morning air.",
            "Well met, {other}! Cloaked in wool and leaning on thy crook.",
            "Hail, drover of the flock! How fare the sheep on the commons?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:shepherd"],
                "lines": [
                    "Morrow, neighbor! Watching for hawks and counting tails.",
                    "Well met! The bells on the wethers make a pleasant music.",
                    "Greetings! Wind on the hill, but the ewes are peaceful.",
                ],
                "then": "SHEPHERD_EWES", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 8. Brewer & Ale
    # -----------------------------------------------------------------
    {
        "name": "BREWER_CUP",
        "act": "Remark",
        "concepts": ["trade:brewer"],
        "calls": [
            "Clear spring water and sound malt make an ale fit for a baron.",
            "An ale-stake hung with green boughs means a fresh barrel is tapped.",
            "A tankard at sundown washes the harvest dust from the throat.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy vats never sour, brewer. Fair day.",
                    "I will bring my pewter mug at sundown. Farewell.",
                    "A good brew makes neighbors into brothers. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "BREWER_MASH",
        "act": "Question",
        "concepts": ["trade:brewer"],
        "calls": [
            "Is the new brewing of October ale ready for the tap?",
            "Did the maltsters bring in the sprouted barley from the floor?",
            "Dost thou have small beer to sell for the field workers' pails?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:brewer"],
                "lines": [
                    "It has fermented sweet and strong; we broach the cask today.",
                    "Three carts of roasted malt came down from the kiln yesterday.",
                    "Aye, fill thy pails; small beer keeps the reapers from fainting.",
                ],
                "then": "BREWER_CUP", "end": None
            }
        ]
    },
    {
        "name": "BREWER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:brewer"],
        "calls": [
            "Good morrow, brewer! Warm steam rises over thy brew-house.",
            "Well met, {other}! The rich scent of mash drifts through the lane.",
            "Hail to the tapster! Tapping a fresh barrel this morning?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:brewer"],
                "lines": [
                    "Morrow, {other}! Stirring the copper vat since before cockcrow.",
                    "Well met! The yeast is working its miracle in the vats.",
                    "Greetings! A thirsty throat finds a ready welcome here.",
                ],
                "then": "BREWER_MASH", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 9. Beekeeper & Apiary
    # -----------------------------------------------------------------
    {
        "name": "BEEKEEPER_SWARM",
        "act": "Remark",
        "concepts": ["trade:beekeeper"],
        "calls": [
            "Sweet yellow wax makes the church tapers that burn without smoke.",
            "The clover blossoms give the clearest honey in the valley.",
            "Bees love a quiet tongue and a gentle step among the skeps.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy hives thrive through the winter, beekeeper. Fair day.",
                    "I will buy a jar of comb at Michaelmas. Farewell.",
                    "The little workers teach us all to labor together. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "BEEKEEPER_HONEY",
        "act": "Question",
        "concepts": ["trade:beekeeper"],
        "calls": [
            "Did the swarm from the orchard settle in the new straw skep?",
            "Can I buy a pot of honey to soothe my child's raw throat?",
            "Hast thou cakes of beeswax ready for the shoemaker's thread?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:beekeeper"],
                "lines": [
                    "Aye, we beat the brass pan and hived them safely under the hedge.",
                    "Take this jar of dark thyme honey; it mends a cough overnight.",
                    "Three cakes of yellow wax, filtered clean and smelling of clover.",
                ],
                "then": "BEEKEEPER_SWARM", "end": None
            }
        ]
    },
    {
        "name": "BEEKEEPER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:beekeeper"],
        "calls": [
            "Good morrow, beekeeper! The apiary hums like an organ in the sunshine.",
            "Well met, {other}! Straw skeps standing tidy in the garden rows.",
            "Hail, master of the hives! Smoking the bees this morning?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:beekeeper"],
                "lines": [
                    "Morrow, {other}! The bees are flying thick to the heather hills.",
                    "Well met! Sweet honey on the comb and wax for the monastery.",
                    "Greetings! Walk soft, neighbor; the queen is in her prime.",
                ],
                "then": "BEEKEEPER_HONEY", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 10. Herb-wife & Apothecary
    # -----------------------------------------------------------------
    {
        "name": "HERB_SALVE",
        "act": "Remark",
        "concepts": ["trade:herb_wife"],
        "calls": [
            "Dried elder flowers and mint leaves keep the ague away in winter.",
            "Yarrow leaves stop a blood-cut faster than cobwebs.",
            "The roots must be gathered in the dark of the moon for full virtue.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Thy knowledge has saved many a soul in this town. Fare well.",
                    "I will brew the tea as thou biddest. Good day.",
                    "May the earth yield its medicine to thee. Farewell.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "HERB_DRAUGHT",
        "act": "Question",
        "concepts": ["trade:herb_wife"],
        "calls": [
            "Hast thou a salve for stiff knees and swollen knuckles?",
            "Canst thou brew a draught for the fever that shakes the child?",
            "What herb will keep the pests away from our flour bin?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:herb_wife"],
                "lines": [
                    "Rub this grease rendered with comfrey and marigold twice a day.",
                    "Take this dried willow bark; steep it in hot water and give three sips.",
                    "Hang dried tansy and wormwood above the bin; no weevil will touch it.",
                ],
                "then": "HERB_SALVE", "end": None
            }
        ]
    },
    {
        "name": "HERB_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:herb_wife"],
        "calls": [
            "Good morrow, wise woman {other}! Bunches of herbs hang drying everywhere.",
            "Well met, herb-wife! Thy basket is full of green leaves from the fen.",
            "Hail, healer of the sick! Gathering roots in the dew?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:herb_wife"],
                "lines": [
                    "Morrow, {other}! The earth gives her secrets to those who look.",
                    "Well met! Stinging nettle and plantain leaves gathered at sunrise.",
                    "Greetings, neighbor! What ails thee or thy folk today?",
                ],
                "then": "HERB_DRAUGHT", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 11. Merchant & Road
    # -----------------------------------------------------------------
    {
        "name": "MERCHANT_BARTER",
        "act": "Remark",
        "concepts": ["trade:merchant"],
        "calls": [
            "A good trade leaves both men smiling when hands are struck.",
            "Silver coin or salted hams - sound goods are always welcome.",
            "The road is long, but fair trade makes every mile worthwhile.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Fair journey on the highway, trader. Good day.",
                    "May thy pack horses stay sure-footed on the ridge.",
                    "Until the spring fair, merchant. Fare well.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "MERCHANT_WARES",
        "act": "Question",
        "concepts": ["trade:merchant"],
        "calls": [
            "Dost thou carry steel sewing needles and linen thread from the city?",
            "What price dost thou ask for a pound of sea salt?",
            "Hast thou any ribbons of silk or spice from overseas?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:merchant"],
                "lines": [
                    "Fine iron needles in a bone case; two eggs or a penny each.",
                    "White sea salt from the coast; three groats for a full measure.",
                    "Scarlet ribbon from Flanders and a pinch of black pepper to trade.",
                ],
                "then": "MERCHANT_BARTER", "end": None
            }
        ]
    },
    {
        "name": "MERCHANT_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:merchant"],
        "calls": [
            "Good morrow, trader! Thy pack mules are laden heavy from the highway.",
            "Well met, merchant! What exotic wares bringest thou to our valley?",
            "Hail to the traveling merchant! How stood the roads through the forest?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:merchant"],
                "lines": [
                    "Morrow, good folk! Goods from the sea ports and goods from the hills.",
                    "Well met! Unbuckling the leather packs; step up and see the wares.",
                    "Greetings! Mud on my boots, but treasures in my packs for honest barter.",
                ],
                "then": "MERCHANT_WARES", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 12. Lord & Manor Leases
    # -----------------------------------------------------------------
    {
        "name": "LORD_PETITION",
        "act": "Remark",
        "concepts": ["concern:rents"],
        "calls": [
            "The manor must maintain the bridge if the peasants are to bring in the harvest.",
            "A good lord shears his sheep but does not flay the skin.",
            "The court rolls show every customary right since the grandfather's time.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "We will see what the manor court decides on Tuesday. Good day.",
                    "May justice be remembered at the high table. Fare well.",
                    "I take my leave of the manor gate. Fair day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "LORD_LEASES",
        "act": "Question",
        "concepts": ["concern:rents"],
        "calls": [
            "Will the rent in silver be abated because the hail ruined the oats?",
            "Must our cottage provide three days of boon-ploughing this season?",
            "When will the lord's bailiff hear our plea concerning the common ditch?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["concern:rents"],
                "lines": [
                    "The steward will inspect the damaged fields before the tally is set.",
                    "Two days of plough-service are due by custom, as the parchment records.",
                    "The court sits under the great oak on Thursday morning; be present.",
                ],
                "then": "LORD_PETITION", "end": None
            }
        ]
    },
    {
        "name": "LORD_GREET",
        "act": "Greet",
        "concepts": ["greeting", "concern:rents"],
        "calls": [
            "Good morrow to the manor's steward. We come on village business.",
            "Well met, sir. We seek speech concerning the leases and tallies.",
            "Greetings, lord's man. The tenants have gathered to speak.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["concern:rents"],
                "lines": [
                    "Morrow, tenants. The tally sticks are on the table; state thy grievance.",
                    "Well met, villagers. The lord listens to lawful petitions.",
                    "Speak thy mind plainly, goodmen; the court roll is open.",
                ],
                "then": "LORD_LEASES", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 13. Fisher & River
    # -----------------------------------------------------------------
    {
        "name": "FISHER_NETS",
        "act": "Remark",
        "concepts": ["trade:fisher"],
        "calls": [
            "Trout leap in the shallows when the evening mayflies hatch.",
            "Wicker eel pots set in the weir reeds yield a fat catch by dawn.",
            "A fisherman knows every deep hole and eddy between here and the bridge.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy nets come up heavy with silver fish. Fair day.",
                    "Mind the river current by the mill dam. Farewell.",
                    "I will bring an eel pie to the festival. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "FISHER_CURRENT",
        "act": "Question",
        "concepts": ["trade:fisher"],
        "calls": [
            "Is the river water clear or muddy after the hill rains?",
            "Did the fish trap take any pike in the reed bed today?",
            "Can I buy three fresh trout for the Friday supper?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:fisher"],
                "lines": [
                    "The current runs green and sweet; the fish bite eager.",
                    "A stout pike and two dozen silver dace lie in the willow creel.",
                    "Take these three trout fresh from the river; three pennies or a loaf.",
                ],
                "then": "FISHER_NETS", "end": None
            }
        ]
    },
    {
        "name": "FISHER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:fisher"],
        "calls": [
            "Good morrow, riverman! Thy boots are wet to the knee.",
            "Well met, {other}! Mending thy linen gill-nets on the bank?",
            "Hail, fisher of the weir! Any fish in the basket today?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:fisher"],
                "lines": [
                    "Morrow, {other}! The river is awake and cold as ice.",
                    "Well met! Tying knots and checking the willow osiers.",
                    "Greetings! River water in my veins and fresh catch in the creel.",
                ],
                "then": "FISHER_CURRENT", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 14. Charcoal Burner & Pits
    # -----------------------------------------------------------------
    {
        "name": "CHARCOAL_COALS",
        "act": "Remark",
        "concepts": ["trade:charcoal"],
        "calls": [
            "Slow heat with no open flame turns dense oak into pure black coal.",
            "Black dust in the beard and lungs, but the smith's forge cannot live without us.",
            "A charcoal stack needs watching through three nights of vigil.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May thy stacks burn true without flaring. Fair day.",
                    "I will tell the smith his coals are ready. Farewell.",
                    "Watch the sparks in the dry brushwood, collier. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "CHARCOAL_SMOKE",
        "act": "Question",
        "concepts": ["trade:charcoal"],
        "calls": [
            "Is the earth-covered mound ready to open and rake?",
            "Canst thou spare a sack of charcoal for the tavern braziers?",
            "Did the night wind blow the turf cover off the kiln?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["trade:charcoal"],
                "lines": [
                    "The blue smoke has turned clear; the coal is cooled and cured.",
                    "Two sacks of hard black charcoal wait by the pit hut.",
                    "The sods held fast; not a spark escaped into the trees.",
                ],
                "then": "CHARCOAL_COALS", "end": None
            }
        ]
    },
    {
        "name": "CHARCOAL_GREET",
        "act": "Greet",
        "concepts": ["greeting", "trade:charcoal"],
        "calls": [
            "Good morrow, charcoal burner! Smoke drifts thick from the deep wood.",
            "Well met, {other}! Black as an imp from tending the pits.",
            "Hail, master of the kiln! How burns the timber mound?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["trade:charcoal"],
                "lines": [
                    "Morrow, {other}! Watching the slow smoke day and night.",
                    "Well met! Soot on my face, but clean fuel for the iron forge.",
                    "Greetings! The mound is smoking sweet under its turf blanket.",
                ],
                "then": "CHARCOAL_SMOKE", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 15. Elder Wisdom
    # -----------------------------------------------------------------
    {
        "name": "ELDER_BLESS",
        "act": "Remark",
        "concepts": ["elder"],
        "calls": [
            "Lords come and go, but the plough and the furrow remain.",
            "Listen to the land; it was here before us and will keep our graves.",
            "Kindness to thy neighbor is the best hedge against misfortune.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "Thy words are gold to my ears, elder. Fair day.",
                    "May God grant thee peace in thy old age. Farewell.",
                    "We will heed thy counsel at the village meeting. Good day.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "ELDER_WINTERS",
        "act": "Question",
        "concepts": ["elder"],
        "calls": [
            "Dost thou recall the great famine fifty winters ago, grandfather?",
            "Where did the old village boundary run before the ditch was dug?",
            "What signs tell us when the frost will finally break?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["elder"],
                "lines": [
                    "Aye, we ate wild acorns and shared what porridge there was, and survived.",
                    "The boundary ran straight from the standing oak to the weeping willow.",
                    "When the dandelions flower on the ditch bank, the frost is broken for good.",
                ],
                "then": "ELDER_BLESS", "end": None
            }
        ]
    },
    {
        "name": "ELDER_GREET",
        "act": "Greet",
        "concepts": ["greeting", "elder"],
        "calls": [
            "Good morrow to thee, venerable {other}.",
            "Well met, grandfather! Sitting in the sunny doorway to warm thy joints?",
            "Greetings, mother of the village! How fares thy health today?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["elder"],
                "lines": [
                    "Morrow, young soul. The sun feels good on eighty winters of bones.",
                    "Well met, child! Another day Heaven grants to watch the fields.",
                    "Greetings! My step is slow, but my memory is long and clear.",
                ],
                "then": "ELDER_WINTERS", "end": None
            }
        ]
    },

    # -----------------------------------------------------------------
    # 16. Newborn Cradle
    # -----------------------------------------------------------------
    {
        "name": "NEWBORN_GIFT",
        "act": "Remark",
        "concepts": ["family"],
        "calls": [
            "May the little one grow tall as an ash and hardy as an oak.",
            "A baby's cry in the cottage banishes the winter gloom.",
            "The parish grows by another soul; blessed be the hearth.",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Farewell", "concepts": ["farewell"],
                "lines": [
                    "May the saints guard the cradle day and night. Fair day.",
                    "I will bring goat's milk for the mother. Farewell.",
                    "Joy to thy house, {other}! God bless the babe.",
                ],
                "then": None, "end": "Farewell"
            }
        ]
    },
    {
        "name": "NEWBORN_JOY",
        "act": "Question",
        "concepts": ["family"],
        "calls": [
            "Is the little one sleeping peacefully in the wicker cradle?",
            "Has the parish priest set the day for the holy baptism?",
            "Does the mother have warm elderberry cordial to restore her strength?",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Answer", "concepts": ["family"],
                "lines": [
                    "Aye, wrapped in soft lamb's wool, dreaming sweet dreams.",
                    "On Sunday morning when the great church bell tolls.",
                    "The neighbors brought honey broth and fresh eggs for her recovery.",
                ],
                "then": "NEWBORN_GIFT", "end": None
            }
        ]
    },
    {
        "name": "NEWBORN_GREET",
        "act": "Greet",
        "concepts": ["greeting", "family"],
        "calls": [
            "Joy to thy household, {other}! I heard a newborn babe has arrived!",
            "Good morrow, new mother! How fares the precious child?",
            "Blessings on the cradle, {other}! What a joyous day for our lane!",
        ],
        "replies": [
            {
                "criteria": [], "weight": 1.0, "bias": [],
                "act": "Greet", "concepts": ["family"],
                "lines": [
                    "Morrow, {other}! A healthy babe, praise be to Heaven!",
                    "Well met! Exhausted, but my heart is overflowing with gratitude.",
                    "Greetings! A little girl with eyes bright as the morning sky.",
                ],
                "then": "NEWBORN_JOY", "end": None
            }
        ]
    },
]
print(f"Loaded {len(VOCATIONS_DATA)} vocations exchanges.")
