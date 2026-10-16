# Traps

Engineering causes that have each cost a round. Not taste — these are true of the platform, and they
are the answers to "the design is right and the screen is wrong."

Open this when something is wrong and the cause is not obvious. Add to it whenever a defect's cause
generalises past the project it was found in.

---

## Layout

**A flex item's automatic minimum is its min-content, not zero.** An item holding text refuses to
shrink below the widest unbreakable thing in it, and pushes the document into a horizontal scroll.
`min-width: 0` on the item fixes it. The rule is **every flex item that holds text needs it**, not
just the ones that have bitten already — the wrapper you did not think about is the one that breaks
at 390.

**`box-sizing: border-box` means a 1px border is 2px off a fixed-size clip.** Ruling a frame box with
`border` shrinks the clip below the content sized to fill it, and the content loses its right and
bottom edge. At a scale factor those two pixels are seven real ones, and they are usually the two
edges the frame exists to show. **A box whose size is dictated by its contents is ruled with an
`outline`** — painted outside, changes no layout.

**A column is laid out at the width of the state it is _entering_, and zero is not a width.** Sizing
content to an animating clip box re-lays it out on every frame: lines rewrap, rows stack, the
scroller's content height changes under someone for no reason. A column arriving takes its arrived
width immediately; a column leaving keeps the width it last rested at while the clip shrinks out from
under it.

**A control that cancels its own padding with a negative margin overhangs the line it ends.** The
label-as-button pattern — `-mx-1 px-1`, so the word sits exactly where the plain word sat and only
the hover ground is wider than it — is right everywhere except at the end of a line, where the
ground lands past the line's content edge and the row reports `scrollWidth − clientWidth = 4`. It
is invisible: a scroller clips it, so the page's own overflow check passes and only a per-row
measurement finds it. Measured at 4px on four rows of one document, in two files that had each
written the row out by hand.

**Padding the row is the wrong fix**, and it is the one to reach for first: `pr-1` does put the
ground on the measure, but it moves the **word** in with it, and a word at the end of a line is
usually flush with something above it that is not a button — measured, a section band's own aside
left 4px adrift, trading an invisible overflow for a visible misalignment. Fix it on the control
instead: **an edge control trims its overhang on the edge side**, so the word still sits where a
plain word sits _and_ the ground stops where the page stops. Make it opt-in — a caller that says
nothing overhangs both ways exactly as before — and the row constant's job is only to document
that a control opening or closing the line takes it.

**Clipping is for the length of a transition, never for a state.** Drawing content at one width
always and clipping the surplus leaves real content — gutters included — permanently off-screen.

**A portal escapes `display: none` as happily as it escapes `overflow: hidden`.** The moment a
floating panel is portalled out to fix clipping, every hidden copy of its trigger starts posting a
panel too — and a responsive frame that draws the wide chrome and the narrow chrome at once, letting
a media query pick, has one of those for every control. The copy nobody can see has a zero-sized
reference, so its panel lands at the top corner of the page, at full size, looking like a rendering
bug with no cause. **Gate the panel on the reference having a box** (`rects.reference.width === 0`
in floating-ui's `size` middleware, re-evaluated by `autoUpdate` when the media query flips). The
same shape catches `visibility: hidden`, `content-visibility`, and a collapsed `<details>`.

---

## Scroll

**A scroller only clips what it is the containing block for.** `position: absolute` with no offsets —
which is what `sr-only` is — resolves against the nearest positioned ancestor, so a visually-hidden
label sits at its static position far down the scrolled content and escapes the clip. The symptom is
phantom document scroll (100px of nothing) and a stray scrollbar. `overflow: hidden` does not fix it;
`position: relative` on the scroller does.

**A native scrollbar rides the _scroller's_ box, not the container's.** Padding on the box around a
scroller pushes the bar inboard. If a design draws its own thumb only above a breakpoint, below that
breakpoint the indicator is the native bar — so **the gutter belongs to the scroller at every width**,
never to the box around it.

**A scroll container's own `padding-bottom` is left out of its scrollable overflow** once the content
is taller than the box (Chrome). A foot margin written on the scroller is there right up until the
moment you would have seen it. Put it on a block in the flow.

**A column rule is painted down the multicol element's content box.** Writing the foot margin as
`padding-bottom` on that element is the whole of the fix for a rule that runs into the bottom edge.

**A responsive shell that draws both chromes at once draws the column twice, and the hidden copy
still answers questions.** `display: none` gives a subtree no boxes, so every rect in it is zero —
which means any "is this on screen?" arithmetic inside it concludes that _everything_ is off the top
of a zero-height viewport. The copy nobody can see then renders the affordance that only an
off-screen thing should have, and it is invisible, so nothing looks wrong: the defect shows up as a
`querySelector` in a test or a probe picking the phantom rather than the real one. Same family as the
portal escaping `display: none`, one level up. Gate the arithmetic on the scroller having a height,
and filter by a non-zero rect before reading anything back.

**A sticky object is held against the scroller's _content_ box, so padding on or above the scroller
is room it can never reach.** A column that spends 8px of chrome above its first line — as a padding
on the scroller, or on the box around it — hands that 8px to nothing: an object pinned at `top: 0`
lands 8px down, with a strip of ground over it, and reads as hanging rather than as held against the
glass. Negative offsets do not recover it, because the scroller clips them. Spend the room **inside
the document** instead — a margin on the first block in the flow — where it is the same pixels at
rest and the first thing that scrolls away. Symptom: an indicator that is meant to be flush with the
top of a column and is stubbornly a hair below it at every width.

**`scrollIntoView` walks its own way up the ancestors, and an `overflow: clip` wrapper is on that
path.** In an app frame the scroller you mean is usually two or three boxes down from a clip box and
an animating width, and the browser's choice of what to scroll is not yours. Where the scroller has
already been found — and anything measuring scroll position has already found it — write the scroll
on that element (`scrollTo({ top })`) with the arithmetic in one function shared with whatever seeds
the position. Two ways of centring the same row is two answers to one question.

**A pinned head's `z-index` climbs out of its scroller unless something isolates it.** A scroller is
not a stacking context, so `sticky top-0 z-[1]` on a month head ranks against the whole page — and
chrome drawn as a positioned layer with no `z-index` of its own (a slug, a scrim) paints in the
`z: auto` layer, under every z-indexed thing in the content however early it comes. Symptom: the
page's sticky heads painting over the chrome's own bar when the content scrolls behind it, at the
widths where the chrome overlays the page and nowhere else. Fix it where the page meets the chrome:
`isolation: isolate` on the box that holds the page, so every z-index inside it ranks only against
the page. Never by raising the chrome's z-index to outbid whatever the next page brings.

---

## Multi-column

**`column-fill: balance` is the default and it divides the flow evenly however short it is.** A
four-row page comes out as two two-row columns with a rule between them and empty glass underneath.
`auto` fills the first column to the foot and starts the second only with what is left over.

**`column-count` divides the width evenly and hands each column the surplus.** Two columns in a wide
block are each far wider than the measure, so the extra is cut into strips and hidden _inside_ the
columns — text hard against one edge and starting past the other. Set the columns at the measure and
cap the block at what they and their gutters come to, so the surplus stays in one piece.

**Margins are truncated at a column break only when the break is forced.** Relying on it to swallow a
section's top margin is a coin toss. Spend section rhythm _below_ a section, never above it, and two
columns start on the same line.

**`column-count: n` with an explicit height is a fill target, not a cap.** Content past `n × height`
is laid into **overflow columns to the right**, outside the block's own width, where the scroller
clips them — so a height that is merely _stale_ does not leave the page short, it prints a column the
page has no room for. Measured: one item's details opened on a page whose height had been computed
before, 256px of column standing off the side of the glass. Any measured-height multicol therefore
needs **two** things: the contents watched, and a search whose answer is guaranteed.

**A measured layout must observe its contents, never be told about them.** A token prop that a caller
changes "whenever the contents change" is a contract every caller eventually breaks, because a page
changes size without changing which page it is — a disclosure opening, a list growing, an image
landing. `ResizeObserver` the block's **children**, plus a `MutationObserver` for children arriving:
observing the block itself is observing your own output, whereas a child's box is the same at every
column count, because a column is the measure whatever else changes.

**A search over layout needs a bound that is known to be safe, not a step count.** Growing a height
by a quarter and giving up after four passes writes an _unverified_ answer, and for multicol the
failure of an unverified answer is sideways overflow. Bisect between the arithmetic ideal and a
height proved to fit — for a spread that is **the whole flow in one column**, which every column can
hold — and only ever move the upper bound onto a height just measured not to spill. Then the answer
is safe by construction rather than by luck.

**`ResizeObserver` does not fire in a hidden tab; `requestAnimationFrame` may.** An automation tab
that backgrounds itself between calls therefore reports layouts frozen at whatever the last visible
frame was, and a resize test run through it measures a bug that is not there and misses the one that
is. Cost a long stretch of a round chasing a phantom. Drive the resize with real clicks that keep the
tab in front, or take a screenshot to force a render before reading.

---

## Motion

**Anything that reads as attached to a moving edge is positioned _from_ that edge**, or it detaches
for exactly the length of the move. A thumb drawn at a column's rail-facing edge stays where the rail
_was_ while the boundary animates, and lands hundreds of pixels out over the other half.

**Animating out takes `fill: 'both'`.** `fill: 'backwards'` holds the first keyframe through the
delay and then hands the element back to its own style the instant the animation ends — so a layer
that was just wiped away reappears whole and stays up until React unmounts it, a frame or two later.
Read as a flicker after the move lands. `backwards` is only for something animating _in_, where its
own style is the arrived state.

**Two layers replacing each other need complementary clips**, one opening as the other closes, so the
edge has the new thing on one side and the old on the other and never both. Drawing the new over the
old leaves the old visible under it for the whole transition and then vanishing in one frame.

**A transition only exists if something causes it.** A variant that renders its end state directly
has nothing to pin or capture. The specimen has to _do_ the thing — open the menu, press the cell.

**Check the easing against the duration before blaming the number.** An emphasized-decelerate curve
is most of the way done in the first quarter of its duration; run beside a second channel on a
different clock, one gesture reads as two.

**A field that crosses elements cannot be N per-element animations.** Giving each element its own
delayed animation quantises the effect to element granularity: a box has no inside, so it lights
whole, and its neighbour lights whole at a different value — a hard step across the seam and flat
ink within each box. Read as "things taking turns" rather than light passing over them. The fix is
**one animation in a shared coordinate space**: animate a single unitless 0→1 on the common
ancestor, publish the geometry (the container's extent, the band's depth, each element's own offset)
as custom properties, and let every element paint the slice of the one field that crosses it — a
gradient `background` positioned by that arithmetic. Continuity then falls out for free, and the
document holds one animation rather than sixty.

**Animate a unitless number, not a length built from other custom properties.** Interpolating a
registered `<length>` whose keyframes are `calc()` over other `var()`s is asking the engine to
resolve variables inside an animation of a variable. A 0→1 `<number>` always interpolates; do every
length conversion on the element that consumes it.

**A delayed animation with no `backwards` fill paints the element's own base style until its delay
elapses.** So a mark meant to be handed down a stack one row at a time appears on _every_ row below
the live one at once, on the first pass. The symptom looks like a stagger that is not working; the
cause is that the stagger has not started yet.

---

## React

**An element passed down as a prop does not re-render when its parent does.** React compares the
element object, sees the same one, and bails out of the whole subtree — so a shell handed
`aside={<Panel/>}` from a `draw()` that ran once re-renders itself, changes which half of the screen
is displayed, and the panel inside never runs a line of code. Anything inside that subtree which has
to notice a change in its surroundings must observe the DOM (`ResizeObserver`, `IntersectionObserver`)
rather than rely on a render happening. A `useLayoutEffect` with no dependency array does **not**
rescue it: no render, no effect.

**A ref written during render must be idempotent, or StrictMode reads back its own overwrite.** The
"previous value" pattern — assign at render, read on the render after the change — survives a double
render only when writing the same value twice is a no-op. A width overwritten with the _new_ value
does not: the second render reads the new one. Where the previous value is implied by the state, name
it from the state instead of remembering it.

**A press that takes its own control off the page leaves the keyboard on `<body>`.** The browser
does not move focus to a neighbour when the focused element is removed or hidden: a cell in a row
that is replaced by another row, a foot that goes with the errand it answered, a link on a page that
another page is drawn over. Every one of them measured `document.activeElement === body` after the
press, at every width. Whatever draws the replacement decides where the keyboard lands — the way
back on something that opened, the row that came back in the same place, the head of a page whose
contents changed — and it does so **a frame later and only if focus is still lost**, so a dialog
restoring its own focus, or a control that took it on purpose, wins. Key the move on the identity of
what opened, never on a button mounting: a button drawn again after a peek would take the keyboard
back from wherever the peek left it.

---

## Capture

**The capture's coordinate frame is not the CSS viewport.** Never read a coordinate off an image.
See `PROBES.md`.

**A screenshot taken in the same batch as the navigation before it comes back black**, repeatedly,
with the DOM fully rendered underneath. It is the capture that is early, not the page that is broken.
Let the page paint; re-shoot rather than debugging the fixture.

**A smooth scroll does not happen in a hidden tab — it is dropped, not queued.**
`scrollTo({ behavior: 'smooth' })` and `scrollIntoView({ behavior: 'smooth' })` are driven by the
rendering pipeline, which is suspended when `document.visibilityState === 'hidden'`; the call
returns, nothing moves, and nothing moves later either. An automation tab in a background window is
hidden even while you are screenshotting it. `scrollTop = n` works there and is the way to _verify_
the arithmetic; the behaviour itself has to be judged in a visible window.

**`scroll` events are not delivered in a hidden tab either**, so anything that recomputes from scroll
position — a thumb, a sticky readout, an "is it above or below" indicator — is frozen at whatever it
was when the tab went to the back. Measured: a programmatic `scrollTop` write moved the column and
fired **zero** scroll handlers. Taking a screenshot flushes a frame and the handlers then run, so the
sequence is _write, screenshot, read_ — the same fix as the frozen `ResizeObserver`.

**The animation clock is frozen in a hidden tab.** A live frame does not advance in a capture tab: an
entrance sits at its first frame forever and the capture is of a half-drawn thing. Turn motion off or
pin it for any URL you are screenshotting. An endless animation must be _pinned_, not finished —
`finish()` on an infinite animation throws.

**A scaled iframe changes how `position: fixed` resolves inside it.** Review anything fixed at full
scale or in its own tab.

**A capture in a fresh browser context fetches the web fonts again, and a face that is late or
never arrives reads as a rendering difference.** Comparing 120 frames pixel by pixel, a new context
per shot gave a different 10 to 40 "changed" frames on every run — a few hundred pixels each, the
same count for the same frame, and the workbench differing from _itself_ as often as from the thing
under test. One face of the family had not loaded: the layout is identical and only the glyphs set
in that weight differ, so two captures side by side look the same. `document.fonts.ready` does not
help, because it resolves for the faces already asked for. **Take every capture of a comparison in
one browser context**, so the faces are fetched once and then come from its cache, and treat a
whole run that disagrees with the last one by small constant amounts as a font that failed, not as
a change. On one shared context the same 120 frames gave the same answer three times.

---

## Small platform facts

- **A Tailwind one-sided utility beats the axis it narrows; two values of the same axis do not.**
  `pl-0` overrides `px-1`, because Tailwind orders by property group and `pl` sorts after `px`. But
  `px-0` against `px-1` is one group ordered by value, so the **larger value wins** and the override
  silently does nothing. Measured: an edge-trimming button written `mx-0 px-0` kept all 4px of its
  padding and its word stopped 4px short at both ends. Write the one-sided pair (`ml-0 pl-0 mr-0
pr-0`), never the axis.
- **Tailwind v4 gives `button` `cursor: default`.** `cursor-pointer` has to be written.
- **A `title` attribute on a touch target** fires as a native grey box on tap and covers the thing it
  describes. The accessible name carries the word instead.
- **An SVG dash pattern in user units** has to be retuned per size and lands mid-corner. `pathLength`
  pins the rhythm so it is identical at every rendered size.
- **Never trigger a native `alert` / `confirm`** while driving a browser — it blocks every subsequent
  command.

---

## On a real phone

None of these show up in a 390px frame on a desktop browser. A frame is a viewport with no browser
chrome, no keyboard and a mouse, so a design can pass every probe at every declared width and still
arrive cut in half on glass. **Open it on a phone before calling a responsive design finished.**

**`100vh` is the _large_ viewport, and a phone browser almost never shows you that much.** It is the
height the glass would have with every collapsible toolbar retracted, and it stays that height
whether they are retracted or not — so a shell written `h-screen` stands roughly 90px taller than
what the user can see, and its foot sits under the browser's own bottom bar. It cannot be scrolled
back into view when the shell owns the viewport and the scrolling happens _inside_ it, which is the
usual shape for an app frame. Use `100dvh`. The objection to `dvh` — that it changes as the toolbars
collapse — only applies to a design whose _document_ scrolls, because that is the only thing that
collapses them.

**A `focus()` made after paint gets the caret but not the keyboard.** iOS raises the keyboard only
for a focus that happens inside a user gesture. React runs a passive effect (`useEffect`) in a task
of its own after the paint, by which time the tap is over — so focusing from one leaves a caret
blinking in a box with nothing to type into, which reads as a broken control rather than a missing
keyboard. `useLayoutEffect` runs inside the commit React flushes synchronously at the end of the
discrete event, which is still the gesture. Focusing straight from the handler is better still.

**A phone browser magnifies the page when focus lands on a control under 16px**, and pans it
sideways to keep the caret in view. A system whose body size is 13px trips this on every input.
There is no CSS switch for it: the only clean fix is 16px on the typing surface where the pointer is
a finger. `maximum-scale=1` in the viewport meta is not a fix, it is a WCAG 1.4.4 failure.

**The browser's own focus scroll moves every scrollable ancestor, on both axes, by the minimum.**
Minimum means the control lands at whichever edge was nearest — often the bottom, which is where the
keyboard is about to be. Both axes means a page exactly as wide as the glass still jolts sideways.
Taking it over is one `focusin` listener writing `scrollTop` and nothing else.

---

## One enum, two drawings

**When a second component draws a state enum the first one already draws, its ink has to come from
the same mapping — or the two will disagree in one frame and only a screenshot will catch it.**

Measured: a progress meter and a row mark sat 200px apart on the same table and drew `pending` in
two different hues, because the mark read its ink from a `Record<State, string>` written rounds
earlier and the meter's author wrote the mapping again from memory. Both drawings passed their own
review — a candidate is checked against its thesis, and neither thesis mentioned the other drawing.

The check that catches it: **name every state the system has, and for each one list every component
that draws it.** More than one drawing of a state is a mapping that has to be shared, not copied. A
constant one import away is the cheapest fix; two constants agreeing today is the bug waiting.

---

## Probes

**A probe function is serialised into the page, and nothing around it goes with it.** A driver hands
the browser the function's source; every module-scope constant it referenced stays behind, and the
call fails with a bare `ReferenceError` naming an identifier that is plainly right there in the
file. Each probe inlines everything it uses, and a test asserts the module declares no top-level
constant for one to close over.

**`elementFromPoint` is viewport-relative and returns `null` outside it.** So a hit-test probe
reports every control below the fold as covered — measured at five identical false positives on
every variant of a scrolling table, which reads as a real and consistent defect. Filter to targets
whose centre is inside the viewport and count the rest separately.

**Being covered is the design when a modal is open.** A dialog, or a bare scrim with a panel painted
over it, is supposed to make the page behind it unreachable. A probe that does not know this reports
the whole page as broken on every modal variant. Detect the overlay, report what it covers as behind
it, and check only what is reachable.

**A disabled control is correctly unreachable by Tab.** Counting it as a focus-order defect reported
nine phantom a11y failures on a frame whose keyboard order was perfect. Exclude `:disabled`,
`[inert]` and `aria-hidden` before asking what Tab reached.

**An open floating list covers what is under it, and that is the design.** The same rule as a modal,
one level down: a select's list is placed over the page by `@floating-ui`, so every control it lands
on reports as covered, and a frame seeded with one open fails the hit test on controls that are
perfectly fine. Before reading a covered-target list as a defect, probe a frame the round did not
touch that has the same thing open — an unchanged frame failing identically is the probe, not the
page.

**The probe driver's browser build is pinned to the driver's version.** A Playwright bump leaves the
cached browser behind and every probe dies at `launch` with an executable-not-found path, which
looks like the workbench being down. Install the matching build (or point the launch at a browser
that is there) — and do it before concluding anything about the design, because a round with no
probes is a round judged by eye.

**A browser that is not painting a frame delivers no `ResizeObserver` and no `requestAnimationFrame`
to it.** Observation delivery rides the rendering lifecycle, and Chrome skips it for an iframe that
is off-screen in its parent, and for a tab that is not being composited — which is exactly the state
a workbench stage is in when it is driven through a debugging protocol rather than by a hand. The
symptom is a reveal that never re-measures: the column comes back from `display: none` and the thing
that watches it stays asleep, so you conclude the code is wrong when the environment is. Force a
frame — a real input event, or a screenshot — before reading anything back, and confirm a
reveal-driven behaviour with an actual click rather than a scripted one.

**A programmatic `.focus()` does not set `:focus-visible`.** A probe that focuses in script measures
a state the keyboard never produces, and passes a control that shows nothing to a person tabbing
through. Drive it with real Tab presses, with a budget of about twice the target count — focus
leaves the document for the browser's own chrome and comes back.

**A Tab target that disappears while the probe tabs reads as unreachable.** Targets are counted
once, before the first press, and Tab scrolls things into view as it goes: a `4 proposed below`
pointer at a column's foot is removed the moment the proposals it points at scroll up, and the probe
then reports one control that never received focus — one run in three, on a page nobody touched.
Record every target's label when counting, and name the unseen one before calling it a defect; a
target that is no longer in the document is the probe's timing, not the page's keyboard order.

---

## Closing the session

**The project's own `format` command writes outside the design root.** A repo-wide Prettier or
equivalent walks the whole tree, so a round that ends `format → lint → blast radius` reports a dirty
blast radius it caused _itself_ — measured once at three files, one of them a markdown corruption
(`SERVER_CONFIG.md` → `SERVER*CONFIG.md`) in a file the round never opened. **Run the blast-radius
check after formatting, never before**, and revert what the formatter touched outside the root
rather than committing it. The formatter is not a design decision and its collateral is not a
change the round is allowed to make.
