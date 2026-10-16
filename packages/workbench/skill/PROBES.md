# Probes

The measured checks a frame has to pass before it is called finished.

**Why this file exists.** Every expensive miss in this loop's history was a measurement failure, not
a taste failure — a bar two pixels off an edge, a column that would not shrink, a phantom 101px of
scroll, a position read off a screenshot whose coordinate frame was not the CSS viewport. None of
them were visible at the width everything gets looked at. All of them were one line of script away.

**Run them, do not eyeball them.** A screenshot tells you a thing looks wrong. It does not tell you
what is wrong, and it is not evidence that a fix worked.

Run them in the page, against the real frame, at every width the design declares a breakpoint for.

---

## Running them

Most of these are a command now, not a paste into a console. `PROJECT.md` names it for this repo.
It drives a real browser at each declared width, navigates to the **frame** address with motion
off, lets the page paint, and prints every measurement — passing ones included, because a probe run
only after a fix proves the current state rather than the change.

| #   | Probe                     | How                                                                            |
| --- | ------------------------- | ------------------------------------------------------------------------------ |
| 1   | no sideways scroll        | automated                                                                      |
| 2   | nothing overflows its box | automated                                                                      |
| 3   | indicators ride the edge  | **by hand** — it is a comparison against the frame, not a threshold            |
| 4   | targets hit-test to self  | automated                                                                      |
| 5   | every nav target opens    | **swept** — `sweep` presses each one; you still say what it should have opened |
| 6   | focus visible             | automated, driven with real Tab presses                                        |
| 7   | every state draws         | automated                                                                      |
| 8   | baseline states answered  | **by hand** — it is a question about the old design                            |
| 9   | both tracks start level   | **by hand**                                                                    |
| 10  | contrast, measured        | automated; anything on a gradient or image is reported as unresolved           |
| 11  | the thing has been used   | **swept** — `sweep` presses every control; `drive` for the one you are asking  |
| 12  | the thesis's numbers hold | automated, against the thesis file written before the build                    |

**An automated probe that cries wolf is one nobody runs**, which is the same as not having it. Four
filters in the runner exist because they each produced a wall of false positives on a frame that was
correct: a control below the fold is not covered (`elementFromPoint` is viewport-relative and returns
`null` outside it), a control behind a scrim is not covered (that is what a modal is for), a disabled
control is correctly unreachable by Tab, and — the fourth — **a target's centre has to be inside every
box that clips it, not only the viewport**. A row half-cut by the top edge of a scrolling column has
its centre exactly on the clip line: the viewport filter admits it, the scroller paints nothing there,
and the hit falls through to the wrapper behind, which reads precisely like a covered control. Twelve
of forty-five frames on one round, on a column deliberately parked mid-row.

### The baseline — the fifth filter, and the one that is written down

The four filters above are conditions the runner can recognise. A floating object the design put
there is not one of them: the probe is **right** that a slug at the foot covers what is under it,
and what makes that a design rather than a defect is an argument somebody had, once. Left there,
the views space exited non-zero on a clean tree and printed 35 adjudicated lines on every run —
which is the "cries wolf" failure this very section warns about, arriving through the front door.

So the adjudication is written down instead of re-done from memory:

```
bun workbench probe --space views --accept   # the findings now occurring are expected
bun workbench probe --space views --quiet    # only what changed since
bun workbench probe --space views --full     # ignore the baseline entirely
```

`probe/baseline.json` holds one key per held finding — frame, state, width, probe, and the finding's
own words — plus a note saying what was adjudicated and why. **Held findings still print.** They
stop failing the command, which is the thing this file already asks for: _"the number to watch is
whether it grew."_

Three properties it has on purpose, each because the sloppy version of it would be worse than no
baseline at all:

- **A result fails the moment one of its findings is new**, even if its others are held. A baseline
  is not a mute button on a probe.
- **A held finding that stops occurring is reported**, so a baseline nobody prunes cannot quietly
  stop being true — which is the same failure a hand-kept caller count has.
- **Accepting a subset keeps the rest.** `--frames one:state --accept` must never be a way to erase
  every other frame's adjudication.

**Re-read the keys before re-accepting.** `--accept` is how a defect becomes permanent if it is run
without looking.

---

## The sweep, and the thesis's numbers

Two commands that are not the probe runner, because they answer different questions. `PROJECT.md`
names both.

**The sweep presses every control of a frame at every width and prints only what went wrong.** It
is probe 11 without a hand. What it looks for needs no judgement:

- a press that moved nothing
- a keyboard left on the page body after a press
- something a press opened that Escape does not close, or that does not hand the keyboard back
- a page that scrolls sideways only once something has been pressed
- a press that throws
- text wider than the box it is set in, which no other probe sees: the page does not scroll and
  nothing is covered
- a control with no name

A long run of alike controls, sixty rows of one list, is pressed at its first, middle and last. A
control already standing on its answer (a pressed toggle, the current tab) is not pressed, and nor
is one an open modal has put out of reach.

**What it cannot see is whether the thing that moved was the right thing.** That is a reader's, and
it is why a brief still says what each control should do.

**A frame is a pinned state, so some of its controls are inert on purpose.** A `deleting` state
drawn from a fixture does not close when `Cancel` is pressed. That is a reading, not a finding, and
it is written down the way a probe's is: read it, then `--accept` it, and the command fails only on
what is new. `--only '<regex>'` accepts the findings that match and leaves the rest failing, and each
finding is held on its own, so a frame can hold its inert controls and still fail on a real fault. The same three properties hold as for the probes' baseline, and so does the warning:
`--accept` without reading is how a defect becomes permanent.

**The thesis's numbers are held against the live frames by one command.** The thesis file is
written when the thesis is approved and before anything is built. Each line names a part and what
it has to measure: its box, how many lines its text runs to, how many of it there are, any computed
style, or the room between two parts. A width can have its own number. Every drawing of the part
has to hold, in every state that draws it.

- **A check on a part nothing draws fails.** A number nobody can measure would otherwise pass for
  ever, which is how a renamed part turns a check into decoration.
- **A thesis file with no numbers fails.** An empty file is a build nobody is holding to anything.
- **It prints what was not kept**, with the state, the width and the figure. `--all` prints what
  held as well, with the figure measured, for a report that has to show both.

---

## The floor — every frame, every width

**1. The document does not scroll sideways.** One probe, and it finds what the eye does not.

```js
[...document.querySelectorAll('iframe')].map(f => {
	const d = f.contentDocument.documentElement;
	return { src: f.src, over: d.scrollWidth - d.clientWidth };
});
// every `over` must be 0
```

A non-zero result is almost always one of two traps: a flex item without `min-width: 0`, or an
absolutely-positioned child escaping a scroller's clip. Both are in `TRAPS.md`.

**2. Nothing overflows its own scroller vertically that should not.** Content taller than the glass
inside a box that is not a scroller is a page running past its frame.

```js
const s = el.querySelector('[data-scroller]') ?? el;
({ scrollH: s.scrollHeight, clientH: s.clientHeight, isScroller: getComputedStyle(s).overflowY });
```

**3. Scroll indicators ride the edge they belong to.** Compare the scroller's box against the frame's,
not against the container's. A gutter on the container pushes the bar inboard while the column beside
it rides the glass, and two halves of one screen then report scroll at two different insets.

```js
const r = scroller.getBoundingClientRect();
({ x: r.x, right: r.right, frame: innerWidth, padL: getComputedStyle(scroller).paddingLeft });
```

**4. Every interactive target is reachable and hit-tests to itself.** A scrim, an overlay or a
transformed ancestor can cover a control without changing how it looks.

```js
[...document.querySelectorAll('nav button')]
	.filter(b => b.getBoundingClientRect().width > 0) // skip the hidden width's copy
	.map(b => {
		const r = b.getBoundingClientRect();
		const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
		return { label: b.ariaLabel ?? b.textContent, self: b.contains(hit) };
	});
// every `self` must be true
```

Filter by a non-zero rect first. A responsive frame that renders both the wide and the narrow chrome
with one hidden will match both in a selector, and the hidden copies hit-test to whatever is on top —
which reads exactly like the control being covered.

**A floating object covering column content is a reading, not a finding.** A slug at the foot, a
pointer pinned to the top of a scroller, any deliberate overlay — each one covers whatever is under
it at that scroll position, and the probe is right that it does. What makes it a design and not a
defect is that the column scrolls: the covered control is one gesture away, and the object is inset
or flush by a rule somebody argued. **Read the covering element before deciding.** If it is the
floating object the design put there, the finding is expected and the number to watch is whether it
grew; if it is anything else, it is a defect. The one thing that is never acceptable is a floating
object covering the **last** row of a column, because no scroll reveals it — that is what a floor
constant is for, and it is a different check.

**A control that correctly stops existing reads as a control Tab never reached.** The focus probe
stamps every target with an index, then walks with real Tab presses; an element that unmounts during
the walk never reports its index and the run prints `n−1 of n targets reached`. A control whose whole
job is to disappear when its condition clears — a pointer that exists only while the record it names
is off screen — will do this every time, because Tab scrolls the column and the scroll is what
clears the condition. Check it by hand: `.focus()` it at a position where it exists and confirm
`document.activeElement` is it. Nothing needs fixing if it is.

**5. Every nav target opens something.** A cell that opens nothing is not a finished frame. Press
each one and read back what the frame says it is showing.

**6. Focus is visible on every control**, and `disabled` is on the real attribute rather than a class
that only looks disabled.

---

## Before settling

**7. Every state draws at every width.** A variant is a state of the app, never a screen size. Load
each variant at each breakpoint and confirm none of them is an empty box. This is the check that
catches a frame whose halves are `hidden lg:flex` / `lg:hidden`, where half the stage is blank at any
width you pick.

**8. Every state the baseline had is answered for**, even if the answer is "this state stops existing
because the new structure makes it impossible."

**9. Both scroll tracks start at the same height**, where a design has two columns that report scroll
onto a shared edge.

**10. Contrast, measured.** Every ink on every ground it is actually drawn on, against AA — and the
3:1 non-text floor for anything carrying meaning that is not text. A token's documented ratio is the
ratio on the ground it was specified against, not on the one it ended up on.

**11. The thing has been _used_, not looked at.** Press every control at every width. The defects
that reach the user are almost never visible in a still: a menu that will not dismiss, a cell that
will not collapse, a page that overflows only once its content is real.

`sweep` does this for the whole frame and prints what went wrong (see above). `drive` is the same
press asked of one control, when the question is what that control does:

```
bun workbench drive order-edit:edit --list                       # what everything is really called
bun workbench drive order-edit:edit --do "click:Show the lines on Spring order*; wait:400"
```

**Always `--list` first.** Round 49 spent a round-trip per state guessing selectors — a `combobox`
that was a `button`, a name that matched the side column's control instead of the document's — and the
guess that matched the wrong one reported a press that changed nothing, which is indistinguishable
from a state that does not exist. `--list` is one call and it prints the accessible name of every
target at that width. `drive` then refuses an ambiguous name rather than taking the first match,
and names the candidates.

What it prints is the **change**, with the before-number: text length, painted boxes, named targets,
which names appeared and disappeared, and — when the layout moved — the page's `column-count`, how
many targets shifted and by how far, and the span of glass the frame is using.

**The layout lines are not a nicety.** A state reached by clipping a column away changes no text at
all: collapsing the side column reads `text —, boxes −6` and looks like nothing happened, while the real
evidence is `columns 1 → 2` and `moved 150 of 151 targets, largest 782px`. Round 49's own
justification for deleting the `collapsed` variant was _"1 → 2 columns"_ — a number a content diff
cannot produce.

Movement is measured in **content coordinates**, not viewport ones, and that distinction is load-
bearing. A click has to scroll its target into view before it can press it, so pressing anything
near the foot of a column scrolls the column; read in viewport coordinates, an inert button reported
67 targets moved and the run concluded the press had reached a state. The floor is a line's worth —
under that is rounding, a sticky row, or a scrollbar.

A press that moves nothing is called out in as many words, because a silent no-op is the one result
that reads like success. Every run ends in a `RESULT:` line, so a pipe through `tail` cannot
swallow the verdict the way it swallows an exit code.

Opening the frame in a real tab is still worth doing — the browser window as the viewport, no stage,
no `scale()`. `drive` answers _did this state exist_; a tab answers _is it any good_.

---

## Measuring at all

**The capture's coordinate frame is not the CSS viewport.** A screenshot may come back at a
different pixel size than the page reports for `innerWidth`, and every position read off it is then
scaled by a factor you did not account for. This has cost a session: a rail measured at ~393px from
an image, a hunt for a root font-size override, and no override existed — the image was 1270 wide and
the viewport was 1720.

Never read a coordinate off an image. Ask the page:

```js
({
	vw: innerWidth,
	rootFont: getComputedStyle(document.documentElement).fontSize,
	rect: el.getBoundingClientRect(),
});
```

**Measure inside the frame, not the stage.** A workbench grid scales its iframes; a rect taken from
the outer document is in stage pixels, not design pixels.

**Take the before-number.** A probe run only after the fix proves the current state, not the change.
Both numbers belong in the report.
