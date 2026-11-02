# Lit Review - Sprint 7: Fermentation Trades (Cheese, Ale/Beer, Wine, Spirits)

**Date:** 2026-10-25
**Domain:** Food History / Food Science / Medieval Economics
**Backlog items:** Cheese, Alcohol (and the "valuable commodity → defense" note), feeds Perfume
**Sources:** Lenoir et al. cheese classification (Microbiology Spectrum), Folia Magazine on medieval cheesemaking, Bennett *Ale, Beer and Brewsters in England 1300-1600*, J. Medieval History on Dutch beer brewers in England, *Water of Life* (history of distilling), Crescentis on medieval wine storage, Wikipedia "Aqua vitae"

## Sources

### 1. Cheese Classification (Lenoir et al., via *Microbiology Spectrum* 2014)
- **Three key steps make all the difference:** coagulation → draining → ripening
- **Coagulation type:** lactic/acid (small, moist, fresh, >70% water, little calcium) vs rennet/enzymatic (large, firm, low moisture, ages months to years)
- **Draining intensity:** cutting, stirring, pressing, "cooking" the curd (<40°C uncooked, <50°C semicooked, >50°C cooked) pushes water out
- **Format → aging:** big low-moisture wheels last longest; small acid cheeses must be eaten fresh
- **Relevance:** CRITICAL - a tiny 3-choice process makes a combinatorial cheese tree (fits the [flexible recipe system](../software-design/flexible-recipe-system.md))

### 2. Medieval Cheesemaking (Folia Magazine; Montanari (Columbia University Press), ch. "From Milk to Cheeses")
- Cheese = **preserved milk**: keeps a perishable resource through winter and turns it into a trade good
- *Caseus recens* (fresh) vs *caseus vetus* (old/hard) - hard cheese travels with merchants over long distances
- Many famous cheeses date from this era (Parmesan, Gruyère, Cheddar, Camembert); **sheep milk dominated until ~14th-15th c.**, not cow
- **Roquefort:** Lacaune sheep milk, aged in specific local caves whose soil carries the mold - **terroir is literally the cave**
- Monasteries were major producers (stable labor, cellars, long time horizons)
- **Relevance:** HIGH - regional varieties come from local animal + local cave/climate; ties to the Caves backlog item

### 3. Ale, Beer and Brewsters (Bennett 1996) + Dutch Beer in England (J. Medieval History 2019)
- **Ale (unhopped) was women's domestic work** - "alewives/brewsters" brewed at home, sold the surplus
- Ale spoiled within days → brewing was frequent, local, small-scale
- **Hopped beer** (brought by Low Countries immigrants) keeps far longer and ships → larger capital-intensive breweries → **men and guilds took over**, women pushed out
- Authorities regulated quality/strength and price (assize of ale; ale-tasters)
- **Relevance:** CRITICAL - one tech (hops) changes who does the job, the scale, and the gender/power structure. A great "small change, big impact" example

### 4. Wine Storage and Trade (Crescentis, *Le vin et la ville*, 12th-15th c.)
- Most wine **rarely kept beyond a year** before turning to vinegar (sweet wines excepted)
- Needed ventilated, dim, clean cellars; casks were the unit of storage and trade
- Wine was taxed and traded through cities (Bordeaux), a big source of urban revenue
- **Relevance:** HIGH - wine decays on a clock unless stored well, so cellar quality is a stat

### 5. Distillation / Aqua Vitae (*Water of Life*; Wikipedia "Aqua vitae")
- Medieval distilled spirit = **aqua vitae** ("water of life"), usually distilled from wine; also "spirit of wine"
- First valued as **medicine/alchemy** (John of Rupescissa: ethanol as the imperishable "quintessence"), only later as a drink
- The name runs through local spirits: whiskey (*uisce beatha*), eau de vie, acquavite, akvavit, okowita
- **Relevance:** CRITICAL - spirits are the **solvent for perfume** and tinctures (herbology), and they don't spoil

## Mechanics

### Cheese
- **Recipe = (milk animal) × (coagulation: acid | rennet) × (draining: soft | pressed | cooked) × (aging place)**
- **Aging place** adds region-unique traits: cave mold → blue cheese; a dry loft → hard rind
- Fresh cheese spoils fast; aged cheese gains value over time (a **wealth store that matures**)
- Rennet needs calf/lamb stomachs → links cheese to slaughter, so it competes with the herd

### Alcohol
| Product | Input | Spoils | Scale | Who (historically) |
|---|---|---|---|---|
| Ale | grain | days | household | women / alewives |
| Beer (hopped) | grain + hops | months | brewery | guild men |
| Wine | grapes | ~1 year (cellar-dependent) | vineyard + cellar | vintners, monasteries |
| Aqua vitae | wine/beer + still | never | distillery | apothecaries, monks → distillers |

- **Value density rises down the table** → higher tiers are worth stealing → **guards, locked cellars, taxes, smuggling**
- Ale is a social good (alehouse = gathering place, gossip hub → plugs into `sim/gossip.rs`)
- Drunkenness: short mood boost, work penalty, raised conflict chance (contrast with tea's sobriety boost from the [Sprint 4 tea review](sprint-4-tea-herbology.md))

## Design Proposals

1. **Fermentation recipe family** in the [flexible recipe system](../software-design/flexible-recipe-system.md): shared verbs (curdle, press, ferment, age, distill) and a maturity clock per item
2. **Cellar/cave as a building with traits** (temperature, humidity, mold): the same recipe aged in different cellars yields different named products → regional identity
3. **Hops event:** introducing hopped beer moves brewing from households to guild breweries. Shows labor displacement and who loses a livelihood (good fit for the labor organizing scenarios)
4. **Guarded stock:** stockpiles above a value threshold attract theft events and need a guard or lock; this gives military/watch jobs an economic reason to exist
5. **Assize/regulation:** a ruler can set price/strength rules and appoint ale-tasters, a light-touch governance lever
6. **Vocations:** cheesemaker, alewife/brewer, vintner, cooper (casks!), distiller, each with an apprenticeship ladder

## Rating
**Critical** - fermentation gives the game preservation (food survives winter), wealth that matures, region-unique goods via caves, and a clean historical example of technology reshaping labor (ale → beer). Spirits unlock perfume and tinctures.

## In game (as of the alcohol-trades PR)

- **Cheese:** cellar building, milk → wheels, aging clock, village-named ("stone-cellar"), noble pays triple for aged wheels. *Not yet:* the coagulation/draining recipe tree, rennet→slaughter link.
- **Alcohol table:** `Alewife` (wheat → ale, sours to vinegar inside the week), `Brewer` (wheat + hops → beer, keeps), `Vintner` (grapes → wine, turns on a shelf, keeps in casks and the cellar), `Distiller` (wine → spirits, never turns, value-dense), `Cooper` (wood → casks that slow the souring). Exactly one drink trade is guaranteed per village, rolled off the seed.
- **Value density → defense & revenue:** the `CellarThief` moment targets the fattest drink stock (a living watch catches him); the tax round takes an ale excise on stocked drink; a fourth seasonal writ is the castle's share of the barrel; the `Assize` moment fines thin brews and praises masters.
- **Social:** `Want::Drink`, drinks consumed at the counter, a `Drunk` day costs the next day's work, `AlehouseBrawl` picks real drinkers, feasts tap the barrel, the wine merchant sells cups, the alehouse keeper is a gossip contact.
- **Chains:** spirits are an input to perfume (Hungary water) and to the feverfew tincture (a measure the still owes the wise woman each fortnight); hops/grapes/wine/spirits ride the trader's road.
