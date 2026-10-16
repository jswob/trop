# Gear: correct

Use when the user's message carries the **answer** rather than a problem: "rows should be 28px",
"drop the shadow", "#3 this should say the app name, not the page name".

This is the common gear once a system has settled. Most rounds are a numbered list of things wrong
with something already on screen, usually with screenshots, written after the user has _used_ it.
The loop in `SKILL.md` does not apply — there are no theses, no candidates, no comparison. What
applies is this.

---

## Announce

One line: the gear and the count. `CORRECT — 4 items.` Then work them in the user's own numbering
and keep that numbering all the way to the report. Never renumber, never merge two of their items
into one, never silently drop one.

If an item is genuinely ambiguous, do every other item first and ask about that one at the end.
Blocking four fixes on one question is the wrong trade.

---

## Classify each item before touching it

Three kinds, and they route differently. Getting this wrong is what produces a bans file full of
typo fixes and a traps file with nothing in it.

| Kind         | What it is                                                                                                  | Where it is fixed             | What it logs                                                            |
| ------------ | ----------------------------------------------------------------------------------------------------------- | ----------------------------- | ----------------------------------------------------------------------- |
| **defect**   | It does not do what the design already says. A page overflows; a bar is 20px off the edge                   | at the layer that _caused_ it | a **trap**, if the cause generalises. No ban — the rule already existed |
| **decision** | The design says X, the user now wants Y                                                                     | wherever X is stated          | a **ban**, and possibly a **revocation** of an earlier one              |
| **gap**      | The design never said. Pressing the open destination does nothing, because nobody decided what it should do | wherever the decision belongs | a **ban**, stating the rule for the first time                          |

A decision that reverses an earlier ban is the one to slow down on. Find the ban it kills, strike it
through, name the round that killed it, and say in the report that it was reversed. A bans file that
accumulates contradictions is worse than no bans file, because it is read as authoritative.

---

## Reproduce before fixing

**A screenshot is where the item came from, not proof of the cause.** Reproduce it as a measurement
— a probe from `PROBES.md`, or a targeted one — and write down the number. Without a before-number
you cannot tell a fix from a coincidence, and half of these items are two pixels.

The failure this prevents, concretely: reading a position off a screenshot when the capture's
coordinate frame is not the CSS viewport, concluding the root font size is overridden, and going
hunting for an override that does not exist. Measure in the page.

---

## Fix at the layer that caused it

Walk down, not across:

1. **Is it the prototype's own bug?** Fix it there. Do not touch shared code because the prototype is
   the thing on screen.
2. **Is it the shared component's bug?** Fix it there — and then **check every other caller**. A
   component with three callers has three chances to be a regression.
3. **Is it the design system's rule that is wrong?** That is not a correction any more. Say so, and
   ask whether to open a round.

**Never widen a settled component's API for one caller without making the addition opt-in.** A new
prop with an undefined default, a new behaviour that only switches on when a handler is passed: the
existing callers must come out byte-identical in behaviour. Then verify that they did, by loading
them, not by reasoning about them.

**A correction is not an invitation to redesign.** Fix the item, not the neighbourhood. If working
on item 2 reveals something else wrong, finish the four items and raise the fifth thing separately —
the user gets to decide whether it is worth a round.

---

## Verify with the same probe

Same measurement, after. Both numbers go in the report. Then re-run whatever the item touched at
every width the design has a breakpoint for — most of these items only exist at one width, and the
fix for one width is how you break another.

Where the fix touched shared code, load the other callers and check the thing that would break:
the specific button, the specific label, the specific absence of a focusable element.

Then the commands, on the frames the item touched: the sweep, and the thesis's numbers where the
thing has a thesis file (`PROJECT.md` names both). They take under a minute, and they are how a fix at one
width is caught breaking another.

**A correction gets no reader unless it touched shared code.** Where it did, one short read of the
change against the matching bans, as `REVIEW.md` describes. A correction to one screen's own file
is checked by the commands and by the user who asked for it.

---

## Report per item

One item, one short block, in the user's numbering, opening with what they asked. The block says:

- **the cause**, in one sentence — not what you changed, what was wrong
- **the fix**, in one sentence
- **the measurement**, before and after, where it is a measurable thing

Then, once for the whole message: lint, tests, blast radius, and the round number logged.

**Carry forward what you found and did not fix.** If an item exposed something the user has not seen
— a page that does not use the width it is given, a control with no data to print — say it in one
line at the end. Do not fix it, and do not bury it.

**Say when you were wrong.** If an item reverses something you proposed and built two rounds ago,
say that plainly, once, and log the revocation. Do not litigate it.

---

## Log

- **defects** → `TRAPS.md` in this skill, but only when the cause generalises past this project. "The
  native scrollbar rides the scroller box, not the container" is a trap. "The blurb said Timline" is
  not a trap and is not anything; it is just fixed.
- **decisions and gaps** → the project's bans file, as one checkable rule each, with the user's own
  words in parentheses.
- Correction rounds get a **letter suffix** on the round they are correcting — `26f`, `26g`, `26h` —
  so the record shows that the frame took three passes, which is a true and useful thing for it to
  say.

Nothing else is logged. A round of four items where three were typos logs one rule.

---

## The shape of a good correction round

Drawn from a real one, so the numbers are real:

```
CORRECT — 4 items.

#1 "orders view on mobile overflows the container"
   The page went straight into the flex child with no scroller, so a document taller
   than the glass ran past the frame. Wrapped it in RuledScroll like every other
   column. Frame document: scrollWidth 437 → 390 at 390.

#2 "logo should be changed to the back arrow icon"
   ...

Reverses 26f's "the mark in the slug is the way back" — struck in the bans file,
with the reason.

lint clean · 1603 tests pass · nothing outside design/ · logged as round 26g.

Still open, not fixed: the table does not use the width it is given on the landing
page (1672px given, ~340 used).
```
