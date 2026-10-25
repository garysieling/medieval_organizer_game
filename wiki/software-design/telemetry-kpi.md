# Open Telemetry & KPI Recording

**Category:** Architecture / Observability
**Source:** Backlog item — "open telemetry to record KPIs" for playtest observation. Implemented: `game2-core/src/infra/telemetry.rs`; consumed by `sim500`, `mega_sim` binaries.

## Description

OTel-style metrics + spans so every sim run produces a measurable record — the substrate that lets agents evaluate "did it work?" without a human watching. This is the project's answer to Gary's evidence standard: telemetry IS the evidence trail.

## What's implemented

- `Metric { name, value, timestamp, labels }` — `gauge(name, value)` records KPI points.
- `Span { name, start, end, attributes }` — `span(name)`/`end_span(idx)` time operations.
- `Telemetry` collector + `export_metrics()` for text dumps — the artifact a Data-Scientist tester or report consumes.

## KPIs the systems should emit

- **World:** population, births/deaths per age band, food stock, treasury
- **Economy:** price levels per good, trade volume, wage distribution
- **Psych/community:** stress compartment mean, relationship density, cohesion index
- **Religion:** congregation counts, radicalization precursors
- **Performance:** tick wall-time, entities/system, memory — for the Performance Engineer

## Design points

- Metrics are append-only facts (consistent with the Datomic save/load worldview) — a run = its metric series, diffable across runs.
- Label discipline: `system`, `region`, `scenario` labels so test runs are sliceable.
- Spans for the expensive paths (world gen, scheduler pass, save) feed perf regression detection — the Performance Engineer agent's raw material.

## Gaps

- Export is text-only; add JSON + a stable schema so agent tooling can diff runs programmatically.
- No rate/aggregation yet (mean/max per window) — keep aggregation out of the hot loop, do it at export.

## Rating

Boring and load-bearing — every testability claim in this wiki depends on it.
