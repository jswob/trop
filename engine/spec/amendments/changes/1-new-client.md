# Change 1: a new job (a third B2B client)

**Statement (household):** "Od lutego 2027 mam trzeciego klienta, C: 6 000 zł netto miesięcznie, termin 21 dni. Umowę podpisałem 20 stycznia."

The user's main work is B2B, so a new job is a new client contract. (The wife's change of employer is the same shape as LANGUAGE §6.3 change 1; change 5 below gives her a new job abroad.)

**Operation:** add a fact (OPERATIONS §2.1), with `add_keys`. **Kind:** `add`. **Known:** 2027-01-20, after the base's `on` (2026-10-19), so `on` advances first (README, "Advancing `on`").

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2027-01-20

--- facts/2027.tally   (new file)
+part facts_2027
+  tally 2
+
+source contract_c "Umowa B2B z klientem C, 2027-01-20"
+
+version client_c_2027
+  cite contract_c
+  governs always
+  known 2027-01-20
+  client + [client_c] lives 2027-01-20..
+  fee_net[client_c] = 0 governs 2026-01..2027-01 "przed umową"
+  fee_net[client_c] = 6000 governs 2027-02.. "§ 5"
+  terms[client_c] = 21 "§ 6"
```

Lines added: 13 (11 non-blank, 2 blank), plus the `on` line. Lines edited: the `on` line only.

**New keys checked against closed-world rows.** `client` keys two inputs: `fee_net` and `terms`. The base rows for both are keyed rows (`fee_net[client_a]`, `terms[client_b]`), and neither keyless row covers `client_c`, home_2026's `fee_net = 0 governs 2026-01..2026-02` nor forecast_2026's `fee_net = 0 governs 2026-03.. assumed`: a keyless row covers the keys its index had on the row's `known` date (LANGUAGE §4.6). Without the two `fee_net[client_c]` rows and the `terms` row, every `sum(c in client: …)` would be a hole for every month. With them, there are no new holes.

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/1-new-client.md`.

The plan's sanity line: the first day anything differs is **2027-03-15**. That day the rule of the 15th invests 5 280 zł more in OKI, since `room[2027-03]` sees client C's February pay arriving on 2027-03-22. `room[2027-02]` is unchanged at 9 835.50: 2027-03-22 is outside its window (02-15..03-14). Its register differs from 2027-02-15, but sanity counts movements and balances only (OPERATIONS §7). The amendment's earliest `governs` is 2026-01, so the sanity check holds.

**Movements**
- **New:** `client_pay[client_c, m]` from 2027-02, 7 380 zł gross a month (6 000 + 1 380 VAT). The February invoice, issued 2027-02-28, is due 21 days later on 2027-03-21, a Sunday, so it arrives Monday **2027-03-22**.
- **VAT:** `vat_pay` rises by 1 380 zł a month from the February return, due 2027-03-25.
- **Ryczałt advances:** they rise by 720 zł a month (12% of 6 000), from February's advance, paid 2027-03-22. Each April's rises by 699 zł only (2027-04's, paid 2027-05-20, and 2028-04's, paid 2028-05-22): it deducts half of March's health contribution, now in the mid tier.
- **Health contribution, earlier tier:** 2027 revenue passes 60 000 zł in March instead of April, with year-to-date revenue at 17 000, 40 000 and 63 000. So March's health contribution moves from the low tier (0.6) to the mid tier (1.0). The payment of 2027-04-20 rises by the difference, 345.52 zł. The same happens each year: the payment of 2028-04-20 rises by 359.34 zł.
- **Health settlement:** the 2027 settlement, due 2028-05-22, falls by the same difference. The year still ends in the mid tier, at 270 000 zł, under 300 000. Only the monthly path changed. The 2028 settlement falls likewise, from 1 078.02 to 718.68.
- **Investing:** the rule of the 15th invests more each month from March 2027, as room grows. That money goes to OKI: in 2027 the IKE fills on 03-15 in both versions. In 2028 it fills a month sooner, on 02-15 instead of 03-15.
- **Unchanged:** client A's and client B's pays; the wife's pays; the rent.

**Balances**
- `konto` first differs on 2027-03-15, 5 280 zł lower: the rule of the 15th invests client C's pay before it arrives on 03-22. From then on the difference moves up and down within each month: 0 at 2027-06-30, +2 967.06 at 2027-12-31.
- `wcash[oki]` and `wunits[oki]` grow faster.
- `vat_owed` is higher on the days between a client C payment and the next VAT day. At the asked dates `vat_owed` is 0, so `konto_balance`'s two columns (konto, and konto minus VAT owed) are equal.

**Asks**
- `pit28`: the 2027 tax is higher by 7 899 zł (23 241 → 31 140), 12% of the new revenue less the larger health deduction. The advances carry all of it, so `pit28_due` is unchanged, for 2027 and 2028.
- `health_settlement`'s 2027 and 2028 rows are lower: 691.04 and 718.68.
- `taxes_paid` for 2027 and 2028 are higher.
- `net_worth` rises at every date from 2027.
- `ike_left` is unchanged, 0 at each year's end in both versions. The 2028 row reaches 0 sooner, on 2028-02-15 instead of 2028-03-15; the 2027 row reaches it on 2027-03-15 in both.
- **Unchanged:** `wife_net`, `pit37`, `replay_september` (at September 2026's booking cuts the key `client_c` doesn't exist yet, LANGUAGE §5.3), `explain_room`, `social_and_health`, `pit28`, `ike_left`, `konto_balance` at the 2026 dates and at 2027-06-30, and the 2026 rows of `net_worth`, `health_settlement` and `taxes_paid`.

**Register:** no new assumptions. One new held figure: every line that reads client C's fee for a month after K gains `held: fee_net[client_c] = 6000 (client_c_2027)`, read for those months. There is one new source, and one new key.

**Warnings:** none new.

**Depends on:**
- L12: none needed, because the span start `2026-01` is written out;
- L10, for the `on` advance.
