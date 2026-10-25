# Literature Review: Anthropology of Religion

**Sprint:** 5 — Anthropology (requested targeted research)
**Focus:** the anthropology of religion — what religion *is* functionally, how it evolves with social scale, and how it connects to the societal-development chain (the shepherd→feudalism→revolt model in `sprint-5-anthropology-social-evolution.md`).
**Date:** 2026-10-25

## Why religion needs its own review

Religion is doing enormous work in this game — `sim/religion.rs`, `sim/fundamentalism.rs`, `magic.rs`, legitimacy for rulers, village cohesion, festival calendars, witch hunts, reformations. The anthropology says religion isn't a belief system bolted onto society — it's **one of the main mechanisms societies use to hold themselves together, to justify hierarchy, and to organize revolt.** It belongs to the same causal chain as the development lit: ecology→subsistence→warfare→hierarchy→*legitimation*→revolt — and religion is in that last link as both stabilizer and mobilizer.

## The canonical corpus

### Émile Durkheim — Elementary Forms of Religious Life (1912)
The founding sociological claim: **religion is society worshipping itself.** The sacred/profane split is how a group marks itself off and recharges its solidarity — ritual doesn't describe belief, it *produces* the collective. The god is the group, transfigured.

- **For the game:** rituals/festivals/masses are *cohesion mechanics*, not flavor — a festival literally regenerates the community's solidarity meter. Remove religion and the village doesn't just lose a building, it loses its *binding mechanism*.

### Max Weber — Sociology of Religion (1922); The Protestant Ethic (1905)
Religions differ by their answer to suffering (theodicy) and their *ethic*: how belief translates into conduct. Key moves:
- **Traditional vs charismatic vs rational-legal authority** — the prophet disrupts routine religion; charismatic movements are the *other* kind of religious change alongside drift.
- **Religion as a *drive to meaning*** — people need the world to make sense (why do the good suffer?); theodicy is a felt need, not an abstract doctrine.
- **Religious ethic → economic conduct** — Protestant asceticism → capitalism (the famous thesis, contested but the *shape* — doctrine changes daily behavior — is right).

- **For the game:** a villager's religion isn't a stat, it's a *sense-making system* — a plague hits and religion supplies the interpretation (punishment? test? witch?). Charismatic preachers appear under stress and can redirect the community — a real mechanic (see `community.rs` organizing + `fundamentalism.rs`).

### Clifford Geertz — "Religion as a Cultural System" (1966)
Religion as a set of symbols that establishes **moods and motivations** and models a general order of existence — religion works on feeling/interpretation, not just propositions.

- **For the game:** religion's effect on villagers isn't "belief +20" — it's *mood and meaning* — the same event (a famine, a death) reads differently under different religious stances.

### Robert Bellah — Religion in Human Evolution (2011)
The direct answer to "how religion develops with society": **tribal → archaic → axial**. Each stage tracks social scale.
- **Tribal:** immanent, ritual, no abstract doctrine — religion is *what we do*, not what we affirm.
- **Archaic:** god-kings, cosmic order, state religion — religion *is* the state's ideology.
- **Axial (~800-200 BCE):** the leap — religions that criticize the existing order (the Buddha, the prophets, Confucius, Socrates). Religion can now *judgment* the powers.
- **For the game:** religion stage should track polity scale — village folk-religion (tribal), lord's sanctioned cult (archaic), reformers/prophets (axial move). A reformation is the axial move happening *on screen*.

### Roy Rappaport — Ritual and Religion in the Making of Humanity (1999)
Ritual does computational work: it transmits the sacred, establishes conventions, encodes ecological relationships (his famous pig-feast cycle in New Guinea — ritual **regulates** the actual material system: population, pigs, warfare, land).

- **For the game:** rituals aren't decorative — they *regulate* things (harvest festivals re-sync the village; feasts redistribute surplus; mourning rites process grief-load).

### Harvey Whitehouse — Modes of Religiosity (2004)
The most gameable finding in the whole corpus. Two modes:
- **Imagistic:** rare, intense, episodic rituals (initiations, fire-walkings, the peasant revolt's apocalyptic moment) — tight cohesion in small groups, high arousal, strong bonding.
- **Doctrinal:** frequent, routine, repetitive rituals (weekly mass, daily prayer) — standardized doctrine, large-scale identification, hierarchies of religious specialists.
- The **imagistic flashpoint**: emotionally-arousing events produce intense bonding AND the seeds of radicalization — which is why oppression + catastrophe births religious revolts (Münster, the peasant prophets).

- **For the game:** `religion.rs` should distinguish *routine* religion (mass, feast days — cohesion maintenance) from *flashpoint* religion (a prophetic event, a martyrdom — bonding that can go revolutionary). `fundamentalism.rs` is literally Whitehouse's imagistic-mode mechanic.

### Ara Norenzayan — Big Gods (2013)
Moralizing, watchful gods correlate with **society scale**: small societies run on kin-loyalty and local enforcement; large anonymous societies need gods who watch *everything* and punish *everyone* — supernatural monitoring substitutes for social monitoring. "Big Gods came before big societies" — moralizing religion may be what *enabled* large-scale cooperation.

- **For the game:** small village = spirits/ancestors/saints (local enforcers); kingdom-scale = a moralizing universal deity (enables trust among strangers, markets, hierarchy). Religious form should *follow* polity scale — the church's god gets bigger as the realm does.

### Pascal Boyer — cognitive science of religion (Religion Explained, 2001; and Atran's In Gods We Trust)
Religious concepts spread because they fit *cognitive architecture* — minimally counterintuitive ideas (a ghost: person + can walk through walls) stick better than either mundane or wildly bizarre ones. Religion isn't invented, it's *selected* by what minds find sticky.

- **For the game:** the "believability" of a superstition/prophecy could model virality — ideas that are just-weird-enough spread (see `dynamic-magic-story-text.md` for generated content grounded in this).

### David Sloan Wilson — Darwin's Cathedral (2002)
Group-selection account: religions function to make groups behave as *units* — moral communities with coordinated action. The church is a human-built superorganism.

- **For the game:** the village's religion is literally a coordination technology — it's what lets strangers cooperate at village scale and above.

## Religion in the societal-development chain (the synthesis)

```
tribal/spirit religion ──►  village folk-religion (spirits, saints,
   small groups                local enforcers, ritual cycle)
        │                          │
        │                     polity grows → archaic religion
        │                     (state cult, priestly class,
        │                     cosmic-order ideology)
        │                          │
        │                     hierarchy hardens → religion
        │                     legitimizes (three orders, divine
        │                     right) ── stability doctrine
        │                          │
        │                     extraction breaches moral economy
        │                          │
        │                     charismatic/preacher appears
        │                     (Weber, Whitehouse imagistic)
        │                          │
        │                     reform religion / prophetic movement
        │                     ── the revolt's theology
        │                          │
        │                     outcome: crushed (heresy) /
        │                     negotiated (reform) /
        │                     new orthodoxy (reformation)
        │
   (frontier escape: heterodox holdouts — Scott's Zomia
    religions persist where states can't reach)
```

## Specific medieval mechanics the lit supports

- **Saints & feast days** — local protectors, village identity, the calendar spine of the year (`dantes-divine-comedy.md`, `regional-ruler-personality.md`)
- **Pilgrimage** — travel, information flow, faith demonstration (ties `civ-style-text-network.md`)
- **The parish** — the real organizational unit of village life: priest + congregation + church fabric + poor relief; pre-state social services
- **Heresy/heterodoxy** — religious dissent is how political dissent *talks* (the Peasants' War's theological program, Münster); `fundamentalism.rs` is this mechanic
- **Witch hunts** — scapegoating under collective stress + the hedgewitch's real social function (`novel-magical-powers.md`, `herbology-apothecary-tree.md` — also Federici's account, covered in sprint-4)
- **Monasteries** — literate power centers, land-holders, sanctuaries, scribes (knowledge economy — `printmaking-tech-tree.md`)

## Rating

Religion in this game isn't a theme — it's a mechanic of cohesion, legitimation, and revolt. Whitehouse and Bellah are nearly spec documents; Weber and Scott do the revolt theology; Rappaport the ritual-as-regulation.
