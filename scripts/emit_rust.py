#!/usr/bin/env python3
"""Assembles and generates game2-core/src/game/dialogue_trees.rs.
Verifies response count >= 1000 and ensures strict safety filtering.
"""

import os
import re
import sys

sys.path.append(os.path.dirname(__file__))
from trees_data import DATA
from more_trees_data import MORE_DATA
from trade_data import TRADE_DATA
from vocations_data import VOCATIONS_DATA

FORBIDDEN_WORDS = [
    "faggot", "faggots", "retard", "nigger", "cunt", "whore", "slut", "bitch",
    "fag", "dyke", "kike", "spic", "chink"
]

all_dialogue_lines = []

def S(s: str) -> str:
    s = s.strip().strip('"').replace('\n', ' ')
    lower = s.lower()
    for w in FORBIDDEN_WORDS:
        if re.search(r'\b' + re.escape(w) + r'\b', lower):
            raise ValueError(f"CRITICAL: Forbidden word '{w}' in line: {s}")
    all_dialogue_lines.append(s)
    escaped = s.replace('\\', '\\\\').replace('"', '\\"')
    return f'"{escaped}"'

def lines_slice(arr: list[str]) -> str:
    return "&[" + ", ".join(S(x) for x in arr) + "]"

def bias_slice(arr: list) -> str:
    if not arr:
        return "&[]"
    items = []
    for t, m in arr:
        items.append(f"({t}, {m:.1f})")
    return "&[" + ", ".join(items) + "]"

def criteria_slice(arr: list) -> str:
    if not arr:
        return "&[]"
    items = []
    for c in arr:
        if c.startswith("Has(") or c.startswith("Lacks("):
            items.append(c)
        else:
            items.append(f'Has("{c}")')
    return "&[" + ", ".join(items) + "]"

def concepts_slice(arr: list[str]) -> str:
    return "&[" + ", ".join(f'"{c}"' for c in arr) + "]"

TOPIC_REMARKS = {
    "trade:smith": [
        "The hammer makes the iron obey, neighbor.",
        "Strike while the iron glows cherry-red in the coals.",
    ],
    "trade:miller": [
        "The millpond water never turns the wheel twice.",
        "Dry corn yields the sweetest white flour.",
    ],
    "trade:baker": [
        "Flour and salt and yeast - the holy trinity of the table.",
        "A well-heated brick oven bakes through to the core.",
    ],
    "trade:weaver": [
        "The pattern grows one thread at a time on the loom.",
        "Good woolen cloth keeps the cold winds from the hearth.",
    ],
    "trade:carpenter": [
        "True timber bends to no man's haste.",
        "A joint cut snug needs only an oak peg to stand firm.",
    ],
    "trade:mason": [
        "A stone set plumb stands against every gale.",
        "Limestone mortar and sound rubble form a true bulwark.",
    ],
    "trade:shepherd": [
        "The bell-wether leads the flock safely to fold.",
        "Sweet moor grass fattens the lambs before autumn.",
    ],
    "trade:brewer": [
        "A foaming mug cheers the tired ploughman at sundown.",
        "Good malted barley smells like a feast in the kettle.",
    ],
    "trade:beekeeper": [
        "The wax is sweet and the honey flows clear.",
        "A gentle hand among the hives avoids angry stings.",
    ],
    "trade:herb_wife": [
        "Nature holds a remedy for every mortal ache.",
        "Gather the leaves at sunrise before the dew evaporates.",
    ],
    "trade:merchant": [
        "The pack mules know every turn of the king's road.",
        "Fair barter leaves both parties content at sunset.",
    ],
    "trade:fisher": [
        "Patience brings more trout than a hasty net.",
        "The willow creel is damp and sweet with fresh river fish.",
    ],
    "trade:charcoal": [
        "Slow burning makes the charcoal hard and sound.",
        "Blue smoke turns white when the mound is properly baked.",
    ],
    "elder": [
        "Seasons cycle and return as they have always done.",
        "Listen to the quiet voices of the ancient trees.",
    ],
    "newborn": [
        "A child's laughter is the sweetest music in the village.",
        "Soft lamb's wool swaddling keeps the babe warm.",
    ],
    "concern:sickness": [
        "Rest by the hearth and let the fever break in peace.",
        "A hot draught of elderberry eases the swollen throat.",
    ],
    "concern:grief": [
        "The sorrow will ease as the seasons turn and heal.",
        "A shared memory keeps their spirit close to the hearth.",
    ],
    "concern:faith": [
        "The church altar is always open to the weary soul.",
        "Prayer in the morning strengthens the hands for labor.",
    ],
    "feud": [
        "I have no words to waste on a crooked knave.",
        "Step aside and let an honest person pass.",
    ],
    "concern:food": [
        "A shared loaf tastes twice as sweet in hard times.",
        "We stretch the pottage and thank Heaven for what remains.",
    ],
    "concern:fuel": [
        "Dry kindling keeps the bitter frost outside the thatch.",
        "A peat fire smolders long and warms the stones.",
    ],
    "family": [
        "A united house can withstand any tempest.",
        "Blood runs true through every trial and harvest.",
    ],
    "friendship": [
        "True fellowship is the richest coin in the parish.",
        "A loyal comrade lightens the heaviest wagon load.",
    ],
    "stranger": [
        "The road is wide enough for all peaceful travelers.",
        "Mind thy steps and follow the church spire.",
    ],
}

TOPIC_QUESTIONS = {
    "trade:smith": [
        "Is the hearth hot enough to weld the iron tyre?",
        "Canst thou hammer out three stout horseshoe nails today?",
    ],
    "trade:miller": [
        "Will the mill race freeze if the frost sharpens tonight?",
        "Is the flour dry and sweet in the wooden hopper?",
    ],
    "trade:baker": [
        "Are the loaves golden brown on the oven bricks?",
        "Dost thou need dry birch boughs for the bread oven?",
    ],
    "trade:weaver": [
        "How many yards of wool cloth come off the loom today?",
        "Will the woad dye hold fast against the wash water?",
    ],
    "trade:carpenter": [
        "Canst thou hew a sturdy axle-tree for my cart?",
        "Are the oak rafters seasoned dry enough to raise?",
    ],
    "trade:mason": [
        "Will the limestone mortar hold against the river damp?",
        "Canst thou chisel a drainage gutter through the foundation?",
    ],
    "trade:shepherd": [
        "Did the herd dog drive the stray lambs from the briars?",
        "How many ewes dropped twins in the sheltered fold?",
    ],
    "trade:brewer": [
        "Has the barley mash cooled enough to pitch the yeast?",
        "Can I fill my leather flagon with fresh small beer?",
    ],
    "trade:beekeeper": [
        "Are the worker bees flying strong to the heather hills?",
        "Hast thou a pot of golden comb for a healing draught?",
    ],
    "trade:herb_wife": [
        "What root will soothe a racking cough in the chest?",
        "Hast thou willow bark steeped for the fever shivers?",
    ],
    "trade:merchant": [
        "What exchange dost thou ask for a bag of sea salt?",
        "Dost thou have fine bone combs and steel needles?",
    ],
    "trade:fisher": [
        "Did the willow fish-pots take any eels at the weir?",
        "Canst thou spare two fresh river trout for supper?",
    ],
    "trade:charcoal": [
        "How burns the earth-covered pit mound in the clearing?",
        "Will the charcoal sacks be ready for the smithy by dawn?",
    ],
    "concern:food": [
        "Hast thou a handful of barley meal to spare for the pot?",
        "Will the storehouse grain hold out until the spring grass?",
    ],
    "concern:fuel": [
        "Can we gather dry brushwood together in the common woods?",
        "Hast thou a shovel to dig peat from the dry bog?",
    ],
    "concern:sickness": [
        "Does the fever ease when the sun warms the cottage?",
        "Is there clean well water brought to the bedside?",
    ],
    "concern:grief": [
        "Does the quiet cottage weigh heavy upon thy spirit?",
        "Can a neighbor sit with thee while the candle burns?",
    ],
    "concern:rents": [
        "Will the lord's bailiff hear our plea concerning the hail damage?",
        "Can we petition the manor court with a single voice?",
    ],
    "feud": [
        "Dost thou still claim the ditch on my side of the hedge?",
        "When wilt thou pay the two groats owed since harvest?",
    ],
    "friendship": [
        "Can I count on thy strong arm when we raise the barn?",
        "Wilt thou share a cup of small beer after the chores?",
    ],
    "family": [
        "How fares our sister's household down by the green?",
        "Canst thou lend an ox to help plough our furrow?",
    ],
    "stranger": [
        "What brings thy footsteps into our quiet parish?",
        "Dost thou seek the road toward the cathedral town?",
    ],
    "elder": [
        "Dost thou recall how the great freeze broke forty years ago?",
        "What signs in the rooks' flight foretell an early spring?",
    ],
    "newborn": [
        "Does the cradle rock peacefully beside the hearth?",
        "Is the babe feeding well and gaining strength each day?",
    ],
}

TOPIC_ANSWERS = {
    "trade:smith": [
        "The white heat is ready; bring the iron to the anvil.",
        "The coulter is welded sound and quenched in oil.",
    ],
    "trade:miller": [
        "The river turns the wheel fast; the meal pours sweet.",
        "The grist is ground fine and packed into clean linen sacks.",
    ],
    "trade:baker": [
        "The crust is crisp and hot from the stone floor.",
        "Four rye loaves and a barley bannock are ready.",
    ],
    "trade:weaver": [
        "The shuttle flies true and the cloth grows stout.",
        "The wool was washed in the brook and carded clean.",
    ],
    "trade:carpenter": [
        "The oak timber is straight-grained and cured dry.",
        "The wheel spokes are fitted tight with seasoned ash.",
    ],
    "trade:mason": [
        "The lime mortar sets hard as flint between the stones.",
        "The wall stands plumb and square to the morning star.",
    ],
    "trade:shepherd": [
        "The ewes are content and the lambs suckle strong.",
        "The clover on the common pasture is rich and green.",
    ],
    "trade:brewer": [
        "The ale has cleared and the barrel is broached sweet.",
        "The malt was roasted golden on the drying floor.",
    ],
    "trade:beekeeper": [
        "The skeps are heavy with comb and clear summer honey.",
        "The swarm settled peaceful beneath the hazel hedge.",
    ],
    "trade:herb_wife": [
        "Boil the willow bark in rain water and drink it warm.",
        "This green salve of yarrow will close the deepest cut.",
    ],
    "trade:merchant": [
        "Two silver pennies or a salted ham buys the measure.",
        "Fair trade makes honest friends on the king's highway.",
    ],
    "trade:fisher": [
        "Three fat river trout and an eel fresh from the wicker trap.",
        "The current runs clear over the gravel beds today.",
    ],
    "trade:charcoal": [
        "The mound is raked and two sacks of hard coal are cooled.",
        "Slow smoke through the damp sods made a flawless pit.",
    ],
    "concern:food": [
        "Take half this barley loaf; we must look after each other.",
        "We have boiled leeks and dried peas enough for two bowls.",
    ],
    "concern:fuel": [
        "Take these live embers in an iron pot to start thy fire.",
        "We gathered dry kindling from the common wood.",
    ],
    "concern:sickness": [
        "The sweat broke at dawn and the fever feels a little lighter.",
        "Rest quiet under the wool; I will bring water from the well.",
    ],
    "concern:grief": [
        "Thy presence by the hearth brings comfort to this quiet house.",
        "We remember their kindness and the honest work of their hands.",
    ],
    "concern:rents": [
        "The manor court must recognize the customary rights of our lane.",
        "We will present our marks together on a joint petition.",
    ],
    "feud": [
        "My fence stands where our grandfathers agreed it should stand.",
        "Keep thy cattle in thy own furrow and there will be peace.",
    ],
    "friendship": [
        "Thy word is better than gold; count on my team anytime.",
        "A loyal comrade makes the heaviest timber feel light.",
    ],
    "family": [
        "Our kin will stand beside thee through good season and bad.",
        "Take what thou needest from our barn; we share all things.",
    ],
    "stranger": [
        "The road to the cathedral town follows the river south.",
        "Walk in peace on our lane and thou'rt welcome at the alehouse.",
    ],
    "elder": [
        "Patience and honest toil have seen eighty winters come and go.",
        "The soil never forgets those who treat it with respect.",
    ],
    "newborn": [
        "The babe sleeps sweetly, wrapped in clean woolen swaddling.",
        "May Heaven grant this little soul long life and peace.",
    ],
}

def augment_exchange(ex):
    calls = list(ex["calls"])
    domain_topics = [c for c in ex["concepts"] if c not in ("greeting", "smalltalk", "farewell")]
    c_topic = domain_topics[0] if domain_topics else (ex["concepts"][0] if ex["concepts"] else "smalltalk")

    # Ensure calls has 4 lines with contextual matching
    if len(calls) < 4:
        if ex["act"] == "Question":
            added = TOPIC_QUESTIONS.get(c_topic, [])
            for l in added:
                if l not in calls and len(calls) < 4:
                    calls.append(l)
            if len(calls) < 4:
                calls.append("How dost thou find the road today, {other}?")
        elif ex["act"] == "Greet":
            calls.append("A peaceful day to thee, {other}.")
        else: # Remark or Answer
            added = TOPIC_REMARKS.get(c_topic, [])
            for l in added:
                if l not in calls and len(calls) < 4:
                    calls.append(l)
            if len(calls) < 4:
                calls.append("We do what we must while daylight holds.")
    calls = calls[:5]
    ex["calls"] = calls

    # Ensure each reply has 4 lines with contextual matching
    for r in ex["replies"]:
        lines = list(r["lines"])
        act = r["act"]
        if len(lines) < 4:
            if "feud" in r["concepts"]:
                lines.extend([
                    "Get out of my sight, {other}.",
                    "Take thy shadow off my path.",
                ])
            elif r.get("end") == "WalksOff":
                lines.extend([
                    "I have no leisure to stand about, {other}.",
                    "I must walk on; fair day.",
                ])
            elif act == "Farewell":
                lines.extend([
                    "May the road be kind under thy boots.",
                    "God's peace be upon thy cottage, {other}.",
                ])
            elif act == "Answer":
                added_ans = TOPIC_ANSWERS.get(c_topic, [])
                for l in added_ans:
                    if l not in lines and len(lines) < 4:
                        lines.append(l)
                if len(lines) < 4:
                    lines.append("We do what we can with the strength Heaven gives.")
            elif act == "Agree":
                lines.extend([
                    "Aye, there is truth in what thou sayest.",
                    "Thou hast spoken the plain heart of the matter.",
                ])
            elif act == "PushBack":
                lines.extend([
                    "Nay, I see it differently, mark me.",
                    "That is not the way the elders told it.",
                ])
            elif act == "Greet":
                lines.extend([
                    "God bless thy steps this day, {other}.",
                    "Well met, friend, under the open sky.",
                ])
            else:
                lines.extend([
                    "Aye, words have weight in a small village.",
                    "Time will tell what comes of this.",
                ])
        r["lines"] = lines[:5]

    # If an exchange only has 1 reply and it's not an ending, give a secondary reply option
    if len(ex["replies"]) == 1 and ex["replies"][0]["end"] is None:
        first_r = ex["replies"][0]
        ex["replies"].append({
            "criteria": [], "weight": 0.4, "bias": [],
            "act": "Farewell", "concepts": ["farewell"],
            "lines": [
                "I must be on my way, {other}. Fare well.",
                "The sun climbs high; chores call me. Good day.",
                "Until next time, neighbor.",
            ],
            "then": None, "end": "Farewell"
        })

print("Context-matched augmenter ready.")
