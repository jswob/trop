# Tally: the language definition

Version 2 (`tally 2`). This document and `ENGINE.md` are the whole definition. A modeller writes models from them, and an evaluator works a model by hand from them. Nothing else is needed or allowed.

Tally 2 is the redesign of tally 1.0.1 made in the design round of 2026-10-25. Each change is recorded in `DECISIONS.md` (records D-1 to D-15, cited as D-n), and `MIGRATION.md` says how a tally 1.0.1 model becomes a tally 2 model. What tally 1 decided and tally 2 keeps is recorded in `DECISIONS.md` as it was.

**Contents**
1. Overview and core concepts
2. Syntax: lexical rules, grammar, canonical format, names and stable ids
3. Types and units
4. Time, knowledge and the read rule
5. Constructs: what each computes, its refusals, its cost and its cases (kinds and moves among them)
6. Files, versions and amendments
7. Numbers, rounding, randomness and determinism
8. Holes, the register, refusals and warnings: the catalogue
9. Versioning of the language
10. What Tally can't say, and why

---

## 1. Overview and core concepts

<!-- tally2: D-1, D-2, D-6, D-11, D-14; core.md -->

A **model** describes a financial reality: money, law, time, people and uncertainty. The reality can be a household over thirty years, one worked example from an official leaflet, or a trust with forty beneficiaries. The engine (`ENGINE.md`) runs a model two ways:
- it **replays the past** against real records;
- it **forecasts the future**, one path or many.

It also answers **asks**, the questions written in the model.

A model is plain text in files with the extension `.tally`. It has one canonical form: the engine reprints any model in that form, so two equal models print identically and a diff shows only what changed.

**The qualities that settle every trade-off, in order:**
1. **Never wrong in silence.** A missing fact is a hole that prints its question. It is never 0. A missing statement is refused, never filled in by a default.
2. **Simple, stated once.** Every fact appears once. A change is an addition beside what was there.
3. **Explainable.** Every number traces to the rows, rules and sources that made it.
4. **Deterministic to the minor unit,** on any machine.
5. **Fast enough for conversation,** with headroom.
6. **Expressive as data.** A new law year, contract or household rule is data, not engine code.
7. **Writable by an LLM.** The surface is small and typed, and it is checked before anything runs.

**Everything is explicit.** That is quality 1 applied throughout: what a text doesn't state is refused or is a hole, never a default, and a figure no source states is a row marked `assumed` with its reason (§4.5). Two working rules follow from qualities 1, 2 and 7, and the sections below cite them by name:
- **R-load.** What isn't defined is refused: at load where load can see it, otherwise per value, at the key (§8.4).
- **R-once.** One home per fact and one name per meaning: two forms that say one thing are one too many.

**The one idea.** Everything a model talks about is a declared **thing** of a declared **kind**: a person, an account, a unit of value, a document. Every statement about a thing is a dated fact. A movement of money or units is a thing too, of an event kind, on a stated day. A rule about a kind of account (an act's rules, a contract's terms) reaches things and movements through bindings over their kinds, so it names a kind of account, never a list of accounts. Law, money in places, forecasts and randomness are this one idea, not four systems (`DECISIONS.md` D-1, D-2).

**The principles.** Seventeen principles apply the one idea to its parts; they restate the records D-1 to D-15, each with the solved form it reduces to (`DECISIONS.md`). A rule that follows from a principle is stated once, in the principle's home, and other sections cite it as P-k.

| P | Principle | Home |
|---|---|---|
| P-1 | One idea: declared things of declared kinds, dated facts about them, movements as things, rules that reach by kind (above). | §1 |
| P-2 | A thing is a key of a declared kind with a stated lifespan and one base kind; outside its lifespan, where no row governs, its facts are `none` (a movement's never lapse); every binding over a kind, but an item's key bind (§5.6), states its day or span. | §5.6, §5.7, §3.5 |
| P-3 | A place holds positions in its units, valued by a dated price or a hole; every leg names both ends; units multiply and cancel, a unit kind's key checked per value. | §5.10, §3.2, §3.6 |
| P-4 | A movement, planned or observed, is one construct of an event kind; a rule reaches movements and things only through bindings over kinds, never a list of things. | §5.14, §5.12 |
| P-5 | Every fact states when it holds and when it became known; which row a read takes is written; a reading is read away from its instant only through a written carry. | §4.5, §4.6, §5.3, §5.8, §3.7 |
| P-6 | Which day's law applies is written: every chain of readers of a law-dated fact ends at a written date, and a period is not a law date. | §4.3 |
| P-7 | Reads are declared, and a day's order is the stratification of the declared reads; a tie that changes a result is broken by a stated `ties by`. | §5.14.1, §5.14.2, §5.11–§5.13 |
| P-8 | The forecast is a query over planned movements; certainty belongs to the event kind; a retroactive change is a readable adjustment, never a re-measured decision. | §5.14.1, §5.14.3; `ENGINE.md` §4.3, §5 |
| P-9 | Absence has kinds, a hole, `none` and `never`, and only the hole spreads; a day not yet known is a window, decided by comparisons or a hole. | §3.1, §4.4, §8.1; `partial` in §5.18 |
| P-10 | An observation is a movement, its link to a plan is written, and a statement's gap is a readable fact whose effect is written. | §5.19; `ENGINE.md` §6 |
| P-11 | A question across models is a joint model of pinned, closed imports, with identities written and disagreements holes. | §6.1; `ENGINE.md` §3.1 |
| P-12 | A hypothetical is an overlay of ordinary versions that changes nothing before its divergence day or in its base; worlds are compared by a keyed difference. | §6.4; §5.18 (`diff`, worlds) |
| P-13 | A random quantity is a fact whose value is a distribution; its samples are pure functions of address, seed and path; a planning figure is a query written where it is asked. | §5.16, §7.4; `ENGINE.md` §7 |
| P-14 | Exceptions are written pairs, a replacement says `replace`, a rule citing a version exists only where it governs, and conventions are codes that nothing defaults. | §5.11, §3.3, §3.1, §5.4, §5.5, §5.9 |
| P-15 | Calendars, schedules and periods are generated by rules with stated coverage; spans are half-open; the run bounds booking, not facts. | §4.1, §4.2; `ENGINE.md` §2 |
| P-16 | One sign defines and another compares, values are named, figures carry units or sources, the value comes first, and grammar and prose are one text. | §2, §3.1, §3.2, §7.1, §7.2 |
| P-17 | Every value traces to its rows, rules and sources in a stated order; inverses and choices are aggregates over finite domains; a step no row can give is a hole saying why. | §5.18, §5.11, §8.2; `ENGINE.md` §10 |

**One reading order.** A construct that gives a value states the value first, then its bindings, filters, condition and dates (P-16, §2.3): `rule high = income[p] * top_rate when bracket[p] == top`, `sum(phone[p] for p in persons on d where phone[p] != none)`, `def x: money PLN = v at end_of(y)`.

### The concepts

| Concept | In one line | An analogy |
|---|---|---|
| **source** | A document that figures come from: an act, a leaflet, a bank export, the modeller's own assumptions | a dependency in a lock file |
| **version** | A dated, sourced batch of facts and rules: which source, which days it governs (written unless every row carries its own time), the day it became known; a row or rule may state its own span, and one that overlaps an earlier row at the same keys says `replace` | a commit with an author date and a commit date |
| **index** | A set of keys that aren't things: integers, years, months, days, options, ordered when listed or ranged; `days`, `months` and `years` of the run are built in | an enum or a range |
| **kind** | A declared class of things (people, places, units, events), with subkinds. Each thing enters with a lifespan; a thing has one base kind for its life and may belong to any number of plain kinds, each for a stated span | a type, and the traits it implements |
| **axis** | A list of readings no fact settles. Every ask shows each option side by side; the engine never picks | a build matrix |
| **place** | A thing of a place kind, where money or units sit; its kind states the unit, the sign rule and how it is valued | a bucket in a ledger |
| **input** | A named, typed fact whose values are rows in versions: a state over a span, a reading at an instant, a distribution, a code, a calendar. A missing row is a hole | a column whose cells are filled by commits |
| **band** | A table of intervals, checked for gaps and overlaps | a lookup table with ranges |
| **def** | A computed value, keyed, defined by rules with written exceptions, at a written law date | a pure function memoised per argument |
| **given** | A version row aimed at a def: pins it at stated keys to a result the source states | a test stub |
| **fill** | Allocation of supplies to demands along arcs, in a stated order with stated ties | a greedy scheduler |
| **move** | A planned movement of an event kind on a stated day, with named legs in any unit, declaring what running state it reads; real movements are rows of an `observed` input | an append-only ledger entry with a type |
| **scenario** | A file of ordinary versions layered over the reality from a date, changing nothing before it, compared to it by a keyed difference | a feature branch that is never merged unless promoted |
| **ask** | A question at a stated law date: values, partial answers, path statistics, scores, roots, as-known views, what-ifs, differences, readers, explanations, checks | a query |

**How they fit.** Facts come from sources. They enter as rows in versions, and inputs read those rows. Things of declared kinds enter with lifespans. Defs compute from inputs, bands and other defs. Moves book movements between places, using defs for their amounts; observations of real movements are booked where a statement covers a place. Rules about kinds of movements and things reach them through bindings over the kinds, and apply by themselves. Asks read defs and balances. A version's two dates let any ask see the reality as it was known on any day.

---

## 2. Syntax

<!-- tally2: D-1, D-2, D-3, D-4, D-5, D-6, D-7, D-8, D-9, D-10, D-11, D-12, D-13, D-14, D-15 -->

### 2.1 Lexical rules

- **Files** are UTF-8. Lines end with `\n`. A body line is indented by exactly two spaces per level. Tabs are refused.
- **Identifiers** (ids) match `[a-z][a-z0-9_]*`, at most 48 characters. They are ASCII (N-1611).
- **Keys** match `[a-z0-9][a-z0-9_-]*` (`u5`, `ty2014-15`, `t4e-171`), or are integers, years (`2025`), months (`2025-03`) or days (`2025-03-31`). A token that matches the key pattern is one key, `-` included (or a date or a month-day literal, below, where its position takes one, §2.5 Keys), so binary minus is written with a space on each side (`a - b`), as the canonical form prints it.
- **Numbers.**
  - Integers: `40000`. Decimals: `1190.05`, `0.0023`. No exponent, except the step `1e-N` in a unit position (§2.3 `UNIT`, §7.2), and no thousands separators in the source text (the canonical form groups thousands with a space, `96 000`, and the loader accepts that spacing inside a number).
  - Fractions: `5/900`, allowed only in rows and only when the value doesn't terminate as a decimal.
  - **Signs.** A minus sign directly before a number where a literal stands (a row, a field, an interval, an index range, a `solve` bracket) is its sign, `-3`, `-0.0023` (§2.3 `slit`); in an expression it is unary minus.
  - **`%`** after a number is the unit 1/100: `19 %` is 0.19, `100 %` is 1 (D-14).
  - **Units** after a number are written without spaces inside them: `8 PLN/hour`, `21 %/year`, `PLN^2`, `1/year`. A `*` or `/` with a space before it is an operator, so `10 PLN * rate` is a product; a touching `*` or `/` belongs to a unit only when a unit term follows it, and is an operator otherwise (`a/b`, `x[k]*2`, `8 PLN/hour*2`).
- **Dates** are ISO: `2026-07-14` (a day), `2026-07` (a month), `2026` (a year). A quarter, which ISO 8601 doesn't write, is `2026-Q1` to `2026-Q4`, SDMX's form of a reporting quarter (calendar quarters, from January, §4.1). A month or a year literal is a **period**, never an imprecise date: an imprecise date is written as a window (§4.4). A **month-day**, a day of the year without its year, is `MM-DD`: `04-30` (§3.1). A four-digit literal is a year where a period is expected and an integer in an integer index; where both readings fit, it is refused (`LOAD-AMBIG`, D-14).
- **Strings** are in double quotes, with `\"` and `\\` as the only escapes. Polish text is allowed in strings, and only there.
- **`-`** is only minus. "Not applicable" is the keyword `none` (D-7).
- **No comments.** The only free text is `note "..."`, a row's locator string, a source's title and address, the label of a unit or an axis option, an input's `question`, a value of type `text`, and the question of a hole or the reason of an assumption.

### 2.2 Reserved words

Words of the language fall in three classes. Only the first is reserved everywhere, so the everyday words of the domain (`age`, `discount`, `debt`, `share`, `order`, `rate`, `start`, `up`) stay free for ids.

**Keywords** may not be used as an id, a key, a record field or a bound variable (`LOAD-RESERVED`), except `kind version source known governs assumed at`, which may name a record's field and so follow a `.` (§2.3 `FIELD`); an observed row's literal names its `kind:` field so (§5.19), though a movement's kind is never read as `.kind` (§2.5). They are the section kinds, the words of expressions, pins and row meta, and the heads of asks:

```
model part scenario source version unit kind axis index place input band def given fill move
ask tally note if then else and or not in for where with as when true false inf by at
law known governs assumed none never check solve diff explain inputs-of readers-of residuals
```

**Built-in names** are read as built-ins only directly before `(`, so any id, key, record field, bound variable or rule label may take one, except `unit`, which is also a keyword and so may name only a rule label. A band may not, since it is also called with `(` (`LOAD-DUP`). They are the functions, aggregates, queries and distributions:

```
sum count min max any all first last argmin argmax share mean quantile prorata flows bal value
low next covered round pow exp ln sqrt abs periodic factor dcf accrue days_between add_days
add_months add_years start_of end_of year_of quarter_of month_of day_of overlap_days pos age
age_months weekday add_business last_business is_business schedule calendar window one_of since
band_lo band_hi marginal unit minor lifespan gap observed missing normal lognormal exponential
gamma poisson binomial negbin uniform discrete cdf pdf pmf crps
```

**Contextual words** are reserved only in their own position, and are free everywhere else:
- types and their parts, in a type: `money num int bool date dates month monthday quarter year key rate text dist calendar rounding daycount roll compounding reading observed`;
- period words: an index's period kind (`day`, `month`, `year`, §4.1), the period argument of `periodic` and `factor` (`day`, `month`, `quarter`, `year`, §3.3), and a duration's period word, singular or plural (`1 day`, `5 years`, `36 months`, §4.1);
- codes, in a value of a code type (§3.1): the rounding modes `down up floor ceil half-up half-down half-even` (§7.2); the day count `act365` (§3.3); the rolls `following preceding modified_following unadjusted` (§4.2); the compoundings `simple effective discount nominal` (§3.3);
- kind clauses, in a `kind` line or body: `event certain uncertain leg attr any one signed min max priced outside owned batched takes`; outside is a clause word only after the place part of a kind line, and elsewhere the prelude kind's id, so a kind of public offices may be a subkind of outside; `prorata` at the end of a `takes` clause, not followed by `(`, is its clause word (§3.6), and elsewhere the built-in (§5.13);
- clause words, at the head of a line of their section or in their clause: `on start end use same cite enters moves covers pad anchor replace lives priced supply demand arc order ties from to amount over reads before after fulfils question prices rule beats v rest largest last paths seed path orient lower higher weight step windows each random reader partial against uncertain sample always`;
- pin words, at the head of a pin: `wording`;
- words of one construct, in it: `open` (an open index), `sha256` (an import's line), `currency minor` (a currency's declaration, §5.4), `weekends yearly added` and the weekdays `mon tue wed thu fri sat sun` (a calendar value, §4.2), `modelled` (after a balance read), `of` (a read of parts, after a balance, value or leg read, and after a reading row's keys, §3.6, §5.10, §5.19);
- the built-in fields, after a `.`: a movement's `day fulfils` (§2.5); the attributes of the prelude kind `adjustments`, `adjusts change` (§5.14.3); a `lifespan`'s `from to`; `u`, a sample's uniform, in a sample pin (§5.16); and a part's `owner batch`, in a `takes` clause (§3.6).

**Built-in ids** are top-level ids of every reality, so they follow the rules of ids: declaring one again is `LOAD-DUP`, and a bound variable with one's name is `LOAD-SHADOW`. They are the built-in inputs `opening stated account_place`, the built-in indexes `days months years bank_account` (§4.1, §5.19), the value `cut` (§4.5), and the prelude's kinds (§5.7): `places accounts cash holdings assets debts claims outside currencies events transfers adjustments pads parties`.

**Rule labels** are read only in `beats` and in `replace rule d.r`, so a label may be any word of the id pattern, keywords included (`rule first`, `rule none`).

### 2.3 Grammar (EBNF)

`NL` is a newline. `IN` is one more level of two-space indent than the enclosing line. `{x}` is zero or more, `[x]` optional, `|` choice, `"..."` a literal token.

```
file      = header {section}
header    = "model" ID NL IN "tally" INT NL IN "on" DAY NL IN "start" DAY NL IN "end" DAY NL
            {IN "use" ID "=" STR "sha256" STR NL} {IN "same" sref "=" sref NL} {IN "note" STR NL}
          | "part" ID NL IN "tally" INT NL [IN "law" NL] {IN "note" STR NL}       (law: a law part, §6.1)
          | "scenario" ID "over" ID NL IN "tally" INT NL IN "from" DAY NL {IN "note" STR NL}
section   = source | version | unitdecl | kind | axis | index | place | input | band | def
          | fill | move | ask

source    = "source" ID STR ["at" STR] NL
version   = "version" ID NL IN "cite" ID NL [IN "governs" (lspan | "always") NL] [IN "known" known NL]
            [IN "enters" DAY NL] [IN "assumed" STR NL] {IN "moves" ref "to" lspan NL}
            [IN "covers" ref [lspan] {"," ref [lspan]} NL] {IN ("pad" ref "from" ref | "anchor" ref) NL}
            {IN "note" STR NL} {IN vitem NL}
                              (governs: required unless every item carries its own time; known: required; each missing one is refused by its code, LOAD-GOVERNS or LOAD-KNOWN, not LOAD-PARSE, §5.3)
known     = DAY | "window" "(" DAY "," DAY ")"
vitem     = enter | row | "given" row | bandrow | "same" sref "=" sref
          | "replace" (enter | row | bandrow | "rule" qid "." LABEL rulebody)
enter     = qid "+" "[" key {"," key} "]" [CUR | ID] ["priced" price] ["lives" (lspan | lit ".." hole)]
                                                          (a key enters a kind, or an index)
row       = qid [rowkeys] ["of" ofrow] "=" (value | expr) rowmeta       (expr: only in a scenario's version, §6.4, LOAD-SCENARIO otherwise;
                                                          of: only on a row of opening or stated, §5.19, LOAD-PARTS otherwise)
ofrow     = sref | "(" sref "," sref ")"                      (a party, a batch, or the pair, §3.6)
rowmeta   = ["governs" (lspan | "always")] ["known" known] [instant] [STR]       (the STR is the locator)
instant   = "at" DAY ["start" | "end"] | "after" ref                  (a reading's instant, §4.6)
bandrow   = qid ["[" key "]"] ":" interval "=" (expr | hole) [STR]    (§5.9; the key for a keyed band)
unitdecl  = "unit" ID STR NL | "unit" CUR ":" "currency" "minor" lit NL
kind      = "kind" ID [":" kspec] NL {IN kline NL}
kspec     = ID ["certain" | "uncertain"]                              (a subkind of ID)
          | "place" [CUR | ID | "any" | "one"] [sign] ["priced" price] ["outside"]   (unit and sign: required, LOAD-PLACEKIND, not LOAD-PARSE, §5.7)
          | "unit" | "event" ("certain" | "uncertain")
sign      = "signed" | "min" "0" | "max" "0"
kline     = "owned" "by" ID [tieby] | "batched" | "step" UNIT
          | "takes" ID "for" ID ["v" ID] ("order" expr {"," expr} ["prorata" [tieby]] | "prorata" [tieby])
          | "leg" ID ":" (CUR | ID | "any") | "attr" ID ":" type | "note" STR
                                                          (the parts clauses: only on a place kind, §3.6, §5.7, LOAD-PARTS otherwise)
axis      = "axis" ID NL IN key STR NL IN key STR NL {IN key STR NL}
index     = "index" ID ( "=" "[" key {"," key} "]"
            | "=" ["year"|"month"|"day"|"int"] slit ".." slit ["by" INT] ["from" (MMDD | INT)]
            | ":" "open" ) NL [IN "v" ID NL] [IN "assumed" STR NL] {IN "note" STR NL}
                                                              (MMDD is "04-06", a month-day every year has, so never 02-29, LOAD-PARSE; INT is a day 1-28)
place     = "place" ID [binds] ":" ID [CUR | ID] ["priced" price] "lives" lspan NL {IN "note" STR NL}
                                                          (things the model declares, §5.10)
price     = ref | "last" "(" ref ")"                                  (a reading's carry, §4.6, §5.10)
input     = "input" ID [binds] ":" type ["reading"] ["prices" lit ["by" ref]] ["question" STR] NL
            {IN "note" STR NL}
          | "input" ID ":" "observed" NL {IN "note" STR NL}
band      = "band" ID [binds] "(" ID ":" type ")" ":" type NL {IN "note" STR NL}   (binds: one, §5.9)
def       = "def" ID [binds] ":" type "=" (expr | hole) ["at" lawdate] NL {IN "note" STR NL}
          | "def" ID [binds] ":" type ["at" lawdate] NL {IN "note" STR NL} rule {rule}
                                                          (a one-line def with notes means the long form, §2.4)
lawdate   = expr | "reader"
rule      = IN "rule" LABEL rulebody NL
rulebody  = ["beats" LABEL {"," LABEL}] ["v" ID] ["governs" lspan] "=" (expr | hole | solvex) ["when" expr]
                                        (solvex: only in a rule with v, §5.18; LOAD-ASKONLY otherwise)
fill      = "fill" ID [binds] NL [IN "v" ID NL] {IN "note" STR NL}
            IN "supply" expr ["for" bind {"," bind}] NL
            IN "demand" expr ["for" bind {"," bind}] NL
            IN "arc" expr NL IN "order" expr {"," expr} [tieby] NL
move      = "move" ID [binds] ":" ID NL IN "on" expr NL [IN "at" expr NL] [IN "for" expr NL]
            [IN "when" expr NL] [IN "reads" ref {"," ref} NL] {IN ("before" | "after") ID NL} [IN tieby NL]
            {IN "attr" ID "=" expr NL} {IN leg NL} [IN "fulfils" ref NL] {IN "note" STR NL}
leg       = "leg" ID "from" expr "to" expr "amount" expr ["on" expr]
ask       = "ask" ID ":" askbody ["for" bind {"," bind}] ["in" ID] ["with" pin {"," pin}]
            ["law" expr] ["as" "known" expr] ["windows" wchoice] ["with" "uncertain"]
            ["random" "at" rchoice] ["paths" INT "seed" INT ["path" INT]] ["partial"]
            ["orient" ("lower" | "higher")] NL
askbody   = expr {"," expr} | "explain" (ref | balx | lowx) | "inputs-of" ref | "readers-of" ref
          | "check" expr "==" expr | "residuals" ref span ["by" ID] | solvex
          | "diff" "(" expr ["for" bind {"," bind}] ")" "in" ID "against" ID
solvex    = "solve" expr "==" expr "for" ref "in" "[" slit "," slit "]" "to" UNIT

binds     = "[" bind {"," bind} "]"
bind      = ID "in" (qid | "[" key {"," key} "]" | range) ["on" expr | "over" span] ["where" expr]
                                                          (qid: a member's kind or index too, k in a.ix, §6.1)
range     = expr ".." expr ["step" expr]                      (a stepped range, §5.11)
rowkeys   = "[" rk {"," rk} "]"            rk = key | "*" | key ".." key | ID "[" key {"," key} "]"
                                                          (the last: a place keyed by things, konto[ana])
value     = (slit | STR | record | "[" [slit {"," slit}] "]" | code | dist ["to" UNIT] | calval | dayset) ["assumed" STR]
          | "none" ["assumed" STR] | "never" | hole
record    = "(" field {"," field} ")"
field     = FIELD ":" (slit | STR | hole | "none" | ref | code | dayset) | leg     (named fields, never positional; dayset: a window, §4.4)
hole      = "?" STR
code      = a code word of §2.2, or "nominal" INT
calval    = "calendar" "(" "weekends" "[" ID {"," ID} "]" ["," "yearly" "[" MMDD {"," MMDD} "]" ["from" INT]]
            ["," "added" "[" cday {"," cday} "]"] ")"                      (§4.2)
cday      = DAY | hole | dayset
dayset    = "window" "(" DAY "," DAY ")" | "one_of" "(" "[" DAY {"," DAY} "]" ")"      (a window, §4.4)
span      = expr ".." [expr] | ".." [expr] | expr        (each end a date or period; ".." alone is every day, §4.1)
lspan     = lit ".." [lit] | ".." lit | lit            (a version's spans: literal dates or periods, §4.1)
interval  = ("[" | "(") (slit | "-" "inf") "," (slit | "inf") ("]" | ")")
pin       = pref "=" expr | "sample" pref "=" expr | "law" expr
          | "wording" expr "for" "[" ID {"," ID} "]"
          | "known" expr | "scenario" ID | ID "=" key                      (the last: an axis option)
                          (law, wording and known take a date; a pin's expr is evaluated unpinned, §5.11)
pref      = ID [["." ID] "[" pk {"," pk} "]"] {"." FIELD}   pk = expr | "*"
                          (*: every key at that position; a.x[k]: a member's keyed item, as in ref)

type      = "money" CUR | "num" [unit] | "int" | "bool" | "text" | "date" | "dates" | "month"
          | "monthday" | "quarter" | "year" | "key" ID | "rate" | "dist" [unit] | "calendar"
          | "rounding" | "daycount" | "roll" | "compounding" | "(" FIELD type {"," FIELD type} ")"
unit      = (uterm | "1" "/" uterm) {("*" | "/") uterm}   uterm = (CUR | ID | "%") ["^" INT]   (no spaces inside, §2.1)

expr      = "if" expr "then" expr "else" expr | or
or        = and {"or" and}       and = not {"and" not}       not = ["not"] cmp
cmp       = sum [("==" | "!=" | "<" | "<=" | ">" | ">=") sum | "in" sum ["on" sum]]
sum       = prod {("+" | "-") prod}          prod = unary {("*" | "/") unary}
unary     = ["-"] post
post      = atom {"[" expr {"," expr} "]" | "." FIELD} ["." FIELD "of" ofq] ["to" UNIT] ["as" "if" "{" pin {"," pin} "}"]
                                                          (of after a field: a leg's amount, m.cash.amount of p, §5.14)
atom      = lit | key | ref | call | agg | query | balx | lowx | valuex | flowsx | coveredx | crps | distfn | dist
          | lastx | prorata | CUR | code | record | STR | "none" | "never" | "cut"
          | "(" expr ")" | "[" [key {"," key}] "]"                     (STR: a text literal, §7.1)
agg       = AGG "(" [expr "for"] bind {"," bind} ")" [tieby]
          | ("first" | "last") "(" bind {"," bind} ")" [tieby]          (no term, §5.11)
tieby     = "ties" "by" (post | "(" expr "," expr {"," expr} ")")   (a tuple compares in order, §5.11)
AGG       = "sum" | "count" | "min" | "max" | "any" | "all" | "argmin" | "argmax"
query     = ("mean" | "share") "(" expr ("for" bind {"," bind} | ["where" expr]) ")"
          | "quantile" "(" expr ("for" bind {"," bind} | ["where" expr]) "," expr ")"
                                    (with for: over keys, §5.11; without: a random value's query, §5.16)
balx      = "bal" "(" ref ["," [expr ","] expr ["start"]] ")" ["of" ofq] [proj | "modelled"]   (bal(x), bal(x, d), bal(x, u, d))
ofq       = ref | "(" expr "," expr ")"                        (a party, a batch, or the pair, §3.6)
lowx      = "low" "(" ref "," span ")" [proj]
proj      = ["from" post] ["windows" wchoice] ["random" "at" rchoice] ["with" "uncertain"]
wchoice   = "each"            (§4.4)            rchoice = "mean" | "quantile" "(" expr ")"     (§5.16)
valuex    = "value" "(" ref "," expr ")" "in" CUR ["by" (ref | "last" "(" ref ")")] ["of" ofq]
lastx     = "last" "(" ref "," expr ")"                                (a reading's carry, §4.6)
flowsx    = "flows" "(" ref "," span ["," "for" expr] ")" ["in" CUR "at" ref MODE]
coveredx  = "covered" "(" ref "," span ")"                         (§5.19)
crps      = "crps" "(" (expr | expr ["weight" expr] "for" bind) "," expr ")"
distfn    = ("cdf" | "pdf" | "pmf") "(" expr "," expr ")"
prorata   = "prorata" "(" expr "," expr "for" bind "," expr "," UNIT ","
            ( MODE "," "rest" ("none" | "last") | "rest" "largest" [tieby] ) ")"
dist      = ("normal" | "lognormal" | "exponential" | "gamma" | "uniform" | "poisson" | "binomial"
          | "negbin") "(" expr {"," expr} ")"
          | "discrete" "(" expr "for" bind ")"
call      = (ID {"." ID} ["[" expr "]"] | BUILTIN) "(" [expr {"," expr}] ")"   (a band, keyed or not, with its prefixes; BUILTIN: §2.2,
            except the names with a production of their own: the aggregates and queries, bal, low,
            value, flows, covered, crps, cdf, pdf, pmf, prorata, the distributions, last; min and max
            also take two or more plain arguments, §2.3 Calls)
ref       = ID [["." ID] "[" expr {"," expr} "]"] {"." FIELD}
                          (a.x[k], bal(a.p[k], d): a keyed item of a member, read from outside, §2.5)
sref      = ID {"." FIELD | "[" lit {"," lit} "]"}      (a kind, thing, row or movement of this model or a member,
            its keys written as literals: `a.konto`, `a.a_stmt[a_g1]`, `b.rent[2026-03]`, §6.1)
qid       = ID ["." ID]
FIELD     = ID | "kind" | "version" | "source" | "known" | "governs" | "assumed" | "at"
            (a record field or a built-in field after ".", §2.2)
lit       = a number with an optional unit (`19 %`, `21 %/year`, `8 PLN/hour`), a date, a month-day
            (MMDD, §3.1), a key, a duration (an int and a period word: 5 years, 1 day), or true / false
slit      = ["-"] lit                                              (a signed literal)
CUR       = 3 upper-case letters, or "XXX" for a stated abstract unit (N-203)
UNIT      = a positive lit (a step), "1e-" INT, "minor" "(" CUR ")", or a ref (§7.2)
MODE      = a rounding code, a ref to an input of type rounding, or an axis whose options are rounding codes (§5.5)
LABEL     = a word of the id pattern; keywords allowed (§2.2)
```

**Seven parse rules.**
1. **Comparisons don't chain.** `a < b < c` is refused (`LOAD-PARSE`). The `==` of `check` and `solve` counts: `check a == b == c` is refused.
2. **`as if` binds to the reference before it.** `x[t] + y as if {…}` applies the pins to `y` only; write `(x[t] + y) as if {…}` for both.
3. **A value runs to its clause word.** A rule's value runs to `when` at bracket depth 0, a short def's value to `at` (other than the `at` of a converted `flows`, `in CUR at`, and of a `random at` clause), an aggregate's term to `for`, a query's argument to `where`, a leg's `from` place to `to` and its `to` place to `amount`. A word right after `.` is a field, never a keyword or a clause word, so `a.adjusts.cash.to` is one reference and doesn't end a leg's place. A type runs to its line's clause word: in `input x: num question "…"`, `question` ends the type and is no unit (likewise `reading`, `prices`, `lives`, `priced`). Each of these words is a keyword or a clause word in that position, so no value is split by guessing (D-14; tally 1's parse rule 1, the longest prefix before ` = `, is gone).
4. **`in` after `flows` and an ask's body.** After `flows(…)`, a currency code after `in` makes the conversion and anything else the membership test (`value(…)` always converts), and keys or fields after its `by p` belong to p (`by px[month_of(d)]`). The printer parenthesises a converted `flows` or `value` that is an operand of a comparison. On an ask line, `in ID` after the body and its `for` names the world (§5.18), so a membership test that ends an ask's body is written in parentheses.
5. **`on` after a membership test** belongs to it: `x in wrapped on d` tests that x is a member of `wrapped` on day d (§5.7).
6. **Bindings in an aggregate.** Inside an aggregate's parentheses, `ID in …` at a binding's place (the start, after `for`, or after the comma that ends a binding) is a binding, never a membership test: `min(k in ks)` is the aggregate, and `count(k in a, j in b)` binds two. Inside a binding's `where` it is a membership test (`where not x in outside`).
7. **A clause belongs to the nearest read.** A `windows`, `random` or `with uncertain` clause directly after a projection read (`bal(…) from t`, `low(…)`) belongs to that read, not to the ask line, and a `ties by` directly after an aggregate belongs to it, so an aggregate that ends a fill's order list is parenthesised. `from` and `ties by` take one term, so `from (d + 1)` is written with parentheses. An `of` belongs to the read its production gives it (§3.6): a balance's comes before its projection clauses (`bal(x, d) of p from t`), a value's after its `by` (`value(x, d) in PLN by px of p`), and a leg's directly after the leg's field (`m.cash.amount of p`); a writer who means another read parenthesises.

**Calls.** A call takes a built-in's name (§2.2) or a band's id, with the prefixes that name the band outside its member or its base: `a.b(x)` for a member's (§6.1), `household.b(x)` in a scenario's expression row (§6.4). There are no user functions: a def is read with brackets at its keys, a def keyed by a stepped range included (`net_of[g]`, §5.11). `min` and `max` also take two or more plain arguments, `min(a, b)`, besides the aggregate form `min(t for k in IDX)`. A period word is accepted as the period argument of `periodic` and `factor` (`factor(i, 1, year, effective)`).

**Value first** (D-14). A construct that gives a value states the value first, then its bindings (`for`), filters (`where`), condition (`when`) and dates (`at`). The forms that give no value keep their order: a bare binding (`count(p in persons on d where c)`, `first(s in sales over y)`), `if c then a else b`, and headers, which are fields.

Expressions nest at most **8** deep (`LOAD-DEPTH`). The depth at a point is the number of brackets `( )`, `[ ]` and `{ }` open there (calls, keys and `as if` included) plus the `if`s it is inside; a chain of operators at one level (`a + b + … + j`) adds none. `as if` nests at most **4** deep (`LOAD-ASIF`).

### 2.4 Canonical format

The loader accepts any spacing that the grammar allows. It then reprints the model as follows, so equal models print identically (N-1902).

- **Header first:** `model`, then `tally`, `on`, `start`, `end`, `use` lines (by name), `same` lines (by left side), notes.
- **Sections in this kind order:** `source`, `unit`, `kind`, `axis`, `index`, `place`, `input`, `band`, `def`, `fill`, `move`, `ask`, `version`.
  - Within a kind, sections are sorted by id (ASCII), except that `kind` sections print in tree order: each kind is followed by its subkinds (and theirs, depth first), kinds with the same parent (or no parent) sorted by id (ASCII).
  - **Versions come last**, sorted by `known` date (a window by its first day), then id. A file's versions therefore read like a log, newest at the bottom, and an amendment usually appends at the end.
- One blank line between sections, always, one-line sections included.
- **Inside a section:** its clause lines and notes first, in the grammar's order (§2.3), then rules (sorted by label, then by the `known` date of their `v`, a rule without `v` first, then by their own `governs` as a version's rows are, below; a `v` that names no version is refused, `LOAD-NAME`, §5.11) or version items. Version items are sorted: entries into kinds and indexes first, then by target id, a `same` item by its left side (as an item whose target and keys are its left side's), then keys in index order (things and an open index's keys in the text order of their stable ids, §2.5; a key its index doesn't hold is refused, `LOAD-KEY`, §5.6), with a keyless row first, then by the start of their own `governs` span or their instant (a row with no span of its own first, then a span with no start, a row governing `always` among them, then by start; a row read `after m` after the dated ones of its target and keys); a band's rows stay ascending (§5.9). At one key position the wider key comes first: `*` before every other key, and a range before the keys and the ranges it holds (otherwise a range sorts at its first key); a place keyed by things (`konto[ana]`) is one key, sorted as things are. Two rules, or two items, that every key above leaves tied (two rows with no start, two rows read `after m`, two rows with one start, a `same` item and a row of its target and keys) sort by their own canonical line, compared as ASCII (GD-28).
- **Clause order** is the order in the grammar. A `beats` list is sorted by label, with ", ".
- **Short form:** a def with one rule labelled `base`, no `v`, no `when`, no own `governs`, no notes prints as `def x: T = expr at E`. A one-line def followed by `note` lines means the long form, the same def with its one rule labelled `base` and those notes, and prints as it: the short form with notes is accepted input, and the canonical form of every text is as before (R-11).
- **Numbers** print as the shortest exact decimal, with a leading `0` before the point (`0.05`) and thousands grouped by a space (`96 000`, `1 190.05`), or as a reduced `a/b` when the value doesn't terminate. A value of the unit `%` prints with it. Dates print in ISO, a quarter as `2026-Q1` (§2.1).
- **Expressions:** one space around binary operators and after commas, none inside brackets. Parentheses appear only where precedence needs them, except that a `… to u` call used as an operand and a converted `flows` or `value` inside a comparison (parse rule 4) are always parenthesised.
- **Names (a style rule, not a refusal):** an index and a kind of nouns take plural names, because each is a set of keys (`months`, `persons`, `sales`); a kind named by an adjective keeps it (`wrapped`, `ordinary`). The spec's examples follow it, and the prelude's kinds are named so (§5.7).

### 2.5 Names and stable ids

- **One namespace** holds every top-level id of a reality, across all its files: sources, versions, units, kinds, axes, indexes, places, inputs, bands, defs, fills, moves and asks. Two top-level items with one id are refused (`LOAD-DUP`), but two scenarios over one base never meet, so each may declare an id the other declares (§6.4). This makes "a new file silently overrides an id" unrepresentable.
- **An imported model's items** are read as `a.item`, where `a` is the name its `use` line gives (§6.1). A keyed item read from outside puts its keys after the qualified id, `a.x[k]` (§2.3 `ref`).
- **A stable id is the item's id.** A key's stable id is `index:key`, a thing's `kind:key`, a movement's `move:keys` (an observation's `input:rowid`, a pad's `pads:<version>:<x>@<instant>`, `ENGINE.md` §6.1); a row's is `version:target[keys]`, followed by the row's own `governs` span, its instant, or a band row's interval, when its version holds more than one row of that target at those keys (`forecast_2026:inflation 2028..`); a rule's is `def.label`, followed by its own `governs` when it has one. A change a version V makes to a booked plan leg is `adjustments[m[keys].leg, V]`, one per changed leg, so a settlement names it and the name holds across runs (§5.14.3; P-8, P-17). Ids are never reused for something else. Renaming is a new item plus an end to the old one (§6).
- **The text order of stable ids.** A rule reads no order of things or of an open index's keys (§5.7, §5.6), but the engine needs one where no rule reads it: the canonical sort of a version's items (§2.4), the order fixed uniforms are assigned in (§5.16), the explanation order and the order of ask lines (§5.18, `ENGINE.md` §10), and the movement order that breaks the residual ledger's last ties (`ENGINE.md` §4.1, §6.3). There, and in **binding order**, the order an aggregate's or an ask's bindings run in (§7.5), things and an open index's keys go in the text order of their stable ids, compared by code point, as ids are (§2.4); a list or a range index keeps its own order. No rule can read this order, so it never decides a value a rule computes (P-7): it fixes printing, explanation and which fixed uniform goes to which random fact, so that a run is reproducible. `first` and `last` over things, movements or an open index's keys run in their `ties by` order instead (§5.11).
- **Bound variables** (`k` in `k in persons`) are local to their item. A bound variable with the same name as a top-level id, or with the spelling of a key of the index or kind it ranges over, is refused (`LOAD-SHADOW`).
- **Rule labels** are local to their def. They are read only in `beats` and in `replace rule d.r`, and they may repeat other names, keywords included (§2.2).
- **Record fields** are local to their record type and are read only after a `.`. A movement's fields are its legs, its declared attributes and the built-in fields `day` and `fulfils` (§5.14). Its kind is not a field: membership, subkinds included, is tested with `m in K` (§5.7). An observed row still states its kind, in its `kind:` field (§5.19).
- **Keys.** A key literal is resolved by its position: in `limit[u5]` it must be a key of `limit`'s first index or kind. A bare key used elsewhere (`k == u5`, `a == vpi`) takes the index, kind or axis of the other operand. A key literal in a value position (a rule's value, an `if` branch, a band row that returns a key) takes the index of the declared type of the value it gives (`key IDX` of the def or band); the branches of an `if` take the type of the whole. A key literal that could belong to two indexes, kinds or axes in one position is refused (`LOAD-KEY`), and so is a key literal that is also a top-level id. At a position keyed by a stepped range (§5.6), a key is a value of the range's type, written or computed, and must be one of the range's values, a + i·s ≤ b for a whole number i ≥ 0. Load refuses what it can see, as everywhere (R-load): a literal key where the range's ends and step are literals is `LOAD-KEY`; any other is checked per value where it is read, `RUN-KEY`, and a `fulfils` or a `same` side at a key the range doesn't hold names an instance that doesn't exist, `RUN-RECORD`.
- **Bare names** resolve in this order: a bound variable in scope, then a top-level id, then a key. A bound variable over a key list (`k in [1, 12, 13]`) takes its index from the key positions it is used in; two different indexes there are `LOAD-KEY`.

---

## 3. Types and units

<!-- tally2: D-1, D-7, D-11, D-12, D-13, D-14, D-15 -->

### 3.1 The types

| Type | Meaning | Literal |
|---|---|---|
| `money CUR` | A whole number of minor units of currency CUR. Every currency a model uses is declared with its minor unit, `unit PLN: currency minor 0.01` (§5.4); nothing is taken from an outside list (P-14). `XXX` is a stated abstract unit, declared like any other, for sources that give no currency (N-203). | `1190.05 PLN`; finer than the minor unit is refused (`LOAD-PRECISION`) |
| `num U` | An exact rational with unit U. `num` alone is dimensionless (a ratio, a share, a probability). | `0.0585`, `5/900`, `19 %` |
| `int` | An integer. | `12` |
| `bool` | `true` or `false`. | `true` |
| `text` | A name or a note held as data. Nothing computes with it but `==` and `!=`. Its literal states a fact, so it stands in a row, a pin or an ask line, never in a rule or a def's `as if` (§7.1). | `"Bank Pocztowy"` |
| `date` | A day, or a **window**: a day not yet known, inside a stated set, `window(a, b)` (any day from a to b) or `one_of([d1, d2])` (§4.4). Any `date` may be a window: a row's value, an attribute, a planned movement's day, never an observation's `on` or its legs' (§5.19). | `2026-07-14`, `window(2026-11-26, 2026-11-28)` |
| `dates` | A finite list of days. | `[2026-01-01, 2026-01-06]` |
| `month`, `quarter`, `year` | A period value. | `2026-03`, `2026-Q1`, `2026` |
| `monthday` | A day of the year without its year (13 April). `02-29` is no month-day, since not every year has it (refused, `LOAD-PARSE`). Month-days compare with `==` and `!=` only; any order or arithmetic on them is refused (§8.4). `day_of(y, md)` is its day in year y, and a calendar's `yearly` entries are month-days (§4.2). | `04-30` |
| `key K` | A key of index K, or a thing of kind K (§5.7). | `u5` |
| `rate` | An interest rate, with its period in its unit: `21 %/year`, `0.5 %/month` (§3.3). | `5.85 %/year` |
| `dist U` | A distribution of a quantity of unit U (§5.16). | `normal(4 %/year, 9 %)` |
| `calendar` | A business-day calendar (§4.2). | `calendar(weekends [sat, sun], …)` |
| `rounding`, `daycount`, `roll`, `compounding` | A named convention, one of the codes of its family in §2.2. A convention is a value, never a default (P-14): an input can hold it, a row can be `assumed`, a source can state it. | `half-up`, `act365`, `following`, `nominal 12` |
| `(f1 T1, f2 T2, …)` | A record. Fields are named in every literal and read as `x.f1`; a positional record is refused (`LOAD-FIELD`). A field may hold a hole or `none`. A literal states every field of its type (`LOAD-FIELD`), so a field not known yet is a hole and one that doesn't apply `none`. A leg is a field only of an observed row (§5.19), never of a record (`LOAD-FIELD`). Two records are never compared: `==`, `!=` and orders on them are refused (§8.4), and their fields are compared instead; a test against `none` stays (below). | `(share: 60 %, base: 12 000.00 PLN)` |

A position that takes a code takes any expression of the code's type, such as a read of an input that holds one (`MODE`, in `flows … at` and `prorata`, takes a code word, such a read or an axis of rounding codes, §2.3, §5.5). A code load can't see (§8.3) is checked where it is read, and one of another family is refused there (`RUN-DOMAIN`).

**A day key where a date is expected** (tally 2.1, R-14). Where a `date` is expected (an operand of a comparison, `min` or `max` with a date, an `if` branch whose other branch is a date, a date argument), a key of a day index may stand, and means `start_of(k)`: a day has one date. A day index is `days` or a range of `day` periods without `by n` (§4.1). A key of a month or a year index, or of a day range `by n`, doesn't convert, since `start_of` or `end_of` is then a choice: it is written, and the mixed form stays refused (`LOAD-UNIT`).

**The absences are three** (P-9). Every value of any type may also be:
- a **hole** `?"question"`: a fact that is missing, which a row could give; only a hole spreads (§8.1);
- **`none`**, not applicable: there is nothing here, as for a key outside its thing's lifespan (§4.6). It can only be tested, with `x == none` or `x != none`; any other comparison, arithmetic, ordering or aggregation of it is refused (`RUN-NONE`). A row, a field or a rule may give `none`: producing it is not reading it. A def whose winning rule gives `none` is not applicable at that key, and prints `none`.

A value of type `date` may also be **`never`**: later than every date. Ordering against it is allowed (`d < never` holds); arithmetic and aggregation on it are refused (`RUN-NONE`). A deadline that doesn't exist is a row `never`.

`first`, `last` and `next` over a binding that is empty at the cut and whose span reaches past it give a hole naming the binding (`HOLE-NOTYET`, §8.1), since a later row may bring a member.

### 3.2 Units and the unit rules

A **unit** is a product of base units with integer powers (P-3, P-16: units multiply and cancel as a free abelian group). Base units are currencies, the units declared with `unit` (`unit hour "godzina"`), and the keys of unit kinds (`kind funds: unit`, each fund a unit, §5.7). Units are normalised, so `PLN*PLN` is `PLN^2` and `PLN/hour*hour` is `PLN`. `%` is the dimensionless unit 1/100, a scale: `19 %` is the number 0.19, so `r / 1 %` is r as a plain number of per cent and `num %` differs from `num` only in how it prints (`19 %` against `0.19`). Durations (`5 years`, `-1 day`) carry period units.

**A unit is for a quantity every result multiplies back out**, such as hours against a rate per hour or fund units against a price per unit. A count the law divides by to get a per-period amount (instalments, months of life expectancy) is a plain `num` or `int`, not a unit.

Checked at load (`LOAD-UNIT` on any violation):

| Operation | Result |
|---|---|
| `a + b`, `a - b`, `min`, `max`, comparisons | both operands of one unit. `money CUR` with `money CUR` gives money; with `num CUR` it gives `num CUR` |
| `a * b` | units multiply. `money × int` → money. `money × num` → `num CUR`. `money × money` → `num CUR^2` |
| `a / b` | units divide. `money / int` → `num CUR`. `money / num` → `num CUR`. `money / money` (one currency) → `num` |
| int with int | `+ - *` give int; `/` gives num |
| int with num | num |
| bool in arithmetic | refused |

- **Money is a subtype of `num CUR`.** A money value is accepted wherever `num CUR` is.
- **A `num` that is an integer** is accepted where an `int` is expected, such as `round(months / months_per_year, 1, ceil)` used as a count of years. Any other num there is refused at that key (`RUN-NOT-INT`).
- **Back to money.** A `money` def and a leg's money amount must be money. A `num CUR` becomes money only through `round(x, u, mode)` with u a whole number of minor units (`minor(PLN)`, §7.2), or through `prorata` on the same terms (§5.13). A fill's supply and demand share one unit, which may be any (§5.12).
- **Inline figures** (§7.1). In defs, rules, moves and fills the only figures written inline are those §7.1 lists: `0`, `1` and `-1`, with or without a unit (`0 PLN`, `1 day`, `-1 day`, `100 %`), offsets in key arithmetic (`m - 1`) and the same integer in their guards, precision steps, key literals in key positions, a cited `solve`'s bracket and the integer of a code. A band row is a row, and holds literals of any size (§5.9). A bare `0`, `1` or `-1` takes the unit of the other operand in `+`, `-`, `min`, `max` and comparisons, and the unit of `round`'s operand where it is the step (`round(x, 1, down)` rounds to whole units of x's unit), is dimensionless in `*` and `/`, and takes the declared type of its target where it is a whole value (a rule's value, an `if` branch). Every other figure is a row (`LOAD-LITERAL`). Rows, pins (but those in a def's `as if`, which fall under the def's rule, §7.1), headers and index lists hold literals of any size, with their units: a bare literal in a row or a pin takes its target's declared type and unit (`with net_result[b] = 2000` on a `money USD` def is 2 000.00 USD, never a year). An ask holds the literals of §7.1.
- **Period units.** `year`, `quarter` and `month` are one dimension, with 1 year = 4 quarters = 12 months exactly: counts of calendar periods, whatever their length in days. `day` is a dimension of its own, so `1 day / 1 year` is refused (`LOAD-UNIT`); days become a fraction of a year only through a `daycount` code (`dcf`, `accrue`, §3.3). So `annual[y] * (1 month / 1 year)` is a twelfth, and a `num 1/year` of `21 %/year` times `1 month` is 1.75 % (a `rate` meets a duration only through the built-ins of §3.3).
- **Exchange rates** are `num A/B`: an amount of A per unit of B. `money EUR × num PLN/EUR` is `num PLN`. Converting is explicit, and adding two currencies is refused (N-202). A rate applies only from B to A (U-05): a `money A` or `num A` times `num A/B`, or a `money B` or `num B` divided by it, is refused (`LOAD-UNIT`), though the unit algebra above would give it a unit.
- **Per-unit prices of a unit kind** are `num CUR/u`, with u the key: `input px[u in funds]: num PLN/u reading`, so `px[fe] * bal(t_acct, fe, d)` is `num PLN`. A row's value has its type's unit at every key it states, so a row that leaves such a key unstated or `*` gives one unit for keys of several and is refused (`LOAD-UNIT`), unless its value is a bare literal, which takes each key's unit (Inline figures, above).
- **A key that load can't see.** A quantity of a unit kind K whose key is known only at run (§8.3) carries K at load and its key per value (P-3; Kennedy's dimension per unit kind, and the per-value currency check of a money value object, JSR 354): the amount of a leg declared `leg units: funds`, `bal(x, u, d)` at a bound or computed u, a fill's take over such legs, a price keyed by K read at a computed key. At load K matches K and each key of K, so `px[unit(t.units.amount)] * t.units.amount` is `num CUR` and a fill's supply and demand of such legs share a unit (§5.12). The run checks the keys load couldn't see, as load checks two currencies: values in two keys of K that meet in `+`, `-`, `min`, `max`, a comparison, a `sum`'s terms or a fill's take, or in a product or quotient whose load unit cancelled K (a price read at another key than the quantity's), are refused (`RUN-DOMAIN`, §8.4). A 0 with no key (a bare `0`, a sum over no keys, a fill's `taken` of a pair it never matched) takes the key of what it meets. `unit(q)` reads the key a value carries (§5.10). A declared type names a unit, never a kind (`num PLN/funds` is `LOAD-NAME`), so a figure per unit of whichever key is read at its key where it is used.

### 3.3 Interest rates

A rate is not money and not a bare number (N-206). Its value carries its period (`21 %/year`, `0.5 %/month`, `0.01 %/day`). How it compounds is a convention, and a convention is a code, never part of the rate's type and never a default (P-14): each built-in that applies a rate takes its convention as an argument, a code word or a read of an input that holds one (§3.1).

| Compounding | Meaning |
|---|---|
| `nominal m` | an annual rate compounded m times a year (m = 12 for monthly compounding) |
| `effective` | an effective rate per its period |
| `simple` | simple interest |
| `discount` | a rate of discount per its period |

- **Allowed:** `rate ± rate` of one period (a reference rate plus a margin), `rate × num`, and comparisons within one period.
- **Anything else** (`rate × money`, `rate + num`) is refused at load. A rate enters arithmetic only through these built-ins:

| Built-in | Value |
|---|---|
| `periodic(r, P, c)` | The rate per period P (`day`, `month`, `quarter`, `year`) as a `num`, under compounding c. For `month`, `quarter` and `year`, let t be P's length in r's periods (for a yearly r: 1/12, 1/4, 1). For `nominal m` it is (1 + r/m)^(m·t) − 1, which is r/m for a period of 1/m year. For `effective` i it is (1 + i)^t − 1. For `discount` d, the effective rate is i = d / (1 − d), then that conversion. For `simple` it is r × t. Any pairing with `day` other than a daily r, or a daily r over a longer period, is refused (`LOAD-UNIT` where load sees the rate's period, `RUN-DOMAIN` otherwise, §8.4): a day's share of a year or of a month is a day count, which can't be a default (§10), and `dcf` gives it. A power that isn't an integer needs `to u`. |
| `factor(r, n, P, c)` | (1 + periodic(r, P, c))^n, exact when its whole power is an integer (t·n for `effective` and `discount`, m·t·n for `nominal m`, n for `simple`, t as in `periodic`); otherwise it needs `to u` (§7.3), `LOAD-TO` where load can't show the power is an integer. |
| `dcf(a, b, dc)` | The year fraction from date a to date b under day count dc, exact. With n the days from a to b, a counted and b not, `act365` gives n/365 (Actual/365 Fixed, ISDA 2006 Definitions, section 4.16(d)). b before a gives the negative of dcf(b, a). A text that needs another day count writes a def over the two dates. A missing convention can't be written (N-207). |
| `accrue(p, r, a, b, dc, c)` | The interest on principal p at yearly rate r from day a to day b: under `simple`, p × r × dcf(a, b, dc); under `effective`, p × ((1 + r)^dcf(a, b, dc) − 1); under `nominal m`, p × ((1 + r/m)^(m·dcf(a, b, dc)) − 1). Exact where the power is an integer; otherwise it needs `to u` (§7.3). Nothing rounds: a rounding is written with `round` (D-12, C-8). |

### 3.4 Prices and price years

A money input is nominal: its value is money of the date it governs. A figure stated "in today's prices" is declared with its price year and its index (N-216):

```
input rent_wish: money PLN prices 2026 by cpi
```

Reading it at law date d gives `num PLN` = value × `cpi[d]` / `cpi[2026]`, exactly. `cpi` is an input or a def keyed by one time index (a level built from an inflation row is a def), so `cpi[d]` reads the key whose period holds d, and `cpi[2026]` the key that is the price year itself: a `prices Y by idx` input needs an index with a key for the year Y (`LOAD-KEY` otherwise). Which month, or which mean of months, stands for a year is the modeller's def over a monthly index (§10). The model then rounds where the payer rounds. A money input that names no `prices` is nominal, so "2 500 in 2028" is a nominal row governing 2028. Neither can be read as the other.

**A price year with no index.** `prices Y` alone states the year of a figure "in 2015 dollars" when the case gives no index. Read at a law date in Y it is the figure itself, in its declared type (`money CUR` for a money input). At any other date it is a hole: `<x> is in Y prices; no index moves it to <d>` (`HOLE-PRICE`). So a figure in old prices is never read as nominal in silence.

### 3.5 Parties, groups and relations

Parties are things (§5.7), each with its lifespan. The prelude kind `parties` is their root (tally 2.1, R-2): a model declares its own subkinds of it (`kind persons: parties`, `kind couples: parties`, `kind estates: parties`, `persons + [ana] lives 1984-03-02..`), or enters its parties at `parties` itself when it declares none. A person, a couple, an estate or a company is a party; an entity's members hold it through `stake` (§3.6). Only a thing of `parties` owns parts (§3.6). A kind of people outside `parties`, or a list of people as an index's keys, stays legal for everything else, but owns no parts. Everything about them is a keyed input:
- **dated facts:** `born[p]`, `residence[p, m]`;
- **relations:** `spouse[p]: key persons`, `parent[c, k]: bool`;
- **groups:** a group is a thing of its own kind with a membership input `member[g, p]: bool` whose rows govern dated spans, or a plain kind whose memberships have lifespans (`retired + [ola] lives 2031-02-01..`, §5.7).

A law reads a relation as it stood on its law date, because the relation's rows govern spans like any other row (N-404).
- One person can be in several groups, each computed on by a different law (N-405, N-410).
- "The household of X" is a def keyed by X (N-406).
- A family tree is read through relation inputs and bounded aggregates, never through open recursion (N-407).
- A fact about a person after their life (a court's confirmation of an estate) is a row like any other: a read takes the row that governs the day, even outside the lifespan (§4.6). Facts at the end of a life are read `at lifespan(p).to` (§5.7).
- **The holder is not the owner.** The name on an account is a model's own input (`holder[x]: key persons`), read by a law about the holder; a law about the owner reads the parts (§3.6). No built-in names the holder.

### 3.6 Places and ownership

Places are things of place kinds, which hold positions valued at their own units' prices (§5.10, P-3). A place is owned in one of two ways, never both:
- **by shares,** kept from tally 1: an input of fractions, `owns[p, x]: num`, whose rows govern dated spans. Each row's version holds the provenance: stated, set by a court, or derived. A kind of ownership is a key of an axis or an index (N-506). A share that moves between owners is a dated row of `owns`;
- **by parts,** at a place whose kind opts in (tally 2.1, below). `owns[p, x]` at such a place is refused (`LOAD-PARTS`): one place has one source of ownership.

**Parts** (R-1). A place kind may keep its balance as **parts**: in each unit, a finite map from a label to an amount, whose amounts sum to the balance. It is the free semimodule over labels (Amsterdamer, Deutch and Tannen, PODS 2011), fund accounting's pooled cash, and a lot inventory. The kind opts in with its clauses (§5.7):
- **`owned by D`** labels the parts by owner, a thing of `parties` (§3.5), and names D, the kind's **arrival rule** (below);
- **`batched`** labels them by **batch**, the movement that brought the units into the model's batched places (R-10);
- a kind that states both labels its parts by the pair (owner, batch);
- **`step u`** is the smallest part. Every split an arrival or a take makes is §5.13's `rest largest` to u, so a part is always a whole number of u and the parts always sum to the total. An amount to split that isn't a whole number of u is refused (`RUN-PRORATA`, as in §5.13). Money parts are money (§7.1);
- **`takes k for w …`** is the kind's **leaving rule** (below).

An owned or batched kind is `min 0` (§5.10), so a part never goes below zero and whose an overdraft is never has to be guessed; debts and overdrafts stay places that keep no parts, shared by `owns`. A place whose kind keeps no parts books as it always has, its balance one number per unit.

**The arrival rule** (R-3). D is a def keyed by a movement and a party, `def D[m in events, p in parties]: num at m.day` (a subkind at either key allowed): the share of what movement m brings into the place that is p's. The booking applies it to every amount that arrives at a thing of the kind, so the law labels each arrival by its kind of event, written once (trust principal-and-income accounting's classification of receipts; Fowler's posting rules): every leg of m whose `to` is such a place brings its amount as parts, each owner's part its share of it.
- **The shares** of one arrival are each at least 0 and sum to exactly 1, or the arrival is refused (`RUN-LABEL`, naming the movement), since a part is never below zero (§10, signed parts). Shares that are holes leave one hole part of the remainder, 1 less the known shares, whose possible owners are the parties whose shares are holes (§8.1). A known share is never lost to a hole.
- **No rule is a refusal.** An arrival the rule doesn't reach is a def with no rule that applies (`RUN-NO-RULE`, §5.11): missing law is never a hole and never a default. A transfer keeps its labels only because a rule says so: "keep the labels" and "as paid" are one rule over what m took, `m.cash.amount of p / m.cash.amount` (§5.14).
- **Its split** rounds to the step by `rest largest`. Parties whose remainders tie go by the `owned` clause's `ties by e`, whose term reads D's two bound variables, the movement and the party, by the names D's binds give them; a tie with none is refused (`RUN-TIE`, §5.13). So a kind whose rule splits an amount between owners states one. The hole part of hole shares ranks after every party, so a known share is never lost to a hole.
- **What it may read:** the movement (its kind, attributes and legs' places, and what it took, `m.leg.amount of q`, §5.14), a subkind's fields after `when m in K` (§5.7), facts and law as any def does, and parts as they stood at the start of the arrival's day or earlier (`bal(y, m.day start) of q`), so income from a co-owned asset can follow its owners' parts. The walk runs it when it books the arrival, so these are the walk's reads (§5.14.1); a read at the end of the arrival's day or later is a cycle with the booking (`LOAD-CYCLE`). Its law date is the event's: it states `at m.day` (`LOAD-PARTS` otherwise).
- **Labels are read at the ask's cut** K, as balances are, for every movement (§5.14.3).
- **Batches start here.** At a batched place, an arrival from a place that keeps no batches starts a batch, the movement m; an arrival from a batched place keeps each part's batch.
- **Labels that leave the books.** A leg from an owned place to a place of the model that keeps no parts, and isn't `outside`, drops its labels. That is the opt-in's meaning, and it is listed: `WARN-LABELS`, in the register of every answer whose reads reach the receiving place from that leg on (§8.2).

**The leaving rule** (R-4). `takes k for w [v V] order e1, e2, … [prorata [ties by e]]`, or `takes k for w [v V] prorata [ties by e]`, orders which parts an outflow takes. k is a part, read through its built-in fields `k.owner` (a party, or `none` at a place that keeps no owners) and `k.batch` (a movement, or `none` at a place that keeps no batches); w is the outflow, a movement whose kind and fields the terms read after a kind test (§5.7). A take is a fill over (part, outflow) arcs (§5.12, §5.14). `prorata` shares one outflow among its parts tied on every term, weighted by their amounts, by `rest largest` at the kind's step, its remainder's ties going by its `ties by` over k (§5.13). The tracing rules and lot booking methods are orders, and there is no default:

| Rule | Its clause |
|---|---|
| priority between funds | `order (if k.owner == ania_tomek then 0 else 1)` |
| first in, first out | `order k.batch.day` |
| in proportion | `prorata` |
| the row says whose | `order (if w in withdrawals and k.owner == w.paid_from then 0 else 1)` |

`v V` names the version whose law states the rule, and `explain` prints it; an order that changes by law reads rule-versioned defs, as a fill's does (§5.12).

**Relabelling** (R-5). A gift, a death, a divorce or an estate's division changes whose parts are: a leg from an owned place to itself takes parts by the leaving rule and returns them labelled by the arrival rule, batches kept (§5.14).

```
kind persons: parties
kind couples: parties
kind inheritances: transfers
  attr heir: key persons
kind savings: place one min 0
  owned by civ_arrive
  step minor(PLN)
  takes k for w v civ order (if k.owner == ania_tomek then 0 else 1) prorata ties by (if k.owner == ania then 0 else 1)

def civ_arrive[m in events, p in parties]: num at m.day
  rule community = (if p == ania_tomek then 1 else 0)
  rule inherited beats community = (if p == m.heir then 1 else 0) when m in inheritances
  rule moved beats community, inherited = m.cash.amount of p / m.cash.amount when m in transfers and m.cash.from in savings
```

The example labels an inheritance the heir's, money moved from another savings account as it was, and everything else the couple's; an outflow takes the couple's part first, and shares what is left in proportion.

**Net worth** is a written ask or def, the outside places left out in writing (D-1). A place is counted by its parts or by `owns`, never both, so a place held in any split counts and nothing tests a share's value (R-9):

```
def net_worth[o in parties, d in days]: money PLN = round(sum(value(x, d) in PLN by last(px) of o for x in savings on d) + sum((value(x, d) in PLN by last(px)) * owns[o, x] for x in places on d where not x in outside and not x in savings), minor(PLN), half-even) at d
```

A place counted by `owns` that has received dropped labels carries `WARN-LABELS` (above), so the figure says so. A split of a place by the origin of what flowed in, rather than by share (the units bought before and after a date), is parts (`owned`, `batched`); a fill over the acquisition movements (§5.12) or a def over the flows in by their origin stays legal (P-4).

**Entities** (a trust, an estate, a fund, a company) are parties like persons, with their own places, periods and laws (CHG-07). A stake in an entity is an input of fractions, `stake[p, e]: num`, with dated rows. A party's share of an entity's parts is `stake[p, e] * value(x, d) in C by px of e`. **Look-through** ownership is a recurrence over a bounded depth index, never open recursion (§5.11, N-403):

```
index depths = 1..4

def through[p in parties, x in places, k in depths]: num at reader
  rule first = owns[p, x] when k == 1
  rule deeper = owns[p, x] + sum(stake[p, e] * through[e, x, k - 1] for e in parties over ..) when k > 1
```

The depth is the law's own when it states one. Cross-holdings (A owns part of B and B part of A) never end, so a `check` ask compares the last two depths. A difference fails the check, and is never truncated in silence.

### 3.7 Provenance

Every row carries its provenance (P-5, P-17): its version, and through it its source, governs span and known date (the row's own, where it states them), and a locator string where it adds one (a paragraph, a page, a table cell). `explain` prints all of these for every input a result reached (§5.18, N-1701). A def's rule cites a version through its `v`; a rule with no `v` is model wiring, with no source, and `explain` shows it as such.

Asks read provenance (`explain`, `inputs-of`, `readers-of`, §5.18); rules read facts. What a rule needs about a row (a span's end) is a row of its own, a reading's age is `since` (§4.6), and a rule that holds only where a version governs and is known cites it with `v` (§5.11). A row carried past what it states is marked `held` in the register: a row with an open span read at a law date after the cut, and a row that `last(x[k], d)` reads past its span (§4.6, §8.2); a plain read past a closed row is a hole (`HOLE-UNGOVERNED`).

---

## 4. Time, knowledge and the read rule

<!-- tally2: D-3, D-4, D-6, D-7, D-12, D-13, D-14 -->

### 4.1 Spans and periods

Periods are generated by rules over a stated run, and spans over them are half-open (P-15).
- **A span** `a..b` over periods names every period from a through b, inclusive. As time it is the half-open interval [start of a, start of the period after b). So `2015-11..2016-10` is [2015-11-01, 2016-11-01), and `2021` alone is [2021-01-01, 2022-01-01). `a..` is open at the end, and `..b` has no start: read by `flows`, it reaches before S, where `flows` is a hole (`HOLE-PRERUN`, `ENGINE.md` §2). `..` alone is every day.
- **Span ends are expressions** of a date or period type, so spans built from keys are written directly: `start_of(year_of(m))..day_of(m, pay_dom)` is a year to date, and `flows(konto, m)` reads one month. In a version's `governs`, the ends are literals (§7.1).
- **The run.** Every model states `start` and `end`, S and E (§5.1): all of Tally is about time, and a single-date case states its frame, for example one year (D-13, LC-11).
- **Built-in period indexes.** `days`, `months` and `years` are the calendar periods that meet [S, E]; a partial period at either end is a whole key. They are the run's calendar, which moves and most keyed values use, and they are written once, in the header. A model declares any other index: history or law calendars outside the run (`index contribution_years = year 1990..2025`), weeks (a week's first day is a choice), quarters (`index qs = month 2026-01..2027-12 by 3`), anchored periods.
- **A quarter** (`2026-Q1`, or a value of `quarter_of(d)`, §4.2) is a calendar quarter, counted from January: `2026-Q1` is [2026-01-01, 2026-04-01). It is a value, never a key: an index's period kinds are `day`, `month` and `year` (§2.3), so an index of quarters is a range of months by 3 (`index qs = month 2026-01..2027-12 by 3`, `by n` below), whose keys are months.
- **Index ranges** of periods: `index ms = month 2027-01..2059-12`, `index tys = year 2010..2036`, `index ds = day 2026-01-01..2026-12-31`. A four-digit range with no period word is refused where an integer range would also fit (`LOAD-AMBIG`, D-14); an integer range says `int` (`index cohort_years = int 1977..2026`).
- **`by n`.** A key of a range `by n` stands for the n periods from it to the next key: `day 2026-01-05..2026-12-28 by 7` gives weeks. On an integer range, `by n` only steps the keys.
- **Anchored periods** (N-306). `index fiscal_years = year 2014..2025 from 04-06` makes key 2014 the span [2014-04-06, 2015-04-06), a year from 6 April. `index nov_periods = year 2015..2016 from 11-01` gives periods from November to October. `index assessment_months = month 2018-05..2019-04 from 17` gives assessment months that start on the 17th. A key prints as the year or month in which its period starts.
- **A one-off period** (N-307) is a key of a list index plus two inputs that give its start and end.
- **Durations** are an integer and a period word, `5 years`, `36 months`, `-1 day`. Inline, only the durations of §7.1 (`1 day`, `-1 day`, …) are allowed; any other is a row.

### 4.2 Calendar built-ins

| Built-in | Value |
|---|---|
| `start_of(k)`, `end_of(k)` | the first day of period k; the first day after it. Spans are half-open (§4.1), so k's last day is `end_of(k) - 1 day` |
| `year_of(d)`, `quarter_of(d)`, `month_of(d)` | the calendar year, quarter or month that contains date or period d |
| `day_of(m, n)` | day n of month m, clamped to the month's last day. Of a key of an anchored index (§4.1) it is refused, and so is `schedule`, at load where load sees the index (§8.3, §8.4), otherwise at the key (`RUN-DOMAIN`): the period's n-th day is `add_days(start_of(m), n - 1)`, the calendar month's `day_of(month_of(start_of(m)), n)` |
| `day_of(y, md)` | the day of month-day md (§3.1) in year y, y a year value or a key of a year index. Of a key of an anchored year index it is refused as `day_of(m, n)` of an anchored key is |
| `days_between(a, b)` | b − a in days (tally 1's `days`, renamed so that `days` names only the built-in index) |
| `add_days(d, n)` | d moved by n days |
| `add_months(d, n)`, `add_years(d, n)` | the same day number, clamped to the last day of the target month. `add_years(d, n)` is `add_months(d, 12n)`. n is an int or a duration of its period |
| `overlap_days(a, b, k)` | the number of days x with a ≤ x ≤ b that lie in period k (any part of a day counts, N-308) |
| `age(b, d)` | completed years from birth date b to date d. The anniversary of 29 February in a common year is 1 March. **Before b it is `none`.** The US rule, where a person born on 1 January reaches an age the day before, is written `age(b, add_days(d, 1))` |
| `age_months(b, d)` | completed months: the largest n with `add_months(b, n)` ≤ d; `none` before b |
| `weekday(d)` | 1 for Monday to 7 for Sunday (ISO 8601) |
| `pos(k)` | a key's 1-based position in its index |

**A month-day** (`04-30`) is a value of type `monthday` (§3.1), a day of the year without its year. A day a source fixes in every year is an input of that type, with rows (`fam1_file_by = 04-30`) and pins (`with fam1_file_by = 05-31`) like any other, and `day_of(y, fam1_file_by)` is that day in year y.

**Calendars and schedules** are generated by rules with stated coverage (P-15). A business-day calendar is a value of type `calendar`, held by an input whose rows govern spans:

```
input len: calendar
version len_rules
  cite synthetic_calendar
  governs 2020..2035
  known 2020-01-01
  len = calendar(weekends [sat, sun], yearly [01-01, 01-06, 05-01, 05-03, 08-15, 11-01, 11-11, 12-25, 12-26])
version len_2026
  cite journal
  governs 2026
  known 2025-12-01
  replace len = calendar(weekends [sat, sun], yearly [01-01, 01-06, 05-01, 05-03, 08-15, 11-01, 11-11, 12-25, 12-26], added [2026-06-22])
version len_ahead
  cite modeller
  governs 2036..2060
  known 2025-12-01
  assumed "the rules of 2035 carried forward until each year's calendar is published"
  len = calendar(weekends [sat, sun], yearly [01-01, 01-06, 05-01, 05-03, 08-15, 11-01, 11-11, 12-25, 12-26])
```

- **The value.** `calendar(weekends [...], yearly [...] [from Y], added [...])` lists the weekdays that are never business days (`mon tue wed thu fri sat sun`, ISO 8601's days 1 to 7; another word is refused, `LOAD-PARSE`), the month-days that are holidays every year (values of type `monthday`, §3.1; from year Y, if stated), and the days added in the row's span. A year whose days differ from the rules in any other way is stated by a version that replaces that year's row, as `len_2026` does.
- **A day not yet fixed.** A day in `added` may be a hole (`HOLE-CALENDAR` where it decides a result) or a window (`one_of([…])`, `window(a, b)`): every day known but one (`O:A-3`). Such a window is one the line meets, so a result that depends on its day is `HOLE-IMPRECISE` unless the line states `windows each` (§4.4).
- **Coverage.** A calendar's coverage is the span of the row that states it; a day outside it is a hole naming the calendar and the year (`HOLE-CALENDAR`), never a weekends-only guess. A calendar's years past the published ones are rows marked `assumed` (a forecast, §4.5), as `len_ahead` is; the published row replaces each when it is known.
- **The day asked.** Every calendar read reads the calendar at the day it asks about: `add_business(d, n, len)` reads `len`'s row governing d (so it needs no law date, §4.3). The calendar a business-day function takes is therefore a read of a calendar input, at any keys (`cal[country[p]]`); any other calendar expression (a def, an `if`) is refused (`LOAD-UNIT`).

| Built-in | Value |
|---|---|
| `is_business(d, cal)` | whether d is a business day of calendar cal |
| `add_business(d, n, cal)` | n business days after d (before it, for negative n); n is at least 1 in absolute value, counting from the day after (or before) d, so a holiday d needs no roll. `n = 0` is refused (`RUN-ZERO`): rolling a day is `schedule`'s job. The n-th business day of a period p is `add_business(start_of(p) - 1 day, n, cal)` |
| `last_business(p, n, cal)` | the n-th last business day of period p |
| `schedule(p, e, roll, cal)` | the day of period p given by e (an int day of the month, p a month; a day in every year is `schedule(month_of(start_of(y)) + (pay_month - 1), pay_dom, roll, cal)`), moved by the roll when it isn't a business day (ISDA 2006 Definitions, section 4.12): `following` to the next business day, `preceding` to the one before, `modified_following` to the next unless that is in another month, then the one before, `unadjusted` not at all. e is an expression: a day stated by a source is a row (§7.1). A day that doesn't exist in p (31 February) gives `none`, no date that period, which a reader tests (`RUN-NONE` otherwise), and a move whose day is `none` has no instance at that key (§5.14). A schedule that gives no date in the run is refused at load where load sees it (`LOAD-SCHEDULE`, §8.3); where it doesn't, each period whose day doesn't exist gives `none`, as above |

**Periods and keys.**
- **Key arithmetic** works on ordered indexes: `y - 1`, `m + 12`, `k - 1`, and `g + 1` for the next key of a list. A result outside the index is refused at that key (`RUN-KEY`) where it stands in a key position. In a value position a key of an integer index is an int, so `k - 1` at the first key is plain int arithmetic.
- **Keys as values.** A key of an integer index is also an `int`, and an int in a key position of an integer index is that key. A key of a year, month or day index is a value of that period type, so `y + 1`, `y > first_yr[l]` and a comparison with a `month` input work; a key of a year, month or day range `by n` with n above 1 stands for its n periods (§4.1), so it is a period, never a law date: `at k` with k such a key is refused (`LOAD-LAWPERIOD`, §4.3). A value of type `key K` (an input or def typed so) takes part in key arithmetic and comparison as a key of K, and may stand in a key position (`std_ded[filing_status[t]]`). Keys compare in index order; things of a kind and the keys of an open index have no order a rule can read, and compare only by `==` and `!=` (§5.7, §5.6).
- **Differences.** `month`, `quarter` and `year` are one dimension of counted periods (§3.2), so two values of one period type, keys or not, differ by an int of periods, as two keys of one ordered index do (the difference of their positions, for an integer or period index of the integers or periods): `m - month_of(d)` is the int number of months from `month_of(d)` to m, so `month_of(d) + (m - month_of(d))` is m. Two dates differ by `days_between`.
- **Date arithmetic:** `date ± duration` moves by that duration; `date ± int` moves by days. A period moves by an int of its own periods (`month ± int` by months, `year ± int` by years, a key of an anchored year index by its years) or by a duration that is a whole number of them (`y + 5 years` is `y + 5`, `m + 1 year` is `m + 12`); a period plus any other duration (`m + 10 days`) is refused (§8.4).
- **Lists.** `[]` is the empty list: a `dates` row with no days is `x = []`. A calendar with no added days leaves `added` out.

### 4.3 The law date

A def is evaluated at a **law date**: the day whose law applies. Which day's law applies is written, never inherited (P-6, D-4): whatever reads a law-dated fact states its law date, or declares that it takes its reader's, and every chain of readers ends at a written date. A fact is **law-dated** when its value can change with the law date: an input, band, def or rule with a row or version whose span isn't `always`; a def that states its own `at e` dates its own reads, so its readers need no law date for them (below). Two reads aren't law-dated, since their own day picks the row: an input keyed by one period index, read at its key's period (§4.6 step 1), and a calendar read by a business-day function or `schedule`, which reads the row governing the day it asks about (§4.2). So for a def:
1. a def that reads anything law-dated states `at e`, any date expression of its keys, inputs and defs, or **`at reader`**; the keys and days of a read are read as any other;
2. else, if it reads nothing law-dated, it needs no law date: an input whose rows all govern `always` (`due_day = 2029-04-30 governs always`) is read without one, by a def as by a move, and a later version that gives it a dated row makes such a reader state `at` (`LOAD-LAWDATE`);
3. else the def is refused at load (`LOAD-LAWDATE`).

Tally 1's rules 2–4 (a time key sets the law date, the reader's law date is inherited in silence, an ask's cut is the law date at its top) are gone.

**Where a chain ends.** Each of these is P-6 applied once:
- **`at reader`** declares in the def's header that it takes its law date from whoever reads it: a written dynamic scope, an implicit parameter with a static type. The loader checks that every chain of readers ends where a law date is written, at an ask's `law D` or a movement's `at` (`LOAD-LAWDATE` otherwise). One expression may still give different values to different readers, and the header says so. The value is memoised per (keys, law date), and `explain` prints the law date it ran at.
- **An ask line** that reads a law-dated input or band directly, or reaches an `at reader` def, states `law D`, a date expression: the line is its reader (`LOAD-LAWDATE` otherwise). `law cut` is allowed, and is then written, not defaulted. A read through a def that states its own `at` needs none, unless that `at` itself reads something law-dated, which is read at the line's law date (`at e`, below).
- **A movement's law date is its `at`** (§5.14), required whenever the move reads anything law-dated. It dates everything the move reads: its keys' `where`, its `on`, its `when`, its legs and its attributes. The `at` ends every chain, so it reads nothing law-dated itself (`LOAD-LAWDATE`). Which key tuples are instances is read at the ask's cut, and their amounts at the booking cut (§4.5).
- **A key binding's `where`** (`def x[k in K where c]`, `move m[k in K where c]`, a fill's or an input's keys) belongs to its item: it is read for each tuple at the law date the item is read at there, a def's `at` (its reader's for `at reader`), a move's `at`, and its reader's for an input or a fill.
- **Inputs and bands** have no rules of their own: they are read at the law date of their reader (M-10).

**A period is not a law date.** `at m` with m a month, or a key of a year, month or day range `by n` with n above 1 (§4.2), is refused (`LOAD-LAWPERIOD`). Write `at start_of(m)`, `at end_of(m) - 1 day` (m's last day, §4.2) or another day of m. A value combined over the days of m, each under the law of its day, is a def keyed by days, each day at its own law date, summed over m:

```
def charge_day[d in days]: money PLN = round(base[month_of(d)] * day_rate, minor(PLN), half-even) at d
def charge[m in months]: money PLN = sum(charge_day[d] for d in days where month_of(d) == m)
```

`charge` reads only a def with its own `at`, so it states none.

**`at e`** is evaluated at the law date the def's own reads need (its keys, and the inputs and defs it names, read under these same rules); the reads inside e are made at the law date of the read that asks for the def (its reader's, or the ask's `law`), never at e's own value, so `at day_of(m, surprise_dom)` reads `surprise_dom` once, as its reader would. An `at` that gives a window (§4.4) reads the def once per **law-date class** the window overlaps (`ENGINE.md` §3.2), each at its first day inside the window. A stretch over which the rows and rules the def reaches don't change is one class even where no row covers it, so a window inside such a stretch is read once and gives one hole, naming that day. If all give one value, that is the value; otherwise it is `HOLE-IMPRECISE`, unless the line states `windows each` (§4.4). An `at` that is a hole makes the def a hole at those keys, carrying the `at`'s questions, and its rules aren't evaluated.

**A commencement** that rules test (a provision in force from a day a later act may move) is a row of its own, and every rule that tests it reads that row; a postponement is a new version of the row.

**Pins.**
- Inside `as if {law D}`, every `at reader` def is read at D, and an input or band read directly in the ask too. That is the law that governed D, as known at the cut: a version known later that governs D retroactively is read too.
- **The wording in force on D.** `as if {wording D for [x, y]}` keeps every law date and restricts the named items' candidates (rows, band tables, rules through their `v`) to those that entered into force by D (§5.3; §4.6). So "the current act, with the scale in its wording of 30 June" is `wording 2022-06-30 for [scale]`. `law D` and `wording D for` together give the act in its wording of D, applied at D. `wording` always names its items. A `for` list may name an item that has no candidate under the pin; it reads as any item with no candidate does (§4.6).

### 4.4 Windows: a day not yet known

A day not yet known is a **window** (P-9, D-7): `window(a, b)`, any day from a to b, or `one_of([d1, d2, …])`. A month or year literal is a period, never an imprecise date; "some day in 2019" is written `window(2019-01-01, 2019-12-31)`. A window is a `date` value, not a type of its own (§3.1): a `date` input's row may state one, `x = window(2019-01-01, 2019-12-31)`. A movement may be dated by a window (a pay that lands on one of the last three business days), and so may an attribute (the day a giver bought a lot). An observation that fulfils the movement fixes its day (§5.19).

- Arithmetic on a window gives an interval of days.
- A comparison is **decided** when the whole interval lies on one side. Otherwise it is a hole: `?"date within … not precise enough for <the comparison>"` (`HOLE-IMPRECISE`).
- **A value compared with itself** is decided. Two reads of one item at the same keys, law date, cut and pins are one value, whatever variables bind the keys, so `x < x` is false and `x == x` and `x <= x` are true even when x is a window. Equal intervals from different reads stay undecided.
- **Built-ins** on a window (`age`, `add_months`, `year_of`, `overlap_days` and the rest) give the interval of their values at its first and last day. When the two are equal it is that value. A business-day function over a calendar whose `added` holds a window (`schedule`, `add_business`, `last_business`, `is_business`, §4.2) is different: the calendar, not the argument, is not yet fixed, and the function isn't monotone in that day, so it is evaluated for each day the window can take, and its value is the interval of those results (one value when they agree; for `is_business`, `HOLE-IMPRECISE` when they differ). `weekday` is 1..7 on any window of seven days or more.
- **Only a comparison decides an interval.** Wherever one value is needed (a print, a money amount, a key, a term of an aggregate other than `min`, `max`, `argmin` and `argmax`, which order their terms by these comparisons), an undecided interval is `HOLE-IMPRECISE`.
- **A movement dated by a window** counts in a span when its whole window lies inside it, and not at all when it lies wholly outside. When it crosses an end of the span, `flows` over that span is a hole (`HOLE-IMPRECISE`), and so is `bal(x, d)` for a d inside the window.

**One rule for every window a line meets** (SF-6). A read whose value depends on where in a window the day falls is a hole, `HOLE-IMPRECISE`, unless the line states **`windows each`**. That holds for every window, wherever the line meets it: a movement's day, an attribute, an `at` (§4.3), a version's `known` (§4.5), a day in a calendar's `added` list (§4.2).
- **`windows each`** prints one line per maximal sub-window, or combination of sub-windows, over which the answer is constant, and one plain line when the answer doesn't depend on the day. Where the windows a line meets depend on the day an earlier window takes (`b[a]` with a a window), the combinations nest: under each sub-window of the earlier window, the windows met there are split in turn.
- The clause stands on a projection read (`bal(x, d) from t windows each`, §5.14.1), for that read, or on an ask line, for every window the line meets through the whole walk. An ask's clause, like its pins and its `as known`, sets the world the line is evaluated in and is printed with it: every def has one value in that world, whichever def reads it.
- A text that takes a window as one day pins it to that day with a `with` pin (§5.18), which the register prints (`pinned:`, §8.2).

The cost is two endpoint evaluations per operation, with no enumeration of days except a calendar's `added` window (one evaluation per day it can take), and one evaluation per law-date class for a window `at`. `windows each` costs one run per sub-window combination it finds, bounded by the windows' days.

### 4.5 Knowledge

**Every version has a `known` date:** the day the reality learned its contents (enactment, publication, the day a statement arrived), its transaction time beside the valid time its spans state (P-5, D-3). A row may state its own `known`, later than its version's, so one source restated over time is one version with dated rows. `known` may be a window when the day is known only to lie in it ("known by 1 November": `known window(2026-10-15, 2026-11-01)`); an as-known read on a day inside the window then states `windows each` or is `HOLE-IMPRECISE` (§4.4). This holds for everything the version or row brings: its rows, band tables and rules, its `moves` lines (the span in force is undecided), its observations, and its entries, whose things are unknown members of every binding over their kinds, as keys whose `where` is a hole (§5.11). There is no `known always`.

- **Law** is known from the day its text was made public in the form that binds, not the day it enters into force and not the day the modeller fetched it.
  - Where a law binds only once published, that is its publication in the official journal (Dziennik Ustaw, Journal officiel, BGBl., Staatsblad, BOE, the Federal Register for a regulation). Otherwise it is its assent, approval or making (Royal Assent, a Public Law's approval, an S.I.'s made date).
  - Guidance, rulings and releases are known from their first public release: a dated issue of the authority (a notice, a ruling, a revenue procedure, a circular, a press release). A document the authority keeps up to date (form instructions, a leaflet, a manual, a guide, a web page) is an other source, dated as below.
  - A date that can't be found takes the latest it can be (the entry into force, or the earliest document that cites the text), and the note says so.
  - A treaty published after it entered into force is unseen by views between the two: the reality couldn't know it yet.
- **Which act.** A row or rule cites the act that gave the cited text its present wording. The cited text is the smallest unit the locator names (a point, a sentence, a subparagraph); for a band it is the table, which one version states whole. A restatement as a whole counts; a restructuring or renumbering that keeps the wording doesn't. Where that act isn't established, the version takes the latest amendment in the section's history, and its note says so. Where the enacting text wasn't available (neither in the case's sources nor found by the modeller), a restating text (a consolidated text, a code, a brochure, a guide) may be the source, with a note naming the acts it stands for (§5.3). It is known at the enacting act's known date, by the rules above, where it names that act (a source note, a footnote); a signature date it records identifies the act, whose publication is then found from its record. Else it is known at its own date.
- **Other sources** are known from their earliest **public** stamp: the earliest day by which they were surely public in the form read. A **public stamp** is a date the source prints (its publication, issue or last update; a month, an issue's months or a year counts as the period's last day), a server's date for the file (an HTTP Last-Modified), or a dated archive capture. The day the source was retrieved is the latest it can be. So a report dated by a month with no other stamp takes the month's last day, or the retrieval day if that is earlier. No view sees a source before it was surely public in that form.
  - **A file's own stamp** (a PDF's creation or modify date, XMP or document metadata, an office file's save date, a page's own meta tags) can precede public release by days, so it counts only where the source has no public stamp at all. Then the version takes it if it is earlier than the retrieval day, and is marked `assumed` with the reason (§5.3), so `explain` and the register of every result that reads the version show that its date rests on the file's own stamp (§8.2).
  - **The form read is the text read.** Another file of the same edition that holds the same text (the PDF of a web page, an archive capture) is evidence of it, by its public stamps.
  - An **updated stamp** is a date the source prints as its last change, else a public record of it (an HTTP Last-Modified, a dated archive capture). A printed stamp beats a record. A manual-wide stamp serves a page with none of its own.
  - A source with a later updated stamp takes the stamp, unless the rows read (for a version with no items, the text its rules follow) are known to predate the update, by an earlier capture or another file of the edition. Then it takes its own date, or, undated, the earliest public stamp of that evidence.
  - A label naming the period a document serves (a form's year, an exam sitting, an edition "for 2025") isn't its date.
  - The version's note says which date it took and why.
- **A later text read for an earlier period** (a 2022 text standing for an unfetched 2020 one) gives a version marked `assumed` (§5.3), citing the later text and governing the earlier period, with a note naming the text it stands for. Its rows, and the rules that point at it with `v`, are assumed. Where only some rows stand in, those rows are marked instead. Such a version is known, and enters into force, at the dates of the text it stands for where they are found, since that text's contents were public then; else at the later text's.
- **A hypothetical case's stipulations** (who, when, how much, in a worked example or a composed case) are known from the earlier of the model's `start` and its `on`: they are the case's world, given as though known all along. A modeller's assumption that stands for a case's own fact takes the same date.

Two clocks read it:
- **The ask's cut K.** An ask sees the versions and rows known on or before K. K is the run's stated cut, the model's `on` date (§5.1); an ask's `as known D` states another (N-705, N-1219). A version or row known after `on`, or whose `known` window ends after it, is refused (`LOAD-KNOWN`): the reality can't know its own future, and `on` is its today, by which it holds the source. The cut is readable as `cut`, so a pin or a def can use `min(d, cut)` (D-3).
- **A movement's booking cut.** A movement is booked with what was known on its own day, cut min(t, K) (`ENGINE.md` §5).

**A scenario's versions** are ordinary versions with their own `known` (§6.4, D-10).

**A fix of a wrong record** (a mistyped payslip, a revised measurement of a past quantity) is a later version with the same `governs` and a later `known`, whose row says `replace` (§4.6 step 2); no flag marks it. Where it changes an instance already booked, the change is an adjustment (§5.14), settled by a movement the writer dates on the instance's own day, which re-books the past in place (D-6, LC-23). Late-known law or stipulations are not fixes: their `known` follows the rules above.

**`assumed`** marks any figure no source states: a forecast (a calendar's years past the published ones, §4.2), or a fact the source leaves out (a year a leaflet omits, an election a table presumes), and a convention the source doesn't name (a rounding mode). An assumption is knowledge like any other, visible from its version's or row's `known`.

### 4.6 The read rule

Which row a read takes is written, never inferred from order (P-5, D-3, D-12). One rule says it, for rows of inputs, givens of defs, band rows and dated rules, and it is the same rule at load and at read: load refuses what the rule leaves tied (steps 2 and 4), and a read takes what it leaves. To read item x at keys k, with law date d and cut K:

1. **Candidates.** A row is a candidate if:
   - every key it states equals the corresponding key in k (a `*` matches every key; a range `a..b` matches every key from a to b in index order, on any index but an open one, which has no order (§5.6), and a range whose ends are out of order is refused at load, `LOAD-KEY`);
   - a key it leaves unstated, or covers with `*`, is one its index or kind had on the row's `known` date. A row for every key, stated before a key was added, doesn't cover that key (§6.2);
   - its `known` (its own, else its version's) ≤ K;
   - its governs span contains the period of x's key when x is an input keyed by one period index, at any law date, since the key is the value's valid time; otherwise it contains d (a rule, a band, and an input keyed by no period index or by more than one, are tested at the law date d, §4.3). A span contains a period when it contains the period's first day, so `2025-04-06..2026-04-05` governs key 2025 of `year … from 04-06`. The row's own span, when it has one, replaces its version's wholly and may reach past it. `always` contains every day. A version's span is the one in force under K: the latest `moves V to SPAN` known by K replaces V's span (§5.3). An observation (§5.19) passes this test at every law date: its version's span says what the statement covers, not when the row applies;
   - inside a `wording D` pin that covers x, it entered into force by D (§4.3, §5.3).
2. **Replacement.** Rows at the same keys are rows that state their keys alike: the same literal, the same range or `*` at each position. Among candidates at the same keys whose spans overlap, a row that says `replace` wins on its span over the rows known before it; a later row that overlaps an earlier-known row at the same keys without `replace` is refused at load (`LOAD-OVERLAP`), and so is a `replace` with no earlier row to replace (`LOAD-REPLACE`), except a `replace rule` naming a label its def doesn't have, which adds that rule (§5.11). A scenario's `replace` replaces its base's row at the same keys on its span whatever the two rows' `known` dates, since a branch stands above its base (§6.4).
3. **Precedence.** Among the remaining candidates, prefer in turn:
   1. inside a scenario, the row from the higher layer (§6.4);
   2. the narrower row, by containment key position by key position: at every position its key is the other's or lies inside it (a literal inside a range that holds it, a range inside a wider one, anything inside `*` or a trailing key left out), and at one position strictly inside. So `[t1_15, *]` beats a keyless row, `[t5_023, 1..60]` beats `[t5_023, *]`, and `[2026]` beats `[2025..2029]`; of two rows neither of which contains the other (`[2025..2027]` and `[2026..2029]`, or `[a, *]` and `[*, b]`), neither wins here;
   3. a row with a span over one with `always`.
4. **Ties.** Two candidates still tied are refused at load (`LOAD-OVERLAP`).
5. **No candidate.** For an input or a band, a read with no candidate is a hole, `HOLE-UNGOVERNED`, before the first row as after the last (D-7): a row could give it. Its question is the input's `question`, if it has one, otherwise `x at k: no value for <period>`, the period of x's time key; with no time key, `x at k: no value at <law date>`, and with no keys at all, `x: no value at <law date>`. Read with no law date (an input whose rows all govern `always`, §4.3 rule 2), the question ends at `no value`: `x at k: no value`, `x: no value`. For a def, a given that doesn't match goes on to its rules, and a rule label with no candidate version drops out (§5.11). This is the **gap rule**: a value exists only where a row says so (N-702).
   - **Outside a lifespan.** Where k names a thing of a kind, other than a movement (whose facts hold on every read day, §5.7), and d lies outside its lifespan (for an input keyed by one period index, the first day of its key's period, as in step 1), a read with no candidate is `none`, not a hole: nothing is expected there. A row that does govern d is read even outside the lifespan, since a fact about a thing may arise after its life (a court's confirmation of an estate) (D-1, M-1).
   - **Past the last row.** `last(x[k], d)` on a state input reads the latest row whose span began on or before d, and the register marks the value `held` (§8.2), so the carry is written where it is read.

**Readings.** An input declared `reading` holds observations at instants, not states over spans (D-3). Its rows state `at D` instead of `governs` (a row of a reading without an instant, or an instant on a row of an input that isn't a reading, is refused, `LOAD-INSTANT`); a reading of a balance (`stated`, `opening`) states its place in the day, `at D start` (before the day's movements), `at D end` (after them) or `after m` (just after movement m, for a running balance printed row by row; on a covered day, where m is modelled past and never booked, just after the observations that fulfil m, which stand in for it, on the day of the last of them, and refused where nothing fulfils m by the cut, `RUN-RECORD`, §5.19); `at D` alone is refused for it (`LOAD-INSTANT`). A reading `after m` with m an observation or a plan is refused at load, which sees m's class (`LOAD-ORDER`): the batch has no order (§5.14.2, P-7). The refusal is by class, so it holds for an observation booked at a decision's point too; a reading after it names that decision (above). A reading of a place of unit `any` is one row per unit.
- A plain read of a reading, `x[k]` at law date d (or `bal`'s anchor at an instant), takes the reading at that instant; with none there it is a hole naming the reading and the day (`HOLE-NOREADING`). A reading is never carried in silence.
- **The carry is written where the reading is named.** `last(x[k], d)` takes the last reading on or before d, and `since(x[k], d)` is the number of days from that reading to d. With no reading on or before d, both are holes (`HOLE-NOREADING`). A bound on a reading's age is a rule that tests `since`:

```
def quote_now[d in days]: num at d
  rule base = last(quote, d)
  rule stale beats base = ?"the last quote is older than the bound" when since(quote, d) > max_age
```

**Reading at a date.** An item keyed by one time index, read with a date or a period of another granularity in that key position (`cpi[d]` for a day d, a monthly price at a day), reads the key whose period holds it. A date outside the index is `RUN-KEY`. A period longer than the index's keys holds no key: it is refused (`LOAD-KEY`). A period no longer than the keys that no key's period holds (it lies across two keys, as a calendar month across the months of an anchored index) is refused at load where load sees the period and the keys (`LOAD-KEY`, §8.3), and otherwise at that value (`RUN-DOMAIN`).

**What follows from the rule:**
- **A later version overrides an earlier one only on its own span, and says so with `replace`.** A law change, a correction, a new job or a stopped job is written as an addition (N-107).
- **Rows are open-ended only when written so.** `governs 2026-01-01..` stays in force until a later row replaces it, so a law change restates nothing it doesn't change. A row with a closed span leaves later periods as holes, which is loud.

---

## 5. Constructs

<!-- tally2: every record; each subsection names its own -->

Each construct below gives its syntax, what it computes (a rule an evaluator can apply by hand), its refusals, its cost and the cases that use it. The case lists name the carried regression models (the 79, the household and its changes); the construct-by-case matrix in `coverage.md` is the record. The cost units are those of `PERFORMANCE.md`: one **evaluation** is one rule body computed at one key. Refusal codes are listed in §8. The examples follow the style rule of §2.4 (plural names for indexes and kinds).

### 5.1 The header

<!-- tally2: D-9, D-13 -->

```
model ul05
  tally 2
  on 2016-03-01
  start 2015-11-01
  end 2016-10-31
  note "Zasiłek rodzinny, okres zasiłkowy 2015/2016, złotówka za złotówkę."
```

- `tally 2` is the language version (§9). It is required.
- `on` is the model date: the reality's today. It is required. It is the run's stated cut K; an ask's `as known` states another (§4.5). It is never a law date of itself: an ask that reaches an `at reader` def states `law D` (§4.3).
- `start` and `end` are the run's first and last day, S and E. They are required in every model (LC-11, D-13): a single-date case states its frame, for example its year. The built-in indexes `days`, `months` and `years` are the periods that meet [S, E] (§4.1).
- **A joint model** (D-9) adds `use` lines, each importing a closed model pinned by its hash, `use a = "kowalski.tally" sha256 "9f1c…"`, and `same` lines, each stating that two kinds, two things, two rows or two movements of the members or the joint model, of one sort as §6.1 states, are one, `same a.ola = b.ola_k`, `same a.a_stmt[a_g1] = d.d_stmt[d_g4]` (§6.1).

**Refusals:** `LOAD-HEADER` if `tally`, `on`, `start` or `end` is missing, or `start` > `end`. The header is checked before the grammar, so a missing header line is `LOAD-HEADER`, not `LOAD-PARSE`. `LOAD-PIN` for an imported file whose hash doesn't match. **Cost:** none. **Cases:** all.

### 5.2 source

```
source rm1238 "Rozporządzenie Rady Ministrów, Dz.U. 2015 poz. 1238" at "https://isap.sejm.gov.pl/..."
source modeller "The model's own assumptions, each stated with its reason"
```

A source is a document. Versions cite it, and `explain` prints it. **Refusals:** a source that no version cites gets `WARN-UNREAD`. **Cost:** none. **Cases:** all.

### 5.3 version

<!-- tally2: D-3, D-6, D-8, D-12 -->

```
version rm2015
  cite rm1238
  governs 2015-11..
  known 2015-09-07
  allowance[u5] = 89 PLN "§ 2 ust. 1 pkt 1 lit. a"
  allowance[u18] = 118 PLN "§ 2 ust. 1 pkt 1 lit. b"
  criterion = 674 PLN "§ 1 ust. 1 pkt 1"
```

A version is a dated, sourced batch of facts and rules' dates.
- It has one source (`cite`), a `governs` span, one `known` date (or window, §4.5) and an optional `enters` date. It may have `moves` lines, an `assumed` line, a `covers` line, and `pad` or `anchor` lines.
- **`governs` is required** on a version that holds any row or rule without its own time (D-3). A fact that holds for all time is written `governs always`: it governs every day and is never `held`. Rows that carry their own time (a lifespan's `lives`, a reading's instant) take no `governs`, so a version of only such rows has no `governs` line. A row may state its own span (`governs 2026-03..`), which replaces the version's wholly; so may a rule (`rule base governs 2028.. = …`, §5.11). Two items of one label with overlapping spans in one version are refused (`LOAD-OVERLAP`); rows of one target in one version are ordered by precedence (§4.6 step 3), and two it leaves tied are refused (`LOAD-OVERLAP`, step 4).
- **`known`** is required, in a scenario's versions too (§6.4). A row may state its own `known`, later than its version's; one before its version's (a window by its first day) is refused (`LOAD-KNOWN`).
- **`assumed STR`** marks the whole version as resting on something no source states: it stands in for a text it doesn't cite (§4.5, a later text read for an earlier period), or its `known` date rests on its file's own stamp because the source has no public one (§4.5, Other sources). Every row of it, and every rule that points at it with `v`, puts the reason in the register of each result that reads it (§8.2).
- **Entry into force**, read only by the `wording` pin (§4.3), is by definition, for each row of the version, the later of the version's `known` and the start of the row's own time (P-6): its own span, lifespan or instant where it states one, else the version's `governs`. A rule that cites the version with `v` takes it likewise, from its own span or the version's `governs`. So a version with no `governs`, all of whose rows carry their own time, still enters into force row by row. **`enters D`** states another day, for every row of the version.
- **`moves V to SPAN`** postpones or extends an earlier version's commencement or end. Under cuts K ≥ this version's `known`, V's governs span is SPAN, and the rows of V that take its span enter into force from SPAN unless V states `enters` (above); under earlier cuts V keeps its own. The latest-known `moves` of V wins. A row of V with its own span keeps it. A sunset extended by a later instrument is the same line.
- **A commencement left to a later act**, "from a day to be fixed, at the latest D", is a `moves` to D: the latest it can be (§4.5). The act that fixes the day moves it again.
- **An act with several commencement dates** is one version whose rows and rules state their own spans, or one version per date, each citing the act and known the same day.
- **It may have no items.** Such a version carries a source, a span and a known date for the rules that point at it with `v`, or for a fill or an index that cites it.
- **A consolidated text** (a *tekst jednolity*, a republished act) restates and changes nothing. It is not a version of the rows it reprints, which come from the acts that enact or amend them; it may be their locator. Only where the enacting text isn't available may a restating text be the source (§4.5, Which act); its note then names the acts it stands for and any transitional provision it doesn't reprint.
- **`covers x [span]`** states that the version is a statement of place x for its `governs` span, or for its own span when the two differ (`covers konto 2026-09`, LC-25); covered days book observations instead of planned movements (§5.19).
- **`pad x from y`** or **`anchor x`** says what the gaps of x's balance readings in this version are: taken up by a movement of the prelude kind `pads` from y, or restarting the walk from the reading. A version that states balance readings (`stated`) of a place says one of them for it (`LOAD-GAPROLE`, D-8).
- Its items are:
  - **input rows:** `x[keys] = value [governs span] [known D] ["locator"]`, or for a reading `x[keys] = value at D [start|end] ["locator"]` (§4.6);
  - **band rows:** `b: [lo, hi) = expr ["locator"]`;
  - **givens:** `given d[keys] = value …`, a row aimed at a def. It pins the def at those keys, and the def's rules aren't evaluated there (§5.11);
  - **entries:** `persons + [ola] lives 1990-02-01..` enters a thing into a kind with its lifespan (§5.7); `funds + [fe] lives 2010-01-01..`; a place of a kind of unit `one` states its unit, `accounts + [konto] PLN lives 2019-05-01..`; `index_id + [k]` adds keys to a list index from this version on;
  - **replacements:** `replace` before a row, a band row, an entry (a new lifespan: a death, a closed account) or a rule (`replace rule relief.disabled = disabled_relief when disabled[p]`), making a new version of that one item from this version's `governs`; an entry's new lifespan, which carries its own time, replaces the old one whole under every cut that sees the version (§5.7; in a scenario, §6.4) (D-12);
  - **identities:** `same bank[r32] = bank[r31]`, two rows that state one movement (§5.19).

A row's value, or a field of it, written `?"…"` is a stated hole: a read that takes it is a hole with that question (`HOLE-STATED`, §8.1). Rules point at a version with `v` (§5.11).

**What it computes.** Nothing by itself. It supplies the source, governs span and known date of every item in it. Those feed the read rule (§4.6), and asks as known before the version's date don't see it.

**An entry is dated twice.** A thing entered by version V is seen only by asks with K ≥ V's `known` (earlier asks don't list it, and a read of it is refused, `RUN-KEY`), and it exists only inside its lifespan (§5.7). Things are never deleted: a thing that ends gets a lifespan that ends, by a `replace` of its entry.

**Refusals:**
- `LOAD-NAME`: an item names an unknown id.
- `LOAD-GOVERNS`: a version with a row or rule that has no time of its own, and no `governs`.
- `LOAD-KNOWN`: a version with no `known`, or one known after the model's `on`.
- `LOAD-KIND`: an addition to a range or an open index.
- `LOAD-DUP`: an entry of a key that already exists in that kind or index.
- `LOAD-LIFESPAN`: an entry into a kind without `lives`.
- `LOAD-MOVES`: `moves` naming an unknown version or one not known before this one; two `moves` of one version in versions known the same day.
- `LOAD-OVERLAP`, `LOAD-REPLACE`: the replacement rule of §4.6.
- `LOAD-LABEL`: a rule whose label already exists, without `replace` (§5.11).
- `LOAD-GAPROLE`: balance readings of a place with neither `pad` nor `anchor` for it.

**Cost:** rows are grouped at load per (target, keys) and sorted by span, so a read is O(log r). **Cases:** all. Rows with their own `governs`: CB-03, CH-05, CMP-01, LH-05, LH-07, MP-09, MP-11, RC-03, RC-08, UL-05, UL-06, UL-08, UL-13, the household, change 1, change 4, change 5. `enters`: RC-08, UL-04, UL-15. `moves`: RC-01, RC-05. Entries: change 1, change 4, change 5, change 6, the household.

### 5.4 unit

<!-- tally2: D-12 -->

```
unit hour "godzina"
unit PLN: currency minor 0.01
unit JPY: currency minor 1
```

`unit hour "godzina"` declares a quantity unit for `num` types (§3.2). `unit X: currency minor m` declares a currency and its minor unit: a key of the prelude kind `currencies`, alive always, so `u in currencies` binds over the declared currencies (D-12). No currency is known without its declaration; `minor(X)` reads its minor unit. **Refusals:** `LOAD-NAME` for an undeclared unit (a unit is a top-level id, §2.5); `LOAD-UNIT` for money in an undeclared currency. **Cost:** none (units are checked at load). **Cases:** every model with money (currencies); CB-06, CH-02, CH-04, CH-06, LH-01, LH-02, the household (other units).

### 5.5 axis

<!-- tally2: D-12 -->

```
axis ul05_cut
  floor "the cut rounds down to whole złoty"
  exact "no rounding is stated"
```

An axis lists readings no fact settles, each with a note (N-109, N-912): which text governs, which edge, a rounding the law leaves open. A convention a fact can set (a contract's day count, a payer's rounding) is a code value of an input, not an axis (D-12, `LC-24`); hypothetical facts are scenarios (§6.4).
- A rule may test it: `when ul05_cut == floor`. Several axes may be tested in one `when`.
- An input may be keyed by it, and its rows for different options may sit in different versions. **The axis id in a key position** reads the input at the current option, and fans out: `selbstbehalt[dt_u_work]`, `friend_of[cmp_spouses, i]`.
- `round(x, u, a)`, `prorata` and a conversion may take it as the mode when every option is a rounding code (§2.2); otherwise `LOAD-UNIT`. An axis whose options mix modes with readings such as `exact` is tested by rules, each calling `round` with a mode or with none. Two axes may share option names; a bare option is resolved by its position (§2.5).

**Fan-out.** An ask evaluates every combination of the axes its evaluation reads, found at run time: a line whose result read no option of an axis prints `*` for it. It prints one line per distinct result, a tuple for an ask of several expressions (§5.18). Two holes are the same result when they carry the same questions at the same keys.
- **Naming the options.** On a line, an axis prints `*` when, with the line's other options held, every option of it gives the line's result. Otherwise it prints the options that give it, joined by ` | ` (`rate_reading = down | half-up`), and the line's `reading:` entry names the same options (§8.2). An axis printed `*` has no `reading:` entry.
- **A set that isn't a product.** When the combinations that give one result aren't one option set per axis (`(x1, y1)` and `(x2, y2)` give A, the cross pairs B), the set is split by the axes in id order: the first axis's options are grouped by the set of the other axes' combinations they give the result with, each group is a line, and each line is split again on the next axis. Every line then names a product.
- `with a = opt` fixes an axis for one ask. A pinned axis is listed under `pinned:` only, never `reading:`, since it doesn't fan out.

**Refusals:** `LOAD-FORKS` if one ask reaches more than **64** combinations (after its `with` pins), counted at load over the axes its cone reaches on the item graph, whatever its `when`s read (`ENGINE.md` §3.1 step 11). A run may read fewer; the `*` above follows what it read. **Cost:** multiplies only the cone of defs that depend on the axis. **Cases:** 62 models (`coverage.md` table 1); an axis option pinned by `with`: LH-15.

### 5.6 index

<!-- tally2: D-1, D-13, D-15 -->

```
index lots = 1..12
index options = [keep, sell, wait]
index fiscal_years = year 2014..2025 from 04-06
index bank_rows: open
```

An index is a set of keys that aren't things: integers, periods, options (D-1). A list or a range is ordered; an open index isn't (below). Things with lifespans are kinds (§5.7).
- **Lists** are ordered as written, and versions can extend them (§5.3).
- **Ranges** are integers (with `by`), days, months and years, plain or anchored (§4.1).
- **Built-in indexes:** `days`, `months` and `years` of the run (§4.1).
- **An open index** gets its keys from the rows of inputs keyed by it, and has no order a rule can read: the order rows arrived in orders nothing (D-5), so a range, a key comparison, key arithmetic or `pos` over its keys is refused (§8.4), and `first` and `last` over it run in the order of their `ties by`, without which they are refused (§5.11). An order no rule reads takes the text order of its keys' stable ids (§2.5). A key exists from the earliest `known` of the versions whose rows bring it, so an earlier cut doesn't see it.

An index may carry `v ID` and notes, when its keys or their order are a source's (a workbook's closing age, the order a law takes parts in). It may carry `assumed STR` when its keys or its periods are the modeller's assumption (a fiscal year whose start no source gives); every figure keyed by it, and every result that reaches one, then lists `assumed: <index>: <reason>` in its register (§8.2).

**Bindings.** A bind is over an index or a kind (a member's included: `k in a.ix`, `p in a.persons on d`, §6.1), an observed input's rows (`o in X over s`, §5.19), a key list or a stepped range, so it is bounded at load.
- A bound that depends on an earlier key is a `where`: `k in instalments where k <= lh_terms[l].months` (LH-10). Binds are read left to right: each bind's `where` may read the binds before it, and each `where` filters at its own bind (`c in brackets where pos(c) == pos(b) + 1`).
- A bind over a kind states `on d` (the things alive on day d) or `over s` (alive on some day of span s; for an event kind, the movements dated in s; `over ..` is all of them), except the key bind of a def, input, place or fill, which is defined for every key (§5.7, `LOAD-ALIVE`).
- **A stepped range** is a bind over `a .. b` of integers, or `a .. b step s` of quantities, dates or durations (`g in 0 PLN .. gross_cap step minor(PLN)`, `n in 1 n7 .. owned step 1 n7`); a range of quantities without `step` is refused (`LOAD-DOMAIN`, D-15). It is finite by construction, so an aggregate over it is exact.

**Refusals:** `LOAD-KEY` for a key that isn't in its index; `RUN-KEY` for key arithmetic outside the index. **Cost:** O(1) per key. **Cases:** all.

### 5.7 kind

<!-- tally2: D-1, D-2, D-12; C-1, C-32, M-1, M-3 -->

```
kind persons
kind accounts_pl: place PLN signed
kind lines: place PLN max 0
kind brokerages: place any min 0
kind ordinary: brokerages
kind ira: brokerages
kind wrapped
kind notes: unit
kind sales: event certain
  leg units: notes
  leg cash: PLN
  leg fee: PLN
  attr trade_ref: text

version opened
  cite bank_letter
  known 2019-05-01
  persons + [ola] lives 1990-02-01..
  accounts_pl + [konto] lives 2019-05-01..
  ordinary + [broker] lives 2021-03-01..
  wrapped + [broker] lives 2024-01-01..
  notes + [n7] lives 2021-01-04..2031-03-01
```

The example declares people; an account kind of one unit with any sign; a kind for what is owed on a line, never above zero; a kind of accounts holding any declared unit, with two subkinds; a plain kind `wrapped` whose memberships are dated; a unit kind whose keys are note series; and an event kind with three legs and an attribute.

A kind is a declared class of things and is the set of its things: an index whose keys are things, each entering with its lifespan (P-2). The rules below follow from that.

**What a kind can be.**
- **A plain kind** (`kind persons`, `kind wrapped`): things with no place, unit or event part.
- **A place kind** (`kind K: place UNIT SIGN [priced p] [outside]`): its things hold positions (§5.10). UNIT is a currency or unit, `any` (positions in several units) or `one` (each thing states its single unit as it enters, `accounts + [konto] PLN lives …`). SIGN is `signed`, `min 0` or `max 0`. `priced p` values a place whose single position is valued by the price input p; a thing of a kind of unit `one` may instead name its own price input as it enters (§5.10). `outside` marks the world beyond the model.
- **The parts clauses** (tally 2.1, §3.6) are body lines of a place kind: `owned by D [ties by e]`, `batched`, `step u` and `takes k for w …`, in that order. A kind that states `owned by` or `batched` states `step` and `takes`, and `step` or `takes` stands only beside one of them. Its sign rule is `min 0`, and it is no subkind of `assets` (whose places hold nothing that moves, §5.10). D is a def of type `num` keyed by an event kind and by `parties` (either a subkind), in that order, at the law date `m.day` of its movement key m. A subkind of a kind that keeps parts keeps its clauses and states none of its own; a subkind of a place kind that keeps none may state them for its own things. Anything else is refused (`LOAD-PARTS`). One clause can name one arrival rule for several kinds.
- **A unit kind** (`kind funds: unit`): each key is a unit of measure, so holdings are counted in units, not money (§3.2). Declared currencies are the keys of the prelude's `currencies`.
- **An event kind** (`kind sales: event certain`): its things are movements (§5.14). It states its certainty, `certain` or `uncertain` (§5.14), its legs, each with its unit, a unit kind (`leg units: notes`) or `any`, and any attributes, `attr name: type`. It may have no legs: a notice issued before anything is paid is an event that moves nothing.
- **A subkind** (`kind ordinary: brokerages`) is a kind of its parent: its things are the parent's too. A subkind of an event kind keeps its parent's legs and attributes, may add its own, and may state its own certainty (`kind maybe_spend: own_housing uncertain`); a binding over the parent reads only the parent's legs and attributes, except where a kind test narrows it (Reading things, below).

**Things.**
- **A thing enters with its lifespan**, `K + [k] lives span`, the open end written `..` and an end not yet known a hole (`lives 1950-01-01..?"date of death"`, `HOLE-STATED`, D-7). A lifespan has a start (`lives ..b` is refused, `LOAD-LIFESPAN`), and a thing whose start no source gives states an early day: an entry in a version marked `assumed` (§5.3), or, for a place the model declares, an early `lives` in its `place` section, as `world` does (`lives 1900-01-01..`, §5.10). A later version states a new lifespan with `replace K + [k] lives span`: a death, a closed account. A thing the model itself declares, rather than a source, is declared by a `place` section (§5.10).
- **A key enters only at a kind with no subkinds** (`LOAD-LEAF`): a kind with subkinds is a class, so a thing is never silently in none of them (M-3). This is a rule of entries (`+`): a movement is of the event kind its move or row names, a kind with subkinds included (`move rent: transfers` beside `kind groceries: transfers`), and a binding over a subkind doesn't reach it.
- **One base kind, any number of plain kinds.** A thing's base kind, for life, is the kind it entered at: the kind that isn't plain, if it enters one, else the kind of its first plain entry, by `known`, then by its lifespan's start. Two entries into kinds that aren't plain, or two first plain entries tied on both, can't say where it entered, and are refused (`LOAD-DUP`: a key entered twice). Besides its base, a thing may be a member of any number of plain kinds, each membership with its own lifespan: `wrapped + [broker] lives 2024-01-01..` makes the account wrapped from that day. Two independent classifications of one thing are two memberships, and a class that changes on a date is a membership that ends as another begins (C-32).
- **Units declared with `unit X: …` live always**; keys of a unit kind entered with `+` state their lifespan.
- **A place ends only empty**: its lifespan may end only on a day when every position in it is zero, and a leg into or out of a thing outside its lifespan is refused (`RUN-LIFESPAN`), unless its amount is 0, which books nothing (§5.14), so ending a thing never makes money disappear from a total (C-1).
- **Parts and lifespans** (tally 2.1). A part labelled to a party on a day outside the party's lifespan is refused (`RUN-LIFESPAN`, naming the movement), and, as a place ends only empty, a party's parts must be gone by the end of its lifespan. That is checked after the whole order of the lifespan's last day: a death's posting reads the place, so it runs after every movement that changes the place that day and doesn't depend on it (§5.14.2), and a part labelled to the party by a movement after the posting that day is refused (`RUN-LIFESPAN`, naming that movement). An estate's lifespan starts on the death day, even when its entry is known later (a version's `known`, §5.14.3).
- **`lifespan(k)`** is a record with `.from` and `.to` (`never` for an open end), so a rule about a thing that has ended reads its facts `at lifespan(p).to`.

**Reading things.**
- **Bindings state when** (§5.6): `x in accounts_pl on d` ranges over the things alive on day d, subkinds included; `x in brokerages over s` over those alive on some day of span s. A total over places on a day therefore counts every place alive that day and no other, outside places included unless the total says `where not x in outside`. The key bind of a def, input, place or fill keyed by a kind needs no `on` or `over`: the item is defined for every key. A move's keys over a kind state it, since each books an event on a day (`move assess[y in years, p in persons on end_of(y)]`).
- **A binding whose day or span is a hole** is a hole only where a thing of the kind, known at the cut, could be alive on one of its possible days and pass the binding's `where`; elsewhere it binds nothing (a binding over `adjustments` before any exists), since a hole spreads only where the outcome could differ (P-9).
- **In a joint model**, a member's bindings range over that member's own things, the prelude's kinds included; only the joint model's own bindings over a shared kind range over every member's things (§6.1).
- **`x in K`** tests membership of a kind whose memberships aren't dated (a base kind and its parents); **`x in K on d`** tests membership on day d, for any kind, plain kinds with dated memberships included.
- **A kind test narrows** (tally 2.1, R-17; occurrence typing, Tobin-Hochstadt and Felleisen, POPL 2008). A movement or thing variable v bound over a kind P is narrowed to a subkind K of P where a written test says it is in K: in a rule whose `when` is `v in K`, or a conjunction with that test (`when v in K and …`), throughout its value and its `when`; and in `if c then e1 else e2` whose condition c is `v in K` or such a conjunction, throughout c and e1. There v's legs and attributes of K are readable. A conjunct that reads them is evaluated only where the test holds, and the conjunction is false where it doesn't. Only a written test narrows: a field of K read where no test narrows v stays refused, as in 2.0 (`LOAD-FIELD`, `<field> is not a field of <type>`), `or` narrows to neither side, and `x in K on d` narrows nothing, since a dated membership isn't a kind's legs and attributes.
- **Reads and lifespans** follow the read rule (§4.6): a governing row is read even outside a lifespan; with none, a read outside the lifespan is `none`, inside it `HOLE-UNGOVERNED`. A movement's facts (its legs, its attributes, defs keyed by it) don't lapse: an event's facts hold on every read day, and its lifespan only places it on its day for bindings (M-1).
- **Things have no order a rule can read.** A thing's key is its id, and ids never order what a rule computes (D-5, P-7), so things compare only by `==` and `!=`: `<`, `min`, `max` or any other order on things is refused (§8.4). Whatever orders things states its order: `argmin`, `argmax`, `first` and `last` over things or movements take `ties by` (§5.11), a fill states its `order` (§5.12), and a move's instances on one day go by its `ties by` (§5.14.2). An order no rule reads (printing, explanation) takes the text order of their stable ids (§2.5).

**The prelude.** Every model starts with the roots `places` (every place kind is a subkind of it) and `events` (every event kind is), the place kinds `accounts`, `cash`, `holdings`, `assets`, `debts`, `claims` and `outside` (§5.10), the unit kind `currencies`, whose keys are the declared currencies (§5.4), and the event kinds `transfers` (`event certain`, one leg `cash: any`), `adjustments` (`event certain`, no legs, the attributes `adjusts` and `change`, §5.14.3) and `pads` (`event certain`, one leg `cash: any`, §5.19), and the plain kind `parties`, the root of the model's owners (tally 2.1, §3.5), whose subkinds the model declares or whose things it enters directly. They are read like the model's own, so "every place" and "every movement" are ordinary bindings: `x in places on d` ranges over every place alive on d, and `m in events over s` over every movement dated in s.

**Equivalences of tally 1 that break** (D-1): "an index key exists on every day of the run" and "a thing that doesn't exist is a row of 0". A zero row is now a stated zero.

**Refusals:** `LOAD-LIFESPAN` (an entry without `lives`, or a lifespan without a start), `LOAD-LEAF`, `LOAD-DUP` (a key entered twice), `LOAD-ALIVE` (a binding over a kind without `on` or `over`, but the key bind of a def, input, place or fill, §5.6), `LOAD-PLACEKIND` (a place kind without its unit or sign rule, or a thing of a `one` kind without its unit), `LOAD-PARTS` (a parts clause missing or where it can't stand), `LOAD-FIELD` (a subkind's field read where no test narrows), `RUN-LIFESPAN` (parts included). **Cost:** a lifespan test is O(1); a binding `on d` is O(things of the kind) per evaluation, memoised per day. **Cases:** CMP-01, the household, change 1, change 4, change 5, change 6 (from their places and index additions, by migration); the hard set of the design round.

### 5.8 input

<!-- tally2: D-3, D-7, D-8, D-11, D-12, D-13 -->

```
input allowances[g in age_groups]: money PLN question "Jaka jest kwota zasiłku dla tej grupy wieku?"
input born[p in persons]: date
input wibor: rate
input day_basis: daycount
input fx[u in currencies]: num PLN/u reading
input growth[y in years]: dist 1/year
input len: calendar
input rent_wish: money PLN prices 2026 by cpi
input bank: observed
```

An input declares a typed fact. Its values are rows in versions (§5.3).

**Kinds of input.**
- **A state**: an input declared with neither `reading` nor `observed`. The plain form has its own meaning, as the other two have: its rows hold over spans (`governs`).
- **A reading** (`reading`): rows are observations at instants (`at D`), read with a written carry (§4.6). Prices, valuations and stated balances are readings unless a contract fixes them for a period.
- **Observed movements** (`observed`): rows are real movements with named fields (§5.19).

**Values.** A row's value is one of:
- a literal with its unit, a record with named fields, or a list (`[]` included);
- a code of the input's code type (`half-up`, `act365`, `following`, `nominal 12`);
- a distribution of the closed list (§5.16), for an input of type `dist`;
- a calendar value (§4.2);
- a hole `?"question"`;
- `none` (not applicable), or `never` for a date;
- a value with `assumed "reason"`: the model states a figure or a convention no source states (§4.5). Every result that reaches it lists the reason (§8.2);
- in a scenario's version, an expression over base facts (`household.valuation[flat] * keep_share`, the base named by its id, §6.4).

A row `x[*] = v` states every key of x that exists at the row's `known`, its own or else its version's (§4.6). A key that a version known after it brings needs its own row, since a row speaks as of its `known` (P-5, P-6).

**Reading.** Rows are read by the read rule (§4.6). The `question` is the text of the hole that the gap rule gives. For the household it is Polish (N-1001, N-1906).

**Refusals:**
- at load: `LOAD-UNIT` for a value of the wrong type or unit, `LOAD-PRECISION` for a money literal finer than its minor unit, `LOAD-OVERLAP`, `LOAD-INSTANT`, `LOAD-FIELD` (a positional record);
- per value: `RUN-NONE` for a use of `none` other than a test.

**Cost:** O(log r) per read, memoised per (keys, period, K). **Cases:** all.

### 5.9 band

<!-- tally2: D-12, D-14 -->

```
band zr(n: int): money PLN
  note "kwota zasiłku według wieku dziecka"

version rm2015
  cite rm1238
  governs 2015-11..
  known 2015-09-07
  zr: [0, 5) = levels[u5] "§ 2 ust. 1 pkt 1 lit. a"
  zr: [5, 18) = levels[u18] "lit. b"
  zr: [18, 24] = levels[u24] "lit. c"
  zr: (24, inf) = ?"zasiłek nie przysługuje powyżej 24 lat"
```

A band is a function of one argument over intervals. The argument is a number, a date, a period or a key of a list or range index, never a thing or an open index's key (`LOAD-UNIT`): a band's rows are intervals of an order, and neither has one (§5.7, §5.6).

**One table per version.** The rows of one band in one version form one table: ascending, without overlaps, and meeting without a gap, closed against open (`int` arguments are checked over the integers; a point interval `[a, a]` is a row). The first row may start at `-inf`, and the last may end at `inf`. A deliberate gap is a row whose value is a hole, which may carry a locator (N-709, N-710).

A table that breaks these rules is refused at load (`LOAD-BAND`), and with it the whole reality (a table in a scenario file: that scenario, §8): nothing prints from an inconsistent law table, asks that never read the band included. In the store an amendment that breaks loading is refused before it lands (`ENGINE.md` §11).

**Rows replaced** (P-14, P-5). A later version may replace rows of a table from its `governs`: `replace scale: (120000, inf) = …`. Its `replace` rows either all take intervals of the table they replace, and then that table stands with those rows' values replaced, or they form a whole table (ordered, without gaps or overlaps), which replaces it on the version's span with its own intervals. A version that mixes `replace` rows and plain rows of one band, or whose `replace` rows do neither, is refused (`LOAD-BAND`) (D-12, H-12). The table they replace is, on each day of the span, the one in force there by the read rule among the versions known before (§4.6), so a patch applies to the table in force on each day. On a day of its span where no table is in force, only a whole table stands, as an input's `replace` row is its value where no earlier row governs; a patch there is refused (`LOAD-REPLACE`) under every cut that sees the version, and so is a whole table that replaces no table on any day of its span (§4.6 step 2).

**Reading `b(x)`** at law date d and cut K:
1. Choose the table by the read rule (§4.6), treating each version's table as one row: its version's governs span and known date, with replaced rows applied.
2. Return the row whose interval holds x.

So a new version's table governs its span whole, and a read never mixes rows of two tables. An x outside every row is a hole, `HOLE-UNGOVERNED` (`<b>: no row holds <x> at <law date>`, or `<b>: no row holds <x>` read with no law date), and so are `band_lo(b, x)`, `band_hi(b, x)` and `marginal(b, x)` at it. A hole x makes the read a hole with x's questions only: no row is chosen, so no row's question is added.

**A row's ends.** `band_lo(b, x)` and `band_hi(b, x)` are the lower and upper end of the row of b that holds x, in b's argument type; an infinite end is `none`. They and `marginal` may be called anywhere an expression may: in a band row, a scenario's expression row, a def, a `when` or an ask. A row's own value may read them: they read the interval, not a value, so there is no cycle, and "10 800 zł + 32% of the excess over 120 000 zł" states 120 000 once, as the interval's end:

```
  scale: (120000, inf) = 10800 PLN + 32 % * (x - band_lo(scale, x)) "art. 27 ust. 1"
```

**Marginal rates.** `marginal(b, x)` reads a band whose rows give marginal rates, as a progressive scale or a tiered table states them: the sum, over b's rows whose lower end lo is below x, of the row's value × (min(x, hi) − lo). Its unit is x's times the rows'. A hole row that x passes makes it a hole; an x outside every row is a hole; a band whose first row starts at `-inf` can't be read this way (`LOAD-BAND`).

**Rows.**
- A row's value may use the argument, inputs, other bands, its own row's ends, and literals of any size: a band row is a row of a version, with its source and locator (§7.1).
- Bands are acyclic.
- A band may return any type, a key included (an age group from an age).

**Keyed bands.** `band b[k in IDX](x: T): U` holds one table per key, an axis key included, when two readings or two groups have different tables. Rows are written `b[k]: [lo, hi) = …`, the table checks run per key, and the band is called `b[k](x)`. Edges stay literals.
- In a call, k is any key expression: a def's key, a read (`rate_band[class_of[q]](x)`), or an axis id, which fans out (§5.5). The ends and `marginal` take the band with its key: `band_lo(b[k], x)`.
- Each key's table is chosen by the read rule on its own, so one key's table may sit in another version than another key's.

**Refusals:** `LOAD-BAND` (unordered, overlapping or gapped rows; `marginal` on a band from `-inf`; `replace` rows that neither match the earlier intervals nor form a whole table, or that mix with plain rows), `LOAD-REPLACE` (`replace` rows that patch no table on a day of their span, or that replace no table on any day of it), `LOAD-CYCLE`, `LOAD-DUP` (a band with a built-in's name, §2.2). **Cost:** O(log rows) per call; `marginal` O(rows). **Cases:** CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, the household. Keyed bands: MP-06, UL-03. `marginal`: CB-03, CB-04, CMP-01, MP-11, UL-09. `band_lo` and `band_hi`: CB-02, CB-04, CB-07, MP-06, MP-08, RC-08, UL-03, UL-10, UL-15.

### 5.10 place

<!-- tally2: D-1, D-2, D-8; C-1, C-9, M-4 -->

A place is a thing of a place kind (§5.7): where money or units sit (N-501). It holds positions in one unit or several, and its kind states its unit, its sign rule and how it is valued (P-3). Places enter as things of kinds, by a source's version (`accounts + [konto] PLN lives 2019-05-01..`), or are declared by the model with a `place` section:

```
place owed[p in persons]: debts PLN lives 2024-01-01..
place world: outside lives 1900-01-01..
```

which declares one place of the kind per binding tuple, with its unit and its lifespan. A unit is stated only for a kind of unit `one`, whose things each name their one unit (§5.7); a place of any other kind has its kind's unit, a stated one or `any` (as `outside`), so a unit stated on it is refused (`LOAD-UNIT`, P-3). A place keyed by things lives on the overlap of that span and its key things' lifespans: `wcash[oki]` lives from the day `oki` enters.

**The prelude** (§5.7) writes tally 1's seven place kinds in Tally, six of unit `one` and `outside` of unit `any`, so a migrated model keeps its words:

| Prelude kind | Its declaration | Holds |
|---|---|---|
| `accounts`, `cash` | `place one signed` | money; may go below zero, with `WARN-NEGATIVE` on the first day of each stretch below zero (N-1607) |
| `holdings` | `place one min 0`, each thing stating its unit and its price input as it enters (`holdings + [w1] etf_units priced etf_price lives …`) | units of one unit kind; `value` = units × p. A holding allowed to go short (N-511) is a thing of a kind declared `signed` |
| `assets` | `place one min 0`, each thing naming its unit and its price input v as it enters or in its `place` section (`assets + [flat] PLN priced flat_val lives …`) | nothing that moves; `value` = v at d, from valuation readings (N-502). A leg to or from an asset is refused (`LOAD-KIND`) |
| `debts` | `place one max 0` | money owed, as a balance ≤ 0; a balance above 0 where the sign rule checks it (below) is refused (`RUN-PLACE`) |
| `claims` | `place one min 0` | money owed to the model's own parties, ≥ 0; below 0 is refused (`RUN-PLACE`) |
| `outside` | `place any signed outside` | the world beyond the reality, unconstrained |

A model declares its own kinds whenever a rule needs to tell things apart by kind (`kind ordinary: brokerages`), rather than listing them.

**The sign rule** (`min 0`, `max 0`) constrains a place's balance at the points where the day's order (§5.14.2) gives the balance a value. Observations and plans on one day have no order among them, so a check between them would invent one (P-7). The rule is therefore checked:
1. after the day's first batch, taken together: its readings at the start, its observations (but those booked at a decision's point, §5.19) and its plans (so an amount that arises and is paid on one day nets inside it);
2. after each decision's legs, or the observations booked at its point, and after each reading that follows one, as they are booked;
3. at the close, after the readings at the day's end.

A balance that breaks the rule at one of these points is refused from there (`RUN-PLACE`), with the legs at the place that brought it there: the batch's, or the decision's. It is a constraint checked at the end of a batch whose parts have no order, as a deferred constraint is checked at the end of a transaction (ISO SQL, `DEFERRABLE INITIALLY DEFERRED`).

**Balances.** `bal(x, d)` is the balance of a one-unit place at the end of day d, after every movement of d; `bal(x, d start)` at its start; `bal(x, u, d)` the quantity of unit u in a place of unit `any`. Inside a movement, `bal(x)` with no day is the running balance at the movement's place in the day's order (§5.14). A place's balance is zero at its lifespan's start. A place alive before S needs an anchor reading (`opening[x]` or `stated[x]`, §4.6) at or before its first read; else the read is a hole naming the place (`HOLE-ANCHOR`). `flows` reads movements only, so it is not a hole for want of an anchor. A balance reading re-anchors or pads the place, as its version says (§5.3, §5.19).

**Value.** `value(x, d) in CUR [by p]` is the place's positions on day d (end of day) times their prices in CUR. A position already in CUR counts at its amount. A position in another unit u counts at u's price p, which `by p` names: an input or def of type `num CUR/u`, keyed by unit when the place may hold several units (read at the position's unit), or an expression of that type written with its keys (`by etf_price[month_of(d)]`). `by p` is required when some position of a place that isn't priced (below) is in another unit (`LOAD-PRICE`), and a price that is a reading is read through its carry, `by last(px)` (§4.6). A position with no price on d is a hole naming it and the price (`HOLE-PRICE`), never left in its own unit (P-3). `value` multiplies exactly; nothing in it rounds (C-8).
- **A priced place.** A place of a `priced` kind is valued at that kind's price input. A place's price input has one home: its kind, or the place itself (its `place` section or its entry), never two of them (`LOAD-PRICE`, R-once). A thing of `assets`, a subkind included, names its price input in one of these homes (`LOAD-PRICE` otherwise). A price that is a reading is named there with its carry, `priced last(val)`, and `value` reads it so; a plain `priced val` reads it as any reading is read, a hole on a day with no reading (§4.6).
- **Movements conserve.** Every leg names both ends, so money and units are conserved by construction and the outside is explicit. A purchase of units is one movement with a units leg (from the market to the holding) and a cash leg (from the account to the market), each in its unit (§5.14).
- **Obligations are places.** What is owed (a statement's balance, an amount collected for another and not yet passed on) is a place of a kind with a sign rule, with a balance before any payment, since what a rule carries between events is one of its places, defs or fills (§5.12, P-4). The posting records the amount owed on the day it arises (out of a `debts` place, into a `claims` place), the payment clears it, and it has a balance in between (REA's claim; LC-16, LC-21). Tally 1's `holds` and `free` are gone: a rule that must not spend what is owed reads the sum of the two balances in writing.
- **Places need dates.** A movement is a dated event, and the ledger's invariants need its date. An illustration that gives no dates writes its money as defs (MP-14, MP-15, CMP-02); where money must move, the model's dates are `assumed`.
- **Kind and unit** are read by `x in K` and `unit(x)`. `unit(x)` of a one-unit place, and `unit(q)` of a quantity, a leg's amount included, are the unit as a key of the unit kind that holds it (`currencies` for a currency, §5.7). So `unit(x) == PLN` compares two keys, lots in a place of unit `any` are matched series by series with `unit(b.units.amount) == unit(s.units.amount)` (§5.12), and a price keyed by unit is read at a leg's own key, `px[unit(t.units.amount)]`. Where load can't see the key (a leg declared with a unit kind), it is read per value (§3.2). A place of unit `any` has no one unit, and a quantity unit (`unit hour …`, §5.4), a compound unit such as `PLN/year` and a plain number are keys of no unit kind, so `unit` of such a place or quantity is refused (§8.4): at load where load sees it (`LOAD-UNIT`), otherwise at the key (`RUN-DOMAIN`), as for a 0 with no key (§3.2). Asks select places by kind or owner, never by a hand-kept list (N-1606).

**Parts** (tally 2.1, §3.6). A place whose kind is `owned` or `batched` keeps, besides its balance, its parts: per unit, a map from a label (a party, a batch, or the pair) to a whole number of the kind's step, the amounts summing to the balance at every point of the day's order where the balance has a value. Arrivals add parts by the arrival rule, outflows take them by the leaving rule (§5.14), and readings state or leave them (§5.19). A place of unit `any` keeps parts per unit, and its relabels and takes go unit by unit. A place alive before S has no known parts before its first anchor reading, as it has no balance there (Balances, above; `HOLE-ANCHOR`): its parts are one hole part, of the hole balance, possibly anyone's (§8.1), besides the parts arrivals add; a take from it reaches that hole part (§5.14), and the sign rule is not checked against a hole balance, as in 2.0.
- **Its sign rule is `min 0`** (§5.7), checked at the points above as for any place: an outflow larger than the place's balance is `RUN-PLACE`, and a part never goes below zero, so whose an overdraft would be is never guessed. Signed parts are left for later (§10).
- **`bal(x, d) of q`** is the sum of x's parts labelled q at the end of day d: q a party (every batch of it, at a place that is both), a batch (every owner of it), or a pair `(p, b)`. `bal(x, d start) of q`, `bal(x, u, d) of q` and, inside a decision, the running `bal(x) of q` read the same points as their totals do (§5.14.1); a decision that reads it declares `reads x`. **`value(x, d) in C [by p] of q`** is those parts at their prices, as `value` prices the place's positions (above). Both are 0 where x holds no part of q, a hole where x holds a hole part that may be q's (`HOLE-PART`, naming its questions, §8.1), and refused at a place that keeps no labels of q's sort, a party at a place only `batched` or a batch at one only `owned` (`RUN-DOMAIN`, or `LOAD-PARTS` where load sees the place's kind and q's kind). The total `bal(x, d)` stays as it is, known while its parts are holes.
- **A gain follows the units** (R-7; unitization). Parts are kept in the place's unit, so a rise in price changes q's value by q's units, and nothing is booked: a price is no event, and the arrival rule labels only movements. Income (interest, a distribution) is a movement, and its arrival is labelled. A law that gives a gain to another party writes it, as a relabel on its event (§5.14) or as a def over `value(x, d) … of q`. A thing that changes owners by events (a car, a flat bought with mixed money) is a holding of one unit, priced by its readings (`priced last(val)`), since `assets` places hold nothing that moves.

**Refusals:** `LOAD-KIND` (a leg to or from an asset), `LOAD-UNIT` (a leg in another unit than its places (§5.14), a unit stated on a place whose kind isn't of unit `one`), `LOAD-PRICE`, `LOAD-PLACEKIND`, `LOAD-PARTS`, `RUN-PLACE`, `RUN-LIFESPAN`, `RUN-DOMAIN` (`of` at a place without labels of q's sort). **Cost:** a balance read is O(1) during the day walk, or O(log Mₓ) at a past date with per-place prefix sums, Mₓ the place's change points (`PERFORMANCE.md` §2.2). Parts cost only where a kind opts in: one entry per live label per (place, unit), and a take or an arrival O(L log L) for L live labels. **Cases:** CMP-01, the household, change 5. The cases hold their money as defs keyed by their periods; only CMP-01 and the household move it between places.

### 5.11 def and rule

<!-- tally2: D-4, D-5, D-7, D-12, D-14, D-15 -->

```
def payable[f in families, m in months]: money PLN at start_of(m)
  note "kwota wypłacana po obniżeniu, art. 5 ust. 3"
  rule base = max(0 PLN, entitled[f, m] - excess[f, m])
  rule small beats base = 0 PLN when entitled[f, m] - excess[f, m] < min_payment

version rm2016
  cite rm2016_act
  governs 2016-04..
  known 2016-02-10
  replace rule payable.base = max(0 PLN, entitled_2016[f, m] - excess[f, m])
```

A def is a keyed value defined by rules, at a written law date (§4.3, P-6). Each rule is written value first and states its value: `rule L [beats …] [v V] [governs span] = value [when condition]` (D-14, §2.3). Steps 2 to 6 state P-14 for rules. To evaluate def x at keys k, with law date d and cut K:

1. **Givens.** If a given of x (a version row `given x[…] = …`) matches k by the read rule (§4.6), return it. A `with` or `as if` pin wins over a given. A given may state a source's own legal conclusion (a guide that says these persons qualify), citing it.
2. **Rule versions.** A rule's label is unique in its def; a new version of the rule is a version item `replace rule x.L = …`, from that version's `governs` (D-12). A rule whose label already exists, written without `replace`, is refused (`LOAD-LABEL`). A `replace rule` item belongs to the version that holds it; a `v` on it may name only that version and adds nothing (`LOAD-VERSION` where it names another version). `v V` has one meaning on every construct that takes it: it names the version whose source states the item (H-9; a `v` that names no version of the reality is refused, `LOAD-NAME`), a rule's law, a fill's order (§5.12), an index's keys or their order (§5.6). On a rule it also gates, since only a rule has something to fall back to, its def's other rules: a rule with `v V` exists only where V governs and V's `known` ≤ K (P-14). A rule may also state its own span, `governs 2028..`, as a row does (LC-6). Among the versions of one rule that exist at d and K, the replacing one wins on its span (§4.6). A rule written in its def with no `v` and no own span exists always.
3. **Applicability.** Evaluate each remaining rule's `when`: true means it applies, a hole means unknown, and no `when` means it applies. The `when`s are evaluated in `beats` order, a rule's after those of the rules that beat it, and by label among rules that `beats` doesn't order. A rule already beaten by an applicable rule (step 4) has its `when` skipped, so a read guarded by an exception is never reached.
4. **Exceptions are written pairs** (D-12). Every rule of one def is a rival of every other. `rule r beats q, …` says that r, when it applies, defeats q. The **candidates** are the applicable rules that no applicable rule beats. `beats` is not transitive: if a beats b and b beats c, an applicable a defeats c only if it says `beats c` too. One rule can't win with the help of another (there is no team defeat). An exception to an exception that only lifts it is a condition of the rule it excepts: `rule late beats base = … when overdue[h] and not minor[h]` (D-12).
5. **Unknown applicability.** If a rule of unknown applicability could change the outcome, the result is a hole that carries that `when`'s questions. It could change the outcome when it would beat a candidate, be a candidate beside the winner, or be the only rule left. Its value isn't evaluated, so it is a hole even if that value would equal the winner's.
6. **Result.**
   - One candidate: evaluate its value, and return it. Its label is kept for `explain` (§5.18). **Only the winner's value is evaluated**: a beaten rule's division by zero, `RUN-KEY`, `RUN-NONE` or `RUN-EMPTY` never happens.
   - Two candidates: refused, `RUN-CONFLICT` ("rules a and b both apply to x[k] on d"), even when their values are equal: the law's structure is stated with `beats`. Where a law's exceptions don't say which of two wins, the conflict is loud, never a silent pick (D-12's example: an exception to an exception whose middle rule doesn't apply leaves the base rule unbeaten beside it).
   - None: refused, `RUN-NO-RULE`.

A rule's value may be a hole, meaning "the law for this case is not written" (`HOLE-STATED` where the rule wins, §8.1), or `none` (§3.1). A def whose only rule is a hole is a stub: a law not yet written, or an open fact the source names, which prints its question.

**Law changes are additions** (N-104, N-107).
- A figure changes: a row in a new version, with `replace` where it overlaps the earlier row (§4.6).
- A formula changes from a date: `replace rule x.L = …` in a version that governs from that date.
- A new exception: a rule with `beats`, in the def or added with the version that brings it (the def's rules may also come from a version item `replace rule x.L` naming a new label, which adds it).

**Recurrences.**
- A def may read itself, or anything that depends on it, at an **earlier key** of an ordered index (`x[k - 1]`), or at a **later key** instead (`x[k + 1]`, a value computed back from the last key). In one def, the reads on one index all go one way; reads both ways are `LOAD-CYCLE`.
- Each such read must sit under a guard: a condition that tests k or `pos(k)`, in its own rule's `when`, in the `when` of a rule that beats it, in an `if`, or in an `and` to its left (`when k > 1`, `when pos(y) > 1`, `if k == 1 then … else …`, `k == 1 or …`, `when k == sheet4`). On a period or list index, `pos(k) > n` and `pos(k) == n` are the guards for `k - n`. `LOAD-GUARD` checks that a guard exists; whether it suffices is checked per key (`RUN-KEY`). Only a recurrence read needs a guard: a shifted read of another item (`cpi[m - 1]`) is checked per key alone (`RUN-KEY`).
- **At a computed key.** A def may also read itself, or anything that depends on it, at a key given by an expression or a binding (an earlier acquisition found by `where`, the figure of another thing a binding finds). Such a read isn't checked at load. At run time, a chain that returns to a (def, keys) still being evaluated is refused at that key (`RUN-CYCLE`). It terminates, since a chain can't be longer than the keys.
- Same-key dependencies form an acyclic graph (`LOAD-CYCLE`). The check covers every rule, whatever its `when`, so two readings that select rules forming a cycle are written as two defs. A cycle in `beats` is refused (`LOAD-CYCLE`).
- **No forced rounding.** A carried value may be an exact `num`. Every exact value is capped at **4 096 decimal digits** in its numerator and its denominator; above that it is refused at that key (`RUN-DIGITS`: "round where the law rounds").

**An iteration the law names** (N-918, UL-07) is a recurrence over a bounded index, with `first` choosing the stopping key:

```
index iters = 0..50
def d[t in filers, i in iters]: num USD at reader
  rule init beats step = d0[t] when i == 0
  rule step = step_from[t, i - 1]

def settled[t in filers]: num USD = d[t, stop[t]] at reader

def stop[t in filers]: key iters = first(i in iters where i > 0 and abs(d[t, i] - d[t, i - 1]) < tolerance and abs(c[t, i] - c[t, i - 1]) < tolerance) at reader
```

The exception is the first key: `init` applies at i = 0 and beats `step`, whose read of `i - 1` is guarded by `init`'s `when` and is never evaluated there. If no key of the index satisfies the condition, `first` is refused (`RUN-EMPTY`). An iteration can therefore never run unbounded or stop in silence. The tolerance is a row (§7.1).

**A def keyed by a stepped range** (`def net_of[g in 0 PLN .. gross_cap step minor(PLN)]: money PLN = …`, its upper end a row) is read with brackets like any keyed def and evaluated only at the keys read. There are no user functions (D-15, P-17). So a def keyed by an index of days (`days`, a range of days, or a stepped range of them) is a function of a day: it is read at any computed day of its range, `f[d]`, and a computed day outside the range is refused at the key (`RUN-KEY`; R-load, §2.5 Keys).

**Aggregates** are written value first: `sum(term for bind, … [where c])`, with the term before `for` (D-14). An aggregate runs over bindings with `where`; a binding over a kind states `on` or `over` (§5.6). Over keys, every aggregate is ordinary and exact. `mean`, `share` and `quantile` with `for` are aggregates over keys too: `mean(t for b)` the exact average, `share(c for b)` the exact fraction of bindings where c holds, `quantile(t for b, q)` type 1 (the smallest term with at least q·n terms at or below it); without `for` they are queries over a random value (§5.16).
- A bare binding takes no term: `count(p in persons on d where c)` is the number of bindings whose `where` is true, all of them with no `where`; `first(s in sales over y where c)` and `last(…)` return the key (one bind) or the tuple; they take no term (`LOAD-PARSE`), so the term at the first key is read there, `v[first(k in ks where c)]`. `argmin(t for b)` and `argmax(t for b)` return the key or the tuple, typed `key K`.
- `first` evaluates its `where` from the first binding in binding order and `last` from the last, each stopping at the first key whose `where` is true; those keys are what it read (§8.2). Over a binding that is empty at the cut and whose span reaches past it, `first` and `last` give a hole naming the binding (`HOLE-NOTYET`), since a member may still come (§3.1, P-9); any other empty `first` or `last` is refused (`RUN-EMPTY`).
- `min`, `max`, `argmin` and `argmax` order numbers, money, dates and keys of a list or range index (index order), never things or an open index's keys, which have no order (§5.7, §5.6); false < true. Windows compare by §4.4: `min` and `max` give the interval of the ends' min (max); `argmin` and `argmax` give the key whose value is decided below (above) every other key's, and otherwise `HOLE-IMPRECISE`.
- **Ties are stated** (D-5, P-7). `argmin`, `argmax`, `first` and `last` over things, movements or an open index's keys, and `argmin`/`argmax` anywhere two candidates can share a value, take `ties by e`: among tied candidates, the one with the least e; `ties by (e1, e2)` compares e1, then e2 among those still tied. A tie met at run with no `ties by` is refused (`RUN-TIE`); index order never breaks a tie. Over things, movements or an open index's keys, which have no order a rule can read (§5.7, §5.6), the `ties by` term is also the order `first` and `last` run in, and without one they are refused, at load where load sees the binding (§8.4), otherwise at the key (`RUN-TIE`).

| Aggregate | Over no keys | With holes |
|---|---|---|
| `sum` | 0, in the term's type | a hole if any term is |
| `count` | 0 | a hole if any `where` is |
| `min`, `max` | refused, `RUN-EMPTY` | a hole if any term is |
| `min(a, b)`, `max(a, b)`, the plain form | — | a hole if any argument is |
| `any` | false | true if any term is true; otherwise a hole if any term is |
| `all` | true | false if any term is false; otherwise a hole if any term is |
| `first`, `last` | a hole naming the binding (`HOLE-NOTYET`) when it is empty at the cut and its span reaches past it (§3.1); otherwise refused, `RUN-EMPTY` | a hole if a hole comes before the first key whose condition is true (after the last, for `last`) |
| `argmin`, `argmax` | refused, `RUN-EMPTY` | a hole if any term is. Ties by `ties by`, else `RUN-TIE` |
| `mean`, `share`, `quantile` with `for` | refused, `RUN-EMPTY` | a hole if any term or `where` is |

The "with holes" column covers a hole `where` as well as a hole term. A key whose `where` is a hole is an unknown member: `count` is a hole; `sum` is a hole unless that key's term is known to be 0; `min`, `max`, `argmin` and `argmax` are holes; `any` and `all` follow Kleene logic; `first` and `last` keep their rule. A hole carries the questions of every operand, `where` and term it evaluated, each once (§8.1). A term that is `none` is refused (`RUN-NONE`) unless the `where` tests it out.

**"The least x such that c"** is `min(v for v in a .. b step s where c)` over a stepped range (D-15, P-17): finite by construction, so the result is exact, and no value satisfying c is the aggregate's refusal of an empty set (`RUN-EMPTY`). The engine may bisect when c is monotone in v; that is speed, not meaning. **An inverse**, the argument that gives an observed value, is this form (or an `argmin` over the same range) with each candidate read under an `as if` pin, `min(g for g in lo .. hi step minor(PLN) where net as if {gross = g} >= target)`; its derivation is the winning candidate's. A choice among options is `argmin(cost[o] for o in options) ties by rank[o]`.

**Logic.** `and` and `or` evaluate left to right and stop early, with Kleene values (P-9): false `and` anything is false, true `or` anything is true, and otherwise a hole stays a hole. `if c then a else b` evaluates only the branch c takes; with a hole c it evaluates both, and is a hole unless they are equal. A guarded read such as `k > 1 and x[k - 1] > 0 PLN` is therefore safe at k = 1. A test for `none` is written before the use it guards: `group[p] != none and group[p] != g2` (§3.1).

**Refusals outrank holes:** when both reach a result, the result is refused.

**A law's own rule for a missing record** (N-917, CHG-11). `missing(r)`, r a read of an input (`LOAD-MISSING` otherwise), is true when the read r is a hole because no row gives it, or its row is a stated `?`, and false otherwise; it is never itself a hole. It may appear only in the `when` of a rule that points at a version with `v`, so the law that says what a missing record means is cited (`LOAD-MISSING` elsewhere). Every result that such a rule decides, where a `missing` test in its `when` read true and the rule won, lists it in its register: "missing record counted by <version>". A `missing` test that read false lists nothing of its own. Every other missing value still gives a hole.

```
def outlier[d in days]: bool at d
  rule base = pnl[d] < -var[d]
  rule no_record beats base v mar32 = true when missing(pnl[d]) or missing(var[d])
```

**`as if`** (N-907, N-908). `e as if {pins}` evaluates e in a world where the pins hold. A pin is one of:
- `input[keys] = value`: replaces every read of that input at those keys;
- `def[keys] = value`: pins the def;
- `axis = option`;
- `law D`: the law date of every `at reader` def, and of an input or band read directly, inside (§4.3);
- `wording D for [x, y]`: the named items take only versions that entered into force by D (§4.3, §4.6);
- `known D`: the cut K inside;
- `scenario s`: the world of scenario s inside (§6.4);
- `sample x[keys] = v`: a random fact's sample (§5.16).

**A pin's value is any expression** the ask or def could hold: `with parts[f] = base_parts[f]`, `with law forms_from`, `as if {known origin_day[o]}`. It is evaluated first, in the context without that `as if` (its own pins excluded), then pinned as a value. So a pin that reads its own target reads the unpinned value, and there is no cycle. A value or sample pin whose value is a hole pins that hole. A pin whose key is a hole, a `law`, `known` or `wording` pin whose date is a hole, and an ask's `law` or `as known` that is a hole make the value inside the `as if`, or the ask line, a hole carrying their questions, and nothing inside is evaluated, as for an `at` that is a hole (§4.3). The pins of one `as if` apply only inside it; nested pins don't leak to their siblings. A pin on a record field pins that field.

**A pin's keys** are any key expressions: an ask's `for` variables, a def's keys, an aggregate's bound variable (`adv[c, m] as if {mon_cut[c, m] = 0 PLN}`). The pins are formed per binding tuple, and the memo keys on them.

Everything inside is recomputed on the changed facts, gates included. Movements inside are computed but never booked, and `flows` inside sees the recomputed movements. The result combines with the real one like any value, by min, max, a cap or a difference. `as if` may nest 4 deep. A superseded law stays runnable inside `as if {law 2021-12-31}` (N-908).

**Refusals:** `LOAD-ASIF` (deeper than 4, or a pin on an unknown id). **Cost:** each distinct pin set gets its own memo, and only the pinned items' cone is recomputed.

**Cost of a def.** Per key, O(rules of the def) tests; a def in the cases has 1 to 6 rules. Values are memoised per cell (`ENGINE.md` §3.2). A recurrence over n keys is O(n) time and memory.

**Refusals of a def:** `LOAD-LAWDATE`, `LOAD-LAWPERIOD` (§4.3), `LOAD-LABEL`, `LOAD-CYCLE`, `LOAD-GUARD`, `LOAD-MISSING`, `RUN-CONFLICT`, `RUN-NO-RULE`, `RUN-TIE`, `RUN-EMPTY`, `RUN-CYCLE`, `RUN-DIGITS`, `RUN-NONE`, `RUN-KEY`.

**Cases:** all. `beats` (tally 1's `excepts`): 67 models. Recurrences, a read at k − n or k + n: CH-02, CH-06, LH-02, LH-04, LH-05, LH-07, LH-10, LH-13, LH-15, LH-16, LH-18, MP-15, RC-01, the household. Givens: CB-10, CB-11, CB-12, CB-14, LH-13, LH-17, MP-03, MP-04, MP-06, MP-07, MP-08, RC-01, RC-02, UL-13. `as if`: CB-03, CB-09, CH-04, CMP-01, LH-04, LH-12, MP-11, RC-08, UL-14, UL-15; `with` pins on asks: 24 models. `missing()`: the household. An iteration the law names: UL-07 writes its steps as defs and UL-08 uses `solve`; the form above is a recurrence with `first`, both used elsewhere (`coverage.md` table 1).

### 5.12 fill: greedy allocation

<!-- tally2: D-2, D-5 -->

```
fill absorb
  supply credit[sx, o] for sx in payers, o in years
  demand room[dx, t] for dx in payers, t in years
  arc sx == dx and t >= o - 1 and t - o <= carry_years
  order o, t ties by pos(sx)
```

Supplies are matched to demands in a stated order. This covers carrybacks and carryforwards, lot matching (lots, below), pension carry-forward by vintage, loss pools and reliefs sharing one cap (N-513, N-514, N-919).

**Its parts.**
- **Units.** Supply and demand share one unit, any unit: money, `num CUR`, units of a unit kind, or a plain `num` with none (§3.2).
- **`v` and notes.** A fill may carry `v ID`, the version whose law states the order, and notes. `explain` prints it. It doesn't gate the fill, which has one procedure and nothing to fall back to; an order that changed by law is written as an `order` term read from rule-versioned defs.
- **Sides.** `for` is optional on either side. A side without it has one value (one cap shared by every supply) and adds no keys to `taken`. A side's bindings may range over movements (`supply b.units.amount for b in purchases over ..`), as lots do. A key whose membership in a side is a hole (its binding's day, span or `where` a hole, §5.7) takes part with every arc of it a hole condition carrying those questions (Holes, below).
- **`arc`** is any bool expression, `true` included.
- **Law dates.** Each term is a read like any other: an `at reader` def among them takes the law date of the def or ask that reads the fill.
- **Keyed fills.** `fill F[y in IDX]` is one fill per key. Its terms may read `F[y - 1].taken` behind a guard (§5.11 Recurrences), so a cap carried from year to year is a recurrence over fills, not a cycle.

**Procedure.**
1. **Arcs.** Evaluate every (supply key, demand key) pair's `arc`, and collect those whose `arc` is true or a hole. A false arc is left out; a pair whose arc is a hole takes by the Holes rules below. A refusal in any arc refuses the fill at that key, with the pair named.
2. **Sort.** Sort the arcs ascending by the `order` tuple, compared lexicographically, keys in index order, false < true; arcs still tied go by the `ties by` term (D-5). Arcs still tied after it, or tied with no `ties by`, are refused (`RUN-TIE`) when their order changes a take; index order never breaks a tie. An order that can't be decided (a hole) is refused (`RUN-ORDER`).
3. **Start.** Remaining supply R_s = supply(s); remaining demand Q_d = demand(d).
4. **Take.** For each arc in order, `taken[s, d] = min(R_s, Q_d)`, and subtract it from both.

**Outputs:** `absorb.taken[supply keys, demand keys]`, a ref with the supply keys first, then the demand keys; a side without `for` adds none. It is 0 for pairs that aren't arcs, keys a side's `where` leaves out included. What remains is written with `sum` over `taken`. A keyed fill's output is `F[y].taken[…]`.

**Holes.**
- A take is 0 wherever its arc is false, or R_s or Q_d is known to be 0, whatever the other side: on a known arc as on an arc whose condition is a hole. Otherwise a hole side, or a hole condition, makes the take a hole.
- After a hole take, R_s and Q_d are holes carrying that take's questions, so an unknown later year never spoils an earlier result.
- **What a take reads:** its own `arc` and `order`, supply(s), demand(d) and, through R_s and Q_d, what the earlier takes on the same s or the same d read. Its register follows that (§8.2).

**A place's takes** (tally 2.1). A place kind's `takes` clause is this procedure run by the walk over the place's parts: its supplies are the parts, its demands the outflows taking at the place at one point of the day's order, its `order` per (part, outflow) arc, with `prorata` within one outflow and a hole kept as a hole part instead of `RUN-ORDER` (§3.6, §5.14).

**Refusals:** `RUN-NEG` (a negative supply or demand), `RUN-ORDER`, `RUN-TIE`, and `LOAD-UNIT` (supply and demand must share a unit). **Cost:** O(A log A) for A arcs: 30 years of CB-05's `spouses` (a year back and ten forward, between two spouses each way) is at most 30 × 11 × 4 = 1 320 arcs. A keyed fill costs that per key. **Cases:** CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13; keyed fills: LH-04, UL-13.

**Lots.** Lots are a fill whose sides bind over acquisition and disposal movements, its `order` the matching method (first in, first out by acquisition day, or another stated order); a split of a place by the origin of what flowed in is written so (§3.6). Lots held before the run are observed acquisitions dated before the start (§5.19), each with its value and day, which a fill binds and nothing books; the opening reading at the start anchors their sum (`HOLE-LOTS` when they disagree, §8.1). A movement between one holder's places, or from one holder to another, is of its own kind, and whether a lot keeps its acquisition day through it is the rule about that kind (Beancount's booking; C-16). What a rule carries between events (amounts carried forward, a cost basis, what is owed) is likewise one of its places, defs or fills, never implicit.

```
kind trades: event certain
  leg units: notes
  leg cash: PLN
  attr seq: int
kind purchases: trades
kind sales: trades
  leg fee: PLN
kind assessments: event certain
  leg tax: PLN

input rate: num
input owner[x in brokerages]: key persons
place owed[p in persons]: debts PLN lives 2024-01-01..
place fisc: outside lives 1900-01-01..
fill matched[x in ordinary over ..]
  supply b.units.amount for b in purchases over .. where b.units.to == x
  demand s.units.amount for s in sales over .. where s.units.from == x
  arc b.day <= s.day and unit(b.units.amount) == unit(s.units.amount)
  order b.day ties by (b.seq, s.day, s.seq)
def cost[s in sales]: money PLN = round(sum(matched[s.units.from].taken[b, s] * b.cash.amount / b.units.amount for b in purchases over ..), minor(PLN), half-even) at s.day
def gain[s in sales]: money PLN = s.cash.amount - s.fee.amount - cost[s] at s.day
def tax[p in persons, y in years]: money PLN = round(sum(gain[s] for s in sales over y where s.units.from in ordinary and owner[s.units.from] == p) * rate, minor(PLN), half-up) at end_of(y)
move assess[y in years, p in persons on end_of(y)]: assessments
  on end_of(y)
  at end_of(y)
  leg tax from owed[p] to fisc amount tax[p, y]

version act_2024
  cite pit_act
  governs 2024..
  known 2023-12-01
  rate = 19 % "art. 30b"
```

The gains rule above names no account: it reaches every sale of a place of kind `ordinary` by its binding, so a new brokerage account of that kind is covered the day it enters, with nothing written on it.

### 5.13 prorata

<!-- tally2: D-5, D-14 -->

`prorata(total, w[j] for j in IDX [where c], k, u, mode, rest none | last)` or `prorata(total, w[j] for j in IDX [where c], k, u, rest largest [ties by e])` is key k's share of `total`, split by the weights. The total is money or a `num CUR`. **The shares' type follows `round` (§7.2):** they are money when the total's unit is a currency and u is a whole number of minor units, and under `rest last`, whose last share is the remainder, only when the total is money too. Otherwise they are `num CUR`.

1. `raw_j = total × w_j / Σw`. The weights must be ≥ 0 with Σw > 0; otherwise refused (`RUN-WEIGHTS`). A key k that the binding leaves out (its `where` false) has share 0, as a fill's take off its arcs (§5.12).
2. Round:
   - `rest none`: each raw_j is rounded to u with `mode`, independently.
   - `rest largest`: no mode; each raw_j is rounded toward zero to u (`down`). The |total − Σ|/u leftover units, each of the total's sign, go one each to the keys with the largest remainders |raw_j − share_j|, so a negative total's shares are the negatives of its absolute value's (Hamilton's largest-remainder method, kept symmetric in sign); keys whose remainders tie go by `ties by e`, and a tie with no `ties by` is refused (`RUN-TIE`, D-5). The shares then sum to the total. A total that isn't a whole number of u is refused (`RUN-PRORATA`: `<total> isn't a whole number of <u>`).
   - `rest last`: each key but the last is rounded with `mode`, and the last takes whatever remains (the instalment that absorbs the odd unit, N-213).

**In a place kind** (tally 2.1). The same split, `rest largest` at the kind's `step`, divides every arrival among its owners and, where a `takes` clause ends in `prorata`, one outflow among its tied parts, weighted by their amounts, never one part among outflows (§3.6, §5.14). Each split's ties go by its own clause's `ties by`, and an amount that isn't a whole number of the step is `RUN-PRORATA`.

`mode` is a rounding code, an input of type `rounding`, or an axis whose options are rounding codes (§5.5). **`prorata` is for shares the law rounds.** An exact split is plain arithmetic, `total * w[k] / sum(w[j] for j in IDX)`, as CB-13 and CB-14 write it.

**Cost:** O(n log n) once per (total, weights), memoised. **Cases:** CMP-01, MP-01, MP-09, MP-12.

### 5.14 move

<!-- tally2: D-2, D-4, D-5, D-6, D-7, D-8, D-13; SF-3, SF-5, SF-15, C-5, C-13, C-17, M-2, M-5, M-7, M-11 -->

```
kind wages: event certain
  leg cash: PLN
kind shortfalls: event certain
  leg cash: PLN

move rent[m in months]: transfers
  on schedule(m, rent_dom, following, len)
  at start_of(m)
  for m
  leg cash from konto to landlord amount rent_amt[m]

move salary[m in months]: wages
  on window(last_business(m, pay_lead, len), last_business(m, 1, len))
  at start_of(m)
  leg cash from employer to konto amount pay_amt

move draw[d in days]: shortfalls
  on d
  at d
  reads konto, kredyt
  leg cash from kredyt to konto amount min(max(0 PLN, -bal(konto)), limit + bal(kredyt))
```

A move writes planned movements of an event kind (§5.7), one per binding tuple. A movement, planned here or observed (§5.19), is a thing of its kind with a day, named legs and attributes; the two differ only in certainty (P-4). It lives on its day, and its facts (legs, attributes, the defs keyed by it) hold on every read day (§5.7).

- **`on d`** is the movement's day, `s.day`: a date, a window (§4.4), a hole, or `none`. `none` is "nothing here", so it gives no instance at that key, as a false `when` does (a `schedule` day that doesn't exist, §4.2) (P-9). Each leg may state its own `on` (cash that lands two business days after the units leave); a leg without one is on the movement's day, as a row is under its version's span (C-13). `over span` and `on d` select movements by their day. A movement whose day is a hole makes every `flows` and binding over a span it might fall in a hole (G-MP-2). A day that is a hole and not a window has no bound, so that is every span, earlier ones included, whatever the instance's `when` says: its booking cut min(t, K) is a hole with it (§5.14.3).
- **`at e`** is the movement's law date, required when anything it reads is law-dated (`LOAD-LAWDATE`). §4.3 states what it dates (the keys' `where`, the `on`, `when`, legs and attributes, for their binding tuple), at which cut each is read, and that e itself reads nothing law-dated (P-6). Since which tuples are instances is read at the ask's cut K and the amounts at the booking cut (§5.14.3), a condition whose later change is to be measured as an adjustment is a `when`, not a key's `where`. An `at` that gives a window evaluates the instance once per law-date class, as a def's does (§4.3).
- **Keys.** A move's binds over a kind state `on` or `over` (`move assess[y in years, p in persons on end_of(y)]`), since each key books an event on a day (`LOAD-ALIVE`, M-2). A tuple its keys' `where` leaves out at the ask's cut is no instance, so it has no day. A series that exists only from a version is a move whose keys' `where` reads a def whose winning rule cites that version with `v` (§5.11 step 2): before the version governs and is known, the series has no instance, so its day is never read and spoils no span (P-14, P-9).
- **`for k`:** the period the amount is for, kept apart from the day it is paid (N-305). `flows(x, span, for k)` reads only instances for period k. A movement with no `for` (of a move without one, a pad, an observation that fulfils nothing) is for no period, and `, for k` leaves it out; an observation that fulfils an instance is for the instance's period.
- **`when c`:** the instance exists only where c holds; with no `when`, at every binding tuple. Its amount doesn't decide whether it exists. A hole `when` is a hole movement (M-7).
- **Legs.** `leg name from A to B amount q [on d]` for every leg the kind names (`LOAD-LEG` when one is missing or unknown). A and B are places of the leg's unit; the amount is ≥ 0 in that unit (`RUN-NEG` otherwise: swap the ends with a `when`, or write two moves). Each leg conserves its unit by construction: what leaves A arrives at B on the leg's day. A leg's place may be a hole of a place type, a place nobody has told us (`account_of[marta]`, `HOLE-STATED` where a row states it `?"…"`): the leg books its known side, and every read of the unknown side is a hole naming it; it is never a made-up place (P-9). An amount of 0 books nothing, but the instance exists.
- **`attr name = e`** states an attribute the kind declares (`LOAD-LEG` when one is missing).
- **`fulfils m[k]`** links a movement to the planned instance it settles (an adjustment, §5.14.3) or realises (§5.19).
- **Certainty** is the kind's (§5.7), never the movement's (P-8). A movement of a `certain` kind is booked on its day. A movement of an `uncertain` kind counts in balances only once an observation fulfils it (§5.19); a projection or a binding counts it unfulfilled only with `with uncertain`, and so does a plain balance read that states it, `bal(x, d) with uncertain`: x's balance plus the unfulfilled movements of uncertain kinds dated through d, each at its planned amount (both ends count, so worth is conserved). The clause is the read's, so no decision sees it. "Will it arrive" as a probability is a random amount or day (§5.16).

**Legs at a place that keeps parts** (tally 2.1, §3.6). At its `from` place a leg's amount is a **take** of parts by the place's leaving rule; at its `to` place it is an **arrival**, labelled by the arrival rule at an `owned` place, and keeping or starting batches at a `batched` place. A movement's take never includes its own arrivals. Two observed rows stated `same` are one movement (§5.19), so its take at the `from` end and its arrival at the `to` end belong together, and "keep the labels" reads that take.
- **The take is a fill over arcs** (R-4; C-1, RC-3). An outflow is one leg whose `from` is x, and the order's w is that leg's movement, so two legs of one movement out of x tie on every term on w (the leg read below is per leg). At one point of the day's order (§5.14.2) the supplies are x's parts there and the demands are the outflows taking at x there: in the day's first batch, every outflow at x of its observations and plans, which have no order among them; at a decision's point, that decision's outflows, or the observations booked at its point. The `takes` clause's order is evaluated per arc (part k, outflow w), and the arcs are sorted and taken as §5.12's steps 2–4 do, with these differences:
  - **Ties** between arcs whose order changes a take are refused (`RUN-TIE`, naming the outflows), with one exception: arcs of **one outflow** tied on every term share it in proportion when the clause ends in `prorata`, by §5.13's `rest largest` at the kind's step, weighted by those parts' amounts at that point, the remainder's ties going by the clause's `ties by`. **Two outflows never share a part:** two outflows tied for one part that can't cover both are refused (`RUN-TIE`), with or without `prorata`, and a law that ranks them writes a term on w.
  - **The batch's arrivals are supplies.** Parts that arrive at x in the same batch are supplies of its takes, as the sign rule checks the batch together (§5.10). An arrival that keeps labels reads a take of the batch at another place, so the batch's takes are computed in the order of those reads (§5.14.2).
  - **A hole in a term** (an attribute that is a hole, or a field of a hole part on which its possible labels differ, §8.1) makes the arcs it ranks undecided. Only the parts the take could reach under some reading of the holes merge, into one hole part with the total known; a part ranked after enough certainly-ranked parts to cover the outflow stays known (P-9). A fill refuses an undecided order (`RUN-ORDER`), since its output is a value; a take's outcome is a fact, so it stays a hole. A refusal in a term refuses the take.
  - **More than the parts hold** is the sign rule's `RUN-PLACE` (§5.10).
- **Whose parts a leg takes is the order's** (RC-7). The order reads the outflow's kind and fields after a kind test (§5.7), so the law names each case once: a death's relabel ranks first the deceased's parts (`if w in deaths and k.owner == w.deceased then 0 else 1`); a row that says whose money paid, by a fact it states (`if w in withdrawals and k.owner == w.paid_from then 0 else 1`); a broker's row per lot (`if w in lot_outs and k.batch == w.lot then 0 else 1`). Where q's parts are fewer than the amount, the take goes on down the order, which is tracing's own rule; a writer who wants such a shortfall refused writes a `check` ask over the rows. A hole in such a field is an undecided take (above).
- **A relabel** (R-5) is a leg whose two ends are one place that keeps parts. It takes parts by the leaving rule and returns them labelled by the arrival rule, batches kept; it books nothing net, and `flows` counts it as 0. Its amount is written: "all of p's" is `bal(x) of p`, a running read, so such a posting is a decision with `reads x` (§5.14.1). The arrival rule labels it by the movement's kind: a gift to the receiver, a death to the estate, a divorce to each spouse by `stake`, a division to each heir by a share keyed by the division, the place and the heir, from the agreement. A relabel keeps the thing in its place, so rules over the place's kind still reach it; a gift that takes it out of the household is a leg to another place. A place of unit `any` is relabelled by one leg per unit. At a place that keeps no parts, a leg from a place to itself books its amount out and in on its day, as in 2.0.
- **The leg read** `m.leg.amount of q` (R-10) is what leg `leg` of movement m took at its `from` place from the parts labelled q, a party, a batch or a pair: 0 where it took nothing of q, a hole where the take was undecided or a hole part it took may be q's (`HOLE-PART`), and refused at a `from` place that keeps no labels of q's sort (`RUN-DOMAIN`). Its class is §5.14.1's. A batch's later events are found by a binding (`g in gifts over .. where g.what.amount of b > 0`); no list of events is kept per part.

**Prelude event kinds** are §5.7's: `transfers`, `adjustments` and `pads`. A tally 1 `pay` becomes a one-leg move of `transfers`.

**Postings and reach by kind** (P-4). A move keyed by another movement (`move fee[s in sales over .. where s.cash.to in current]: charges`) is Fowler's posting rule: one instance per movement reached. The rule in force on the event's day applies, since the move's `at` names it (§4.3). A rule that would apply to a list of accounts is written as a kind, and the membership is a fact (`wrapped + [broker] lives 2024-01-01..`): a binding over a kind ranges over every thing or movement of that kind, its subkinds included, planned and observed, with `where`, `on` and `over` (§5.7), so a new account or instrument of the kind is covered the day it enters. Naming a particular thing inside a rule (an office the rule pays, the account a line of credit belongs to) isn't refused, since some rules are about one thing; a rule written for one account names that account once, as an input row (`home_account = konto`), and its rules read the input.

#### 5.14.1 Plans and decisions

A movement is a **decision** if it declares **`reads`**: the places whose running balance its amount or condition read, and the movements whose legs, day or attributes it reads, directly or through defs (`reads konto, settle`); a keyed place or a move named without keys stands for all its instances (`reads wcash`). Every other movement is a **plan**. Reads are declared, so the class is written, and an edit to a def can't move a movement between classes in silence (P-7). A decision's day reads no running state, since the walk must know it before it reaches that day (`LOAD-READS`): "pay when the balance drops" is a daily decision with a `when`, as `draw` above.
- A movement that reaches a running read, or a projection, of a place or a movement it doesn't declare, or declares a read it never makes, is refused (`LOAD-READS`).
- Plans are computed in dependency order over the whole run, like a spreadsheet; decisions in the day walk (`ENGINE.md` §4.2).

**Running reads.** Allowed only in a decision and the defs it reaches:

| Read | Value |
|---|---|
| `bal(x)` | x's running balance at this point of the day walk: the start of the day, plus the day's first batch of observations and plans, plus the decisions already run that day, with the observations booked at their points (§5.19, `ENGINE.md` §4.2) |
| `bal(x, d)` | x's balance at the end of day d, for d before the decision's day. For d on its day, a plain `bal(x, d)` would read the decision's own result, a cycle; for d after it, the decision's future. Both are refused: `LOAD-CYCLE` where load sees the day (§8.3), otherwise `RUN-CYCLE` on the decision's day and `RUN-FUTURE` after it. The future is read through the projection |
| `bal(x, d start)` | x's balance before day d's movements (`d start`, §4.6). On the decision's day it reads none of that day's movements, its own result included, so it is no cycle; for d after that day it is the decision's future, refused as `bal(x, d)` is |
| `flows(x, span)` | for a place x, or a decision m: every movement booked there. A span that ends on the decision's own day counts what is booked up to this point of the day, so a yearly limit is re-checked after each posting (R-08) |
| a leg, day or attribute of a decision m | m's booked values; the reader is above m (§5.14.2) |
| a binding over a decision's movements, or over `pads` | those booked up to this point of the walk, as `flows` of a decision counts them; the decision declares the move or the kind in `reads` (`reads pads`). A span past the reader's day, in a binding or in `flows`, is its future, refused as a plain `bal(x, d)` after its day is (`LOAD-CYCLE` where load sees the day, `RUN-FUTURE` otherwise) |

**The arrival rule is a reader too** (tally 2.1, RC-6, M-15). The walk runs a kind's arrival rule (§3.6) when it books an arrival, so what the rule reads of parts (`bal(y, d start) of q` with d the arrival's day, `bal(y, d) of q` with d before it, `value(…) of q`) and of what the movement took (`m.leg.amount of q`) are the walk's reads, as a decision's are. The movement stays a plan or an observation and declares no `reads` for them: running reads are allowed in a decision and the defs it reaches, and in an arrival rule and the defs it reaches. The start of the arrival's day is booked before anything that day, so reading it is no cycle; a read of the end of the arrival's day or later is (`LOAD-CYCLE` where load sees the day, `RUN-CYCLE` otherwise).

**The leg read's class** (R-10). `m.leg.amount of q` reads booking state, as a balance does: in a def no movement reaches, or an ask, after the walk; in an arrival rule, for the movement it books; in another movement, only in a decision that declares `reads` of the place the leg took from, the reader above m (§5.14.2). So a tax per giver on a gift is a posting that is a decision.

**A def that reaches a running read is evaluated at each reader's point of the walk**: its memo cell carries the walk point (day, stratum, movement, keys), `ENGINE.md` §3.2. So two decisions on one day that read one `room` def each see the balance as it stands when they run.

**The projection is written.** The forecast is a query over planned movements (P-8): `bal(x, d) from t` is x's balance at the end of day d in the walk continued from day t.
- It starts from x's balance at the start of day t (a t before S is a hole, `HOLE-PRERUN`: nothing before S is walked), and counts every movement dated on t through d, each as known on t (its keys, `when`, day and amount at cut min(t, K)), observations and anchors after t unused. **Read at a point of the walk on day t** (by a decision, through a def or its own expression), it starts from x's balance at that point instead, so what that day has booked so far (its readings, observations, plans and earlier decisions) is in it, and it counts only the movements not yet booked. Such a read is a running read (`ENGINE.md` §3.1 step 7), so two decisions on one day that read one `room` def each see what the other booked before it.
- It leaves out every movement that depends on the item holding the read: in a def, the movements that read the def, directly or through other defs and movements (the graph of §5.14.2); in an ask line, none. A movement that depends on the item can't be seen by it, so a rule never counts its own result, nor a later rule's that follows from it (R-04, N-1102). The movements left out that fall in the window are listed in the register.
- Uncertain movements count only with `with uncertain`. A movement dated by a window needs `windows each`, on the read or on the ask line, or the read is `HOLE-IMPRECISE` (§4.4). A random amount dated after t is read by the projection's `random` clause (`random at mean` or `random at quantile(q)`), by the ask line's `random at`, or by a closed-form `mean` or `quantile` query around the read; amounts on or before t take their samples. With none of these, a projection a decision reaches is refused (`LOAD-RANDOM` where load sees the day, `RUN-FUTURE` otherwise), and an ask line's own projection reads the amount, on a path, by that path's sample, as §5.16 states (P-13).
- What it leaves out depends on the def that holds it, never on who reads the def, so an ask that reads the def sees what the decision saw.
- A fact it reads before that fact is known makes it a hole that nothing later answers, unless an `assumed` row known from the start gives it (§5.14.3).

A decision that reaches a projection of x declares `reads x`. In an ask, or a def no movement reaches, `bal(x, d) from t` is the same walk; a plain `bal(x, d)` there is the balance on day d at the ask's cut, as in tally 1.

| Built-in | Value |
|---|---|
| `low(x, a .. b) [from t] [clauses]` | the lowest end-of-day balance of x on days a..b: `min(bal(x, d) from t for d in a .. b step 1 day)` with the same clauses; without `from`, of the plain balances |
| `next(K, d)` | the first movement of event kind K whose day is on or after d (`first(m in K over d ..) ties by m.day`): two movements on that first day are a tie, refused (`RUN-TIE`); a reader that needs one of them writes the `first` itself with a fuller `ties by`, `first(m in K over d ..) ties by (m.day, e)`. When none is planned yet, its binding is empty at the cut and its span reaches past it, so it is a hole naming the binding (`HOLE-NOTYET`, §3.1) |

`low` takes a place and passes on the projection's clauses, and `next` takes a kind; a def can do neither, so they are built-ins (C-24). **Groups of places** are written as sums: the lowest end-of-day sum of the liquid accounts is `min(sum(bal(x, d) from t for x in liquid on d) for d in a .. b step 1 day)`, which isn't the sum of their lows (R-07).

**Refusals:** `LOAD-READS`; `LOAD-CYCLE`, `RUN-CYCLE` (a running read of the decision's own day, or of the arrival's day's end, reached through a computed day); `RUN-FUTURE` (a running read of a later day, reached through a computed day).

#### 5.14.2 Order within a day

Order within a day is the **stratification of the dependency graph**, whose edges are the declared reads (P-7). A decision that reads place x acts after every movement that changes x that day and doesn't depend on it, and a movement that reads another's legs acts after it. Observations and plans come first, since they read nothing running; they have no order among them, so what they book on a day is booked together, and the sign rule waits for their whole batch (§5.10). So nothing is ordered against a member of the batch: a reading `after m` (§4.6) and a `before m` (below) with m an observation or a plan are refused (`LOAD-ORDER`). An observation that fulfils a decision instance of its own day is the exception: it is that instance's movement, booked at the instance's point, right after the instance is computed (§5.19). Any order consistent with the graph gives the same result, so no order is written where none is needed.

A movement changes the places its legs give. A leg whose place is an expression (`to cash_of[x]`, a keyed place at a computed key) changes, for an instance, the place it gives there. Load, which doesn't see that place, puts in the graph every place the expression can give (the things of the place kind its type names, every place of the keyed place); the walk then orders the instances due on a day by the places their legs give that day, so one that reads a place another changes that day acts after it (`ENGINE.md` §4.2 step 3). The static graph only bounds the day's order.
- **Cycles.** When two decisions on one day each read what the other changes, the writer states `before m` or `after m` on one of them. An unordered cycle is refused at load where its days and places are known then (`LOAD-ORDER`); otherwise at run, only between instances due on one day that read what each other changes there (`RUN-ORDER`).
- **A written order outside a cycle.** `before m` may also put a decision ahead of another decision it would otherwise follow, and so ahead of the observations booked at that decision's point (§5.19); the decision then reads the balance before m (SF-15). It orders a decision only against another decision: plans and observations come first, and `before m` with m a plan or an observation is refused (`LOAD-ORDER`). An order that contradicts a read of m's legs is refused (`LOAD-ORDER`).
- **Ties** between instances of one move on one day, where their order changes a result, go by the move's `ties by`, else `RUN-TIE`. Ids and declaration order never order what a rule computes: the text order of stable ids fixes only the orders no rule reads (§2.5); tally 1's `phase` is gone.
- **Takes in the first batch** (tally 2.1, R-4, RC-6). The batch has no order, but an arrival that keeps labels reads a take of the batch at another place, so the batch's takes are computed in the order of those reads (§5.14). A cycle among them (two places sweeping into each other in one batch, each arrival keeping labels) breaks where it can: each take is tried with the cycle's arrivals as parts of unknown labels, and a take whose outcome is the same under every reading of them (the reach of an undecided take, §5.14) is decided, which breaks the cycle. Otherwise a cycle that holds a plan is refused (`RUN-CYCLE`, naming the movements), and the writer makes a plan of it a decision, which the day's order then places. A cycle of observed rows only is an unknown fact, which transfer went first and so whose money each carried: no form orders two rows of one batch, so the takes it reaches are undecided, each place holds a hole part with its total known (§8.1), and a later reading's stated parts heal it (§5.19).

A rule that invests "at the start of the day" and one that invests "at the end" are therefore different models, the first reading `bal(konto, d start)`, and the difference is visible (N-318).

**Several rules on one event.** Two rules may reach the same movement, as two acts may apply to one event: each posts its own movements, their order in a day is the stratification above, and `readers-of K` on an event kind lists every binding that reaches it (§5.18). No load check compares their filters: whether two `where` clauses overlap is undecidable in general, and two acts reaching one event is lawful (C-6). "One value, one definition" holds for each name: a def's rivals are its rules with `beats` (§5.11).

#### 5.14.3 Booking, back pay and adjustments

**The cut of an instance.** An instance's `on` is evaluated at the ask's cut K. Its `when` and its amounts are then evaluated with what was known on its day t: cut min(t, K). A decision's running reads are the exception: they read the day walk at cut K (`ENGINE.md` §4.2). **Labels are walk state** (tally 2.1, R-3, G1 Q-4): the walk at cut K labels every movement, planned or observed, by the arrival rule read at K, as it books balances at K. So a later ruling about whose past money was applies under every cut that sees it, `explain` names its version, and `as known D` shows the labels before it; a booked plan's amount and an observed row's amount never change by it. So a day learned later moves the instance under later asks only; under an earlier cut the instance is where it was then known, or a hole. When t is a window, a version's visibility at that cut is a §4.4 comparison: visible when known by the window's first day, not visible when known after its last, and otherwise the instance's amount is decided by §4.4's rule: `HOLE-IMPRECISE` where the version changes it, unless the line states `windows each`. Scenario versions are visible to the instance by their `known`, as any version (§6.4).

**Changes in the world.** A retroactive change is a readable difference (P-8). A version that changes a booked plan leg's amount, or its places, makes an **adjustment** of that leg (a settlement's legs aside, What a later version never measures, below), named `adjustments[m[keys].leg, V]`; a settlement leg counts by the way it runs against the leg's places before and after the change. Such a version is known after the instance's day and by the cut, t < V.known ≤ K, and versions are taken in `known` order. δ counts from 0 when the instance didn't exist at its cut and V makes it exist, and a change of places makes an adjustment even with δ = 0: the booked movement stays where it was booked, and the change is never silent. The engine books nothing by itself.
- **The adjustment** is a movement of the prelude kind `adjustments`, dated V.known, with the attributes `adjusts` (the instance, a key of its move's kind) and `change` (δ, signed, of the leg's type). There is one per changed leg, since a movement's legs may be in different units (P-3). Its name carries the leg and the version, so it is stable across runs and an observed row can name what it settles, `fulfils: adjustments[rent[2026-03].cash, v2]` (P-17). Versions with one `known` are one step (`ENGINE.md` §5) and make one adjustment per changed leg, which each version of the step that can change the instance (`ENGINE.md` §5 step 2) names: `adjustments[wage[2026-03].cash, v2a]` and `adjustments[wage[2026-03].cash, v2b]` are one movement, which a binding gives once, printed under the first of the ids in ASCII order, and whose `HOLE-CHANGED` question names every version of the step. Bindings and asks read it (`sum(a.change for a in adjustments over 2026 where a.adjusts in wages)`). Only the engine makes adjustments: a move or an observed row of kind `adjustments`, or of a subkind of it, is refused (`LOAD-KIND`).
- **Settlement is written.** The writer settles an adjustment with ordinary movements on any schedule, each stating `fulfils` the adjustment: an amount in parts (100 in July, 200 in August), through a later yearly settlement, or one movement on a day. A settlement of 0 is written too: a movement of an event kind with no legs that fulfils the adjustment **waives** what remains of it. A settlement may be dated on the instance's own day, which re-books the past in place: the fix of a wrong record (§4.5). What remains of an adjustment is its change less its settlement legs, or nothing once it is waived.
- **How a settlement leg counts.** It counts +amount when it runs the leg's way, out of a place the leg runs from or into a place it runs to, and −amount when it runs the other way, out of a place it runs to or into a place it runs from. For a change of places, the leg's places are both those it had before V and those V gives it, so a settlement leg between two of its `to` places, or between two of its `from` places, runs both ways and counts 0: moving the booked amount to where V puts it (konto to konto2) settles a change of places alone, and a second payment into the new place counts in full, its amount owed back. A version that turns a leg round, a place its `from` on one side and its `to` on the other, gives settlement legs no sign: they are refused (`RUN-RECORD`), and only a waiver settles it. A settlement leg that touches none of the leg's places is refused (`RUN-RECORD`), and one in another unit than the leg's (`LOAD-UNIT`). `ENGINE.md` §5 step 5 computes the remainder by this rule.
- **Detection stays.** An adjustment with an unsettled remainder at the cut is listed in the register of every answer that reads its places. One that no movement fulfils makes every `flows` that could count it a hole, `HOLE-CHANGED: "<move> on t changed after it was paid (version V); is back pay due, and when?"`: one of the move, or of a place it moves money between, over a span that reaches V.known or later. The adjusted leg's places' balances are holes from V.known on with it, past later readings too, since the settlement may fall after any of them. A later-known change is never silently paid or silently unpaid.
- **An amount, or a `when`, that was a hole** at its cut stays a hole movement until a version answers it. A version under which the `when` is false answers it with 0: nothing is booked.

**What a later version never measures.**
- **Decisions are never re-measured** (decision 22 kept). A decision instance is a decision taken on its day: the law and facts it reads, and its projection, are what was known then (cut min(t, K)). A later version never re-measures it. Its balances are the walk's at cut K, so observations dated up to its day and anchors before it, seen by a later cut, can change what it computes and books (`ENGINE.md` §9).
  - **The one exception: parts** (tally 2.1, RC-1, G1 Q-4). Its parts are the walk's at cut K too, as its balances are. So a past decision that no observation fulfils, and whose amount or `when` reads parts (`bal(x) of p`, `m.leg.amount of q`), follows a later version that changes whose past money was, a ruling or a late-known entry, under every cut that sees that version, as it follows a bank row learned late. Its law and its other facts stay those known on its day. A decision dated after K, and one an observation fulfils (booked by the row, §5.19), don't change.
- **Modelled-past instances are never measured either.** On a covered day, or wherever an observation fulfils it at the ask's cut (§5.19), an instance is computed at its booking cut for the ledger (`ENGINE.md` §6.2) and moves no money, so no version changes it by a δ, and no earlier cut judges its link (§5.19).
- **Settlements are never measured.** A movement whose `fulfils` names an adjustment settles a change; a later version reaches what it settles through a new adjustment of the original instance, never by measuring the settlement, so the instance graph stays acyclic (P-8; `ENGINE.md` §5 step 2).

**Facts a projection reads.** A decision rests for good on what was known on its day. A fact its projection reads before that fact is known (a contract signed next month, a form chosen later, a debt not yet taken on) makes the decision a hole that nothing later answers, and every later balance of its places a hole with it, up to the place's next balance reading (§5.19). So such a fact is given an `assumed` row known from the start, in a forecast version (0, or `none` for a thing that doesn't exist yet), and things that appear later are things whose lifespan starts later, or keys of an open index; the later version replaces that row.

An instance dated after K is planned, and it is evaluated with cut K. An instance dated outside [S, E] isn't booked; bindings and reads still see it, but `flows` over days before S is a hole whatever is dated there (`HOLE-PRERUN`, `ENGINE.md` §2), and the register lists, per move, how many were left out and the first such day (D-13, LC-11).

**A booked amount is fixed.** Once an instance is booked at its cut, its amount never changes; a later version reaches it only through an adjustment and its settlements, each with its own day. Anything that reads the instance later, through `flows`, reads the booked amount plus the settlements dated in its span.

**`flows(x, span)`** is the sum of the movements into place x in the span, minus those out of x, settlements included. **`flows(m[keys], span)`** with a move m is the sum of that move's movements in the span, over its instances at the stated keys (all of them when no keys are given), plus the settlements of those instances' adjustments dated in the span, each leg counted as How a settlement leg counts (above) says: the history of transfers by kind (N-926). A movement's amount is its leg's: a move whose kind has several legs names the leg, `flows(buy.eur, 2026)` (`LOAD-LEG` otherwise). On a day a statement covers, and on any day for an instance an observation fulfils, the observations that fulfil the move's instances stand in for them (§5.19). `, for k` keeps only the instances whose `for` is k (N-305). **Its unit** is that of the places it reads (x's, or the leg's places' for a move), known at load where they have one fixed unit, otherwise taken from the movements it counts; a place of unit `any`, movements in two units, or an empty span whose unit load can't see is refused (`LOAD-UNIT` where load sees it, `RUN-DOMAIN` otherwise), unless the `flows` converts (below). **`flows(x, span) in CUR at fx mode`** converts each movement separately, reading the rate `fx` (a `num CUR/OTHER` input) at the movement's day, rounding each movement's amount (≥ 0) to CUR's minor unit with `mode`, and only then applying its direction, before summing (N-606, N-932). So a movement in and one out of the same size convert to the same size. A converted total of the whole span is a different number, and the model writes it as one if it needs it.

**Refusals:** `LOAD-LEG`, `LOAD-READS`, `LOAD-ORDER`, `LOAD-ALIVE`, `LOAD-LAWDATE`, `LOAD-UNIT`, `LOAD-KIND`, `RUN-NEG`, `RUN-ORDER`, `RUN-TIE`, `RUN-PLACE`, `RUN-LIFESPAN`, `RUN-RECORD`, `HOLE-CHANGED`, `HOLE-IMPRECISE`; and at a place that keeps parts (tally 2.1), `RUN-LABEL`, `RUN-NO-RULE`, `RUN-PRORATA`, `RUN-CYCLE`, `RUN-DOMAIN`, `HOLE-PART`.

**Cost:** O(1) per instance. `flows` is O(log Mₓ) with per-place prefix sums (`PERFORMANCE.md` §2.2). One more evaluation per version that reaches a booked instance. A projection costs one walk of its window, memoised per (item, day, cut, clauses). **Cases:** CMP-01 (moves, places, conversion); the household, and change 5 adds moves (changes 3, 4, 6 and 7 change what moves read). Decisions: the household, change 5. LH-14 states back pay but books no movement; UL-09 and RC-08 are retroactive law read by defs. The undated illustrations (MP-14, MP-15, CMP-02) are defs (§5.10).

### 5.15 Scores

<!-- tally2: D-11, D-15 -->

| Built-in | Computes |
|---|---|
| `crps(normal(m, s), y) to u` | s·[z(2Φ(z) − 1) + 2φ(z) − 1/√π] with z = (y − m)/s, correctly rounded to u |
| `crps(x[i] for i in I, y)` | (1/n)·Σ\|x_i − y\| − (1/(2n²))·ΣΣ\|x_i − x_j\|, exact, with denominator 2n². Computed through order statistics in O(n log n) |
| `crps(e, y)`, e random, in a paths ask | the row above over e's N path values (§5.16) |
| `crps(x[i] weight w[i] for i in I, y)` | Σŵ_i·\|x_i − y\| − ½·Σ_iΣ_j ŵ_i·ŵ_j·\|x_i − x_j\|, with normalised weights ŵ = w/Σw in place of 1/n in both terms |

- Interval scores, the weighted interval score and skill scores are plain expressions over `quantile`, `abs`, `sum` and `if` (N-1502).
- **A sample has no density.** A log score is written only for a parametric forecast, as `ln(pdf(…) to v) to u`, each transcendental with its own step (§7.3). The log score of a sample or of paths is a step no row can give: the model writes it as a hole whose question says why (`?"a sample has no density"`, N-1504, CH-03 step 14; D-15).
- **`orient`.** An ask that reaches `crps` or `pdf` states `orient lower` or `orient higher` (`LOAD-ORIENT`), whatever it prints, a hole or a refusal included. Any other ask may state it, and it is printed. A score written as a plain expression (an interval score) states it by choice. A convention that varies with a reading is two asks.

**Cost:** O(n log n) per sample score. **Cases:** `crps`: CH-03; `orient`: CH-03. CH-01, CH-02 and CH-06 evaluate with exact distributions and arithmetic (backtest zones, actual-to-expected ratios), not these scores.

### 5.16 Distributions and paths

<!-- tally2: D-6, D-11, D-15; C-10, C-25, SF-21, SF-28, M-15 -->

```
input growth[y in years]: dist 1/year
input theta: dist 1/year
input vol: num 1/year
input start_balance: money PLN
input goal: money PLN
input theta_mid: num 1/year
def returns[y in years]: dist 1/year = normal(theta, vol) to 1e-9 at end_of(y)
def balance_end[y in years]: money PLN at end_of(y)
  rule first = start_balance
  rule later beats first = round(balance_end[y - 1] * (1 + returns[y] * 1 year), minor(PLN), half-even) when pos(y) > 1

version assumptions
  cite own_assumption
  governs 2026..
  known 2026-01-01
  growth[*] = normal(3 %/year, 10 %/year) to 1e-9 "planning assumption"
  theta = uniform(1 %/year, 5 %/year) to 1e-9 "own range"
  vol = 10 %/year "own guess"
  start_balance = 50 000.00 PLN "statement"
  goal = 100 000.00 PLN "own goal"
  theta_mid = 3 %/year "the range's middle, for the test mode"

ask growth_q1: quantile(growth[2030], 25 %) law 2030-12-31
ask median_2040: quantile(balance_end[2040], 50 %) law 2040-12-31 paths 10000 seed 7
ask over_goal: share(balance_end[2040] > goal) law 2040-12-31 paths 10000 seed 7
ask bad_years: mean(balance_end[2040] where returns[2030] < 0) law 2040-12-31 paths 10000 seed 7
ask one_path: balance_end[2040] law 2040-12-31 paths 10000 seed 7 path 512
ask test_mode: mean(balance_end[2040]) with sample theta = theta_mid law 2040-12-31 paths 10000 seed 7
```

A random quantity is a fact whose value is a distribution, and a planning figure is a question about it (P-13); there is no separate random section (D-11). A **random fact** is an input or def of type `dist U`: its value is a distribution, from the closed list below, over quantities of unit U. Its rows and rules give samples of one type, quantities of unit U or keys of one index (each a `discrete` over it); a mix is refused (`LOAD-UNIT`). A row or a rule gives the distribution; its parameters may be facts, holes, or other random facts (a parameter drawn once per path, as `theta` above).

**Samples.** A random fact's **address** is its stable id and keys (§2.5): `growth[2030]`, `costs.rise[ewa, 2031]`. On path p of an ask with `paths N seed S`, its sample is:
1. the canonical text A = `address|S|p`, with the keys printed canonically and S and p in decimal;
2. k, the first 8 bytes of SHA-256(A), read big-endian;
3. the uniform u = (k + 1/2) / 2^64, which is never 0 or 1;
4. the value `quantile(D, u)`, where D is the fact's distribution on that path (its random parameters at their samples), correctly rounded to the step stated with the distribution (`to u`) for a continuous D, and exact for a discrete one. A continuous distribution without `to`, in a row or a rule, is refused (`LOAD-TO`).

So no key can be forgotten: the address is the fact's own, and anything that reads it is random by **lifting** (the stochastic lambda calculus, D-11, C-25): arithmetic, comparisons, `if`, aggregates and bands over a random value give a random value, computed path by path. A def of any type that reads a random fact is random; only `dist` facts are sampled. Samples are inversion samples, so a pinned sample or a pinned uniform gives a plain number and a path can be worked by hand (N-1407).
- **The same address gives the same uniform** in every scenario, every `as if` and every reading of an axis, and so the same sample under the same distribution: what-ifs share their random numbers (N-1302). A pin that changes a distribution keeps its uniforms.
- **A path is a pure function of its seed and number,** so any path is recomputed alone (`path p`).
- **Order.** When an evaluator is handed fixed uniforms u1, u2, …, they are assigned to the addresses an ask reaches in canonical order: path; then the address's law date, ascending: the first law date at which the ask reads it (a fact read at no date first). An address has one uniform whatever law date reads it; a later law date may read it under another row's distribution, from the same uniform; then the fact's id (ASCII); then its keys in index order, things and an open index's keys in the text order of their stable ids (§2.5). `explain` prints samples in the same order.

**Asking about a random value.** A random value is read by a **query**, a question about its distribution written where it is asked, or by its sample on one path; an ask line whose value is random does one or the other, or it is refused (`LOAD-RANDOM`).
- **`paths N seed S`** runs N paths (N ≥ 1, an int literal; S an int literal). The line states N, and an expression on it that needs N writes the same literal (§7.1). `path p` (1 ≤ p ≤ N) replays path p of that run: the line prints its value on that path, as plain values with every sample it reached in the register, and holds no query over paths (`LOAD-RANDOM`).
- **`mean(e)`** over the paths prints the mean with its standard error s/√N, s² the sample variance over N − 1, correctly rounded to 1e-6; with N = 1 the standard error is `none`. The sums under them aren't values at a key, so the 4 096-digit cap doesn't reach them (§7.1).
- **`quantile(e, q)`** is type 1 on the N samples: the smallest value v with at least q·N samples ≤ v, for q above 0 and at most 1 (another q has no such v and is refused, §8.4). It prints the value with its standard error by the Maritz–Jarrett method: with the N samples sorted x(1) ≤ … ≤ x(N) and m = ⌈q·N⌉, the rank of the type-1 value, the weights are W_i = I(i/N) − I((i − 1)/N), where I is the distribution function of Beta(m, N − m + 1), equivalently the chance that a binomial of N trials with success chance x has at least m successes (exact for whole N); with C_k = Σ W_i·x(i)^k, the standard error is √(C₂ − C₁²), correctly rounded to 1e-6, as `mean`'s; with N = 1 it is `none`, as `mean`'s.
- **`share(c)`**, c a random bool, prints "n of N" and the fraction, with its standard error √(s(1 − s)/N) correctly rounded to 1e-6.
- **`where c`** inside a query's argument restricts it to the paths where c holds (`quantile(x where c, q)`, `mean(x where c)`, `share(d where c)`), and the line prints how many paths that leaves; none is refused (`RUN-EMPTY`) (SF-21).
- Every query prints the **horizon**: the last law date at which its argument reads a random fact, each at the fact's own law date, a projection that takes a fact at its `random` clause's value included. There is no built-in verdict on whether "n in N futures" may be said: that is a rule the model or the reader writes over the printed figures (`LC-28`; `ENGINE.md` §8).

**Closed forms.** Without `paths`, a query is computed exactly, or correctly rounded where it is transcendental (a `mean` or a `quantile` to the facts' sample steps; a `share`, or a `cdf` of a form, which has no step of its own, states `to u`, `LOAD-TO`), when this table gives it, and is refused otherwise (`LOAD-RANDOM`: the ask states `paths N seed S`), so no hidden N or seed is ever used (C-10). X is one random fact, read directly; a and b are known, a ≠ 0.

| Query | Closed form |
|---|---|
| `mean(e)`, e a sum of known multiples of random facts plus a known constant | the same sum of their means (the table below), exact or correctly rounded to the facts' steps |
| `quantile(a * X + b, q)` | a · quantile(D, q) + b for a > 0; a · quantile(D, 1 − q) + b for a < 0 (the smallest value with cdf ≥ 1 − q, for a discrete D) |
| `share(a * X + b op y)`, op a comparison | from cdf(D, ·) at (y − b)/a, the side and the edge chosen by op and the sign of a; for a discrete D, the mass at the point counts exactly |
| `cdf(a * X + b, y)`, `pdf(…)`, `pmf(…)` | the transformed function of D (`pdf` divided by \|a\|) |

`cdf`, `pdf` and `pmf` take a distribution value, a random fact read directly, or an affine form of one; anything else is refused (`LOAD-RANDOM`: the question is a `share` over paths). A query with `where` has no closed form, nor has one that reaches a fact whose sample is pinned (Pins, below): it states `paths N seed S`.

**Decisions and projections.** A decision never sees its own path's future (P-13, R-04). On a path it reads a random fact by its sample only at a law date on or before its own day; a sample at a later date is refused (`RUN-FUTURE`). So a projection a decision reaches (§5.14.1) reads a random amount dated after the decision's day as a question about the distribution as known on that day, which reads no sample and is the same on every path and in a run without paths (`plan v` is gone, D-11). The question is written in one of two ways:
- a **`random` clause**, on the projection read or on the ask line: `random at mean` evaluates each random amount the projection reaches after day t at its distribution's mean, and `random at quantile(q)` at its q-quantile; amounts read on or before t take their samples;
- a query around the read, `mean(…)` or `quantile(…, q)`, computed in closed form.

With neither, the read would be the decision's future, which no run can give it (on a path the sample is that path's future, and a run without paths has no sample), so it is refused: at load where load sees the projection reach the random fact after the decision's day (`LOAD-RANDOM`, §8.3), otherwise at the read (`RUN-FUTURE`). A query that has no closed form is a statistic over paths: it stands only in an ask line that states `paths N seed S` (`LOAD-ASKONLY` elsewhere), never in a rule of the same run, so a decision that depends on one is two scenarios, one per outcome, compared there (§6.4, `diff` in §5.18). An ask line's own projection is no decision: it reads an amount after t by its `random` clause, by the line's query, or on a path by that path's sample (§5.14.1).

**Pins.**
- `with sample growth[2030] = 1 %/year` fixes the sample at that address on every path; `with sample growth[*] = e` fixes every address of the fact; `with sample growth[2030].u = 0.37` fixes its uniform (Gen's constraints, SF-28). The pinned expression is evaluated unpinned; a hole pins the hole (§5.11).
- **The test mode** for statistics against closed forms (CHG-13) pins a parameter's sample, `with sample theta = theta_mid`. Every sample pin is listed in the register (`pinned sample:`), so a forecast with a pinned parameter can't be mistaken for one without.
- `with growth[2030] = normal(…) to …` is an ordinary pin: it replaces the distribution and keeps the address's uniforms.

**The closed list of distributions.** Each has `cdf`, `pdf` or `pmf`, `quantile` (type 1: the smallest x with cdf(x) ≥ q) and, but for `discrete`, a mean. Called on a distribution value, `cdf(d, x)`, `pdf(d, x)`, `pmf(d, k)` and `quantile(d, q)` are these functions, in a def as in an ask, exact or written with `to u` where transcendental (`quantile(normal(0, 1), 97.5 %) to 1e-9`); `mean(d)` of a distribution value is refused (`LOAD-RANDOM`, A-42), and the table's mean is the one a closed form and `random at mean` read. A distribution the list doesn't hold is written from one it does: one of n known values drawn with equal weight is `e[x]`, e keyed by IDX and x a random fact whose value is `discrete(1 for k in IDX)`.

| Distribution | Parameters | Sample | `cdf`, `pdf`, `pmf`, mean |
|---|---|---|---|
| `normal(m, s)` | s > 0 | needs `to` | `cdf` and `pdf` need `to`; mean m |
| `lognormal(m, s)` | of the log; s > 0 | needs `to` | need `to`; mean exp(m + s²/2) needs `to` |
| `exponential(rate)` | rate > 0 | needs `to` | need `to`; mean 1/rate |
| `gamma(shape, scale)` | both > 0 | needs `to` | need `to`; mean shape · scale |
| `uniform(a, b)` | a < b | needs `to` (its exact sample has a 2^64 denominator) | exact; mean (a + b)/2 |
| `poisson(mean)` | mean > 0 | exact (quantile decided exactly) | `pmf` and `cdf` need `to`; mean exact |
| `binomial(n, p)` | n an int ≥ 0, 0 ≤ p ≤ 1 | exact | exact (N-220, CHG-10); mean n·p |
| `negbin(mean, size)` | mean > 0, size an integer > 0. The count of failures before the size-th success: pmf(k) = C(k + size − 1, k)·p^size·(1 − p)^k for k = 0, 1, …, with p = size/(size + mean); variance mean + mean²/size | exact | exact; mean exact |
| `discrete(w for k in IDX)` | w ≥ 0 and known, Σw > 0; its values are the keys of IDX, in index order; over a key list or a stepped range it is refused (`LOAD-DOMAIN`) | exact (a draw by lot, N-1402) | `pmf` and `cdf` exact; no mean (keys aren't numbers) |

An invalid parameter is refused at that key (`RUN-PARAM`), never NaN (N-221). A `cdf` below its support's lower end (or at it, for a continuous distribution) is exactly 0, and at or above its upper end exactly 1; a `pdf` or `pmf` outside its support is 0. Neither is refused: each is a rational on a cell edge (§7.3). A hole parameter makes the distribution, and every sample of it, a hole naming it.

**A random number of items.** Sampling makes no keys: a kind's things are declared (P-2). A random number of items is a bounded index of slots, read by a range that ends at the draw (`sum(x[m, s] for s in 1 .. n[m] step 1)`, with `x` keyed by the slots; a draw of a `dist` with no unit is a `num`, so the range states its step, §5.11). A draw past the last slot reads a key outside the slots' index and is refused on that path (`RUN-KEY`, §4.2), and at load where load sees the draw's support exceed the index (`LOAD-RANDOM`).

**A foreign program's random stream** is no row a text can give, since a path's samples are the engine's own (§7.4). A step that needs one is a hole whose question says why (§8.1, P-17).

**Refusals:** `LOAD-RANDOM`, `LOAD-TO`, `LOAD-ASKONLY`, `LOAD-DOMAIN`, `RUN-FUTURE`, `RUN-PARAM`, `RUN-EMPTY`, `RUN-KEY`.

**Cost:** one SHA-256 and one quantile per sample, about 0.3–1 µs; discrete quantiles cost O(k). A path ask costs N times the random cone, plus O(N log N) for its statistics. **Cases:** none of the 79 models samples: LH-17, LH-18 and CH-06 compute their probabilities exactly, CH-03 scores a stated sample and writes numpy's stream as a hole with its reason, and MP-14 and CMP-02 hold the decided lot as a row (`coverage.md` K13, K14). The distribution functions (`cdf`, `quantile`) are used by CH-01, CH-03 and LH-17; the two Monte Carlo models of `montecarlo/` use paths (the hard set's Q-D-10).

### 5.17 (removed in 2.0: `regime`)

<!-- tally2: GD-1 (the user, 2026-10-27); D-2, P-4 -->

Tally 2's design round had a `regime` block, a namespace with an export list. It was dropped at G2 because it adds no answer that top-level items can't give (the minimality bar's test (a)); namespaces and modules are tally 3.0's. Its general parts are the language's own and live in their homes: postings, reach by kind and things named inside rules in §5.14; lots and the state a rule carries between events in §5.12; obligations as places in §5.10; several rules on one event in §5.14.2. The section number is kept, so §5.18 and §5.19 keep theirs.

### 5.18 ask

<!-- tally2: D-4, D-6, D-7, D-10, D-11, D-14, D-15; C-4, C-20, SF-6, SF-22, SF-23, M-12 -->

```
ask q1: payable[f, m] for f in families, m in months law 2026-12-31
ask q2: check flows(wallet, 2026-03) == rec_total[wallet, 2026-03] law 2026-03-31
ask q3: solve npv == 0 PLN for rate in [0 %/year, 20 %/year] to 0.0001 %/year law 2026-06-30
ask q4: net_due[ewa] with outside_income[ewa] = 0 PLN law 2021-12-31 as known 2021-06-30
ask q5: explain payable[f1, 2016-04] law 2016-04-30
ask q6: readers-of base_amt
ask q7: firm_value law 2026-06-30 partial
ask q8: diff(free_cash[m] for m in months) in job_change against base law 2026-12-31
ask q9: low(wallet, 2026-07-01 .. 2026-12-31) from 2026-07-01 law 2026-07-01 windows each
```

An ask is a question. It can't book anything, since its grammar holds no move (N-1601).

| Form | Prints |
|---|---|
| `expr, expr` | values, one line per binding tuple and per distinct result across the axes reached, the expressions in the order written. A random value needs a query or a path (§5.16) |
| `check e == f` | `ok`, or `differs: e = …, f = …` and, where `e - f` is defined (§3.2, §4.2), the difference e − f; a hole or refused side prints in its place (`?` with its questions, or `refused: …`), with the other side's value beside it (`e = 120.00 PLN, f = ?`). Sides read at different law dates list each. Used for replay against records and for stated preconditions |
| `solve f == g for v in [lo, hi] to u` | the grid point nearest the root of F(v) = f − g (below). v is a ref and may name keys (`for rate[line_a]`): only that cell is pinned |
| `explain ref` | the derivation, in the stated order (below); `explain bal(x, d)` explains a balance, `explain bal(x, d) from t` a projection, and `explain low(…)` the `min` over days it is (§5.14.1, `ENGINE.md` §10) |
| `inputs-of ref` | the set of rows its evaluation read, with their sources (N-1705), the rows an `at` reads included; a rule's `v` version isn't a row (`explain` shows it). For `marginal`, the rows it sums (lo < x). Across an axis it fans out like any result, one set per line (§5.5) |
| `readers-of x` | every item that reads x, directly or not (any node of the item graph, bands and places included, `ENGINE.md` §3.1 step 5), each once, the direct readers first, then by depth, then by id. Of an event kind, every binding over it or over a kind it is a subkind of (D-15) |
| `residuals x span [by f]` | the residual ledger of place x over the span: the model at the ask's cut against the observations, grouped by month and move, or by field f of the observation input (`ENGINE.md` §6.3) |
| `diff(e) in S against B` | e evaluated in world S and in world B, keyed by stable ids (§2.5): for each line, `added` (only in S), `removed` (only in B) or `changed` with both values; the unchanged lines are counted, not printed (D-10) |

**Explanation order.** Every value traces to the rows, rules and sources that made it, in a stated order (P-17, D-15). `explain` prints the def's rules in canonical order (§2.4), each with its label and state: won; beaten, by whom; not applicable; unknown, when its `when` is a hole (a rule that wins with a hole value is `won`, and its value is the hole). Under the winning rule come its causes, depth first, in the order their terms are written in the rule, left to right; an aggregate's terms in binding order (§2.5). A `min`, `max`, `argmin` or `argmax` is explained by its first winning term in binding order. Each cause prints once; a cause met again is referenced by its line. A cause that is a row prints its version, source, locator, governs span or instant, and known date (§3.7); then come the law date and cut, givens, assumptions, holes, pins and samples. A def evaluated at walk points (§5.14.1) is explained as a def cell at each walk point that read it, in walk order, each with its point (day, stratum, movement, keys) and its running reads as balances up to that point (`ENGINE.md` §10); with none, that nothing read it at those keys.

**The world of an ask.** Every clause after the body sets the world the lines are evaluated in, and each is printed with the result (C-4). Their order is the grammar's.
- `for` binds keys: one line per binding tuple whose `where` holds. A tuple whose `where` is a hole prints its line as that hole, its value unevaluated, and so does a `diff` line in the world where its `where` is a hole.
- `in S` names the world: a scenario or the model itself, by its id (`in household`). An ask in a scenario file states it (`LOAD-WORLD`); a base ask can't name an item declared only in a scenario, which `diff` can (D-10). An ask in a base file that names none is in the world of the file that holds it, the model's: that is scope, as for every item of the file. A branch changes nothing in its base and nothing reads an ask (P-12), so an ask is checked and answered only in the worlds its line is evaluated in (its `in`, both worlds of a `diff`, each `as if {scenario s}` in it), and one that names a scenario loads only with that scenario's file (§6.4). A base ask that reads an item a scenario's rule makes random (`replace rule x.drawn beats base = r`) stays plain in the base and refuses nothing in the scenario (`ENGINE.md` §3.1 step 10).
- `with` pins (§5.11): values, sample pins (§5.16), `law`, `known`, `wording`, an axis option, a scenario. In a joint model a value pin or an `as if` may name a member's row (§6.1); like every pin, it sets that ask's world only and changes no member's file (P-11, P-12).
- `law D` is the line's law date: that of every `at reader` def the body reaches through `at reader` defs only (a chain of readers stops at a def with a written `at e`, except for what e itself reads, §4.3), and of a law-dated input or band the line reads directly. Which lines must state it is §4.3's (`LOAD-LAWDATE`).
- `as known D` states the line's cut K, in place of the run's stated cut, the model's `on` (§4.5).
- `windows each` decides every window the line meets through the whole walk (§4.4): one line per maximal sub-window, or combination of sub-windows, over which the answer is constant, and one plain line when the answer doesn't depend on the day (D-7, SF-6, SF-23).
- `with uncertain` counts unfulfilled movements of uncertain kinds in every projection and binding the line reaches (D-6).
- `random at mean|quantile(q)` gives every projection the line reaches its `random` clause (§5.16).
- A read's own `windows`, `random` or `with uncertain` clause overrides the line's for that read.
- `paths N seed S` and `path p`: §5.16.
- `partial`: below.
- `orient lower|higher`: §5.15.

**Reads in an ask.** An ask reads any balance on any day of the run: `bal(x, d)` at the ask's cut, the projection `bal(x, d) from t` (§5.14.1), `low` and `next`, `bal(x, d) modelled` and `gap` (§5.19), `flows` and `value`. **Sums over planned movements** count what is planned and observed at the cut, and the register lists the planned instances counted that no observation has fulfilled yet (M-12).

**Partial answers** (D-7, P-9). A line marked `partial` prints, when its value is a hole, its known part and the named holes it is linear in, each with its coefficient: sums, differences, and products and quotients by known numbers (`412 300.00 PLN + 1 × ?"value of the firm share"`). A rounding, `min`, `max`, a band, a comparison or a product of two holes meeting a hole makes the line a plain hole naming every hole. Every named hole is printed. Defs are unchanged: only the ask line computes the residual, and filling the holes later gives the same result as a rerun.

**Literals.** An ask allows every literal a def allows, plus those that state the question (§7.1).

**Ask-only functions** (`LOAD-ASKONLY` elsewhere, N-1409):
- a query that has no closed form, a statistic over paths (§5.16);
- `solve`, except in a rule that points at a version whose source names the method (below).

A deflator is written in the ask, `x * idx[base] / idx[d]`: a lens applied only when a question is asked (N-718, N-1604).

**`solve`.** v is an input or a def, pinned at each step; the pin replaces v's own row while the ask runs, a hole row included. F(v) = f − g. The grid is lo + i·u, and lo and hi must lie on it. The bracket's literals and u take v's type, so a rate is solved over rates. A hole in F at any point the solver evaluates makes the result a hole carrying F's questions; `RUN-SOLVE` is only for a known F. The pin of v isn't listed in the register, since v is the question (`pinned:` is for `with` pins), and the result prints plain: a grid point is exact.
1. If F(lo) = 0 the answer is lo; if F(hi) = 0 it is hi. Otherwise F(lo) and F(hi) must have opposite signs, or the ask is refused (`RUN-SOLVE`: "no sign change on [lo, hi]"). A root outside the range is therefore never reported as the range's edge.
2. a = lo, b = hi. While b − a > u: m = a + u·⌊(b − a)/(2u)⌋. If F(m) = 0 the answer is m. Otherwise replace whichever of a and b has F of the same sign as F(m) by m.
3. Now b = a + u. Evaluate F at c = a + u/2. If F(c) = 0, or F(c) has the sign of F(a), the answer is b; otherwise a.

The result is the grid point nearest the root, with a tie going up. A rounding the source states ("to the nearest cent") is then a `round` of that result. That is the question the source asks, not "the largest cent under the level". **The least value that meets a condition**, the inverse of a step function, is not a `solve`: it is `min(v for v in lo .. hi step u where c)` over a stepped range (§5.11, D-15), exact, with no sign change needed. `solve` may also appear in a def rule that points at a version (`v`) whose source names the method: a gross-up the law solves, N-217. It is then the rule's whole value (§2.3 `solvex`), its bracket's literals are allowed (§7.1), and its pin of v holds while the rule's value is evaluated, at the rule's keys and law date.

**Output.**
- **Lines.** An ask prints one line per binding tuple and per distinct tuple of its expressions' results across the axes reached (§5.5). Lines are ordered by binding tuple in binding order (§2.5), then by the axes' options in declaration order, the axes taken in id order (§2.4). On each axis a line sorts by the first option it names: a set (`down | half-even`) at its first option's place, and a `*` before every option. A tie goes to the next axis. A line's register is the union of what its expressions read; `explain` gives each expression's own.
- Money prints with its minor-unit digits and its currency (`6 138.00 USD`).
- A `num` prints exact, as the shortest terminating decimal (§2.4: `6 536`, `2.7`, a rounded `num` included) followed by its unit (`6 536 PLN`), or as `a/b` when it doesn't terminate. Beside an `a/b`, and beside a terminating decimal of more than 15 significant digits, comes a 15-significant-digit display, rounded half to even: a reading aid, never the value.
- A rate prints as a decimal with its period (`0.076 /year`), by the same rule, never as `%`.
- A value that rests on a correctly rounded transcendental, with no `round` after it, prints `≈` and its error bound (§7.3).
- A hole prints `?` with all its questions, each once, sorted by code point (as ids are, §2.4). A refusal prints `refused: CODE message`.
- `none` and `never` print as themselves. An imprecise date prints as its window (`window(2020-07-01, 2020-07-31)`); an interval no comparison decided is `HOLE-IMPRECISE` (§4.4).
- Under each result comes its register (§8.2): the assumptions, givens, held rows, anchors, pads, adjustments, pins and samples it reached, and the planned instances it counted unfulfilled or left out (N-713, D-13).

**Refusals:** `LOAD-ASKONLY`, `LOAD-LAWDATE`, `LOAD-WORLD`, `LOAD-RANDOM`, `LOAD-ORIENT`, `RUN-SOLVE`, and every refusal of what the body reads.

**Cost:**
- An ask costs its cone.
- A path ask costs N times the random cone, plus O(N log N) for statistics.
- `solve` costs ⌈log₂((hi − lo)/u)⌉ + 3 evaluations of F's cone; a `min` over a stepped range costs its range, or its bisection when the engine proves c monotone (§5.11).
- `as known` costs one evaluation per distinct cut reached; `diff` two worlds.

**Cases:** all. `solve`: LH-10, LH-11, LH-17, MP-07, UL-08. `check`: CB-01, CB-03, CB-05, CB-08, CB-09, CH-02, CH-04, CH-05, CH-06, CMP-01, CMP-02, MP-02, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, UL-03, UL-04, UL-08, UL-11, UL-13. `explain`: CB-07, the household; `inputs-of`: CMP-01; `residuals`: the household; `diff`: change 7. Path statistics: the two Monte Carlo models (§5.16).

### 5.19 Observations

<!-- tally2: D-2, D-3, D-6, D-7, D-8; C-11, C-26, SF-14, M-14 -->

```
kind groceries: transfers
kind fund_buys: event certain
  leg units: funds
  leg cash: PLN

input bank: observed
input fund: observed

version statement_11
  cite bank_export_2025_11
  governs 2025-11
  known 2025-12-01
  covers konto
  pad konto from unexplained
  bank[r17] = (on: 2025-11-03, kind: transfers, leg cash from konto to landlord amount 2 800.00 PLN, fulfils: rent[2025-11]) "line 17"
  bank[r18] = (on: 2025-11-04, kind: groceries, leg cash from konto to world amount 234.22 PLN) "BIEDRONKA 1234"
  stated[konto] = 4 150.00 PLN at 2025-11-30 end "saldo końcowe"

version register_2026_02
  cite fund_register
  governs 2026-02
  known 2026-03-05
  covers ike
  fund[r3] = (on: 2026-02-02, kind: fund_buys, leg units from market to ike amount 4.2167 etf, leg cash from ike_cash to market amount 500.00 PLN) "register line 3"
```

An observation is a real movement from an import: a bank row, a broker's fill, a register's line (N-1201). It is the same construct as a planned movement (§5.14), of an event kind, with named fields and legs in any unit; only its certainty differs. This section is the home of P-10: an observation's link to the instance it realises is written, never inferred; where statements cover a place, its observations are booked and the planned movements are the modelled past; and a statement's gap is a readable fact, what it does written.

**The rows.**
- **`input X: observed`** declares the rows. Its keys are the rows' ids, an open set stated by the rows themselves (`bank[r17]`, M-14).
- **Fields are named**, never positional (`LOAD-FIELD`): `on`, the day the movement was observed; `kind`, an event kind; each leg its kind names, written as in a move (`leg cash from A to B amount q [on d]`); `fulfils`, when it realises a planned instance or settles an adjustment; and the kind's attributes. A missing leg or attribute, a leg of the wrong unit, or a kind of `uncertain` certainty is refused (`LOAD-LEG`): an observation is what happened. A window belongs in an attribute (a giver's acquisition day), never in `on`, the observation's or a leg's (`LOAD-LEG`).
- **Membership.** An observed row is in the kind its `kind:` names and in that kind's parents, and `o in K` tests it, as for any thing (P-2).
- **Bindings.** A binding over an observed input, `o in X over s`, ranges over X's rows dated in s, each once, whether or not a version states two of them `same`, since an input's keys are its rows. A binding over an event kind ranges over movements, planned and observed, each once: a pair of rows stated `same` is one movement (One movement on two statements, below), and a planned instance and the observations that fulfil it are counted once, by the observations where they are booked, otherwise by the instance (Coverage, below).
- **Before the run.** Observed rows dated before the run's start are bound by bindings and fills and booked by none; the opening reading at the start anchors the balance. The run bounds booking, not facts (P-15), so history before S, lots held then included (§5.12), is given by observed rows.
- **A correction or a dropped row** is a later version restating the same key, with `replace` (§5.3); `replace bank[r18] = none` removes the movement with its legs (SF-14). The row keeps its id (N-1207).

**Links.**
- **Only confirmed links are written.** The importer proposes `fulfils` links, and the engine never infers one (N-1207, `OPERATIONS.md` §2). A `fulfils` naming something that is neither a planned instance nor an adjustment (a place, an input) is refused (`LOAD-KIND`).
- **A link is judged by existence, at the ask's cut.** A link reads only what it names: the existence of the instance, by its keys, `where` and `when` (an instance exists only where all three hold, `ENGINE.md` §4.1), never its amounts. So it is judged at the cut that reads its row: the ask's, or, in the residual ledger, which reads observations as known on the model's `on`, that date (`ENGINE.md` §6.3). At that cut, a `fulfils` naming an instance that doesn't exist, or one whose legs don't touch the observation's places or that has a leg the observation lacks, is refused (`RUN-RECORD`), and so is a `fulfils`, an observation's or a settling move's, naming an adjustment that no version made by that cut, or whose version makes no change to that leg there (§5.14.3). An instance fulfilled at that cut is counted by its observations and never re-measured, so no earlier cut judges its link (`ENGINE.md` §5 step 2).
- **A link to a change learned later** is written in a version known on or after the change, by `replace` (`replace bank[r1] = (…, fulfils: adjustments[wage[2026-03].cash, raise_2026])`), so cuts before it read the row unlinked.
- **The importer's rules are data too.** The built-in input `account_place[a]` maps the bank's account id a, a key of the built-in open index `bank_account`, to a place, each row with its source. The model may read it like any input, and the importer reads it to propose links (`P:REC-01`). The counterparty map and the matching tolerances are the importer's own settings, outside the language (D-8).

**Realised amounts.** `observed(m[k])` is the realised movement of a planned instance: a record whose legs are the sums of the legs of the observations that fulfil it, rows stated `same` counting as one (a leg's `from` or `to` is `none` where their legs name different places), known at the cut, and whose `day` is the last of their days; `none` when nothing fulfils it yet. What remains of the instance is written as a difference (`rent_amt[m] - observed(rent[m]).cash.amount`, behind a `!= none` test). An observation that fulfils a hole movement answers it: its day and amounts are the instance's (`S:9`, `Q:B-09`).

**Coverage.** `covers x [span]` on a version says that its observations are complete for place x over its `governs` span, or the span written (§5.3). Each of its observations of x must be dated inside that span (`LOAD-COVERS` otherwise: an October row in a statement that covers September is refused, never booked or dropped in silence). On a covered day before the cut K:
1. the observations of that place and day are booked;
2. the planned movements that touch the place are the **modelled past**: computed at their booking cut and compared with the observations (`ENGINE.md` §6), never booked (N-1202), but for those of uncertain kinds that nothing fulfils, which stay uncertain (§5.14, `ENGINE.md` §4.1 step 3). A planned instance of a certain kind in a covered span that nothing fulfils is listed in the register as open.

On days no version covers, and from K on, the planned movements are booked: the plan. An observation realises the instance it fulfils, so the instance is counted once, by its observations, wherever they fall, and is kept as modelled past: on any day, an instance that an observation the cut sees fulfils is never booked or measured, and its observations stand in for it where they are booked (below). So a payment planned for a day no statement covers and seen on a covered day is booked once, on the day it was seen, and a binding over its kind counts it once.

**Booking an observation.** Each end of an observed leg is booked at its place from the observation of the version that covers that place on that day; an end at a place no version covers (the outside places, an account with no statement) is booked from the observation itself. So an observation never books an end twice. It is a movement on its own day (D-8), booked in that day's first batch with the plans (§5.14.2), a leg with its own `on` in its own day's (§5.14), except one that fulfils a decision instance dated the same day: that is the instance's movement (`fulfils`), so it is booked at the instance's point in the day's order, right after the instance is computed, and a decision before that point doesn't see it (P-7). One dated before or after the instance it fulfils is booked in its own day's first batch, like any other. A decision instance that is modelled past (Coverage, above) is computed against the booked balances less its own observations (`ENGINE.md` §4.2 step 3).

**One movement on two statements.** A transfer between two places with statements appears on both, as two rows. A version states that they are one movement with `same r1 = r2` (§5.3), and rows stated `same` are one movement (§6.1). For observed rows this means: each end is booked once, from the statement that covers it (above); the movement is bound once by a binding over its kind and counted once by `observed(m[k])` and `flows`; and its `fulfils` is stated on either row, or on both alike. Where the two rows disagree on an amount or a day, each end is booked as its own statement says, `WARN-LEGS` is printed, the difference shows in the residual ledgers of both places (`ENGINE.md` §6.3), and a read of the movement away from its places (its leg's amount in `observed`, in `flows` of a move or through a binding) is the disagreement's hole, `HOLE-SAME`, naming both rows, as is a binding that meets the movement over either of two kinds the rows state (§6.1). Rows not stated `same` are movements of their own, each counted in full (the parts of one payment). Two unjoined rows of one transfer that both fulfil one instance, their legs naming the same two places and their versions each covering a different one of them, would each book one end and count the whole: they are refused (`RUN-RECORD`).

**Balance readings and their gaps.** Rows of the built-in reading input `stated[x]` give a place's balance at an instant (`at D end`, `at D start`, `after m`, §4.6); `opening[x]` gives its balance where the walk starts, and always anchors. A reading whose value is `none` states no balance: it neither pads nor anchors, and with `replace` it removes the reading it replaces, as `replace bank[r18] = none` removes a movement. The **gap** at a `stated` reading is the reading less the booked balance at its instant. Each version that states readings of x says what the gaps are (`LOAD-GAPROLE`, C-11):
- **`pad x from y`**: each gap is taken up by a movement of the prelude kind `pads` between y and x at the reading's instant, of the gap's size, its direction set by the gap's sign. `flows` and bindings see it, and the register flags it (`pad:`). x's balance after the instant is the reading.
- **`anchor x`**: the walk restarts from the reading: x's balance after the instant is the reading, nothing is booked, and `flows` doesn't count the difference. The difference is the reading's balance line in the residual ledger (`ENGINE.md` §6.3).

Either way, **a balance means what the statement states at the last reading, plus what is booked since**; before a place's first reading, its opening plus what is booked since. Every figure that reads x's balance from that instant on lists the reading in its register as §8.2 says: `anchor:` for the last anchoring reading, `pad:` for each pad it counts.
- **`gap(x, d)`** is the gap at the last `stated` reading of x on day d, a fact an ask or a def reads (`S:10`, `Q:B-08`); with no reading that day it is `HOLE-NOREADING`.
- **`bal(x, d) modelled`** is the model's own balance of the modelled past on a covered day: from x's last reading before d, plus the planned movements that touch x, at their booking cut (`P:RULE-07`, `ENGINE.md` §6.2). Comparing it with the reading is the residual.
- **Holes.** A hole movement dated on or before the reading keeps x's balances before it holes, and its questions stay listed on those balances, on the gap and on the pad (whose amount is then a hole). It no longer makes x's balances after the reading holes: the reading states them. A movement that may fall after the reading (a hole day, or a window that runs past it) still does, from the first day it may fall on. A hole movement's other place is fixed only by a reading of its own.
- **Decisions before a reading stay as booked.** A reading at the end of D applies after D's decisions, so it never changes them. A decision after the reading reads the balance it states, under every cut that sees the row, even one known after the decision's day (§5.14.3).
- **The read rule decides who sees it.** A view whose cut is before the version's `known` doesn't see the reading, so it doesn't pad or anchor (§4.6). Like an observation, a reading applies only before the cut K, except an `opening` reading at S start, which anchors whatever K (`ENGINE.md` §4.2 step 0).
- A place alive before S with no `opening` has a hole balance until its first reading (`HOLE-ANCHOR`, §5.10).

**Readings at a place that keeps parts** (tally 2.1, R-6, G1 Q-5). A reading states a total; what it says of the parts:
- **Stated parts.** A version may state a reading's parts as rows of the reading input with `of`: `stated[x] of q = v` at the reading's instant and, where the walk starts, `opening[x] of q = v`, q a party, a batch or the pair, each row citing its document as any row does. They are history that no event gives, stated once per reading. They stand beside the reading's total, in the same version and at the same instant (`LOAD-PARTS` otherwise), are each at least 0 and no two of them overlap (a party and a batch at a place that keeps pairs, or a party or a batch and a pair that holds it), both `LOAD-PARTS` otherwise, and sum to at most it (`LOAD-PARTS` where load sees both, `RUN-RECORD` otherwise); the rest of the total, if any, is the unstated rest (below). An `opening[x] = 0` needs no parts. A batched place's opening is 0, or its batches named by the movements that brought them in (observed rows before S, which bindings reach), or a part whose batch is a hole.
- **A pad** is a movement of `pads`: at x it is an arrival, labelled by the arrival rule (a cited house reading of the gap, or a written `?` rule that makes it a hole part, §8.1), or an outflow, taken by the leaving rule, like any other. A pad no rule reaches is refused (`RUN-NO-RULE`).
- **An anchor whose gap is 0** leaves the parts as they are: the reading confirms the booked total, and nothing is unknown.
- **An anchor's gap with no stated parts** is unknown money: a positive gap is a hole part of the gap's amount, possibly anyone's; a negative gap is a take of its size by an outflow nobody knows, so the order's terms on w are holes, its terms on k still rank, and the take is undecided as far as the terms on w reach (§5.14).
- **Stated parts against booked parts** (RC-5). After the reading's pad or anchor has set the total, a stated part is q's balance, as a stated total is the place's. Where it differs from q's booked part, the difference is q's own gap: where positive, a part of q's of that amount, from the reading's document; where negative, a take of q's parts by the leaving rule, its terms on w holes as above. The unstated rest has a gap of its own, over the labels the reading doesn't state, by the anchor's rules above. Every such gap is listed with the reading, as a total's gap is, so nothing is overridden in silence, and a stated part never takes another label's parts. A reading at a day's start that pads and states parts, at a place the day's first batch also moves, is refused (`RUN-RECORD`): its pad takes and arrives with the batch (`ENGINE.md` §4.2), so q's booked part at the reading is no one point's; the reading written at the end of the day before states the same balance, and the batch takes from it.
- **Nothing is inferred** from the parts before the reading: neither their proportions nor their owners carry across a gap.

**Reading history.** Observation rows are ordinary rows, masked at K like any other (N-1219), and read at any law date: their version's `governs` span says what the statement covers, not when its rows apply (§4.6).
- A def may aggregate them, so an estimator over history up to K is a def (N-1212, CHG-12): `sum(-o.cash.amount for o in groceries over y - 1 where o.cash.from == konto)`.
- **`covered(x, span)`** is true when versions known by the cut cover place x on every day of the span, and false otherwise; it is never a hole. A def that aggregates an observed input's rows over a span (not a binding over an event kind, whose plan instances stand on the days no statement covers) must guard the aggregate with it, in its rule's `when` or an `if` (`LOAD-COVERED` otherwise): a `covered` test of the place over the aggregate's span, written as the aggregate writes it (`when covered(konto, (m - 3) .. (m - 1))` for an aggregate `over (m - 3) .. (m - 1)`), so a month with no statement is a hole with its question, never a sum of nothing read as 0:

```
def spend_month[c in categories]: money PLN
  rule budget = spend_budget[c]
  rule learned beats budget = sum(-o.cash.amount for o in spending over spend_window where o.category == c) when covered(konto, spend_window)
```

Until a statement covers the window, the budget stands; a rule with no fallback would give a hole in its place.

**Refusals:**
- `LOAD-FIELD`: a positional observation.
- `LOAD-LEG`: a missing leg or attribute, a leg of the wrong unit, an uncertain kind, a window in `on`.
- `LOAD-KIND`: a `fulfils` naming something that is neither a planned instance nor an adjustment.
- `LOAD-COVERS`: a `covers` naming something that isn't a place, or a version holding an observation of a covered place dated outside its span.
- `LOAD-COVERED`: a def aggregates observations over a span without a `covered` guard.
- `LOAD-GAPROLE`: `stated` readings of a place with neither `pad` nor `anchor` for it.
- `LOAD-PARTS`: `of` on a row of another input, a reading's parts without its total in the same version at the same instant, or stated parts over the total (where load sees both).
- `RUN-RECORD`: a `fulfils` naming an instance that doesn't exist or doesn't touch the observation's places, or an adjustment no version made by the cut; two rows of one transfer, not stated `same`, that both fulfil one instance; a reading's stated parts over its total.
- `RUN-NO-RULE`: a pad at an owned place that no arrival rule reaches.

**Cost:** the observations of a K class are sorted once, O(R log R), and booking is O(1) each. **Cases:** the household and its replay (spec §3.7, G2, G4); the hard set's Q-B-08 and Q-A-05 (a unit-moving register). None of the 79 cases replays bank rows; their history (LH-15's market series, LH-13's earnings) is ordinary input rows.

---

## 6. Files, versions and amendments

<!-- tally2: D-6, D-9, D-10, D-12, D-13; C-14, C-17, SF-14, SF-25, SF-31, SF-32 -->

### 6.1 A reality is a set of files

A model is one or more `.tally` files in one directory tree. Each file starts with one of three headers:
- **`model ID`**, then `tally N` and the clauses of §5.1, with any `use` and `same` lines (below). Exactly one file has it (`LOAD-HEADER` otherwise).
- **`part ID`**, then `tally N`, and `law` for a **law part**. It holds sections only. A law part holds an act's sources, kinds, inputs, bands, defs, fills and moves, the versions of its figures, and the `check` asks its gate runs (M-07); the gate of `OPERATIONS.md` §3 applies to it, and to nothing else. A law part whose defs are stubs (a hole rule) passes the gate at once and gives its holes.
- **`scenario ID over ID`**, then `tally N` and `from DAY` (§6.4). Its sections exist only in that scenario.

Every file declares the same `tally N` (`LOAD-VERSION`).

**The meaning of a reality is the set of its items.** The order of files, and of sections in a file, changes nothing. An item lives wholly in one file, so a def and the rules written in it are in one place (a rule a version brings lives with that version, §5.11). Splitting is a convention for readers; the engine checks only the headers, `law` included. For the household the layout is:

| File | Holds |
|---|---|
| `reality.tally` | the header, the kinds, the axes |
| `law/<country>/<act>.tally` | one act, a law part: its items, its sources and the versions of its figures, and its `check` asks |
| `people.tally`, `places.tally`, `moves.tally` | the household's structure: entries of things, its own places, its planned movements |
| `facts/<year>.tally` | the household's own versions: persons' entries, jobs, contracts, choices |
| `statements/<source>/<period>.tally` | imported observations (§5.19) |
| `scenarios/<id>.tally` | one scenario per file |
| `asks/<topic>.tally` | saved questions |

A law used by several realities is **copied in, never linked**. A reality's hash covers every file, so a result can always be reproduced from the reality alone. A case model is usually one file.

**Joint models** (D-9). A question across models (two households, a family's members) is asked in a **joint model** of pinned, closed imports with written identities (P-11): a model of its own that imports each member, pinned by the hash of its canonical text (`ENGINE.md` §3.1 step 1).
- **`use a = "file" sha256 "…"`** in the header imports a closed model; its items are read as `a.item` (`a.wallet`, `a.due[ola, 2025]`). A file that doesn't match its hash is refused (`LOAD-PIN`). The joint model may add items of its own (kinds, defs, versions, asks); it never changes an imported one, and each member stays reproducible alone.
- **A member is closed.** A member's items mean what they mean alone: a binding inside a member ranges over that member's things, the prelude's kinds included. The joint model's own bindings over a shared kind range over every member's things, read under their prefixes (`a.wallet`), and over the joint model's own. The prelude (§5.7) is one, in the joint model and in every member, so its kinds are shared: `parties` among them, so every member's owners are things of one kind (tally 2.1, §3.5), while two members' parties are one party only by a `same` line on the things (`same a.ania = d.anna`), never by equal names. A kind a member declares is that member's (`a.spending`): a kind under it stays under it, and two members' kinds are one, and so shared, only by a `same` line. Things stated `same` are bound once.
- **Identities are written**, never inferred from equal names. `same p = q` states that two are one; each side names a kind, a thing (a keyed place at its keys included), a row (an input's value at its keys) or a movement (a move instance or an observed row) of the joint model or of a member, with its keys written as literals (`same a.persons = b.persons`, `same a.a_stmt[a_g1] = d.g_in`, `same a.fee[2026-03] = b.fee_in[2026-03]`). The two sides are of one sort: two kinds declared alike (the same clauses, legs and attributes, and both subkinds of one kind, a prelude kind or two kinds stated `same`, or both of none; their notes aside), two things of one base kind (an `outside` place aside, below), two rows of one type, or two movements; any other pair is refused (`LOAD-SAME`; a law part both members copy: Shared law, below). Two places of which neither is `outside` must also be of one unit and name one price input, if any (`LOAD-SAME`), and a place whose kind keeps parts (§3.6) can't be stated `same` at all (`LOAD-PARTS`), since two members' arrival rules would book its parts; a member's outside place stated `same` as another member's place contributes only its legs to that place, so no thing changes kind (C-14). `same` is an equivalence: lines that share a side form one class, and two kinds stated `same` are one kind, so a binding over either ranges over the things and movements of both. An entity two members name is a thing of a kind in each, joined by `same`; a key of an index is no thing, and isn't joined. An identity isn't dated, so a `same` in a version joins only rows and movements, inside one model (§5.19); one in a version that names a kind or a thing is refused (`LOAD-SAME`): only the header joins them.
- **What `same` does.** Facts about two things, rows or movements stated `same` are read from both sides together; a fact on which they disagree for a day is a hole naming both rows (`HOLE-SAME`), never a silent pick, and the joint model settles it with a version row of its own that states the value. A party stated `same` is one owner: `of p` reads the parts labelled with either side, in every member (§3.6). A thing's entries are such facts: an entry only one side states stands, and on a day on which the two sides' entries into one kind disagree, whatever depends on the thing being alive there is a `HOLE-SAME` hole (in a binding over that kind, an unknown member, §5.11). Movements stated `same`, observed rows among them, are one movement: bound once by a binding over its kind, counted once by `flows` and `observed`, and booked once at each place. A place one of them books (an observed row books where its statement covers, §5.19) takes that one's legs; a place two of them book (one place, or places stated `same`) takes it at the point of the last of them in the day's order (§5.14.2), since a decision's amount is known only at its point, and where they give it different amounts or days it is a hole movement there (`HOLE-SAME`) that may fall on either day. A read of the movement away from the places that book it is a hole naming both rows (`HOLE-SAME`), and two observed rows that disagree print `WARN-LEGS`. One movement has one base kind (P-2), so rows that state different kinds disagree: a binding over either kind that meets the movement is a `HOLE-SAME` hole naming both rows. A binding over a kind that holds both kinds meets the one movement once, whatever its kind, since its membership there doesn't rest on the kind the rows disagree on. A side that names an instance that doesn't exist is refused, as a `fulfils` that names one is (`RUN-RECORD`).
- **Writes into members.** An import is closed, so the joint model's versions state rows and entries of its own items, and of a member's item only at a pair stated `same`, which they settle; a row or an entry of any other member's item would change an imported one, and is refused (`LOAD-SAME`). A scenario over a joint model (§6.4) may replace the members' rows too, named with their prefixes (`replace a.pay[ola] = 0 PLN`): a scenario changes nothing in its base, and each member's files stay as pinned. **Currencies** are one by their code in the joint model and in every member, as the prelude's kinds are, and a member's declaration serves the joint model; two declarations of one code with different minor units are refused (`LOAD-SAME`). Any other unit a member declares is its own (`a.hour`).
- **Consolidation.** A binding over places of several members counts each place once. A movement whose legs cross between members through places stated `same` is internal; the joint model links it to its counterpart on the other side by a written `same` or `fulfils`. The engine never matches by equal day and amount: an internal leg with no written link is listed and makes the group total a hole naming it (`HOLE-SAME`, C-14).
- **The joint model's own movements.** A movement of the joint model's own versions into a member's place changes no member's answer, since the member is closed: it is booked there only where `same` or `fulfils` links it to the member's own movement. Otherwise it is not booked there. A read of that place in the joint model from the movement's day until the member's next reading of the place, a balance included, is a hole naming it, since a balance is a total over the place's movements; from that reading on, the reading states the balance, as after any hole movement (§5.19). Every result that reads the place on or after the movement's day lists it under `left out:` (§8.2) (SF-32). The member's own answers, asked as `a.q`, are unchanged.
- **Shared law.** Two members that copy different versions of one law part (identified by the part's id and the versions it holds, never by equal names) are refused (`LOAD-SAME`).
- A member's asks are asked by name in a run of the joint model, as `a.q`, and answered by the member alone; as for every ask, no line reads one (§5.18). Since imports are pinned and never changed, joining changes no member's answer.

### 6.2 A change adds; it never edits facts or law

Every change to the reality's facts and law is a set of **added lines**: new sections (most often one version), new rules inside an existing def, new notes. A version, once confirmed, never changes what it states and is never removed: the only edit that touches it is a `restate` (a locator's typo, a note, its move to another file), which changes no result. A later version that changes a row, a rule, a band row or a lifespan says `replace` (§5.3, D-12). Because nothing is edited, the old view stays runnable: an ask `as known D`, with D before the change, sees the reality as it was (N-705).

Four kinds of change exist (`OPERATIONS.md` §4):
- **add:** added lines only. This is every common change, and every fact and law.
- **advance:** the header's `on` moves forward, and nothing else changes. It changes results, since `on` is the run's stated cut (§4.5), so it is not a restate: its plan shows what time passing did, the plan that became the past. An add that needs a later `on` (a version known after it) follows an advance.
- **restate:** an edit that changes no result, such as a locator's typo, a note, or moving items between files. The operation proves it by comparing every result.
- **revise:** an edit of the model's structure (a kind, a def, a move, an ask) that does change results, because the model was wrong, not the world. It needs a reason. Versions can't be revised. The store keeps the old text, so the reality as it was recorded can still be run (record time R, `ENGINE.md` §2).

| The change | What is added |
|---|---|
| a new figure, or a figure that changes | a row in a new version; `replace` where a row of an earlier version governs the same keys and days |
| a new person, job or loan | an entry of a thing into its kind, with its lifespan, in a version (`persons + [jan] lives 2029-03-02..`), and the rows about it |
| a new account | an entry into a place kind (`accounts + [lokata] PLN lives 2026-07-01..`) and its `opening` reading; a place declared by the model (`place konto[p in persons]: …`) gains one with each key its kind gains, and that one may take its `opening` reading too, since a place's balance is zero at its lifespan's start (§5.10) |
| a law that changes from a date | a version governing from that date, holding its rows, its band rows, or both, each with `replace` where it changes an earlier one; and `replace rule d.r` if a formula changes (§5.11) |
| a law that changes retroactively | the same, with a `known` after the start of `governs`. The engine records each change to a booked plan instance as an adjustment, but a change to a settlement (a movement whose `fulfils` names an adjustment) makes none (§5.14.3); until a movement settles it, the `flows` that could count it are holes (`HOLE-CHANGED`) |
| back pay decided after the change | a move, or an observed movement, that `fulfils` the adjustment, on the days the law gives; a settlement of 0 where the law gives none (§5.14.3) |
| a commencement postponed, or a sunset extended | a version with `moves V to SPAN` (§5.3) |
| a corrected observation | a version with `replace x[k] = (…)`, or `replace x[k] = none` for a row that never happened (§5.19) |
| something that ends | its lifespan, `replace K + [k] lives a..b` (a death, a closed account, which must be empty, §5.7); rows `none` from its end where a fact stops applying |
| a renamed thing | a new item, and an end to the old one |
| a what-if | a scenario file (§6.4) |

**A new thing no law handles is never a silent 0** (M-22). A new person, job, loan or child is an entry and rows, and every rule that binds over its kind finds it; nothing is expanded. Where the law has nothing for it, the reads say so: a def with no rule that applies at the new keys is refused there (`RUN-NO-RULE`, §5.11); an input with no row for them, inside the thing's lifespan, is a hole with its question (`HOLE-UNGOVERNED`, §4.6); a rule written as a hole prints its question. An aggregate over the kind that meets such a key is refused or a hole with it (§5.11), never a total that skips the key. So the new thing is either computed by a rule that cites its law, or it is reported with what is missing.

**A closed-world row is restated when the world grows.** A row that leaves keys unstated, or states `*`, covers only the keys its index or kind had on the row's `known` date (§4.6). A row for every key stated in 2026 ("no one else is a parent", "everyone lives in Poland") therefore doesn't silently cover a child added in 2029. The child's version restates it with `replace`, in one line, or the reads are holes that name what is missing.

### 6.3 How amendments compose

1. **Order doesn't matter.** The read rule orders rows by their own dates, not by the order they were added. Two adds applied in either order give the same reality, once `on` has reached both their `known` dates (an add known after `on` waits for its advance, §6.2).
2. **Conflicts are found at load.** Two amendments that each load alone can still be refused together, by a load check that reads items of both. Which row wins is one rule, at load as at read (P-5), so load refuses only what that rule leaves unordered. The common kinds:
   - two additions with one id (`LOAD-DUP`);
   - two rows for one target and day that the read rule doesn't order (§4.6 steps 2 to 4: `LOAD-OVERLAP`, `LOAD-REPLACE`);
   - two rules with one label, neither a `replace` (`LOAD-LABEL`);
   - two versions known the same day that both move one version (`LOAD-MOVES`).
3. **Text merges cleanly.** The canonical sort puts each added version at its own line: versions sort by `known` date, then id. Rules sort by label, then by their version's `known`. Two amendments that touch different items therefore merge without a textual conflict. Two edits (`restate`, `revise`) of one line meet at the store's first step instead, which refuses them with a store code outside §8 (`OP-BASE`, `OPERATIONS.md` §1, §10).
4. **Outcomes are compared by `plan`.** `plan` loads the reality with and without an amendment and prints which ask results, which movements and which adjustments change (`OPERATIONS.md` §7).

### 6.4 Scenarios

```
scenario ben_stops over household
  tally 2
  from 2031-01-01
  note "Co jeśli Ben przestanie pracować od stycznia 2031 roku?"

version ben_stops_pay
  cite household_plan
  governs 2031-01-01..
  known 2026-10-20
  replace pay[ben_bolt] = 0 PLN "założenie scenariusza"

ask net_2040: diff(net_worth[2040-12-31]) in ben_stops against household law 2040-12-31
```

A scenario is a branch: the base's facts plus a set of ordinary versions, evaluated by the same rules and read only by asks that name it (N-1301, D-10). It changes nothing before its divergence day and nothing in its base (P-12).
- **Its versions are ordinary versions.** They state `cite`, `governs` and `known` (`LOAD-GOVERNS`, `LOAD-KNOWN`), and every rule of §4.5 and §4.6 applies to them. `known` is the writer's choice and means what it means anywhere: a `known` before the changed days is "as if known then", so plans and decisions made before the change see it and no adjustment arises; a later `known` is a change learned later, with its adjustments (§5.14.3) (SF-31).
- **A replacement says `replace`, and the branch stands above its base.** A scenario row that overlaps a base row at the same keys says `replace` (`LOAD-OVERLAP`), and replaces it on its span whatever the two rows' `known` dates, since a stipulation isn't news: a base row known on the scenario's own `known` day is replaced too (§4.6 step 2). A row at other keys, narrower or wider, needs none: it adds a candidate, and the higher layer wins over the base's rows (§4.6 step 3.1).
- **`from D`** is the day the branch diverges, and nothing before it changes:
  - No scenario row or `replace rule` of an item the base declares governs a day before D (`LOAD-SCENARIO`). So a scenario row over a base fact that governs `always` governs only from D, and in that world the fact changes with the law date: its readers there state their law date (§4.3, P-6).
  - A row of an input the scenario declares may govern `always` (it then needs no law date, §4.3), since only the scenario reads it.
  - A lifespan is a fact (P-2): an entry the scenario states into a kind the base declares (the prelude's included), or a `replace` of a base thing's entry, may change who is alive only from D. The new lifespan replaces the old one whole and agrees with it on every day before D (none, for a new thing), so a scenario ends a thing's life on D − 1 day or later (`replace jobs + [beta_job] lives 2027-08-01..2028-02-29` under `from 2028-03-01`) and starts one on D or later (`LOAD-SCENARIO` otherwise). Such a version needs no `governs`, since a lifespan carries its own time (§5.3).
  - A move the scenario declares books nothing before D: an instance whose day, a window's first day, or a leg's own day falls before D is left out (`LOAD-SCENARIO` where load sees the day, §8.3).
  - Movements before D can still differ from the base, but only through what reads the scenario's facts: a decision whose projection reads a scenario row known by its day, an item read at a law date from D (`at end_of(y)`), `lifespan(k)` of a thing whose lifespan the scenario changes, a binding over a span that reaches D; the engine lists them (`ENGINE.md` §9).
- **The base is whole without its scenarios.** The base loads and answers without any scenario file: an item of the base that names an item of a scenario file, a rule's `v` naming a scenario's version included, is refused (`LOAD-SCENARIO`). So an act that isn't law yet is a scenario's rule, brought in the scenario's version, `replace rule due.proposed beats base = … when …`, a new label adding the rule (§5.11). This changes a meaning of tally 1 (N-931, L-24), and `MIGRATION.md` lists it. Asks are the exception, since nothing in the model reads them: an ask that names a scenario (its `in`, a `diff`, `with scenario s`, `as if {scenario s}`) loads only with that scenario's file (`LOAD-SCENARIO` without it), and every ask is checked only in the worlds its line is evaluated in (§5.18).
- **Its items.** Its file may hold versions, entries of things (a hypothetical person), and new sections: kinds, places, inputs, defs, moves, indexes, asks. A read of an item declared in a scenario file is allowed only inside that scenario and the scenarios over it (`LOAD-SCENARIO`), and in `diff`. Two scenarios over one base never meet, so each may declare an id the other declares. A read is by name: a base item evaluated in the scenario reaches the scenario's things through its bindings over their kind (`x in places on d` holds a place the scenario's file declares, as it holds a thing a scenario version enters), which names none of them.
- **A scenario row may be an expression over the base's facts**, the base's items named with the base's id as a prefix and the scenario's own items bare (`replace sale_price[plot] = household.valuation[plot] * keep_share`, with `sale_price` a `num PLN` input of the base and `keep_share` a scenario input), so a linked figure isn't restated (`Q:D-11`). The expression is evaluated at the row's keys and law date: its prefixed names in the base, its bare names in the scenario. It reads inputs, bands and defs, never a place or a movement (a balance, `flows`, a move's instance), directly or through a def (`LOAD-SCENARIO`). `replace x[k] = none` on an observed row removes that movement with its legs (SF-14).
- **`over s0`** names the base: the model, or another scenario, which stacks this one on it. Stacks are at most 4 deep and acyclic (`LOAD-SCENARIO`).
- **Asking.** An ask names its world, `in s` or `in household` (the model's id); an ask in a scenario file that names neither is refused (`LOAD-WORLD`, §5.18). Inside an expression, `x as if {scenario s}` is x in s. `diff(e) in s against b` prints each key's outcome (§5.18). Random addresses don't name the scenario, so the paths of a scenario and its base are paired (common random numbers, N-1302).
- **Staleness.** The base moves on weekly, and a scenario is always read over the current base, so it rebases by itself (N-1303). The store flags a scenario `WARN-STALE` (a run has no file times, `ENGINE.md` §1) when a base version known after the scenario file was last changed writes a target, at a key and span, that the scenario replaces: the scenario now overrides something newer than itself.
- **Promotion** copies a scenario's versions into the reality with a `known` date of the day of promotion, since that is when the reality learned them (`OPERATIONS.md` §5); a version whose scenario `known` was earlier gives adjustments in the reality that it didn't give in the scenario, and `plan` lists them.

**Cost:** a scenario's versions are one more candidate per read of a target they write. Its run shares every memo outside the cone of what it writes (N-2004). **Cases:** the household's change 7. No case model uses a scenario: their options are readings of the law, so they are axes (§5.5), and their what-ifs are `as if` pins.

### 6.5 The seven common changes

The base is an excerpt of a household reality:

```
kind persons
kind jobs
kind loans
kind wages: event certain
  leg cash: PLN

place konto[p in persons]: accounts PLN lives 2019-05-01..
place world: outside lives 1900-01-01..

input loan_terms[l in loans]: (principal money PLN, months int, plan key instalments)
input overpay[l in loans, m in months]: (amount money PLN, choice key overpay_choices)
input parent[c in persons, k in persons]: bool
input pay_dom[j in jobs]: int
input pit_free: money PLN
input pit_rate[b in pit_brackets]: num
input residence[p in persons, m in months]: key countries
input salary[j in jobs, m in months]: money PLN question "Ile wynosi pensja brutto z tej pracy w tym miesiącu?"
input works_for[j in jobs]: key persons

def pit[p in persons, y in tax_years]: money PLN at start_of(y)
  rule base = round(pit_scale(taxable[p, y]), 1 PLN, half-up)
  rule abroad beats base = ?"Podatek nierezydenta Polski nie jest opisany." when residence[p, month_of(start_of(y))] != pl

move wage[m in months, j in jobs over m]: wages
  on day_of(m, pay_dom[j])
  at start_of(m)
  leg cash from world to konto[works_for[j]] amount net_wage[j, m]

version home_2026
  cite household
  governs 2026-10..
  known 2026-10-19
  persons + [ana] lives 1990-04-02..
  persons + [ben] lives 1988-11-30..
  jobs + [ana_acme] lives 2020-02-01..
  jobs + [ben_bolt] lives 2021-09-01..
  parent = false "brak dzieci"
  pay_dom = 10 "umowy o pracę, § 5"
  residence = pl "meldunek w Krakowie"
  works_for[ana_acme] = ana
  works_for[ben_bolt] = ben
  opening[konto[ana]] = 12 400.00 PLN at 2026-10-01 start "wyciąg"
  opening[konto[ben]] = 3 150.00 PLN at 2026-10-01 start "wyciąg"
```

Things enter their kinds with their lifespans, and defs, fills and moves range over the kinds, so a new thing is picked up wherever it belongs, from the day it lives. Each change below is one added block. **No line of the base is edited in any of them.** Each one known after the base's `on` follows an `advance` of `on` to its `known` date (§6.2): a change of its own, not counted in the table below.

**1. A new job.** Ben moves to a new employer from March 2028.
```
source cogs_contract "Umowa o pracę Bena z Cogs sp. z o.o., 2028-02-10"

version ben_cogs_job
  cite cogs_contract
  governs 2028-03..
  known 2028-02-10
  jobs + [ben_cogs] lives 2028-03-01..
  replace jobs + [ben_bolt] lives 2021-09-01..2028-02-29
  salary[ben_cogs] = 8 200.00 PLN "§ 3"
  pay_dom[ben_cogs] = 10 "§ 5"
  works_for[ben_cogs] = ben
```
The Bolt job ends with February, so no wage instance of it is bound from March, and no zero row is needed: a job outside its lifespan has no salary to read (§5.7). The wage move, the employer's tax advances and anything summed over Ben's jobs pick up `ben_cogs` from March. Its `pay_dom` is stated here: the base's row covers only the jobs known when it was stated (§6.2).

**2. A law change for the coming year.**
```
source pit_2028 "Ustawa z 2027-11-20 o zmianie ustawy o PIT, Dz.U. 2027 poz. ..."

version pit_2028_figures
  cite pit_2028
  governs 2028..
  known 2027-11-20
  replace pit_free = 36 000.00 PLN "art. 1 pkt 3"
  replace pit_rate[low] = 11 % "art. 1 pkt 4"
```
Only the changed figures appear, each saying that it replaces the earlier row from 2028. Every other figure stays in force from its own version (§4.6). Nothing booked before 2027-11-20 is for 2028, so no adjustment arises.

**3. A retroactive law change.** In June 2028 the new figure is made to apply to 2027 as well, with refunds in the 2028 return. The base's moves `pit_advance` (monthly, of kind `pit_advances`) and `pit_settle` (the annual return, of kind `pit_settlements`, paid from the taxpayer's account to `fisc`) aren't shown above.
```
source pit_2027_retro "Ustawa z 2028-06-15, art. 2: stosuje się od 2027-01-01"

kind waivers: event certain
input refund_day: date
input act_day: date

version pit_2027_refund
  cite pit_2027_retro
  governs 2027
  known 2028-06-15
  replace pit_free = 36 000.00 PLN "art. 2 ust. 1"
  refund_day = 2029-04-30 governs always "art. 2 ust. 2"
  act_day = 2028-06-15 governs always "tytuł ustawy"

move keep_advances[a in adjustments on act_day where a.adjusts in pit_advances]: waivers
  on a.day
  fulfils a

move refund_2027[a in adjustments on act_day where a.adjusts in pit_settlements]: transfers
  on refund_day
  leg cash from fisc to a.adjusts.cash.from amount -a.change
  fulfils a
```
- On 2028-06-15 the engine records an adjustment for each booked 2027 instance whose amount the new figure changes (§5.14.3): the twelve advances and the settlement booked on 2028-04-30.
- The advances are not recomputed: `keep_advances` waives their adjustments. The settlement's adjustment is refunded on 2029-04-30. Booked amounts never change, so the change in the year's tax is counted once, through the settlement, which reads the advances as booked.
- `ask … as known 2028-06-14` still shows the old view.
- Without the two moves, every `flows` that could count the adjustments, from 2028-06-15 on, is a hole asking whether back pay is due (`HOLE-CHANGED`).

**4. A child is born.**
```
source birth_ola "Akt urodzenia Oli, USC Kraków"

version ola_born
  cite birth_ola
  governs 2029-05..
  known 2029-05-14
  persons + [ola] lives 2029-05-12..
  replace parent = false "brak innych relacji rodzicielskich"
  parent[ola, ana] = true
  parent[ola, ben] = true
  residence[ola] = pl "rezydencja od urodzenia"
  opening[konto[ola]] = 0 PLN at 2029-05-12 start "brak konta"
```
- `konto` is keyed by person, so Ola has one, and its opening reading gives it a balance (§5.10).
- The base's `parent = false` covers only the persons of 2026, so it is restated for the new world (§6.2). Without that line, `parent[ana, ola]` would be a hole.
- Benefits and reliefs over `persons` that read `parent` and Ola's lifespan pick her up from the day she lives. One that needs a fact not stated yet, such as the date a benefit was claimed, is a hole with its question.

**5. A move abroad, with a job there.** Ana moves to Berlin in July 2030 and is paid in euro.
```
kind wages_eur: event certain
  leg cash: EUR
place world_eur: outside lives 1900-01-01..
input salary_eur[j in jobs, m in months]: money EUR question "Ile wynosi pensja w euro z tej pracy w tym miesiącu?"
move wage_eur[m in months, j in jobs over m]: wages_eur
  on day_of(m, pay_dom[j])
  at start_of(m)
  when salary_eur[j, m] > 0 EUR
  leg cash from world_eur to konto_eur amount net_wage_eur[j, m]

version ana_berlin_job
  cite household
  governs 2030-07..
  known 2030-05-01
  accounts + [konto_eur] EUR lives 2030-05-01..
  jobs + [ana_berlin] lives 2030-07-01..
  replace jobs + [ana_acme] lives 2020-02-01..2030-06-30
  opening[konto_eur] = 0 EUR at 2030-05-01 start "konto w euro, otwarte przed przeprowadzką"
  pay_dom[ana_berlin] = 25 "Arbeitsvertrag, § 5"
  residence[ana] = de "wymeldowanie; umowa najmu w Berlinie"
  salary[ana_berlin] = 0 PLN "pensja w euro, nie w złotych"
  salary_eur = 0 EUR governs 2026-10.. "brak pensji w euro"
  salary_eur[ana_berlin] = 5 200.00 EUR "Arbeitsvertrag, § 4"
  works_for[ana_berlin] = ana
```
- A job in another currency needs its own event kind, input and move, since money in two currencies is never added (N-202); its own outside place, `world_eur`, only keeps the euro flows apart, since `world` holds any unit (§5.10). These are added sections, and nothing is edited. The account enters the prelude kind `accounts` in the version, with its unit and its opening reading. `salary[ana_berlin] = 0 PLN` gives the Polish wage move its figure at the new key (an instance of 0, which books nothing); `salary_eur = 0 EUR`, a row governing from the base's first month since the move ranges over every month, and the `when` keep the other jobs free of euro instances.
- The move names `konto_eur`, a particular account: a movement between two named places, not a rule over a kind.
- `net_wage_eur` is a def of the German law part, which isn't written yet. A name no file declares is refused at load (`LOAD-NAME`), so the change adds the law part as a stub, and every result that needs it is a hole with its question:
```
part de_employment
  tally 2
  law

def net_wage_eur[j in jobs, m in months]: money EUR = ?"Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane." at start_of(m)
```
- The Polish `pit` for Ana from 2031 shows "Podatek nierezydenta Polski nie jest opisany": the rule `abroad` reads her residence in the January of each year, and she moves in July 2030. When the German law is written, its rules replace the stub in the part (a `revise` of the part, or rules that each beat `base`) and pass the gate (`OPERATIONS.md` §3).

**6. A new loan, and an overpayment on the first.**
```
source car_loan "Umowa kredytu samochodowego nr ..., 2028-04-02"
source stmt_2028_05 "Wyciąg z rachunku kredytu hipotecznego, maj 2028"

version car_loan_terms
  cite car_loan
  governs 2028-04..
  known 2028-04-02
  loans + [car] lives 2028-04-02..2032-04-02
  loan_terms[car] = (principal: 40 000.00 PLN, months: 48, plan: equal) "§ 2"
  overpay[car] = (amount: 0 PLN, choice: none) "brak nadpłat, o ile wyciąg ich nie pokazuje"

version house_overpay_2028_05
  cite stmt_2028_05
  governs 2028-05
  known 2028-05-20
  overpay[house, 2028-05] = (amount: 10 000.00 PLN, choice: shorter) "nadpłata 10 000 zł, skrócenie okresu"
```
The schedule defs are recurrences over the months of each loan. They read `overpay` and its choice, so the instalment stays and the term shortens. The car loan states its own "no overpayment" row, because the base's row was stated before `car` entered (§6.2); the house's row for May, at more keys than the base's, wins by precedence on its span (§4.6 step 3), so it needs no `replace`.

**7. What if one partner stops working.** This is the scenario file of §6.4: a header and one version in a new file. Every ask can be run in it (`in ben_stops`), compared with the base by `diff`, or read inside an expression with `as if {scenario ben_stops}`.

| Change | Lines added | Lines edited | Places touched |
|---|---|---|---|
| 1. New job | 9 | 0 | one source, one version |
| 2. Law for next year | 7 | 0 | one source, one version |
| 3. Retroactive law | 16 | 0 | one source, one kind, one input, one version, two moves |
| 4. Child born | 11 | 0 | one source, one version |
| 5. Move abroad, job there | 27 | 0 | four sections, one version, one stub law part |
| 6. Loan and overpayment | 14 | 0 | two sources, two versions |
| 7. Partner stops working | 9 | 0 | one new scenario file |

Change 3 is longer than in tally 1, whose `arrears` lines stated back pay in two words each: the refund and the waiver are now ordinary movements a reader sees and a binding reads (D-6, C-17).

---

## 7. Numbers, rounding, randomness and determinism

### 7.1 Exact numbers and the literal rule

<!-- tally2: D-14, D-15; C-3, SF-24 -->

**Every number is exact.**
- Money is a whole number of minor units.
- `num` and `int` values are exact rationals.
- `+`, `-`, `*`, `/` and comparisons are exact.
- `pow(x, n)` with an integer n is exact; where x has a unit, n is a literal, so the result's unit is known at load (`LOAD-UNIT` otherwise).
- Division by zero is refused at that key (`RUN-DIV`).
- A value whose numerator or denominator passes 4 096 decimal digits is refused at that key (`RUN-DIGITS`). The digits are counted on the reduced fraction. The cap is there for cost, not precision: unbounded growth would make one key cost more than the run (`PERFORMANCE.md` §6). A long product of market data, such as 720 monthly ratios, can reach it. A rounding no source states, forced by the cap, is the modeller's reading: it is written in the def's note, and `explain` shows the `round`.
- **What the cap covers** (tally 2.1, R-13). The cap applies to a value at a key. A path statistic's own accumulation (the sum under a mean, a variance) is the engine's, unbounded or exact by any method, and must give the printed rounding exactly (§5.16). No value a rule can read changes.
- There is no floating point anywhere in a result (N-219).

**The literal rule** (P-16; LC-1's option (b), D-14). A figure of the world carries its source, so it is a row. In defs, rules, moves and fills, the only literals allowed, beside `true` and `false`, are:
- **0, 1 and −1**, with or without a unit: `0 PLN`, `1 day`, `-1 day`, `1 year`, `1 %` (whose value is 0.01, so `x / 1 %` is x in per cent), `100 %` (whose value is 1). A bare `0`, `1` or `-1` takes the unit of the other operand in `+`, `-`, `min`, `max` and comparisons, and the unit of `round`'s operand where it is the step (`round(x, 1, down)` rounds to whole units of x's unit), is dimensionless in `*` and `/`, and takes the declared type of its target when it is a whole value (a rule's value, an `if` branch): `rule none_due = 0` in a `money PLN` def is 0 zł (SF-24);
- **integer offsets in key arithmetic**: `m - 1`, `y + 1`, `m - 12` on a key of a period index, an integer index or a range, and the same integer in the guard that protects the read (`pos(k) > 12`, `pos(k) == 12`, §5.11);
- **precision steps** in the unit position of `to` (`to 1e-9`, `to 0.0001 %/year`), which set how finely the engine computes, not a figure of the world; the step of a `round` is `1` with a unit, `minor(CUR)` (a currency's declared minor unit, C-3), or a row;
- **key literals** that aren't dates or periods, in key positions: a subscript, a binding list, or a comparison with a key (`limit[u5]`, `k == pl`, `k <= 10` on an integer index);
- **the bracket of a `solve`** in a rule with `v` (§5.18): where the method the law names searches, not a figure of the law;
- **the integer of a code** (`nominal 12`, §2.2): part of the code's name, not a figure.

Every other figure, **a day of the month and a date included**, is a row in a version, with a source (N-215): moving the investing day from the 15th to the 20th is one added row. 100, 360, 365, 366 and 1000 are no longer allowed: a day count is a `daycount` code (§3.3), and a figure the law states is a row.
- **A text literal** (`"…"`, §3.1) is a literal like any other that states a fact: it stands in a row, a pin or an ask line, and in a rule it is refused (`LOAD-LITERAL`).
- **Band rows** are rows of a version, with their source and locator, so they may hold any literal: "10 800 zł plus 32 % of the excess over 120 000 zł" is one row, written once (§5.9). A bare literal in a band row takes its unit as a bare `0` does in a def: the other operand's in `+`, `-`, `min`, `max` and comparisons, dimensionless in `*` and `/`, and the band's declared unit where it is the row's whole value (`kz: (13000, 85528] = 525.12` in a band of `PLN` is 525.12 zł; in `0.12 * x - 3600`, 3600 is in x's unit).
- **Date literals** (days, months and years, used as dates or keys, and quarters, which are never keys, §4.1) appear only in rows, in the clauses `on`, `start` and `end` of the header, `from`, `governs`, `known`, `enters`, `lives`, `moves … to`, `covers`, a `same` line's keys, a row's instant (`at`), an input's `prices Y` (§3.4), and `as known`, in index declarations, in pins, and in asks (below). A rule can't test `m >= 2028-01` on a time key, nor read `rate[2025]`: a law that starts on a date is a version with `governs` (`LOAD-LITERAL`).
- **Month-day literals** (`04-30`, §3.1) are date literals here: they appear only in rows (a calendar's `yearly` list included), an anchored index's `from`, pins and asks, and a rule can't hold one (`LOAD-LITERAL`).

**An ask** allows every literal a def allows, plus its own: `paths`, `seed`, `path`, its pins, a `solve` bracket and step, a stepped range's ends and step, the level of a `quantile`, the step of a `round` (a bare step takes the unit of `round`'s operand, §7.2), date and period literals and spans (`flows(konto, 2026-03)`), and text literals (`check name[p] == "…"`), because those are the question. Other figures in an ask are rows. Pins in a def's `as if` fall under the def's rule.

**Refusals:** `LOAD-LITERAL`. **Cost:** none (checked at load). **Cases:** all; the migration lists every figure it lifts into a row (`MIGRATION.md`).

### 7.2 Rounding

<!-- tally2: D-14 -->

`round(x, u, mode)` is the multiple of u that the mode chooses.
- u is a **step**: `1` with a unit (`1 PLN`, whole złoty), `minor(CUR)`, or an input or axis-keyed input whose every value is positive (§7.1). Steps that aren't powers of ten exist in law, such as 5 Rappen (N-208), and are rows. An ask may write any positive step.
- To give money, u must be a whole number of minor units (`LOAD-UNIT`).
- **Its type.** `round` of a `num U` gives a `num U`. It gives money when U is a currency and the step is a whole number of minor units, wherever it stands, an operand included; money is a `num CUR` too (§3.2), so a `num CUR` def still accepts it. `round(x, 1, mode)` of a dimensionless `num` is an `int`, so `premium * round(months / 12, 1, up)` is money × int, money. A step of type `money CUR` is a whole number of minor units by its type.

| Mode | Chooses |
|---|---|
| `down` | toward zero (truncation) |
| `up` | away from zero |
| `floor` | toward −∞ |
| `ceil` | toward +∞ |
| `half-up` | the nearest; a tie goes away from zero |
| `half-down` | the nearest; a tie goes toward zero |
| `half-even` | the nearest; a tie goes to the even multiple, k·u with k even |

- **The mode may be an axis** whose options are mode names. That is how a rounding the source doesn't state is written as readings, side by side (N-209). Mode names are contextual words (§2.2), so they may be the axis's options and keys elsewhere. A reading "no rounding" is a rule that tests the axis and has no `round`, not a mode: an axis whose options mix modes with `exact` is tested by rules (§5.5).
- **Nothing is rounded unless the model says `round`.** "No rounding stated", "no intermediate rounding" and "to the third decimal, mode unstated" are three different models: no `round`; one `round` at the end; and a `round` at that step with a mode axis (N-209).
- **Converting to money is a `round`.** A `num CUR` becomes money only through `round(x, u, mode)`, or `prorata` with the same step (§3.2, §5.13).

### 7.3 Transcendental values

These need `to u`, with u a step:
- `exp(x)`, `ln(x)`, `sqrt(x)`;
- `pow(x, q)` with a non-integer q;
- `cdf`, `pdf` and `quantile` of `normal`, `lognormal`, `exponential` and `gamma`, `pmf` and `cdf` of `poisson`, and the mean of `lognormal` (the rows of §5.16 that need `to`), and the sample of a continuous distribution;
- `crps` of a normal forecast;
- `periodic(r, P, c)`, `factor(r, n, P, c)` and `accrue(p, r, a, b, dc, c)` when the power they take isn't an integer (§3.3).

**The value is the exact result, correctly rounded to u, ties half-even.**
- The engine decides it by interval arithmetic, refining until the interval lies within one rounding cell.
- Where the exact value is a rational on a cell edge, the engine detects it exactly: perfect powers, `exp(0)`, `ln(1)`, Φ(0) = 1/2, the median of a symmetric distribution.
- A rounding not decided within 2^16 bits of precision is refused at that key (`RUN-UNDECIDED`, `<function> to <u> not decided within the precision cap`; the writer states another `to`). It needs a true value within 2^−65536 of a cell edge, and it is the guarantee that two machines never disagree.
- `ln(x)` with x ≤ 0, `sqrt(x)` with x < 0, and `pow(x, q)` (or `factor`) with x < 0 and a non-integer q are refused (`RUN-DOMAIN`); `pow(0, q)` with q < 0 is a division by zero (`RUN-DIV`).

**Composites are exact.** An expression over such values is exact arithmetic on the rounded results. So the model's number is defined to the last digit, and an evaluator gets it by hand from the rounded values.

**The error bound.** A printed result that rests on rounded transcendentals, with no `round` after them, prints `≈` and a bound on its distance from the true real-number value. The bound comes from interval arithmetic with each rounded value r widened to [r − u/2, r + u/2] (a sample is one, §5.16 step 4; a value detected exactly on a cell edge isn't), run over the expression as written, each operation in its own order: a transcendental whose argument is itself an interval takes the hull of its exact values over that interval, then its own ± u/2. The bound prints to two significant digits, rounded up. A first-order estimate is not the bound. Examples are a ruin probability from a closed form, an annuity factor, and a CRPS. Money that has passed a `round` prints plain. It is the model's defined value, and `explain` shows the `to u` steps under it.

### 7.4 Randomness and seeds

<!-- tally2: D-11 -->

The only randomness is the sample of a random fact (§5.16).
- A sample is a pure function of its address, the ask's `seed` and the path number. There is no stream, and no state between samples.
- A path is a pure function of its seed and number, so any path can be recomputed alone (`path p`, N-1406).
- **Asks without `paths`** compute their queries in closed form, where §5.16's table gives one, and are refused otherwise (`LOAD-RANDOM`); no hidden N or seed is ever used.
- **Working by hand** uses sample pins: `with sample z[k] = v` fixes a value, `with sample z[k].u = 0.37` fixes a uniform. With fixed uniforms u1, u2, … given in a list, they go to the addresses in the canonical order of §5.16.
- **A test mode for statistics against closed forms** (CHG-13) is an ask that pins a parameter's sample for every path: `with sample theta = theta_mid`. It is printed in the ask's register, so a forecast with a pinned parameter can't be mistaken for one without.

### 7.5 Determinism

<!-- tally2: D-5, D-11, D-15 -->

The same reality, the same ask and the same seed give the same output to the last digit, on any machine (R-16, N-1401). These hold the guarantee:
- Arithmetic is exact, and transcendentals are correctly rounded.
- Samples are SHA-256 addressed.
- **Every order is defined:**
  - index order for the keys of a list or a range index; things of a kind and an open index's keys have no order a rule can read, so whatever orders them for a rule states it (§5.7, §5.6), and an order no rule reads takes the text order of their stable ids (§2.5);
  - binding order for aggregates (§2.5), and `ties by` wherever an order changes a result (`RUN-TIE` otherwise);
  - for `fill`, its stated `order` tuple, then its `ties by` term; index order never breaks a tie (§5.12);
  - within a day, the stratification of the declared reads, with `before`/`after` for cycles and `ties by` between instances of one move (§5.14.2);
  - the canonical order of samples (§5.16);
  - the explanation order (§5.18);
  - sorted questions in holes;
  - the canonical format for printing.
- **The engine reads nothing outside the reality and the ask:** no clock, locale, environment or file order. "Today" is the ask's cut, or the model's `on`.
- **Parallel work combines exactly.** Path statistics are exact sums of rationals, combined in path order, so the number of workers changes nothing.

---

## 8. Holes, the register, refusals and warnings: the catalogue

<!-- tally2: D-1..D-15; C-20 -->

A result is one of three things (D-15 removes tally 1's fourth, the bound):
- a **value**, which may be `none` or `never` (§3.1);
- a **hole**, which prints its questions;
- a **refusal**, which prints its code and message.

Warnings print beside a result and leave it as it is (§8.5).

A model with holes or per-value refusals still runs, and its other results stand. A load refusal stops the model before anything runs; one in a scenario file refuses only that scenario and the scenarios over it, and an ask's `LOAD-SCENARIO` for a scenario whose file isn't loaded refuses only that ask (§6.4) (`ENGINE.md` §11). Each code has one row here; the sections that raise it name it.

### 8.1 Holes

A hole is a fact that is missing: one a row could give (P-9). It is never 0, never a default and never the last known value (N-1001, spec §4#8). A step no row can give (a density from a sample, a foreign program's random stream, a hard optimisation) is written by the model as a hole whose question says why (P-17, C-20).
- **It propagates** through every value that depends on it: arithmetic, comparisons, rules, movements (as a hole movement) and `flows`.
- **It stops propagating** only where the outcome is decided without it: `and`, `or`, `any` and `all` under Kleene logic; an `if` whose branches are equal; a `sum` term known to be 0 under a hole `where`; a `fill` take that is known to be 0; a rule that is beaten, whose `when` and value aren't evaluated; `missing(r)` and `covered(x, s)`, which are never holes (§5.11, §5.19); and a place's balance after a reading, which a hole movement dated on or before it no longer reaches (§5.19); and a place's total, which a hole part leaves known (below).
- **It carries** the questions of every operand, `where` and term it evaluated: the union, each once. A read whose key is a hole reads no cell, so it carries the key's questions only, as a band read at a hole x does (§5.9).
- **It prints** every question it carries, sorted by code point and without duplicates, with the item and keys each came from: a question reached at many keys prints once, with its list of keys. A `partial` ask line prints the known part beside them (§5.18).

| Code | Arises when | Question printed | The writer |
|---|---|---|---|
| `HOLE-STATED` | a row, field, rule value, leg end or lifespan end is `?"…"` (§5.3, §5.11, §5.14) | its text | answers it with a row |
| `HOLE-UNGOVERNED` | the gap rule: no row governs an input or a band at those keys on that law date, inside the thing's lifespan (§4.6, §5.7); no row of the band's table holds its argument (§5.9) | the input's `question`, else `x at k: no value for <period>` (`no value at <law date>` with no time key, `no value` with no law date); `<b>: no row holds <x> at <law date>` (`<b>: no row holds <x>` with no law date) | adds a row, a `none` row, or a `?` row with a better question |
| `HOLE-NOREADING` | a plain read of a reading, or `gap(x, d)`, at an instant with no reading; `last` or `since` with no reading on or before d (§4.6, §5.19) | `<x>[k] has no reading at <instant>`; `<x>[k] has no reading on or before <d>` | adds the reading, or writes the carry with `last` |
| `HOLE-NOTYET` | `first`, `last` or `next` over a binding of things or movements that is empty at the cut and whose span reaches past it: a later fact may bring a member (§3.1, §5.11, §5.14.1) | `<x>: no <event> yet, and its span reaches past the cut` | adds the row that brings a member, or tests `count(…) > 0` first |
| `HOLE-ANCHOR` | a balance of a place alive before S, read before its first `opening` or `stated` reading (§5.10, §5.19) | `<place> has no balance before <d>`, d its first reading's instant; with no reading at all, `<place> has no opening or stated reading` | adds an `opening` reading |
| `HOLE-LOTS` | lots held before the run (observed acquisitions dated before S, which a fill binds and nothing books) whose sum disagrees with the opening reading that anchors it (§5.12) | `<place>: the lots held before <S> sum to <a>, the opening reading states <b>` | corrects the lots or the reading |
| `HOLE-PRICE` | a position with no price on d, in `value` (§5.10); a `prices Y` figure read outside Y with no index (§3.4) | `<unit> has no price in <CUR> on <d>`; `<x> is in Y prices; no index moves it to <d>` | adds the price, or the index |
| `HOLE-CALENDAR` | a business-day function reaches a day outside its calendar's coverage, or an `added` day that is a hole, and it decides the result (§4.2) | `calendar <c> doesn't cover <year>`; the added day's question | adds the calendar's row for that year (an `assumed` row for a year not yet published, §4.2), or answers the day's question |
| `HOLE-IMPRECISE` | a comparison on a window isn't decided; one value is needed from an undecided interval (a print, money, a key, a term of an aggregate other than `min`, `max`, `argmin` and `argmax`); an `at` window whose law-date classes give different values (§4.3); a windowed movement crosses an end of a `flows` span, or holds the day of a `bal(x, d)`; a version known inside a windowed instance's window changes its amount and the line states no `windows each` (§5.14.3); a read depends on where in a window the day falls and the line states no `windows each`: a projection's or a decision's window, a version's `known` window, a window in a calendar's `added` list (§4.2, §4.4, §4.5, §5.14.1) | `date within <window> not precise enough for <comparison or use>` | states the day (a row, or a `with` pin), or writes `windows each` |
| `HOLE-CHANGED` | a version changed a booked plan instance, and no movement fulfils or waives the adjustment (§5.14.3) | `<move> on <t> changed after it was paid (version V); is back pay due, and when?` | writes a move that fulfils the adjustment, or one with no legs that waives it |
| `HOLE-SAME` | two things, rows or movements stated `same` disagree on a fact for a day: a row's value, a thing's entry into one kind, a movement's amount or day at a place both touch, or its kind, where a binding over either kind meets the movement; for observed rows a version states `same`, whose ends each statement books, a read of the movement away from its places (§5.19); a joint total over a member's place that a movement of the joint model's own versions enters with no `same` or `fulfils` link to the member's own movement (§6.1) | `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`; `<movement> crosses to <member> with no link` | states the value in a row of the joint model, or writes the `same` or `fulfils` link |
| `HOLE-PRERUN` | `flows` reaches days before the run's start S, or a projection starts before it (§4.1, §5.14.1, `ENGINE.md` §2) | `before the run's start` | reads from S on, or starts the model earlier |
| `HOLE-PART` | a read of parts, `bal(…) of q`, `value(…) of q` or `m.leg.amount of q`, where a hole part may be q's (§3.6, §5.10, §5.14): an arrival's hole shares, an anchor's gap or a reading's unstated rest, an undecided take, a cycle of observed rows in one batch (§5.14.2, §5.19) | `<place>: <amount> may be <q>'s: <questions>` | answers the questions (a row, an attribute), or states the parts at a reading |

**A hole part** (tally 2.1, R-8). At a place that keeps parts, an unknown fact makes a part whose label is a hole; missing law never does, and is refused (`RUN-NO-RULE`). A hole part keeps its amount, the questions of what made it (an arrival's hole shares' own; for a gap or a reading's unstated rest `whose are <a> of <x> at <instant>?`; for a cycle of observed rows `which of <movements> went first on <d>?`; for a negative gap's unknown outflow `which outflow took <a> from <x> by <instant>?`; for a stated part whose batches are open `which batches are <q>'s <a> of <x> at <instant>?`; for a stated batch whose owners are open `which owners are <b>'s <a> of <x> at <instant>?`), and its **possible labels**: at an owned place the parties whose shares were holes, or any party for a gap; at a batched place the batches it may be (an arrival's hole part has the arrival's batch, so only a gap or a merge across batches leaves its batch open); at a place that is both, the pairs. A read `of q` and an order term over it (`k.owner`, `k.batch.day`) are each known where every possible label gives it one value, and a hole otherwise: so `of q` is a hole only for a q the part may be, and a part whose every possible label is q's is read as q's. Distinct hole parts stay distinct. The only merge is an undecided take's: the parts it could reach under some reading of the holes become one hole part, their amounts summed and their possible labels joined, and the parts it can't reach stay known (§5.14). The place's total stays known throughout. A hole part keeps its amount, so a bound on the unknown can be added later without changing how parts are kept (§10).

### 8.2 The register

Every result lists, under it, the things it rests on that aren't plain sourced facts; a hole lists them like a value. The list is transitive: whatever its evaluation read, through any chain (N-713), each `when` it tested, each operand it evaluated and an ask's own `where` included, and the reads that decided that a movement doesn't count (a false `when`, a day outside the span). Arithmetic evaluates both operands; what §5.11 leaves unevaluated is unread (`and`, `or` and `if` stopping early, `first` and `last` stopping at their key, a beaten rule's `when`, every rule's value but the winner's), so a row an `and` never read isn't listed. Each entry prints once, its keys or periods merged (P-17, Q-3).

| Entry | From |
|---|---|
| `assumed: <reason>` | a row value with `assumed` (§5.8), once, with the keys it filled; an index with `assumed`, printed `assumed: <index>: <reason>`, for every figure keyed by it and every result that reaches one (§5.6); a version with `assumed`, for every result that reads one of its rows or a rule that points at it (§5.3) |
| `given: <def>[keys] = v (<version>)` | a version row aimed at a def (§5.11), printed once as written, with its key range if it has one |
| `missing record counted by <version>` | a rule that won because a `missing` test read true (§5.11) |
| `pinned: <pin>` | a `with` or `as if` pin (§5.11); a pinned axis is listed here only, never under `reading:`. A pin given by an expression prints its value, `pinned: <pin> = v`, and with it the register of what the expression read. Pins formed per binding tuple print once, as written, with their binding (`pinned: gross = g for g in gross_lo .. gross_hi step minor(DUK)`), never one line per tuple. The pins one def forms under many `as if` (a `known d` at each of its keys, as `cpi_yy[m] as if {known d}`) print once, their days merged into maximal spans |
| `pinned sample: <address> = v` | a sample pin, or a pinned uniform (§5.16, §7.4) |
| `sample: <address> = v (u = …)` | on a `path p` line, each sample it reached, in canonical order (§5.16) |
| `world: <scenario or model>` | the ask's `in`, and any `as if {scenario s}` it reached (§6.4) |
| `reading: <axis> = <option>` | each axis option that the line's result depends on (§5.5); options that give the line's result together are named together (`down \| half-up`). An axis the line prints as `*` has no `reading:` entry |
| `anchor: <place> <instant>, stated <s>, difference <d>` | a balance read at or after a reading of a version that anchors the place (§5.19): the last such reading, its value and its difference, stated − booked, carried like every entry. A difference of 0 is listed too; a hole difference prints `difference ?` with its questions |
| `pad: <place> <instant>, <movement> of <d>` | a balance, a `flows` or a binding that counts a pad movement (§5.19) |
| `adjustment: <move>[keys] by <δ> on <V.known> (version V), remaining <r>` | an adjustment with a remainder at the cut, reached by a balance or `flows` of the places it moves between (§5.14.3) |
| `unfulfilled: <move>[keys]` | a planned instance that a sum counts, or that a balance before the cut counts, and that no observation fulfils (D-7's sums over planned movements, M-12) |
| `open: <move>[keys]` | a planned instance of a certain kind in a covered span that nothing fulfils, left unbooked (§5.19) |
| `left out: <move>, <n> instances, first <day>` | instances outside [S, E], not booked (§5.14.3, D-13); under a projection, the movements dated in its window that depend on the item holding it (§5.14.1); and, under a result of a joint model that reads a member's place on or after their day, the joint model's own movements into that place that no `same` or `fulfils` links to the member's own, not booked there (§6.1) |
| `uncertain: <move>[keys]` | an unfulfilled movement of an uncertain kind counted under `with uncertain` (§5.14) |
| `held: <target>[keys] = v (<version>), read for <periods>` | the mark of a value carried past what its rows state: a row whose span has a start and no end (its own `governs`, else its version's), band rows included, read at a law date after the cut K, a figure carried into the future unchanged, as known today (spec Q-13); and a row that `last(x[k], d)` reads past the end of its span (§4.6). K is the ask's cut, also for a read made at an earlier cut (an instance's `when` or amount at min(t, K), a decision's projection on day t, §5.14): the register says what this ask takes as known today, not what the instance knew. An input read at its own time key (§4.6) is read for that key's period, at the period's first day, so it is `held` when that day is after K. A row that governs `always` is timeless and is never `held`. A figure that is published per period is written with closed spans instead, so its future is a hole until a rule or an `assumed` row gives it. It prints once per row, the keys and the periods it was read for merged into maximal runs (`read for 2026-06-01 .. 2026-12-31`) |
| `law date <d>, cut <K>` | always, for the top-level value, with K the ask's cut even where the value rests on reads at earlier cuts; a `check` whose sides are read at different law dates lists each |

**Dropped labels** (tally 2.1, §3.6). The warning that labels left the books (§8.5) is listed in the register like an entry: with every result whose reads reach the place that received them, from the first such leg on, transitively as every entry is, so a net worth that counts that place by `owns` carries it.

### 8.3 Refused at load

What load can see that isn't defined is refused here, before anything runs (R-load); a refusal in a scenario file refuses only that scenario and the scenarios over it, and an ask's `LOAD-SCENARIO` for a scenario whose file isn't loaded refuses only that ask (§6.4) (`ENGINE.md` §11). Each refusal names its file, line and item.

**What load sees.** Load sees these, and nothing else (the steps are `ENGINE.md` §3.1's):
1. **The text.** Every file's tokens and lines, and its canonical reprint (step 1), and every `use` pin.
2. **Every declaration and name.** Ids and their sections, kinds and subkinds, indexes, units, axes and their options, sources, versions with their `cite`, `governs`, `known`, `moves` and `enters` lines (steps 2, 4, 9).
3. **Every literal.** Numbers, days, months, years, quarters, month-days, windows written as literals, code words and texts (step 3; §7.1).
4. **Types and units** of every expression (step 3; §3), except a unit kind's key that comes from a value ("A key that load can't see", §3.2) and a code read from an input, which is checked where it is read (§3.1).
5. **The keys of every index whose keys are written:** a list, a range with literal ends and step, an anchored index with its anchor (§4.1, §5.6). An open index's keys come from rows, so load doesn't see them.
6. **The things of every kind** entered in versions, with each thing's base kind, its dated memberships and its lifespan as written (step 4); a lifespan end written as a hole is seen as a hole.
7. **The rows' shape, not their values.** Which targets, keys, spans, instants and `known` dates the rows state, their `replace` lines, their overlaps and band tables (step 9). Which row a read takes depends on the ask's cut and law date (§4.6), so no row's value is a value load sees.
8. **The item graph** and its checks (steps 5–6): every read with its key relation; a binding over a kind as an edge to every item that gives its things or movements.
9. **The moves' classes, running reads and the decision graph** (steps 7–8). A move's day and a leg's place are seen where they are written as literals or keys of a declared index, and where they are computed from these alone, folded through built-ins that read no fact (`on end_of(m)` for a key m of a declared index is seen); a day or place computed from a row's value is not seen.
10. **The random cone** (step 10): every item that reaches a `dist` fact through the item graph, in each world, and through every place a random movement books into: a balance or a projection of such a place is in the cone, so a line that reads it without a query is refused at load, as any random line is (`LOAD-RANDOM`, §5.16).
11. **Each ask's worlds, axes, law date and ask-only functions** (step 11).

Load never sees: a value read from a row, or computed from one; which row a read takes at a cut and law date; whether a reading exists at an instant (§4.6); an open index's keys; a sample of a random fact.

| Code | Message | What the writer does |
|---|---|---|
| `LOAD-PARSE` | `cannot parse: <what was expected>`; also tabs, bad indentation, chained comparisons | fixes the text |
| `LOAD-HEADER` | `missing tally version`, `missing on date`, `missing start or end`, `missing from date` (a scenario's header), `start after end`; `<n> files have a model header; exactly one does` | completes the header, or keeps one model header |
| `LOAD-VERSION` | `files declare different tally versions`; `replace rule <d>.<r> in <V> names version <W>` (a `replace rule` belongs to the version that holds it, §5.11) | migrates the old files (§9), or drops the `v` |
| `LOAD-PIN` | `<file> doesn't match its sha256 in use <a>` | re-pins the import after checking it |
| `LOAD-RESERVED` | `<word> is a keyword` (§2.2: built-in names and contextual words are free outside their positions) | renames |
| `LOAD-DUP` | `<id> is declared twice`; `a key entered twice`; `band <b> has a built-in's name`; a built-in id declared again | renames, or removes the copy |
| `LOAD-NAME` | `unknown id <x>` (in a pin, `LOAD-ASIF`); `<x> is a kind, not a unit` (a declared type, §3.2); `rule <d>.<r>: v <x> names no version` (§5.11) | declares it, fixes the spelling, or names a unit |
| `LOAD-SHADOW` | `bound variable <k> has the name of a top-level id`, or the spelling of a key of the index or kind it ranges over | renames the variable |
| `LOAD-KEY` | `<key> is not a key of <index or kind>`; `ambiguous key <key>`; `a key is also an id`; `wrong number of keys`; `range <a>..<b> is out of order`; `<k> is used as a key of <index> and <index'>`; `<period> lies across keys <a> and <b> of <index>` (§4.6, where load sees it) | fixes the key or names its index |
| `LOAD-AMBIG` | `<literal> reads as a year and as an integer here` (§2.1, §4.1) | writes the period word (`year 2026..2030`) or `int` (`int 1977..2026`), or names the integer's index |
| `LOAD-KIND` | `<operation> on a <kind>`: an addition to a range or an open index, a leg to or from an asset, a move or observation of kind `adjustments` (§5.14.3), a `fulfils` naming neither a planned instance nor an adjustment (§5.19) | uses the right kind |
| `LOAD-LIFESPAN` | `<k> enters <kind> without lives`; `<k>'s lifespan has no start` | states the lifespan |
| `LOAD-LEAF` | `<k> enters <kind>, which has subkinds` | enters it at a subkind |
| `LOAD-PLACEKIND` | `place kind <K> has no unit or sign rule`; `<k> enters a kind of unit one without its unit` | completes the kind or the entry |
| `LOAD-PARTS` | `kind <K>: <clause> missing` (`owned by` or `batched` without `step` or `takes`; `step` or `takes` without either); `kind <K> keeps parts and is <signed, max 0, an asset kind, not a place kind>`; `<K> states parts clauses under <kind>, which keeps parts`; `arrival rule <D> is not a num keyed by a movement and a party at its movement's day`; `owns[<p>, <x>]: <x> keeps parts`; `same <p> = <q>: <x> keeps parts`; `of on a row of <input>`; `the parts of <x> at <instant> have no total in <version>`; `the stated parts of <x> sum to <a>, over its total <b>`; `the stated part of <q> at <x> is below 0`; `the stated parts <q> and <q'> of <x> overlap`; `of <q> at <x>, which keeps no labels of its sort` (§3.6, §5.7, §5.10, §5.19, §6.1) | completes the kind, makes it `min 0`, moves the clauses, fixes the arrival rule, drops the `owns` rows or the `same`, or fixes the reading |
| `LOAD-ALIVE` | `binding over <kind> needs on or over` (any binding over a kind but the key bind of a def, input, place or fill, §5.6) | states the day or the span |
| `LOAD-DOMAIN` | `range of quantities <a> .. <b> has no step`; `<operation> has no meaning for <values>` (§8.4, seen at load: a built-in's argument outside its domain, `day_of` or `schedule` of a key of an anchored index among them, §4.2); `discrete over a key list or a stepped range` (§5.16) | states `step`, or fixes the operation |
| `LOAD-UNIT` | `<a> and <b> have different units` (money in two currencies, money ± a plain number (a bare literal takes the other operand's unit, §7.1), a `num CUR` stored as money, an exchange rate applied to the wrong currency (§3.2), rate arithmetic outside §3.3, `1 day / 1 year`, a step finer than the minor unit for money, a leg or settlement in another unit than its places, a value of the wrong type, a row in another unit than its type gives a key it states (§3.2), a calendar argument that isn't a calendar input's read (§4.2), a band over things (§5.9), a unit stated on a place whose kind isn't of unit `one` (§5.10), a random fact whose rows mix sample types (§5.16), `pow` of a quantity with a unit to a power that isn't a literal (§7.1)); `<operation> orders things of <K>` (`<`, `min`, `max`, `first` or `last` without `ties by`, §8.4); `axis <a> is used as a mode, but <o> is not a mode name`; `money in an undeclared currency` | converts explicitly, rounds, declares the currency, or tests the axis by rules (§5.5) |
| `LOAD-PRECISION` | `<literal> is finer than the minor unit`; `<a/b> terminates; write it as a decimal` | fixes the literal |
| `LOAD-LITERAL` | `figure <n> in a rule; make it a row`; `a date in a rule; make it a version` (§7.1) | moves the figure to a row with a source |
| `LOAD-FIELD` | `positional record in <row>; name its fields`; `<field> is not a field of <type>`; `<row> lacks field <f>` | names the fields |
| `LOAD-INSTANT` | `balance reading <x> at <D> needs start, end or after`; `row <x>[k] of a reading has no instant`; `<x> is not a reading; its rows take no instant` | states the instant, or removes it |
| `LOAD-GOVERNS` | `version <v> has rows with no time of their own and no governs` | adds `governs`, or a span to each row |
| `LOAD-KNOWN` | `version <v> has no known date`; `version <v> is known after the model's on date`; `row <x>[k] is known before its version <v>` | adds the date, or moves `on` |
| `LOAD-OVERLAP` | `rows <a> and <b> of <x> overlap at <keys> on <span>`, neither replacing the other (§4.6) | adds `replace`, ends one row's span, or makes one more specific |
| `LOAD-REPLACE` | `replace <x>[keys]: no earlier row to replace` (in a scenario, no base row at those keys either); `replace rows of <b> in <V> patch no table on <span>` (§5.9) | removes the `replace`, or states the whole table |
| `LOAD-LABEL` | `rule <label> already exists in <def>; write replace`; `beats names no rule <label>` | writes `replace`, or fixes the label |
| `LOAD-MOVES` | `moves <V>: unknown, or not known before this version`; `two moves of <V> known the same day` | fixes the line |
| `LOAD-GAPROLE` | `<version> states balance readings of <x> with neither pad nor anchor` | adds `pad x from y` or `anchor x` |
| `LOAD-BAND` | `band <b> in <version>: rows unordered, overlapping or with a gap at <x>` (per key for a keyed band; x is the lower end of the first row, in the table's order, that doesn't start where the row before it ends: an overlap, a gap or a row out of order); `marginal(<b>, x)` over a first row from `-inf`; `band <b> in <version>: replace rows that neither patch the table nor form a whole one, or that mix with plain rows` (§5.9) | fixes the intervals, declares the gap with `?`, or states the whole table |
| `LOAD-LAWDATE` | `<def or move> reads law-dated facts and states no at`; `ask <q> reaches an at reader def, or reads law-dated <x> directly, without law`; `move <m>'s at reads law-dated <x>` (§5.14) | states `at`, or the ask's `law`, or reads the date through a def with its own `at` |
| `LOAD-LAWPERIOD` | `at <m>: a period is not a law date` | writes a day of m (`start_of(m)`, `end_of(m) - 1 day`), or sums a def keyed by days over m (§4.3) |
| `LOAD-SCHEDULE` | `schedule <s> gives no date in the run` | fixes the schedule, or the run |
| `LOAD-CYCLE` | `<x> depends on itself at the same key: <path>` (over every rule, whatever its `when`); `<x> reads itself both earlier and later on <index>`; `a beats cycle`; `<decision> reads its own future without from` | breaks the cycle, reads one direction, or writes the projection |
| `LOAD-GUARD` | `<x>[k ± n] is a recurrence read with no condition on k or pos(k) guarding it` (a check that a guard exists, §5.11; whether it suffices is `RUN-KEY`) | adds `when pos(k) > n`, a rule that beats it, or an `if` |
| `LOAD-TO` | `<function> needs to u`; `continuous distribution <d> needs to u` | states the precision |
| `LOAD-LEG` | `<movement> lacks leg <l>`; `leg <l> is not a leg of <kind>`; `attribute <a> is missing or unknown`; `leg <l> of observation <r> is in <u>, not its kind's unit`; `observation of uncertain kind <K>`; `observation <r> is dated by a window` | completes or fixes the legs |
| `LOAD-READS` | `<move> reaches a running read or projection of <x> without reads <x>`; `<move> declares reads <x> and never reads it`; a running read outside a decision and its defs | declares the read, or removes it |
| `LOAD-ORDER` | `decisions <a> and <b> read each other on <d> with no before or after`; `<a> before <b> contradicts a read of <b>'s legs`; `<a> before <m>: <m> is a plan or an observation` (§5.14.2); `<x> after <m>: <m> is a plan or an observation` (a reading's instant, §4.6) | states `before` or `after`, against a decision only |
| `LOAD-PRICE` | `value(<x>) in <CUR> holds units in <u> and states no by`; `asset <x> names no price input`; `<x> names a price input on its kind <K> and on itself` | names the price input |
| `LOAD-COVERS` | `<x> is not a place`; `observation <r> of <x> is dated outside the span <version> covers` | fixes the name, or moves the row to the version that covers its day |
| `LOAD-COVERED` | `<def> aggregates observations over a span with no covered() guard` | guards the aggregate with `covered(x, span)` (§5.19) |
| `LOAD-SCENARIO` | `<item> of scenario <s> is read outside it`; `<item> of the base names <x> of scenario <s>` (a rule's `v` included, §6.4); `a row of <s> for <x>, an item of the base, governs a day before <from>`; `move <m> of <s> books on <d>, before <from>`; `an expression row outside a scenario`; `an expression row reads <x>, a place or a movement`; `over cycle or deeper than 4`; `<s> changes who is alive in <K> on <d>, before <from>` (a new entry, or a replaced lifespan that differs from the one it replaces before `from`, §6.4); `ask <q> names <w>, neither the model nor a loaded scenario` (an ask's world loads with its scenario's file, §6.4) | moves the item, fixes the span, or loads the scenario's file |
| `LOAD-WORLD` | `ask <q> in a scenario file names no world` | adds `in <s>` or `in <model>` |
| `LOAD-SAME` | `same <p> = <q>: <what p is> and <what q is>` (places or things of kinds <K> and <K'>, a thing and a movement, a row and a movement, rows of types <T> and <T'>, kinds declared differently, places of two units or two price inputs, a def, an index or another item, §6.1); `same <p> = <q> in a version names a kind or a thing`; `the joint model states <x>[k], an item of member <a> that no same line joins`; `currency <C> is declared with different minor units by <a> and <b>`; `members copy different versions of law part <L>` | fixes the identity, writes the row on an item of the joint model's own, or pins members that copy one version of the law part |
| `LOAD-RANDOM` | `<line> is random; apply a query`; `<query> has no closed form; state paths N seed S`; `<projection> reaches random <x> after <t> with no random clause or query`; `cdf of a non-affine random value`; `<query> on a path p line`; `the draw <x> can exceed the last slot of <index>` (a random number of items, where load sees the draw's support pass the index) (§5.16) | adds the query, `paths` or the clause, or widens the index of slots |
| `LOAD-ASIF` | `as if deeper than 4`; `pin on unknown id <x>` | flattens it |
| `LOAD-DEPTH` | `expression nested deeper than 8` | splits it into defs |
| `LOAD-FORKS` | `ask <q> reaches <n> axis combinations; the cap is 64` | fixes axes with `with` |
| `LOAD-ASKONLY` | `solve outside an ask` (`solve` is allowed in a rule with `v`, §5.18); `<query> has no closed form outside an ask` (§5.16) | moves it into an ask, or a `solve` into a rule that cites the law with `v` |
| `LOAD-MISSING` | `missing() outside the when of a cited rule`; `missing(<r>): <r> is not a read of an input` | cites the law, or lets the hole stand |
| `LOAD-ORIENT` | `ask <q> reaches crps or pdf without orient` | states `orient lower` or `higher` |

### 8.4 Refused per value

Printed as `refused: CODE message` at that key. A refusal propagates like a hole and outranks it.

**What isn't defined is refused** (R-load). Where this definition gives an operation no meaning for the values it is given (a comparison of two records (§3.1), an order on things (`<`, `min`, `max`, `pos`, key arithmetic, §5.7) or on an open index's keys (a range, a key comparison, key arithmetic, `pos`, §5.6), `window(a, b)` with a after b, a step or a count that isn't positive, a built-in's argument outside the domain its row states), the operation is refused, at load where the types show it (`LOAD-UNIT`, `LOAD-DOMAIN`) and otherwise at the key (`RUN-DOMAIN`). At load, an operation the values' type doesn't have is `LOAD-UNIT`, as a value of the wrong type is: an order on things (`<`, `min`, `max`, `pos`, key arithmetic, or `first` or `last` over things without `ties by`), on an open index's keys, on texts, codes or month-days, a comparison of two records, `periodic` of a rate whose period can't pair with `day` (§3.3), and `unit` of a place of unit `any` or of a quantity of no unit kind (§5.10). A built-in's argument outside its domain (`day_of` or `schedule` of an anchored index's key among them, §4.2) is `LOAD-DOMAIN`. It is never given a meaning by guess (§1, quality 1). The same holds where the definition leaves a construct's value open, a case no rule of it reaches: it is refused in the same way, and `coverage.md` lists each such case the reference interpreter found as an open edge.

| Code | Message | What the writer does |
|---|---|---|
| `RUN-NONE` | `<x>[k] is none here: <arithmetic, ordering, aggregate or comparison with a value>` (§3.1) | tests it with `== none`, or limits the binding |
| `RUN-NOT-INT` | `<value> is not an integer` | rounds it |
| `RUN-KEY` | `<k ± n> is outside <index>` (a guard that doesn't suffice); `<key> was added after the cut`; `<d> is outside the days of <def>` (a def keyed by an index of days, read at a computed day, §5.11); `draw <n> of <x> passes the last slot of <index>` (a random number of items, on that path, §5.16) | guards the read, or widens the index |
| `RUN-CYCLE` | `<x>[k] reads itself through <path>` (a read at a computed key, §5.11; a decision's plain read of its own day, or an arrival rule's read of its arrival's day's end, reached through a computed day, §5.14.1); `the takes of <movements> at <places> read each other in one batch` (a cycle that holds a plan, §5.14.2) | breaks the chain, reads an earlier key, writes the projection, or makes a plan of the cycle a decision |
| `RUN-CONFLICT` | `rules <a> and <b> both win for <x>[k] on <d>` (§5.11) | adds a `beats`, or narrows a `when` |
| `RUN-NO-RULE` | `no rule applies to <x>[k] on <d>`; also an arrival at a place that keeps owners, or a pad there, that its arrival rule doesn't reach (§3.6, §5.19) | adds a base rule, or a `?` rule |
| `RUN-LABEL` | `the shares of <movement>'s arrival at <place> sum to <s>, not 1`; `the share of <party> in <movement>'s arrival at <place> is below 0` (§3.6) | fixes the arrival rule, or writes an unknown share as a `?` rule |
| `RUN-TIE` | `<aggregate, fill, move> has a tie at <keys> that changes the result, and no ties by`; `<first or last> over <things, movements or open-index keys> has no ties by` (§5.11); `<outflows> tie for a part of <place> that can't cover both` (§5.14); `the split of <movement> at <place> ties at <labels>` (an arrival's or a `prorata` take's remainder, §3.6) | states `ties by`, or a term on the outflow |
| `RUN-EMPTY` | `<min, max, first, last, argmin, argmax, mean, share, quantile> over no keys` (an empty set wholly before the cut, a closed index, or a `where` that leaves no path) | widens the binding, or tests `count` first |
| `RUN-DIGITS` | `<x> passes 4 096 digits; round where the law rounds` | adds the law's `round` |
| `RUN-DIV` | `division by zero` | guards it |
| `RUN-DOMAIN` | `ln, sqrt or pow outside its domain`; `<operation> has no meaning for <values>` (above); `<a> and <b> are in different keys <u> and <u'> of <K>` (§3.2); `<code> is not a <family> code` (a code read from a row where a position takes another family, §3.1); `of <q> at <x>, which keeps no labels of its sort`, a balance, value or leg read (§5.10, §5.14) | guards it, reads the price at the quantity's key, or fixes the row |
| `RUN-UNDECIDED` | `<function> to <u> not decided within the precision cap` | states another `to` |
| `RUN-PARAM` | `<distribution> parameter <p> = <v> is invalid` | fixes the input |
| `RUN-ZERO` | `add_business(<d>, 0, <cal>)` (§4.2) | uses `schedule` to roll a day |
| `RUN-NEG` | `negative amount <v> for <leg or fill>` | swaps the ends with a `when`, or writes two moves |
| `RUN-ORDER` | `fill <f>: the order of <arc> is a hole`; `decisions <a> and <b> read each other on <d>` (days known only at run) | answers the hole, or states `before` or `after` |
| `RUN-WEIGHTS` | `prorata weights <w> are negative or sum to 0` | fixes the weights |
| `RUN-PRORATA` | `<total> isn't a whole number of <u>` (under `rest largest`); `<amount> arriving at or taken from <place> isn't a whole number of its step <u>` (§3.6) | rounds the total, or uses `rest last`; rounds the amount, or states a finer step |
| `RUN-PLACE` | `<movement> takes <debt> above 0`, `<claim> below 0`, `<holding> below 0 units`; `<place> is <b> after the day's observations and plans of <d>` (§5.10) | fixes the amount, or declares the kind `signed` |
| `RUN-LIFESPAN` | `<leg> moves into or out of <thing> outside its lifespan`; `<place> ends on <d> holding <positions>`; `<movement> labels a part of <place> to <party> outside its lifespan`; `<party> ends on <d> holding parts of <place>` (checked after the day's whole order, §5.7) | fixes the day, or empties the place first; relabels the party's parts on its last day |
| `RUN-FUTURE` | `<decision> on <d> reads <x> at <d'> after its day` (a running read or a sample, reached through a computed day) | reads an earlier day, or writes the projection |
| `RUN-SOLVE` | `no sign change on [lo, hi]` | widens the bracket |
| `RUN-RECORD` | `observation <r> fulfils <instance>, which doesn't exist or doesn't touch its places`; `observation <r> fulfils <instance> and lacks its leg <l>`; `settlement <m> of <adjustment> touches neither place of its leg`; `settlement <m> of <adjustment>: the leg is turned round` (§5.14.3); `<movement> fulfils <adjustment>, which no version made by the cut` (§5.19); `reading <r> after <m>: <m> is modelled past and nothing fulfils it` (§4.6); `same <p> = <q>: <p> doesn't exist`; `<r> and <r'> state one transfer and both fulfil <instance>: state them same` (§5.19); `the stated parts of <x> at <instant> sum to <a>, over its total <b>`; `reading <r> of <x> at <d> start pads and states parts while the day's first batch moves <x>` (§5.19) | fixes the link, waives the adjustment, or fixes the reading |

### 8.5 Warnings

A warning leaves the value as it is and prints beside it.

| Code | Message | Means |
|---|---|---|
| `WARN-NEGATIVE` | `<account> is below zero from <d>` | the first day of each stretch whose end-of-day balance is below zero (N-1607); tested only where the balance is known |
| `WARN-UNREAD` | `<source, input, def or place> is read by nothing` | something was declared and never used, such as an account no total includes (N-1903, D-15's lint). Decided on the item graph (`ENGINE.md` §3.1), so it doesn't depend on what the asks reach |
| `WARN-LEGS` | `the two rows of <movement>, stated same, disagree: <a> and <b>` | the import has a gap or an error (§5.19) |
| `WARN-LABELS` | `labels dropped at <place> from <d>: <movements>` | a leg took labelled parts from a place that keeps them to a place of the model that keeps none and isn't `outside` (tally 2.1, §3.6); listed in the register of every result whose reads reach that place from the leg on (§8.2) |
| `WARN-STALE` | `scenario <s> replaces <target> at <keys>, changed in the base by <version>` | the base moved under the scenario (§6.4); raised by the store, never by a run |

The operations that change a reality have their own errors (`OP-*`) and warnings (`WARN-BACKPAY`, `WARN-DRAFT`, `WARN-ENDED`), listed in `OPERATIONS.md` §10.

---

## 9. Versioning of the language

<!-- tally2: D-1..D-15 -->

- **Every file states `tally N`.** This document defines `tally 2`. Until the engine that runs it is done (`DONE.md`), the frozen text takes patches (`tally 2.0.1`, …, `DONE.md` rule 1): each corrects the definition of `tally 2`, files still state `tally 2`, and it is released with the last of them.
- **A released version never changes.** An engine runs every version it supports, each with its own semantics and its own canonical printer, so an old result can always be reproduced (C-15). `tally 1` (1.0.1) stays runnable as released.
- **A new version comes with a migration:** a mechanical rewrite of any `tally N` model into `tally N+1`. Each migration is tested by running both on every model and case. Every ask must print the same result, or the change must be listed with its reason. The migration from `tally 1` is `MIGRATION.md`: each change is automatic, warned (rewritten, with the lines a writer checks listed) or manual.
- **A new reserved word** that collides with an id is renamed by the migration.
- **Every result records** the `tally` version and the engine build that produced it.
- **One version per reality.** All files of a reality declare the same N (`LOAD-VERSION`), so a reality is migrated as a whole, in one amendment.
- **What may change between versions:** new built-ins, distributions, codes of the code types (§3.1) and refusals. A change to the meaning of an existing construct is allowed only with a migration that rewrites every use whose result would change.

---

## 10. What Tally can't say, and why

<!-- tally2: D-2, D-6, D-11, D-13, D-15; SF-19 -->

| Tally can't say | Why | What to write instead |
|---|---|---|
| **Open recursion or an unbounded loop** | Every model must terminate, with a known cost (spec C-02, §4#2) | a recurrence at earlier keys, or at later keys of a bounded index; a read at a computed key, which `RUN-CYCLE` stops; a bounded index or a stepped range with `first` or `min` (§5.11) |
| **"Choose the best" without a stated cost** | The engine never optimises or picks for the household by a measure it wasn't given (spec §9, N-1802) | a choice over a finite set by a stated cost, `argmin(f for o in S) ties by e`, as a question in an ask or as a rule the household or the law states (D-15); the least value meeting a condition, `min(v for v in a .. b step s where c)`; a breakeven by `solve`; axes or scenarios side by side. No search over an unbounded or continuous space beyond `solve`'s root |
| **A default for a missing fact or statement** | Never wrong in silence (spec C-03, §4#8) | a row, an `assumed` row with its reason, `none`, or the hole |
| **A default convention** (day count, compounding, roll, rounding mode, a currency's minor unit) | A default is a silent assumption (CHG-06, D-12) | the convention as a code in a row, or an axis |
| **Floating point, or an approximation without a stated precision** | Results must agree on every machine to the minor unit (N-219) | `to u`, correctly rounded |
| **A density from a sample, or an infinite horizon from paths** | A sample has no density, and a path is finite (N-1408, N-1504) | a hole with the reason; the closed form asked as a value |
| **A later file or line that overrides an earlier one** | One namespace, and precedence by the facts' own dates (§2.5, §4.6) | a version with its `known` and `governs`, and `replace` |
| **An edit of history** | The old view must stay runnable (N-705) | an added version with `replace`; a settlement dated on the instance's own day (§5.14.3) |
| **Reading the future** | A decision on day t sees only what was known on t (spec §4#3, CHG-02, R-04) | the projection `bal(x, d) from t` over the plan as known on t, with its `windows each` and `random` clauses; observations once they exist |
| **Keys made by computation** | Every key is data, so its existence is sourced and dated | an entry into a kind or an index in a version; an open index from observations; a stepped range, finite and written |
| **A user-defined function of values** | Totality and a fixed cost per evaluation | a def keyed by its arguments (a stepped range included, read with brackets), a band of one argument (keyed bands for a table per key), or `x as if {pins}` to rerun a formula on other arguments (N-1904) |
| **Classification by pattern** (a rule that applies to any movement whose legs look alike) | Classification by data drifts in silence (D-2) | a kind, and a dated membership of it stated as a fact |
| **A random lifespan** | A lifespan is a fact, and a thing's existence is never sampled (SF-19) | a random date fact and a def that tests it (`alive[p, d]`) |
| **Correlation between distinct random facts** (copulas, correlation matrices) | Deferred: no visible case needs it (D-11) | one random fact read in both places, for full correlation; a `discrete` distribution over an index of joint outcomes |
| **A distribution outside the closed list** | Inversion needs a known quantile, correctly rounded | `discrete` (a draw by lot over keys, `e[x]` for known values, x a random fact of value `discrete(1 for k in IDX)`), a band from a uniform to a value, or a transformation of a listed one; a compound distribution as an enumeration or a recurrence (Panjer) over a bounded index |
| **The known part of a sum one of whose terms meets a rounding or a comparison with an unknown** | A partial answer is exact only while it is linear in its holes (P-9, §5.18) | the known part under a pin of the unknown to a value that zeroes its term (`with x = 0`), the pin printed beside the result |
| **A rerun of covered days under another rule or plan** | Observations stand (P-10): a branch over covered days would book them a second time | the difference by cause, written as defs over the plan's legs; or the swapped rule's chain beside the real one |
| **A total the law defines without its split** | A split the law leaves open is not a fact (UL-13's shared cap) | the total, as a fill against the one cap; if a source asks for the split, its orders are an axis |
| **Built-in lot pooling or vintages** (CHG-01) | The pooling rule belongs to the law, and each jurisdiction has its own | a `batched` place, its `takes` order the method (FIFO, an average pool by `prorata`; §3.6), or lots as a `fill` over acquisition movements (wash-sale lots; §5.12) |
| **Text processing, input and output, the network** | The model is pure data over numbers and keys; a `text` value is only held and compared (§3.1) | notes, locators and questions |
| **Identifiers in Polish** | ASCII ids are unambiguous for every tool (N-1611) | Polish in notes, locators and questions |
| **Signed parts: whose an overdraft is** (tally 2.1) | Owners of a debt need parts below zero and rules for netting one owner's deposit against another's overdraft, a design of its own; owned and batched places are `min 0` (G1 Q-3; the round's pressure list, PR-1, PR-2) | the overdraft as a place that keeps no parts (a `debts` place), shared by `owns`; or the account left unowned, so money passing through it drops its labels, with `WARN-LABELS` |
| **`same` on a place that keeps parts** (tally 2.1) | Two members' arrival rules would book one place's parts (PR-3) | the place in one member, the other's movements into it joined by `same` or `fulfils` on the movements |
| **Sharing a sale among one batch's owners while refusing a tie between two lots of one day** (tally 2.1) | One `takes` clause can't do both, and no row needs it (PR-4) | an order term on the outflow's seller, so the owners don't tie |
| **"The day it arrived here" as a batch's order** (tally 2.1) | A batch is the acquisition; the day it reached a place isn't writable as a term (PR-5) | the acquisition day, `k.batch.day` |
| **A lifespan that ends on a drawn day** (tally 2.1) | A lifespan is a fact (SF-19); a drawn death is a move keyed on the draw, which no lifespan check reads (PR-6) | the relabel and the rules that pay the party keyed on the same draw |
| **Another remainder mode for an arrival's split** (tally 2.1) | Every split is §5.13's `rest largest` at the kind's step (PR-7) | the `owned` clause's `ties by` naming the party that takes the odd step |
| **A leg that refuses to take beyond one party's parts** (tally 2.1) | A shortfall goes on down the leaving order, as tracing does; the leg restriction `from x of q` was dropped (PR-8) | a `check` ask over the rows |
| **An order between two observed rows of one day** (tally 2.1) | The day's first batch has no order (§5.14.2; PR-9) | a reading's stated parts, which heal the hole the batch leaves (§5.19) |
| **A range on an unknown** (tally 2.1 keeps the slot) | Research first (item 4); the best method isn't chosen | the hole; a hole part keeps its amount, so a bound can be added later (§8.1) |
