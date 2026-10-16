# Workbench

A design workbench is a second dev server, rooted at the design root, that renders design work in
isolation from the app. `PROJECT.md` says how to start this project's and where its instance doc is
— routes, file map, local gotchas. This file is the **contract**: what a workbench has to provide,
why each part earns its place, and how to drive one.

If the project has no workbench, building one is its own task, not a step inside a design round.

---

## What a workbench must provide

**1. A frame address.** One variant, alone, owning the whole viewport, with no workbench chrome in
it. This is the screenshot address and the iframe source; everything else is chrome around it.

**2. Real iframes at true pixel width.** Not a resized div, not a `max-width`. A genuine viewport, so
`100vh` measures the frame, a portalled overlay lands inside it, and **media queries actually apply**.
A workbench that fakes width cannot review a breakpoint, which means it cannot review a design that
has one.

**3. Everything in the URL.** Which variants, at what width, at what scale, with which chrome. If the
arrangement is deep-linkable then "screenshot that again" is one navigation and a review is
reproducible. Build frame URLs with the project's own helper, never by hand.

**4. Cross-space frame lists.** Old and new in one viewport is the whole point of the apparatus. One
lookup has to resolve an id across every space the workbench holds, so a baseline and the thing
replacing it sit in one list.

**5. A baseline that renders the real app's components.** Not a redrawing of them — the actual
components on fixtures. They are the before-picture. **Never restyle a baseline**, ever; a redesign
is a new subject beside it.

**6. No backend.** No API, no database, no auth. Stub the data client so a screen that tries to fetch
fails loudly instead of hanging. Boot cost near zero — start it without asking.

**7. Motion that runs.** See below.

**8. Isolation from production.** Whatever scopes the CSS build must be pinned so nothing invented in
the design root can leak into the app's bundle. This is what makes new tokens and utilities safe by
construction.

**9. A way to drive it from a terminal.** Something that opens a frame at a named width, lists what
every control is actually called, presses one, and reports the change with the before-number. Probe
11 — _the thing has been used, not looked at_ — is the check that finds what stills never show, and
a check with no tool behind it gets done with a throwaway script, a guessed selector and a number
somebody typed into a report. The lister matters as much as the presser: a name guessed from the
source is how a press lands on the wrong control and reports a state that does not exist.

**10. A port it refuses to drift off.** A workbench that boots on the next free port when its own is
taken serves a stale build to every tool that addresses it by the documented number, and each of
those tools then measures the wrong thing while appearing to work.

---

## The verbs

`bun workbench <verb>` is the mechanical half of a round. It moves paper; it does not design.
Everything in it was being done by hand: three rounds of transcripts put ~80% of a round's tool
calls in the shell, and the commonest single shape was a script doing text surgery on the
candidates' index or the bans file, 96 of them in one round. `bun workbench` alone prints every
verb with its flags, and `PROJECT.md` says which of them is this project's gate.

| Verb                                         | What it replaces                                                                                                                                                               |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `serve [--host]`                             | Starting the workbench. It refuses a port that is taken rather than drifting to the next one                                                                                   |
| `frames [--space view] [--json]`             | Building a frame address by hand. Prints every state with the URL that opens it                                                                                                |
| `drive <id>:<state> --list`                  | A `page.evaluate` dump to find what a control is really called. **Start every interaction here**                                                                               |
| `drive <id>:<state> --do "click:…; press:…"` | A throwaway Playwright script. Presses the live frame and reports what moved, with the before-number                                                                           |
| `drive <id>:<state> --shot out.png`          | The four-step screenshot ritual: right viewport, motion off, let it paint, capture                                                                                             |
| `sweep <id>[:<state>]`                       | A reviewer pressing every control by hand, one throwaway script each. Prints only what went wrong: nothing moved, keyboard lost, Escape did not close, text wider than its box |
| `conform [<id>…]`                            | A reviewer measuring the thesis's numbers by hand. Holds the frames against the thesis file and prints what was not kept                                                       |
| `probe [--space views]`                      | Eyeballing the floor. Sideways scroll, overflow, hit-testing, visible focus and contrast, measured on every frame at every width                                               |
| `census`                                     | Reading every component to learn which ones paint a state. Fails when one state is painted from two places                                                                     |
| `shots <folder> [--against <folder>]`        | Looking through every frame after a change that must move nothing. One picture of each frame at each width in both appearances, and the name of every one that differs         |
| `kit [<id>]` · `tokens`                      | Opening the bench to read a component's states, props, callers and parts, or a token's values and users                                                                        |
| `board <folder>`                             | Redrawing the kit by hand on every board, tokens copied by value. Builds the workbench's real frames and the board's own drawings into the board's folder, in ~2s              |
| `new <id> --title "…" --thesis "…"`          | Writing a candidate file and hand-editing the candidates' index to register it. Also creates the candidate's thesis file, empty, for the thesis's numbers                      |
| `new <id> --from <drawing.tsx>`              | Typing an approved drawing again into a blank candidate. The board's drawing file becomes the candidate as it is                                                               |
| `rm <id>`                                    | Step 9's "delete the losing candidates", as one command. Takes the thesis file with it                                                                                         |
| `promote <id> --to kit\|views [--dry]`       | The move, the un-register, the kit's export, and **rule 7's caller count, read off the source**                                                                                |
| `promote kit/<Name> --to app [--dry]`        | Moving a settled kit or views module into the app: `git mv`, callers and barrels re-pointed at the app's alias, and a refusal listing every workbench-only import              |
| `ban find --for table,density`               | Reading the whole bans file to build one screen. Prints the matching sections **verbatim**                                                                                     |
| `ban add --section 5 --round 50 --title "…"` | The anchor-asserting script. Body on stdin; warns about a rule carrying no round number                                                                                        |
| `check [--skill] [--allow …]`                | The project's whole-repo lint and tests on the inner loop, at a third of the cost, with the census, the skill's copy and the blast radius                                      |
| `skill`                                      | Copying this skill into a repo by hand. Writes every file but `PROJECT.md`, stamped with the engine's version                                                                  |

Three things are deliberately **not** verbs, because automating them would cost the thing the loop
exists for: **writing the theses** (one thread, by hand, with the bans in front of it), **choosing a
winner** (the user's, at step 6), and **summarising a ban** (the exact wording _is_ the check).

**This skill's files in a repo are a copy.** Their source is the engine's `skill/` folder. A change
is made there and copied with `bun workbench skill` (or `bun toolkit sync`, which copies every
package's); `check` fails while the copy and the source differ. `PROJECT.md` is the one file that
is the repo's own.

---

## The spaces

A workbench page belongs to **exactly one** space, and the set answers the only two questions a page
list has to answer: _which design system is this drawn in_ and _how settled is it_.

| Space  | Holds                                                                                                                       | Empty means                                                    |
| ------ | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| tokens | the system's values as live specimens, parsed from the source of truth                                                      | no system yet                                                  |
| kit    | the settled primitives, with a caller count read off the source, plus one composite page with every primitive on it at once | nothing promoted                                               |
| views  | settled places only                                                                                                         | nothing drawn                                                  |
| proto  | **only what is unsettled**                                                                                                  | nothing is being argued about — a true and useful thing to say |

Do not add a fifth, and do not add a nav section that is really a fifth. Baselines are not a space:
a baseline is the comparison argument inside a view, named by the thing that replaces it.

**A settled thing in the prototype bucket is how a design spends four rounds calling itself
settled.** Once something settles it leaves — into views as a place, or into the kit — and the
prototype files are deleted. The record is the bans file, not the files.

**Caller counts are read off the source, never written down.** Promotion turns on that number, and a
hand-kept list is the one thing that can quietly stop being true.

**A component's own variant grid is not a view.** Its states live on its own page in the kit.

**A component's page owes six things**: what it is and when to reach for it, every state, a line
for every prop, its parts, what it holds, and **what it refused**. A page that only says what a
thing does gets re-read as a menu of options.

---

## Motion

**A frame animates by default, and a capture has to ask it not to.** Motion is how a component
behaves, not a coat applied afterwards; a stage that freezes it is reviewing half the design. This is
the opposite of the obvious default and it is the right one — no other control on a stage shows the
subject as anything but itself.

**"Does it move" and "how fast" are one question, not two controls.** A _Motion_ toggle beside a
_Speed_ group gives two ways to reach the same still and lets one silently flip the other. One
group, four answers.

**Off is off, not a stopped clock.** `playbackRate = 0` freezes everything wherever the last thing
you pressed left it, which is indistinguishable in a still from no animation at all and is a
different thing to reason about. Spell one of them: off means **every animation settled at its end
state**, which is the frame a capture wants.

**A moment _inside_ a move is a pin**, and a pin is for captures. Clamp each animation to its own
end, so one pin is a coherent moment across a page whose transitions are different lengths.

**Several captures of one frame at different pins is the filmstrip**, and it is the only form a
motion axis can be reviewed in. An axis that does not show in a screenshot is not an axis — and a
pinned frame _is_ a screenshot, which is what makes motion reviewable at all.

**Looking and capturing want different controls.** To judge motion, leave it running and slow it:
5× slower turns a 240ms move into 1.2s, which is the difference between seeing that something
happened and seeing what. To capture it, pin or turn it off: both are exact and repeat.

---

## Screenshots

Four steps, and all four are load-bearing — a capture that skips the last one comes back black with
the DOM fully rendered underneath:

1. Resize the window to the viewport being reviewed. Do not rely on a scaled grid.
2. Navigate to the **frame** URL — one variant owning the viewport, no chrome in the shot.
3. Turn motion off or pin it.
4. Let the page paint, then capture.

Which is why it should be one command rather than a ritual. Where the project has one (`PROJECT.md`
names it), it does all four and cannot forget the fourth.

For a side-by-side, capture the grid and accept the scaling, or capture each frame separately and
compare them yourself.

**Never read a measurement off a screenshot.** `PROBES.md` says why and what to do instead.

---

## Widths

Three is enough: a phone, a tablet, a desktop. Pick real device sizes and keep them fixed, because a
review is only comparable against earlier reviews if the numbers did not move.

**A width is a control, never a variant.** Every state of a design draws at every width from one
seed. A candidate whose halves are `hidden lg:*` / `lg:hidden` renders half the stage as empty boxes
at whatever width you pick, and three of its six variants are literally blank. Where a width
genuinely cannot hold a state — a phone has one column, so "collapsed" and "both open" read the same
down there — the variant says so rather than disappearing.

---

## Keeping the workbench honest

Things that have quietly stopped being true in a workbench before:

- **An eager glob resolves at startup.** A brand-new file that a page enumerates will not appear
  until the dev server restarts. Restart before debugging its absence.
- **A router that generates a route tree** can put a running server into a full-reload loop when a
  new route file appears — the page reloads about once a second and nothing mounts, with no console
  error. Recognise it and restart; do not debug it.
- **A raw-source import that a page parses itself** must never point at the file whose styles apply
  to it, or every edit invalidates that module. Keep the parsed source in a leaf.
- **Raw-source imports resolve to an empty string under a test runner.** Assert on file contents with
  the filesystem instead.
- **Alias order matters** where one alias is a prefix of another. Write the alias config as an array
  so the order is explicit rather than dependent on key insertion.
