# Review: the short read

One fresh reader, after the user has picked, on the winner only. It reads the change against the
bans. It measures nothing.

---

## Why it is this small

This file used to describe a reviewer that did everything: census, conformance, probes, and
pressing every control at every width, in a session that did not build the thing. Thirty-three of
those runs were read back from their transcripts:

|                                              |                                          |
| -------------------------------------------- | ---------------------------------------- |
| time                                         | 11 minutes at the median, 42 at worst    |
| catching up on bans, thesis, board, source   | 46% of it                                |
| writing one-off browser scripts to measure   | 28% of it                                |
| findings a command could have printed        | about 6 in 10                            |
| findings that needed a reader                | about 4 in 10, and none needed a browser |
| reviews of a candidate that was then deleted | two of every three, in four rounds       |

So the measuring moved to commands that run in under a minute while the candidate is built
(`SKILL.md` step 4, `PROBES.md`), and what is left here is the part only a reader can do.

**The argument for a fresh reader still holds, for that part.** An author has the intent in context
and reads the code through it. They see the mapping they meant to import, not the one they typed
again. A session that arrives with only the brief and the change sees what is there.

---

## When it runs

- **After step 6, on the candidate the user picked.** Never on one they did not: it is about to be
  deleted.
- **In the background, while the round's rejections are logged.** The user is never waiting on it.
  What it finds rides into the next build (step 8), and nothing settles with a finding open.
- **Once per winner.** A later iteration is read again only if it changed shared code or crossed
  into a new ban section.
- **Not for a `CORRECT` round that touched no shared code.** One screen's own file is checked by
  the commands and by the user who asked.
- **Not at all until the commands are clean.** A reader handed a candidate with a failing sweep
  spends its time rediscovering what the sweep already printed.

---

## Independent of the build, never of the brief

| Independent from        | Effect                                                                 |
| ----------------------- | ---------------------------------------------------------------------- |
| the author's intent     | **good** — it reads what was written, not what was meant               |
| the thesis and the bans | **fatal** — it has nothing to check against, so it falls back on taste |

A reader with no thesis and no bans in front of it does the only thing left: it scores the change
against a general idea of good UI, which is the one thing `SKILL.md` forbids. So it arrives carrying
all of the following, and it is forbidden an aesthetic opinion.

---

## What it is handed

Main gathers these and puts them in the brief. The reader does not go looking for them, because
looking is where the old review's time went.

1. **The brief and the thesis, verbatim**, with the path to the thesis file and to the board.
2. **The ban sections that match the round's subjects, verbatim.** `PROJECT.md` names the command
   that prints them. Main names the subjects; a reader that picks its own scope is deciding which
   rules apply.
3. **The change.** The list of files the round touched, and the diff of the ones that already
   existed. A new file is read whole.
4. **The system doc's path**, and which sections bear on the round.
5. **The last lines of the commands**: the thesis's numbers, the sweep, the probes, the census. So
   it knows what is already measured and does not measure it again.

```
git status --porcelain -- <design root>     # what the round touched
git diff -- <design root>                   # what changed in the files that existed
bun workbench ban find --for <subjects>     # the sections that apply, verbatim
```

---

## What it reads for

Four things, and each is something no command sees.

### 1. A ban broken by its meaning

A rule a lint cannot hold: _a disabled control with no reason is a dead end_, _a count that cannot
vary is not a count_, _the foot names the thing, not the page_. Read each matching ban against the
change and name the line that breaks it.

### 2. A second copy of something that already exists

A mapping, a sentence pattern, a red `!` line, a class string typed again from memory when the kit
already draws it. The census catches a _state_ painted twice; it does not catch the sixth hand copy
of an error line. Name the copy and the thing it copies.

### 3. Behaviour the brief described and the code does not do

The brief said the count follows what is accepted; the code counts what was proposed. The thesis
said a held page asks on every way out; one way out does not ask. This is read off the code, with
the brief beside it. Where the code cannot settle it, say what to press with `drive`, and leave the
pressing to the author.

### 4. Words that are no longer true

The system doc still says six destinations. A comment says twelve tokens and the block defines
fourteen. A state's note says 16px where the frame is 13. The round's own docs, comments and listed
states, checked against what the round changed.

---

## What it is never asked, and may not do

- **Open a browser, or write a script.** Anything it wants measured is a line in its report for the
  author to run. A read that has started measuring has become the review this file replaced.
- **Edit a file.** A reader that fixes is an author, and nobody independent is left.
- **Re-run the commands.** Their last lines are in the brief.
- **Say whether it likes the candidate**, in a ranking, a "minor note" or a closing paragraph. Taste
  is the user's, and they have already chosen.
- **Say whether the thesis was a good idea.** It reads the change against the thesis, not the thesis.
- **Report what passed.** A row that says PASS is a row somebody has to read to learn nothing.

An instruction to _rank_, _score_, _rate_ or _pick_ is the tell that a brief has drifted back into a
quality rubric.

---

## What it hands back

Failures only, each with the file and line, the rule or sentence it breaks, and one line of what
is there. Then what it could not settle by reading. Nothing else.

```
READ — round 68, A · Last cell

  views/sign-in.tsx:137    A sixth hand copy of the error line kit/Field.tsx:80 draws.
                           The two measure 20px and 19.5px line-height on one screen.
  views/sign-in.tsx:98     BANS §4 (35), "a disabled control with no reason is a dead end":
                           the empty-field plate is disabled with no describedBy.
  the system doc §8        Still says "one of six destinations"; the round made it seven.

Could not settle by reading: whether the keyboard returns to the field after a wrong
password pressed with the plate. Press it: drive account:signed-out --do "click:Sign in".
```

A read with nothing to report says so in one line.

---

## The brief, to adapt

```
You are reading one design change against its rules. You did not write it.

Read only. Do not open a browser, do not write or run a script, do not edit any file, and do not
re-run the checks: their results are below. Give no opinion on how it looks.

The brief and thesis, verbatim: <…>          Thesis file: <path>     Board: <path>
The rules that apply, verbatim: <output of the ban command>
The system doc: <path>, sections <…>
What changed: <file list>, and the diff: <…>
Already measured, do not repeat: <last lines of conform, sweep, probes, census>

Report only what fails, each with file and line and the rule or sentence it breaks:
1. a rule above broken by its meaning
2. a second hand-written copy of something that already exists (name both)
3. behaviour the brief or thesis describes that the code does not do
4. a doc, comment or listed state the change made untrue
Then, separately, what you could not settle by reading and what to press to settle it.
No rows that pass. If nothing fails, say so in one line.
```

**It should take a few minutes.** One that runs past ten has started measuring, or was handed a
scope instead of a change.

---

## What main does with it

Findings are the author's to fix and the round's to log. A finding that is a decision and not a
defect goes to the user as a question, one at a time, with a recommendation. A finding the same
reader makes twice across rounds is a candidate for a command of its own: that is how the bans that
can check themselves get written.
