# ROADMAP staleness sweep — 2026-08-03 (FULL)

**Status: IN PROGRESS (skeleton committed early per the skill's dispatch rule).**

*What this sweeps:* the live board — `ROADMAP.md` § In flight / Sequenced / Observed and
the two live close blocks — against **what has shipped since the recorded watermark**.

- **Watermark at start:** `d8407e1` (2026-08-02 sweep).
- **Swept to:** `2985273` — every quotation below was read at that commit unless noted.
- **Delta:** `git log d8407e1..HEAD` = **143 commits**.
- **Mode:** FULL. Two triggers fired: the watermark is past the ~5-active-day / commit-count
  threshold, and the board was restructured in the window (a close block archived, two new
  close blocks written).

**NO EDITS WERE MADE to `ROADMAP.md`, to any design doc, to `journal/`, or to
`corrections.md`.** This sweep is read-only except for this file and its watermark entry in
`docs/audits/.sweep-watermarks.json`. **The integrator adjudicates every finding below.**

## Sections

1. Recalibration scan (run BEFORE walking entries)
2. Findings, ranked by value
3. CANNOT-DETERMINE
4. Coverage and honest limits

*(body to follow in amend commits)*
