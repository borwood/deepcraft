# Nobody owned the look, so the engine took it

*2026-08-02 · the look-ownership ruling + the follow-travel slice · bodies thread, first
topic of the session*

The 0140 walk ended on a caveat the user had been carrying for two weeks: driven bodies
strafe. The mechanism (read at the walk, confirmed live in one minute) was an inversion —
`resolve_orientation`'s clamp rule, *a look beyond the neck clamp drags the trunk around*,
is correct for a player whose look the camera re-writes every frame, and exactly wrong for
a character whose look nobody ever writes. The dead spawn-time gaze dragged the trunk while
`steer()` chased travel, and the legs walked sideways under a body facing its birth
direction. S6 row 95 had carried *"nobody owns body orientation"* OPEN since 07-19.

## The ruling

Presented as the two ownership shapes: **look-follows-travel by default, explicit look held
until released** vs **every driver owns the look**. The user took the first — *"A
definitely sounds like the right call"* — with one caveat worth recording verbatim: *"in
the future we may revisit the issue of animals with multi-segment necks. as long as the
look-at can be replaced with a different function in the future / is compartmentalized
enough, I don't think this is an issue."* That compartmentalization already exists and is
now named as the seam: `resolve_orientation` is a pure `(trunk_yaw, look_yaw, look_pitch) →
Orientation` function; a multi-segment neck replaces its body, not its owner.

The deciding argument was not aesthetic. Driver-owned look is the arrangement that had
*already failed*, silently, for two weeks — a driver obligation is invisible and its
failure mode is a body that merely looks wrong. Default-correct beats remember-to-set, and
it is the same shape as the same-day postures ruling: the engine owns the contract and the
correct default; drivers invoke deviations explicitly by verb.

## The slice

Sim-side, deliberately — the gaze aims the senses (`sense_raycast`'s default direction is
`view_dir()`), so a renderer-only fix would have left a driven creature walking east while
its perception pointed at its spawn heading. `step_character` now derives an unheld gaze
from the horizontal intent (yaw = travel heading via the view convention `θ = atan2(−x,
−z)`, pitch level; stationary keeps the last heading, like the trunk). It is a pure
function of sim state, so replay is untouched.

`CharacterState` gains `look_held` (appended, `serde(default)` — pre-ruling streams decode
to follow-travel). `set_look` now HOLDS the gaze; the new `dc:character/clear_look` verb
releases it — held-until-released rather than a decay timer, because a decay constant would
be a tuning knob nobody ratified. The pose readback reports `look_held`, the walk-11
round-trip principle applied on the day the field was born instead of as a later loose end.
The MCP tool (`character_clear_look`) came free from the command-table macro; the one
manual list — the completion-intent test's partition of the registry — caught its own
omission by count, which is that test doing precisely its job.

The renderer needed nothing. With the sim's yaw tracking travel, `resolve_orientation`
gets a look that agrees with the trunk's target, and the walk-8 fix works on the path it
was never built for. A held look beyond the neck clamp still drags the trunk — which is now
the *deliberate* strafe, a predator circling with its eyes locked on prey, available to any
driver that asks for it.

Evidence at merge (batching ruling, cheap tier): fmt clean; clippy `-D warnings` green on
dc-api + dc-client; dc-api suites green with the two new tests verified by name
(`unheld_gaze_follows_travel_held_gaze_survives_it`,
`gaze_follows_travel_unless_held`). dc-client's behavior suite rides as recorded batch
debt to the arc gate.

> blogworthy: ownership gaps don't fail loudly — they invert quietly on the path nobody
> built for (lens 3). The fix is never "remember to set it": make the engine own the
> default and demand explicitness only for deviation. Two rulings in two days took the
> same shape from opposite directions (postures: invoke-by-id; look: follow-by-default),
> which is what a spine looks like while it is forming.
