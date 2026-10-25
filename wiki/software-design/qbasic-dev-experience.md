# QBasic-Style Simple Dev Experience

**Category:** Architecture / Dev Experience
**Source:** Backlog item (High) — "QBasic-style simple dev experience: low-friction iteration."
**Date:** 2026-10-25

## Description

The aspiration: a development loop as immediate as QBasic — type, run, see. No build ceremony, no dependency hell, no IDE worship; the codebase invites tinkering.

## What made QBasic's DX work

- **Instant run** — type code, press F5, it runs; no compilation ceremony
- **Immediate feedback** — graphics output right there; the screen was the REPL
- **Everything in one place** — no project files, no build config, no toolchain decisions
- **Legible errors** — errors pointed at lines, in plain language
- **Tinkerability** — change a number, re-run, see what happens

## What that means for game2's DX

- **Cargo, not complexity** — `cargo run` is the F5; one-command builds, no multi-tool setup, no codegen dance
- **Hot reload is the QBasic F5** — change a mod, reload it, see the effect without restarting the world (`hot-reload.md`) — the closest thing to QBasic's immediacy in a systems language
- **BMP/terminal output** — the sim renders to BMP files/terminal: the "screen" is right there, no engine editor needed
- **Small, legible modules** — every system is a plain Rust module with a contract; a dev can read one file and understand one system (`modular-architecture-everything-is-a-mod.md`)
- **Telemetry as print** — instead of `PRINT X`, emit a metric; the evidence is the debug print (`telemetry-kpi.md`)

## Where it intentionally diverges

- **Not *no* safety** — QBasic let you shoot yourself; Rust prevents the foot-gun. The goal is QBasic's *immediacy* with Rust's *correctness*
- **Agents are the tinkerers** — much of the DX is for AI agents: clear module boundaries, contract-driven interfaces, mechanical verification — agent-legible code IS the modern QBasic DX

## Friction budget

- Anything that takes >10 seconds to go from "I changed this" to "I see the effect" is a DX tax: compile time, startup, asset processing — worth policing
- Keep binaries small and focused: `sim500`, `mega_sim`, `agent_task` — run *the thing* not *the everything*

## Rating

An aspiration, not a feature — the measure of whether the codebase is pleasant to live in.
