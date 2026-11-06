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

`Act(i)` is one of the three action buttons (0 Talk, 1 Recruit, 2 Build);
`CtxAct(i)` the i-th context action the scene lists
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

```json client
{"AgentCmd":"FetchInput"}
```

`FetchInput` gets the raw good the bench is short of (`me.needs` in `Obs`):
the villager's own errand, which sends a kid or walks to where it's had.

Also: `"Idle"`, `"WorkDay"`, `"JoinOrg"`, `{"TalkTo":{"e":51}}`,
`{"BuyAt":{"shop":37,"good":"bread"}}`,
`{"Gather":{"good":"wood","x":40,"y":12}}`, `{"Press":{"prisoner":61}}`.
Entities (`e`, `seller`, `shop`...) are the numbers `Obs` reports.

### Pick (humans)

Take a card off the roster the `Welcome` dealt: it rewrites your
character - name, trade or childhood, purse, wants, and the figure you
draw as. Send it once, after the Welcome; the seat's default wanderer
stands if you never do.

```json client
{"Pick":3}
```

### Ping

```json client
{"Ping":7}
```

## Server -> client

### Welcome

`{"Welcome":{"you":87,"token":"9f3a1c...","model":{...},"roster":[...]}}`:
`you` is your character's entity, `token` your claim (keep it), `model` a
first frame (as in `Scene`, with no map), and `roster` the seed's premade
wanderers - `[{"name":"Sabyn","backstory":"...","work":"baker","kid":false,"look":3}, ...]` -
for a fresh seat's character picker. Older servers omit it; read it as
empty (no picker).

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
 "selected":false,"in_reach":true,"hungry":false,"sick":false,"act_mark":0,"bond":false,
 "find":"wystan heart helper","look":0}
```

The figure whose `e` is your `you` is you. Only what the seat can see is
here: no one's secret wants, no hidden sickness, no true strength of the
org (see `docs/architecture.md`, public vs secret).

`inspector` is the card for the current selection: `fig` (the portrait's
villager index, or `null` for a building or a grave), `kid` (draw the
portrait with a child's look, not the grown-up one `fig`'s bare index
would otherwise cycle to), `dead` and `died_day` (a grave portrait), and
`lines` (pre-formatted inspector text). Old servers never sent `kid`; it
defaults to `false` when absent.

`toasts` holds the banners announced at the top of the map, each a
`{"text":"...","at":[x,y]|null}`: `at` is where it happened, for a click
to jump the camera there, or `null` for a sky-wide omen with no spot to
visit (hail, drought, aurora...).

```
{"text":"A chimney fire!","at":[12.0,9.0]}
{"text":"A hailstorm batters the thatch","at":null}
```

`panel.social` is the seat's bond ledger: folk it has actually talked
with (`met`), split by how far each bond has climbed. Only conversations
count toward `met` - a barter or a shared meal alone doesn't.

```json
{"met":12,"acquaintances":5,"known":4,"followers":2,"coleaders":1}
```

Scenes from an older server may omit `social`; read it as all zeroes.

`panel.meeting` is the house-meeting plan being gathered, or absent:
`{"coming":["Willa","Bram"],"spreaders":[],"roles":["house: Aldric",
"notes: Corvin"]}` - the folk who've said yes, by name, so the seat can
watch the plan fill before it calls the meeting.

`snowballs` lists the snowballs in the air this frame, each `[x,y]` where
it has got to - empty outside a snowball fight. Older servers omit it;
read it as empty.

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
"the village is full") and the socket closes. After one it's a notice, not
an ending: "slow down" (you're past the message budget, and the excess was
dropped) or "seat gone".

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
