# Agriculture & Food Systems Literature Review

**Survey Date:** 2026-10-25
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Agriculture, Food Systems, Medieval Economics

## Summary

Medieval agriculture was a complex, seasonal system built around crop rotation, food preservation, and vulnerability to weather. The three-field system revolutionized yields (+33%), while food security swung between feast and famine based on harvest success. This research provides rich mechanics for agricultural simulation, resource management, and seasonal gameplay.

## Key Sources

### Historical Yields Data - Winchester Manors (1283-1349)
**Rating:** A
**Type:** Historical Data / Academic Research
**Relevance:** High - Provides quantitative data for game balancing

**Key Findings:**
- Wheat: 515 kg/ha gross yield, 385 kg/ha net yield
- Barley: 755 kg/ha gross yield, 540 kg/ha net yield
- Seed ratios: Wheat 4.34:1, Barley 4.01:1 (seeds out:seeds in)
- Yields per acre: Wheat 7-17 bushels, Barley 700-720 liters

**Source Quality:** Based on actual manorial records from Winchester estates
**Game Application:** Provides realistic yield numbers for agricultural simulation

---

### Three-Field System - Oxford Academic / InTech
**Rating:** A+
**Type:** Peer-reviewed academic research
**Relevance:** High - Core agricultural mechanic

**Key Findings:**
- **System Structure:**
  - Field 1: Winter cereals (wheat, rye)
  - Field 2: Spring crops (oats, barley, peas, lentils, broad beans)
  - Field 3: Fallow
  - Rotation annually

- **Benefits:**
  - +33% yield increase vs two-field system
  - Reduced plowing effort
  - Nitrogen fixing via legumes
  - Risk diversification

- **Historical Context:**
  - Introduced during Carolingian period
  - Persisted throughout medieval period
  - Required collective village decision-making

**Source Quality:** Multiple peer-reviewed sources confirm
**Game Application:** Crop rotation mechanic with yield bonuses

---

### Medieval Food Preservation Methods
**Rating:** B+
**Type:** Historical analysis
**Relevance:** High - Food storage and spoilage mechanics

**Key Techniques:**
- **Salting:** Primary preservation method
- **Smoking:** For meats
- **Drying:** For grains, fruits, vegetables
- **Pickling:** In brine or vinegar
- **Fermentation:** For long-term storage
- **Cold storage:** Underground cellars (natural refrigeration)

**Storage Systems:**
- Vast underground vaults
- Enormous barrels for wine
- Sacks of grain
- Wheels of aging cheese
- Vats of salted meat

**Temperature Effects:**
- Food lasts much longer in winter
- Underground = lower temperature = better preservation
- Cellarer managed storage logistics

**Game Application:** Temperature-based spoilage, preservation tech tree

---

### Seasonal Food Security - Multiple Sources
**Rating:** B
**Type:** Historical synthesis
**Relevance:** High - Seasonal gameplay loop

**Key Patterns:**
- **Spring (Hunger Gap):** Stores depleting, new crops not ready
- **Summer:** Fresh produce available, harvest preparation
- **Autumn (Harvest):** Peak abundance, storage for winter
- **Winter:** Reliance on preserved foods, vulnerability

**Vulnerability Factors:**
- **Weather dependence:** Bad weather = crop failure
- **No winter growing:** Complete reliance on stores
- **Famine risk:** Great Famine 1315-1317 (prolonged adverse weather)
- **80-90% agricultural workforce:** Society-wide impact

**Social Dynamics:**
- **Nobles:** Elaborate feasts during holidays
- **Peasants:** Communal gatherings, modest fare
- **Religious calendar:** Lent fasting, Lammas harvest celebrations

**Game Application:** Seasonal resource cycle, famine/feast events

---

### APSIM Crop Simulation Model
**Rating:** A+
**Type:** Peer-reviewed agricultural research
**Relevance:** Medium - Scientific basis for yield modeling

**Key Concepts:**
- Models crop growth based on weather, soil, management
- Simulates photosynthesis, water use, nutrient uptake
- Predicts maturity dates, yields, fertilizer effectiveness
- Handles stress factors (cold, heat, drought)

**Model Features:**
- Mathematical equations for biological processes
- Climate-soil-management interactions
- Validated against real-world data

**Game Application:** Simplified crop growth simulation with weather/soil factors

---

### Educational Farming Games - Sowterra
**Rating:** C
**Type:** Game design example
**Relevance:** Medium - Shows how agriculture translates to gameplay

**Features:**
- 3D farming simulation
- Traditional agricultural practices
- Objective-based gameplay (harvest X crops)
- Economic rewards (Indonesian Rupiah)

**Lessons:**
- Agriculture works well as objective-based gameplay
- Traditional practices can be educational
- Economic integration important

**Game Application:** Examples of agricultural objectives and rewards

---

## Game Mechanic Proposals

### Mechanic 1: Three-Field Crop Rotation
**Academic Basis:** Three-field system (Carolingian period), 33% yield increase

**Translation:**
- **Variables:**
  - Field state (winter crop / spring crop / fallow)
  - Soil fertility (0-100)
  - Crop type planted
  - Years in rotation

- **Mechanics:**
  - Player assigns crops to fields
  - Following rotation = +33% yield bonus
  - Breaking rotation = fertility penalty
  - Legumes restore nitrogen (+10 fertility)
  - Fallow restores fertility (+20)

- **Formulas:**
  ```
  base_yield = crop_type.base_yield
  fertility_modifier = field.fertility / 100.0
  rotation_bonus = following_rotation ? 1.33 : 1.0
  final_yield = base_yield * fertility_modifier * rotation_bonus * weather_modifier
  ```

**Educational Value:** Teaches sustainable farming and why rotation matters

**Fun Factor:** Strategic planning, optimization puzzle, long-term thinking

**Implementation:** `game2-core/src/sim/agriculture.rs`
```rust
struct Field {
    fertility: u8,  // 0-100
    current_crop: Option<CropType>,
    rotation_year: u8,  // 0-2
}

enum CropType {
    WinterWheat,
    SpringBarley,
    Peas,
    Fallow,
}
```

---

### Mechanic 2: Seasonal Food Security Cycle
**Academic Basis:** Medieval seasonal patterns, Great Famine 1315-1317

**Translation:**
- **Variables:**
  - Season (Spring/Summer/Autumn/Winter)
  - Food stores (quantity by type)
  - Population food needs
  - Storage capacity
  - Preservation tech level

- **Mechanics:**
  - **Autumn:** Harvest bulk of food
  - **Winter:** No production, consume stores
  - **Spring (Hunger Gap):** Stores low, new crops not ready
  - **Summer:** Fresh food + harvest prep

- **Events:**
  - **Good harvest:** Surplus, celebrations
  - **Poor harvest:** Shortage, famine risk
  - **Spoilage:** Food decays without preservation
  - **Famine:** Multi-year crop failure

**Formulas:**
```
food_needed_per_season = population * base_consumption * season_multiplier
hunger_gap_severity = (stores_remaining / food_needed) - 1.0
famine_risk = hunger_gap_severity < -0.5
```

**Educational Value:** Shows agricultural vulnerability and why storage matters

**Fun Factor:** Tension (will we survive winter?), planning challenge

**Implementation:** Season-based resource management with failure states

---

### Mechanic 3: Food Preservation Tech Tree
**Academic Basis:** Medieval preservation methods (salting, smoking, drying, cellaring)

**Translation:**
- **Tech Tree:**
  1. Basic Storage (granaries) - 10% spoilage/month
  2. Salting - Reduces meat spoilage to 2%/month
  3. Smoking - Reduces meat spoilage to 1%/month
  4. Cellaring - Temperature control, 0.5%/month for all
  5. Fermentation - Wine/beer, long-term storage

- **Variables:**
  - Food type (grain/meat/produce)
  - Preservation method applied
  - Temperature (season-based)
  - Storage duration

- **Mechanics:**
  - Each tech unlocks new preservation option
  - Better preservation = less waste
  - Underground storage = temp bonus
  - Winter = natural refrigeration

**Formulas:**
```
base_spoilage_rate = food_type.base_spoilage
preservation_modifier = preservation_tech.spoilage_reduction
temperature_modifier = season.temp_effect
monthly_spoilage = base_spoilage * preservation_modifier * temperature_modifier
```

**Educational Value:** Shows how preservation enables civilization

**Fun Factor:** Tech progression, resource optimization

**Implementation:** Technology unlock system affecting spoilage rates

---

### Mechanic 4: Crop Yield Variability
**Academic Basis:** Historical yield data (4-10 bushels/acre wheat), weather dependence

**Translation:**
- **Variables:**
  - Base yield (by crop type)
  - Weather events (drought, rain, frost)
  - Soil fertility
  - Pest/disease events

- **Mechanics:**
  - Each harvest rolls for yield
  - Range: 50% to 150% of base
  - Weather events modify rolls
  - Consecutive bad years = famine

**Yield Ranges (per acre):**
- Wheat: 6-10 bushels (poor to great)
- Barley: Higher volume, more reliable
- Legumes: Lower yield, fertility restoration

**Formulas:**
```
yield_roll = random(0.5, 1.5)
weather_modifier = calculate_weather_impact()
pest_modifier = has_pest_outbreak ? 0.7 : 1.0
final_yield = base_yield * yield_roll * weather_modifier * pest_modifier * fertility_modifier
```

**Educational Value:** Shows agricultural unpredictability

**Fun Factor:** Risk management, adaptability

**Implementation:** Randomized yields with modifier system

---

### Mechanic 5: Village Collective Farming
**Academic Basis:** Open-field system, collective rotation decisions

**Translation:**
- **Social Mechanics:**
  - Village decides rotation collectively
  - Individual farmers assigned strips
  - Shared grazing on fallow
  - Communal harvest festivals

- **Variables:**
  - Village consensus (0-100)
  - Individual farmer satisfaction
  - Shared resources (oxen, tools)

- **Mechanics:**
  - Rotation requires majority vote
  - Breaking rotation hurts collective
  - Shared tools increase efficiency
  - Festivals boost morale post-harvest

**Educational Value:** Shows medieval agricultural cooperation

**Fun Factor:** Social dynamics, consensus building

**Implementation:** Voting system for village-wide agricultural policy

---

## Domain Translation Patterns

### Pattern: Historical Yields → Game Balance
**Approach:**
1. Use historical data as baseline (Winchester manors)
2. Scale to game time (1 year = X turns)
3. Add variability range (±50%)
4. Tune for fun (not pure realism)

**Example:**
- Historical: 7-17 bushels wheat/acre
- Game: 10 units ± 5 (roll 5-15)

---

### Pattern: Preservation Methods → Tech Tree
**Approach:**
1. Identify historical techniques
2. Order by complexity
3. Map to spoilage reduction
4. Balance against cost

**Example:**
```
No tech: 20% spoilage/month
Salting: 5% spoilage/month (cost: salt)
Cellaring: 2% spoilage/month (cost: construction)
```

---

### Pattern: Seasonal Cycle → Gameplay Loop
**Approach:**
1. Map real seasons to game seasons
2. Identify key pressure points (hunger gap)
3. Create tension and release rhythm
4. Add events for variety

**Loop:**
- Spring: Planting, anxiety (low food)
- Summer: Growth, preparation
- Autumn: Harvest, celebration/despair
- Winter: Consumption, survival

---

## Recommended Implementation Priority

### High Priority (Core Gameplay)
1. **Three-Field Rotation** - Core mechanic, strategic depth
2. **Seasonal Cycle** - Fundamental game loop
3. **Basic Yields** - Must have for agriculture to work

### Medium Priority (Depth)
4. **Food Preservation** - Adds strategic choices
5. **Yield Variability** - Adds risk/excitement

### Low Priority (Polish)
6. **Collective Farming** - Social layer, can come later

---

## Integration Points

**Existing Systems:**
- `economics.rs` - Market prices for crops
- `villager.rs` - Hunger needs
- `community.rs` - Collective decisions
- `emergence.rs` - Weather/climate effects

**New Module Needed:**
- `agriculture.rs` - Crop management, yields, rotation

**Data Structures:**
```rust
struct Farm {
    fields: Vec<Field>,
    storage: FoodStorage,
    preservation_tech: Vec<PreservationMethod>,
}

struct Field {
    fertility: u8,
    crop: Option<CropType>,
    rotation_year: u8,
}

struct FoodStorage {
    grain: u32,
    meat: u32,
    produce: u32,
    preservation_applied: PreservationMethod,
}
```

---

## Research Gaps Identified

### Need More Detail On:
1. **Specific crop caloric values** - For accurate food needs
2. **Labor requirements per acre** - For workforce simulation
3. **Oxen/tools economics** - Equipment costs and sharing
4. **Regional variations** - Different climates, different crops

### Future Surveys:
1. **Animal Husbandry** - Livestock integration
2. **Brewing/Fermenting** - Ale as calories and social good
3. **Gardens vs Fields** - Household vs village agriculture
4. **Disaster Recovery** - Rebuilding after famine

---

## Confidence Assessment

**Overall Confidence:** High

**Strong Evidence For:**
- Three-field system mechanics (multiple sources)
- Historical yield data (Winchester manors)
- Preservation methods (well-documented)
- Seasonal patterns (consistent across sources)

**Weaker Evidence For:**
- Exact spoilage rates (extrapolated from game precedents)
- Social dynamics of rotation (inferred from open-field system)

**Assumptions Made:**
- Scaling from real time to game time
- Simplifying continuous processes to discrete turns
- Balancing realism vs fun (using 33% boost not exact ratios)

---

## Next Steps

1. **Prototype `agriculture.rs` module** with Field and Crop structures
2. **Implement three-field rotation** with yield bonuses
3. **Add seasonal food cycle** to existing game loop
4. **Test yield variability** for fun vs frustration balance
5. **Create preservation tech tree** in tech system
6. **Playtest hunger gap** - is it too punishing?

---

## Bibliography

- Oxford Academic: "Crop Rotation and Seasonal Sowing"
- Gresham College: "The Medieval Agricultural Revolution" (Prof. Helena Hamerow)
- InTech: "Concepts in Crop Rotations" (H. Arnold Bruns)
- Winchester Manors historical data (1283-1349)
- APSIM crop simulation model documentation
- Historical sources on medieval food preservation
- Medieval famine and feast historical analysis

---

**Tags:** agriculture, medieval, crop-rotation, food-security, seasonal-gameplay, three-field-system, preservation, yields
