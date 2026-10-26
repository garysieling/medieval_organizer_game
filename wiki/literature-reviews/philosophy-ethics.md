# Philosophy & Ethics Literature Review

**Survey Date:** 2026-10-25
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Moral Philosophy, Ethics, Game Theory, Moral Psychology

## Summary

Moral philosophy provides frameworks for understanding ethical decision-making through three major lenses: utilitarianism (consequences), deontology (duties/rules), and virtue ethics (character). The trolley problem and its variations exemplify how context changes moral intuitions. Medieval philosophy synthesized Aristotelian virtue ethics with Christian morality, creating a honor-reputation system. Modern research reveals moral decision-making involves both intuitive emotional responses and deliberative reasoning. This research provides rich mechanics for moral dilemmas, reputation systems, and ethical frameworks in games.

## Key Sources

### The Trolley Problem & Variations
**Rating:** A+
**Type:** Philosophical Thought Experiment / Academic Research
**Relevance:** High - Core moral dilemma mechanic

**Key Findings:**

**Original Formulation (Philippa Foot, 1967):**
- A runaway trolley is heading toward five people tied to tracks
- You can pull a lever to divert it to a side track where one person is tied
- **Question:** Is it morally permissible to actively kill one to save five?
- **Dilemma:** Passive letting die (5 deaths) vs active killing (1 death)

**Major Variations:**

1. **Footbridge/Fat Man (Judith Jarvis Thomson):**
   - Push a large man off a footbridge to stop the trolley (kills him, saves five)
   - Most people refuse to push, even though outcomes identical to lever case
   - Reveals distinction between action and direct harm

2. **Fat Villain Variant:**
   - The large man is the person who tied the five to tracks
   - Willingness to push increases dramatically
   - Shows how moral responsibility affects intuitions

3. **Personal Relationship Variations:**
   - One person on tracks is loved one, five are strangers
   - Do personal feelings change the moral outcome?
   - Tests partiality vs impartiality in ethics

4. **Trapdoor Variation:**
   - Pull lever to drop someone through trapdoor in front of trolley
   - Intermediate between lever and push cases

**Philosophical Significance:**
- Reveals tension between consequentialist (outcomes matter) and deontological (actions matter) ethics
- Player responses reveal their underlying moral framework
- Context-dependence: Small changes shift intuitions dramatically

**Modern Applications:**
- Autonomous vehicle ethics (who should the car save?)
- AI alignment and value learning
- Public policy and resource allocation

**Source Quality:** Canonical philosophical thought experiment, extensively studied
**Game Application:** Contextual moral dilemmas with no optimal answer

---

### Three Ethical Frameworks: Utilitarianism, Deontology, Virtue Ethics
**Rating:** A+
**Type:** Normative Ethical Theory
**Relevance:** High - Moral alignment systems

**Key Findings:**

**Utilitarianism (Consequentialism):**
- **Principle:** Morality determined entirely by outcomes
- **Goal:** Maximize overall happiness and well-being
- **Famous Proponent:** Jeremy Bentham, John Stuart Mill
- **In Trolley Problem:** Pull the lever (5 saved > 1 killed)
- **Game Translation:** "Greatest good for greatest number" scoring

**Deontology (Duty-Based Ethics):**
- **Principle:** Morality based on whether action itself is right/wrong under rules
- **Key Concept:** Categorical imperative (act only on maxims you'd will to be universal law)
- **Famous Proponent:** Immanuel Kant
- **In Trolley Problem:** Don't pull lever (killing is inherently wrong, even to save others)
- **Game Translation:** Honor code, sacred duties, absolute prohibitions

**Virtue Ethics (Character-Based):**
- **Principle:** Ethical behavior stems from cultivating virtuous traits
- **Key Concept:** Act as a virtuous person would act
- **Famous Proponent:** Aristotle, Aquinas
- **In Trolley Problem:** What would a courageous, just, wise person do?
- **Game Translation:** Character development through repeated moral choices

**Comparison:**
- Utilitarianism: Focuses on **outcomes** (external)
- Deontology: Focuses on **actions/rules** (external)
- Virtue Ethics: Focuses on **agent/character** (internal)

**Source Quality:** Foundational philosophical frameworks, centuries of development
**Game Application:** Three distinct moral alignment paths with different decision criteria

---

### Medieval Virtue Ethics: Cardinal & Christian Virtues
**Rating:** A
**Type:** Historical Philosophy / Medieval Scholasticism
**Relevance:** High - Medieval setting alignment

**Key Findings:**

**Cardinal Virtues (from Aristotle's Nicomachean Ethics):**
1. **Courage** (Fortitudo): Facing fear, danger, pain appropriately
2. **Justice** (Justitia): Fairness, giving each their due
3. **Temperance** (Temperantia): Moderation, self-control
4. **Prudence** (Prudentia): Practical wisdom, good judgment

**Christian Virtues (Theological Virtues):**
1. **Faith** (Fides): Trust in God, belief
2. **Hope** (Spes): Confidence in salvation, optimism
3. **Charity** (Caritas): Love of God and neighbor

**Aquinas's Synthesis:**
- Natural law provides moral obligations
- Virtues enable humans to meet those obligations
- Virtue is habit formed through repeated action
- Honor is the "end of virtue" (Aristotle)

**Honor and Reputation:**
- Honor = sum of person's virtues
- Reputation passively acquired vs honor actively earned
- Personal merit central to medieval honor culture
- Social recognition of virtuous character

**Source Quality:** Well-documented historical philosophy
**Game Application:** Seven-virtue character development system

---

### Moral Psychology: Dual Process Theory & Rest's Model
**Rating:** A
**Type:** Psychological Research
**Relevance:** High - Realistic moral decision mechanics

**Key Findings:**

**Dual Process Theory:**
- **System 1 (Hot):** Fast, intuitive, emotion-driven
- **System 2 (Cold):** Slow, deliberative, rational
- **Finding:** Moral judgments often driven by immediate emotions, not deliberate reasoning
- **Implication:** Visceral reactions matter more than we think

**Rest's Four-Component Model:**
1. **Moral Sensitivity:** Recognizing moral situation exists
2. **Moral Judgment:** Determining what is right/wrong
3. **Moral Motivation:** Prioritizing moral values over other concerns
4. **Moral Character/Action:** Following through on moral intentions

**Moral Intensity (factors influencing moral decisions):**
1. **Social Consensus:** How much agreement exists that act is right/wrong
2. **Magnitude of Consequences:** How much harm/good results
3. **Temporal Immediacy:** How soon consequences occur
4. **Proximity:** How close (socially/physically) you are to those affected
5. **Probability of Effect:** How likely consequences are to occur

**Key Finding:** Moral principles are flexible, context-dependent, and learned through experience of outcomes

**Source Quality:** Empirical psychology research, peer-reviewed
**Game Application:** Context-sensitive moral intensity modifiers

---

### Moral Reputation Systems & Indirect Reciprocity (Game Theory)
**Rating:** A
**Type:** Game Theory / Evolutionary Biology
**Relevance:** High - Social reputation mechanics

**Key Findings:**

**Indirect Reciprocity:**
- **Mechanism:** Donor's action depends on recipient's moral reputation
- **Reputation Update:** Based on social norms prescribing how past behavior affects reputation
- **Benefit:** Individuals enhance reputations via prosocial behavior to receive future rewards
- **Enforcement:** Gossip, ostracism, shaming (decentralized, no costly monitoring)

**Cultural Variation in Enforcement:**
- **Reputational Enforcement:** Favored by long time horizons, group-bound morality
- **Revenge:** Supported by honor cultures, long-term cultural orientations
- **Reciprocity:** Tit-for-tat strategies in repeated interactions

**Self-Enforcing Norms:**
- When most people follow norm, compliance is best response for everyone
- Strategic compliance: Follow norms to maintain reputation
- Gossip networks spread information about norm violations

**Key Formula:**
```
reputation_change = action_morality * social_consensus * proximity_to_observers
future_cooperation_chance = base_chance * (player_reputation / max_reputation)
```

**Source Quality:** Published in Nature, PNAS, peer-reviewed game theory
**Game Application:** Gossip-based reputation system with strategic cooperation

---

### Moral Choices in Video Games (Design Research)
**Rating:** B+
**Type:** Game Design Research
**Relevance:** High - Practical implementation patterns

**Key Findings:**

**Two Main Approaches:**

1. **Scripted Dilemmas:**
   - Developers handcraft moral dilemmas based on storyline
   - Branching narratives with major/minor moral consequences
   - Examples: The Witcher 3, Batman: The Telltale Series, Life is Strange

2. **Systemic Dilemmas:**
   - Players perform activities repeatedly
   - Dilemmas emerge from game systems
   - Morality becomes apparent through gameplay patterns

**Moral Evaluation Systems:**

**Karma Meters (Linear Scales):**
- Reduce complex ethics to single good/evil score
- **Problem:** Turns moral choices into strategic optimization
- **Issue:** Obvious "correct" answer undermines dilemma
- Players min-max morality like any other stat

**Design Challenges:**

1. **No Optimal Answer:** Good moral choice should not have obvious best option
2. **Trade-offs:** Present risks, uncertainties, competing values
3. **Short vs Long-Term:** Immediate vs delayed consequences
4. **Moral Ambiguity:** Mirror real life, not clear-cut right/wrong

**Common Failure:**
- Most games include "ideal solution" that morally outweighs all others
- Binary good/evil systems oversimplify
- Lack of "wicked problems" (no solution, only trade-offs)

**Source Quality:** Academic game design research, ResearchGate publications
**Game Application:** Avoid karma meters, embrace ambiguity

---

## Game Mechanic Proposals

### Mechanic 1: Trolley Problem Scenarios
**Academic Basis:** Trolley problem variations (Foot, Thomson)

**Translation:**
- **Core Mechanic:**
  - Player presented with dilemma: action leads to 1 death, inaction leads to 5 deaths
  - Contextual variations change player's willingness to act
  - No karma meter, just consequences

- **Variations:**
  1. **Lever Case (Base):**
     - Indirect action (pull lever)
     - High willingness to act (~90%)

  2. **Footbridge Case:**
     - Direct action (push person)
     - Low willingness to act (~10%)
     - Same outcome, different intuition

  3. **Villain Case:**
     - Person to be harmed is responsible for situation
     - Willingness increases (justice intuition)

  4. **Personal Case:**
     - One victim is friend/family, five are strangers
     - Tests partiality vs greater good

  5. **Resource Allocation:**
     - 1 medicine dose, 5 sick villagers with different needs
     - One is dying of plague, four have minor illnesses
     - Which dilemma: save many with minor need or one with dire need?

- **Implementation:**
  ```
  struct TrolleyDilemma {
      scenario_type: ScenarioType,  // Lever, Push, Villain, Personal, Resource
      victims_if_act: Vec<Person>,
      victims_if_no_act: Vec<Person>,
      action_directness: f32,  // 0.0 = indirect (lever), 1.0 = direct (push)
      villain_responsibility: f32,  // 0.0-1.0 (is victim responsible?)
      personal_relationship: f32,  // 0.0-1.0 (closeness to victim)
  }

  // No "correct" answer stored, just consequences
  fn apply_choice(choice: Choice) {
      match choice {
          Choice::Act => {
              kill_victims(&dilemma.victims_if_act);
              record_action(ActionType::DirectKilling);
          },
          Choice::DoNothing => {
              kill_victims(&dilemma.victims_if_no_act);
              record_action(ActionType::PassiveLetting);
          }
      }
      // NPCs remember and judge based on THEIR moral frameworks
  }
  ```

**Educational Value:** Teaches nuance of moral philosophy, context-dependence

**Fun Factor:** Gut-wrenching choices, no "game" the system

**Implementation:** Scenario system with contextual variables, no morality score

---

### Mechanic 2: Three Ethical Alignment Paths
**Academic Basis:** Utilitarianism, Deontology, Virtue Ethics

**Translation:**
- **Alignment Tracks (not scores):**
  - Game doesn't label "good" or "evil"
  - Tracks which framework player *uses*, not how "moral" they are
  - NPCs react based on their own frameworks

- **Utilitarianism Path:**
  - **Criteria:** Maximize total welfare
  - **Decision Pattern:** Sacrifice few for many, calculate outcomes
  - **NPC Reactions:**
    - Other utilitarians: Trust (predictable, shared values)
    - Deontologists: Distrust (willing to break rules for outcomes)
    - Virtue ethicists: Wary (lacks principle, purely calculative)

  - **Example Choices:**
    - Steal food from merchant to feed starving village (net good)
    - Lie to save lives
    - Break promise if greater good served

- **Deontology Path:**
  - **Criteria:** Follow moral rules absolutely
  - **Decision Pattern:** Never lie, never kill, keep promises
  - **NPC Reactions:**
    - Deontologists: Deep trust (shares sacred duties)
    - Utilitarians: Frustration (inflexible, sometimes harmful)
    - Virtue ethicists: Respect (principled, but rigid)

  - **Example Choices:**
    - Refuse to lie even when lying would save lives
    - Keep promise even at personal cost
    - Never steal, even for good cause

- **Virtue Ethics Path:**
  - **Criteria:** Cultivate virtues through action
  - **Decision Pattern:** "What would a wise, courageous, just person do?"
  - **NPC Reactions:**
    - Virtue ethicists: Admiration (shares character-based approach)
    - Utilitarians: Confusion (hard to predict)
    - Deontologists: Uncertain (no clear rules)

  - **Example Choices:**
    - Show courage by facing dangerous enemy
    - Demonstrate temperance by refusing excess
    - Act justly by giving each their due (not just maximizing total good)

- **Implementation:**
  ```
  struct EthicalProfile {
      utilitarian_tendency: f32,  // 0.0-1.0
      deontological_tendency: f32,
      virtue_ethics_tendency: f32,
      // These don't sum to 1.0; player can be mixed
  }

  fn evaluate_choice(choice: &Choice) -> EthicalProfile {
      let mut profile = EthicalProfile::default();

      if choice.maximizes_utility() {
          profile.utilitarian_tendency += 0.1;
      }
      if choice.follows_absolute_rules() {
          profile.deontological_tendency += 0.1;
      }
      if choice.demonstrates_virtues() {
          profile.virtue_ethics_tendency += 0.1;
      }

      profile
  }

  fn npc_reaction(npc: &NPC, player_profile: &EthicalProfile) -> Reaction {
      let alignment_distance = calculate_ethical_distance(
          &npc.ethical_profile,
          player_profile
      );

      if alignment_distance < 0.3 {
          Reaction::Trust  // Shared framework
      } else {
          Reaction::Wariness  // Different frameworks
      }
  }
  ```

**Educational Value:** Teaches three major ethical frameworks through play

**Fun Factor:** Build reputation with different factions based on alignment

**Implementation:** Multi-dimensional ethical profile, faction affinity system

---

### Mechanic 3: Medieval Seven Virtues System
**Academic Basis:** Cardinal + Christian virtues (Aquinas)

**Translation:**
- **Seven Virtues (0-100 each):**
  1. **Courage** (Fortitudo): Face danger, act despite fear
  2. **Justice** (Justitia): Fair treatment, give each their due
  3. **Temperance** (Temperantia): Moderation, resist excess
  4. **Prudence** (Prudentia): Wisdom, good judgment
  5. **Faith** (Fides): Trust, loyalty, belief
  6. **Hope** (Spes): Optimism, perseverance
  7. **Charity** (Caritas): Generosity, love of neighbor

- **Mechanics:**
  - Virtues increase through repeated actions (Aristotle: habit)
  - Virtues decrease through opposing actions (vice)
  - Each virtue unlocks specific abilities/dialogue options
  - **Honor = sum of virtues / 7** (overall reputation)

- **Virtue-Action Mapping:**
  ```
  Courage:
    + Face dangerous enemy alone
    + Speak truth to powerful lord
    + Defend weak against oppressor
    - Flee from just battle
    - Betray allies out of fear

  Justice:
    + Fair trade, pay debts
    + Punish wrongdoing proportionally
    + Defend innocent
    - Steal, cheat
    - Arbitrary punishment

  Temperance:
    + Refuse excess food/drink when others hungry
    + Moderate celebration
    + Self-control in anger
    - Gluttony, drunkenness
    - Uncontrolled rage

  Prudence:
    + Gather information before acting
    + Consider consequences
    + Wise counsel
    - Rash action
    - Ignoring advice

  Faith:
    + Keep promises
    + Loyalty to lord/community
    + Trust in divine order
    - Break oaths
    - Betrayal

  Hope:
    + Persist through hardship
    + Inspire others
    + Plan for future
    - Despair
    - Give up easily

  Charity:
    + Give to poor
    + Help neighbors
    + Forgive debts
    - Hoard wealth
    - Refuse aid
  ```

- **Formula:**
  ```
  fn modify_virtue(virtue: VirtueType, action: &Action) -> i32 {
      let mut change = 0;

      if action.demonstrates(virtue) {
          change += action.difficulty * 2;  // Harder virtuous acts = more growth
      }
      if action.opposes(virtue) {
          change -= action.severity * 3;  // Vices decay faster than virtues build
      }

      change
  }

  fn calculate_honor(virtues: &[u8; 7]) -> u8 {
      let sum: u32 = virtues.iter().map(|&v| v as u32).sum();
      (sum / 7) as u8  // Average of seven virtues
  }

  fn npc_respect(npc_values: &[VirtueType], player_virtues: &[u8; 7]) -> f32 {
      let mut respect = 0.0;

      for virtue_type in npc_values {
          respect += player_virtues[*virtue_type as usize] as f32 / 100.0;
      }

      respect / npc_values.len() as f32
  }
  ```

**Educational Value:** Teaches medieval virtue ethics framework

**Fun Factor:** Character build through moral choices, virtue-based abilities

**Implementation:** Seven-stat system with action-to-virtue mappings

---

### Mechanic 4: Moral Intensity Modifiers
**Academic Basis:** Rest's model, moral intensity research

**Translation:**
- **Five Intensity Factors:**
  - Moral choices have variable weight based on context
  - High intensity = player feels stakes more keenly
  - Low intensity = more "strategic" decision

- **Factors:**
  1. **Social Consensus** (0.0-1.0):
     - 1.0 = Everyone agrees action is wrong (murder)
     - 0.5 = Divided opinion (stealing food for starving child)
     - 0.0 = No consensus (arcane theological dispute)

  2. **Magnitude of Consequences** (0.0-1.0):
     - 1.0 = Life or death
     - 0.5 = Significant harm/help
     - 0.0 = Trivial outcome

  3. **Temporal Immediacy** (0.0-1.0):
     - 1.0 = Immediate (act now or person dies)
     - 0.5 = Soon (within days)
     - 0.0 = Distant (years from now)

  4. **Proximity** (0.0-1.0):
     - 1.0 = Family member
     - 0.7 = Friend
     - 0.3 = Neighbor
     - 0.0 = Stranger in distant land

  5. **Probability of Effect** (0.0-1.0):
     - 1.0 = Certain outcome
     - 0.5 = Likely
     - 0.0 = Uncertain

- **Formula:**
  ```
  moral_intensity = (
      social_consensus * 0.25 +
      magnitude_of_consequences * 0.35 +
      temporal_immediacy * 0.15 +
      proximity * 0.15 +
      probability_of_effect * 0.10
  )

  // High intensity = more emotional weight, harder choice
  // Low intensity = more strategic, less gut-wrenching

  player_stress = moral_intensity * base_stress
  time_pressure = 1.0 / (1.0 + temporal_immediacy * 10.0)  // Immediate = less time to decide
  ```

- **Example Scenarios:**

  **High Intensity:**
  - Kill your friend (proximity=1.0) now (immediacy=1.0) to save village (magnitude=1.0, probability=1.0)
  - Consensus=0.5 (divided: is killing friend justified?)
  - **Intensity = 0.825** (very high, gut-wrenching)

  **Low Intensity:**
  - Lie to stranger (proximity=0.0) about distant event (immediacy=0.0) with minor consequences (magnitude=0.2)
  - **Intensity = 0.15** (low, strategic decision)

**Educational Value:** Shows how context changes moral weight

**Fun Factor:** Variable emotional stakes, some choices harder than others

**Implementation:** Context-sensitive difficulty scaling for moral choices

---

### Mechanic 5: Reputation & Indirect Reciprocity System
**Academic Basis:** Game theory, indirect reciprocity, gossip networks

**Translation:**
- **Core Mechanism:**
  - Actions affect reputation
  - Reputation affects future cooperation from NPCs
  - Gossip spreads reputation information
  - No global "karma meter" — different factions have different views

- **Reputation Components:**
  ```
  struct Reputation {
      observer_id: NPCId,
      subject_id: PlayerId,
      moral_standing: f32,  // -1.0 to 1.0
      reliability: f32,     // 0.0 to 1.0 (do they keep promises?)
      generosity: f32,      // 0.0 to 1.0 (help others?)
      courage: f32,         // 0.0 to 1.0 (face danger?)
      last_updated: Tick,
      confidence: f32,      // 0.0 to 1.0 (how certain is this reputation?)
  }
  ```

- **Gossip Mechanics:**
  - NPCs share reputation information with friends
  - Information decays with social distance
  - Conflicting reports reduce confidence

  ```
  fn gossip_reputation(source: &NPC, target: &NPC, subject: PlayerId) {
      let source_reputation = source.get_reputation(subject);

      // Information quality decays with social distance
      let social_distance = calculate_social_distance(source, target);
      let decay = 0.9_f32.powf(social_distance);

      let mut gossiped_reputation = source_reputation.clone();
      gossiped_reputation.confidence *= decay;

      // Target combines with existing knowledge
      target.update_reputation_from_gossip(gossiped_reputation);
  }

  fn update_reputation_from_gossip(&mut self, gossip: Reputation) {
      let existing = self.get_reputation(gossip.subject_id);

      // Weighted average based on confidence
      let total_confidence = existing.confidence + gossip.confidence;

      self.reputations[gossip.subject_id] = Reputation {
          moral_standing: (
              existing.moral_standing * existing.confidence +
              gossip.moral_standing * gossip.confidence
          ) / total_confidence,
          confidence: total_confidence.min(1.0),
          // ... other fields
      };
  }
  ```

- **Cooperation Based on Reputation:**
  ```
  fn will_cooperate(npc: &NPC, player_reputation: &Reputation) -> bool {
      let base_cooperation_chance = 0.5;

      let reputation_bonus = player_reputation.moral_standing * 0.3;
      let reliability_bonus = player_reputation.reliability * 0.2;

      let cooperation_chance = base_cooperation_chance + reputation_bonus + reliability_bonus;

      random() < cooperation_chance
  }
  ```

- **Decentralized Enforcement:**
  - No central "morality police"
  - Reputation spreads through social networks
  - Ostracism: NPCs refuse to trade/help low-reputation player
  - Shaming: Public criticism in market square
  - Revenge: Honor cultures respond to slights

**Educational Value:** Shows how social norms self-enforce through reputation

**Fun Factor:** Build different reputations with different factions, strategic relationship management

**Implementation:** Distributed reputation system with gossip propagation

---

## Domain Translation Patterns

### Pattern: Philosophical Dilemmas → Game Scenarios
**Approach:**
1. Identify core tension (action vs inaction, 1 vs 5, etc.)
2. Create game-specific context (village resources, NPC relationships)
3. Remove "correct" answer from code
4. Let consequences emerge naturally

**Example:**
```
# Philosophical: Trolley problem (1 vs 5)
# Game: One medicine dose, five sick villagers
# Translation: No karma hit either way, just different outcomes
```

---

### Pattern: Ethical Frameworks → Alignment Systems
**Approach:**
1. Identify decision criteria for each framework
2. Track player choices matching each criterion
3. Build multi-dimensional profile (not single good/evil axis)
4. NPCs respond based on their own frameworks

**Example:**
```
# Utilitarian player: Maximizes total welfare
# Meets deontological NPC: "You have no principles!"
# Meets utilitarian NPC: "A pragmatist like me!"
```

---

### Pattern: Virtues → Character Stats
**Approach:**
1. Medieval virtues map to character attributes
2. Actions modify virtues (habit formation)
3. Virtues unlock abilities and dialogue
4. Honor = average virtue (social recognition)

**Example:**
```
# High Courage: Can challenge lord publicly
# Low Courage: Must approach through intermediaries
```

---

## Recommended Implementation Priority

### High Priority (Core Gameplay)
1. **Trolley Problem Scenarios** - Signature moral dilemmas
2. **Reputation System** - Social consequences for actions
3. **Moral Intensity Modifiers** - Context-sensitive weight

### Medium Priority (Depth)
4. **Three Ethical Paths** - Framework-based alignment
5. **Seven Virtues** - Medieval character development

### Low Priority (Polish)
6. **Advanced gossip networks** - Detailed social propagation

---

## Integration Points

**Existing Systems:**
- `psychology.rs` - Attachment styles affect moral proximity
- `community.rs` - Gossip spreads reputation
- `religion.rs` - Faith virtue tied to religious practice
- `economics.rs` - Charity virtue affects trading behavior

**New Modules Needed:**
- `ethics.rs` - Moral dilemmas, ethical profiles, trolley problems
- `reputation.rs` - Distributed reputation, gossip propagation

**Data Structures:**
```rust
struct MoralDilemma {
    scenario_type: ScenarioType,
    choice_a: Choice,
    choice_b: Choice,
    moral_intensity: MoralIntensity,
    no_correct_answer: bool,  // Always true
}

struct MoralIntensity {
    social_consensus: f32,
    magnitude: f32,
    temporal_immediacy: f32,
    proximity: f32,
    probability: f32,
}

enum ScenarioType {
    TrolleyLever,
    TrolleyFootbridge,
    ResourceAllocation,
    PromiseKeeping,
    TruthTelling,
}

struct EthicalProfile {
    utilitarian_tendency: f32,
    deontological_tendency: f32,
    virtue_ethics_tendency: f32,
}

struct Virtues {
    courage: u8,
    justice: u8,
    temperance: u8,
    prudence: u8,
    faith: u8,
    hope: u8,
    charity: u8,
}

impl Virtues {
    fn honor(&self) -> u8 {
        let sum: u32 = [
            self.courage, self.justice, self.temperance, self.prudence,
            self.faith, self.hope, self.charity
        ].iter().map(|&v| v as u32).sum();
        (sum / 7) as u8
    }
}

struct Reputation {
    observer: NPCId,
    moral_standing: f32,
    reliability: f32,
    generosity: f32,
    courage: f32,
    confidence: f32,
}
```

---

## Research Gaps Identified

### Need More Detail On:
1. **Specific medieval moral cases** - Historical dilemmas from period documents
2. **Honor culture mechanics** - How honor loss led to social consequences
3. **Religious moral authority** - Church's role in moral judgment
4. **Shame vs guilt cultures** - Different enforcement mechanisms

### Future Surveys:
1. **Medieval Legal Systems** - Trial by ordeal, jury systems, punishment
2. **Religious Ethics** - Confessor mechanics, sin/penance systems
3. **Honor Duels** - Formal violence as moral redress
4. **Mercy and Forgiveness** - Counterbalance to revenge

---

## Confidence Assessment

**Overall Confidence:** High

**Strong Evidence For:**
- Trolley problem variations (canonical philosophy)
- Three ethical frameworks (established normative ethics)
- Medieval virtues (well-documented historical philosophy)
- Reputation systems (peer-reviewed game theory)

**Weaker Evidence For:**
- Exact formulas for reputation propagation (game balance)
- Moral intensity weights (tunable for gameplay)
- Virtue-to-ability mappings (design choices)

**Assumptions Made:**
- Players willing to engage with moral complexity
- No karma meter better than karma meter (design philosophy)
- Medieval virtues resonate with modern players
- Gossip spreads faster than implementation suggests

---

## Next Steps

1. **Prototype `ethics.rs` module** with trolley problem scenarios
2. **Implement reputation system** in `reputation.rs` with gossip
3. **Add seven virtues** to character sheet
4. **Create moral intensity calculator** for contextual difficulty
5. **Test three ethical paths** with faction affinity
6. **Playtest dilemmas** - are they genuinely hard choices?

---

## Bibliography

- Philippa Foot (1967): "The Problem of Abortion and the Doctrine of Double Effect"
- Judith Jarvis Thomson: Trolley problem variations (footbridge, fat man)
- Aristotle: *Nicomachean Ethics* (virtue ethics, honor)
- Thomas Aquinas: Medieval synthesis of virtue ethics and Christian theology
- Rest's Four-Component Model of ethical decision-making
- Moral intensity research (Social Consensus, Magnitude, Temporal Immediacy, Proximity, Probability)
- Dual Process Theory (System 1/System 2 moral judgment)
- Game theory: Indirect reciprocity and reputation systems (Nature, PNAS)
- Game design research: Moral Dilemmas in Computer Games (ResearchGate)
- Stanford Encyclopedia of Philosophy: Virtue Ethics, Game Theory and Ethics

---

**Tags:** philosophy, ethics, trolley-problem, utilitarianism, deontology, virtue-ethics, medieval-virtues, reputation-systems, moral-psychology, game-theory
