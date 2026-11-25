# Conversation System Benchmark & AI Scoring Rubric

This document contains a generated benchmark of procedurally synthesized characters
and simulated conversations across various social relations and circumstances.
An AI evaluator or reviewer should grade the conversations against the rubric below.

## Scoring Rubric (Total 100 Points)

| Category | Max Points | Evaluation Criteria |
| :--- | :---: | :--- |
| **1. Topical Coherence & Discourse Flow** | 25 | Does each reply naturally respond to the preceding discourse act (greet->greet, remark->agree/pushback, question->answer)? Do turns stay on topic without abrupt non-sequiturs? |
| **2. Character & Backstory Alignment** | 25 | Does the character's dialogue match their backstory, traits, and status? E.g., does a sick character cough or mention the fever? Does a cleric invoke blessings? Does a hothead push back? |
| **3. Specificity Cascade & Fallback** | 20 | Does the most specific circumstantial tree fire first (e.g., sickness, bereavement, rivalry), and does the conversation smoothly degrade into generic smalltalk rather than terminating abruptly? |
| **4. Exclusivity & Repetition Avoidance** | 15 | Are lines strictly non-repetitive within the same conversation? Does material exhaustion naturally conclude the dialogue without robotic looping? |
| **5. Relationship & Ending Dynamics** | 15 | Do relationships (rivals vs friends vs strangers) color the tone? Do conversations reach plausible natural closures (polite farewell, sudden walk-off, or mutual exhaustion)? |

### Grading Scale
- **90-100 (Grade A):** Superb verisimilitude, sharp character voice, seamless fallback flow.
- **80-89 (Grade B):** Strong coherence, appropriate personality cues, minor phrasing quirks.
- **70-79 (Grade C):** Functional dialogue with occasional tonal mismatch.
- **< 70 (Grade D/F):** Repetition, broken topic flow, or ignored character facts.

---

## Generated Character Roster (N = 10)

### Character #1: Aldric
- **Ideology:** clergy
- **Traits:** [Hotheaded]
- **Status:** ["wx-fair", "season:autumn", "market", "grieving"]
- **Facts:** archetype:firebrand, value:freedom, value:pleasure
- **Backstory:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.

### Character #2: Eliza
- **Ideology:** lord
- **Traits:** [Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "sick"]
- **Facts:** archetype:merrymaker, value:standing, value:custom
- **Backstory:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.

### Character #3: Bram
- **Ideology:** merchant
- **Traits:** [Hotheaded, Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "grieving"]
- **Facts:** archetype:firebrand, value:adventure, value:advancement
- **Backstory:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.

### Character #4: Corvin
- **Ideology:** lord
- **Traits:** [Proud, Pious]
- **Status:** ["wx-fair", "season:autumn", "market", "smith", "grieving"]
- **Facts:** archetype:devout, value:freedom, value:fairness
- **Backstory:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.

### Character #5: Dagny
- **Ideology:** clergy
- **Traits:** [Gregarious, Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "drunk"]
- **Facts:** archetype:tongue_of_the_well, value:care, value:fairness
- **Backstory:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.

### Character #6: Hob
- **Ideology:** clergy
- **Traits:** [Stingy, Gregarious]
- **Status:** ["wx-fair", "season:autumn", "market", "smith", "selling", "sick"]
- **Facts:** archetype:tongue_of_the_well, value:freedom, value:safety
- **Backstory:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.

### Character #7: Marga
- **Ideology:** lord
- **Traits:** [Gregarious, Kind, Ambitious]
- **Status:** ["wx-fair", "season:autumn", "market", "fisher", "selling", "sick"]
- **Facts:** archetype:soft_heart, value:safety, value:freedom
- **Backstory:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.

### Character #8: Gisela
- **Ideology:** lord
- **Traits:** [Ambitious, Lazy, Gregarious]
- **Status:** ["wx-fair", "season:autumn", "market"]
- **Facts:** archetype:striver, value:custom, value:care
- **Backstory:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.

### Character #9: Wystan
- **Ideology:** merchant
- **Traits:** [Pious, Gregarious, Proud]
- **Status:** ["wx-fair", "season:autumn", "market", "glazier", "drunk"]
- **Facts:** archetype:devout, value:freedom, value:custom
- **Backstory:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.

### Character #10: Aelfric
- **Ideology:** clergy
- **Traits:** [Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "sick"]
- **Facts:** archetype:merrymaker, value:custom, value:propriety
- **Backstory:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.

---

## Generated Conversations (N x M = 10 x 5 = 50 Total)

### Conversation #01: Aldric and Wystan
- **Speakers:** Aldric (clergy) & Wystan (merchant)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "merchant_trade"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Aldric     [Greet in a_death_in_the_house]: "A gray day, Wystan. The hearth is dark - as the mill grinds."
Wystan     [Agree in a_death_in_the_house]: "The saints receive them kindly - as my mother taught."
Aldric     [Remark in a_death_in_the_house]: "All things ripen that are let be, trader."
Wystan     [Agree in a_death_in_the_house]: "Their toil shaped this parish; their memory will outlive us."
Aldric     [Remark in a_death_in_the_house]: "Thank you for listening, Wystan - it lightens the heart."
Wystan     [Agree in a_death_in_the_house]: "Gently does it, friend. Day by day."
Aldric     [Remark in the_cleric_blesses]: "The church altar is always open to the weary soul."
Wystan     [Agree in the_cleric_blesses]: "We keep as best we can, father. The toil is hard."
Aldric     [Question in merchant_trade    ]: "What exchange do you take for a bag of salt?"
Wystan     [Answer in merchant_trade    ]: "Saints keep us. Ribbon off the fair cart, and a pinch of pepper to spare, or is that not the way of it?"
Aldric     [Remark in merchant_trade    ]: "The tally keeps honest count, and honest count brings folk back."
Wystan     [Farewell in merchant_trade    ]: "May your shelves never stand empty of stock."
```

### Conversation #02: Aldric and Dagny
- **Speakers:** Aldric (clergy) & Dagny (clergy)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 11

```text
Aldric     [Greet in a_death_in_the_house]: "Dagny. Even thou wilt know the house has gone quiet - as the mill grinds."
Dagny      [Question in a_death_in_the_house]: "Is anyone feeding thee? Even I would not see thee starve."
Aldric     [Answer in a_death_in_the_house]: "All things ripen that are let be, father."
Aldric     [Remark in the_cleric_blesses]: "The church altar is always open to the weary soul."
Dagny      [Agree in the_cleric_blesses]: "Aye, there is truth in what thou sayest - so it goes."
Aldric     [Question in asking_the_father ]: "Can the church spare a peck of grain for the hungry children, father?"
Dagny      [Evade in asking_the_father ]: "Ask me another."
Aldric     [Remark in asking_the_father ]: "Thy counsel is wise, father. We will trust in Providence."
Dagny      [Errand in asking_the_father ]: "Would the parish take a letter to the bishop's man? I'll see the wax paid for."
Aldric     [Remark in asking_the_father ]: "As thou sayest, friend."
Dagny      [Farewell in asking_the_father ]: "Fair day, Aldric - such as it is, with thee in it."
```

### Conversation #03: Aldric and Bram
- **Speakers:** Aldric (clergy) & Bram (merchant)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_consoles"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aldric     [Greet in the_cleric_consoles]: "A peaceful day to thee, Bram - as the mill grinds."
Bram       [Question in the_cleric_consoles]: "Pray for them, father - they were taken so sudden."
Aldric     [Question in the_cleric_consoles]: "Open thy heart; what sorrow weighs upon thee?"
Bram       [Answer in the_cleric_consoles]: "He went in the night, father, and I never said farewell."
Aldric     [Remark in the_cleric_consoles]: "Green wood burns when it's ready and not before."
Bram       [Farewell in the_cleric_consoles]: "Thank thee, father - thy blessing brings comfort - saints keep us."
```

### Conversation #04: Aldric and Dagny
- **Speakers:** Aldric (clergy) & Dagny (clergy)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Kin
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Aldric     [Greet in a_death_in_the_house]: "Dagny. Blood knows blood - the house is empty without them - as the mill grinds."
Dagny      [Agree in a_death_in_the_house]: "They were ours - and we carry them now - so it goes."
Aldric     [Remark in a_death_in_the_house]: "All things ripen that are let be, father."
Dagny      [Agree in a_death_in_the_house]: "Thou hast spoken the plain heart of the matter."
Aldric     [Remark in a_death_in_the_house]: "The sorrow will ease as the seasons turn and heal!"
Dagny      [Agree in a_death_in_the_house]: "We'll see it through, Aldric - the village does."
Aldric     [Remark in the_cleric_blesses]: "How fares thy house in these lean weeks, child?"
Dagny      [Agree in the_cleric_blesses]: "Aye, there is truth in what thou sayest."
Aldric     [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Dagny      [Answer in asking_the_father ]: "The church will share its grain, child - none shall starve at the door."
Aldric     [Remark in asking_the_father ]: "The church has been our refuge through many a cold year."
Dagny      [Agree in asking_the_father ]: "Blessings on thy house, father, or am I mistook?"
Aldric     [Remark in asking_the_father ]: "True enough - the fields don't gossip."
Dagny      [Farewell in asking_the_father ]: "I take my leave."
```

### Conversation #05: Aldric and Gisela
- **Speakers:** Aldric (clergy) & Gisela (lord)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "old_scores"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Aldric     [Greet in a_death_in_the_house]: "Gisela. Even thou wilt know the house has gone quiet - as the mill grinds."
Gisela     [Agree in a_death_in_the_house]: "I'll not pretend tears, Aldric - but I'll not mock the dead - softly now."
Aldric     [Remark in a_death_in_the_house]: "All things ripen that are let be, m'lord."
Gisela     [Agree in a_death_in_the_house]: "Aye, there is truth in what thou sayest."
Aldric     [Remark in a_death_in_the_house]: "Thank thee for listening, Gisela - it lightens the heart."
Gisela     [Agree in a_death_in_the_house]: "We'll see it through, Aldric - the village does."
Aldric     [Remark in the_cleric_blesses]: "How fares thy house in these lean weeks, child?"
Gisela     [Agree in the_cleric_blesses]: "The season has been lean, father, but faith sustains us, as I see it."
Aldric     [Remark in old_scores        ]: "I saw where thy cart rutted the lane by my ditch!"
Gisela     [PushBack in old_scores        ]: "Say that again, if thou darest!"
Aldric     [Remark in old_scores        ]: "Come closer and say that to my beard."
Gisela     [Remark in old_scores        ]: "I'll settle this before the magistrate, not in the dirt."
Aldric     [Remark in old_scores        ]: "Take thy shadow off my land."
Gisela     [Farewell in old_scores        ]: "Until we meet at the boundary ditch."
```

### Conversation #06: Eliza and Marga
- **Speakers:** Eliza (lord) & Marga (lord)
- **Backstory A:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_sickness      ]: "*cough* Ah, Marga - forgive the croak. Stand off, mind; I'd not share this with a friend - God send it."
Marga      [Agree in the_sickness      ]: "Thou shouldst be under woolen blankets, Eliza - thy folk will fret, and that's the long of it - by the rood."
Eliza      [Answer in the_sickness      ]: "Heard, and heard again - aye."
Marga      [Remark in the_sickness      ]: "Marga minds their own affairs, thanks."
Eliza      [Remark in the_sickness      ]: "Thou art kind, Marga. The saints remember a neighborly hand."
Marga      [Farewell in the_sickness      ]: "The village stands together or it falls. Fare well."
```

### Conversation #07: Eliza and Corvin
- **Speakers:** Eliza (lord) & Corvin (lord)
- **Backstory A:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Eliza      [Greet in the_sickness      ]: "Morrow, Corvin - a foul chest this week - God send it."
Corvin     [Agree in the_sickness      ]: "You should be under woolen blankets, m'lord - your folk will fret, or do I hear wrong - mind the stile."
Eliza      [Answer in the_sickness      ]: "Heard, and heard again - aye."
Corvin     [Agree in the_sickness      ]: "You have spoken the plain heart of the matter."
Eliza      [Remark in condolence_call   ]: "My sons can plough your furrow until you are ready."
Corvin     [Slip in condolence_call   ]: "The harvest sang that year, when - it doesn't matter."
Eliza      [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Corvin     [Farewell in condolence_call   ]: "Go gently, Eliza. The door's always open to you."
```

### Conversation #08: Eliza and Wystan
- **Speakers:** Eliza (lord) & Wystan (merchant)
- **Backstory A:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness", "stranger_road"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Eliza      [Greet in the_sickness      ]: "*cough* Morrow, traveler. Best keep to the far side of the path - God send it."
Wystan     [Question in the_sickness      ]: "How long has the fever held you, m'lord?"
Eliza      [Answer in the_sickness      ]: "Heard, and heard again - aye."
Wystan     [Agree in the_sickness      ]: "Drink hot boiled water with a scrape of honey - as my mother taught."
Eliza      [Question in stranger_road     ]: "What town do you hail from, stranger?"
Wystan     [Evade in stranger_road     ]: "We don't know each other so well as that."
Eliza      [Remark in stranger_road     ]: "Keep to the beaten track; the marshes to the east are treacherous."
Wystan     [Worry in stranger_road     ]: "They say the levy rides again come spring - is it true, Eliza?"
Eliza      [Remark in stranger_road     ]: "We have spoken our piece for today."
Wystan     [Farewell in stranger_road     ]: "Your servant, m'lord. Good day."
```

### Conversation #09: Eliza and Aldric
- **Speakers:** Eliza (lord) & Aldric (clergy)
- **Backstory A:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Eliza      [Greet in the_sickness      ]: "*cough* Ah, Aldric - forgive the croak. Stand off, mind; I'd not share this with a friend - God send it."
Aldric     [Agree in the_sickness      ]: "Thou shouldst be under woolen blankets, Eliza - thy folk will fret - as the mill grinds."
Eliza      [Answer in the_sickness      ]: "Heard, and heard again - aye."
Aldric     [Agree in the_sickness      ]: "Aye, and I'd carry half thy load if thou wouldst let me."
Eliza      [Remark in condolence_call   ]: "We have flour and eggs; let us share with thy house."
Aldric     [Worry in condolence_call   ]: "I dream of the bell some nights - does it mean aught?"
Eliza      [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Aldric     [Farewell in condolence_call   ]: "Thy coming did the house good, Eliza."
```

### Conversation #10: Eliza and Marga
- **Speakers:** Eliza (lord) & Marga (lord)
- **Backstory A:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Eliza      [Greet in the_sickness      ]: "*cough* Ah, Marga - forgive the croak. Stand off, mind; I'd not share this with a friend - God send it."
Marga      [Agree in the_sickness      ]: "Poor soul - get thee back to the manor's warm hearth, Eliza - by the rood."
Eliza      [Answer in the_sickness      ]: "Heard, and heard again - aye."
Marga      [Agree in the_sickness      ]: "Aye, and I'd carry half thy load if thou wouldst let me."
Eliza      [Question in visiting_the_sick ]: "Does the fever ease when the sun warms the cottage?"
Marga      [Answer in visiting_the_sick ]: "Marga was not born yesterday, Eliza."
Eliza      [Remark in visiting_the_sick ]: "We do what we must while daylight holds."
Marga      [Agree in visiting_the_sick ]: "The house thanks thee for it."
Eliza      [Remark in visiting_the_sick ]: "The parish keeps its own, Marga - sleep easy."
Marga      [Farewell in visiting_the_sick ]: "Then I'll see thee at first light."
```

### Conversation #11: Bram and Marga
- **Speakers:** Bram (merchant) & Marga (lord)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Bram       [Greet in visiting_the_sick ]: "The saints keep you, m'lord - saints keep us."
Marga      [Answer in visiting_the_sick ]: "Marga says what Marga says, trader."
Bram       [Question in visiting_the_sick ]: "Do you have grain left in the bin?"
Marga      [Answer in visiting_the_sick ]: "A jug of fresh well water would be a godsend, as I see it."
Bram       [Remark in visiting_the_sick ]: "It is done - look for it by sundown."
Marga      [Worry in visiting_the_sick ]: "This cough that's going round - does it take the old ones, Bram?"
Bram       [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Marga      [Farewell in visiting_the_sick ]: "Rest well, Bram - by the rood."
```

### Conversation #12: Bram and Marga
- **Speakers:** Bram (merchant) & Marga (lord)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Bram       [Greet in visiting_the_sick ]: "The saints keep you, m'lord - saints keep us."
Marga      [Answer in visiting_the_sick ]: "Marga says what Marga says, trader."
Bram       [Question in visiting_the_sick ]: "Can I fetch water or chop kindling for your hearth?"
Marga      [Answer in visiting_the_sick ]: "The sweat broke at dawn and the fever feels a little lighter."
Bram       [Remark in visiting_the_sick ]: "Rest easy; the parish will not let you freeze."
Marga      [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Bram - by the rood."
Bram       [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Marga      [Farewell in visiting_the_sick ]: "Then I'll see you at first light."
```

### Conversation #13: Bram and Wystan
- **Speakers:** Bram (merchant) & Wystan (merchant)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["a_death_in_the_house", "apprentice_eager", "the_merchants_ledger"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Bram       [Greet in a_death_in_the_house]: "The saints keep you, trader - saints keep us."
Wystan     [Agree in a_death_in_the_house]: "The saints receive them kindly - as my mother taught."
Bram       [Remark in a_death_in_the_house]: "I keep setting his wooden spoon at table!"
Wystan     [Agree in a_death_in_the_house]: "Their toil shaped this parish; their memory will outlive us."
Bram       [Remark in a_death_in_the_house]: "The house feels less empty when a neighbor sits a spell."
Wystan     [Agree in a_death_in_the_house]: "Gently does it, friend. Day by day."
Bram       [Remark in apprentice_eager  ]: "Tell me again how the old ones did it - I want it right!"
Wystan     [Agree in apprentice_eager  ]: "Well enough - keep your hand steady and your eyes open."
Bram       [Remark in the_merchants_ledger]: "The price of wool has climbed a penny since the spring fair."
Wystan     [Agree in the_merchants_ledger]: "You speak the trader's own gospel."
Bram       [Remark in the_merchants_ledger]: "Silver coin or salted hams - sound goods are always welcome."
Wystan     [Farewell in the_merchants_ledger]: "Until the spring fair, merchant. Fare well."
```

### Conversation #14: Bram and Dagny
- **Speakers:** Bram (merchant) & Dagny (clergy)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Bram       [Greet in a_death_in_the_house]: "The saints keep thee, father - saints keep us."
Dagny      [Agree in a_death_in_the_house]: "We'd our quarrels, Bram - but the grave settles them - so it goes."
Bram       [Remark in a_death_in_the_house]: "Forty winters together, and now only the silence!"
Dagny      [Agree in a_death_in_the_house]: "Thou hast spoken the plain heart of the matter."
Bram       [Remark in a_death_in_the_house]: "It does a body good to speak their name aloud."
Dagny      [Agree in a_death_in_the_house]: "We'll see it through, Bram - the village does."
Bram       [Question in asking_the_father ]: "Will the parish altar provide candles for our sick neighbor, father?"
Dagny      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor court."
Bram       [Remark in asking_the_father ]: "Thank thee, father - that lifts a great stone from my mind."
Dagny      [Agree in asking_the_father ]: "The parish is lucky in its father!"
Bram       [Remark in asking_the_father ]: "We have spoken our piece for today!"
Dagny      [Farewell in asking_the_father ]: "May the road pay thy way, merchant."
```

### Conversation #15: Bram and Dagny
- **Speakers:** Bram (merchant) & Dagny (clergy)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Bram       [Greet in a_death_in_the_house]: "The saints keep thee, father - saints keep us."
Dagny      [Agree in a_death_in_the_house]: "I'll not pretend tears, Bram - but I'll not mock the dead - so it goes."
Bram       [Remark in a_death_in_the_house]: "I keep setting his wooden spoon at table!"
Dagny      [Agree in a_death_in_the_house]: "Aye, there is truth in what thou sayest."
Bram       [Remark in a_death_in_the_house]: "The house feels less empty when a neighbor sits a spell."
Dagny      [Agree in a_death_in_the_house]: "We'll see it through, Bram - the village does."
Bram       [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Dagny      [Evade in asking_the_father ]: "That's not for the telling, friend."
Bram       [Remark in asking_the_father ]: "Thy blessing gives me heart to face the coming week, father."
Dagny      [Agree in asking_the_father ]: "Blessings on thy house, father, or am I mistook?"
Bram       [Remark in asking_the_father ]: "Aye, words won't fill the storehouse!"
Dagny      [Farewell in asking_the_father ]: "I take my leave - gladly, and more besides, or am I mistook?"
```

### Conversation #16: Corvin and Wystan
- **Speakers:** Corvin (lord) & Wystan (merchant)
- **Backstory A:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["a_death_in_the_house", "mentor_counsel", "friendship_bonds"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Corvin     [Greet in a_death_in_the_house]: "Wystan. Forgive me - the house is still since they passed, and thy face is a kindness - mind the stile."
Wystan     [Agree in a_death_in_the_house]: "They were the best of us, Corvin - I'll miss them beside thee, for my part - as my mother taught."
Corvin     [Remark in a_death_in_the_house]: "God witness. The haar's a fisher's blanket - keeps the cold off, mind."
Wystan     [Agree in a_death_in_the_house]: "By the rood. Thou hast spoken the plain heart of the matter."
Corvin     [Remark in a_death_in_the_house]: "It does a body good to speak their name aloud."
Wystan     [Agree in a_death_in_the_house]: "Gently does it, friend. Day by day."
Corvin     [Remark in mentor_counsel    ]: "Mark this well - the work outlives the wage."
Wystan     [Agree in mentor_counsel    ]: "Thy words are worth more than a day's wage, master."
Corvin     [Question in friendship_bonds  ]: "Can I count on thy team when we haul the heavy timber?"
Wystan     [Answer in friendship_bonds  ]: "Saints keep us. My oxen are thine whenever thou needest them, friend."
Corvin     [Remark in friendship_bonds  ]: "A good friend is worth more than a pouch of silver."
Wystan     [Farewell in friendship_bonds  ]: "We will share a cup of cider when the work is done."
```

### Conversation #17: Corvin and Dagny
- **Speakers:** Corvin (lord) & Dagny (clergy)
- **Backstory A:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["a_death_in_the_house", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 9

```text
Corvin     [Greet in a_death_in_the_house]: "A gray day, Dagny. The hearth is dark, or so I make it - mind the stile."
Dagny      [Question in a_death_in_the_house]: "Is there any chore I can take off your shoulders?"
Corvin     [Answer in a_death_in_the_house]: "A few split logs, if you have any seasoned oak."
Corvin     [Question in asking_the_father ]: "How do you find the road today, Dagny?"
Dagny      [Evade in asking_the_father ]: "That's not for the telling, friend."
Corvin     [Remark in asking_the_father ]: "God witness. The haar's a fisher's blanket - keeps the cold off, mind."
Dagny      [Agree in asking_the_father ]: "The parish is lucky in its father - so it goes."
Corvin     [Remark in asking_the_father ]: "God witness. As you say, friend."
Dagny      [Farewell in asking_the_father ]: "God keep the manor and you with it, m'lord."
```

### Conversation #18: Corvin and Eliza
- **Speakers:** Corvin (lord) & Eliza (lord)
- **Backstory A:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Backstory B:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Corvin     [Greet in visiting_the_sick ]: "How dost thou fare today, Eliza - mind the stile."
Eliza      [PushBack in visiting_the_sick ]: "Nay, I see it differently, mark me."
Corvin     [Question in visiting_the_sick ]: "By the rood. Does the fever ease when the sun warms the cottage?"
Eliza      [Answer in visiting_the_sick ]: "Aye. Aye - as thou sayest, as thou sayest."
Corvin     [Remark in visiting_the_sick ]: "Saints keep us. Rain before seven, dry by eleven, they say."
Eliza      [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Corvin - God send it."
Corvin     [Remark in visiting_the_sick ]: "The parish keeps its own, Eliza - sleep easy, or so I make it."
Eliza      [Farewell in visiting_the_sick ]: "Rest well, Corvin."
```

### Conversation #19: Corvin and Wystan
- **Speakers:** Corvin (lord) & Wystan (merchant)
- **Backstory A:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Stranger
- **Trees Cascade:** ["a_death_in_the_house", "stranger_road"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Corvin     [Greet in a_death_in_the_house]: "A heavy morning - we've buried one of ours - mind the stile."
Wystan     [Agree in a_death_in_the_house]: "I did not know them, stranger, but I grieve your grief - as my mother taught."
Corvin     [Remark in a_death_in_the_house]: "God witness. The haar's a fisher's blanket - keeps the cold off, mind."
Wystan     [Agree in a_death_in_the_house]: "By the rood. You have spoken the plain heart of the matter."
Corvin     [Remark in a_death_in_the_house]: "The house feels less empty when a neighbor sits a spell, as I see it."
Wystan     [Agree in a_death_in_the_house]: "We'll see it through, Corvin - the village does."
Corvin     [Question in stranger_road     ]: "Where does this trail lead past the ridge, traveler?"
Wystan     [Answer in stranger_road     ]: "I come from the valley beyond the high moors, and more besides."
Corvin     [Remark in stranger_road     ]: "The parish watch keeps guard after the curfew bell."
Wystan     [Agree in stranger_road     ]: "Then the parish wishes you well of it, and that's the long of it."
Corvin     [Remark in stranger_road     ]: "We have spoken our piece for today."
Wystan     [Farewell in stranger_road     ]: "I take my leave, or am I mistook?"
```

### Conversation #20: Corvin and Hob
- **Speakers:** Corvin (lord) & Hob (clergy)
- **Backstory A:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Backstory B:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Corvin     [Greet in visiting_the_sick ]: "Peace to this cottage, Hob. How goes the fever - mind the stile."
Hob        [Answer in visiting_the_sick ]: "A barred door keeps out friends as well as wolves, and more besides."
Corvin     [Question in visiting_the_sick ]: "Dost thou have grain left in the bin?"
Hob        [Answer in visiting_the_sick ]: "A little bread or salt, if thou canst spare it."
Corvin     [Remark in visiting_the_sick ]: "Saints keep us. Rain before seven, dry by eleven, they say."
Hob        [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Corvin - and what of it."
Corvin     [Remark in visiting_the_sick ]: "The parish keeps its own, Hob - sleep easy, or so I make it."
Hob        [Farewell in visiting_the_sick ]: "Then I'll see thee at first light."
```

### Conversation #21: Dagny and Bram
- **Speakers:** Dagny (clergy) & Bram (merchant)
- **Backstory A:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_consoles"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in the_cleric_consoles]: "A peaceful day to thee, Bram - so it goes."
Bram       [Question in the_cleric_consoles]: "A blessing for my sorrowing cottage, father!"
Dagny      [Question in the_cleric_consoles]: "The Lord comfort thee, child. Tell me of thy affliction, as I've cause to say."
Bram       [Answer in the_cleric_consoles]: "He went in the night, father, and I never said farewell."
Dagny      [Remark in the_cleric_consoles]: "Sixty winters I've kept this patch, Bram; I know its ways."
Bram       [Farewell in the_cleric_consoles]: "Thank thee, father - thy blessing brings comfort - saints keep us."
```

### Conversation #22: Dagny and Marga
- **Speakers:** Dagny (clergy) & Marga (lord)
- **Backstory A:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee, child - so it goes."
Marga      [Question in the_cleric_visits_the_sick]: "Wilt thou hear my confession ere the fever takes me?"
Dagny      [Question in the_cleric_visits_the_sick]: "How dost thou find the road today, Marga?"
Marga      [Answer in the_cleric_visits_the_sick]: "Marga minds their own affairs, thanks."
Dagny      [Remark in the_cleric_visits_the_sick]: "Sixty winters I've kept this patch, Marga; I know its ways."
Marga      [Farewell in the_cleric_visits_the_sick]: "God's peace be upon thy cottage, Dagny - by the rood."
```

### Conversation #23: Dagny and Eliza
- **Speakers:** Dagny (clergy) & Eliza (lord)
- **Backstory A:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Backstory B:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_visits_the_sick", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Dagny      [Greet in the_cleric_visits_the_sick]: "God's grace light upon thy steps, Eliza - so it goes."
Eliza      [Greet in the_cleric_visits_the_sick]: "Blessings on thy holy house as well - God send it."
Dagny      [Remark in the_cleric_visits_the_sick]: "In all my years I never saw the like, m'lord, as I've cause to say."
Eliza      [Agree in the_cleric_visits_the_sick]: "The season has been lean, father, but faith sustains us."
Dagny      [Question in visiting_the_sick ]: "Does the fever ease when the sun warms the cottage?"
Eliza      [Answer in visiting_the_sick ]: "Heard, and heard again - aye."
Dagny      [Remark in visiting_the_sick ]: "I'll have my girl bring the loaf before dark."
Eliza      [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Dagny."
Dagny      [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Eliza      [Farewell in visiting_the_sick ]: "Rest well, Dagny."
```

### Conversation #24: Dagny and Wystan
- **Speakers:** Dagny (clergy) & Wystan (merchant)
- **Backstory A:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_blesses", "morning_after_drunk"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Dagny      [Greet in the_cleric_blesses]: "The peace of the saints be with you, child - so it goes."
Wystan     [Remark in the_cleric_blesses]: "It'd cost you a groat to hear worse."
Dagny      [Remark in the_cleric_blesses]: "In all my years I never saw the like, trader, as I've cause to say."
Wystan     [Agree in the_cleric_blesses]: "We keep as best we can, father. The toil is hard - as my mother taught."
Dagny      [Remark in morning_after_drunk]: "We do what we must while daylight holds."
Wystan     [Agree in morning_after_drunk]: "Aye, there is truth in what you say."
Dagny      [Remark in morning_after_drunk]: "A bucket of icy well water over the skull will clear the cobwebs."
Wystan     [Farewell in morning_after_drunk]: "May the sun be gentle on your poor eyes today."
```

### Conversation #25: Dagny and Eliza
- **Speakers:** Dagny (clergy) & Eliza (lord)
- **Backstory A:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Backstory B:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_visits_the_sick", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Dagny      [Greet in the_cleric_visits_the_sick]: "God's grace light upon thy steps, Eliza - so it goes."
Eliza      [Remark in the_cleric_visits_the_sick]: "Aye, words have weight in a small village."
Dagny      [Remark in the_cleric_visits_the_sick]: "In all my years I never saw the like, Eliza, as I've cause to say."
Eliza      [Agree in the_cleric_visits_the_sick]: "Thou hast spoken the plain heart of the matter - God send it."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest brought to the cottage?"
Eliza      [Answer in visiting_the_sick ]: "Heard, and heard again - aye."
Dagny      [Remark in visiting_the_sick ]: "Rest easy; the parish will not let thee freeze."
Eliza      [Agree in visiting_the_sick ]: "The house thanks thee for it."
Dagny      [Remark in visiting_the_sick ]: "The parish keeps its own, Eliza - sleep easy."
Eliza      [Farewell in visiting_the_sick ]: "Then I'll see thee at first light."
```

### Conversation #26: Hob and Bram
- **Speakers:** Hob (clergy) & Bram (merchant)
- **Backstory A:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_consoles", "the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Hob        [Greet in the_cleric_consoles]: "A peaceful day to thee, Bram - and what of it."
Bram       [Greet in the_cleric_consoles]: "Blessings on thee and thy kin, Hob - saints keep us."
Hob        [Remark in the_cleric_consoles]: "Caution's a warm cloak, and a heavy one."
Bram       [Agree in the_cleric_consoles]: "Thou hast spoken the plain heart of the matter."
Hob        [Answer in the_sickness      ]: "The calves won't feed themselves - though our kin have offered, bless them."
Bram       [Agree in the_sickness      ]: "Blood doesn't watch blood labor alone for long."
Hob        [Remark in condolence_call   ]: "We have flour and eggs; let us share with thy house."
Bram       [Agree in condolence_call   ]: "Kindly said, Hob. Stay the hour."
Hob        [Remark in condolence_call   ]: "The house won't grieve alone, Bram."
Bram       [Farewell in condolence_call   ]: "Go gently, Hob. The door's always open to thee."
```

### Conversation #27: Hob and Dagny
- **Speakers:** Hob (clergy) & Dagny (clergy)
- **Backstory A:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness"]
- **Ending:** WalksOff
- **Turns Count:** 4

```text
Hob        [Greet in the_sickness      ]: "*cough* Morrow, traveler. Best keep to the far side of the path - and what of it."
Dagny      [PushBack in the_sickness      ]: "I'll hear you from across the ditch, and no nearer."
Hob        [Remark in the_sickness      ]: "Caution's a warm cloak, and a heavy one."
Dagny      [PushBack in the_sickness      ]: "I'll not catch my death for your pride."
```

### Conversation #28: Hob and Corvin
- **Speakers:** Hob (clergy) & Corvin (lord)
- **Backstory A:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Backstory B:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Relation:** Friend
- **Trees Cascade:** ["the_cleric_consoles"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_cleric_consoles]: "God's grace light upon thy steps, Corvin - and what of it."
Corvin     [Question in the_cleric_consoles]: "Pray for them, father - they were taken so sudden, as I see it."
Hob        [Question in the_cleric_consoles]: "How dost thou find the road today, Corvin?"
Corvin     [Answer in the_cleric_consoles]: "Saints keep us. He went in the night, father, and I never said farewell."
Hob        [Remark in the_cleric_consoles]: "A barred door keeps out friends as well as wolves, and more besides."
Corvin     [Farewell in the_cleric_consoles]: "I will be at the church when the bell sounds - mind the stile."
```

### Conversation #29: Hob and Aldric
- **Speakers:** Hob (clergy) & Aldric (clergy)
- **Backstory A:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_consoles", "the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Hob        [Greet in the_cleric_consoles]: "A peaceful day to you, Aldric - and what of it."
Aldric     [Greet in the_cleric_consoles]: "Peace be with you, good priest - as the mill grinds."
Hob        [Remark in the_cleric_consoles]: "Caution's a warm cloak, and a heavy one."
Aldric     [Agree in the_cleric_consoles]: "I try to walk uprightly before God and my neighbors."
Hob        [Answer in the_sickness      ]: "There is no one else to fetch water from the river."
Aldric     [Agree in the_sickness      ]: "Aye, there is truth in what you say."
Hob        [Remark in condolence_call   ]: "We have flour and eggs; let us share with your house."
Aldric     [Agree in condolence_call   ]: "Kindly said, Hob. Stay the hour."
Hob        [Remark in condolence_call   ]: "The house won't grieve alone, Aldric."
Aldric     [Farewell in condolence_call   ]: "Your coming did the house good, Hob."
```

### Conversation #30: Hob and Eliza
- **Speakers:** Hob (clergy) & Eliza (lord)
- **Backstory A:** Hob is the village smith. Their purse strings are knotted tight. They know every soul and every soul's business. They work the village anvil and forge. A cough's had them these two weeks.
- **Backstory B:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with you, child, as I see it - and what of it."
Eliza      [Question in the_cleric_visits_the_sick]: "A prayer for me, father - this fever won't quit."
Hob        [Question in the_cleric_visits_the_sick]: "How do you find the road today, Eliza?"
Eliza      [Answer in the_cleric_visits_the_sick]: "Aye. Aye - as you say, as you say."
Hob        [Remark in the_cleric_visits_the_sick]: "A barred door keeps out friends as well as wolves, and more besides."
Eliza      [Farewell in the_cleric_visits_the_sick]: "The Lord keep you, father, and preserve your voice - God send it."
```

### Conversation #31: Marga and Wystan
- **Speakers:** Marga (lord) & Wystan (merchant)
- **Backstory A:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in the_sickness      ]: "Morrow, Wystan. Step wide; even my fever wants no part of thee, as I see it - by the rood."
Wystan     [Question in the_sickness      ]: "Hast thou seen the herb-wife for a draught, m'lord?"
Marga      [Answer in the_sickness      ]: "Marga was not born yesterday, Wystan."
Wystan     [Remark in the_sickness      ]: "Two pennies' worth of worry, that is."
Marga      [Remark in the_sickness      ]: "Bless thee for thinking of our small house."
Wystan     [Farewell in the_sickness      ]: "God's peace be upon thy cottage, Marga - as my mother taught."
```

### Conversation #32: Marga and Bram
- **Speakers:** Marga (lord) & Bram (merchant)
- **Backstory A:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in the_sickness      ]: "A sick man's greeting, stranger - from a distance - by the rood."
Bram       [Agree in the_sickness      ]: "Poor soul - get you back to the manor's warm hearth, m'lord - saints keep us."
Marga      [Answer in the_sickness      ]: "Marga was not born yesterday, trader."
Bram       [Remark in the_sickness      ]: "Aye, words have weight in a small village."
Marga      [Remark in the_sickness      ]: "My thanks. A warm bowl does more than herbs."
Bram       [Farewell in the_sickness      ]: "Think nothing of it. We look after our own."
```

### Conversation #33: Marga and Corvin
- **Speakers:** Marga (lord) & Corvin (lord)
- **Backstory A:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Backstory B:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Marga      [Greet in the_sickness      ]: "A sick man's greeting, stranger - from a distance - by the rood."
Corvin     [Agree in the_sickness      ]: "You should be under woolen blankets, m'lord - your folk will fret, or do I hear wrong - mind the stile."
Marga      [Answer in the_sickness      ]: "Marga was not born yesterday, m'lord."
Corvin     [Agree in the_sickness      ]: "Aye, there is truth in what you say."
Marga      [Remark in condolence_call   ]: "My sons can plough your furrow until you are ready."
Corvin     [Agree in condolence_call   ]: "Sit then, Marga - the fire's low but there's room, as I see it."
Marga      [Remark in condolence_call   ]: "The house won't grieve alone, Corvin."
Corvin     [Farewell in condolence_call   ]: "Your coming did the house good, Marga, or do I hear wrong?"
```

### Conversation #34: Marga and Bram
- **Speakers:** Marga (lord) & Bram (merchant)
- **Backstory A:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** WalksOff
- **Turns Count:** 4

```text
Marga      [Greet in the_sickness      ]: "Bram - aye, it's me, coughing like a broken bellows. Keep out of the draft - by the rood."
Bram       [PushBack in the_sickness      ]: "Keep thy distance, Marga - I have no wish to catch it!"
Marga      [Remark in the_sickness      ]: "Marga was not born yesterday, Bram."
Bram       [PushBack in the_sickness      ]: "Stay in thy own yard if thou art frightened!"
```

### Conversation #35: Marga and Dagny
- **Speakers:** Marga (lord) & Dagny (clergy)
- **Backstory A:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Marga      [Greet in the_sickness      ]: "Dagny - aye, it's me, coughing like a broken bellows. Keep out of the draft - by the rood."
Dagny      [Agree in the_sickness      ]: "Thou shouldst be under woolen blankets, Marga - thy folk will fret - so it goes."
Marga      [Answer in the_sickness      ]: "Marga was not born yesterday, father."
Dagny      [Agree in the_sickness      ]: "Aye, but our house will see thee through it."
Marga      [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Dagny      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor court."
Marga      [Remark in asking_the_father ]: "Thy counsel is wise, father. We will trust in Providence."
Dagny      [Errand in asking_the_father ]: "Would the parish take a letter to the bishop's man? I'll see the wax paid for."
Marga      [Remark in asking_the_father ]: "Aye, words won't fill the storehouse."
Dagny      [Farewell in asking_the_father ]: "Enough chatter. Good day."
```

### Conversation #36: Gisela and Dagny
- **Speakers:** Gisela (lord) & Dagny (clergy)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Gisela     [Greet in asking_the_father ]: "Mm-mm-mm - Greetings, holy father - I seek thy pastoral ear - softly now."
Dagny      [Greet in asking_the_father ]: "The saints keep thee, Gisela. What weighs on thee - so it goes."
Gisela     [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Dagny      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor court."
Gisela     [Remark in asking_the_father ]: "Thank thee, father - that lifts a great stone from my mind."
Dagny      [Errand in asking_the_father ]: "Would the parish take a letter to the bishop's man? I'll see the wax paid for."
Gisela     [Remark in asking_the_father ]: "As thou sayest, friend."
Dagny      [Farewell in asking_the_father ]: "I take my leave."
```

### Conversation #37: Gisela and Corvin
- **Speakers:** Gisela (lord) & Corvin (lord)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Backstory B:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in condolence_call   ]: "Mm-mm-mm - I was deeply grieved to hear of thy loss, Corvin, or so I make it - softly now."
Corvin     [Agree in condolence_call   ]: "Aye, there is truth in what thou sayest - mind the stile."
Gisela     [Remark in condolence_call   ]: "My sons can plough thy furrow until thou art ready."
Corvin     [Worry in condolence_call   ]: "Do the dead sleep, Gisela? Or do they wait on us?"
Gisela     [Remark in condolence_call   ]: "I'll sit with thee a while yet, if it helps."
Corvin     [Farewell in condolence_call   ]: "Thy coming did the house good, Gisela, or do I hear wrong?"
```

### Conversation #38: Gisela and Aelfric
- **Speakers:** Gisela (lord) & Aelfric (clergy)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Backstory B:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Gisela     [Greet in visiting_the_sick ]: "Mm-mm-mm - A peaceful day to thee, Aelfric, and that's the long of it - softly now."
Aelfric    [Answer in visiting_the_sick ]: "Poorly, if thou wilt have the plain truth."
Gisela     [Question in visiting_the_sick ]: "Can I fetch water or chop kindling for thy hearth?"
Aelfric    [Answer in visiting_the_sick ]: "A little bread or salt, if thou canst spare it."
Gisela     [Remark in visiting_the_sick ]: "I'll have my girl bring the loaf before dark."
Aelfric    [Agree in visiting_the_sick ]: "The house thanks thee for it - the saints see all."
Gisela     [Remark in visiting_the_sick ]: "The parish keeps its own, Aelfric - sleep easy, and that's the long of it."
Aelfric    [Farewell in visiting_the_sick ]: "Then I'll see thee at first light."
```

### Conversation #39: Gisela and Marga
- **Speakers:** Gisela (lord) & Marga (lord)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Gisela     [Greet in visiting_the_sick ]: "Mm-mm-mm - A peaceful day to you, Marga, and that's the long of it - softly now."
Marga      [Answer in visiting_the_sick ]: "Marga says what Marga says, m'lord."
Gisela     [Question in visiting_the_sick ]: "Can I fetch water or chop kindling for your hearth?"
Marga      [Answer in visiting_the_sick ]: "A jug of fresh well water would be a godsend, as I see it."
Gisela     [Remark in visiting_the_sick ]: "We do what we must while daylight holds."
Marga      [Worry in visiting_the_sick ]: "This cough that's going round - does it take the old ones, Gisela?"
Gisela     [Remark in visiting_the_sick ]: "The parish keeps its own, Marga - sleep easy, and that's the long of it."
Marga      [Farewell in visiting_the_sick ]: "Rest well, Gisela - by the rood."
```

### Conversation #40: Gisela and Marga
- **Speakers:** Gisela (lord) & Marga (lord)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They'd sooner watch the clouds than the plough. They know every soul and every soul's business.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Gisela     [Greet in visiting_the_sick ]: "Mm-mm-mm - I heard the sickness took your household - how is it - softly now."
Marga      [PushBack in visiting_the_sick ]: "That is not the way the elders told it."
Gisela     [Question in visiting_the_sick ]: "Is there aught you need brought to the cottage?"
Marga      [Answer in visiting_the_sick ]: "Marga minds their own affairs, thanks."
Gisela     [Remark in visiting_the_sick ]: "We do what we must while daylight holds."
Marga      [Agree in visiting_the_sick ]: "The house thanks you for it - by the rood."
Gisela     [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Marga      [Farewell in visiting_the_sick ]: "Then I'll see you at first light."
```

### Conversation #41: Wystan and Eliza
- **Speakers:** Wystan (merchant) & Eliza (lord)
- **Backstory A:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Eliza holds the manor's leases. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Wystan     [Greet in visiting_the_sick ]: "Peace to this cottage, Eliza. How goes the fever - as my mother taught."
Eliza      [Answer in visiting_the_sick ]: "So you say, so it is."
Wystan     [Question in visiting_the_sick ]: "Do you have grain left in the bin?"
Eliza      [Answer in visiting_the_sick ]: "A jug of fresh well water would be a godsend."
Wystan     [Remark in visiting_the_sick ]: "It'd cost you a groat to hear worse."
Eliza      [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Wystan - God send it."
Wystan     [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Eliza      [Farewell in visiting_the_sick ]: "Then I'll see you at first light."
```

### Conversation #42: Wystan and Aelfric
- **Speakers:** Wystan (merchant) & Aelfric (clergy)
- **Backstory A:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Wystan     [Greet in visiting_the_sick ]: "Peace to this cottage, Aelfric. How goes the fever - as my mother taught."
Aelfric    [Answer in visiting_the_sick ]: "The night was hard, but the morning brings a little ease."
Wystan     [Question in visiting_the_sick ]: "Is there aught you need brought to the cottage?"
Aelfric    [Answer in visiting_the_sick ]: "Some dry firewood would keep the chill out of our bones."
Wystan     [Remark in visiting_the_sick ]: "It'd cost you a groat to hear worse."
Aelfric    [Agree in visiting_the_sick ]: "Aye, that would sit kindly, Wystan - the saints see all."
Wystan     [Remark in visiting_the_sick ]: "I'll look in again on the morrow, rest now."
Aelfric    [Farewell in visiting_the_sick ]: "Rest well, Wystan."
```

### Conversation #43: Wystan and Bram
- **Speakers:** Wystan (merchant) & Bram (merchant)
- **Backstory A:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Wystan     [Greet in condolence_call   ]: "I was deeply grieved to hear of your loss, Bram - as my mother taught."
Bram       [PushBack in condolence_call   ]: "That is not the way the elders told it."
Wystan     [Remark in condolence_call   ]: "Worth a farthing at market, that news."
Bram       [Agree in condolence_call   ]: "Kindly said, Wystan. Stay the hour - saints keep us."
Wystan     [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Bram       [Farewell in condolence_call   ]: "Your coming did the house good, Wystan."
```

### Conversation #44: Wystan and Corvin
- **Speakers:** Wystan (merchant) & Corvin (lord)
- **Backstory A:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Corvin is the village smith. They bow to no one. They never miss a service. They work the village anvil and forge. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Wystan     [Greet in condolence_call   ]: "God's mercy comfort thee in thy sorrow, neighbor, for my part, or do I hear wrong - as my mother taught."
Corvin     [Agree in condolence_call   ]: "The road is lonely now, but kind words help - mind the stile."
Wystan     [Remark in condolence_call   ]: "Worth a farthing at market, that news."
Corvin     [Slip in condolence_call   ]: "The harvest sang that year, when - it doesn't matter."
Wystan     [Remark in condolence_call   ]: "I'll sit with thee a while yet, if it helps."
Corvin     [Farewell in condolence_call   ]: "Go gently, Wystan. The door's always open to thee."
```

### Conversation #45: Wystan and Dagny
- **Speakers:** Wystan (merchant) & Dagny (clergy)
- **Backstory A:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Kin
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Wystan     [Greet in asking_the_father ]: "A word with thee, father, if thou canst spare a moment - as my mother taught."
Dagny      [Greet in asking_the_father ]: "Always, child. Speak freely in the sight of God - so it goes."
Wystan     [Question in asking_the_father ]: "Will the parish altar provide candles for our sick neighbor, father?"
Dagny      [Answer in asking_the_father ]: "The church will share its grain, child - none shall starve at the door."
Wystan     [Remark in asking_the_father ]: "It'd cost thee a groat to hear worse."
Dagny      [Agree in asking_the_father ]: "Blessings on thy house, father, or am I mistook?"
Wystan     [Remark in asking_the_father ]: "Aye, words won't fill the storehouse."
Dagny      [Farewell in asking_the_father ]: "Fair profit to thee, trader."
```

### Conversation #46: Aelfric and Wystan
- **Speakers:** Aelfric (clergy) & Wystan (merchant)
- **Backstory A:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Wystan is the village glazier. They never miss a service. They know every soul and every soul's business. They bow to no one. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness", "the_cleric_blesses", "mentor_counsel", "merchant_trade"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Aelfric    [Greet in the_sickness      ]: "*cough* Fair morning, Wystan. Mind the breeze between us - the saints see all."
Wystan     [Agree in the_sickness      ]: "A rough fever, by the look of you - as my mother taught."
Aelfric    [Answer in the_sickness      ]: "There is no one else to fetch water from the river."
Wystan     [Agree in the_sickness      ]: "A poor man's bed is a luxury he can rarely afford."
Aelfric    [Remark in the_cleric_blesses]: "Remember that honest work in the field is a holy prayer, for mine own sins."
Wystan     [Agree in the_cleric_blesses]: "The season has been lean, father, but faith sustains us."
Aelfric    [Remark in mentor_counsel    ]: "I was green once, same as you - the hand learns what the eye cannot."
Wystan     [Agree in mentor_counsel    ]: "Aye, I mark it - as you have taught me."
Aelfric    [Question in merchant_trade    ]: "What price for a pound of good salt this week?"
Wystan     [Answer in merchant_trade    ]: "Saints keep us. Ribbon off the fair cart, and a pinch of pepper to spare, or is that not the way of it?"
Aelfric    [Remark in merchant_trade    ]: "The scales do not lie - a scruple light and the whole village remembers."
Wystan     [Farewell in merchant_trade    ]: "God's peace be upon your cottage, Aelfric."
```

### Conversation #47: Aelfric and Aldric
- **Speakers:** Aelfric (clergy) & Aldric (clergy)
- **Backstory A:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_consoles"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aelfric    [Greet in the_cleric_consoles]: "Blessings on thee, Aldric - the saints see all."
Aldric     [Question in the_cleric_consoles]: "Hah! Father, where do the dead wait for us?"
Aelfric    [Question in the_cleric_consoles]: "How dost thou find the road today, Aldric?"
Aldric     [Answer in the_cleric_consoles]: "The ditch fills one stone at a time, my mother said."
Aelfric    [Remark in the_cleric_consoles]: "Cast thy burdens before the altar; the Lord does not forget the meek, for mine own sins."
Aldric     [Farewell in the_cleric_consoles]: "The Lord keep thee, father, and preserve thy voice - as the mill grinds."
```

### Conversation #48: Aelfric and Marga
- **Speakers:** Aelfric (clergy) & Marga (lord)
- **Backstory A:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Marga is the village fisher. They know every soul and every soul's business. They're the first with a loaf when a house is bare. They mean to be more than they were born. They set eel traps in the river current. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_visits_the_sick", "the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Aelfric    [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee, child - the saints see all."
Marga      [Remark in the_cleric_visits_the_sick]: "Marga says what Marga says, father."
Aelfric    [Remark in the_cleric_visits_the_sick]: "The church altar is always open to the weary soul, and for my sins beside."
Marga      [Agree in the_cleric_visits_the_sick]: "Thou hast spoken the plain heart of the matter - by the rood."
Aelfric    [Answer in the_sickness      ]: "The calves won't feed themselves - though our kin have offered, bless them."
Marga      [Remark in the_sickness      ]: "Our house has barley to spare - blood takes care of blood, as I see it."
Aelfric    [Remark in the_sickness      ]: "My thanks. A warm bowl does more than herbs."
Marga      [Farewell in the_sickness      ]: "Think nothing of it. We look after our own."
```

### Conversation #49: Aelfric and Bram
- **Speakers:** Aelfric (clergy) & Bram (merchant)
- **Backstory A:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_consoles", "the_sickness", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Aelfric    [Greet in the_cleric_consoles]: "A peaceful day to you, Bram - the saints see all."
Bram       [Greet in the_cleric_consoles]: "Blessings on you and your kin, Aelfric - saints keep us."
Aelfric    [Remark in the_cleric_consoles]: "Do you find peace in your daily labor, child?, and for my sins beside."
Bram       [Agree in the_cleric_consoles]: "You have spoken the plain heart of the matter."
Aelfric    [Answer in the_sickness      ]: "I'd rest in the straw if the chores would do themselves."
Bram       [Agree in the_sickness      ]: "Aye, the dirt cares nothing for an aching back!"
Aelfric    [Remark in condolence_call   ]: "If you need aught at all, say but the word."
Bram       [Agree in condolence_call   ]: "Kindly said, Aelfric. Stay the hour."
Aelfric    [Remark in condolence_call   ]: "The house won't grieve alone, Bram."
Bram       [Farewell in condolence_call   ]: "Go gently, Aelfric. The door's always open to you."
```

### Conversation #50: Aelfric and Dagny
- **Speakers:** Aelfric (clergy) & Dagny (clergy)
- **Backstory A:** Aelfric serves the village church. They'd sooner watch the clouds than the plough. A cough's had them these two weeks.
- **Backstory B:** Dagny serves the village church. They know every soul and every soul's business. They'd sooner watch the clouds than the plough. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aelfric    [Greet in the_sickness      ]: "*cough* Ah, Dagny - forgive the croak. Stand off, mind; I'd not share this with a friend - the saints see all."
Dagny      [PushBack in the_sickness      ]: "I will hear thee from across the ditch."
Aelfric    [Remark in the_sickness      ]: "A little cough won't kill an honest worker, and for my sins beside."
Dagny      [Agree in the_sickness      ]: "Forgive me - fear makes cowards of us all - so it goes."
Aelfric    [Remark in the_sickness      ]: "Thou art kind, Dagny. The saints remember a neighborly hand."
Dagny      [Farewell in the_sickness      ]: "Rest quiet by the coals. I'll check on thee tomorrow."
```

## Benchmark Summary

- Total Characters Generated: 10
- Total Conversations Generated: 50
- Generator Seed: 42

Benchmark generation complete.
