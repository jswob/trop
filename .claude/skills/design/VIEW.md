# Scope: view

Use when the system and the frame are settled and one screen needs redrawing.

---

## What a view is

**A view is a place you can be: the smallest thing a person can arrive at, and the largest thing that
can be judged in one frame.**

The decomposition rule, in four consequences:

1. **One place per destination, plus one for the landing surface.** The ground is a place because on
   a phone it is where you are, and above the breakpoint it is the other half of every screen.
2. **A state of one surface caused by another belongs to the one that causes it.** A draft landing in
   date order in a list is the state of the page that creates it, not the list's. Where the causing place
   does not exist yet, the state waits with the component rather than being parked on a place it is
   not about. A bag of states belonging to no place is what makes a view unreadable.
3. **A variant is a state of the app, not a screen size.** Every state draws at every width from one
   seed. There are no phone variants; width is a control on the workbench.
4. **A place never draws its own chrome.** It declares a page, what that page calls itself, what is
   in the other half, an overlay and what the chrome says; the frame supplies the rest. Enforce it
   with a test — one file may import the shell, and a test asserts no place file does. A view that
   framed itself could disagree with every other view about where you are, and that would not show
   until both were on screen.

**A destination with nothing drawn in it is not a place yet.** A page whose only content is a
sentence saying it has not been designed cannot be judged, reviewed or compared, and placeholder
pages make the space they sit in look busier and less settled than it is. Each becomes a place in the
round that draws it.

**Removing a place does not remove the destination.** The nav is what the app _is_; a nav with two
entries would be a different design. And the frame still opens the undrawn one — with its head from
the nav and skeleton rows under it — because a frame is judged by what it does with a page it has not
been given.

**Corollary: do not draw a place in order to hold a slot.** A section with three of four pages
undrawn should look that way.

A screen that is not a place is not a view. Reusable parts go to the kit on their second caller; a
composite of them goes to the kit's stress page; what is still being argued about stays in proto.

---

## Before anything

**Read the baseline.** Open the current version of this screen in the workbench and read its source.
You need to know what the screen currently does — every state, every control, what data it shows —
before proposing a different shape for it. A redesign that silently drops a state is not a redesign.

List the states the current view has. The redesign owes an answer for each one, even if the answer is
"this state stops existing because the new structure makes it impossible."

---

## Axes

| Axis          | Options                                                                               |
| ------------- | ------------------------------------------------------------------------------------- |
| **structure** | what the screen _is_ — a table · a board · a grid of things against time · a document |
| **IA**        | what is primary, what is secondary, what moves off-screen or behind a disclosure      |
| **rhythm**    | how the page breathes — where the space goes, what aligns to what                     |

**Domain vocabulary belongs here**, not in the system scope. No general-purpose SaaS reference solves
a specific domain; the printed artefacts that domain already has do. Call sheets, departure boards,
rehearsal grids, order books, shift rosters, scores, timetables. Ask what form the content actually
wants before reaching for a table because a table is easy.

**A screen is drawn out of the app's own materials or it does not belong to the app.** Name the
materials explicitly — the marks, the figures, the way an interval is drawn, the way a count is
written — and check the candidate against that list. A screen that uses none of them reads as a
different product, and that is the most common failure at this scope.

---

## Theses

Two or three. Name, bet, axis values, drawn on the thesis board (`BOARDS.md`). Stop for approval.

At view scope the drawing is the whole point: it is where a structure that is awkward in flex gets
proposed instead of filtered out. It is drawn on the view's real data, because a structure that
looks right in outline can break at the real row count.

**Every view thesis states its mobile rule** — what stacks, what collapses behind a disclosure, what
is dropped outright, what the narrow layout actually is. Not "responsive". The rule.

**Check the three differ at rest.** Three candidates that differ only in what happens _while you
work_ draw the same thing in a still, and the round offers one design three times.

---

## Building

Use the kit for anything already promoted. Build anything else **inside the view file**. Do not
promote on first caller.

The frame is live, so a place has to be true of the screen it seeds and honest about the screen it
does not: press a control the place never declared and the chrome must print something correct —
the nav's own word for that destination, not the seed's.

---

## Comparing

Old and new in one viewport is the whole point. The baseline goes in the frame list beside the
redraw, at every width.

---

## Settling

- Every state the baseline had is answered for.
- Reviewed at every breakpoint; the mobile rule is stated and holds.
- Anything now used by a **second** view is promoted into the kit and both views refactored onto it.
- The place declares no chrome, and the test that asserts it passes.
- Each state sits in the place that _causes_ it, not the place it is _drawn on_.
- No variant is a screen width. If two variants differ only by width, there is one variant.
- `PROBES.md` run at every width.
- Rejections appended to the bans file.
