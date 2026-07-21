# 0049 — The first guided tour: five stations, five verdicts

> blogworthy: the protocol itself — a live co-walk where the user stands at
> each station and gates the tour with their verdict, replacing the
> screenshot-report ratification loop. The magnitudes were judged by a
> person standing in the wind, not by an agent describing it.

journal/0047 flipped the erosion-agent roster on with its seven magnitudes
explicitly unratified, and built the tour map. This entry is the tour —
the first time on this project an appearance ratification happened *live*,
station by station, with the user in the world and the integrator driving
teleports and briefings, each move gated on their comment.

The world: stock production (tectonic history, corrected climate, full
roster), `--horizon 3` — dropped from 6 after the second DeviceLost crash
(see Observed; an idle `--horizon 6` session died in ~4.5 minutes, so the
wide horizon waits on the leak diagnosis).

## The verdicts

**1 — Deflation basin (5993, 14732), 13.06 m stripped: RATIFIED.** The
user's read: impossible to fully judge while the veneer paints grass and a
topsoil layer over the most wind-stripped country in the world — "always
going to have topsoil." The sharp part: that is literally stub #3 — the sim
holds `H = 0` here and the collapse tier re-invents soil from present-day
precip. The user's complaint is the carry-`H` slice's gameplay-impact trace,
verbatim: when `H` is carried, deflation basins genuinely bare out. Louder
or quieter would not change felt terrain until then.

**2 — Dune field (107183, 9672), 2.09 m of sand: ACCEPTED AS IN-PROGRESS**,
and it produced the tour's biggest design directive: quantization will be
solved by the partials mechanism — **sand becomes the first loose material
spawned in-world, emitted as partials** — and beyond that, "to be perfectly
honest, everything should be spawned in partials." The world isn't cubes;
it's substances in structures — partials, loose volumes, forms. A
**forms mechanism across all systems** is now a named design pass. (The
substrate is mostly built: journal/0010's partial-height loose rendering
ships dormant; S8's loose mechanics are live in the placer.)

**3 — Loess margin (82346, 24391), 2.47 m: RATIFIED — "i can actually
believe this one."** Broad flattish mounds read as landforms through the
one-grass-block veneer. Bonus find: the user spotted their first
"shore-looking shore" — the veneer's below-datum Dirt band drawing the
future waterline, a beach waiting for its ocean.

**4 — Periglacial summit (−4586, −3206, 1000 m), 11.8 m frost-stripped:
WIN — RATIFIED.** The user dug in and found "the most variety in kind and
shape/chunks of stone I've seen so far" — the frost story reads *in the
column*, where normally a few unbroken bands sit. The signal lives
underground; the surface legibility debts are the same three (veneer,
carry-`H`, partials) now attached to every station.

**5 — Wave coast (95224, 22091), 0.68 m: NULL CONFIRMED — RETUNE.** "I
can't really tell anything. It's a flattish shore." The user's call: more
dramatic by default. And the mechanism insight that outranks the knob: wave
energy should ultimately be a fact about the WATER BODY — its size, shape,
depth, and the wind across it, changing over millennia — i.e. a **fetch
model**, whose ingredients already exist (S11 body graph + 0037 wind field).
Recorded as the heir; the retune is the explicit interim.

## Cross-cutting findings

- **The magnitudes were mostly fine; the legibility wasn't.** Four of five
  stations ratified as mechanisms, every one of them dampened by the same
  three expression debts. The tour's real output is priority evidence for
  carry-`H`, the veneer's ecology replacement, and the forms/partials pass.
- **The renderer is leaking something.** Late in the tour the user reported
  texture *smearing* to the human eye after heavy teleporting — at
  `--horizon 3`. Filed with the DeviceLost entry as a corroborating
  symptom: the leak isn't exclusive to wide horizons, just slower.

Assets: `0049-station1-deflation-basin.png` … `0049-station5-wave-coast-null.png`.
