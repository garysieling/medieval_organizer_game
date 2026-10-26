# Urban Planning Literature Review

**Survey Date:** 2026-10-25
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Urban Planning, Medieval History, Urban Scaling Theory

## Summary

Medieval cities evolved from organic growth patterns shaped by trade, defense, and natural geography. Unlike Roman grid-planned cities, medieval settlements grew spontaneously around markets, cathedrals, and fortifications, creating irregular street layouts with strategic public spaces. Research reveals that medieval cities followed scaling laws similar to modern cities (larger = denser), with sophisticated infrastructure in water supply and sanitation. This research provides rich mechanics for village-to-city transitions, infrastructure development, and urban simulation.

## Key Sources

### Medieval Urban Layout & Design
**Rating:** A
**Type:** Historical Analysis / Academic Research
**Relevance:** High - Core urban planning mechanics

**Key Findings:**

**Organic vs Planned Development:**
- Medieval cities developed spontaneously with crooked streets, strong fortifications, high cathedrals, active market squares, and compact residential quarters
- Late medieval cities favored geometric plans; cities built on Roman ruins gravitated toward gridiron patterns
- Organic, need-to-need planning was most common, following natural topography and evolving from paths connecting castle, market, and main roads

**Growth Points:**
- Cathedral, church, cloister, monastery (Bishop's seat) served as important original growth points
- The marketplace ultimately proved most important, proving victorious over castle and cathedral as primary urban driver
- Trade revival from 11th century onward drove urban expansion

**Defense:**
- Security was the dominant factor shaping medieval cities
- Massive defensive walls, watchtowers, moats, and fortified gateways were standard
- Walls constrained growth, creating density pressures

**Source Quality:** Multiple historical sources confirm patterns
**Game Application:** Organic growth mechanics with infrastructure-driven expansion

---

### Population-Area Scaling Laws (PLOS ONE Study)
**Rating:** A+
**Type:** Peer-reviewed quantitative research
**Relevance:** High - Provides mathematical basis for urban scaling

**Key Findings:**

**Dataset:**
- 173 medieval European settlements (Belgium, France, England, Switzerland, Germany, Italy)
- Data from circa AD 1300
- Analyzed relationship between population size and settled land area

**Scaling Exponents:**
- Scaling coefficient α falls within 2/3 ≤ α ≤ 5/6 range (predicted by social reactor model)
- Point estimates: 0.71 to 0.79 across regional groups
- Interpretation: **Larger cities are denser**

**Mathematical Relationship:**
```
Area = k * Population^α
where α ≈ 0.75 (average)
```

**Urban Densification:**
- Medieval cities exhibit economies of scale with respect to spatial agglomeration
- Larger population cities occupied proportionally less settled area per capita
- Pattern identical to modern cities despite 700-year gap

**Institutional Effects:**
- Hierarchical institutions (guilds, churches) had no statistically significant effect on agglomeration patterns

**Source Quality:** Rigorous quantitative analysis, peer-reviewed
**Game Application:** Formulaic city growth with density scaling

---

### Medieval Market Squares & Civic Architecture
**Rating:** A
**Type:** Historical Data / Architectural Analysis
**Relevance:** High - Public space design

**Key Findings:**

**Market Square Dimensions:**
- Typical range: 100-200 meters diameter
- Kraków's Rynek Główny (largest in Europe): ~660 feet per side (~200m)
- Wrocław Market Square: 600 × 480 feet (~183m × 146m)
- Poznań Old Market: ~140m per side
- Tarnów Market: 89.4m × 74.5m

**Scale Formula:**
- Market square size roughly proportional to town population
- Larger cities had proportionally larger central squares

**Civic Architecture:**
- Town hall: Almost always present, housed civic administration (typical 30m × 10m)
- Guild hall: Economic power center for trade guilds
- Cloth halls: Massive structures (Ypres: 125m long) facing market square
- Ground floor: Covered trading area
- Upper floors: Administrative functions, guild meetings

**Social Hierarchy:**
- Square surrounded by guild halls, shops, inns reflecting economic hierarchy
- Mixed-use buildings: Commercial ground floor, residential upper floors

**Source Quality:** Multiple European examples with measured dimensions
**Game Application:** Public space mechanics with scaling formulas

---

### Medieval Infrastructure: Water & Sanitation
**Rating:** A
**Type:** Historical Research / Archaeological Evidence
**Relevance:** High - Infrastructure mechanics

**Key Findings:**

**Water Supply Systems:**

**Basic Systems:**
- Rivers and shallow wells: Primary supply for most towns
- Households collected water directly from riverbanks or communal wells
- Uses: Drinking, cooking, washing, brewing

**Sophisticated Systems:**
- North German towns: Private wells + public infrastructure (water pipes)
- Canals, water conduits, wells, fountains provided fresh water
- London: Water-wheels under London Bridge powered pumps for municipal/commercial use
- Augsburg, Freiburg, Salzburg: Sophisticated networks of canals, water towers, fountains supplying households, trade, and industry

**Sanitation:**

**Advanced Example - Córdoba:**
- Sanitation infrastructure "not surpassed until 20th century in Europe"
- Ashlar masonry sewers sealed with lime mortar
- Subterranean infrastructure channeling wastewater to Guadalquivir River

**Typical Systems:**
- London: Cesspools filled with deodorizing charcoal, cleaned by privy-cleaners
- Wastes deposited in landfills outside city walls or riverbanks
- No distinct separation between water and wastewater = serious contamination

**Streets and Roads:**
- Wider streets: Two gutters separating road from footpaths
- Narrow streets: Single gutter down center
- Late medieval: Heavy investment in paving and waste disposal
- Town councils imposed sanitary obligations (clean street in front of your property)

**Source Quality:** Archaeological evidence, historical records
**Game Application:** Tech tree for infrastructure with health/efficiency impacts

---

### SimCity & Urban Simulation Models
**Rating:** B+
**Type:** Game Design Analysis / Academic Research
**Relevance:** Medium - Shows how urban systems translate to gameplay

**Key Findings:**

**Core Mechanics:**
- High-level town planning: Players outline zones (residential, commercial, industrial)
- Build hard infrastructure: Roads, trains, power stations
- Build services: Schools, hospitals, fire stations, police
- By SimCity 4: Sophisticated simulation of real-world scenarios (low-density suburbs vs high-density Manhattan)

**Agent-Based Systems:**
- Since version 4: Agent-based system where Sims interact and react autonomously
- Captures functions of city government
- Programming based on planning policy and economic development theories

**Zoning:**
- Residential, commercial, industrial zones
- Player outlines zones, AI fills in buildings
- Proximity effects (residential near industrial = bad)

**Educational Impact:**
- SimCity created a generation of urban planners
- First video game with bibliography (Kevin Lynch, Le Corbusier)
- Used in planning education and participatory workshops

**Limitations:**
- Optimized for entertainment, not environmental engineering accuracy
- Simplified models vs real urban complexity

**Source Quality:** Academic analysis of game design, multiple sources
**Game Application:** Zoning system, infrastructure requirements, agent-based population

---

### Medieval Town Growth Timeline
**Rating:** B
**Type:** Historical Synthesis
**Relevance:** Medium - Growth pacing mechanics

**Key Findings:**

**Early Middle Ages:**
- Most people lived in countryside
- Towns small and limited

**High Middle Ages (11th century onward):**
- Towns grew significantly in size and importance
- Trade revival = major growth factor
- People moved to towns seeking work and opportunity
- Towns became larger and more complex

**Growth Catalysts:**
- Trade routes and markets
- Cathedral construction
- Royal/noble patronage
- Guild formation
- Charter granting (legal rights to self-governance)

**Transformation Sequence:**
1. Hamlet (no market)
2. Village (local market)
3. Town (regional market, basic fortifications, charter)
4. City (major market hub, full fortifications, cathedral, guilds)

**Source Quality:** Historical consensus across sources
**Game Application:** Milestone-based progression system

---

## Game Mechanic Proposals

### Mechanic 1: Urban Scaling with Density Laws
**Academic Basis:** PLOS ONE population-area scaling study (α ≈ 0.75)

**Translation:**
- **Variables:**
  - Population size (P)
  - Settled area (A)
  - Scaling exponent (α = 0.75)
  - Base constant (k)

- **Mechanics:**
  - As population grows, city area grows sublinearly
  - Larger cities become progressively denser
  - Density creates efficiency gains (shared infrastructure)
  - Density creates problems (sanitation, disease, fire risk)

- **Formulas:**
  ```
  settled_area = k * population^0.75

  density = population / settled_area

  # Example:
  # Village: 100 pop → area = k * 100^0.75 ≈ 31.6k
  # Town: 1000 pop → area = k * 1000^0.75 ≈ 177.8k (5.6x denser)
  # City: 10000 pop → area = k * 10000^0.75 ≈ 1000k (31.6x denser than village)

  infrastructure_efficiency = 1.0 + (density / base_density) * 0.1
  disease_risk = base_risk * (density / base_density)^0.5
  ```

**Educational Value:** Teaches urban density economics and tradeoffs

**Fun Factor:** Strategic tension between growth and density problems

**Implementation:** `game2-core/src/sim/urban.rs`
```rust
struct Settlement {
    population: u32,
    settled_area: f32,  // km²
    density: f32,       // pop/km²
    infrastructure_level: u8,
}

fn calculate_required_area(population: u32, k: f32) -> f32 {
    k * (population as f32).powf(0.75)
}
```

---

### Mechanic 2: Zoning & Mixed-Use Development
**Academic Basis:** Medieval mixed-use patterns, SimCity zoning mechanics

**Translation:**
- **Zone Types:**
  - Residential (housing)
  - Commercial (market, shops, inns)
  - Industrial (workshops, smithies, tanneries)
  - Civic (town hall, guild halls, churches)
  - Agricultural (within walls for siege preparation)

- **Variables:**
  - Zone type assignment per tile
  - Proximity effects (positive/negative)
  - Land value
  - Building density

- **Mechanics:**
  - Player designates zone types
  - Buildings spawn organically based on needs
  - Proximity rules:
    - Residential + Commercial = good (convenience)
    - Residential + Industrial = bad (noise, smell, pollution)
    - Commercial + Commercial = competition (diminishing returns)
    - Civic + Market = excellent (governance + economy)
  - Medieval pattern: Natural mixed-use (commercial ground floor, residential above)

- **Formulas:**
  ```
  land_value = base_value * proximity_modifier * infrastructure_modifier

  proximity_modifier = sum(
    neighbor_zone.compatibility[this_zone.type] * distance_decay
  )

  # Compatibility matrix:
  # Residential-Commercial: +0.2
  # Residential-Industrial: -0.5
  # Commercial-Civic: +0.3
  # Industrial-Residential: -0.5

  building_spawn_chance = zone_demand * land_value * available_workers
  ```

**Educational Value:** Shows trade-offs in urban land use

**Fun Factor:** City planning puzzle, optimization challenge

**Implementation:** Grid-based zone assignment with spawn rules

---

### Mechanic 3: Infrastructure Tech Tree
**Academic Basis:** Medieval water/sanitation systems progression

**Translation:**
- **Tech Tree:**
  1. **Wells & Cisterns** - Basic water (communal wells)
  2. **Paved Streets** - Reduces disease, improves commerce speed
  3. **Gutters & Drainage** - Reduces flooding, disease
  4. **Water Conduits** - Piped water to fountains (efficiency boost)
  5. **Cesspools** - Basic sanitation (prevents worst contamination)
  6. **Underground Sewers** - Advanced sanitation (major health boost)
  7. **Water Towers** - Pressurized distribution (high-end cities)
  8. **Public Fountains** - Water access points (quality of life)

- **Variables:**
  - Infrastructure coverage % (what portion of city has it)
  - Maintenance cost per turn
  - Health impact
  - Commerce efficiency impact

- **Mechanics:**
  - Each tech unlocks new infrastructure option
  - Infrastructure must be built per district (costs resources)
  - Coverage % affects entire city's health/efficiency
  - Higher density = higher infrastructure requirements
  - Neglected infrastructure degrades over time

- **Formulas:**
  ```
  health_modifier = 1.0 + (
    water_coverage * 0.3 +
    sanitation_coverage * 0.4 +
    street_paving_coverage * 0.2
  )

  disease_outbreak_chance = base_chance / health_modifier

  commerce_speed = base_speed * (1.0 + paved_street_coverage * 0.5)

  infrastructure_cost_per_turn = (
    population * infrastructure_level * cost_per_capita
  )

  # Advanced system (Córdoba-level) massively expensive but huge health gains
  advanced_sanitation_cost = population * 5.0
  advanced_sanitation_health = 2.0  # doubles health modifier
  ```

**Educational Value:** Shows importance of public health infrastructure

**Fun Factor:** Tech progression, investment decisions, crisis prevention

**Implementation:** Tech unlock system with per-district building

---

### Mechanic 4: Market Square & Public Spaces
**Academic Basis:** Historical market square dimensions and civic architecture

**Translation:**
- **Variables:**
  - Market square size (scales with population)
  - Civic buildings (town hall, guild halls, church)
  - Market activity level
  - Social gathering frequency

- **Mechanics:**
  - Every town/city needs a central market square
  - Square size requirement scales with population
  - Too small = commerce penalty, overcrowding
  - Too large = wasted space, low activity
  - Civic buildings around square boost effects
  - Weekly market events (trading, socializing, information spread)

- **Square Size Formula:**
  ```
  # Based on historical data:
  # 100-200m diameter for typical towns
  # Scales with population

  required_square_size = 50 + (population / 100) * 10  # meters per side

  # Examples:
  # Village (500 pop): 50 + 50 = 100m square
  # Town (2000 pop): 50 + 200 = 250m square (exceeds typical, might split into multiple)
  # City (10000 pop): 50 + 1000 = 1050m (unrealistic, need multiple districts)

  # Revised formula with district splitting:
  district_population_cap = 5000
  required_square_size_per_district = 50 + (district_pop / 100) * 10

  commerce_efficiency = actual_size / required_size
  # <1.0 = overcrowded penalty
  # 1.0-1.5 = optimal
  # >1.5 = too sparse, activity diluted
  ```

- **Civic Buildings:**
  - Town Hall: +20% governance efficiency, required for town status
  - Guild Hall: +15% commerce, stabilizes prices
  - Cathedral: +30% religious cohesion, pilgrim draw
  - Cloth Hall: +25% textile trade specifically

**Educational Value:** Central role of public space in medieval cities

**Fun Factor:** Civic building placement puzzle, event venue

**Implementation:** District system with required public spaces

---

### Mechanic 5: Village → Town → City Progression
**Academic Basis:** Medieval urban growth timeline and milestones

**Translation:**
- **Settlement Tiers:**
  1. **Hamlet** (10-100 pop): No market, subsistence farming
  2. **Village** (100-500 pop): Weekly local market, communal well
  3. **Town** (500-5000 pop): Regional market, charter, basic walls, town hall
  4. **City** (5000+ pop): Major hub, full fortifications, cathedral, guilds, advanced infrastructure

- **Variables:**
  - Population threshold
  - Infrastructure requirements
  - Charter status (legal rights)
  - Economic activity level
  - Political autonomy

- **Mechanics:**
  - Progression locked behind both population AND infrastructure
  - Each tier unlocks new buildings, policies, mechanics
  - **Hamlet → Village:**
    - Requirement: 100 pop + market building
    - Unlocks: Trade with nearby settlements, basic economy
  - **Village → Town:**
    - Requirements: 500 pop + charter (granted by lord/king) + town hall + basic walls
    - Unlocks: Guilds, self-governance, tax collection, regional trade
  - **Town → City:**
    - Requirements: 5000 pop + cathedral + full fortifications + 3+ guilds + advanced infrastructure (sewers OR water conduits)
    - Unlocks: Bishop seat, major trade hub, craft specialization, urban policies

- **Progression Formula:**
  ```
  tier_score = (
    population_met * 0.4 +
    infrastructure_met * 0.3 +
    economic_activity_met * 0.2 +
    charter_status * 0.1
  )

  can_advance = tier_score >= 1.0 && all_hard_requirements_met

  # Hard requirements (must have):
  # Village: market building
  # Town: charter, town hall, walls
  # City: cathedral, full walls, 3 guilds, 1 advanced infrastructure
  ```

**Educational Value:** Historical city development milestones

**Fun Factor:** Clear progression goals, unlockable content

**Implementation:** Milestone system with unlock gates

---

## Domain Translation Patterns

### Pattern: Scaling Laws → Growth Formulas
**Approach:**
1. Use empirical exponent (α = 0.75) from research
2. Apply to area-population relationship
3. Derive density increases as emergent property
4. Balance against gameplay (prevent exponential sprawl)

**Example:**
```
# Historical: Area ∝ Population^0.75
# Game: settled_area = 0.01 * population^0.75
# Result: Natural densification as city grows
```

---

### Pattern: Historical Dimensions → Building Scales
**Approach:**
1. Identify measured historical examples
2. Extract typical ranges
3. Create scaling formulas
4. Tune for visual/gameplay clarity

**Example:**
```
# Historical: Market squares 100-200m diameter
# Game: market_size = 50 + (population / 100) * 10
# Caps at reasonable sizes, splits into districts if too large
```

---

### Pattern: Infrastructure Progression → Tech Tree
**Approach:**
1. Order technologies by historical chronology
2. Map to health/efficiency impacts
3. Add cost-benefit tradeoffs
4. Create meaningful choices

**Example:**
```
Basic wells (cheap, low health) →
Conduits (medium, good health) →
Sewers (expensive, excellent health)
```

---

## Recommended Implementation Priority

### High Priority (Core Gameplay)
1. **Urban Scaling Laws** - Fundamental growth mechanics
2. **Zoning System** - Core city planning layer
3. **Village → Town → City Progression** - Clear goals

### Medium Priority (Depth)
4. **Infrastructure Tech Tree** - Strategic investment
5. **Market Square Mechanics** - Public space importance

### Low Priority (Polish)
6. **Advanced civic buildings** - Guild halls, cathedrals as unique structures

---

## Integration Points

**Existing Systems:**
- `agriculture.rs` - Agricultural zones within city walls
- `economics.rs` - Market prices, trade routes through cities
- `community.rs` - Town meetings in market square
- `illness.rs` - Disease spread influenced by sanitation infrastructure

**New Modules Needed:**
- `urban.rs` - City growth, density calculations, zoning
- `infrastructure.rs` - Water, sanitation, roads tech tree

**Data Structures:**
```rust
struct City {
    tier: SettlementTier,  // Hamlet/Village/Town/City
    population: u32,
    settled_area: f32,
    density: f32,
    districts: Vec<District>,
    infrastructure: Infrastructure,
    charter_status: bool,
}

struct District {
    zone_type: ZoneType,  // Residential/Commercial/Industrial/Civic
    buildings: Vec<Building>,
    land_value: f32,
}

enum ZoneType {
    Residential,
    Commercial,
    Industrial,
    Civic,
    Agricultural,
}

enum SettlementTier {
    Hamlet,    // 10-100
    Village,   // 100-500
    Town,      // 500-5000
    City,      // 5000+
}

struct Infrastructure {
    water_supply: WaterTech,
    sanitation: SanitationTech,
    roads: RoadTech,
    coverage: f32,  // 0.0-1.0
}
```

---

## Research Gaps Identified

### Need More Detail On:
1. **Specific construction costs** - Resources needed for walls, buildings, infrastructure
2. **Labor requirements** - Workers needed for construction projects
3. **Maintenance costs** - Ongoing expenses for infrastructure upkeep
4. **Fire mechanics** - Dense cities = fire risk (major medieval problem)

### Future Surveys:
1. **Medieval Fortifications** - Detailed wall/gate/tower systems
2. **Guild Economics** - Internal workings of craft guilds
3. **Urban Warfare** - Sieges, city defense
4. **Public Health** - Plague, fire, disaster response

---

## Confidence Assessment

**Overall Confidence:** High

**Strong Evidence For:**
- Scaling laws (rigorous quantitative study)
- Market square dimensions (multiple measured examples)
- Infrastructure types (archaeological evidence)
- Growth timeline (historical consensus)

**Weaker Evidence For:**
- Exact infrastructure costs (must estimate from game balance)
- Precise progression thresholds (tuned for gameplay)

**Assumptions Made:**
- Scaling constant (k) tuned for game pacing
- Simplified zoning (medieval reality more complex)
- District splitting at 5000 pop (game convenience)
- Linear infrastructure costs (reality likely more complex)

---

## Next Steps

1. **Prototype `urban.rs` module** with scaling calculations
2. **Implement zoning system** with proximity effects
3. **Add infrastructure tech tree** to existing tech system
4. **Test village → town → city progression** for pacing
5. **Create market square mechanics** for public events
6. **Playtest density problems** - is disease/fire risk balanced?

---

## Bibliography

- PLOS ONE: "Population-Area Relationship for Medieval European Cities" (Cesaretti et al., 2016)
- ArXiv: "Urban Scaling in Europe" (Bettencourt et al., 2015)
- StudioMatrx: "Medieval Urban Design: Architecture, City Planning Principles"
- Kobold Press: "Medieval Urbanism: The Labyrinthine Realities of the Medieval City"
- Medieval Hygiene - World History Encyclopedia
- Water-related Infrastructure in Medieval London (Medievalists.net)
- Medieval City Had Sophisticated Sanitation System (Córdoba study)
- SimCity and Urban Planning (academic analysis)
- Various European market square architectural studies (Kraków, Wrocław, Poznań)

---

**Tags:** urban-planning, medieval-cities, scaling-laws, zoning, infrastructure, market-squares, city-growth, density, public-health
