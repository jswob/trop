# Change 3: a retroactive law change (lower health share, back to January)

**Statement:** none is needed. This is a law version (OPERATIONS §3), with illustrative content.
- An act passed on 2027-06-15 lowers the ryczałt health contribution for the middle revenue tier from 100% to 90% of the base. It applies from 2027-01-01.
- Monthly payments already made stand. The difference is settled in the annual settlement, due by 20 May of the next year.

**Operation:** add a law version. **Kind:** `add`. **Known:** 2027-06-15. `on` advances to it first.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2027-06-15

--- law/pl/business.tally
+++ law/pl/business.tally
 source ryczalt_act "…"
+source health_2027_cut "Ustawa z 2027-06-15 o zmianie ustawy o świadczeniach opieki zdrowotnej (przykład), art. 1 pkt 2 i art. 3: stosuje się od 2027-01-01"
 source vat_act "…"
 …
 version vat_2026
   …
+
+version health_ryczalt_2027
+  cite health_2027_cut
+  governs 2027..
+  known 2027-06-15
+  arrears none for health_pay
+  health_share[mid] = 0.9 "art. 1 pkt 2"
```

Lines added: 8, plus the `on` line. Lines edited: the `on` line only.

**Why only one `arrears` line:**
- The settlement pays `health_settle_pay` and `health_settle_back` for 2027 are dated 2028-05-22, after K. They are planned with the new law, not changed after booking, so they need no line.
- The ryczałt advances read `flows(health_pay, m)`, the booked amounts. So they change only where a `health_pay` they read is planned after K.

## Back pay listed (OPERATIONS §3, `WARN-BACKPAY`)

The operation re-evaluates every booked instance whose read set the version touches (ENGINE §5, step 2). That is `health_pay[2027-04]`, booked on 2027-05-20, the one booked mid-tier month. The low-tier instances for 2027-01 to 2027-03 (booked 02-22, 03-22 and 04-20) read `health_share[low]`, for which the version has no row, so step 2 skips them; the table shows them with δ 0 to say why.

| Instance | Tier | δ | Under the version's line |
|---|---|---|---|
| `health_pay[2027-01]` | low: year-to-date revenue 17 000 | 0 | — |
| `health_pay[2027-02]` | low: 34 000 | 0 | — |
| `health_pay[2027-03]` | low: 51 000 | 0 | — |
| `health_pay[2027-04]` | mid: 68 000 | −86.38 | `arrears none`: no movement; the payment stands |

`health_pay[2027-05]` is due 2027-06-21, after K. It is planned at the new share.

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/3-retroactive-law.md`.

The health base for 2027 is projected from 2026's: 9 228.64 × 1.04 = 9 597.79. The base doesn't know the published 2027 figure.

At that base, the mid-tier contribution is:
- 863.80 at a share of 1.0;
- 777.42 at 0.9.

The difference is 86.38 a month. The low tier, at 0.6, is 518.28 and doesn't change.

**First day anything differs:** 2027-06-15. That day the rule of the 15th measures room up to 2027-07-14, and the window includes the lower health payment of 2027-06-21.

**Movements**
- **Health, monthly:** `health_pay` from 2027-05, paid 2027-06-21, is 777.42 instead of 863.80. Every later mid-tier month is cut to 0.9 too, since the version governs `2027..`: in 2028, on that year's base, it is 808.52 instead of 898.35.
- **Health, 2027 settlement** (2028-05-22):
  - **Before:** the year at 12 × 863.80 = 10 365.60, minus 9 329.04 paid (3 × 518.28 + 9 × 863.80), leaves **1 036.56** to pay.
  - **After:** the year at 12 × 777.42 = 9 329.04, minus 8 638.00 paid (3 × 518.28 + 863.80 for April, as booked, + 8 × 777.42), leaves **691.04** to pay.
  - The settlement falls by **345.52** = 4 × 86.38. April's overpayment of 86.38 comes back through it. The three low-tier months are topped up to 0.9 instead of 1.0, which accounts for the other 3 × 86.38.
- **Ryczałt advances:**
  - From June's advance (paid 2027-07-20), half of the health paid in the month is deducted, and it is now 43.19 zł less. The advance rises by about 5 zł a month.
  - The May advance, paid 2027-06-21, reads April's health payment, booked at 863.80 on 2027-05-20. It is unchanged.
- **PIT-28 for 2027:**
  - The year deducts half of the health paid in 2027: Dec 2026's payment, then 2027-01 to 2027-11's. Of these, the seven payments from 2027-05 are 86.38 lower.
  - The deduction falls by 302.33, and the tax rises by about 36 zł. Nearly all of it is already in the higher advances, so `pit28_due` moves by rounding only.
- **Investing:** room rises by 86.38 zł in June 2027, before the higher advance starts, then by 81.38 zł (86.38, less the higher advance) in July and October to December. August's is unchanged, since its window's minimum is day t, and September's rises by 162.76. In 2027 the difference goes to OKI. In 2028 it goes to IKE first (+81.38 in January, −5.00 in February), and IKE then fills with 76.38 less in March, which OKI takes.

**Balances:** `konto` isn't higher at month ends. The rule of the 15th invests the freed money before it lands, so konto differs only between a 15th and that month's group day (−86.38, then −81.38), and `konto_balance` at 2027-06-30 and 2027-12-31 is unchanged. The gain sits in OKI: +574.66 at 2027-12-31. The 345.52 of 2028-05-22 falls in the 2028-05-15 window, so it is invested ahead the same way.

**Asks**
- `health_settlement`:
  - 2027 is 691.04 instead of 1 036.56;
  - 2028 is also lower, 808.53 instead of 1 078.02, because every 2028 mid-tier month is at 0.9;
  - 2026 is unchanged, since the version governs from 2027.
- `social_and_health`: the 2028-09 row's health is lower; 2026-10 and 2027-01 are unchanged (2027-01 is low tier).
- `pit28`: the tax rises by tens of złoty (+36 for 2027), but the advances already carry nearly all of it, so the due rises by 1 zł for 2027 (2.00 → 3.00) and 3 zł for 2028 (1.00 → 4.00). `taxes_paid` rises by tens: 69 089.00 for 2027 and 69 045.00 for 2028.
- `net_worth`: higher from 2027.
- **Unchanged:** `pit37`, `wife_net`, `replay_september`, `explain_room`, `ike_left`, `konto_balance` at all four dates, `net_worth` at 2026-12-31, and every other 2026 cell.
- **The old view:** `ask … as known 2027-06-14` still shows it, since the version is invisible to cuts before its `known`.

**Without the `arrears` line:**
- `health_pay[2027-04]` would get a hole movement, `HOLE-CHANGED`, dated on or after 2027-06-15.
- Every `flows` over `konto` or `zus` reaching that day or later would be a hole asking "is back pay due, and when?". That means the room of every month from June 2027, and every later balance. So would every `flows` of `health_pay` that can count the hole movement (LANGUAGE §5.13.3): `health_settlement` 2027, `pit28` 2027 and 2028, and through them `taxes_paid` 2027 and 2028. `health_settlement` 2028 stays known, 808.53: its `flows` are keyed to 2028's instances.

**Register:** one new law version with a back-pay line. Nothing is assumed. The `held` figure for `health_share[mid]`, read after K, becomes the new version's: `held: health_share[mid] = 0.9 (health_ryczalt_2027)`.

**Warnings:** `WARN-BACKPAY` lists the one instance above, with the line that covers it, `arrears none for health_pay` (OPERATIONS §10). The line answers the back pay, so no hole arises.

**Depends on:**
- L17: the settlement and the advances are flow pays, re-measured at their own cut, and the `arrears` line names a flow pay;
- L10, for the `on` advance.
