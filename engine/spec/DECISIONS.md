# Tally: decisions (tally 2.1)

Why tally 2 is as it is. This file has two parts.

- **Part I, tally 2's records**, from session 4's design round:
  - the fifteen records D-1 to D-15, one per problem (`problems.md` in the round's files, PR-1 to PR-15);
  - the settlements of the sketches, the critic's findings and the modeller's findings (phase 3);
  - the decisions after G1, A-1 to A-51 (phases 4 to 6);
  - the redesign review's verdicts;
  - the G2 decisions, GD-1 to GD-28 (session 4b);
  - the tally 2.1 records, R-1 to R-17, with G1's decisions and the spec round's picks (sessions 7 and 7b).

  **Which text each part cites.** D-1 to D-15 and the phase 3 settlements were written before tally 2's text existed: in them, L, E and D are **tally 1.0.1's** `LANGUAGE.md`, `ENGINE.md` and `DECISIONS.md`. From "Decisions after G1" on, L and E are **tally 2's**. Spec rounds after the review cite tally 2's sections. Each record's "Change" section sketches a grammar that `LANGUAGE.md` §2.3 settled. Where they differ, `LANGUAGE.md` holds the form and the record holds the reason.
- **Part II, tally 1.0.1's decisions**, carried as they are. They are the reasons for everything tally 2 kept: `source`, `axis`, `index`, `band`, the rule form, `given`, `fill`, `round`, exact numbers, units, ownership, `as if` and the rest of `core.md`'s "Kept unchanged". Where a tally 2 record changes a decision of Part II, the record holds. `MIGRATION.md` names each changed construct with its record, and the round's trace names each closed item with its record. Part II is carried whole, not cut row by row, so that no reason is lost. Its references are to tally 1.0.1's texts (`engine/spec/` at the tag `tally-1.0.1`).

---

# Part I. Tally 2's records

Written in phase 3 of the design round (2026-10-25), one record per problem of `problems.md`, PR-1 (LC-29's frame) first. Each record follows the section list of the brief (§5, phase 3): Problem, Today, Solved form, Reduction, Change (with its kind: **sugar**, **library** or **core**), Explicit form, Alternatives, Axes closed, Effect on the 79, Shown, Checked against, Open. Record D-n answers PR-n. The grammar in the Change sections is a sketch, settled in phase 4 (`spec/LANGUAGE.md` §2.3 of tally 2); the meaning written beside it is the decision.

References: L is tally 1.0.1's `LANGUAGE.md`, E its `ENGINE.md`, D its `DECISIONS.md` (all in `engine/spec/`); shelf files are in `tmp/design/prior-art/`, new research in `prior-art/`. "The 79" are `engine/spec/models/`, with the household and its seven changes; reach is read from `engine/spec/coverage.md`.


**Codes the records name that the definition doesn't** (the final reviewer's finding; `xref_check.py --cite decisions.md`). The Explicit form lines and the critic's table name refusal codes as decided at G1. The catalogue is L §8 alone. Eight names in these records have no row there:
- `LOAD-COHERENCE`: removed in the record that names it (C-6, D-2).
- `RUN-UNGOVERNED`: the gap rule's hole, `HOLE-UNGOVERNED` (L §8.1).
- `RUN-PENDING`: went with the `pending` value (W-1, after G1): a binding empty at the cut whose span reaches past it is a hole.
- `HOLE-STALE`: went with the carry's `within n days` limit: a rule tests a reading's age with `since(x[k], d)` (A-2; L §4.6).
- `LOAD-CERTAINTY` and `LOAD-CONVENTION`: the grammar requires both. An event kind states `certain` or `uncertain`, and `round`, `prorata` and a conversion take their mode (L §2.3, §7.2). A missing one is `LOAD-PARSE`.
- `LOAD-CARRY`: dropped at G2 (GD-8). L §4.6 makes a plain read of a reading away from its instant a hole, where D-3 said `LOAD-CARRY`.
- `LOAD-PRIVATE`: went with `regime` at G2 (GD-1). D-2's Explicit form names it for a regime's private item; with no regime, no item is private.

The records' `L §5.13.1`–`§5.13.3` cite tally 1.0.1's sections, as the References line says.

**The one idea (LC-29).** The learner saw three subsystems glued together: law, money in places, and forecasts with draws. Tally 2 answers with one idea taken from temporal data models and resource-event-agent ledgers: **everything the model talks about is a declared thing of a declared kind, every statement about a thing is a dated fact, and a movement is a thing too, of an event kind.** Law is rules over facts (as before). Money and units sit in places, which are things of place kinds (D-1). A movement, planned or observed, is one construct with legs, of an event kind, and a regime is a unit of rules that reaches movements and places by their kinds (D-2). The forecast is a query over planned movements (D-6), and a random quantity is a fact whose value is a distribution (D-11). The five records D-1, D-2, D-6, D-8 and D-11 are this one idea applied to five parts of 1.0.1.

---

## D-1. Things are declared: kinds, lifespans, dated attributes and positions in any unit

**Problem.** PR-1: an entity has no declared lifetime, kind, unit or value of its own, so its absence is written as zero, every reader repeats a guard, and a total over entities silently counts or skips what it shouldn't. Items (12): `P:OWN-01`, `Q:E1-01`, `O:A-4`, `O:A-7`, `O:E1-4`, `O:E1-6`, `O:E1-7`, `LC-12`, `LC-13`, `LC-17`, `LC-29`, `G:C6-F9`. Axes: CUR, FLOW, LIFE, OWN, PTY, REC, SURF, VAL.

**Today.** A thing is a key of an index, and a key added by a version exists on every day of the run and is never removed (L §5.3, §5.6; D S-05). A thing is ended by rows that make it 0 or `-` and started by a zero row before its start (L §6.2, §6.5 change 1), so "doesn't exist" is written as 0 and every rule that counts keys repeats a guard (G-1). An input keyed by an index ranges over every key, so a key of another sort is stubbed in every input (G-2). A place exists every day, has a built-in kind and one unit for all its keys (L §5.9), so a keyed place over instruments in several units can't be declared once (`O:A-4`, `O:E1-4`). Value is `bal` for money and `worth` for holdings and assets, conversion exists only on `flows` (L §5.13.3), and net worth is rebuilt by hand per kind and currency (L §3.6; `LC-17`, G-7). A thing has no text attribute (`O:A-7`, `O:E1-6`). `DECISIONS.md` chose M-08 and M-10 (lifetime by rows), M-12 (parties are keys), M-19 (valued things are places), M-22 (a new thing is keys and rows) and S-05 (keys never removed), and rejected a second kind of thing (M-02).

**Solved form.** Three known structures. (1) **Temporal entities with lifespans and dated attributes, no nulls in storage**: anchor modelling (Rönnbäck et al. 2010), TimeER's lifespans (Gregersen and Jensen 1999), SQL:2011 application-time periods (`prior-art/rea-entities.md` form 2). (2) **Declared kinds as coherent classes**: type classes (Wadler and Blott) and Rust's traits with coherence (`prior-art/kinds-modules.md` forms 1–2); shelf: OpenFisca's entities and roles (`openfisca.md` §2.2), Beancount's `open` and `close` (`double-entry-ledgers.md` §2.1), ACTUS's lifetime ending at the earliest of several dates (`actus.md` §2.4). (3) **Positions in any unit valued through a dated price table**: Beancount's inventories and `get_price`, hledger's valuation (`prior-art/rea-entities.md` form 3; `double-entry-ledgers.md` §2.3–2.4), with their fallback (an unpriced position left in its own unit) inverted to a refusal.

**Reduction.**
- A thing is an entity's identity: a key of a **kind**. Its lifespan is TimeER's lifespan, stated when the key is added. Its attributes are anchor modelling's attributes: inputs keyed by the kind, each dated by its rows' spans.
- "Not alive on day d" is the absence of every fact, which reads as not applicable (`none`, D-7), never as 0 and never as a hole. "Alive, but no row" is a fact a row could give: a hole, as today.
- A subkind is a class with a superclass: every thing of `ordinary` is a thing of `brokerage`. A rule that names a kind reaches its subkinds (D-2 uses this for regimes).
- A place is a thing of a **place kind**; a position is Beancount's (place, unit) quantity; a value is quantity × price, read from a dated price fact, and a missing price is a hole.

**Change** (core).
```
kind person
kind current: place DUK signed            # a place kind: one unit, any sign
kind line: place DUK max 0                # what is owed, never above 0
kind brokerage: place any min 0           # positions in any declared unit
kind ordinary: brokerage                  # a subkind
kind wrapped: brokerage
kind note: unit                           # each note series is a unit of measure

version open_2019
  cite bank_letter
  known 2019-05-01
  current + [konto] DUK lives 2019-05-01..
  ordinary + [broker] lives 2021-03-01..
  note + [n7] lives 2021-01-04..2031-03-01
```
- **`kind K`**, optionally `: P` (a subkind of P), `: place UNIT|any SIGN [priced p] [outside]`, `: unit`, or `: event …` (D-2). A kind is an index whose keys are things. `index` stays for keys that aren't things (years, options, integers) and is unchanged.
- **A key enters with its lifespan**, `K + [k] lives span`, the open end written as `..`. A later version may state a new lifespan for a key with `replace` (D-12), as a death or a closed account does. A thing the model itself declares, rather than a source, keeps 1.0.1's declaration form: `place owed[p in person]: debt DUK lives ..` declares one place of kind `debt` per key, with its unit and lifespan.
- **Reads and lifespans.** A read of an input `x[k]` on day d takes the row that governs d, if one does, even outside k's lifespan (a court's confirmation about someone who has died is a stated fact). Where no row governs, the read is `none` when d is outside k's lifespan (nothing is expected there) and `HOLE-UNGOVERNED` inside it (D-3). A def keyed by a kind is defined for every key and computes with these reads at its own law date. A movement's facts (its legs, its attributes, and defs keyed by it) don't lapse: an event's facts hold on every read day, so the lifespan of a movement only places it on its day for bindings (M-1). Facts about a thing stay readable at days inside its lifespan: `lifespan(k)` gives its span (`.from`, `.to`), so a rule about an estate reads the deceased's facts `at lifespan(p).to`. A place's lifespan may end only when every position in it is zero, and a leg into or out of a thing outside its lifespan is refused (`RUN-LIFESPAN`), so ending a thing never makes money disappear from a total (C-1).
- **Keys enter at leaves.** A key is entered only at a kind with no subkinds (`LOAD-LEAF`); a kind with subkinds is a class, so a thing is never silently in none of them (M-3). Units declared with `unit X: …` (currencies) live always; keys of a unit kind entered with `+` state their lifespan.
- **Balances.** `bal(x, d)` is the balance at the end of day d, after every movement of d; `bal(x, d start)` is at its start; inside a movement, `bal(x)` with no day is the running balance at the movement's place in the day's order (D-5). A place's balance is zero at its lifespan's start; a place alive before `start` needs an anchor reading (D-3) before any read of its balance, else the read is a hole naming the place (`HOLE-ANCHOR`) (M-4).
- **One base kind, any number of plain kinds.** A thing has one base kind for its life (a place, unit or event kind, or a plain kind such as `person`). It may also be a member of any number of plain kinds (kinds with no place, unit or event part), each membership with its own lifespan: `wrapped + [broker] lives 2021-03-01..` makes the account wrapped from that day. Two independent classifications of one account are two memberships, and a class that changes on a date is a membership that ends as another begins (C-32).
- **Bindings over a kind** state when: `x in current on d` ranges over the things alive on day d, subkinds included; `x in current over s` over those alive on some day of span s (for an event kind, the movements dated in s; `over ..` is all of them). A total over places on a day therefore counts every place alive that day and no other. The key binding of a def, input or place keyed by a kind needs no `on` or `over`: the item is defined for each key; aggregates, listings and a `move`'s keys state it (`move assess[y in years, p in person on end_of(y)]`), since each move key books an event on a day (SF-8, M-2). `x in K on d` is also a test, true when x is of kind K and alive on d (SF-18).
- **`x in K`** tests kind membership, subkinds included (the membership test `in` of 1.0.1, applied to a kind; no new word); for a plain kind whose memberships are dated, the test is written with its day, `x in K on d`. A thing's base kind is fixed for its life.
- **Place kinds** state their unit (`DUK`; `any` for positions in several units; or `one`, when each thing states its single unit as it enters, `account + [konto] DUK lives 2019-05-01..`, SF-1), their sign rule (`signed`, `min 0` or `max 0`), how they are valued (`priced p`, for a place whose single position is valued by a price input, as 1.0.1's asset), and `outside` for the world beyond the model. A binding over places includes outside places like any other; a total that should leave them out says `where not x in outside` (C-9). 1.0.1's seven place kinds become a prelude written in Tally (`account`, `cash`, `holding`, `asset`, `debt`, `claim`, `outside`), each of unit `one`, so migrated models keep their words (`place konto: account PLN` becomes `account + [konto] PLN lives …`).
- **Positions and value.** `bal(x, d)` is a one-unit place's balance; `bal(x, u, d)` the quantity of unit u in a place of `any`. `value(x, d) in CUR by p` sums each position times its price p (an input of type `num CUR/u`, keyed by unit) on day d. A position with no price on d is a hole that names it, never left in its own unit. A position already in CUR counts at its amount, so `by p` is required only when some position is in another unit, and is then refused when missing (`LOAD-PRICE`, SF-16). `unit(q)` gives the unit of a quantity, a leg's amount included, or of a one-unit place (SF-7). `worth` merges into `value`; `kind(x)` and `currency(x)` become `x in K` and `unit(x)`. `by p` naming a reading input reads it with its stated carry, `by last(px) within 5 days` (D-3).
- **A text attribute**, type `text`, holds a name or a note as data; nothing computes with it.
- **Ownership** is kept from 1.0.1 (`owns`, L §3.6): a dated input of each owner's share of a thing. A person's net worth is a sum over places weighted by `owns`, and a share that moves between owners is a dated row; a split by origin, not by share (`O:A-6`), is a regime's lots (D-2) (C-16).
- **Exact values.** `value(…)` multiplies exactly; nothing in it rounds, and a rounding is written with `round` and its mode (C-8).
- **Equivalences that break:** "an index key exists on every day of the run" (L §5.3) and "a thing that doesn't exist is a row of 0" (L §6.2). A zero row is now a stated zero.

**Explicit form.** A key of a kind without `lives` is refused (`LOAD-LIFESPAN`). A binding over a kind without `on` or `over` is refused (`LOAD-ALIVE`). A place kind without its unit or sign rule is refused (`LOAD-PLACEKIND`); a thing of a `one` kind without its unit is refused (`LOAD-PLACEKIND`); a value of positions in other units with no `by` is refused (`LOAD-PRICE`); a place ended with positions in it, or a leg booked to a thing outside its lifespan, is refused (`RUN-LIFESPAN`). A valued position with no price row on the day is a hole naming the position and the price (`HOLE-PRICE`). A price that is a reading (D-3) read on another day needs its stated carry (`LOAD-CARRY`).

**Alternatives.**
- Keys and rows of 0 (1.0.1; M-08, M-22): rejected, since the guard repeated in every reader is G-1's failure and a forgotten guard is wrong in silence.
- Lifespans derived from the first and last row: rejected; it conflates a stated 0 with absence again, and classification by data lets a thing drift between rules (`prior-art/kinds-modules.md`, AOP's fragility).
- Nulls stored for absent attributes: rejected; SQL's aggregates skip them in silence (`prior-art/rea-entities.md`), which is PR-1's failure.
- A kind as a dated attribute of a thing: rejected for now. A rule that names a kind would need a day for every membership test; a dated class is written as an attribute instead (Open).
- Built-in place kinds kept in the core: rejected; `debt` and `claim` are one place kind with a sign rule, and keeping seven built-in kinds would keep product words in the core. They live in the prelude.

**Axes closed.** LIFE (`LC-12`, `G:C6-F9`), OWN (`P:OWN-01`), CUR (`O:A-4`, `O:E1-4`), VAL (`LC-17`), REC (`O:A-7`), SURF (`O:E1-6`), PTY (`LC-13`), with `LC-29` across OWN, PTY, VAL and LIFE; `Q:E1-01` and `O:E1-7` are closed with D-9.

**Effect on the 79.** Meaning: index additions with a lifespan reach chg1, chg4, chg5 and chg6; places and `bal`, `worth`, `kind()` and `currency()` reach CMP-01, the household and chg5 (coverage table 1). Surface only for the rest: no other model declares a place, and an `index` of names stays an index. The household's zero rows before a start (L §6.5 change 1) become lifespans; this is a manual migration.

**Shown.**
LIFE: a count that can't see a thing before it exists.
```
kind job
version jobs_2026
  cite contract
  known 2026-01-10
  job + [ana_berlin] lives 2026-12-01..
def jobs_held[m in month]: int at start_of(m)
  = count(j in job on start_of(m))
```
`jobs_held[2026-01]` is 0 without a guard, since `ana_berlin` isn't alive on 1 January.

CUR and VAL: net worth over places in several units, with a hole for the one unpriced position.
```
def net_worth[d in [2026-06-30]]: money DUK at d
  = sum(x in place on d where not x in outside: value(x, d) in DUK by px)
```

**Checked against.** Hard set: `Q:E1-01`, `P:OWN-02`, `S3` (the notes as units, the brokerage of the ordinary kind), `Q:A-05` (the wrapped kind). Invented cards: I-2 (a kind of account), I-3 (the flat as a thing with a lifespan).

**Open.** A class that changes on a date stays an attribute; if the visible cases show many rules that need "the kind on day d", a dated kind is a later record. Whether `index` and `kind` merge (an index as a kind of things that live forever) is left to phase 4's counts.

*Changed at G2: see GD-15.*

---

## D-2. A movement is one construct of an event kind, and a regime is one unit that applies by kind

**Problem.** PR-2: a movement carries no event kind or cause and an obligation has no balance until it is paid, so rules about a kind of event must be wired to each event, account and instrument by hand, and what is owed or carried between events is invisible. Items (8): `Q:A-08`, `Q:B-04`, `Q:D-09`, `O:FLOW-02-1`, `O:A-6`, `LC-16`, `LC-21`, `LC-29`. Axes: ASK, FLOW, LAW, LIFE, NUM, OWN, PTY, SCEN, VAL. With it, bar B7: a regime is one unit, written once, that applies by itself.

**Today.** A pay is one transfer between two places, and "one event is several pays that share keys" (L §5.13; D M-04), so nothing groups an event's legs (`O:FLOW-02-1`) and nothing selects movements by cause (`Q:A-08`; `flows(p, span)` reads one pay by name). A set of rules about a kind of event is written as defs and pays that name each place and pay they read, so each new instrument or account is wired in by hand. An obligation exists only once a pay moves money into a `debt` or `claim` place; `holds h` marks part of a balance as someone else's but reaches only `free` and `plan_low` (L §5.9; G-5, `LC-16`, `LC-21`). Lots are derived by `fill` (D M-15) but an acquisition date has no fact of its own (`Q:B-04`), a lot passing by inheritance keeps its acquisition only by hand (`Q:D-09`), and `owns` splits a place by share, not by origin (`O:A-6`). `DECISIONS.md` chose M-02 and decision 14 (no memo place), M-04 (no multi-leg movement), M-13, M-15, L-01 and L-08.

**Solved form.** (1) **Event-typed posting with an agreement of dated rules**: Fowler's Accounting Event, Agreement Dispatcher, Posting Rule and Accounting Entry (`prior-art/posting-rules.md` form 1; shelf `bitemporal-event-sourcing.md` §2.6–2.7). (2) **Coherent type classes as regimes**: an instance over declared kinds reaches every thing of the kind with nothing written on the thing, and two instances for one question are refused (`prior-art/kinds-modules.md` forms 1–2). (3) **Obligations as claims with balances**: REA's claims and Valueflows' `settles` (`prior-art/rea-entities.md` form 1). (4) **Lots as a regime's state, the matching method as an ordering**: Beancount's inventories and booking methods (`prior-art/posting-rules.md` form 2), which is 1.0.1's `fill` with its order stated (D M-15, kept).

**Reduction.**
- Fowler's accounting event is a **movement**: a thing of an **event kind** (D-1), with a day, named **legs** (each a quantity of one unit from a place to a place), and attributes. Planned movements (`move`) and observed ones (rows of an observation input, D-8) are the same thing with different certainty.
- Fowler's agreement is a **regime**: a named unit that holds its rules (defs), its state (its own places and defs) and its postings (moves keyed by the movements it reaches). It reaches movements by binding over an event kind, and places by binding over a place kind. Nothing outside names it.
- Fowler's posting rule is a `move` keyed by another movement: `move assess[s in sale where …]`. The rule in force on the event's day applies (D-4).
- A type class's coherence is a load check: two rules that post the same movement or define the same value for one event are refused, within a regime and across regimes.
- REA's claim is a place of a place kind with a sign rule, owned by the regime: a posting moves the amount owed into it the day it arises and the settlement moves it out, so it has a balance between the two. `holds` goes.
- Beancount's lot is a supply of a `fill` whose supplies are acquisition movements (their legs give units, cost and day), and the booking method is the fill's `order`, stated as one of the regime's rules.

**Change** (core: event kinds, legs, bindings over movements; sugar: the regime block).
```
kind sale: event certain
  leg units: note                 # every sale states these legs
  leg cash: DUK
  leg fee: DUK

move sell[t in [2026-06-10]]: sale
  on t
  leg units from broker to market amount sold[t] of n7
  leg cash  from market to broker amount gross[t] on add_business(t, 2, len)
  leg fee   from broker to market amount fee[t] on add_business(t, 2, len)

regime gains
  place owed[p in person]: debt DUK lives ..
  def gain[s in sale where s.units.from in ordinary]: money DUK at s.day
    = s.cash - s.fee - cost[s]
  move assess[y in years, p in person on end_of(y)]: assessment
    on end_of(y)
    leg tax from owed[p] to fisc amount tax[p, y]
```
- **`kind K: event C`** declares an event kind, its certainty C (`certain` or `uncertain`, D-6) and its legs, each named with its unit (or a kind of units), and any attributes, `attr name: type`, which each movement states (SF-20). An event kind may have no legs: an invoice issued is an event with attributes that moves nothing until it is paid (M-5). A subkind keeps its parent's legs and attributes and may add its own; a binding over the parent reads only the parent's (M-6). Event kinds have subkinds like any kind; a subkind may state its own certainty (`kind maybe_spend: own_housing uncertain`, SF-12).
- **`move id[binds]: K`** writes planned movements of kind K, one per binding tuple. A line `on d` states the movement's day (`s.day`; a date or a window), and each leg may state its own `on`, else it is on the movement's day, as a row is under its version's `governs` (C-13). `over span` and `on d` select movements by their day; a windowed day partly inside a span needs a `windows` clause (D-7). A line `at e` gives the movement's law date (D-4), required when anything it reads is law-dated (SF-4); `attr name = e` states an attribute. Each leg the kind names is stated: `leg name from A to B amount q [of u] on d`. Legs may have their own days (cash lands later than units leave). Each leg conserves its unit by construction; a movement missing a leg its kind names, or a leg of the wrong unit, is refused. A line `when c` makes the instance exist only where c holds, as 1.0.1's `pay` did, so a conditional movement isn't a zero one (M-7).
- **A movement is a thing** whose lifespan is its day, with attributes: `s.day`, `s.units` (a leg: `.amount`, `.from`, `.to`, `.on`), declared attributes of its kind, and `s.fulfils` (D-8).
- **Bindings over movements**: `s in sale` ranges over every movement of kind sale (subkinds included), planned and observed, with `where` and `over span`. The rule never names a movement id, a place or an instrument.
- **`regime R`** opens a block. Every item inside is R's own, named `R.item` from outside. The block may hold kinds (its own event kinds and places, the tax office among them), inputs, bands, defs, places, fills and moves; it reaches the rest of the model only through bindings over kinds and through attributes of things. An item outside R's block that names R or one of its items is refused (B7's count, `tools/refcount.py`), except: an ask; a version row that states a row of R's input or a `given` value of R's def, since a row states the law's figures and wires nothing (SF-10); and any item, in another regime or outside every regime, that reads an item R lists in `export x, …`, as one act refers to another's result and a spending rule reads the tax owed (SF-17, C-7). `tools/refcount.py` prints three counts per regime: wiring references (names of R's unexported items from outside, and names of particular things or movements inside R's rules), which must be 0; fact rows naming R's inputs; reads of R's exports. The last two are reported, never hidden (C-7). Inside R, naming a particular thing or movement of the model (rather than a kind) is what the refcount reports.
- **One value, one definition**: an item's name defines one value per key, and the rivals within a def are D-12's `beats`. Several regimes may post for one event (VAT and income tax on one invoice); each posts its own movements, their order on a day is D-5's, and `readers-of` on an event kind lists every regime that reaches it. There is no load check that two regimes' `where` filters overlap: that is undecidable in general, and two acts taxing one event is lawful. What the type-class solved form gives is the namespace and the reach by kind, not instance coherence (C-6).
- **Obligations** are places of a regime with a sign rule (D-1); `holds` and `free` are removed (`free` is used by no model; `holds` by the household only).
- **Lots** stay `fill` (D M-15) over movement bindings, whose sides take any binding (`supply b.units.amount for b in purchase over ..`); the order (first in, first out by acquisition day, or another stated one) is a rule of the regime. A transfer between one holder's places and an inheritance are movements of their own kinds, and whether a lot keeps its acquisition day through them is the regime's rule about those kinds.
- **Kind of change**: event kinds, legs, movement bindings and coherence are core. The `regime` block is sugar for a prefix on every item's name plus the refcount and coherence checks.
- **Equivalences that break:** "one event is several pays that share keys" (D M-04): an event is one movement. `pay` and records are replaced (migration: a pay is a one-leg `move` of a prelude kind `transfer`, automatic).

**Explicit form.** An event kind states its certainty (`LOAD-CERTAINTY`) and its legs; a movement states every leg its kind names (`LOAD-LEG`). A regime's state (losses carried, basis, what is owed) is one of its places or defs, never implicit. A regime reaches a movement only through a binding whose kind and `where` are written; a rule that would apply to a list of accounts is written as a kind. A read of another regime's unexported item is refused (`LOAD-PRIVATE`); a movement missing an attribute its kind declares is refused (`LOAD-LEG`).

**Alternatives.**
- Several pays sharing keys (1.0.1, M-04): rejected; nothing groups the legs or selects by kind (`O:FLOW-02-1`, `Q:A-08`).
- A regime as a set of hooks on movements, each movement naming its regimes: rejected; that is the wiring PR-2 names, and B7 counts it.
- Classifying events by pattern (a rule applies to any movement whose legs match): rejected; classification by data drifts (`prior-art/kinds-modules.md`).
- Lots as a built-in inventory with booking methods (Beancount): rejected as core; `fill` already does greedy allocation with a stated order, and Beancount's global default method breaks the explicit rule (`prior-art/posting-rules.md` form 2).
- A hold as a label on a balance (1.0.1): rejected; a hold is a place that the label pretends isn't there (`LC-21`).

**Axes closed.** FLOW (`O:FLOW-02-1`, `LC-16`, `LC-21`, `Q:A-08`), LAW and NUM (`Q:B-04`, `Q:D-09`), OWN (`O:A-6`), PTY and LIFE (`Q:B-04`), ASK (`Q:D-09`), SCEN (`Q:A-08`), with `LC-29`.

**Effect on the 79.** Meaning: `pay` reaches CMP-01, the household and chg5 (automatic to a one-leg `move` of kind `transfer`, except where legs belong together, which is manual for the household); `holds` reaches the household (manual: VAT owed becomes a place). `fill` (9 models) is unchanged. No other model moves money.

**Shown.**
FLOW: a refund that the gains regime and a fee rule both reach, with no reference from the refund.
```
kind refund: event certain
  leg cash: DUK
regime fees
  move charge[r in refund where r.cash.to in current]: fee
    leg cash from r.cash.to to bank amount fee_for(r.cash.amount) on r.day
```
NUM and LAW: the first-in, first-out matching of the gains regime as its own rule.
```
regime gains
  fill matched[x in ordinary over run, u in note over run]
    supply b.units.amount for b in purchase where b.units.to == x and b.units.unit == u
    demand s.units.amount for s in sale where s.units.from == x and s.units.unit == u
    arc b.day <= s.day
    order b.day ties by b.seq
```

**Checked against.** Hard set: `Q:B-04`, `S3`, `Q:A-05`, `Q:B-01` (VAT as a regime), `S2` (the credit line's rules as a regime over a place kind). Invented cards: I-1 (levy), I-3 (letting regime), I-4 (three regimes on one event).

**Open.** The order of postings inside one event when two regimes both act on it is D-5's rule; whether regimes need a declared order between them beyond D-5 is left to the visible cases. The day a claim arises is each regime's own rule. The `regime` block may later carry parameters (one regime text applied to two jurisdictions); not needed by any item.

*Changed at G2: see GD-1.*

---
## D-3. Every fact states when it holds and when it became known; readings are instants

**Problem.** PR-3: when a fact holds and when it became known are partly left to defaults and to version boundaries, so a missing clause silently means every date, a reading taken on one day is read as a lasting state, and facts known on different days of one source can't be told apart. Items (11): `P:OWN-04`, `P:VAL-01`, `P:RULE-08`, `O:PTY-04-1`, `O:D-2`, `S:8`, `LC-6`, `LC-10`, `LC-25`, `G:C2-F9`, `G:C2-F5`. Axes: LAW, OWN, PTY, REC, RULE, TIME, VAL.

**Today.** `governs` is optional; a row or version without it governs every date, loses ties to a row with a span, is never `held` and weighs 0 in the carry (L §4.6 steps 1–2, §5.3, §8.2; E §8; H-3). Rows are open-ended ("an amount stays in force until a later act changes it", L §4.6), a rule written for law that also turns a valuation or a price read on one day into a lasting state (G-6, `P:OWN-04`). `opening` and `stated` take spans and a span longer than a day anchors at its last day (L §5.18; H-21, `LC-10`); `covers` borrows the version's span (`LC-25`). A rule can't read a row's span (`P:VAL-01`); `covered(x, span)` takes one period (`S:8`). Knowledge is per version, so a figure restated later in one source ties (`O:PTY-04-1`), data that arrives month by month needs one version per arrival (`P:RULE-08`), and the `known` pin can't take the earlier of a date and the cut (`O:D-2`). `DECISIONS.md` chose M-16 (three times: governs, known, record time), M-06, Q-13, Q-15 and decision 9 (`held`), decisions 4, 32 and 40.

**Solved form.** **Bitemporal facts**: Snodgrass's valid and transaction time, SQL:2011 periods, XTDB's as-of slices (shelf `bitemporal-event-sourcing.md` §2.1–2.2, §3); **instants against periods** as two kinds of temporal data (Snodgrass; shelf §2.1); **watermarks and a stated carry-forward** for reads between observations (stream processing's event time against processing time; shelf §2.8).

**Reduction.** A row's `governs` is its valid time; its `known` is its transaction time in the bitemporal sense (when the information became available to the model's sources); a run "as known on K" is XTDB's slice, taken before evaluation, so the run sees one plain valid-time model. A state input holds over periods; a **reading** input holds at instants. A read of a reading on a day without one is a sequenced read with a stated carry, and the carry's age is data. M-16's third time, record time in the store, is the store's transaction log and belongs to `OPERATIONS.md`.

**Change** (core).
```
version letter_2026
  cite bank_letter
  known 2026-01-15
  governs 2026-01-01..
  fee_rate = 0.4 %
  cap = 12.00 DUK  governs always "§4, a constant of the contract"
  rate[2026-02] = 5.1 %  known 2026-02-03 "the February notice, same file"

input px[u in note]: num DUK/u reading
version quotes
  cite exchange
  known 2026-06-10
  px[n7] = 102.50 DUK/n7 at 2026-06-10
```
- **`governs` is required** on every version that holds a row or rule without its own time; rows that carry their own time (a lifespan's `lives`, a reading's `at`) take none, so a version of only such rows has no `governs` line (M-8). It applies to the version's rows and rules unless a row or a rule states its own: `rule base governs 2028.. = …`, as LC-6's option (a) asks; two items of one label with overlapping spans are refused, `LOAD-OVERLAP`) (C-15). A row for a key that an earlier-known row already governs on an overlapping span says `replace` (D-12), so which row a read takes is written, never inferred from order (C-22). A fact that holds for all time is written `governs always`; it governs every day, is never `held`, and loses a tie to a row with a narrower span (the read rule states this). Open ends are written: `2026-01-01..`.
- **A row may state its own `known`**, later than its version's, so one source restated over time is one version with dated rows. `known` takes a date or a window ("known by 1 November"); an `as known` read on a day inside the window states a `windows` clause or is `HOLE-IMPRECISE` (M-9). There is no `known always`.
- **The read day.** An input read inside a def is read at the def's law date; a reading read so is read at that date and needs its carry unless a reading falls on it (M-10).
- **Readings**: an input declared `reading` has rows `at D`, each an observation at one instant. A reading of a balance (`stated`, `opening`) states its place in the day: `at D start` (before the day's movements), `at D end` (after them) or `after m` (just after movement m, for a running balance printed row by row); `at D` alone is refused for it (`LOAD-INSTANT`, SF-2). A reading of a place of unit `any` is one row per unit (SF-30). `x[k]` read on a day with no reading is refused at load unless the read states its carry: `last(x[k], d)` (the last reading on or before d), with `within n days` to bound its age, or `age(x[k], d)` to read the age. `opening` and `stated` are readings.
- **A state input past its rows**: read on a day no row governs, it is a hole naming the input and the day (`HOLE-UNGOVERNED`: a row could give it, D-7), before the first row as after the last (C-22); `last(x, d)` on a state input reads the latest row whose span began on or before d, marked `held` in `meta` and the register, so the carry is written where it is read (SF-13).
- **Coverage** (`covers x`, L §5.18) takes the version's `governs` as the covered span, or its own when the two differ: `covers konto 2026-09` (LC-25) (C-15).
- **Keys and law dates are apart**: an input keyed by a period (`cpi[m]`) has the key as data, which period's value it is; the reader's law date picks, among the rows for that key, the one that governs (C-28).
- **Spans are readable**: `meta(x[k], d).governs` gives the governing row's span (D-15), `covered(x, span)` takes a span.
- **The cut is readable** as `cut`, so a pin can be `known min(d, cut)`.
- **Equivalences that break:** "a row without `governs` governs every date" (now refused); "an open row read later is a lasting value" holds only for state inputs, not readings.

**Explicit form.** A version or row with no `governs` is refused (`LOAD-GOVERNS`). A reading read away from its instant without a carry is refused (`LOAD-CARRY`). A carry with `within n days` whose last reading is older is a hole naming the reading (`HOLE-STALE`). A balance reading without its place in the day is refused (`LOAD-INSTANT`); a state input read where no row governs, with no `last`, is a hole (`HOLE-UNGOVERNED`).

**Alternatives.** `governs` optional with "every date" as the default (1.0.1): rejected, it is H-3's silent default. Reading against state inferred from the type (a price is always a reading): rejected; a contract can fix a price for a period. Record time in the language: rejected; reproducing an old printout is the store's job (L §6.1), and `known` is the time the language reasons with. Last observation carried forward with no statement: rejected (G-6).

**Axes closed.** TIME (`LC-6`, `LC-10`, `G:C2-F9`; `G:C2-F5` answered by naming record time `OPERATIONS.md`'s), OWN (`P:OWN-04`), VAL (`P:VAL-01`, with D-15), REC (`S:8`, `LC-25`), RULE (`P:RULE-08`), PTY (`O:PTY-04-1`), LAW (`O:D-2`).

**Effect on the 79.** Surface: the 36 models whose versions have no `governs` get `governs always` (automatic; the tie rule and `held` are unchanged for them). Meaning: `opening`, `stated` and prices read on other days reach the household and CH-05 (warned: the writer checks whether the input is a reading).

**Shown.**
VAL: a value held at its last entry, with its age, and no second input.
```
def house_value[d in [2026-12-31]]: money DUK at d
  = last(valuation[house], d) within 400 days
```
RULE: monthly data in one version with dated rows.
```
version index_2026
  cite statistics_office
  known 2026-02-10
  governs 2026-01..
  cpi[2026-01] = 101.2
  cpi[2026-02] = 101.5  known 2026-03-10
```

**Checked against.** Hard set: `P:VAL-01`, `S1` (the recorded balance as a reading), `S3` (the price as a reading on the trade day). Invented: I-5 (answers as known on three dates).

**Open.** Whether `always` should also be refused for inputs keyed by time (a timeless row on a monthly input is likely a mistake): left to phase 6's writers.

*Changed at G2: see GD-8.*

---

## D-4. Which day's rules apply is written, never inherited by default

**Problem.** PR-4: which day's rules apply to a computation is decided by defaults (the reader, a key's first day, the cut) rather than written, so one expression gives different values to different readers and a period that crosses a change silently uses the older rule. Items (9): `O:TIME-03-1`, `O:SCEN-02-1`, `O:A-5`, `LC-9`, `LC-14`, `G:C2-F7`, `G:C3-F5`, `G:C4-F7`, `G:MC2-9`. Axes: LAW, SCEN, TIME.

**Today.** A def's law date is its `at`, else its first time key's start, else its reader's law date, else the ask's cut K (L §4.3 rules 1–5). Rule 2 sets the law date by a time key in silence (`LC-9`, `O:TIME-03-1`); rule 3 is a dynamic scope (H-1: 1 092 of 1 454 defs have no time key and no `at`); rule 4 makes each advance of `on` rewrite a dateless ask's law date (`G:C3-F5`, `G:C4-F7`), and leaves open the law date of an ask line whose top value isn't a def (`G:C2-F7`). A period stands for its first day (L §4.6), so a month-keyed value reads the law of the 1st and a mid-month change needs `at` (`O:SCEN-02-1`; H-2). `next_business(d, h)` reads h at the def's law date, not at d (`O:A-5`). No expression reads whether a provision is in force (`LC-14`, G-3). Scenario rows read by a dateless ask before `from` are holes (`G:MC2-9`). `DECISIONS.md` chose decision 1 (propagation, over the model's `on`, "wrong in silence", and refusal, "loud but forces a snapshot model to be restructured"), decision 36, decisions 34–35 and L-24.

**Solved form.** **The agreement dispatcher** (Fowler: the rule for an event is chosen by the event's date; shelf `bitemporal-event-sourcing.md` §3; `prior-art/posting-rules.md` form 1); **the read instant stated at each read** (OpenFisca: a formula for a period reads parameters at an instant it states; shelf `openfisca.md` §2.3–2.5); **a date as an explicit scope input with a gap and overlap check** (Catala; shelf `catala-default-logic.md` §2.8).

**Reduction.** A def's law date is OpenFisca's instant: written on the def as an expression, or written as the reader's (`at reader`). `at reader` is a written dynamic scope, and its solved form is the implicit parameter with a static type (Lewis, Launchbury, Meijer and Shields, POPL 2000): the def declares in its header that it takes the law date from its reader, and the loader checks that every chain of readers binds it at an ask's `law` or a movement's `at`. One expression still gives different values to different readers (H-1's observation holds), but the dependence is now in the def's header and can't be missed; nothing is passed implicitly where the text doesn't say so (C-25). The chain of `at reader` ends at something whose law date is a written date: a movement (its `at`, or its day, stated) or an ask (its `law` pin). A period is not an instant: a def keyed by a period states which instant of it, or how values under several laws inside it combine.

**Change** (core).
```
def allowance[y in year]: money DUK at end_of(y) = band_amount(base[y])
def rate_now: rate at reader = statutory_rate
def levy[m in month]: money DUK at each day of m split by days = base[m] * rate_now
ask t: allowance[2026] law 2026-12-31
ask in_effect: in_force(gains.exempt_long, 2031-03-15) law 2031-03-15
```
- **Rules 2, 3 and 4 of L §4.3 are removed.** A def that reads anything law-dated (a versioned input, band or def whose rows change over time) states `at e` (any date expression), or `at reader`. Rule 1 (`at`) and rule 5 (refusal) remain.
- **`at reader`** makes the def take its law date from whoever reads it. An ask that reaches an `at reader` def states `law D` (a date expression; `law cut` is allowed and is then written, not defaulted). A movement that reaches one states its own `at` as a line of its `move` (its day, its `for` period's start, or another date), as it must whenever it reads anything law-dated (SF-4).
- **A period as a law date** is refused (`at m` with m a month). Write `at start_of(m)`, `at end_of(m)`, or `at each day of m split by days` (the value is the day-weighted combination of the values under each law in force inside m) (a single stated day is `at d0`; C-26). The combination is exact; a rounding is written (C-8).
- **`in_force(r, d)`** reads whether a rule, a version or a row governs day d (G-3, `LC-14`), so a postponed commencement (`moves`) reaches every rule that tests it.
- **Calendar reads** read the calendar's facts at the day asked (`add_business(d, n, cal)` reads `cal` at d; D-13).
- **Equivalences that break:** "a dateless ask reads at K" (`on` advances no longer move a law date), and "a time key sets the law date".

**Explicit form.** A def that reads law-dated facts with no `at` is refused (`LOAD-LAWDATE`); an ask or movement that reaches `at reader` without a stated law date is refused (`LOAD-LAWDATE`); a period as a law date is refused (`LOAD-LAWPERIOD`). A period whose laws change inside it with `at each day of` and no `split` is refused.

**Alternatives.** Propagation as default (decision 1): rejected; it is H-1. The model's `on` as the law date: rejected in 1.0.1 already ("wrong in silence"). Refusing every def that reads dated rows without a date (decision 1's option b): rejected as too costly; `at reader` keeps snapshot models as they are, with one written word per def. A period's first day as its law date: rejected (H-2; decision 36 rejected it for imprecise dates).

**Axes closed.** LAW (`LC-9`, `LC-14`, `G:C2-F7`, `G:C3-F5`, `G:C4-F7`), TIME (`O:TIME-03-1`, `O:A-5`), SCEN (`O:SCEN-02-1`, `G:MC2-9`, with D-10).

**Effect on the 79.** Surface, automatic: every def that relied on rule 3 gets `at reader`, every time-keyed def that relied on rule 2 gets `at start_of(k)`; both keep their answers by construction. Warned: every ask that relied on rule 4 gets `law` with the model's `on` date; its answers are the same today and stop moving when `on` advances, which is the fix (the writer checks that this was meant). `def at` (13 models) and `pin law` (10) are unchanged.

**Shown.**
LAW: a monthly levy across a mid-month rate change.
```
def levy[m in month]: money DUK at each day of m split by days = base[m] * rate_now
```
TIME: a business-day shift that reads the calendar of the day it shifts.
```
def due[m in months]: date at start_of(m) = schedule(m, day due_dom, following, len)
```

**Checked against.** Hard set: `O:SCEN-02-1`, `S2` (the due day shifted by the calendar), `S3` (rules at the sale's day). Invented: I-5 (a rate changed on 1 July).

**Open.** Whether `split by days` covers the visible cases' mid-period changes, or some need a weighting the law states (by amounts earned per day): phase 6.

---

## D-5. Whether a movement reads running state is declared, and a day's order is stratified, with only cycles ordered by hand

**Problem.** PR-5: whether a transfer reads running state, and the order in which same-day transfers act, are inferred from what they read, from names and from declaration order, so an edit elsewhere or a rename silently changes who gets the money. Items (2): `P:TIME-07`, `LC-18`. Axes: FLOW, TIME. With H-5, H-7, H-16 and H-18.

**Today.** "A pay is a state pay if its amount, date or condition makes a state read …, directly or through defs. This is inferred, never declared" (L §5.13.1; `P:TIME-07`), and the class decides where it runs, its look-ahead, its back pay and whether it is re-measured (H-5). Flow movements book in pay id order; state pays by phase (default 5), pay id and key order (L §5.13.2; H-7, `LC-18`). Ties in fills, `rest largest` and `argmin` go by index order (L §5.10–5.12; H-16), against CHG-09's "an unstated order is a hole"; an open index's order isn't written (H-18). `DECISIONS.md` chose R-03 (strata inferred), T-12 (the day's order), R-09 and CHG-09.

**Solved form.** **Stratification over declared reads** and its order-independence theorem (Abiteboul, Hull and Vianu; Dedalus's time strata; shelf `datalog-incremental.md` §2.1–2.2, §3); **written order or proven commutativity** (CALM, Hellerstein and Alvaro; the cautionary cases of triggers fired by name and rules tied by definition order; `prior-art/posting-rules.md` form 4); **a declared sequence of same-instant events** (ACTUS; shelf `actus.md` §2.3).

**Reduction.** A movement's declared reads are its edges in the dependency graph. A movement with no running-state reads is in the lowest stratum (a **plan**, 1.0.1's flow pay); one with reads is a **decision** (1.0.1's state pay), in a stratum above every movement that changes what it reads that day. Within a stratum, plans only add legs, so they commute (CALM) and need no order. A same-day cycle (two decisions each reading what the other changes) has no stratification; there the writer states the order, and an unordered cycle is refused. Ties are an order, so they are stated too.

**Change** (core).
```
move draw[d in days]: shortfall
  reads konto
  leg cash from line to konto amount min(max(0 DUK, -bal(konto)), room(d)) on d
move settle[m in month]: settlement
  reads konto
  before draw
  leg cash from konto to line amount min(owed_on(m), max(0 DUK, bal(konto))) on due[m]
```
- **`reads x, …`** on a `move` names the places whose running balance its amount, day or condition read, directly or through defs. A movement that reaches an undeclared running read, or declares reads it never makes, is refused (`LOAD-READS`). The class is written, so an edit to a def can't move a movement between classes in silence.
- **The graph's edges** are the declared reads: `reads` names the places whose running balance a movement reads and the movements whose legs, day or attributes it reads, directly or through defs (`reads konto, settle`). An undeclared read of either sort is refused (`LOAD-READS`), so no edge is inferred (C-5). A movement that reads a decision's leg is above that decision, and is its dependent in a projection (D-6), though it reads no balance itself (SF-5).
- **Order within a day** is the stratification of the dependency graph: a decision that reads place x acts after every movement that changes x that day and doesn't depend on it. Observed movements and plans come first because they read nothing running. Any order consistent with the graph gives the same result.
- **Cycles**: when two decisions on one day each read a place the other changes, the writer states `before m` or `after m`; an unordered cycle is refused at load where the days are known then (`LOAD-ORDER`) and at run otherwise (`RUN-ORDER`). `phase` is removed; ids and declaration order never order anything. `before m` may also be written where no cycle needs it, to put a decision ahead of a movement it would otherwise follow; the decision then reads the balance before m (SF-15). An order that contradicts a read of m's legs is refused (`LOAD-ORDER`).
- **Ties**: `fill`'s `order`, `prorata`'s `rest largest`, `argmin`, `argmax`, `first` and `last` need `ties by e` when two candidates can tie; a tie met at run with no `ties by` is refused (`RUN-TIE`). The key order of an open index is a stated attribute, never the order rows arrived in.
- **Equivalences that break:** "state pays run by phase, then id" (L §5.13.2): order is the graph's; "a tie goes by index order".

**Explicit form.** `reads` (refused when missing: `LOAD-READS`); the order of a same-day cycle (`LOAD-ORDER`, `RUN-ORDER`); a tie-break (`RUN-TIE`).

**Alternatives.** Inferred classes (1.0.1, R-03): rejected (H-5). A total order of every same-day movement (ACTUS's sequence table, 1.0.1's phases): rejected; it asks the writer to order movements that commute, and a default phase is H-7. Order by name or text position: rejected (PostgreSQL's alphabetical triggers; CLIPS and Drools ties).

**Axes closed.** FLOW (`LC-18`), TIME (`P:TIME-07`).

**Effect on the 79.** Automatic: the household's and chg5's state pays get `reads` computed from 1.0.1's inference, and `before` is written wherever 1.0.1's phase and id order runs a reader of a place before a movement that changes it on the same day, which covers both the cycles and the reversed pairs; the answers are the same by construction, since every pair the graph would order otherwise gets the 1.0.1 order in writing (C-5). Warned: `argmin`/`argmax` (5 models), `prorata rest largest` (3) and `fill` (9) get `ties by` where a tie can occur; the writer checks the stated tie-break against the source.

**Shown.**
FLOW: a sweep that reads its account; the day's plans act first, with nothing written.
```
move sweep[m in month]: transfer
  reads konto
  leg cash from konto to savings amount spare[m] on sweep_day[m]
```
TIME: two same-day decisions in a cycle, ordered by hand (`S2`'s settlement and draw).

**Checked against.** Hard set: `P:TIME-07`, `S1`, `S2` (the draw after the day's movements, the settlement ordered against the draw). Invented: I-1 (input and output levy on one day).

**Open.** The cost of `reads` in the household (the number of decisions) is measured in phase 5.

*Changed at G2: see GD-11, GD-14.*

---

## D-6. The forecast is a query over planned movements; certainty belongs to the event kind; a retroactive change is a readable difference

**Problem.** PR-6: the forecast a decision may rely on is a special look-ahead that counts only some transfers and has no ordinary query form, and retroactive changes reach booked transfers through separate correction machinery, so planned commitments drop out of the forecast in silence and the size of a change is hidden. Items (14): `P:OWN-02`, `P:RULE-02`, `P:RULE-05`, `O:RULE-02-1`, `LC-15`, `LC-19`, `LC-20`, `LC-23`, `G:C1-F6`, `G:C6-F4`, `G:HH4-b-ambiguity-C`, `G:C4-F3`, `G:V3-S4`, `G:MC2-14`. Axes: FLOW, OWN, RULE.

**Today.** `plan_low(x, a, b)` is the only look-ahead: it exists only inside state pays and the defs they reach, counts only flow pays and leaves out state pays and promised pays (L §5.13.1; E §4.2). A transfer that follows a state decision (a purchase, then its instalments) is a state pay, so no look-ahead sees the instalments (G-4, `LC-15`), and `next_on` refuses to name it (`P:RULE-05`). `promised` is a flag that `bal` counts and `plan_low` and `free` don't, and after K a promised movement is never confirmed (`G:MC2-14`, `LC-20`). Back pay has its own machinery: a booking cut, `correction` versions, `arrears` lines and `HOLE-CHANGED` (L §5.13.3; E §5); the change δ is computed but can't be read (`LC-23`). Five open gaps sit here (`G:C1-F6` day K, `G:C6-F4`, `G:HH4-b-ambiguity-C`, `G:C4-F3`, `G:V3-S4`). `DECISIONS.md` chose R-04 (no foresight), R-05, R-06 ("a transfer planned in advance is written as a flow pay"), R-13, U-19 and decisions 7, 8, 10, 13, 18, 22, 24 and 33.

**Solved form.** **REA's commitments and fulfilment** (McCarthy; Geerts and McCarthy; Valueflows' `fulfills`; `prior-art/rea-entities.md` form 1); **certainty levels on one flow table, the projected ledger as booked plus open commitments** (SAP Cash Management's planning levels; hledger's periodic transactions and forecast; `prior-art/rea-entities.md` form 4; shelf `double-entry-ledgers.md` §2.9); **time stratification** for what a decision can know of the future (Dedalus; shelf `datalog-incremental.md` §2.2); **the difference adjustment** (Fowler; shelf `bitemporal-event-sourcing.md` §2.7, §3).

**Reduction.**
- A planned movement is a commitment; an observed one fulfils it (D-8); the projected ledger is observed movements plus the open remainders of planned ones.
- What a decision on day t may count in that ledger is fixed by time stratification: everything already decided (any movement whose day is on or before t, whatever its class), every plan (it reads no running state), and every later decision that doesn't depend on this one. A later movement that depends on this decision can't be counted without a cycle; it is left out, and listed.
- A certainty level is a property of the event kind (`certain`, `uncertain`): an uncertain movement counts in a balance once it is observed, and in a projection only when the read says so.
- 1.0.1's δ becomes Fowler's difference adjustment: an adjustment movement of a prelude kind, linked to the instance it adjusts, with δ readable.

**Change** (core: the projection; library: `low`, `next`).
```
def spare[m in month]: money DUK at sweep_day[m]
  = max(0 DUK, low(konto, sweep_day[m] .. add_days(latest(pay_day[m + 1]), -1 day)) from sweep_day[m] windows latest - buffer)

kind repayment: event uncertain
  leg cash: DUK
```
- **The projection is written**: `bal(x, d) from t` is the walk continued from day t, each later movement read as known on t (cut min(t, K), as 1.0.1's look-ahead), records and anchors after t unused, with every movement that depends on the item holding the read left out (D-5's graph: in a def, the movements that read the def directly or not; in an ask line, none). Its value depends on the def that holds it, never on who reads the def, so an ask that reads the def sees what the decision saw (SF-3). Movements left out that fall in the window are listed in the register. A movement that reaches a projection of x declares `reads x` (D-5) (M-11). A plain `bal(x, d)` with d after the day of a decision that reaches it is a cycle and is refused (`LOAD-CYCLE`); in an ask it is the plain balance on day d at the ask's cut, as in 1.0.1.
- **`low(x, a .. b) from t`** is library: `min(d in days(a .. b): bal(x, d) from t)`; without `from`, the plain balances. **`next(K, d)`**, the first movement of event kind K on or after d, is library over a binding (`first(m in K over d ..: m.day)`), `pending` when none is planned yet (D-7). `plan_low` and `next_on` are removed.
- **Imprecise and random movements in the window** need the read to say how: `windows latest`, `windows earliest` or `windows each` (one answer per possible day, D-7), and `random at mean` or `random at quantile(q)` (D-11). A projection that meets one with no such clause is a hole that names it.
- **Certainty**: an event kind states `certain` or `uncertain`. An uncertain movement counts in `bal` once an observation fulfils it, and in a projection only with `with uncertain`. A binding over an event kind counts observed movements and certain planned ones; `with uncertain`, on the binding or on an ask line for every binding and projection it meets, adds the unfulfilled uncertain ones (SF-12). `promised` and `free` are removed. Day K is past: observations dated K are booked (`G:C1-F6`).
- **Back pay** (LC-23, as the learner directed): `arrears` lines and `correction` versions are removed. When a later version changes a booked instance, the engine books nothing by itself: it creates an `adjustment` (prelude kind) with the attributes `adjusts` (the instance) and `change` (δ), readable by bindings and asks. The writer settles it with ordinary movements on any schedule, each stating `fulfils` the adjustment (D-8); a settlement may be dated on the instance's own day, which re-books the past in place (the old `correction`). An adjustment with an unsettled remainder at the cut is listed, and one that nothing settles is `HOLE-CHANGED` (C-17). A projection counts the adjustments known on its day (`G:C4-F3`). A decision of a section added later, dated before the section was known, is booked as an adjustment from the section's `known` (`G:V3-S4`).
- **Decisions are not re-measured** (decision 22 kept): a decision's day fixes what it knew.
- **Equivalences that break:** "a look-ahead counts only flow pays" (it counts every movement that doesn't depend on the reader, and lists the rest); "a promised movement is in `bal`".

**Explicit form.** The window choice and the random query in a projection (`HOLE-IMPRECISE`, `LOAD-RANDOM`). The certainty of every event kind (`LOAD-CERTAINTY`, D-2). What was left out of a projection is listed, never dropped in silence. A fact a projection reads before it is known stays a hole unless an `assumed` row says otherwise (unchanged: `G:C6-F4` is closed by the rule that every such default is marked `assumed`, which the loader checks).

**Alternatives.** Keep `plan_low` and widen what it counts (LC-19's `planned(x, d)` as a built-in): rejected; a second balance function with its own counting rule is the special mechanism PR-6 names, while the projection is the ordinary `bal` at a later day. Simulate every later decision, including those that depend on the reader (a fixed point): rejected; it can have no solution or several, and it is foresight (R-04). A certainty flag on each movement (`promised`): rejected; it is a default (no flag means certain) and LC-20 asks to drop it.

**Axes closed.** FLOW (`LC-15`, `LC-19`, `LC-20`, `LC-23`, `G:C1-F6`, `G:C6-F4`, `G:HH4-b-ambiguity-C` (the register lists what the projection left out, as decided here), `G:C4-F3`, `G:V3-S4`, `G:MC2-14`), OWN (`P:OWN-02`), RULE (`P:RULE-02`, `P:RULE-05`, `O:RULE-02-1`).

**Effect on the 79.** Meaning, manual: `plan_low`, `promised` and `bal(x, d)` reach only the household (coverage table 1); its invest rule's projection now counts its own instalments and later decisions that don't depend on it, so its answers may change, and each change is listed. `arrears` (LH-14, the household, chg3, chg4) is unchanged in surface; its corrections become readable.

**Shown.**
OWN: a decision that sees the instalments of an earlier decision (G-4).
```
def room[m in months]: money DUK at start_of(m)
  = low(konto, start_of(m) .. end_of(m)) from start_of(m) - keep
```
FLOW: the size of a retroactive change, read as a fact.
```
ask back_pay: sum(a in adjustment over 2026 where a.adjusts in salary: a.change) law 2026-12-31
```

**Checked against.** Hard set: `P:OWN-02`, `S1` (the forecast with a window-dated pay), `S2` (a settlement that depends on when the pay lands). Invented: I-5 (both branches of an unknown settlement day).

**Open.** The size of the dependency graph when many decisions read one place, and whether listing what was left out is enough or some models need to name it, are measured on the household and the visible cases.

*Changed at G2: see GD-13.*

---

## D-7. Absence has kinds, and an answer may be known up to named unknowns

**Problem.** PR-7: the language has one kind of unknown that swallows every result it touches and a not-applicable mark that can only be tested, so an answer known except for named parts, an event that hasn't happened or never will, and an event with no exact day all need workarounds or a second question. Items (22): `P:VAL-01`, `P:LAW-04`, `P:REC-05`, `P:ASK-02`, `Q:A-01`, `Q:A-02`, `Q:A-05`, `Q:A-11`, `Q:B-01`, `Q:B-03`, `Q:B-10`, `Q:B-11`, `Q:D-03`, `Q:D-04`, `Q:D-06`, `Q:D-08`, `Q:E1-02`, `O:ASK-03-1`, `O:A-3`, `LC-4`, `G:C5-F4`, `G:C7-F5`. Axes: thirteen, the widest.

**Today.** A hole propagates, and "a `sum` with a hole term is a hole" (L §5.10), so a partial answer is a second ask that leaves the unknown parts out by hand (`P:ASK-02` and eight more). `-` can only be tested, and a `where` comparing a key that can be `-` is refused, so every read carries `x != - and x = s` (`Q:A-01`, `Q:A-02`). An event that hasn't happened is an empty set and `first` over it is refused (`RUN-EMPTY`; `Q:B-01`, `Q:B-03`); one that never will is a far date (`Q:D-03`, `Q:D-04`). A state pay needs a precise date, so an undated plan takes an `assumed` day (`P:LAW-04`, `O:ASK-03-1`); an imprecise date that matters needs one ask per reading (`Q:A-05`); "every day known but one" has no form (`O:A-3`). `DECISIONS.md` chose the gap rule (N-702), Kleene logic, decision 29 and CHG-11.

**Solved form.** **Kinds of missing value as a tagged sum with a total match** (Codd's A- and I-marks; Imieliński and Lipski; Libkin's certain answers; option types; EDTF's unknown against open ends and one-of sets; `prior-art/holes-provenance.md` form 1; shelf: Catala's three outcomes, CUE's incomplete against bottom, Terraform's listed unknowns). **Residuals in named unknowns** (Hazel's indeterminate forms and commutativity theorem; partial evaluation; conditional tables; `prior-art/holes-provenance.md` form 2), linear only, with a cap.

**Reduction.** A value is one of: known; **unknown** (`?"…"`, a hole, a fact a row could give; it spreads, as today); **`none`** (not applicable: Codd's I-mark, the option type's empty case); **`pending`** (an event set that may still gain its first member: what `first` over it gives before then); **`never`** (a stated date that comes after every date). Only unknown spreads; the other three must be tested for before arithmetic. A partial answer is Hazel's residual: the known part plus a linear form in named holes, filled later by substitution with the same result as a rerun. A day that isn't exact is EDTF's interval or one-of set.

**Change** (core: the value kinds and windows; core, contained to asks: `partial`; no new matching form).
```
def paid_on[i in invoice]: date at i.day = first(p in payment over i.day .. where p.invoice == i: p.day)
def tax_point[i in invoice]: date at i.day
  = if paid_on[i] == pending then i.delivered else min(paid_on[i], i.delivered)
input deadline[c in claim]: date          # a row may be `never`
ask total: estate_value partial law 2026-06-30
move salary[m in month]: pay
  leg cash from employer to konto amount 6 400.00 DUK on window(last_business(m, 3, len), last_business(m, 1, len))
```
- **`none`** replaces `-` (the sign stays minus only, `LC-4`). Comparing `none` with anything but `none` is refused (`RUN-NONE`): the tests are `x == none` and `x != none`, so `where car[p] != diesel` over people with no car is refused until it says `car[p] != none and …`. The silent pick of option equality buys brevity at the cost of quality 1, which ranks higher (C-2). Arithmetic, ordering and aggregation on `none` are refused (`RUN-NONE`) unless tested for.
- **`pending`**: `first` and `last` over a movement or thing binding whose span reaches past the cut, and that is empty at the cut, give `pending`, which must be tested for (`RUN-PENDING` otherwise). Over a span wholly before the cut, or a closed index, empty is a refusal (`RUN-EMPTY`), so "no payment in 2025" asked in 2026 is not `pending` forever (C-30).
- **`never`** is a date value: later than every date, so ordering against it is allowed (`d < never` holds); arithmetic and aggregation on it are refused (C-22).
- **Matching is a test**: `x == none`, `x == pending` and `x == never` are the tests, so the option type's match is written with `if`; a value of one of these kinds that reaches arithmetic, ordering or an aggregate untested is refused where it meets them (`RUN-NONE`, `RUN-PENDING`), never turned into a number.
- **Sums over planned movements** count what is planned and observed at the cut, and the ask's register lists the planned instances counted that no observation has fulfilled yet; `pending` is only for `first` and `last` (M-12).
- **A day that is a hole**: a leg's or a movement's day may be a hole (as 1.0.1's pay date, G-MP-2), and so may a lifespan's end; a binding or `flows` over a span the day might fall in is then a hole naming it (M-13).
- **Partial answers**: an ask line marked `partial` prints, when its value is a hole, the known part and the named holes it is linear in, each with its coefficient (sums, and products with known numbers). A rounding, `min`, `max`, band or product of holes meeting a hole makes the line a plain hole naming every hole. Every named hole is printed (C-26). Defs are unchanged: only the ask line computes the residual.
- **Windows**: `window(a, b)` and `one_of([d1, d2])` are date values for a day not yet known; a movement may be dated by one. A read whose value depends on where in the window the day falls states `windows latest|earliest|each` (D-6) or is `HOLE-IMPRECISE` (as L §4.4). An ask line may state the clause for every window it meets through the whole walk; a read's own clause overrides it for that read (SF-6). Like the ask's pins and its `as known`, such a clause sets the world the line is evaluated in and is printed with it; every def has one value in that world, whichever def reads it, so this is not the reader-dependence D-6 removed (C-4). `windows each` prints one line per maximal sub-window (or combination of sub-windows) over which the answer is constant, and one plain line when the answer doesn't depend on the day (SF-23). An observation that fulfils the movement fixes its day.
- **A place nobody has told us**: a leg's `from` or `to` may be a hole of a place type. The leg books its known side, and every read of the unknown side is a hole naming it; it is never a made-up place (SF-11).
- **A calendar with an unknown day** is D-13's (`O:A-3`).
- **Equivalences that break:** "a `where` comparing a key that can be `-` is refused"; "`first` of an empty set is refused" (for open bindings it is `pending`).

**Explicit form.** Arithmetic, comparison or aggregation on `none` or `pending`, and arithmetic or aggregation on `never`, are refused unless the expression tests for them first (`RUN-NONE`, `RUN-PENDING`); a read that depends on a window's day states its choice (`HOLE-IMPRECISE`). A partial answer prints its named holes, never a number that silently leaves them out.

**Alternatives.** One unknown with a test (1.0.1): rejected, as above. A `match` form with a load-time totality check (option types): rejected by Iverson's test, since option equality and `if` say the same and the run refuses an untested value loudly; a static check needs flow typing the language doesn't otherwise have. Three-valued logic for every kind: rejected (Imieliński and Lipski: it can't give complete answers). Full symbolic evaluation with case splits everywhere: rejected for size; residuals are linear and only at the ask (cap). `none` that aggregates as 0: rejected (SQL's silent skip).

**Axes closed.** ASK (`P:ASK-02`, `O:ASK-03-1`, `Q:A-11`, `Q:D-08`, `Q:E1-02`, `Q:B-10`, `Q:B-11`), LAW (`P:LAW-04`, `Q:D-03`, `Q:D-04`), TIME (`O:A-3`, with D-13; `Q:A-05`), VAL (`P:VAL-01`, `Q:D-06`), REC (`P:REC-05`, `Q:A-02`), CUR (`Q:B-03`), SURF (`LC-4`), FLOW (`G:C5-F4`, `G:C7-F5`: an instance whose `when` is a hole has no amount read, and an instance of amount 0 exists and books nothing, as stated rules of the catalogue), LIFE, NUM, OWN, PTY, RULE (`Q:A-01`, `Q:B-01`).

**Effect on the 79.** Surface, automatic: `-` as a value (19 models) and as a test (16) become `none`; the guards `x != - and x == s` keep their meaning. Meaning: none of the 79 relies on `RUN-EMPTY` being reached; `partial` is new and reaches no model.

**Shown.**
REC: an estate's value known except for one named part.
```
ask estate: house_value + savings + share_of_firm partial law 2026-06-30
```
prints `412 300.00 DUK + 1 × ?"value of the firm share"`.
LAW: a deadline that never comes.
```
version contract
  cite agreement
  known 2026-01-01
  governs always
  deadline[c1] = never "§7: no term"
```

**Checked against.** Hard set: `Q:B-01` (the levy with a supply not yet paid), `Q:A-05`, `P:VAL-01`, `S1` and `S2` (window-dated pay). Invented: I-1 (tax point as the earlier of two events), I-5 (an unknown closure day).

**Open.** Products of holes (the residual's cap) and whether a partial answer should carry bounds when a hole is bracketed (a price known to lie in a range): later, if the visible cases need it.

---

## D-8. An observation is a movement, in any unit, and a statement's gap is a fact

**Problem.** PR-8: observed movements are a separate construct with fixed positional fields in money only, so movements of units, the realised amount of a planned transfer and the gap between a statement and the booked balance can't be read as ordinary facts. Items (14): `P:PTY-05`, `P:RULE-07`, `P:REC-01`, `Q:A-05`, `Q:B-08`, `Q:B-09`, `Q:D-07`, `O:FLOW-01-1`, `O:A-2`, `S:7`, `S:9`, `S:10`, `S:11`, `G:C5-F9`. Axes: ten.

**Today.** A record is a row of a movement input with built-in fields in a fixed order, `on`, `place`, `amount`, `explains` (L §5.18; H-15), in money of the place's currency, so a broker's fill or a fund register moving units can't be a record (`S:7`, `O:A-2`, `Q:A-05`). `flows(p, span)` reads what a flow pay booked, not what realised it (`S:11`), and no ask prints the model's own balances on covered days (`P:RULE-07`). A stated balance re-anchors the walk but the difference isn't a movement, so `flows` misses it (`O:FLOW-01-1`), and residuals print at anchors, not at the rows a question names (`S:10`, `Q:B-08`). `explains` on a hole movement is undefined (`S:9`, `Q:B-09`). Whether an ask may read the importer's rules is unsaid (`P:REC-01`). `DECISIONS.md` chose decision 5 (records as a construct), decisions 15, 21, 30 and 40, CHG-12, M-14 and M-18.

**Solved form.** **Observations as events in any resource** (REA; `prior-art/rea-entities.md` form 1); **balance assertions with a readable difference, and pad** (Beancount's `balance` and `pad`, hledger's assertions; `prior-art/rea-entities.md` form 3; shelf `double-entry-ledgers.md` §2.5, §2.10, §3).

**Reduction.** A record is an observed movement (D-2), the same construct as a planned one, with legs in any unit; `explains` is REA's `fulfils`. A statement's closing balance is a dated assertion (a reading, D-3); its difference from the booked balance is Beancount's `diff_amount`, a fact; a stated pad movement takes it up and is flagged.

**Change** (core).
```
input bank: observed
version statement_11
  cite bank_export_2025_11
  known 2025-12-01
  governs 2025-11
  bank[r17] = (on: 2025-11-03, kind: transfer, leg cash from konto to world amount 2 800.00 DUK, fulfils: rent[2025-11]) "line 17"
  bank[r18] = (on: 2025-11-04, kind: purchase, leg units from market to broker amount 12 of n7, leg cash from broker to market amount 1 230.00 DUK) "fill 18"
  stated[konto] = 4 150.00 DUK at 2025-11-20 end "closing balance"
```
- **`input X: observed`** declares rows that are observed movements: named fields (`on`, `kind`, its legs, `fulfils`, the kind's attributes), never positional. Legs may move units. Its keys are the rows' ids, an open key set stated by the rows themselves, as 1.0.1's records (M-14).
- **`fulfils m[k]`** links an observation to the planned instance it realises; `observed(m[k])` reads the realised amount and day, and the planned instance's open remainder is its amount less what fulfils it (`S:11`). On a hole movement, a fulfilling observation answers it (`S:9`, `Q:B-09`).
- **Coverage stays**: where statements cover a place, observed movements are booked instead of planned ones (L §5.18, kept); a planned instance in a covered span with nothing fulfilling it is listed as open, not booked.
- **Gaps**: `gap(x, d)` is the stated balance less the booked balance on d, a fact an ask reads and prints at any day a question names (`S:10`, `Q:B-08`). `pad x from y` in a version is a movement of the prelude kind `pad` that takes up each gap of x, flagged in the register, so `flows` and bindings see it (`O:FLOW-01-1`). Each version that states balance readings of a place says what its gaps are: `pad x from y` (taken up by a movement) or `anchor x` (the walk restarts from the reading); a version that says neither is refused (`LOAD-GAPROLE`), so leaving out the pad never silently keeps `flows` short (C-11). `gap(x, d)` is sugar for the stated balance less `bal(x, d) modelled` (C-26). (After G1, P-10 and L §5.19 settle it as the stated balance less the booked one, which is what a pad takes up; round 18, C2-22.)
- **Modelled against observed**: `bal(x, d) modelled` reads the model's own balance on a covered day (`P:RULE-07`).
- **The importer's rules** (`account_place`, `match_days`, …) are ordinary inputs, readable by asks (`P:REC-01`). The importer proposes `fulfils` links; the engine never infers one.
- **Equivalences that break:** "a record's fields come first in a fixed order"; "a record moves money only".

**Explicit form.** Each field of an observation is named (`LOAD-FIELD` for a positional row); a `fulfils` that names a missing instance is refused; an observation whose kind's certainty or legs don't match is refused (`LOAD-LEG`); a pad is written, never applied by default.

**Alternatives.** Records as plain input rows, as the cases had them (decision 5's option a): rejected in 1.0.1 because the walk needs their dates; kept here in another form, since an observed movement is both a row and a movement. A ledger outside the language (option b): rejected, as in 1.0.1. Automatic matching by amount and date in the engine: rejected (the engine never infers a link).

**Axes closed.** REC (`P:REC-01`, `O:A-2`, `S:7`, `S:9`, `S:10`, `G:C5-F9`: a converted movement on the modelled past prints its rate and both amounts, and a month with no statement prints no residual, as stated rules), FLOW (`O:FLOW-01-1`, `S:11`), RULE (`P:RULE-07`), ASK and TIME (`Q:B-08`, `Q:B-09`), OWN (`Q:D-07`), LAW, NUM and VAL (`Q:A-05`), PTY (`P:PTY-05`, with D-9).

**Effect on the 79.** Meaning, manual: records, `covered()` and `stated` reach the household only (coverage table 1); its records become observed movements with named fields and its `explains` becomes `fulfils`.

**Shown.**
REC: a fund register that moves units.
```
  fund[r3] = (on: 2026-02-02, kind: purchase, leg units from market to ike amount 4.2167 of etf, leg cash from ike_cash to market amount 500.00 DUK) "register line 3"
```
FLOW: the realised amount of a planned transfer.
```
def short[m in month]: money DUK at end_of(m) = rent_amt[m] - observed(rent[m]).cash.amount
```

**Checked against.** Hard set: `Q:B-08`, `Q:A-05` (a unit-moving register), `P:PTY-05` (with D-9). Invented: none directly.

**Open.** Kinds of gap (a duplicate, a reversal, a missing row) are listed by the importer, not the engine; whether the language needs a gap kind is left to the visible cases.

---

## D-9. A question across models is a joint model of pinned imports, with written identities

**Problem.** PR-9: a model is one closed namespace with one header, so facts and rules shared by several models are copied, one party known under several identifiers can't be joined, and a question across models has no form. Items (9): `P:PTY-05`, `Q:D-01`, `Q:E1-01`, `Q:E1-02`, `Q:E1-03`, `O:E1-1`, `O:E1-2`, `O:E1-3`, `O:E1-7`. Axes: nine.

**Today.** A reality has one header and one namespace (L §6.1, §2.5); a law part is "copied in, never linked" and an ask reads one reality, so relations between households are copies (`O:E1-1`, `Q:D-01`, `Q:E1-01`, `Q:E1-03`). Joining by copies forces stubs: `owns` rows of 0 for every place of the other model (`O:E1-2`) and an `opening[*]` default (`O:E1-3`). One person in several models is several keys with no stated identity (`O:E1-7`), and a transfer seen from both sides can't be matched (`P:PTY-05`). `DECISIONS.md` chose L §6.1's copy rule (a result is always reproducible from the reality alone), M-07 and M-21.

**Solved form.** **Modules with content-pinned imports and stated sharing** (ML structures and sharing constraints; Dhall's semantic hashes; Nix locks; Halpin et al. on `owl:sameAs`; `prior-art/kinds-modules.md` form 3; shelf `terraform-plan-apply.md` §3 point 10, `config-languages.md` T2); **group consolidation** with intragroup items eliminated when matched (IFRS 10; `prior-art/kinds-modules.md` form 4); (the orphan rule of `prior-art/kinds-modules.md` form 1 was cited in the draft and isn't used; C-25).

**Reduction.** A question across models is a new closed model (ML's functor application without parameters) that imports each member by hash and names their items with a prefix; each member stays reproducible alone, and the joint model's pins cover them all, so L §6.1's rule holds for the joint model. `same a.x = b.y` is a sharing constraint, written, local to the joint model. A group total over places of several members is consolidation: a movement between two members is internal, and it must be matched on both sides or it is listed.

**Change** (core).
```
model families
  tally 2
  on 2026-06-30
  use a = "kowalski.tally" sha256 "9f1c…"
  use b = "nowak.tally" sha256 "47ab…"
  same a.ola = b.ola_k
  same a.to_nowak = b.from_kowalski
ask both: sum(x in place on 2026-06-30 where not x in outside: value(x, 2026-06-30) in DUK by a.px) law 2026-06-30
```
- **`use name = "file" sha256 "…"`** in a model's header imports a closed model; its items are `name.item`. A changed file under a pinned hash is refused (`LOAD-PIN`). The joint model may add items of its own; it never changes an imported one.
- **`same p = q`** states that two things of one kind are one; facts about them from both sides are read together, and a fact on which they disagree for a day is a hole naming both rows (`HOLE-SAME`), never a silent pick; the joint model settles it with a version row of its own that states the value, the existing way to answer a hole (C-26). `same` also joins two rows or movements inside one model: a movement listed twice counts once and is listed (SF-25). A member's outside place stated `same` as another member's place contributes only its legs to that place; no other fact of the outside place is read, so no thing changes kind. Two places that aren't `outside` must be of one kind (`LOAD-SAME`) (C-14).
- **Consolidation**: a binding over places of several members counts each place once; a movement whose legs cross between members through places stated `same` is internal, and the joint model links it to its counterpart on the other side by a written `same` or `fulfils`. The engine never matches by equal day and amount (D-8's rule); an internal leg with no written link is listed and makes the group total a hole (C-14). A movement the joint model writes into a member's place is matched the same way against the member's own observations (SF-32).
- **Shared law**: two members that copy different versions of one law part (identified by the part's declared id and version, L §6.1, never by equal names) are refused unless the joint model states which one both follow.
- **Law parts** keep L §6.1's copy rule; `use` is for models.

**Explicit form.** Every import pinned (`LOAD-PIN`); every identity written (`same`), never inferred from equal names; disagreement and unmatched internal movements are holes, never resolved by order.

**Alternatives.** Copies (1.0.1): rejected, as above. A global merge by equal ids: rejected (`owl:sameAs` is too strong; equal names in two households are not one person). Live links without pins: rejected; the joint result would stop being reproducible (L §6.1).

**Axes closed.** PTY (`P:PTY-05`, `O:E1-1`, `O:E1-3`, `O:E1-7`), OWN (`O:E1-2`), and across ASK, FLOW, LIFE, NUM, REC, LAW and TIME: `Q:D-01`, `Q:E1-01`, `Q:E1-02`, `Q:E1-03`.

**Effect on the 79.** None: no model of the 79 reads another. Family E1 of the case set is the reach.

**Shown.**
PTY: one person under two ids.
```
same a.ola = b.ola_k
```
FLOW: a transfer between the two households, matched and eliminated from the group total.
```
same a.to_nowak = b.from_kowalski
```

**Checked against.** Hard set: `P:PTY-05`, `Q:E1-01`. Invented: none.

**Open.** A joint model of joint models (nesting) is allowed by the rule but untested.

---

## D-10. A hypothetical is an overlay of ordinary versions, and a comparison is a keyed difference

**Problem.** PR-10: a hypothetical is a separate layer whose versions follow other defaults than real ones and whose items a real question can't name, so writers mix the two kinds of version and a comparison between a hypothetical and the base needs workarounds. Items (11): `P:RULE-09`, `P:SCEN-01`, `Q:B-04`, `Q:B-05`, `Q:B-12`, `Q:D-11`, `O:SCEN-02-1`, `S:16`, `G:C7-F2`, `G:MC2-9`, `G:MC2-11`. Axes: ten.

**Today.** A file's `scenario` header changes what its versions mean: no `known`, `arrears` or `correction`, a default span of `D..`, rows never `held`, outranking stated keys and later knowledge (L §6.4, §4.6; H-17); writers mix the two (`Q:B-04`, `Q:B-05`, `Q:B-12`). A scenario row can't govern a month that started before `from` (`O:SCEN-02-1`); the unstated-key test cites a `known` a scenario row lacks (`G:C7-F2`). An item declared in a scenario file can't be named by a base ask (`P:SCEN-01`); an override linked to another is restated by hand (`Q:D-11`). Whether an ask in a scenario file runs in it is open (`G:MC2-11`, `S:16`). `DECISIONS.md` chose decision 3, L-24, S-05 and N-1301 to N-1303.

**Solved form.** **A speculative branch on the same code** (XTDB `with-tx`, Datomic `with`, Fowler's parallel model; shelf `bitemporal-event-sourcing.md` §2.7, §3); **overlays that amend what exists, with replace apart from add** (Pkl `amends`, KCL, CUE; shelf `config-languages.md` T3, T4, T10); **a plan as the keyed diff of two pure evaluations** (Terraform; shelf `terraform-plan-apply.md` §3 points 1–4).

**Reduction.** A scenario is a branch: the base's facts plus a set of ordinary versions, evaluated by the same rules. Its versions follow every rule a real version follows; the only difference is that an ask reads them only when it names the scenario. A replacement is D-12's `replace`; anything else adds. A comparison is the diff of two runs keyed by stable ids, with a closed list of outcomes (added, removed, changed).

**Change** (core).
```
scenario sell_flat over base
  tally 2
  from 2026-07-01
version plan
  cite own_assumption
  known 2026-06-01
  governs 2026-07-15..
  replace rent_income[flat] = 0 DUK
  sale_price[flat] = base.valuation[flat] * 95 %
ask delta: diff(net_worth[2027-12-31]) in sell_flat against base law 2027-12-31
```
- **A scenario's versions are ordinary versions**: `known`, `governs` (days from `from` on; `known` may be any day), `arrears` and `correction` mean what they mean in reality. `from` is the day the branch diverges: no scenario row governs a day before it. A scenario version's `known` decides whether the branch is "as if known then" (a `known` before the changed days: no back pay) or a change learned later (back pay as adjustments, D-6); both mean something, and the writer states which (SF-31). `replace x[k] = none` on an observed row removes that movement with its legs (SF-14).
- **A scenario row may be an expression over base facts** (`base.x`), so a linked override isn't restated (`Q:D-11`).
- **Asks name their world**: `in S` or `in base`. An ask in a scenario file that names neither is refused (`G:MC2-11`, `S:16`). A base ask can't name an item declared only in a scenario; `diff(e) in S against base` can, and prints each key's outcome (added, removed, changed, with both values) (`P:SCEN-01`).
- **Chosen paths** are D-11's pins on samples (`P:RULE-09`).
- **Equivalences that break:** the special defaults of scenario versions (L §6.4) are gone.

**Explicit form.** Every clause a real version requires is required in a scenario version (`LOAD-GOVERNS`, `LOAD-KNOWN`); an ask's world is stated; a replacement says `replace`.

**Alternatives.** Keep the layer with its own defaults (1.0.1): rejected (H-17, three writers confused). Scenarios as axes: rejected; an axis is a reading shown side by side, and a branch can add items and versions.

**Axes closed.** SCEN (`P:SCEN-01`, `O:SCEN-02-1`, `S:16`, `G:C7-F2`, `G:MC2-9`, `G:MC2-11`), RULE (`P:RULE-09`, with D-11), LAW, LIFE, NUM, OWN, PTY, VAL (`Q:B-04`, `Q:B-05`, `Q:D-11`), RAND and SURF (`Q:B-12`).

**Effect on the 79.** chg7 is the only scenario (coverage table 1): warned (its versions get `known` and `governs` from its old defaults; the writer checks them).

**Shown.**
SCEN: a branch that diverges mid-month.
```
scenario job_change over base
  tally 2
  from 2026-03-16
```
NUM and VAL: a comparison keyed by month.
```
ask months: diff(free_cash[m] for m in month) in job_change against base law 2026-12-31
```

**Checked against.** Hard set: `Q:B-05`, `O:SCEN-02-1`. Invented: none.

**Open.** Scenarios over scenarios (`over` another scenario) keep 1.0.1's rule (K11); untested.

---

## D-11. A random quantity is a fact whose value is a distribution, and a planning figure is a question about it

**Problem.** PR-11: random quantities live in a separate section with their own key, their own planning figure and a built-in verdict, so a forgotten path key silently drops them, a distribution can't be a fact's value, and a planning figure is a second number beside the distribution instead of a question about it. Items (10): `P:RULE-09`, `Q:D-10`, `S:18`, `LC-26`, `LC-27`, `LC-28`, `G:MC1-7`, `G:MC2-7`, `G:MC2-12`, `G:MC2-13`. Axes: LIFE, NUM, RAND, RULE, SURF.

**Today.** A draw is its own section, `draw r[p in path, …] ~ D to u`, with a `plan v` for look-aheads (L §5.14, decision 13). Every item over a draw is keyed `p in path`; forgetting the key is silently wrong (`LC-26`), and with no `paths` an item keyed by path has no instance (H-6). Rows are literals, so a price draw can't replace an input (`G:MC2-12`); the list of distributions is closed (`Q:D-10`); a path can't be replayed alone (`G:MC2-13`); an ask can't read N (`G:MC1-7`); chosen paths are rows keyed by path (`P:RULE-09`); the grammar and text disagree on `uniform` (`S:18`). The sayable verdict is built into `share`, so a note is a weight (H-8), with four readings of the carry open (`G:MC2-7`). `DECISIONS.md` chose Q-03, R-04, P-16, decision 13, CHG-08 and CHG-13.

**Solved form.** **Addresses from stable ids** (Gen, Pyro; shelf `probabilistic-programming.md` §2.1, §3 point 1); **counter-based draws and trace operations** (Philox, JAX `fold_in`, Gen `constraints`, Pyro `condition`; shelf §3 points 2 and 4); **random variables as values, lifted through expressions** (the stochastic lambda calculus, Ramsey and Pfeffer 2002, as the draft's lifting needs; Stan's "only declared places consume randomness" is kept for what samples, not for what is random) (C-25); **one named quantile type** (Hyndman and Fan type 1; shelf §2.7).

**Reduction.** A random fact is an input or def whose value is a distribution; its address is its own id and keys, so no separate key exists to forget. A path's sample of it is a counter-based function of seed, address and path. Anything that reads it is random by lifting. A deterministic read is a query on the distribution (Pyro's `condition` and a quantile), written at the read.

**Change** (core).
```
input growth[y in year]: dist 1/year
version assumptions
  cite own_assumption
  known 2026-01-01
  governs 2026..
  growth[*] = lognormal(0.03 /year, 0.10) "planning assumption"
def balance_end[y in year]: money DUK at end_of(y) = balance_end[y - 1] * (1 + growth[y] * 1 year)
ask median_2040: quantile(balance_end[2040], 50 %) paths 10000 seed 7 law 2040-12-31
ask spread: share(balance_end[2040] > 1 000 000.00 DUK) paths 10000 seed 7 law 2040-12-31
```
- **`draw` is removed.** Type `dist UNIT`; a row or a def's value may be a distribution from the closed list, `discrete(w for k in IDX)` (1.0.1's form, value first), or `empirical(e for k in IDX)` (a fact's own sample rows). A row `x[*] = v` states every key of x, as 1.0.1's `x[*]` pin does (M-15).
- **Lifting**: an expression that reads a random fact is random; in an ask with `paths N seed S`, each path samples every random fact at its address; without `paths`, an ask line whose value is random must apply a query (`mean`, `quantile(·, q)` of type 1, `cdf`, `share`), else it is refused (`LOAD-RANDOM`).
- **Decisions and projections** read a random fact by sample on a path and, in a projection, with the stated `random at mean` or `random at quantile(q)` (D-6), so a decision never sees its path's future (R-04). `plan v` is removed.
- **Conditional queries**: `where c` inside a query's argument (`quantile(x where c, q)`, `mean(x where c)`) restricts it to the paths where c holds, and the ask prints how many paths that leaves (SF-21).
- **Pins on samples**: `with sample growth[2030] = 0.01 /year` fixes an address, and `with sample growth[*] = …` every address of the fact (Gen's constraints; SF-28): the test mode and the hand-worked path (`P:RULE-09`). `path p` replays one path (`G:MC2-13`); `paths()` reads N (`G:MC1-7`).
- **Queries without paths**: a query on a random value is computed in closed form where the spec's table of closed forms gives one; otherwise the ask states `paths N seed S`, else it is refused (`LOAD-RANDOM`), so no hidden N or seed is used (C-10).
- **No built-in verdict**: `share` prints n/N, its standard error and the horizon (the carry, which needed `plan`, goes with it; C-10); whether a share is sayable is the model's or the reader's rule (`LC-28`, `G:MC2-7`; E §8 changes).
- **Equivalences that break:** "items over a draw are keyed by path"; "a look-ahead reads a draw's plan value".

**Explicit form.** A random value in a deterministic line names its query (`LOAD-RANDOM`); a projection over random amounts names its query (D-6); a seed and N are stated on a paths ask.

**Alternatives.** Keep `draw` with an implicit path key: rejected; it keeps two systems. A planning value stored beside the distribution: rejected (PR-11's statement). Correlation by explicit copulas: deferred; reading one random fact twice gives full correlation, which is what 1.0.1's shared draws expressed.

**Axes closed.** RAND (`LC-26`, `LC-27`, `LC-28`, `G:MC1-7`, `G:MC2-7`, `G:MC2-12`, `G:MC2-13`; `S:18` with D-14), RULE (`P:RULE-09`), LIFE, NUM, SURF (`Q:D-10`, with D-15's choice).

**Effect on the 79.** None of the 79 draws (K13); the closed-form distributions (CH-01, CH-03, LH-17) keep `cdf`, `pdf` and `pmf`, now over `dist` values (automatic). Manual: the two Monte Carlo models in `montecarlo/`.

**Shown.**
RAND: a forgotten key is impossible; this line is random without one.
```
def pension[y in year]: money DUK at end_of(y) = contributions[y] * (1 + growth[y] * 1 year)
```
NUM: a planning figure as a question.
```
def plan_growth: rate at reader = quantile(growth[2027], 0.25)
```

**Checked against.** Hard set: `Q:D-10`. Invented: none.

**Open.** Correlation between distinct random facts beyond reading one fact twice; the size of the closed list once the visible cases are carried.

---

## D-12. Exceptions are written pairs, a replacement says so, and conventions are named codes a fact can set

**Problem.** PR-12: which rule wins, which rule replaces which, and which convention applies are carried by transitive chains, shared labels, the target's kind and closed lists, so an exception to an exception, an amendment of one row and an unstated convention can't be written directly. Items (9): `P:LAW-12`, `Q:B-05`, `O:ASK-04-1`, `O:LAW-12-1`, `O:B-5`, `O:D-3`, `S:4`, `LC-2`, `LC-24`. Axes: eight. With H-9 to H-12.

**Today.** `excepts` is transitive "whatever the rules in between do" (L §5.10 step 4; `LC-2`), so a three-level exception moves to a second def (`P:LAW-12`, `O:LAW-12-1`). Rules sharing a label are versions of one rule, so the label decides replace or add (H-10). `v V` does different things on different constructs (H-9); a version row is an input row or a given by its target's kind (H-11); a band version replaces the whole table, an input version one row (H-12). A rate's basis is part of its type and can't be `assumed` (`O:ASK-04-1`, `O:B-5`, `Q:B-05`); an unstated rounding mode goes on an axis (`O:D-3`); a made-up currency has no minor unit (`S:4`); an axis is a closed list no fact can set (`LC-24`). `DECISIONS.md` chose the read rule for rule versions, decision 11, N-206, row 19, decision 28 and CHG-09.

**Solved form.** **Defeasible superiority as written pairs** (Nute; Antoniou, Billington, Governatori and Maher; LegalRuleML's `Override`; `prior-art/posting-rules.md` form 3): acyclic, pairwise, not transitive, scoped to rivals. **Amendment as a version, apart from an exception** (Governatori and Rotolo's two timelines; Pkl `amends`, KCL `:` against `=`; shelf `config-languages.md` T4; Catala's labels, shelf `catala-default-logic.md` §2.3). **Conventions as named, versioned codes** (FpML's day-count scheme, required, citing the ISDA definitions; `prior-art/calendars.md` form C; ACTUS's conventions as pure functions, shelf `actus.md` §3 point 5); **declared minor units** (ISO 4217; `prior-art/units-naming.md` form 2).

**Reduction.** A def's rules are rivals (they define one value at one key). `beats` is the superiority relation, used pairwise: an applicable rule wins if no applicable rule that beats it applies. An exception to an exception is two written pairs; the outer rule beats only what it names. A replacement is a new version of one rule or one row on its own timeline; it is written as `replace`. A convention is a value of a code type with a fixed meaning defined in the spec; since it is a value, an input can hold it and a source can state it.

**Change** (core: `beats`, `replace`, code types; sugar: `given` keyword).
```
def relief[p in person]: money DUK at reader
  rule base = 0 DUK
  rule disabled beats base when disabled[p] = 1 200.00 DUK
  rule child_disabled beats disabled when age(p) < 18 years = 2 400.00 DUK

version amend_2027
  cite act_2026_amendment
  known 2026-11-30
  governs 2027-01-01..
  replace relief.disabled when disabled[p] = 1 500.00 DUK
  replace band_rate row [50 000.00 DUK, inf) = 32 %
  given tax_due[ola] = 4 210.00 DUK "assessment notice"

unit DUK: currency minor 0.01
input day_basis: daycount
version contract
  cite loan_contract
  known 2026-01-01
  governs always
  day_basis = act/365 "contract §5"
def interest[m in months]: money DUK at end_of(m) = accrue(unpaid[m], line_rate, days_in(m), day_basis, simple)
```
- **`rule r beats q, …`** replaces `excepts`. Every rule of one def is a rival of every other. The applicable rules that no applicable rule beats are the candidates; one candidate is the value; two are a conflict (`RUN-CONFLICT`, as today). A cycle in `beats` is refused (`LOAD-CYCLE`). One rule can't win with the help of another (no team defeat).
- **`replace`** in a version makes a new version of one rule (`replace d.r …`), one row of an input (`replace x[k] = …`), one row of a band (`replace b row I = …`) or a thing's lifespan, `replace K + [k] lives span` (D-1, M-18), from the version's `governs`. A rule whose label exists without `replace` is refused (`LOAD-LABEL`, H-10). A band version may replace one row (H-12).
- **`given`** is written as a keyword on a version row aimed at a def (H-11); `v V` means the same on every construct: the item cites V and exists only where V governs (H-9). (After G1, P-14 and L §5.11 step 2 narrow the gating to rules: a rule citing a version exists only where it governs; verdicts PL-26, PL-32; round 18, C3-18.)
- **Code types**: `rounding`, `daycount`, `roll` (business-day convention) and `compounding` (renamed from `basis`, which is also a cost basis; C-32), each with a closed list of codes whose definitions are in the spec, versioned with the language. A code is a value: an input can hold it, a row can be `assumed`, a source can state it. A computation that needs a convention takes one as an argument; none has a default: `accrue(principal, rate, days, daycount, compounding)` takes both (M-16). The `roll` codes are `following`, `preceding`, `modified_following` and `unadjusted` (no roll: the day as it falls, M-17). The rate's basis moves from its type to a code (`O:ASK-04-1`, `O:B-5`, `Q:B-05`).
- **Minor units**: `unit X: currency minor m` declares a currency and its minor unit; `currency` is a prelude kind of units, so `u in currency` binds over the declared currencies (SF-29); no currency is in the prelude: every currency a model uses is declared with its minor unit (`S:4`, `LOAD-UNIT`; C-31).
- **Axes stay** for readings shown side by side that no fact settles (`LC-24`: a convention a fact can set is a code, not an axis).
- **Equivalences that break:** "`excepts` is transitive" (the migration writes the closure as pairs, so every 1.0.1 model keeps its answers); "a shared label replaces".

**Explicit form.** Every superiority written (`beats`), never inferred from order or specificity; every replacement written (`LOAD-LABEL`); every convention named (`LOAD-CONVENTION` for a computation with none); every currency's minor unit declared (`LOAD-UNIT`).

**Alternatives.** Catala's exception tree by reference: close, and kept in spirit (a rule names what it overrides), but its tree is transitive along a branch; written pairs are simpler and scoped. Specificity as priority: rejected (hidden; production systems' cautionary cases). Conventions as axes (1.0.1, decision 11): rejected for conventions a source states (`LC-24`); kept for true readings.

**Axes closed.** RULE (`LC-2`, `LC-24`), LAW (`P:LAW-12`, `O:LAW-12-1`), ASK (`O:ASK-04-1`), NUM (`O:B-5`, `O:D-3`), CUR (`S:4`), LIFE, SCEN, VAL (`Q:B-05`).

**Effect on the 79.** Warned: `excepts` (67 models) becomes `beats` with the transitive closure written, so answers are kept by construction, and every pair the closure adds (the chain links of LC-2, 35 of them) is listed for the writer to check against the source, since a pass-through may be the hazard LC-2 found (C-19); label reuse becomes `replace` where a later version reuses a label. The rate basis (LH-10, LH-11, CH-05, LH-18) moves to a code (automatic). `round` modes (53 models) are unchanged: every `round` already names its mode.

**Shown.**
LAW: an exception to an exception whose middle rule doesn't apply (`LC-2`).
```
def allowance[p in person]: money DUK at reader
  rule general = 500.00 DUK
  rule relief beats general when disabled[p] = 1 200.00 DUK
  rule reduced beats relief when income[p] > limit = 800.00 DUK
```
For a high earner who isn't disabled, 1.0.1's transitive chain lets `reduced` beat `general` through `relief` and prints 800.00, wrong in silence. Here `reduced` beats only `relief`; `general` and `reduced` are both unbeaten, so the value is a conflict (`RUN-CONFLICT`) that names both, and the writer states the intent: `when disabled[p] and income[p] > limit`, or `reduced beats relief, general`.
CUR: a currency the model invents.
```
unit LEN: currency minor 1
```

**Checked against.** Hard set: `S:4`, `Q:B-05`. Invented: I-4 (a cost chosen by a rule), I-5 (an amendment mid-year).

**Open.** Team defeat, if a visible case needs several weaker rules to beat a stronger one together; the size of each code list.

---

## D-13. Calendars, schedules and periods are generated by rules with stated coverage, and nothing outside the run is dropped in silence

**Problem.** PR-13: calendars, recurring dates and periods are written out by hand as lists and indexes, and an occurrence outside the run's window is dropped without a word. Items (8): `Q:D-05`, `O:TIME-02-1`, `O:OWN-03-1`, `O:A-3`, `O:A-5`, `O:B-4`, `O:D-4`, `LC-11`. Axes: FLOW, NUM, OWN, REC, TIME.

**Today.** A holiday calendar is a `dates` list of literals per span (L §3.1, §4.2): a day stated by a rule ("22 June, from 2026") has no form (`O:TIME-02-1`), lists stop at the last year typed (`O:B-4`), "all known but one" can't be said (`O:A-3`), and `next_business(d, h)` reads h at the def's law date (`O:A-5`). Periods are indexes each model declares; there is no quarter (`O:D-4`, `Q:D-05`; `LC-11` asks for built-in plural calendars). "An instance dated outside [S, E] isn't part of the run" (E §4.1 step 2), with no word (`O:OWN-03-1`). `DECISIONS.md` chose N-306, N-307, the T rows of §2 and decision 12.

**Solved form.** **A recurrence set algebra** (RFC 5545's rule parts in a fixed order, `(start ∪ rule ∪ added) − removed`; RFC 7529's `SKIP`; `prior-art/calendars.md` form A); **an unadjusted schedule, then a business-day adjustment** (ISDA 2006 §4.12; FpML's schedules and relative date offsets, with its consistency rules as load checks; QuantLib's `Schedule`; `prior-art/calendars.md` form B); **holiday calendars as rule-generated sets with stated coverage** (Jollyday's validity ranges, QuantLib's added and removed sets, Strata's silent weekends-only fallback as the warning; `prior-art/calendars.md` form D; shelf `actus.md` §2.6, §3 point 5).

**Reduction.** A calendar is a set of non-business days given by rules (weekdays off, a fixed day each year from a year, added and removed days), and its coverage is the span its row governs (D-3). A schedule is a generator of unadjusted dates (anchor, frequency, day, stubs) followed by an adjustment under a named `roll` code (D-12) against a named calendar. A window of business days ("the last three business days of the month") is a schedule whose result is D-7's window. The built-in period indexes are generated, as `days`, `weeks`, `months`, `quarters` and `years` over the run.

**Change** (core built-ins and one type).
```
input len: calendar
version len_rules
  cite synthetic_calendar
  known 2020-01-01
  governs 2020..2035
  len = calendar(weekends [sat, sun], yearly [01-01, 01-06, 05-01, 05-03, 08-15, 11-01, 11-11, 12-25, 12-26])
version len_2026
  cite gazette
  known 2025-12-01
  governs 2026
  replace len = calendar(weekends [sat, sun], yearly [01-01, 01-06, 05-01, 05-03, 08-15, 11-01, 11-11, 12-25, 12-26], added [06-22])

def sweep_day[m in months]: date at start_of(m) = schedule(m, day 20, following, len)
def pay_window[m in months]: window at start_of(m) = window(last_business(m, 3, len), last_business(m, 1, len))
```
- **`calendar` type**: a value built by `calendar(weekends […], yearly […] [from Y], added […], removed […])`; a day in `added` may be a hole or a window (`one_of`, `window`; `O:A-3`: every day known but one; M-19). Its coverage is the span of the row that states it.
- **Business-day functions** (`is_business`, `add_business(d, n, cal)` with n of at least 1, counting business days after d, so a holiday d needs no roll (n = 0 is refused; rolling a day is `schedule`'s job, C-8), `last_business(p, n, cal)`, `first_business(p, n, cal)`, `schedule(p, day n, roll, cal)`) read the calendar at the day they ask (D-4). A day outside the calendar's coverage is a hole naming the calendar and the year (`HOLE-CALENDAR`), never a weekends-only guess.
- **Schedules** take every parameter: the day, the `roll` code and the calendar; an invalid day (31 February) is refused unless the schedule states `last` or `skip`; a schedule that yields no date in the run is refused at load (`LOAD-SCHEDULE`).
- **Dates** keep 1.0.1's calendar built-ins (L §4.2: `start_of`, `end_of`, `year_of`, `month_of`, `day_of`, `add_days`, `add_months`, `add_years`, `overlap_days`, `age`, `weekday`, `pos`), with `quarter_of`, and `days(a, b)` renamed `days_between(a, b)` so that `days` names only the built-in index; `next_business` and `prev_business` become `schedule` and a roll code (M-20). Spans `a..b` include both end periods, as in 1.0.1 (L §4.1).
- **Built-in period indexes** `days`, `months`, `quarters`, `years`: the calendar periods (quarters from January) that meet the run's [S, E], a partial period at either end included as a whole key (`LC-11`, `O:D-4`, `Q:D-05`). `weeks` is not built in, since a week's first day is a choice; a model declares its own index of weeks, or any anchored period. `start` and `end` are required in every model (LC-11, the learner's "you can't have laws without time") (C-12, C-21).
- **The run bounds booking, not facts**: a binding over movements, or a read of one, ranges over every stated movement (observed rows of any date; planned instances of every key the move's binding gives), and only booking into balances is limited to [S, E] (SF-9).
- **Outside the run**: a movement instance dated outside [S, E] isn't booked, and the register lists, per movement, how many were dropped and the first such day (`O:OWN-03-1`; E §4.1 changes; invented, no source defines it).
- **Equivalences that break:** "a holiday list ends where the writer stopped" (a day past coverage is now a hole).

**Explicit form.** A calendar's coverage (its row's `governs`, required by D-3); every schedule parameter (`LOAD-SCHEDULE`); a read past coverage is a hole (`HOLE-CALENDAR`); dropped instances are listed.

**Alternatives.** Calendars as literal lists (1.0.1): rejected (`O:B-4`). A library's defaults (QuantLib's backward rule and Following, ISDA's fallback): rejected (rule 8). Recurrence rules with RFC 5545's silent drop of invalid days: rejected; the schedule states `last` or `skip`.

**Axes closed.** TIME (`O:TIME-02-1`, `O:A-3`, `O:A-5`, `O:B-4`, `O:D-4`, `LC-11`), OWN (`O:OWN-03-1`), FLOW, NUM and REC (`Q:D-05`).

**Effect on the 79.** Meaning, warned: business-day functions and `dates` holiday lists reach the household (a list becomes a calendar with `added` days and its coverage the list's years). `day_of`, `start_of`, `end_of` and the index ranges (34 models) are unchanged.

**Shown.**
TIME: a holiday stated by rule from a year.
```
  len = calendar(weekends [sat, sun], yearly [06-22] from 2026, added [], removed [])
```
FLOW: the next business day of the 10th, as a schedule.
```
def due[m in months]: date at start_of(m) = schedule(m, day 10, following, len)
```

**Checked against.** Hard set: `O:OWN-03-1`, `S1`, `S2`, `S3`. Invented: I-5 (a calendar with an unknown closure day).

**Open.** A rule for a holiday that moves with a lunar or computed date is not provided; a synthetic case would list it as `added` per year.

*Changed at G2: see GD-5, GD-7.*

---

## D-14. One sign defines and another compares, values are named, and figures carry units or sources

**Problem.** PR-14: one symbol carries two meanings, some values are read by position, the literal allowlist admits figures, and the grammar disagrees with the text, so a reader can't tell a definition from a comparison and a swapped or bare figure loads. Items (17): `P:SURF-01`, `Q:B-10`, `S:1`, `S:2`, `S:3`, `S:6`, `S:12`, `S:15`, `S:17`, `S:18`, `S:19`, `LC-1`, `LC-3`, `LC-4`, `LC-5`, `LC-7`, `LC-8`. Axes: ASK, RAND, REC, RULE, SURF, TIME.

**Today.** `=` is definition and comparison (`LC-3`); `-` is minus and not applicable (`LC-4`); a rule line runs condition and value together (`LC-5`); aggregates put the binding first (`LC-7`) and declarations put the value last (`LC-8`). The literal rule admits 0–31, 100, 360, 365, 366 and 1000 (L §7.1), so 100 or 1000 load bare (`P:SURF-01`, `S:6`, `LC-1`). Tuple rows are read by position (L §5.7, §5.18; H-15). A month literal is a period or an imprecise date by its target (H-20); a bound variable outranks a key of the same spelling (H-23). The grammar and the text disagree in seven places (`S:1`, `S:2`, `S:3`, `S:12`, `S:18`, `S:19`); a pin on a record field and a pin's place in the cycle check are unsaid (`S:15`, `S:17`, `Q:B-10`). `DECISIONS.md` chose N-215 and decision 12 (the literal rule), decision 25 and decision 27.

**Solved form.** **Units of measure as a free abelian group** (Kennedy; F#; `prior-art/units-naming.md` form 1), already 1.0.1's model (L §3.2), with Kennedy's refusal of a bare figure in arithmetic; **the money value object** (Fowler; JSR 354; `prior-art/units-naming.md` form 2); **named binding at the point of use** (records and labelled arguments; Rice et al. 2017 on swapped arguments; form 3); **definition apart from comparison, and one text for grammar and prose** (ALGOL's `:=`, Catala's `equals`; CommonMark's executable examples, SpecTec; form 4).

**Reduction.** Comparison gets its own sign; `=` keeps definition and rows, where it is most often written. Not applicable is D-7's `none`. Every figure is a quantity with a unit, and one that the model doesn't take from a source is a bare figure, refused in arithmetic. Positional tuples become records with named fields. The grammar's examples are parsed as tests in phase 4.

**Change** (sugar, except the literal rule, which is core).
```
def over_limit[p in person]: bool at reader = income[p] > limit
rule high when bracket[p] == top = income[p] * top_rate
version rates
  cite act
  known 2026-01-01
  governs 2026..
  split[ola] = (share: 60 %, base: 12 000.00 DUK)
```
- **`==` compares**, `!=` stays, `=` defines and states rows. L §2.3's parse rule 1 (the longest prefix before ` = `) goes: a rule's value starts at the first `=` at bracket depth 0 (`LC-3`, `LC-5`).
- **`none`** replaces `-` as not applicable (D-7; `LC-4`).
- **The literal rule** (LC-1's option (b), the learner's): in defs, rules, moves, fills and band formulas, the only figures written inline are 0, 1 and −1, with or without a unit (`0 DUK`, `1 day`, `-1 day`, `100 %`), and offsets in key arithmetic (`m - 1`). Every other figure, a day of the month and a date included, is an input stated by a row, so moving the investing day from the 15th to the 20th is one added row (`LOAD-LITERAL`, checkable at load). Rows, asks, pins, headers and index lists hold literals; `paths n` and `seed s` are counts on an ask. 100, 360, 365, 366 and 1000 leave the list (a day count is a `daycount` code, D-12). A currency's minor unit is read as `minor(DUK)` (C-3). `0`, `1` and `-1` with no unit are allowed where the unit is fixed by the other operand or the expression is a plain number (SF-24). `%` is the unit 1/100 (`P:SURF-01`, `S:6`, `LC-1`).
- **Records** with named fields replace every positional tuple, in rows and in observed movements (D-8; H-15).
- **A month literal is a period**, and so is a four-digit year where a period is expected (`governs 2026`, a key of `years`); in an integer index it is an integer; where both readings fit, it is refused (`LOAD-AMBIG`, LC-11's `2026..2030`). An imprecise date is written as a window (D-7) (H-20) (C-21). A binding that shadows a key of the same spelling is refused (`LOAD-SHADOW`, H-23).
- **Grammar and prose**: every Tally example in tally 2's L is parsed by parsecheck in phase 4, which closes `S:1`, `S:2`, `S:3`, `S:12`, `S:18` and `S:19`; a pin on a record field pins that field, and a pin's value is evaluated unpinned, outside the cycle check (`S:15`, `S:17`, `Q:B-10`: stated in L §5.10).
- **Value first** (LC-5's option (b), LC-7's option (b) and LC-8's principle, each the learner's preference): a construct that gives a value states it first, then its bindings, filters, condition and dates. The rule line is `rule L [beats …] [governs …] = v [when c]`; a short def is `def x[k]: T = v at E`; aggregates are `sum(t for x in K over s where c)`, `argmin(t for x in K) ties by e`, `min(v for v in a .. b step s where c)`; `solve lhs == rhs for x in [a, b] to u`. A bare binding (`count(x in K on d where c)`) and `if c then a else b` keep their form, and headers are fields, not values. It deletes L §2.3's parse rule 1 (a rule's value no longer meets its condition mid-line), and the migration is a mechanical reprint (automatic). The examples in these records keep the phase-3 order; the spec (phase 4) is written value first (C-18).

**Explicit form.** A bare figure in arithmetic is refused (`LOAD-LITERAL`); a positional row is refused (`LOAD-FIELD`); a shadowing binding is refused (`LOAD-SHADOW`).

**Alternatives.** `:=` for definition: rejected, since rows and definitions (the most common lines) would all change, while comparisons are fewer. Keep the allowlist: rejected (`S:6`). Units optional: rejected (`P:SURF-01`).

**Axes closed.** SURF (`P:SURF-01`, `S:1`, `S:2`, `S:3`, `S:6`, `S:12`, `S:19`, `LC-1`, `LC-3`, `LC-4`, `LC-5`), ASK (`S:15`, `S:17`), RAND (`S:18`), REC, RULE, TIME (`Q:B-10`). Deferred: `LC-7`, `LC-8` (*closed at G2 as adopted here: see GD-25*).

**Effect on the 79.** Surface, automatic: every comparison `=` becomes `==` (nearly every model); `-` becomes `none` (19 models); tuple rows become named records (7 models) by the type's field order. Manual per occurrence: bare figures 100, 360, 365, 366 and 1000 in arithmetic (counted in phase 5).

**Shown.**
SURF: a bare figure refused.
```
def monthly[y in year]: money DUK at reader = annual[y] / 12
```
is refused (`LOAD-LITERAL`); `annual[y] / months_per_year` with a sourced input, or `annual[y] * (1 month / 1 year)`, loads.
REC: a record row with named fields that can't be swapped.
```
  split[ola] = (share: 60 %, base: 12 000.00 DUK)
```

**Checked against.** Hard set: `P:SURF-01`. Invented: all (every card's figures).

**Open.** Whether `1 month / 1 year` is a sound unit identity (months have different lengths) or a `daycount`-style code is needed for it: phase 4.

---

## D-15. A model reads its facts' metadata and dependents, explanations have a stated order, and inverses and choices have their own forms

**Problem.** PR-15: a model can't read the metadata of its own facts or list what depends on a fact, the order of an explanation is undefined, and a question that asks for a choice or for the inverse of a step function has no form of its own. Items (12): `P:VAL-01`, `P:REC-05`, `P:SCEN-03`, `P:ASK-04`, `P:ASK-05`, `Q:A-05`, `Q:B-07`, `Q:D-10`, `Q:D-11`, `S:5`, `LC-22`, `G:C2-F6`. Axes: twelve.

**Today.** A rule can't read a fact's version, span or end (`P:VAL-01`). No ask lists what reads an input (`P:SCEN-03`, `Q:D-11`). `explain`'s order of causes is "canonical order", undefined (`S:5`, `P:ASK-05`), and a list of reasons is reachable only through `explain` (`Q:A-05`). Whether a `flows` register lists instances dropped as out of span is open (`G:C2-F6`). A choice question has no refusal of its own, so `bound` is misused (`P:ASK-04`, `Q:D-10`; `LC-22` asks to drop `bound`). `solve` needs a sign change on a bracket (L §5.16), so the inverse of a step function is an `argmin` over an index of cents (`Q:B-07`), and an inferred figure with an unknown part needs a second ask (`P:REC-05`). `DECISIONS.md` chose decision 29, the Q rows of §2, and E §11 (no optimisation but a `solve` the law names).

**Solved form.** **Provenance as data** (why- and where-provenance, Buneman et al.; provenance semirings, Green, Karvounarakis and Tannen; W3C PROV-DM; spreadsheets' trace dependents; `prior-art/holes-provenance.md` form 3; shelf `datalog-incremental.md` §3 point 11, `openfisca.md` §2.9), with the semiring's commutativity forcing a stated order; **the generalized inverse, "least x such that"** (Embrechts and Hofert; MiniZinc's one solve item; CLP(Q)'s exact `inf`; `prior-art/holes-provenance.md` form 4).

**Reduction.** A fact's metadata are PROV attributes: a record a rule reads. Dependents are the inverse of lineage, an ask body. An explanation lists the derivation's terms; since the structure is commutative, the spec states the order. A gross from a net is the least x in a stated stepped domain whose function value reaches the target. A choice is the least over a finite set by a stated cost, with a stated tie-break. A step no row can give (decision 29's bound) is a kind of hole.

**Change** (core: `meta` and the stepped range binding; sugar: `readers-of`, `reasons`; no new form for inverses or choices, which are `min` and `argmin`).
```
def notes_to_sell[t in [2026-06-10]]: int at t
  = min(n in 0 n7 .. stock[t] step 1 n7 where net_proceeds[n, t] >= shortfall[t]: n)
ask which: argmin(o in options: total_cost[o]) ties by o.rank law 2026-12-31
def stale[k in asset]: bool at reader = meta(valuation[k], cut).known < add_years(cut, -1 year)
ask who_reads: readers-of rent_amt
ask gross_1: min(g in 0.00 DUK .. 100 000.00 DUK step 0.01 DUK where net_of[g] >= 5 000.00 DUK: g) law 2026-06-30
```
- **`meta(x[k], d)`** is a record: `version`, `source`, `known`, `governs` (with `.from` and `.to`; for a reading, `at`, the reading's instant), `held`, `assumed`, of the row that governs x[k] on d (`P:VAL-01`; `in_force` is D-4's).
- **No user functions.** A def keyed by a stepped range is read with brackets like any keyed def (`net_of[g]`), evaluated only at the keys read; `low(x, a .. b)` and `next(K, d)` are built-ins, not library, since they take a place or a kind and pass on `from`, `windows` and `random` clauses, which a def can't (C-24).
- **`readers-of x`** lists every item that reads x, directly or not (`P:SCEN-03`, `Q:D-11`); `inputs-of` stays. **`reasons(x)`** is the list of rule labels that gave x its value, as data (`Q:A-05`).
- **Explanation order**: causes in the order their terms are written in the winning rule, depth first, keys in index order (`S:5`, `P:ASK-05`).
- **"The least x such that c"** is the existing aggregate over a filtered binding, `min(v in D where c: v)`, once a binding may range over a **stepped range**: `a .. b` of integers, or `a .. b step s` of quantities (the range binding that D7 removed for want of use comes back, with a required step for quantities). D is finite by construction, so the result is exact; no value satisfying c is the aggregate's refusal of an empty set (`RUN-EMPTY`), loud. The engine may bisect when c is monotone in v; that is speed, not meaning. This gives the inverse a gross-from-net question asks (`Q:B-07`); with D-7's `partial`, an inferred figure with an unknown part is one ask (`P:REC-05`).
- **A choice** is `argmin(o in S: f)` with D-5's `ties by`; an empty S is refused (`RUN-EMPTY`), so a choice question no longer borrows `bound` (`P:ASK-04`, `Q:D-10`).
- **`bound` is removed with its outcome** (LC-22, the learner's "drop it"): a step no row can give is an ordinary hole whose question says why (`?"the paper's figure comes from R's random stream"`); no fourth outcome and no hole kind (C-20).
- **`solve`** stays for continuous roots (5 models).
- **The register** lists movement instances that a `flows` span or the run's window left out (`G:C2-F6`, with D-13), and inputs and defs that no ask, movement or check reaches (a lint, not a refusal: a model under construction has them; SF-22).

**Explicit form.** A range of quantities without a `step` is refused (`LOAD-DOMAIN`); an `argmin` without `ties by` where ties can occur is refused (D-5's `RUN-TIE`); a step out of reach is a hole with its reason, never a figure.

**Alternatives.** A new `least … such that` form (the candidate in `problems.md`): rejected by Iverson's test, since `min` over a stepped range already says it. Numeric goal seek: rejected (not exact). `argmin` over an index of cents (1.0.1): rejected (a workaround with a pin per key). Provenance only through `explain`: rejected (`Q:A-05`).

**Axes closed.** ASK (`P:ASK-04`, `P:ASK-05`, `S:5`, `LC-22`, `G:C2-F6`, `Q:B-07`), VAL (`P:VAL-01`), REC (`P:REC-05`), SCEN (`P:SCEN-03`, `Q:D-11`), LAW, NUM, OWN, PTY, TIME (`Q:A-05`, `Q:B-07`), LIFE, RAND, SURF (`Q:D-10`).

**Effect on the 79.** Automatic: `bound` (CH-03, LH-17, MP-15) becomes a `beyond` hole with the same reason. `argmin` (5 models) and `solve` (5) are unchanged. `explain` (CB-07, the household) prints in the stated order, which may reorder its lines (warned).

**Shown.**
VAL: a valuation older than a year is flagged by a rule.
```
def stale[k in asset]: bool at reader = meta(valuation[k], cut).known < add_years(cut, -1 year)
```
ASK: the gross pay that leaves a net of 5 000.00 DUK under a step-wise tax.
```
ask gross_1: min(g in 0.00 DUK .. 100 000.00 DUK step 0.01 DUK where net_of[g] >= 5 000.00 DUK: g) law 2026-06-30
```

**Checked against.** Hard set: `Q:A-05`, `Q:D-10`, `S3` (the least number of notes). Invented: none.

**Open.** Provenance through aggregation and negation (how `reasons` reports a `count` or a `not`): phase 4.

*Changed at G2: see GD-19.*

---

## Settlements of the sketches (phase 3)

The author sketched the 18 hard-set cards in the core's notation (`hardset/sketches/`) before the modeller and the critic started. Each strain found is settled here: **fixed** (the record's Change or Explicit form now says it; the record is named) or **answered** (no change, with the reason). None adds a concept; each fix states what a record left unsaid or makes two records agree.

| SF | Card | Finding | Settlement |
|---|---|---|---|
| SF-1 | S3, migration | The prelude's place kinds can't fix a unit, so `place konto: account PLN` had no tally 2 form | fixed in D-1: a place kind's unit may be `one`, stated by each thing as it enters |
| SF-2 | S1, S2, P-TIME-07, Q-B-08 | A reading of a balance had no place in its day, and the answer turns on it when a decision acts that day | fixed in D-3: `at D start`, `at D end`, `after m`; `LOAD-INSTANT` |
| SF-3 | S1 | The projection's value depended on who read the def (a dynamic scope, like the one D-4 removes) | fixed in D-6: `bal(x, d) from t`, with the dependents of the item holding the read left out; a plain future read inside a decision is a cycle |
| SF-4 | S1, all | No record showed where a movement's law date is written | fixed in D-2 and D-4: `at e` as a line of `move` |
| SF-5 | S2 | A movement that reads a decided movement's leg, but no balance, was classed as a plan | fixed in D-5: reads of other movements are edges, visible in the text |
| SF-6 | S2 | A window that matters through the whole walk had nowhere to state its choice | fixed in D-7: the clause on an ask line |
| SF-7 | S3 | A leg's unit had no reader | fixed in D-1: `unit(q)` of a quantity |
| SF-8 | S3 | `LOAD-ALIVE` refused D-2's own example (a def keyed by sales with no `over`) | fixed in D-1: key bindings need no `on`/`over`; aggregates and listings do |
| SF-9 | S3, O-OWN-03-1, Q-B-05 | Lots bought before `start`, and a payment after `end`, were invisible to bindings | fixed in D-13: the run bounds booking, not facts (*`flows` before S changed at G2: see GD-5*) |
| SF-10 | S3 | A regime's inputs could never get rows, since versions sit outside its block; a regime couldn't declare its own event kinds | fixed in D-2: version rows may state a regime's input rows and `given` values; the block may hold kinds |
| SF-11 | Q-E1-01 | "An account nobody has told us" had no form | fixed in D-7: a leg's place may be a hole; the known side books |
| SF-12 | Q-B-04 | Whether an unfulfilled uncertain movement is in a binding was unsaid | fixed in D-6 (bindings count observed and certain planned movements; `with uncertain` adds the rest) and D-2 (a subkind may state its own certainty) |
| SF-13 | P-VAL-01 | A state input read past its last span was unsaid (`last` was for readings only) | fixed in D-3: `RUN-UNGOVERNED`, and `last` on a state input marks the value `held` |
| SF-14 | O-SCEN-02-1 | Dropping an observed movement in a branch | fixed in D-10: `replace x[k] = none` removes it with its legs |
| SF-15 | P-TIME-07 | The reverse order couldn't be written without a cycle | fixed in D-5: `before m` allowed outside cycles, as a written order (*against a plan, changed at G2: see GD-11*) |
| SF-16 | P-OWN-02, S-4 | `value(x, d) in CUR by p` demanded a price input in a model of one currency | fixed in D-1: `by p` only when some position is in another unit; `LOAD-PRICE` |
| SF-17 | Q-B-01 | One regime couldn't read another's result (income tax reads VAT not deducted) | fixed in D-2: `export x, …`; `LOAD-PRIVATE` for the rest |
| SF-18 | Q-B-01 | "Alive on a day" had no test form | fixed in D-1: `x in K on d` as a test |
| SF-19 | Q-D-10 | A thing whose lifespan is random | answered: not in the core; a random death is a random fact and "alive" a def, which the card needs and the core says. Listed in `core.md` as not yet said |
| SF-20 | Q-B-04, Q-B-01, Q-A-05 | Event kinds' attributes had no declaration form | fixed in D-2: `attr name: type`, stated by each movement |
| SF-21 | Q-D-10 | Percentiles over the paths that meet a condition | fixed in D-11: `where c` inside a query |
| SF-22 | P-SURF-01 | A figure nothing reads wasn't flagged | fixed in D-15: the register lists unreached inputs and defs |
| SF-23 | Q-A-05 | `windows each` printed one line per day, not per case | fixed in D-7: one line per maximal sub-window with a constant answer |
| SF-24 | Q-D-10, D-11's example | The literal rule refused a bare 1, which D-11's own example uses | fixed in D-14: bare `0` and `1` where the unit is fixed or the number plain |
| SF-25 | Q-B-08 | A duplicated statement row | fixed in D-9: `same` also inside one model |
| SF-26 | P-PTY-05 | "The deriving model's statement stands" needs a precedence | fixed in D-9: `same p = q prefer q` |
| SF-27 | P-PTY-05 | An outside place joined to an account: which kind | fixed in D-9: the non-outside kind; `LOAD-SAME` when two others differ |
| SF-28 | Q-D-10 | Pinning every sample of a fact | fixed in D-11: `with sample x[*] = …` |
| SF-29 | S-4 | `currency` as a kind of units to bind over | fixed in D-12 |
| SF-30 | S-4 | A balance reading of an `any` place in two units | fixed in D-3: one row per unit |
| SF-31 | Q-B-05 | A branch "as if known then" against a change learned later | fixed in D-10: the scenario version's `known` decides, and the writer states it |
| SF-32 | Q-E1-01 | The joint model's own movement into a member's place | fixed in D-9: matched like any internal movement |
| SF-33 | Q-B-04 | The refund's amount needs a figure "as known" on another day inside a def | answered: two asks give it today; whether a def read may carry `as known` is checked against the visible cases in phase 4 |
| SF-34 | Q-B-08 | One list in date order across four kinds of problem | answered: four asks answer the question; a merged printout is left to phase 6 to judge |
| SF-35 | S3, Q-A-05 | "At most half of a loss in one year" has no fill parameter | answered: a keyed fill whose supply reads earlier takes (L §5.11, kept) says it; phase 6 judges whether a per-arc cap earns a construct |
| SF-36 | Q-A-05 | Defs that differ by subkind (a gift's holding starts with the giver) | answered: a def keyed by the parent kind with `if x in gift`; no def overloading |

## Settlements of the critic's findings (phase 3)

A fresh critic read `bar.md`, `problems.md`, these records, `core.md`, the learner's concerns and L §1 as an unnamed author's draft (its report: `agents/critic.md`, 32 findings). Each is settled here, fixed in the record named or answered with a reason. Where a fix adds a form, the last column gives Iverson's test: what the language already had, and why it doesn't say it.

| C | Finding (short) | Settlement | Iverson's test for an addition |
|---|---|---|---|
| C-1 | Ending a lifespan makes money vanish from totals; past facts of ended things unreadable | fixed in D-1: a place ends only at zero, a leg outside a lifespan is refused (`RUN-LIFESPAN`); `lifespan(k)` lets a rule read facts at the end of a life | `lifespan(k)` is `meta` of the key's lifespan row; it is written as a built-in because a lifespan isn't an input row |
| C-2 | `x != s` silently true for `none` | fixed in D-7: any comparison with `none` but `x == none` / `x != none` is refused | none added; a form removed |
| C-3 | The literal rule leaves LC-1 open and lets unit figures load inline | fixed in D-14: LC-1 (b), only 0, 1, −1 and key offsets inline; day numbers and dates are rows; checkable at load | `minor(u)` reads a declared minor unit, which the rule would otherwise force into a row restating the declaration |
| C-4 | Ask-line clauses bring back reader-dependent values | answered in D-7: an ask's clauses set its world like its pins and `as known`, and are printed; one def, one value per world | |
| C-5 | Migration of phases claimed answer-preserving but wasn't; edges through defs inferred | fixed in D-5: `reads` also names movements read; migration writes `before` for every reversed pair, not only cycles | none added: `reads` takes one more sort of name |
| C-6 | Coherence of postings undecidable and contradicts "two regimes may act on one event" | fixed in D-2: the posting check is dropped; one name, one value stays; `readers-of` lists the regimes reaching an event kind | none added; `LOAD-COHERENCE` removed |
| C-7 | D-4's example breaks the refcount; exemptions hide references; plain defs can't read a regime's result | fixed in D-2 and D-4: any item reads exports; the refcount prints wiring, fact rows and export reads separately; the example is an ask | none added |
| C-8 | Conventions unstated: `value`, `split`, `accrue`, `add_business(…, 0, …)` | fixed in D-1, D-4, D-13: these compute exactly and round only where `round` says; `add_business` needs n ≥ 1, rolling is `schedule`'s. *Changed at G2: see GD-7.* | none added |
| C-9 | `outside` places silently left out of totals | fixed in D-1: bindings include them; totals filter in writing | none added; a hidden filter removed |
| C-10 | Random query without `paths`; the carry kept without `plan` | fixed in D-11: closed form where the spec's table has one, else `paths N seed S` or `LOAD-RANDOM`; `share` prints no carry | none added |
| C-11 | A gap with no pad re-anchors by default | fixed in D-8: each statement version writes `pad x from y` or `anchor x` (`LOAD-GAPROLE`) | `anchor x` is new: a reading had no way to say "restart the walk here" apart from silence, which is the default removed |
| C-12 | Built-in period indexes' anchors unstated | fixed in D-13: calendar periods meeting [S, E], partial ends as whole keys; `weeks` not built in | none added; one index removed |
| C-13 | A movement's day undefined with legs on several days or a window | fixed in D-2: `move` states `on d`; legs state their own or take the movement's, as rows take a version's span | none added: `on` was a leg line; it is also a move line |
| C-14 | D-9 matches internal movements by equality; an outside place changes kind; law copies identified by name | fixed in D-9: links are written (`same`, `fulfils`); an outside place lends only its legs; law parts by declared id | none added |
| C-15 | LC-6 and LC-25 claimed closed, untouched | fixed in D-3: a rule may state its own `governs` (LC-6 (a)); `covers x` may state its own span (LC-25) | none added: the rule line takes the row's clause |
| C-16 | Ownership absent | fixed in D-1: `owns` kept from 1.0.1 and listed in `core.md`; a split by origin is a regime's lots | none added |
| C-17 | LC-20 and LC-23 closed only in part | LC-23 fixed in D-6 as the learner directed: `arrears` and `correction` removed; δ is an adjustment settled by ordinary movements that `fulfils` it; detection kept. LC-20 answered: certainty stays a stated property of the event kind because the deterministic forecast needs a yes or no (qualities 2 and 4); "will it arrive" as a probability is a random amount or day (D-11), which the core already says | none added; two forms removed |
| C-18 | LC-5 closed with the option it rules out; deferring LC-7/LC-8 contradicts the `==` change | fixed in D-14: value first everywhere (LC-5 (b), LC-7 (b), LC-8), as the learner chose; parse rule 1 deleted; a mechanical reprint | none added; a parse rule removed |
| C-19 | Migrating `excepts` chains keeps LC-2's hazard unseen | fixed in D-12: the closure is written and its added pairs are warned, not automatic | |
| C-20 | `bound` renamed, not dropped | fixed in D-15: an ordinary hole with its reason; no hole kind | none added; an outcome removed |
| C-21 | LC-11's agreed parts (start and end everywhere, plural names, four-digit ranges) open | fixed in D-13 and D-14: `start`/`end` required; built-ins plural; a four-digit literal where a period and an integer both fit is refused (`LOAD-AMBIG`); the spec's examples use plural index names | none added |
| C-22 | Contradictions: overlapping rows without `replace`; ungoverned day a refusal and a hole; ordering `never`; scenario `governs` before `from`; `core.md`'s kept list | fixed in D-3 (`replace` for overlapping rows, `HOLE-UNGOVERNED`), D-7 (`never` orders), D-10 (`governs` from `from`), `core.md` | none added |
| C-23 | Examples a sibling record refuses | fixed in the records: `over` added to aggregates, the reading's carry on `by`, `95 %`, the code row in a version, `meta` of a reading, the lot order by a stated `seq` attribute; D-8's reading instant written `at … end` | |
| C-24 | `low` and `next` called library but need what defs can't do; parenthesised user calls | fixed in D-6 and D-15: `low`, `next` are built-ins; keyed defs are read with brackets; there are no user functions | none added; a hidden feature named and refused |
| C-25 | Solved forms that don't fit: `at reader`, D-11's lifting, D-9's orphan rule | fixed: D-4's reduction is the implicit parameter with a static type; D-11 cites the stochastic lambda calculus for lifting; D-9's orphan-rule claim withdrawn | |
| C-26 | Constructs already said: `attr`, `same` in a model, `gap`, `split by d0`, `prefer`, `partial`'s cap of 8 | `split by d0` and `prefer` dropped; `gap` marked sugar; the cap removed. Answered: `attr` is the field schema an observed movement needs anyway (D-8's named fields), and a movement's fields are dated by its day, so the one idea holds; `same` inside a model keeps both rows listed as evidence, where `replace … none` would remove one | |
| C-27 | The vocabulary is under-counted | fixed in `core.md`: a contextual-words row; `low`, `next`, `lifespan`, `minor` counted as built-ins; the library is the prelude's kinds only; exact counts by `counts.py` in phase 4 | |
| C-28 | Time-keyed inputs keep a hidden role | fixed in D-3: an input's period key is data; the law date picks among that key's rows | |
| C-29 | Q:B-04's refund needs an "as known" read inside a def | answered: the return is a movement; a later spending changes it, which is an adjustment (D-6) whose settlement is the refund, dated within the 45 days; no new construct (supersedes SF-33) | |
| C-30 | Every movement binding "may gain members", so `pending` forever | fixed in D-7: `pending` only when the binding's span reaches past the cut | |
| C-31 | "ISO currencies in the prelude" against `LOAD-UNIT` | fixed in D-12: no currency in the prelude | |
| C-32 | `basis` names two things; D-1's reason for the prelude confused; one superclass forces product kinds | fixed: the code type is `compounding` (D-12); the prelude's reason is migration only (the words are not product words); a thing has one base kind and any number of dated memberships of plain kinds (D-1), which also closes D-1's open "dated kind" | none added: a membership is `K + [k] lives span`, the existing form, on a plain kind |

**The examples.** The records' snippets keep the phase-3 reading order (condition before value, `bind: term` aggregates). The spec is written value first (C-18), and every example in it is parsed in phase 4.

**The central claim, after these settlements.** The concept count stays fifteen, with the contextual words now counted beside it (B6). A regime is a namespace that reaches movements and things by kind, posts its own movements and keeps its own state; it is "applied by itself" in the sense that no item outside it wires it to a thing or a movement, which the refcount's first number shows, while fact rows and reads of its exports are counted and shown. Two acts may both reach one event, as law allows; nothing checks that their filters are disjoint.

## Settlements of the modeller's findings (phase 3)

A fresh modeller wrote the 18 hard-set cards and the 5 invented cards in the core's notation from `core.md` and the records' Change and Explicit form sections only, without the author's sketches (`hardset/modeller/`, report `agents/modeller.md`). It found no card the core can't say (0 CANT); 18 cards needed an invention, 5 none. It read the records before the critic's settlements, so several of its findings were already settled by C-n; the rest are settled here (M-n). Its R-numbers are its own.

| R | Finding (short) | Settlement |
|---|---|---|
| R36 | Which day a read happens on; movement-keyed defs `none` outside the movement's day; a dead person's later facts `none` | fixed (M-1, D-1): a governing row is read even outside a lifespan; `none` only where no row governs outside it; a movement's facts don't lapse |
| R6 | D-2's `move assess[p in person, …]` has no `on`/`over` | fixed (M-2, D-1, D-2): a move's keys over a kind state `on`/`over`; the example is rewritten |
| R2 | D-1's example version has no `governs`; `governs` meaningless for lifespan and reading rows | fixed (M-8, D-3): rows with their own time take no `governs` |
| R14 | A total over planned movements prints a number, not `pending` | fixed (M-12, D-7): that is the forecast's meaning, and the register lists the planned instances it counted |
| R10 | A place's balance before its first reading starts from an unstated value | fixed (M-4, D-1): zero at the lifespan's start; a place alive before `start` needs an anchor, else `HOLE-ANCHOR` |
| R38 | `accrue` takes no compounding code: simple interest by default | fixed (M-16, D-12): `accrue` takes the day count and the compounding |
| R16 | A sourced literal with a unit loads inline | settled by C-3 (LC-1 (b)) |
| R3, R11 | `bal(x, d)` start or end of day; `bal(x)` undefined | fixed (M-4, D-1): end of day; `bal(x, d start)`; `bal(x)` is the running balance inside a movement |
| R7 | Keyed and listing asks | answered: 1.0.1's ask `for` bindings are kept (L §5.16: `ask q1: payable[f, m] for f in family, m in month`); the modeller couldn't read that section |
| R27 | No date functions | answered (M-20, D-13): 1.0.1's calendar built-ins (L §4.2) are kept, with `quarter_of`; `days(a, b)` is renamed `days_between` |
| R20 | A key entered at a parent kind is in no subkind | fixed (M-3, D-1): keys enter only at leaves (`LOAD-LEAF`) |
| R5 | No form for a `fill` over movements | answered (D-2): a fill's sides take any binding; the form is shown |
| R1 | No run header | answered: 1.0.1's header is kept (L §5.1), with `start` and `end` now required (C-21) |
| R4 | `lifespan(k)` undefined | settled by C-1 |
| R33 | Currency keys have no `lives` | fixed (M-3, D-1): declared units live always |
| R13 | No "no roll" code | fixed (M-17, D-12): `unadjusted` |
| R12 | A conditional movement becomes a zero one | fixed (M-7, D-2): `when c` on a move |
| R26 | Span ends inclusive? | answered (D-13): both end periods included, as in 1.0.1 (L §4.1) |
| R37 | May a subkind add legs or attributes? | fixed (M-6, D-2): yes; a parent binding reads only the parent's |
| R8 | `s.day` with legs on several days; binding by a stated date | settled by C-13 (`move … on d`); a binding by another date is a `where` on an attribute |
| R39 | Out-of-run instances; observed rows before the run | settled by SF-9 (the run bounds booking, not facts) |
| R40 | A window meeting an order, a second window or `low` | answered (D-7): the chosen world (the read's or the ask's `windows` clause) fixes every window, then the ordinary order applies; `windows each` gives one line per combination with a constant answer |
| R30 | A day that is a hole | fixed (M-13, D-7): kept from 1.0.1, extended to a lifespan's end |
| R28 | Observed rows dated by a window; observed keys undeclared | fixed (M-14, D-8): keys are the rows' ids; a window belongs in an attribute (a giver's acquisition), while a row's `on` is the day it was observed |
| R17 | Member identities with movement `same`, `prefer` and kinds | settled by C-14 and C-26 |
| R18, R19 | `one_of` in a calendar's `added`; `known always` | fixed (M-19, D-13) for the first; answered for the second: `known` takes a date or a window (M-9), never `always` |
| R9 | An invoice moves nothing | fixed (M-5, D-2): event kinds may have no legs |
| R23, R31 | The projection's left-out set depends on the def chain; does a projection count as a read | the first is the design (SF-3); fixed (M-11, D-6) for the second: `reads x` |
| R24 | No form for replacing a lifespan | fixed (M-18, D-12): `replace K + [k] lives span` |
| R29 | A payment that crosses days needs a transit place | answered: a leg conserves on one day; money in flight is a place, which is what it is |
| R32 | An order clause can't be reversed by a scenario | answered: the reverse order is a def reading the balance before the other movement, or a second model text (P-TIME-07's sketch) |
| R34 | No way to ask "joining changes no answer" | answered (D-9): by construction, since imports are pinned and never changed; a member's asks are asked inside the joint model as `a.q` |
| R35 | No date-ordered listing across kinds | deferred with SF-34 to phase 6 |
| R21, R22, R25 | `mean` without paths; a random death; `discrete` | C-10; SF-19; fixed (M-15, D-11): `discrete(w for k in IDX)` |
| others | a reading read inside a def (S3); `x[*]` rows (Q-B-05); `known` imprecise (I-5); `exists` (I-3); branching on the winning rule (I-2) | fixed (M-10, M-15, M-9) for the first three; answered for the last two: `count(…) > 0`, and `reasons(x)` is a list of labels a rule can test |

**What the modeller's run shows.** With the 36 sketch settlements, 32 critic settlements and these, every invention the modeller made has a written form in the records or a stated answer, and no card needed a construct that serves it alone: the fixes are general rules (which day a read happens on, which keys enter where, what a move's keys state) that each card met. The modeller's texts stay as written, as evidence of what the draft said before these settlements.

## Decisions after G1 (phases 4 to 6)

G1 passed on the records D-1 to D-15 and the three settlement tables above. Writing the spec (phase 4), running the reference interpreter over the migrated models (phase 5) and writing probes and households against it (phase 6) needed further choices. `bar.md` B6 asks that every construct or built-in added after G1 have its decision record on the G2 list; this section is that list, together with the judgment calls the rounds made where the spec had to pick a meaning. Each row names what was added, the row, site or finding that needed it, the solved form it reduces to, what it replaced or was chosen over, its effect on the counts, and its home in the spec. Rounds are logged in `scratch/lead/phase4-log.md` (entries 37 to 50) and their edit scripts are in `scratch/lead/`.

The vocabulary check behind the word rows: every word `counts.py` lists for the current spec was looked for in the code of the G1 package (`core.md`, `decisions.md`, `G1.md`) and of 1.0.1's `LANGUAGE.md` (`scratch/lead/postg1/new-words.txt`). Concepts, keywords and built-in names are all named there, except `since`. The contextual words `mon tue wed thu fri` (and `sat sun`, which the check finds in prose), `bond30_360`, `of_leg` and `adjustments`, and the built-in ids of the prelude's kinds, are new.

### Words and built-ins

| A | Added after G1 | Needed by | Form and source | Chosen over | Counts | Home |
|---|---|---|---|---|---|---|
| A-1 | The prelude's kinds named as sets, plural, with two roots: `places`, `events`, `accounts`, `holdings`, `assets`, `debts`, `claims`, `currencies`, `transfers`, `adjustments`, `pads` (G1 named `account`, `adjustment`, `pad`, `currency`) | phase 4 spec writing: a binding reads `x in accounts on d`; a total over every place or every movement (`flows(world, y)`, `m in events over s`) is a binding over a kind, with no special form | a kind is the set of its things (the binding reads as set membership); the roots make "every place" and "every movement" ordinary kinds | singular kind names, and a separate form for "all places" | built-in ids: 13 of the 26 are the prelude's kinds | L §5.7 "The prelude" |
| A-2 | `since(x[k], d)`, the age in days of the last reading of x[k] on or before d | the carry rules (D-3): a rule that tests a reading's age itself, beside `last(x[k], d) within n days` (`HOLE-STALE`) | the age of a sample, as a value; the same last reading `last` takes | writing the age as `d - day_of(last(…))`, which needs the reading's day as a value | built-in names: counted among the 84 | L §4.6 |
| A-3 | The weekdays `mon tue wed thu fri sat sun` as calendar values | round 12 (T-OPEN's sites on calendars): a calendar's weekend days and a weekday test written as words, not numbers | RFC 5545 `BYDAY` (`MO` … `SU`); `weekday(d)` stays ISO 8601's 1 to 7 | numbers 1 to 7 in a calendar's `weekends` (unreadable, and Sunday is 0 or 7 by convention) | contextual words +7 (words of one construct, a calendar value) | L §2.2, §4.2 |
| A-4 | The quarter literal `2026-Q1` to `2026-Q4` | round 12 (T-1): quarters printed with no written form; a quarter index's key had to be written as a span | SDMX's quarter notation (ISO 8601 has none) | writing a quarter as `2026-01 .. 2026-03` | none (a literal form, no word) | L §2.1, §2.4 |
| A-5 | `bond30_360` beside `us30_360` and `e30_360`, with every day count's formula stated | round 12b (T-OPEN's G2 item: 30/360 had two conventions with users) | ISDA 2006 4.16(f) (30/360, no February rule) as `bond30_360`; the SIA rule with its end-of-month February rules as `us30_360`; 4.16(g) as `e30_360`; 30E/360 ISDA, which reads a termination date, written as a def | one `30_360` meaning one of them in silence | contextual words +1, built-in codes +1 | L §3.3 `dcf` |
| A-6 | An adjustment's name, `adjustments[m[keys].leg, V]`, and its attribute `of_leg` | round 12b (T-89): an observed row could not name the adjustment it settles | the adjustment is keyed by what it adjusts and the version that made it, so the name is stable across runs | an anonymous adjustment found by its `adjusts` only | contextual words +1 (`of_leg`) | L §5.14.3, E §5 step 4 |

### Constructs and meanings

| A | Added or decided after G1 | Needed by | Form and source | Chosen over | Home |
|---|---|---|---|---|---|
| A-7 | A scenario row may be an expression over the base's facts, the base's items named with the base's id as a prefix | round 7 (the migrated scenario models; D-10's overlays could only state literals) | a stipulation computed from the base, as a spreadsheet's what-if cell | restating each scenario value by hand, wrong once the base changes | L §6.4 |
| A-8 | A move's `on` may be `none`, and a binding whose day is a hole binds nothing where no thing could be alive on its possible days | round 8 (the household's change 3; a binding over `adjustments` before any exists) | three-valued membership: a hole only where the answer could differ | a hole for every binding with a hole day, which made every later forecast a hole | L §5.7, §5.14 |
| A-9 | A scenario's moves book nothing before its `from`; a move whose window or a leg's own day starts before `from` is left out | rounds 8 and 9 (the scenario models) | a scenario is a change from `from` on (D-10), so it can't book the past | booking scenario movements before `from`, which rewrote the base's history | L §6.4 |
| A-10 | `explain low(x, span)` without `from` explains the first day the lowest balance is reached, as a balance | rounds 8 and 9 | `low` is a `min` over days (L §5.14.1), so its witness is the first day reaching it | refusing `explain low` without `from` | L §5.18 |
| A-11 | A priced place whose price is a reading names its carry: `priced last(val) within valuation_age`, on the kind, the place or the entry | round 10 (S:201, P:ASK-02) | the carry rule of every other reading (D-3, L §4.6), now also in `price` | a valuation read with no carry, refused in every `value` | L §2.3 `price`, §5.10 |
| A-12 | A `same` line's sides are references (`sref`): a thing, a row or a move instance of the joint model or a member, with literal keys; a joint binding over a prelude kind ranges over every member's things under their prefixes; a member's own kind stays its own unless a `same` line joins it | round 11 (S:221, S:222, P:PTY-05) | identities written once, as in record linkage; the prelude is shared, so its kinds are one across members (kinds-modules prior art) | `same` over whole things only, and members' kinds never joined | L §2.3 `sref`, §6.1, E §3.1 step 1 |
| A-13 | Rows "at the same keys" are rows that state their keys alike; among the rest the narrower row wins by containment, key position by key position (`[2026]` beats `[2025..2029]`); neither narrower is still a tie (`LOAD-OVERLAP`) | round 11b (S:243) | most-specific match, as longest-prefix routing; no loading text changed its answer, since such rows tied and were refused before | "the row that states more keys" | L §4.6 steps 2 and 3.2 |
| A-14 | A scenario's own input may govern `always`; a base item evaluated in a scenario reaches the scenario's things through bindings over their kind (reads are by name); pins formed per binding tuple print once, with their binding | round 11b (S:242, S:244, S:241) | a scenario is ordinary versions (D-10), so its rows take every span a version's rows take | scenario rows limited to spans from `from` | L §6.4, §8.2 `pinned:` |
| A-15 | A defeater: a rule with `beats` and no value, which defeats the rules it beats when it applies and is never a candidate | round 13 (S:262, P:LAW-12: an exception to an exception that restores no value of its own) | the defeaters of Nute's defeasible logic (1994) | writing the restored value again in the defeater, a second home for one fact | L §2.3 `rulebody`, §5.11 steps 4–5 |
| A-16 | L §6.3's load conflict follows the read rule, so two same-day rows that precedence orders load | round 13 (S:261) | one rule for "which row wins" at load and at read | a separate load-time test that refused rows the read rule orders | L §6.3 item 2 |
| A-17 | A scenario row's `replace` replaces the base row at the same keys whatever the two rows' `known` dates | round 13b (S:282, same-day stipulations) | a scenario is a stipulation, not news | refusing a scenario row known before the base row it replaces | L §4.6 step 2, §6.4, §8.3 `LOAD-REPLACE` |
| A-18 | A version that changes a booked leg's places makes an adjustment even when the amount is unchanged | round 12b (T-OPEN) | never wrong in silence: a moved booking is a change | a place change with δ = 0 passing without a trace | L §5.14.3 |
| A-23 | A `same` line joins two of one sort: two kinds declared alike, two things of one base kind (places also of one unit and one price input), two rows of one type, or two movements; any other pair is `LOAD-SAME`; a `same` in a version joins only rows and movements; `same` is an equivalence, so lines that share a side make one group | round 14 (R6's contradictions 1 and 2; U-1, U-2, U-15 to U-25) | record linkage's identity classes; a kind joined only when declared alike, as module type sharing (kinds-modules prior art) | kinds never joined, and only places of two kinds refused | L §5.1, §6.1, E §3.1 step 1 |
| A-24 | A group of movements stated `same` is one movement, booked once at a place two of them touch, at the latest member's point in the day; where they disagree there it is a hole movement (`HOLE-SAME`); a joined thing's entries are read together | round 14 (U-30, U-31, U-36, U-37) | one event with two records, as in reconciliation; a decision's amount is known only at its point | the left side counting, refcore's stopgap | L §6.1 |
| A-25 | The joint model writes a member's rows only at a pair stated `same`; a scenario over a joint model may replace members' rows; currencies are one by code across members | round 14 (U-23, U-33, U-35) | an import is closed (D-9), so only an identity the joint model states is its to settle | a joint model overriding a member's facts in silence | L §6.1 |
| A-26 | A band is called with its prefixes: `R.b(x)`, `a.b(x)`, `household.b(x)` | round 14 (R6's contradiction 4: expression rows and members read bands by prefix, which the grammar couldn't write) | qualified names, as every module system | a band reachable only inside its own regime or model | L §2.3 `call` |
| A-27 | A member's pin is the sha256 of its canonical text, files taken in path order, written as 64 lower-case hex digits; a member that is a joint model pins its own members through its `use` lines | round 14 (R6's contradiction 3; U-3, U-27, U-34) | a semantic hash, as Dhall's of a normal form (D-9) | a hash of the files' bytes, which spacing changes | L §6.1, E §3.1 step 1 |

### Judgment calls

| A | Decided | Needed by | Why this reading | Home |
|---|---|---|---|---|
| A-19 | An ask line that reads a law-dated input or band directly states `law`; otherwise `LOAD-LAWDATE`, a load refusal of the whole reality (G-LAW) | round 12 | the explicit principle (a missing statement is refused); a def already had to state `at e` or `at reader` | L §4.3 |
| A-20 | A thing entered in two plain kinds has as its base kind the one entered first by `known`, then by lifespan start; a tie on both is `LOAD-DUP` (T-46) | round 12 | a thing has one base kind (D-1), so the order of entry decides it, and a tie that order can't break is refused | L §5.7 |
| A-21 | A negative total under `rest largest` rounds each share toward zero and gives the leftover units, of the total's sign, to the largest absolute remainders (T-64) | round 12 | Hamilton's largest-remainder method made symmetric in sign, so the split of -x is the negative of the split of x | L §5.13 |
| A-22 | The last day of a period `p` is `end_of(p) - 1 day` | round 13b (S:281: the `schedule` row contradicted `end_of`) | `end_of` is the first instant after p (half-open spans), so the last day is the day before | L §4.2 |
| A-28 | `first` and `last` take no term (`LOAD-PARSE`), now in the grammar | round 14 (R6's contradiction 9) | the prose's rule, which the grammar contradicted; chosen over a grammar wider than the language | L §2.3 `agg`, §5.11 |
| A-29 | A movement's built-in field `kind` is `text`, the id its move or observed row names; membership is tested with `x in K` | round 14 (R6's contradiction 10) | as `meta`'s `version` and `source` are text (round 12); chosen over a key of a kind index, which would need one | L §2.5 |
| A-30 | An input keyed by one period index isn't law-dated: it is read at its key's period | round 14 (R6's contradiction 7, against G-LAW) | L §4.6 step 1 already reads it there; chosen over requiring `law` on an ask line that reads a yearly input at its year | L §4.3 rule 1 |
| A-31 | A key at a stepped-range position is `LOAD-KEY` only where the range's ends and step are literals; otherwise `RUN-KEY` where it is read | round 14 (R6's contradiction 8) | load refuses only what load can see; chosen over a load refusal raised during a run | L §2.5 |
| A-32 | A `windows` clause also chooses the day of an `added` calendar window; without one the result is `HOLE-CALENDAR` | round 14 (R6's contradiction 6) | one rule for every window a line meets (L §4.4); chosen over a calendar window outside the `windows` clause | L §4.2, §8.1 |
| A-33 | Each cell of a walk-bound def is explained as a def cell, its running reads as balances up to its point | round 14 (R6's contradiction 5) | explainable: every value has its derivation; chosen over a list of points and values without their reasons | L §5.18, E §10 |
| A-34 | A place's price input is named once, on its kind or on the place, never both (`LOAD-PRICE`) | round 14 (U-14) | one home per fact; chosen over "the narrower wins", a second precedence rule for one case | L §5.10 |
| A-35 | `unit(q)` of a quantity (a leg's amount included) as a key of the unit kind that holds it; a quantity of a unit kind whose key load can't see has the kind as its load unit, its key checked per value at run (`RUN-DOMAIN`) | round 15 (S:302, D-1/SF-7's promise, L §5.17's example refused otherwise) | completes a built-in D-1 named; units of measure with a dimension per unit kind (Kennedy), the key checked per value as a money object checks its currency (JSR 354); chosen over a share-of-holding workaround | L §3.2, §5.10, §5.17 |
| A-36 | The base loads without its scenarios: a base item naming a scenario's item, a rule's `v` naming a scenario version included, is `LOAD-SCENARIO`; an act not yet law brings its rules in the scenario's own version | round 15 (F3; contradiction between L §6.4 and §6.1/§5.3) | one rule, the base is whole alone; **a change of meaning from 1.0.1** (tally 1's N-931/L-24 allowed the pointing), on the G2 list | L §6.4 |
| A-37 | `covered(x, span)` has a production of its own, as `flows` has; the grammar's `field` may hold a `dayset` | round 15 (contradictions: prose stated forms the grammar couldn't parse) | the grammar made to say what the prose already said; chosen over a span argument for every call | L §2.3, §5.19, §3.1, §4.4 |
| A-38 | A change of a leg's places: a settlement moving the booked amount from the old places to the new counts 0, a second payment into the new place counts in full, a leg turned round is settled only by a waiver | round 15 (U-44, R7-1: L §5.14.3 against E §5 step 5) | E §5's signs applied to both sets of places; off the G2 list | L §5.14.3, E §5 step 5 |
| A-39 | A move's `at` dates its keys' `where`, read with `on` at the ask's cut; `at e` reads nothing law-dated; a key binding's `where` is read at its item's law date | round 15 (S:308, the §4.3 site) | one law date per item, as L §4.3 gives defs | L §4.3, §5.14, E §4.1 |
| A-40 | A leg whose place is an expression changes the place it gives; the walk orders a day's instances by the places given; `RUN-ORDER` only between instances that read what each other changes | round 15 (S:307) | dependency order on the actual graph of the day, the static graph only bounding it; chosen over refusing every static cycle | L §5.14.2, E §3.1 step 8, §4.2 step 3 |
| A-41 | Band `replace` rows patch the table in force on each day of their span; on a day with no table, a whole table stands and a patch is `LOAD-REPLACE` | round 15 (S:305) | as an input's `replace` row on days no earlier row governs | L §5.9 |
| A-42 | `mean` of a distribution value is refused (`LOAD-RANDOM`); a mean is asked of a random fact | round 15 (R7-7) | the four functions of a distribution value stay four; chosen over widening `mean` | L §5.16 |
| A-43 | A place's sign rule (`min 0`, `max 0`) is checked after the day's unordered batch (start readings, observations and plans together), then after each decision and reading as it is booked, then at the close; a break is `RUN-PLACE` with the batch's or the decision's legs | round 16 (S:321; contradiction: L §5.14.2 gives the day's observations and plans no order, E §4.1/§4.2 step 5 checked each leg in move-id order) | a deferred constraint, checked at the end of a batch whose parts have no order (ISO SQL `DEFERRABLE INITIALLY DEFERRED`); gives up intraday bounds between unordered postings | L §5.10, §5.14.2, §8.4, E §4.1, §4.2 step 5 |
| A-44 | A plan or decision instance that an observation seen at the cut fulfils is never booked, on any day: its observations stand in for it where they fall, and it is kept for the ledger as the modelled past | round 16 (S:322, O:B-7; contradiction: L §5.19's bind and `flows` count it once, its booking rule booked it twice) | one count per movement, as on a covered day; an instance from K on that is paid early is not booked either | L §5.19, §5.14.3, E §4.1 step 3, §4.2 step 3, §6.1–6.3 |
| A-45 | The two rows of one transfer on two statements are one movement by a written `same r1 = r2` in a version: each end booked once, bound once, counted once by `observed` and `flows`; rows that disagree print `WARN-LEGS` and a read away from their places is `HOLE-SAME`; an unjoined pair of one transfer that fulfils one instance is `RUN-RECORD` | round 16 (S:323; contradiction: L §5.3's identities pointed at §5.19, which never said what they do; "books each end once" against `observed`'s sum of both) | record linkage's identity classes, as a joint model's `same` (A-23); chosen over inferring the pair from a shared `fulfils`, since the parts of one payment share one too | L §5.3, §5.19, §8.1, §8.4, §8.5, E §6.1 |
| A-46 | Two values of one period type (`month`, `quarter`, `year`), keys or not, differ by an int number of periods | round 16 (S:325) | the key rule of L §4.2 applied to every period value; the snapshot's `LOAD-UNIT` not adopted | L §4.2 |
| A-47 | An ask is checked and answered only in the worlds its line is evaluated in; a base file's ask that names none is in the model's world; randomness belongs to a world. Asks are the exception to A-36: an ask that names a scenario loads only with that scenario's file | round 16 (W-B-1; the triage's contradiction 7 between A-36 and L §5.18, §6.4, settled by the lead, R16-36) | a scenario changes nothing in the base (L §6.4), and nothing in the model reads an ask | L §5.18, §6.4, E §3.1 steps 10–11 |
| A-48 | A scenario's entries, new or replacing a base thing's, may change who is alive only from `from` (`LOAD-SCENARIO` otherwise); a replaced lifespan replaces the old one whole; what may differ before D is every read of the scenario's facts (projections, reads at a law date from D, `lifespan(k)`, bindings over spans reaching D) | round 16 (I-7; contradictions: L §6.4 and E §9 invariant 3 against L §4.3; L §5.3's "from this version's `governs`" for a version of entries) | "changing nothing before it" applied to lifespans as to rows | L §6.4, §5.3, §8.3, E §9 |
| A-49 | A month-day is no value: it stands only in a calendar's `yearly` list and an anchored index's `from`; `day_of(y, md)` and `schedule`'s month-day argument are removed; a day a source fixes in every year is two int rows (*reversed at G2: see GD-2*) | round 16 (finding 10, household A's writer; contradiction: L §4.2 against §3.1 and §7.1) | the spec made to say what its types allow; **removes a form** (`day_of`'s second form, added after the first blind modeller's `day_of(y + 1, 04-30)` (hardset R27) but unreachable under §7.1; no carried model, household or amendment uses it); the month-day value stays a pressure line for the review (Q:A-04, I-7, I-9) | L §4.2, §3.1, §7.1 |
| A-50 | `held` prints once per row, the periods it was read for merged into maximal runs | round 16 (household A's judge, A-02's 214 lines per row) | as `assumed` lists its keys once | L §8.2 |
| A-51 | A calendar read (a business-day function, `schedule`) needs no `at`: it reads the row governing the day it asks about | round 16 (household A's judge) | L §4.3 rule 1 now says what L §4.2 (D-4) implied | L §4.3, §4.2 |

## The redesign review's verdicts (review 1, 2026-10-27)

Copied by script from `review/ledger.md`, columns 1 to 4. The test column holds the principle a rule is derived from (P-k, `review/principles.md`, now L §1) or, for a form that would be admitted, the four tests: (a) it can't be written with what exists, (b) at least 3 rows across 2 or more axes need it, (c) it has a named solved form, (d) it is no more general than its rows need. Each item's statement and the rows it leaves awkward are in the ledger. Changes after the ledger: `review/lead-decisions.md` (G2-4 code families whole; `random at` and `exponential` kept, since carried models use them) and `review/critic/lead-settlement.md` (CR-2's 17 findings). No form was admitted.

### Post-G1 records

| item | problem | verdict | from P-k / (a) / (b) / (c) / (d) |
|---|---|---|---|
| A-1 | RP-1 | derive | P-2: a kind is the set of its things and a subkind's things are its parent's, so "every place" and "every movement" are bindings over the kinds every place kind and every event kind are subkinds of; the plural names are C-21's style rule. |
| A-2 | RP-4 | derive | P-5: D-3 reads a reading's age (its `age(x[k], d)`); `age` already means completed years (L §4.2), so the reading's age takes its own name (R-once). |
| A-3 | RP-18 | derive | P-15: D-13's calendar lists the weekdays that are never business days, and its own example names them as words (`weekends [sat, sun]`); a weekday word is a code of that construct. |
| A-4 | RP-21 | derive | P-16: every value prints as its literal and every period type has one; `quarter_of` gives quarter values, so the quarter has its literal (SDMX's, since ISO 8601 writes none). |
| A-5 | RP-17 | drop | No text uses `bond30_360`, `us30_360` or `e30_360`, nor `act360` or `actact` (W-3); a later version may add a code (L §9). |
| A-6 | RP-10 | derive | P-8 with P-17's stable ids: a settlement states `fulfils` the change it settles, so the change is a thing with a stable id; it is one per changed leg, since a movement's legs may be in different units (P-3), and keyed by the version that made it, so the name holds across runs. |
| A-7 | RP-13 | derive | P-12: D-10 states it ("a scenario row may be an expression over base facts (`base.x`), so a linked override isn't restated"); the base is named by its id. |
| A-8 | RP-5 | derive | P-9: `none` is "nothing here", so a day that doesn't exist (a schedule's 31 February) gives no instance, as a false `when` does; a hole stops spreading where the outcome is decided without it, so a binding whose day is a hole is a hole only where some thing could be alive on a possible day. |
| A-9 | RP-13 | derive | P-12: a branch changes nothing before its divergence day, so a move it declares books nothing before it. |
| A-10 | RP-22 | derive | P-17: `low` is a `min` over days (L §5.14.1), and an aggregate's derivation is its winning term in binding order, so the witness of a `min` over days is the first day that reaches it. |
| A-11 | RP-4 | derive | P-5 with P-3: a reading is read only through the carry written where it is named; a priced place names its price on its kind, place or entry, so the carry is written there. |
| A-12 | RP-12 | derive | P-11: an identity joins two things, rows or movements, which may be keyed instances, so its sides are references with literal keys; consolidation needs the joint model's own bindings over a shared kind to range over every member's things; a member's own kind stays its own unless a `same` line joins it. With OQ-5: only the joint model's own items range across members. |
| A-13 | RP-2 | derive | P-5: which row a read takes is written; tally 1's precedence "a specific row beats a default row" is set containment, applied key position by key position, and two rows neither of which contains the other tie, as before. |
| A-14 | RP-13, RP-22 | derive | P-12: a branch's versions are ordinary and its own input is read only inside it, so its rows may govern `always` without changing anything before `from`; P-2: reads are by name and by kind, so a base item reaches the branch's things through its bindings; P-17/Q-3: a register entry is listed once, with its binding. |
| A-15 | RP-16 | drop | (a) fails: the lifting condition written in the excepted rule's `when` is right, and it is the fix D-12's own example prescribes; (b) fails: two rows of one probe, one axis (`P:LAW-12`, S:262, G4); (c) passes: Nute's defeaters; (d) passes: one rule form. |
| A-16 | RP-2 | derive | P-5 with R-once: which row wins is one rule, so load refuses only what that rule leaves tied. |
| A-17 | RP-2 | derive | P-12: a branch stands above its base (tally 1's layer precedence, kept), and a stipulation isn't news; so a branch's `replace` replaces the base row at those keys whatever the two rows' `known`. |
| A-18 | RP-10 | derive | P-8 with Q-1: a retroactive change is a readable difference, and moving a booked leg to other places is a change though its amount stays. |
| A-19 | RP-7 | derive | P-6: every read of a law-dated fact ends a chain at a written date, and an ask line that reads one directly is that read's reader. |
| A-20 | RP-1 | derive | P-2: a thing's base kind is the kind it entered at; with several entries into plain kinds the first (by `known`, then by lifespan start) is where it entered, and a tie can't say which, so it is refused (`LOAD-DUP`). |
| A-21 | RP-17 | derive | Q-4 with the kept `prorata`: Hamilton's largest-remainder method works on magnitudes, so the split of −x is the negative of the split of x. |
| A-22 | RP-18 | derive | P-15: spans are half-open over periods and `end_of(p)` is the first day after p (kept from tally 1), so the last day of p is `end_of(p) - 1 day`. |
| A-23 | RP-12 | derive | P-11: an identity joins two of one sort (D-9: two things of one kind; SF-25: two rows or movements; two kinds declared alike, as module type sharing); an identity isn't dated, so a version joins only rows and movements; identities sharing a side form one class. |
| A-24 | RP-9 | derive | P-11: facts of things stated `same` are read together and a disagreement is a hole (C-26), and a movement listed twice counts once (SF-25); P-7: a decision's amount is known only at its point, so a group is booked at its latest member's point. |
| A-25 | RP-12 | derive | P-11: an import is closed, so the joint model writes a member's row only to settle a disagreement at a pair stated `same` (C-26); P-12: a branch over a joint model may replace members' rows, since a branch changes nothing in its base; currencies are one by code, as the prelude is one. |
| A-26 | RP-21 | derive | P-16 (the grammar says what the prose says) with P-4, P-11, P-12: an item outside its namespace is named with its prefix, and a band is an item. |
| A-27 | RP-12 | derive | P-11: D-9's pin is a semantic hash of a normal form (Dhall), so it hashes the canonical text, files in path order, and a member that is a joint model pins its own members. |
| A-28 | RP-21 | derive | P-16: value first applies to constructs that give a value; `first` and `last` give a key, a bare binding, which keeps its form (D-14). |
| A-29 | RP-1 | drop | R-once: D-1 merged `kind(x)` into `x in K`; a readable `kind` field is a second form for membership; no text reads `.kind`. |
| A-30 | RP-7 | derive (G2-1) | P-5: for an input keyed by one period index, the key is the value's valid time, so a read takes the row whose span contains the key's period, at any law date, and needs no law date (P-6 binds only law-dated reads). This rewords C-28's sentence in D-3 ("the reader's law date picks, among the rows for that key, the one that governs"): G2-1. |
| A-31 | RP-17 | derive | R-load: load refuses only what it can see; a key at a stepped range whose ends and step are literals is checked at load, any other per value where it is read. |
| A-32 | RP-18 | derive (restated) | P-9: one rule for every window a line meets (D-7, SF-6), so an `added` calendar day that is a window is chosen by the line's `windows` clause; R-once: its hole is the window's, `HOLE-IMPRECISE`, not a calendar code. |
| A-33 | RP-22 | derive | P-17 with P-7: a walk-bound def has one cell per walk point (D-5), and every value has its derivation, so each cell is explained as a def cell at its point. |
| A-34 | RP-4 | derive | R-once: one home per fact, so a place's price input is named on its kind or on the place, never both. |
| A-35 | RP-11 | derive | P-3: D-1 values each position at its own unit's price (`num CUR/u`, keyed by unit) and gives `unit(q)` of a quantity (SF-7); a quantity of a unit kind whose key load can't see carries the kind at load and its key per value (Kennedy's dimension per unit kind; the per-value currency check of JSR 354). |
| A-36 | RP-13 | derive | P-12: a branch changes nothing in its base, so the base loads and answers without any scenario file, and an act not yet law is a branch's rule. A change of meaning from tally 1 (N-931/L-24), listed for B3 in `MIGRATION.md`. |
| A-37 | RP-21 | derive | P-16: the prose let a record field hold a window and `covered` take a span, so the grammar does. |
| A-38 | RP-10 | derive | P-8 (with A-18): a settlement leg counts by the way it runs against the changed leg's places before and after the change, E §5's signs on both sets. |
| A-39 | RP-7 | derive | P-6 and P-8: a movement's law date is its `at`, which dates everything the move reads (its keys' `where`, `on`, `when`, legs, attributes); the `at` ends every chain, so it reads nothing law-dated itself; which tuples are instances is read at the ask's cut, their amounts at the booking cut. |
| A-40 | RP-8 | derive | P-7: the day's order is its dependency graph; a leg whose place is computed changes the place it gives, so the walk orders a day's instances by the places they touch, the static graph only bounding it. |
| A-41 | RP-2 | derive | P-14 (a replacement is a new version of one row) with P-5: a band's `replace` row patches the table in force on each day of its span, and where no table is in force only a whole table can stand. |
| A-42 | RP-15 | derive | P-13: `mean` is a query over a random value (D-11); the functions of a distribution value are `cdf`, `pdf`, `pmf` and `quantile`; one name, one meaning (R-once). |
| A-43 | RP-8 | derive | P-7: observations and plans on one day have no order among them, so a check between them would invent one; the sign rule is checked after their batch, after each decision and reading, and at the close (a deferred constraint, ISO SQL). |
| A-44 | RP-8 | derive | P-10 and P-8: an observation realises the planned instance it fulfils, so the instance is counted once, by its observations, wherever they fall, and is kept as modelled past. |
| A-45 | RP-9 | derive | P-11 and P-10: rows stated `same` are one movement (SF-25); each end is booked from the statement that covers it; a disagreement is a hole naming both rows, with `WARN-LEGS`; two unjoined rows of one transfer that both fulfil one instance would count it twice, so they are refused (`RUN-RECORD`). |
| A-46 | RP-21 | derive | P-16: `month`, `quarter` and `year` are one dimension of counted periods (D-14), so two values of one period type differ by an int of periods, as two keys do. |
| A-47 | RP-13 | derive | P-12: a branch changes nothing in its base, and nothing reads an ask, so an ask is checked only in the worlds its line is evaluated in, and one that names a scenario loads only with it. |
| A-48 | RP-13 | derive | P-12 with P-2: a lifespan is a fact, so a branch may change who is alive only from its divergence day, and a replaced lifespan replaces the old one whole (D-12's `replace K + [k] lives span`). |
| A-49 | RP-19 | derive | P-16 with L §3.1's types: a month-day literal stands only where a literal does and no type holds one, so `day_of(y, md)` was unreachable; the form goes. *Reversed at G2: see GD-2.* |
| A-50 | RP-22 | derive | P-17/Q-3: the register lists a fact once, its keys or periods merged, as `assumed` does. |
| A-51 | RP-7 | derive | P-6 (D-4: calendar reads read the calendar at the day asked): a business-day function or `schedule` reads the calendar row governing the day it asks about, so it needs no `at`. |

### Pressure lines

| item | problem | verdict | from P-k / (a) / (b) / (c) / (d) |
|---|---|---|---|
| PL-1 | RP-21 | derive | P-16 (A-37): the prose let a record field hold a window, so the grammar does. |
| PL-2 | RP-11 | derive | P-3 (A-35): a position is valued at its own unit's price, and `unit(q)` gives a quantity's unit, checked per value. |
| PL-3 | RP-14 | drop | (a) fails: a difference by cause written as defs gives the counterfactual sums, and is right (round 15 accepted it); (b) passes with RP-14's rows (three rows, three axes); (c) passes: Fowler's parallel model with a branch point; (d) not reached. |
| PL-4 | RP-6 | drop | (a) fails: the known part is printed by pinning the unknown to a value that zeroes its term, the pin printed beside the result; (b) fails: two rows (`Q:A-11`, `Q:E1-02`), two axes; (c) passes: c-tables, Hazel's guarded indeterminate forms; (d) not reached. |
| PL-5 | RP-21 | derive | P-16: `covered` takes a span (A-37), and a sum over a key range is an aggregate over a moving window. |
| PL-6 | RP-18 | derive | P-15 with P-9: a calendar's `added` list takes a window, `one_of([…])`, so one day open among settled ones is said. |
| PL-7 | RP-11 | derive | P-4: a split of a place by the origin of what flowed in is a regime's lots (D-1, C-16), or a def over the flows in by origin (L §3.6), as round 15 accepted. |
| PL-8 | RP-11 | derive | P-15 with P-10 (SF-9: the run bounds booking, not facts): history before the run's start is given by observed rows, which bindings and fills see and nothing books, and the opening reading at the start anchors each balance. |
| PL-9 | RP-23 | drop | Not a language matter: D-8 puts the importer outside the language (it proposes rows and links; the engine never infers one), and L §10 keeps file input out. |
| PL-10 | RP-5 | derive | P-5 with X and P-15: a calendar past its coverage is a forecast, so its years are rows marked `assumed`, replaced by the published row when it is known. The candidate, a hole date carrying its interval (Cousot and Cousot 1977): (a) fails, the assumed row is right (household A's own forecast does it for 2027–2035); (b) passes (five rows, A and D); (c) passes; (d) not reached. |
| PL-11 | RP-20 | derive | P-17 with the key rule (A-31): a def keyed by an index of days (the built-in `days`, or a stepped range of days) is a function of a day, read at computed days; a day load can't see is checked per value. |
| PL-12 | RP-19 | drop | The month-day row value: (a) fails, two int rows and `day_of(month_of(start_of(y)) + (month - 1), day)` give the right day in every row that uses them; (b) passes (seven rows, three axes, the first blind modeller's included); (c) passes: RFC 5545's `BYMONTH`/`BYMONTHDAY`, FpML's fixed day; (d) not reached. The strongest rejected candidate. *Adopted at G2: see GD-2.* |
| PL-13 | RP-13 | derive | P-12 with P-2 (A-48): a lifespan is a fact; a branch changes who is alive only from its divergence day, and a replaced lifespan replaces the old one whole. |
| PL-14 | RP-10 | drop | Pooled settlement: (a) fails, a fill whose order the source leaves open is written with that order as an axis or `assumed` (L §10: a total the law defines without its split); (b) fails, one row (I-8); (c), (d) not reached. |
| PL-15 | RP-11 | derive | P-15 with P-10 (as PL-8) and P-4: lots held before the run are observed acquisitions dated before the start, each with its value and day, which a fill binds and nothing books; the opening reading anchors their sum. |
| PL-16 | RP-13 | derive | P-12: sibling branches never meet (each world is its base plus its own versions), so each may declare an id the other declares; P-6: a branch row over a base fact that governs `always` dates it from `from`, so its readers in that world state `at` (A-14 lets a branch's own input govern `always`). |
| PL-17 | RP-17 | derive | R-load with P-14 (A-31): a code is a value of its code type, which a row can hold; what load can't see is checked per value where it is read. |
| PL-18 | RP-1 | derive | P-2: membership is `o in K` (D-1 merged `kind()` into it), for observed rows as for things. |
| PL-19 | RP-20 | derive | P-17 (an inverse is a search over stated candidates, its derivation the winning one): `min(v for v in a .. b step s where …)`, each candidate read with an as-if pin, is the form; its cost is speed, not meaning (no speed work in design). |
| PL-20 | RP-14 | drop | (a) fails: the swapped rule's chain beside the real one is right (every figure matches); (b) passes with RP-14's rows; (c) passes (as PL-3); (d) not reached. |
| PL-21 | RP-15 | derive | P-13: a statistic over paths is a question in an ask (D-11), never an input of a rule of the same run; a rule that depends on one is two worlds compared. |
| PL-22 | RP-15 | derive | P-13 with P-2: keys aren't made by sampling (a kind's things are declared), so a random count of items is slots over a bounded index, each live while its slot is at most the draw. |
| PL-23 | RP-7, RP-13 | derive | P-12 with P-6: a branch's own input may govern `always` (A-14), so a what-if value read by the branch's own rules is no law-dated read; only a branch row over a base fact dates that fact's readers. |
| PL-24 | RP-22 | drop | Lines ordered by a computed value: (a) fails, a rank computed by counting the keys with a larger value, ties by list position, is right; (b) fails, one row; (c) passes (`ORDER BY`); (d) not reached. |
| PL-25 | RP-19 | drop | As PL-12. |
| PL-26 | RP-3 | derive | P-14: a rule that cites a version with `v` exists only where that version governs and is known (closed by PL-32). |
| PL-27 | RP-19 | drop | As PL-12. |
| PL-28 | RP-23 | drop | As PL-9. |
| PL-29 | RP-3 | derive | P-14 with P-6 and P-9 (A-8): a series that exists only from a version is a move whose keys' `where` reads a def whose winning rule cites that version with `v`; before the version the series has no instance, so its day is never read and spoils no span. |
| PL-30 | RP-15 | derive | P-13 with P-17: paths are the engine's own seeded stream (D-11), so a foreign program's stream is no row a text can give; the step is a hole stated with its reason. |
| PL-31 | RP-5 | derive | As PL-10. |
| PL-32 | RP-3 | derive | P-14: the closing note of PL-26. |
| PL-33 | RP-12 | derive | P-11 with P-2: what two realities name as one entity has identity, so it is a thing of a kind in each, and `same` joins things; an index key is no thing and isn't joined. |
| PL-34 | RP-6 | drop | As PL-4. |

### Open questions

| item | problem | verdict | from P-k / (a) / (b) / (c) / (d) |
|---|---|---|---|
| OQ-1 | RP-8 | derive | P-10 and P-8 (A-44): an instance an observation fulfils is counted by its observations and is never measured; P-7: a link reads only what it names, the instance's existence (its keys and `where`), never its amounts. So the link is judged at the cut that reads it, the ask's. *Changed at G2: see GD-16.* |
| OQ-2 | RP-9 | derive | P-2 with P-11: one movement has one base kind (the kind it entered at), so its kind is one of the facts `same` joins, and two rows stated `same` with different kinds disagree. |
| OQ-3 | RP-9 | derive | P-11 with P-10: an input's keys are its rows, so a binding over an observed input ranges over its rows, each once; a binding over a kind ranges over movements, and rows stated `same` are one movement, bound once; both rows stay listed as evidence (SF-25). |
| OQ-4 | RP-12, RP-21 | derive | P-16 with P-11: L §6.1 reads a member's keyed item and binds over a member's kind, and `sref` admits both, so `ref`, `pref` and `bind` admit them; P-12 with P-11: a pin sets only the world one ask reads (D-10) and changes no member's file, as a scenario over the joint model may (A-25). |
| OQ-5 | RP-12 | derive | P-11: a member is a closed module (imports are never changed; joining changes no member's answer, D-9), so a member's items read as `a.item` mean what they mean alone; only the joint model's own items range over every member's things (A-12). |
| OQ-6 | RP-21 | derive | P-16 (every value has its literal) with D-14's literal rule: a text value has a literal, so the grammar has a text atom; like any literal that states a fact, it stands in rows, pins and asks and never in a rule. |
| OQ-7 | RP-12 | derive | P-11 with OQ-5: a member's place means what it means alone, so a movement of the joint model's own versions changes it only where it is the member's own movement, linked by `same` or `fulfils`; any other is the joint model's, and a group total over that place shows it as a hole (SF-32). |
| OQ-8 | RP-22 | derive | P-17 with Q-3: the register lists an entry once, its keys or periods merged, as `assumed` and round 16's `held` do. |

### Words no text uses (RP-24)

| item | problem | verdict | from P-k / (a) / (b) / (c) / (d) |
|---|---|---|---|
| W-1 | RP-24 | drop (G2-2) | No text uses the keyword `pending`, models and amendments included; P-9: a binding empty at the cut whose span reaches past it is "not known yet", which is a hole, not a fourth kind of absence. |
| W-2 | RP-24 | drop (G2-2) | Ten built-in names no text calls: `first_business`, `earliest`, `in_force`, `meta`, `reasons`, `real`, `exponential`, `empirical` (no text at all), `latest` (spec examples only), `paths` (never called; `paths N seed S` stays). |
| W-3 | RP-24 | drop (G2-2, G2-4) | 22 contextual words no row or card uses: the day counts `act360 actact bond30_360 us30_360 e30_360` (A-5), the rolls `modified_following unadjusted`, the clause words `latest earliest` (windows), `random` (`random at`), `split days of` (law date `each day of … split by days`), `within`, the field `of_leg`, the calendar's `removed` and weekdays `tue wed thu fri`, and `meta held` (the fields of `meta`). Three more words no row uses stay, because a principle keeps them: the rounding mode `half-down` (`core.md` keeps `round` and its modes from tally 1) and the type words `daycount` and `roll` (P-14, D-12: a convention is a code a row can hold, never a default; PL-17). |
| W-4 | RP-24 | drop (G2-2) | Five built-in ids no text uses: `party_place match_days match_amount bank_party` (the importer's matching data; D-8 puts the importer outside the language) and `quarters`. Two ids no text names stay, because rows make their members: `pads` (FLOW-01's `pad wallet from spent` makes a movement of it) and `currencies` (every `unit X: currency` declares a key of it, and `unit(x)` of money is one, L §5.4, §5.10). |
| W-5 | RP-24 | keep (G2-3) | Thirteen words only the carried models use (`given`; `abs weekday band_hi gamma negbin pdf pmf crps`; `dates moves orient weight`); the 79 models are protected by B3. |

## The G2 decisions (session 4b, 2026-10-28)

The user's answers at G2 (`G2-list.md`), written into L and E by spec round 19 (`scratch/lead/round19/`: `regime.py`, then `answers.py`). One entry per decision that changes or confirms the definition; where an answer departs from a G1 record, the entry says so and the record carries a note. The labels G2-1 to G2-4 in the verdict tables above are `review/lead-decisions.md`'s; these are GD-1 to GD-28.

- **GD-1. `regime` is dropped from 2.0.** It adds no answer that top-level items can't give (the minimality bar's test (a)); its general parts live in L §5.10, §5.12 and §5.14, and L §5.17 is a stub that keeps the numbering. `export`, `LOAD-PRIVATE`, `R.x` names and E's refcount step go, and texts unbox their regimes (`migration.md`); it departs from D-2, whose regime was one unit that applies by kind.
- **GD-2. A month-day type,** `monthday`, written `MM-DD`, with no year (the user's a04, "restore"): a row or a pin may hold one, `day_of(y, md)` gives its date in year y, and a calendar's `yearly` entries are month-days (L §3.1, §4.2, §7.1). `02-29` is no month-day, and month-days compare by `==` and `!=` only. It reverses A-49 and PL-12's drop of the month-day row value.
- **GD-3. A `quantile` over paths prints its standard error,** by the Maritz–Jarrett method, correctly rounded to 1e-6, `none` with N = 1 (L §5.16, E §7), so a band figure from paths can be tested against a key. The keyed form over keys stays exact, with none.
- **GD-4. What load sees is one fixed list** (L §8.3, cited by E §3.1; `load-list.md`). A move's day and a leg's place are seen also where they are computed from literals and declared keys alone, through built-ins that read no fact (`q-load-value`), and the random cone runs through every place a random movement books into (`q-load-cone`).
- **GD-5. `flows` over a day before S is a hole** (`HOLE-PRERUN`; C1-9 = C2-10, side (a)): movements dated before S are seen by bindings and fills, never by `flows` (E §2, §4.1). It narrows D-13's "a read of one ranges over every stated movement" (SF-9).
- **GD-6. S is always the header's `start`** (C1-10 (a)): a request states asks, pins and each ask's K, and `REQ-CLOCK` loses its S case (E §1, §2, §11).
- **GD-7. `add_business` takes a negative n** (C1-21 (a)): `add_business(t, -2, len)` is the second business day before t, as L §4.2 states. It departs from D-13 and C-8, which settled n ≥ 1.
- **GD-8. A plain read of a reading at an instant with no reading is a hole at that key** (`HOLE-NOREADING`; C1-23 (a)), as L §4.6 states; there is no `LOAD-CARRY`. It departs from D-3's explicit form.
- **GD-9. `kind` sections print in tree order** (C1-31 (b)): each kind is followed by its subkinds, kinds with the same parent (or none) sorted by id (L §2.4).
- **GD-10. A reading `after m` with m in the day's first batch** (an observation or a plan) is refused at load (`LOAD-ORDER`; C2-3 (b)), since the batch has no order (L §4.6, §5.14.2; P-7).
- **GD-11. Plans always come first** (C2-4 (b)): `before m` orders a decision only against another decision, and with m a plan or an observation it is refused (`LOAD-ORDER`); the start-of-day balance is `bal(x, d start)` (L §5.14.2). It departs from D-5 and SF-15, which let `before m` put a decision ahead of a plan.
- **GD-12. C2-8 is closed by GD-1:** `LOAD-PRIVATE` went with `regime`.
- **GD-13. Day K is plan** (C2-19 (b)): observations dated K are unread at cut K, and E §2's evaluation window is [K, E]. It departs from D-6 ("Day K is past").
- **GD-14. A decision's day reads no running state** (C2-20 (b); `LOAD-READS`), as L §5.14.1 and E §3.1 step 7 state. It departs from D-5, whose `reads` named the places a decision's day reads.
- **GD-15. The prelude's `outside` is of unit `any`** (C2-21 (b)), as L §5.10 states. It departs from D-1, which made the prelude's place kinds each of unit `one`.
- **GD-16. A link's instance exists by its keys, `where` and `when`** (C2-23 (b)), as L §5.19 and E §4.1 state. OQ-1's parenthesis "(its keys and `where`)" is incomplete.
- **GD-17. Every movement that fulfils an adjustment is a settlement, never measured** (C2-26 (a)): L §5.14.3, §6.2 and E §5 step 2 say so in place of "a move that binds over `adjustments`".
- **GD-18. An ask that names a scenario whose file isn't loaded is refused alone** (C3-14 (a)): its `LOAD-SCENARIO` refuses only that ask, and the base answers (L §8, E §11).
- **GD-19. `WARN-UNREAD` stays "read by nothing"** (C3-19 (a)), one edge deep, as L §8.5 and E §3.1 step 6 state. It narrows D-15's lint ("inputs and defs that no ask, movement or check reaches").
- **GD-20. A key of a range `by n`, n above 1, is a period, never a law date** (refcore's contradiction 47, (b)): `at` such a key is `LOAD-LAWPERIOD` (L §4.2, §4.3).
- **GD-21. An `opening` reading at S start always anchors, whatever K** (48, (a); L §5.19, E §4.2 step 0).
- **GD-22. A law-date class may hold no row** (49, (a)): a stretch over which the rows and rules an item reaches don't change is one class even where no row covers it, so a window `at` inside it is read once and gives one hole, naming its first day inside the window (E §3.2, L §4.3).
- **GD-23. The load codes of what isn't defined** (refcore's mapping): an order on things is `LOAD-UNIT`, a built-in's argument outside its domain `LOAD-DOMAIN` (L §8.4, §8.3).
- **GD-24. The 13 silent places** (`refcore/UNSPECIFIED.md`): the canonical order settles rules of one label and v, a `same` item, `*` and ranges against other keys, and rows governing `always` or read `after m` (L §2.4); a rule's `v` naming no version is `LOAD-NAME` and a key its index doesn't hold `LOAD-KEY`; a period read across two keys is `RUN-DOMAIN` (L §4.6; at load where load sees it, GD-27); a band read with no law date that no row holds is the same hole without its date (L §5.9, §8.1); `discrete` over a key list or a stepped range is `LOAD-DOMAIN` (L §5.16); members copying different versions of a law part are `LOAD-SAME`, with no `same`-line exception (L §6.1); E §3.2 is GD-22.
- **GD-25. LC-7 and LC-8 are closed as done:** value first everywhere was adopted at G1 by D-14 and C-18.
- **GD-26. SF-34 and SF-35 are deferred to tally 3.0:** one merged dated list, and a per-arc cap in `fill`.
- **GD-27. A period read across two keys is refused at load where load sees it** (C4-9, the round's short contradiction read, `review/contra/verified-4.md`; the user, 2026-10-28): a literal period and declared keys are refused at load (`LOAD-KEY`), as a period that holds no key is; a period computed from a row is refused at that value (`RUN-DOMAIN`). R-load, against GD-24's "per value" read alone.
- **GD-28. The canonical order's last key** (the user, 2026-10-29): two rules or items that every key of L §2.4 leaves tied sort by their own canonical line, compared as ASCII. It closes the five §2.4 ties refcore refused (`REFCORE-UNSPECIFIED`), among them a `same` item beside a row of its target and keys, which blocked E1's canonical pins for households B and D. B's hand-written roll (`cases/households/B/flat.tally:221`, the 2027 Spring Day in Piotr's 2025 worth) goes to session 5's text changes, as the user chose.

---

## The tally 2.1 records (sessions 7 and 7b, 2026-10-30)

Session 7 designed tally 2.1 from session 6's decided items (ownership, batches, people as things, the joint cost, ranges as research, and four small items), and closed at gate G1, which the user answered whole. Session 7b wrote it into L and E. The records are `records.md` R-1 to R-17 in the round's files (`tmp/session7/`), with three tables of settled findings (the critic's C-1 to C-15, the modeller's M-1 to M-19, the critic's re-check RC-1 to RC-10); G1 is `gate.md` there, and the spec round's four picks are `tmp/session7b/picks.md`. Here each record is rendered short: what it decides, its solved form, and where it is written. In this part, L and E are tally 2.1's.

**The design in one paragraph.** A place kind may opt in to keeping its balance as **parts**, a map from a label to an amount: `owned` keys them by owner, a thing of the prelude kind `parties`; `batched` keys them by batch, the movement that brought the units in. Three rules move them: the **arrival rule**, a def the kind names, labels every arrival by its kind of event; the **leaving rule**, an `order` over parts optionally ending in `prorata`, says which parts an outflow takes; a **relabel** is a leg from a place to itself. Reads add `of q`. Unknowns stay holes, and places that don't opt in book as in 2.0.

### The records

- **R-1. Parts** (P-1). An opted-in place keeps its balance as label → amount per unit, the free semimodule over labels (Amsterdamer, Deutch and Tannen, PODS 2011), pooled cash, a lot inventory. Clauses `owned by D`, `batched`, `step u` and `takes`; every split is L §5.13's `rest largest` to u (C-2), each split with its own `ties by` (RC-2); a non-whole amount is `RUN-PRORATA`. Owned and batched kinds are `min 0` (G1 Q-3); `owns` on such a place, `same` on it (C-9), and a missing clause are `LOAD-PARTS`. A part labelled to a party outside its lifespan is `RUN-LIFESPAN`, a party's parts gone by its end, checked after the last day's whole order (C-10, RC-9). Rejected: the share table with a written fill (no reach, tracing re-done by every writer), sub-places per origin, taint labels. L §3.6, §5.7, §5.10.
- **R-2. Owners are things of the prelude kind `parties`** (P-9, item 5). Fowler's party; REA's economic agent. Decided at G1 Q-1 ("yes, built in"), as `accounts` is built in: the prelude is one in every member, so owners share one kind across a joint; two people are one only by `same` on the things. Subkinds (`persons`, couples, estates) are the model's own (pick 1). The record's pick, no prelude kind, was withdrawn at G1. L §3.5, §5.7, §6.1.
- **R-3. The arrival rule** (P-4). A def `D[m in events, p in parties]: num at m.day`, applied by the booking to every arrival at a thing of the kind: trust principal-and-income accounting's classification of receipts, provenance through an exchange, posting rules by event kind. Known shares sum to 1 (`RUN-LABEL`); hole shares leave one hole part of the remainder (C-6); a reachless arrival is `RUN-NO-RULE`, never a hole or a default. "Keep the labels" and "as paid" are one rule over `m.leg.amount of q`. It may read the start of the arrival's day, as a reader beside decisions (M-15, RC-6). Labels are read at the ask's cut K (G1 Q-4, below). Labels dropped at a place that keeps none are `WARN-LABELS` in the register (M-9, RC-8). Rejected: a single owner per arrival plus an "as paid" form; built-in keeping for transfers; an exchange inferred from leg directions. L §3.6, §5.14.1, §8.2, §8.5.
- **R-4. The leaving rule** (P-6). `takes k for w [v V] order … [prorata [ties by e]]`: the tracing rules and lot booking methods (priority, FIFO, pro rata, specific identification) as one ordered greedy allocation. A take is a fill over (part, outflow) arcs (C-1); ties between arcs that change a take are `RUN-TIE`, except arcs of one outflow under `prorata`; two outflows never share a part (RC-3). The order reads the outflow's fields after a kind test, so whose parts a leg takes is the order's, and a shortfall goes on down the order (RC-7). A hole term makes an undecided take that merges only what it could reach (C-4). In one batch, takes go in the order of the reads among them; a cycle breaks where it changes no take, else plans are `RUN-CYCLE` and observed rows hold a hole (C-11, RC-6). Rejected: a leaving def per outflow; method words (`fifo`, `hifo`, …); the leg restriction `from x of q`, added after the modeller and dropped after the re-check (RC-7). L §3.6, §5.12, §5.14, §5.14.2.
- **R-5. Relabelling** (P-8). A leg whose two ends are one place that keeps parts takes by the leaving rule and returns by the arrival rule, batches kept; it books nothing net. A homomorphism on labels; release from restriction. A division is per asset, by a share from the agreement (M-8); a relabel keeps the thing in its place (M-12); a drawn death in a forecast is a move keyed on the draw (RC-9). Rejected: `of p` on both leg ends; a round trip through a holding place; a `relabel` clause. L §5.14.
- **R-6. Readings** (P-3). `stated[x] of q` and `opening[x] of q` state a reading's parts, in the reading's own version (pick 2), at most its total; a pad is labelled by the arrival rule or refused (RC-4); a zero-gap anchor keeps the parts (C-5, M-4); a gap with no stated parts is a hole part or an undecided take; a stated part's difference is its own gap, listed (RC-5). Opening and balance-only reading parts are stated once, citing their document (G1 Q-5; bar B4 amended). Rejected: keeping proportions across an anchor; refusing anchors at owned places. L §5.19.
- **R-7. A gain follows the units** (P-5). Unitization: parts are kept in the place's unit, so a price rise is no event and nothing is labelled; a law that gives a gain elsewhere writes a relabel or a def. `owned` on an asset kind is `LOAD-PARTS`. L §5.10.
- **R-8. The unknown split is a hole part** (P-10). Symbolic provenance tokens: a hole part keeps its amount, its questions and its possible labels (RC-5); `of q` is a hole only for a q it may be (`HOLE-PART`); the total stays known; the only merge is an undecided take's. Its kept amount is the seam for ranges (item 4). L §8.1.
- **R-9. Net worth per party** (P-2). A def per model over `value … of p` and `owns`, each place counted by one of them; no built-in, since the rounding, price, places and depth are the law's or the house's. L §3.6.
- **R-10. Batches** (P-7, item 2). A batch is the movement that brought the units in; a transfer and a relabel keep it; the gain is written once over batches through `s.units.amount of b`. The leg read `m.leg.amount of q` is booking state: after the walk in a def or an ask, for its own movement in the arrival rule, else only in a decision that declares the place (M-15). "The day it arrived here" (o13) isn't writable and is on the pressure list. `fill` stays. L §3.6, §5.14, §5.14.1.
- **R-11. Notes after a one-line def and a place section** (item 7.1). A docstring; the canonical form stays 2.0's long form (C-12). L §2.3, §2.4.
- **R-12. A negative record field** (item 7.2). An interpreter fault: the grammar already allows it. No spec text.
- **R-13. What the 4 096-digit cap covers** (item 7.3). A value at a key; a path statistic's own accumulation is the engine's. L §7.1, §5.16.
- **R-14. A day key where a date is expected** (item 7.8a). Means `start_of(k)`; month and year keys still convert in writing. It fails test (a) on the letter and is kept by its spirit, as the user decided (G1 Q-6). L §3.1.
- **R-15. A joined household's cost** (decision 6). No language change: an engine requirement (`engine/BUILD-NOTES.md` § J), a load row and a probe card.
- **R-16. Unknowns with a range** (item 4). Research only (`research-ranges.md` in the round's files), presented at G2; R-8 keeps the slot.
- **R-17. A kind test narrows** (found by the modeller, M-3). Occurrence typing (Tobin-Hochstadt and Felleisen, POPL 2008): after `v in K` in a rule's `when` or an `if`'s condition, v's fields of K are readable; only a written test narrows, and `or` narrows to neither side. L §5.7.

### G1, the user's words (2026-10-30)

| Q | Decided | The user's words |
|---|---|---|
| Q-1 | owners are things of the prelude kind `parties` | "How is that different than requiring every single tally document for defining a word for account?" … "yes, built in" |
| Q-2 | both: A's withdrawal keyed by case, and a variant whose house rule is "in proportion" | "I'm mainly interested in examples of properly calculating some rule and not only about finding the unspecified one." … "in proportion" |
| Q-3 | owned and batched places are `min 0` in 2.1 | "You can have debt, but this is a separate account." … "yes, agree, and log it for later" |
| Q-4 | labels at the ask's cut, with one stated exception to decision 22 | "yes, A" |
| Q-5 | B4 applies to events; opening and balance-only reading parts stated once, cited | "yes, agree" |
| Q-6 | R-14 kept | "I agree, next" |

### The exception to decision 22 (RC-1, G1 Q-4)

2.0's decision 22 says a later version never re-measures a decision. Labels are walk state at cut K, as balances are, so a past decision that no observation fulfils and whose amount or `when` reads parts follows a later version that changes whose past money was, as it already follows a bank row learned late. Its law and other facts stay those known on its day; booked plans' and rows' amounts never change. The sentence is L §5.14.3's, "The one exception: parts" (pick 4).

### The spec round's picks (session 7b)

1. `persons`, couples and estates are the model's own subkinds of `parties` (R-2's pick).
2. A reading's parts stand in the version that states its total, at the same instant (M-17).
3. The parts clauses are kind lines; `of` binds by its read's production; `prorata` ends a `takes` clause as a clause word; a member's thing is `a.ania` (M-18).
4. Decision 22's exception is one nested bullet in L §5.14.3 (RC-1).

### Left out of 2.1

Signed parts (whose an overdraft is), `same` on a place that keeps parts, sharing among a batch's owners while refusing same-day lots (M-10), "the day it arrived here" (o13), lifespans ending on a drawn day, and the rest of the round's pressure list (`pressure.md`, PR-1 on). L §10 lists them.

---

# Part II. Tally 1.0.1's decisions, as released

Carried from `engine/spec/DECISIONS.md` at the tag `tally-1.0.1`, unchanged except that every heading is one level down. Its L, E, O, P and B are tally 1.0.1's documents.

## Tally: decisions

Every requirement of the spec (`tmp/engine-requirements.md`), every proposed change (`tmp/verification/changes.md`) and every fix the judge asked for (`judging.md` §8), each with what the design does about it and why. It also records the decisions the definition made on its own, with the alternatives weighed, and which construct each case needs.

**Statuses:**
- **taken**: Tally does what the ID asks, in the form it asks. "Taken (data)" marks law content: the constructs exist, and the law itself is data in a law part, written after the engine (principle 10).
- **merged**: the need is met by a more general construct than the one the ID names. The row says which.
- **rejected**: the design doesn't do it, on purpose. The row says why.
- **out of scope**: it belongs to a layer that isn't the engine: the MCP layer, the app, the import pipeline or the process. The row says which, and what the engine supplies to it.
- **taken differently**: the need is met, but not in the form the ID asks for. The row says how (CHG-01, CHG-04).

**References:** L = `LANGUAGE.md`, E = `ENGINE.md`, O = `OPERATIONS.md`, P = `PERFORMANCE.md`, B = `BOUNDS.md`. Requirement texts are shortened; the spec has them in full.

**Contents**
1. Design rules DR1–DR7
2. Spec requirements: M, T, U, R, P, L, S, Q, C, A (222)
3. Proposed changes CHG-01 to CHG-14
4. The judge's fixes (judging.md §8)
5. Decisions the definition made, with the alternatives weighed
6. Departures from the spec, and the questions they raise
7. Constructs and the cases that need them

---

### 1. Design rules

| Rule | Text | Status | How, and why |
|---|---|---|---|
| DR1 | Derive, don't declare; no fact stated twice | taken | One namespace, `LOAD-DUP` (L §2.5). Instances, lots, claims and movement kinds are computed, never stated (E §1). A default row covers only keys known when it was stated, so a new key isn't silently covered (L §4.6) |
| DR2 | Checks and joins evaluated across time | taken | Rules are evaluated at each law date (L §4.3). A `check` ask binds a time index, so it runs at every period (L §5.16) |
| DR3 | Read by role or query, never by name; an exclusive role has ≤ 1 provider per window; each provider of a collective role is consumed by exactly one consumer per window | merged | Laws collect by query: sums over keys with `where` (L §5.10); asks pick places by kind (L §5.9). The role rules are `check` asks in the law part: a count per window. A built-in role system was rejected: it would be a second wiring language beside keys and sums, and checks make a violation loud |
| DR4 | Place-kind invariants | taken | `RUN-PLACE`, `LOAD-KIND` (L §5.9) |
| DR5 | Richer units: real vs nominal, certain vs promised, one money representation | taken | `prices Y by idx` (L §3.4); `promised` (L §5.13); money in minor units (L §3.1) |
| DR6 | Status outside the content, against a content hash | taken | O §1: draft, tested, confirmed, per hash |
| DR7 | Law values only through tested law templates, with the cross-check inside the gate | taken | O §3: a law part is read by the reality only when `tested`; independent checks are the cross-check |

---

### 2. Spec requirements

#### M. Model and data

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| M-01 | Plain data from a closed list of block kinds, no code, serialisable for structured output | taken | A closed grammar (L §2.3), no user functions (L §10), a canonical form (L §2.4). Nesting is bounded (8 deep), so any schema of it is finite |
| M-02 | Place kinds: account, cash, holding (in lots), asset, debt (with contract state), claim, outside, memo | merged | Kinds `account`, `cash`, `holding`, `asset`, `debt`, `claim`, `outside` (L §5.9). Lots are keyed inputs and `fill`, not part of the place (CHG-01). A debt's contract state is inputs and defs. **Memo rejected:** a number no money moves through is a def |
| M-03 | A movement is dated, ≥ 0, direction only in from/to, carries why and period | taken | `pay`: `from`, `to`, `amount` ≥ 0 (`RUN-NEG`), `on`, `for` (L §5.13); every movement names its instance (E §1) |
| M-04 | Multi-leg movements: one event, several legs | merged | One event is several pays that share keys; an exchange is two pays, one per currency (L §5.9). Conservation holds per pay, so no multi-leg construct is needed. Two records of one instance are its legs (`WARN-LEGS`, L §5.18) |
| M-05 | A value: named number or table with dated entries {from, value or expression, repeat, source}, law or assumption | merged | Inputs with rows in versions (source, governs, known; L §5.3, §5.7); bands for tables (L §5.8); `assumed` rows (L §8.2). Rows are open-ended, so `repeat` isn't needed. Expressions live in defs, not rows |
| M-06 | Lookup: last entry on or before the date; `before`; `repeat` | taken | The read rule (L §4.6). "Just before" is a read at an earlier key or with `at` |
| M-07 | A template: versioned, typed parameters, lines, checks, sources, tests, roles and joins | merged | A law part (L §6.1): typed inputs with questions, defs with rules pointing at versions (`v`), `check` asks, sources. Law versions are versions of rows and rules, not of the part. Roles and joins: DR3 |
| M-08 | An instance: template, arguments, lifetime | merged | A thing is a key of an index with keyed rows. Its lifetime is its rows' spans, or a `when` on membership. There is no instantiation step |
| M-09 | A line: one formula on a grid, with a gate, a payment {from, to, fx, reverse, match} and a posting day | merged | A def keyed by a time index is the grid; `when` is the gate; a pay is the payment (L §5.13). Conversion is explicit (L §5.13.3); "reverse" is two pays or swapped ends; matching is the importer's data (O §2.2) |
| M-10 | Attached instances (law) live while what they attach to lives | merged | A law reads its subject's keys. When the subject's rows end, the law's sums see nothing. `end` checks that nothing keyed by it still moves money (`WARN-ENDED`, O §6) |
| M-11 | A parameter declares what it reads (an interface) | merged | Typed inputs and defs, unit-checked at load. The item graph is the interface (E §3.1) |
| M-12 | Party: a person with dated facts who owns places and incomes | taken | Parties are keys; their facts are keyed inputs (L §3.5); entities too (L §3.6) |
| M-13 | Ownership: dated exact fractions with kind and provenance; community property; one allocation with a declared remainder | taken | `owns[p, x]` with dated rows and a kind key; provenance by version (L §3.6); `prorata` with its remainder (L §5.12) |
| M-14 | Stored statements append-only | taken | Facts and law are add-only (L §6.2); the store is append-only (O §1). Records are movements, `stated` rows are balance assertions, notes are annotations, a confirmed match is a correction version |
| M-15 | Derived, never stored: occurrences, lots, law windows, certainty marks, claims | taken | E §1 movement kinds; instances computed; lots by `fill`; claims by pays |
| M-16 | Valid time and record time on every statement | taken | `governs` and `known` on every version (L §5.3); record time R in the store (O §1). There are three times, not two: what a row governs, when it was known, and when it was recorded |
| M-17 | Stable ids, used by patches and random keys | taken | L §2.5; draw addresses are `name[keys]` (L §5.14) |
| M-18 | Every bank account and category claimed by a block or explicitly ignored | merged | An account with no place is `OP-ACCOUNT` (O §2.2). A category's records that nothing explains are unexplained lines in `residuals … by cat` (E §6.3), so an unclaimed category is visible, biggest first |
| M-19 | Things no bank shows exist as places with a valuation | taken | `asset CUR priced v` (L §5.9) |
| M-20 | `run(model, history, clocks[, scenario]) → timeline` | taken | `run(reality, request) → result` (E §1) |
| M-21 | ASCII ids; Polish only in labels | taken | L §2.1, §10 |
| M-22 | Composite templates: one block per real-world thing expands into its wired pieces, visibly | merged | A new thing is keys and rows (L §6.5 change 1: 9 added lines). The law parts collect it by query, so nothing needs expanding; `describe` shows what reads it (O §9). **The user's answer (D8, 2026-10-19): accepted, rows plus queries, no expansion step.** A new thing no law handles is refused (`RUN-NO-RULE`) or prints a hole with its question, never a silent 0 (L §6.2) |
| M-23 | Collect by query, not by list | taken | Sums over keys with `where` (L §5.10) |
| M-24 | Law chosen from facts; one version per window; several laws in order; a coverage check refuses gaps and overlaps at load | taken | Rules with `when` and `excepts` (L §5.10); rows by the read rule. Overlaps of rows are refused at load (`LOAD-OVERLAP`); bands with gaps too (`LOAD-BAND`). **Departure:** whether rules cover every key and date can't be decided at load when `when` reads facts, so it is checked per value: `RUN-RULES` (two apply), `RUN-NO-RULE` (none) |

#### T. Time and calendar

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| T-01 | Arrival windows, planned pessimistically, listed as an assumption | merged | An imprecise date (L §4.4) makes every span read it might fall in a hole, unless decided. Planning pessimistically is the model's own choice, written as an `assumed` row or rule. A built-in pessimism was rejected: it would be a silent default |
| T-02 | Once a windowed item happens, the forecast conditions on it | taken | Its record books on the covered day, and the instance is matched (E §6) |
| T-03 | Business-day shift over weekends and Polish holidays | taken | `next_business`, `prev_business`, `add_business` with a holiday `dates` input (L §4.2). Holidays are rows, so Wigilia from 2025 is a version |
| T-04 | Holiday calendars per country | taken | One `dates` input per country |
| T-05 | "The third-to-last business day" | taken | `add_business(end_of(m), -3, h)` |
| T-06 | Period ≠ payment date | taken | `for` and `on` on every pay (L §5.13). Card cycles are anchored months, `from 17` (L §4.1) |
| T-07 | Cumulative law: year to date minus paid; bases that carry forward | taken | Defs over month keys, `flows(p, span)` of the advances (L §5.13.1), recurrences (L §5.10) |
| T-08 | Last year as an input, from history or a stated fact | taken | A read at an earlier key behind a guard; the first year's figure is a row (L §5.10) |
| T-09 | Which date a value is read at | taken | The law date and `at` (L §4.3) |
| T-10 | Cash basis: law reads payment dates of other movements across years | taken | `flows(p, span)` sums by date; `for` by period (L §5.13.1) |
| T-11 | Relative dates from speech: anchor, phrase, window, the defaults rule, all shown | merged | The calendar built-ins (L §4.2) are the resolver. The MCP layer maps the phrase to a built-in with the statement's day as anchor; the engine evaluates it in an ask; the row keeps the phrase and anchor in its locator. So the engine computes the date, not Claude. **The user's answer (D8, 2026-10-19): the split is accepted:** the MCP layer maps the phrase, the engine computes the date, the row keeps the phrase and anchor |
| T-12 | Fixed phases within a day; a declared posting order | merged | Records and flow pays first, then state pays by `phase` 0–9, pay id and keys (L §5.13.2). The spec's eight named phases map onto numbers the model chooses |
| T-13 | The environment resolves monthly, declared as an assumption | merged | A price's rows govern months, and the read rule reads the month's row. An assumed resolution is an `assumed` row |
| T-14 | Law periods without built-in periods | taken | Anchored indexes, spans, one-off periods (L §4.1) |
| T-15 | "What did we think in March?", "what changed since last month?" | taken | The reality at R plus `as known` (E §2, O §1); `history` lists what changed between record times (O §9), and two record times are compared by running the reality at each (E §2). `plan` diffs a draft against the current reality only (O §7), so a diff of two record times in one result is a gap (`CHANGES.md`) |
| T-16 | Four clocks S, K, (K, E], R; every read masked at K; a plain forecast sets S = K | taken | E §2. **Departure:** S is the header's `start`, not K, so a plain forecast replays the covered past before K. A request may set S = K (E §2) |

#### U. Money, units and inflation

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| U-01 | Unit algebra | taken | L §3.2 |
| U-02 | Only small integers as literals | taken | Stricter: L §7.1 also bans date literals in rules |
| U-03 | Round where the law rounds, with its mode and unit at each step; tests tolerate published 1-grosz disagreements with a reason | taken | `round(x, u, mode)`: seven modes, any positive step (L §7.2). A published disagreement is a check with its errata (O §3, L-41) or an axis |
| U-04 | EUR and USD places; FX as dated values; NBP previous business day; a group balance at a declared FX read | taken | Places per currency; rates `num A/B` (L §3.2); `prev_business` with `at`; `flows … in CUR at fx mode` (L §5.13.3) |
| U-05 | FX applies only between different currencies | taken | Unrepresentable: `money EUR × num PLN/EUR` is PLN, and the same rate applied to PLN money is `LOAD-UNIT` |
| U-06 | Holdings in FIFO lots; value = units × price; revaluation by unit flows | merged | Holdings in units, priced (L §5.9); lots by `fill` in FIFO order (CHG-01); revaluation by an ask over `worth` and unit flows |
| U-07 | A gain split into FX, price and interaction | merged | Ask expressions with `as if` pins of the rate or the price (L §5.10) |
| U-08 | Simulate nominal; past amounts recorded, never recomputed | taken | L §3.4; booked amounts are fixed (L §5.13.3) |
| U-09 | The price index lives in the model, per scenario | taken | The index is an input; a scenario's rows override its future (L §6.4) |
| U-10 | A today's-money read side, the default on every chart | merged | `real(x, d, base, idx)` in asks (L §5.16). Chart defaults are the app's (out of scope) |
| U-11 | `today_money(x, date)` in questions | taken | `real` |
| U-12 | Spending learned from history converts each past row to today's prices before averaging | merged | The estimator is a def over record rows (L §5.18), and it writes the conversion with the index (`× idx[base] / idx[d]`). `real` itself is a lens for asks |
| U-13 | Every entered amount says which year's złoty it is in | taken | `prices Y by idx`, else nominal (L §3.4). A `prices` input is converted on every read, so a formula can't forget |
| U-14 | The deflator as a lens over any dated series | taken | `real` takes any index (L §5.16) |
| U-15 | Rates carry value, per, basis, convention | taken | The basis is the rate's type; day counts are named in `dcf` (L §3.3) |
| U-16 | Store what was stated; gross from net by the payroll law, by integer bisection under its rounding | merged | The stated net is a row. The gross is found by `solve … to 0.01` in an ask (L §5.16), and recorded as an `assumed` row that cites it (P-21). A def `solve` is allowed only where a law names the method |
| U-17 | Run-time conservation; net worth moves only at the boundary or with a price | taken | E §9 |
| U-18 | Money that isn't ours is a liability rules can't invest | taken | `holds`, `free`, `WARN-HELD` (L §5.9) |
| U-19 | Certain vs promised money | taken | `promised` is never counted by `plan_low` or `free` (L §5.13) |
| U-20 | One money representation | taken | Integer minor units (L §3.1) |

#### R. Run semantics

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| R-01 | The flow pass | taken | E §4.1 |
| R-02 | The day walk | taken | E §4.2 |
| R-03 | Strata inferred; questions are state | taken | State pays are inferred (L §5.13.1); asks may make state reads |
| R-04 | No foresight by construction: look-ahead sees only planned flows, never a rule's own future, drawn spending, random keys or the agenda | taken | `plan_low` counts only flow pays as known that day, each draw at its plan value (L §5.13.1, §5.14); state pays can't read later days (`LOAD-FUTURE`); `next_on` can't name a state pay. The draw's `plan` closes the path case, found while writing this file |
| R-05 | Look-ahead primitives (`project`, `min_over`) with the literal sentence as default | taken | `plan_low` over a place or a group (L §5.13.1); the household's planning figures are draws' `plan` values |
| R-06 | The forecast subtracts the household's planned transfers, including those made by other rules | merged | Planned transfers that are flow pays are subtracted. A state rule's future result isn't, by design: no rule reads another rule's future (N-1102, spec §4#1). A transfer planned in advance is written as a flow pay. **The user's answer (D8, 2026-10-19): accepted.** A planned transfer is a flow pay; a rule never reads another rule's future |
| R-07 | Measured pool vs paying pool; defined behaviour when one goes negative | taken | `plan_low` over a group of places, the lowest sum (L §5.13.1, added while writing this file). An account below zero prints `WARN-NEGATIVE`; a debt, claim or holding past its limit is refused |
| R-08 | Ordered allocation, the limit re-checked after each posting | taken | State pays in phase order; a same-day `flows` total counts what is booked so far (L §5.13.1, added while writing this file) |
| R-09 | Competing rules by priority, then stable id | taken | `phase`, then pay id, then keys (L §5.13.2) |
| R-10 | A fallback (sell to cover) reads realised state only, counts T+2 proceeds, counts its firings | merged | A state pay; its proceeds are a second state pay two business days later reading the sale's history. The rule counts its own unsettled proceeds from `flows` of past days. Firings are an ask: `count(d in day where flows(sell, d) > 0)` |
| R-11 | Every run reports a rule's breach rate and mean shortfall, and a run with the rule off | merged | Asks (`share`, `mean`); "the rule off" is an axis the rule's `when` reads, pinned with `with` (L §5.5). Not automatic: the model asks for it |
| R-12 | Reacting to published signals is allowed; to the model's own forecast is not | taken | A rule reads inputs (published series, masked at K) and the past. Asks can't be read by defs or pays, so a forecast can't be a signal. A typed lagged signal was rejected: `plan_low` is the one sanctioned look-ahead |
| R-13 | "Once, when …" triggers on a state condition; the firing date anchors the template's schedules | merged | A state pay with `when` on state and on its own history (`flows(buy, d0..d - 1) = 0`, with d0 a stated day on or after S) fires once. Dependents read the firing day from `flows` of past days; they are state pays, inferred |
| R-14 | Goal-seek only at query level, never inside a run | taken | `solve` in asks; in a def only where the law names the method (L §5.16) |
| R-15 | Back-testing a rule on real history; fits use data up to each decision; "did we follow it?" | taken | Masking at K and rolling origins (E §8); a state pay's modelled past is computed against the real balances (E §4.2, §6.2) |
| R-16 | Deterministic to the grosz | taken | L §7.5 |
| R-17 | Fast | taken | P §8 |
| R-18 | Bands, "N in 1 000", common random numbers by stable id | taken | `quantile`, `share` with SE (L §5.16); addresses (L §5.14) |
| R-19 | Runs stored as aggregates by hash; paths replayed on demand | taken | E §1, §7; P §1 |
| R-20 | Law per path at an acceptable cost | taken | E §7; the cost is estimated in P §5 and measured at step F |
| R-21 | Defaults are preferences: the literal rule is the default; caution is the household's choice | merged | No built-in caution. Plan values and window choices are the household's rows |
| R-22 | Solving inside a law only with a solver the law names; every other cycle refused | taken | A bounded recurrence with `first` (L §5.10); `solve` in a rule with `v` (L §5.16); the previous period's value is an earlier key. Same-key cycles are `LOAD-CYCLE` |

#### P. Past meets future

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| P-01 | Before today bank rows; from today generated rows; start from real balances | taken | Records on covered days, the plan from K (E §6.1); `opening` (L §5.9) |
| P-02 | The modelled past computed, never booked, reconciled with the bank | taken | E §6.2, §6.3 |
| P-03 | Law for the past reads records, not the modelled past | taken | `flows` of a place reads records on covered days (E §6.2) |
| P-04 | An earner's past comes from dated history | taken | Rows govern dated spans (L §4.6) |
| P-05 | Certainty marks derived, plus a dated missed state | merged | Movement kinds `record`, `past`, `plan`, `promised`, `correction`, `hole`, and the modelled past (E §1). "Missed" is the ledger's "expected but missing" line (E §6.3) |
| P-06 | Partial fulfilment: the remainder's date and chance are asked | taken | The "partly fulfilled" ledger line, with its question (E §6.3) |
| P-07 | A missed window: keep open, move or write off; the household decides | merged | The ledger line asks; each answer is an amendment: a row moving `on`, or a `-` row (O §2) |
| P-08 | Exact matches link automatically; fuzzy ones proposed; a window per prediction; holds replaced; repricing as `replace` with a valid-from | taken | O §2.2: the household's confirmed rule (`match_days`, `match_amount`); holds restated by key. Repricing is a version that governs from its date |
| P-09 | A chance item not happened by its window's end is missed; later chances update | merged | A covered day before K shows what happened; an instance with no record is "expected but missing", and the modelled past is never booked. A chance is an input; updating it is an addition |
| P-10 | Author kinds; only confirmed ones count | merged | Authors in the store (O §1); provenance per version (L §3.7) |
| P-11 | A stated number beats a probability word; an editable lexicon | out of scope | The MCP layer. The engine takes numbers only |
| P-12 | Uncounted cash: a balance assertion with a pad flow | merged | A `stated` row for a cash count (L §5.18). It re-anchors the place's balance from its day (D8, J70), with no movement; the difference is a ledger balance line (E §6.3), and the household may add the record that explains it. An automatic pad was rejected: it would be a figure nobody stated |
| P-13 | Claims derived from open commitments | merged | A claim place booked by pays (L §5.9) |
| P-14 | Refunds, split bills, own-account transfers, card cycles, a 13th salary, re-categorisation, a disagreeing forecast, 0% instalments | merged | Each is existing constructs: a refund pay or an unexplained record; a claim; two legs of one instance; anchored months; a yearly pay; a correction restating a record's field; the ledger; a debt place |
| P-15 | Money to a new counterparty: still ours (a claim) or gone (outside)? | taken | O §2.2 asks it for a counterparty with no `party_place` row |
| P-16 | Spending fitted from history; parameter uncertainty drawn once per path, unreadable by forecasts | merged | Generators are draws and defs (L §5.14). Fitting is Claude's loop (E §6.4); the engine never fits (the user's decision). A parameter's draw has a `plan` value, so forecasts never read the drawn parameter |
| P-17 | Agreement with the bank counts as evidence only on real imports | out of scope | The process. Every ledger line carries its records' source, so a synthetic import is visible as one |
| P-18 | The import interface: classifier, normalisation, clustering, broker history, holds and duplicates | out of scope | The user's pipeline (spec §2.3), except holds and duplicates, which the import operation handles (O §2.2) |
| P-19 | Today's state settled before what-ifs | merged | At K, missing expected records are ledger lines with questions (E §6.3). A scenario runs over whatever the reality says (L §6.4) |
| P-20 | Each bank row linked to the instance and period it explains | taken | `explains` (L §5.18) |
| P-21 | Gross pay history from payslips; inferred from net by running the law backwards, marked inferred, checked against PIT-11 | merged | Payslip rows; `solve` in an ask; an `assumed` row citing it; a `check` against PIT-11 |
| P-22 | A pay trajectory fitted from history, marked modelled, a draft until confirmed | merged | Claude's fitting loop writes `assumed` rows (E §6.4); drafts until confirmed (O §1) |
| P-23 | Reconcile timing too, with a suggested day or window | merged | The ledger's timing difference (E §6.3). The suggestion is Claude's, from the ledger |
| P-24 | The import contract: counterparty resolved to a place; matching on places, period, window, tolerance; category a hint | taken | `account_place`, `party_place`, `match_days`, `match_amount` (L §5.18, O §2.2) |
| P-25 | One registry of entities with stable ids; renames keep ids; dated merges and splits; parent categories | merged | `bank_party` keys are stable; `party_place` rows are dated, so a merge is two keys mapping to one place from a date. A parent category is a relation input. The registry itself is the pipeline's (out of scope) |

#### L. Law and tax

The law content (L-01 to L-23) is data: law parts the engine runs, written after it (principle 10, the user's "universal engine"). For each, the row names the constructs that express it.

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| L-01 | Employee: gross to net, the 30× ZUS cap, the 120k threshold, KUP, PIT-2, the annual return, the refund | taken (data) | Year-to-date sums with `min` for the cap; cumulative bases (T-07); rows for KUP and PIT-2; joint or separate as an axis or a fact; the refund as a pay dated by the return |
| L-02 | PPK legs; reliefs | taken (data) | Pays per leg; reliefs as rules with `when`; ordered reliefs by `fill` |
| L-03 | A raise says how it meets the wage index | merged | The raise row says its basis: `prices Y by wage_index`, or nominal (L §3.4). CHG-14 gives G6 its statement |
| L-04 | Sole trader: forms, ZUS stages, health, FP, ryczałt tiers, a mixed car, the settlement | taken (data) | Rules on the form (a fact or an axis); bands by months since start; shares as `num`; the settlement reads `flows` of advances |
| L-05 | Sole-trader gaps: cash-basis deduction, health true-up, health as a cost, refusing a mid-year form change, sick pay | taken (data) | `flows` by date; a settlement pay; defs; a `check` or a hole rule for a mid-year change; `when` on voluntary insurance |
| L-06 | A fair B2B vs employment comparison | taken (data) | Scenarios or an axis side by side; no winner (L §10) |
| L-07 | VAT: output minus input; a mixed car at 50% | taken (data) | Defs |
| L-08 | VAT extras | taken (data) | Anchored quarters; a carry-forward recurrence; `flows` for the cash method; year-to-date thresholds; the due date apart from the payment (`for` and `on`); the split-payment account left out of the liquid group by kind or id; KSeF amounts as record fields |
| L-09 | Belka / PIT-38 | taken (data) | Yearly sums; FIFO by `fill`; `max(0, …)`; a pay on 30 April |
| L-10 | Belka extras | taken (data) | `prev_business` with the NBP rows; losses by `fill` with a 50% cap per year; opening lots as rows; "if we sold today" by `as if`; withholding as pays |
| L-11 | IKE: dated limit never going down, exceeding refused, the qualified-withdrawal gate | taken (data) | Rows, with a `check` that the limit doesn't fall; the investing rule takes `min` with the room; a direct pay over the limit fails a `check`; the gate compares dates with `age` |
| L-12 | IKZE and PPK | taken (data) | The refund at the marginal rate by `as if` |
| L-13 | OKI from 2027 | taken (data) | The average daily value as a state read over days (a state pay); the reference rate row on 31 October; `round(…, 0.0001, down)`; `max` for the floor; `min` for the exempt amount |
| L-14 | IKE vs OKI vs an account as scenarios, with a breakeven, never a choice | taken | Scenarios side by side and `solve` (L §5.16, §6.4) |
| L-15 | Bonds: capitalisation, CPI with a lag and a vintage, early-redemption fee | taken (data) | A recurrence; the index read at `m - lag`; a vintage key; `when` |
| L-16 | An annuity mortgage | taken (data) | `factor` and a recurrence (L §3.3) |
| L-17 | Loan extras: resets, WIBOR vs POLSTR by signing date, overpayment, 0% instalments | taken (data) | Rate rows; the POLSTR start date is a row, read by `when signed[l] >= polstr_from` (no date literal in a rule, L §7.1); overpayment as a fact or an axis; a debt place |
| L-18 | Property: PCC with the first-home exemption, fees, running costs; rent ends at purchase or a warning | taken (data) | Defs; the household's `check` "one housing cost at a time" (S-08) |
| L-19 | The 5-year PIT-39 clock and its relief | taken (data) | `add_years` and year ends; a def |
| L-20 | Zasiłek | taken (data) | Rules with `when` |
| L-21 | Benefits | taken (data) | Defs and pays keyed by child and month |
| L-22 | German employment (PAP), all classes, a joint return with Poland | taken (data) | A law part. The tariff used three times is one def read `as if` with each argument (A-06) |
| L-23 | Polish social insurance for a German job | taken (data) | A classification def (CHG-05) that the ZUS rules read |
| L-24 | A published law change is a dated entry with its applies and known dates; an unpublished one only a scenario | taken | Versions with `governs` and `known` (L §5.3); scenario versions have no `known` (L §6.4) |
| L-25 | Unverified points | out of scope | Law research. Each is an `assumed` row or an axis until checked |
| L-26 | Eligibility, not only arithmetic: the joint return checked against its conditions | taken (data) | Rules with `when` over facts; an ineligible return is not applicable (`-`) |
| L-27 | Retroactive law books the difference as dated arrears or a refund; with no date, a hole | taken | `arrears on D | none for <pays>`; no line gives `HOLE-CHANGED` (L §5.13.3, E §5) |
| L-28 | Several laws on one income in a stated order; trying orders only where the law says, with a cap | taken | The order is the read graph (L §5.10); `fill` for shared caps; orders tried as an axis, capped at 64 (`LOAD-FORKS`) |
| L-29 | "As if" inside a law | taken | `as if` (L §5.10) |
| L-30 | The gate, in order | taken | O §3, seven steps. Coverage and planted errors were added to O §3 while writing this file |
| L-31 | Tests from published figures, collected by a non-author | taken | O §3 step 3 |
| L-32 | Both kinds of test kept | taken | O §3 step 3 |
| L-33 | The gate's scripts and figures hidden from the author | taken | O §3 step 3 (process) |
| L-34 | The cross-check inside the gate | taken | O §3: the independent checks are the cross-check |
| L-35 | Payslips and real returns become tests | taken | O §3 step 3 |
| L-36 | An independent oracle agrees to the grosz | taken | An oracle is a check. D7's evaluators and the build's tests are this process |
| L-37 | Rounding errors caught | taken | O §3 step 5 plants rounding errors |
| L-38 | Behaviour between and beyond tested points probed | taken | O §3 step 6, the threshold table |
| L-39 | Expected answers are themselves checked | merged | Every expected figure is a `check` from a published example, a payslip or a real return, collected by someone who isn't the author (O §3 step 3; L-31, L-35); a figure two sources disagree on is an errata check (L-41). Collecting it twice is process |
| L-40 | Who writes law doesn't matter | taken | O §3 (the user's decision) |
| L-41 | A published figure that disagrees with its law: match the law, keep both, report the gap | taken | O §3 |

#### S. Scenarios and their life

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| S-01 | A scenario: base, `from`, operations; reality never changed | taken | Scenario files (L §6.4) |
| S-02 | Stacking | taken | `over`, at most 4 deep |
| S-03 | Patches on stable ids: add, remove, replace, end, override | merged | Rows override, `-` and 0 rows end, new sections add. "Remove" is `-` rows from `from`: nothing is deleted, so the base's history stays readable |
| S-04 | `replace` keeps draws only if numbering is preserved | taken | Addresses are keys: the same keys keep their draws (L §5.14) |
| S-05 | Rebase, a gone target, stale, promote | taken | O §5. A target can't be gone, since ids and keys are never removed |
| S-06 | Counterfactual pasts: a branch date, plus which later facts survive | merged | A scenario `from` a past date; every later reality version still applies unless the scenario overrides it (L §6.4). `as known` for what was known then |
| S-07 | Linked overrides | taken | The model's own derivations carry the override (O §5) |
| S-08 | Valid but wrong warns | taken | The model's `check` asks run in the scenario (O §5) |
| S-09 | Scenarios only from household statements | taken | `OP-STATEMENT` (O §5) |
| S-10 | A saved analysis | taken | O §8 |
| S-11 | Shared futures; environment counters in absolute months | taken | Draw addresses don't name the scenario; time keys are calendar keys, not offsets from `from` |
| S-12 | Isolation: new pieces don't move proven results | taken | `plan` lists every result that moves (O §7); checks re-run in the gate |

#### Q. Questions, explanation and the read side

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| Q-01 | Questions read-only; answers are queries over the run | taken | Asks have no pay (L §5.16); aggregates `first`, `last`, `min`, `max`, `argmin` over days |
| Q-02 | Measures: futures, first passage with censoring, the lowest day, lenses, IRR/TWR, price/FX split, change, sensitivity | merged | `share`; `first` with a test of `any` for censoring; `argmin`; sums by field or week; `solve` for IRR; `as if` for splits and sensitivity; `plan` for change |
| Q-03 | One rule for when "N in 1 000 futures" may be said | taken | **The user's answer (D8, 2026-10-19): the engine owns the rule.** Every `share` over paths prints `sayable` or `not sayable` with its reasons (E §8, L §5.16): two standard errors ≤ 0.05 (Agresti–Coull, so "0 of 10" isn't sayable; clean-side audit F1, `AUDIT-D8.md`), the carry (the money-weighted years the term's movements rest on `held` or `assumed` figures) ≤ 2 years, and no pin on a draw. On the household, 3 years of mostly known money gives a carry of about 1.50 years (sayable); a 30-year flat-price bet about 15 (not sayable). Cost O(movements in the window), no refusals of its own. The MCP layer words the verdict. Build: M7 (BUILD-ORDER) |
| Q-04 | Every number from a tool result: small, units, source tag, Polish sentence | merged | Typed results with units and source tags (O §1). The Polish sentence is the MCP layer's |
| Q-05 | Templates describe themselves | taken | Inputs carry questions; `describe` (O §9); `check` asks as sanity checks |
| Q-06 | Difference ledger; attribution; representative paths; Shapley | merged | `plan` (O §7); `as if` differences; a path chosen by `quantile` and replayed by `explain`. Shapley is an ask over `as if` with up to 6 causes (2⁶ = 64, the fork cap) |
| Q-07 | Net worth at market value and after liquidation, neither the default | merged | Two asks; liquidation by `as if` a sale |
| Q-08 | No weighting, no winner; breakevens; the median path-wise difference | taken | L §10; `solve`; paired paths (L §5.14) give the path-wise difference |
| Q-09 | Provenance on every movement | taken | E §1, L §3.7 |
| Q-10 | A derivation tree | taken | `explain` (E §10) |
| Q-11 | An assumption register | taken | L §8.2 |
| Q-12 | Transitive, uncapped, with falsifiers | merged | The register is transitive and uncapped (L §8.2). A falsifier is a `check` the model writes against records to come |
| Q-13 | A known-through horizon, so "held" separates known from guessed | taken | `held` in the register (L §8.2) |
| Q-14 | Today's money by default on charts | out of scope | The app; `real` is the engine's part |
| Q-15 | A warning when a reader outlives what it reads | taken | Stricter: a read past the last row is a hole (the gap rule); an open-ended row read past K is `held` |
| Q-16 | Read places by kind or role | taken | `kind`, `currency`, `owns` (L §5.9) |
| Q-17 | Warnings: below zero, no further pay, selling more than held | taken | `WARN-NEGATIVE`; `next_on` with no instance left is a hole (L §5.13.1); `RUN-PLACE` |
| Q-18 | "What does my job really pay?" | merged | Asks over the linked records and the payslip rows |

#### C. Built-in correctness

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| C-01 | The validator runs first | taken | Load (E §3.1, L §8.3) |
| C-02 | The language is total; templates nest a fixed depth | taken | L §10; 8 levels of expression, 4 of `as if` |
| C-03 | No value on a date is refused, never 0 | taken | The gap rule (L §4.6, §8.1) |
| C-04 | Every employment sits in some annual return | merged | A job read by nothing is `WARN-UNREAD`; the role rule is a `check` in the law part (DR3) |
| C-05 | Joins checked per year, with "any of" | merged | `check` asks over years with `any` |
| C-06 | Checks run on every date the arguments change | taken | A check binds a time index (DR2) |
| C-07 | Two instances in one role at once warn | merged | A `check` (a count ≤ 1 per month), which also runs in scenarios (S-08) |
| C-08 | Debt ≤ 0; a payment into a debt capped at its balance | merged | A departure from the spec's "capped": `RUN-PLACE` refuses rather than caps, since a cap would hide the error. A model that wants a cap writes it, with `min` or a `last` rule (the household's loans: `principal_part` takes `min(debt_before, …)` and its `last` rule the remainder, and `overpay_paid` is `min(overpay, debt_before − principal_part)`, so no pay into `loan_debt` can overshoot; checked in D8 for the laptop loan and change 6's car loan). **The user's answer (D8, 2026-10-19): refuse; a cap is written explicitly** |
| C-09 | Draft law refused in reality, warns in a scenario | taken | O §3, `WARN-DRAFT` |
| C-10 | Status outside the content; Claude's parts load as draft; only the user confirms | taken | O §1 |
| C-11 | Derive, don't declare | taken | DR1 |
| C-12 | Law values only through tested templates | taken | DR7 |
| C-13 | Property tests for no foresight | taken | Build tests (`BUILD-ORDER.md`); the design makes foresight unrepresentable (R-04) |
| C-14 | Conservation at run time | taken | E §9 |
| C-15 | The engine frozen by hash; data-only additions provably data-only | taken | `tally N` (L §9); an amendment never changes the engine |
| C-16 | Regression tests for the prototype's bugs | taken | Exact arithmetic has no rounding helper to lose a cent; FX by unit types (U-05); both are build tests |
| C-17 | An earner nobody reads is flagged | taken | `WARN-UNREAD` |
| C-18 | Redefining an id refused unless versioned | taken | `LOAD-DUP`; `revise` with a reason (O §4) |
| C-19 | A question read-only by construction | taken | L §5.16 |
| C-23 | The scenario suite | taken | Process: D6's `amendments/`, the build's tests |
| C-24 | A combinatorial matrix and every threshold | merged | Checks bound over a matrix of inputs, and the threshold table (O §3 step 6) |

#### A. Claude and MCP

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| A-01 | One catalogue is the whole contract, at a size that fits | merged | LANGUAGE §2, §5 and §8 are the catalogue. A reference card cut from them, about the prototype's 13k tokens, is a build task (`BUILD-ORDER.md`) |
| A-02 | A fresh Claude writes correct blocks | taken | Evidence: D4's fresh modellers; D6's modellers |
| A-03 | The validator is the last line; drafts run on a fixed environment; no LLM judge | taken | Load, the gate, `plan` (O §1). No LLM judge anywhere |
| A-04 | The schema Claude writes is non-recursive | merged | Claude writes text, checked by the parser. Bounded nesting keeps any structured schema finite |
| A-05 | The formula surface chosen by evidence; a change keeps pinned results and gate tests | taken | D3–D4 (`judging.md`); migrations (L §9) |
| A-06 | Named formula macros | merged | No user functions (L §10). A formula used with several arguments is one def read `as if` each argument (N-1904) |
| A-07 | Sign or direction: legs only, kinds checked | taken | L §5.13 |
| A-08 | Rate period or basis: every field required | taken | L §3.3 |
| A-09 | Nominal vs real: a required price year | taken | L §3.4 |
| A-10 | Gross vs net: store what was stated; the gross is a hole | taken | An input with no row is a hole |
| A-11 | Relative dates: anchor and phrase, resolved by the engine | merged | T-11. **The user's answer (D8, 2026-10-19): the split is accepted** |
| A-12 | Probability words: the lexicon; stated numbers win | out of scope | The MCP layer (P-11) |
| A-13 | Silent defaults: a named default entry, shown | taken | `assumed` rows (L §8.2) |
| A-14 | LLM arithmetic: expressions; only stated literals | taken | L §7.1 |
| A-15 | Tax or market literals banned | taken | L §7.1 |
| A-16 | Omitted consequence: a warning and a question | merged | New keys list the holes they open (O §2.1); `WARN-ENDED` (O §6); checks in scenarios (S-08) |
| A-17 | Entity confusion: resolve against ids; ambiguity a question | taken | `OP-TARGET` (O §1) |
| A-18 | Duplication: replace, not duplicate | taken | `LOAD-DUP`; duplicate records (O §2.2); `restate` |
| A-19 | Scenarios only from statements | taken | `OP-STATEMENT` |
| A-20 | An answer key of 80–150 Polish utterances on the eval harness | out of scope | The MCP layer's evaluation (`apps/evals`) |
| A-21 | The MCP surface | taken | O §9 |
| A-22 | API data retention | out of scope | The app's operations |
| A-23 | Tuning tools: `reconcile`, `drill`, `explain`, `trial` | taken | O §9 |

---

### 3. Proposed changes

| Change | Status | How, and why |
|---|---|---|
| CHG-01 Sub-ledgers by vintage inside a place | taken differently | No built-in lots or pooling. Lots are vintage-keyed inputs and defs, matched by `fill` in the law's own order (L §5.11, §10). Each jurisdiction pools differently (FIFO, a UK pool, wash sales, a deemed sale), so a built-in rule would be wrong for most. A vintage-keyed def keeps the original size (LH-04) |
| CHG-02 History queries in rules | taken | `flows(p, span[, for k])` of a flow pay is not a state read (L §5.13.1); `bal(x, d)` and `worth(x, d)` in state pays; reads at earlier keys; aggregates over record rows (L §5.18). The look-ahead never reads the future (the change's risk) |
| CHG-03 Group units and relations with dates | taken | Membership and relation inputs with dated rows; sums over keys (L §3.5). No built-in unit type |
| CHG-04 Carryback | taken differently | The later year's loss is matched to earlier years by `fill`; the refund is its own pay, dated when claimed. The closed year's booked amount stays fixed (L §5.13.3), so history isn't rewritten |
| CHG-05 Classifications from facts | taken | Defs, read by other laws like any def; a court's ruling is a given (L §5.10) |
| CHG-06 Conventions: FX per leg, day count, compounding | taken | `dcf` with five day counts; the rate basis as type (L §3.3); FX per movement with a mode (L §5.13.3); per leg as rows or axes. No default can be written (L §10) |
| CHG-07 Entity parties and look-through | taken | Entities are keys with places; `stake`; look-through by a bounded depth recurrence (L §3.6) |
| CHG-08 Stochastic lifetimes | taken | A draw per party per path, `alive[p, m]`, `when alive` (E §7) |
| CHG-09 Ranked allocation | taken | `fill` in a stated order; `prorata` with its remainder; an unstated order is a hole (L §5.11, §5.12) |
| CHG-10 Scores with exact statistical functions | taken | `crps` exact over samples, closed form for normals; exact `binomial`; `orient` (L §5.15) |
| CHG-11 A law's own rule for a missing record | taken | `missing(r)`, only in a `when` of a rule with `v`; listed in the register; `LOAD-MISSING` elsewhere (L §5.10) |
| CHG-12 Estimators over history with what they carry | taken | Defs over record rows, masked at K; price years and indexes as rows (L §5.18, §3.4) |
| CHG-13 A test mode for Monte Carlo statistics | taken | Pins of a parameter draw to its mean, listed in the register; oracle `check` asks with SE and tolerance (L §7.4, E §8) |
| CHG-14 What the goldens need | taken | The goldens are pinned on a canonical projection of the result: movements and end-of-day balances (E §1). Ownership is stated as facts. L-26 joins the fixes that may move a pin. G6 gets its raise statement (L-03). All four are build tasks (`BUILD-ORDER.md`) |

---

### 4. The judge's fixes

`judging.md` §8 listed 21 fixes for B. Each, with where the definition settles it:

| # | Fix | Decision |
|---|---|---|
| 1 | A correction at or before K when the pay is a hole | A hole instance stays a hole; a later version is applied to it like any other (E §5). A correction known by K counts as known on t (L §5.13.3) |
| 2 | A later-known change to a booked pay with no `correct on` | C's `arrears` on the version, `HOLE-CHANGED` with no line, now per pay with `for` (L §5.13.3) |
| 3 | The forced `round(…, 1e-30)` for carried ratios | Exact rationals with the `RUN-DIGITS` cap at 4 096 digits, which refuses (L §7.1) |
| 4 | `solve`: which root; flag the edges | The grid point nearest the root, a tie going up; no sign change is `RUN-SOLVE` (L §5.16) |
| 5 | The error of composite transcendentals | Composites are exact on the rounded values; `≈` with an error bound when no `round` follows (L §7.3) |
| 6 | Age before birth; the child-born pattern on a monthly base | `age` is `-` before birth (L §4.2). The pattern is change 4 of L §6.5 |
| 7 | The empty `sum`; short-circuit `and` / `or` | `sum` over no keys is 0; `and` / `or` left to right with Kleene values (L §5.10) |
| 8 | Whether a `with` pin reaches an input read through `at` | It replaces every read of the input at those keys, `at` included (L §5.10) |
| 9 | A cite per row | Every row's version cites a source; a row may add a locator (L §3.7) |
| 10 | A law change without restating figures | Open-ended rows and the `version` header (L §5.3) |
| 11 | Name clashes | One namespace, every reserved word listed, `LOAD-RESERVED`, `LOAD-DUP`, `LOAD-SHADOW` (L §2.2, §2.5) |
| 12 | The single `=` after `when` and in `solve` | C's parse rule (L §2.3) |
| 13 | Types: a bare 0, `int × ratio`, `round`'s result, `year_of` on a month, `m + 1` past the end | L §3.1, §3.2, §4.2; `RUN-KEY` past the end |
| 14 | Whether the precision rule covers a recurrence's start rule | Every rule, the start rule included, is one rule body under the same rules (L §5.10, §7.1) |
| 15 | A declared bound: per value or at load | At load for defs, pays and fills (`LOAD-BOUND`); printed in asks (L §5.17) |
| 16 | The order of draws; `*` in keys | The canonical draw order (L §5.14); `*` covers the keys known at the row's date (L §4.6) |
| 17 | A pay whose date is a hole; a "nobody" key | A hole date makes every `flows` it might fall in a hole (L §5.13); not applicable is `-` |
| 18 | Poisson; argmin / argmax; an assertion ask; a lattice span | `poisson` (L §5.14); `argmin`, `argmax` (L §5.10); `check` (L §5.16); a span over an index needs no input (L §4.1) |
| 19 | Guidance on `given` against `input` | A given needs a source that states the result; an assumption is an `assumed` input row (L §5.10, `OP-GIVEN`) |
| 20 | A def without a time key evaluates at the model date | Law-date propagation (L §4.3 rule 3), not a refusal: §5 decision 1 |
| 21 | The cost of imprecise dates; a cap on axes | Two endpoint evaluations per operation (L §4.4); 64 forks (`LOAD-FORKS`) |

---

### 5. Decisions the definition made, with the alternatives weighed

Decisions taken in D5, beyond the candidates and the judge. Each weighs the alternatives against the qualities in order (spec §2.4: never wrong in silence; stated once; explainable; deterministic; fast; expressive; writable).

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 1 | **Law-date propagation:** a def with no `at` and no time key takes its reader's law date (L §4.3) | (a) B: the model's `on` date; (b) the lead's §9: refuse such a def when it reads dated rows | (a) is wrong in silence: a child's needs read by a July pay used `on`'s law. (b) is loud but forces a snapshot model to be restructured when it grows a time axis (quality 2). Propagation is right, and `explain` prints the law date it ran at (quality 3) |
| 2 | **K defaults to the model's `on`, which is required**; a version known after `on` is refused (L §4.5) | (a) no default cut, every version visible; (b) the latest known date | (a) lets a forecast read tomorrow's news. (b) moves with every addition. `LOAD-KNOWN` also catches a mistyped year |
| 3 | **Scenario versions are visible by date**, from `from`, not masked by K (L §6.4) | Scenario versions known on `from`, masked by K like reality's | With K = `on`, a scenario starting in the future was invisible to every forecast: a contradiction found in drafting |
| 4 | **A default row covers only the keys known at its date** (L §4.6) | C's `complete`, covering every key | A child added later would silently get the default 0 (quality 1). Instead the new key makes holes with questions (O §2.1) |
| 5 | **Records as a construct** (L §5.18): movement inputs, `covers`, `stated`, `explains`, the modelled past | (a) Records as plain input rows, as in the cases; (b) records in a separate ledger outside the language | (a) can't book the past or reconcile it, and replay is the core point (the user's decision). (b) splits one reality in two. Records as rows keep masking at K and provenance for free |
| 6 | **Four kinds of change:** add, advance, restate, revise (L §6.2, O §4; `advance` added in D6, gap L10) | (a) Add-only (B); (b) free edits (A's `replace`); (c) moving `on` as a restate of the header (D5) | (a) can't fix the model's own wiring. (b) can edit history in silence. (c) changes results, so it can't be a restate. Restate is proven by equal results; revise never touches a version; advance moves only `on`, forward, and shows its plan |
| 7 | **`arrears … for <pays>`** (L §5.13.3) | One arrears choice per version (C) | A retroactive version that changes both the monthly advances and the settlement would correct both: double counting |
| 8 | **A booked amount is fixed**; later versions give correction movements (L §5.13.3, E §5) | Re-evaluate past instances on every read | Re-evaluation rewrites history and double-counts with the corrections |
| 9 | **`held` in the register** (L §8.2) | No mark; or a warning per read | Q-13: a figure carried past K unchanged is a guess and must say so, without a warning per read |
| 10 | **`flows` of a flow pay is not a state read** (L §5.13.1) | Every history read is a state read | Law would need the day walk to read the year's advances; annual settlements would become state pays for no reason |
| 11 | **Rounding to any positive step, seven modes, the mode an axis** (L §7.2) | Fixed decimal units only | 5 Rappen and whole-złoty bases are steps; an unstated mode is the commonest hole (B row 25), so it must be an axis |
| 12 | **No date literals in rules** (L §7.1) | Allowed, as small integers are | A date in a rule is law data in code: a law change would edit a formula (A-15) |
| 13 | **Draws carry a `plan` value** for look-aheads (L §5.14) | (a) Look-ahead counts drawn values; (b) look-ahead skips draw-dependent flows | (a) is foresight on every path (R-04). (b) silently treats drawn spending as 0. The household's planning figure is their stated sentence (R-05) |
| 14 | **Same-day `flows` totals and groups of places** (L §5.13.1) | A memo place counting contributions; a sum of per-place lows | A memo place is a second kind of thing (M-02, rejected). A sum of lows is a different, wrong number |
| 15 | **The importer's rules are data** (`account_place`, `party_place`, `match_days`, `match_amount`) | Rules inside the importer's code | As data they have sources, the household confirms them once, and changing a tolerance is an addition |
| 16 | **Look-through by a bounded depth recurrence** (L §3.6) | Open recursion over entities; a fixed point | Open recursion breaks totality; a cross-holding has no finite answer, so a `check` compares the last two depths |
| 17 | **An exchange is two pays** (L §5.9) | A multi-currency pay | Conservation per currency holds per pay; the rate per leg is explicit |
| 18 | **The look-ahead is a segment tree** (P §2.3) | A sparse table | F changes as the cut rises during the walk; a sparse table would be rebuilt each time |
| 19 | **The gate has spec L-30's steps**, planted errors included (O §3) | Checks alone | Independent figures alone caught 75% of planted errors (L-32); coverage and mutants find the untested parts |
| 20 | **Pins `law` and `known` take expressions** (L §5.10) | Date literals only | Rolling origins need `as if {known origin_day[o]}` over an index (E §8) |
| 21 | **Matches the importer is sure of are written by the household's confirmed rule** (O §2.2) | Every match confirmed one by one | P-08 links exact matches automatically. A rule the household confirmed once keeps "drafts count only once confirmed" |

**Decided in D6**, from the round-1 gaps (`gaps/resolution.md`, `gaps/lead-household.md`):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 22 | **A state pay instance is never re-measured** by a later version that isn't a `correction`; `arrears` covers flow pays only (L §5.13.3, E §5; L17) | Re-measure every booked instance, state pays included (D5) | A retroactive law made every look-ahead decision since its start a `HOLE-CHANGED`: loud and wrong. A decision is taken with what was known that day |
| 23 | **A def that reaches a state read is evaluated at each reader's walk point** (L §5.13.1, E §3.2; L9) | Memoise it per cell, as other defs | Two state pays on one day reading one `room` def would invest the same money twice, in silence |
| 24 | **An instance's `on` is read at the ask's cut; its `when` and amount at min(t, K)** (L §5.13.3; L18) | Read all three at min(t, K) (D5) | The cut needs t, and t came from `on`: circular |
| 25 | **Three classes of reserved words**: keywords, built-in names (only before `(`), contextual words (L §2.2; G54, G1, L4) | One flat list of about 180 words (D5) | Every group renamed domain words (`age`, `rate`, `share`, `start`); the definition's own examples didn't load |
| 26 | **Recurrences at later keys, and at computed keys with `RUN-CYCLE`** (L §5.10; G14, G35) | Earlier keys only (D5) | Annuities are computed back from the last age; an earlier acquisition is found by `where`. Termination still holds: one direction per index, and a chain is no longer than the keys |
| 27 | **A pin's value is any expression, evaluated unpinned** (L §5.10; G55); `law D for [x, y]` (G64; replaced by the wording in force on D, decision 34, and removed in D7: no case used it, `coverage.md` R1) | Literal pins (D5) | "As if the parts were the base parts" and "the scale in its 2022 wording" are questions the law asks; evaluating outside the `as if` keeps them free of cycles |
| 28 | **Band rows hold any literal and read their own ends; `marginal`; keyed bands** (L §5.8; L16, G4, G45, G63) | Literal-rule band rows; edges restated as inputs (D5) | "10 800 + 32% over 120 000" wrote the edge twice and nothing checked they agreed (quality 2) |
| 29 | **A bound ranks between a refusal and a hole, and may appear anywhere in an ask; a hole is a fact a row could give, a bound a step no row can give** (L §5.17; G39, G52) | Bounds only as a whole ask body (D5) | A comparison with a bound is a question about the bound; data a row could give must stay a hole, so a later row fills it |
| 30 | **`covered(x, span)` guards every aggregate over a movement input** (L §5.18; L2) | Sum what is there | A month with no statement summed to 0: a silent undercount |
| 31 | **A part is a law part only when its header says `law`; the gate skips hole rules** (L §6.1, O §3; L19) | Law parts by directory (D5's layout) | The engine checks only headers; an honest stub was refused by the coverage step |

**Decided in D6 round 2**, from the round-2 gaps (`gaps/resolution.md` § Round 2):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 32 | **Known dates follow a fixed order of evidence** (L §4.5; H1, H10, H11): law from the publication that binds it, else its assent or making; the act that gave the wording; other sources from the earliest day they surely existed in the form read | (a) the modeller's judgement per source; (b) always the retrieval day | (a) gave eleven groups eleven readings of "publication", "updated stamp" and month dating. (b) hides every source from views before the fetch. The earliest sure day never shows a text too early, and dating a row by the act that gave its wording dates it once |
| 33 | **Back pay may be answered by a later version, `arrears … of V`** (L §5.13.3, E §5; H2) | (a) add the line to V once the answer is known (the old `HOLE-CHANGED` fix); (b) answer it with a `correction` | (a) breaks §6.2: a confirmed version is never edited, and a cut between the two dates would see an answer no one knew. (b) makes the answer known all along, the same error |
| 34 | **`enters` and the `wording D` pin** (L §4.3, §4.6, §5.3; H17) | (a) `law D for` alone (round 1, decision 27; removed in D7); (b) `known D for [x]`, a cut for named items | (a) reads the latest-known version governing D, so a retroactive act known before D but in force after it beats the text in force on D (UL-15, art. 26 ust. 3). (b) still passes a text published before D and not yet in force. Entry into force is a third date, apart from governs and known |
| 35 | **`moves V to SPAN`** (L §5.3, §4.6, E §3.1; H18) | (a) a new version per postponement, copying the rules (RC-01: three postponements of about 15 rules); (b) edit V's span | (a) copies law, which then drifts apart. (b) breaks §6.2, and an earlier cut would see a date no one knew yet |

**Decided in D6 round 3**, from the round-3 gaps (`gaps/resolution.md` § Round 3):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 36 | **An imprecise `at` or `on` reads one law date per law-date class its period overlaps; an `at` that is a hole makes a hole** (L §4.3, E §3.2; I3, I4) | (a) the period's first day (round 2); (b) both ends of the period | (a) picks the older law in silence when a governs start falls inside the period (MP-08: deaths "in 2015" against regulations from 2015-06-12). (b) misses a change inside the period that later reverts. The classes are what the memo is already keyed by, so the cost is one evaluation per class, usually one |
| 37 | **A stand-in is a version marked `assumed`, dated as the text it stands for** (L §4.5, §5.3, §8.2; I1, I2) | (a) rows marked `assumed` only (round 2); (b) the later text's dates | (a) leaves the rules that point at the stand-in out of every register (MP-16: Lines 5 and 10, the rounding, the minimum order). (b) hides the earlier text from every view before the later one, though it was public: `RUN-NO-RULE` where an assumed rule belongs |
| 38 | **Guidance is a dated issue; a document kept up to date is an other source, dated by its earliest public stamp (a printed date, a server date, a dated capture); a file's own stamp only where no public stamp exists, marked `assumed`** (L §4.5, §5.3; I5, I6; D8) | (a) every publication of an authority as guidance; (b) the earliest sure stamp, file records included (D6 round 3 to D7) | (a) gives no date to a leaflet or form instructions with no release notice (LH-14, CB-13, CB-14). (b) can show a text a few days before its public release, since a file is saved before it is posted. **The user's answer (D8, 2026-10-19): public stamps only, with a visible fallback.** The fallback's date is in the register, so a view that rests on it says so |
| 39 | **A commencement "to be fixed, at the latest D" is a `moves` to D** (L §5.3; I11) | (a) a hole start; (b) the original start until the fixing act | A governs end is a literal (§7.1), so (a) can't be written. (b) applies the act early under cuts before the fixing act (RC-05, décret 2019-1574). The latest date never shows law too early |
| 40 | **A `stated` balance re-anchors the walk** (E §4.2, §6.1, §6.3, §9; L §5.18; J70, D8) | (a) no re-anchoring: a balance is the opening plus everything booked (D5 to D7); (b) re-anchor and drop the difference | (a) lets one hole movement in an uncovered past spoil every later balance, through every statement (the household, D7). (b) hides the drift. Re-anchoring keeps stated − booked on the ledger's balance line, one line per statement, against the last anchor. **The user's answer (D8, 2026-10-19).** And every figure that reads an anchored balance lists the anchor, with its difference, in its register (`anchor:`, L §8.2): the user's answer to REPORT §13's question (clean side, 2026-10-19) |

---

### 6. Departures from the spec, and the questions they raise

Each departure is above with its reason. Those that touch a decision the user made, or a requirement where the evidence points another way, went to `REPORT.md` as questions. **All five were answered on 2026-10-19 (D8), each "yes" to the recommendation; the rows above carry the answers** (REPORT §12):

1. **M-22, composite templates.** The user decided that one block per real thing should expand into its wired pieces. In Tally a new thing is rows, and the law parts find it by query, so nothing expands (L §6.5: a new job is 9 added lines, no edits). Is "rows plus derived wiring" the "one block" you meant, or do you want a visible expansion step?
2. **R-06, other rules' transfers in the look-ahead.** Tally's look-ahead counts planned flows only. A rule never sees another rule's future result, which is what keeps the walk free of fixed points (spec §4#1). A transfer planned in advance is written as a flow pay, and then it is counted. Is that enough?
3. **T-11 and A-11, relative dates.** The engine computes every date, through its calendar built-ins. Mapping a Polish phrase to a built-in is the MCP layer's job, and the row keeps the phrase. Is a phrase map outside the engine acceptable?
4. **C-08, a payment into a debt capped at its balance.** Tally refuses the payment instead (`RUN-PLACE`), since a cap hides an error; a model that wants a cap writes it. Refuse, or cap?
5. **Q-03, when "N in 1 000 futures" may be said.** Left to the MCP layer, with the engine supplying the standard error and the register. Should the rule be the engine's?

Departures that touch no user decision and need no answer: M-24 (coverage checked per value, not at load), T-16 (S is the header's `start`), U-12 (the estimator writes its own conversion), A-06 (no macros), M-02 (no memo places), P-12 (no automatic pad).

---

### 7. Constructs and the cases that need them

From the models (D7, 2026-10-19): `coverage.md` counts every construct and built-in word over the 79 models, the household and the seven changes. The D5 version of this table listed the cases from the case files before the models were written; where the models differ, the last column says so. `coverage.md` §3 has the constructs no case uses, each removed or kept with its reason.

| Construct | LANGUAGE | Cases (models) | Note |
|---|---|---|---|
| header, source, version, input, index, def and rule, ask | §5.1–5.3, §5.6, §5.7, §5.10, §5.16 | all |  |
| a row with its own `governs` | §4.6 | CB-03, CH-05, CMP-01, LH-05, LH-07, MP-09, MP-11, RC-03, RC-08, UL-05, UL-06, UL-08, UL-13, the household, chg1, chg4, chg5 |  |
| `enters` (entry into force) | §5.3 | RC-08, UL-04, UL-15 |  |
| `moves` | §5.3 | RC-01, RC-05 |  |
| `arrears` | §5.13.3 | LH-14, the household, chg3, chg4 | planned: LH-14, UL-09, RC-08, UL-06 |
| `correction` versions | §4.5 | CH-04 | planned: CH-04, LH-05 |
| unit | §5.4 | CB-06, CH-02, CH-04, CH-06, LH-01, LH-02, the household |  |
| axis | §5.5 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-10, CB-11, CB-12, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-04, RC-05, RC-08, UL-01, UL-02, UL-03, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-14, UL-15 |  |
| `index … assumed`, `index …: open` | §5.6 | CB-14, the household |  |
| band | §5.8 | CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, the household |  |
| place | §5.9 | CMP-01, the household, chg5 | planned: also CMP-02, MP-14, MP-15, LH-15, LH-16, MP-01, MP-04, CB-10 |
| `excepts` | §5.10 | CB-02, CB-03, CB-06, CB-07, CB-09, CB-11, CB-13, CB-14, CH-02, CH-03, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, the household |  |
| recurrences | §5.10 | CH-02, CH-06, LH-02, LH-04, LH-05, LH-07, LH-10, LH-13, LH-15, LH-16, LH-18, MP-15, RC-01, the household |  |
| `as if` and `with` pins | §5.10, §5.16 | CB-03, CB-07, CB-09, CH-04, CH-05, CMP-01, LH-03, LH-04, LH-06, LH-12, LH-15, MP-03, MP-04, MP-05, MP-07, MP-08, MP-10, MP-11, MP-16, RC-03, RC-05, RC-06, RC-07, RC-08, UL-01, UL-03, UL-11, UL-13, UL-14, UL-15 | planned: 18 models |
| `missing()` | §5.10 | the household | planned: CH-01 |
| fill | §5.11 | CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13 |  |
| prorata | §5.12 | CMP-01, MP-01, MP-09, MP-12 |  |
| pay | §5.13 | CMP-01, the household, chg5 | planned: 10, the others hold their money as defs |
| state reads (`bal`, `plan_low`, `worth`) | §5.13.1 | the household, chg5 |  |
| `flows` | §5.13.3 | CMP-01, the household |  |
| draw | §5.14 | none | planned: LH-17, LH-18, CH-06, MP-14, CMP-02, CH-03, CH-01 (coverage.md K13) |
| exact distributions | §5.14 | CH-01, CH-03, LH-17 |  |
| scores | §5.15 | CH-03 | planned: also CH-01, CH-02, CH-06, which evaluate with exact distributions and arithmetic (backtest zones, actual-to-expected ratios), not §5.15 scores |
| path statistics | §5.16 | none | planned: LH-17, LH-18, CH-06 |
| `solve` | §5.16 | LH-10, LH-11, LH-17, MP-07, UL-08 |  |
| `check` | §5.16 | CB-01, CB-03, CB-05, CB-08, CB-09, CH-02, CH-04, CH-05, CH-06, CMP-01, CMP-02, MP-02, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, UL-03, UL-04, UL-08, UL-11, UL-13 | planned: CH-06, the household |
| `explain`, `inputs-of`, `residuals` | §5.16 | CB-07, CMP-01, the household |  |
| bound | §5.17 | CH-03, LH-17, MP-15 | planned: also LH-18 |
| records | §5.18 | the household |  |
| scenario | §6.4 | chg7 | planned: LH-10, CB-01, LH-04, UL-10, MP-14, the household |
| transcendentals `to u` | §7.3 | CH-01, CH-03, CH-06, LH-10, LH-11, LH-15, LH-16, LH-17, LH-18, MP-07, UL-08 |  |
| rounding | §7.2 | CB-02, CB-06, CB-10, CB-11, CB-12, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-04, RC-05, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-15, the household, chg5 |  |
