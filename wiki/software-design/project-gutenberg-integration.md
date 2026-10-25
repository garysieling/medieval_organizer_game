# Project Gutenberg Integration

**Category:** Content / Text Source
**Source:** Backlog item (High) — "Project Gutenberg integration: public domain texts for content."
**Date:** 2026-10-25

## Description

Project Gutenberg = ~70k free public-domain texts — a massive corpus of medieval/period sources usable as in-game text: chronicles, herbals, cookbooks, sermons, travel accounts, letters. The game can draw real period language/text instead of invented pastiche.

## Useful categories for game2

| Corpus | Use |
|---|---|
| **Medieval chronicles** (Froissart, Holinshed, Gregory of Tours) | event/narrative text, village gossip, court chronicle voice |
| **Herbals & medical texts** (Culpeper, Dioscorides) | herbology/apothecary entries — real period knowledge |
| **Cookbooks** (Forme of Cury, Apicius) | recipe system items (`flexible-recipe-system.md`, `tea-tech-tree.md`) |
| **Sermons/devotional** | priest dialogs, sermon events, religious text for `sim/religion.rs` |
| **Travel accounts** (Mandeville, Hakluyt) | explorer/messenger reports, foreign-region descriptions |
| **Letters** (Paston letters, Cely papers) | authentic medieval correspondence — merchant gossip, family news |
| **Fabliaux/folktales** | villager banter, festival tales, humor register (`dantes-divine-comedy.md`) |
| **Agricultural treatises** (Walter of Henley, Crescenzi) | farming technique descriptions, estate-management flavor |

## Mechanic integration

- **Dialog/bark source** — real period phrases for villager chatter; more authentic than invented "ye olde"
- **Book/scroll items** — readable in-game objects pulling text from the corpus; literacy + printing (`printmaking-tech-tree.md`) makes these spread
- **Chronicle generation** — the game's emergent events rendered in period-chronicle voice using the corpus as style reference (ties to `dynamic-magic-story-text.md`, `playthrough-video-generation.md`)
- **Asset & name banks** — saints' names, village names, place names, ship names, heraldic terms
- **Recipe/technique data** — actual period recipes/techniques as the data backing recipe/herbology systems

## Processing pipeline

1. Pull texts (Gutenberg API / bulk download)
2. Chunk by genre/source; index with metadata (date, region, genre)
3. Store in `agent_kb` + a game-side text corpus
4. Retrieval at generation time: query by tag → inject as dialog/event/caption text

## Design principles

- **Period-voice, not period-text verbatim** — use the corpus as *style and vocabulary*, not pasted paragraphs; a little real language goes far
- **Curate, don't dump** — the corpus needs filtering for usable chunks; the LLM pipeline can do genre/tone tagging
- **Attribution** — public domain, but credit sources in-game (a "Sources" codex) — fits the project's intellectual-honesty streak

## Rating

Cheap, high-authenticity content — a real edge the project can pull.
