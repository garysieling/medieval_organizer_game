# Military History & Strategy - Medieval Warfare for Simple Battle Systems

**Research Domain:** Military History, Medieval Warfare, Battle Tactics, Fortification
**Survey Date:** 2026-10-25
**Researcher:** Literature Survey Agent (Cycle 3, Survey 6)
**Status:** COMPLETE

---

## Executive Summary

This survey examines medieval military organization, tactics, and battle mechanics to support simple battle system implementation in the game. **Note:** This is deliberately lightweight coverage since detailed military simulation is low priority (simple battle system planned). Medieval warfare evolved through distinct phases: early feudal levies and shield walls gave way to professional soldiers and combined-arms tactics featuring longbowmen, pike squares, and heavy cavalry. Fortifications progressed from wooden motte-and-bailey structures to sophisticated concentric stone castles resistant to siege. Military organization transitioned from feudal obligations to mercenary professionals paid in coin. Logistics determined campaign success more than tactics, with foraging limiting armies to 5-10 miles/day versus 20 miles/day with naval supply. For game mechanics, Lanchester's Linear Law models ancient/medieval combat attrition, morale determines rout points, and fortifications provide defensive multipliers. This provides sufficient depth for a simple but realistic battle system without requiring detailed tactical simulation.

**Confidence Level:** HIGH (A tier sources, well-documented history)
**Implementation Priority:** Low-Medium (simple battle system sufficient)
**Mechanics Yield:** 4 core systems for basic combat

---

## 1. Medieval Warfare Evolution

### 1.1 Early Medieval (500-1000 CE)

**Dominant Tactics:**
- **Shield Wall**: Soldiers stand shoulder-to-shoulder, overlapping shields to create nearly impenetrable barrier
- **Infantry-focused**: Limited cavalry, mostly foot soldiers
- **Tribal/local forces**: No professional armies

**Limitations:**
- Effective against frontal assault, weak against flanking
- Immobile once formed (cannot pursue routed enemy)
- Vulnerable to missile fire from above

---

### 1.2 High Medieval (1000-1300 CE)

**Feudal System Peak:**
- Heavy cavalry (knights) as elite force
- Feudal levies provide infantry mass
- Castle-building boom

**Tactics:**
- Cavalry charge to break enemy lines
- Infantry shield walls for defense
- Archers provide missile support

**Evolution:**
- Stirrups enabled heavy cavalry shock charges
- Crossbows threaten armored knights
- Castles force siege warfare (costly, slow)

---

### 1.3 Late Medieval (1300-1500 CE)

**Revolutionary Changes:**

**English Longbow (Agincourt 1415):**
- Capable of releasing volleys that decimate opposing forces
- Effective against armored cavalry at range
- English longbowmen famous for prowess

**Swiss Pike Squares:**
- Soldiers armed with long pikes form tight square with pikes protruding outward
- Excellent deterrent against cavalry
- Almost invincible on battlefield with famed discipline
- In Late Middle Ages, shield abandoned in favor of polearms carried with both hands

**Mercenary Professionals:**
- Coinage replaces feudal obligations
- Rulers hire soldiers directly rather than summon vassals
- Professional companies emerge (Condottieri, Landsknechts)

---

## 2. Fortifications

**Primary Sources:** Medieval castle history, Fortification evolution studies

### 2.1 Motte-and-Bailey (Early Medieval)

#### Structure

- **Motte**: Natural or artificial mound with wooden tower on top
- **Bailey**: Walled courtyard at base (living area + secondary defense)
- **Ditch/moat**: Surrounds entire structure

#### Advantages

> "It provided a maximum amount of defense at the lowest cost of construction, and it was possible to build one within days."

- Quick to construct (days to weeks)
- Cheap (local materials: wood, earth)
- Effective against raids and small attacks

#### Disadvantages

- Wooden structures vulnerable to fire
- Limited garrison capacity
- Relatively weak against determined siege

---

### 2.2 Stone Castles (High Medieval)

**Evolution:**
- Replace wooden structures with stone
- Thicker walls resist battering rams
- Taller keeps provide better visibility and defense

**Key Features:**
- **Keep**: Central fortified tower (last refuge)
- **Curtain walls**: Outer defensive walls
- **Towers at intervals**: Provide flanking fire from archers
- **Gatehouses**: Heavily fortified entrances (murder holes, portcullis)

---

### 2.3 Concentric Castles (Late Medieval, 1270-1350)

**Peak Design:**

> "Concentric castles represented the peak of medieval military design."

#### Structure

- **Series of concentric defensive walls**
- **Enlarged keep** at center as last stronghold
- **Multiple layers**: Enemy must breach each wall sequentially

#### Defensive Advantage

> "Even if the enemy managed to break through the first line of defense, they could at least be ambushed before they reached the second."

- Defenders on inner walls fire over outer walls
- Attackers trapped between walls under crossfire
- Multiple fallback positions

#### Decline

- Gunpowder weapons (cannons) eventually made thick stone walls vulnerable
- Extremely expensive to build and maintain
- Required large garrisons

---

## 3. Military Organization

**Primary Sources:** Medieval army organization studies, Feudal military system research

### 3.1 Feudal Levy (Early-High Medieval)

#### Structure

**Before 13th century**, feudal levy of knights was customary method:
- Lords owed military service to king (typically 40 days/year)
- Knights owed service to lords
- Peasant levies supplement professional warriors

#### Composition

> "Most medieval armies had at their core the royalty, nobility, and knights of a kingdom, with these men forming the heavy cavalry, supplemented with levies of peasants and townsmen who formed the infantry, archers/crossbowmen, and light cavalry."

**Hierarchy:**
- **Heavy cavalry**: Armored knights (nobility)
- **Infantry**: Peasant levies with spears
- **Archers/crossbowmen**: Specialized missile troops
- **Light cavalry**: Scouts, skirmishers

#### Limitations

- Limited campaign duration (40-day service obligation)
- Seasonal (agricultural calendar determines availability)
- Variable quality (peasants untrained)
- Difficult to coordinate (lords maintain independent forces)

---

### 3.2 Professional Soldiers (Late Medieval)

#### Household Troops

**Huscarls** and similar:
- Permanent retainers of lords
- Trained soldiers (precursors of standing armies)
- Reliable core of military force

**Man-at-Arms:**
- Professional fighting core of infantry
- Highly trained soldiers (often retainers or mercenaries)
- Dedicate lives to warfare

---

### 3.3 Mercenaries

#### Rise of Mercenaries

> "As coinage replaced traditional obligations of feudal service, rulers found it easier to hire soldiers directly rather than summon vassals for limited campaigns."

#### Advantages

- No service time limits (paid for duration)
- Professional quality
- Specialists available (pikemen, heavy cavalry, archers)
- Cohesive units under captains

#### Famous Mercenary Groups

**Swiss Pikemen:**
- Devastating pike formations
- Almost unbeatable on battlefield
- Discipline legendary

**Italian Condottieri:**
- Elite captains leading professional companies
- Sophisticated tactics

**German Landsknechts:**
- Flamboyant infantry
- Skilled with pikes and zweihänder (two-handed swords)

#### Disadvantages

- Expensive (ongoing coin payments)
- Loyalty to money, not realm (may switch sides)
- Can plunder if unpaid

---

## 4. Logistics: The Real Constraint

**Primary Sources:** A Collection of Unmitigated Pedantry (Bret Devereaux), Military logistics research

### 4.1 Campaign Season

> "The campaign season normally corresponded to the seasons when there would be food on the ground and relatively good weather, usually from spring to autumn."

**Fodder Seasonality:**
- **Green fodder**: Grasses in early spring
- **Dry fodder**: Agricultural products (barley mid-summer, oats late spring)

**Implication:** Campaigns outside harvest season face severe supply difficulties.

---

### 4.2 Foraging ("Living Off the Land")

#### Definition

> "The usual method for solving logistical problems for smaller armies was foraging or 'living off the land' - effectively stealing whatever was needed: animals, crops, wood and so on."

#### Process

- Foragers fan out from army
- Collect spoils (food, fodder, livestock)
- Carry to baggage train

#### Speed Cost

> "Foraging would slow the army's progress to 5 to 10 miles a day."

Why slow:
- Time to search countryside
- Dispersed foraging vulnerable to attacks
- Must maintain cohesion (can't spread too far)

#### Devastation

Foraging devastates local population:
- Crops stolen, fields trampled
- Livestock taken
- Buildings burned (firewood, denial to enemy)

**Result:** Scorched earth along line of march.

---

### 4.3 Baggage Trains

#### Contents

> "The baggage train had to transport all supplies that a medieval army could possibly need during the campaign."

Includes:
- Food (grain, salted meat)
- Clothes and tents
- Weapons and armor (replacements)
- Arrows (lots!)
- Materials for siege equipment
- Wood for bridges

#### Transport

- **Carts**: 1102 lb (500 kg) capacity
- **Draft animals**: Horses or oxen pull carts
- **Huge size**: Baggage train often longer than army itself

#### Vulnerability

Baggage trains:
- Slow (limit army march speed)
- Vulnerable to raids
- Require guards (reducing combat strength)
- Tempting target for enemy

---

### 4.4 Supply Requirements

**Per Soldier:**
- 2.2 lb (1 kg) of grain per day
- Additional supplies (arrows, equipment)

**Per Horse:**
- 11 lb (5 kg) of grain per day

**Calculation:**
An army of 10,000 men with 2,000 horses needs:
- 10,000 kg grain for soldiers
- 10,000 kg grain for horses
- **Total: 20,000 kg (20 tons) per day**

Over a 30-day campaign: **600 tons of grain** minimum.

---

### 4.5 Naval Supply Advantage

> "Medieval armies resupplied by ships could move much faster, up to 20 miles per day, compared to 5 to 10 miles a day while foraging."

**Why faster:**
- Ships carry massive supplies (no foraging delays)
- Army marches unencumbered
- Can campaign farther from base

**Limitation:**
- Requires navigable rivers or coastlines
- Naval superiority needed
- Not available inland

---

## 5. Battle Mechanics

**Primary Sources:** Lanchester's Laws research, Combat modeling studies

### 5.1 Lanchester's Linear Law (Ancient/Medieval Combat)

#### Applicable Model

> "Lanchester's Linear Law applies to ancient combat."

**Why Linear?**
> "Between phalanxes of soldiers with spears, one soldier could only ever fight exactly one other soldier at a time."

Medieval shield walls, pike squares, and melee combat fit this model: Each combatant engages a single opponent.

#### Formula

```
Casualties_A = k_B * B * t
Casualties_B = k_A * A * t
```

Where:
- `A`, `B` = Number of soldiers on each side
- `k_A`, `k_B` = Combat effectiveness (kill rate per soldier)
- `t` = Time of battle

#### Result

> "If each soldier kills, and is killed by, exactly one other, then the number of soldiers remaining at the end of the battle is simply the difference between the larger army and the smaller."

**Example:**
- Army A: 1000 soldiers (k=1)
- Army B: 600 soldiers (k=1)
- Result: Army A wins with ~400 survivors

**Linear scaling:** Victory margin = Initial numerical advantage.

---

### 5.2 When Square Law Applies

**Square Law:**
> "In situations where the more numerous side can make its superior numbers tell, bringing everyone into the battle and ganging up on opponents, numerical advantage becomes disproportionately important."

**Example Scenario:**
- Ranged combat (archers can all fire at enemy)
- Open field (can surround and gang up)
- One-sided mobility (cavalry vs infantry)

**Medieval Context:**
- Mostly linear (melee, shield walls, narrow fronts)
- Occasionally square (massed archery, cavalry charges)

**Game Design:**
- Use linear law for simplicity
- Add modifiers for combined-arms tactics

---

### 5.3 Morale and Rout

#### Importance

> "Lanchester's Laws don't take into account such considerations as terrain, morale, weapon range, movement and maneuver, surprise, weather, and many other issues that have decided battles."

**Historical Reality:**
Most medieval battles ended not with annihilation but with **rout** (one side flees).

#### Rout Mechanics

**Triggers:**
- Heavy casualties (>20-30% losses)
- Commander killed/captured
- Flanked or surrounded
- Surprise attack
- Allies retreat

**Cascade:**
Once a few soldiers flee, panic spreads:
- Reinforcing feedback loop (more flee → others panic → even more flee)
- Can turn winning position into total rout

**After Rout:**
- Fleeing soldiers vulnerable (can't defend)
- Pursuit cavalry inflicts most casualties here
- Battle decisively won once enemy routs

**Game Mechanic:**
Morale as separate stat from casualties. When morale breaks → rout → rapid defeat.

---

## 6. Game Mechanic Translations

### Mechanic 1: Simple Battle Resolution System

**System:** Lanchester's Linear Law with morale and modifiers

```rust
struct Army {
    soldiers: u32,
    combat_effectiveness: f32,  // k value (kills per soldier per time unit)
    morale: f32,                // 0.0-1.0
    commander_skill: f32,
    fortified: bool,
}

struct BattleModifiers {
    terrain: TerrainType,
    fortification: FortificationType,
    surprise: bool,
    numerical_advantage: f32,
}

enum TerrainType {
    OpenField,      // No modifier
    Forest,         // -20% effectiveness for cavalry
    Hills,          // +10% for defender
    River,          // -30% for attacker crossing
    Marsh,          // -40% movement, -20% effectiveness
}

enum FortificationType {
    None,
    FieldWorks,      // Hastily dug trenches (+20% defense)
    MotteBailey,     // Wooden fort (+50% defense)
    StoneCastle,     // (+100% defense)
    ConcentricCastle, // (+200% defense, requires siege)
}

impl Army {
    fn battle_round(&mut self, enemy: &mut Army, modifiers: &BattleModifiers, time_step: f32) -> BattleOutcome {
        // Apply terrain and fortification modifiers
        let my_effectiveness = self.combat_effectiveness * self.get_terrain_modifier(modifiers);
        let enemy_effectiveness = enemy.combat_effectiveness * enemy.get_terrain_modifier(modifiers);

        // Lanchester's Linear Law: Casualties proportional to enemy force size and effectiveness
        let my_casualties = (enemy.soldiers as f32 * enemy_effectiveness * time_step) as u32;
        let enemy_casualties = (self.soldiers as f32 * my_effectiveness * time_step) as u32;

        self.soldiers = self.soldiers.saturating_sub(my_casualties);
        enemy.soldiers = enemy.soldiers.saturating_sub(enemy_casualties);

        // Morale damage from casualties
        self.morale -= (my_casualties as f32 / (self.soldiers + my_casualties) as f32) * 0.3;
        enemy.morale -= (enemy_casualties as f32 / (enemy.soldiers + enemy_casualties) as f32) * 0.3;

        // Check for rout
        if self.morale < 0.3 {
            return BattleOutcome::Rout { routed: ArmyId::Self_ };
        }
        if enemy.morale < 0.3 {
            return BattleOutcome::Rout { routed: ArmyId::Enemy };
        }

        // Check for annihilation
        if self.soldiers == 0 {
            return BattleOutcome::Annihilated { defeated: ArmyId::Self_ };
        }
        if enemy.soldiers == 0 {
            return BattleOutcome::Annihilated { defeated: ArmyId::Enemy };
        }

        BattleOutcome::Ongoing
    }

    fn get_terrain_modifier(&self, modifiers: &BattleModifiers) -> f32 {
        let mut modifier = 1.0;

        // Terrain effects
        match modifiers.terrain {
            TerrainType::OpenField => {},
            TerrainType::Forest => modifier *= 0.8,
            TerrainType::Hills => modifier *= 1.1,  // Assuming defender
            TerrainType::River => modifier *= 0.7,   // Assuming attacker
            TerrainType::Marsh => modifier *= 0.6,
        }

        // Fortification effects (defender only)
        if self.fortified {
            modifier *= match modifiers.fortification {
                FortificationType::None => 1.0,
                FortificationType::FieldWorks => 1.2,
                FortificationType::MotteBailey => 1.5,
                FortificationType::StoneCastle => 2.0,
                FortificationType::ConcentricCastle => 3.0,
            };
        }

        // Surprise penalty
        if modifiers.surprise {
            modifier *= 0.5;
        }

        modifier
    }
}

enum BattleOutcome {
    Ongoing,
    Rout { routed: ArmyId },
    Annihilated { defeated: ArmyId },
}

enum ArmyId {
    Self_,
    Enemy,
}
```

**Gameplay:**
- Simple but realistic (Lanchester's Linear Law)
- Morale matters (rout more common than annihilation)
- Fortifications powerful (concentric castle = 3x defensive advantage)
- Terrain affects tactics (don't cross rivers into battle!)

---

### Mechanic 2: Military Organization & Recruitment

**System:** Feudal levies vs. professional soldiers vs. mercenaries

```rust
enum MilitaryUnit {
    FeudalLevy {
        soldiers: u32,
        quality: f32,           // 0.3-0.6 (peasant militia, poor quality)
        service_days_remaining: u32,  // Limited to 40 days typically
        morale_bonus: f32,      // +10% defending home territory
    },
    ProfessionalSoldiers {
        soldiers: u32,
        quality: f32,           // 0.6-0.9 (trained warriors)
        monthly_wages: u32,
        morale_bonus: f32,      // +20% (professional pride)
    },
    Mercenaries {
        soldiers: u32,
        quality: f32,           // 0.7-1.0 (best fighters)
        monthly_wages: u32,
        loyalty: f32,           // 0.0-1.0 (depends on payment, can switch sides)
        specialty: MercenaryType,
    },
}

enum MercenaryType {
    SwissPikemen,       // +50% vs cavalry
    ItalianCondottieri, // +20% tactical skill
    EnglishLongbowmen,  // +100% ranged effectiveness
    Landsknechts,       // Versatile
}

struct MilitaryForce {
    units: Vec<MilitaryUnit>,
    commander: CommanderId,
    supply_days: u32,       // Days of food remaining
}

impl MilitaryUnit {
    fn maintain_cost_per_month(&self) -> u32 {
        match self {
            MilitaryUnit::FeudalLevy { soldiers, .. } => {
                0  // Feudal obligation, no coin cost
            },
            MilitaryUnit::ProfessionalSoldiers { soldiers, monthly_wages, .. } => {
                soldiers * monthly_wages
            },
            MilitaryUnit::Mercenaries { soldiers, monthly_wages, .. } => {
                soldiers * monthly_wages
            },
        }
    }

    fn desertion_risk(&self, months_unpaid: u32) -> f32 {
        match self {
            MilitaryUnit::FeudalLevy { .. } => {
                0.0  // Can't desert (feudal obligation), but service expires
            },
            MilitaryUnit::ProfessionalSoldiers { .. } => {
                if months_unpaid > 2 {
                    0.2 * months_unpaid as f32  // 20% per month unpaid
                } else {
                    0.0
                }
            },
            MilitaryUnit::Mercenaries { loyalty, .. } => {
                if months_unpaid > 0 {
                    (1.0 - loyalty) * 0.5 * months_unpaid as f32  // High risk if unpaid
                } else {
                    0.0
                }
            },
        }
    }
}
```

**Progression:**
- Early game: Feudal levies (cheap, limited duration, poor quality)
- Mid game: Professional soldiers (reliable, ongoing cost)
- Late game: Elite mercenaries (expensive, high quality, loyalty risk)

**Gameplay:**
- Feudal levies expire after 40 days (must dismiss or convert to paid)
- Mercenaries can switch sides if unpaid
- Professionals most reliable but expensive

---

### Mechanic 3: Fortification & Siege

**System:** Fortification tiers with siege requirements

```rust
enum Fortification {
    None,
    Wooden Palisade {
        hp: u32,                 // Hit points
        garrison_capacity: u32,
        construction_time_days: u32,  // ~7 days
        cost: u32,               // Cheap (local wood)
    },
    MotteBailey {
        hp: u32,
        garrison_capacity: u32,
        construction_time_days: u32,  // ~30 days
        cost: u32,               // Moderate (earthworks + wood)
    },
    StoneCastle {
        hp: u32,
        garrison_capacity: u32,
        construction_time_days: u32,  // ~365 days (1 year)
        cost: u32,               // Expensive (stone, masons)
    },
    ConcentricCastle {
        hp: u32,
        walls: u32,              // Number of concentric walls (2-3)
        garrison_capacity: u32,
        construction_time_days: u32,  // ~1825 days (5 years)
        cost: u32,               // Very expensive
    },
}

struct SiegeOperation {
    attacker: ArmyId,
    defender: ArmyId,
    fortification: Fortification,
    days_elapsed: u32,
    supply_stockpile: u32,  // Defender's food (bushels)
}

impl SiegeOperation {
    fn daily_update(&mut self) -> SiegeOutcome {
        self.days_elapsed += 1;

        // Defender consumes food
        let garrison_size = self.get_garrison_size();
        let daily_consumption = garrison_size * 1;  // 1 unit per person per day
        self.supply_stockpile = self.supply_stockpile.saturating_sub(daily_consumption);

        // Starvation check
        if self.supply_stockpile == 0 {
            return SiegeOutcome::Surrender { reason: "Starvation".to_string() };
        }

        // Attacker assault attempts (risky, high casualties)
        if self.attacker_attempts_assault() {
            let assault_result = self.resolve_assault();
            if assault_result.success {
                return SiegeOutcome::Captured;
            } else {
                // Heavy attacker casualties
                return SiegeOutcome::AssaultRepelled { attacker_casualties: assault_result.casualties };
            }
        }

        // Defender morale check (long siege wears down morale)
        if self.days_elapsed > 180 && random() < 0.01 * self.days_elapsed as f32 {
            return SiegeOutcome::Surrender { reason: "Morale collapse".to_string() };
        }

        SiegeOutcome::Ongoing
    }

    fn resolve_assault(&self) -> AssaultResult {
        // Fortified defenders have massive advantage
        let defender_advantage = match self.fortification {
            Fortification::WoodenPalisade => 2.0,
            Fortification::MotteBailey => 3.0,
            Fortification::StoneCastle => 5.0,
            Fortification::ConcentricCastle => 10.0,
            _ => 1.0,
        };

        let attacker_strength = self.get_attacker_strength();
        let defender_strength = self.get_defender_strength() * defender_advantage;

        if attacker_strength > defender_strength * 2.0 {
            AssaultResult { success: true, casualties: (attacker_strength * 0.3) as u32 }
        } else {
            AssaultResult { success: false, casualties: (attacker_strength * 0.5) as u32 }
        }
    }
}

enum SiegeOutcome {
    Ongoing,
    Captured,
    Surrender { reason: String },
    AssaultRepelled { attacker_casualties: u32 },
    SiegeLifted,  // Attacker gives up
}
```

**Gameplay:**
- Sieges take months (historically accurate)
- Starvation primary siege tactic
- Assaults very costly for attackers
- Concentric castles nearly impregnable (10x defender advantage)
- Siege logistics matter (attacker also needs food!)

---

### Mechanic 4: Campaign Logistics

**System:** Foraging and supply management

```rust
struct Campaign {
    army: Army,
    position: Location,
    supply_stockpile: u32,  // Bushels of grain
    days_on_campaign: u32,
    supply_method: SupplyMethod,
}

enum SupplyMethod {
    Foraging {
        devastation_radius: f32,  // Miles around path
    },
    BaggageTrain {
        cart_capacity: u32,
        carts: u32,
    },
    NavalSupply {
        port: LocationId,
        ships_available: u32,
    },
}

impl Campaign {
    fn daily_march(&mut self) -> MarchResult {
        let consumption = self.army.daily_consumption();

        match &mut self.supply_method {
            SupplyMethod::Foraging { devastation_radius } => {
                // Slow movement (5-10 miles/day)
                let march_distance = random_range(5.0, 10.0);

                // Forage from countryside
                let foraged = self.forage_area(*devastation_radius);
                self.supply_stockpile += foraged;

                // Devastate local population (game consequence)
                self.devastate_countryside(*devastation_radius);

                MarchResult {
                    distance: march_distance,
                    supplies_gained: foraged,
                    population_loss: (foraged as f32 * 0.5) as u32,  // Half of food stolen = local starvation
                }
            },
            SupplyMethod::BaggageTrain { cart_capacity, carts } => {
                // Moderate movement (10-15 miles/day)
                let march_distance = random_range(10.0, 15.0);

                // No foraging, rely on stored supplies
                self.supply_stockpile = self.supply_stockpile.saturating_sub(consumption);

                if self.supply_stockpile == 0 {
                    return MarchResult {
                        distance: 0.0,  // Can't march, must forage or retreat
                        supplies_gained: 0,
                        population_loss: 0,
                    };
                }

                MarchResult {
                    distance: march_distance,
                    supplies_gained: 0,
                    population_loss: 0,
                }
            },
            SupplyMethod::NavalSupply { port, ships_available } => {
                // Fast movement (up to 20 miles/day)
                let march_distance = random_range(15.0, 20.0);

                // Ships provide continuous supply
                let naval_resupply = ships_available * 100;  // 100 bushels per ship
                self.supply_stockpile += naval_resupply;

                MarchResult {
                    distance: march_distance,
                    supplies_gained: naval_resupply,
                    population_loss: 0,
                }
            },
        }
    }

    fn forage_area(&self, radius: f32) -> u32 {
        // Collect food from area (steals from peasants)
        let area = radius * radius * 3.14;  // Approximate circle area
        let farms_in_area = area * 5.0;  // ~5 farms per square mile
        let grain_per_farm = 50.0;  // Bushels available

        (farms_in_area * grain_per_farm) as u32
    }

    fn devastate_countryside(&mut self, radius: f32) {
        // Historical reality: Foraging armies leave famine in their wake
        // Reduce population, destroy crops, burn buildings
        // (Game mechanic: Creates long-term consequences for player)
    }
}

struct MarchResult {
    distance: f32,           // Miles marched
    supplies_gained: u32,
    population_loss: u32,    // Civilians who starve due to foraging
}
```

**Gameplay:**
- Foraging enables long campaigns but devastates countryside (moral consequence)
- Baggage trains reliable but slow
- Naval supply fastest but requires coastline/rivers
- Campaign season matters (spring-autumn only)
- Logistics often more important than tactics

---

## 7. Implementation Notes

### 7.1 Simplicity Justification

**This survey is deliberately lightweight** because:
- Game plan includes "simple battle system"
- Detailed tactical simulation not required
- Four core mechanics sufficient for strategic layer

**What's Covered:**
- Battle resolution (Lanchester's Linear Law)
- Military organization (feudal/professional/mercenary)
- Fortifications (defensive advantage)
- Logistics (foraging, supply, campaign limits)

**What's Omitted:**
- Detailed formation tactics
- Individual unit types (beyond basic categories)
- Weapon-specific mechanics
- Siege equipment details
- Naval warfare

**Rationale:** These mechanics provide sufficient depth for strategic decisions (hire mercenaries? build castle? forage or baggage train?) without requiring complex tactical combat simulation.

---

### 7.2 Balance Considerations

**Fortifications:**
- Concentric castles very expensive (5 years construction, massive cost)
- But provide 3x defensive advantage (or 10x for assault)
- Sieges take months → time cost even if you win

**Mercenaries:**
- High quality (0.7-1.0 effectiveness)
- Expensive (ongoing wages)
- Loyalty risk (can switch sides if unpaid)

**Logistics:**
- Foraging devastates countryside (moral consequence, population loss)
- Naval supply fastest but requires geography
- Campaign season limited (spring-autumn)

---

### 7.3 Historical Accuracy vs. Fun

**Prioritize Fun:**
- Real sieges could last years (Acre 1189-1191: 2 years!)
- Compress timescales for playability
- But preserve core dynamics (sieges slow, foraging devastating, fortifications powerful)

**Educational Value:**
- Players learn logistics matter more than tactics
- Understand why medieval armies moved slowly
- See consequences of foraging (famine for civilians)

---

## 8. Sources and Confidence

### Medieval Warfare (A tier)
- Shield wall tactics (well-documented, multiple battles)
- English longbow effectiveness (Agincourt 1415, historical records)
- Swiss pike squares (military history consensus)
- Mercenary transition (economic records, coin evidence)

### Fortifications (A tier)
- Motte-and-bailey construction (archaeological evidence, Bayeux Tapestry)
- Concentric castle evolution (surviving structures, construction records)
- Siege warfare (extensive siege chronicles)

### Military Organization (A tier)
- Feudal levy system (feudal contracts preserved)
- Professional soldiers (household troop records)
- Mercenary companies (payment records, contracts)

### Logistics (A+ tier)
- Foraging practices (chronicler accounts, devastation evidence)
- Baggage train requirements (supply calculations from campaigns)
- Naval supply advantage (historical campaign speeds)
- Bret Devereaux "A Collection of Unmitigated Pedantry" (excellent sourcing)

### Combat Modeling (A tier)
- Lanchester's Laws (mathematical framework, validated)
- Linear law applicability to ancient/medieval (consensus among modelers)
- Morale and rout (military history consensus, battle accounts)

---

## 9. Conclusion

Medieval warfare evolved from infantry shield walls and feudal levies to professional combined-arms forces featuring longbowmen, pike squares, and heavy cavalry. Fortifications progressed from quick wooden motte-and-bailey structures to sophisticated concentric stone castles that required months-long sieges to capture. Military organization transitioned from feudal obligations (40-day service limits) to mercenary professionals paid in coin. But **logistics determined success** more than tactics: foraging limited armies to 5-10 miles/day and devastated countryside, while naval supply enabled 20 miles/day movement but required coastal geography.

For game mechanics, this provides four core systems:

1. **Battle Resolution**: Lanchester's Linear Law (casualties = enemy size × effectiveness), modified by terrain, fortifications, and morale. Rout more common than annihilation.

2. **Military Organization**: Feudal levies (cheap, limited duration, poor quality) vs. professional soldiers (reliable, expensive) vs. mercenaries (elite, very expensive, loyalty risk).

3. **Fortifications**: Wooden palisade (2x defense) → Motte-and-bailey (3x) → Stone castle (5x) → Concentric castle (10x). Construction time and cost scale with effectiveness.

4. **Campaign Logistics**: Foraging (slow, devastates population) vs. Baggage trains (moderate, limited by supplies) vs. Naval supply (fast, requires coastline). Campaign season spring-autumn only.

These mechanics enable strategic military gameplay (when to hire mercenaries? build castles? where to campaign?) without requiring detailed tactical combat simulation. Players learn that logistics matter more than tactics, understand why medieval campaigns moved slowly, and face moral consequences of foraging (causing civilian famines).

This lightweight coverage is appropriate for the game's "simple battle system" plan, providing sufficient depth for strategic decisions while keeping implementation manageable.

---

**Next Research:** If expanding military system, could explore: Siege equipment (trebuchets, battering rams), Naval warfare, Specific famous battles, Medieval military doctrine.
