# 0038 — Record-walk: console v2, edges on current main, and an honest circulation null

*2026-07-20. A screenshot record-walk of current `main` (seed 1337, Extent::Medium,
the boot worldgen authority). Goal: put the day's shippable work into the visual
record — console v2 (0035), the edges pass re-shot on current main (0031), and the
zonal-circulation climate change (0037). No diagnosis; capture, name, describe
honestly. Two launches: `--fullbright --edges` for the console and edge shots, a
plain lit launch for circulation. Every pose reply reported `eye_in_solid: false`;
the sun is fixed (S4 constant 0.35), so the frames are comparable.*

## How the walk was driven (a tooling note worth recording)

The in-client MCP surface (`mcp.rs`) exposes exactly three client tools —
`client_screenshot`, `client_player_pose_get/set` — plus the dc-api registry
(`world_get_block`, …). **There is no keystroke/console tool over MCP.** The dev
console shares the bridge channel internally but has no MCP door: it is driven by
real keyboard events (`KeyboardInput`, toggled on `KeyCode::KeyT`, text taken from
`e.text`). So the console shots could not be produced through MCP alone.

They were produced by injecting OS keystrokes into the game window:
`AttachThreadInput` to the foreground thread + an ALT tap to clear Windows' fore-
ground lock, then `SetForegroundWindow`, then `SendKeys`. A plain
`SetForegroundWindow` from a background process is silently refused (the first
attempt sent `t` into the wrong window and nothing opened); the attach+ALT dance
is what actually gave the game focus. winit turns the `WM_CHAR` stream into
`KeyboardInput.text`, so typed commands land in the console exactly as a human
would type them. This is a walker's scaffold, not a shipped affordance — worth a
line in case a console MCP tool is ever wanted for hands-off walks.

## Launch A — `--fullbright --edges`

**`0038-console-help.png`** — pressing T then typing `help` renders the full
command surface: every command on its own line, tagged `[query]` / `[command]` /
`[client]`, with its one-line description (character_sense_raycast, _set_look,
world_get_block, client_player_pose_set, …). This is the 0035 discoverability win:
the surface is schema-generated, zero per-command code.

**`0038-console-signature.png`** — the live signature area, the actual point of
console v2. Typing `world_set_block block=` shows three things at once:
- the **signature line**: `world_set_block  block=<string> pos.x=<int> pos.y=<int> pos.z=<int>`;
- the **per-arg hint** at the caret: `block : string (required)  block name, e.g.
  dc:stone   e.g. dc:air, dc:stone, dc:dirt, dc:grass` — and those example values
  are *live*, pulled from the running world through the 0033 `Completer` hook, not
  a static string;
- the **input line** with the caret.

A layout quirk surfaced while shooting this and is worth recording. The console
panel is bottom-anchored at 45% window height, a flex column of scrollback → hint →
input. At the 1280×720 capture size, a full `MAX_VISIBLE_LINES` (22) scrollback
fills the panel and pushes the hint+input line *below the window's bottom edge* —
so a fresh, empty console shows the signature fine, but immediately after a long
`help` dump the input line is off-screen. That is why help and a live signature
can't share one frame at this window size; the signature shot was taken in a fresh
console with empty scrollback. Not a defect to fix blind — appearance is the
dev-tool default and the user restyles at will (0035) — but a real interaction at
720p, noted here for whoever tunes the panel.

**`0038-edges-hillside.png`** — a near/mid dissected slope in the origin region
(~43° latitude, a high ~1000 m plateau). Fullbright flat colour would be a
featureless field; the crease+silhouette outlines make every voxel bench legible
from the foreground blocks through the mid terraces to the far ridge silhouette
against sky. The 0031 win, intact on current main. (The tan is dirt-sided benches
seen edge-on — material albedo, the correct fullbright register; the green patch
top-right is real-worldgen grass past the near/far load-radius seam.)

**`0038-edges-massif.png`** — the moiré check, honestly. The origin region is a
high dissected plateau, **not** an alpine massif — a full horizon sweep (N/E/S/W)
found mesa-like rims and badland relief but no 3 km mountain. That is expected:
tectonic landforms come from `tectonic_history`, which `production_config`
inherits OFF with no client affordance to enable it (see below), so no exhumed
core or orogenic massif exists in any booted world. This frame shows the tallest
distant landform available — a stepped mesa rim on the horizon. The check it can
still answer: **no moiré at range.** The mid-field dense edges read as discrete
crease lines on the eroded ground, not a beating interference pattern, and the
far rim carries essentially no edge signal — faded to nothing under the
350→1400 m window, exactly as designed. Judge the far silhouette by the skyline
(it is clean), not by missing outlines.

## Launch B — lit pass, for circulation: the honest read

This is the one the walk was most meant to photograph (0037: "fly the 30°
latitude line and confirm a desert belt that isn't behind a mountain"). **The
clean latitudinal aridity band is not legible in the rendered surface.** That is
the honest result, and the mechanism is specific.

Latitude runs along world **Z** (8° S edge → 78° N edge, compressed onto the
17-cell Medium grid). I swept x=0 across six bands and read the surface both by
eye and with `world_get_block`:

```
lat ~18° (z −81 km, surf −81 m)   bare grey STONE   (near/below sea; abyssal→Stone)
lat ~31° (z −37 km, surf 193 m)   green GRASS       (the "desert belt" — greenest!)
lat ~43° (z   +7 km, surf 1024 m) green GRASS
lat ~51° (z +37 km, surf 1523 m)  bare grey STONE   (high mountains, bedrock)
lat ~59° (z +66 km, surf 956 m)   bare grey STONE
lat ~68° (z +96 km, surf 567 m)   bare grey STONE
```

The pattern that *is* visible is **elevation and temperature**, not the Hadley
aridity: green grass in the temperate mid-latitudes at moderate height; bare
Stone at the extremes (sub-sea coast, high-altitude bedrock, cold poleward). The
30° band is the **greenest** latitude sampled, the opposite of a desert.

Why: `collapse.rs` flips the surface to bare Dirt only where
`riverbed || precip < 0.10 || (fringe && precip < 0.35)`. The zonal profile drops
the 30° belt below the pipeline's **0.32 arid *biome*** threshold (0037) — arid
enough to change erodibility and biotic tags — but the subsidence-gated convective
floor keeps its precip around ~0.2–0.3, comfortably **above the 0.10 bare-*surface***
threshold. So the Hadley desert is real in the climate the sim carries, and it
shapes erosion and vegetation *tags*, but the top voxel stays grass. The circulation
change is loud to the tests and to erodibility, and quiet to the eye at the surface.

Bare Dirt *does* occur, patchily: sampling the 30° line across longitude, x=−30 km
surfaced `dc:dirt` while its neighbours grew grass — but that spot sits at 1258 m,
a dry eroded plateau, so its aridity is entangled with altitude, not a clean band.

Given the task's instruction to report a null plainly and capture the most arid
area rather than fake a band, the pair is:

**`0038-circulation-desert.png`** — the bare-Dirt plateau at ~30° / x=−30 km:
a broad unvegetated brown expanse with eroded gullies. Genuinely arid ground at
the subtropical latitude, but a patch on a high plateau, not an unbroken belt.

**`0038-circulation-wet.png`** — green grass lowland at ~30° / x=0 (193 m): the
lush contrast. Both frames sit at the *same* latitude, which is the honest story:
the visible dry/wet split here is intra-latitude and elevation-driven, not the
clean poleward march the 30° line was supposed to show.

> blogworthy: **a climate the map can't see.** The zonal circulation gives every
> world a real Hadley desert — it bends erosion and biofacies — yet the player
> walking the 30° line finds it green, because the surface-block rule only strips
> vegetation below precip 0.10 and the desert sits at 0.2–0.3. The simulation
> knows something the terrain refuses to say out loud. Whether that gap should be
> closed (lower the bare threshold in the arid band, or add a sparse-vegetation /
> sand facies between grass and bare Dirt) is a design call, not a bug — but the
> render currently under-tells the climate, and that is the walk's finding.

## What could not be photographed, and why

`full_agents` (wind/frost/wave eolian landforms — dune fields, periglacial
stripes, wave-cut coasts) and `tectonic_history` (orogenic massifs, exhumed
cores, forelands) are gen-time `DeepConfig` flags that `production_config`
inherits **OFF**, and the client's `Pregen::run` takes only `{seed, extent}` —
there is no launch flag, console command, or MCP door to turn them on. Every
booted world is a flags-off production world. So those landforms are structurally
absent from anything this walk could generate; they are a separate slice, not a
capture failure. This is why the massif shot is a plateau rim and why no dune
field or exhumed core appears anywhere above.

## Owed follow-ups (not filed here — this entry only)

- The circulation-not-legible finding is a genuine field observation and belongs
  in ROADMAP **Observed** (this walk was scoped to the journal entry only, so it
  is recorded here for that hand-off, not written to ROADMAP).
- The 720p console-panel overflow (help scrollback pushes the input/signature line
  off the bottom of the 45%-height panel) is an appearance loose end for whoever
  tunes the console layout.
