# Build: fanning out the candidates

Step 3 of the loop, when N is more than one. One agent per candidate, in parallel.

**This file is only reachable after the theses are approved.** Hard rule 2 is not relaxed by
parallelism — it is the reason parallelism is safe here at all. An agent handed an approved thesis
is executing a spec; an agent asked to invent one is sampling, and N samples converge.

---

## Why the theses are never fanned out

The skill's own argument, and it is the reason this file starts at step 3 rather than step 2:

> Theses must be **spread on purpose**, not sampled. Independent generation converges on one
> default — that is what makes AI design recognisable.

Three agents writing theses independently produce one design three times, each convincingly
argued. The axes are picked once, by one thread, with the bans in front of it. Only the building
divides.

The same bar applies to anything else that decides what the candidates _are_: the shared subject,
the fixture data, the stress content. One author, before the fan-out.

---

## What main does before launching anything

1. **Write the shared scaffolding.** The subject the candidates are variants of, the fixtures they
   draw, any harness they all mount in. Every agent is told these exist and that writing its own is
   out of scope. Skip this and three agents write three fixture sets that differ in ways nobody
   chose, and the comparison is no longer of the theses.
2. **Fix the axis values as numbers.** "Dense" is not a brief; `28px rows, 8px gutter, no radius
above 4px` is. Anything left as an adjective comes back as three different numbers.
3. **List the shared mappings by name.** Every `Record<State, …>`, every token map, every constant
   that more than one drawing reads. The brief says: import these, write none of your own.
4. **Decide the file each agent owns**, and say so. One prototype file per candidate. Where the
   board holds the approved drawing as a file, main starts the candidate from it before the agent
   runs (`bun workbench new <id> --from <drawing>`), so the agent builds on the drawing the user
   saw rather than typing it again.
5. **Write each thesis's numbers into its thesis file.** `new` creates the file empty beside the
   candidate. Main fills it from the approved board before any agent runs: every figure the board
   states, as a check, with the names of the parts the agent has to mark. An agent never edits its
   own thesis file. The numbers are what it is checked against, and a check written after the build
   is written by the build.

---

## The brief every build agent gets

Verbatim, not summarised. An agent missing any of these is the one that produces the round's
rejection.

| Item                        | Why it cannot be left out                                                                                                               |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| **Its own thesis, in full** | Including the axis numbers, and the board's file path with the section that draws it. It builds that drawing and nothing adjacent to it |
| **The bans file, to read**  | Not a summary. A ban's exact wording _is_ the check — "no shadow on any surface" is checkable, "keep it flat" is not                    |
| **The system doc, to read** | Once a system has settled, a candidate that is unbanned and still wrong is one that never read it                                       |
| **The one file it owns**    | Its blast radius inside the blast radius                                                                                                |
| **Its thesis file**         | To read, never to edit. It marks the parts the file names, and runs the two commands on its own candidate before it hands back          |
| **The shared mappings**     | Named, with the instruction to import rather than write. See below                                                                      |
| **Rule 0**                  | Never write outside the design root. N agents are N chances to get this wrong                                                           |
| **What it must not do**     | Not register itself, not touch shared code, not edit another candidate, not improve its thesis, not change a number in its thesis file  |

**An agent hands back with the last lines of both commands**: the thesis's numbers and the sweep,
run on its own candidate (`PROJECT.md` names them). A number not kept is fixed before the hand-back,
not reported as a finding for somebody else. That is the whole of the checking a build agent does,
and it is why nothing has to review the candidate before the user sees it.

**Reading the bans is not summarisable and not skippable.** It is the largest single cost of the
fan-out and it is the cost that buys the whole thing: an agent that skips it re-proposes what died
in round 2, and the round is spent rediscovering that.

**It is, however, scopable — and the brief does the scoping.** `PROJECT.md`'s `ban find` prints the
sections that apply, _verbatim_; it is a selection, never a summary, and the difference is the whole
point. A round on one component reads its own section and the standing ones rather than §1 through §12, which is ~14k
tokens instead of ~40k, three times over.

Two rules on it, and the second is the one that makes the first safe:

- **Main names the sections in the brief**, by number. An agent that chooses its own scope is an
  agent deciding which rules apply to it.
- **The command always prints the settled and pre-seeded sections whatever was asked for**, and
  names every section it left out. A selection that can silently drop a binding rule is worse than
  the 40k read it replaced.

When a round genuinely crosses subjects, the answer is the whole file. Scoping is for the common
case, not a licence to narrow a round that is actually wide.

---

## The failure this shape makes worse

`TRAPS.md`'s **one enum, two drawings** was measured on a single thread: two components drew the
same state in two hues because one author wrote the mapping again from memory. Both passed their
own review, because a candidate is checked against its thesis and neither thesis mentioned the
other drawing.

Three isolated authors make that more likely, not less. Two mitigations, and use both:

- **The brief names the shared mappings** and forbids writing one.
- **`PROJECT.md`'s census command** is run at fan-in, before anything is compared. It reads the
  source and reports any state painted from more than one place.

---

## Fan-in, in this order

1. **Register the candidates**, where `new` did not already before the build. It should have:
   `bun workbench new <id> --title "…" --thesis "…"` writes the candidate file, its thesis file
   and the registration in one step, so the shared `proto/index.ts` is no longer a text edit that N
   agents can race. Main runs it — the list is one file and one author is cheaper than a merge.
2. **Restart the workbench** if the space enumerates its files with an eager glob. A brand-new
   prototype will not appear until it does, and debugging its absence is a wasted half hour. `new`
   prints the reminder; the workbench refuses to boot on a taken port rather than drifting to the
   next one, so a restart that appears to work now actually has.
3. **Run the census.** Any state painted twice is fixed before the round is looked at.
4. **Run the thesis's numbers and the sweep** across every candidate, once more, from main. An
   agent's own run was on its own file; this one is after everything landed together.
5. **Run the probes** across every candidate at every width. A new candidate has no baseline, so
   every finding on it is new by construction — which is correct, and is what the round has to
   answer for.
6. **Then the comparison.** Nothing reviews a candidate before the user sees it. The short read is
   after the pick, on the winner only (`REVIEW.md`).

---

## When not to fan out

- **N is 1.** A single candidate is a correction or an iteration; the overhead is the whole cost.
- **The candidates share most of their surface.** Rounds that argue one channel — a palette on one
  identical drawing — are one file with a variant bar, not N files. Fanning them out produces N
  copies that drift on axes the round was holding still.
- **The round is a `CORRECT` gear round.** Items are small, and they route _down_ to the layer that
  caused them — which is shared code. Two agents fixing two items in one kit component is a
  conflict for a few minutes saved.
