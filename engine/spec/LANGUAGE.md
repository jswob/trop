# Tally: the language definition

Version 1 (`tally 1`). This document and `ENGINE.md` are the whole definition. A modeller writes models from them, and an evaluator works a model by hand from them. Nothing else is needed or allowed.

Tally comes from candidate B (`candidates/B/sketch.md`), with ideas from A and C and the fixes from the smoke tests (`judging.md` §8–9). Each departure from B is recorded in `DECISIONS.md`.

**Contents**
1. Overview and core concepts
2. Syntax: lexical rules, grammar, canonical format, names and stable ids
3. Types and units
4. Time, knowledge and the read rule
5. Constructs: what each computes, its refusals, its cost and its cases
6. Files, versions and amendments
7. Numbers, rounding, randomness and determinism
8. Holes, the register, refusals and warnings: the catalogue
9. Versioning of the language
10. What Tally can't say, and why

---

## 1. Overview and core concepts

A **model** describes a financial reality: money, law, time, people and uncertainty. The reality can be a household over thirty years, one worked example from a tax office leaflet, or a trust with forty beneficiaries. The engine (`ENGINE.md`) runs a model two ways:
- it **replays the past** against real records;
- it **forecasts the future**, one path or many.

It also answers **asks**, the questions written in the model.

A model is plain text in files with the extension `.tally`. It has one canonical form: the engine reprints any model in that form, so two equal models print identically and a diff shows only what changed.

**The qualities that settle every trade-off, in order:**
1. **Never wrong in silence.** A missing fact is a hole that prints its question. It is never 0.
2. **Simple, stated once.** Every fact appears once. A change is an addition beside what was there.
3. **Explainable.** Every number traces to the rows, rules and sources that made it.
4. **Deterministic to the minor unit,** on any machine.
5. **Fast enough for conversation,** with headroom.
6. **Expressive as data.** A new law year, contract or household rule is data, not engine code.
7. **Writable by an LLM.** The surface is small and typed, and it is checked before anything runs.

### The concepts

| Concept | In one line | An analogy |
|---|---|---|
| **source** | A document that figures come from: an act, a leaflet, a bank export, the modeller's own assumptions | a dependency in a lock file |
| **version** | A dated, sourced batch of facts: which source, which period it governs, the day it became known, whether back pay is due | a commit with an author date and a commit date |
| **index** | An ordered set of keys: names, integers, years, months or days | an enum or a range |
| **axis** | A list of readings or choices. Every ask shows each option side by side; the engine never picks | a build matrix |
| **place** | Where money or units sit: an account, cash, a holding, a debt, a claim, the outside world | a bucket in a ledger |
| **input** | A named, typed figure whose values are rows in versions. A missing row is a hole | a column whose cells are filled by commits |
| **band** | A table of intervals, checked for gaps and overlaps | a lookup table with ranges |
| **def** | A computed value, keyed, defined by rules with exceptions (default logic) | a pure function memoised per argument |
| **given** | A version row aimed at a def: pins it at stated keys to a result the source states | a test stub |
| **fill** | Allocation of supplies to demands along arcs, in a stated order | a greedy scheduler |
| **pay** | A dated transfer between two places | an append-only ledger entry |
| **draw** | A reproducible random number per address, by inversion of a closed list of distributions | a hash-seeded PRNG call |
| **bound** | A step declared out of reach, with its reason | a `todo!()` that refuses at compile time |
| **scenario** | A file of versions and items layered over the reality from a date, changing nothing before it | a feature branch that is never merged unless promoted |
| **record** | A real movement from an import, booked instead of the model's pays where a statement covers the place | a log line from production, replayed against the test's expectations |
| **ask** | A question: values, path statistics, scores, roots, as-known views, what-ifs, explanations, checks | a query |

**How they fit.** Facts come from sources. They enter as rows in versions, and inputs read those rows. Defs compute from inputs, bands and other defs. Pays move money between places, using defs for their amounts. Asks read defs and balances. A version's two dates let any ask see the reality as it was known on any day.

---

## 2. Syntax

### 2.1 Lexical rules

- **Files** are UTF-8. Lines end with `\n`. A body line is indented by exactly two spaces per level. Tabs are refused.
- **Identifiers** (ids) match `[a-z][a-z0-9_]*`, at most 48 characters. They are ASCII (N-1611).
- **Keys** match `[a-z0-9][a-z0-9_-]*` (`u5`, `ty2014-15`, `t4e-171`), or are integers, years (`2025`), months (`2025-03`) or days (`2025-03-31`). A token that matches the key pattern is one key, `-` included, so binary minus is written with a space on each side (`a - b`), as the canonical form prints it.
- **Numbers.**
  - Integers: `40000`, `-3`.
  - Decimals: `1190.05`, `-0.0023`. No exponent, except the step `1e-N` in a unit position (§2.3 `UNIT`, §7.2), and no thousands separators.
  - Fractions: `5/900`, allowed only in rows and only when the value doesn't terminate as a decimal.
- **Dates** are ISO: `2026-07-14` (a day), `2026-07` (a month), `2026` (a year). As a value of type `date`, a month or a year is an **imprecise date** (§4.4).
- **Strings** are in double quotes, with `\"` and `\\` as the only escapes. Polish text is allowed in strings, and only there.
- **No comments.** The only free text is `note "..."`, a row's locator string, a source's title and address, the label of a unit or an axis option, an input's `question`, and the question or reason of a hole, an assumption or a bound.

### 2.2 Reserved words

Words of the language fall in three classes. Only the first is reserved everywhere, so the everyday words of the domain (`age`, `discount`, `debt`, `share`, `order`, `rate`, `start`, `up`) stay free for ids.

**Keywords** may not be used as an id, a key, a record field or a bound variable (`LOAD-RESERVED`). They are the section kinds, the words of expressions, pins and row meta, and the heads of asks:

```
model part scenario source version unit axis index place input band def fill draw pay bound ask
tally note if then else and or not in for where with as when excepts true false inf by at law
known governs assumed path check solve explain inputs-of residuals
```

Two keywords also name built-ins, and are read as those where the grammar takes an id: `place` is the built-in index of places (`x in place`, `key place`, §5.6) and the built-in record field (`bank[r].place`, §5.18), and `path` is the index of paths (`p in path`, §5.6). A model still may not declare either, or use it as a key, a bound variable or a record field of its own (`LOAD-RESERVED`, not `LOAD-DUP`).

**Built-in names** are read as built-ins only directly before `(`, so any id, key, record field, bound variable or rule label may take one. A band may not, since it is also called with `(` (`LOAD-DUP`). They are the functions, aggregates and distributions:

```
sum count min max any all first last argmin argmax share mean quantile prorata flows bal free
plan_low next_on worth covered round pow exp ln sqrt abs periodic factor dcf days add_days
add_months add_years start_of end_of year_of month_of day_of overlap_days pos age age_months
weekday next_business prev_business add_business band_lo band_hi marginal kind currency
missing real normal lognormal exponential gamma poisson binomial negbin uniform discrete
cdf pdf pmf crps u
```

**Contextual words** are reserved only in their own position, and are free everywhere else:
- types and their parts, in a type: `money num int bool date dates month year key rate nominal effective simple discount movement`;
- period words, as an index's period kind and as the period argument of `periodic` and `factor` (`month`, `year`, `day`, `quarter`);
- mode names, in a mode position (`round`, `prorata`, a conversion, §7.2): `down up floor ceil half-up half-down half-even`;
- day counts, as the convention of `dcf`: `act365 act360 actact us30_360 e30_360`;
- place kinds and their clauses, in a `place`: `account cash holding asset debt claim outside holds priced open`;
- clause words, at the head of a line of their section or in their clause: `on start end cite enters arrears of none correction moves covers supply demand arc order from to amount phase promised plan over because question prices rule v rest largest last paths seed orient lower higher weight`;
- pin words, at the head of a pin: `wording`;
- the built-in record fields, in a record of type `movement`: `on place amount explains` (`place` is also a keyword, read as this field after a `.`, above).

**Built-in ids** are top-level ids of every reality, so they follow the rules of ids: declaring one again is `LOAD-DUP`, and a bound variable with one's name is `LOAD-SHADOW`. They are the built-in inputs `opening stated short account_place party_place match_days match_amount`, the indexes `place pay bank_account bank_party`, the axis `placekind`, and the outside places `outside_<cur>` and `outside_<unit>` (§5.9).

**Rule labels** are read only in `excepts`, so a label may be any word of the id pattern, keywords included (`rule first`, `rule none`).

### 2.3 Grammar (EBNF)

`NL` is a newline. `IN` is one more level of two-space indent than the enclosing line. `{x}` is zero or more, `[x]` optional, `|` choice, `"..."` a literal token.

```
file      = header {section}
header    = "model" ID NL IN "tally" INT NL IN "on" DAY NL [IN "start" DAY NL] [IN "end" DAY NL]
            {IN "note" STR NL}
          | "part" ID NL IN "tally" INT NL [IN "law" NL] {IN "note" STR NL}       (law: a law part, §6.1)
          | "scenario" ID NL IN "tally" INT NL IN "from" DAY NL [IN "over" ID NL] {IN "note" STR NL}
section   = source | version | unitdecl | axis | index | place | input | band | def
          | fill | draw | pay | bound | ask

source    = "source" ID STR ["at" STR] NL
version   = "version" ID NL IN "cite" ID NL [IN "governs" lspan NL] [IN "known" DAY NL] [IN "enters" DAY NL]
            {IN "arrears" ("on" lit | "none") ["of" ID {"," ID}] ["for" ID {"," ID}] NL} [IN "correction" NL] [IN "assumed" STR NL]
            {IN "moves" ID "to" lspan NL} [IN "covers" ref {"," ref} NL]
            {IN "note" STR NL} {IN vitem NL}                    (known: required outside a scenario file)
                              (arrears on: a day, or a month or year, §5.13.3; enters, moves: §5.3)
vitem     = ID "+" "[" key {"," key} "]"                                   (adds keys to an index)
          | ID [rowkeys] "=" value rowmeta                                  (a row of an input, or a given of a def)
          | ID ["[" key "]"] ":" interval "=" (expr | hole) [STR]           (a band row, with no governs of its own, §5.8; the key for a keyed band)
unitdecl  = "unit" ID STR NL
axis      = "axis" ID NL IN key STR NL IN key STR NL {IN key STR NL}
index     = "index" ID ( "=" "[" key {"," key} "]"
            | "=" ["year"|"month"|"day"] lit ".." lit ["by" INT] ["from" (MMDD | INT)]
            | ":" "open" ) NL [IN "v" ID NL] [IN "assumed" STR NL] {IN "note" STR NL}
                                                              (MMDD is "04-06"; INT is a day 1-28)
place     = "place" ID [binds] ":" pkind NL
pkind     = ("account" | "cash") CUR ["holds" ref] | ("debt" | "claim" | "outside") CUR
          | "holding" ID "priced" ref | "asset" CUR "priced" ref
input     = "input" ID [binds] ":" type ["prices" lit ["by" ref]] ["question" STR] NL {IN "note" STR NL}
band      = "band" ID [binds] "(" ID ":" type ")" ":" type NL {IN "note" STR NL}   (binds: one, §5.8)
def       = "def" ID [binds] ":" type ["at" expr] ( "=" (expr | hole) NL | NL {IN "note" STR NL} rule {rule} )
rule      = IN "rule" LABEL ["excepts" LABEL {"," LABEL}] ["v" ID] ["when" cond] "=" (expr | hole | solvex) NL
                                                        (solvex: only in a rule with v, §5.16; LOAD-ASKONLY otherwise)
fill      = "fill" ID [binds] NL [IN "v" ID NL] {IN "note" STR NL}
            IN "supply" expr ["for" bind {"," bind}] NL IN "demand" expr ["for" bind {"," bind}] NL
            IN "arc" expr NL IN "order" expr {"," expr} NL
draw      = "draw" ID binds "~" (cdist "to" UNIT | ddist) ["plan" expr] NL
pay       = "pay" ID [binds] NL IN "from" ref NL IN "to" ref NL IN "amount" expr NL IN "on" expr NL
            [IN "for" expr NL] [IN "when" expr NL] [IN "phase" INT NL] [IN "promised" NL]
bound     = "bound" ID STR "because" STR NL
ask       = "ask" ID ":" askbody ["for" bind {"," bind}] ["with" pin {"," pin}] ["as" "known" DAY]
            ["paths" INT "seed" INT] ["orient" ("lower" | "higher")] NL
askbody   = expr {"," expr} | "explain" ref | "inputs-of" ref | "check" expr "=" expr
          | "residuals" ref span ["by" ID]
          | solvex
solvex    = "solve" ref "in" "[" lit "," lit "]" "to" UNIT ":" expr "=" expr

binds     = "[" bind {"," bind} "]"
bind      = ID "in" (ID | "[" key {"," key} "]" | "path") ["where" expr]
rowkeys   = "[" rk {"," rk} "]"            rk = key | "*" | key ".." key   (a key of place may be a keyed place, x[k1, …], §5.6)
rowmeta   = ["governs" lspan] [STR]                             (the STR is the locator)
value     = (["-"] lit | "(" field {"," field} ")" | "[" [lit {"," lit}] "]") ["assumed" STR] | hole
          | "-" ["assumed" STR]
field     = ["-"] lit | hole | "-" | ID "[" key {"," key} "]"    (the last: a pay instance, in explains, §5.18)
hole      = "?" STR
span      = expr ".." [expr] | ".." expr | expr          (each end a date or period, §4.1)
lspan     = lit ".." [lit] | ".." lit | lit            (a version's spans: literal dates or periods, §4.1)
interval  = ("[" | "(") (["-"] lit | "-inf") "," (["-"] lit | "inf") ("]" | ")")
pin       = pref "=" expr | "u" "(" ref ")" "=" lit | "law" expr
          | "wording" expr "for" "[" ID {"," ID} "]"
          | "known" expr | "scenario" ID | ID "=" key                      (the last: an axis option)
                          (law, wording and known take a date; a pin's expr is evaluated unpinned, §5.10)
pref      = ID ["[" pk {"," pk} "]"] {"." ID}      pk = expr | "*"     (*: every key at that position, §7.4)

type      = "money" CUR | "num" [unit] | "int" | "bool" | "date" | "dates" | "month" | "year"
          | "key" ID | "rate" basis | "(" ID type {"," ID type} ")"
          | "movement" CUR ["(" ID type {"," ID type} ")"]                (records only, §5.18)
unit      = uterm {("*" | "/") uterm}           uterm = (CUR | ID) ["^" INT]
basis     = "nominal" INT "/year" | "effective" "/year" | "simple" "/year" | "discount" "/year"
          | "/month" | "/day"

cond      = expr                              (see the parse rule below)
expr      = "if" expr "then" expr "else" expr | or
or        = and {"or" and}       and = not {"and" not}       not = ["not"] cmp
cmp       = sum [("=" | "!=" | "<" | "<=" | ">" | ">=" | "in") sum]
sum       = prod {("+" | "-") prod}          prod = unary {("*" | "/") unary}
unary     = ["-"] post
post      = atom {"[" expr {"," expr} "]" | "." ID} ["to" UNIT] ["as" "if" "{" pin {"," pin} "}"]
atom      = lit | key | ref | call | agg | group | flowsx | crps | distfn | prorata | CUR
          | "(" expr ")" | "[" [key {"," key}] "]" | "-"
agg       = AGG "(" bind {"," bind} [":" expr] ["," expr] ")"
AGG       = "sum" | "count" | "min" | "max" | "any" | "all" | "first" | "last" | "argmin" | "argmax"
          | "share" | "mean" | "quantile"
group     = ("plan_low" | "bal") "(" bind {"," expr} ")"        (a set of places, §5.13.1; CUR: compared with currency(x), §5.9)
flowsx    = "flows" "(" ref "," span ["," "for" expr] ")" ["in" CUR "at" ref (MODE | ID)]
crps      = "crps" "(" (dist | expr ["weight" expr] "for" bind) "," expr ")"
distfn    = ("cdf" | "pdf" | "pmf" | "quantile") "(" dist "," expr ")"
prorata   = "prorata" "(" expr "," expr "for" bind {"," bind} "," expr "," UNIT ","
            ( (MODE | ID) "," "rest" ("none" | "last") | "rest" "largest" ) ")"
dist      = cdist | ddist
cdist     = ("normal" | "lognormal" | "exponential" | "gamma" | "uniform") "(" expr {"," expr} ")"
ddist     = ("poisson" | "binomial" | "negbin") "(" expr {"," expr} ")"
          | "discrete" "(" expr "for" bind ")"                (the weights w[j] for j in IDX, §5.14)
call      = (ID ["[" expr "]"] | BUILTIN) "(" [expr {"," expr}] ")"   (a band, keyed or not; BUILTIN: §2.2)
ref       = ID ["[" expr {"," expr} "]"] {"." ID}
lit       = a number, a date, a key, or true / false
CUR       = 3 upper-case letters, or "XXX" for a stated abstract unit (N-203)
UNIT      = a positive lit (a step: 0.01, 0.05, 1, 100), "1e-" INT, or a ref (§7.2)
MODE      = "down" | "up" | "floor" | "ceil" | "half-up" | "half-down" | "half-even"
LABEL     = a word of the id pattern; keywords allowed (§2.2)
```

**Five parse rules.**
1. **`when … = …`, `at … = …`, `check … = …` and `solve … = …`.** In a rule's `when`, a short-form def's `at`, a `check` and a `solve`'s equation, the left part is the longest prefix, ending just before an ` = ` at bracket depth 0, that parses as a complete expression; what follows that ` = ` is the rule's value, the def's value, or the right side of the check or the equation. So `rule r when q = q164 = a + b` has condition `q = q164` and value `a + b`, `rule died when decedent[c] = 0` has condition `decedent[c]` and value `0`, and `def x[y in ty]: money PLN at law_day[y] = base[y]` has `at law_day[y]` and value `base[y]`. There, a value or a right side with an `=` at bracket depth 0 is written in parentheses, so the split falls at the last such ` = ` (`LOAD-PARSE` otherwise): `rule r when k > 1 = (a2_on[a] = d2_on[d])`. A `solve` value is never parenthesised: the split falls before the keyword `solve`, and the ` = ` after its `:` divides f from g. Elsewhere, where nothing stands before the value but the rule's or the def's own `=`, the value parses as written, and the printer parenthesises it all the same (§2.4). (From C §1. It settles the single-`=` doubts of B-LH-17 and B-fresh, and D7's J7 and J42.)
2. **Comparisons don't chain.** `a < b < c` is refused (`LOAD-PARSE`).
3. **`as if` binds to the reference before it.** `x[t] + y as if {…}` applies the pins to `y` only; write `(x[t] + y) as if {…}` for both.
4. **`-` alone.** `-` is the atom "not applicable" when the next token can't start an operand: `:`, `,`, `)`, `]`, `}`, `=`, `then`, `else`, `and`, `or`, or the end of the line. Otherwise it is unary minus. So `where x[k] != -:` tests for `-`, `rule r when x[k] != - = v` has the condition `x[k] != -`, and `add_days(d, -1)` subtracts.
5. **`in` after `flows`.** The token after `in` decides: a currency code makes the conversion of `flowsx`; `[`, a range or an index makes the comparison. The printer parenthesises a converted `flows` that is an operand of a comparison.

**Calls.** A call takes a built-in's name (§2.2) or a band's id. `min` and `max` also take two or more plain arguments, `min(a, b)`, besides the aggregate form `min(k in IDX: e)`. A period word is accepted as the period argument of `periodic` and `factor` (`factor(i, 1, year)`).

Expressions nest at most **8** deep (`LOAD-DEPTH`). The depth at a point is the number of brackets `( )`, `[ ]` and `{ }` open there (calls, keys and `as if` included) plus the `if`s it is inside; a chain of operators at one level (`a + b + … + j`) adds none. `as if` nests at most **4** deep (`LOAD-ASIF`).

### 2.4 Canonical format

The loader accepts any spacing that the grammar allows. It then reprints the model as follows, so equal models print identically (N-1902).

- **Header first:** `model`, then `tally`, `on`, `start`, `end`, notes.
- **Sections in this kind order:** `source`, `unit`, `axis`, `index`, `place`, `input`, `band`, `def`, `fill`, `draw`, `pay`, `bound`, `ask`, `version`.
  - Within a kind, sections are sorted by id (ASCII).
  - **Versions come last**, sorted by `known` date, then id. A file's versions therefore read like a log, newest at the bottom, and an amendment usually appends at the end.
- One blank line between sections, always, one-line sections included.
- **Inside a section:** notes come first, then rules (sorted by label, then by the `known` date of their `v`, a rule without `v` first) or version items. Version items are sorted by target id, then keys in index order, with a keyless row first, then by the start of their own `governs` span (a row with no span of its own first, then a span with no start); a band's rows stay ascending (§5.8). Index additions come before rows.
- **Clause order** is the order in the grammar. An `excepts` list is sorted by label, with ", ".
- **Short form:** a def with one rule labelled `base`, no `v`, no `when`, no notes prints as `def x: T = expr`.
- **Numbers** print as the shortest exact decimal, with a leading `0` before the point (`0.05`), or as a reduced `a/b` when the value doesn't terminate. Dates print in ISO.
- **Expressions:** one space around binary operators and after commas, none inside brackets. Parentheses appear only where precedence needs them, except that a `… to u` call used as an operand, a rule's or short-form def's value (other than a `solve`) and either side of a `check` with an `=` at bracket depth 0 (parse rule 1), and a converted `flows` inside a comparison (parse rule 5) are always parenthesised.

### 2.5 Names and stable ids

- **One namespace** holds every top-level id of a reality, across all its files: sources, versions, units, axes, indexes, places, inputs, bands, defs, fills, draws, pays, bounds and asks. Two top-level items with one id are refused (`LOAD-DUP`). This makes "a new file silently overrides an id" unrepresentable (§4#18 of the spec).
- **A stable id is the item's id.** A key's stable id is `index:key`; a row's is `version:target[keys]`, followed by the row's own `governs` span, or a band row's interval, when its version holds more than one row of that target at those keys (`forecast_2026:inflation 2028..`); a rule's is `def.label`. Ids are never reused for something else. Renaming is a new item plus an end to the old one (§6).
- **Bound variables** (`k` in `k in person`) are local to their item. A bound variable with the same name as a top-level id is refused (`LOAD-SHADOW`). This removes the "parameter shadows a law value" doubts of all three smoke rounds.
- **Rule labels** are local to their def. They are never read in expressions, only in `excepts`, and they may repeat other names, keywords included (§2.2).
- **Record fields** are local to their record type and are read only after a `.`.
- **Keys.** A key literal is resolved by its position: in `allowance[u5]` it must be a key of `allowance`'s first index. A bare key used elsewhere (`k = u5`, `a = vpi`) takes the index or axis of the other operand. A key literal in a value position (a rule's value, an `if` branch, a band row that returns a key) takes the index of the declared type of the value it gives (`key IDX` of the def or band); the branches of an `if` take the type of the whole. A key literal that could belong to two indexes or axes in one position is refused (`LOAD-KEY`). A key literal that is also a top-level id is refused (`LOAD-KEY`), except a place id where a key of the built-in index `place` is expected, or a pay id where a key of `pay` is expected, since their keys are those ids (§5.6).
- **Bare names** resolve in this order: a bound variable in scope, then a top-level id, then a key. A bound variable over a key list (`k in [1, 12, 13]`) takes its index from the key positions it is used in; two different indexes there are `LOAD-KEY`.

---

## 3. Types and units

### 3.1 The types

| Type | Meaning | Literal |
|---|---|---|
| `money CUR` | A whole number of minor units of currency CUR. The minor unit comes from ISO 4217 (0.01 for PLN, EUR, USD, GBP, CHF, NOK; 1 for JPY). `XXX` is a stated abstract unit with minor unit 0.01, for sources that give no currency (N-203). | `1190.05`; finer than the minor unit is refused (`LOAD-PRECISION`) |
| `num U` | An exact rational with unit U. `num` alone is dimensionless (a ratio, a share, a probability). | `0.0585`, `5/900` |
| `int` | An integer. | `12` |
| `bool` | `true` or `false`. | `true` |
| `date` | A day, or an **imprecise date**: a month or a year meaning "some day in that period" (§4.4). | `2026-07-14`, `2019` |
| `dates` | A finite list of days, such as a holiday calendar. | `[2026-01-01, 2026-01-06]` |
| `month`, `year` | A period value. | `2026-03`, `2026` |
| `key IDX` | A key of index IDX. | `u5` |
| `rate BASIS` | An interest rate with its basis (§3.3). | `0.0585` under a declared basis |
| `(f1 T1, f2 T2, …)` | A record. Fields are read as `x.f1`. A field may hold a hole or `-`. | `(60000, 48, equal)` |

Every value of any type may also be:
- a **hole** `? "question"`: a fact that is missing (§8.1);
- **not applicable**, written `-`. It can only be tested, with `x = -` or `x != -`; any other use is refused (`RUN-NA`). A row, a field or a rule may give `-`: producing it is not reading it. A def whose winning rule gives `-` is not applicable at that key, and prints `-`.

### 3.2 Units and the unit rules

A **unit** is a product of base units with integer powers. Base units are currencies and the units declared with `unit` (`unit hour "godzina"`, `unit shr "jednostka funduszu"`). Units are normalised, so `PLN*PLN` is `PLN^2` and `PLN/hour*hour` is `PLN`.

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
- **A `num` that is an integer** is accepted where an `int` is expected, such as `round(months / 12, 1, ceil)` used as a count of years. Any other num there is refused at that key (`RUN-NOT-INT`).
- **Back to money.** A `money` def and a pay's money amount must be money. A `num CUR` becomes money only through `round(x, u, mode)` with u a whole number of minor units (§7.2), or through `prorata` on the same terms (§5.12). A fill's supply and demand share one unit, which may be any (§5.11).
- **Literal adoption.** In `+`, `-`, `min`, `max` and comparisons, a bare literal takes the unit of the other operand (`base - 8000` is money if `base` is). In `*` and `/`, an integer literal is an `int` and a decimal literal a dimensionless `num`. A literal argument takes its parameter's type. **A bare literal that is a whole value** (a rule's value, a band row's value, a row, a pin, an `if` branch) takes the declared type and unit of its target, so `rule none = 0` on a `num PLN` def is `0` PLN. Unary minus on an allowed literal is allowed (`add_days(d, -1)`).
- **Exchange rates** are `num A/B`: an amount of A per unit of B. `money EUR × num PLN/EUR` is `num PLN`. Converting is explicit, and adding two currencies is refused (N-202). A rate applies only from B to A (U-05): a `money A` or `num A` times `num A/B`, or a `money B` or `num B` divided by it, is refused (`LOAD-UNIT`), though the unit algebra above would give it a unit.

### 3.3 Interest rates

A rate is not money and not a bare number (N-206, spec §4#7). Its basis is part of its type:

| Basis | Meaning |
|---|---|
| `nominal m /year` | an annual rate compounded m times a year (m = 12 for most mortgages) |
| `effective /year` | an effective annual rate |
| `simple /year` | simple interest per year |
| `discount /year` | an annual rate of discount |
| `/month`, `/day` | a rate per period |

- **Allowed:** `rate ± rate` of one basis (a reference rate plus a margin), `rate × num`, and comparisons within one basis.
- **Anything else** (`rate × money`, `rate + num`) is refused at load. A rate enters arithmetic only through these built-ins:

| Built-in | Value |
|---|---|
| `periodic(r, P)` | The rate per period P (`day`, `month`, `quarter`, `year`) as a `num`. For `month`, `quarter` and `year`, let t be P's length in years (1/12, 1/4, 1). For `nominal m /year` it is (1 + r/m)^(m·t) − 1, which is r/m for a period of 1/m year. For `effective /year` i it is (1 + i)^t − 1. For `discount /year` d, the effective annual rate is i = d / (1 − d), then that conversion. For `simple /year` it is r × t. For `/month` it is (1 + r)^n − 1, with n = 1, 3 or 12 months. For `/day` with P = `day` it is r. Any other pairing with `day`, or a `/day` rate over a longer period, is refused at load (`LOAD-UNIT`): a day's share of a year or of a month is a day count, which can't be a default (§10), and `dcf` gives it. A power that isn't an integer needs `to u`. |
| `factor(r, n, P)` | (1 + periodic(r, P))^n, exact for an integer n; a non-integer n needs `to u` (§7.3). |
| `dcf(a, b, c)` | The year fraction from date a to date b under day count c: `act365`, `act360`, `actact` (ISDA), `us30_360`, `e30_360`. A missing convention can't be written (N-207). |

### 3.4 Prices and price years

A money input is nominal: its value is money of the date it governs. A figure stated "in today's prices" is declared with its price year and its index (N-216, spec §4#5):

```
input rent_wish: money PLN prices 2026 by cpi
```

Reading it at law date d gives `num PLN` = value × `cpi[d]` / `cpi[2026]`, exactly. `cpi` is keyed by one time index, so `cpi[d]` reads the key whose period holds d, and `cpi[2026]` the key that is the price year itself: a `prices Y by idx` input needs an index with a key for the year Y (`LOAD-KEY` otherwise). Which month, or which mean of months, stands for a year is the modeller's def over a monthly index (§10). The model then rounds where the payer rounds. A money input that names no `prices` is nominal, so "2 500 in 2028" is a nominal row governing 2028. Neither can be read as the other.

**A price year with no index.** `prices Y` alone states the year of a figure "in 2015 dollars" when the case gives no index. Read at a law date in Y it is the figure itself, in its declared type (`money CUR` for a money input). At any other date it is a hole: `<x> is in Y prices; no index moves it to <d>`. So a figure in old prices is never read as nominal in silence.

### 3.5 Parties, groups and relations

Tally has no built-in party type. Parties are keys of an index (`index person = [ana, ben, ola]`), and everything about them is a keyed input:
- **dated facts:** `born[p]`, `residence[p, m]`;
- **relations:** `spouse[p]: key person`, `parent[c, k]: bool`;
- **groups:** `member[g, p]: bool`, with rows that govern dated spans.

A law reads a relation as it stood on its law date, because the relation's rows govern spans like any other row (N-404).
- A group is an index of groups plus a membership input, so one person can be in several groups, each computed on by a different law (N-405, N-410).
- "The household of X" is a def keyed by X (N-406).
- A family tree is read through relation inputs and bounded aggregates, never through open recursion (N-407).

### 3.6 Places and ownership

Places are declared with `place` (§5.9). Ownership is an input of fractions, `owns[p, x]: num`, whose rows govern dated spans. Each row's version holds the provenance: stated, set by a court, or derived. A kind of ownership is a key of an axis or an index (N-506). Net worth over a set of parties is an ask expression over `bal`, `worth` and `owns`, with outside places left out (§5.16, `ENGINE.md` §9).

**Entities** (a trust, an estate, a fund, a company) are keys like persons, with their own places, periods and laws (CHG-07). A stake in an entity is an input of fractions, `stake[p, e]: num`, with dated rows. **Look-through** ownership is a recurrence over a bounded depth index, never open recursion (§5.10, N-403):

```
index party = [ana, ben, family_trust, holding_co]
index depth = 1..4

def through[p in party, x in place, k in depth]: num
  rule first when k = 1 = owns[p, x]
  rule deeper when k > 1 = owns[p, x] + sum(e in party: stake[p, e] * through[e, x, k - 1])
```

The depth is the law's own when it states one. Cross-holdings (A owns part of B and B part of A) never end, so a `check` ask compares the last two depths. A difference fails the check, and is never truncated in silence.

### 3.7 Provenance

Every row carries its version, and the version carries its source, its governs span and its known date. A row may add a locator string (a paragraph, a page, a table cell). `explain` prints all of these for every input a result reached (N-1701). A def's rule cites through its `v`. A rule with no `v` is model wiring: it has no source, and `explain` shows it as such.

---

## 4. Time, knowledge and the read rule

### 4.1 Spans and periods

- **A span** `a..b` over periods names every period from a through b, inclusive. As time it is the half-open interval [start of a, start of the period after b). So `2015-11..2016-10` is [2015-11-01, 2016-11-01), and `2021` alone is [2021-01-01, 2022-01-01). `a..` is open at the end, and `..b` has no start: read by `flows`, it reaches before S, where `flows` is a hole (`ENGINE.md` §2).
- **Span ends are expressions** of a date or period type, so spans built from keys are written directly: `start_of(year_of(m))..day_of(m, 15)` is a year to date, and `flows(konto, m)` reads one month. In a version's `governs`, the ends are literals (§7.1).
- **Index ranges** of periods: `index m = month 2027-01..2059-12`, `index y = year 2010..2036`, `index d = day 2026-01-01..2026-12-31`.
- **`by n`.** A key of a range `by n` stands for the n periods from it to the next key: `day 2026-01-05..2026-12-28 by 7` gives weeks, `month 2026-01..2026-12 by 3` quarters. Its law date is its first day. On an integer range, `by n` only steps the keys.
- **Anchored periods** (N-306). `index ty = year 2014..2025 from 04-06` makes key 2014 the span [2014-04-06, 2015-04-06), as for a UK tax year. `index bp = year 2015..2016 from 11-01` gives benefit periods from November to October. `index ap = month 2018-05..2019-04 from 17` gives assessment months that start on the 17th. A key prints as the year or month in which its period starts.
- **A one-off period** (N-307) is a key of a list index plus two inputs that give its start and end.

### 4.2 Calendar built-ins

| Built-in | Value |
|---|---|
| `start_of(k)`, `end_of(k)` | the first day of period k; the first day after it |
| `year_of(d)`, `month_of(d)` | the calendar year or month that contains date or month d |
| `day_of(m, n)` | day n of month m, clamped to the month's last day |
| `days(a, b)` | b − a in days |
| `add_days(d, n)` | d moved by n days |
| `add_months(d, n)`, `add_years(d, n)` | the same day number, clamped to the last day of the target month. `add_years(d, n)` is `add_months(d, 12n)` |
| `overlap_days(a, b, k)` | the number of days x with a ≤ x ≤ b that lie in period k (any part of a day counts, N-308) |
| `age(b, d)` | completed years from birth date b to date d. The anniversary of 29 February in a common year is 1 March. **Before b it is `-`** (not applicable). The US rule, where a person born on 1 January reaches an age the day before, is written `age(b, add_days(d, 1))` |
| `age_months(b, d)` | completed months: the largest n with `add_months(b, n)` ≤ d; `-` before b |
| `weekday(d)` | 1 for Monday to 7 for Sunday (ISO 8601) |
| `next_business(d, h)`, `prev_business(d, h)` | d if it is a business day, else the next (or previous) one. Saturdays, Sundays and the days in `h` (a `dates` input) are not business days |
| `add_business(d, n, h)` | n business days after d (before it, for negative n) |
| `pos(k)` | a key's 1-based position in its index |

- **Key arithmetic** works on ordered indexes: `y - 1`, `m + 12`, `k - 1`, and `g + 1` for the next key of a list. A result outside the index is refused at that key (`RUN-KEY`) where it stands in a key position. In a value position a key of an integer index is an int, so `k - 1` at the first key is plain int arithmetic.
- **Keys as values.** A key of an integer index is also an `int`, and an int in a key position of an integer index is that key. A key of a year, month or day index is a value of that period type, so `y + 1`, `y > loss_yr[l]` and a comparison with a `month` input work. A value of type `key IDX` (an input or def typed so) takes part in key arithmetic and comparison as a key of IDX, and may stand in a key position (`std_ded[filing_status[t]]`). Keys compare in index order.
- **The difference of two keys** of one ordered index, `k1 - k2`, is an `int`: the difference of their positions (for an integer or period index, of the integers or periods).
- **Date arithmetic:** `date ± int` moves by days, `month ± int` by months, `year ± int` by years.
- **Lists.** `[]` is the empty list, so "no holidays listed" is a row `holidays = []`.

### 4.3 The law date

A def is evaluated at a **law date**: the day whose law applies. The law date of def x at keys k is, in order:
1. x's `at` expression, if it has one;
2. else the start of the first of x's keys whose index is a time index (a day, month or year index, anchored or not): `max_cut_m[e, n, m]` takes m;
3. **else the law date of whatever is reading x:** the def, pay or ask that asked for x at this moment;
4. else, at the top of an ask with no date of its own, the ask's cut K: the model's `on`, or the D of its `as known D`;
5. else the read is refused (`RUN-NO-DATE`).

Rule 3 is the departure from B, which used the model's `on` date there. It means a def with no time key, such as a child's needs under a maintenance table, read by a monthly pay in July is evaluated with July's law. A snapshot model therefore extends to a year without being restructured (judge's diff b). The value is memoised per (keys, law date), and `explain` prints the law date it ran at. Rule 4 makes a view as known in 2020 read its undated law at 2020: the view's today is its cut.

- **`at`** is any date expression of the def's keys, inputs and defs. It is evaluated at the law date the def would have without it, and its reads follow these same rules. An `at` that gives an imprecise date (§4.4) reads the def once per **law-date class** the period overlaps (the stretches between the governs boundaries of what the def reaches, `ENGINE.md` §3.2), each at its first day inside the period. If all give one value, that is the value; otherwise it is `HOLE-IMPRECISE`. So a period that lies inside one class reads at its first day. An `at` that is a hole makes the def a hole at those keys, carrying the `at`'s questions, and its rules aren't evaluated.
- **A pay instance's law date** is the start of its first time key; with none, the start of its `for` period; with neither, its `on` date, where `on` itself is read at the ask's cut (§5.13.3), or at cut min(t, K) in a look-ahead on walk day t (§5.13.1), with that cut's day as its own law date. So a hole in an `on` asks `… no value at <that day>`, once. Where that falls to an imprecise `on`, it follows the `at` rule above: one law date per law-date class the period overlaps, and one value or `HOLE-IMPRECISE`.
- **Inputs and bands** have no rules of their own: they are read at the law date of their reader, or at their own time key's period (§4.6). **A draw** is dated the same way: its time key's start, else its reader's law date.
- **Pins.** Inside `as if {law D}`, every def and input read with no `at` and no time key is read at D, an input read directly in the ask included. That is the law that governed D, as known at the cut: a version known later that governs D retroactively is read too.
- **The wording in force on D.** `as if {wording D for [x, y]}` keeps every law date and restricts the named items' candidates (rows, band tables, rules through their `v`) to versions that entered into force by D (`enters`, §5.3; §4.6). So "the current act, with the scale in its wording of 30 June" is `wording 2022-06-30 for [scale]`, and a later act that governs that day but entered into force after it isn't read. `law D` and `wording D for` together give the act in its wording of D, applied at D. `wording` always names its items: no case needed every item's wording at once (`coverage.md` R1). A `for` list may name an item that has no candidate under the pin; it reads as any item with no candidate does (§4.6).

### 4.4 Imprecise dates

A month or a year used as a `date` means "some day in that period" (N-301).
- Arithmetic on it gives an interval of days.
- A comparison is **decided** when the whole interval lies on one side. Otherwise it is a hole: `? "date within 2019 not precise enough for <the comparison>"`.
- **A value compared with itself** is decided. Two reads of one item at the same keys, law date, cut and pins are one value, whatever variables bind the keys (`pet_date[h] < pet_date[g]` at h = g), so `x < x` is false and `x = x` and `x <= x` are true even when x is imprecise. Equal intervals from different reads stay undecided.

For example, a lot bought "during 2019" is decidably outside a 30-day window around 2024-12-27, so the comparison has a value.

- **Built-ins** on an imprecise date (`age`, `add_months`, `year_of`, `overlap_days` and the rest) give the interval of their values at the two ends of the period, its first and its last day (2019-01-01 and 2019-12-31 for 2019). When the two are equal it is that value: `year_of(2015-01)` is 2015. `weekday`, whose value doesn't follow its date's order, is 1..7 on any month or year, since every weekday falls in one.
- **Only a comparison decides an interval.** Wherever one value is needed (a print, a money amount, a key, a term of an aggregate other than `min`, `max`, `argmin` and `argmax`, which order their terms by these comparisons, §5.10), an undecided interval is `HOLE-IMPRECISE`.
- **An imprecise date prints as its period** (`2020-07`).
- **A pay whose `on` is imprecise** counts in a span when its whole interval lies inside it, and not at all when the interval lies wholly outside. When the interval crosses an end of the span, `flows` over that span is a hole (`HOLE-IMPRECISE`), and so is `bal(x, d)` for a d inside the period. A state pay's date must be precise (`HOLE-IMPRECISE` otherwise), since it runs at a point of a day.

The cost is two endpoint evaluations per operation, with no enumeration of days. An imprecise `at` or `on` costs one evaluation per law-date class it overlaps, usually one.

### 4.5 Knowledge

**Every version has a `known` date:** the day the reality learned its contents (enactment, publication, the day a payslip arrived).
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
  - A label naming the period a document serves (a form's tax year, an exam sitting, an edition "for 2025") isn't its date.
  - The version's note says which date it took and why.
- **A later text read for an earlier period** (a 2022 text standing for an unfetched 2020 one) gives a version marked `assumed` (§5.3), citing the later text and governing the earlier period, with a note naming the text it stands for. Its rows, and the rules that point at it with `v`, are assumed. Where only some rows stand in, those rows are marked instead. Such a version is known, and enters into force, at the dates of the text it stands for where they are found, since that text's contents were public then; else at the later text's.
- **A hypothetical case's stipulations** (who, when, how much, in a worked example or a composed case) are known from the earlier of the model's `start` and its `on` (its `on` when it has no `start`): they are the case's world, given as though known all along. A modeller's assumption that stands for a case's own fact takes the same date.

Two clocks read it:
- **The ask's cut K.** An ask sees the versions known on or before K. K is the ask's `as known D`, or the model's `on` date if it states none (N-705, N-1219). A version known after `on` is refused (`LOAD-KNOWN`): the reality can't know its own future, and `on` is its today.
- **A pay's booking cut.** A pay instance is booked with what was known on its own date (§5.13.3, and `ENGINE.md` §5).

A version in a scenario file has no `known` line. It isn't knowledge but a supposition, and it is visible by date, from the scenario's `from` (§6.4).

A version marked **`correction`** fixes the record of something that was already true at the time, such as a mistyped payslip or a revised measurement of a past quantity (a second estimate of a year's GDP). It is visible to an ask only from its `known` date, but a pay booked earlier is evaluated as though the payer had always known it. A version without `correction` is a change in the world, such as new law. It reaches a pay booked earlier only through its `arrears` (§5.13.3). Late-known law or stipulations are not corrections: their `known` follows the rules above.

**`assumed`** marks any figure no source states: a forecast, or a fact the source leaves out (a year a leaflet omits, an election a table presumes). An assumption is knowledge like any other, visible from its version's `known`.

### 4.6 The read rule

One rule reads rows of inputs, givens of defs, band rows and dated rules. To read item x at keys k, with law date d and cut K:

1. **Candidates.** A row is a candidate if:
   - every key it states equals the corresponding key in k (a `*` matches every key; a range `a..b` matches every key from a to b in index order, on any index, and a range whose ends are out of order is refused at load, `LOAD-KEY`);
   - a key it leaves unstated, or covers with `*`, is one its index had on the row's `known` date. A default row stated before a key was added doesn't cover that key (§6.2);
   - its version's `known` ≤ K; a scenario version has no `known`, so K doesn't mask it, and it is visible by date instead (§6.4);
   - its governs span contains the period of x's time key, or d when x has no time key. A span contains a period when it contains the period's first day, so `2025-04-06..2026-04-05` governs key 2025 of `year … from 04-06`. The row's own span, when it has one, replaces its version's wholly and may reach past it. A row with no governs span at all governs every date. A version's span is the one in force under K: the latest `moves V to SPAN` known by K replaces V's default span (§5.3). A record row (§5.18) passes this test at every law date: its version's span says what the statement covers, not when the row applies;
   - inside a `wording D` pin that covers x, its version entered into force by D (§4.3).
2. **Precedence.** Among the candidates, prefer in turn:
   1. inside a scenario, the row from the higher layer (§6.4);
   2. the row that states more keys (a specific row beats a default row). A key counts as stated when the row gives it as a literal or a range; `*` and a trailing key left out don't count, so `[t1_15, *]` (one) beats a keyless row (none), and `[t5_023, 1..60]` beats `[t5_023, *]`;
   3. the row with the later `known`;
   4. a row with a governs span over one without.
3. **Ties.** Two candidates still tied, with overlapping spans, are refused at load (`LOAD-OVERLAP`).
4. **No candidate** for an input or a band gives a hole (for a def, a given that doesn't match goes on to its rules, and a rule label with no candidate version drops out, §5.10). Its question is the input's `question`, if it has one, otherwise `x at k: no value for <period>`, the period of x's time key; with no time key, `x at k: no value at <law date>`, and with no keys at all, `x: no value at <law date>`. This is the **gap rule**: a value exists only where a row says so (N-702).

**Reading at a date.** An item keyed by one time index, read with a date or a period of another granularity in that key position (`cpi[d]` for a day d, a monthly price at a day), reads the key whose period holds it. A date outside the index is `RUN-KEY`. A period longer than the index's keys holds no key: it is refused (`LOAD-KEY`).

**What follows from the rule:**
- **A later version overrides an earlier one only on its own span.** A law change, a correction, a new job or a stopped job is written as an addition (N-107).
- **Rows are open-ended unless the source ends them.** An amount stays in force until a later act changes it, so a law change restates nothing it doesn't change. A row with a closed span leaves later periods as holes, which is loud.

---

## 5. Constructs

Each construct below gives its syntax, what it computes (a rule an evaluator can apply by hand), its refusals, its cost and the cases that use it. The case lists were corrected from the round-1 models; the construct-by-case matrix in `coverage.md` is the record. The cost units are those of `PERFORMANCE.md`: one **evaluation** is one rule body computed at one key. Refusal codes are listed in §8.

### 5.1 The header

```
model ul05
  tally 1
  on 2016-03-01
  start 2015-11-01
  end 2016-10-31
  note "Zasiłek rodzinny, okres zasiłkowy 2015/2016, złotówka za złotówkę."
```

- `tally 1` is the language version (§9). It is required.
- `on` is the model date: the reality's today. It is required. It is the default cut K (§4.5), and so the law date at the top of an ask that gives no date of its own and no `as known` (§4.3 rule 4: the ask's cut K).
- `start` and `end` are the run's first and last day, S and E. They are required when the model has places, pays or draws. `start` may stand alone, to date a case's stipulations (§4.5).

**Refusals:** `LOAD-HEADER` if `tally` is missing, a model's `on` is missing, `start` > `end`, or a model with places, pays or draws lacks `start` or `end`. The header is checked before the grammar, so a missing header line is `LOAD-HEADER`, not `LOAD-PARSE`. **Cost:** none. **Cases:** all.

### 5.2 source

```
source rm1238 "Rozporządzenie Rady Ministrów, Dz.U. 2015 poz. 1238" at "https://isap.sejm.gov.pl/..."
source modeller "The model's own assumptions, each stated with its reason"
```

A source is a document. Versions cite it, and `explain` prints it. **Refusals:** a source that no version cites gets `WARN-UNREAD`. **Cost:** none. **Cases:** all.

### 5.3 version

```
version rm2015
  cite rm1238
  governs 2015-11..
  known 2015-09-07
  allowance[u5] = 89 "§ 2 ust. 1 pkt 1 lit. a"
  allowance[u18] = 118 "§ 2 ust. 1 pkt 1 lit. b"
  criterion = 674 "§ 1 ust. 1 pkt 1"
```

A version is a dated, sourced batch of facts.
- It has one source (`cite`), an optional default governs span, one `known` date (none in a scenario file, §6.4) and an optional `enters` date. It may then have `arrears` lines, each `on D` or `none`, optionally `of` earlier versions and `for` named pays; or it is a `correction` (§4.5, §5.13.3). It may have `moves` lines, and an `assumed` line.
- **`assumed STR`** marks the whole version as resting on something no source states: it stands in for a text it doesn't cite (§4.5, a later text read for an earlier period), or its `known` date rests on its file's own stamp because the source has no public one (§4.5, Other sources). Every row of it, and every rule that points at it with `v`, puts the reason in the register of each result that reads it (§8.2).
- **`enters D`** is the day it enters into force, read only by the `wording` pin (§4.3). With no `enters`, it is the later of `known` and the start of `governs`, or `known` when there is no `governs`.
- **`moves V to SPAN`** postpones or extends an earlier version's commencement or end. Under cuts K ≥ this version's `known`, V's default governs span is SPAN, and V's `enters`, when defaulted, follows it; under earlier cuts V keeps its own. The latest-known `moves` of V wins. A row of V with its own span keeps it. A sunset extended by a later instrument is the same line.
- **A commencement left to a later act**, "from a day to be fixed, at the latest D", is a `moves` to D: the latest it can be (§4.5). The act that fixes the day moves it again.
- **An act with several commencement dates** is one version per date, each citing the act and known the same day.
- **It may have no items.** Such a version carries a source, a span and a known date for the rules that point at it with `v`, or for a fill or an index that cites it. A version with no `governs` governs every date, for its rules as for its rows.
- **A consolidated text** (a *tekst jednolity*, a republished act) restates and changes nothing. It is not a version of the rows it reprints, which come from the acts that enact or amend them; it may be their locator. Otherwise its later `known` would beat a transitional provision it doesn't reprint. Only where the enacting text isn't available may a restating text be the source (§4.5, Which act); its note then names the acts it stands for and any transitional provision it doesn't reprint.
- Its items are:
  - **input rows:** `x[keys] = value [governs span] ["locator"]`;
  - **band rows:** `b: [lo, hi) = expr ["locator"]`;
  - **givens:** a row aimed at a def. It pins the def at those keys, and the def's rules aren't evaluated there (§5.10);
  - **index additions:** `person + [ola]`, which adds keys to a list index from this version on.

Rules point at a version with `v` (§5.10).

**What it computes.** Nothing by itself. It supplies the source, governs span and known date of every item in it. Those feed the read rule (§4.6), and asks as known before the version's date don't see it.

**An index addition is dated.** A key added by version V exists only for asks with K ≥ V's `known`. Earlier asks don't list it, and a read of it is refused (`RUN-KEY`). Inside an ask that lists it, a booking cut before V's `known` doesn't see it either (§5.6): an instance at the key counts as A₀ = 0 there (`ENGINE.md` §5 step 1), and the modelled past at that cut doesn't compute it (`ENGINE.md` §6.2). Keys are never removed. A thing that ends gets rows that make it `-` or 0 from its end (§6).

**Refusals:**
- `LOAD-NAME`: an item names an unknown id.
- `LOAD-KIND`: an addition to a range or an open index.
- `LOAD-DUP`: an addition of a key that already exists.
- `LOAD-ARREARS`: `arrears on D` with D (for a month or year, its last day) before `known`; `arrears` together with `correction`; two `arrears` lines that cover one pay for one version's change, or that disagree for one pay across versions known the same day; `for` naming something that isn't a pay; `of` naming a version not known before this one.
- `LOAD-MOVES`: `moves` naming an unknown version, one not known before this one, or a scenario version; two `moves` of one version in versions known the same day.
- `LOAD-OVERLAP`: two rows of one target that are still tied after the precedence of §4.6 at some keys, with overlapping spans: the same layer, as many stated keys, the same known date, and both or neither with a governs span. They need not state the same keys: `x[a, *]` and `x[*, b]` tie at `(a, b)`.

**Cost:** rows are grouped at load per (target, keys) and sorted by span, so a read is O(log r). **Cases:** all. Rows with their own `governs`: CB-03, CH-05, CMP-01, LH-05, LH-07, MP-09, MP-11, RC-03, RC-08, UL-05, UL-06, UL-08, UL-13, the household, change 1, change 4, change 5. `enters`: RC-08, UL-04, UL-15. `moves`: RC-01, RC-05. Index additions: change 1, change 4, change 5, change 6. `correction`: CH-04 (`coverage.md` table 1).

### 5.4 unit

`unit hour "godzina"` declares a quantity unit for `num` types (§3.2). **Refusals:** `LOAD-NAME` for an undeclared unit (a unit is a top-level id, §2.5). **Cost:** none (units are checked at load). **Cases:** CB-06, CH-02, CH-04, CH-06, LH-01, LH-02, the household.

### 5.5 axis

```
axis ul05_cut
  floor "the cut rounds down to whole złoty"
  exact "no rounding is stated"
```

An axis lists the options of a reading or a choice, each with a note (N-109, N-912). Readings of the law (which text governs, which rounding, which edge) are axes; hypothetical facts are scenarios (§6.4).
- A rule may test it: `when ul05_cut = floor`. Several axes may be tested in one `when`.
- An input may be keyed by it, and its rows for different options may sit in different versions. **The axis id in a key position** reads the input at the current option, and fans out: `selbstbehalt[dt_u_work]`, `friend_of[cmp_spouses, i]`.
- `round(x, u, a)`, `prorata` and a conversion may take it as the mode when every option is a mode name (mode names are contextual words, §2.2); otherwise `LOAD-UNIT`. An axis whose options mix modes with readings such as `exact` is tested by rules, each calling `round` with a literal mode or with none. Two axes may share option names; a bare option is resolved by its position (§2.5).

**Fan-out.** An ask evaluates every combination of the axes its evaluation reads, found at run time: a line whose result read no option of an axis prints `*` for it. It prints one line per distinct result, a tuple for an ask of several expressions (§5.16). Two holes are the same result when they carry the same questions at the same keys.
- **Naming the options.** On a line, an axis prints `*` when, with the line's other options held, every option of it gives the line's result. Otherwise it prints the options that give it, joined by ` | ` (`rate_reading = down | half-up`), and the line's `reading:` entry names the same options (§8.2). An axis printed `*` has no `reading:` entry.
- **A set that isn't a product.** When the combinations that give one result aren't one option set per axis (`(x1, y1)` and `(x2, y2)` give A, the cross pairs B), the set is split by the axes in id order: the first axis's options are grouped by the set of the other axes' combinations they give the result with, each group is a line, and each line is split again on the next axis. Every line then names a product.
- `with a = opt` fixes an axis for one ask. A pinned axis is listed under `pinned:` only, never `reading:`, since it doesn't fan out.

**Refusals:** `LOAD-FORKS` if one ask reaches more than **64** combinations (after its `with` pins), counted at load over the axes its cone reaches on the item graph, whatever its `when`s read (`ENGINE.md` §3.1 step 8). A run may read fewer; the `*` above follows what it read. This is C's cap; B had none. **Cost:** multiplies only the cone of defs that depend on the axis. **Cases:** 62 models (`coverage.md` table 1); an axis option pinned by `with`: LH-15.

### 5.6 index

```
index person = [ana, ben]
index month = month 2027-01..2059-12
index ty = year 2014..2025 from 04-06
index lot = 1..12
index bank_row: open
```

An index is an ordered set of keys.
- **Lists** are ordered as written, and versions can extend them (§5.3).
- **Ranges** are integers (with `by`), days, months and years, plain or anchored (§4.1).
- **An open index** gets its keys from the rows of inputs keyed by it (records, §5.18; things that appear later, such as a loan not yet made), in key order. A key exists from the earliest `known` of the versions whose rows bring it, as an index addition does (§5.3), so an earlier cut doesn't see it.

An index may carry `v ID` and notes, when its keys or their order are a source's (a workbook's closing age, the order a law takes parts in). It may carry `assumed STR` when its keys or its periods are the modeller's assumption (a tax year whose start no source gives); every figure keyed by it, and every result that reaches one, then lists `assumed: <index>: <reason>` in its register (§8.2).

**Bindings.** A bind is over an index, a key list or `path`, so it is bounded at load. A bound that depends on an earlier key is a `where` over a declared index: `k in inst where k <= lh_terms[l].months` (LH-10). Binds are read left to right: each bind's `where` may read the binds before it, and each `where` filters at its own bind (`c in bracket where pos(c) = pos(b) + 1`). The index `path` (1..N) exists only in an ask with `paths N`.

**Built-in index:** `place` holds every declared place (§5.9). Its keys are the place ids; a keyed place `x[k1, …]` has one key per binding tuple, written `x[k1, …]` wherever a key of `place` is expected (`opening[wcash[ike]]`). `pay` holds every declared pay (§5.13); its keys are the pay ids, and a keyed pay is one key: its instances share its rows.

**Refusals:** `LOAD-KEY` for a key that isn't in its index; `RUN-KEY` for key arithmetic outside the index. **Cost:** O(1) per key. **Cases:** all.

### 5.7 input

```
input allowance[g in agegroup]: money PLN question "Jaka jest kwota zasiłku dla tej grupy wieku?"
input born[p in person]: date
input wibor: rate nominal 12 /year
input rent_wish: money PLN prices 2026 by cpi
```

An input declares a typed figure. Its values are rows in versions (§5.3).

**Values.** A row's value is one of:
- a literal, a record or a list (`[]` included);
- a hole `? "question"`;
- `-` (not applicable);
- a literal with `assumed "reason"`: the model states a figure no source states (§4.5). Every result that reaches it lists the reason (§8.2).

**Reading.** Rows are read by the read rule (§4.6). The `question` is the text of the hole that the gap rule gives. For the household it is Polish (N-1001, N-1906).

**Refusals:**
- at load: `LOAD-UNIT` for a value of the wrong type or unit, `LOAD-PRECISION` for a money literal finer than its minor unit, `LOAD-OVERLAP`;
- per value: `RUN-NA` for a read of `-` other than a test.

**Cost:** O(log r) per read, memoised per (keys, period, K). **Cases:** all.

### 5.8 band

```
band zr(n: int): money PLN
  note "kwota zasiłku według wieku dziecka"

version rm2015
  cite rm1238
  governs 2015-11..
  known 2015-09-07
  zr: [0, 5) = allowance[u5] "§ 2 ust. 1 pkt 1 lit. a"
  zr: [5, 18) = allowance[u18] "lit. b"
  zr: [18, 24] = allowance[u24] "lit. c"
  zr: (24, inf) = ? "zasiłek nie przysługuje powyżej 24 lat"
```

A band is a function of one argument over intervals.

**One table per version.** The rows of one band in one version form one table.
- Its rows must be ascending.
- They must not overlap.
- They must meet without a gap, closed against open; `int` arguments are checked over the integers. A point interval `[a, a]` is a row.
- The first row may start at `-inf`, and the last may end at `inf`.
- A deliberate gap is a row whose value is a hole (N-709, N-710). A hole row may carry a locator.

A table that breaks these rules is refused at load (`LOAD-BAND`), and with it the whole reality (a table in a scenario file: that scenario, §8): nothing prints from an inconsistent law table, asks that never read the band included. In the store the gate refuses the amendment before it lands (`OPERATIONS.md` §3).

**Reading `b(x)`** at law date d and cut K:
1. Choose the table by the read rule (§4.6), treating each version's table as one row: its version's governs span and known date.
2. Return the row whose interval holds x.

An x outside every row is a hole. A hole x makes the read a hole with x's questions only: no row is chosen, so no row's question is added. This way a new version replaces a whole table on its span and never mixes brackets of two years.

**A row's ends.** `band_lo(b, x)` and `band_hi(b, x)` are the lower and upper end of the row of b that holds x, in b's argument type; an infinite end is `-`. They and `marginal` may be called anywhere an expression may: in a row, a def, a `when` or an ask. A row's own value may read them: they read the interval, not a value, so there is no cycle, and "10 800 zł + 32% of the excess over 120 000 zł" states 120 000 once, as the interval's end:

```
  pit_scale: (120000, inf) = 10800 + 0.32 * (x - band_lo(pit_scale, x)) "art. 27 ust. 1"
```

**Marginal rates.** `marginal(b, x)` reads a band whose rows give marginal rates, as a progressive scale or a tiered wealth table states them: the sum, over b's rows whose lower end lo is below x, of the row's value × (min(x, hi) − lo). Its unit is x's times the rows'. A hole row that x passes makes it a hole; an x outside every row is a hole; a band whose first row starts at `-inf` can't be read this way (`LOAD-BAND`).

**Rows.**
- A row's value may use the argument, inputs, other bands, its own row's ends, and any number literal: a band row is a row of a version, with its source and locator (§7.1).
- Bands are acyclic.
- A band may return any type, a key included (an age group from an age).

**Keyed bands.** `band b[k in IDX](x: T): U` holds one table per key, an axis key included, when two readings or two groups have different tables. Rows are written `b[k]: [lo, hi) = …`, the table checks run per key, and the band is called `b[k](x)`. Edges stay literals.
- In a call, k is any key expression: a def's key, a read (`rate_band[class_of[q]](x)`), or an axis id, which fans out (§5.5). The ends and `marginal` take the band with its key: `band_lo(b[k], x)`.
- Each key's table is chosen by the read rule on its own, so one key's table may sit in another version than another key's.

**Refusals:** `LOAD-BAND` (unordered, overlapping or gapped rows; `marginal` on a band from `-inf`), `LOAD-CYCLE`, `LOAD-DUP` (a band with a built-in's name, §2.2). **Cost:** O(log rows) per call; `marginal` O(rows). **Cases:** CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, the household. Keyed bands: MP-06, UL-03. `marginal`: CB-03, CB-04, CMP-01, MP-11, UL-09. `band_lo` and `band_hi`: CB-02, CB-04, CB-07, MP-06, MP-08, RC-08, UL-03, UL-10, UL-15.

### 5.9 place

```
place konto: account PLN holds vat_owed
place ike: holding etf_unit priced etf_price
place house: asset PLN priced house_value
place mortgage: debt PLN
place marek: claim PLN
place pln[p in person]: account PLN
place employer: outside PLN
```

A place is where money or units sit (N-501, spec §3.2). Kinds:

| Kind | Holds | Rule |
|---|---|---|
| `account`, `cash` | money | may go below zero, with `WARN-NEGATIVE` on the first day of each stretch below zero (N-1607) |
| `holding U priced p` | units U | balance in units. `worth(x, d)` = units at the end of day d × p at d, where p is `num CUR/U`. Balance below zero is refused (`RUN-PLACE`) unless the holding is short (N-511): a row of the built-in input `short[x] = true` allows it |
| `asset CUR priced v` | nothing that moves | `worth(x, d)` = v at d, from valuation rows (N-502). A pay to or from an asset is refused (`LOAD-KIND`) |
| `debt` | money owed, as a balance ≤ 0 | a pay that would take it above 0 is refused at that day (`RUN-PLACE`, spec §4#15) |
| `claim` | money owed to us, ≥ 0 | below 0 is refused (`RUN-PLACE`) |
| `outside` | the world beyond the reality | unconstrained; never in net worth |

- **Built-in outside places:** `outside_pln`, `outside_eur`, `outside_usd`, … exist for every currency the model uses, and `outside_<unit>` for every declared unit (`outside_etf_unit`). Every pay names both ends, so money and units are conserved by construction and the outside is explicit (C's idea).
- **A purchase or sale of units** is two pays, like an exchange: the money leg between the account and `outside_<cur>`, and the units leg between the holding and `outside_<unit>`.
- **An exchange of currencies** is two pays, one per currency, each between a place of that currency and its outside place: euros leave to `outside_eur`, złoty arrive from `outside_pln`. The rate each leg uses (spot at receipt, a period average, spot at settlement) is its own row or axis (U-04, CB-08). So legs sum to zero per currency.
- **Holds.** `holds h` names a money def (no keys, or one time key) that gives, on each day, the part of the balance that isn't ours, such as VAT collected (N-504, spec §4#24). `free(x)` is the balance minus the hold, less unconfirmed promised movements (§5.13). A state pay that takes money out of a place with a hold, and reads that place's own balance, must read it through `free` or `plan_low` (both net of the hold), not `bal`, or it is refused at load (`LOAD-HOLD`). Its reads of other places are unaffected.
- **Opening.** A place's balance at the start of the run's first day S is the row of the built-in input `opening[x]` that governs S. A place with no such row has `bal` a hole from S until its first anchor. `flows` reads movements only, so it is not a hole for want of an opening. Statements' closing balances are rows of `stated[x]`: the model never reads them by name, and each one re-anchors the place's balance from its day, with the difference shown in the residual ledger (§5.18).
- **Places need dates.** A movement is a dated event, and the ledger's invariants need its date. An illustration that gives no dates writes its money as defs (MP-14, MP-15, CMP-02); where money must move, the model's dates are `assumed`.
- **Kind and currency** can be read: `kind(x)` (a key of the built-in axis `placekind`) and `currency(x)`. Asks select places by kind or owner, never by a hand-kept list (N-1606).

**Refusals:** `LOAD-KIND`, `LOAD-UNIT` (a pay between places of different units), `LOAD-HOLD`, `RUN-PLACE`. **Cost:** a balance read is O(1) during the day walk, or O(log Mₓ) at a past date with per-place prefix sums, Mₓ the place's change points (`PERFORMANCE.md` §2.2). **Cases:** CMP-01, the household, change 5. The cases hold their money as defs keyed by their periods; only CMP-01 and the household move it between places.

### 5.10 def and rule

```
def payable[f in family, m in month]: money PLN
  note "kwota wypłacana po obniżeniu, art. 5 ust. 3"
  rule base = max(0, entitled[f, m] - excess[f, m])
  rule base v rm2016 = max(0, entitled_2016[f, m] - excess[f, m])
  rule small excepts base when entitled[f, m] - excess[f, m] < min_payment = 0
```

A def is a keyed value defined by rules. To evaluate def x at keys k, with law date d and cut K:

1. **Givens.** If a given of x (a version row aimed at x) matches k by the read rule (§4.6), return it. A `with` or `as if` pin wins over a given. A given may state a source's own legal conclusion (a guide that says these parents qualify), citing it.
2. **Rule versions.** Rules that share a label are versions of one rule. A rule with `v V` is a candidate only if V governs d and V's `known` ≤ K. Among the candidates of one label, the read rule picks one (the later known wins). A rule with no `v` is always a candidate, with known −∞.
3. **Applicability.** Evaluate each remaining rule's `when`: true means it applies, a hole means unknown, and no `when` means it applies. The `when`s are evaluated in exception order, a rule's after those of the rules that except it, and by label (§2.4) among rules that `excepts` doesn't order. A rule already beaten by an applicable rule (step 4) has its `when` skipped. So a read guarded by an exception is never reached.
4. **Exceptions.** A rule is beaten if an applicable rule `excepts` it, directly or through a chain. The chain is transitive whatever the rules in between do: if a excepts b and b excepts c, an applicable a beats c, whether b applies or not.
5. **Unknown applicability.** If a rule of unknown applicability could change the outcome, the result is a hole that carries that `when`'s questions. It could change the outcome when it would except a winner, be a co-winner, or be the only rule left. Its value isn't evaluated, so it is a hole even if that value would equal the winner's.
6. **Result.**
   - One winner: evaluate its value, and return it. Its label is kept for `explain`. **Only the winner's value is evaluated**: a beaten rule's division by zero, `RUN-KEY`, `RUN-NA` or `RUN-EMPTY` never happens.
   - Two winners: refused, `RUN-RULES` ("rules a and b both apply to x[k] on d"), even when their values are equal: the law's structure is stated with `excepts`.
   - None: refused, `RUN-NO-RULE`.

A rule's value may be a hole, meaning "the law for this case is not written", or `-` (§3.1). A def whose only rule is a hole is a stub: a law not yet written, or an open fact the source names, which prints its question.

**Law changes are added rules or rows** (N-104, N-107).
- A figure changes: add a row in a new version.
- A formula changes from a date: add a rule with the same label, pointing at a version that governs from that date.
- A new exception: add a rule with `excepts`.

**Recurrences.**
- A def may read itself, or anything that depends on it, at an **earlier key** of an ordered index (`x[k - 1]`), or at a **later key** instead (`x[k + 1]`, an annuity computed back from the last age). Within one def, the reads on one index all go one way; reads both ways are `LOAD-CYCLE`.
- Each such read must sit under a guard: a condition that tests k or `pos(k)`, in its own rule's `when`, in the `when` of a rule that excepts it, in an `if`, or in an `and` to its left (`when k > 1`, `when pos(y) > 1`, `if k = 1 then … else …`, `k = 1 or …`, `when k = sheet4`). On a period or list index, `pos(k) > n` and `pos(k) = n` are the guards for `k - n`. `LOAD-GUARD` checks that a guard exists; whether it suffices is checked per key (`RUN-KEY`). Only a recurrence read needs a guard: a shifted read of another item (`cpi[m - 1]`) is checked per key alone (`RUN-KEY`).
- **At a computed key.** A def may also read itself, or anything that depends on it, at a key given by an expression or a binding (an earlier acquisition found by `where`, a deceased spouse's own figure). Such a read isn't checked at load. At run time, a chain that returns to a (def, keys) still being evaluated is refused at that key (`RUN-CYCLE`). It terminates, since a chain can't be longer than the keys.
- Same-key dependencies form an acyclic graph (`LOAD-CYCLE`, spec §4#2). The check covers every rule, whatever its `when`, so two readings that select rules forming a cycle are split into two defs.
- **No forced rounding.** A carried value may be an exact `num`. Every exact value is capped at **4 096 decimal digits** in its numerator and its denominator; above that it is refused at that key (`RUN-DIGITS`: "round where the law rounds"). This replaces B's rule that every carried ratio must be a top-level `round`, which forced invented units such as `1e-30` (G-CH-2).

**An iteration the law names** (N-918, UL-07) is a recurrence over a bounded index, with `first` choosing the stopping key:

```
index iter = 0..50
def d[t in taxpayer, i in iter]: num USD
  rule init excepts step when i = 0 = d0[t]
  rule step = deduction_from_credit[t, i - 1]

def deduction[t in taxpayer]: num USD = d[t, stop[t]]

def stop[t in taxpayer]: key iter = first(i in iter where i > 0 and abs(d[t, i] - d[t, i - 1]) < 1 and abs(c[t, i] - c[t, i - 1]) < 1)
```

The exception is the first key: `init` applies at i = 0 and beats `step`, whose read of `i - 1` is guarded by `init`'s `when` and is never evaluated there. If no key satisfies the condition within the index, `first` is refused (`RUN-EMPTY`). An iteration can therefore never run unbounded or stop in silence. This is A's `loop … max`, written with no new construct.

**Aggregates.** An aggregate runs over bindings with `where`. Over a key binding, every aggregate is ordinary and exact, `mean`, `share` and `quantile` (type 1) included, and may appear anywhere. Over `path` they exist only in asks, where they print a standard error (§5.16).
- With no `: term`, `first` and `last` return the key (one bind) or the tuple. `argmin` and `argmax` always return the key or the tuple, typed `key IDX`. `count` takes no term: it is the number of bindings whose `where` is true, all of them with no `where`.
- `first` evaluates its `where` from the first binding in binding order and `last` from the last, each stopping at the first key whose `where` is true; those keys are what it read (§8.2).
- `min`, `max`, `argmin` and `argmax` order numbers, money, dates and keys (index order); false < true. Imprecise dates compare by §4.4: `min` and `max` give the interval of the ends' min (max), an imprecise value like any other; `argmin` and `argmax` give the key whose value is decided below (above) every other key's, the first in binding order among keys decided equal, and otherwise `HOLE-IMPRECISE`.

| Aggregate | Over no keys | With holes |
|---|---|---|
| `sum` | 0, in the term's type | a hole if any term is |
| `count` | 0 | a hole if any `where` is |
| `min`, `max` | refused, `RUN-EMPTY` | a hole if any term is |
| `min(a, b)`, `max(a, b)`, the plain form | — | a hole if any argument is |
| `any` | false | true if any term is true; otherwise a hole if any term is |
| `all` | true | false if any term is false; otherwise a hole if any term is |
| `first`, `last` | refused, `RUN-EMPTY` | a hole if a hole comes before the first key whose condition is true (after the last, for `last`) |
| `argmin`, `argmax` | refused, `RUN-EMPTY` | a hole if any term is. Ties go to the first key in binding order |
| `mean`, `share`, `quantile` | refused, `RUN-EMPTY` | a hole if any term is |

The "with holes" column covers a hole `where` as well as a hole term. A key whose `where` is a hole is an unknown member: `count` is a hole; `sum` is a hole unless that key's term is known to be 0; `min`, `max`, `argmin`, `argmax`, `mean`, `share` and `quantile` are holes; `any` and `all` follow Kleene logic; `first` and `last` keep their rule. A hole carries the questions of every operand, `where` and term it evaluated, each once (§8.1).

**Logic.** `and` and `or` evaluate left to right and stop early, with Kleene values: false `and` anything is false, true `or` anything is true, and otherwise a hole stays a hole. `if c then a else b` evaluates only the branch c takes; with a hole c it evaluates both, and is a hole unless they are equal. A guarded read such as `k > 1 and x[k - 1] > 0` is therefore safe at k = 1.

**Refusals outrank holes:** when both reach a result, the result is refused.

**A law's own rule for a missing record** (N-917, CHG-11). `missing(r)` is true when the read r is a hole because no row gives it, or its row is a stated `?`, and false otherwise; it is never itself a hole. It may appear only in the `when` of a rule that points at a version with `v`, so the law that says what a missing record means is cited (`LOAD-MISSING` elsewhere). Every result that such a rule decides, where a `missing` test in its `when` read true and the rule won, lists it in its register: "missing record counted by <version>". A `missing` test that read false lists nothing of its own. Every other missing value still gives a hole.

```
def outlier[d in day]: bool
  rule base = pnl[d] < -var[d]
  rule no_record excepts base v mar32 when missing(pnl[d]) or missing(var[d]) = true
```

**`as if`** (N-907, N-908). `e as if {pins}` evaluates e in a world where the pins hold. A pin is one of:
- `input[keys] = value`: replaces every read of that input at those keys;
- `def[keys] = value`: pins the def;
- `axis = option`;
- `law D`: the law date of every def and input read inside with no `at` and no time key (§4.3);
- `wording D for [x, y]`: the named items take only versions that entered into force by D (§4.3, §4.6);
- `known D`: the cut K inside.

**A pin's value is any expression** the ask or def could hold: `with parts[f] = base_parts[f]`, `with law forms_from`, `as if {known origin_day[o]}`. It is evaluated first, in the context without that `as if` (its own pins excluded), then pinned as a value. So a pin that reads its own target reads the unpinned value, and there is no cycle. The pins of one `as if` apply only inside it; nested pins don't leak to their siblings.

**A pin's keys** are any key expressions: an ask's `for` variables, a def's keys, an aggregate's bound variable (`adv[c, m] as if {mon_relief[c, m] = 0}`). The pins are formed per binding tuple, and the memo keys on them.

Everything inside is recomputed on the changed facts, gates included. Pays inside are computed but never booked, and `flows` inside sees the recomputed pays. The result combines with the real one like any value, by min, max, a cap or a difference. `as if` may nest 4 deep, so CB-02's "as if exempt" inside "as if the foreign income were Polish" is written once and read twice, never copied. A superseded law stays runnable inside `as if {law 2021-12-31}` (N-908).

**Refusals:** `LOAD-ASIF` (deeper than 4, or a pin on an unknown id). **Cost:** each distinct pin set gets its own memo, and only the pinned items' cone is recomputed.

**Cost of a def.** Per key, O(rules of the def) tests; a def in the cases has 1 to 6 rules. Values are memoised per cell (`ENGINE.md` §3.2): keys, law date class, axis options reached, cut class, pin set, scenario and path, and the walk point for a walk-bound def (§5.13.1). A recurrence over n keys is O(n) time and memory.

**Cases:** all. `excepts`: 67 models. Recurrences, a read at k − n or k + n: CH-02, CH-06, LH-02, LH-04, LH-05, LH-07, LH-10, LH-13, LH-15, LH-16, LH-18, MP-15, RC-01, the household. Givens: CB-10, CB-11, CB-12, CB-14, LH-13, LH-17, MP-03, MP-04, MP-06, MP-07, MP-08, RC-01, RC-02, UL-13. `as if`: CB-03, CB-09, CH-04, CMP-01, LH-04, LH-12, MP-11, RC-08, UL-14, UL-15; `with` pins on asks: 24 models. `missing()`: the household. An iteration the law names: UL-07 writes its steps as defs and UL-08 uses `solve`; the form above is a recurrence with `first`, both used elsewhere (`coverage.md` table 1).

### 5.11 fill: greedy allocation

```
fill absorb
  supply credit[sx, o] for sx in payer, o in year
  demand room[dx, t] for dx in payer, t in year
  arc sx = dx and t >= o - 1 and t <= o + 10
  order o, t
```

Supplies are matched to demands in a stated order. This covers carrybacks and carryforwards, wash-sale lot matching, pension carry-forward by vintage, loss pools and reliefs sharing one cap (N-513, N-514, N-919).

**Its parts.**
- **Units.** Supply and demand share one unit, any unit: money, `num CUR`, `num shr`, or a plain `num` with none, so abstract amounts need not be `money XXX` (§3.2).
- **`v` and notes.** A fill may carry `v ID`, the version whose law states the order (a carry-back before a carry-forward, the earliest vintage first), and notes. `explain` prints it. Unlike a rule's `v`, it doesn't gate the fill, which has one procedure and nothing to fall back to; an order that changed by law is written as an `order` term read from rule-versioned defs.
- **Sides.** `for` is optional on either side. A side without it has one value (one cap shared by every supply) and adds no keys to `taken`.
- **`arc`** is any bool expression, `true` included.
- **No law date of its own.** Each term is read at its own law date (its time key, or its `at`), else at the law date of the def or ask that reads the fill. A side's `where` is a term like any other.
- **Keyed fills.** `fill F[y in IDX]` is one fill per key. Its terms may read `F[y - 1].taken` behind a guard (§5.10 Recurrences), so a cap carried from year to year is a recurrence over fills, not a cycle. The read may also go through defs (`use[y]` → `want[l, y]` → … → `use[y - 1].taken`), under the same guard rule.

**Procedure.**
1. **Arcs.** Evaluate every (supply key, demand key) pair's `arc`, and collect those whose `arc` is true or a hole. A false arc is left out; a pair whose arc is a hole takes by the Holes rules below. A refusal in any arc refuses the fill at that key, with the pair named.
2. **Sort.** Sort the arcs ascending by the `order` tuple, compared lexicographically. Keys compare in index order, and false < true. Remaining ties go by supply keys, then demand keys, in index order. An order that can't be decided (a hole) is refused (`RUN-ORDER`).
3. **Start.** Remaining supply R_s = supply(s); remaining demand Q_d = demand(d).
4. **Take.** For each arc in order, `taken[s, d] = min(R_s, Q_d)`, and subtract it from both.

**Outputs:** `absorb.taken[supply keys, demand keys]`, a ref with the supply keys first, then the demand keys; a side without `for` adds none. It is 0 for pairs that aren't arcs, keys a side's `where` leaves out included. What remains is written with `sum` over `taken`. A keyed fill's output is `F[y].taken[…]`.

**Holes.**
- A take is 0 wherever its arc is false, or R_s or Q_d is known to be 0, whatever the other side: on a known arc as on an arc whose condition is a hole. Otherwise a hole side, or a hole condition, makes the take a hole.
- After a hole take, R_s and Q_d are holes carrying that take's questions, so an unknown later year never spoils an earlier result.
- **What a take reads:** its own `arc` and `order`, supply(s), demand(d) and, through R_s and Q_d, what the earlier takes on the same s or the same d read. Its register follows that (§8.2), so one example's assumed row doesn't reach another example's takes.

**Refusals:** `RUN-NEG` (a negative supply or demand), `RUN-ORDER`, and `LOAD-UNIT` (supply and demand must share a unit). **Cost:** O(A log A) for A arcs: 30 years of CB-05's `spouses` (a year back and ten forward, between two spouses each way) is at most 30 × 11 × 4 = 1 320 arcs; the example above has 608. A keyed fill costs that per key. **Cases:** CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13; keyed fills: LH-04, UL-13.

### 5.12 prorata

`prorata(total, w[j] for j in IDX [where c], k, u, mode, rest none | last)` or `prorata(total, w[j] for j in IDX [where c], k, u, rest largest)` is key k's share of `total`, split by the weights. The total is money or a `num CUR`. **The shares' type follows `round` (§7.2):** they are money when the total's unit is a currency and u is a whole number of minor units, and under `rest last`, whose last share is the remainder, only when the total is money too. Otherwise they are `num CUR`. So MP-12's whole-dollar shares of a `num USD` total, `rest largest`, are money.

1. `raw_j = total × w_j / Σw`. The weights must be ≥ 0 with Σw > 0; otherwise refused (`RUN-WEIGHTS`).
2. Round:
   - `rest none`: each raw_j is rounded to u with `mode`, independently.
   - `rest largest`: no mode; each raw_j is rounded down to u. The (total − Σ)/u leftover units go one each to the keys with the largest remainders, ties in index order. The shares then sum to the total. A total that isn't a whole number of u is refused (`RUN-PRORATA`: `<total> isn't a whole number of <u>`).
   - `rest last`: each key but the last is rounded with `mode`, and the last takes whatever remains (the instalment that absorbs the odd unit, N-213).

`mode` may be an axis whose options are mode names (§5.5). **`prorata` is for shares the law rounds.** An exact split is plain arithmetic, `total * w[k] / sum(j in IDX: w[j])`, as CB-13 and CB-14 write it.

**Cost:** O(n log n) once per (total, weights), memoised. **Cases:** CMP-01, MP-01, MP-09, MP-12.

### 5.13 pay

```
pay rent[m in month]
  from konto
  to landlord
  amount rent_amt[m]
  on day_of(m, 1)
  for m

pay invest[m in month]
  from konto
  to ike_cash
  amount max(0, plan_low(konto, day_of(m, 15), add_days(next_on(wage[m + 1], day_of(m, 16)), -1)) - keep)
  on day_of(m, 15)
  phase 7
```

A pay is a dated transfer between two places, with one movement per binding tuple (N-601). With `paths`, there is one per path.
- **`from`, `to`:** places of one unit. Money goes from one place to another, so direction can't be wrong (N-218).
- **`amount`:** money, or units for a holding, ≥ 0. A negative amount is refused at that key (`RUN-NEG`); write two pays, or swap the ends with a `when`. An amount of 0 books nothing, but the instance still exists: `next_on` finds it.
- **`on`:** a date. It may be imprecise (§4.4) or a hole. A pay whose date is a hole makes every `flows` over a span it might fall in a hole (G-MP-2).
- **`for`:** the period the amount is for, kept apart from the date it is paid (N-305). `flows(x, span, for k)` reads only instances for period k.
- **`when`:** the instance exists only where true; with no `when`, at every binding tuple. Its amount doesn't decide whether it exists. A hole `when` is a hole movement.
- **`phase`:** an integer 0–9 (default 5) that orders state pays within a day (§5.13.2).
- **`promised`:** the pay is expected but not certain, like a friend's promised repayment. It is booked on its day like any pay, so `bal` counts it, but `plan_low` and `free` leave it out until a record confirms it: `plan_low` drops its future instances, and both take a booked one back out of the balance they start from (N-505, spec §4#23, U-19).

#### 5.13.1 Flow pays and state pays

A pay is a **state pay** if its amount, date or condition makes a state read (the table below), directly or through defs. This is inferred, never declared (spec §3.6). Every other pay is a **flow pay**.
- Flow pays are computed in dependency order over the whole run, like a spreadsheet.
- State pays are computed in the day walk (`ENGINE.md` §4.2).

**State reads.** A def may make one unless a flow pay reaches it (`LOAD-STATE`). The walk's own forms, `bal(x)`, `free(x)` and `plan_low`, are allowed only in state pays and the defs they reach. An ask, and a def that no pay reaches (a figure several asks share), use the dated forms `bal(x, d)`, `worth(x, d)` and `flows(x, span)`.

**A def that reaches a state read is evaluated at each reader's point of the walk**: its memo cell carries the walk point (day, phase, pay, keys), `ENGINE.md` §3.2. So two state pays on one day that read one `room` def each see the balance as it stands when they run, and the second never reuses the first one's value.

| Read | Value |
|---|---|
| `bal(x)` | x's running balance at this point of the day walk: the start of the day, plus the day's records and flow movements booked so far, plus the state pays already run that day (`ENGINE.md` §4.2) |
| `bal(x, d)` | x's balance at the end of day d: in a state pay and the defs it reaches, for d before the pay's day (`LOAD-FUTURE`, `RUN-FUTURE`); in an ask or a def no pay reaches, any day of the run (§5.16) |
| `free(x)` | `bal(x)` minus x's hold (§5.9), less the promised movements booked so far that no record confirms (§5.13) |
| `plan_low(x, a, b)` | the lowest end-of-day balance x reaches on days a..b, starting from `bal(x)` less the promised movements booked so far that no record confirms (as `free` takes them out) and counting only flow pays, each future instance as known on the walk's day (records and anchors after the walk's day are never used, `ENGINE.md` §4.2): its keys (index additions, and open-index keys, §5.6), its `when`, its date and its amount all at cut min(day, K), with every draw at its plan value (§5.14). A `when` that is false drops the instance, its date unread; then a date known to fall outside (t, b], t the walk's day, drops it, its amount unread, whatever its `when` (such a date can't move the balance on the window). A hole in the `when`, the date or the amount of an instance that is left makes `plan_low` a hole, with the questions of every hole so read (§8.1): a hole date may fall anywhere in the window. State pays and promised pays are left out, so a rule never sees its own result or another rule's (N-1102, spec §4#1). The look-ahead never reads what wasn't known that day (CHG-02's risk). On a place with a hold it is the lowest end-of-day **free** balance (the balance less the unconfirmed promised movements, as above, and less that day's hold, read at the same cut; a hole hold makes it a hole), and the register lists the hold, so the look-ahead never invests money held for someone else |
| `next_on(p[keys], d)` | the date of the first instance of **flow** pay p on or after d. Naming a state pay is refused (`LOAD-STATE`, spec §4#3). With no instance left in the run, it is a hole: `no further <p> after <d>` (spec Q-17), so a rule that waits for the next pay never waits for nothing in silence |
| `worth(x, d)` | a holding's value at the end of day d, as `bal(x, d)`: its units then, times its price at d (§5.9). An asset's `worth` reads only its valuation rows, so it isn't a state read |
| `flows(x, span)` | for a place x, or a state pay x: every movement booked there, which the day walk makes |

`flows(p, span)` and `next_on(p, d)` of a flow pay p are not state reads, so a flow pay may read the history and the next date of other flow pays: an annual settlement reads the year's advances. In the flow pass `next_on` is read at the cut of the part that calls it (in `on`, at K; in `when` or the amount, at min(t, K), `ENGINE.md` §5); in the walk, as known on the walk day (`ENGINE.md` §4.2). Record rows are inputs (§5.18), so reading them isn't a state read either.

**Same-day totals.** In a state pay, a `flows` span may end on the pay's own day. It then counts what is booked up to this point of the day, as `bal(x)` does. So a yearly limit is re-checked after each posting, and a second rule on the same day sees the first one's contribution (R-08).

**Groups of places** (R-07). `plan_low` and `bal` also take a set of places, written as a binding: `plan_low(x in place where kind(x) = account and currency(x) = PLN, a, b)` is the lowest end-of-day *sum* of those places on days a..b, which isn't the sum of their lows. So room can be measured on the liquid group and taken from one account. The set's condition reads only `kind`, `currency` and place ids, so it is fixed at load. A set over places of more than one currency is refused (`LOAD-UNIT`).

**Refusals:**
- `LOAD-STATE`: a flow pay, or a def a flow pay reaches, makes a state read (through its `for`, its places' keys or its bindings' `where`, since one in its amount, date or `when` makes it a state pay); `bal(x)`, `free(x)` or `plan_low` is read where no state pay reaches it; or `next_on` names a state pay.
- `LOAD-FUTURE`: a state pay reads `bal(x, d)` for a day that isn't before its own day, or `flows` over a span that ends after its own day. When the day is computed, this is `RUN-FUTURE` instead.

#### 5.13.2 Order within a day

Each day of the walk:
1. the day's records are booked, then its flow pays' movements in pay id order, except a record that explains a state pay instance, which is booked at that instance's point in step 2 (`ENGINE.md` §4.2);
2. state pays run in order of phase, then pay id, then keys in index order.

A rule that invests "at the start of the day" and one that invests "at the end" are therefore different models, with different phases, and the difference is visible (N-318, spec §4#39).

#### 5.13.3 Booking and back pay

**The cut of an instance.** An instance's `on` is evaluated at the ask's cut K. Its `when` and its amount are then evaluated with what was known on its date t: cut min(t, K). A state pay's state reads are the exception: they read the day walk at cut K (§5.13.1, `ENGINE.md` §4.2). So a date learned later moves the instance under later asks only; under an earlier cut the instance is where it was then known, or a hole. When t is imprecise, a version's visibility at that cut is a §4.4 comparison: visible when known by the interval's first day, not visible when known after its last, and otherwise the instance's amount is `HOLE-IMPRECISE`.
- **`correction` versions** with known ≤ K count as known on t.
- **Scenario versions** are visible to the instance if t ≥ the scenario's `from` (§6.4).
- **Changes in the world.** For each other version V with t < V.known ≤ K, taken in known order (versions with one `known` together, as one step, `ENGINE.md` §5), whose contents change a **flow pay** instance's amount by δ (from 0 when the instance didn't exist at its cut and V makes it exist, `ENGINE.md` §5):
  - **`arrears on D`:** a correction movement of |δ| on D, in the pay's direction if δ > 0 and reversed if δ < 0 (N-921). D is a day, or a month or year (an imprecise date, §4.4): back pay paid over a window that ends by a date is dated by the period that holds the window, or by its last day, with a note.
  - **`arrears none`:** no movement. The law gives no back pay.
  - An `arrears` line with `for p1, p2` covers only those pays. A line without `for` covers every pay that no other line names. So a retroactive tax change can refund through the annual settlement only, with `arrears none for pit_advance` and `arrears on D for pit_settle` (§6.5, change 3).
  - **A later answer.** A line in a later version W, `arrears on D of V` or `arrears none of V`, answers the changes V made, for the pays it covers, where V's own lines don't cover them: back pay decided or announced after the change. V's own lines are read alone for this, so a line of V without `for` covers every pay that V's other lines don't name. A line `of V` that names a pay V's own lines cover is refused (`LOAD-ARREARS`); one without `for` covers only the pays V's lines leave open. Under a cut before W.known the movement is still `HOLE-CHANGED`; from W.known it follows W's line.
  - **Neither, in V or in a later version known by K:** a hole movement, `HOLE-CHANGED: "<pay> on t changed after it was paid (version V); is back pay due, and when?"`, dated on or after V.known. Every `flows` that can count the movement is a hole until the version says: one of the pay, or of a place it moves money between, over a span that reaches V.known or later. A `flows(p[keys], span)` whose keys don't hold the changed instance can't count it, and stays known. A later-known change is never silently paid or silently unpaid. This replaces B's per-pay `correct on` (judging.md §9).

- **An amount, or a `when`, that was a hole** at its cut stays a hole movement until a version answers it: a `correction` answers it at t, and a change in the world measures δ from 0 under its `arrears` line. A version under which the `when` is false answers it with 0: δ = 0, nothing is booked, and no `arrears` line is needed.
- **State pays are never re-measured.** A state pay instance is a decision taken on its day: the law and facts it reads, and its look-ahead, are what was known then (cut min(t, K)). A later version that isn't a `correction` never re-measures it, and `arrears` lines cover flow pays only. A `correction` reaches it through the walk at cut K, like any fact known on t. Its balances are the walk's at cut K, so records dated up to its day and anchors before it, seen by a later cut, can change what it computes and books (`ENGINE.md` §9).
- **Modelled-past instances are never measured either.** On a covered day an instance is computed at its booking cut for the ledger (`ENGINE.md` §6.2) and moves no money, so no version changes it by a δ.
- **Facts a look-ahead reads.** A state pay's decision rests for good on what was known on its day. A fact its look-ahead reads before that fact is known (a contract signed next month, a tax form chosen later, a loan not yet made) makes the decision a hole that nothing later answers, and every later balance of its places a hole with it, up to the place's next statement (an anchor, §5.18). So such a fact is given a default known from the start, `assumed`, in a forecast version (0, or `-` for a thing that doesn't exist yet), and things that appear later are keyed by an open index (§5.6), which has no keys until a row brings them; the later version replaces the default. The household is written so.

An instance dated after K is planned, and it is evaluated with cut K.

**A booked amount is fixed.** Once an instance is booked at its cut, its amount never changes; a later version reaches it only as a correction movement with its own date. Anything that reads the instance later, through `flows`, reads the booked amount plus the corrections dated in its span. So a settlement computed as the year's tax minus the advances booked counts a retroactive change once.

**`flows(x, span)`** is the sum of the movements into place x in the span, minus those out of x, corrections included. **`flows(p[keys], span)`** with a pay p is the sum of that pay's movements in the span, over its instances at the stated keys (all of them when no keys are given), corrections included: the history of transfers by kind (N-926). `, for k` keeps only the instances whose `for` is k (N-305). **`flows(x, span) in CUR at fx mode`** converts each movement separately, reading the rate `fx` (a `num CUR/OTHER` input) at the movement's date (on an imprecise date, at the instance's law date, §4.3), rounding each movement's amount (≥ 0) to CUR's minor unit with `mode`, and only then applying its direction, before summing (N-606, N-932). So a movement in and one out of the same size convert to the same size. A converted total of the whole span is a different number, and the model writes it as one if it needs it.

**Cost:** O(1) per instance. `flows` is O(log Mₓ) with per-place prefix sums (`PERFORMANCE.md` §2.2). One more evaluation per version that reaches a booked instance. **Cases:** CMP-01 (pays, places, conversion); the household, and change 5 adds pays (changes 3, 4, 6 and 7 change what pays read). State reads: the household, change 5. LH-14 states its `arrears` line but books no pay, so it reaches nothing; UL-09 and RC-08 are retroactive law read by defs. The undated illustrations (MP-14, MP-15, CMP-02) are defs (§5.9).

### 5.14 draw and distributions

```
draw r[p in path, y in year] ~ normal(mu, sigma) to 1e-9
draw life[p in path] ~ exponential(lam) to 0.01
```

A draw is a reproducible random value at an address (N-1401).
1. The address A is the canonical text `name[keys]|seed`. The path key is one of the keys, and the seed is the ask's `seed`. An item keyed by `p in path` (a def or a pay over a path-keyed draw) has, on path q, only its instance with p = q; with no `paths`, it has none.
2. k is the first 8 bytes of SHA-256(A), read big-endian.
3. The uniform is u = (k + 1/2) / 2^64, which is never 0 or 1.
4. The value is `quantile(D, u)`, correctly rounded to the stated unit for a continuous D, and exact for a discrete one.

Draws are inversion samples only, so a pinned uniform gives a plain number and a path can be worked by hand (N-1407).
- **The same address gives the same draw** in every scenario and every `as if`. That gives common random numbers across what-ifs (N-1302).
- **A path is a pure function of its key,** so any path can be recomputed alone.
- **`plan v`** gives the value a look-ahead uses for the draw: the household's own planning figure, such as the median spending, the mean rare cost or a stated margin (spec R-05). `plan_low` and `next_on` evaluate every future instance with each draw at its plan value, the same on every path. So a rule never sees its path's drawn future, and a parameter drawn once per path is never read by a forecast (R-04, P-16). An instance before the walk day has happened on the path, with its drawn value. A look-ahead that reaches a draw with no `plan` is refused at load (`LOAD-PLAN`).

**Working by hand.**
- `with u(r[1, 2027]) = 0.37` pins a uniform, and `with r[1, 2027] = 0.05` pins a value.
- **The canonical order of draws** is: path; then the draw's time key, ascending (a draw with no time key first; periods by their first day); then draw id (ASCII); then its other keys in index order. When an evaluator is handed fixed uniforms u1, u2, …, they are assigned to the draws an ask reaches in this order, and `explain` prints them in the same order. No sketch had an order, and all three LH-17 pairs logged that.

**The closed list of distributions.** Each has `cdf`, `pdf` or `pmf`, and `quantile` (type 1: the smallest x with cdf(x) ≥ q):

| Distribution | Parameters | Exact? |
|---|---|---|
| `normal(m, s)` | s > 0 | needs `to` |
| `lognormal(m, s)` | of the log; s > 0 | needs `to` |
| `exponential(rate)` | rate > 0 | needs `to` |
| `gamma(shape, scale)` | both > 0 | needs `to` |
| `uniform(a, b)` | a < b | exact |
| `poisson(mean)` | mean > 0 | pmf and cdf need `to`; quantile is decided exactly |
| `binomial(n, p)` | n an int ≥ 0, 0 ≤ p ≤ 1 | exact (N-220, CHG-10) |
| `negbin(mean, size)` | mean > 0, size an integer > 0. The count of failures before the size-th success: pmf(k) = C(k + size − 1, k)·p^size·(1 − p)^k for k = 0, 1, …, with p = size/(size + mean); variance mean + mean²/size | exact |
| `discrete(w[j] for j in IDX)` | w ≥ 0, Σw > 0; its values are keys | exact (a draw by lot, N-1402) |

An invalid parameter is refused at that key (`RUN-PARAM`), never NaN (N-221). A `cdf` at or below its support's lower end is exactly 0, and at or above its upper end exactly 1; a `pdf` or `pmf` outside its support is 0. Neither is refused: each is a rational on a cell edge (§7.3).

**Cost:** one SHA-256 and one quantile, about 0.3–1 µs. Discrete quantiles cost O(k). **Cases:** none of the models draws. LH-17, LH-18 and CH-06 compute their probabilities exactly, CH-03 scores a stated sample and declares numpy's stream a bound, and MP-14 and CMP-02 hold the decided lot as a row. `coverage.md` K13 and K14 keep them with their reasons. The distribution functions (`cdf`, `quantile`) are used by CH-01, CH-03 and LH-17.

### 5.15 Scores

| Built-in | Computes |
|---|---|
| `crps(normal(m, s), y) to u` | s·[z(2Φ(z) − 1) + 2φ(z) − 1/√π] with z = (y − m)/s, correctly rounded to u |
| `crps(x[i] for i in I, y)` | (1/n)·Σ\|x_i − y\| − (1/(2n²))·ΣΣ\|x_i − x_j\|, exact, with denominator 2n². Computed through order statistics in O(n log n) |
| `crps(x[i] weight w[i] for i in I, y)` | Σŵ_i·\|x_i − y\| − ½·Σ_iΣ_j ŵ_i·ŵ_j·\|x_i − x_j\|, with normalised weights ŵ = w/Σw in place of 1/n in both terms |

- Interval scores, the weighted interval score and skill scores are plain expressions over `quantile`, `abs`, `sum` and `if` (N-1502).
- **A sample has no density.** A log score is written only for a parametric forecast, as `ln(pdf(…) to v) to u`, each transcendental with its own step (§7.3). The log score of a sample or of paths is a declared bound (N-1504, CH-03 step 14).
- **`orient`.** An ask that reaches `crps` or `pdf` states `orient lower` or `orient higher` (`LOAD-ORIENT`), whatever it prints, a bound or a refusal included. Any other ask may state it, and it is printed. A score written as a plain expression (an interval score) states it by choice. A convention that varies with a reading is two asks.

**Cost:** O(n log n) per sample score. **Cases:** `crps`: CH-03; `orient`: CH-03. CH-01, CH-02 and CH-06 evaluate with exact distributions and arithmetic (backtest zones, actual-to-expected ratios), not these scores.

### 5.16 ask

```
ask q1: payable[f, m] for f in family, m in month
ask q2: share(p in path: ruined[p]) paths 1000 seed 7
ask q3: check flows(konto, 2026-03) = rec_total[konto, 2026-03]
ask q4: solve rate in [0, 0.2] to 0.0001: npv[rate] = 0
ask q5: net_tax[ewa] with foreign_income[ewa] = 0 as known 2021-06-30
ask q6: explain payable[f1, 2016-04]
```

An ask is a question. It can't book anything, since its grammar has no pay (N-1601, spec §4#19).

| Form | Prints |
|---|---|
| `expr, expr` | values, one row per binding tuple and per distinct result across the axes reached, the expressions in the order written. A body `e = f` is one bool value; a bound's id alone prints `bound: <id>: <reason>` |
| `check e = f` | `ok`, or `differs: e = …, f = …` and the difference e − f; a hole or refused side prints in its place (`?` with its questions, or `refused: …`), with the other side's value beside it (`e = 120.00, f = ?`). Sides read at different law dates list each (`law date: e 2019-01-03, f 2026-10-18`). Used for replay against records and for stated preconditions (from A; B-fresh's G-CH-5) |
| `solve v in [lo, hi] to u: f = g` | the grid point nearest the root of F(v) = f − g (below). v is a ref and may name keys (`solve rate[loan_a] …`): only that cell is pinned |
| `explain ref` | the derivation: every rule of each def reached, with its label and its state (won; beaten, by whom; not applicable; unknown, when its `when` is a hole; a rule that wins with a hole value is `won`, and its value is the hole), rows with their versions, sources and locators, the law date and cut, givens, assumptions, holes, pins and draws, in the canonical order. Of a walk-bound def (§5.13.1), the cell at each walk point where a state pay read it, in walk order, each with its point (day, phase, pay); with none, that nothing read it at those keys |
| `inputs-of ref` | the set of rows its evaluation read, with their sources (N-1705), the rows an `at` reads included; a rule's `v` version isn't a row (`explain` shows it). For `marginal`, the rows it sums (lo < x). Across an axis it fans out like any result, one set per line (§5.5) |
| `residuals x span [by f]` | the residual ledger of place x over the span: the model at the ask's cut against the records, grouped by month and pay, or by field f of the movement input (`ENGINE.md` §6.3) |

An ask may make every state read of §5.13.1 at any day of the run: `bal(x, d)`, `worth(x, d)`, `flows`.

**Options.**
- `for` binds keys.
- `with` is `as if` around the whole ask (§5.10).
- `as known D` sets the cut K.
- `paths N seed S` runs N paths.
- `orient` gives a score's direction.

**Literals.** An ask allows every literal a def allows, plus its own: `paths`, `seed`, pins, a `solve` bracket and step, a `quantile` level, and date and period literals and spans (`flows(konto, 2026-03)`, §7.1).

**Bounds in asks.** A bound may appear anywhere in an ask, inside arithmetic too (`ruin[s] - rp2000_exact`). A result that reaches it prints `bound: <id>: <reason>` (§5.17).

**Ask-only functions** (`LOAD-ASKONLY` elsewhere, N-1409). Over `path`, the aggregates below print a standard error; over keys they are ordinary aggregates allowed anywhere (§5.10).
- `share(p in path: c)` prints "n of N" and the fraction, with its standard error √(s(1 − s)/N) correctly rounded to 1e-6. Under it comes the engine's verdict on whether "n in N futures" may be said (spec Q-03, `ENGINE.md` §8): `sayable`, or `not sayable` with the reasons. The rule reads the standard error, the horizon and the **carry**, the money-weighted years that the term's movements rest on `held` or `assumed` figures (§8.2); it says `sayable` when two standard errors are at most 0.05 (the test's standard error is Agresti–Coull's, taken at (n + 2)/(N + 4) over N + 4 paths, so a share of 0 or 1 at small N doesn't pass), the carry is at most 2 years, and no pin on a draw (the test mode, §7.4) is in the register. `mean` over paths prints the carry and the horizon beside its standard error, and `quantile` beside its value, with no verdict.
- `mean(p in path: e)` prints the mean with its standard error s/√N, s² the sample variance over N − 1, correctly rounded to 1e-6.
- `quantile(p in path: e, q)` is type 1 on the paths. It prints no standard error.
- `real(x, d, base, idx)` is x × idx[base] / idx[d]: the deflator as a lens, applied only when a question is asked (N-718, N-1604).
- `solve`.

**`solve`.** v is an input or a def, pinned at each step; the pin replaces v's own row while the ask runs, a hole row included. F(v) = f − g. The grid is lo + i·u, and lo and hi must lie on it. The bracket's literals and u take v's type, so a rate is solved over rates. A hole in F at any point the solver evaluates makes the result a hole carrying F's questions; `RUN-SOLVE` is only for a known F. The pin of v isn't listed in the register, since v is the question (`pinned:` is for `with` pins), and the result prints plain: a grid point is exact.
1. If F(lo) = 0 the answer is lo; if F(hi) = 0 it is hi. Otherwise F(lo) and F(hi) must have opposite signs, or the ask is refused (`RUN-SOLVE`: "no sign change on [lo, hi]"). A root outside the range is therefore never reported as the range's edge.
2. a = lo, b = hi. While b − a > u: m = a + u·⌊(b − a)/(2u)⌋. If F(m) = 0 the answer is m. Otherwise replace whichever of a and b has F of the same sign as F(m) by m.
3. Now b = a + u. Evaluate F at c = a + u/2. If F(c) = 0, or F(c) has the sign of F(a), the answer is b; otherwise a.

The result is the grid point nearest the root, with a tie going up. A rounding the source states ("to the nearest cent") is then a `round` of that result. That is the question the source asks, not "the largest cent under the level", which is what A's and C's LH-17 models asked by mistake (19 cells one cent low). `solve` may also appear in a def rule that points at a version (`v`) whose source names the method: a gross-up the law solves, N-217. It is then the rule's whole value (§2.3 `solvex`), its bracket's literals are allowed (§7.1), and its pin of v holds while the rule's value is evaluated, at the rule's keys and law date.

**Output.**
- **Lines.** An ask prints one line per binding tuple and per distinct tuple of its expressions' results across the axes reached (§5.5). Lines are ordered by binding tuple in binding order, then by the axes' options in declaration order, the axes taken in id order (§2.4). On each axis a line sorts by the first option it names: a set (`down | half-even`) at its first option's place, and a `*` before every option. A tie goes to the next axis. A line's register is the union of what its expressions read; `explain` gives each expression's own.
- Money prints with its minor-unit digits and its currency (`6138.00 USD`).
- A `num` prints exact, as the shortest terminating decimal (§2.4: `6536`, `2.7`, a rounded `num` included) followed by its unit (`6536 PLN`), or as `a/b` when it doesn't terminate. Beside an `a/b`, and beside a terminating decimal of more than 15 significant digits, comes a 15-significant-digit display, rounded half to even: a reading aid, never the value.
- A rate prints as a decimal with its kind and basis (`0.076 effective /year`), by the same rule, never as `%`.
- A value that rests on a correctly rounded transcendental prints `≈` and its error bound (§7.3).
- A hole prints `?` with all its questions, each once, sorted by code point (as ids are, §2.4). A refusal prints `refused: CODE message`. A bound prints `bound: <id>: <reason>`, the reason being its `because` string (§5.17).
- `-` prints `-`. An imprecise date prints as its period (`2020-07`); an interval no comparison decided is `HOLE-IMPRECISE` (§4.4).
- Under each result come the assumptions, givens, holds, bounds and pins it reached (the register is transitive, N-713).

**Cost:**
- An ask costs its cone.
- A path ask costs N times the draw-dependent cone, plus O(N log N) for statistics.
- `solve` costs ⌈log₂((hi − lo)/u)⌉ + 3 evaluations of F's cone.
- `as known` costs one evaluation per distinct cut reached.

**Cases:** all. `solve`: LH-10, LH-11, LH-17, MP-07, UL-08. `check`: CB-01, CB-03, CB-05, CB-08, CB-09, CH-02, CH-04, CH-05, CH-06, CMP-01, CMP-02, MP-02, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, UL-03, UL-04, UL-08, UL-11, UL-13. `explain`: CB-07, the household; `inputs-of`: CMP-01; `residuals`: the household. Path statistics: none (§5.14).

### 5.17 bound

```
bound eventual_ruin "ruin over an infinite horizon from paths" because "a path is finite (N-1408); the closed form is asked as a value"
```

A bound declares a step out of reach (N-2008).
- A def, pay or fill that reads it is refused at load, with its reason (`LOAD-BOUND`). The step is never approximated in silence.
- An ask may read it anywhere, inside arithmetic too. A result that reaches it is `bound: <id>: <reason>`, the reason being the `because` string; the first string describes the step, and `explain` prints it. It propagates like a hole, and ranks between them: a refusal outranks a bound, and a bound outranks a hole. Every argument of a call is evaluated, so a known invalid parameter beside a bound argument is refused (`RUN-PARAM`). A bound, like a hole, takes the type of its position.
- Every result whose ask reaches a bound lists it.

**A hole or a bound?** A hole is a fact that a row could give: a figure the source doesn't state, an election not yet made. It is a `?` row, a def whose rule is a hole, or an input with no row, read through the gap rule (§4.6). A bound is a step that no row can give: a computation the language can't do (a numerical integral, a minimum over every schedule of transfers), or data that can't be held, such as a foreign program's random stream.

**Cost:** none. **Cases:** LH-17 (the ruin figures integrated over a mortality table the case doesn't give), CH-03 (numpy's stream), MP-15 (the fewest transfers, a hard problem); `coverage.md` lists every model's.

### 5.18 Records

```
index bank_row: open
input bank[r in bank_row]: movement PLN (cat key category)

version mbank_2026_03
  cite mbank_export
  governs 2026-03
  known 2026-04-02
  covers konto, savings
  bank[r20260305a] = (2026-03-05, konto, -234.22, -, groceries) "BIEDRONKA 1234 KRAKOW"
  bank[r20260310a] = (2026-03-10, konto, 7412.30, wage[ben_bolt, 2026-03], salary) "BOLT SP Z O O WYNAGR 03/2026"
  stated[konto] = 10214.55 governs 2026-03-31 "saldo końcowe"
```

A record is a real movement from an import: a bank row, a broker's fill (N-1201). Records are rows of a **movement input**, keyed by an open index.
- **The built-in fields** come first, in this order:
  - `on`: a date;
  - `place`: a key of `place`;
  - `amount`: money of the place's currency, signed, positive into the place;
  - `explains`: the pay instance the record realises, written `pay[keys]`, or `-` when nothing in the model explains it.
- **The model's own fields** follow, such as a category or a counterparty.
- **Only confirmed links are written.** A match the importer proposes stays outside the reality until it is confirmed (N-1207, `OPERATIONS.md` §2).
- **The importer's rules are data too.** Four built-in inputs hold them, each row with its source: `account_place[a]` and `party_place[c]` map the bank's account and counterparty ids (keys of the built-in open indexes `bank_account` and `bank_party`) to places; `match_days[p]` and `match_amount[p]`, keyed by the built-in index `pay`, give each pay's matching window and tolerance. The model never reads them; the importer does (`OPERATIONS.md` §2.2).

**Coverage.** `covers x, y` on a version says that its records are complete for those places over its governs span: a statement for March covers the account for March. A version that covers needs a governs span (`LOAD-COVERS`), and each of its records must be dated inside that span (`LOAD-COVERS` otherwise: an October row in a statement that covers September is refused, never booked or dropped in silence). On a covered day before the cut K:
1. the records of that place and day are booked;
2. the model's pays that touch the place are the **modelled past**. They are computed and compared with the records (`ENGINE.md` §6), and never booked (N-1202).

A record is booked as follows:
- **Explained, one end covered:** it is booked as that pay instance's movement, between the pay's two places, with the record's date and amount.
- **Explained, both ends covered:** the two records that explain it are its two legs, and it books once. Legs that disagree print `WARN-LEGS`.
- **Unexplained:** it is booked between its place and the outside place of its currency.

On days that no version covers, and from K on, the model's pays are booked: the plan.

**Stated balances re-anchor the walk.** Rows of the built-in input `stated[x]` give a place's end-of-day balance, from a statement, on the day D they govern (a row whose span is longer than a day anchors at its last day). The model never reads them by name. In a view whose cut sees the row, it is an **anchor**: x's balance at the end of D is the stated one, and later balances build on it (`ENGINE.md` §4.2, §6.1). So **a balance means what the bank states at the last anchor, plus what is booked since**; before a place's first anchor, its opening plus what is booked since.
- **The difference is never hidden.** An anchor isn't a movement: nothing is booked and `flows` doesn't count it. The difference, stated − booked, where booked is the last anchor (or the opening) plus the movements booked since, is the statement's balance line in the residual ledger (`ENGINE.md` §6.3). Each `stated` row gives its own line, so drift points at its period. And every figure that reads x's balance from D on lists the anchor in its register (`anchor:`, §8.2).
- **Holes.** A hole movement dated on or before D keeps x's balances before D holes, and its questions stay listed, on those balances and on the balance line, which is a hole too. It no longer makes x's balances after D holes. A movement that may fall after D (a hole date, or an imprecise date whose period runs past D) still does, from the first day after D on which it may fall. A hole movement's other place is anchored only by a `stated` row of its own.
- **Decisions before an anchor stay as booked.** The anchor applies at the close of D, after D's state pays, so it never changes them. A state pay after D reads the anchored balance, under every cut that sees the row, even one known after the pay's day (§5.13.3).
- **The read rule decides who sees it.** A view whose cut is before the statement's `known` date doesn't see the row, so it doesn't re-anchor (§4.6). Like a record, an anchor applies only on a day before the cut K.
- A place with no `opening` has a hole balance (§5.9) until its first anchor, and a known one from it.

**Reading history.** Record rows are ordinary rows, masked at K like any other (N-1219), and read at any law date: their version's governs span says what the statement covers, not when its rows apply (§4.6).
- A def may aggregate them, so an estimator over history up to K is a def (N-1212, CHG-12): `sum(r in bank_row where bank[r].cat = groceries and year_of(bank[r].on) = y - 1: -bank[r].amount)`.
- **`covered(x, span)`** is true when versions known by the cut cover place x on every day of the span, and false otherwise; it is never a hole. A def that aggregates a movement input over a span must guard the aggregate with it, in its rule's `when` or an `if` (`LOAD-COVERED` otherwise), so a month with no statement is a hole with its question, never a sum of nothing read as 0:

```
def spend_month[c in category]: money PLN
  rule budget = spend_budget[c]
  rule learned excepts budget when covered(konto, spend_window) = sum(r in bank_row where bank[r].cat = c and month_of(bank[r].on) = spend_window: -bank[r].amount)
```

Until a statement covers the window, the household's budget stands; a rule with no fallback would give a hole rule in its place.
- A corrected record, or a card hold replaced when it books, is a later version restating the same key. The record keeps its id (N-1207).

**Refusals:**
- `LOAD-COVERS`: a `covers` without a governs span, naming something that isn't a place, or holding a record dated outside its span.
- `LOAD-COVERED`: a def aggregates a movement input over a span without a `covered` guard.
- `LOAD-UNIT`: a record amount in another currency than its place.
- `RUN-RECORD`: a record explains an instance that doesn't touch its place, or an instance that doesn't exist.

**Cost:** the records of a K class are sorted once, O(R log R), and booking is O(1) each. **Cases:** the household and its replay (spec §3.7, G2, G4). None of the 79 cases replays bank rows; their history (LH-15's market series, LH-13's earnings) is ordinary input rows.

---

## 6. Files, versions and amendments

### 6.1 A reality is a set of files

A model is one or more `.tally` files in one directory tree. Each file starts with one of three headers:
- **`model ID`**, then `tally N` and the clauses of §5.1. Exactly one file has it (`LOAD-HEADER` otherwise).
- **`part ID`**, then `tally N`, and `law` for a **law part**. It holds sections only. A law part holds an act's sources, inputs, bands, defs and pays, the versions of its figures, and the `check` asks its gate runs (M-07); the gate of `OPERATIONS.md` §3 applies to it, and to nothing else. A law part whose defs are stubs (a hole rule) passes the gate at once and gives its holes.
- **`scenario ID`**, then `tally N`, `from DAY` and optionally `over ID` (§6.4). Its sections exist only in that scenario.

Every file declares the same `tally N` (`LOAD-VERSION`).

**The meaning of a reality is the set of its items.** The order of files, and of sections within a file, changes nothing. An item lives wholly in one file, so a def and all its rules are in one place. Splitting is a convention for readers; the engine checks only the headers, `law` included. For the household the layout is:

| File | Holds |
|---|---|
| `reality.tally` | the header, the indexes of parties, the axes |
| `law/<country>/<act>.tally` | one act, a law part: its sources, inputs, bands, defs and pays, the versions of its figures, and its `check` asks |
| `people.tally`, `places.tally`, `pays.tally` | the household's structure |
| `facts/<year>.tally` | the household's own versions: births, jobs, contracts, choices |
| `records/<source>/<period>.tally` | imported records (§5.18) |
| `scenarios/<id>.tally` | one scenario per file |
| `asks/<topic>.tally` | saved questions |

A law used by several realities is **copied in, never linked**. A reality's hash covers every file, so a result can always be reproduced from the reality alone. A case model is usually one file.

### 6.2 A change adds; it never edits facts or law

Every change to the reality's facts and law is a set of **added lines**: new sections (most often one version), new rules inside an existing def, new notes. A version, once confirmed, never changes what it states and is never removed: the only edit that touches it is a `restate` (a locator's typo, a note, its move to another file), which changes no result. Because nothing is edited, the old view stays runnable: an ask `as known D`, with D before the change, sees the reality as it was (N-705).

Four kinds of change exist (`OPERATIONS.md` §4):
- **add:** added lines only. This is every common change, and every fact and law.
- **advance:** the header's `on` moves forward, and nothing else changes. It changes results, since `on` is the default cut, so it is not a restate: its plan shows what time passing did, the plan that became the past. An add that needs a later `on` (a version known after it) follows an advance.
- **restate:** an edit that changes no result, such as a locator's typo, a note, or moving items between files. The operation proves it by comparing every result.
- **revise:** an edit of the model's structure (a def, a pay, a place, an ask) that does change results, because the model was wrong, not the world. It needs a reason. Versions can't be revised. The store keeps the old text, so the reality as it was recorded can still be run (record time R, `ENGINE.md` §2).

| The change | What is added |
|---|---|
| a new figure, or a figure that changes | a row in a new version |
| a new person, job or loan | an index addition in a version, and the rows about it |
| a new account | a `place` section (§5.9) and its `opening` row in a version; a place keyed by an index (`konto[p in person]`) gains one with each key the index gains, and that one needs its `opening` row too |
| a law that changes from a date | a version governing from that date, holding its rows, its band tables, or both; and rules with `v` pointing at it if a formula changes |
| a law that changes retroactively | the same, with a `known` after the start of `governs`, and `arrears on D` or `arrears none` (without either, booked pays it changes are holes, §5.13.3) |
| back pay decided after the change | a version with `arrears on D of V` or `arrears none of V` (§5.13.3) |
| a commencement postponed, or a sunset extended | a version with `moves V to SPAN` (§5.3) |
| a corrected record | a version marked `correction` |
| something that ends | rows that make it 0 or `-` from its end |
| a renamed thing | a new item, and an end to the old one |
| a what-if | a scenario file (§6.4) |

**A new thing no law handles is never a silent 0** (M-22). A new person, job, loan or child is rows, and the law finds it by query; nothing is expanded. Where the law has nothing for it, the reads say so: a def with no rule that applies at the new keys is refused there (`RUN-NO-RULE`, §5.10); an input with no row for them is a hole with its question (the gap rule, §4.6); a rule written as a hole prints its question. An aggregate over the index that meets such a key is refused or a hole with it (§5.10), never a total that skips the key. So the new thing is either computed by a rule that cites its law, or it is reported with what is missing.

**A closed-world row is restated when the world grows.** A row that leaves keys unstated, or states `*`, covers only the keys its index had on the row's `known` date (§4.6). A default stated in 2026 ("no one else is a parent", "everyone lives in Poland") therefore doesn't silently cover a child added in 2029. The child's version restates it, in one line, or the reads are holes that name what is missing.

### 6.3 How amendments compose

1. **Order doesn't matter.** The read rule orders rows by their own dates, not by the order they were added. Two adds applied in either order give the same reality, once `on` has reached both their `known` dates (an add known after `on` waits for its advance, §6.2).
2. **Conflicts are found at load.** Two amendments that each load alone can still be refused together, by a load check that reads items of both. The common kinds:
   - two additions with one id (`LOAD-DUP`);
   - two rows tied after precedence (`LOAD-OVERLAP`);
   - two rules with one label and one `v` (`LOAD-DUP`);
   - two versions known the same day whose `arrears` lines disagree for one pay (`LOAD-ARREARS`), or that both move one version (`LOAD-MOVES`).
3. **Text merges cleanly.** The canonical sort puts each added version at its own line: versions sort by `known` date, then id. Rules sort by label, then by their version's `known`. Two amendments that touch different items therefore merge without a textual conflict. Two edits (`restate`, `revise`) of one line meet at the store's first step instead (`OP-BASE`, `OPERATIONS.md` §1).
4. **Outcomes are compared by `plan`.** `plan` loads the reality with and without an amendment and prints which ask results and which movements change (`OPERATIONS.md` §7).

### 6.4 Scenarios

```
scenario ben_stops
  tally 1
  from 2031-01-01
  note "Co jeśli Ben przestanie pracować od stycznia 2031 roku?"

version ben_stops_salary
  cite household
  salary[ben_bolt] = 0 "założenie scenariusza"
```

A scenario is a named layer over the reality (N-1301). It changes nothing in the reality.
- **Its items.** Its file may hold versions, index additions (a hypothetical child), and new sections: places, inputs, defs, pays, draws, indexes, asks. A scenario version may also be the version that a rule of the reality points at with `v`: an act that isn't law yet runs only in a scenario (N-931, L-24). Pointing is not a read: outside the scenario such a rule is never a candidate, and inside it the rule is a candidate where its version governs d, with no `known` for K to mask (§5.10 step 2). Any other read of an item declared in a scenario file is allowed only inside that scenario and the scenarios over it (`LOAD-SCENARIO`).
- **`from D`.** The scenario changes nothing before D.
  - Its versions have no `known`, `arrears` or `correction` line (`LOAD-SCENARIO`). They aren't knowledge, so the cut K doesn't mask them.
  - Their default governs span is `D..`, and a row whose span starts before D is refused (`LOAD-SCENARIO`).
  - **They are visible by date.** An evaluation made for a day t sees them only if t ≥ D: a pay instance dated t, and a state pay's look-ahead on walk day t. An ask sees them through their governs spans, which start at D.
  - So a pay dated before D can't see the scenario, and neither can a look-ahead from a day before D, whatever K is. A forecast as known today sees the scenario from D on, and a counterfactual past ("what if Ben had stopped in 2025") books from D.
  - After a run, the engine checks that every movement and balance before D equals the base run (`ENGINE.md` §9).
- **`over s0`** stacks the scenario on another. Stacks are at most 4 deep and acyclic (`LOAD-SCENARIO`).
- **Reading.** Inside a scenario, the read rule (§4.6) first prefers the row from the higher layer: this scenario over the one it stacks on, and any scenario over the reality. Only then do the usual steps apply. What a scenario states therefore wins on its span, even against reality rows added after it was written.
- **Asking.** `ask q: … with scenario s` evaluates the whole ask in s. Inside an expression, `x as if {scenario s}` is x in s, so `net[2040] as if {scenario ben_stops} - net[2040]` is the difference. Draw addresses don't name the scenario, so the paths of a scenario and its base are paired (common random numbers, N-1302).
- **Staleness.** The reality moves on weekly, and a scenario is always read over the current reality, so it rebases by itself (N-1303). A scenario is flagged `WARN-STALE` when a reality version known after the scenario file was last changed writes a target, at a key and span, that the scenario also writes: the scenario now overrides something newer than itself.
- **Promotion** copies a scenario's versions into the reality with a `known` date of the day of promotion, and with `governs D..` for a version that has no span of its own, so every row keeps the span it had in the scenario (`OPERATIONS.md` §5).

**Cost:** a scenario's layer is one more candidate per read of a target it writes. Its run shares every memo outside the cone of what it writes (N-2004). **Cases:** the household's change 7. No case model uses a scenario: their options are readings of the law, so they are axes (§5.5), and their what-ifs are `as if` pins.

### 6.5 The seven common changes

The base is an excerpt of a household reality:

```
index job = [ana_acme, ben_bolt]
index loan = [house]
index month = month 2027-01..2059-12
index person = [ana, ben]

place konto[p in person]: account PLN

input born[p in person]: date
input loan_terms[l in loan]: (principal money PLN, months int, kind key instalment)
input overpay[l in loan, m in month]: (amount money PLN, choice key overpay_choice)
input parent[c in person, k in person]: bool
input pit_free: money PLN
input pit_rate[b in pit_bracket]: num
input residence[p in person, m in month]: key country
input salary[j in job, m in month]: money PLN question "Ile wynosi pensja brutto z tej pracy w tym miesiącu?"
input works_for[j in job]: key person

def pit[p in person, y in tax_year]: money PLN
  rule base = round(pit_scale(taxable[p, y]), 1, half-up)
  rule abroad excepts base when residence[p, month_of(start_of(y))] != pl = ? "Podatek nierezydenta Polski nie jest opisany."

pay wage[j in job, m in month]
  from outside_pln
  to konto[works_for[j]]
  amount net_wage[j, m]
  on day_of(m, 10)

version home_2026
  cite household
  known 2026-10-19
  born[ana] = 1990-04-02 "dowód osobisty"
  born[ben] = 1988-11-30 "dowód osobisty"
  parent = false "brak dzieci"
  residence = pl governs 2027-01.. "meldunek w Krakowie"
  works_for[ana_acme] = ana
  works_for[ben_bolt] = ben
```

Defs and pays range over `person`, `job` and `loan`, so a new key is picked up wherever it belongs. Each change below is one added block. **No line of the base is edited in any of them.** Each one known after the base's `on` follows an `advance` of `on` to its `known` date (§6.2): a change of its own, not counted in the table below.

**1. A new job.** Ben moves to a new employer from March 2028.
```
source cogs_contract "Umowa o pracę Bena z Cogs sp. z o.o., 2028-02-10"

version ben_cogs_job
  cite cogs_contract
  known 2028-02-10
  job + [ben_cogs]
  salary[ben_bolt] = 0 governs 2028-03.. "wypowiedzenie; ostatnia pensja za luty"
  salary[ben_cogs] = 0 governs 2027-01..2028-02 "przed zatrudnieniem"
  salary[ben_cogs] = 8200 governs 2028-03.. "§ 3"
  works_for[ben_cogs] = ben
```
The new row for `ben_bolt` is known later than the base row, so it wins from March. The wage pay, the employer's tax advances and anything summed over Ben's jobs pick up `ben_cogs`. The zero row gives the new job a salary for the months before it starts, so the wage instances there are 0, not holes (the gap rule, §4.6).

**2. A law change for the coming year.**
```
source pit_2028 "Ustawa z 2027-11-20 o zmianie ustawy o PIT, Dz.U. 2027 poz. ..."

version pit_2028_figures
  cite pit_2028
  governs 2028..
  known 2027-11-20
  pit_free = 36000 "art. 1 pkt 3"
  pit_rate[low] = 0.11 "art. 1 pkt 4"
```
Only the changed figures appear. Every other figure stays in force from its own version, because rows are open-ended (§4.6). Nothing booked before 2027-11-20 is for 2028, so there is no back pay to state.

**3. A retroactive law change.** In June 2028 the new figure is made to apply to 2027 as well, with refunds in the 2028 return. The base's pays `pit_advance` (monthly) and `pit_settle` (the annual return) aren't shown above.
```
source pit_2027_retro "Ustawa z 2028-06-15, art. 2: stosuje się od 2027-01-01"

version pit_2027_refund
  cite pit_2027_retro
  governs 2027
  known 2028-06-15
  arrears none for pit_advance
  arrears on 2029-04-30 for pit_settle
  pit_free = 36000 "art. 2"
```
- The advances of 2027 are not recomputed.
- The 2027 settlement, booked on 2028-04-30 with the old figure, gets a correction movement of the difference on 2029-04-30 (§5.13.3). Booked advances keep their booked amounts, so the change in the year's tax is counted once.
- `ask … as known 2028-06-14` still shows the old view.
- Without the `arrears` lines, every `flows` over those pays' accounts from 2028-06-15 on is a hole asking whether back pay is due.

**4. A child is born.**
```
source birth_ola "Akt urodzenia Oli, USC Kraków"

version ola_born
  cite birth_ola
  known 2029-05-14
  person + [ola]
  born[ola] = 2029-05-12
  parent = false "brak innych relacji rodzicielskich"
  parent[ola, ana] = true
  parent[ola, ben] = true
  residence[ola] = pl governs 2029-01.. "rezydencja za cały rok urodzenia"
```
- The base's `parent = false` covers only the keys of 2026, so it is restated for the new world (§6.2). Without that line, `parent[ana, ola]` would be a hole.
- Benefits and reliefs keyed by person that read `parent` and `born` pick Ola up. One that needs a fact not stated yet, such as the date a benefit was claimed, is a hole with its question.

**5. A move abroad, with a job there.** Ana moves to Berlin in July 2030 and is paid in euro.
```
place konto_eur[p in person]: account EUR
input salary_eur[j in job, m in month]: money EUR question "Ile wynosi pensja w euro z tej pracy w tym miesiącu?"
pay wage_eur[j in job, m in month]
  from outside_eur
  to konto_eur[works_for[j]]
  amount net_wage_eur[j, m]
  on day_of(m, 25)
  when salary_eur[j, m] > 0

version ana_berlin_job
  cite household
  known 2030-05-01
  job + [ana_berlin]
  opening[konto_eur[ana]] = 0 "konto w euro, otwarte przed przeprowadzką"
  opening[konto_eur[ben]] = 0 "brak konta w euro"
  residence[ana] = de governs 2030-07.. "wymeldowanie; umowa najmu w Berlinie"
  salary[ana_acme] = 0 governs 2030-07.. "wypowiedzenie"
  salary[ana_berlin] = 0 governs 2027-01.. "pensja w euro, nie w złotych"
  salary_eur = 0 governs 2027-01.. "brak pensji w euro"
  salary_eur[ana_berlin] = 5200 governs 2030-07.. "Arbeitsvertrag, § 4"
  works_for[ana_berlin] = ana
```
- A job in another currency needs its own place, input and pay, since money in two currencies is never added (N-202). These are three added sections; nothing is edited. The pay's `when` and the keyless `salary_eur = 0` keep the other jobs and the months before July free of instances; `salary[ana_berlin] = 0` gives the Polish wage pay its figure at the new key; the `opening` rows give the new accounts a balance (§5.9).
- `net_wage_eur` is a def of the German law part, which isn't written yet. A name no file declares is refused at load (`LOAD-NAME`), so the change adds the law part as a stub, and every result that needs it is a hole with its question:
```
part de_employment
  tally 1
  law

def net_wage_eur[j in job, m in month]: money EUR
  rule stub = ? "Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane."
```
- The Polish `pit` for Ana from 2031 shows "Podatek nierezydenta Polski nie jest opisany": the rule `abroad` reads her residence in the January of each year, and she moves in July 2030. When the German law is written, its rules are added to the part, each excepting `stub`, and pass the gate (`OPERATIONS.md` §3).

**6. A new loan, and an overpayment on the first.**
```
source car_loan "Umowa kredytu samochodowego nr ..., 2028-04-02"
source stmt_2028_05 "Wyciąg z rachunku kredytu hipotecznego, maj 2028"

version car_loan_terms
  cite car_loan
  known 2028-04-02
  loan + [car]
  loan_terms[car] = (40000, 48, equal) "§ 2"
  overpay[car] = (0, -) "brak nadpłat, o ile wyciąg ich nie pokazuje"

version house_overpay_2028_05
  cite stmt_2028_05
  known 2028-05-20
  overpay[house, 2028-05] = (10000, shorter) "nadpłata 10 000 zł, skrócenie okresu"
```
The schedule defs are recurrences over the months of each loan. They read `overpay` and its choice, so the instalment stays and the term shortens. The car loan states its own "no overpayment" row, because the base's row covered only `house`.

**7. What if one partner stops working.** This is the scenario file of §6.4: seven lines in a new file. Every ask can be run in it (`with scenario ben_stops`), or compared with the base inside an expression.

| Change | Lines added | Lines edited | Places touched |
|---|---|---|---|
| 1. New job | 9 | 0 | one source, one version |
| 2. Law for next year | 7 | 0 | one source, one version |
| 3. Retroactive law | 8 | 0 | one source, one version |
| 4. Child born | 10 | 0 | one source, one version |
| 5. Move abroad, job there | 25 | 0 | three sections, one version, one stub law part |
| 6. Loan and overpayment | 12 | 0 | two sources, two versions |
| 7. Partner stops working | 7 | 0 | one new scenario file |

---

## 7. Numbers, rounding, randomness and determinism

### 7.1 Exact numbers and the literal rule

**Every number is exact.**
- Money is a whole number of minor units.
- `num` and `int` values are exact rationals.
- `+`, `-`, `*`, `/` and comparisons are exact.
- `pow(x, n)` with an integer n is exact.
- Division by zero is refused at that key (`RUN-DIV`).
- A value whose numerator or denominator passes 4 096 decimal digits is refused at that key (`RUN-DIGITS`). The digits are counted on the reduced fraction. The cap is there for cost, not precision: unbounded growth would make one key cost more than the run (`PERFORMANCE.md` §6). A long product of market data, such as 720 monthly ratios, can reach it. A rounding no source states, forced by the cap, is the modeller's reading: it is written in the def's note, and `explain` shows the `round`.
- There is no floating point anywhere in a result (N-219).

**The literal rule.** In defs, rules, pays, fills and draws, the only number literals allowed are:
- the integers 0–31, 100, 360, 365, 366 and 1000, and their negatives (`add_days(d, -1)`: the rule counts magnitudes);
- a rounding unit, in the unit position of `round`, `to` or `prorata`;
- key literals in key positions (`x[2025]`, `k = u5`, `i = 0`);
- the bracket of a `solve` in a rule with `v` (§5.16): where the method the law names searches, not a figure of the law.

Every other figure is a row in a version, with a source (N-215). **Band row values** are rows of a version, with its source and locator, so they may hold any literal: "10 800 zł plus 32% of the excess over 120 000 zł" is one row, written once (§5.8). **Date literals** (days, and months or years used as dates) appear only in rows, in the clauses `on`, `start`, `end`, `from`, `governs`, `known`, `enters`, `moves … to`, `arrears on`, `at` and `as known`, in index declarations, in pins, and in asks (below). A rule can't test `m >= 2028-01` on a time key: a law that starts on a date is a version with `governs` (`LOAD-LITERAL`).

**A bare literal that is a whole value** (a rule's value, a band row's value, a row's value, a pin or an `if` branch) takes the declared type and unit of its target: `rule none = 0` in a `money PLN` def is 0 zł, and `0` adopts any unit.

**An ask** allows every literal a def allows, plus its own: `paths`, `seed`, its pins, a `solve` bracket and unit, the level of a `quantile`, and date and period literals and spans (`flows(konto, 2026-03)`), because those are the question. Other figures in an ask are rows. Pins in a def's `as if` fall under the def's rule.

**Refusals:** `LOAD-LITERAL`. **Cost:** none. **Cases:** all.

### 7.2 Rounding

`round(x, u, mode)` is the multiple of u that the mode chooses.
- u is a **step**: a positive literal (`0.01`, `0.05`, `1`, `100`, `1e-9`), or an input or axis-keyed input whose every value is positive. Steps that aren't powers of ten exist in law, such as 5 Rappen (N-208).
- To give money, u must be a whole number of minor units (`LOAD-UNIT`).
- **Its type.** `round` of a `num U` gives a `num U`. It gives money when U is a currency and the step is a whole number of minor units, wherever it stands, an operand included; money is a `num CUR` too (§3.2), so a `num CUR` def still accepts it. An integer `num` from `round(x, 1, mode)` is accepted as an `int`.

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
- **Converting to money is a `round`.** A `num CUR` becomes money only through `round(x, u, mode)`, or `prorata` with the same step (§3.2, §5.12).

### 7.3 Transcendental values

These need `to u`, with u a step:
- `exp(x)`, `ln(x)`, `sqrt(x)`;
- `pow(x, q)` with a non-integer q;
- `cdf`, `pdf` and `quantile` of `normal`, `lognormal`, `exponential` and `gamma`, and `pmf` and `cdf` of `poisson` (the rows of §5.14 that need `to`);
- `crps` of a normal forecast;
- `periodic(r, P)` when it needs a non-integer power, and `factor(r, n, P)` with a non-integer n (§3.3).

**The value is the exact result, correctly rounded to u, ties half-even.**
- The engine decides it by interval arithmetic, refining until the interval lies within one rounding cell.
- Where the exact value is a rational on a cell edge, the engine detects it exactly: perfect powers, `exp(0)`, `ln(1)`, Φ(0) = 1/2, the median of a symmetric distribution.
- A rounding not decided within 2^16 bits of precision is refused at that key (`RUN-UNDECIDED`, `<function> to <u> not decided within the precision cap`; the writer states another `to`). It needs a true value within 2^−65536 of a cell edge, and it is the guarantee that two machines never disagree.
- `ln(x)` with x ≤ 0, `sqrt(x)` with x < 0, and `pow(x, q)` (or `factor`) with x < 0 and a non-integer q are refused (`RUN-DOMAIN`); `pow(0, q)` with q < 0 is a division by zero (`RUN-DIV`).

**Composites are exact.** An expression over such values is exact arithmetic on the rounded results. So the model's number is defined to the last digit, and an evaluator gets it by hand from the rounded values.

**The error bound.** A printed result that rests on rounded transcendentals, with no `round` after them, prints `≈` and a bound on its distance from the true real-number value. The bound comes from interval arithmetic with each rounded value r widened to [r − u/2, r + u/2] (a draw is one, §5.14 step 4; a value detected exactly on a cell edge isn't), run over the expression as written, each operation in its own order: a transcendental whose argument is itself an interval takes the hull of its exact values over that interval, then its own ± u/2. The bound prints to two significant digits, rounded up. A first-order estimate is not the bound. Examples are a ruin probability from a closed form, an annuity factor, and a CRPS. Money that has passed a `round` prints plain. It is the model's defined value, and `explain` shows the `to u` steps under it.

### 7.4 Randomness and seeds

The only randomness is `draw` (§5.14).
- A draw's value is a pure function of its address and the ask's `seed`.
- There is no stream, and no state between draws.
- A path is a pure function of its key, so any path can be recomputed alone (N-1406).
- **Asks without `paths`** see no draws. A def that reads one there is refused (`RUN-NO-PATH`).
- **Working by hand** uses pins: `u(z[...]) = 0.37` fixes a uniform, and `z[...] = v` fixes a value. With fixed uniforms u1, u2, … given in a list, they go to the draws in the canonical order of §5.14.
- **A test mode for statistics against closed forms** (CHG-13) is an ask that pins a parameter draw to its mean for every path: `with theta[*] = theta_mean`. It is printed in the ask's register, so a forecast with a pinned parameter can't be mistaken for one without.

### 7.5 Determinism

The same reality, the same ask and the same seed give the same output to the last digit, on any machine (R-16, N-1401). These hold the guarantee:
- Arithmetic is exact, and transcendentals are correctly rounded.
- Draws are SHA-256 addressed.
- **Every order is defined:**
  - index order for keys;
  - binding order for aggregates and `argmin`/`argmax` ties;
  - the order tuple, then supply keys, then demand keys, for `fill`;
  - pay id, phase and keys within a day;
  - the canonical order of draws;
  - sorted questions in holes;
  - the canonical format for printing.
- **The engine reads nothing outside the reality and the ask:** no clock, locale, environment or file order. "Today" is the ask's cut, or the model's `on`.
- **Parallel work combines exactly.** Path statistics are exact sums of rationals, combined in path order, so the number of workers changes nothing.

---

## 8. Holes, the register, refusals and warnings: the catalogue

A result is one of four things:
- a **value**;
- a **hole**, which prints its questions;
- a **bound**, which prints `bound: <id>: <reason>` (§5.17);
- a **refusal**, which prints its code and message.

Warnings print beside a result and leave it as it is (§8.5).

A model with holes or per-value refusals still runs, and its other results stand. A load refusal stops the model before anything runs; one in a scenario file refuses only that scenario and the scenarios over it (`ENGINE.md` §11).

### 8.1 Holes

A hole is a fact that is missing. It is never 0, never a default and never the last known value (N-1001, spec §4#8).
- **It propagates** through every value that depends on it: arithmetic, comparisons, rules, pays (as a hole movement) and `flows`.
- **It stops propagating** only where the outcome is decided without it: `and`, `or`, `any` and `all` under Kleene logic; an `if` whose branches are equal; a `sum` term known to be 0 under a hole `where`; a `fill` take that is known to be 0; a rule that is beaten, whose `when` and value aren't evaluated; `missing(r)`, which reads a hole as true (§5.10); and a place's balance after an anchor, which a hole movement dated on or before it no longer reaches (§5.18).
- **It carries** the questions of every operand, `where` and term it evaluated: the union, each once.
- **It prints** every question it carries, sorted by code point and without duplicates, with the item and keys each came from: a question reached at many keys prints once, with its list of keys.

| Code | Arises when | Question printed | The writer |
|---|---|---|---|
| `HOLE-MISSING` | the gap rule: no row gives an input or a band at those keys and that date (§4.6); a `prices Y` figure read outside Y with no index (§3.4); `bal` of a place with no `opening`, before its first anchor (§5.9, §5.18); `next_on` with no instance left in the run (§5.13); `flows` over days before the run's start S (`ENGINE.md` §2) | the input's `question`, else `x at k: no value for <period>` (`no value at <law date>` with no time key, §4.6); `<x> is in Y prices; no index moves it to <d>`; `no further <p> after <d>`; `before the run's start` | adds a row, or a `?` row with a better question; for S, reads from S on, or starts the model earlier |
| `HOLE-STATED` | a row, field or rule value is `? "…"` | its text | answers it with a row |
| `HOLE-IMPRECISE` | a comparison on an imprecise date isn't decided; one value is needed from an undecided interval (a print, money, a key, a term of an aggregate other than `min`, `max`, `argmin` and `argmax`); an imprecise `at` or `on` whose law-date classes give different values (§4.3); a movement's period crosses an end of a `flows` span, or holds the day of a `bal(x, d)`; a version is known inside an imprecise instance's period (§5.13.3); a state pay's date is imprecise (§4.4) | `date within <period> not precise enough for <comparison or use>` | states the day |
| `HOLE-CHANGED` | a version changed a booked pay, and neither it nor a later version says anything on back pay (§5.13.3) | `<pay> on <t> changed after it was paid (version V); is back pay due, and when?` | adds a version with `arrears on D of V` or `arrears none of V` (§5.13.3) |

### 8.2 The register

Every result lists, under it, the things it rests on that aren't plain sourced facts; a hole lists them like a value. The list is transitive: whatever its evaluation read, through any chain (N-713), each `when` it tested, each operand it evaluated and an ask's own `where` included. Arithmetic evaluates both operands; only `and`, `or` and `if` stop early (§5.10), so a row an `and` never read isn't listed.

| Entry | From |
|---|---|
| `assumed: <reason>` | a row value with `assumed` (§5.7), once, with the keys it filled; an index with `assumed`, for every figure keyed by it and every result that reaches one (§5.6); a version with `assumed`, for every result that reads one of its rows or a rule that points at it (§5.3) |
| `given: <def>[keys] = v (<version>)` | a version row aimed at a def (§5.10), printed once as written, with its key range if it has one |
| `hold: <place> holds <def>` | a balance read through `free`, or a `plan_low` on a place with a hold (§5.9, §5.13.1) |
| `missing record counted by <version>` | a rule that won because a `missing` test read true (§5.10) |
| `pinned: <pin>` | a `with` or `as if` pin, a fixed uniform, a test-mode parameter (§7.4); a pinned axis is listed here only, never under `reading:`. A pin given by an expression prints its value, `pinned: <pin> = v`, and with it the register of what the expression read (its `assumed` rows, its givens) |
| `scenario: <id>` | an ask in a scenario (§6.4) |
| `reading: <axis> = <option>` | each axis option that the line's result depends on (§5.5); options that give the line's result together are named together (`down \| half-up`). An axis the line prints as `*` has no `reading:` entry: every option of it gives the line's result |
| `bound: <id>: <reason>` | a declared bound the ask reached (§5.17) |
| `anchor: <place> <D>, stated <s>, difference <d>` | a balance of a place read on or after one of its anchors (§5.18), through `bal`, `free`, `plan_low`, `worth` or the walk: the place's last anchor at or before the day read, with its stated value and its difference, stated − booked. It is listed once per anchor and carried like every entry, so a figure built on an anchored balance (a later state pay, a net worth) shows that it rests on a statement whose difference the model doesn't explain. A difference of 0 is listed too. A hole difference prints `difference ?` with its questions (J70's register, the user's answer, 2026-10-19) |
| `held: <target>[keys] = v (<version>), read for <period>` | a row whose span has a start and no end (its own `governs`, else its version's), band rows included, read at a law date after the cut K: a figure carried into the future unchanged, as known today (spec Q-13). K is the ask's cut, also for a read made at an earlier cut (an instance's `when` or amount at min(t, K), a look-ahead on walk day t, §5.13.1): the register says what this ask takes as known today, not what the instance knew. An input read at its own time key (§4.6) is read for that key's period, at the period's first day, as a pay instance's law date is (§4.3), so it is `held` when that day is after K. A row with no span at all is timeless and is never `held`. Nor is a scenario's row, which isn't knowledge (§6.4): the `scenario:` entry stands for it. A figure that is published per period is written with closed spans instead, so its future is a hole until a rule or an `assumed` row gives it |
| `law date <d>, cut <K>` | always, for the top-level value, with K the ask's cut even where the value rests on reads at earlier cuts; a `check` whose sides are read at different law dates lists each |

### 8.3 Refused at load

The model does not run; a refusal in a scenario file refuses only that scenario and the scenarios over it (`ENGINE.md` §11). Each refusal names its file, line and item.

| Code | Message | What the writer does |
|---|---|---|
| `LOAD-PARSE` | `cannot parse: <what was expected>`; also tabs, bad indentation, chained comparisons | fixes the text |
| `LOAD-HEADER` | `missing tally version`, `missing on date`, `missing from date` (a scenario's header), `start after end`, `start and end are needed for places, pays and draws`; `<n> files have a model header; exactly one does` | completes the header, or keeps one model header |
| `LOAD-VERSION` | `files declare different tally versions` | migrates the old files (§9) |
| `LOAD-RESERVED` | `<word> is a keyword` (§2.2: built-in names and contextual words are free outside their positions) | renames |
| `LOAD-DUP` | `<id> is declared twice`; `a key added twice`; `two rules <label> with version <v>`; `band <b> has a built-in's name` | renames, or removes the copy |
| `LOAD-NAME` | `unknown id <x>` | declares it, or fixes the spelling |
| `LOAD-SHADOW` | `bound variable <k> has the name of a top-level id` | renames the variable |
| `LOAD-KEY` | `<key> is not a key of <index>`; `ambiguous key <key>`; `a key is also an id`; `wrong number of keys`; `range <a>..<b> is out of order`; `<k> is used as a key of <index> and <index'>` | fixes the key or names its index |
| `LOAD-KIND` | `<operation> on a <kind>`: an addition to a range or an open index, a pay to or from an asset | uses the right kind |
| `LOAD-UNIT` | `<a> and <b> have different units` (money in two currencies, money ± a bare number, a `num CUR` stored as money, an exchange rate applied to the wrong currency (§3.2), rate arithmetic outside §3.3, a step finer than the minor unit for money, a value of the wrong type); `axis <a> is used as a mode, but <o> is not a mode name` | converts explicitly, rounds, or tests the axis by rules (§5.5) |
| `LOAD-PRECISION` | `<literal> is finer than the minor unit`; `<a/b> terminates; write it as a decimal` | fixes the literal |
| `LOAD-LITERAL` | `figure <n> in a rule; make it a row`; `a date in a rule; make it a version` | moves the figure to a row with a source |
| `LOAD-OVERLAP` | `rows <a> and <b> of <x> tie at <keys> on <span>` | ends one row's span, or makes one more specific |
| `LOAD-BAND` | `band <b> in <version>: rows unordered, overlapping or with a gap at <x>` (per key for a keyed band); the whole reality is refused (a table in a scenario file: that scenario); `marginal(<b>, x)` over a first row from `-inf` | fixes the intervals, or declares the gap with `?` |
| `LOAD-CYCLE` | `<x> depends on itself at the same key: <path>` (over every rule, whatever its `when`); `<x> reads itself both earlier and later on <index>` | breaks the cycle, reads one direction, or splits the def per reading |
| `LOAD-GUARD` | `<x>[k ± n] is a recurrence read with no condition on k or pos(k) guarding it` (a check that a guard exists, §5.10; whether it suffices is `RUN-KEY`; a shifted read that isn't a recurrence needs none) | adds `when pos(k) > n`, an excepting rule's `when`, or an `if` |
| `LOAD-EXCEPTS` | `excepts names no rule <label>`; `an excepts cycle` | fixes the label |
| `LOAD-TO` | `<function> needs to u` | states the precision |
| `LOAD-STATE` | `<flow pay or a def it reaches> makes a state read`; `bal(x), free(x) or plan_low outside a state pay and its defs`; `next_on names a state pay` | moves the read into a state pay, or uses `bal(x, d)` |
| `LOAD-PLAN` | `<look-ahead> reaches draw <z>, which has no plan value` | adds `plan v` to the draw |
| `LOAD-FUTURE` | `<state pay> reads <balance> on or after its own day, or <flows> past it` | reads an earlier day, or `plan_low` |
| `LOAD-HOLD` | `<pay> takes money from <place> through bal, but the place holds <def>` | reads `free` |
| `LOAD-ARREARS` | `arrears on D before known`; `arrears with correction`; `two arrears lines for <pay>` (for one version's change, or disagreeing across versions known the same day); `arrears of <V>: V is not known before this version`; `arrears of <V> for <pay>: V's own line covers it`; `arrears for <x>: not a pay` | fixes the version |
| `LOAD-MOVES` | `moves <V>: unknown, not known before this version, or a scenario version`; `two moves of <V> known the same day` | fixes the line |
| `LOAD-COVERS` | `covers without a governs span`; `<x> is not a place`; `record <r> is dated outside the span <version> covers` | adds the span, or moves the record to the version that covers its day |
| `LOAD-COVERED` | `<def> aggregates <movement input> over a span with no covered() guard` | guards the aggregate with `covered(x, span)` (§5.18) |
| `LOAD-KNOWN` | `version <v> has no known date`; `version <v> is known after the model's on date` | adds the date, or moves `on` |
| `LOAD-SCENARIO` | `<item> of scenario <s> is read outside it`; `a row of <s> starts before <from>`; `a scenario version has known, arrears or correction`; `over cycle or deeper than 4` | moves the item, fixes the span, or removes the line |
| `LOAD-ASIF` | `as if deeper than 4`; `pin on unknown id <x>` | flattens it |
| `LOAD-DEPTH` | `expression nested deeper than 8` | splits it into defs |
| `LOAD-FORKS` | `ask <q> reaches <n> axis combinations; the cap is 64` | fixes axes with `with` |
| `LOAD-ASKONLY` | `<share, mean or quantile over path, real, solve> outside an ask` (`solve` is allowed in a rule with `v`; aggregates over keys are allowed anywhere) | moves it into an ask |
| `LOAD-MISSING` | `missing() outside the when of a cited rule` | cites the law, or lets the hole stand |
| `LOAD-ORIENT` | `ask <q> reaches crps or pdf without orient` | states `orient lower` or `higher` |
| `LOAD-BOUND` | `<item> reads bound <b>: <reason>` | asks the bound in an ask, or computes something else |

### 8.4 Refused per value

Printed as `refused: CODE message` at that key. A refusal propagates like a hole. It outranks a bound, and a bound outranks a hole (§5.17).

| Code | Message | What the writer does |
|---|---|---|
| `RUN-NA` | `<x>[k] is not applicable here` | tests it with `= -`, or limits the binding |
| `RUN-NOT-INT` | `<value> is not an integer` | rounds it |
| `RUN-KEY` | `<k ± n> is outside <index>` (a guard that doesn't suffice); `<key> was added after the cut` | guards the read |
| `RUN-CYCLE` | `<x>[k] reads itself through <path>` (a read at a computed key, §5.10) | breaks the chain, or reads an earlier key |
| `RUN-NO-DATE` | `<x> has no law date` | adds `at`, a time key, or the model's `on` |
| `RUN-RULES` | `rules <a> and <b> both apply to <x>[k] on <d>` | adds an `excepts` |
| `RUN-NO-RULE` | `no rule applies to <x>[k] on <d>` | adds a base rule, or a `?` rule |
| `RUN-EMPTY` | `<min, max, first, last, argmin, argmax, mean, share, quantile> over no keys` | widens the binding, or tests `count` first |
| `RUN-DIGITS` | `<x> passes 4 096 digits; round where the law rounds` | adds the law's `round` |
| `RUN-DIV` | `division by zero` | guards it |
| `RUN-DOMAIN` | `ln, sqrt or pow outside its domain` | guards it |
| `RUN-UNDECIDED` | `<function> to <u> not decided within the precision cap` | states another `to` |
| `RUN-PARAM` | `<distribution> parameter <p> = <v> is invalid` | fixes the input |
| `RUN-NEG` | `negative amount <v> for <pay or fill>` | writes two pays, or swaps the ends |
| `RUN-ORDER` | `fill <f>: the order of <arc> is a hole` | answers the hole |
| `RUN-WEIGHTS` | `prorata weights <w> are negative or sum to 0` | fixes the weights |
| `RUN-PRORATA` | `<total> isn't a whole number of <u>` (under `rest largest`) | rounds the total, or uses `rest last` |
| `RUN-PLACE` | `<pay> takes <debt> above 0`, `<claim> below 0`, `<holding> below 0 units` | fixes the amount, or marks the holding short |
| `RUN-FUTURE` | `<state pay> on <d> reads <d'> ≥ <d>` (a balance) or `<d'> > <d>` (flows) | reads an earlier day |
| `RUN-SOLVE` | `no sign change on [lo, hi]` | widens the bracket |
| `RUN-NO-PATH` | `<x> reads a draw in an ask without paths` | adds `paths`, or pins the draw |
| `RUN-RECORD` | `record <r> explains <instance>, which doesn't touch <place>` or doesn't exist | fixes the link |

### 8.5 Warnings

A warning leaves the value as it is and prints beside it.

| Code | Message | Means |
|---|---|---|
| `WARN-NEGATIVE` | `<account> is below zero from <d>` | the first day of each stretch whose end-of-day balance is below zero (N-1607); tested only where the balance is known |
| `WARN-HELD` | `free(<place>) is below zero on <d>` | on a day whose hold is above 0: money that wasn't ours was spent, such as VAT collected (N-504). A day with no hold prints `WARN-NEGATIVE` only |
| `WARN-UNREAD` | `<source, input, def or place> is read by nothing` | something was declared and never used, such as an account no total includes (N-1903). Decided on the item graph (`ENGINE.md` §3.1), so it doesn't depend on what the asks reach |
| `WARN-LEGS` | `the two records of <instance> disagree: <a> and <b>` | the import has a gap or an error (§5.18) |
| `WARN-STALE` | `scenario <s> overrides <target> at <keys>, changed in the reality by <version>` | the reality moved under the scenario (§6.4) |

The operations that change a reality have their own errors (`OP-*`) and warnings (`WARN-BACKPAY`, `WARN-DRAFT`, `WARN-ENDED`), listed in `OPERATIONS.md` §10.

---

## 9. Versioning of the language

- **Every file states `tally N`.** This document defines `tally 1`. Until the engine that runs it is done (`DONE.md`), the frozen text takes patches (`tally 1.0.1`, …, `DONE.md` rule 1): each corrects the definition of `tally 1`, files still state `tally 1`, and it is released with the last of them.
- **A released version never changes.** An engine runs every version it supports, each with its own semantics and its own canonical printer, so an old result can always be reproduced (C-15).
- **A new version comes with a migration:** a mechanical rewrite of any `tally N` model into `tally N+1`. Each migration is tested by running both on every model and case. Every ask must print the same result, or the change must be listed with its reason.
- **A new reserved word** that collides with an id is renamed by the migration.
- **Every result records** the `tally` version and the engine build that produced it.
- **One version per reality.** All files of a reality declare the same N (`LOAD-VERSION`), so a reality is migrated as a whole, in one amendment.
- **What may change between versions:** new built-ins, distributions, day counts and refusals. A change to the meaning of an existing construct is allowed only with a migration that rewrites every use whose result would change.

---

## 10. What Tally can't say, and why

| Tally can't say | Why | What to write instead |
|---|---|---|
| **Open recursion or an unbounded loop** | Every model must terminate, with a known cost (spec C-02, §4#2) | a recurrence at earlier keys, or at later keys of a bounded index; a read at a computed key, which `RUN-CYCLE` stops; a bounded index with `first` (§5.10) |
| **"Choose the best"** | The engine never optimises or picks for the household (spec §9, N-1802) | an axis or scenarios side by side, a breakeven by `solve`; a solver only where the law names one. `argmin` and `argmax` in a rule that points at a version (`v`) whose law names the choice (TIOPA s.36(3)'s most beneficial order) are the law's own; elsewhere they match a stated figure (the nearest answer choice), never choose. An ask may take the max over `as if` options, since it prints and decides nothing |
| **A default for a missing fact** | Never wrong in silence (spec C-03, §4#8) | a row, an `assumed` row with its reason, or the hole |
| **A default convention** (day count, compounding, FX leg, rounding mode) | A default is a silent assumption (CHG-06) | the convention as a row or an axis |
| **Floating point, or an approximation without a stated precision** | Results must agree on every machine to the minor unit (N-219) | `to u`, correctly rounded |
| **A density from a sample, or an infinite horizon from paths** | A sample has no density, and a path is finite (N-1408, N-1504) | a `bound` with the reason; the closed form asked as a value |
| **A later file or line that overrides an earlier one** | One namespace, and precedence by the facts' own dates (§2.5, §4.6) | a version with its `known` and `governs` |
| **An edit of history** | The old view must stay runnable (N-705) | an added version, `correction` or `arrears` |
| **Reading the future** | A decision on day t sees only what was known on t (spec §4#3, CHG-02) | `plan_low` over the known plan; records once they exist |
| **Keys made by computation** | Every key is data, so its existence is sourced and dated | an index addition in a version; an open index from records |
| **A user-defined function of values** | Totality and a fixed cost per evaluation | a def keyed by its arguments, a band of one argument (keyed bands for a table per key), or `x as if {pins}` to rerun a formula on other arguments, a pin taking any expression (N-1904) |
| **A distribution outside the closed list** | Inversion needs a known quantile, correctly rounded | `discrete`, a band from a uniform to a value, or a transformation of a listed one; a compound distribution as an enumeration or a recurrence (Panjer) over a bounded index |
| **A total the law defines without its split** | A split the law leaves open is not a fact (UL-13's shared cap) | the total, as a fill against the one cap; if a source asks for the split, its orders are an axis |
| **Built-in lot pooling or vintages** (CHG-01) | The pooling rule belongs to the law, and each jurisdiction has its own | lots as keyed inputs and defs, matched by `fill` (FIFO, an average pool, wash-sale lots) |
| **Text, input and output, the network** | The model is pure data over numbers and keys | notes, locators and questions |
| **Identifiers in Polish** | ASCII ids are unambiguous for every tool (N-1611) | Polish in notes, locators and questions |
