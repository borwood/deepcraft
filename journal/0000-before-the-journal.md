# 0000 — Before the journal

*Written 2026-07-19, recording 2026-07-18 from session memory · retrospective*

> blogworthy: the founding day as it actually happened — the reasoning and
> rejected alternatives that the docs record only as conclusions. Written by
> the AI collaborator from living context before session end, because
> otherwise this history exists nowhere.

The journal practice was adopted mid-project (0001 onward compresses the
spike day to outcomes). This entry preserves the *conversational* history —
what was proposed, what was pushed back on, why things are the way they are.

## The founding question

The project opened with: Minecraft-clone-that-becomes-more, plugin-first,
first-class MCP, in-game editors — **Unreal, or bespoke?** The user leaned
bespoke (Rust or C++ foundation + scripting runtime over an API) and had
already identified the two feared problems: physics and rendering.

The case I made for bespoke-Rust, which the user ratified: Unreal's value
(content pipeline, skeletal animation, Nanite/Lumen, its editor) is nearly
orthogonal to a voxel game whose product is *its own authoring tools*; voxel
engines fight Unreal's renderer; and the real architecture — one command API
consumed by scripts, MCP, and editors — wants to be owned, not embedded.
The feared problems dissolved on inspection: Minecraft-style physics is
swept-AABB vs the grid (~200 lines you want to own anyway; a real engine is
only for the later dynamic tier), and wgpu + the enormous voxel-rendering
prior art (Veloren as the shipped existence proof) covers rendering. Bevy
chosen as foundation because plugin-first ECS *is* its architecture; WASM
(not Lua) for plugins because sandboxing user content is non-negotiable;
MCP in-process over the same API. "One API, three consumers" was on the
table within the first hour and never moved.

Platform intent from the start: Win/Mac/Linux first-class, consoles
protected-not-built (thin platform layer, no native APIs outside wgpu).

## Convictions that arrived before any code

The user's second message set the ambition that shaped everything: vastly
expanded depth; Distant-Horizons-class far terrain and custom shaders as
first-class; globally-aware generation ("a step in the direction of Dwarf
Fortress, but 3d"); and — in their own words — keeping the entire world
simulated through "stepped abstractions," with far state as "a wavefunction
that ticks far slower," collapsed "by simulating something connected to that
system: meeting an NPC that has travelled from that territory, or traveling
there ourselves, or using psionic powers." I renamed the mechanism
(deferred resolution under observation constraints — it is not the WFC
texture algorithm) and identified the two hard problems that became S2:
consistency under partial observation (the constraint ledger) and bounded
collapse (no planet-wide cascades). The committed/fluid distinction was
designed *before* the repo existed, as was the insight that worldgen
history and live far-sim should be one system.

Scale intents: player = 3 or 4 voxels was the initial instinct (the S1
spike + a hands-on feel pass settled on 2, with sub-voxel shapes as the
fidelity lever). "Chunks have 3d position" was raised as a possible
Minecraft misconception — it isn't one (MC loads strictly by column) — and
became the cubic-chunks decision; later the user corrected my
interpretation: they'd meant *tall* chunks, and the discussion (remesh
granularity, octree LOD, palette-compressed boring chunks) landed on cubes
with the tall-chunk instinct reimplemented as adaptive load volume + 3D
sim-tier bubbles. Depth as a pillar — "deep earth trogs with their own
society" — dates to this exchange.

## The API day

docs/API.md was designed in conversation while S1/S2 ran. Decisions made
live: `dc:domain/verb_noun` ids; **player input as commands from day one**
(the user's reasoning: replay is the substrate for an AI-native dev
pipeline — repro = replay a command log agents can generate); and the
character/controller split, which arrived when the user described wanting
MCP-driven characters — "if a player started two characters on a world, it
could give one to claude" — plus the two-surface instinct ("maybe even a
separate MCP to keep tool list tidy"). The observe/inspect split (reading
distant fluid state is a *mutation*) emerged from colliding the API design
with S2's ledger, and immediately suggested diegetic scrying.

## Process history (the expensive lessons)

The first two spikes ran as parallel worktree agents, each compiling Bevy —
the machine hung ~15 minutes and the user had to tell us. Everything about
resource discipline (one cargo invocation anywhere, --jobs 4, shared
CARGO_TARGET_DIR for worktrees) descends from that failure. The user's
walk-2 feedback (chunk flashes, z-fighting) produced the first corrections:
my depth-bias fix was wrong-in-kind (Bevy depth_bias only sorts draws), and
the user's "flip the sign — still z-battling" report forced the radial-push
solution. The pattern of *the user reading renders better than the agent*
started here and peaked at walk 3 (corrections.md #3).

Design-in-the-gaps also started day one: API.md, materials (the user's
8-eighths debris/structure/pore model, packing, stratification), visuals
("this is an elevated pixel game" — their phrase; LabPBR packing was their
spec from modded-MC fluency), and worldgen (bounded-vs-infinite mulled
openly; their "base state + neighbor summary, not final states or we would
have infinite regression" formulation, completed with per-level hierarchy)
were all designed in conversation while agents built.

## Provenance notes

- The journal/roadmap/corrections practice is imported from the user's
  `orogeny` Minecraft mod (their prior art; also the geology idea quarry).
- The materials combinatorial-cap result (S8) and the year-zero ledger
  handoff (S2+S7) were the day's two genuinely novel findings.
- The user's standing instruction: my proposals stay proposals until
  ratified; their ideas get sharpened, not replaced. That dynamic — now the
  session-workflow skill — was implicit from the first exchange and named
  explicitly at the end of day two.

What this entry cannot preserve: the full texture of every exchange. What
it does preserve: nothing in the docs was handed down — all of it was
argued into existence, mostly in the gaps between builds.
