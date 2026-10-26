# Lit Review - Sprint 7: Luthiery and Minstrelsy (String Instruments as Two Vocations)

**Date:** 2026-10-25
**Domain:** Music History / Organology / Craft Guilds
**Backlog items:** Luthier / string instruments (both playing and making are vocations); Travelling circus (music); relates to the [evolving music system](../software-design/evolving-music-system.md)
**Sources:** Metropolitan Museum of Art "The Lute" essay; Wikipedia "History of lute-family instruments"; Early Music Muse "The rebec" and "The psaltery"; Case Western Early Music Instrument Database "Hurdy-gurdy (Medieval)"; HAL "The role of tonewood selection and aging in instrument quality as viewed by violin makers"; UND thesis "The history, evolution, and maintenance of violin strings"; Haines "From Jongleur to Minstrel" and "Minstrels, Guild Organization... Jongleurs of Paris, 1292-1350" (JHU); "Confrérie, Bruderschaft and guild: musicians' fraternal organisations" (Early Music History)

## Sources

### 1. Lute Family History (Met Museum; Wikipedia)
- European lute descends from the Arab **'ud**: pear-shaped body, **bowl back built from many thin wooden ribs**, wooden soundboard, carved **rose** soundhole, **gut strings**, bent-back pegbox
- Spread through al-Andalus and Sicily; lute-type instruments also existed in Greco-Roman antiquity (contested origin)
- Short lutes spread east too (pipa in China, vina in India)
- **Relevance:** HIGH - instruments arrive through trade/cultural contact, a reason for cross-faction exchange

### 2. Medieval String Instrument Roster (Early Music Muse; Case MEDREN database)
| Instrument | Type | Social setting |
|---|---|---|
| Lute / gittern / citole | plucked | court, tavern |
| Harp | plucked | bards, court, prestige |
| Psaltery | plucked/struck box zither | church imagery, angels, court |
| Rebec | bowed, pear-shaped, 3 strings | from court down to street and dance |
| Vielle / fiddle | bowed | jongleurs, dance, epic recitation |
| Hurdy-gurdy (organistrum) | cranked wheel + keys | started as a 2-person church instrument, became beggars'/peasants' instrument |
- **Relevance:** HIGH - instruments **move down the status ladder** over time (hurdy-gurdy, rebec), which makes status drift a mechanic

### 3. Tonewood and Strings (HAL violin-maker study; UND string thesis; general organology)
- Makers choose **spruce for soundboards, maple (or other hardwoods) for backs/ribs**; **seasoning/aging the wood** for years is believed to improve quality
- Makers judge wood by tap tone, density, grain straightness, a mix of measurable traits and tacit skill
- Strings were **gut** (sheep intestine), made by specialist string-makers; they break, go out of tune with humidity, need replacing
- **Relevance:** HIGH - inputs chain to forestry (old trees, seasoned stock) and shepherds (gut)

### 4. Jongleurs, Minstrels and Their Guilds (Haines; Early Music History)
- **Jongleurs**: itinerant performers (music, juggling, tales), low status, often distrusted by church authorities
- **Minstrels**: the professionalized, often household-retained version; a patron gives livery, wages, protection
- Paris minstrels formed a **confraternity/guild** (statutes 1321): rules on apprenticeship, who could hire, territorial rights, a hospital for members
- Across Europe, musicians organized confréries/Bruderschaften: **mutual aid + monopoly**
- **Relevance:** CRITICAL - the itinerant-to-guild arc is exactly the vocation/apprenticeship/org-membership model

## Mechanics

### Two vocations
- **Luthier:** inputs = seasoned tonewood (time-gated: wood must age), gut strings, glue, tools (from the toolmaker). Quality = skill × wood quality × time
- **Musician:** skill per instrument; performance quality = skill × instrument quality × audience fit (a court wants the harp, a tavern the fiddle)
- **Feedback:** better musicians demand better instruments; luthiers' reputations travel with the musicians who play their work (vocation-driven relationships)

### Status and career
- Career paths: **jongleur (wandering, low status) → household minstrel (patron) or guild minstrel (town)**
- Patronage gives security but ties you to a patron's fortunes; the guild gives mutual aid but enforces rules and territory
- Instruments have **status drift**: popularity flows downward through classes over generations

### Music as social glue
- Performances raise group mood and build relationships among listeners; songs carry news and satire (gossip vector via `sim/gossip.rs`)
- A satirical song about a lord is a **non-violent weapon** (pairs with the deposition review)

## Design Proposals

1. **Luthier workshop** with a wood-seasoning stock: planning years ahead is the core luthier decision
2. **Instrument items** with maker, wood, age, and quality; famous instruments become heirlooms with history
3. **Minstrel career ladder:** wandering → patron or guild; travelling circus is the itinerant org option
4. **Musicians' guild** as a template org: apprenticeship, territory, mutual aid fund, hiring rules
5. **Songs as media:** player/NPC-composed songs spread reputation (praise or satire) across regions
6. **Status drift** of instruments over generations, a cheap source of cultural change for the evolving music system

## Rating
**High** - two interlocking vocations with deep input chains (forestry, shepherding, toolmaking) and a ready-made guild/patronage career model; music also works as a social and satirical medium.
