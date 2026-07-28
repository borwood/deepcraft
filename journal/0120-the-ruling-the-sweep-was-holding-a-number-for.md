# 0120 — The ruling the sweep was holding a number for

*2026-07-28. Five decisions parked by the user in the morning, unpacked and ruled
in the afternoon. Four of the five turned out to be about the same thing: the
difference between what a document says **exists** and what it says we **want**.*

> blogworthy: **lens 1 (AI-native development)** — the entry number was reserved
> four days early by a sweep that declined to write a narrative, on the explicit
> condition that *"if #1 goes to the user and produces a ruling, **that** is the
> entry."* It did, and this is it. A deliberate gap in a numbered sequence turned
> out to be a **promise**, and the only reason it was kept is that the sweep wrote
> down *why* it was skipping rather than just skipping.
> Also **lens 3** for the middle section: what it costs when a doctrine written in
> the file every agent loads is one word wider than the thing it describes — for
> the second time in three days.

---

## The reservation

`journal/0120` did not exist, and its absence was not an accident. The 2026-07-26
`doc-topology` sweep had declined to write it, and said so in its own § *Numbered
artifacts*:

> *"`journal/0120` — deliberately NOT written; the number is unspent. This sweep
> found bookkeeping drift, four un-propagated supersessions and one genuine
> user-vs-user conflict. That is a result, not a narrative… **Recorded here so the
> gap reads as a decision, not as a lost entry.** If #1 (the ecology doctrine) goes
> to the user and produces a ruling, **that** is the entry, and it should take
> 0120."*

Two days later the wrap block repeated it — *"journal/0117 and 0120 are deliberate
gaps… not lost entries"* — and two days after that, a session opened to unpack five
parked user decisions went looking for the next free number, found **0122**, and
stopped. The gap was the tell.

This is worth more than the tidiness. A numbered append-only sequence has exactly
one failure mode that cannot be distinguished by inspection: **a reserved number and
a lost number look identical.** The project already learned that once, when journal
0105 was absent at a wrap because its agent was still running. The rule that came out
of it — *account for every gap in the close block* — is what made 0120 recoverable
four days later, by a session that had no other reason to know the number meant
anything.

## What was actually asked

The five items came from the bootstrap-history removal (journal/0121, same day, and
yes, written before this one — the removal is why there was anything to rule on).
They were parked, deliberately, with an instruction to keep them durable: *"I won't
muddy waters by addressing that in this conversation."* They were unpacked cold in a
separate session, which turned out to matter — the whole exercise is a test of
whether a consolidated ROADMAP entry can carry enough context to be actionable by a
reader who was not there. It could, with one exception, described below.

The user's framing on the first item reframed the other four:

> *"That system was not designed — it will have a designed successor at one point,
> but I had nothing to do with it. Came entirely from Claude bootstrapping the
> project, attempting to satisfy the list of things I mentioned that I would like to
> be in it **eventually**. (Maybe I wasn't clear.)"*

And then the fact that closes the appearance question:

> *"For the record, I never once saw a wooden post in the world and no images were
> captured — it predates journals and our walk protocols."*

The removal entry had treated the missing before/after pair as a limitation — *"no
screenshot is possible, the 'after' is ordinary ground."* That was the wrong frame.
**The content was never observed by the one eye that ratifies appearance, across its
entire life.** It rendered in the shipped world for a week, was defended by three
independent corpus instruments, and had no witness. Standing was never lost; it was
never acquired. What looked like accrued weight was accrued **inertia**.

## The thing four of the five had in common

The three docs still describing civilization as a live pipeline stage looked, in the
ROADMAP entry, like one finding with three sites. They are not the same kind of claim
at all, and the ruling split them three ways:

- `worldgen.md` § *Above the region scale* item 4 asserted a **pipeline stage that
  exists**. False as of two days earlier.
- `things-that-will-happen.md`'s looted sword asserts an **ambition**, in the document
  whose entire charter is one-line examples of what the game *is*.
- `ideas.md:384` mentions ruins in a bullet about **visual language**.

The user's ruling on the second is the load-bearing one, and it is a statement about
the document's standing rather than about civilization:

> *"Things that will happen is correctly 'what kind of engine this WILL BE and what
> kind of experience the default pack WILL BE'. The ambitions there are recorded with
> a high amount of user involvement and are **not claims about what we currently have
> built** as content or can support as an engine."*

So: **the doctrine governs unratified bootstrap CONTENT; it has no reach over
RECORDED AMBITION.** The tell, when they are hard to separate: *does it RUN, or does
it PROMISE?* A thing in the tree asserts it exists — challenge it. A future-tense
line in a design doc asserts we want it, and **striking it retires a goal**, which is
not a sweeper's call or a slice's.

**This is the second time in three days that this exact directive was one word too
wide.** The first was `ecology.md` — the word *"ecology"* was inserted by an assistant
transcribing the user's words, and it unmoored a doc whose header says the design in
it is the user's. That was caught by the first `doc-topology` sweep, hours after it
was written. This one would have struck a user-authored ambition, and it was caught
only because someone asked which of two readings was meant instead of picking one.

The pattern is worth naming precisely, because "be careful transcribing" is not a
mechanism: **a directive that names what to remove is executable, and a directive that
names what to remove *and implies a scope* is not.** *"They are NOTHING"* is a claim
about what exists. Written into the file every agent loads, next to *"anything in the
tree that looks like one is fabrication,"* it reads as a claim about what is wanted.
CLAUDE.md now carries both halves explicitly, plus the status correction the user gave
in the same breath: **on hold, not never.** *"We do want these systems eventually."*

## The document that turned out to be built on the thing

The ROADMAP entry named one line in `worldgen.md`. That was the exception to
"actionable cold" — and it was wrong by roughly the whole document.

History is `worldgen.md`'s **thesis**. It is in the title (*"bounded world, lazy
detail, real history"*). It is in the core decision. The extent knob is calibrated on
it — *"dozens of cultures, dense-enough history that players collide with it… sparse
history reads as empty; dense history reads as ancient."* The borders decision is
defined by its absence — *"the wilds use the lazy machinery with no history layer,
there is nothing out there to record, which is the point."*

And one consequence that a strike would have left behind, silently. The argument for
**bounding the world at all** has two legs:

1. *History is a process with long-range causal coupling… that requires a finite
   coarse world.*
2. *Boundedness also buys closure: poles, tropics, winds, currents, and tectonics as
   consequences of a closed system rather than painted-on features.*

Leg 1 is on hold. Leg 2 is pure earth science and is carrying the decision today. Had
history simply been struck, `worldgen.md` would hold a ratified DECIDED resting
visibly on a suspended premise, with nothing saying the other leg holds — and the next
reader to notice would "discover" that the boundedness argument had collapsed. It has
not. That note is now in the document, which is the whole point: **A-2 is usually
described as a justification outliving its premise, and this is the rarer inverse —
a premise going away underneath a justification that survives it for other reasons.**
Both need the same treatment, and neither is visible without writing it down.

The verb matters more than the sites. The user said *strike*; what shipped is
**ON HOLD**, because *"history is coming, eventually, for the reasons worldgen states
(not exhaustive)"* is not an instruction to delete the reasons. A strike deletes
recorded intent and buys a rebuild from scratch later. A banner corrects the
implementation claim and keeps the design.

## The gate that is deliberately not checkable

The hold needed an end condition, and the user gave one that is firm about **order**
and explicitly soft about **timing**:

> *"We have a lot of engine work and non-bio default-modpack work to do prior to
> considering ecology, then social concepts. All non-bio earth science will have to be
> in the ratified SDK-plugin shape and we will fully respect the plugin-agnostic engine
> shape: engine owns primitives (field and refinement kernels) and runner, plugins
> declare all content including fields, field+cell passes, pass order+rate, materials,
> refinement, etc: this list may not be exhaustive, and 'sufficiently complete to begin
> thinking about bio/soc' will be a user call, and each of these sections has its own
> ongoing design conversations/AC and flux in one may ripple to others."*

Three things fell out of one paragraph.

**It closed an open architectural question by accident.** `north-star.md` § *The
core/plugin boundary* had carried a red flag since 2026-07-26: **the refinement tier
is in neither column**, *"never decided, arrived at by default."* Three candidate
answers were framed, with (c) — *refinement primitives are core, refinement operators
are content, exactly as the field passes were split* — as the direction of travel. The
sentence above **is** (c). It is now recorded as decided, with its provenance stated
plainly: the user gave it while ruling on sequencing, not as a deliberate adjudication
of three candidates, so the ruling is marked as such and is contestable on that basis.
Recording it silently as a considered answer to a question the user was not looking at
would be corrections #65 in a new coat.

**It flagged its own incompleteness.** *"This list may not be exhaustive."* So the
boundary is decided and its inventory is not — and both documents now say so, rather
than presenting the new list as closed. This is the fourth known instance of the same
gap: **no enumeration in the docs is checked for completeness**, while the *code* layer
has had that control for months (`build_checked` refuses to build a world if a class a
pass selects from has zero members — which north-star calls the entire meaning of
"validation by construction"). Every instance so far was found by a human noticing.
Filed as a pattern, deliberately not as a build: the ratified doctrine here is *"the
general registry is deliberately unbuilt — four conversions is not enough to design one
from,"* and four hand-found instances is the same evidence base.

**And it refused to be a checklist.** Each section *"has its own ongoing design
conversations/AC and flux in one may ripple to others"*, so *"sufficiently complete"*
is a **user call** — not a test, not a threshold, and not something an agent may
declare met by tracing code. Which is the sequencing form of the same rule the whole
day turned on: **existence is not standing, and neither is progress.** No amount of
shipped earth science entitles anyone to open the social thread.

## Immutable body, mutable header

The fifth decision was the one with the widest reach, and the only one whose cost was
already paid.

Two rules, both live, in different files, never reconciled. `corrections.md` #12:
*"`S10-results.md` is left unamended — a spike result is a dated record of what was
measured; **this entry is the pointer**."* `CLAUDE.md` read-first item 5: *"Spike
results live in `docs/spikes/` — **measured numbers, don't re-guess them**."* Go read
the spike and trust it, versus the spike is frozen and the correction knows better.

The bill: `S10-results.md`'s cost table says enabling biology takes world creation
from 15.17 s to **25.19 s**. Production measures **13.79 s** — the spike drove the
scalar path, production takes the parallel one. **A user ratified a ship decision on
the 25.19 s figure**, and for eight days `S10` held no reference to its own correction
while read-first sent every cold session to it.

The ruling: **the measurements are never rewritten, and the doc must carry a banner
pointing at whatever refuted it.** Testimony about a day is not edited; the header is
not testimony.

The half that decides whether this survives is *who* writes the banner, and it was
chosen from measurement rather than taste. **The obligation lands on the author of the
correction, in the same commit** — while both files are already open. This week's
reading pass produced the relevant law after falsifying its own first version of it:
*a convention is adopted when it is inseparable from an act the author must perform
anyway, and dies when it asks them to restate in a second notation something already
said in prose.* The counter-example is in the repo: `JUSTIFIED-BY`, documented in
`spines.md` § 5 **and** in a skill's check list, with a promised sweep — **3
occurrences, 0 in `crates/`**.

What it fixes is structural rather than clerical, and the notebook had already named
it: **a one-directional pointer is not a pointer, it is a note to whoever already
found the answer.** Measured corpus-wide, **8 of 15** correction→file edges run one
way and **14 of 30** audit/spike files carry no staleness marker at all. A chain of
authority cannot be walked from the stale end if the stale end holds no link — and the
stale end is exactly where a cold reader enters.

Two banners shipped (`S10`, `S2`). The corpus was **not** swept, and the backlog is
filed with a note that matters more than the backlog: **the policy's real test is the
next correction written, not the 14 files behind it.** If the same-commit obligation
holds, the hole stops growing and the remainder is finite. If it does not, sweeping is
treating a symptom — and we find out within a week, for free.

## The one that was not a decision at all

The S2 statistical tier — the constraint-ledger prototype, its toy world, and the
append-only fact machinery — lost its last production caller when the history pass
died. The ROADMAP framed it as a fork: keep it as a shape reference, or dispose of it
as the same class of thing as the content it served.

The user's answer refused the fork:

> *"I didn't even know this system existed… The statistical system is genuinely
> intended, though I can't say whether as-is it will fit the desired shape when we
> actually do move on to implementing the civ/socia part of the default pack and the
> engine affordances. Keep the primitive and toy."*

The operative half is not the keep. It is the instruction that whoever finds it must
read **the code that read it is gone · the primitive is the deliverable · it is a
candidate to be re-checked against requirements that do not exist yet · and when that
thread opens is a user call.** That note now sits in all three places a reader
actually arrives — the module doc, the spike's banner, the spines row — because a
ruling that lives only in a ROADMAP entry is a ruling nobody will find.

And it forced a structural admission out of `spines.md` § 3. The index knew one exit
— *consumed*, the good event — and journal/0121 had just added a second, *deleted*.
This row is neither. It is **held as a candidate**: ratified as wanted, with nothing
yet to judge it against, so it is neither owed a consumer nor eligible for disposal.
Without that third state a reader assumes the first and goes hunting for a consumer
nobody wants found — which is precisely the reading that, six days ago, had three
working instruments arguing to keep 102 wood posts that no one had ever seen.
