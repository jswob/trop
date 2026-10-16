# Scope: system

Use when there is no design system yet, or when the system itself is what is wrong — not a
particular screen.

---

## Canvas

An **invented composite stress page**. Not a real view, not a swatch grid. Laid out as a plausible
screen so page-scale rhythm is judged alongside everything else, and holding, in one desktop frame:

- an action row — a filter left, one primary action right
- a dense data block, ~12 rows, with status marks and row actions
- a strip or region showing **every state the domain has**, all at once
- an open modal over it, with form controls — input, select, a validation error
- one region in its empty state

Everything you are choosing between is then visible at once: density, depth strategy, accent
discipline, status treatment, type scale, rhythm. A candidate that looks good until two of these sit
next to each other has not been judged.

Data is literal. No fixtures, no queries, no API. The candidate brings its own shell, because the
current app shell is part of what is being replaced.

Candidates live in the proto space, one file each, free-hand. **No existing kit components, no
existing tokens** — those are the thing being replaced, and building from them re-anchors the output
on the past.

---

## Axes

Pick values before writing theses, then make sure no two theses share a value on the same axis.

| Axis        | Options                                                                       |
| ----------- | ----------------------------------------------------------------------------- |
| **depth**   | lightness delta only · hairline borders · shadow · a mix, stated explicitly   |
| **density** | tight grid (28–32px rows) · generous (40–44px) · mixed by region              |
| **accent**  | one accent, once per viewport · status-led · accent as structure              |
| **type**    | one family at four sizes · a display/UI pair · one family with a numeric face |

State numbers, not adjectives. "Dense" is not a thesis; "32px rows, 13/18 UI text, 1px hairlines at
8% white" is.

---

## Theses

Three. Each gets a name, a one-line bet, its axis values, and a drawing on the thesis board
(`BOARDS.md`): the composite stress page, drawn in that thesis's own proposed tokens, since nothing
is settled at this scope to borrow. Present all three on one board and stop. Do not write UI code
until the user has killed or redirected.

```
B — "Instrument panel"
    Status colour carries the hierarchy; the surface is near-silent.

    depth    lightness delta only, no borders, no shadow
    density  32px rows, 28px controls
    accent   status-led — the only saturated colour on screen is state
    type     one family, 13/15/20/28, tabular figures in the grid

    drawn    the stress page at 360 and 1280, in these tokens, on the board
```

---

## Baseline

There is no baseline for an invented page, so use the closest real screen — a dense one, with the
app's real chrome on. It shows the current density, the current depth strategy and the current
palette on comparable content, which is what the comparison is for.

---

## Questions a system owes an answer to

The ones that come back to bite if a round leaves them open:

- **What carries state?** If the answer is hue, say what happens in a desaturation test.
- **What does hue do, and how many jobs does it have?** A system where colour has no stated job
  grows one per screen.
- **What is depth?** One answer, everywhere.
- **What is the radius, and is there an exception?** If there is one, write it down as an exception
  rather than a precedent.
- **What is the minimum control height, and what does it become under a finger?**
- **Where does emphasis come from once size and indentation are spent?**

---

## Settling

1. **Tokens** — a new namespace. New names generating new utilities. Never override the existing
   token names; the baseline uses them and the baseline is the before-picture.
2. **Primitives** — only the components the stress page actually used. Typically around seven: shell,
   panel, button, input, select, badge, table row. Nothing speculative.
3. **Write the system down** — a new document in the design root, describing _when_ to reach for each
   token, not only what each one is. Do not modify the current app's system doc; it describes the app
   that is not changing.
4. **Build the kit's own stress page** — every primitive on one sheet at once, which is the only way
   to see whether they still agree with each other, and the reason nothing in the kit is speculative.

The kit deliberately stays small. Views grow it from demand — a component enters it on its **second**
caller.

## Acceptance

- Every thesis's axis values are visible in its candidate and match the numbers stated.
- The a11y floor holds in all three: AA contrast, visible focus, colour never the sole carrier.
- The baseline frame is in the comparison.
- No candidate imports from the app's components or uses an existing token name.
