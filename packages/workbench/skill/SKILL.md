---
name: design
description: The design loop for a repo with a design workbench — an interview and research, theses drawn on a board, prototypes, side-by-side comparison, reaction, bans, iteration. Covers settling a new design system, designing the frame that holds the app, redrawing a view, or reworking a single component, all inside the project's design root with the live app never touched. Use for design, redesign, restyle, new look, new layout, theme, design system, prototype a screen, mock up a view, make this view better, polish this component, rethink this page, fix a design, design workbench, proto, thesis, board, artifact, mobile rules, visual direction.
---

# Design

The loop for designing anything in a repo that has a design workbench. One process; scope is a
parameter. Nothing in this file is specific to a project — `PROJECT.md` carries that, and it is the
only file here to rewrite when the skill is copied somewhere new.

**Load `frontend-design` alongside this.** The two do not overlap: `frontend-design` supplies
aesthetic calibration and the catalogue of generated-UI tells. This file supplies the loop, the
invariants, and how to drive the workbench.

---

## Open the session

Four reads, in this order, before anything else:

1. **`PROJECT.md`** — where the design root is, how to start the workbench, what the project calls
   its own docs. Everything below refers to those by role, not by path.
2. **The project's bans** — the accumulated rejections. Every session, without exception. It is the
   thing that stops session 9 re-proposing what died in session 2. Where `PROJECT.md` names a
   command for reading them by subject, use it and name the scope: it prints the matching sections
   **verbatim** — a selection, never a summary — and says which ones it left out. Read the whole
   file when the round crosses subjects, and whenever you are not sure it does not.
3. **The project's system doc**, if the system has settled. The bans are the negative record; this
   is the positive one, and building against a settled system without reading it produces a
   candidate that is technically unbanned and still wrong.
4. **Exactly one scope file** — `SYSTEM.md`, `FRAME.md`, `VIEW.md` or `COMPONENT.md`.

`WORKBENCH.md`, `TRAPS.md` and `PROBES.md` are reference: open them when you are driving the stage,
debugging a layout, or checking work, not up front.

**Then start the workbench and leave it up.** Every command below addresses it, boot cost is near
zero, and a round that starts it four times has paid the startup four times.

---

## Rule 0 — blast radius

**Never write outside the design root named in `PROJECT.md`.**

Not the live app, not the API, not a migration. The redesign is a parallel universe. The current app
is the before-picture and must keep rendering exactly as it does today.

This skill's own files are the one sanctioned exception, and only when the user is working _on the
skill_.

Migrating a settled design into the real app is a **separate project**, requested explicitly, with
its own plan. It is never a step at the end of a design session, and it is never implied by "I like
it."

Reading the live app is fine and often necessary — that is how you know what the baseline looks like.

Boards (`BOARDS.md`) are not an exception either: they are written in the session's scratchpad and
published as private Artifacts, outside the repo entirely.

---

## Two gears

Announce which gear you are in, in one line, before doing anything. The user can always overrule.

| Their message contains                                                                                      | Gear        | What you do                                            |
| ----------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------ |
| the answer — "rows should be 28px", "drop the shadow", "#3 this should say the app name, not the page name" | **CORRECT** | Read `CORRECT.md` and run that. No theses, no options. |
| a problem — "this feels cramped", "the empty state is sad", "I don't like this page"                        | **EXPLORE** | Run the loop below.                                    |

When genuinely ambiguous, ask in one sentence rather than guessing — a wrong EXPLORE wastes their
time, a wrong CORRECT throws away the whole point of the apparatus.

**CORRECT is the common gear.** Most rounds after a system settles are numbered corrections against
something already on screen, not fresh candidate sets. It has its own file because it has its own
procedure, and doing it by feel is how a proto's bug gets fixed in a settled component.

---

## The loop (EXPLORE)

```
1  BRIEF     scope, research and an interview, woven together, one question at a
             time, until the brief can be said back and the user agrees
2  THESES    N bets drawn on a board (an Artifact, BOARDS.md), on the project's real
             data at every breakpoint, spread on named axes
             ^^^ STOP. The user kills or redirects here. A board is cheap; building is not.
             On approval each thesis's numbers are written down as checks, before any code.
3  BUILD     N candidates, in the design root only — one agent each (BUILD.md)
4  CHECK     commands, run by whoever built it, on every candidate: the thesis's
             numbers, the sweep, the probes, the census. Under a minute. Never aesthetics.
5  COMPARE   side by side in one viewport, with the current thing as baseline
6  REACT     the user says what they like and don't
7  LOG       every rejection → the bans file, as a concrete rule
   READ      meanwhile, one fresh reader on the winner only: the change against the
             bans, with no browser (REVIEW.md)
8  ITERATE   winner + explicitly named borrowings → back to 3
9  SETTLE    promote, rebuild the callers on it, fold the round's bans
```

### Step 1 — the brief is a conversation

A thesis can only be spread on the right axes once it is clear what is being asked. So step 1 does
not end at naming the scope. It ends when the user and you agree on the brief, and it gets there by
weaving research into an interview rather than running one after the other.

- **Interview the way `grill-me` does**, and invoke that skill where it exists. Ask one question at a
  time, each with your recommended answer and the reason for it. Walk the decision tree in order:
  a question whose answer depends on another waits for it. Never a batch of multiple-choice
  questions; the user rejected that outright (_"When asking me questions use /grill-me"_).
- **Answer from the repo before asking.** The bans, the system doc, the fixtures and the baseline
  answer most "what does it show" questions. Ask only what only the user can decide: what matters,
  what may break, what they want to feel.
- **Research when a question cannot be decided blind.** How do other fields draw this? Put what you
  find on a research board (`BOARDS.md`), redrawn on the project's data, and ask the next question
  against it. Research that is not shown does not inform the user, and a question that is not
  grounded in anything asks them to guess.
- **What the brief has to settle before step 2:** what the thing must answer and for whom; what must
  jump out first; which standing rules and bans the round may break, each named; the data it is
  drawn on; the widths; what is out of scope. And the altitude, which is step 1's old job: pick it
  from what is actually wrong (see **Scope** below).
- **It ends when you can say the brief back in a few lines and the user says yes.** Those lines head
  the thesis board.

### Step 2 is the one that matters

Never write a line of UI in the design root before the theses are approved. A board costs minutes to
redraw; a built candidate costs a session. Each thesis is:

- **A name and a one-line bet.** "Instrument panel — status colour carries the whole hierarchy."
- **The axis values it commits to.** Stated as numbers where numbers exist.
- **Those numbers, written down as checks the moment it is approved.** A row height, a gap, a line
  count, "no shadow": each one a line in the candidate's thesis file, which `PROJECT.md` names. The
  thread that holds the approved board writes them, before any agent builds, so a build cannot bend
  a number it was handed. A thesis whose numbers are still adjectives is not ready to build.
- **A drawing on the thesis board.** Drawn with the project's real components wherever it has a
  board kit, on its real data, at every breakpoint. It forces a structure to be proposed rather
  than filtered out by what is easy in flex, and it lets the user judge the structure as seen
  rather than as imagined. ASCII
  layouts are retired for theses (_"Instead of showing me some weird ascii representation of what
  it wants to build it should draft what it wants in an artifact"_). `BOARDS.md` is the contract.

Theses must be **spread on purpose**, not sampled. Independent generation converges on one default —
that is what makes AI design recognisable. Pick the axes first, then make sure no two theses share a
value on the same axis by accident.

**An axis that does not show in a screenshot is not an axis.** Three candidates that differ only in
what happens _while you work_ draw the same thing at rest, and the round offers one design three
times. If the difference is behavioural, the candidate has to do the behaviour — a staged
interaction, or a filmstrip of pinned frames — and so does its drawing on the board.

**One channel on its own is not a candidate for shipping, only for comparison.** Isolating what each
channel contributes is a legitimate technique for a round; the isolated variants are deleted with it.

### Step 3 — one agent per candidate

Candidates are independent by construction: separate files, no shared writes, and the thesis has
already fixed what each one is. So build them in parallel — it is the longest phase of a round, and
it is the only one that divides cleanly.

**It divides only because step 2 already happened.** An agent handed an approved thesis is executing
a spec. An agent asked to invent one is sampling, and N samples converge on the same default — which
is why the theses are written by one thread and never fanned out.

`BUILD.md` carries the contract: what main writes before launching anything, what every agent's
brief must contain verbatim, and the order of the fan-in. Read it before launching the first agent.

### Step 4 — commands check every candidate

Step 4 is run by whoever built the candidate, while it is built and again at fan-in. It is four
commands, it takes seconds to a minute, and `PROJECT.md` names each one.

1. **The thesis's numbers.** One command holds the live frames against the thesis file at every
   width and prints what was not kept. The thesis said 32px rows; did they drift to 36? It said
   depth by lightness; is there a `box-shadow`?
2. **The sweep.** One command presses every control at every width and prints only what went
   wrong: a press that moved nothing, a keyboard left on the page body, something Escape does not
   close, text wider than its box. It is "the thing has been used, not looked at" without a hand.
3. **`PROBES.md`** — the measured floor: sideways scroll, overflow, hit-testing, visible focus, AA
   contrast. Every expensive miss in this loop's history was a measurement failure, not a taste
   failure. Run them; do not eyeball them.
4. **Cross-drawing consistency** — every state the system has, and every component that draws it.
   A probe is scoped to one frame and a thesis to one candidate, so a state drawn in two hues in
   one frame passes both. `PROJECT.md` names the command that reads this off the source.

**Every finding is fixed or answered before the comparison.** A finding that is a design and not a
defect is written down with the command's `--accept`, after it has been read. A new candidate has
no baseline, so everything printed for it is new.

**These checks are independent of the build without a second session.** The thesis file was written
before the build started, and a sweep has no intent to read the screen through. That independence
is what a reviewer in a fresh session used to be for. Thirty-three of those reviews took eleven
minutes at the median and forty-two at worst, three quarters of it spent catching up on what the
author already knew and writing measuring scripts that were thrown away. About six findings in ten
were things a command prints.

### After the pick — one short read

The other four in ten need a reader, and only a reader: a ban broken by its meaning, a second
hand-written copy of something the kit already draws, behaviour the brief described and the code
does not do, a doc or a comment gone stale. None of them needed a browser.

So once the user has picked (step 6), **one fresh reader reads the winner's change against the
bans**, while the round's rejections are being logged. It is handed the change, the ban sections
that match, the thesis and the brief. It opens no browser, writes no script, edits nothing, and
returns only what fails. `REVIEW.md` is its contract.

- **The winner only.** A candidate the user did not pick is deleted, and a review of it bought
  nothing. Four rounds reviewed three candidates each and threw two of the three reviews away.
- **Never before the pick, and the user never waits on it.** Its findings ride into step 8.
- **Not for a correction that touches no shared code** (`CORRECT.md`).
- **Nothing settles with one of its findings open.**

**Never score candidates against a general quality rubric.** Three orthogonal bets judged against one
standard converge into one well-formed average, which destroys the only thing the spread bought.
Aesthetics are the user's call, always. An instruction to rank, score, rate or pick is the tell that
a brief has drifted back into one.

### Steps 3, 4, 7 and 9 are mostly clerking, and clerking has commands

`WORKBENCH.md` § The verbs lists the workbench's commands: reach for them instead of editing files
by hand. Creating a candidate, deleting one, promoting one, appending a ban, measuring a thesis's
numbers, sweeping a frame and checking the round are each one command, and each of them was
previously a multi-file text edit that had to be got right from memory. The measured cost of not
doing this: ~80% of a round's tool calls in the shell, and ~96 ad-hoc scripts per round doing
surgery on two files.

**This changes nothing about what you decide** — only about who types it. The theses, the spread,
the winner and the wording of a ban are all still yours and the user's.

### Step 7 — bans are the real output

Every rejection becomes a concrete rule in the project's bans file. Not "too heavy" — `no shadow on
any surface; depth is lightness delta only`. Three standing requirements:

- **Written in the session the rejection is voiced.** Never later, never from memory. Where
  `PROJECT.md` names a command for appending one, it puts the entry in the right section and warns
  when a rule carries no round number.
- **Checkable.** A rule you could hold a screenshot against. Where it is not obvious, one clause on
  why, and the user's own words in parentheses.
- **A rule that reverses an earlier one strikes it through and names the round that killed it.**
  Never delete the old line: a ban that vanishes gets re-proposed, and a struck one records that the
  ground was already walked. Say plainly whether the new rule _revokes_ the old one or _amends_ it —
  an amendment narrows a rule's condition and leaves it standing.

### Step 9 — settling

- **A prototype leaves whole, or it has not settled.** Promoting the inner piece of a four-round
  frame leaves the frame living somewhere nothing is allowed to depend on. What settles is the thing
  the user has been looking at.
- **A promoted shell is controlled; the behaviour that goes with it is a hook beside it.** Baking
  state into a shell makes every caller drill props through it; leaving the behaviour out makes
  every caller re-derive rules the design already settled, and the second caller gets them subtly
  wrong.
- **Rebuilding the callers is the other half of settling.** A promoted thing with nothing drawn in it
  has not been tested — the contract is only real once something outside it has had to meet it.
- **Deleting a default beats overriding it.** A candidate forbidden to edit its baseline cancels
  defaults with `!`; the moment the baseline is yours, delete the default instead.
- **When a decomposition's frame is replaced, the decomposition goes with it.** Keeping the old shape
  as a synonym for the new one leaves the types describing a frame that no longer exists.
- **Fold the round's bans** into the standing list and delete the narrative, or archive it. A bans
  file that grows a round of prose per session stops being read, which is the same as not existing.
- **Delete the losing candidates.** The record is the bans file, not the files. Do not keep a dead
  candidate around "to compare".

---

## Close the session

Rule 0 is stated loudly and is worth nothing unverified. Every round ends with:

1. The project's lint command, clean.
2. The project's test command, passing.
3. **The blast-radius check** — a diff or status filtered to everything outside the design root. It
   must be empty.
4. The thesis's numbers kept and the sweep clean on what the round touched, or each finding
   answered.
5. The round logged in the bans file.

Where the project has a scoped check command, run **that** on the inner loop — it does the same four
things over the design root in a fraction of the time, and a gate that is cheap is a gate that
actually runs between edits. Run the project's own whole-repo lint and test **once** before calling
a round settled: the scoped one cannot see a shared type breaking outside the design root, and that
is exactly the failure Rule 0 exists to catch.

Report the results plainly. If tests fail, say so with the output.

---

## Scope

|                | **system**                      | **frame**                                     | **view**                    | **component**               |
| -------------- | ------------------------------- | --------------------------------------------- | --------------------------- | --------------------------- |
| a candidate is | a whole visual language         | the chrome that holds every screen            | one screen's layout         | one component's states      |
| drawn on       | invented composite stress page  | the frame, holding real and undrawn pages     | the view, redrawn           | a variant grid              |
| baseline       | current app                     | the frame it replaces                         | current mirror of that view | current component           |
| N              | 3                               | 2–3                                           | 2–3                         | 3–4                         |
| axes           | depth · density · accent · type | where you are · how you move · what collapses | structure · IA · rhythm     | shape · weight · affordance |
| settles into   | tokens + primitives             | the kit, as a controlled shell + a hook       | the views space             | the kit                     |
| read           | SYSTEM.md                       | FRAME.md                                      | VIEW.md                     | COMPONENT.md                |

Pick the altitude from what is actually wrong. "This page feels pointless" is usually a frame
problem wearing a view's clothes; "these two components disagree" is usually a system problem.

---

## Anti-defaults

`frontend-design` carries the full catalogue. The ones that bite hardest in a dense internal tool,
and which belong in every project's pre-seeded bans:

- **Inter.** The single most recognisable generated-UI tell. Choosing it must be an argued decision.
- **Identical rounded cards** with one radius for everything and the same soft grey shadow beneath
  each. In a data-dense app this is what turns 400 rows into 400 boxes.
- **Tracked-out ALL-CAPS eyebrow labels** above every heading.
- **Meta strings joined with middle dots**, `A · B · C`.
- **A monospace face for small data labels** as decoration rather than because figures must align.
- **`→` appended to button and link text.**
- **Numbered markers** (01 / 02 / 03) on content that is not a sequence.
- **Fade-and-slide-up on every section**, hover transitions on every card. Motion that answers
  something the user did is welcome; motion that greets them is the tell.

---

## Hard rules

1. **NEVER** write outside the design root named in `PROJECT.md`.
2. **NEVER** write UI code in the design root before the theses are approved. A board is a drawing
   in the scratchpad, never a candidate.
3. **NEVER** score candidates against a general quality rubric — thesis conformance, the a11y floor
   and `PROBES.md` only.
4. **NEVER** build a candidate on the **baseline's** kit or the **baseline's** token names. They are
   what is being replaced, and using them re-anchors the output on it. Once a new system has settled,
   its kit and its tokens are exactly what you build on — `PROJECT.md` says which is which.
5. **ALWAYS** put a baseline frame in the comparison. "Better" is meaningless without one.
6. **ALWAYS** append rejections to the bans file in the same session they are voiced.
7. **A component enters the kit on its SECOND caller, not its first.** One caller is a part of that
   screen.
8. **ALWAYS** review at every width the design declares a breakpoint for, before calling anything
   settled. A variant is a state of the app, never a screen size.
9. **NEVER** write a second mapping for a state another component already draws. Import the one
   that exists. Two constants agreeing today is the bug waiting, and it has shipped here once.
10. **ALWAYS** ask the frame what its controls are called before pressing one. A guessed selector
    that matches the wrong control reports a press that changed nothing, which is indistinguishable
    from a state that does not exist — and that is a wrong conclusion, not a lost minute.
11. **NEVER** read a measurement off a screenshot. Ask the page. This has cost a session outright.
12. **ALWAYS** show theses and research on a board, never as ASCII in the terminal.
13. **ALWAYS** ask one question at a time, with a recommendation. Never a batch.
14. **NEVER** build a candidate whose thesis numbers are not written down. A check written after
    the build is written by the build.
15. **NEVER** hand a reader a browser, or a candidate the user has not picked. Measuring is a
    command's job; reading is the reader's.

---

## Reference files

| File                                          | Focus                                                                                   | Read                                                 |
| --------------------------------------------- | --------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| PROJECT.md                                    | The one project-specific file — paths, commands, what has settled                       | every session, first                                 |
| CORRECT.md                                    | The correction gear: classify, reproduce, fix at the causing layer, verify, report, log | whenever the user's message carries the answer       |
| BOARDS.md                                     | The research and thesis boards: the Artifact pages steps 1 and 2 show the user          | step 1 when research is shown; step 2 always         |
| BUILD.md                                      | Fanning step 3 out to one agent per candidate — the brief contract and the fan-in       | step 3, whenever N > 1                               |
| REVIEW.md                                     | The short read after the pick: what the reader is handed, and what it is never asked    | after step 6, on the winner only                     |
| PROBES.md                                     | The measured checks, the sweep and the thesis's numbers a frame has to pass             | step 4, and before settling                          |
| TRAPS.md                                      | Engineering traps that have each cost a round — layout, scroll, motion, capture         | when something is wrong and the cause is not obvious |
| WORKBENCH.md                                  | What a design workbench must provide, and how to drive one                              | when arranging or capturing frames                   |
| SYSTEM.md · FRAME.md · VIEW.md · COMPONENT.md | One per scope                                                                           | exactly one, per round                               |
