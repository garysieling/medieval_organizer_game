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
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Gisela     [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Eliza      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Eliza      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Gisela     [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
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
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Hob        [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Eliza      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Eliza      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #10: Eliza and Hob
- **Speakers:** Eliza (clergy) & Hob (merchant)
- **Backstory A:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Hob        [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Eliza      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Eliza      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #11: Bram and Eliza
- **Speakers:** Bram (lord) & Eliza (clergy)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** WalksOff
- **Turns Count:** 4

```text
Bram       [Greet in the_sickness      ]: "*cough* Good morrow, Eliza."
Eliza      [PushBack in the_sickness      ]: "Keep thy distance, friend."
Bram       [Remark in the_sickness      ]: "As thou wilt, then."
Eliza      [PushBack in the_sickness      ]: "A fine friend thou art."
```

### Conversation #12: Bram and Gisela
- **Speakers:** Bram (lord) & Gisela (lord)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "*cough* Good morrow, Gisela."
Gisela     [Agree in the_sickness      ]: "Thou shouldst be by the fire, not out here."
Bram       [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Gisela     [Remark in the_sickness      ]: "I'll send soup round this evening."
Bram       [Remark in the_sickness      ]: "My thanks, Gisela."
Gisela     [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #13: Bram and Aldric
- **Speakers:** Bram (lord) & Aldric (lord)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Friend
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Aldric     [Agree in the_sickness      ]: "Thou shouldst be by the fire, not out here."
Bram       [Answer in the_sickness      ]: "There's no one else to fetch the water."
Aldric     [Remark in the_sickness      ]: "I'll send soup round this evening."
Bram       [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Aldric     [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #14: Bram and Hob
- **Speakers:** Bram (lord) & Hob (merchant)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "Morrow, Hob. Don't stand too close."
Hob        [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Bram       [Answer in the_sickness      ]: "There's no one else to fetch the water."
Hob        [Remark in the_sickness      ]: "I'll send soup round this evening."
Bram       [Remark in the_sickness      ]: "My thanks, Hob."
Hob        [Farewell in the_sickness      ]: "We look after our own."
```

### Conversation #15: Bram and Hob
- **Speakers:** Bram (lord) & Hob (merchant)
- **Backstory A:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in the_sickness      ]: "Morrow, Hob. Don't stand too close."
Hob        [Agree in the_sickness      ]: "Thou shouldst be by the fire, not out here."
Bram       [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Hob        [Remark in the_sickness      ]: "I'll send soup round this evening."
Bram       [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Hob        [Farewell in the_sickness      ]: "Think nothing of it."
```

### Conversation #16: Corvin and Wystan
- **Speakers:** Corvin (clergy) & Wystan (merchant)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Corvin     [Greet in the_cleric_blesses]: "Blessings on thee, Wystan."
Wystan     [Remark in the_cleric_blesses]: "Morrow, father."
Corvin     [Remark in the_cleric_blesses]: "How fares thy house, child?"
Wystan     [Agree in the_cleric_blesses]: "The season's been lean, father."
Corvin     [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Wystan     [PushBack in village_smalltalk ]: "Rain? It's the driest year in memory."
Corvin     [Remark in village_smalltalk ]: "Even so, we'll manage."
Wystan     [Agree in village_smalltalk ]: "That it will - the store's not empty yet."
Corvin     [Remark in village_smalltalk ]: "Folk saw the reeve's cart loaded past dark."
Wystan     [Remark in village_smalltalk ]: "Aye? And what else?"
Corvin     [Remark in village_smalltalk ]: "And the herb-wife was seen with the tinker, so it's told."
Wystan     [Agree in village_smalltalk ]: "Ha! I'll believe the half of it."
Corvin     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Wystan     [Farewell in village_smalltalk ]: "Go well, Corvin."
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
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in visiting_the_sick ]: "How dost thou fare, Gisela?"
Gisela     [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Corvin     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Corvin     [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Gisela     [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #19: Corvin and Hob
- **Speakers:** Corvin (clergy) & Hob (merchant)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Corvin     [Greet in visiting_the_sick ]: "How dost thou fare, Hob?"
Hob        [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Corvin     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Farewell in visiting_the_sick ]: "Just a prayer, Corvin."
```

### Conversation #20: Corvin and Aelfric
- **Speakers:** Corvin (clergy) & Aelfric (clergy)
- **Backstory A:** Corvin serves the village church. Their temper's shorter than a winter's day. They never miss a service. They'd sooner watch the clouds than the plough. The store ran out and the belly knows it.
- **Backstory B:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Corvin     [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Aelfric    [Remark in the_cleric_blesses]: "Morrow, father."
Corvin     [Remark in the_cleric_blesses]: "How fares thy house, child?"
Aelfric    [Farewell in the_cleric_blesses]: "I'll see thee at the bell, father."
```

### Conversation #21: Dagny and Bram
- **Speakers:** Dagny (merchant) & Bram (lord)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Bram       [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Dagny      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Bram       [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #22: Dagny and Aldric
- **Speakers:** Dagny (merchant) & Aldric (lord)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Aldric holds the manor's leases. They never miss a service.
- **Relation:** Friend
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Dagny      [Greet in village_smalltalk ]: "Good morrow to thee, Aldric."
Aldric     [Greet in village_smalltalk ]: "The saints keep thee, Dagny."
Dagny      [Remark in village_smalltalk ]: "They say the harvest'll be thin this year."
Aldric     [Farewell in village_smalltalk ]: "I should get on. Fair day to thee."
```

### Conversation #23: Dagny and Marga
- **Speakers:** Dagny (merchant) & Marga (merchant)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Acquaintance
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Dagny      [Greet in village_smalltalk ]: "A fair day to thee, Marga."
Marga      [Remark in village_smalltalk ]: "Aye. What is it?"
Dagny      [Remark in village_smalltalk ]: "They say the harvest'll be thin this year."
Marga      [Question in village_smalltalk ]: "And how fares thy folk?"
Dagny      [Answer in village_smalltalk ]: "We rub along, as ever."
Marga      [Agree in village_smalltalk ]: "Good to hear it."
Dagny      [Remark in village_smalltalk ]: "They say the miller's a knave and no mistake."
Marga      [Remark in village_smalltalk ]: "Thou dost not say it! Go on."
Dagny      [Remark in village_smalltalk ]: "They do say the reeve counts twice."
Marga      [Agree in village_smalltalk ]: "Well, that's talk enough for one day."
Dagny      [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Marga      [Farewell in village_smalltalk ]: "Fair day to thee, Dagny."
```

### Conversation #24: Dagny and Bram
- **Speakers:** Dagny (merchant) & Bram (lord)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Bram holds the manor's leases. They know every soul and every soul's business. They never miss a service. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Bram       [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Dagny      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Bram       [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #25: Dagny and Hob
- **Speakers:** Dagny (merchant) & Hob (merchant)
- **Backstory A:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Backstory B:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Relation:** Friend
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Hob        [Answer in visiting_the_sick ]: "Poorly, if thou'lt have the truth."
Dagny      [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Dagny      [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Hob        [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #26: Hob and Eliza
- **Speakers:** Hob (merchant) & Eliza (clergy)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Eliza serves the village church. They never miss a service. They bow to no one. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Hob        [Greet in the_sickness      ]: "Morrow, Eliza. Don't stand too close."
Eliza      [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Hob        [Answer in the_sickness      ]: "There's no one else to fetch the water."
Eliza      [Agree in the_sickness      ]: "We rub along or we don't eat, that's the truth of it."
Hob        [Greet in asking_the_father ]: "Good morrow, father."
Eliza      [Greet in asking_the_father ]: "The saints keep thee, Hob. What is it?"
Hob        [Question in asking_the_father ]: "Is it true the dead walk in dreams to warn us?"
Eliza      [Answer in asking_the_father ]: "The tithe stands - but I'll speak for thee at the manor."
Hob        [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Eliza      [Farewell in asking_the_father ]: "Go in peace, Hob."
```

### Conversation #27: Hob and Gisela
- **Speakers:** Hob (merchant) & Gisela (lord)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_sickness      ]: "Morrow, Gisela. Don't stand too close."
Gisela     [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Hob        [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Gisela     [Remark in the_sickness      ]: "I'll send soup round this evening."
Hob        [Remark in the_sickness      ]: "My thanks, Gisela."
Gisela     [Farewell in the_sickness      ]: "We look after our own."
```

### Conversation #28: Hob and Marga
- **Speakers:** Hob (merchant) & Marga (merchant)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Hob        [Greet in the_sickness      ]: "Forgive the cough - it won't quit."
Marga      [Question in the_sickness      ]: "Half the village is coughing - is it that?"
Hob        [Answer in the_sickness      ]: "Aye, since the last rain. The fever comes and goes."
Marga      [Agree in the_sickness      ]: "The saints speed thy mending."
Hob        [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Marga      [Agree in village_smalltalk ]: "The thatch won't thank it."
Hob        [Remark in village_smalltalk ]: "Even so, we'll manage."
Marga      [Farewell in village_smalltalk ]: "I'll not keep thee. Go well."
```

### Conversation #29: Hob and Marga
- **Speakers:** Hob (merchant) & Marga (merchant)
- **Backstory A:** Hob trades on the road between villages. They bow to no one. They mean to be more than they were born. A cough's had them these two weeks.
- **Backstory B:** Marga trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_sickness      ]: "Morrow, Marga. Don't stand too close."
Marga      [Question in the_sickness      ]: "Is it the sickness going round?"
Hob        [Answer in the_sickness      ]: "They say the herb-wife's draught helps."
Marga      [Remark in the_sickness      ]: "I'll fetch the herb-wife round."
Hob        [Remark in the_sickness      ]: "Thou'rt kind. The saints remember it."
Marga      [Farewell in the_sickness      ]: "We look after our own."
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
Marga      [Question in asking_the_father ]: "Must the miller's tithe stand when the mill's half idle?"
Eliza      [Answer in asking_the_father ]: "The dreams of the grieving are their own counsel, child."
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
Marga      [Question in asking_the_father ]: "Is it true the dead walk in dreams to warn us?"
Eliza      [Answer in asking_the_father ]: "The dreams of the grieving are their own counsel, child."
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
Marga      [Remark in village_smalltalk ]: "They say the harvest'll be thin this year."
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
Marga      [Question in asking_the_father ]: "Is it true the dead walk in dreams to warn us?"
Aelfric    [Answer in asking_the_father ]: "The dreams of the grieving are their own counsel, child."
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
- **Turns Count:** 8

```text
Gisela     [Greet in the_sickness      ]: "*cough* Good morrow, Bram."
Bram       [Agree in the_sickness      ]: "Poor thing - get thee to bed."
Gisela     [Answer in the_sickness      ]: "I'd rest if the work'd wait."
Bram       [Agree in the_sickness      ]: "We rub along or we don't eat, that's the truth of it."
Gisela     [Greet in visiting_the_sick ]: "How dost thou fare, Bram?"
Bram       [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Gisela     [Question in visiting_the_sick ]: "Is there aught thou needest?"
Bram       [Farewell in visiting_the_sick ]: "Just a prayer, Gisela."
```

### Conversation #41: Wystan and Dagny
- **Speakers:** Wystan (merchant) & Dagny (merchant)
- **Backstory A:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Backstory B:** Dagny trades on the road between villages. They mean to be more than they were born. The store ran out and the belly knows it.
- **Relation:** Stranger
- **Trees Cascade:** ["village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Wystan     [Greet in village_smalltalk ]: "A fair day to thee, Dagny."
Dagny      [Greet in village_smalltalk ]: "The saints keep thee, Wystan."
Wystan     [Remark in village_smalltalk ]: "Looks like rain again, doesn't it?"
Dagny      [Agree in village_smalltalk ]: "Aye, and the mud'll take thy boots."
Wystan     [Remark in village_smalltalk ]: "Even so, we'll manage."
Dagny      [Agree in village_smalltalk ]: "Aye, we've seen worse."
Wystan     [Remark in village_smalltalk ]: "Folk saw the reeve's cart loaded past dark."
Dagny      [PushBack in village_smalltalk ]: "Idle tongues, that."
Wystan     [Remark in village_smalltalk ]: "True enough - the fields don't gossip."
Dagny      [Farewell in village_smalltalk ]: "Fair day to thee, Wystan."
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
- **Turns Count:** 10

```text
Aelfric    [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Corvin     [Greet in the_cleric_blesses]: "And on thee, father."
Aelfric    [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Corvin     [Agree in the_cleric_blesses]: "We keep as we can, father."
Aelfric    [Greet in asking_the_father ]: "A word with thee, father, if thou'lt spare it."
Corvin     [Greet in asking_the_father ]: "Always, child. Speak."
Aelfric    [Question in asking_the_father ]: "Is it true the dead walk in dreams to warn us?"
Corvin     [Answer in asking_the_father ]: "The tithe stands - but I'll speak for thee at the manor."
Aelfric    [Remark in asking_the_father ]: "Thank thee, father - that's a weight off."
Corvin     [Farewell in asking_the_father ]: "Go in peace, Aelfric."
```

### Conversation #47: Aelfric and Gisela
- **Speakers:** Aelfric (clergy) & Gisela (lord)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Gisela holds the manor's leases. They mean to be more than they were born. They never miss a service. They know every soul and every soul's business. A cough's had them these two weeks.
- **Relation:** Friend
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aelfric    [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Gisela     [Answer in visiting_the_sick ]: "Weak as a kitten, but here I am."
Aelfric    [Question in visiting_the_sick ]: "Is there aught thou needest?"
Gisela     [Answer in visiting_the_sick ]: "A little bread, if thou canst spare it."
Aelfric    [Remark in visiting_the_sick ]: "It's done - look for it by dark."
Gisela     [Farewell in visiting_the_sick ]: "Thou'rt a true neighbor."
```

### Conversation #48: Aelfric and Wystan
- **Speakers:** Aelfric (clergy) & Wystan (merchant)
- **Backstory A:** Aelfric serves the village church. They know every soul and every soul's business. They're the first with a loaf when a house is bare. Last night's cup is still on them.
- **Backstory B:** Wystan trades on the road between villages. They mean to be more than they were born. They'd sooner watch the clouds than the plough.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses", "village_smalltalk"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aelfric    [Greet in the_cleric_blesses]: "The peace of the saints be with thee."
Wystan     [Greet in the_cleric_blesses]: "Blessings on thy house as well."
Aelfric    [Remark in the_cleric_blesses]: "Dost thou find peace in thy labor, child?"
Wystan     [Agree in the_cleric_blesses]: "The season's been lean, father."
Aelfric    [Remark in village_smalltalk ]: "Cold wind off the moor this morning."
Wystan     [Farewell in village_smalltalk ]: "I should get on. Fair day to thee."
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
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 4

```text
Aelfric    [Greet in visiting_the_sick ]: "I heard the sickness took thee - how is it?"
Hob        [PushBack in visiting_the_sick ]: "It'll pass. Don't fuss."
Aelfric    [Question in visiting_the_sick ]: "Is there aught thou needest?"
Hob        [Farewell in visiting_the_sick ]: "Just a prayer, Aelfric."
```

## Benchmark Summary

- Total Characters Generated: 10
- Total Conversations Generated: 50
- Generator Seed: 42

Benchmark generation complete.
