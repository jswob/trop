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
+  tally 1
+
+source contract_c "Umowa B2B z klientem C, 2027-01-20"
+
+version client_c_2027
+  cite contract_c
+  known 2027-01-20
+  client + [client_c]
+  fee_net[client_c] = 0 governs 2026-01..2027-01 "przed umową"
+  fee_net[client_c] = 6000 governs 2027-02.. "§ 5"
+  terms[client_c] = 21 "§ 6"
```

Lines added: 10, plus the `on` line. Lines edited: the `on` line only.

**New keys checked against closed-world rows.** `client` keys two inputs: `fee_net` and `terms`. The base rows for both are keyed rows (`fee_net[client_a]`, `terms[client_b]`), and the keyless `fee_net = 0 governs 2026-01..2026-02` covers only the keys of its own version. Without the two `fee_net[client_c]` rows and the `terms` row, every `sum(c in client: …)` would be a hole for every month. With them, there are no new holes.

## What it changes, in words

The plan's sanity line: the first day anything differs is **2027-02-15**. That is when `room[2027-02]` first sees client C's money arriving on 2027-03-22. The amendment's earliest `governs` is 2026-01, so the sanity check holds.

**Movements**
- **New:** `client_pay[client_c, m]` from 2027-02, 7 380 zł gross a month (6 000 + 1 380 VAT). The February invoice, issued 2027-02-28, is due 21 days later on 2027-03-21, a Sunday, so it arrives Monday **2027-03-22**.
- **VAT:** `vat_pay` rises by 1 380 zł a month from the February return, due 2027-03-25.
- **Ryczałt advances:** they rise by 720 zł a month (12% of 6 000), from February's advance, paid 2027-03-22.
- **Health contribution, earlier tier:** 2027 revenue passes 60 000 zł in March instead of April, with year-to-date revenue at 17 000, 40 000 and 63 000. So March's health contribution moves from the low tier (0.6) to the mid tier (1.0). The payment of 2027-04-20 rises by the difference.
- **Health settlement:** the 2027 settlement, due 2028-05-22, falls by the same difference. The year still ends in the mid tier, at 270 000 zł, under 300 000. Only the monthly path changed.
- **Investing:** the rule of the 15th invests more each month from March 2027, as room grows. In 2027 that money goes to OKI, since the IKE limit is reached sooner.
- **Unchanged:** client A's and client B's pays; the wife's pays; the rent.

**Balances**
- `konto` is higher from 2027-03-22 by the net inflow, less the higher taxes.
- `wcash[oki]` and `wunits[oki]` grow faster.
- `vat_owed` is higher on the days between a client C payment and the next VAT day. So `konto_balance`'s second column (konto minus VAT owed) rises less than its first.

**Asks**
- `pit28` for 2027 is higher by about 7 920 zł of tax on the new revenue. Most of it is already paid through the advances, so `pit28_due` barely moves.
- `health_settlement`'s 2027 row is lower.
- `taxes_paid` for 2027 and 2028 are higher.
- `net_worth` rises at every date from 2027.
- `ike_left`'s 2027 row reaches 0 sooner, though it is 0 at the year's end in both versions.
- **Unchanged:** `wife_net`, `pit37` and `replay_september`.

**Register:** no new assumptions or held figures. There is one new source, and one new key.

**Warnings:** none new.

**Depends on:**
- L12: none needed, because the span start `2026-01` is written out;
- L10, for the `on` advance.
