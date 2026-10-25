# Lit Review - Sprint 4: Multiplayer/Persistent Worlds

**Date:** 2026-10-25
**Domain:** Backend Architecture / Persistent Worlds
**Sources:** Supercraft multiplayer patterns, Asobi world server, Photon MMO, Klotho sync

## Sources

### 1. Multiplayer Architecture Patterns (Supercraft)
- **4 decisions:** Authority (who decides), Transport (how packets move), Discovery (how players connect), Durable state (where it survives)
- **Key insight:** Pick trust model first; transport/discovery/persistence can change independently
- **Relevance:** CRITICAL - for our turn-based game, backend-authoritative async is the right pattern

### 2. Backend-Authoritative Async Sessions
- **Pattern:** Turn-based/strategy games validate actions in backend code, persist state, notify participants
- **No realtime sim needed:** no 30/60Hz server, just action validation + persistence
- **Relevance:** CRITICAL - exactly matches our game: villagers, economy, turns, not twitch combat

### 3. Asobi World Server (Erlang)
- **Zone partitioning:** world split into grid zones, each a process, parallel tick sim
- **Interest management:** players only get updates for nearby zones
- **Supervision tree:** world -> zones -> entities, per-node snapshot writer
- **Relevance:** HIGH - for scale: partition world into zones, parallelize tick sim, interest-based updates

### 4. Photon MMO Backend
- **Seamless across machines:** simulation spread across nodes, transitions invisible to player
- **Server-authoritative + prediction:** clients predict locally, server is authority
- **Replication:** only nearby objects, only changed fields, 30Hz snapshots
- **Relevance:** MEDIUM - we don't need realtime yet, but prediction/interest management apply

### 5. Klotho Synchronization Design
- **Two chains:** Verified (confirmed inputs, authoritative, immutable) vs Predicted (guessed inputs, rollback-able)
- **Rollback:** predicted state rewound when real input arrives
- **Relevance:** MEDIUM - for async multiplayer, two-chain model lets players see immediate feedback

## Key Insights

1. **Backend-authoritative async is the right pattern** - turn-based strategy doesn't need realtime server, just validate actions + persist + notify
2. **4 decisions are independent** - authority, transport, discovery, persistence can change separately
3. **Zone partitioning scales** - split world into zones, parallelize tick, interest-based updates
4. **Two-chain model** - verified vs predicted state, rollback for async multiplayer
5. **Persistent state is separate** - durable state lives in its own service (DB/leaderboard), not the sim

## Design Proposals

- **Backend-authoritative:** server validates player actions, persists world state, notifies other players
- **Async turn-based:** no realtime server needed - actions validated on submit, world ticks on schedule
- **Zone partitioning:** world -> zones -> entities, parallel tick per zone, interest-based updates
- **Two-chain:** local prediction for UI responsiveness, server verification for authority
- **Persistent state:** separate service for durable world/player data (Datomic-style history)
- **Sister can play:** backend API + lightweight client, she joins an existing world

## Rating
**Critical** - Backend-authoritative async is the architecture that fits our game (turn-based, sim-heavy, not twitch). Zone partitioning + two-chain + persistent state service = scalable multiplayer without realtime overhead.
