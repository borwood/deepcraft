# The tax had a name and a textbook

*2026-08-03 · P2's gen-time conflict → the E4 design pass
(`docs/audits/2026-08-03-e4-implicit-kernel-design.md`) → E4-1, the
byte-identical kernel extraction (merge `62b0449`) · stubs #30 discharged*

> blogworthy: **procgen dev against the backdrop of priors** — the moment a
> "simulation is just slow" problem resolved into a named numerical-methods tax
> with a forty-year-old literature fix; and **AI-native development** — a design
> pass that read the operator before proposing the scheme, and a build slice
> whose entire gate was "nothing may change."

## The complaint and the diagnosis

The P2 measurement ladder put honest erosion rates within reach and priced
them: 38 minutes of solve per Medium world at the literature-band calibration,
against 41 seconds shipped. The user: *"this gen time is actually unacceptable
if true. there has got to be ways we aren't thinking of."*

There were, and the ladder itself had already printed the diagnosis without
anyone reading that column: sub-steps per epoch rise from 2 to 896 across the
calibration range — linearly with the rate multiplier. The hillslope operator
is an explicit diffusion scheme, and explicit schemes carry a stability speed
limit that shrinks as rates grow. The cost is not physics. It is CFL tax, and
the world pays it ~180,000 sweeps at a time.

## The design pass, and the fact that decided everything

One reading of `creep_kernel.rs` settled the space: the flux law is **linear**
— Culling diffusion on the surface, heterogeneous coefficient frozen per
epoch, donor-upwind selection. No Roering nonlinearity, no depth dependence.
Which means no Newton machinery, no coupled solve: the implicit fix is the
textbook one, and the textbook is ours already — the same FastScape lineage the
calibration literature comes from solves hillslope diffusion implicitly at
exactly these rates.

Two rejections worth keeping: Peaceman–Rachford ADI (A-stable but not L-stable
— the grid-scale checkerboard journal/0122 killed would return through an
"unconditionally stable" front door), and the one-shot flux clamp (which
quietly reinstates the one-cell-per-epoch conveyor of corrections #27). The
recommended kernel is backward-Euler factorized ADI with today's limiter
arithmetic kept as a final projection — mass-exactness and h ≥ 0 guaranteed by
construction, never by convergence. Projected: the 38-minute solve returns to
roughly a minute, the pregen-budget conflict dissolves, and the deferred pit
safari becomes an errand.

## E4-1: the slice whose gate was silence

Extraction first, scheme second. E4-1 lifted the explicit kernel out of the
pass and behind `dc-core::field`'s flux-form API — `plan` derives the sub-step
count from the kernel's own von Neumann bound, so **a pass can no longer state
a sub-step count at all**; the unsafe call became inexpressible, which was
always E4's charter. The gate was byte-identity: 67 suites, 657 tests, every
golden fixed point by name, zero moved. A bit-level test pins the flux-form
gather to the pre-extraction arithmetic under limiter binding, zero
coefficients, and borders.

stubs #30 — "the operator still sub-cycles in the pass" — discharged after
living in the inventory since the operator repair. The venue (`dc-core::field`,
which brings rayon into dc-core) rides as NEEDS RATIFICATION. E4-2 lands the
implicit scheme non-default with its convergence study; E4-3 flips adoption
together with the P2 calibration re-pick, so the goldens move once for both —
and the D3(M) curve gets re-measured under the new integrator before any flip,
because it was measured under the old one.
