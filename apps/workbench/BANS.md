# Bans

Concrete rules, accumulated from rejections and **folded**: organised by subject rather than by
round, with each rule carrying the round that produced it. Read at the start of every design
session. Appended in the same session a rejection is voiced — never later, never from memory, and
through `bun workbench ban add` rather than by hand.

**Format.** A ban is a rule you could check, not a feeling. Write what must not happen and, where it
is not obvious, one clause on why.

```
- No shadow on any surface; depth is lightness delta only.   (2 — "the cards look stuck on")
```

Not: "too heavy", "felt off", "less corporate".

**When a later round reverses an earlier rule**, strike the old line through and name the round that
killed it. Never delete it: a ban that vanishes gets re-proposed, and a struck one is a record that
the ground was already walked.

**What is not here.** The engineering traps a design round turns up are universal and live with the
skill, in `.claude/skills/design/TRAPS.md`. The positive half of the system — what to reach for,
rather than what not to — is the theme (`packages/ui/src/theme.css`) and the kit.

**The sections start empty.** Each fills from the user's own rejections. §12 is the only one with
rules in it on day one, and they are defaults to argue with rather than settled taste.

---

## 1. Hue and status

## 2. Depth, geometry, shape

## 3. Type and hierarchy

## 4. Density and controls

## 5. The ground

## 6. Navigation and chrome

## 7. Below `lg`

## 8. Records and forms

## 9. The spread

## 10. Motion

## 10b. Waiting

## 10c. What earns a place, and what earns a state

## 11. Settled — do not re-open

The winner, and what died beside it. Re-proposing anything on the right is re-running a round the
user already paid for.

| Round | Settled | Dead, do not re-propose |
| ----- | ------- | ----------------------- |

## 12. Pre-seeded

Not from the user's rejections — from the research the design skill was built on and
`frontend-design`'s calibration list. Defaults to argue with, not settled taste. If the user
deliberately chooses one, delete the line and note that they chose it.

- **No Inter**, unless argued for explicitly. It is the most recognisable generated-UI tell.
- No identical rounded cards with one radius for everything and the same soft grey shadow under each.
- No tracked-out ALL-CAPS eyebrow label above a heading.
- No meta strings joined with middle dots — `A · B · C`.
- No monospace face for small data labels as decoration; only where figures must align.
- No `→` appended to button or link text.
- No numbered markers (01 / 02 / 03) on content that is not a sequence.
- No fade-and-slide-up on every section as it scrolls in, and no hover transition on every card.
- No accenting a single word in a headline by colour, weight or italic.
- No warm cream background with a high-contrast serif and a terracotta accent near `#D97757`.
- No near-black background with a single acid-green or vermilion accent.
- No tinted near-black (`#0B0B0B`, `#111`) standing in for black.
