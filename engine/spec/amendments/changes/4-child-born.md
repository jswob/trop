# Change 4: a child is born

**Statement (household):** "12 maja 2028 urodziła się Ola. Ulgę na dziecko odlicza żona. Wniosku o 800+ jeszcze nie złożyliśmy." A follow-up, two weeks later: "Wniosek o 800+ złożony 1 czerwca przez PUE ZUS."

**Operation:** add a fact (OPERATIONS §2.1), with `add_keys`, three times:
- the birth certificate (one document, so one version);
- the household's decisions (one statement, so another version);
- later, the claim.

**Kind:** four amendments: an `advance` of `on` to 2028-05-14 (OPERATIONS §4), the `add` of the birth's two versions, an `advance` to 2028-06-02, then the `add` of the claim. Each `add` alone would be refused (`OP-EDIT`), since it would edit the `on` line. **Known:** 2028-05-14, then 2028-06-02.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2028-05-14

--- facts/2028.tally   (new file)
+part facts_2028
+  tally 1
+
+source birth_ola "Akt urodzenia Oli, USC Kraków"
+
+version ola_born
+  cite birth_ola
+  known 2028-05-14
+  person + [ola]
+  born[ola] = 2028-05-12
+  parent = false "brak innych relacji rodzicielskich"
+  parent[ola, me] = true
+  parent[ola, wife] = true
+
+version ola_home
+  cite household
+  known 2028-05-14
+  claim_800[ola] = - "wniosek jeszcze nie złożony"
+  relief_parent[ola] = wife "ulgę na dziecko odlicza żona"
+  residence[ola] = pl governs 2026-01.. "mieszka z rodzicami w Krakowie"
```

Then, at the follow-up (the `on` line moves again, to 2028-06-02):

```diff
--- facts/2028.tally
+++ facts/2028.tally
+
+source zus_pue "Potwierdzenie złożenia wniosku o 800+ w PUE ZUS"
+
+version ola_800
+  cite zus_pue
+  known 2028-06-02
+  arrears on 2028-06-20 for b800_pay
+  claim_800[ola] = 2028-06-01 "data złożenia wniosku"
```

**Lines:** 20 added at the birth and 8 at the claim (17 and 6 non-blank), plus the `on` line each time. Lines edited: the `on` line only.

**New keys checked against closed-world rows.** The operation lists every input keyed by `person` whose base rows don't cover `ola`.

| Input | Base row | Covers `ola`? | In the diff |
|---|---|---|---|
| `born` | keyed rows for me and wife | no | `born[ola]` |
| `parent[c, k]` | keyless `parent = false` (known 2025-12-31) | no: it covers only the keys `person` had on its `known` date (LANGUAGE §4.6) | restated keyless, with the two true rows (more stated keys win) |
| `residence[p, m]` | keyless `= pl governs 2026-01..` | no | `residence[ola] governs 2026-01..`, from the base's own start. `b800_due` stops at `born[c] < end_of(m)`, but `pit37_tax`'s `abroad` rule (`law/pl/employment.tally`) reads `residence[p, m]` for every month of the tax year, for every person, so her months before birth are read for 2026 to 2028. Stated from 2028-05 only, `pit37_due[ola, 2026..2028]` would be holes, and through her pay and refund instances so would `konto`, the room and the wrappers. |
| `relief_parent[c]` | none | no | stated. It is read only where `parent[c, p]` is true. |
| `claim_800[c]` | none | no | stated as `-`, then as a date |

Without the `parent = false` restatement, `is_child[me]` would read `parent[me, ola]`, a hole. Every `b800_pay` instance for me and the wife would then be a hole, and so would `konto` from the first one.

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/4-child-born.md`, with the diff as first written as a variant.

**First day anything differs:** 2028-06-20, when the 800+ arrives (below). The birth alone first changes a movement on 2029-06-14, the wife's larger refund: reading the new keys changes none on 2028-05-14. The earliest `governs` is 2026-01 (Ola's residence), so the sanity check holds.

**At the birth** (`on` = 2028-05-14):
- **800+:**
  - `b800_due[ola, m]` is false while the claim is `-`, so there are no `b800_pay` instances for Ola yet;
  - me and the wife are not children: `is_child` is false, so nothing changes for them.
- **Child relief:**
  - `relief_months[ola, 2028]` is 8 (May to December: born 05-12, before the end of May).
  - The wife's `child_relief[wife, 2028]` is 8 × 92.67 = 741.36.
  - Her `pit37_base` for 2028 is 69 484 (12 × (7 000 − 959.70 − 250), rounded), under the 112 000 limit.
  - Her tax is round(69 484 × 0.12 − 3 600 − 741.36) = **3 997** instead of 4 738.
  - The advances paid are 12 × 395 = 4 740.
  - So `pit37_due[wife, 2028]` is −743 instead of −2. The refund of **743 zł** comes on 2029-06-14, which is 45 days after the return day of 2029-04-30.
  - From 2029 the relief is 12 × 92.67 = 1 112.04 a year for 17 full years (2029 to 2045), then 370.68 for the four months of 2046: 18 tax years with 2028's 741.36. The tax stays above the relief, so the `unused` rule's hole never fires.
- **Unchanged for me:** `child_relief[me, …]` reads `relief_parent[ola] = wife`, so my PIT is unaffected. As a ryczałt taxpayer I couldn't take it anyway, and the model doesn't need to know that.

**At the claim** (`on` = 2028-06-02):
- **The claim falls in the window:** 2028-06-01 is within 3 months of the birth (`b800_window`), so `b800_from[ola]` is 2028-05. Ola is entitled from her birth month.
- **May (`b800_pay[ola, 2028-05]`):**
  - It is dated 2028-05-22 (the 20th is a Saturday), before the claim was known.
  - Booked at its own cut, it was false, so it had no movement.
  - The claim version changes it by δ = +800. Under `arrears on 2028-06-20`, that is a correction movement of 800 from `zus` to `konto` on 2028-06-20.
- **June:** the June instance, dated 2028-06-20, is planned after K with the claim known: 800.
- **On the day:** 2028-06-20 brings 1 600 zł, which is what ZUS does: the first payment carries the back month.
- **Later months:** 800 a month until the month before her 18th birthday. The last is 2046-04, since `age(…, last day of 2046-05) = 18`.

**Balances:** `zus` pays `konto` 1 600 zł on 2028-06-20, then 800 a month (`zus` is −172 800.00 at E). konto is higher by what has arrived and the rule of the 15th has not yet invested, and the wrappers take the rest; month by month that needs the run.

**Asks**
- `pit37`: the wife's 2028 row is −743 instead of −2. The ask gains three lines for Ola, 2026 to 2028, each 0.00.
- `net_worth`: higher from 2028-06 by the accumulated 800+, and by the refund from 2029-06-14.
- The other asks are unchanged until those dates.
- **A new ask** the household may save:
  ```
  ask ola_800_paid: flows(b800_pay[ola], y) for y in [2028, 2029]
  ```
  It returns 6 400 for 2028 (May to December, with May paid in June) and 9 600 for 2029.

**Holes:** none, since Ola's residence is stated from 2026-01 (the table above). Had the household not said `claim_800[ola] = -`, then from 2028-05 the instances would ask "Kiedy złożono wniosek o 800+ dla tego dziecka?", and `konto` would be a hole until it was answered.

**Register:** one household version (`ola_home`) and two document versions (`ola_born`, citing the birth certificate, and `ola_800`, citing the ZUS confirmation). Nothing is assumed. New `held` entries: `residence[ola] = pl (ola_home)`, and the three rows of the 800+ law's `b800_2024`, read from 2028-07.

**Warnings:**
- `WARN-BACKPAY` at the claim lists `b800_pay[ola, 2028-05]`, δ +800, with the `arrears` line that covers it (OPERATIONS §2.1, §10). The birth changes no booked pay: Ola's re-measured instances have δ 0.
- The household's spending budget doesn't grow with a child. This is not a warning: the model can't know it. The household's next statement would restate `spend_budget`.

**Depends on:**
- The fix to `b800_due` in the base (`born[c] < end_of(m)` and `claim_800[c] != -`, logged in L20). Without it, every pre-birth month asks for Ola's residence, and an unclaimed benefit is a hole. `pit37_tax`'s `abroad` rule has no such guard, so the diff states her residence from 2026-01.
- L8: an instance exists only where its `when` is true.
- L10: the `on` advances.
