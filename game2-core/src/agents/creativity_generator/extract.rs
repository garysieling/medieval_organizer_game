//! Concept kinds, request parsing, and corpus mining (candidates pulled straight from the texts).

use super::corpus::{tokenize, Passage};
use std::collections::HashMap;

/// What kind of concept is being asked for
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConceptKind {
    CharacterName,
    PlaceName,
    Vocation,
    PlotPoint,
    Creature,
    Artifact,
    Custom(String),
}

impl ConceptKind {
    /// Parse free text like "I need a character name" or "i need a vocation"
    pub fn parse(request: &str) -> Self {
        let r = request.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|w| r.contains(w));
        if has(&[
            "plot",
            "twist",
            "story beat",
            "quest",
            "conflict",
            "hook",
            "event",
        ]) {
            ConceptKind::PlotPoint
        } else if has(&[
            "place", "city", "town", "kingdom", "realm", "river", "mountain", "location", "region",
        ]) {
            ConceptKind::PlaceName
        } else if has(&[
            "vocation",
            "job",
            "profession",
            "occupation",
            "trade",
            "role",
            "career",
            "guild",
        ]) {
            ConceptKind::Vocation
        } else if has(&["creature", "monster", "beast", "species", "race"]) {
            ConceptKind::Creature
        } else if has(&[
            "artifact", "artefact", "item", "weapon", "relic", "treasure", "sword",
        ]) {
            ConceptKind::Artifact
        } else if has(&["name", "character", "person", "hero", "villain", "npc"]) {
            ConceptKind::CharacterName
        } else {
            let cleaned = r
                .trim_start_matches("i need")
                .trim_start_matches(" a ")
                .trim_start_matches(" an ")
                .trim()
                .to_string();
            ConceptKind::Custom(if cleaned.is_empty() { r } else { cleaned })
        }
    }

    pub fn label(&self) -> String {
        match self {
            ConceptKind::CharacterName => "character name".into(),
            ConceptKind::PlaceName => "place name".into(),
            ConceptKind::Vocation => "vocation".into(),
            ConceptKind::PlotPoint => "plot point".into(),
            ConceptKind::Creature => "creature".into(),
            ConceptKind::Artifact => "artifact".into(),
            ConceptKind::Custom(s) => s.clone(),
        }
    }

    /// Retrieval seed terms for this kind
    pub fn seed_terms(&self) -> Vec<String> {
        let s: &[&str] = match self {
            ConceptKind::CharacterName => &[
                "son", "daughter", "called", "named", "king", "queen", "wife", "kinsman", "lord",
            ],
            ConceptKind::PlaceName => &[
                "land",
                "city",
                "river",
                "mountain",
                "isle",
                "realm",
                "hall",
                "kingdom",
                "dwelt",
                "journeyed",
            ],
            ConceptKind::Vocation => &[
                "smith", "priest", "herdsman", "skald", "bard", "steward", "craft", "servant",
                "merchant", "weaver", "hunter",
            ],
            ConceptKind::PlotPoint => &[
                "betrayed",
                "oath",
                "curse",
                "prophecy",
                "exile",
                "slew",
                "vengeance",
                "stole",
                "fled",
                "foretold",
            ],
            ConceptKind::Creature => &[
                "dragon", "giant", "serpent", "beast", "monster", "spirit", "troll", "demon",
                "bird", "wolf",
            ],
            ConceptKind::Artifact => &[
                "sword", "ring", "cup", "spear", "helm", "stone", "crown", "cloak", "staff",
                "jewel",
            ],
            ConceptKind::Custom(t) => return tokenize(t),
        };
        s.iter().map(|w| w.to_string()).collect()
    }

    /// Format instruction given to the LLM
    pub fn format_hint(&self) -> &'static str {
        match self {
            ConceptKind::CharacterName => "Each line: a name, then ' - ' and a few words on who they are.",
            ConceptKind::PlaceName => "Each line: a place name, then ' - ' and a few words describing it.",
            ConceptKind::Vocation => "Each line: a vocation/trade/office, then ' - ' and one sentence on what they do in this world.",
            ConceptKind::PlotPoint => "Each line: one plot point in one or two sentences.",
            ConceptKind::Creature => "Each line: a creature name, then ' - ' and a short description.",
            ConceptKind::Artifact => "Each line: an artifact name, then ' - ' and its power or history.",
            ConceptKind::Custom(_) => "Each line: one idea, short.",
        }
    }

    /// Ideas whose main text is a short label (dedupe on the label)
    pub fn is_short_form(&self) -> bool {
        !matches!(self, ConceptKind::PlotPoint | ConceptKind::Custom(_))
    }
}

/// A candidate pulled directly from the corpus
#[derive(Debug, Clone)]
pub struct Mined {
    pub text: String,
    pub note: String,
    pub source: String,
    pub tradition: String,
    /// How many times it appeared in the retrieved passages
    pub frequency: usize,
}

const NOT_NAMES: &[&str] = &[
    "the",
    "and",
    "but",
    "then",
    "now",
    "when",
    "there",
    "this",
    "that",
    "they",
    "thou",
    "thee",
    "thy",
    "his",
    "her",
    "him",
    "she",
    "for",
    "with",
    "from",
    "into",
    "upon",
    "unto",
    "after",
    "before",
    "yet",
    "nor",
    "lord",
    "god",
    "king",
    "queen",
    "chapter",
    "book",
    "canto",
    "part",
    "verse",
    "all",
    "what",
    "who",
    "whom",
    "how",
    "why",
    "where",
    "which",
    "our",
    "your",
    "you",
    "may",
    "let",
    "said",
    "saith",
    "behold",
    "also",
    "even",
    "here",
    "one",
    "two",
    "three",
    "not",
    "came",
    "went",
    "these",
    "those",
    "some",
    "many",
    "such",
    "great",
    "good",
    "sir",
    "lady",
    "mr",
    "mrs",
    "project",
    "gutenberg",
    "ebook",
    "note",
    "footnote",
    "illustration",
    "she",
    "its",
    "has",
    "have",
    "had",
    "was",
    "were",
    "are",
    "say",
    "come",
    "go",
    "see",
    "while",
    "though",
    "although",
    "because",
    "so",
    "if",
    "of",
    "in",
    "on",
    "at",
    "to",
    "as",
    "by",
    "no",
    "or",
    "an",
    "be",
    "do",
    "he",
    "it",
    "me",
    "my",
    "we",
    "us",
    "ye",
    "o",
    "oh",
    "lo",
    "ah",
    "thus",
    "hence",
    "english",
    "latin",
    "greek",
    "christ",
    "jesus",
    "israel",
    "rome",
    "england",
    "france",
    "spain",
    "india",
    "africa",
    "europe",
    "america",
    "london",
    "english",
    "translated",
    "translator",
    "introduction",
    "preface",
    "contents",
    "index",
];

const PLACE_PREPS: &[&str] = &[
    "of", "in", "to", "from", "at", "towards", "unto", "across", "beyond",
];

const VOCATION_SUFFIXES: &[&str] = &[
    "smith",
    "wright",
    "herd",
    "monger",
    "keeper",
    "maker",
    "wife",
    "man",
    "master",
    "binder",
    "weaver",
    "seller",
    "bearer",
    "reeve",
    "warden",
    "priest",
    "priestess",
    "singer",
    "teller",
    "hunter",
    "fisher",
    "rider",
    "carver",
];

const VOCATION_WORDS: &[&str] = &[
    "priest",
    "priestess",
    "skald",
    "bard",
    "druid",
    "brahmin",
    "rishi",
    "sage",
    "seer",
    "augur",
    "scribe",
    "steward",
    "chamberlain",
    "jarl",
    "thane",
    "thegn",
    "squire",
    "page",
    "herald",
    "minstrel",
    "jongleur",
    "troubadour",
    "cooper",
    "tanner",
    "chandler",
    "potter",
    "vintner",
    "falconer",
    "fletcher",
    "bowyer",
    "armourer",
    "armorer",
    "mason",
    "miller",
    "brewer",
    "baker",
    "butcher",
    "shepherd",
    "swineherd",
    "goatherd",
    "cowherd",
    "ploughman",
    "reaper",
    "ferryman",
    "boatman",
    "helmsman",
    "pilot",
    "navigator",
    "physician",
    "leech",
    "healer",
    "midwife",
    "astrologer",
    "alchemist",
    "sorcerer",
    "sorceress",
    "witch",
    "wizard",
    "enchanter",
    "enchantress",
    "shaman",
    "diviner",
    "oracle",
    "prophet",
    "prophetess",
    "hermit",
    "anchorite",
    "monk",
    "nun",
    "abbot",
    "abbess",
    "friar",
    "pilgrim",
    "palmer",
    "pardoner",
    "summoner",
    "bishop",
    "deacon",
    "levite",
    "rabbi",
    "imam",
    "guru",
    "yogi",
    "ascetic",
    "mendicant",
    "charioteer",
    "archer",
    "spearman",
    "berserker",
    "viking",
    "mercenary",
    "champion",
    "outlaw",
    "brigand",
    "pirate",
    "corsair",
    "merchant",
    "trader",
    "peddler",
    "usurer",
    "moneylender",
    "goldsmith",
    "silversmith",
    "jeweller",
    "lapidary",
    "embroiderer",
    "dyer",
    "fuller",
    "spinner",
    "carpenter",
    "shipwright",
    "wheelwright",
    "thatcher",
    "woodcutter",
    "forester",
    "gamekeeper",
    "huntsman",
    "fowler",
    "trapper",
    "fisherman",
    "pearl-diver",
    "diver",
    "navigator",
    "steersman",
    "cupbearer",
    "taster",
    "jester",
    "fool",
    "juggler",
    "dancer",
    "drummer",
    "piper",
    "harper",
    "lutenist",
    "poet",
    "chronicler",
    "genealogist",
    "lawspeaker",
    "judge",
    "magistrate",
    "tax-gatherer",
    "publican",
    "centurion",
    "tribune",
    "satrap",
    "vizier",
    "emir",
    "rajah",
    "chieftain",
    "headman",
    "elder",
    "matriarch",
    "nurse",
    "tutor",
    "gaoler",
    "jailer",
    "executioner",
    "headsman",
    "torturer",
    "spy",
    "messenger",
    "courier",
    "runner",
    "scout",
    "guide",
    "interpreter",
    "envoy",
    "ambassador",
    "hostage",
    "slave",
    "thrall",
    "bondsman",
    "serf",
    "villein",
    "yeoman",
    "franklin",
    "knight",
    "paladin",
    "templar",
    "hospitaller",
    "crusader",
    "almoner",
    "sexton",
    "gravedigger",
    "embalmer",
    "mourner",
    "keener",
    "soothsayer",
    "necromancer",
    "exorcist",
    "relic-seller",
    "beekeeper",
    "salter",
    "collier",
    "miner",
    "quarryman",
    "stonecutter",
    "sculptor",
    "painter",
    "illuminator",
    "copyist",
    "bookbinder",
    "papermaker",
    "glassblower",
    "bellfounder",
    "locksmith",
    "cutler",
    "saddler",
    "farrier",
    "groom",
    "ostler",
    "innkeeper",
    "taverner",
    "cook",
];

const VOCATION_STOP: &[&str] = &[
    "woman",
    "man",
    "wife",
    "master",
    "keeper",
    "maker",
    "forward",
    "toward",
    "towards",
    "afterward",
    "backward",
    "inward",
    "outward",
    "upward",
    "downward",
    "reward",
    "seaward",
    "homeward",
    "herd",
    "human",
    "german",
    "norman",
    "roman",
    "yeoman",
    "chairman",
    "nobleman",
    "gentleman",
    "kinsman",
    "countryman",
    "husbandman",
    "freeman",
    "foeman",
    "madman",
    "fireman",
    "oman",
];

const PLOT_VERBS: &[&str] = &[
    "betray",
    "betrayed",
    "swore",
    "sworn",
    "oath",
    "cursed",
    "curse",
    "prophesied",
    "foretold",
    "prophecy",
    "exiled",
    "banished",
    "slew",
    "slain",
    "murdered",
    "avenged",
    "vengeance",
    "stole",
    "stolen",
    "fled",
    "escaped",
    "disguised",
    "kidnapped",
    "carried",
    "married",
    "wed",
    "burned",
    "besieged",
    "drowned",
    "transformed",
    "turned",
    "sold",
    "ransom",
    "hostage",
    "poisoned",
    "bewitched",
    "deceived",
    "tricked",
    "wager",
    "dreamed",
    "vowed",
    "forbade",
    "forbidden",
    "outlawed",
    "hid",
    "buried",
    "found",
    "returned",
    "rescued",
    "challenged",
    "dueled",
    "wrestled",
];

/// Pull candidates of `kind` straight from retrieved passages
pub fn mine(kind: &ConceptKind, passages: &[&Passage]) -> Vec<Mined> {
    match kind {
        ConceptKind::CharacterName => mine_proper_nouns(passages, false),
        ConceptKind::PlaceName => mine_proper_nouns(passages, true),
        ConceptKind::Vocation => mine_vocations(passages),
        _ => mine_sentences(kind, passages),
    }
}

fn mine_proper_nouns(passages: &[&Passage], places: bool) -> Vec<Mined> {
    let lowercase_seen = lowercase_words(passages);
    let mut found: HashMap<String, Mined> = HashMap::new();
    for p in passages {
        let words: Vec<&str> = p.text.split_whitespace().collect();
        for i in 1..words.len() {
            let prev_raw = words[i - 1];
            if prev_raw.ends_with(['.', '!', '?', ':', ';', '"', '\u{201c}']) {
                continue;
            }
            let w = words[i].trim_matches(|c: char| !c.is_alphabetic() && c != '-' && c != '\'');
            let w = w
                .trim_end_matches("'s")
                .trim_matches('\'')
                .trim_matches('-');
            let mut chars = w.chars();
            let Some(first) = chars.next() else { continue };
            if !first.is_uppercase()
                || w.chars().count() < 3
                || w.chars().skip(1).all(|c| c.is_uppercase())
            {
                continue;
            }
            let lower = w.to_lowercase();
            if !w
                .chars()
                .all(|c| c.is_alphabetic() || c == '-' || c == '\'')
                || NOT_NAMES.contains(&lower.as_str())
                || lowercase_seen.contains(&lower)
            {
                continue;
            }
            let prev = prev_raw
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase();
            let is_place_ctx = PLACE_PREPS.contains(&prev.as_str());
            if places != is_place_ctx {
                continue;
            }
            let entry = found.entry(w.to_string()).or_insert_with(|| Mined {
                text: w.to_string(),
                note: snippet(&p.text, w),
                source: p.source.clone(),
                tradition: p.tradition.clone(),
                frequency: 0,
            });
            entry.frequency += 1;
        }
    }
    let mut v: Vec<Mined> = found.into_values().collect();
    v.sort_by(|a, b| b.frequency.cmp(&a.frequency).then(a.text.cmp(&b.text)));
    v
}

/// Words that appear starting with a lowercase letter somewhere in the passages
fn lowercase_words(passages: &[&Passage]) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for p in passages {
        for w in p.text.split(|c: char| !c.is_alphabetic()) {
            if w.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                set.insert(w.to_string());
            }
        }
    }
    set
}

fn mine_vocations(passages: &[&Passage]) -> Vec<Mined> {
    let mut found: HashMap<String, Mined> = HashMap::new();
    for p in passages {
        let lowercase_seen = lowercase_words(&[*p]);
        for tok in tokenize(&p.text) {
            if !lowercase_seen.contains(&tok) {
                continue;
            }
            let is_voc = VOCATION_WORDS.contains(&tok.as_str())
                || (tok.len() > 5
                    && VOCATION_SUFFIXES
                        .iter()
                        .any(|s| tok.ends_with(s) && tok.len() > s.len() + 1)
                    && !VOCATION_STOP.contains(&tok.as_str()));
            if !is_voc {
                continue;
            }
            let entry = found.entry(tok.clone()).or_insert_with(|| Mined {
                text: tok.clone(),
                note: snippet(&p.text, &tok),
                source: p.source.clone(),
                tradition: p.tradition.clone(),
                frequency: 0,
            });
            entry.frequency += 1;
        }
    }
    let mut v: Vec<Mined> = found.into_values().collect();
    v.sort_by(|a, b| b.frequency.cmp(&a.frequency).then(a.text.cmp(&b.text)));
    v
}

fn mine_sentences(kind: &ConceptKind, passages: &[&Passage]) -> Vec<Mined> {
    let seeds = kind.seed_terms();
    let mut out = Vec::new();
    for p in passages {
        for s in p.text.split_inclusive(['.', '!', '?']) {
            let s = s.trim();
            if s.len() < 50 || s.len() > 240 {
                continue;
            }
            let toks = tokenize(s);
            let hit = match kind {
                ConceptKind::PlotPoint => toks.iter().any(|t| PLOT_VERBS.contains(&t.as_str())),
                _ => toks.iter().any(|t| seeds.contains(t)),
            };
            if hit {
                out.push(Mined {
                    text: s.to_string(),
                    note: String::new(),
                    source: p.source.clone(),
                    tradition: p.tradition.clone(),
                    frequency: 1,
                });
            }
        }
    }
    out
}

/// ~120 chars of context around `needle`
pub fn snippet(text: &str, needle: &str) -> String {
    let idx = text.find(needle).unwrap_or(0);
    let mut start = idx.saturating_sub(60);
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = (idx + needle.len() + 60).min(text.len());
    while end < text.len() && !text.is_char_boundary(end) {
        end += 1;
    }
    format!("...{}...", text[start..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(text: &str) -> Passage {
        Passage {
            source: "test".into(),
            tradition: "norse".into(),
            text: text.into(),
        }
    }

    #[test]
    fn test_parse_kinds() {
        assert_eq!(
            ConceptKind::parse("I need a character name"),
            ConceptKind::CharacterName
        );
        assert_eq!(
            ConceptKind::parse("i need a vocation"),
            ConceptKind::Vocation
        );
        assert_eq!(
            ConceptKind::parse(" i need a plot point"),
            ConceptKind::PlotPoint
        );
        assert_eq!(
            ConceptKind::parse("name for a kingdom"),
            ConceptKind::PlaceName
        );
        assert_eq!(
            ConceptKind::parse("I need a tavern song"),
            ConceptKind::Custom("tavern song".into())
        );
    }

    #[test]
    fn test_mine_names_and_places() {
        let a = p("Then came Hrafnkel the priest of Frey, riding from Adalbol with Sam.");
        let names: Vec<String> = mine(&ConceptKind::CharacterName, &[&a])
            .into_iter()
            .map(|m| m.text)
            .collect();
        assert!(names.contains(&"Hrafnkel".to_string()));
        assert!(!names.contains(&"Then".to_string()));
        let places: Vec<String> = mine(&ConceptKind::PlaceName, &[&a])
            .into_iter()
            .map(|m| m.text)
            .collect();
        assert!(places.contains(&"Adalbol".to_string()));
    }

    #[test]
    fn test_mine_vocations() {
        let a = p("The swordsmith and the skald sat with the woman and the ferryman.");
        let v: Vec<String> = mine(&ConceptKind::Vocation, &[&a])
            .into_iter()
            .map(|m| m.text)
            .collect();
        assert!(v.contains(&"swordsmith".to_string()));
        assert!(v.contains(&"skald".to_string()));
        assert!(v.contains(&"ferryman".to_string()));
        assert!(!v.contains(&"woman".to_string()));
    }

    #[test]
    fn test_mine_plot_sentences() {
        let a = p("The weather was mild that spring in the valley. Gudrun swore an oath that her brothers would pay for the killing.");
        let v = mine(&ConceptKind::PlotPoint, &[&a]);
        assert_eq!(v.len(), 1);
        assert!(v[0].text.starts_with("Gudrun"));
    }
}
