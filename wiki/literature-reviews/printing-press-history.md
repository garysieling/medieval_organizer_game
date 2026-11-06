# Printing Press History — Chronology for the Printing Arc

**Survey Date:** 2026-11-05
**Domain:** History of Technology / Printmaking / Book History
**Status:** COMPLETE — targeted research pass for the printer feature arc (errands, fundraising/flyers, cutaway, era-based press designs)
**Confidence:** High

## Why this exists

The printer feature arc needs era-correct press designs ("a litho stone, a Gutenberg press, later the large Kelsey style presses"). Gary flagged that the remembered order "isn't 100% historical" — this survey fixes the sequence against sources before D1–D5 wire it into `Game`/`UiModel`/sprites.

## The corrected chronology

### Substrates (the gate)

| ~Date | Milestone | Note |
|---|---|---|
| ~3000 BCE | Papyrus (Egypt) | Plant-pith sheet, Mediterranean standard for ~3000 yrs |
| ~200 BCE | Parchment/vellum | Animal skin; durable but expensive (14-day liming/scraping process) |
| 105 CE | Paper invented (China, Ts'ai Lun) | Rag-based sheet |
| 8th c. | Paper reaches Islamic world | Samarkand mills ~751 |
| ~1390 | Paper mills in Europe (Nuremberg/Ulma) | The enabler: cheap paper makes print runs pay |

Paper mill is the real prerequisite for everything below — already reflected in `extended_printmaking_tree()` ("Paper Mill" gates printmaking).

### Image/text multiplication (pre-type)

| ~Date | Milestone | Note |
|---|---|---|
| ~7th–8th c. | Woodblock printing, China | 868 Diamond Sutra is the earliest dated complete printed book |
| ~1430s | Copperplate engraving, Europe | Intaglio: ink held in incised lines; fine detail, maps, currency |
| 15th c. | Block books, playing cards, devotional prints (Europe) | Mass market arrives as *images*, not books (Met Museum finding) |

### Movable type — East Asian precedents

| ~Date | Milestone | Note |
|---|---|---|
| 1041–48 | Bi Sheng's clay type (Song China) | Earthenware characters, baked; too fragile for heavy reuse |
| 1234–41 | Korean metal type (Detailed Rituals of Old and New) | Cast-metal type precedes Gutenberg by ~200 yrs |
| 1298 | Wangzhen's wooden type (Yuan China) | Rotating type cases — compositing workflow |
| 1377 | **Jikji** — oldest extant book printed from metal type (Heungdeok-sa, Cheongju) | UNESCO Memory of the World; proof metal type was mature in Korea |
| 1403–34 | Korean copper type faces (Gyemi, Gyeongja, Byeongjin) | State-scale type foundries |
| 1447 | Movable Hangul type (Korea) | Alphabet-specific type, ~8 yrs before Gutenberg's Bible |

### Movable type & press — Europe

| ~Date | Milestone | Note |
|---|---|---|
| ~1440–50 | Gutenberg's workshop, Mainz | Adapted a wine/cheese **screw press**; wooden frame, hand-pulled bar |
| ~1454/55 | 42-line Bible (Gutenberg Bible) | ~180 copies; the icon of the revolution |
| 1450s–1500s | Print spreads: ~250 European towns have presses by 1500 | ~8–10M books by 1500 (incunabula) |
| ~1550–1800 | **Common press** era | The standard wooden hand press, essentially unchanged ~350 yrs |

**Common press mechanics (for sprite/anim reference):** wooden frame (cheeks often built into the ceiling), iron screw, heavy pull-bar. A *two-pull* press prints half a form at a time. Two pressmen (beater inks the type, puller works the bar) produce ~240 impressions/hour — a "token". Fewer than ~70 survive worldwide; none pre-17th c.

### The machine age (19th century — where the "large presses" live)

| ~Date | Milestone | Note |
|---|---|---|
| **1796** | Senefelder discovers stone printing (Munich) | Greasy crayon on polished Solnhofen limestone; water/grease repel |
| **1798** | Planography patented ("chemical printing") | The process lithography is named for |
| ~1820 | Lithography commercially popular | Cheap posters, maps, sheet music, advertising |
| **1800** | **Stanhope press** — first all-iron book press | Earl Stanhope; compound levers, full folio in one pull |
| **1814** | **Koenig steam press** debuts (The Times, London) | First powered press; ~1,000 sheets/hour, ~10x hand press |
| ~1820 | **Albion press** (R.W. Cope, London) | Iron hand press; still made today; toggle-joint action |
| 1828 | Imperial press (Cope, Sherwin & Co.) | Larger, more powerful cousin of the Albion |
| 1830 | Zinc replaces limestone for litho plates | Cheaper, portable plates |
| 1837 | **Chromolithography** patented (Engelmann, Mulhouse) | Multi-stone color printing; term coined this year |
| 1840s | Color litho commercial boom | Advertising posters, trade cards |
| 1843–51 | **Hoe rotary press** (Richard March Hoe, NY) | Metal cylinder instead of flat platen; ~10,000 sheets/hour by 1851 — 100x the 1800 hand press |
| 1872/75+ | **Kelsey Excelsior** (William A. Kelsey Co., Meriden CT) | Hobby/table-top press (3x5–6x10 beds) for amateurs; ran to ~1990 |
| 1875/1904 | Offset lithography (Barclay tin / Rubel paper) | Blanket-transfer process; the modern newspaper press |

### The ordering correction

The remembered sequence "litho stone → Gutenberg press → large Kelsey-style presses" is wrong in a specific way:

- **Lithography (1796) is ~350 years AFTER Gutenberg (1450).** Senefelder's stone is an early-*industrial*-era machine, not a medieval one. In game terms it belongs two tiers above the screw press, not beside woodblock.
- **The "large presses" are the 19th-century iron/steam/rotary machines** — Stanhope (1800), Koenig steam (1814), Albion/Imperial (~1820–28), Hoe rotary (1840s–51). 
- **Kelsey presses were small.** The William A. Kelsey Company of Meriden, Connecticut made inexpensive *table-top hobby presses* for amateurs from ~1872 onward — the 3x5/5x8/6x10 Excelsior is a desktop machine, not an imposing works press. If "Kelsey style" means "large antique press," the better visual referents are the iron platen presses (Stanhope/Albion/Imperial) and the big cylinder/rotary machines. **Recommend:** name the tier "iron hand press" / "steam press" / "rotary press" in-game; keep Kelsey as a possible *small* shop variant, not the flagship.

### Era bands for game use

| Band | Years | Press imagery | Throughput feel |
|---|---|---|---|
| Hand-press era | ~1450–1800 | Wooden screw press, two-pull common press | Sheets/hour; one pull = half a form |
| Planographic era | 1796–~1900 | Litho stone + scraper/baren, later zinc plate | Draw-on-stone; cheap posters |
| Machine era | 1800–~1900 | Iron platen → steam cylinder → rotary | 10x–100x hand press; steam whistle |
| Offset era | 1904+ | Rubber blanket + plate cylinder | Modern newspaper/magazine |

The bands overlap (litho and machine presses coexist ~1800–1900) — that's historically accurate and gives the tech tree a *web*, not a line, matching `sprint-4-printmaking.md`'s finding.

## Game mapping (feeds D1–D5)

- **D1 tech state:** era bands above map cleanly onto `Tech.era` — current tree's `"Late"` lumps press+litho together; split into `early_modern` (press refinements, iron hand press) and `industrial` (steam/rotary, offset).
- **D5 sprite progression:** wooden screw press → common press (visually near-identical; differentiate by size/condition) → litho stone slab → iron hand press → steam cylinder press → rotary drum. Each is a distinct silhouette — good `BSprite` candidates.
- **D2 errands / D3 flyers:** flyers/posters are exactly what lithography was *invented* for (playbills, posters, ads) — era-gate flyer commissioning on litho; handbills on the common press before that.
- **Cutaway (D4):** common-press interior is simple (screw, platen, tympan, frisket) — drawable as a labeled cutaway; steam press adds flywheel/belts.

## Sources

| Source | Rating | Used for |
|---|---|---|
| APHA History of Printing Timeline (printinghistory.org) | A- | Master chronology spine; woodcut press image 1499; incunabula landmarks |
| Smithsonian NMAH — "Wooden Hand Presses" + press collection | A+ | Common press mechanics: two-pull, ~240 imp/hr token, wooden frame/iron screw, <70 survive |
| Franklin & Marshall LibGuides — Hand Press 1450–1800 / Mechanization 1800–1950 | A | Press era split; Stanhope 1803, Koenig ~1811–14, rotary 1851 |
| Museum of Printing (North Andover) — press history poster | A | Gutenberg 1440 wine/cheese press adaptation; Ruthven/Albium/Washington lineage |
| Global Jikji timeline (globaljikji.org) | A+ | East Asian movable-type dates: Bi Sheng 1041–48, Korean metal type 1234, Jikji 1377, Hangul 1447 |
| Rijksmuseum — *From Jikji to Gutenberg* (2020 project) | A | Jikji = oldest extant metal-type book; UNESCO register |
| DPMA (German patent office) — Senefelder biography | A | Lithography 1796 discovery, 1798 planography, Solnhofen limestone |
| Met Museum — "Lithography in the Nineteenth Century" | A+ | ~1796 invention; commercial spread; 1880s–90s poster boom |
| History of Science Museum (Oxford) — Lithography | A | 1798/1820 commercial dates; zinc 1830, aluminium 1890 |
| Wikipedia — Chromolithography / Offset printing | B+ | Engelmann patent July 1837; term coined 1837; offset 1875/1904; Hoe 1843 rotary |
| Smithsonian NMAH — Kelsey Excelsior 5x8 press object record | A+ | Kelsey = Meriden CT hobby press, mid-20th c. example; amateur-press lineage 1872 |
| ExcelsiorPress.org / privatepress.org — Kelsey catalogs | B | Kelsey Excelsior sizes (3x5, 5x8, 6x10), production 1875–1990 |
| Chetham's Library / NIU — common press records | A | Common press construction details, pressmen roles |

## Mechanics proposed (for the arc)

1. **Era-gated press sprites** — `PressDesign` enum keyed to tech node; oldest unlocked design renders on the printer's building.
2. **Throughput tiers** — hand press ~240 imp/hr → steam ~1,000/hr → rotary ~10,000/hr; flyer/pamphlet job size scales with design.
3. **Litho = poster medium** — flyers/posters around town (D3) get a fidelity/coverage boost once litho is known; handbills remain the pre-litho option.
4. **Cutaway contents** — press interior changes per design (screw+platen → flywheel+steam → cylinder+blanket).
5. **Kelsey note** — if a "small shop press" variant is wanted, Kelsey Excelsior is the historically right object (hobby tier, post-1870).
