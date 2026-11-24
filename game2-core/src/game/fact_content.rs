//! The rule-of-three content for the first twenty fact types (E3-4,
//! #1425): every covered fact carries a routine tell (a visible habit,
//! emitted as `tell:<slug>`), a narration tell (the slip they let fall
//! in talk), a gossip tell (what a neighbor remarks on seeing the
//! tell), a telling at its disclosure tier, and the choice it explains
//! under pressure (the refusal reason and the nerve it costs them to
//! show). The table is the catalog the coverage test walks - a fact
//! that lacks a leg ships as uncovered and fails it.

use crate::game::conversation::{Area, Facts};

/// One covered fact's rule-of-three content. `fact` is the fact-table
/// tag ("ghost:levy"); `tell` is the visible habit's slug
/// ("shuns_the_gate" - the status reads `tell:shuns_the_gate`);
/// `slip` is what the carrier lets fall in ordinary talk; `gossip` is
/// the neighbor's remark; `tiers` are the tellings at each tier the
/// fact can surface at; `pressure` is the reason they give when the
/// ask presses them; `showing` scales how readily they come out when
/// the cause calls (the pressure choice made measurable).
pub struct FactContent {
    pub fact: &'static str,
    pub area: Area,
    pub tell: &'static str,
    pub slip: &'static str,
    pub gossip: &'static str,
    /// (tier, telling) pairs, deepest last
    pub tiers: &'static [(u8, &'static str)],
    pub pressure: &'static str,
    /// Multiplier on their coming when only the bare word calls them
    pub showing: f32,
}

/// The first twenty: every ghost kind and every lie a ghost can teach -
/// the wounds and the lessons, which is where the play lives (E3-4).
pub const FACT_CONTENTS: &[FactContent] = &[
    FactContent { fact: "ghost:fire", area: Area::Trouble, tell: "keeps_water",
        slip: "Smoke on the wind and I'm counting buckets before I'm thinking.",
        gossip: "Fetch her a bucket and she's easy again - she keeps them filled by the door.",
        tiers: &[(2, "The barn went up before my eyes could credit it. I sleep light when the thatch is dry."),
                 (3, "I pulled the cart out myself and left the roof to burn. Some nights I still hear it.")],
        pressure: "I've seen a thatch take a whole year to stop smelling of it. Not by my hand.",
        showing: 0.7 },
    FactContent { fact: "ghost:levy", area: Area::Trouble, tell: "shuns_the_gate",
        slip: "The gate? I take the field way, is all.",
        gossip: "She won't come near the gate since they marched Hal through it.",
        tiers: &[(2, "They took Hal at the gate in '39. I watched them count him like a sheep."),
                 (3, "He wrote once. Then the road ate him - the levy never sends them back.")],
        pressure: "You'll not see me stand under that gate. Ask another.",
        showing: 0.55 },
    FactContent { fact: "ghost:death_duty", area: Area::Heart, tell: "tends_the_graves",
        slip: "I keep my mother's plot tidy. It's what's left to keep.",
        gossip: "She's at the churchyard most days - the duty took the cow and the will both.",
        tiers: &[(2, "When father died they came for his best cow. The dead pay their way out."),
                 (3, "I sold the ring to cover what they valued him at. He'd have hated that I had to.")],
        pressure: "They took the cow off my mother's grave. I'll not lift a hand that keeps them counting.",
        showing: 0.7 },
    FactContent { fact: "ghost:fever", area: Area::Heart, tell: "burns_rue",
        slip: "The summer the fever took the row - I still burn rue at the door.",
        gossip: "Rue and vinegar on her doorstep every morning - she saw the row go down.",
        tiers: &[(2, "Half the row lay fevered that summer. The well ran with rue water."),
                 (3, "I nursed three of them and stood up well myself. I've wondered since why.")],
        pressure: "Sickness emptied our row once. I keep my door and my rue - I'll not crowd a sickroom for any cause.",
        showing: 0.7 },
    FactContent { fact: "ghost:miller_debt", area: Area::Trouble, tell: "takes_the_field_way",
        slip: "I cross by the upper ford, is all - shorter, some days.",
        gossip: "He owes the miller still - watch him take the field way round the pond.",
        tiers: &[(2, "My father's bond sits in the miller's drawer with my name under it."),
                 (3, "Twelve years paying off a hand I never shook. The pond road's longer but it's mine.")],
        pressure: "Till the miller's drawer is empty of my name, I keep my head down. Not this cause.",
        showing: 0.6 },
    FactContent { fact: "ghost:broken_betrothal", area: Area::Heart, tell: "walks_alone",
        slip: "Feast days are for those who've someone to stand with.",
        gossip: "She was promised to the wheelwright's boy once - it broke, and she walks alone since.",
        tiers: &[(2, "We were to be wed the spring the levy took him. The banns were read once."),
                 (3, "He came back with another's ring on his promise. I've kept my own counsel since.")],
        pressure: "I stood alone under the banns once. I'll not stand under another's banner.",
        showing: 0.8 },
    FactContent { fact: "ghost:old_faith", area: Area::Heart, tell: "rings_the_stones",
        slip: "The church bell's a new voice. The stones were talking first.",
        gossip: "You'll find her at the stones of an evening - keeps the old ways, quiet-like.",
        tiers: &[(2, "My grandmother taught me the old words at the stones. The priest preaches; the stones remember."),
                 (3, "I was marked at the grove before ever the font. If the reeve knew the half of it.")],
        pressure: "The stones keep their counsel and I keep mine. {2:subj} {2:will} not name me in {2:poss} cause.",
        showing: 0.75 },
    FactContent { fact: "ghost:eviction", area: Area::Trouble, tell: "keeps_the_door",
        slip: "A door is a thing a body can lose. Mine stays latched and me inside it.",
        gossip: "He was put out of the Holt place in the hard winter - keeps his door close since.",
        tiers: &[(2, "We were put out at Martinmas with the snow on the cart. A door isn't given twice."),
                 (3, "I sleep where I can see the latch. Once put out, a body keeps every door like the last.")],
        pressure: "I lost one roof for saying the wrong thing to the wrong man. This roof I keep.",
        showing: 0.6 },
    FactContent { fact: "ghost:harvest_feast", area: Area::Heart, tell: "counts_the_days",
        slip: "Come the harvest call {2:subj} {2:will} see me dancing - whatever the year brings.",
        gossip: "She danced the last feast till the torches drowned - counts the days to the next.",
        tiers: &[(2, "The year the barns were full we danced the ring till the moon quit. I keep that year."),
                 (3, "It was the last feast before the hard years. I hold it like a coin I won't spend.")],
        pressure: "I'll stand with {2:obj} after the feast. The one good day isn't the day's for the asking.",
        showing: 1.15 },
    FactContent { fact: "lie:lord_always_wins", area: Area::Trouble, tell: "makes_way",
        slip: "The lord's men always win. My grandfather said it, and his before.",
        gossip: "Ask him about the reeve and he'll tell {2:obj} the same: the lord's men always win.",
        tiers: &[(2, "Push back and the rope answers. The lord's men always win - that's the whole lesson."),
                 (3, "I saw them hang a man for less than {2:subj} {2:be} asking me. They always win.")],
        pressure: "The lord's men always win. I've seen what happens to the other side of that.",
        showing: 0.55 },
    FactContent { fact: "lie:someone_to_blame", area: Area::Trouble, tell: "points_first",
        slip: "There's always someone to blame, mark me - best it isn't {2:obj}.",
        gossip: "She names a culprit before the straw's counted - she knows how blame travels.",
        tiers: &[(2, "When the grain ran short they found a face for it. The village always finds someone to blame."),
                 (3, "They blamed the tinker for the sickness year. He never came back, and the sickness did.")],
        pressure: "When this goes wrong they'll want a face for it. It won't be mine.",
        showing: 0.7 },
    FactContent { fact: "lie:wars_take_sons", area: Area::Trouble, tell: "watches_the_road",
        slip: "Two boys went up that road in lord's colors. The road keeps what it takes.",
        gossip: "He still watches the road of an evening, like the wars might send one back.",
        tiers: &[(2, "The lord's wars take our sons - mine went in '41 and the road's been empty since."),
                 (3, "The levy took the strong and sent the bill home. I count what's left.")],
        pressure: "The lord's wars took both my boys. You'll not talk me into another's quarrel.",
        showing: 0.5 },
    FactContent { fact: "lie:lord_takes_from_dead", area: Area::Heart, tell: "keeps_close",
        slip: "Bury it deep or they'll value it at the reading.",
        gossip: "She buried her mother's ring with her - says the lord takes even from the dead.",
        tiers: &[(2, "They priced my mother's wedding ring at the grave's edge. The lord takes even from the dead."),
                 (3, "I hid the copper before the tally man came. Let them count what's left.")],
        pressure: "They counted my dead mother's ring. You'll get nothing more of mine.",
        showing: 0.65 },
    FactContent { fact: "lie:god_sends", area: Area::Heart, tell: "leaves_it_to_god",
        slip: "God sends what God sends - rain or reeve.",
        gossip: "Ask him to fret and he'll tell {2:obj} God sends what God sends.",
        tiers: &[(2, "When the crop failed I knelt and ate what came. God sends what God sends."),
                 (3, "I stopped asking why the year the well soured. It's sent or it isn't.")],
        pressure: "If the cause is just, God sends the hands for it. It doesn't send mine.",
        showing: 0.85 },
    FactContent { fact: "lie:mill_owns_us", area: Area::Trouble, tell: "counts_twice",
        slip: "Every sack leaves the mill lighter than it came. The mill owns us.",
        gossip: "He weighs the flour twice before he'll pay - swears the mill owns us all.",
        tiers: &[(2, "The miller takes his cut of every sack. The mill owns us, and no man's father says different."),
                 (3, "My father counted against the miller's weights and lost the year's bread for it.")],
        pressure: "The mill owns us already. I'll not sign my name under one more power.",
        showing: 0.7 },
    FactContent { fact: "lie:promises_for_keepers", area: Area::Heart, tell: "promises_nothing",
        slip: "I don't say 'I will' - I say 'I'll see'. Promises are for those who can keep them.",
        gossip: "Ask her to promise and she won't - she keeps her word too dear to spend it.",
        tiers: &[(2, "I promised my brother I'd stand by him and the fever took the keeping out of it. Promises are for those who can keep them."),
                 (3, "I made one promise I couldn't keep. I don't make them now.")],
        pressure: "Promises are for those who can keep them - and I can't promise this.",
        showing: 0.6 },
    FactContent { fact: "lie:keep_hidden", area: Area::Trouble, tell: "keeps_to_shadow",
        slip: "Best keep {2:poss} true self hidden - the square has eyes.",
        gossip: "Never seen him stand in the open - keeps himself hidden, always has.",
        tiers: &[(2, "Show the reeve {2:poss} true thoughts and count the cost. Keep {2:poss} true self hidden."),
                 (3, "I said one true thing at the wrong fire. Three years paying for it taught me to hide.")],
        pressure: "Keep {2:poss} true self hidden - standing in {2:poss} cause is the opposite of hidden.",
        showing: 0.5 },
    FactContent { fact: "lie:never_cross_reeve", area: Area::Trouble, tell: "steps_aside",
        slip: "The reeve counts heads and bread alike - never cross the reeve.",
        gossip: "He bows deeper than the rest of us - never cross the reeve, that's his rule.",
        tiers: &[(2, "The reeve marked my cousin for a short count once. Never cross the reeve."),
                 (3, "I crossed him over a beam of timber and paid for three winters. Never again.")],
        pressure: "Never cross the reeve - and {2:poss} cause crosses him by noon.",
        showing: 0.45 },
    FactContent { fact: "lie:strong_together", area: Area::Trouble, tell: "first_to_lend",
        slip: "We were strong together once - before the bailiff learned our names.",
        gossip: "She talks still of the year they moved the tithe together - strong together, she says.",
        tiers: &[(2, "The year the bridge went we raised it in a day, all of us. We were strong together once."),
                 (3, "They broke us one at a time after the barns stood empty. Together once - I'd have it again.")],
        pressure: "We were strong together once and paid for it singly. Show me this ends different.",
        showing: 1.15 },
    FactContent { fact: "lie:truth_comes_out", area: Area::Trouble, tell: "says_it_plain",
        slip: "The truth comes out if someone says it - my mother never feared to.",
        gossip: "She'll say the thing out loud that the rest of us only think - truth comes out, she swears.",
        tiers: &[(2, "My mother named the short count out loud and the miller paid her back. The truth comes out if someone says it."),
                 (3, "I watched the truth free a man once. It cost the speaker a winter, and I'd pay it again.")],
        pressure: "If the truth's on {2:poss} side I'll stand - the truth comes out if someone says it.",
        showing: 1.2 },
];

/// The covered fact's content, by fact-table tag
pub fn content_for_fact(fact: &str) -> Option<&'static FactContent> {
    FACT_CONTENTS.iter().find(|c| c.fact == fact)
}

/// The covered fact a tell slug belongs to, if any
pub fn content_for_tell(tell: &str) -> Option<&'static FactContent> {
    FACT_CONTENTS.iter().find(|c| c.tell == tell)
}

/// What the carrier tells at `tier` in `area`, when the asked-about's
/// facts carry a covered one: the fact's own line instead of the
/// topic's generic telling. `facts` are the asked-about's `self:` set;
/// lines already said this talk are spent and skipped (#1425).
pub fn telling(facts: &Facts, area: Area, tier: u8, roll: f32,
               used: &crate::engine::det::HashSet<&'static str>) -> Option<&'static str> {
    let mut found: Vec<&'static str> = vec![];
    for c in FACT_CONTENTS {
        if c.area != area || !facts.has(&format!("self:{}", c.fact)) { continue; }
        for &(t, line) in c.tiers {
            if t == tier && !used.contains(line) { found.push(line); }
        }
    }
    if found.is_empty() { None } else { Some(found[(roll * found.len() as f32) as usize % found.len()]) }
}

/// How readily they come when the bare word calls (E3-4's pressure leg):
/// the ghosts' and lies' showing factors, multiplied and clamped
pub fn showing_factor(world: &crate::engine::ecs::World, who: crate::engine::ecs::Entity) -> f32 {
    let Some(p) = world.get_component::<crate::game::persona::Persona>(who) else { return 1.0 };
    let mut f = 1.0f32;
    for g in &p.ghosts {
        if let Some(c) = content_for_fact(&format!("ghost:{}", g.kind.tag())) { f *= c.showing; }
        // a turned ghost's lie stopped holding them back (E8-2) - the
        // wound's drag stays, the lesson's goes
        if !g.turned {
            if let Some(c) = content_for_fact(&format!("lie:{}", g.lie.tag())) { f *= c.showing; }
        }
    }
    f.clamp(0.3, 1.4)
}

/// The reason a ghost-carrier gives when the ask presses them
/// (E3-4's pressure leg): the strongest ghost's pressure line, or none
pub fn pressure_reason(world: &crate::engine::ecs::World, who: crate::engine::ecs::Entity) -> Option<&'static str> {
    let p = world.get_component::<crate::game::persona::Persona>(who)?;
    p.ghosts.iter().max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
        .and_then(|g| content_for_fact(&format!("ghost:{}", g.kind.tag())).map(|c| c.pressure))
}

/// The tell slugs a persona wears (E3-4): one per ghost's wound and one
/// per lie it taught - the visible habits the rule of three hangs on
pub fn tells_of(p: &crate::game::persona::Persona) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = vec![];
    for g in &p.ghosts {
        // a turned ghost sheds the lie's tell - the wound's habit stays
        // (E8-2): they still keep water by the door, they've stopped
        // saying the lord's men always win
        let mut facts = vec![format!("ghost:{}", g.kind.tag())];
        if !g.turned { facts.push(format!("lie:{}", g.lie.tag())); }
        for fact in facts {
            if let Some(c) = content_for_fact(&fact) {
                if !v.contains(&c.tell) { v.push(c.tell); }
            }
        }
    }
    v
}

fn leaked(s: String) -> &'static str { Box::leak(s.into_boxed_str()) }

/// Two trees per covered fact, built once (E3-4):
/// - `tell:<fact>` - the carrier lets the slip fall mid-talk (`self:`
///   holds the wound; it fires wherever the carrier holds the floor,
///   player talk or passing chat)
/// - `wear:<tell>` - the neighbor names the wear (`other:` carries the
///   tell status) - the gossip leg; the player wears no tells, so it
///   only ever fires villager to villager
/// Both ride the rejoin walk ahead of the bare smalltalk tail: a
/// circumstance or a bond still opens the talk first.
pub fn tell_trees() -> &'static [crate::game::conversation::DialogueTree] {
    use crate::game::conversation::{Act, Criterion::Has, DialogueTree, Exchange, Reply};
    static TREES: std::sync::OnceLock<Vec<DialogueTree>> = std::sync::OnceLock::new();
    TREES.get_or_init(|| {
        let mut v = vec![];
        for c in FACT_CONTENTS {
            let slip: &'static Exchange = Box::leak(Box::new(Exchange {
                act: Act::Remark, concepts: &["smalltalk"],
                calls: std::slice::from_ref(&c.slip), voice: &[],
                // the answer chair can be the player's: replies stay
                // neutral acknowledgments - the wear's gossip naming
                // lives on the `wear:` tree (ambient chats only)
                replies: Box::leak(Box::new([
                    Reply { criteria: &[], weight: 1.0, bias: &[],
                        act: Act::Agree, respond: None, concepts: &["smalltalk"],
                        lines: &["Aye.", "So they say.", "Mm - it may be."],
                        voice: &[], then: None, end: None },
                ])),
            }));
            v.push(DialogueTree { name: leaked(format!("tell:{}", c.fact)),
                criteria: Box::leak(Box::new([Has(leaked(format!("self:{}", c.fact)))])),
                root: slip, rejoin: Some(slip) });
            let wear: &'static Exchange = Box::leak(Box::new(Exchange {
                act: Act::Remark, concepts: &["gossip", "smalltalk"],
                calls: std::slice::from_ref(&c.gossip), voice: &[],
                replies: Box::leak(Box::new([
                    Reply { criteria: &[], weight: 1.0, bias: &[],
                        act: Act::Agree, respond: None, concepts: &["smalltalk"],
                        lines: &["Aye, that's the way of it.", "You've the seeing of it."],
                        voice: &[], then: None, end: None },
                ])),
            }));
            v.push(DialogueTree { name: leaked(format!("wear:{}", c.tell)),
                criteria: Box::leak(Box::new([Has(leaked(format!("other:status:{}", c.tell)))])),
                root: wear, rejoin: Some(wear) });
        }
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::persona::{BELIEFS, EventKind};

    /// E3-4's done-when: every covered fact has the whole rule of
    /// three - a routine tell, a talk slip, a neighbor's remark, a
    /// telling at its tier and a reason under pressure
    #[test]
    fn every_covered_fact_carries_the_three() {
        assert_eq!(FACT_CONTENTS.len(), 20, "the first twenty fact types");
        let mut facts: Vec<&str> = vec![];
        let mut tells: Vec<&str> = vec![];
        for c in FACT_CONTENTS {
            assert!(!facts.contains(&c.fact), "duplicate fact {}", c.fact);
            assert!(!tells.contains(&c.tell), "duplicate tell {}", c.tell);
            facts.push(c.fact); tells.push(c.tell);
            assert!(!c.slip.is_empty() && !c.gossip.is_empty(), "{}: tell legs", c.fact);
            assert!(c.tiers.len() >= 2, "{}: a telling at more than one depth", c.fact);
            assert!(c.tiers.iter().all(|(t, l)| *t >= 2 && !l.is_empty()),
                "{}: tier lines only at the deep tiers", c.fact);
            assert!(!c.pressure.is_empty(), "{}: the choice under pressure", c.fact);
            assert!(c.showing > 0.0 && c.showing <= 1.5, "{}: showing factor sane", c.fact);
        }
    }

    /// The twenty are exactly the ghost kinds and the lies - the wounds
    /// and the lessons the cast report measures the village by
    #[test]
    fn the_twenty_are_the_ghosts_and_the_lies() {
        for k in [EventKind::Fire, EventKind::Levy, EventKind::DeathDuty,
                  EventKind::Fever, EventKind::MillerDebt, EventKind::BrokenBetrothal,
                  EventKind::OldFaith, EventKind::Eviction, EventKind::HarvestFeast] {
            assert!(content_for_fact(&format!("ghost:{}", k.tag())).is_some(),
                "ghost:{} uncovered", k.tag());
        }
        for b in BELIEFS {
            assert!(content_for_fact(&format!("lie:{}", b.tag())).is_some(),
                "lie:{} uncovered", b.tag());
        }
    }

    fn wounded_persona(kind: EventKind, lie: crate::game::persona::Belief) -> crate::game::persona::Persona {
        use crate::game::persona::*;
        let mut p = Persona {
            seed: 1, sex: crate::game::villager::Sex::Male, archetype: Archetype::OldGuard,
            temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
            story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
            hidden: Landmark::Skill("y"), contradiction: None,
            idiolect: crate::game::idiolect::Idiolect {
                phrase: "x", habit: crate::game::idiolect::Habit::Echo },
        };
        p.ghosts.push(Ghost { event: EventId(0), kind, hook: None,
            lie, strength: 0.8, fear: Fear::WhispersTurn, need: Need::ToBeSafe, turned: false });
        p
    }

    /// Leg 1 (the routine tell): a tell status marks the area the asker
    /// can name - the wear points at what it covers
    #[test]
    fn a_tell_names_its_area() {
        let mut s = crate::game::conversation::Speaker::new("A");
        s.status.push("shuns_the_gate");
        assert!(s.tell_areas().contains(&Area::Trouble));
        s.status.push("walks_alone");
        assert!(s.tell_areas().contains(&Area::Heart));
    }

    /// Leg 2 (the narration tell): the carrier lets the slip fall when
    /// they hold the floor - the talk surfaces the wear without telling
    #[test]
    fn the_carrier_lets_the_slip_fall() {
        use crate::game::conversation::{converse, Relation, Rng, Speaker};
        let mut a = Speaker::new("A");
        a.facts.hidden = vec!["ghost:levy".into()];
        a.status.push("shuns_the_gate");
        let b = Speaker::new("B");
        let lib = crate::game::dialogue_trees::village_library();
        let tr = converse(&a, &b, Relation::Acquaintance, None, &lib, None, &mut Rng::new(7));
        assert!(tr.turns.iter().any(|t| t.text == content_for_fact("ghost:levy").unwrap().slip),
            "the slip never fell: {:?}", tr.turns.iter().map(|t| t.text.as_str()).collect::<Vec<_>>());
    }

    /// Leg 3 (the gossip tell): a neighbor names the wear they see -
    /// `other:` carries the tell status, the player never does
    #[test]
    fn a_neighbor_names_the_wear() {
        use crate::game::conversation::{converse, Relation, Rng, Speaker};
        let a = Speaker::new("A");
        let mut b = Speaker::new("B");
        b.status.push("shuns_the_gate");
        let lib = crate::game::dialogue_trees::village_library();
        let tr = converse(&a, &b, Relation::Acquaintance, None, &lib, None, &mut Rng::new(7));
        assert!(tr.turns.iter().any(|t| t.text == content_for_fact("ghost:levy").unwrap().gossip),
            "the wear was never named: {:?}", tr.turns.iter().map(|t| t.text.as_str()).collect::<Vec<_>>());
    }

    /// The telling at depth: a covered fact answers an ask in its area
    /// with its own line, not the topic's generic telling
    #[test]
    fn a_covered_fact_tells_in_its_own_words() {
        use crate::game::conversation::{Facts, Relation, Speaker};
        let mut s = Speaker::new("A");
        s.facts.hidden = vec!["ghost:levy".into()];
        let facts = Facts::for_roles(&s, &Speaker::new("B"), Relation::Stranger);
        let c = content_for_fact("ghost:levy").unwrap();
        let empty = crate::engine::det::HashSet::default();
        assert_eq!(telling(&facts, Area::Trouble, 2, 0.0, &empty), Some(c.tiers[0].1));
        assert_eq!(telling(&facts, Area::Trouble, 3, 0.0, &empty), Some(c.tiers[1].1));
        assert_eq!(telling(&facts, Area::Heart, 2, 0.0, &empty), None, "the wrong area stays generic");
        assert_eq!(telling(&Facts::default(), Area::Trouble, 2, 0.0, &empty), None, "a bare past stays generic");
        // a line already said this talk is spent: the generic telling
        // takes over rather than repeat it
        let mut used = crate::engine::det::HashSet::default();
        used.insert(c.tiers[0].1);
        assert_eq!(telling(&facts, Area::Trouble, 2, 0.0, &used), None);
    }

    /// The pressure leg: a ghost's own reason answers the declined ask,
    /// and the showing factor moves with the wound
    #[test]
    fn the_ghost_answers_the_press() {
        let mut w = crate::engine::ecs::World::new();
        let e = w.create_entity();
        w.add_component(e, wounded_persona(EventKind::Levy, crate::game::persona::Belief::WarsTakeSons));
        assert_eq!(pressure_reason(&w, e), Some("You'll not see me stand under that gate. Ask another."));
        assert!(showing_factor(&w, e) < 1.0, "a gate-shunner hangs back");
        let bare = w.create_entity();
        assert_eq!(pressure_reason(&w, bare), None);
        assert_eq!(showing_factor(&w, bare), 1.0);
    }

    /// Persona tells: one slug per wound, one per lie
    #[test]
    fn the_persona_wears_its_tells() {
        let p = wounded_persona(EventKind::Levy, crate::game::persona::Belief::NeverCrossReeve);
        assert_eq!(p.tells(), vec!["shuns_the_gate", "steps_aside"]);
    }

    /// End to end: the player puts the deep trouble ask and the answer
    /// is the wound's own tier-2 telling, not the topic's stock line
    #[test]
    fn an_ask_in_the_facts_area_draws_its_own_line() {
        use crate::game::conversation::{converse_with, Open, Relation, Rng, Speaker};
        let mut a = Speaker::new("A");
        a.facts.hidden = vec!["ghost:levy".into()];
        a.status.extend(["shuns_the_gate", "keeps_secret"]);
        a.open = Open { intimacy: 1.0, benevolence: 1.0, integrity: 1.0,
            safety: 1.0, mood: 0.5, talks: 5, earshot: false, shared: 1 };
        let b = Speaker::new("B");
        let lib = crate::game::dialogue_trees::village_library();
        let mut phase = 0;
        let tr = converse_with(&a, &b, Relation::Friend, None, &lib, None, 1,
            &mut |choices: &[crate::game::conversation::Choice]| {
                // the ask slot appears only from the second player turn
                if let Some(i) = choices.iter().position(|c| c.preview.contains("Put a question")) {
                    if phase == 0 { phase = 1; return Some(i); }
                }
                if phase == 1 {
                    phase = 2;
                    return choices.iter().position(|c| c.preview.contains("what weighs on them"));
                }
                choices.iter().position(|c| !matches!(c.act, crate::game::conversation::Act::Farewell))
            }, &mut Rng::new(11));
        let c = content_for_fact("ghost:levy").unwrap();
        assert!(tr.turns.iter().any(|t| t.text == c.tiers[0].1),
            "the wound's own telling: {:?}", tr.turns.iter().map(|t| t.text.as_str()).collect::<Vec<_>>());
    }
}
