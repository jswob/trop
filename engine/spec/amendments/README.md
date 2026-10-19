# Amendments: the synthetic household and the seven common changes

**What this is.** The prototype's synthetic household (spec §12, "In the prototype", with illustrative figures) is written as one Tally reality. On top of it are the seven changes of the brief's §4.6, each as a diff with the outputs it changes, in words.

**What it is not:**
- **Not the household's data.** None of the household's own data is used, and none of the earlier brief's figures.
- **Not gated law.** The files under `law/pl/` are illustrative excerpts written by the lead, with the prototype's figures. They have not passed a gate (OPERATIONS §3), and each says so in a note. Their locators point at real articles, but the parts were not checked against independent examples. For the seven changes' `plan` results and their expected outputs, the store holds every part under `law/pl/`, and change 5's `law/de/`, as `tested` against its hash: a fixture premise. A status lives in the store, not in the content (OPERATIONS §1), so the files stay as they are. The gate itself is tested on its own fixture (BUILD-ORDER M3).
- **Not evaluated by the lead.** The figures in the change files are the lead's hand arithmetic, as one-line `python3 -c` with Fraction or Decimal, logged in `log/sessions.md`. They are illustrations of what `plan` would print, not expected answers. The household is evaluated in D7 by evaluators who are not its author, like any model.

## The household (`household/`)

**The reality** is 13 files and 1008 lines, laid out as LANGUAGE §6.1 says.

| File | Lines | Holds |
|---|---|---|
| `reality.tally` | 34 | header (`on 2026-10-19`, 2026-01-01..2059-12-31), the two sources `household` and `modeller`, the ETF unit, the indexes |
| `people.tally` | 19 | births, parenthood, residence, who works where |
| `places.tally` | 50 | `konto` (holding VAT owed), cash, Marek's debt to us, the three wrappers' cash and units, the outside places, the bank-statement schema, openings |
| `pays.tally` | 133 | rent, spending (budget, then learned from the statement), business costs, the rule of the 15th (IKE to its limit, then brokerage, OKI from 2027), buying units, the loan to Marek |
| `loans.tally` | 82 | the 0% laptop instalment plan: equal instalments, overpayment that shortens |
| `law/pl/common.tally` | 103 | minimum wage, projected wage, the health base (published, else projected), holidays 2026–2030, the return day |
| `law/pl/employment.tally` | 105 | the wife's employment: ZUS, health, PIT advances, PIT-37 with child relief |
| `law/pl/business.tally` | 212 | my B2B on ryczałt: invoices, VAT and its hold, ZUS stages, health by tier with the annual settlement, ryczałt advances, PIT-28 |
| `law/pl/wrappers.tally` | 86 | the IKE limit; OKI from 2027, with tax on the average value |
| `law/pl/benefits.tally` | 41 | 800+ |
| `facts/2026.tally` | 87 | the household's 2026 versions: forecast assumptions, home, business registration, the broker's prices, Marek |
| `records/mbank/2026-09.tally` | 31 | September's statement: 20 rows and the closing balance. Client B's August invoice is missing on purpose. |
| `asks/core.tally` | 25 | 11 saved asks |

**The household, in short:**
- **Me:** B2B on ryczałt (12%) from 2026-03. Client A pays 14 000 net and client B 3 000. ZUS starts on the start-up relief, then preferential, then standard.
- **The wife:** employed, 7 000 gross.
- **Home:** rent of 3 200.
- **Investing:** a rule on the 15th invests free money above 1 000 zł.
- **Marek** owes us 800 of 2 000.
- **A laptop** is on a 0% instalment plan.

## The seven changes (`changes/`)

Each change is written against the household alone.
- Their effects are independent unless the file says how they compose; change 5 and change 7 each have a note on change 4.
- They also compose in any order (LANGUAGE §6.3): each adds its own items, and the read rule orders rows by their own dates.

| # | File | Change | Operation | Known | Lines added, blank ones too | Lines edited | Gaps it leans on |
|---|---|---|---|---|---|---|---|
| 1 | `1-new-client.md` | a new job: a third B2B client from 2027-02 | add fact, `client + [client_c]` | 2027-01-20 | 12 | `on` | L10 |
| 2 | `2-law-next-year.md` | the 2027 minimum wage and projected wage | add law version ×2 | 2026-09-12, 2026-12-18 | 14 | `on` | L10 |
| 3 | `3-retroactive-law.md` | mid-tier health share cut to 0.9, back to 2027-01, `arrears none` | add law version | 2027-06-15 | 8 | `on` | L10, L17 |
| 4 | `4-child-born.md` | Ola born 2028-05-12; 800+ claimed later with back pay | add fact ×3, `person + [ola]` | 2028-05-14, 2028-06-02 | 28 | `on` | L8, L10, L20 |
| 5 | `5-job-abroad.md` | the wife moves to Berlin with a job paid in euro | amend (new places, input, pays, stub law) + add fact | 2029-05-01 | 71 | `on` | L1, L8, L9, L10, L11, L19 |
| 6 | `6-loan-and-overpayment.md` | a 40 000 car loan; 5 000 overpaid at instalment 9 | add fact ×2 (the open index `loan` takes `car` from its rows), `seller + [car_dealer]` | 2027-04-02, 2028-01-08 | 19 | `on` | L8, L10, L13, L17 |
| 7 | `7-scenario-partner-stops.md` | what if the wife stops working from 2027-01 | scenario patch | — | 8 | 0 | none |

**Advancing `on`.** Every change but the scenario holds a version known after the base's `on` (2026-10-19), so `on` moves forward to that `known` date first, as its own amendment of kind `advance` (OPERATIONS §4); the change's lines follow as an `add`. Change 2 also holds a late entry, a version known before the base's `on` (`min_wage_2027`, known 2026-09-12).
- Before D6, **OPERATIONS §2.1** called that "a restate of the header". But a restate must change no result, and moving the cut K can change results: what is booked versus planned, and which versions are visible.
- In this household the advance alone changes no value, because no version is known between the two dates; it turns plan into past, moves every register's cut line, and drops the `held` entries read for the months it passes. The kind was undefined: gap **L10**, resolved in D6 as a fourth change kind, `advance`, which may only move `on` forward and which runs the plan (OPERATIONS §4, LANGUAGE §6.2).
- Each diff shows the edited `on` line: that line is the `advance`. The rest of the diff, added lines only, is the `add`.

**Expected outputs.** `expected/` holds, for each change, every asked cell before and after, the first day anything differs, the warnings, holes and register, and the variants named there. Each was derived by two evaluators blind to each other and settled by the lead (`expected/README.md`). Where a change file's words and its expected file differ, the expected file holds.

**The gap ids.** The L and J ids (the last column above, and each file's "Depends on") name rows of the design log the spec was written from. The spec doesn't define them: each is resolved in the spec's own text, and that text is what the changes rest on.

**What every change file holds:**
1. the household's statement;
2. the operation and kind;
3. the diff;
4. the closed-world check for new keys;
5. the outputs in words: the first day anything differs, movements, balances, asks, the register and warnings;
6. the gaps it depends on.

## Fixes to the household made while writing the changes

These are logged in `log/sessions.md`. They are the lead's own model errors, fixed in the base before any evaluator sees it.
- **`b800_due`** gained `born[c] < end_of(m)` and `claim_800[c] != -`.
  - Without the first, a child's months before birth read a residence nobody states.
  - Without the second, a benefit not yet claimed is a hole, where it should be "not paid". This is part of L20: what a booked instance does when its `when` was unknowable at its date.
- **`principal_part`** gained the rule `last` (`k >= n` repays the rest). Without it, a rounded annuity leaves grosze to an instalment after the term.
