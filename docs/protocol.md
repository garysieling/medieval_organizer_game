# The game protocol

How a client talks to `game_server`: a window (`game_ui --connect`), a
browser, or a bot in any language. JSON text frames over one WebSocket.
The Rust types are `ClientMsg` / `ServerMsg` in
`game2-core/src/session/net.rs`; serde's externally tagged enums give the
shapes below.

> The examples in this file are tested. Every ```` ```json client ```` block
> must parse as a `ClientMsg`, every ```` ```json server ```` block as a
> `ServerMsg`, and every ```` ```json obs ```` block as an `AgentObs`
> (`game2-core/tests/protocol_doc.rs`). Change the protocol, and this file
> has to change with it.

## A session

1. Connect: `ws://host:8787`.
2. Send `Hello` as the first frame, within 10 seconds.
3. The server answers `Welcome` (your character, a claim token, a first
   frame) or `Denied` (and closes).
4. Then it streams: a **human** gets `Map` once and `Scene` about 10 times a
   second; an **agent** gets `Obs` about 10 times a second.
5. Send `Command`s (human) or `AgentCmd`s (agent) whenever you like; `Ping`
   for latency.
6. Reconnect with the token in `Hello` to take the same character back.

## Client -> server

### Hello

```json client
{"Hello":{"token":null,"kind":"Human","name":"Ada","secret":null}}
```

- `token`: the claim token from an earlier `Welcome` (32 hex digits, made
  by the server; a token the client invents claims nothing), or `null` for
  a new character. A known token always gets its character back, even when the
  village is full, and the older connection to that seat is closed: one
  character, one driver.
- `kind`: `"Human"` (a wanderer seat that sees `Scene`s and sends
  `Command`s) or `"Agent"` (a villager seat that sees `Obs` and sends
  `AgentCmd`s).
- `name`: shown on the roster, optional.
- `secret`: the server's join secret if it runs with one (`--join-secret`),
  else `null` or left out. Clients take it from `GAME2_JOIN_SECRET`.

An agent with a token can leave out what it doesn't need:

```json client
{"Hello":{"token":"6f1c0b2e9d4a7f3058e1c2b4a6d8e0f1","kind":"Agent","name":"bot-1"}}
```

### Command (humans)

The same inputs the window sends, one frame's worth each.

```json client
{"Command":{"Walk":[1.0,0.0]}}
```

```json client
{"Command":{"Act":0}}
```

```json client
{"Command":{"ClickMap":{"mx":400,"my":300,"cam_x":0,"cam_y":0,"zoom":1.0}}}
```

```json client
{"Command":{"TalkPick":1}}
```

```json client
{"Command":"CancelApproach"}
```

`Act(i)` is one of the five action buttons (0 Talk, 1 Recruit, 2 Invest,
3 Organize, 4 Build); `CtxAct(i)` the i-th context action the scene lists
in `ctx`; `Select(i)` picks a villager; `TalkPick(n)` answers mid-Talk
(`null` to take your leave); `Decide(i)` answers a founding call.
`TogglePause`/`ToggleSpeed` are ignored: the shared world runs at one pace.

### AgentCmd (agents)

Walks and errands for the agent's villager; each gets a `CmdDone`.

```json client
{"AgentCmd":{"GoTo":{"x":12,"y":30}}}
```

```json client
{"AgentCmd":"Eat"}
```

```json client
{"AgentCmd":{"BuyFrom":{"seller":17,"good":"bread","qty":2}}}
```

Also: `"Idle"`, `"WorkDay"`, `"JoinOrg"`, `{"TalkTo":{"e":51}}`,
`{"BuyAt":{"shop":37,"good":"bread"}}`,
`{"Gather":{"good":"wood","x":40,"y":12}}`, `{"Press":{"prisoner":61}}`.
Entities (`e`, `seller`, `shop`...) are the numbers `Obs` reports.

### Ping

```json client
{"Ping":7}
```

## Server -> client

### Welcome

`{"Welcome":{"you":87,"token":"9f3a1c...","model":{...}}}`: `you` is your
character's entity, `token` your claim (keep it), and `model` a first frame
(as in `Scene`, with no map).

### Map (humans)

`{"Map":{"width":245,"height":245,...}}`: the tile map, about 1 MB. Sent
after the `Welcome` and again only when the map changes. Scenes leave it
out. Keep the last one and put it into each scene before drawing (in Rust:
`SceneCache`).

### Scene (humans)

`{"Scene":{"tick":600,"people":[...],"panel":{...},...}}`: everything your
seat may see this frame, a `UiModel`, about 22 KB. `map` is empty (see
`Map`). One person in `people` looks like:

```
{"e":57,"kind":{"Villager":8},"pos":[11.4,6.9],"label":"Wystan","dead":false,"kid":false,
 "selected":false,"in_reach":true,"hungry":false,"sick":false,"act_mark":0,"bond":false}
```

The figure whose `e` is your `you` is you. Only what the seat can see is
here: no one's secret wants, no hidden sickness, no true strength of the
org (see `docs/architecture.md`, public vs secret).

### Obs (agents)

What an agent's villager knows: itself, who and what is near, its satchel,
and (in `me`) its own trade, home, bench and wants.

```json obs
{"tick":0,"you":49,"pos":[15.38,11.0],"gold":19.4,"hunger":100.0,"busy":"Idle",
 "near":[{"e":74,"name":"Greta","dist":0.7,"dead":false}],
 "near_buildings":[{"e":37,"name":"the bakery","kind":"shop","dist":2.0}],
 "satchel":[["bread",2]],"walking":false,"held":false,"orgs":[],"campaigns":[],"burning":[],
 "me":{"trade":"baker","level":"Journeyman","home":15,"bench":37,"bench_stock":[["bread",4]],
       "needs":"grain","wants":["Joy","BetterTools"]}}
```

`me` is what the villager knows of themself: their trade and its rung, their
home and bench (entities, as `near_buildings` and `AgentCmd` use them), the
bench's shelf, the raw good the trade is short of (`needs`: go and get it),
and the wants they haven't met. It's only their own; nobody else's wants
appear.

### CmdDone, Denied, Pong

```json server
{"CmdDone":{"ok":false,"note":"nothing to eat"}}
```

```json server
{"Denied":{"reason":"the village is full"}}
```

```json server
{"Pong":7}
```

`Denied` before a `Welcome` means you weren't let in ("wrong join secret",
"the village is full") and the socket closes. After one it means your seat
is gone.

## Limits

- `Hello` within 10s of connecting; at most 256 sockets open at once.
- At most 240 messages a second per connection: past that they're dropped
  (not queued), and one `Denied {"reason":"slow down"}` a second says so.
- `--max-seats` (default 64) new characters per server; reclaims don't count.
- Scenes, observations and maps are snapshots: a slow client is sent the
  newest, never a backlog. Replies (`Welcome`, `CmdDone`, `Pong`) are never
  dropped.

## The probe (tests only)

A separate, read-only socket (`--probe-port`, localhost only, optional
`--probe-token` sent as the first frame) streams the omniscient
`ProbeView`. It's for tests and tools, never for players or agents.
