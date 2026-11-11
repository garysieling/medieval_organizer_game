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

```json client
{"Command":"Unselect"}
```

`Act(i)` is one of the three action buttons (0 Talk, 1 Recruit, 2 Build);
`CtxAct(i)` the i-th context action the scene lists
in `ctx`; `Select(i)` picks a villager; `Unselect` lets the pick go - the
ring, the walk-over and the talk under way; `TalkPick(n)` answers mid-Talk
(`null` to take your leave); `Decide(i)` answers a founding call.
`TogglePause` is ignored: the shared world runs at one pace.

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

Use `Context` for a villager-scale contextual action. The optional target is
one of the nearby villager entity IDs from `Obs.near`; actions such as Douse
and Tame use the agent's position and don't need a target.

```json client
{"AgentCmd":{"Context":{"action":"Douse","target":null}}}
```

Available actions are `TakeJob`, `Douse`, `Tame`, `Recruit`, `Matchmake`,
`Errand`, `PairUp`, `ShareFindings`, `PickCaps`, `Snowballs`, `PetCat`, and
`Pact`. The server checks the same context availability and blockers as the
player menu. Campaign leadership actions remain player-only; the agent
surface does not include meeting, march, delegation, strike, or hall controls.

Also: `"Idle"`, `"WorkDay"`, `"JoinOrg"`, `{"TalkTo":{"e":51}}`,
`{"BuyAt":{"shop":37,"good":"bread"}}`,
`{"Gather":{"good":"wood","x":40,"y":12}}`, `{"Press":{"prisoner":61}}`.
Entities (`e`, `seller`, `shop`...) are the numbers `Obs` reports.

`"TakeNotice"` accepts the market board's oldest posted job where it
hangs - the asker needn't be near. It only works standing at the board
(the `board` position in `Obs`), and only while a job is posted.

### Pick (humans)

Take a card off the roster the `Welcome` dealt: it rewrites your
character - name, trade or childhood, purse, wants, and the figure you
draw as. Send it once, after the Welcome; the seat's default wanderer
stands if you never do.

```json client
{"Pick":3}
```

Or keep the default wanderer - still a decision, so a reclaim doesn't
offer the cards again:

```json client
"DeclinePick"
```

Both answer a `CmdDone` (`{"ok":true,"note":"a card picked"}` or
`"the wanderer kept"`) - a client that wants its choice safely in
before it leaves (a fast Escape-then-reconnect) waits for it.

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
empty (no picker). `intro` is the seed's stage-setting facts -
`["The market towns of ...", ...]` - for the prologue card after the
picker. Older servers omit it; read it as empty (no card).

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
 "find":"wystan heart helper","look":0,"person":"tax-collector","sign":null,"mounted":false,"stale":0.0}
```

`person` names the figure their posting or trade wears ("baker",
"tax-collector", "guard-spear") where the generated people art covers
it; absent for the uncovered trades and on older servers. `stale` is
how long since this seat spoke with them, 0 fresh -> 1 long
silent (strangers stay 0); clients may grey the name by it. `sign` is
the picket-board design (0 wooden hand-lettered, 1 printed
placard) while they gather for a protest or march, else null.
`mounted` is true while a knight rides his horse on a country journey -
the figure drawn is the mounted one. On the dead, `epitaph` is the
grave's hover line ("died day N, of ...") or absent, and `buried` marks
those already laid in the churchyard (they wear the tombstone, not the
lying-in mark). Both are absent from older servers.

The figure whose `e` is your `you` is you. Only what the seat can see is
here: no one's secret wants, no hidden sickness, no true strength of the
org (see `docs/architecture.md`, public vs secret).

`bonds` lists the seat's warmth for each figure it knows, as
`[entity, hearts]` pairs - hearts run 0 to 5 in half-heart steps, the
same number the inspector's meter shows, and the client floats them over
each figure's head. Only bonds with warmth worth showing are listed;
the field is absent from older servers.

`org_members` lists the entities on the viewing seat's org roster (the
seat's own character excluded) - the client marks a member's name tag
where their hearts show. The field is absent from older servers.

`player_satchel` lists what the seat's own character carries, as
`[good, count]` pairs - the client draws it as a strip under the HUD.
`player_hunger` is their belly, 0 to 100; under 40 the strip marks them
hungry. Both are absent from older servers (empty bag, full belly).

`inspector` is the card for the current selection: `fig` (the portrait's
villager index, or `null` for a building or a grave), `kid` (draw the
portrait with a child's look, not the grown-up one `fig`'s bare index
would otherwise cycle to), `dead` and `died_day` (a grave portrait),
`hearts` (the seat's friendship with the villager, 0 to 5 in half-heart
steps - `null` on a building card or a grave, and absent from older
servers, which read as no meter), and `lines` (pre-formatted inspector
text). Old servers never sent `kid`; it defaults to `false` when absent.

`toasts` holds the banners announced at the top of the map, each a
`{"text":"...","at":[x,y]|null,"born":1234}`: `at` is where it happened,
for a click to jump the camera there, or `null` for a sky-wide omen with
no spot to visit (hail, drought, aurora...). `born` is the model frame
it was announced on - a client ages each banner itself (about five
seconds on screen); an older server omits it, which reads as just
arrived.

```
{"text":"A chimney fire!","at":[12.0,9.0]}
{"text":"A hailstorm batters the thatch","at":null}
```

`log` holds the event log's visible window, newest first. Each entry is
`{"text":"...","at":[x,y],"who":8}`: `at` is where it happened,
`who` the villager index it was about - either may be absent, and an
entry with neither is a plain line. A click on a positioned entry jumps
the camera there; one naming a villager also selects them. Older
servers send bare strings in `log`; those read as unclickable lines.

```
{"text":"t640: Aldric died.","at":[11.0,7.5],"who":3}
{"text":"t641: drizzle begins"}
```

`ctx_hint` says what pressing each context action does, one entry per
`ctx` index (`null` when nothing needs explaining) - the hover line
that doesn't bar the press, unlike `ctx_blocked` which both greys and
disables. Older servers omit it; read it as no hints.

`focus` is the sim's camera ask, or absent: `[x, y, frame]` - where to
look and the frame it was asked on, so a client can answer each ask
once (an introduction's name, say). Older servers omit it; read it as
no ask pending.

`player_event` is the sim's civic-event camera ask, or absent - the
`EventFocusV` an event the seat's own part of, or the village's own
gathering, is running:
`{"epoch":1,"kind":0,"pos":[x,y],"extent":12.0,"moving":true}` - `kind`
is 0 a march, 1 a meeting, 2 a rite or celebration; `pos` the event's
live centre in tiles;
`extent` how wide it reaches in tiles (so a client can zoom out to
frame it); `epoch` a counter that names the event, the same while it
runs and new for the next, so a client jumps once and tracks after;
`moving` whether the centre is still worth easing toward. Older
servers omit it; read it as no event pending.

`buildings` lists the village's built structures this frame, each a
`BldgV` - `{"x":12.0,"y":9.0,"sprite":...,"risen":1.0,"derelict":false,
"smoke":true}`: `risen` runs 0..1 while a building goes up, and `smoke`
marks chimney smoke on the cottages nearest the market. `sprite` picks
the look the client should draw:

```
"Church"                  - a single fixed look (a unit variant)
{"House":3}               - index 3 into the cottage set (wraps round)
{"Rowhouse":2}            - a terraced street sharing the one cottage
{"Barracks":1}            - a company town's workers' quarters
"Mansion"                 - a company town's manager's row
{"Work":["Bakery",0]}     - a workplace's own look, plus its variant
```

The unit-look sprites are `Mansion`, `Blacksmith`, `Church`,
`Guildhall`, `RivalHouse`, `RivalWoodhouse` and `CastleKeep`; the
indexed ones are `House`, `Rowhouse` and `Barracks`. `Work`'s looks are
`Tavern`, `Bakery`, `Granary`, `WatchHouse`, `Fold`, `Apiary`, `Cellar`,
`Workshop`, `Factory`, `Tailor`, `Glazier`, `Printshop`, `MasonsYard`,
`Luthier` and `Farmhouse` - the sim names a look and the client's art
pack resolves it to pixels, standing a cottage in where a look has no
art yet. Older servers never send the company-town sprites (`Mansion`,
`Rowhouse`, `Barracks`, `Factory`); a client that doesn't know a sprite
should draw a cottage and move on.

`well` and `hall` are the village well and meeting hall on the grass
beside the market square - `[x,y]` tiles or absent (older servers omit
them; no prop drawn).

`castle_keeps` (`[x,y]` list) and `settlement_houses` (`[x,y,sprite]`
list, the same sprite shapes) carry remote scenery - towns and castles
out in the country that a seat can't select.

`sites` carries the same country's names for the minimap: `x`, `y`,
`name`, `kind` (`Town`, `Castle`, `Farm` or `Monastery`), and `keeper`
- the keep's first bed's name, or null when the bed stands empty.
`head` names the site's styled head and liege ("Earl Godric"), or null
when the site has no head; `treasury` is its gold - tribute in, gifts
and the crown's cut out. Older servers omit `sites`, `head` or
`treasury`; a client should treat a missing `sites` as empty, a missing
`head` as no head, and a missing `treasury` as empty.

```json
{"x":44.0,"y":18.0,"name":"Oakhall","kind":"Castle","keeper":"Sir Brandr","head":"Earl Godric","treasury":120.0}
```

`panel.social` is the seat's bond ledger: folk it has actually talked
with (`met`), split by how far each bond has climbed. Only conversations
count toward `met` - a barter or a shared meal alone doesn't.

```json
{"met":12,"acquaintances":5,"known":4,"followers":2,"coleaders":1}
```

Scenes from an older server may omit `social`; read it as all zeroes.

`panel.leader` names the org's head for the status line - `"you"` when
the seat leads its own, a villager's name when the bond's passed to
another. Older servers omit it or send null; a client should draw no
"led by" then.

`panel.org_status` carries the powers' standing words on the org, as
plain short lines - "grant: the taxes", "part grant: the taxes",
"N knights", "ban >N". Older servers send nothing; treat an absent
field as an empty list.

`panel.power` names the org's own muscle in one line - "muscle N.N ·
turnout N.N · fresh|settled|going soft": its organized-power capacity,
the seat's turnout craft, and whether it's been used. Older servers
send nothing; treat an absent field as no read.

`log_dropped` counts how many log lines have rolled out the top - the
log's a rolling window. Zero or an absent field means the top's the
beginning; the client's last log row marks the count when it isn't.

`food_yield` carries the season's weight on the fields - the climate's
food multiplier (about 1.1 spring to 0.7 winter, before rain and the
year's drift). Zero from an older server means no read; a client
should draw no lean-word then. `harsh_winter` is the every-other-winter
flag: while it runs the snow keeps falling and the store thins.

`panel.campaign` is the org's settled fight, or absent before the vote:
`{"issue","framing","dissenters","seats","shadow","state","resolves",
"due","cadence_days","assembly"}` - the issue the vote picked, how it's framed, the
losers left grumbling, the three seats' keepers ("turnout: Aldric" /
"notes: (empty)"), the resolutions passed, whether a meeting's due, and
the days between them. `shadow` names the member whose ties drive the
room when no live coordinator holds the seat - `null` while the formal
structure holds it. `state` is the cause's posture in a word - "won
through", "their demand's refused", "crackdown's on", "the room's
split", "the room consents" and such - empty while nothing's settled.
Older servers omit it; read it as empty. `assembly` is the
accountability assembly's standing - "assembly called - the agenda's
posted" while an agenda waits, else the pledges' count ("2 pledges
broke - 1 pledge stands"); empty before the institution's called or
when nothing's on record. Older servers omit it; read it as empty.

`panel.meeting` is the house-meeting plan being gathered, or absent:
`{"coming":["Willa","Bram"],"spreaders":[],"roles":["house: Aldric",
"notes: Corvin"]}` - the folk who've said yes, by name, so the seat can
watch the plan fill before it calls the meeting.

`panel.mystery` is the village mystery the seat is piecing together, or
absent: `{title, heard, total, asker_name, asker?, asker_at?}` - the
question, the count of whispers heard, and who's asking, carried as data
so the client can make the asker's name a button (select + camera jump).
Older servers send the whole line as a bare string instead
(`"Mystery: ... - 3/5 whispers (Matilda wants to know)"`); it decodes
with an empty `title`, which the client draws verbatim with nothing
clickable. Servers that predate the field omit it; read it as no
mystery under way.

`snowballs` lists the snowballs in the air this frame, each `[x,y]` where
it has got to - empty outside a snowball fight. Older servers omit it;
read it as empty.

`tech` is the village's research tree: each entry `{"name","era",
"prereqs","cost","unlocks","discovered","progress"}` - what's been
worked out and how far the next art has come. Older servers omit it;
read it as empty.

`flyers` lists where the cause's flyers are posted, each `[x,y]` a tile
centre - empty until the org has passed the hat and bought some.
Older servers omit it; read it as empty.

`trains` lists the freight on the roads, each `[x,y]` where a cart,
levy, remit, tribute or gift train's driver stands - a wagon follows
them. Older servers omit it; read it as empty.

`graveyard` lists the churchyard's grave plots as `[x,y]` tile coords,
in burial order - the worn ground under them. Empty on maps with no
church and from older servers; read it as empty.

`lychgate` is the churchyard gate's `[x,y]` tile coord, or `null` when
the graveyard planning found no spot (or on older servers, which omit
it - read a missing field as `null`).

### Obs (agents)

What an agent's villager knows: itself, who and what is near, its satchel,
and (in `me`) its own trade, home, bench and wants.

```json obs
{"tick":0,"you":49,"pos":[15.38,11.0],"gold":19.4,"hunger":100.0,"busy":"Idle",
 "near":[{"e":74,"name":"Greta","dist":0.7,"dead":false,"job_offer":null}],
 "near_animals":[],
 "near_buildings":[{"e":37,"name":"the bakery","kind":"shop","dist":2.0}],
 "satchel":[["bread",2]],"walking":false,"held":false,"orgs":[],"campaigns":[],"burning":[],
 "me":{"trade":"baker","level":"Journeyman","home":15,"bench":37,"bench_stock":[["bread",4]],
       "needs":"grain","wants":["Joy","BetterTools"]},
 "notices":[["quest","Greta asks for help - the board carries it"]],"board":[40.0,20.0]}
```

`me` is what the villager knows of themself: their trade and its rung, their
home and bench (entities, as `near_buildings` and `AgentCmd` use them), the
bench's shelf, the raw good the trade is short of (`needs`: go and get it),
and the wants they haven't met. It's only their own; nobody else's wants
appear.

People in `near` include `job_offer` when they have a current odd job to take.
`near_animals` lists nearby flock and yard animals with their position and
whether the agent can tame them.

`notices` holds the market board's live postings as `(kind, text)` pairs -
`quest`, `writ`, `feast`, `ban` - but only while the villager stands close
enough to read it. `board` is where it stands, always, so walking there is a
decision rather than a hunt; `TakeNotice` accepts the oldest posted job at it.

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
