# Three bodies stand at their own heights

*2026-08-02 · the derived-hip walk (consumer slice, journal/0137's continuation) · stations
with the user live*

First appearance walk since the rotation quantizer came out, and the first ever where the
pelvis is not pinned. Two stations on a cut granite bench, the user's live view senior
throughout.

## Station 1 — rest (asset `0138-three-derived-heights-bench3`; pose: feet (0, 911.75,
−12.2), yaw 0, pitch −0.1)

Biped, stout, longleg side by side on flat granite at their derived hips — 0.880 m,
0.440 m, 1.020 m. Longleg stands visibly tallest on dead-straight legs: the body that
squatted at −56.2° under the pinned hip reads as a tall creature, which is the ratified
acceptance standing in a frame. **User verdict: "soles look planted other than the idle
bob. reads right."** The idle bob residue is corrections #80's temporal half, known and
deliberately untouched by the slice. (Companion sighting on natural rubble:
`0138-three-derived-heights-idle`.)

## Station 2 — motion (assets `0138-walk-cycle-profile`, `0138-walk-cycle-front`)

The trio driven by move intents across the bench, profile pass then frontal pass. **User
verdict: "walk cycle reads fine with caveat… they do not rotate to face the direction they
walk. they appear strafing toward the camera. this was called out over a week ago and must
not have gotten its dev time."**

The user's memory was exact on both counts. The callout is walk 8 (journal/0009, *"appears
to strafe"*); journal/0014 built the fix — trunk chases travel, head splits off within the
neck clamp — and proved it *for the player path*. The S6 baseline audit (row 95) has
carried *"nobody owns body orientation"* OPEN since 07-19. Nobody ever owned the
non-player path.

**And the mechanism is not absence — it is inversion.** `resolve_orientation`'s clamp rule
(*a look beyond the neck clamp drags the trunk around*) is correct for a player whose look
follows the camera, and reverses the walk-8 fix for any character whose look is stale: the
dead look drags the trunk while `steer()` chases travel, and the legs strafe under a body
facing the wrong way. Confirmed live in one minute: `character_set_look` to the travel
heading, same intent, and the biped walks away showing its back
(`0138-look-steered-control`).

Filed in ROADMAP Observed with the design question it opens — whether move intent defaults
to look-follows-travel unless a look is explicitly held, or every driver owns the look. A
controller-surface call, adjacent to the same day's postures ruling; the user's to make.

> blogworthy: a fix that inverts on the path nobody owned — lens 3 (ownership gaps hide in
> the paths a feature wasn't built on), and lens 1: the user recalled a week-old unowned
> thread faster than any sweep surfaced it. The instrument for dropped threads is still a
> human.
