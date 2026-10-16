# Boards: showing a round before it is built

A board is a private Artifact page that shows the user what the round is about to build, drawn
before anything is built. There are two kinds, one per step of EXPLORE that happens before code:

| Board        | Step | What it shows                                                                      |
| ------------ | ---- | ---------------------------------------------------------------------------------- |
| **research** | 1    | How other fields draw the same problem, each pattern redrawn on the project's data |
| **thesis**   | 2    | The N bets, each drawn as it would look, on the same data, at every breakpoint     |

**Boards replaced the ASCII layout.** A thesis drawn in ASCII is judged by imagining it, and the
user's imagining and the builder's never agree. A drawing on the project's own data is judged in
seconds, as seen. It came out of round 60f: a research board of six spacing patterns, each redrawn
at 360px with one person's history, settled a question that three text-and-ASCII rounds had not
(_"I also like how you utilized artifact"_). The user then retired ASCII theses: _"Instead of
showing me some weird ascii representation of what it wants to build it should draft what it wants
in an artifact before I accept it for prototype."_

---

## Where a board lives

- **In the session's scratchpad, one folder per board.** It is never in the design root and never in
  the repo. Nothing imports it, nothing registers it, nothing promotes it. It is not a candidate,
  so it is not an exception to Rule 0: it is outside the repo entirely.
- **Published with the Artifact tool**, private by default. Follow the host's artifact rules (the
  page contract, theming, the size limit) on the first publish.
- **A redirection republishes the same file to the same URL.** A new question gets a new board.
  Put the round number in the header so an old link says which round it belongs to.
- **With no Artifact tool**, write the same page to the scratchpad and open it in the browser. ASCII
  is not the fallback.

---

## Drawn with the real parts

**Where `PROJECT.md` names a board kit, a drawing on a board is a real frame**: the project's own
components, stylesheet and data, built from the working tree into the board's folder and opened at
its true size. It is the standard, not an option (_"yes, make sketches from real parts the
standard"_).

It replaced drawing by hand. A board used to redraw every component in plain HTML with the tokens
copied by value, so what was approved at step 2 was a look-alike of what got built at step 3.
Measured on one settled page: the hand-drawn version had 8.7% of its pixels wrong against the real
frame, and the kit's had none.

A drawing is one of two things, and neither is redrawn:

- **A frame that already exists** — the baseline, a settled place, a component's specimen — is
  **mounted by name**. The current thing on a thesis board is the current thing.
- **A new drawing** is **a file in the board's folder, written as a prototype is written**: the
  same imports, the same shape, the id it will be registered under. Whether it is built from the
  kit or drawn from nothing — a pattern from another field has nothing to mount — is the drawing's
  own business; the language it is written in is not.

Three consequences:

- **Build the kit again after every change to a drawing.** It is built from the working tree each
  time, so a board never carries a stored copy of a component that has since moved.
- **The page around the drawings is still a page.** The header, the brief, the tables and the prose
  are written by hand, on the kit's stylesheet and the system's variables rather than on values
  copied out of it.
- **A drawing file is not a candidate.** It lives in the scratchpad, nothing registers it and no
  gate runs on it. Hard rule 2 stands: nothing is written in the design root before the theses are
  approved.

**With no board kit**, a board is drawn by hand, and the list below says how in each item.

---

## What every board carries

1. **A header that says what is being asked.** One `h1` stating the question in plain words. The
   round number. One paragraph on what came before: what won, what died, with the user's own words
   where they decided it. Then **the data line**: what is drawn and how big it is ("118 orders
   over about two years, 18 staff in two teams, gaps around 7").
2. **The project's real data.** A drawing file imports the project's fixtures; a hand-drawn page
   copies the numbers from them rather than inventing a tidy sample. Invented data is always well
   behaved, and a drawing that holds on 12 even rows can fall apart on 118 uneven ones. Every
   drawing on the board draws the **same** data, so the drawing is the only thing that differs.
3. **Every breakpoint the design declares.** Each drawing appears at phone width and at desktop
   width side by side — with a board kit, each in a frame of that true width, so the design's own
   breakpoints are what lays it out. A board drawn at one width hides the mobile rule, and the
   mobile rule is where most view theses fail.
4. **The settled system's look.** With a board kit the page links the kit's stylesheet and uses the
   system's variables; without one, copy the tokens _by value_ from the theme file and load its
   faces. A board in some other palette is judged as a different product. At system scope, where
   there is nothing settled, each thesis is drawn in its own proposed tokens.
5. **The bans still apply.** A drawing that uses a banned device is a ban re-proposed, even though
   it is only on a board. Read the bans for the subject before drawing, as for a build. The
   anti-defaults apply too: no eyebrow labels, no `→`, no middle-dot meta strings in the board's
   own chrome.
6. **Plain language.** Each bet is a short paragraph the user reads once: what you see, what you read
   off it, and what it gives up. Axis values go in a small table where they are numbers. No code
   names (no `STATE_INK` or `ink-3` in the prose unless the question is about the token).
7. **What stays the same, listed once.** A "the same in all three" list above the candidates, so the
   eye only has to find the differences.
8. **Behaviour done, not described.** If an axis is behavioural, the board performs it. A real frame
   is live, so the control in it is the control; one switch shared by every drawing (the
   appearance, a state) re-draws all of them at once; a moment inside a move is a pinned frame
   beside its neighbours. A hand-drawn page does the same in vanilla JS and inline SVG or HTML, in
   one file with no build step.
9. **Names that carry through.** Each candidate is named as it will be registered (`A · Stems`,
   `C·1 · Count at the row`), so "I like C-1" maps to a file with no translation. A drawing file
   carries the id it will be registered under.
10. **No ranking.** The board never scores, orders by merit or badges a candidate. Hard rule 3
    applies to a board exactly as it applies to a review. Recommendations belong in the
    conversation, about what to _decide_, and never about which bet is better looking.

---

## The research board (step 1)

Used when a question from the interview cannot be decided blind: how do others draw this?

- **Patterns from outside the project**: other fields' printed and visual artefacts (timetables,
  clinical adherence charts, transit headway plots, ledgers). Each one is **redrawn on the project's
  data** at phone width, with a link to its source. A pattern shown only as a thumbnail from
  somewhere else tells the user nothing about whether it holds this data.
- **Direct answers at the top.** When the interview raised a hard constraint, answer it on the board
  in a sentence with numbers (_"none of the patterns puts the timeline across: at 55 orders that
  is 6px each on a phone"_), and name the ban it would break.
- **A "looked at and left" section.** Patterns considered and why they fail here. It stops the next
  round re-researching the same ground.
- **It feeds the next question; it does not end the step.** After the board, ask one question
  against it (which pattern to carry into the theses) with a recommendation, the way the interview
  asks everything else.

## The thesis board (step 2)

- **The brief at the top**, as agreed at the end of step 1, in a few lines.
- **N theses, spread on named axes.** A small table of axis values across the candidates shows the
  spread, and makes an accidental shared value visible before anything is built.
- **The current thing first, labelled as such**, when one exists: the baseline view, or the previous
  round's winner in an iteration. "Better" is meaningless without it (hard rule 5).
- **Each thesis states its mobile rule** in its bet, and the 360 drawing shows it.
- **Then stop.** The user kills, redirects or picks. A redirection edits the board and republishes
  it. Only approval moves the round to step 3.

---

## After approval: the board is the spec

- **An approved drawing file is where its candidate starts.** `PROJECT.md` names the command that
  copies it into the design root and registers it, and the agent builds on from that file. The
  drawing is not read and typed again, which is the step where a build used to drift from what was
  approved.
- **`BUILD.md`'s brief hands each agent the board's file path and names its section.** The candidate
  builds the drawing the user approved, in the project's kit, and nothing adjacent to it.
- **The thesis's numbers are written down from the board the moment it is approved**, in the
  candidate's thesis file and before any agent builds. Every figure the board states (a row height,
  a gap, a line count) becomes a check a command holds the build to.
- **`REVIEW.md`'s reader gets the board's path too**, for the structure a number cannot carry.
- **Keep the board's folder until the round settles**, then leave it to the scratchpad's cleanup.
  The round's record is the bans file. The board's page is never copied into the design root, and
  the workbench comparison (step 5) is always of the built candidates, never of the board.
