# Change 7: a scenario, "what if one partner stops working"

**Statement (household):** "Co jeśli żona przestanie pracować od stycznia 2027? Ile mniej odłożymy i czy starczy na bieżąco?"

**Operation:** add a scenario patch (OPERATIONS §5). Claude never proposes one (S-09), so the household's words are its required statement. **Kind:** a new scenario file. The reality is untouched and `on` doesn't move: a scenario is read over the current reality, as known today.

## Diff

```diff
--- scenarios/wife_stops.tally   (new file)
+scenario wife_stops
+  tally 1
+  from 2027-01-01
+  note "Co jeśli żona przestanie pracować od stycznia 2027? (oświadczenie gospodarstwa z 2026-10-19)"
+
+version wife_stops_salary
+  cite household
+  salary[wife_acme] = 0 "założenie scenariusza: brak pensji od stycznia 2027"
```

**Lines:** 8 added (7 non-blank). Lines edited: 0.

**Scenario rules** (`LOAD-SCENARIO`) hold:
- the version has no `known`, `arrears` or `correction`;
- its row's span defaults to `2027-01-01..`, not before `from`;
- nothing in the reality reads it.

## How to run it

| Run | What it gives |
|---|---|
| `plan scenario wife_stops` | the diff of every saved ask, movement and balance against the base (OPERATIONS §7) |
| `ask net_worth with scenario wife_stops` | the whole ask in the scenario |
| `net_worth_at[2041-12-31] as if {scenario wife_stops} - net_worth_at[2041-12-31]` | the difference inside one expression, if the household saves a def for net worth |

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/7-scenario-partner-stops.md`.

**First day anything differs:** 2027-01-15 (`ENGINE.md` §9 checks that nothing before `from` differs).
- That day the rule of the 15th measures room up to 2027-02-14. The window includes the wage of 2027-02-10, now 0.
- The December 2026 wage, paid 2027-01-11, is unchanged. Its instance is dated after `from`, so it sees the scenario, but its month key 2026-12 is outside the row's span.

**Movements**
- **The wage:** `wage[wife_acme, m]` from 2027-01 is 0. So are her employee ZUS, health and PIT advances: 5 101.67 less arrives on each wage day, the 10th or the next business day, from 2027-02-10.
- **Her PIT-37:**
  - From 2027 her base is 0: no salary, and no deductible costs (`if salary > 0`). Her tax is 0 and her advances are 0, so `pit37_due` is 0.
  - The base's refunds of 2 zł disappear, from the one for 2027, due 2028-06-16, on.
- **Investing:** the rule of the 15th has about 5 100 less room in most months, but January 2027's is only 638.20 lower: the base's window minimum came before the wage.
  - IKE still fills each year, a month or two later: on 2027-04-15 instead of 03-15, and on 2028-05-15 instead of 03-15.
  - OKI receives less from April 2027, and nothing in March and August 2027 (the base's 5 540.98 and 4 409.93).
  - `room[2027-08]` is 0: client A's July pay arrives on 08-16, after the invest, and the window's lowest point is below the floor. The model takes `max(0, …)`: it invests 0, not borrows.
- **Unchanged:**
  - my business: ZUS, health, ryczałt, VAT and clients;
  - the rent and spending;
  - the loan;
  - 800+. There are no children in the base.

**Balances:** at month ends `konto` is a little higher, not lower: the room falls by more than the missed wage, so a little more stays in konto (+230.58 at 2027-06-30, +475.00 at 2027-12-31). Within a month it is lower before the 15th (−4 525.22 on 2027-08-14). The wrappers grow more slowly.

**Asks**
- `wife_net`: the 2026-10 row is unchanged; 2027-01 is 0.
- `pit37`: the wife's 2027 and 2028 rows are 0.
- `konto_balance`: higher at 2027-06-30 and 2027-12-31, by little (6 990.77 and 7 131.75), because the 15th absorbs the difference. 2026 is unchanged.
- `net_worth`:
  - unchanged at 2026-12-31;
  - lower at 2031, 2041 and 2059, by roughly 61 000 zł a year of lost net wages plus their growth at the assumed 6%;
  - the plan prints the exact figures.
- `ike_left`: the 2027 and 2028 rows are 0.00 in both; they reach 0 later, on 2027-04-15 and 2028-05-15.
- `taxes_paid`: 2028 is 214 zł lower, 68 783.00: `oki_tax[2027]` falls from 368 to 152, and the wife's 2 zł refund is gone.
- **Unchanged:** `replay_september`, `explain_room`, `health_settlement`, `pit28`, `social_and_health`, my `pit37`, `taxes_paid` for 2026 and 2027, and every result before 2027-01-15.

**Not modelled, and said so:**
- Her health cover would come through my contribution, as a family member, at no extra cost, so no pay is missing.
- Her own pension account stops growing. The model has no place for future pension rights, so that is a bound of this household's model, not a hole.

**Composes with change 4:**
- **The relief hole.** With Ola born and `relief_parent[ola] = wife`, the wife's 2028 PIT-37 becomes a hole from the `unused` rule: "Niewykorzystana ulga na dziecko…". Her tax is 0 and the relief is 741.36.
- **The household's answer** is to restate `relief_parent[ola] = me` in the scenario. Then my PIT-37 picks it up and is also a hole, since a ryczałt taxpayer can't take it. The law part should then say so, with a rule.
- **What the scenario shows:** a scenario can surface a question that the reality never reaches.

**Register:** every result gains `scenario: wife_stops`. The scenario's row isn't listed as `held`: it isn't knowledge, and the `scenario:` entry stands for it (LANGUAGE §8.2). Nothing is assumed beyond the scenario itself.

**Warnings:**
- None now. Under the fixture premise every law part is `tested` (`amendments/README.md`), so there is no `WARN-DRAFT`.
- `WARN-STALE` appears if a later reality version writes `salary[wife_acme]` from 2027. Change 5's `salary[wife_acme] = 0 governs 2029-07..` is one: the scenario then overrides something newer than itself.

**Depends on:** nothing open. Scenarios visible by date, layer precedence and `plan` with a scenario are all defined.
