# Dialogue research: Crusader Kings III

**Issue:** #1199 · **Sources:** CK3 Wiki/Paradox Wikis (Interactions, Schemes), CK3DB (Seduce/Befriend data), Neoseeker schemes guide.

CK3's "dialogue" is barely dialogue — it's a **verb menu of social schemes**: Sway, Befriend, Seduce, Send Gift, Fabricate Hook, Influence Ward. Each is a long-running project with success chance, secrecy, breaches, and agents — the conversation is the campaign. Its most interesting idea for us is that relationships are *projects with meters and exposure risk*, and that patient contact can reshape a person's traits. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Interactions as verbs** — a right-click menu of typed social moves with per-verb requirements, costs, and acceptance rules (often auto-accepted; the friction is in cost/conditions, not a tree).
- **Schemes as processes** — Befriend/Seduce/Sway/Hook/Abduct run over months: Success Chance accrues each phase; hostile schemes add Secrecy (vs. monthly Breaches), Advantage for execution, and Agents recruited to raise Potential.
- **Events inside schemes** — a scheme fires narrative events as it runs; the story rides the meter.
- **One scheme of each type at a time** — an explicit throttle on parallel social work.

## What drives variation

- **Traits gate and weight** — a Stubborn target resists; trait compatibility drives romance; Hideous penalizes seduction.
- **Rank distance** — higher-ranked targets are harder for lower-ranked schemers.
- **Stress** — acting against your own traits *costs* (a Greedy character paying a gift takes stress); three stress tiers end in breakdowns and coping traits.
- **Secrets and hooks** — a discovered secret becomes a Hook that compels acceptance; Fabricate Hook manufactures the leverage.

## Choices, relationships, consequences

- **Opinion per pair** — Sway +25, gifts, hooks; opinion gates everything downstream.
- **Typed relation statuses** — Friend, Lover, Soulmate, Rival: durable relation kinds reached through schemes, not just opinion numbers.
- **Exposure as the negative space** — a scheme's fun is the breach risk; discovery has its own consequences.
- **Trait transmission** — Influence Ward passes a personality trait to a ward over years: the mentor literally remakes the pupil.

## What it remembers / forgets

**Remembers:** every relation status, secrets held and known, hooks, ongoing schemes, stress, who tutored whom.

**Forgets:** any actual words — CK3 has no transcript to forget.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Mentor remakes the pupil** — traits passed by long contact (Influence Ward) | **want** | Our Mentor/Apprentice tie is a fixed label — it leaves no mark. → **#1239** |
| 2 | **Social work as a metered project** (Success accrues, phases, throttled) | **have, better** | The bond ladder IS the personal scheme — earned through talk and deeds rather than a clicked progress bar; ours can't be min-maxed from a menu |
| 3 | **Secrecy, breaches, exposure** (agents, breach counters, discovery) | **have** | The sign known/unknown, eyes-inside growing, informers, denouncement — our covert layer is the same shape played for real stakes |
| 4 | **Hooks: a secret held as leverage** | **half have** | Mysteries yield secrets the player can *deliver* for a payoff; a secret *held* as compulsion (the blackmail shape) isn't a mechanic — our informers hold the analogous leverage over the org, so the chair exists on both sides |
| 5 | **Typed relation statuses as durable kinds** (Friend/Lover/Rival) | **have** | `RelKind` inference does exactly this, emergently rather than declared |
| 6 | **Stress for acting against your own traits** | **reject for player; have for villagers** | No player psyche meter by design; villagers' InnerLife/grudges are the internal-cost layer |
| 7 | **Send Gift → opinion** | **reject** | Same as the Stardew call — wealth substituting for organizing work undercuts the premise |
| 8 | **Trait compatibility gating romance** | **have** | Traits already weight replies and courtship runs through routines/relations |
| 9 | **One-scheme-per-type throttle** | **have** | Talk/meeting cadence + bond-gated asks throttle naturally; #1232 (daily-greeting rule) sharpens it further |
| 10 | **Rank distance resists** (low can't scheme up easily) | **have** | The stage ladder + vibe probe do this socially rather than by title — a stranger can't propose to a stranger |

## What the pass confirms about our shape

CK3 abstracts the *project* of winning a person into meters; our game makes the same project out of talk, time, and deeds — the same shape, played instead of clicked. The piece genuinely missing is its slowest one: **the teacher marking the pupil**. Filed as #1239.
