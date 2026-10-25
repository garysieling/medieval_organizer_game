# Faith Axiom System — Combinatorial Religion Generator

**Category:** Design / Religion System
**Source:** Targeted research pass (comparative religions) — Gary's ask: "a set of axioms that can be combined into a faith (with a made up name)... descent-based religion converts rarely; if you violate a rule, conflicts"
**Research base:** `wiki/literature-reviews/sprint-6-comparative-religions.md`
**Status:** design thinking pass — NO implementation

## The model

A religion is **a named vector of axioms** — a small number of discrete choices over shared structural dimensions. Any coherent combination is a plausible faith; real religions are particular vectors. The sim doesn't need Christianity or Judaism — it needs the *axiom-space* those religions occupy.

## Axiom catalogue (18 dimensions)

### A. Membership & boundaries

1. **membership_mode** — who counts and how you join:
   - `descent_matrilineal` / `descent_patrilineal` — born into it; conversion rare/costly (Judaism, Zoroastrianism, Hinduism)
   - `mission_universal` — actively recruits anyone; simple entry (Christianity, Islam, Buddhism)
   - `territorial_civic` — you belong by living there (Roman civic religion, polis cult)
   - `initiatory` — join by rite/knowledge, possibly secret (Mithraism, mystery cults, Manichaean elect)
   - `covenant_chosen` — special people bound by pact; joining is an exception (Judaism's particular variant)
2. **boundary_strictness** — how hard it is to enter/leave/marry-in: `open` / `regulated` / `closed`
   - interacts with membership: `descent_* + closed` = nearly no converts (parsis); `mission + open` = converts flood in
3. **exogamy_rule** — may members marry outsiders? `forbidden` / `permitted-if-converts` / `permitted` / `encouraged`

### B. What's sacred

4. **god_structure** — `strict_monotheism` / `trinitarian` / `dualist_good_evil` / `polytheist_portfolio` / `monist_all_one` / `non_theistic_path` / `spirit_world_local`
5. **truth_stance** — `exclusive` (only we are right) / `supersessionist` (we fulfill/replace an older faith — points at another religion!) / `accommodative` (many paths) / `parallel_valid` (your gods for you, mine for me)
6. **incarnation** — has god become human/animal? `never` / `once` / `recurring` (avatars) / `always_present` (immanent spirit world)
7. **icon_stance** — `aniconic` (images forbidden — iconoclasm-ready) / `venerated` (icons/relics are power-channels) / `neutral`

### C. What you must do/believe

8. **orthopraxy_orthodoxy** — the axis Gary intuited ("violate a rule → conflict"):
   - `orthopraxic` — correct *conduct* is the boundary (Judaism, Islam, Hinduism, civic cults): the rules that matter are visible acts (food, dress, calendar, rites). Violations are *ritual pollution* — fixed by rites, fines, exclusion
   - `orthodoxic` — correct *belief* is the boundary (Christianity, Gnosis): violations are *heresy* — fixed by recantation, penance, or worse. Heresy-hunting only exists on this side!
   - `mixed` — both conduct and belief enforced
9. **law_scope** — `total_life_code` (food/dress/sex/commerce/calendar — halakha, sharia, dharma) / `ritual_only` / `ethical_only` / `customary`
10. **purity_system** — `strong` (pollution logic, dietary laws, caste rules, corpse avoidance) / `moderate` (fasts, ablutions) / `weak`
11. **calendar_rhythm** — `lunar` / `solar` / `mixed` + sabbath-strictness: `strict_day_of_rest` / `festival_cycle_only` / `none`

### D. Cosmology & fate

12. **time_structure** — `linear_eschatological` (creation→judgment, history matters) / `cyclical_rebirth` (yugas, samsara) / `seasonal_agrarian` (the year is the sacred shape)
13. **afterlife** — `judgment_resurrection` / `heaven_hell` / `purgatory_economy` (pray/pay for the dead — the medieval European afterlife-economy axiom) / `reincarnation_exit` / `ancestor_continuation` (dead stay as kin-spirits) / `shadow_underworld` / `deemphasized`
14. **salvation_mechanism** — `faith_grace` / `law_keeping` / `works_plus_faith` / `insight_enlightenment` / `devotion_bhakti` / `initiation_gnosis` / `collective_not_individual` (the group is saved/damned together)
15. **eschatology_urgency** — `imminent` (the end is near — millenarianism-ready) / `distant` / `none`

### E. Organization & power

16. **authority_structure** — `hereditary_priesthood` (brahmin, kohanim) / `ordained_hierarchy` (bishops) / `scholar_jurist` (rabbis, ulama) / `charismatic` (prophets, shamans — authority can't be institutionalized) / `monastic_community` (sangha) / `magistrate` (civic priesthood) / `lay_none`
17. **celibacy** — `required` / `valued_optional` / `forbidden` / `indifferent`
18. **monasticism** — `central` / `marginal` / `absent`
19. **religion_state_relation** — `fused` / `parallel_law_community` (dhimmi/millet — faith runs its own courts inside someone else's state) / `subordinate` / `civic_function` / `anti_political_sect`
20. **martyrdom_value** — `celebrated` / `accepted` / `avoided`

## Real religions as vectors (the sanity check)

| Religion | membership | truth | ortho- | law | god | afterlife | authority | distinctive extras |
|---|---|---|---|---|---|---|---|---|
| **Judaism** | descent_matri + covenant | chosen-nonexclusive | orthopraxic | total_life_code | strict_mono | deemphasized→resurrection | scholar_jurist | sabbath_strict, purity_strong, portable-law |
| **Catholic** | mission | exclusive+supersessionist(J) | orthodoxic | ethical+ritual | trinitarian | purgatory_economy | ordained_hierarchy | celibate, icons_venerated, martyrdom_celebrated |
| **Orthodox** | mission | exclusive | orthodoxic | ethical+liturgical | trinitarian | heaven_hell | ordained | icons_venerated, monastic_central |
| **Islam** | mission | supersessionist(J,C) | orthopraxic | total_life_code | strict_mono | judgment_resurrection | scholar_jurist | aniconic, parallel-law-capable, hajj |
| **Hinduism** | descent + caste | accommodative | orthopraxic | total_life_code | polytheist+monist | reincarnation_exit | hereditary_priesthood | purity_strong, cyclical_time |
| **Theravada Buddhism** | mission | path-claim | orthopraxic-ish | monastic_rule | non_theistic | reincarnation_exit | monastic | celibate, merit-gift-economy |
| **Zoroastrian** | descent | exclusive | orthopraxic | ritual+purity | dualist | judgment_resurrection | hereditary | fire-purity, the ancestor of Abrahamic eschatology |
| **Manichaean/Gnostic** | initiatory | exclusive | orthodoxic | elect_vs_hearers | dualist | spirit_escape | initiatory | matter-evil, high-tension sect |
| **Roman civic** | territorial | parallel_valid | orthopraxic | ritual_only | polytheist_portfolio | shadow | magistrate | syncretism-as-tech |
| **Village folk** | territorial | parallel_valid | orthopraxic | customary | spirit_world_local | ancestor_continuation | charismatic_shaman | imagistic, absorbs whatever overlays it |

## Why axioms → conflicts (the generative conflict table)

Conflicts are **axiom collisions**, not scripted events. Given two faiths (or a faith vs a rule), the collision predicts the fight:

| Collision | Resulting conflict | Medieval instance |
|---|---|---|
| exclusive × exclusive | holy war, contested shrines | crusades, reconquista |
| supersessionist × its parent-faith | the sharpest hatred — kin-betrayal charge | anti-Judaism, pogroms, ghettos |
| orthodoxic + internal dissenter | heresy trials, inquisition | Cathars, Waldensians, Wycliffe |
| orthopraxic law_total under foreign state | daily-rule violations → martyrdom or parallel-law | dhimmi, millet, dietary conflicts |
| aniconic × icon_venerated | iconoclasm, image wars | Byzantine iconoclasm, Reformation icon-smashing |
| mission + exclusive × accommodative-polytheist | conversion pressure, temple destruction, syncretic pushback | Christianization of Europe — gods got *renamed*, not deleted |
| descent+closed boundary under pressure | boundary hardening, expulsion crises | ghettos, blood libel panics |
| imagistic flashpoint × doctrinal church | prophet crushed, millenarian massacre | Flagellants, Münster, Cohn's millennium |
| initiatory secret × suspicious majority | "they do things in secret" panic | early Christians accused of cannibal rites; witch-sabbath panics |
| covenant+closed × covenant+closed | mutual contempt, coexistence only by distance | rare in practice — usually mediated by a third party |
| afterlife=purgatory_economy + wealth | indulgence economy → reform backlash | literally the Reformation trigger |
| religion_state=fused × rival religion_state=fused | wars ARE crusades by structure | Northern Crusades |

## Violations → conflicts (Gary's "break a rule" example)

Violations differ by orthopraxy/orthodoxy:

- **orthopraxic faith, member violates conduct rule** → ritual pollution: purification rite, communal fast, exclusion from rites until cleansed. Conflicts are *ritual-legal*, not moral outrage — e.g., a kosher-keeper who eats pork doesn't spark doctrine debate; they need cleansing/exclusion.
- **orthodoxic faith, member professes wrong belief** → heresy: correction, recantation, penance; if structural (a rival clergy claims it) → schism.
- **orthopraxic, non-member violates *inside* the community** (foreigner eats on a fast day, builds wrongly, marries wrong) → community defense: expel, convert-or-leave, or violence if the violation is pollution-threatening.
- **interfaith marriage** = membership_mode × exogamy rule → the classic boundary crisis (descent+closed + permitted-if-converts → conversion drama, family rupture).
- **law_scope=total_life_code under a foreign ruler** = the standing conflict generator (the ruler's law keeps violating someone's ritual law — taxes on holy days, military service on the sabbath).

## Generated-faith naming & flavor

- Name from founder/place/virtue-word/rite-word: "the Way of X," "the Sons of Y," "the Hearth Faith," "the Unmaking" — name **after** the axiom vector is drawn so the name can echo it (an eschatological-imminent faith gets an end-times name; a hearth/ancestor faith gets a domestic name).
- A faith also gets **derived texture** from the axioms: icon_stance → visual style; calendar_rhythm → festival cadence; purity_system → food rules; eschatology → sermon topics. The name is surface, the axioms are the substance.

## How a faith *arises* in the sim (the emergence path — thinking only)

- **Substrate:** every region starts with folk/spirit_world (territorial + parallel_valid + orthopraxic + imagistic + ancestor_continuation) — the default human religion, per Bellah/Whitehouse.
- **Imposition/overlay:** a dominant faith arrives via ruler/mission/trade and *layers over* the folk substrate — villagers run both (the medieval reality: official + folk).
- **Charismatic event** (Weber/Whitehouse): under stress, a prophet appears in a high-tension zone → mints a new faith-vector (usually `imminent + exclusive + imagistic`); Stark's sect-cycle decides: it either institutionalizes (→ doctrinal church) or burns out/gets crushed.
- **Schism:** an orthodoxic faith under doctrinal strain splits on a contested axiom (usually authority or a single doctrine) → child faith shares parent axioms except the contested one (Arian→Nicene, Sunni→Shia, Catholic→Reform: the schism *is* the contested axiom).
- **Syncretism:** an accommodative faith absorbs a local cult → the local deity becomes a saint/mask of an existing god → new rite + same vector (the usual survival strategy for local religions).

## Couplings to existing systems

- `sim/religion.rs` — religion entities become axiom-vectors + named faiths; regions/regions hold faith-mix
- `sim/fundamentalism.rs` — imagistic-mode events, prophet-generation, radicalization (Cohn's millennium)
- `sim/magic.rs` + `novel-magical-powers.md` — magic stance axiom decides whether the hedgewitch is a saintly patron or a sorcerer
- `legal-framework-contracts.md` — parallel_law_community is a contract/law jurisdiction type
- `dantes-divine-comedy.md` — the medieval-Catholic instance; `sprint-5-anthropology-religion.md` — Bellah/Whitehouse underpinning
- `societal-evolution-model.md` — legitimation link; `medieval-allegory-power-dynamics.md` — church-as-institution politics
- `emergent-storytelling.md` — axiom collisions are event generators (a schism, a martyrdom, a pogrom, an icon-riot are emergent, not scripted)

## Design rules

- **Axioms over assets** — don't model "Judaism"; model `descent_matrilineal + covenant_chosen + law_total + orthopraxic + scholar_jurist + purity_strong + supersessionist-immune` and let the *name* be generated; similarly "Christianity" = `mission_universal + orthodoxic + trinitarian + resurrection + supersessionist-toward-parent`.
- **Conflicts are structural** — never script a crusade; let exclusive×exclusive collide over a shared shrine and the crusade *emerges*.
- **Few axioms, many religions** — ~20 dimensions × 2-8 values each = an enormous space; coherence rules trim it (e.g. `non_theistic` can't have `faith_grace`; `descent+closed` shouldn't have `mission`; `orthodoxic` wants creed-capable authority).
- **Violations are the game's religion game** — conduct rules produce daily-life friction; belief rules produce heresy drama; boundary rules produce marriage/conversion stories. The rule-book is where religion becomes *gameplay*, not lore.

## Rating

The religion substrate done right — combinatorial, emergent, grounded in the actual comparative-religion literature.
