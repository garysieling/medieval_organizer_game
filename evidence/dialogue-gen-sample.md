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
- **Ideology:** lord
- **Traits:** [Pious]
- **Status:** []
- **Backstory:** Aldric holds the manor's leases. They never miss a service.

### Character #2: Eliza
- **Ideology:** clergy
- **Traits:** [Pious, Proud]
- **Status:** ["drunk"]
- **Backstory:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.

### Character #3: Bram
- **Ideology:** lord
- **Traits:** [Gregarious, Pious]
- **Status:** ["sick"]
- **Backstory:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.

### Character #4: Corvin
- **Ideology:** clergy
- **Traits:** [Hotheaded, Pious, Lazy]
- **Status:** ["hungry"]
- **Backstory:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.

### Character #5: Dagny
- **Ideology:** merchant
- **Traits:** [Ambitious]
- **Status:** ["hungry"]
- **Backstory:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.

### Character #6: Hob
- **Ideology:** merchant
- **Traits:** [Proud, Ambitious]
- **Status:** ["sick"]
- **Backstory:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.

### Character #7: Marga
- **Ideology:** merchant
- **Traits:** [Hotheaded, Stingy]
- **Status:** []
- **Backstory:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.

### Character #8: Gisela
- **Ideology:** lord
- **Traits:** [Ambitious, Pious, Gregarious]
- **Status:** ["sick"]
- **Backstory:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.

### Character #9: Wystan
- **Ideology:** merchant
- **Traits:** [Ambitious, Lazy]
- **Status:** []
- **Backstory:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.

### Character #10: Aelfric
- **Ideology:** clergy
- **Traits:** [Gregarious, Kind]
- **Status:** ["drunk"]
- **Backstory:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.

---

## Generated Conversations (N x M = 10 x 5 = 50 Total)

### Conversation #01: Aldric and Marga
- **Speakers:** Aldric (lord) & Marga (merchant)
- **Backstory A:** Aldric holds the manor's leases. They never miss a service.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Friend
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Aldric     [Greet in village_smalltalk ]: "Well met, Marga."
Marga      [Greet in village_smalltalk ]: "And to thee, Aldric."
Aldric     [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Marga      [PushBack in village_smalltalk ]: "Rain? It's the driest year in memory."
Aldric     [Remark in village_smalltalk ]: "Even so, we'll manage."
Marga      [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Aldric     [Remark in village_smalltalk ]: "They say the miller's a knave and no mistake."
Marga      [Remark in village_smalltalk ]: "Thou dost not say it! Go on."
Aldric     [Remark in village_smalltalk ]: "And the herb-wife was seen with the tinker, so it's told."
Marga      [Agree in village_smalltalk ]: "Well, that's talk enough for one day."
Aldric     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Marga      [Farewell in village_smalltalk ]: "Fair day to thee, Aldric."
```

### Conversation #02: Aldric and Wystan
- **Speakers:** Aldric (lord) & Wystan (merchant)
- **Backstory A:** Aldric holds the manor's leases. They never miss a service.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Stranger
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Aldric     [Greet in village_smalltalk ]: "A fair day to thee, Wystan."
Wystan     [Remark in village_smalltalk ]: "Aye. What is it?"
Aldric     [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Wystan     [Question in village_smalltalk ]: "And how fares thy folk?"
Aldric     [Answer in village_smalltalk ]: "Well enough, thank thee."
Wystan     [Agree in village_smalltalk ]: "Long may it last."
Aldric     [Remark in village_smalltalk ]: "They say the miller's a knave and no mistake."
Wystan     [Remark in village_smalltalk ]: "Aye? And what else?"
Aldric     [Remark in village_smalltalk ]: "They do say the reeve counts twice."
Wystan     [Agree in village_smalltalk ]: "Well, that's talk enough for one day."
Aldric     [Remark in village_smalltalk ]: "As thou sayest."
Wystan     [Farewell in village_smalltalk ]: "Go well, Aldric."
```

### Conversation #03: Aldric and Marga
- **Speakers:** Aldric (lord) & Marga (merchant)
- **Backstory A:** Aldric holds the manor's leases. They never miss a service.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Stranger
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aldric     [Greet in village_smalltalk ]: "A fair day to thee, Marga."
Marga      [Remark in village_smalltalk ]: "Aye. What is it?"
Aldric     [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Marga      [Agree in village_smalltalk ]: "Aye, and the mud'll take thy boots."
Aldric     [Remark in village_smalltalk ]: "Even so, we'll manage."
Marga      [Farewell in village_smalltalk ]: "On thy way then, and mine."
```

### Conversation #04: Aldric and Hob
- **Speakers:** Aldric (lord) & Hob (merchant)
- **Backstory A:** Aldric holds the manor's leases. They never miss a service.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aldric     [Greet in visiting_the_sick ]: "How dost thou fare, Hob?"
Hob        [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Aldric     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Aldric     [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #05: Aldric and Marga
- **Speakers:** Aldric (lord) & Marga (merchant)
- **Backstory A:** Aldric holds the manor's leases. They never miss a service.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Rival
- **Trees Cascade:** ["old_scores"]
- **Ending:** WalksOff
- **Turns Count:** 6

```text
Aldric     [Greet in old_scores        ]: "Marga. Still about, I see."
Marga      [Greet in old_scores        ]: "Aldric."
Aldric     [Remark in old_scores        ]: "Thy lot still owes mine, whatever thou sayest."
Marga      [PushBack in old_scores        ]: "Say that again."
Aldric     [Remark in old_scores        ]: "Make me."
Marga      [PushBack in old_scores        ]: "Cur."
```

### Conversation #06: Eliza and Marga
- **Speakers:** Eliza (clergy) & Marga (merchant)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Eliza      [Greet in the_cleric_blesses]: "Blessings on thee, Marga."
Marga      [Greet in the_cleric_blesses]: "Blessings on thy house as well."
Eliza      [Remark in the_cleric_blesses]: "How fares thy house, child?"
Marga      [Farewell in the_cleric_blesses]: "I'll see thee at the bell, father."
```

### Conversation #07: Eliza and Gisela
- **Speakers:** Eliza (clergy) & Gisela (lord)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_visits_the_sick", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee."
Gisela     [Greet in the_cleric_visits_the_sick]: "And on thee, father."
Eliza      [Remark in the_cleric_visits_the_sick]: "How fares thy house, child?"
Gisela     [Agree in the_cleric_visits_the_sick]: "We keep as we can, father."
Eliza      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Farewell in visiting_the_sick ]: "Just a prayer, Eliza."
```

### Conversation #08: Eliza and Corvin
- **Speakers:** Eliza (clergy) & Corvin (clergy)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_blesses"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Eliza      [Greet in the_cleric_blesses]: "Blessings on thee, Corvin."
Corvin     [Greet in the_cleric_blesses]: "And on thee, father."
Eliza      [Remark in the_cleric_blesses]: "How fares thy house, child?"
Corvin     [Farewell in the_cleric_blesses]: "Pray for us, father."
```

### Conversation #09: Eliza and Hob
- **Speakers:** Eliza (clergy) & Hob (merchant)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Eliza      [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee."
Hob        [Remark in the_cleric_visits_the_sick]: "Morrow, father."
Eliza      [Remark in the_cleric_visits_the_sick]: "How fares thy house, child?"
Hob        [Farewell in the_cleric_visits_the_sick]: "Pray for us, father."
```

### Conversation #10: Eliza and Dagny
- **Speakers:** Eliza (clergy) & Dagny (merchant)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Eliza      [Greet in the_cleric_blesses]: "Blessings on thee, Dagny."
Dagny      [Remark in the_cleric_blesses]: "Morrow, father."
Eliza      [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Dagny      [Agree in the_cleric_blesses]: "The season's been lean, father."
Eliza      [Remark in village_smalltalk ]: "Gray clouds gathering over the hill."
Dagny      [Agree in village_smalltalk ]: "Aye, and the mud'll take thy boots."
Eliza      [Remark in village_smalltalk ]: "Aye, the year'll turn."
Dagny      [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Eliza      [Remark in village_smalltalk ]: "Folk saw the reeve's cart loaded past dark."
Dagny      [Remark in village_smalltalk ]: "Thou dost not say it! Go on."
Eliza      [Remark in village_smalltalk ]: "They do say the reeve counts twice."
Dagny      [Agree in village_smalltalk ]: "Ha! I'll believe the half of it."
Eliza      [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Dagny      [Farewell in village_smalltalk ]: "Go well, Eliza."
```

### Conversation #11: Bram and Dagny
- **Speakers:** Bram (lord) & Dagny (merchant)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Bram       [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Dagny      [PushBack in the_sickness      ]: "Keep thy distance, friend."
Bram       [Remark in the_sickness      ]: "As thou wilt, then."
Dagny      [Farewell in the_sickness      ]: "Another time, then."
```

### Conversation #12: Bram and Aldric
- **Speakers:** Bram (lord) & Aldric (lord)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Bram       [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Aldric     [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Bram       [Answer in the_sickness      ]: "There's no one else to fetch the water."
Aldric     [Farewell in the_sickness      ]: "Rest well, Bram."
```

### Conversation #13: Bram and Aldric
- **Speakers:** Bram (lord) & Aldric (lord)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Bram       [Greet in the_sickness      ]: "*cough* Good morrow, Aldric."
Aldric     [PushBack in the_sickness      ]: "Keep thy distance, friend."
Bram       [Remark in the_sickness      ]: "As thou wilt, then."
Aldric     [Farewell in the_sickness      ]: "Another time, then."
```

### Conversation #14: Bram and Wystan
- **Speakers:** Bram (lord) & Wystan (merchant)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "Morrow, Wystan. Don't stand too close."
Wystan     [Question in the_sickness      ]: "Half the village is coughing - is it that?"
Bram       [Answer in the_sickness      ]: "They say the herb-wife's draught helps."
Wystan     [Remark in the_sickness      ]: "Say a prayer and I'll bring broth."
Bram       [Remark in the_sickness      ]: "My thanks, Wystan."
Wystan     [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #15: Bram and Corvin
- **Speakers:** Bram (lord) & Corvin (clergy)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "Morrow, Corvin. Don't stand too close."
Corvin     [Question in the_sickness      ]: "Is it the sickness going round?"
Bram       [Answer in the_sickness      ]: "Aye, since the last rain. The fever comes and goes."
Corvin     [Remark in the_sickness      ]: "I'll fetch the herb-wife round."
Bram       [Remark in the_sickness      ]: "My thanks, Corvin."
Corvin     [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #16: Corvin and Aldric
- **Speakers:** Corvin (clergy) & Aldric (lord)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Corvin     [Greet in the_cleric_blesses]: "Blessings on thee, Aldric."
Aldric     [Greet in the_cleric_blesses]: "Blessings on thy house as well."
Corvin     [Remark in the_cleric_blesses]: "How fares thy house, child?"
Aldric     [Agree in the_cleric_blesses]: "We keep as we can, father."
Corvin     [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Aldric     [Agree in village_smalltalk ]: "It'll be a wet week, mark me."
Corvin     [Remark in village_smalltalk ]: "Even so, we'll manage."
Aldric     [Agree in village_smalltalk ]: "Aye, we've seen worse."
Corvin     [Remark in village_smalltalk ]: "Didst thou hear who was seen down by the mill?"
Aldric     [PushBack in village_smalltalk ]: "I'll not repeat market talk."
Corvin     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Aldric     [Farewell in village_smalltalk ]: "Go well, Corvin."
```

### Conversation #17: Corvin and Aelfric
- **Speakers:** Corvin (clergy) & Aelfric (clergy)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Corvin     [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Aelfric    [Greet in the_cleric_blesses]: "And on thee, father."
Corvin     [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Aelfric    [Farewell in the_cleric_blesses]: "I'll see thee at the bell, father."
```

### Conversation #18: Corvin and Gisela
- **Speakers:** Corvin (clergy) & Gisela (lord)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_visits_the_sick", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in the_cleric_visits_the_sick]: "Blessings on thee, Gisela."
Gisela     [Greet in the_cleric_visits_the_sick]: "And on thee, father."
Corvin     [Remark in the_cleric_visits_the_sick]: "How fares thy house, child?"
Gisela     [Agree in the_cleric_visits_the_sick]: "We keep as we can, father."
Corvin     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Farewell in visiting_the_sick ]: "Just a prayer, Corvin."
```

### Conversation #19: Corvin and Hob
- **Speakers:** Corvin (clergy) & Hob (merchant)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in the_cleric_visits_the_sick]: "Blessings on thee, Hob."
Hob        [Question in the_cleric_visits_the_sick]: "Father, pray for my house."
Corvin     [Question in the_cleric_visits_the_sick]: "The Lord comfort thee, child. Tell me of thy affliction."
Hob        [Answer in the_cleric_visits_the_sick]: "It's the little one mostly - she burns with fever."
Corvin     [Remark in the_cleric_visits_the_sick]: "The saints watch over thy house. I will come by with draughts and broth."
Hob        [Farewell in the_cleric_visits_the_sick]: "The Lord keep thee, father."
```

### Conversation #20: Corvin and Aldric
- **Speakers:** Corvin (clergy) & Aldric (lord)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in the_cleric_blesses]: "Blessings on thee, Aldric."
Aldric     [Greet in the_cleric_blesses]: "And on thee, father."
Corvin     [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Aldric     [Agree in the_cleric_blesses]: "The season's been lean, father."
Corvin     [Remark in village_smalltalk ]: "Gray clouds gathering over the hill."
Aldric     [Farewell in village_smalltalk ]: "I should get on. Fair day to thee."
```

### Conversation #21: Dagny and Marga
- **Speakers:** Dagny (merchant) & Marga (merchant)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Rival
- **Trees Cascade:** ["old_scores"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in old_scores        ]: "You."
Marga      [PushBack in old_scores        ]: "Still breathing, I see. Pity."
Dagny      [Remark in old_scores        ]: "Make me."
Marga      [Remark in old_scores        ]: "Not worth the breath."
Dagny      [Remark in old_scores        ]: "We're done here."
Marga      [Farewell in old_scores        ]: "Aye. Done."
```

### Conversation #22: Dagny and Bram
- **Speakers:** Dagny (merchant) & Bram (lord)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Dagny      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Bram       [Answer in visiting_the_sick ]: "Weak as a kitten, but here I am."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Farewell in visiting_the_sick ]: "Just a prayer, Dagny."
```

### Conversation #23: Dagny and Corvin
- **Speakers:** Dagny (merchant) & Corvin (clergy)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Relation:** Acquaintance
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in asking_the_father ]: "Good morrow, father."
Corvin     [Greet in asking_the_father ]: "The saints keep thee, Dagny. What is it?"
Dagny      [Question in asking_the_father ]: "Must the tithe stand full when the store is half empty, father?"
Corvin     [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor, child."
Dagny      [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Corvin     [Farewell in asking_the_father ]: "Go in peace, Dagny."
```

### Conversation #24: Dagny and Aelfric
- **Speakers:** Dagny (merchant) & Aelfric (clergy)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in asking_the_father ]: "Good morrow, father."
Aelfric    [Greet in asking_the_father ]: "Always, child. Speak."
Dagny      [Question in asking_the_father ]: "Can the parish spare a little grain for the lean weeks, father?"
Aelfric    [Answer in asking_the_father ]: "The church will share what it has, child - none shall starve at the door."
Dagny      [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Aelfric    [Farewell in asking_the_father ]: "Go in peace, Dagny."
```

### Conversation #25: Dagny and Hob
- **Speakers:** Dagny (merchant) & Hob (merchant)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in visiting_the_sick ]: "How dost thou fare, Hob?"
Hob        [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Dagny      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #26: Hob and Wystan
- **Speakers:** Hob (merchant) & Wystan (merchant)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Hob        [Greet in the_sickness      ]: "*cough* Good morrow, Wystan."
Wystan     [Question in the_sickness      ]: "Is it the sickness going round?"
Hob        [Answer in the_sickness      ]: "Aye, since the last rain. The fever comes and goes."
Wystan     [Agree in the_sickness      ]: "The saints speed thy mending."
Hob        [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Wystan     [PushBack in village_smalltalk ]: "Thou seest storms in a clear sky."
Hob        [Remark in village_smalltalk ]: "Even so, we'll manage."
Wystan     [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Hob        [Remark in village_smalltalk ]: "Folk saw the reeve's cart loaded past dark."
Wystan     [Remark in village_smalltalk ]: "Aye? And what else?"
Hob        [Remark in village_smalltalk ]: "And the herb-wife was seen with the tinker, so it's told."
Wystan     [Agree in village_smalltalk ]: "Ha! I'll believe the half of it."
Hob        [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Wystan     [Farewell in village_smalltalk ]: "Go well, Hob."
```

### Conversation #27: Hob and Aldric
- **Speakers:** Hob (merchant) & Aldric (lord)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_sickness      ]: "*cough* Good morrow, Aldric."
Aldric     [Question in the_sickness      ]: "Half the village is coughing - is it that?"
Hob        [Answer in the_sickness      ]: "They say the herb-wife's draught helps."
Aldric     [Remark in the_sickness      ]: "Say a prayer and I'll bring broth."
Hob        [Remark in the_sickness      ]: "My thanks, Aldric."
Aldric     [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #28: Hob and Aldric
- **Speakers:** Hob (merchant) & Aldric (lord)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Hob        [Greet in the_sickness      ]: "*cough* Good morrow, Aldric."
Aldric     [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Hob        [Answer in the_sickness      ]: "There's no one else to fetch the water."
Aldric     [Agree in the_sickness      ]: "We rub along or we don't eat, that's the truth of it."
Hob        [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Aldric     [Question in village_smalltalk ]: "And how fares thy folk?"
Hob        [Answer in village_smalltalk ]: "Well enough, thank thee."
Aldric     [Farewell in village_smalltalk ]: "Give them my best. Fare well."
```

### Conversation #29: Hob and Aldric
- **Speakers:** Hob (merchant) & Aldric (lord)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** WalksOff
- **Turns Count:** 4

```text
Hob        [Greet in the_sickness      ]: "Morrow, Aldric. Don't stand too close."
Aldric     [Question in the_sickness      ]: "Is it the sickness going round?"
Hob        [Answer in the_sickness      ]: "Aye, since the last rain. The fever comes and goes."
Aldric     [Farewell in the_sickness      ]: "I'll be off before it takes me too."
```

### Conversation #30: Hob and Bram
- **Speakers:** Hob (merchant) & Bram (lord)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Bram       [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Hob        [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Bram       [Remark in the_sickness      ]: "I'll send soup round this evening."
Hob        [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Bram       [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #31: Marga and Eliza
- **Speakers:** Marga (merchant) & Eliza (clergy)
- **Backstory A:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in asking_the_father ]: "Good morrow, father."
Eliza      [Greet in asking_the_father ]: "The saints keep thee, Marga. What is it?"
Marga      [Question in asking_the_father ]: "Can the parish spare a little grain for the lean weeks, father?"
Eliza      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor, child."
Marga      [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Eliza      [Farewell in asking_the_father ]: "Go in peace, Marga."
```

### Conversation #32: Marga and Eliza
- **Speakers:** Marga (merchant) & Eliza (clergy)
- **Backstory A:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in asking_the_father ]: "Good morrow, father."
Eliza      [Greet in asking_the_father ]: "Always, child. Speak."
Marga      [Question in asking_the_father ]: "Must the tithe stand full when the store is half empty, father?"
Eliza      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor, child."
Marga      [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Eliza      [Farewell in asking_the_father ]: "Go in peace, Marga."
```

### Conversation #33: Marga and Wystan
- **Speakers:** Marga (merchant) & Wystan (merchant)
- **Backstory A:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Friend
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in village_smalltalk ]: "Good morrow to thee, Wystan."
Wystan     [Remark in village_smalltalk ]: "Aye. What is it?"
Marga      [Remark in village_smalltalk ]: "Gray clouds gathering over the hill."
Wystan     [Agree in village_smalltalk ]: "The thatch won't thank it."
Marga      [Remark in village_smalltalk ]: "Even so, we'll manage."
Wystan     [Farewell in village_smalltalk ]: "I'll not keep thee. Go well."
```

### Conversation #34: Marga and Gisela
- **Speakers:** Marga (merchant) & Gisela (lord)
- **Backstory A:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Gisela     [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Marga      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Marga      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Gisela     [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #35: Marga and Aelfric
- **Speakers:** Marga (merchant) & Aelfric (clergy)
- **Backstory A:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in asking_the_father ]: "A word with thee, father, if thou'lt spare it."
Aelfric    [Greet in asking_the_father ]: "The saints keep thee, Marga. What is it?"
Marga      [Question in asking_the_father ]: "Must the tithe stand full when the store is half empty, father?"
Aelfric    [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor, child."
Marga      [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Aelfric    [Farewell in asking_the_father ]: "Go in peace, Marga."
```

### Conversation #36: Gisela and Eliza
- **Speakers:** Gisela (lord) & Eliza (clergy)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Backstory B:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Morrow, Eliza. Don't stand too close."
Eliza      [Question in the_sickness      ]: "Is it the sickness going round?"
Gisela     [Answer in the_sickness      ]: "They say the herb-wife's draught helps."
Eliza      [Remark in the_sickness      ]: "I'll fetch the herb-wife round."
Gisela     [Remark in the_sickness      ]: "My thanks, Eliza."
Eliza      [Farewell in the_sickness      ]: "We look after our own."
```

### Conversation #37: Gisela and Marga
- **Speakers:** Gisela (lord) & Marga (merchant)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Marga      [Question in the_sickness      ]: "Half the village is coughing - is it that?"
Gisela     [Answer in the_sickness      ]: "Aye, since the last rain. The fever comes and goes."
Marga      [Remark in the_sickness      ]: "I'll fetch the herb-wife round."
Gisela     [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Marga      [Farewell in the_sickness      ]: "We look after our own."
```

### Conversation #38: Gisela and Corvin
- **Speakers:** Gisela (lord) & Corvin (clergy)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Backstory B:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Corvin     [Question in the_sickness      ]: "Half the village is coughing - is it that?"
Gisela     [Answer in the_sickness      ]: "They say the herb-wife's draught helps."
Corvin     [Remark in the_sickness      ]: "I'll fetch the herb-wife round."
Gisela     [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Corvin     [Farewell in the_sickness      ]: "We look after our own."
```

### Conversation #39: Gisela and Wystan
- **Speakers:** Gisela (lord) & Wystan (merchant)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Gisela     [Greet in the_sickness      ]: "Morrow, Wystan. Don't stand too close."
Wystan     [Agree in the_sickness      ]: "Thou shouldst be by the fire, not out here."
Gisela     [Answer in the_sickness      ]: "There's no one else to fetch the water."
Wystan     [Agree in the_sickness      ]: "We rub along or we don't eat, that's the truth of it."
Gisela     [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Wystan     [Agree in village_smalltalk ]: "The thatch won't thank it."
Gisela     [Remark in village_smalltalk ]: "Aye, the year'll turn."
Wystan     [Farewell in village_smalltalk ]: "On thy way then, and mine."
```

### Conversation #40: Gisela and Bram
- **Speakers:** Gisela (lord) & Bram (lord)
- **Backstory A:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "*cough* Good morrow, Bram."
Bram       [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Gisela     [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Bram       [Agree in the_sickness      ]: "We rub along or we don't eat, that's the truth of it."
Gisela     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Farewell in visiting_the_sick ]: "Just a prayer, Gisela."
```

### Conversation #41: Wystan and Aldric
- **Speakers:** Wystan (merchant) & Aldric (lord)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Friend
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Wystan     [Greet in village_smalltalk ]: "A fair day to thee, Aldric."
Aldric     [Remark in village_smalltalk ]: "Morrow."
Wystan     [Remark in village_smalltalk ]: "Gray clouds gathering over the hill."
Aldric     [Agree in village_smalltalk ]: "Aye, and the mud'll take thy boots."
Wystan     [Remark in village_smalltalk ]: "Even so, we'll manage."
Aldric     [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Wystan     [Remark in village_smalltalk ]: "Didst thou hear who was seen down by the mill?"
Aldric     [Remark in village_smalltalk ]: "Aye? And what else?"
Wystan     [Remark in village_smalltalk ]: "They do say the reeve counts twice."
Aldric     [Agree in village_smalltalk ]: "Well, that's talk enough for one day."
Wystan     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Aldric     [Farewell in village_smalltalk ]: "Fair day to thee, Wystan."
```

### Conversation #42: Wystan and Bram
- **Speakers:** Wystan (merchant) & Bram (lord)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Wystan     [Greet in visiting_the_sick ]: "How dost thou fare, Bram?"
Bram       [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Wystan     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Farewell in visiting_the_sick ]: "Just a prayer, Wystan."
```

### Conversation #43: Wystan and Gisela
- **Speakers:** Wystan (merchant) & Gisela (lord)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Wystan     [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Gisela     [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Wystan     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Farewell in visiting_the_sick ]: "Just a prayer, Wystan."
```

### Conversation #44: Wystan and Aldric
- **Speakers:** Wystan (merchant) & Aldric (lord)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Kin
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Wystan     [Greet in village_smalltalk ]: "A fair day to thee, Aldric."
Aldric     [Greet in village_smalltalk ]: "And to thee, Wystan."
Wystan     [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Aldric     [Agree in village_smalltalk ]: "The thatch won't thank it."
Wystan     [Remark in village_smalltalk ]: "Even so, we'll manage."
Aldric     [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Wystan     [Remark in village_smalltalk ]: "They say the miller's a knave and no mistake."
Aldric     [Remark in village_smalltalk ]: "Thou dost not say it! Go on."
Wystan     [Remark in village_smalltalk ]: "And the herb-wife was seen with the tinker, so it's told."
Aldric     [Agree in village_smalltalk ]: "Well, that's talk enough for one day."
Wystan     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Aldric     [Farewell in village_smalltalk ]: "Go well, Wystan."
```

### Conversation #45: Wystan and Hob
- **Speakers:** Wystan (merchant) & Hob (merchant)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Wystan     [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Hob        [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Wystan     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Wystan     [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #46: Aelfric and Corvin
- **Speakers:** Aelfric (clergy) & Corvin (clergy)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Relation:** Friend
- **Trees Cascade:** ["the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Aelfric    [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Corvin     [Greet in the_cleric_blesses]: "And on thee, father."
Aelfric    [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Corvin     [Agree in the_cleric_blesses]: "We keep as we can, father."
Aelfric    [Question in asking_the_father ]: "Can the parish spare a little grain for the lean weeks, father?"
Corvin     [Answer in asking_the_father ]: "The church will share what it has, child - none shall starve at the door."
Aelfric    [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Corvin     [Farewell in asking_the_father ]: "Go in peace, Aelfric."
```

### Conversation #47: Aelfric and Hob
- **Speakers:** Aelfric (clergy) & Hob (merchant)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Aelfric    [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee."
Hob        [Greet in the_cleric_visits_the_sick]: "And on thee, father."
Aelfric    [Remark in the_cleric_visits_the_sick]: "Dost thou find peace in thy labor, child?"
Hob        [Farewell in the_cleric_visits_the_sick]: "Pray for us, father."
```

### Conversation #48: Aelfric and Aldric
- **Speakers:** Aelfric (clergy) & Aldric (lord)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Aelfric    [Greet in the_cleric_blesses]: "Blessings on thee, Aldric."
Aldric     [Greet in the_cleric_blesses]: "And on thee, father."
Aelfric    [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Aldric     [Agree in the_cleric_blesses]: "We keep as we can, father."
Aelfric    [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Aldric     [Agree in village_smalltalk ]: "Aye, and the mud'll take thy boots."
Aelfric    [Remark in village_smalltalk ]: "Aye, the year'll turn."
Aldric     [Agree in village_smalltalk ]: "Aye, we've seen worse."
Aelfric    [Remark in village_smalltalk ]: "They say the miller's a knave and no mistake."
Aldric     [Farewell in village_smalltalk ]: "A tale for another day - I must away."
```

### Conversation #49: Aelfric and Wystan
- **Speakers:** Aelfric (clergy) & Wystan (merchant)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_blesses"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Aelfric    [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Wystan     [Greet in the_cleric_blesses]: "Blessings on thy house as well."
Aelfric    [Remark in the_cleric_blesses]: "How fares thy house, child?"
Wystan     [Farewell in the_cleric_blesses]: "I'll see thee at the bell, father."
```

### Conversation #50: Aelfric and Hob
- **Speakers:** Aelfric (clergy) & Hob (merchant)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aelfric    [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee."
Hob        [Question in the_cleric_visits_the_sick]: "A prayer for me, father - this cough won't quit."
Aelfric    [Question in the_cleric_visits_the_sick]: "Speak, child - my prayers and candles are with thee."
Hob        [Answer in the_cleric_visits_the_sick]: "This cough won't leave me, father, and the store is thin."
Aelfric    [Remark in the_cleric_visits_the_sick]: "The saints watch over thy house. I will come by with draughts and broth."
Hob        [Farewell in the_cleric_visits_the_sick]: "The Lord keep thee, father."
```

## Benchmark Summary

- Total Characters Generated: 10
- Total Conversations Generated: 50
- Generator Seed: 42

Benchmark generation complete.
