# Lit Review - Sprint 7: Shaming, Exile, and Popular Justice (Non-Violent Deposition)

**Date:** 2026-10-25
**Domain:** Social History / Legal History / Anthropology of Ritual
**Backlog items:** Non-violent deposition (stocks, thrown tomatoes, thrown soup, exile, tar and feather); Guillotine (violent end); Castles & dungeons (maintaining order); Injustices & labor organizing
**Sources:** Natalie Zemon Davis "The Reasons of Misrule: Youth-Groups and Charivaris in Sixteenth-Century France" (*Past & Present* 50, 1971); Davis–E.P. Thompson letters on rough music (*Past & Present* 235, 2017); Pettitt "Protesting Inversions: Charivary as Folk Pageantry and Folk-Law" (*Medieval English Theatre* 21); "'Folk Justice' and Royal Justice in Early 17th-c. England: a Charivari in the Midlands" (*Midland History* 1983); Early Modern England Encyclopedia "Public Punishments for Crime" (LEMDO); 1911 Britannica "Tarring and Feathering"; Irvin "Tar, Feathers, and the Enemies of American Liberties, 1768-1776"; Meredith "Banishment in Medieval English Political History"; "Excommunication and Outlawry in the Legal World of Medieval Iceland"; *Outlawry, Governance, and Law in Medieval England* (Peter Lang)

## Sources

### 1. Charivari / Rough Music / Skimmington (Davis 1971; Thompson; Pettitt; Midland History)
- Community **mocking rituals**: banging pots and pans outside the offender's house, effigies, riding someone backward on a donkey, mock processions
- Targets: **moral/social infractions** (remarriage of widowers to young brides, wife-beaters, henpecked husbands, adulterers), later also **unpopular officials and landlords**
- Run by **youth groups/"abbeys of misrule"** with recognized license, a **folk-law** parallel to royal justice
- Charivari could escalate into political protest; royal courts sometimes prosecuted participants
- **Relevance:** CRITICAL - an org-run, non-violent sanction any community can vote to use

### 2. Pillory and Stocks (LEMDO; general legal history)
- Official public punishments: **stocks** (feet) and **pillory** (head and hands) in the market square, often for fraud (short weights, bad bread/ale), slander, perjury
- **The crowd decides the severity**: they could throw rotten food, filth, stones (people were sometimes killed) or protect a popular prisoner (some pilloried dissenters were cheered)
- **Relevance:** CRITICAL - the same punishment is lethal or a triumph depending on public opinion, a direct readout of reputation

### 3. Tarring and Feathering (1911 Britannica; Irvin 1768-1776)
- First official mention: **1189, Richard I's ordinance** for robbers in the crusading fleet (shaved, hot pitch, feathers, put ashore)
- Revived as **crowd justice in revolutionary America** against customs officials and loyalists; it was ritualized, public, and meant to humiliate and expel
- Hot tar could cause serious burns: **borderline violent**
- **Relevance:** HIGH - shows popular justice targeting **agents of power** (customs men), not just deviants

### 4. Exile, Outlawry, Abjuration (Meredith; Iceland outlawry study; Peter Lang volume)
- **Abjuring the realm** (England): a suspect in church sanctuary had **40 days** to stand trial or confess and leave the kingdom by an assigned port, on foot, carrying a cross
- **Outlawry:** placed outside the law's protection; anyone could kill or rob you; helping an outlaw was itself a crime
- **Iceland:** **full outlawry** (permanent, killable, can't be helped) vs **lesser outlawry** (3 years' exile with limited protections)
- Political **banishment** used against nobles and rivals
- **Relevance:** CRITICAL - graded exile maps onto org/community membership loss (connects to the travelling-circus "cost of leaving")

### 5. Ritual Inversion (Davis; carnival "world turned upside down")
- Carnival lets the low **mock the high** within a sanctioned time; it vents pressure but can tip into revolt
- **Relevance:** HIGH - a festival that lowers the cost of mocking power, sometimes the seed of real action

### Modern echo: thrown soup
- Contemporary climate protesters throwing soup at glass-protected paintings (2022): a symbolic, **non-damaging** shock for attention. Thrown food is still a protest grammar centuries later

## Mechanics

### Sanction ladder
| Level | Sanction | Who imposes | Violence | Reputation hit |
|---|---|---|---|---|
| 1 | Gossip / satire song | anyone | none | small |
| 2 | Charivari / rough music | youth group, community vote | none | medium |
| 3 | Stocks / pillory | authority | crowd-dependent | medium-large |
| 4 | Tar and feathers | crowd | some | large + usually expelled |
| 5 | Exile / lesser outlawry | court or assembly | none directly | removed from community |
| 6 | Full outlawry | court | anyone may kill | total |
| 7 | Execution (guillotine) | state | lethal | - |

- **Crowd response in stocks** = f(target's local reputation): popular targets gain sympathy (reputation *rises*), hated ones are pelted
- **Legitimacy check:** sanctions imposed against public opinion cost the authority legitimacy (pairs with the "power recognizes power" model)
- **Popular justice upward:** communities can target officials, landlords, tax collectors with charivari or tar, which is how this hooks into labor/organizing scenarios

## Design Proposals

1. **Sanction actions** available to orgs by vote (community, guild, youth group) and to authorities by decree, using the ladder above
2. **Crowd-decided outcomes** for public punishments, driven by the reputation system (`sim/reputation_qsp.rs`)
3. **Charivari as organizing tool:** low-risk, collective, visible, so it also works as a structure test of community strength
4. **Exile system:** graded exile with timers; exiles become wanderers, outlaw bands, or join other factions (the drop-in character system)
5. **Carnival events** that temporarily lower the cost of mocking power, sometimes triggering larger actions
6. **Dungeon vs stocks trade-off** for rulers: hidden punishment avoids crowd reversal but breeds rumor; public punishment is a gamble on popularity

## Rating
**Critical** - a full non-lethal justice ladder run by communities as well as rulers, with outcomes decided by reputation. It is the order-keeping side of the organizing system.
