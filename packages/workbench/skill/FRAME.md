# Scope: frame

Use when what is wrong is the **chrome that holds every screen** — where you are, how you move, what
the app looks like before any particular page is drawn in it.

This is its own scope because it is neither a system nor a view. The system says what things look
like; a view is one place; the frame is the thing a place is _in_, and it is judged by what it does
with pages it has not been given.

Reach for it when the complaint is "the pages don't feel like they have a point", "there's a lot of
free unused space", "I don't know what page I'm on", or any version of "the navigation".

---

## Canvas

The frame, holding **every destination it claims to have** — the drawn ones with their real pages,
the undrawn ones with a stated placeholder: one line saying what the page is for, then rows at the
pitch its real contents will stand on.

**A nav target that opens nothing is not a finished frame.** A frame prototype is an argument that
the frame holds the app, and it cannot make that argument with dead cells. The placeholder is not a
design of those pages — skeleton rows on purpose, so a drawn-out fake is not judged as a design of
something nobody has designed.

**The frame is live, or it is not a frame.** A still of a nav bar is a picture of a nav bar. Every
cell presses, every collapse collapses, the seed is a starting position rather than a still. A
splitter that can only be looked at is a splitter nobody has tried.

---

## Axes

| Axis               | Options                                                            |
| ------------------ | ------------------------------------------------------------------ |
| **where you are**  | a persistent margin · a band · a foot object · nothing until asked |
| **how you move**   | one register or two · glyphs or words · always visible or summoned |
| **what collapses** | nothing · one half · either half · a continuum                     |

Three more the round has to answer explicitly, because leaving them to the build is how a frame ends
up with two answers to one question:

- **What is the landing surface, and is it a destination?** If it is _the app_ rather than a page in
  it, it is not a row in any menu.
- **What does a mode buy?** If collapsing a half hands the other half space and nothing in the layout
  answers, the mode is decoration — either the layout spends the space or the control should not
  exist.
- **What does a width below the breakpoint do?** Not "responsive". The rule.

---

## Theses

Two or three, and the drawing on the thesis board (`BOARDS.md`) is the whole point at this scope:
three positions of the same frame drawn out, each at both widths and holding a real page, beat any
amount of prose about what collapse means. Where the frame moves, the board moves it: one live
control that collapses every thesis at once.

State the **mobile rule** in the thesis, not after the build.

---

## Building

Free-hand in the proto space, on the settled system's kit and tokens. One file per candidate.

**Every state draws at every width from one seed.** A candidate whose halves are `hidden lg:*` /
`lg:hidden` is two designs in one file, and at whatever width the stage is set to, half its variants
are empty boxes. Where a width genuinely cannot hold a state — one column cannot be both "collapsed"
and "both open" — the variant says so rather than disappearing.

**A variant is a state of the app, not a screen size.** If two variants differ only by width, there
is one variant.

---

## Comparing

The frame it replaces is the baseline, at every width, with the same page in both. A frame round
that compares against nothing is a redesign of navigation judged on whether the new one is pretty.

---

## Settling

- Every destination opens something, and pressing the open one does whatever the design says it does.
- Every state draws at every width; none of them is an empty box. `PROBES.md` §7.
- The mobile rule is stated and holds at the real breakpoint, in a real iframe.
- The frame **leaves whole** — the shell and the page inside it settle together, or neither has.
- It promotes as a **controlled shell plus a hook**: the shell takes state and handlers so any
  position can be seeded; the hook carries the settled behaviour — which control opens which half,
  and what the screen guarantees (there is always at least one half).
- The places are rebuilt on it. That is the other half of settling: the contract is only real once
  something outside the kit has had to meet it.
- Anything a place used to do for itself that the frame now does is **deleted from the place**, not
  overridden in it.
- Rejections appended to the bans file, with the dead marks and dead mechanisms named — a frame round
  generates more "do not re-propose this exact thing" than any other scope.
