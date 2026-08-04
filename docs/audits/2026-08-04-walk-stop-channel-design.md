# A stop channel for the walk loop — design pass

**2026-08-04. Design pass only; NO CODE CHANGED.** Nothing here is ratified. Every mechanism is
provenance-marked **[user-ruled]** / **[corpus]** (stated in a ratified doc) / **[assistant-proposed]**
(a hypothesis until the user rules it).

**The asking documents:** `CLAUDE.md` § Agent walks (*"the walk loop has NO STOP CHANNEL"*),
`ROADMAP.md:3256-3265` § Observed, `journal/0148-*.md` § *The station-keeping finding*.

---

## The recommendation, in five sentences

Add **one registry command, `dc:character/halt_all`** — zero every character's move intent in one
call — and **bind it to a key in the client shell**; the command reaches the driver over MCP, and
because the console and both MCP tool lists are *generated* from the registry (`docs/API.md`
principle 5) the observer gets it in the in-game console for free, with the keybind as the
sub-second path that typing is not. **Do not change what a move intent means**: bounded intent is a
sim-primitive commitment that reaches every controller a body will ever have, it has no consumer
outside this walk problem, and it would be a *second scheduling mechanism beside `target_tick`*,
which the host already implements and no caller can express. **Reject the leash radius outright** —
a tether is NPC steering policy, two good packs would answer it differently, and it fails § 0b's
test as engine work. **Keep the terrain answer as the primary one**: the user's wall was correct,
not a workaround for a missing verb, and `CLAUDE.md` § Agent walks should gain a fourth
station-framing rule saying so — a tool that lets a station be built badly is worse than the wall.
**The honest scale of the whole thing is ~20 lines of engine plus a keybind**, and its first
visible effect will be to expose `stubs.md` #43, the stop transition nobody owns.

---

## 1. Priors — what the corpus already held

Swept: the full `.md` corpus for *leash, tether, freeze, halt, stop, bounded, intent, latency,
round-trip, target_tick, console*; plus `crates/` for the code each claim describes. **Six of the
findings below pre-date this pass and demote most of what I would otherwise have "observed".**

| # | prior | where | provenance |
|---|---|---|---|
| **P-1** | **A freeze mechanism already exists and already ships.** *"A session disconnect zeroes the character's move intent; the body stands where it was left."* | `docs/API.md:273-277` — **DECIDED 2026-07-19**; implemented `crates/dc-client/src/authority.rs:583-607` (`freeze_character`), fired from `crates/dc-client/src/mcp_character.rs:114-123` (`SessionEnd::drop`) via `BridgeRequest::CharacterFreeze` (`authority.rs:1053-1057`) | **[user-ruled]** |
| **P-2** | Freeze was deliberately built as **a zero move-intent command on the ordinary rail**, not as body-side behaviour: *"it rides the same receipted, tick-quantized rail as any controller verb — the replay identity is unchanged"*; and *"freeze is not a body-side behavior, it is the last thing the controller says."* | `authority.rs:585-589`; `journal/0008-the-geology-becomes-ground.md:123-135` | **[corpus]** |
| **P-3** | **The observer ALREADY HAS A CHANNEL, and the walk doctrine never says so.** The in-game dev console (**press T**) *"exposes the ENTIRE dc-api command/query surface"*, generated from the registry, with autocomplete — so the user standing in-world can already type `character_set_move_intent` with `speed=0`. It is registered unconditionally (`app.rs:342`), no flag. **`CLAUDE.md` § Agent walks does not mention the console once**, and neither does `journal/0148` or the ROADMAP entry — all three describe the observer as having *"no channel at all."* | `crates/dc-client/src/console/mod.rs:1-24`, `:342` in `app.rs`; absence verified by grep over `CLAUDE.md`, `ROADMAP.md` § Observed, `journal/0147`, `journal/0148` | **[corpus]**, and the absence is a **doc-topology defect** |
| **P-4** | **The host already schedules commands at a future tick** — `target_tick: Option<Tick>`, *"default: next tick boundary"*, queued and released when due. **No caller can express it:** every MCP and console envelope hardcodes `target_tick: None`. | Host: `crates/dc-api/src/host.rs:658-676` and `:684-693`. Type: `crates/dc-api/src/envelope.rs:72`. Doc: `docs/API.md:47`. Hardcoded `None`: `crates/dc-mcp-dev/src/lib.rs:90`, `crates/dc-client/src/authority.rs:618`, `crates/dc-api/src/schema.rs:1054`, `:1166` | **[corpus]** — and an unindexed **A-4** ("built machinery with no consumer"): it is **not** in `spines.md` § 3 |
| **P-5** | **The tool list is never hand-written.** *"Adding a command to the registry adds the MCP tool"* — and the console's *"command table, the completions, the help text, and the argument→JSON assembly are all built by walking the registry's schemas."* One new registry command therefore lands on **three** surfaces at once. | `docs/API.md` principle 5; `crates/dc-mcp-dev/src/lib.rs:4-8`, `:44-68`; `crates/dc-client/src/console/mod.rs:5-13` | **[corpus]** |
| **P-6** | **A channel HOLDS until explicitly released**, and *"no hold timer: decay would be a tuning constant nobody ratified."* | `docs/design/bodies.md:504-517` — **DECIDED 2026-08-02** | **[user-ruled]** |
| **P-7** | **A stop is already known to be visually broken.** `swing_gain` (`stubs.md` #43) stands in for a **stop transition** nothing owns: *"a body stopping mid-swing freezes with a foot in the air."* `posture-gait.md` § 4 explicitly does not design it. | `docs/dependency-graph.md:167`; `docs/design/stubs.md:1708`; `ROADMAP.md:4851`; `journal/0147:71` | **[corpus]** |
| **P-8** | The corpus's existing answer is **procedural, and it is already written down**: *"prefer intents that terminate safely — drive toward the observer, or into open ground, never toward an edge"*, and station-framing rule 2, *"drive the subject TOWARD the camera so framing improves as it moves."* | `CLAUDE.md:288-301` (§ Agent walks, greenlit fingerprint 2026-08-03) | **[user-ruled]** (greenlit) |
| **P-9** | Bio/eco/socia — anything that would give an NPC a home range or a tether — **is ON HOLD and the gate is a user call.** *"we do NOT have any form of evo/socia/civ modeling even at the design stage."* | `CLAUDE.md` § Conventions, *existence is not standing*; `docs/dependency-graph.md` E8 (*"Gated on bio/eco, a USER call"*) | **[user-ruled]** |
| **P-10** | Zero prior hits for *leash* or *tether* anywhere in the corpus outside the three copies of the same 2026-08-03 note (`CLAUDE.md:300`, `ROADMAP.md:3263`, `journal/0148:121`). **This thread has no design history at all** — the priors above are all adjacent machinery, not prior attempts. | corpus grep | — |

**What P-1 + P-3 + P-4 + P-5 do to the problem statement.** The board's framing — *"none designed",
"the observer has no channel at all"* — is **too pessimistic by a wide margin**. Three of the four
pieces are already in the tree: the *act* of freezing (P-1), the *dispatch path* that would carry it
to a driver, a console and a keybind alike (P-2, P-5), and a *scheduling* mechanism if bounding is
ever wanted (P-4). What is missing is a **fan-out** (one call, all characters) and a **fast input**
(a key, not typing). That is a much smaller thing than "design a stop channel."

---

## 2. What the code actually does today — verified at source

- **Intent is a persistent latch, and the payload doc says so.** *"Persists until countermanded"* —
  `crates/dc-api/src/payload.rs:251-263`. It writes two fields and nothing else:
  `character.input.move_dir = (p.dx, p.dz); character.input.speed = p.speed.clamp(0.0, 1.0)`
  (`crates/dc-api/src/host.rs:1170-1183`).
- **`CharacterInput` carries no expiry, no anchor, no budget** — `{ move_dir, speed, jump }`,
  `crates/dc-api/src/character.rs:131-139`. Any bound is new state.
- **`step_character` re-reads the latch every tick** and derives velocity straight from it, with no
  inertia: `crates/dc-api/src/character.rs:296-320`. Characters step **after** the tick's commands
  apply, *"so a controller verb takes effect on the tick it lands on"* (`host.rs:740-743`,
  `:746-761`).
- **The speed of the problem, measured from the constants:** `walk_speed_m_s: 4.5`
  (`crates/dc-api/src/character.rs:47`), and the client keeps the default — it overrides only
  `voxel_size_m` and `tick_dt_s` (`crates/dc-client/src/authority.rs:247-251`). Tick is
  `HOST_TICK_DT = 1.0/20.0` (`authority.rs:114`). So at `speed: 1.0`:
  - **1 tick of latency = 0.225 m**
  - **1 second of latency = 4.5 m**
  - **4 MCP round-trips (`CLAUDE.md`'s measured figure) at ~1.5 s each = ~27 m** — which is why the
    0148 walkers left frame, and it is ~1.6× the 0148 station's own 63 m runway per shutter cycle.
- **A stop already zeroes the right things, with no side effects to design.** With `speed == 0`,
  `travel_yaw` is `None` (`character.rs:307-311`), so both the unheld gaze *and* the target trunk
  facing **hold their last heading** (`character.rs:313-330`). **A halted body keeps facing where it
  walked** — exactly what a station wants, with no extra rule.
- **The dev surface's reply blocks until the receipt materializes.** A submitted command's oneshot
  is parked in `pending` (`authority.rs:698`) and only answered on the tick that produces its
  receipt (`authority.rs:865-886`). This is the property that prices option D below.
- **Capability shape for a fan-out already exists as a template.** `Requirement::CharacterControl(String)`
  is satisfied by `Grant::CharacterControl { character: None }` (the broad dev/parent form) or by an
  exact name (`crates/dc-api/src/capability.rs:212-214`, `:29-37`). `WorldReadAnywhere` is the exact
  precedent for an "any/all" requirement satisfied only by the unscoped grant
  (`capability.rs:74-75`, `:207`). An attached character session (7778) holds a token attenuated to
  `Some(name)` (`authority.rs:645-652`) and therefore **structurally cannot** halt anyone else.
- **The character surface filters its tool list by a hand-maintained exception.**
  `tool.name.starts_with("character_") && tool.name != "character_spawn_character"`
  (`crates/dc-client/src/mcp_character.rs:45`), with a pinned list test at `:440-462` whose own
  comment says *"this hand-maintained list is the one place a new character verb must be restated,
  which is why it is the thing that broke."* Any new `character_*` command pays this line.

---

## 3. The options, priced

Costs are line-counts and touched-file counts read off the sites above, not guesses. "Owner" uses
`docs/dependency-graph.md` § 0a/§ 0b vocabulary.

| | option | what it costs | what it changes about the sim | owner | **what it does NOT solve** |
|---|---|---|---|---|---|
| **A** | **`dc:character/halt_all` — a registry command that zeroes every character's move intent** **[assistant-proposed]** | New `Payload` variant + schema row + host arm (a loop over `self.characters` doing what `host.rs:1180-1182` does for one) + one `Requirement::CharacterControlAnywhere` mirroring `WorldReadAnywhere` (`capability.rs:74`, `:207`) + the `mcp_character.rs:45` exclusion and its pinned-list test. **~40 lines across 5 files, one of them a test.** No new concept anywhere. | **Nothing.** It is N applications of an existing command's effect, on the existing rail, at a tick boundary. Replay identity unchanged (P-2's argument, unmodified). | **ENGINE capability, dev-gated.** It is dc-api registry surface reachable only by an unscoped `CharacterControl` grant — i.e. the dev surface and the player shell, never an attached companion session. | The **observer's latency** (still one MCP round-trip for the driver ≈ 4.5 m). Nothing about *where* the body already is when you halt it. Does not stop a body **already airborne** — gravity and the swept move still run. And it makes **`stubs.md` #43 maximally visible**: every halt is a mid-swing stop. |
| **A′** | **A keybind in the client shell that submits A** **[assistant-proposed]** | One key arm beside the existing input systems + a `submit_player` call (`authority.rs:612-622`). **~10 lines, 1 file.** Free keys today: `H`, `X`, `Z`, `Q`, `E`, `R`, `C`, `V`, `B`, `P`, `F4` (bound: `WASD`, `Space`, `Shift`, `T`, `F`, `G`, `F3`, `2/3/4`, `Tab`, `Esc`, arrows, `Enter`, `PgUp/PgDn`, `Backspace`). **Which key is a user pick.** | Nothing — it is a second caller of A on the same bridge, the way the console already is (`console/mod.rs:14-20`, *"a third consumer of that one bridge, not a second dispatch path"*). | **CLIENT SHELL** — outside the registry, like `client_screenshot` / `client_player_pose_*` (`console/mod.rs:9-13`). | Nothing about the driver. Requires the user's hands to be on the keyboard and the console **closed**. Adds a key that can be hit by accident during a walk. |
| **B** | **Bounded intent** — `SetMoveIntent` gains a duration/distance/target that expires **[assistant-proposed, and I recommend against]** | New wire field (postcard-positional, appended, `serde(default)`) + new `CharacterInput` state + a decrement in `step_character` + the expiry's interaction with jump, airborne, blocked-by-terrain, and posture. Touches the **replay identity** and every future controller. **The design conversation is bigger than the code.** | **It changes what an intent MEANS**, from a latch to a lease. Every NPC controller, every companion binding, every replay inherits the new semantics. | **ENGINE primitive** if wanted at all (the *mechanism* is an absence, not an opinion; only its *values* would be pack policy). | The **observer** — entirely. You cannot bound an intent you did not issue. Nor a bound chosen wrong. And it is a **second scheduling mechanism beside `target_tick`** (P-4), which is this project's named characteristic failure (`spines.md` header; A-4's *"a second mechanism written beside the one that already existed"*). |
| **C** | **Leash / tether radius** — anchor + radius per character; intent lapses at the boundary **[assistant-proposed, and I recommend against]** | Two new per-character sim fields, a boundary policy (stop? slide? turn back?), and a verb to set/clear it. Comparable to B in code, larger in design. | Adds a **silent countermand**: the sim overrides a driver's stated intent without the driver asking. This is *precisely* the failure `bodies.md:518-522` cites — *"a driver obligation is invisible and its failure is silent"* — inverted onto the sim. | **PACK OPINION** (see § 5) — and the pack that would own it does not exist and is **ON HOLD, a user call** (P-9). | The observer. And it front-loads a **steering policy** decision (what a body does at a tether edge is a locomotion-behaviour question) into a debugging problem. |
| **D** | **Expose `target_tick` on the dev surface** — the driver pre-schedules the zero-intent **[assistant-proposed; file, do not build now]** | One optional arg threaded into `envelope_for_tool_call` (`dc-mcp-dev/src/lib.rs:76-93`) and a decision about whether it is per-tool or a universal envelope arg. **~15 lines.** | **Nothing** — the host already does this (`host.rs:658-676`). It discharges an unindexed A-4. | **ENGINE / dev-surface.** | **The blocking reply** (`authority.rs:865-886`): a stop scheduled 60 ticks out leaves the MCP call parked for 3 s, so the driver **cannot shoot mid-motion** — which is the 0148 failure verbatim. It bounds the *travel* but not the *shutter*. Also nothing for the observer. |
| **E** | **Terrain — build the station bounded** (the user's wall) **[user-ruled, by demonstration]** | **Zero code.** One `world_fill` per station, which the walk loop is already making anyway (`CLAUDE.md` § Agent walks: the bench). | Nothing. | **NOT A MECHANISM** — it is walk doctrine, and it belongs in `CLAUDE.md` § Agent walks beside the three 0148 framing rules. | Nothing *fast*: a wall is placed before the walk, so it cannot answer a surprise. And it costs a station-framing constraint (a wall in frame). |

**Two options I considered and discarded before pricing them.** A **global sim pause** — the tick
loop runs unconditionally on wall time (`tick_authority`, `authority.rs:1121-1137`) and pausing it
stops chunk streaming, the player, and everything else; that is a much larger commitment than the
problem justifies, and it is the A-1 risk named in § 7. A **speed cap for walk stations** (drive at
`speed: 0.2`, so 0.9 m/s) — this is *free today, needs no design at all*, and is a real partial
answer that belongs in doctrine: it converts 27 m of shutter-lag travel into 5.4 m. Recorded here
because it costs nothing and nobody has written it down.

---

## 4. The two deciding questions, answered

### 4a. Dev-surface tool, or sim primitive? → **DEV-SURFACE. Emphatically.**

**A halt verb is not a change to the sim; it is N invocations of a decision already made.** The
2026-07-19 freeze ruling (P-1) established that *stopping a body means zeroing its move intent
through the ordinary command rail*, and `freeze_character` (`authority.rs:583-607`) is that decision
compiled. Option A adds **no new sim concept, no new state, no new semantics** — a fan-out over an
existing effect, gated by an existing grant shape. Nothing in `CharacterState`, `CharacterInput` or
`step_character` changes by one line.

Bounded intent is the opposite: it redefines the latch as a lease, and *"existence is not standing"*
cuts the other way here too — **do not build the general one to serve the specific one.** The
specific need is *"the driver and the observer cannot stop a demo."* The general mechanism is
*"intents expire."* There is no consumer for the general mechanism today: no NPC controller exists,
the bio/socia layer that would want timed behaviours is **ON HOLD and the gate is a user call**
(P-9), and B's real cost is not code but the design conversation about airborne expiry, jump
interaction, and replay. **Building B now would be an unratified generality with one debugging
consumer** — and, worse, the second scheduling mechanism beside `target_tick`, which is the exact
shape `spines.md` says this project reinvents.

*The falsifier for this answer, stated so it can be checked:* if within the next two bodies arcs a
**second, non-debugging consumer** appears that needs an intent to self-terminate — a wander
behaviour, a flee-then-stop, a scripted demo pack — then B stops being a generality-for-one and this
answer should be revisited. **A halt verb does not block B**; it is orthogonal, and building A first
costs B nothing.

### 4b. Does the observer's need differ from the driver's? → **YES, and the difference is quantitative, structural, and decisive.**

They differ in three ways, and only the first is obvious:

1. **Latency, by ~20×.** The driver's floor is one MCP round-trip (~1 s ≈ **4.5 m** of travel), and
   *no verb can beat that* — a halt call is still a round-trip. The observer's floor is one frame
   plus one tick: **≤ 0.225 m**. The observer's channel is therefore worth roughly **4.3 m of
   stopping distance per event** more than the driver's, at `speed: 1.0`. This is the whole reason
   the wall exists: the user was not missing a verb, they were missing a *fast* one.
2. **Direction of authority.** The driver wants its *own* intents to self-terminate. The observer
   wants to **countermand somebody else's** — a veto, not a plan. Those are different verbs, and B
   and C only serve the first.
3. **What they know.** The driver knows what it asked for; the observer knows what they are *seeing*
   and, crucially, sees the ledge first. The channel the observer needs is the one that needs no
   arguments at all — no character name to type, no coordinates. That is what makes A′ (a bare key)
   the right shape and P-3's console (typing `character_set_move_intent character=stout dx=0 dz=0
   speed=0` per body) the wrong one, even though it exists today.

**And the partly-non-technical conclusion the brief asked me to consider seriously: I reach it, but
only halfway, and I want to be precise about which half.**

- **The wall was CORRECT, not a workaround.** `CLAUDE.md`'s own advice — *"prefer intents that
  terminate safely"* (P-8) — plus the three 0148 framing rules already say that a station should be
  built so it can answer. Terrain is a *bounded intent* implemented in voxels, it is free, it is
  deterministic, and it needs no design. **The primary answer to "the walkers left frame" is
  station design, and it should stay primary.** I would add it as a fourth framing rule
  **[assistant-proposed]**: *BOUND THE STATION IN THE WORLD. Every intent commits seconds of motion;
  place the terrain that stops them before the first intent, and prefer driving into the bound.*
  That plus the free `speed: 0.2` cap covers most of the 0148 failure with **zero code**.
- **But it does not cover the surprise**, and the surprise is what the observer is *for*. The user's
  verdict is senior to the screenshot read (`CLAUDE.md` § Agent walks); an observer whose only
  recourse is to pre-build a wall cannot exercise that seniority *during* a station. Step 5 of the
  walk loop — *"brief the station and PAUSE… wait for the user to poke around and give a verdict"* —
  presumes the world holds still while they look. Today it does not, and no doctrine fixes that.
- **So: doctrine for the framing, a keybind for the veto.** The mechanism I recommend is the small
  one that serves the case doctrine structurally cannot — and it must be introduced with the
  doctrine, not instead of it, because a halt key makes it *possible* to run an unbounded station,
  and that is the failure mode of every safety net.

---

## 5. Placement — `docs/dependency-graph.md` § 0a / § 0b, test applied

**§ 0a** (*"bodies is not outside the engine-plugin divide… primitives, kernels, capabilities on the
engine side"* — **[user-ruled]** 2026-08-04) places all four mechanical options on the **engine**
side of the body work, because none of them is a body *plan*, a *material*, or a species trait.
The question § 0b then decides is whether each is a real capability or a hidden opinion.

**§ 0b's test, one question: *would two good packs disagree about this?***

| option | two good packs disagree? | verdict |
|---|---|---|
| **A** `halt_all` | **No — and the question is malformed for it.** No pack has a view about whether a *debugging surface* can stop the world's characters; a pack never sees this verb, because reaching it requires an unscoped `CharacterControl` grant that only the dev surface and the player shell hold. | **ENGINE capability, dev-gated.** Not an opinion, not an absence — an **instrument**. |
| **A′** keybind | No. | **CLIENT SHELL**, outside the registry, exactly like `client_screenshot`. |
| **B** bounded intent | **No** for the *mechanism* (every pack would want an intent to be expressible with an extent, if it existed at all); **yes** for the *values* (how long a wolf commits to a charge). So the mechanism is an **ABSENCE**, engine-side. | **ENGINE primitive — and therefore exactly the thing "existence is not standing" says not to build to serve one debugging consumer.** Correct disposition: **do not build**; if it is ever wanted, it is engine, and it must not be smuggled in as a dev fix. |
| **C** leash radius | **YES.** Home range, territoriality, herd cohesion, flight distance — an Earth pack and a low-gravity-moon pack would answer differently, and a pack of solitary ambush predators differently again. | **PACK OPINION**, in a pack that **does not exist and is ON HOLD (a user call)**. Building it engine-side to serve a walk would be the § 0b failure verbatim: *"hiding an unbuilt mechanism behind a number that now has a respectable owner"*, one level up — hiding an unbuilt **pack** behind an engine verb. **Reject.** |
| **D** `target_tick` exposure | No. | **ENGINE / dev surface.** Discharges an unindexed A-4. File. |

**⚠ One thing § 0b does not cover, stated as a finding rather than reconciled away.** The test's
dichotomy is *opinion → pack, absence → engine stub*. **`halt_all` is neither.** It is a
**dev/creative-tooling capability**, a category `docs/API.md` § *Characters, controllers, and the
two MCP surfaces* already names in its own table (*"`dc-mcp-dev` — audience: building the game,
creative tooling"*) but which § 0b's two-way test has no slot for. Reading it through the test alone
would push it toward "absence → engine stub", which is *nearly* right for the wrong reason: it is
not an unbuilt mechanism awaiting an heir, it is a finished instrument. **Whether § 0b wants a third
answer ("an instrument — engine-side, dev-gated, never pack-visible") is a real question for the
user; I am flagging it, not writing it.** It is exactly the kind of thing that goes wrong quietly:
an instrument filed as an absence acquires an heir it will never have.

---

## 6. Against the `set_look` / `clear_look` HOLD ruling

**The ruling** (`bodies.md:504-517`, **DECIDED 2026-08-02, [user-ruled]**): *"Move intent defaults to
look-follows-travel; an explicit look is HELD until released… No hold timer: decay would be a tuning
constant nobody ratified."* Its stated reason: *"the engine owns the contract and the correct
default; drivers invoke deviations explicitly by verb"*, because *"a driver obligation is invisible
and its failure is silent."*

**Option A is fully consistent with it, and consistent for the ruling's own reason.**

- **The latch stays a latch.** `halt_all` does not add a timer, a decay, or an expiry to move intent.
  It is a **verb that releases**, which is structurally `clear_look` — and the ruling's shape is
  precisely *hold-until-a-verb-releases*. A halt verb is the move channel finally getting the
  release verb the look channel already has. Today `set_move_intent(0,0,0)` is the release, but it
  is per-character and unnamed; `halt_all` names it and fans it out.
- **It leaves the look alone, and the interaction is already correct.** A halt zeroes travel, so
  `travel_yaw` is `None` and the *unheld* gaze holds its last heading (`character.rs:307-320`) —
  which is the ruling's own stationary rule (*"when stationary it keeps its last heading, like the
  trunk"*). A **held** look stays held, as it must. **Nothing to design; verified at source.**
- **Naming, so the parallel is not lost:** if the pair `set_look`/`clear_look` is the house shape,
  the move channel's pair reads `set_move_intent`/**`halt`**. I use `halt_all` for the fan-out
  because `freeze` is already spoken for by the 2026-07-19 disconnect ruling and reusing it would
  collide two meanings on one word. **The name is a user pick, not mine.**

**Option B is a reasoned EXCEPTION to the ruling, and this is the sharpest argument against it.**
A duration-bounded intent *is* a hold timer on the move channel — the exact thing the look ruling
refused by name, for the exact reason it gave (*"decay would be a tuning constant nobody
ratified"*). Any bounded-intent proposal must therefore either argue that movement is different in
kind from gaze, or ask the user to relax a ruling that is nine days old. **I am not making that
argument, and I do not think it can be made from a debugging need.**

Option C is a **worse** exception: it is a hold timer *keyed on geometry* that also countermands
silently — failing the ruling's letter and its stated rationale at once.

---

## 7. What this is NOT — the adjacent features it must not become

Named so a build can be held to them, and so this doc cannot be cited for any of them:

1. **NOT a general NPC-steering feature.** No goals, no behaviour trees, no wander, no follow, no
   flee. `halt_all` has no arguments and no policy.
2. **NOT pathfinding.** Nothing here computes a route, a reachable set, a ledge test, or an
   obstacle-avoidance rule. *"Do not walk off the edge"* is a pathfinding requirement and is
   explicitly out of scope — the answer to a ledge remains terrain (option E).
3. **NOT a scripted-camera or cutscene system.** No timelines, no keyframed driver actions, no
   station DSL. The walk loop stays a human-in-the-loop conversation.
4. **NOT a global sim pause / time-scale / step-frame debugger.** `tick_authority` runs on wall time
   and everything rides it; halting *characters* is a bounded act, halting *the world* is a
   different and much larger commitment. **If a proposal starts saying "and while we're at it, pause
   the sim", that is the A-1 signature — the stand-in becoming the definition — and it should stop.**
5. **NOT the stop transition.** `stubs.md` #43's foot-in-the-air is **exposed** by this work, not
   fixed by it, and the two must not be bundled: one is a dev instrument, the other is B6/gait
   design that `posture-gait.md` § 4 declines to do.
6. **NOT a replacement for the walk doctrine.** The framing rules and *"prefer intents that
   terminate safely"* stay primary; a halt key is the veto for what doctrine cannot foresee.
7. **NOT a leash, under any name.** If a tether appears later it belongs to the pack, behind the
   bio/eco gate the user owns.

---

## 8. Falsifiable predictions and measured numbers

Every number here is read off a cited source or derived from cited constants. A build can check each.

| # | claim | how to falsify |
|---|---|---|
| **N-1** | Character walk speed at `speed: 1.0` is **4.5 m/s** in the live client — `CharacterConfig::default().walk_speed_m_s` (`character.rs:47`), not overridden (`authority.rs:247-251`). | Read `walk_speed_m_s` at runtime; drive a character 5 s and measure displacement. |
| **N-2** | One host tick is **50 ms**, so a full-speed body covers **0.225 m per tick** (`authority.rs:114`). | Diff `character_pose` across one tick. |
| **N-3** | **A driver-side halt cannot reduce stopping distance below ~one MCP round-trip of travel** ≈ **4.5 m** at full speed. No verb changes this. | Time 20 `character_pose` round-trips; if the median is < 0.2 s the bound is 0.9 m and the driver/observer gap in § 4b narrows. |
| **N-4** | **An observer keybind stops a body within ≤ 1 frame + 1 tick ≈ ≤ 0.225 m** of travel plus human reaction — **~20× better than N-3**. | Bind it, run at full speed, measure displacement between keypress frame and the tick the intent zeroes. |
| **N-5** | The 0148 shutter lag (**~4 MCP round-trips**, `CLAUDE.md:289-292`) equals **~27 m** of travel at full speed and ~1.5 s/round-trip — **more than a third of that walk's own 63 m runway per capture cycle**, and enough to leave any single-body frame. | Instrument a set-intent→screenshot sequence and compare measured displacement to 27 m. |
| **N-6** | Driving stations at **`speed: 0.2`** cuts N-5 to **~5.4 m** with zero code. | Same measurement at `speed: 0.2`. |
| **N-7** | **`halt_all` requires no change to `CharacterState`, `CharacterInput`, or `step_character`** — the diff is a payload variant, a schema row, a host arm, a `Requirement` variant, and the `mcp_character.rs:45` exclusion + its pinned test. **≈40 lines / 5 files.** | Build it; if the diff touches `step_character` or adds a `CharacterState` field, this design was wrong. |
| **N-8** | **Goldens and replay are unaffected.** `halt_all` is N applications of an existing, already-journalled effect on the existing rail; `character_session_replays_identically_and_stays_caged` (`authority.rs:1480+`) and the `dc-api` character semantics suite should pass **unchanged in count and by name**. | Run the suite before/after; any moved hash falsifies. |
| **N-9** | **An attached character session (7778) cannot halt any character but its own** — its token is attenuated to `Some(name)` (`authority.rs:645-652`), and `CharacterControlAnywhere` is satisfied only by `character: None` (mirroring `capability.rs:207`). | Attach a session, call `character_halt_all`, expect a capability rejection receipt. |
| **N-10** | **A new `character_*` command breaks `mcp_character.rs`'s pinned tool-list test** (`:440-462`) unless it is added to the exclusion at `:45`. The test's own comment predicts this. | Add the command without touching `:45`; the test must go red. |
| **N-11** | **The first live use of `halt_all` on a multi-body station will show at least one body frozen mid-swing with a foot off the ground** (`stubs.md` #43). | Halt three walkers at full speed and screenshot; zero feet in the air falsifies. |
| **N-12** | **`target_tick` is expressible by no caller in the workspace** — every construction site passes `None` (`dc-mcp-dev/src/lib.rs:90`, `authority.rs:618`, `schema.rs:1054`, `:1166`). | `grep -rn "target_tick" crates/ --include=*.rs`; any site passing `Some(_)` outside a test falsifies. |
| **N-13** | **The in-game console can already stop a character today**, unmodified, via `character_set_move_intent` with `speed=0` (`console/mod.rs:1-13`, `app.rs:342`). | Press **T** in a running client, type it, watch the body stop. **If this fails, P-3 is wrong and the observer really has no channel.** |

---

## 9. Could not determine — flagged, not reconciled

1. **Whether the user knows about the console (P-3).** The walk-loop docs never mention it, and the
   0148 quote (*"there's no way for me to tell you to stop them"*) reads as though the console were
   not in view — but it may simply have been judged too slow, which would be correct. **This changes
   which fix is primary** (a keybind vs a doc line) and only the user can answer it. **I did not
   launch the client to check** (walks are the main session's to drive).
2. **Which key.** A user pick. Free keys listed in § 3 A′.
3. **The verb's name and scope** — `halt_all` vs `halt { character: Option<String> }` (one verb, all
   characters when unnamed). The latter is tidier and costs the same; I did not pick it because
   `Option`-in-payload interacts with the capability requirement in a way that wants a ruling, not a
   guess.
4. **Whether a halt should also clear `look_held`.** I say no (§ 6), but it is a judgement about what
   a walk driver expects, not a fact.
5. **Whether `spines.md` § 3 should gain a `target_tick` row.** It qualifies (built, no caller) and
   the index does not have it. **I did not edit `spines.md`** — out of scope for this pass; handing
   it up.
6. **No `stubs.md` entry is owed.** Nothing here implies a stand-in that exists *today*: `halt_all`
   is not a placeholder for a richer mechanism, and `swing_gain` (#43) and `target_tick` are already
   filed or handed up above. **I wrote no stub and touched no ordinal.**
7. **I ran no cargo command** (the build slot is held; the brief says assume denial). Every code
   claim is read at source and cited; none is compile-verified.
