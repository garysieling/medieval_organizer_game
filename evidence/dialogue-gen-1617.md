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
- **Ideology:** merchant
- **Traits:** [Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "baker", "hungry"]
- **Facts:** archetype:merrymaker, value:standing, value:custom
- **Backstory:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.

### Character #3: Bram
- **Ideology:** merchant
- **Traits:** [Hotheaded, Stingy]
- **Status:** ["wx-fair", "season:autumn", "market"]
- **Facts:** archetype:firebrand, value:adventure, value:advancement
- **Backstory:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.

### Character #4: Corvin
- **Ideology:** lord
- **Traits:** [Kind, Gregarious, Hotheaded]
- **Status:** ["wx-fair", "season:autumn", "market", "shepherd"]
- **Facts:** archetype:soft_heart, value:freedom, value:fairness
- **Backstory:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.

### Character #5: Dagny
- **Ideology:** clergy
- **Traits:** [Gregarious]
- **Status:** ["wx-fair", "season:autumn", "market", "brewer", "hungry"]
- **Facts:** archetype:tongue_of_the_well, value:care, value:fairness
- **Backstory:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.

### Character #6: Hob
- **Ideology:** clergy
- **Traits:** [Kind, Stingy, Ambitious]
- **Status:** ["wx-fair", "season:autumn", "market", "smith", "drunk"]
- **Facts:** archetype:soft_heart, value:freedom, value:safety
- **Backstory:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.

### Character #7: Marga
- **Ideology:** laborer
- **Traits:** [Kind, Hotheaded, Proud]
- **Status:** ["wx-fair", "season:autumn", "market", "weaver"]
- **Facts:** archetype:soft_heart, value:safety, value:freedom
- **Backstory:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.

### Character #8: Gisela
- **Ideology:** laborer
- **Traits:** [Kind, Ambitious]
- **Status:** ["wx-fair", "season:autumn", "market", "weaver", "sick"]
- **Facts:** archetype:soft_heart, value:custom, value:care
- **Backstory:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.

### Character #9: Wystan
- **Ideology:** laborer
- **Traits:** [Ambitious, Hotheaded, Gregarious]
- **Status:** ["wx-fair", "season:autumn", "market", "grieving"]
- **Facts:** archetype:striver, value:freedom, value:custom
- **Backstory:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.

### Character #10: Aelfric
- **Ideology:** laborer
- **Traits:** [Hotheaded, Stingy, Lazy]
- **Status:** ["wx-fair", "season:autumn", "market", "printer", "grieving"]
- **Facts:** archetype:firebrand, value:custom, value:propriety
- **Backstory:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.

---

## Generated Conversations (N x M = 10 x 5 = 50 Total)

### Conversation #01: Aldric and Dagny
- **Speakers:** Aldric (clergy) & Dagny (clergy)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Relation:** Stranger
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 11

```text
Aldric     [Greet in a_death_in_the_house]: "Morrow, stranger. Forgive a grieving house its silence - as the mill grinds."
Dagny      [Question in a_death_in_the_house]: "Why, Do you have folk to help you?"
Aldric     [Answer in a_death_in_the_house]: "All things ripen that are let be, father."
Aldric     [Remark in the_cleric_blesses]: "The church altar is always open to the weary soul."
Dagny      [Agree in the_cleric_blesses]: "Aye, there is truth in what you say - so it goes."
Aldric     [Question in asking_the_father ]: "Can the church spare a peck of grain for the hungry children, father?"
Dagny      [Evade in asking_the_father ]: "We don't know each other so well as that."
Aldric     [Remark in asking_the_father ]: "Your blessing gives me heart to face the coming week, father!"
Dagny      [Agree in asking_the_father ]: "The parish is lucky in its father!"
Aldric     [Remark in asking_the_father ]: "Aye, words won't fill the storehouse."
Dagny      [Farewell in asking_the_father ]: "Enough chatter. Good day, or am I mistook?"
```

### Conversation #02: Aldric and Gisela
- **Speakers:** Aldric (clergy) & Gisela (laborer)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["the_cleric_visits_the_sick", "visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Aldric     [Greet in the_cleric_visits_the_sick]: "A peaceful day to you, Gisela - as the mill grinds."
Gisela     [Greet in the_cleric_visits_the_sick]: "Hm, hm - And on you, father, by my reckoning - softly now."
Aldric     [Remark in the_cleric_visits_the_sick]: "All things ripen that are let be, Gisela."
Gisela     [Agree in the_cleric_visits_the_sick]: "We keep as best we can, father. The toil is hard."
Aldric     [Question in visiting_the_sick ]: "Do you have grain left in the bin?"
Gisela     [Answer in visiting_the_sick ]: "A little bread or salt, if you can spare it, as I've cause to say."
Aldric     [Remark in visiting_the_sick ]: "Rest easy; the parish will not let you freeze."
Gisela     [Worry in visiting_the_sick ]: "This cough that's going round - does it take the old ones, Aldric?"
Aldric     [Remark in visiting_the_sick ]: "The parish keeps its own, Gisela - sleep easy."
Gisela     [Farewell in visiting_the_sick ]: "Then I'll see you at first light, as I've cause to say, or so I hear."
```

### Conversation #03: Aldric and Hob
- **Speakers:** Aldric (clergy) & Hob (clergy)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Aldric     [Greet in a_death_in_the_house]: "Morrow, Hob. Death levels more than our quarrel - as the mill grinds."
Hob        [Agree in a_death_in_the_house]: "I'll not pretend tears, Aldric - but I'll not mock the dead - and what of it."
Aldric     [Remark in a_death_in_the_house]: "All things ripen that are let be, father."
Hob        [Agree in a_death_in_the_house]: "Their toil shaped this parish; their memory will outlive us."
Aldric     [Remark in a_death_in_the_house]: "The sorrow will ease as the seasons turn and heal!"
Hob        [Agree in a_death_in_the_house]: "We'll see it through, Aldric - the village does."
Aldric     [Remark in the_cleric_blesses]: "Why, Dost thou find peace in thy daily labor, child?"
Hob        [Agree in the_cleric_blesses]: "Thou hast spoken the plain heart of the matter."
Aldric     [Question in asking_the_father ]: "Why, How dost thou find the road today, Hob?"
Hob        [Evade in asking_the_father ]: "We don't know each other so well as that."
Aldric     [Remark in asking_the_father ]: "The church has been our refuge through many a cold year."
Hob        [Errand in asking_the_father ]: "Carry a word to the hermit for me, quiet like. There's a loaf in it, as I've cause to say."
Aldric     [Remark in asking_the_father ]: "True enough - the fields don't gossip."
Hob        [Farewell in asking_the_father ]: "Fair day, Aldric - such as it is, with thee in it."
```

### Conversation #04: Aldric and Dagny
- **Speakers:** Aldric (clergy) & Dagny (clergy)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
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
- **Speakers:** Aldric (clergy) & Gisela (laborer)
- **Backstory A:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Backstory B:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Aldric     [Greet in the_cleric_visits_the_sick]: "Blessings on thee, Gisela - as the mill grinds."
Gisela     [Question in the_cleric_visits_the_sick]: "Father, hear my plea; the sickness grips my throat."
Aldric     [Question in the_cleric_visits_the_sick]: "The Lord comfort thee, child. Tell me of thy affliction."
Gisela     [Answer in the_cleric_visits_the_sick]: "It's the little one mostly - she burns with fever."
Aldric     [Remark in the_cleric_visits_the_sick]: "Green wood burns when it's ready and not before."
Gisela     [Farewell in the_cleric_visits_the_sick]: "Thank thee, father - thy blessing brings comfort - softly now."
```

### Conversation #06: Eliza and Aldric
- **Speakers:** Eliza (merchant) & Aldric (clergy)
- **Backstory A:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Friend
- **Trees Cascade:** ["the_bakers_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_bakers_market ]: "Morrow - the oven's been roaring since the small hours, and the square's already smelling it - God send it."
Aldric     [Greet in the_bakers_market ]: "A fair day to the one who feeds us all - as the mill grinds."
Eliza      [Remark in the_bakers_market ]: "A bake's a small miracle: flour, water, salt, and four hours of faith."
Aldric     [Agree in the_bakers_market ]: "Aye - even the priest can't hide a burnt loaf."
Eliza      [Remark in the_bakers_market ]: "Half the village's quarrels start in a queue for bread."
Aldric     [Farewell in the_bakers_market ]: "Keep the bake sweet, baker. Fare well."
```

### Conversation #07: Eliza and Marga
- **Speakers:** Eliza (merchant) & Marga (laborer)
- **Backstory A:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Backstory B:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Relation:** Friend
- **Trees Cascade:** ["the_bakers_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_bakers_market ]: "Morrow, Marga - the stall's out by the cross and the loaves went before the second bell - God send it."
Marga      [Greet in the_bakers_market ]: "A fair day to the one who feeds us all - by the rood."
Eliza      [Remark in the_bakers_market ]: "Burnt one tray this morning - the ash got in with the kindling."
Marga      [Agree in the_bakers_market ]: "The village rises when thy oven does, by my reckoning."
Eliza      [Remark in the_bakers_market ]: "A loaf's weight is a baker's word."
Marga      [Farewell in the_bakers_market ]: "May thy oven never run cold. Fare well."
```

### Conversation #08: Eliza and Corvin
- **Speakers:** Eliza (merchant) & Corvin (lord)
- **Backstory A:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Backstory B:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_bakers_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_bakers_market ]: "Morrow - the oven's been roaring since the small hours, and the square's already smelling it - God send it."
Corvin     [Greet in the_bakers_market ]: "Morrow - your loaves carry the village's mornings - mind the stile."
Eliza      [Remark in the_bakers_market ]: "The dough rose slow today - the weather was in it, like everything else."
Corvin     [Agree in the_bakers_market ]: "A bad bake's a hard day for all of us - but today's smelled fine."
Eliza      [Remark in the_bakers_market ]: "Half the village's quarrels start in a queue for bread."
Corvin     [Farewell in the_bakers_market ]: "May your oven never run cold. Fare well."
```

### Conversation #09: Eliza and Gisela
- **Speakers:** Eliza (merchant) & Gisela (laborer)
- **Backstory A:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Backstory B:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_bakers_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_bakers_market ]: "Morrow, Gisela - the stall's out by the cross and the loaves went before the second bell - God send it."
Gisela     [Greet in the_bakers_market ]: "Hm, hm - And a hot oven to you, baker - softly now."
Eliza      [Remark in the_bakers_market ]: "The crust tells the truth about the fire every single day."
Gisela     [Agree in the_bakers_market ]: "A bad bake's a hard day for all of us - but today's smelled fine, or so I make it."
Eliza      [Remark in the_bakers_market ]: "Half the village's quarrels start in a queue for bread."
Gisela     [Farewell in the_bakers_market ]: "Good day - save me a loaf if the queue's kind."
```

### Conversation #10: Eliza and Hob
- **Speakers:** Eliza (merchant) & Hob (clergy)
- **Backstory A:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_bakers_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Eliza      [Greet in the_bakers_market ]: "Morrow, Hob - the stall's out by the cross and the loaves went before the second bell - God send it."
Hob        [Greet in the_bakers_market ]: "A fair day to the one who feeds us all - and what of it."
Eliza      [Remark in the_bakers_market ]: "The dough rose slow today - the weather was in it, like everything else."
Hob        [Agree in the_bakers_market ]: "Aye - even the priest can't hide a burnt loaf, as I've cause to say."
Eliza      [Remark in the_bakers_market ]: "A loaf's weight is a baker's word."
Hob        [Farewell in the_bakers_market ]: "God speed your loaves and the hands that shape them, as I've cause to say."
```

### Conversation #11: Bram and Hob
- **Speakers:** Bram (merchant) & Hob (clergy)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Kin
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Bram       [Greet in asking_the_father ]: "The saints keep thee, father - saints keep us."
Hob        [Greet in asking_the_father ]: "God bless thy steps this day, Bram, or so I make it - and what of it."
Bram       [Question in asking_the_father ]: "Hah! Can the church spare a peck of grain for the hungry children, father?"
Hob        [Answer in asking_the_father ]: "They that count sheep all day shear none, friend."
Bram       [Remark in asking_the_father ]: "The church has been our refuge through many a cold year."
Hob        [Agree in asking_the_father ]: "Blessings on thy house, father."
Bram       [Remark in asking_the_father ]: "We have spoken our piece for today!"
Hob        [Farewell in asking_the_father ]: "Fair profit to thee, trader."
```

### Conversation #12: Bram and Aelfric
- **Speakers:** Bram (merchant) & Aelfric (laborer)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in condolence_call   ]: "The saints keep thee, Aelfric - saints keep us."
Aelfric    [Agree in condolence_call   ]: "Thou hast spoken the plain heart of the matter - the saints see all."
Bram       [Remark in condolence_call   ]: "The sorrow will ease as the seasons turn and heal."
Aelfric    [Agree in condolence_call   ]: "Sit then, Bram - the fire's low but there's room, or so I make it!"
Bram       [Remark in condolence_call   ]: "The house won't grieve alone, Aelfric!"
Aelfric    [Farewell in condolence_call   ]: "Go gently, Bram. The door's always open to thee."
```

### Conversation #13: Bram and Dagny
- **Speakers:** Bram (merchant) & Dagny (clergy)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Relation:** Kin
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Bram       [Greet in asking_the_father ]: "The saints keep thee, father - saints keep us."
Dagny      [Greet in asking_the_father ]: "The saints keep thee, Bram. What weighs on thee - so it goes."
Bram       [Question in asking_the_father ]: "Hah! Can the church spare a peck of grain for the hungry children, father?"
Dagny      [Answer in asking_the_father ]: "The tithe stands, but I will speak for thy house at the manor court."
Bram       [Remark in asking_the_father ]: "The church has been our refuge through many a cold year."
Dagny      [Agree in asking_the_father ]: "Blessings on thy house, father, or am I mistook?"
Bram       [Remark in asking_the_father ]: "As thou sayest, friend."
Dagny      [Farewell in asking_the_father ]: "I have no leisure to stand about, Bram, and more besides."
```

### Conversation #14: Bram and Hob
- **Speakers:** Bram (merchant) & Hob (clergy)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Stranger
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Bram       [Greet in asking_the_father ]: "The saints keep you, father - saints keep us."
Hob        [Greet in asking_the_father ]: "Always, child. Speak freely in the sight of God - and what of it."
Bram       [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Hob        [Answer in asking_the_father ]: "They that count sheep all day shear none, friend."
Bram       [Remark in asking_the_father ]: "Thank you, father - that lifts a great stone from my mind."
Hob        [Agree in asking_the_father ]: "The parish is lucky in its father."
Bram       [Remark in asking_the_father ]: "We have spoken our piece for today!"
Hob        [Farewell in asking_the_father ]: "Fair profit to you, trader."
```

### Conversation #15: Bram and Eliza
- **Speakers:** Bram (merchant) & Eliza (merchant)
- **Backstory A:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Backstory B:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Relation:** Acquaintance
- **Trees Cascade:** ["baker_hearth"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Bram       [Greet in baker_hearth      ]: "The saints keep you, trader - saints keep us."
Eliza      [Greet in baker_hearth      ]: "Well met! The oven is blazing hot and the loaves are baking - God send it."
Bram       [Question in baker_hearth      ]: "Are the loaves golden brown on the oven bricks?"
Eliza      [Answer in baker_hearth      ]: "Aye. Aye - as you say, as you say."
Bram       [Remark in baker_hearth      ]: "Flour and salt and yeast - the holy trinity of the table."
Eliza      [Farewell in baker_hearth      ]: "God's peace be upon your cottage, Bram."
```

### Conversation #16: Corvin and Aldric
- **Speakers:** Corvin (lord) & Aldric (clergy)
- **Backstory A:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in condolence_call   ]: "The village stands beside thee in this dark trial - mind the stile."
Aldric     [PushBack in condolence_call   ]: "We endure what Heaven sends us."
Corvin     [Remark in condolence_call   ]: "The haar's a fisher's blanket - keeps the cold off, mind!"
Aldric     [Slip in condolence_call   ]: "The harvest sang that year, when - it doesn't matter."
Corvin     [Remark in condolence_call   ]: "The house won't grieve alone, Aldric, as I've cause to say!"
Aldric     [Farewell in condolence_call   ]: "Go gently, Corvin. The door's always open to thee - as the mill grinds."
```

### Conversation #17: Corvin and Dagny
- **Speakers:** Corvin (lord) & Dagny (clergy)
- **Backstory A:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Backstory B:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Relation:** Friend
- **Trees Cascade:** ["asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Corvin     [Greet in asking_the_father ]: "A peaceful day to thee, Dagny - mind the stile."
Dagny      [Greet in asking_the_father ]: "Always, child. Speak freely in the sight of God, and more besides - so it goes."
Corvin     [Question in asking_the_father ]: "Why, Must the tithe stand full when the granary is half empty, father?"
Dagny      [Evade in asking_the_father ]: "Ask me another."
Corvin     [Remark in asking_the_father ]: "Rain before seven, dry by eleven, they say, or am I mistook?"
Dagny      [Agree in asking_the_father ]: "The parish is lucky in its father!"
Corvin     [Remark in asking_the_father ]: "As thou sayest, friend!"
Dagny      [Farewell in asking_the_father ]: "God keep the manor and thee with it, Corvin."
```

### Conversation #18: Corvin and Aelfric
- **Speakers:** Corvin (lord) & Aelfric (laborer)
- **Backstory A:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in condolence_call   ]: "I was deeply grieved to hear of thy loss, Aelfric, and that's the long of it - mind the stile."
Aelfric    [PushBack in condolence_call   ]: "That is not the way the elders told it."
Corvin     [Remark in condolence_call   ]: "The haar's a fisher's blanket - keeps the cold off, mind!"
Aelfric    [Agree in condolence_call   ]: "Sit then, Corvin - the fire's low but there's room, or so I make it - the saints see all."
Corvin     [Remark in condolence_call   ]: "I'll sit with thee a while yet, if it helps."
Aelfric    [Farewell in condolence_call   ]: "Thy coming did the house good, Corvin!"
```

### Conversation #19: Corvin and Aelfric
- **Speakers:** Corvin (lord) & Aelfric (laborer)
- **Backstory A:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in condolence_call   ]: "I was deeply grieved to hear of your loss, Aelfric, and that's the long of it - mind the stile."
Aelfric    [Agree in condolence_call   ]: "You have spoken the plain heart of the matter - the saints see all."
Corvin     [Remark in condolence_call   ]: "The haar's a fisher's blanket - keeps the cold off, mind!"
Aelfric    [Agree in condolence_call   ]: "Sit then, Corvin - the fire's low but there's room, or so I make it!"
Corvin     [Remark in condolence_call   ]: "The house won't grieve alone, Aelfric, as I've cause to say!"
Aelfric    [Farewell in condolence_call   ]: "Go gently, Corvin. The door's always open to you."
```

### Conversation #20: Corvin and Aldric
- **Speakers:** Corvin (lord) & Aldric (clergy)
- **Backstory A:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Friend
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Corvin     [Greet in condolence_call   ]: "The village stands beside thee in this dark trial - mind the stile."
Aldric     [Agree in condolence_call   ]: "The road is lonely now, but kind words help - as the mill grinds."
Corvin     [Remark in condolence_call   ]: "The haar's a fisher's blanket - keeps the cold off, mind!"
Aldric     [Worry in condolence_call   ]: "Do the dead sleep, Corvin? Or do they wait on us?"
Corvin     [Remark in condolence_call   ]: "I'll sit with thee a while yet, if it helps."
Aldric     [Farewell in condolence_call   ]: "Thy coming did the house good, Corvin."
```

### Conversation #21: Dagny and Aelfric
- **Speakers:** Dagny (clergy) & Aelfric (laborer)
- **Backstory A:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Rival
- **Trees Cascade:** ["the_cleric_consoles", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Dagny      [Greet in the_cleric_consoles]: "The peace of the saints be with thee, child, or am I mistook - so it goes."
Aelfric    [Greet in the_cleric_consoles]: "Peace be with thee, good priest - the saints see all."
Dagny      [Remark in the_cleric_consoles]: "In all my years I never saw the like, Aelfric, as I've cause to say."
Aelfric    [Agree in the_cleric_consoles]: "The season has been lean, father, but faith sustains us."
Dagny      [Remark in condolence_call   ]: "We have flour and eggs; let us share with thy house."
Aelfric    [Agree in condolence_call   ]: "Kindly said, Dagny. Stay the hour!"
Dagny      [Remark in condolence_call   ]: "The house won't grieve alone, Aelfric."
Aelfric    [Farewell in condolence_call   ]: "Go gently, Dagny. The door's always open to thee."
```

### Conversation #22: Dagny and Hob
- **Speakers:** Dagny (clergy) & Hob (clergy)
- **Backstory A:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Dagny      [Greet in the_cleric_blesses]: "A peaceful day to you, Hob - so it goes."
Hob        [Remark in the_cleric_blesses]: "A barred door keeps out friends as well as wolves, and more besides."
Dagny      [Remark in the_cleric_blesses]: "In all my years I never saw the like, father, as I've cause to say."
Hob        [Agree in the_cleric_blesses]: "You have spoken the plain heart of the matter - and what of it."
Dagny      [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Hob        [Evade in asking_the_father ]: "Some things keep."
Dagny      [Remark in asking_the_father ]: "Your counsel is wise, father. We will trust in Providence."
Hob        [Agree in asking_the_father ]: "The parish is lucky in its father."
Dagny      [Remark in asking_the_father ]: "We have spoken our piece for today."
Hob        [Farewell in asking_the_father ]: "I have no leisure to stand about, Dagny."
```

### Conversation #23: Dagny and Hob
- **Speakers:** Dagny (clergy) & Hob (clergy)
- **Backstory A:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["the_cleric_blesses", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Dagny      [Greet in the_cleric_blesses]: "God's grace light upon thy steps, Hob - so it goes."
Hob        [Remark in the_cleric_blesses]: "A barred door keeps out friends as well as wolves, and more besides."
Dagny      [Remark in the_cleric_blesses]: "In all my years I never saw the like, father, as I've cause to say."
Hob        [Agree in the_cleric_blesses]: "Aye, there is truth in what thou sayest, as I've cause to say - and what of it."
Dagny      [Question in asking_the_father ]: "Must the tithe stand full when the granary is half empty, father?"
Hob        [Answer in asking_the_father ]: "The church will share its grain, child - none shall starve at the door, and that's the long of it."
Dagny      [Remark in asking_the_father ]: "Thank thee, father - that lifts a great stone from my mind."
Hob        [Errand in asking_the_father ]: "Carry a word to the hermit for me, quiet like. There's a loaf in it, as I've cause to say."
Dagny      [Remark in asking_the_father ]: "We have spoken our piece for today."
Hob        [Farewell in asking_the_father ]: "Fair day, Dagny - the ale waits for us both someday."
```

### Conversation #24: Dagny and Aelfric
- **Speakers:** Dagny (clergy) & Aelfric (laborer)
- **Backstory A:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_consoles"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Dagny      [Greet in the_cleric_consoles]: "God's grace light upon your steps, Aelfric - so it goes."
Aelfric    [Question in the_cleric_consoles]: "A blessing for my sorrowing cottage, father!"
Dagny      [Question in the_cleric_consoles]: "The Lord comfort you, child. Tell me of your affliction, as I've cause to say."
Aelfric    [Answer in the_cleric_consoles]: "The house is so quiet, father. I can't bear the quiet."
Dagny      [Remark in the_cleric_consoles]: "Sixty winters I've kept this patch, Aelfric; I know its ways."
Aelfric    [Farewell in the_cleric_consoles]: "Thank you, father - your blessing brings comfort - the saints see all."
```

### Conversation #25: Dagny and Marga
- **Speakers:** Dagny (clergy) & Marga (laborer)
- **Backstory A:** Dagny is the village brewer. They know every soul and every soul's business. They brew ale and tend the village vats. The store ran out and the belly knows it.
- **Backstory B:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_blesses", "hunger_lean_times"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Dagny      [Greet in the_cleric_blesses]: "The peace of the saints be with you, child, or am I mistook - so it goes."
Marga      [Remark in the_cleric_blesses]: "Marga says what Marga says, father."
Dagny      [Remark in the_cleric_blesses]: "In all my years I never saw the like, Marga, as I've cause to say."
Marga      [Agree in the_cleric_blesses]: "You have spoken the plain heart of the matter - by the rood."
Dagny      [Remark in hunger_lean_times ]: "The children went to sleep crying with empty bellies last night, or am I mistook?"
Marga      [Agree in hunger_lean_times ]: "Come by our hearth; there is pottage enough for two more bowls, or so I hear."
Dagny      [Remark in hunger_lean_times ]: "A shared loaf tastes twice as sweet in hard times."
Marga      [Agree in hunger_lean_times ]: "Aye, hold fast - the year's not written yet!"
Dagny      [Remark in hunger_lean_times ]: "True enough - the fields don't gossip."
Marga      [Farewell in hunger_lean_times ]: "I take my leave, or so I make it."
```

### Conversation #26: Hob and Corvin
- **Speakers:** Hob (clergy) & Corvin (lord)
- **Backstory A:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Backstory B:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Relation:** Kin
- **Trees Cascade:** ["the_smiths_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_smiths_market ]: "Morrow - half the stall-holders owe me a hook or a hinge - and what of it."
Corvin     [Greet in the_smiths_market ]: "Well met - keep the iron honest, as ever - mind the stile."
Hob        [Remark in the_smiths_market ]: "Caution's a warm cloak, and a heavy one."
Corvin     [Agree in the_smiths_market ]: "The forge's work holds the village together, aye."
Hob        [Remark in the_smiths_market ]: "Iron remembers every blow; so does the arm that struck it."
Corvin     [Farewell in the_smiths_market ]: "Good day - the hinges can wait a week for a good weld."
```

### Conversation #27: Hob and Gisela
- **Speakers:** Hob (clergy) & Gisela (laborer)
- **Backstory A:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Backstory B:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_visits_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_cleric_visits_the_sick]: "The peace of the saints be with thee, child, as I see it - and what of it."
Gisela     [Question in the_cleric_visits_the_sick]: "Wilt thou hear my confession ere the fever takes me?"
Hob        [Question in the_cleric_visits_the_sick]: "How dost thou find the road today, Gisela?"
Gisela     [Answer in the_cleric_visits_the_sick]: "It's the little one mostly - she burns with fever."
Hob        [Remark in the_cleric_visits_the_sick]: "A barred door keeps out friends as well as wolves, and more besides."
Gisela     [Farewell in the_cleric_visits_the_sick]: "God's peace be upon thy cottage, Hob - softly now."
```

### Conversation #28: Hob and Bram
- **Speakers:** Hob (clergy) & Bram (merchant)
- **Backstory A:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Rival
- **Trees Cascade:** ["the_smiths_market"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Hob        [Greet in the_smiths_market ]: "Fair day - the coals ran white early; market days don't wait - and what of it."
Bram       [Greet in the_smiths_market ]: "Blessings on thee and thy kin, Hob - saints keep us."
Hob        [Remark in the_smiths_market ]: "Caution's a warm cloak, and a heavy one."
Bram       [Agree in the_smiths_market ]: "Well said - iron keeps honest accounts too."
Hob        [Remark in the_smiths_market ]: "Temper the metal, not the man - though both get tested daily."
Bram       [Farewell in the_smiths_market ]: "May thy welds hold fast, smith. Fare well!"
```

### Conversation #29: Hob and Aelfric
- **Speakers:** Hob (clergy) & Aelfric (laborer)
- **Backstory A:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_cleric_consoles", "the_smiths_market"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Hob        [Greet in the_cleric_consoles]: "Blessings on you, Aelfric - and what of it."
Aelfric    [Greet in the_cleric_consoles]: "God bless your steps this day, Hob - the saints see all."
Hob        [Remark in the_cleric_consoles]: "Caution's a warm cloak, and a heavy one."
Aelfric    [Agree in the_cleric_consoles]: "I try to walk uprightly before God and my neighbors!"
Hob        [Remark in the_smiths_market ]: "A ploughshare came in snapped clean - the field fought hard this spring."
Aelfric    [Agree in the_smiths_market ]: "True - every good tool's a smith's signature."
Hob        [Remark in the_smiths_market ]: "Iron remembers every blow; so does the arm that struck it."
Aelfric    [Farewell in the_smiths_market ]: "Keep the forge hot, friend. Fare well."
```

### Conversation #30: Hob and Wystan
- **Speakers:** Hob (clergy) & Wystan (laborer)
- **Backstory A:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Backstory B:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["the_cleric_consoles", "the_smiths_market"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Hob        [Greet in the_cleric_consoles]: "Blessings on thee, Wystan - and what of it."
Wystan     [Greet in the_cleric_consoles]: "Peace be with thee, good priest - as my mother taught."
Hob        [Remark in the_cleric_consoles]: "Caution's a warm cloak, and a heavy one."
Wystan     [Agree in the_cleric_consoles]: "I try to walk uprightly before God and my neighbors."
Hob        [Remark in the_smiths_market ]: "Three dozen nails, a coulter tip and a cracked hinge already - the day's half wrought."
Wystan     [Agree in the_smiths_market ]: "Well said - iron keeps honest accounts too!"
Hob        [Remark in the_smiths_market ]: "Iron remembers every blow; so does the arm that struck it."
Wystan     [Farewell in the_smiths_market ]: "May thy welds hold fast, smith. Fare well."
```

### Conversation #31: Marga and Eliza
- **Speakers:** Marga (laborer) & Eliza (merchant)
- **Backstory A:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Backstory B:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Relation:** Friend
- **Trees Cascade:** ["mentor_counsel", "friendship_bonds"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Remark in mentor_counsel    ]: "Marga minds their own affairs, thanks."
Eliza      [Agree in mentor_counsel    ]: "Aye, I mark it - as thou hast taught me - God send it."
Marga      [Question in friendship_bonds  ]: "Can I count on thy strong arm when we raise the barn?"
Eliza      [Answer in friendship_bonds  ]: "Aye. Aye - as thou sayest, as thou sayest."
Marga      [Remark in friendship_bonds  ]: "We have shared crusts and sweat for twenty years now, as I've cause to say."
Eliza      [Farewell in friendship_bonds  ]: "Nor thee, me old friend. Walk safe."
```

### Conversation #32: Marga and Wystan
- **Speakers:** Marga (laborer) & Wystan (laborer)
- **Backstory A:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Backstory B:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Relation:** Acquaintance
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in condolence_call   ]: "A peaceful day to you, Wystan - by the rood."
Wystan     [PushBack in condolence_call   ]: "Nay, I see it differently, mark me, for my part."
Marga      [Remark in condolence_call   ]: "Marga was not born yesterday, Wystan."
Wystan     [Agree in condolence_call   ]: "Kindly said, Marga. Stay the hour - as my mother taught."
Marga      [Remark in condolence_call   ]: "The house won't grieve alone, Wystan."
Wystan     [Farewell in condolence_call   ]: "Your coming did the house good, Marga."
```

### Conversation #33: Marga and Eliza
- **Speakers:** Marga (laborer) & Eliza (merchant)
- **Backstory A:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Backstory B:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Relation:** Acquaintance
- **Trees Cascade:** ["apprentice_eager", "baker_hearth"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Remark in apprentice_eager  ]: "Marga minds their own affairs, thanks."
Eliza      [Agree in apprentice_eager  ]: "You learn faster than I did at your age - God send it."
Marga      [Question in baker_hearth      ]: "Are the loaves golden brown on the oven bricks?"
Eliza      [Answer in baker_hearth      ]: "Aye. Aye - as you say, as you say."
Marga      [Remark in baker_hearth      ]: "Flour and salt and yeast - the holy trinity of the table."
Eliza      [Farewell in baker_hearth      ]: "I take my warm crust home to the children. Good day."
```

### Conversation #34: Marga and Aelfric
- **Speakers:** Marga (laborer) & Aelfric (laborer)
- **Backstory A:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Marga      [Greet in condolence_call   ]: "The village stands beside you in this dark trial - by the rood."
Aelfric    [Agree in condolence_call   ]: "Aye, there is truth in what you say, as I see it - the saints see all."
Marga      [Remark in condolence_call   ]: "Marga was not born yesterday, Aelfric."
Aelfric    [Agree in condolence_call   ]: "Sit then, Marga - the fire's low but there's room, or so I make it!"
Marga      [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Aelfric    [Farewell in condolence_call   ]: "Your coming did the house good, Marga!"
```

### Conversation #35: Marga and Corvin
- **Speakers:** Marga (laborer) & Corvin (lord)
- **Backstory A:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Backstory B:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Relation:** Kin
- **Trees Cascade:** ["sweetheart_beat", "kin_warmth"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Marga      [Remark in sweetheart_beat   ]: "Marga minds their own affairs, thanks."
Corvin     [Agree in sweetheart_beat   ]: "Thy words are warmer than the hearth, love - mind the stile."
Marga      [Question in kin_warmth        ]: "Dost thou need an extra hand to dig the potato trench?"
Corvin     [Answer in kin_warmth        ]: "Aye, he brought home two bags of salt and a fine sickle, and more besides."
Marga      [Remark in kin_warmth        ]: "Blood is thicker than river mud, Corvin."
Corvin     [Agree in kin_warmth        ]: "Blood looks to blood, and here we are!"
Marga      [Remark in kin_warmth        ]: "We have spoken our piece for today."
Corvin     [Farewell in kin_warmth        ]: "Walk on, goodman - mind the hedges, as I've cause to say."
```

### Conversation #36: Gisela and Hob
- **Speakers:** Gisela (laborer) & Hob (clergy)
- **Backstory A:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Kin
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Mm-mm-mm - Hob - aye, it's me, coughing like a broken bellows. Keep out of the draft - softly now."
Hob        [Question in the_sickness      ]: "What does the herb-wife say? Our house will hear of it."
Gisela     [Answer in the_sickness      ]: "My lungs feel like wet wool on a rainy morning, as like as not."
Hob        [Remark in the_sickness      ]: "They that count sheep all day shear none, friend."
Gisela     [Remark in the_sickness      ]: "My thanks. A warm bowl does more than herbs."
Hob        [Farewell in the_sickness      ]: "God's peace be upon thy cottage, Gisela - and what of it."
```

### Conversation #37: Gisela and Marga
- **Speakers:** Gisela (laborer) & Marga (laborer)
- **Backstory A:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Backstory B:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Mm-mm-mm - Morrow, Marga. Step wide; even my fever wants no part of thee - softly now."
Marga      [Question in the_sickness      ]: "How long hast thou dragged that about, Gisela?"
Gisela     [Answer in the_sickness      ]: "Half the north lane is coughing into their sleeves."
Marga      [Remark in the_sickness      ]: "Marga minds their own affairs, thanks."
Gisela     [Remark in the_sickness      ]: "Rest by the hearth and let the fever break in peace."
Marga      [Farewell in the_sickness      ]: "The village stands together or it falls. Fare well - by the rood."
```

### Conversation #38: Gisela and Bram
- **Speakers:** Gisela (laborer) & Bram (merchant)
- **Backstory A:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Rival
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Mm-mm-mm - *cough* Bram. Keep to thy side - I'd not give thee even my sickness - softly now."
Bram       [Agree in the_sickness      ]: "Even I'd not wish this on thee, Gisela. Perhaps - saints keep us."
Gisela     [Answer in the_sickness      ]: "The calves won't feed themselves - and I'd not ask thee for help, Bram."
Bram       [Remark in the_sickness      ]: "I'll carry thy pail - once - and I'd not have it spoke of."
Gisela     [Remark in the_sickness      ]: "Thou art kind, Bram. The saints remember a neighborly hand."
Bram       [Farewell in the_sickness      ]: "Think nothing of it. We look after our own."
```

### Conversation #39: Gisela and Aelfric
- **Speakers:** Gisela (laborer) & Aelfric (laborer)
- **Backstory A:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["the_sickness"]
- **Ending:** WalksOff
- **Turns Count:** 4

```text
Gisela     [Greet in the_sickness      ]: "Mm-mm-mm - *cough* Morrow, traveler. Best keep to the far side of the path, for my part - softly now."
Aelfric    [PushBack in the_sickness      ]: "Keep your distance, stranger - I want no new sickness!"
Gisela     [Remark in the_sickness      ]: "Rest by the hearth and let the fever break in peace."
Aelfric    [PushBack in the_sickness      ]: "Stay in your own yard if you are frightened!"
```

### Conversation #40: Gisela and Corvin
- **Speakers:** Gisela (laborer) & Corvin (lord)
- **Backstory A:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Backstory B:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Relation:** Acquaintance
- **Trees Cascade:** ["the_sickness"]
- **Ending:** Farewell
- **Turns Count:** 6

```text
Gisela     [Greet in the_sickness      ]: "Mm-mm-mm - *cough* Fair morning, Corvin. Mind the breeze between us - softly now."
Corvin     [Agree in the_sickness      ]: "A rough fever, by the look of you - mind the stile."
Gisela     [Answer in the_sickness      ]: "I'd rest in the straw if the chores would do themselves."
Corvin     [Remark in the_sickness      ]: "Red sky at night - shepherd's delight, as my gran taught."
Gisela     [Remark in the_sickness      ]: "Rest by the hearth and let the fever break in peace."
Corvin     [Farewell in the_sickness      ]: "May the road be kind under your boots."
```

### Conversation #41: Wystan and Eliza
- **Speakers:** Wystan (laborer) & Eliza (merchant)
- **Backstory A:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Backstory B:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Relation:** Friend
- **Trees Cascade:** ["a_death_in_the_house", "friendship_bonds"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Wystan     [Greet in a_death_in_the_house]: "Eliza. Forgive me - the house is still since they passed, and thy face is a kindness - as my mother taught."
Eliza      [Agree in a_death_in_the_house]: "My heart breaks with thine, Wystan. Always - God send it."
Wystan     [Remark in a_death_in_the_house]: "Worth a farthing at market, that news."
Eliza      [Agree in a_death_in_the_house]: "Thou hast spoken the plain heart of the matter."
Wystan     [Remark in a_death_in_the_house]: "The sorrow will ease as the seasons turn and heal, and that's the long of it!"
Eliza      [Agree in a_death_in_the_house]: "We'll see it through, Wystan - the village does."
Wystan     [Question in friendship_bonds  ]: "Dost thou recall the summer the mill dam broke?"
Eliza      [Answer in friendship_bonds  ]: "So thou sayest, so it is."
Wystan     [Remark in friendship_bonds  ]: "We have shared crusts and sweat for twenty years now!"
Eliza      [Farewell in friendship_bonds  ]: "May the road be kind under thy boots."
```

### Conversation #42: Wystan and Hob
- **Speakers:** Wystan (laborer) & Hob (clergy)
- **Backstory A:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Backstory B:** Hob is the village smith. They're the first with a loaf when a house is bare. Their purse strings are knotted tight. They mean to be more than they were born. They work the village anvil and forge. Last night's cup is still on them.
- **Relation:** Friend
- **Trees Cascade:** ["a_death_in_the_house", "asking_the_father"]
- **Ending:** Farewell
- **Turns Count:** 9

```text
Wystan     [Greet in a_death_in_the_house]: "Well met, dear friend. The days feel longer now - as my mother taught."
Hob        [Question in a_death_in_the_house]: "Shall I sit with thee? Thou needn't be alone."
Wystan     [Answer in a_death_in_the_house]: "Someone to watch the sheep for an afternoon."
Wystan     [Question in asking_the_father ]: "Will the parish altar provide candles for our sick neighbor, father?"
Hob        [Answer in asking_the_father ]: "A barred door keeps out friends as well as wolves, and more besides."
Wystan     [Remark in asking_the_father ]: "Worth a farthing at market, that news."
Hob        [Agree in asking_the_father ]: "The parish is lucky in its father - and what of it."
Wystan     [Remark in asking_the_father ]: "As thou sayest, friend!"
Hob        [Farewell in asking_the_father ]: "My day is waiting."
```

### Conversation #43: Wystan and Aelfric
- **Speakers:** Wystan (laborer) & Aelfric (laborer)
- **Backstory A:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Backstory B:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["a_death_in_the_house", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 7

```text
Wystan     [Greet in a_death_in_the_house]: "Morrow, stranger. Forgive a grieving house its silence - as my mother taught."
Aelfric    [Question in a_death_in_the_house]: "Do you have folk to help you?"
Wystan     [Answer in a_death_in_the_house]: "Someone to watch the sheep for an afternoon."
Wystan     [Remark in condolence_call   ]: "Two pennies' worth of worry, that is."
Aelfric    [Agree in condolence_call   ]: "Sit then, Wystan - the fire's low but there's room, or so I make it - the saints see all."
Wystan     [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Aelfric    [Farewell in condolence_call   ]: "Your coming did the house good, Wystan!"
```

### Conversation #44: Wystan and Eliza
- **Speakers:** Wystan (laborer) & Eliza (merchant)
- **Backstory A:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Backstory B:** Eliza is the village baker. They'd sooner watch the clouds than the plough. They tend the communal bread oven. The store ran out and the belly knows it.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "old_scores"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Wystan     [Greet in a_death_in_the_house]: "Eliza. Even thou wilt know the house has gone quiet, and more besides, or do I hear wrong - as my mother taught."
Eliza      [Agree in a_death_in_the_house]: "We'd our quarrels, Wystan - but the grave settles them - God send it."
Wystan     [Remark in a_death_in_the_house]: "Worth a farthing at market, that news."
Eliza      [Agree in a_death_in_the_house]: "Their toil shaped this parish; their memory will outlive us."
Wystan     [Remark in a_death_in_the_house]: "It does a body good to speak their name aloud."
Eliza      [Agree in a_death_in_the_house]: "We'll see it through, Wystan - the village does."
Wystan     [Remark in old_scores        ]: "Thy lot still owes mine three groats, whatever lies thou tellest."
Eliza      [PushBack in old_scores        ]: "We paid that debt at Michaelmas, as the reeve well knows."
Wystan     [Remark in old_scores        ]: "I have looked upon thy face long enough, as I've cause to say."
Eliza      [Farewell in old_scores        ]: "Aye. Done."
```

### Conversation #45: Wystan and Gisela
- **Speakers:** Wystan (laborer) & Gisela (laborer)
- **Backstory A:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Backstory B:** Gisela is the village weaver. They're the first with a loaf when a house is bare. They mean to be more than they were born. They weave russet cloth on the loom. A cough's had them these two weeks.
- **Relation:** Stranger
- **Trees Cascade:** ["visiting_the_sick"]
- **Ending:** Farewell
- **Turns Count:** 8

```text
Wystan     [Greet in visiting_the_sick ]: "A peaceful day to you, Gisela - as my mother taught."
Gisela     [Answer in visiting_the_sick ]: "Weak as a lamb, but still drawing breath."
Wystan     [Question in visiting_the_sick ]: "Why, Can I fetch water or chop kindling for your hearth?"
Gisela     [Answer in visiting_the_sick ]: "A jug of fresh well water would be a godsend, or so I make it."
Wystan     [Remark in visiting_the_sick ]: "It'd cost you a groat to hear worse."
Gisela     [Worry in visiting_the_sick ]: "This cough that's going round - does it take the old ones, Wystan?"
Wystan     [Remark in visiting_the_sick ]: "The parish keeps its own, Gisela - sleep easy, as I've cause to say."
Gisela     [Farewell in visiting_the_sick ]: "Rest well, Wystan - softly now."
```

### Conversation #46: Aelfric and Wystan
- **Speakers:** Aelfric (laborer) & Wystan (laborer)
- **Backstory A:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Wystan works the fields for a day's wage. They mean to be more than they were born. Their temper's shorter than a winter's day. They know every soul and every soul's business. They buried one of their own last week.
- **Relation:** Stranger
- **Trees Cascade:** ["a_death_in_the_house", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 7

```text
Aelfric    [Greet in a_death_in_the_house]: "A heavy morning - we've buried one of ours - the saints see all."
Wystan     [Question in a_death_in_the_house]: "Why, Do you have folk to help you?"
Aelfric    [Answer in a_death_in_the_house]: "A few split logs, if you have any seasoned oak."
Aelfric    [Remark in condolence_call   ]: "The sorrow will ease as the seasons turn and heal, for my sins."
Wystan     [Agree in condolence_call   ]: "Kindly said, Aelfric. Stay the hour - as my mother taught."
Aelfric    [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Wystan     [Farewell in condolence_call   ]: "Your coming did the house good, Aelfric."
```

### Conversation #47: Aelfric and Bram
- **Speakers:** Aelfric (laborer) & Bram (merchant)
- **Backstory A:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Bram trades on the road between villages. Their temper's shorter than a winter's day. Their purse strings are knotted tight.
- **Relation:** Kin
- **Trees Cascade:** ["a_death_in_the_house", "blood_kin", "kin_warmth"]
- **Ending:** Farewell
- **Turns Count:** 14

```text
Aelfric    [Greet in a_death_in_the_house]: "Bram. Blood knows blood - the house is empty without them - the saints see all."
Bram       [Agree in a_death_in_the_house]: "They were ours - and we carry them now - saints keep us."
Aelfric    [Remark in a_death_in_the_house]: "She loved the smell of the damp earth after rain, and for my sins beside."
Bram       [Agree in a_death_in_the_house]: "Thou hast spoken the plain heart of the matter."
Aelfric    [Remark in a_death_in_the_house]: "The house feels less empty when a neighbor sits a spell!"
Bram       [Agree in a_death_in_the_house]: "We'll see it through, Aelfric - the village does!"
Aelfric    [Remark in blood_kin         ]: "Mother sent word - she prays for thy cough every bell."
Bram       [Agree in blood_kin         ]: "Kin looks after kin - now and always."
Aelfric    [Question in kin_warmth        ]: "How fares our sister's household down by the green?"
Bram       [Answer in kin_warmth        ]: "Our kin will stand beside thee through good season and bad."
Aelfric    [Remark in kin_warmth        ]: "Blood is thicker than river mud, Bram!"
Bram       [Errand in kin_warmth        ]: "Carry a word to the hermit for me, quiet like. There's a loaf in it."
Aelfric    [Remark in kin_warmth        ]: "Aye, words won't fill the storehouse!"
Bram       [Farewell in kin_warmth        ]: "Fair day, kin - give my love to thy hearth."
```

### Conversation #48: Aelfric and Corvin
- **Speakers:** Aelfric (laborer) & Corvin (lord)
- **Backstory A:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Corvin is the village shepherd. They're the first with a loaf when a house is bare. They know every soul and every soul's business. Their temper's shorter than a winter's day. They herd ewes and lambs on the common pasture.
- **Relation:** Rival
- **Trees Cascade:** ["a_death_in_the_house", "old_scores"]
- **Ending:** WalksOff
- **Turns Count:** 10

```text
Aelfric    [Greet in a_death_in_the_house]: "Corvin. Even thou wilt know the house has gone quiet - the saints see all."
Corvin     [Agree in a_death_in_the_house]: "We'd our quarrels, Aelfric - but the grave settles them - mind the stile."
Aelfric    [Remark in a_death_in_the_house]: "Forty winters together, and now only the silence!, and for my sins beside."
Corvin     [Agree in a_death_in_the_house]: "Thou hast spoken the plain heart of the matter."
Aelfric    [Remark in a_death_in_the_house]: "Thank thee for listening, Corvin - it lightens the heart."
Corvin     [Bid in a_death_in_the_house]: "Sit a while longer - the kettle's still warm."
Aelfric    [Remark in old_scores        ]: "I saw where thy cart rutted the lane by my ditch."
Corvin     [PushBack in old_scores        ]: "Say that again, if thou darest."
Aelfric    [Remark in old_scores        ]: "I have no words to waste on a crooked knave."
Corvin     [PushBack in old_scores        ]: "Keep thy fists to thyself before I break them."
```

### Conversation #49: Aelfric and Aldric
- **Speakers:** Aelfric (laborer) & Aldric (clergy)
- **Backstory A:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Aldric serves the village church. Their temper's shorter than a winter's day. They buried one of their own last week.
- **Relation:** Kin
- **Trees Cascade:** ["a_death_in_the_house", "condolence_call"]
- **Ending:** Farewell
- **Turns Count:** 10

```text
Aelfric    [Greet in a_death_in_the_house]: "Aldric. Blood knows blood - the house is empty without them - the saints see all."
Aldric     [Agree in a_death_in_the_house]: "Our blood mourns together, Aelfric. The saints keep them - as the mill grinds."
Aelfric    [Remark in a_death_in_the_house]: "She loved the smell of the damp earth after rain, and for my sins beside."
Aldric     [Agree in a_death_in_the_house]: "Aye, there is truth in what thou sayest."
Aelfric    [Remark in a_death_in_the_house]: "The house feels less empty when a neighbor sits a spell!"
Aldric     [Bid in a_death_in_the_house]: "Walk with me to the well a moment, Aelfric!"
Aelfric    [Remark in condolence_call   ]: "We have flour and eggs; let us share with thy house."
Aldric     [Agree in condolence_call   ]: "Sit then, Aelfric - the fire's low but there's room."
Aelfric    [Remark in condolence_call   ]: "Send for me at any hour - the latch lifts easy."
Aldric     [Farewell in condolence_call   ]: "Go gently, Aelfric. The door's always open to thee."
```

### Conversation #50: Aelfric and Marga
- **Speakers:** Aelfric (laborer) & Marga (laborer)
- **Backstory A:** Aelfric is the village printer. Their temper's shorter than a winter's day. Their purse strings are knotted tight. They'd sooner watch the clouds than the plough. They buried one of their own last week.
- **Backstory B:** Marga is the village weaver. They're the first with a loaf when a house is bare. Their temper's shorter than a winter's day. They bow to no one. They weave russet cloth on the loom.
- **Relation:** Acquaintance
- **Trees Cascade:** ["a_death_in_the_house", "apprentice_eager", "weaver_wool"]
- **Ending:** Farewell
- **Turns Count:** 12

```text
Aelfric    [Greet in a_death_in_the_house]: "Morrow, Marga. The bell tolled for my house this week - the saints see all."
Marga      [Agree in a_death_in_the_house]: "The saints receive them kindly - by the rood."
Aelfric    [Remark in a_death_in_the_house]: "She loved the smell of the damp earth after rain, and for my sins beside."
Marga      [Agree in a_death_in_the_house]: "Tell me of him - good memories are a shield against darkness."
Aelfric    [Remark in a_death_in_the_house]: "It does a body good to speak their name aloud."
Marga      [Agree in a_death_in_the_house]: "Gently does it, friend. Day by day."
Aelfric    [Remark in apprentice_eager  ]: "I set the joints as you showed me - will you look?"
Marga      [Agree in apprentice_eager  ]: "Eager hands make light work - go on."
Aelfric    [Question in weaver_wool       ]: "Did the sheep shearers bring in the upland wool yet?"
Marga      [Answer in weaver_wool       ]: "Marga minds their own affairs, thanks."
Aelfric    [Remark in weaver_wool       ]: "Madder root gives a brave red, but woad blues last thirty years!"
Marga      [Farewell in weaver_wool       ]: "God's peace be upon your cottage, Aelfric."
```

## Benchmark Summary

- Total Characters Generated: 10
- Total Conversations Generated: 50
- Generator Seed: 42

Benchmark generation complete.
