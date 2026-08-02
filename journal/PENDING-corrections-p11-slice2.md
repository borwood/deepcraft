# PENDING corrections.md entries — P11 slice 2 (the conversion)

*Ordinals are the integrator's.*

---

## A SEAM CAN FAIL BY KEEPING THE WRONG CUSTOMER, AND NOTHING GOES RED

**Claim falsified:** *"a provider slot's health is checked by its consumers — if
the seam still has readers, the heir can still arrive."*

**Where it nearly cost us:** P11 slice 2 (2026-08-02). The `outcrop_shares` slot
answers *"how much of each rock fills the near-surface window here"*, and its heir
is **structural deformation** — the whole point is that when beds dip, the dipped
shares arrive here and the erosion rate field dips with them. The slice re-graded
the erosion rate table to `MaterialId`, and for about an hour the rate path read
`lithology::exposed_member_shares` **directly**, bypassing the seam. The seam
still had four readers (the frost, wind and wave blends, and the outcrop verdict),
so no inventory and no test reported anything.

**The mechanism:** a seam is not validated by *having* consumers; it is validated
by having the consumer whose question it was cut for. Erosion's **rate** is that
consumer — `providers/outcrop_shares.rs`'s own module docs say so, twice — and it
was the one that left. The heir would have arrived, plugged into the socket, and
changed nothing about how fast anything erodes.

**How it was caught, and this is the part worth keeping:** by a *test whose
override would have gone inert*. `full_agents.rs::waves_cut_down_the_coastline`
drives a soft and a resistant coast **through the seam**, because the shipped
world's coasts are basement-heavy and the mechanism question is not a question
about this world's composition. Under the bypass that override stops reaching the
rate, and the test's assertion is **differential** — both arms move together — so
it would have stayed green while testing nothing.

**The rule this suggests, offered not asserted:** when a slice changes the *grade,
type or units* of a quantity, the seams that publish it are part of the diff. A
seam that survives a re-grade unchanged is either genuinely grade-agnostic (say
so) or has quietly lost its customer.

**Related:** `providers/mod.rs` § *"A seam's success condition is that it
DISAPPEARS"* (user, 2026-07-28) — this is the failure mode on the other side of
that sentence: a seam that survives with the wrong customer never disappears and
never delivers.

---

## "THE FLAG IS OFF" IS NOT A GUARD UNLESS IT IS WRITTEN AT EVERY READ

**Claim falsified:** *"the identity path's structures are empty when the tier is
off, so the off path cannot touch them."*

**Where it cost us:** the same slice, one suite run. `exchange_cell` and both
transport chains computed `tlayout.row(c)` **before** branching on `sorted`,
because under the dense planes `c * SPECIES` was a pure arithmetic expression that
was safe to compute unconditionally. Under CSR it is an index into a row array the
scalar-load solve deliberately never builds — so it is an out-of-bounds on every
harness that drives `Erosion` directly without turning the material tier on.

**The mechanism:** the dense→sparse conversion turned *arithmetic* into a *lookup*
at three call sites, and a lookup has a precondition an arithmetic expression does
not. The emptiness that makes the off-path byte-identical is exactly what makes
the lookup illegal.

**Caught by:** `mass_is_conserved_up_to_uplift`,
`mass_is_conserved_with_coupling_on`,
`the_differential_erosion_feedback_neither_runs_away_nor_stalls` — three tests
that exist for entirely unrelated reasons, and are the only ones in the suite that
construct `Erosion` by hand. Nothing that goes through `run_cells` could have seen
it, because production turns the tier on.
