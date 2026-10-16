# Scope: component

Use for one component — a new one the views have demanded twice, or reworking one that is not
carrying its weight.

The cheapest scope. Candidates take minutes, so the ceremony is light: a thesis here is one sentence
and one row of the board's variant grid.

---

## Canvas

A variant grid — every state at once, in one frame, so the states are compared against each other
rather than viewed one at a time. **Enumerate the grid from the variant config** rather than
hand-listing, so it cannot drift from the code.

## States owed

Every interactive component answers for all of these. A candidate that only shows `default` is not a
candidate.

`default` · `hover` · `focus-visible` · `active` · `disabled` · `invalid` (if it takes input) ·
`loading` (if it can be pending) · `empty` (if it can hold nothing)

Hover and focus cannot be captured in a screenshot. Render them as explicit forced-state cells rather
than relying on interaction.

**A component that moves owes its motion too** — what changes, over how long, on what easing, and
whether it is still legible with `prefers-reduced-motion`. Review it as a filmstrip: several captures
of one frame at different pins. The catch is that a transition only exists if something causes it — a
cell that renders the finished state has nothing to pin, so the specimen has to make the change
happen.

---

## Axes

| Axis           | Options                                                                                        |
| -------------- | ---------------------------------------------------------------------------------------------- |
| **shape**      | the silhouette — height, radius, padding, whether it has a border at all                       |
| **weight**     | how loudly it asks to be used — filled · outlined · bare · text-only                           |
| **affordance** | what tells you it is interactive — the border, the background, the cursor, nothing until hover |

3–4 candidates. One line each is a sufficient thesis at this scope, drawn as one row of a variant
grid on the thesis board (`BOARDS.md`), with every state the component has in its columns.

---

## Rules

- **Second caller promotes.** A component enters the kit when a _second_ view needs it. Until then it
  lives inside the view that wanted it. One caller is a part of that view, not a primitive.
- **Drive variants from usages that exist.** Never invent a size or a variant because the set looks
  incomplete.
- **2+ variant dimensions ⇒ a variants config** exporting the cross product, re-exported from the
  component's entry point. Fewer than 2 ⇒ inline conditional classes. Where precedence rather than a
  cross product is the real logic, export a pure classes function instead.
- **A controlled component takes state and handlers; the behaviour that goes with it is a hook
  beside it.** Baking state in makes every caller drill props through it; leaving behaviour out makes
  every caller re-derive rules the design already settled.
- **An addition for one caller is opt-in.** A new prop with an undefined default, a new behaviour
  that only switches on when a handler is passed — existing callers must come out behaviourally
  identical, and you verify that by loading them.
- **A promoted component owes its page**: what it is and when to reach for it, every state, a line
  for every prop, its parts, what it holds and **what it refused**. A page that only says what a
  thing does gets re-read as a menu of options. Where the workbench draws that page from a typed
  declaration (`PROJECT.md`), declare it and never write it by hand: nothing checks a hand-written
  props table against the component, and an anatomy drawn by hand is wrong after the next
  correction.

## Settling

- Every owed state is rendered and passes the a11y floor — AA contrast, visible focus, `disabled` on
  the real attribute rather than a class that only looks disabled.
- The variant grid enumerates from the config, not from a hand-written list.
- Promoted into the kit only if a second view needs it — and then every caller is refactored onto it
  in the same round.
