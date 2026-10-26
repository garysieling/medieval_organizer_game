# Ecology & Climate Literature Review

**Survey Date:** 2026-10-25
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Ecology, Climate Systems, Environmental Simulation

## Summary

Ecology and climate form interconnected systems affecting every aspect of medieval life. Historical climate shifts (Medieval Warm Period → Little Ice Age) caused famines and societal changes. Ecosystem dynamics follow mathematical models (Lotka-Volterra predator-prey, carrying capacity limits), while resource depletion (70% deforestation in medieval Europe) demonstrates environmental constraints. This research provides mechanics for climate simulation, ecosystem balance, and resource management.

## Key Sources

### Medieval Climate Patterns - Science (Mann et al. 2009)
**Rating:** A+
**Type:** Peer-reviewed scientific journal
**Relevance:** High - Historical climate data for game events

**Key Findings:**
- **Medieval Climate Anomaly (MCA):** 950-1250 AD (warm period)
- **Little Ice Age (LIA):** 1400-1700 AD (cold period)
- **Temperature Drop:** 0.6°C average, up to 2°C in some regions
- **Transition Period:** 1302-1307 severe droughts → 1310s wet/cold → Great Famine 1315-1321

**Regional Variations:**
- Europe: Heavy rainfall during LIA
- Africa/Central/Southern Asia: Droughts
- North America: Cooler and drier
- Southern Hemisphere: Different timing (not synchronized)

**Causes:**
- Reduced solar output
- Changes in atmospheric circulation
- Increased volcanism
- El Niño / North Atlantic Oscillation interactions

**Game Application:** Climate events, famine triggers, regional weather variation

---

### Crop Simulation Models - Multiple Sources (DSSAT, APSIM, EPIC)
**Rating:** A+
**Type:** Scientific crop modeling frameworks
**Relevance:** High - Climate effects on agriculture

**Key Relationships:**
- **Temperature Effects:**
  - Increased temp → faster crop development → shorter growing season
  - Wheat/faba bean: shorter season, reduced yields (5-41% loss)
  - Potatoes: longer season, increased yields (+33%)

- **Precipitation Effects:**
  - Agricultural drought more sensitive to precipitation than temperature
  - Precipitation thresholds critical in key growth months
  - Changing rainfall patterns reduce crop yields

- **Seasonal Impacts:**
  - Each crop has critical growth periods
  - Extreme weather (drought, floods) reduces yields
  - Multi-year patterns affect long-term food security

**Game Application:** Weather-based yield modifiers, drought/flood events

---

### Ecosystem Simulation - Gen3sis Model
**Rating:** A+
**Type:** Peer-reviewed ecological modeling (PLOS Biology)
**Relevance:** Medium-High - Scientific basis for ecosystem simulation

**Model Components:**
- Species abiotic tolerances
- Biotic interactions (predation, competition)
- Dispersal and migration
- Speciation and evolution
- Ecological trait development

**Emergent Properties:**
- α, β, γ diversity patterns
- Species range distributions
- Ecological trait evolution
- Phylogenetic relationships

**Scale:** Up to 10,000 species in EcoSISTEM variant

**Game Application:** Simplified ecosystem with species interactions

---

### Lotka-Volterra Predator-Prey Dynamics
**Rating:** A+
**Type:** Classical mathematical ecology
**Relevance:** High - Population cycle mechanics

**Core Equations:**
```
Prey:     dN/dt = rN - αNP
Predator: dP/dt = βαNP - δP

Where:
  N = prey population
  P = predator population
  r = prey growth rate
  α = predation rate
  β = conversion efficiency
  δ = predator death rate
```

**Behavior:**
- Oscillatory population cycles
- Time-lagged peaks (predators lag prey)
- Conservation invariants (closed orbits in phase space)
- Realistic dynamics (e.g., lynx-hare cycles)

**Game Application:** Wildlife population cycles, hunting sustainability

---

### Carrying Capacity & Resource Depletion - Multiple Sources
**Rating:** A
**Type:** Ecological game theory, commons research
**Relevance:** High - Resource management mechanics

**Key Concepts:**
- **Carrying Capacity (K):** Maximum sustainable population
- **Renewable Resources:** Regenerate at intrinsic rate
- **Tragedy of the Commons:** Individuals maximize extraction, depleting shared resource
- **Feedback Dynamics:** Resource state affects harvest strategies

**Mathematical Relationships:**
```
Resource growth: dR/dt = r*R*(1 - R/K) - H
Where:
  R = current resource level
  K = carrying capacity
  r = intrinsic growth rate
  H = total harvest rate
```

**Sustainability Condition:** H ≤ r*R*(1 - R/K)

**Game Application:** Hunting, fishing, forestry with sustainability limits

---

### Medieval Deforestation - Nature (Sci Reports)
**Rating:** A
**Type:** Historical ecology research
**Relevance:** High - Historical resource depletion patterns

**Deforestation Scale:**
- **England:** 15% wooded (1086) → 6% wooded (1340)
- **France:** 30M hectares (Charlemagne) → 13M hectares (1285-1314)
- **Overall:** 70% of forests removed in France, England, Eastern Germany by early 14th century

**Primary Causes:**
- Arable agriculture expansion (main driver)
- Fuel demand (growing pressure by 12th-13th centuries)
- Urban development
- Shipbuilding and construction

**Environmental Impacts:**
- Ecosystem imbalance
- Air pollution from coal use
- Soil erosion
- Local climate effects

**Management Responses:**
- King Philip V ordinances (1317, 1319)
- Forest administration reforms (1346)
- Principles: supervision, controlled usage, resource management
- Recognition of environmental problems

**Game Application:** Deforestation mechanic, environmental degradation, regulation policies

---

### Educational Ecosystem Games - Conservation Safari, Niche
**Rating:** B
**Type:** Game design examples
**Relevance:** Medium - Shows ecosystem mechanics in practice

**Conservation Safari Features:**
- Predator-prey relationships (lions, wildebeest, elephants)
- Population balance challenges
- Biodiversity maintenance
- Time-based survival (how long can ecosystem persist?)

**Niche Features:**
- Genetics and adaptation simulation
- Evolution over generations
- Climate change events
- Diseases and predators
- Randomized roguelike elements

**Lessons:**
- Ecosystem balance makes engaging gameplay
- Visual population feedback important
- Disaster events create tension
- Long-term thinking rewards

**Game Application:** Examples of ecosystem UI and feedback loops

---

## Game Mechanic Proposals

### Mechanic 1: Climate Event System
**Academic Basis:** Medieval Climate Anomaly → Little Ice Age transition (Science, Mann et al.)

**Translation:**
- **Climate States:**
  - Normal: Standard weather patterns
  - Warm Period: +10% crop yields, longer growing season
  - Cold Period: -20% crop yields, harsher winters
  - Drought: -50% crop yields, water scarcity
  - Wet Period: +20% flooding risk, disease spread

- **Transition Events:**
  - Gradual shifts over multiple years
  - Sudden events (volcanic eruptions, etc.)
  - Regional variation (different areas affected differently)

- **Variables:**
  - Base temperature (relative to normal)
  - Precipitation level (drought to flood)
  - Season lengths
  - Extreme event frequency

**Formulas:**
```rust
climate_state: ClimateState  // Normal/Warm/Cold/Drought/Wet
temperature_modifier: f32    // -2.0°C to +1.0°C
precipitation_modifier: f32  // 0.5x to 2.0x normal

crop_yield_modifier = 1.0 + (temperature_modifier * -0.1) + (precipitation_modifier - 1.0) * 0.3
famine_risk = climate_state == Cold && precipitation_modifier < 0.7
```

**Educational Value:** Shows climate impact on society, historical climate shifts

**Fun Factor:** Long-term strategic adaptation, crisis management

**Implementation:** `game2-core/src/sim/climate.rs`
```rust
struct Climate {
    state: ClimateState,
    temperature_offset: f32,
    precipitation_factor: f32,
    years_in_state: u32,
}

enum ClimateState {
    Normal,
    MedievalWarm,
    LittleIceAge,
    Drought,
    WetPeriod,
}
```

---

### Mechanic 2: Predator-Prey Wildlife Cycles
**Academic Basis:** Lotka-Volterra equations, lynx-hare cycles

**Translation:**
- **Species Pairs:**
  - Deer (prey) ↔ Wolves (predator)
  - Rabbits (prey) ↔ Foxes (predator)
  - Fish (prey) ↔ Pike (predator)

- **Variables (per species pair):**
  - Prey population (N)
  - Predator population (P)
  - Prey growth rate (r)
  - Predation rate (α)
  - Predator efficiency (β)
  - Predator death rate (δ)

- **Mechanics:**
  - Populations update each turn
  - Oscillating cycles (realistic boom/bust)
  - Hunting reduces prey (player interaction)
  - Overhunting collapses ecosystem

**Simplified Formulas:**
```rust
// Per turn update (discrete time approximation)
prey_growth = prey_pop * growth_rate * dt
predation_loss = prey_pop * predator_pop * predation_rate * dt
new_prey = prey_pop + prey_growth - predation_loss - player_hunting

predator_growth = predator_pop * prey_pop * efficiency * dt
predator_death = predator_pop * death_rate * dt
new_predator = predator_pop + predator_growth - predator_death
```

**Educational Value:** Shows population dynamics, ecosystem balance

**Fun Factor:** Hunting mini-game, sustainability challenge, visual population graphs

**Implementation:** `game2-core/src/sim/wildlife.rs`

---

### Mechanic 3: Forest Depletion & Regeneration
**Academic Basis:** Medieval deforestation (70% loss), carrying capacity models

**Translation:**
- **Forest as Renewable Resource:**
  - Growth rate: 1-3% per year (depending on conditions)
  - Carrying capacity: Maximum forest coverage for region
  - Current coverage: Percentage of maximum

- **Depletion Sources:**
  - Logging for fuel: -X trees/year
  - Land clearing for farms: -Y trees (one-time)
  - Construction: -Z trees per building
  - Natural disasters: Fire, blight

- **Regeneration:**
  - Natural growth when below carrying capacity
  - Planting programs (player intervention)
  - Soil quality affects growth rate

**Formulas:**
```rust
forest_growth_rate = base_rate * (1.0 - current_coverage / carrying_capacity)
depletion_rate = logging + land_clearing + construction
net_change = forest_growth_rate - depletion_rate

// Historical constraint: England went 15% → 6% over 254 years
// Rate: -0.035% per year with heavy use
```

**Historical Benchmarks:**
- England 1086: 15% forested
- England 1340: 6% forested (254 years, -9% total)
- Sustainable: Depletion ≤ Growth

**Environmental Effects:**
- Low forest: Fuel shortages, air pollution (coal use), soil erosion
- High forest: Biodiversity, game abundance, clean air

**Educational Value:** Tragedy of the commons, sustainability, environmental impact

**Fun Factor:** Resource management, long-term planning, crisis if mismanaged

**Implementation:** Track forest coverage, apply modifiers to economy/environment

---

### Mechanic 4: Weather-Modified Crop Yields
**Academic Basis:** DSSAT/APSIM crop models, temperature/precipitation relationships

**Translation:**
- **Weather Variables:**
  - Seasonal temperature (cold/normal/hot)
  - Seasonal precipitation (drought/normal/flood)
  - Extreme events (frost, hail, hurricane)

- **Crop Responses:**
  - Each crop has optimal temperature/precipitation range
  - Outside range = yield penalties
  - Extreme events = potential total loss

**Crop-Specific Effects:**
```rust
// Wheat: Cool-season crop
wheat_temp_optimal = 15-20°C
wheat_precip_optimal = 500-700mm

// Potato: Benefits from warmth
potato_temp_optimal = 18-24°C
potato_yield_bonus_if_warm = 1.33  // +33% from historical data

// Barley: Drought-tolerant
barley_precip_tolerance = 0.7  // Can handle 30% less water
```

**Yield Formula:**
```rust
temp_factor = gaussian_penalty(actual_temp, optimal_temp, tolerance)
precip_factor = gaussian_penalty(actual_precip, optimal_precip, tolerance)
extreme_modifier = has_extreme_event ? 0.5 : 1.0

final_yield = base_yield * temp_factor * precip_factor * extreme_modifier * rotation_bonus
```

**Educational Value:** Shows climate-agriculture coupling, crop specialization

**Fun Factor:** Adaptation challenge (pick right crops for climate), risk management

**Implementation:** Integrate with existing agriculture system, add weather layer

---

### Mechanic 5: Ecosystem Services & Degradation
**Academic Basis:** Ecological economics, carrying capacity, environmental impact

**Translation:**
- **Ecosystem Services Provided:**
  - **Forests:** Fuel, building materials, game, clean air
  - **Wetlands:** Fish, water filtration, flood control
  - **Grasslands:** Grazing, biodiversity

- **Degradation Effects:**
  - **Low forest (<10%):** Coal required (air pollution), reduced game, soil erosion
  - **Low wetlands (<5%):** Water quality drop, flood risk +50%, fish scarcity
  - **Overgrazed grass (<20%):** Desertification, livestock famine

- **Variables:**
  - Coverage % for each ecosystem type
  - Health/quality (0-100)
  - Services provided (quantified benefits)
  - Degradation penalties

**Formulas:**
```rust
forest_fuel_supply = forest_coverage * forest_health * base_fuel_rate
forest_game_abundance = forest_coverage * forest_health * base_game_rate

// Degradation thresholds
if forest_coverage < 0.10 {
    air_pollution = true;
    coal_required = true;
    game_hunting_penalty = 0.5;
}

if wetlands_coverage < 0.05 {
    flood_risk_multiplier = 1.5;
    water_quality = 0.5;
    fish_scarcity = true;
}
```

**Educational Value:** Shows ecosystem services concept, environmental costs

**Fun Factor:** Hidden costs of growth, environmental management challenge

**Implementation:** Track ecosystem coverages, apply service/penalty modifiers

---

## Domain Translation Patterns

### Pattern: Climate Periods → Game States
**Approach:**
1. Identify historical climate periods (MCA, LIA)
2. Extract key characteristics (temp, precip)
3. Map to game state transitions
4. Add event triggers (volcanic eruption, etc.)

**Example:**
```
Medieval Warm → prosperity, expansion
Transition (drought) → stress
Little Ice Age → famine, contraction
```

---

### Pattern: Mathematical Models → Simplified Simulation
**Approach:**
1. Start with academic model (Lotka-Volterra)
2. Discretize continuous equations (differential → difference)
3. Tune parameters for fun (not pure realism)
4. Add player interaction hooks

**Example:**
```
Continuous: dN/dt = rN - αNP
Discrete: N(t+1) = N(t) + r*N*dt - α*N*P*dt
Simplified: prey += growth - predation - hunting
```

---

### Pattern: Historical Data → Game Constraints
**Approach:**
1. Extract historical numbers (England 15% → 6% forest)
2. Calculate rates (-0.035%/year)
3. Use as baseline/constraint in game
4. Allow player to do better (or worse)

**Example:**
```
Historical deforestation: -9% over 254 years
Game: Player can avoid this through sustainable management
Or: Let it happen as historical challenge
```

---

## Recommended Implementation Priority

### High Priority (Core Gameplay)
1. **Climate Events** - Major impact, creates variety
2. **Forest Depletion** - Ties to existing economy, fuel needs
3. **Weather-Modified Yields** - Integrates with agriculture system

### Medium Priority (Depth)
4. **Predator-Prey Wildlife** - Hunting mini-game, ecosystem flavor
5. **Ecosystem Services** - Environmental consequences

### Low Priority (Polish)
6. **Advanced climate modeling** - Regional variations, etc.

---

## Integration Points

**Existing Systems:**
- `agriculture.rs` - Weather affects yields
- `economics.rs` - Forest provides fuel, construction materials
- `emergence.rs` - Climate and ecosystem CA rules
- `sim/` modules - General simulation framework

**New Modules Needed:**
- `climate.rs` - Climate state and weather generation
- `wildlife.rs` - Predator-prey populations
- `environment.rs` - Ecosystem coverage and health

**Data Structures:**
```rust
struct Climate {
    state: ClimateState,
    temperature_offset: f32,
    precipitation_factor: f32,
    years_in_state: u32,
    next_transition: Option<ClimateEvent>,
}

struct Wildlife {
    prey_population: u32,
    predator_population: u32,
    growth_rate: f32,
    predation_rate: f32,
}

struct Environment {
    forest_coverage: f32,      // 0.0-1.0
    wetlands_coverage: f32,
    grassland_coverage: f32,
    ecosystem_health: u8,      // 0-100
}
```

---

## Research Gaps Identified

### Need More Detail On:
1. **Specific medieval weather events** - Historical records of droughts, floods
2. **Regional climate variations** - Different areas of medieval Europe
3. **Species extinction rates** - What happened when forests disappeared?
4. **Soil degradation timelines** - How fast does overfarming damage land?

### Future Surveys:
1. **Historical Weather Records** - Chronicle accounts of famines, climate
2. **Soil Science** - Nutrient depletion, erosion, recovery
3. **Biodiversity Loss** - Species loss from medieval land use
4. **Environmental Economics** - Cost-benefit of conservation

---

## Confidence Assessment

**Overall Confidence:** High

**Strong Evidence For:**
- Climate periods (MCA, LIA) - Multiple scientific sources
- Lotka-Volterra dynamics - Classical, well-validated
- Medieval deforestation - Historical records (Nature)
- Crop-climate relationships - Modern agricultural science

**Weaker Evidence For:**
- Exact carrying capacity values - Model-dependent
- Specific medieval ecosystem compositions - Limited data
- Regeneration rates in game context - Simplified

**Assumptions Made:**
- Discrete time approximation of continuous models
- Simplified from 10,000 species to manageable set
- Tuned for gameplay (30-year forest recovery vs 100+ reality)

---

## Next Steps

1. **Prototype `climate.rs`** with state system and weather generation
2. **Implement forest depletion** tracking and fuel economy integration
3. **Add weather modifiers** to existing agriculture system
4. **Test climate transitions** - Are they noticeable? Too harsh?
5. **Create predator-prey simulation** for hunting system
6. **Playtest environmental degradation** - Can player recover from mistakes?

---

## Bibliography

- Mann et al. (2009): "Global Signatures and Dynamical Origins of the Little Ice Age and Medieval Climate Anomaly" - Science
- Gen3sis ecosystem simulation model - PLOS Biology
- DSSAT, APSIM, EPIC crop simulation frameworks
- Lotka-Volterra predator-prey dynamics (classical mathematical ecology)
- Nature Scientific Reports: Medieval deforestation and land-use change
- Eco-evolutionary games research on renewable resources
- Historical sources on medieval environmental management

---

**Tags:** ecology, climate, medieval-climate, little-ice-age, predator-prey, deforestation, ecosystem, carrying-capacity, environmental-degradation, weather-simulation
