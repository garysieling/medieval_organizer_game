# Creativity Generator

## Job Description

On request ("I need a character name", "I need a vocation", "I need a plot point", or anything else), produces ~100 ideas grounded in a corpus of sagas, scripture, epics and world mythology. It is a retrieval-augmented generator deliberately biased toward surprise: it injects randomness at every step and throws away the ideas an LLM finds most predictable.

Where the Creative Filter borrows *mechanics* across domains, the Creativity Generator supplies *content*: names, places, trades, creatures, artifacts, plot beats.

Code: `game2-core/src/agents/creativity_generator/` · CLI: `cargo run --release --bin creativity -- "I need a plot point"`

## Pipeline

1. **Parse** the request into a concept kind (`character name`, `place name`, `vocation`, `plot point`, `creature`, `artifact`, or free-form).
2. **Sparks** (randomness): seeded RNG numbers; rare words drawn from random corpus passages; Google Trends daily RSS; Wikipedia most-viewed articles (Wikimedia pageviews API); random Wikipedia and Wiktionary entries. Internet sources are best-effort and skipped on failure / `--offline`.
3. **Retrieve**: BM25 over ~74k paragraph chunks. Each LLM batch gets a different randomised query (kind seed terms + request + sparks), passages from traditions not yet used this run, plus one fully random "wildcard" passage.
4. **Mine** candidates verbatim from retrieved text (proper nouns for names/places, occupational words for vocations, event sentences for plot points), keeping provenance.
5. **Invent**: the LLM writes batches of 25 ideas at temperature 0.9-1.3 with a fresh seed per batch, told to transform rather than copy and to avoid Tolkien/D&D/LLM-cliche names.
6. **Rank and cut**: the LLM orders shuffled batches of candidates from most predictable to most surprising. Obviousness = 0.55 LLM rank + 0.25 consensus (same idea produced by several batches) + 0.2 corpus frequency, +1 for known cliches. The top `head_cut` (default 20%) is discarded; cliches are always cut. The next `count` ideas are returned (default 30% corpus-mined, 70% invented), shuffled, with sources. The cut list is reported too.

Without Ollama it runs corpus-only (mining + heuristic ranking).

## Corpus

Manifest: `wiki/creativity-corpus/sources.tsv` (109 Project Gutenberg texts: Norse/Icelandic sagas and Eddas, Beowulf and Old English, Celtic and Arthurian, Spanish Golden Age and medieval (Cid, Quixote, Amadis, Celestina, Lazarillo, Conde Lucanor...), Hindu epics and puranas, African folklore, Egyptian, Polynesian/Hawaiian/Maori, the Bible and Apocrypha, the Book of Mormon, Augustine's City of God, Gilgamesh and Babylonian, Greek/Roman epic, Finnish Kalevala, Persian, Arabian Nights, Chinese/Japanese, Popol Vuh, and Tolkien's own roots: Morris, MacDonald, Dunsany).

Fetch (re-runnable, Python stdlib only, ~75 MB, git-ignored):

```
python scripts/fetch_creativity_corpus.py            # all
python scripts/fetch_creativity_corpus.py --only norse,polynesian
```

Add a source by appending `gutenberg_id<TAB>tradition<TAB>language<TAB>title` to the manifest. Any `.txt` dropped into `data/creativity-corpus/<tradition>/` is indexed too.

## Skills / Career Ladder

### Junior
- Returns corpus-mined names and terms for a request
- Uses fixed seed terms for retrieval

### Mid
- Blends retrieved passages with LLM invention
- Injects randomness from numbers, corpus words and the internet

### Senior
- Ranks ideas by predictability and cuts the obvious head
- Spreads inspiration across traditions; keeps provenance for every idea

### Staff
- Tunes cut ratios, temperatures and source mixes per concept kind from playtest feedback
- Curates the corpus toward the game's evolving tone

## Claude Skills / Rules

1. **Grounded, not copied** - every idea should trace back to a passage or spark, but invented ideas should transform their source.
2. **The first idea is the wrong idea** - the most predictable fifth of the pool is always discarded.
3. **Randomness is a feature** - vary seeds, temperatures, passages and sparks per batch; record the seed so a run can be reproduced (`--seed`).
4. **Blend traditions** - prefer combinations across cultures over a single source.
5. **Respect sources** - scripture and living traditions are inspiration for fiction, not material to mock; flag anything that would read as a caricature of a real culture or faith.
6. **Quantity, then quality** - return the full count; downstream agents (Creative Filter, Gameplay Consultant, Aesthetic Consultant) pick.

## Options

| flag | default | meaning |
|------|---------|---------|
| `--count` | 100 | ideas returned |
| `--cut` | 0.2 | fraction of most-obvious ideas discarded |
| `--model` | gemma3:4b | Ollama model (`OLLAMA_URL` overrides the endpoint) |
| `--seed` | clock | reproducible runs |
| `--offline` | | no internet sparks |
| `--no-llm` | | corpus-only |
| `--only` | all | comma-separated traditions |
| `--context` | | extra setting/genre context |
| `--out` | stdout | write markdown report |
| `--log` | | append a summary to the KB |

## Learning Mechanism

### Knowledge Base Structure

```
.agents/creativity-generator/kb/
├── generation-history.md   # Runs: request, seed, sample, what was cut (written by --log)
└── corpus-notes.md         # Which traditions/sources produce useful ideas per concept kind
```

### Learning Triggers
1. **After each run with `--log`** - append request, seed, sample ideas, and the cut head.
2. **When ideas are adopted into the game** - note the source tradition and concept kind in `corpus-notes.md`.
3. **When a source produces noise** (glossaries, footnotes, index pages) - note it so it can be cleaned or dropped from the manifest.

## Integration with Other Agents

- **Manager Bot** - capabilities: `idea-generation`, `name-generation`, `plot-generation`, `worldbuilding-ideas`.
- **Creative Filter** - takes Creativity Generator output as raw material for mechanic borrowing.
- **Implementation Engineer** - pulls names/vocations for villagers, guilds and events.
