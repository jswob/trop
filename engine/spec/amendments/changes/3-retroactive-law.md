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

The operation re-evaluates every booked instance whose read set the version touches (ENGINE §5, step 2). These are the instances of `health_pay` for 2027-01 to 2027-04, booked on 2027-02-22, 03-22, 04-20 and 05-20.

| Instance | Tier | δ | Under the version's line |
|---|---|---|---|
| `health_pay[2027-01]` | low: year-to-date revenue 17 000 | 0 | — |
| `health_pay[2027-02]` | low: 34 000 | 0 | — |
| `health_pay[2027-03]` | low: 51 000 | 0 | — |
| `health_pay[2027-04]` | mid: 68 000 | −86.38 | `arrears none`: no movement; the payment stands |

`health_pay[2027-05]` is due 2027-06-21, after K. It is planned at the new share.

## What it changes, in words

The health base for 2027 is projected from 2026's: 9 228.64 × 1.04 = 9 597.79. The base doesn't know the published 2027 figure.

At that base, the mid-tier contribution is:
- 863.80 at a share of 1.0;
- 777.42 at 0.9.

The difference is 86.38 a month. The low tier, at 0.6, is 518.28 and doesn't change.

**First day anything differs:** 2027-06-15. That day the rule of the 15th measures room up to 2027-07-14, and the window includes the lower health payment of 2027-06-21.

**Movements**
- **Health, monthly:** `health_pay` from 2027-05, paid 2027-06-21, is 777.42 instead of 863.80. So is every later mid-tier month, in 2028 and on too, since the version governs `2027..`.
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
- **Investing:** room rises by about 81 zł a month from 2027-06 (86.38, less the higher advance). The difference goes to OKI.

**Balances:** `konto` is higher by about 81 zł a month from 2027-06-21, and by 345.52 more on 2028-05-22.

**Asks**
- `health_settlement`:
  - 2027 is 691.04 instead of 1 036.56;
  - 2028 is also lower, because every 2028 mid-tier month is at 0.9;
  - 2026 is unchanged, since the version governs from 2027.
- `social_and_health`: the 2028-09 row's health is lower; 2026-10 and 2027-01 are unchanged (2027-01 is low tier).
- `pit28`: 2027 and 2028 rise by tens of złoty; `taxes_paid` likewise.
- `net_worth`: higher from 2027.
- **Unchanged:** `pit37`, `wife_net` and `replay_september`.
- **The old view:** `ask … as known 2027-06-14` still shows it, since the version is invisible to cuts before its `known`.

**Without the `arrears` line:**
- `health_pay[2027-04]` would get a hole movement, `HOLE-CHANGED`, dated 2027-06-15.
- Every `flows` over `konto` or `zus` reaching that day would be a hole asking "is back pay due, and when?". That means the room of every month from June 2027, and every later balance.

**Register:** one new law version with a back-pay line. Nothing is assumed.

**Warnings:** `WARN-BACKPAY` lists the one instance above. It is cleared by the line.

**Depends on:**
- L17: the settlement and the advances are flow pays, re-measured at their own cut, and the `arrears` line names a flow pay;
- L10, for the `on` advance.
