# The engine's conventions: commits, pull requests, issues

Accepted by the user on 2026-11-05 (the retro after M0, decisions 6 to 8). Every title, and the top of every description, is written for the user: spec words, no Rust, no session jargon. What a session needs stays below a fold. Old commits and merged pull requests stay as they are; an open issue is retitled when a round takes it.

## Commits

`type(scope): what changes, in spec words`, at most 72 characters.

- **Types:** `feat`, `fix`, `spec`, `test`, `refactor`, `perf`, `docs`, `ci`, `chore`.
- **Scopes:** `parse`, `format`, `load`, `eval`, `walk`, `book`, `ask`, `num`, `time`, `refcore`, `fuzz`, `rules`. The walls, the tools and these texts are `rules`.
- **Body:** one to three plain sentences on why. Then the machine lines, one per line: `Issue: #56`, `Pick: P19`, the vector ids and files, and the `Logged-Red:` and `Rule-Change:` lines the checks read.
- **Never** an AI co-author or attribution line: no `Co-Authored-By:`, no `Claude-Session:`, no session link (R-100), whatever the harness's reminder says.

| Before                                                                                           | After                                                       |
| ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------- |
| M0, G12 (2 of 2): a field of an adjustment's adjusts is checked at load (issue #56)              | `feat(load): refuse an unknown field read through .adjusts` |
| T: M0 gate: G13's forms as vectors (…) (issue #56)                                               | `test(load): lifespan of a field, axis counts past 64 bits` |
| R-91: the format target counts a "not built" stop as the load target does, until M5c (issue #56) | `ci(fuzz): format fuzzing allows planned "not built" stops` |
| tally 2.1.4: a prefixed call prints a.b(x), touching its (; refcore follows (issue #80)          | `spec(format): a prefixed call prints as a.f(x), no space`  |

`engine/tools/commit-check` (CI, on a pull request; the guard, before every `git push`, over what it sends) fails a commit that touches `engine/`, `.github/workflows/engine.yml` or `.claude/skills/engine-*` with a subject off this form; a commit whose parent has no `engine/CONVENTIONS.md` is listed, never failed.

## Pull requests

**Title:** the outcome. `M1: household forecasts run`, `tally 2.1.6: spec round, 22 issues`, `rules: per-group review and the sums tool`.

**Description:** `engine/templates/pr.md`, through `gh pr create --body-file`. Its sections, in this order; the first five fit on one screen, about 300 words:

1. `## What this delivers`: two to four sentences in spec terms.
2. `## Changes you can see`: New, Refused now, Changed (was, now), Not yet (with its milestone).
3. `## Decisions taken for you`: a table of the picks that change behaviour (question, choice, why, issue); the rest counted and linked.
4. `## Risks and follow-ups`: each with its issue and impact.
5. `## How it was checked`: tests, gates, cost, time.
6. `## Merge notes`: what to merge first, what it closes (one `Closes #n` per line: "Closes #a, #b" closes only #a).
7. A folded `<details><summary>For sessions</summary>` block with the full technical record.

CI's `pr body` job (`engine/tools/pr-body`) fails an engine pull request whose description lacks the six headings, in order, or the folded block after them; `engine gate` needs it. It reads the description when it runs: after an edit, re-run the job.

## Issues

**Title:** `scope: plain statement`, e.g. `refcore: rejects a refund that names a planned payment (the spec allows it)`.

**Body:** `engine/templates/issue.md`, through `gh issue create --body-file`: **In short**; **Example** (the smallest Tally text, what happens, what the spec says, with its section); **Impact**; **Options**, the recommended first; **Decision** (which, by whom, when); a folded **For sessions** block (files, lines, refcore functions, how found).

**Labels:** one impact label, `impact: wrong number`, `impact: wrong error`, `impact: crash`, `impact: cosmetic` or `impact: process`, and a kind label beside it: `spec`, `refcore`, `build`, `tests`, `rules` or `ci` (`stop-report` stays for a builder's stop).

## Where they live

This file and `engine/templates/` (`pr.md`, `issue.md`, `brief.md`) are rule files: only the `rules` role changes them, through R-112. GitHub's own template folder isn't used, since every app made from this repo would copy it. The skill `engine-coordinator` points here.
