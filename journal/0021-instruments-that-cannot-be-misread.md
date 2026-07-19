# 0021 — instruments that cannot be misread

*2026-07-19 · background agent, instrument batch. Three small walker-facing
fixes that share one lineage: corrections #3 (don't diagnose from a viewpoint
you haven't verified) → #4 (postcard is positional; don't drop fields) → #10
(don't diagnose from an instrument whose *units* you haven't verified). This
patch closes the loop those corrections kept pointing at: a reply a walker can
read wrong is a defect, even when every number in it is correct.*

## The thread

Three separate ROADMAP Observed lines turned out to be the same complaint
wearing different hats. Corrections #10 cost a full agent cycle to a misread:
a pose in **meters** cross-checked against `world_get_block` answers in
**voxels**, at N=2 a factor of 0.9 apart, turned a walker standing on dirt
into a walker entombed in granite. The diagnosis at the time named the real
bug it stumbled into (`eye_in_solid` answering from the wrong world) but the
*originating* fault was never in the code — it was in the reply. Nothing in
either instrument said which language it spoke.

Walk 13 (journal/0018) then found a sibling: a `surface:true` teleport reply
carried `surface_snapped` only on the **miss** path. On success the flag was
absent, and success had to be *inferred* from the y-coordinate moving. Absence
meaning success is the same class of trap — silence is not a reading. And
walk 11 (journal/0014) left a driver unable to read its own posture back:
`set_posture` existed, the pose query omitted it, so a controller could set a
state it could never confirm.

None of these is a computation bug. All three are legibility bugs. The batch
fixes the instruments, not the mechanisms behind them.

## The three fixes

**1. Pose replies echo the feet voxel beside the meters.** Both
`client_player_pose_{get,set}` and the character `dc:character/pose` query now
carry `pos_voxel` alongside the existing meters `pos`. The coordinate is
derived through the *same* conversion the authority uses — `scale.voxel_at`
client-side, the identical floor-divide the host's `eye_voxel` already used
character-side — so it is exactly the argument `world_get_block`/`scan_region`
take. A cross-check no longer needs a mental unit conversion nobody performs
reliably at 3am. The meters field keeps its name; the voxel field is appended
and unambiguously named. Two languages, both labelled.

**2. `surface_snapped` is always present in `surface:true` replies.** True when
the scan seated the feet, false on a miss (position left as requested, with a
`surface_error` string). The teleport reply construction moved into one
`surface_teleport_reply` helper so the flag can never again be set on only one
branch — the success and miss paths now leave through the same door.

**3. `dc:character/pose` exposes posture.** The reply gains a `posture` field in
`set_posture`'s own vocabulary (`standing`/`crouching`), via a new
`Posture::to_wire` that is the exact inverse of the existing `from_wire`. A
readback now round-trips straight back into a posture command.

## Wire discipline

Every new field is **appended**, never reordered, with `serde(default)` —
the ratified pattern for this codebase's positional postcard streams
(corrections #4; the same move `CharacterState.posture` made in step 3 of the
body staircase). `QueryData::CharacterPose` is a query *result*, not a logged
command, so it never enters the replay stream at all — but it follows the
append-only rule anyway, so the doctrine has no exceptions to remember. The
posture-replay and scripted-session bit-identity proofs stay green:
`Vec3i` gaining a `Default` derive (needed by `serde(default)` on `pos_voxel`)
changes no bytes, and no field moved.

> blogworthy: three "bugs" that were all the same bug — a reply a reader can
> misread — and the realization that for an agent walking its own world, the
> instrument's *legibility* is part of its correctness. A correct number in an
> ambiguous field is a wrong instrument.

## Tests

Each fix is pinned. The character pose reply's `pos_voxel` is asserted equal to
the floor-divide of its own meters pose *and* cross-checked against a real
`dc:world/get_block` at that voxel (the slab below the feet reads stone, the
feet voxel reads air) — proving it is the get_block frame, not just a number.
The client pose reply's `pos_voxel` is checked against `scale.voxel_at` and
tied to the meters query through the shared conversion. `surface_snapped` is
exercised on both the success path (a real land column) and a forced miss (the
scan window carved to air, so there is genuinely nothing to stand on). Posture
readback is driven through `set_posture` and read back off the pose query.

## Walk 16 (main session): instruments live, and the pair that convicted the heightlerp

*Appended post-integration (merge `71403e0`, gates green on merged main).*
All three instruments verified against the running game: `surface_snapped:
true` in a successful surface teleport; `pos_voxel` echoed and
cross-checked against `world_get_block` with **no manual conversion** —
feet voxel air, voxel below grass, first try (the corrections-#10 failure
mode is now structurally hard to reproduce); posture set to `crouching`
and read back as `crouching`.

Then the walk paid twice. The user had asked whether the lit PBR path
shows mixtures at all — hard to tell by eye. The instrument for that
question is a **same-framing pair**: stand two voxels from the quarry's
olivine-bearing granite wall, photograph lit, relaunch fullbright,
photograph again (`0021-mixture-lit-closeup` / `0021-mixture-fullbright-
closeup`). Verdict: fullbright shows olivine winning whole world-anchored
cells — including one inside the crosshair-targeted face — and the lit
shot shows **no green anywhere**. Same mesh, same splat attributes, so
the data is present; the heightlerp buries it. Mechanism: per-pixel
elevation = splat weight + texture height, and a ~1/8-fraction accessory
cannot out-elevate a ~7/8 host on any pixel — the minority constituent
loses everywhere, which is exactly the "alpha-mush" failure the
heightlerp was chosen to avoid, inverted: not mushed, erased.

> blogworthy: the diagnostic mode as adversarial witness — fullbright
> exists so an AI walker can see data without lighting noise, and its
> first structural use was to prove the pretty renderer was hiding the
> geology. Auditability caught what taste couldn't.

Filed to Observed with the fix shape (proposed, not decided): quantize
the lit blend by the same world-anchored 4×4 cell hash the fullbright
speckle uses — the cell's categorical pick gets an elevation bonus, so
lit mixture reads as textured cells that AGREE with fullbright about
where the ore is. Lit and diagnostic modes telling one story is itself
an auditability property.
