# Tour map — the three stacked appearance changes (2026-08-02)

> **Status banner (read first).** Nothing has refuted or re-scoped this yet. A dated
> measurement artifact: **immutable body, mutable header** (CLAUDE.md read-first item 5).
> If a later slice moves the surface top-span distribution, the octaves dither's reach, or
> the identity-provenance share, **stamp the pointer here, in the same commit as the
> correction.**

Instrument: `crates/dc-worldgen/examples/appearance_tour_p11.rs`
(`cargo run --release -p dc-worldgen --example appearance_tour_p11 -- --stride 3 --top 3`,
~97 s). World: `BENCH_SEED` 1337, `Extent::Medium`, default `DeepOverrides` — the world
`dc-client` boots. Deep grid 545×545 @ 460 m/cell. Sampling stride 3 deep cells → **31,684
interior cells sampled, 4,792 on land** (near-field `surface_elev_m > 2 m`, i.e. the surface
the player stands on, not the deep bilinear plane).

Every station is filtered by the **expression path**, not by the record: the surface voxel
is `ColumnFill::plan(1)` and the voxel `d` below it is `plan(d + 1)` (`collapse.rs:526`,
`:575`).

---

## The mechanism that decides all three stations

`StrataEvent::dither` splits the record in two, and it is what makes these three changes
visible in three *different* places:

| | `dither` | member decided | where the walk sees it |
|---|---|---|---|
| year-zero veneer (clastic veneer, igneous bodies, placer, weathering front) | `true` | re-picked per voxel column through `selection_field` = `draws::Octaves` | **the octaves slice, and only here** |
| deep-time history (`deposit_deep_history`) | `false` | recorded `DepUnit::species`, fitness at deposition | **P11's member diversity and provenance** |

---

## Station set 1 — octaves near-member stepping: **SURFACE NULL**

**0 of 4,792 sampled land columns** express `Plan::Single` on a movable (`dither: true`,
≥2-member) event **at the surface voxel**. The surface top span is:

| span | columns | share of land |
|---|---:|---:|
| `Mixed` | 4,750 | **99.1 %** |
| `Single`, frozen (`dither:false` or 1-member class) | 20 | 0.4 % |
| `Single`, MOVABLE | **0** | **0.0 %** |
| no record | 22 | 0.5 % |

`mixed_at` resolves a `Mixed` span from the **undithered** `event.member` by design. So the
octaves member dither **cannot move the ground the player stands on anywhere on this
world.**

**This settles the standing ROADMAP § Observed question** (*"whether member stepping was
ever U3's DOMINANT signal is UNSETTLED… Owed before any game time: a tour map that finds a
chunk whose surface top span is `Single` in a two-member class"*). That chunk **does not
exist on the shipped world**. journal/0129's observation at the U3 reference pose — *"the
surface top span is `Mixed` for all 1024 columns"* — was not an unlucky pose. **It is the
universal case, at 99.1 %.**

**Positive control (the null is not an unproven census).** The identical classifier finds
**3,526 of 4,792 land columns (73.6 %) with a movable `Single` span BELOW the surface**,
343,625 spans in all. The instrument sees the category; what is absent is the category *at
the surface*.

**And the ceiling is low even buried.** Of the columns whose movable span actually draws
**both** members over a 115 m window — 1,163 of 4,792 (24.3 %) — **every single one is
`dc:stratum/igneous-extrusive`, i.e. andesite vs basalt: an albedo-adjacent pair.** Not one
is the grain-size pair (sandstone/conglomerate) that the eye could adjudicate.

### Buried stations (need a bench cut; ranked, then re-verified)

| | world (x, z) m | surf m | span depth | mix (octaves) | before → after minority | 28.8 m lattice curvature ratio |
|---|---|---:|---|---|---|---|
| S1 #1 | (−12866, −19765) | 1082.2 | `plan(7)`, 5.4 m | basalt 65.2 / andesite 34.8 | 0.351 → 0.348 | 1.68e14 → **2.08** |
| S1 #2 | (28531, −446) | 942.5 | `plan(9)`, 7.2 m | andesite 55.3 / basalt 44.7 | 0.253 → **0.447** | 7.75e13 → **0.062** |
| S1 #3 | (39569, −45982) | 31.6 | `plan(3)`, 1.8 m | andesite 55.0 / basalt 45.0 | 0.230 → **0.450** | 9.42e13 → **1.30** |

The lattice-curvature collapse (≈1e14 → O(1)) is journal/0129's headline, re-measured at
these sites rather than remembered.

**Contrast: free, no travel.** The `Mixed` surface directly above each cut *is* the
contrast — climb out of the bench.

---

## Station set 2 — member diversity in the deep record

Two distributions, and **they disagree by two orders of magnitude for a real reason**:

- **expressed** (the walk's own instrument): **531 of 4,792 land columns (11.1 %)** show at
  least one **RECORDED** within-class member contact on a cut face — both sides
  `dither: false`. Max 11 contacts in one column.
- **record-side, conservative**: **14 of 43,875 interior land deep cells (0.032 %)** stack
  two members of one class in bands each ≥ 0.9 m. Max 2 alternations.

The gap is the quantizer, not an error: the record is finely laminated (S2 #1 holds **246
coalesced bands**, most of them centimetres), and a sub-voxel band still wins eighths inside
a `Plan::Mixed` span and can be the dominant share at some depth. Both numbers are true of
different questions.

Contacts drawn by the **veneer's** own dither are counted separately (217 columns, 4.5 %)
and are *not* evidence for P11.

### Stations

**S2 #1 — the strongest exemplar on the world.** world **(82346, 13352) m**, deep cell
(435,285), voxel (91495, 14836), chunk (2859, 463), surface **236.6 m**.
**11 recorded within-class contacts, 0 dithered, 0 movable spans** — the face is *entirely*
recorded identity. Expressed column (33 voxels of record):

```
d  1..4   0.0 m   carbonaceous-mudstone
d  5..7   3.6 m   sandstone
d  8..11  6.3 m   conglomerate
d 12..14  9.9 m   sandstone
d 15..15 12.6 m   conglomerate
d 16..17 13.5 m   sandstone
d 18..18 15.3 m   conglomerate
d 19..20 16.2 m   sandstone
d 21..21 18.0 m   conglomerate
d 22..22 18.9 m   sandstone
d 23..25 19.8 m   conglomerate
d 26..27 22.5 m   sandstone
d 28..29 24.3 m   conglomerate
d 30..30 26.1 m   siltstone
d 31..31 27.0 m   carbonaceous-mudstone
d 32..33 27.9 m   conglomerate
```

Bench: `world_fill dc:air x [91488, 91519] y [239, 262] z [14796, 14836]` = **31,488
voxels**, one chunk wide in x. Exposed face at voxel z = 14837.
Pose: `pose_set{x: 82346, z: 13334, surface: true}` → read live ground Y →
`pose_set{x: 82346, y: 217, z: 13334, yaw: 3.142, pitch: -0.05}`.

**S2 #2** (32670, −45982) m, surface 133.7 m, 5 contacts.
**S2 #3** (3694, −37703) m, surface 686.5 m, 4 contacts.

---

## Station set 3 — provenance over site climate

**(a) The instrument, on the record.** `DeepConfig::identity_audit` on the shipped world
reproduces the slice-2b headline exactly:

| | metres | share |
|---|---:|---:|
| identity from the ARRIVING COMPOSITION | 507,515.9 | 77.21 % |
| identity from the FITNESS DRAW | 149,825.6 | 22.79 % |
| …transported metres the SITE would have named differently | **209,670.4** | 41.31 % of transported, **31.90 % of all record** |

Self-check: the audited run's record matches the pregen field the walk renders on
**3063/3063** sampled cells (the audit is bit-inert).

**(b) Localized — and it is rare at the surface.** The aggregate audit exports no per-cell
plane, so the probe re-evaluates the same counterfactual per near-surface band, at the same
draw address (cell, chapter, `TRANSPORT`, 0), under the site's **present** climate. *Stated
rather than assumed away: the in-run audit asks the question under the depositional epoch's
climate, so a post-hoc disagreement can come from provenance OR from climate drift; the
class-vs-tag column separates the two per station.*

**36 of 2,420 expressible bands in the top 30 m (1.49 %), across 28 land cells.**

| | world (x, z) m | surf m | recorded → site's counterfactual | contrast | thickness | expression |
|---|---|---:|---|---|---:|---|
| **S3 #1** | (121901, −3206) | 88.7 | **sandstone → conglomerate** | grain-size (loud) | 0.91 m | **the SURFACE VOXEL, `plan(1)` — no cut** |
| S3 #2 | (−11485, 8752) | 295.2 | conglomerate → sandstone | grain-size (loud) | 0.92 m | `plan(2)`, 0.9 m down |
| S3 #3 | (4613, −28504) | 1053.1 | sandstone → conglomerate | grain-size (loud) | 0.99 m | `plan(2)`, 0.9 m down |

S3 #2 additionally has the **class** the environment tag implies differing from the class
recorded — the load's class outvoted the site's environment. S3 #1 and #3 disagree at
**member** grade only.

S3 #1 pose: `pose_set{x: 121901, z: -3206, surface: true}` → read live ground Y →
`pose_set{x: 121901, y: 90, z: -3206, yaw: 0.0, pitch: -0.5}`. Then `world_get_contents` at
the feet voxel — **never `scan_region`** for a material question.

**Contrast**: recorded == counterfactual, same class, **0.5 km away** at (121901, −2746) m,
surface 103.6 m, recorded sandstone.

---

## Inter-station distances

S1 ↔ S2 100.8 km · S1 ↔ S3 135.8 km · S2 ↔ S3 42.9 km. Teleports are free.

## Conventions this run pinned down

- **Yaw is Bevy's**: view dir at yaw θ is `(−sin θ, ·, −cos θ)` (`dc-client/src/player.rs:57`).
  **yaw 0 looks toward −Z; yaw π toward +Z.** The first draft of every road-cut pose here
  faced the hillside behind the player.
- `Litho::code()` is `"coarse"` / `"fine"`, **not** the namespaced class id — matching the
  string `"clastic-coarse"` scored every sandstone-vs-conglomerate station as
  albedo-adjacent, the exact inverse of the truth.
- A bench cut must keep its **floor above sea level**; S2 #2's first sizing put the player
  at y = −4.9 m.
