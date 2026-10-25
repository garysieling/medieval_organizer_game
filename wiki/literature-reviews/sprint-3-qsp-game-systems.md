# Lit Review - Sprint 3: QSP/ADME Applied to Game Systems

**Date:** 2026-10-25
**Domain:** Pharmacology / Systems Modeling
**Sources:** 4 works on QSP modeling

## Sources

### 1. gQSPSim (Genentech QSP Simulator)
- GUI for QSP model development, calibration, virtual subjects
- **Finding:** QSP = species + doses + variants + rules; virtual populations explore variability
- **Relevance:** HIGH - Virtual populations = our heterogeneous agents. Parameter variability.

### 2. QSP Toolbox (AAPS Journal)
- Multi-scale mechanistic models, workflow components
- **Finding:** QSP models are multi-compartment ODE systems; simultaneous parameter optimization
- **Relevance:** HIGH - Compartment models = our illness stages. Mechanistic vs empirical.

### 3. Flexible Approach for QSP Model Assessment
- 4 assessment areas: biology, implementation, simulation, robustness
- **Finding:** QSP models are complex/underconstrained - need tailored evaluation
- **Relevance:** MEDIUM - Framework for evaluating our sim models (biological plausibility, robustness)

### 4. Computational methods for QSP
- 3-step process: gather info → build model → calibrate/validate
- **Finding:** QSP provides insight into mechanisms, not just prediction
- **Relevance:** HIGH - Our approach: mechanistic insight + gameplay.

## Key QSP Concepts → Game Systems

1. **Compartment models** - states + transition rates. Applied: illness (Latent→Mild→Severe), but could apply to mood, reputation, resources, anything that flows between states.
2. **Virtual populations** - heterogeneous agents with parameter distributions. Applied: villagers with varied traits, not identical clones.
3. **Parameter uncertainty** - models have ranges, not fixed values. Applied: economic systems, social dynamics with realistic variability.
4. **Mechanistic insight** - understanding WHY, not just predicting. Applied: our emergent systems aim for mechanistic plausibility.
5. **Calibration to data** - tune model to match reality. Applied: balance tuning via sim runs.

## Non-Health Applications (Creative Filter opportunity)

QSP-style compartment models could simulate:
- **Reputation** - states (stranger→trusted→betrayed) with transition rates
- **Resource flows** - raw→processed→consumed→waste
- **Emotional states** - mood compartments (calm→stressed→breakdown)
- **Political influence** - apathy→interest→engagement→power

## Actions

### Backlog Items
- Virtual population framework for agent heterogeneity
- Parameter uncertainty in sim systems (ranges not fixed values)

### Design Proposal
- Apply compartment modeling to non-health domains (reputation, mood, resources) - "QSP for everything"

## Rating
**High** - QSP's compartment model + virtual populations directly applicable to agent simulation. Opens up non-health applications.
