# Tally: the manifest

**Only the user changes this file,** in a pull request the user asked for.

Accepted by the user on 2026-11-25 ("its ready"). It is the fixed measure for every brief, study and design. The user's own words behind each line, with dates, are in `tmp/active/manifest-draft.md`. Only the user changes it.

## 1. What Tally is

Tally is a language in which a party can fully model its financial reality, and the engine that runs that model. Its scope is the household: everything one household needs to manage its own finances well, not the economy at large.

It is one system with one model. Because the rules, the facts and time all live in that one model, the same model can do three things:

- say what the rules require, whether the law's or the household's own;
- hold what really happened and show where it departs from the plan;
- project what happens next.

These are abilities of the one model, not three parts joined together. How it is built inside is free, as long as the result is clean, fast, reliable and not insanely hard to build.

## 2. Why it is built

Tally is worth building only if it gives an edge over what already exists. It has that edge when all four hold at once:

- Claude can write whatever the household needs in it and is never told it can't be expressed;
- any law and any household rule can be written in it;
- it can be run under many scenarios and under uncertainty;
- it answers in reasonable time in normal use.

If one of the four is missing, the language is not worth having.

## 3. What it must be able to do

Six abilities are the measure every design is judged against.

1. Express any law and any household rule.
2. Forecast from them at any horizon, from a single month to many years.
3. Compare scenarios against a baseline.
4. Calculate under uncertainty. Some things are uncertain but have a probability distribution, known or assumed, and Tally can calculate across many possible combinations of their outcomes. Monte Carlo is one way to do this, not the requirement itself.
5. Replay against what really happened.
6. Do all of it fast enough for normal human use.

**The test:** in normal use, adding a new law, a new rule or a specific calculation within Tally's scope never ends with "that can't be done, a feature is missing".

**How complete, and when:** nothing may be missing when the language is finished. Work that belongs to a later milestone waits for that milestone. A combination of things that are already built is settled when both exist, and is not pushed to an unrelated later milestone.

**How fast:** fast enough to work well in normal human use. Speed never costs a feature, reliability or traceability. The engine is built capable first and made faster afterwards, with tools that show where time and compute are wasted. Thousands of futures for a complex household over 50 years are not required.

**The speed floors** (approved by the user, 2026-11-26). Slower than a floor, Tally makes no sense; a floor is checked at every milestone, on documents of a real household's size, on the reference Mac. The comfortable level is an aim for the last milestone, not a gate. The spec's earlier budgets (a 33-year forecast at ten times a household's size in 1 s; 1 000 futures over 30 years in 1 s) were "set arbitrarly" and become aims of that kind.

| What a person is doing | Floor | Comfortable |
|---|---|---|
| Claude or the app changes a document and checks it | 5 s | 1 to 2 s |
| One 30-year forecast of a household, day by day | 2 min; 1 min once the performance run has measured it | 10 to 20 s |
| A baseline against five scenarios | 10 min for all; 5 min once the forecast floor tightens | 1 min |
| 300 futures over 30 years | 30 min | 2 to 3 min |
| Memory for any of these | 8 GB | 2 GB |

## 4. Who writes it, who reads it

An LLM and the app write Tally, and people see it through the app.

- **The writer is a machine.** A Tally document is data. Nobody has to learn the language to use it; the app shows it to people in whatever form suits them.
- **It is general within its scope.** It models any household's reality, not one particular household's. It is proven on hard cases worked out by professionals, on the reasoning that an engine that handles those handles a small household.
- **It stands alone.** The engine is a standalone program with nothing of the app in it. It is built for Tropek, and no outside audience is planned.

## 5. What the language should be like

- **A small core.** A solid base of a few building blocks from which any reality in scope can be modelled. Nothing is added for one case alone.
- **Explicit.** The writer always states what is meant. Without a statement, it does not run; there are no hidden defaults.
- **Capable, not complicated.** As capable as the abilities demand, and no more complex than that needs.
- **One seamless thing.** A part either belongs to the one model or is cleanly isolated behind a clear boundary. Nothing is held together by patches. For Tally, most of it is the one model.

## 6. How it changes

The abilities are fixed; the text of the language is free to change.

- **The goal is the abilities, not the text.** The spec may end up a completely different language, as long as the abilities are delivered.
- **No ability is lost without the user's yes.** Removing or narrowing one happens only in an extreme case, and is always put to the user first.
- **Redesign beats patching.** When something clearly doesn't work, it is redesigned, not patched case by case.
- **The warning sign.** A feature that keeps producing edge cases, or grows into a round of work of its own, is probably wrongly designed at heart.
- **Few rules.** A failure is not answered by piling new rules on the old ones.

## 7. Still open, for the research

- **What the small core is:** which single model makes rules, record and forecast emerge from it, and which features are built on top of it.
- **When a redesign starts:** "too many edge cases" has no number and no owner yet.
