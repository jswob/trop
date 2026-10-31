# HH4-b: the household reality, worked by hand

Evaluator D8-HH4-b.

**What I read.** `tmp/design/LANGUAGE.md`, `tmp/design/ENGINE.md` and the thirteen household files named in the brief:
- reality, people, places, pays and loans;
- asks/core;
- facts/2026;
- records/mbank/2026-09;
- law/pl: benefits, business, common, employment and wrappers.

I opened nothing else in the repo.

**Setup.**
- Cut K is the model's `on`, 2026-10-19 (LANGUAGE §4.5). No ask in asks/core.tally has `as known`.
- The run starts at S = 2026-01-01 and ends at E = 2059-12-31.
- All money is in PLN.

**Arithmetic.** Every step is a one-line `python3 -c` with `Decimal` or `Fraction`, or a `bc` line, and each is printed below with its output. There are no loops or scripts, and no files beyond this one and the scratch notes.
- The one-liners use small lambdas, lists and dict literals, plus `min()`. None of them loops.
- Two earlier passes in the session had errors, which were found and fixed (§9). Every figure in this file comes from the commands printed here.

How the file runs:
- §1–§8 give the steps, each with the section of the definition it applies.
- §9 says where I stopped and lists the corrections.
- §10 gives the outputs.
- §11 lists the ambiguities, with each choice computed.

---

## 1. Calendar, pay dates and standing figures

### Calendar

The built-ins are in LANGUAGE §4.2:
- `end_of(k)` is the first day after k, and `days(a, b)` is b − a.
- `day_of` clamps to the month.
- `next_business` skips Saturdays, Sundays and the days in `holidays_pl`.

2026-01-01 is a Thursday. The next line gives the offsets of the 1st of Feb..Dec 2026 and of 2027-01-01:

```
$ echo "31%7; 59%7; 90%7; 120%7; 151%7; 181%7; 212%7; 243%7; 273%7; 304%7; 334%7; 365%7" | bc
3 3 6 1 4 6 2 5 0 3 5 1
```

That makes the 1st of each 2026 month: Jan Thu, Feb Sun, Mar Sun, Apr Wed, May Fri, Jun Mon, Jul Wed, Aug Sat, Sep Tue, Oct Thu, Nov Sun, Dec Tue. 2027-01-01 is a Friday.

The next line gives 2027 and early 2028 (0 = Sunday), and also checks the VAT and the 2026 taxes paid:

```
$ echo "(5+31)%7; (5+59)%7; (5+90)%7; (5+120)%7; (5+151)%7; (5+181)%7; (5+212)%7; (5+243)%7; (5+273)%7; (5+304)%7; (5+334)%7; (5+365)%7; (6+31)%7; (6+60)%7; (6+91)%7" | bc | tr '\n' ' '; echo; python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; v=(14000*D('0.23')+3000*D('0.23')-92).quantize(D(1),H); print('vat_due',v,'VAT paid 2026 (Mar-Nov)',9*v,'ryczalt paid 2026',2040+3*2010+3*1990+2*1935,'taxes_paid 2026',9*v+2040+3*2010+3*1990+2*1935)"
1 1 4 6 2 4 0 3 5 1 3 6 2 3 6
vat_due 3818 VAT paid 2026 (Mar-Nov) 34362 ryczalt paid 2026 17910 taxes_paid 2026 52272
```

That makes the 1st of each 2027 month: Feb Mon, Mar Mon, Apr Thu, May Sat, Jun Tue, Jul Thu, Aug Sun, Sep Wed, Oct Fri, Nov Mon, Dec Wed. In 2028 the 1st is Jan Sat, Feb Tue, Mar Wed and Apr Sat.

### Pay dates

| Pay | Day | Moved to a business day? |
|---|---|---|
| `rent_pay[m]` | the 1st of m | no |
| `costs_pay[m]` | the 5th of m | yes |
| `loan_instalment` / `loan_interest[laptop, k]` | the 5th of 2026-06 + (k − 1), k = 1..10 | no |
| `loan_out[laptop]` | 2026-05-05, from loan_debt to sellers (konto not touched) | no |
| `wage[wife_acme, m]` | the 10th of m + 1 | yes |
| `health_pay`, `zus_social_pay`, `ryczalt_pay[m]` (together, "the group") | the 20th of m + 1 | yes |
| `vat_pay[m]` | the 25th of m + 1 | yes |
| `client_pay[c, m]` | `issue_day[m]` (the last day of m) + terms (A 14, B 30) | yes |
| `invest[w, m]` | the 15th, phase 6 | no |
| `buy_units` / `buy_cash` | the 16th, phases 4 and 5 | no |
| `spend[d]` | every day | no |

These are the moved dates the walk depends on:

**2026**
- costs: Apr 7 (Apr 5 is a Sunday and Apr 6 a holiday), Jul 6, Sep 7, Dec 7.
- wage: May 11, Oct 12.
- group: Jun 22, Sep 21, Dec 21.
- VAT: Apr 27, Jul 27, Oct 26, Dec 28.
- client_a: [May] Jun 15, [Oct] Nov 16.
- client_b: [Apr] Jun 1, [Jul] Aug 31.

**2027**
- wage: Jan 11, Apr 12, Jul 12, Oct 11.
- group: Feb 22, Mar 22, Jun 21, Nov 22.
- VAT: Apr 26, Jul 26, Sep 27, Dec 27.
- costs: Jun 7, Sep 6, Dec 6.
- client_a: [Jan] Feb 15, [Feb] Mar 15, [Jul] **Aug 16** (after the Aug 15 invest), [Oct] Nov 15.
- client_b: [Dec 2026] Feb 1, [Jan] Mar 2, [Apr] May 31, [Sep] **Nov 2** (Oct 30 is a Saturday and Nov 1 a holiday).

**2028**
- costs: Feb 7, Mar 6.
- group: Feb 21.
- VAT: Mar 27.
- client_b: [Dec 2027] Jan 31, [Jan] Mar 1.
- `return_day[2027]` is 2028-05-02, since 2028-05-01 is a holiday.

### Standing figures

These follow the read rule (§4.6), the law date (§4.3) and rounding (§7.2).

**Rent and costs**
- Rent is 3200.00 in 2026–2027 and 3296.00 in 2028: `prices 2026 by cpi` (§3.4), with cpi[2027] = 1 and cpi[2028] = 1.03.
- Costs are 400 + 92 = 492 from 2026-03. dg_2026 is keyless and known 2026-02-20, so it beats forecast_2026's assumed 0 by the later known (§4.6 2.3).

**Fees, from 2026-03**
- `fee_net[client_a]` = 14000 states a key, so it beats the keyless assumed 0. Gross is 17220, of which VAT is 3220.
- client_b: 3000, gross 3690, VAT 690.
- `vat_due[m]` = round(3220 + 690 − 92) = 3818.

**Holds and spending**
- The hold `vat_owed[d]` covers a client's VAT from its `pay_day` until `vat_day`. client_b pays after `vat_day`, so its VAT is never held. Only client_a's 3220 is held, from its pay day to the 25th.
- `spend_day` on the budget (4600 a month) is 148.39, 164.29 or 153.33.
- `spend_day` learned from the September statement applies from cut 2026-10-02: 3574.02 a month, so 115.29, 127.64 or 119.13. In 2028 it is × 1.03: 118.75, 126.94 (29 days) or 122.71.

**Loans and Marek**
- The laptop instalment is 300 under the `zero` rule (3000/10), with interest 0.
- Marek:
  - `lend1` −2000 on 2026-03-12;
  - `back1` +700 on 2026-06-30;
  - `back2` +500 on 2026-09-15;
  - `marek_last` +800 on 2026-12-31, **promised**.

```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; c=lambda x:D(x).quantize(D('0.01'),H); q=lambda x:D(x).quantize(D(1),H); print('spend/day budget 31/28/30',c(D(4600)/31),c(D(4600)/28),c(D(4600)/30),'learned sum',D('412.37')+D('286.15')+D('530.80')+D('348.62')+D('401.09'),250+180,145,D('899.99')+120,'total',D('1979.03')+430+145+D('1019.99'),'per day 31/28/30',c(D('3574.02')/31),c(D('3574.02')/28),c(D('3574.02')/30),'| health_settle 2026',10*D('830.58')-3*D('498.35')-7*D('830.58'),'2027',12*D('863.80')-3*D('518.28')-9*D('863.80'),'2028',12*D('898.35')-3*D('539.01')-9*D('898.35'),'| pit37 wife base',q(12*(7000-D('959.70')-250)),'tax',q(q(12*(7000-D('959.70')-250))*D('0.12')-3600),'due',q(q(12*(7000-D('959.70')-250))*D('0.12')-3600)-12*395)"
spend/day budget 31/28/30 148.39 164.29 153.33 learned sum 1979.03 430 145 1019.99 total 3574.02 per day 31/28/30 115.29 127.64 119.13 | health_settle 2026 996.69 2027 1036.56 2028 1078.02 | pit37 wife base 69484 tax 4738 due -2
```

The learned figure is `spend_month[c]`, rule `learned` (`excepts budget`, `when spend_window != - and covered(konto, spend_window)`, §5.18 "Reading history"). It sums −amount of the unexplained September rows by category: groceries 1979.03, transport 430, health_care 145, other 1019.99.

---

## 2. Figures outside the walk

### 2.1 wife_net (LANGUAGE §5.10, §7.2; employment.tally)

**The deductions**
- `emp_zus` = round(7000 × 0.0976) + round(7000 × 0.015) + round(7000 × 0.0245) = 683.20 + 105.00 + 171.50 = 959.70. The capped base is min(salary, zus_cap − ytd), which never binds.
- `emp_health` = round((7000 − 959.70) × 0.09) = 543.63.
- `emp_pit_base` = round(5790.30, 1) = 5790.

**The PIT advance**
- `emp_pit_adv` = max(0, round(pit_scale(cum) − pit_scale(cum − 5790) − 300, 1)) = round(694.80 − 300) = 395.
- This holds in every month: the cumulative base stays in the first band row (12 × 5790 = 69480 ≤ 120000).
- It holds in January too, where pit_scale(0) = 0 × 0.12 − 3600 by the band's first row (`[0, 120000]`).

**The result and the 2027 inputs**
- **emp_net = 5101.67 for 2026-10 and for 2027-01.** It equals the September record `ACME ... WYNAGR 08/2026`.
- 2027-01 reads `proj_wage_y[2027]` = round(9420 × 1.04) = 9796.80 through rule `projected`. `missing(proj_wage)` is true for 2027, since proj_wage_2026 governs only 2026 (§5.10). So `zus_cap[2027]` = 293904.00.

```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; c=lambda x:D(x).quantize(D('0.01'),H); z=c(7000*D('0.0976'))+c(7000*D('0.015'))+c(7000*D('0.0245')); h=c((7000-z)*D('0.09')); b=(7000-z-250).quantize(D(1),H); p=(b*D('0.12')-300).quantize(D(1),H); pw=c(9420*D('1.04')); print('emp_zus',z,'emp_health',h,'pit base',b,'advance (0.12*base-300)',p,'emp_net',7000-z-h-p,'| proj_wage 2027',pw,'zus_cap 2027',30*pw,'ike_limit 2026/2027/2028',3*9420,3*pw,3*c(pw*D('1.04')))"
emp_zus 959.70 emp_health 543.63 pit base 5790 advance (0.12*base-300) 395 emp_net 5101.67 | proj_wage 2027 9796.80 zus_cap 2027 293904.00 ike_limit 2026/2027/2028 28260 29390.40 30566.01
```

**Registers (§8.2)**
- **2026-10:** law date 2026-10-01, cut 2026-10-19, nothing else. Every row is read at a law date before K.
- **2027-01:**
  - held: `salary[wife_acme]` (home_2026, read for 2027-01), the `ee_rate` rows (sus_2026), `kup_month`, `pit_reduction_month` and `pit_scale` (pit_2022), and `health_rate` (health_2022);
  - `assumed: indeks płac +4% rocznie, założenie prototypu` and `missing record counted by wage_fc_2026`, both through `zus_cap[2027]`.

### 2.2 social_and_health (business.tally, common.tally)

| Month | Stage | `zus_base` | `zus_social` | `rev_ytd`, tier | `health_base_y` | `health_month` |
|---|---|---|---|---|---|---|
| 2026-10 | preferential | round(4806 × 0.3) = 1441.80 | **456.18** | 136000, mid | 9228.64 | **830.58** |
| 2027-01 | preferential | round(4998.24 × 0.3) = 1499.47 | **474.44** | 17000, low | 9597.79 | **518.28** |
| 2028-09 | standard | round(10188.67 × 0.6) = 6113.20 | **2083.99** (with labour_fund) | 153000, mid | 9981.70 | **898.35** |

Each part is rounded separately (`zus_social` sums the rounded parts). Pension 0.1952, disability 0.08, sickness 0.0245 (`sickness_opt` = true) and accident 0.0167 apply in both stages. labour_fund 0.0245 applies in standard only.

```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; c=lambda x:D(x).quantize(D('0.01'),H); s=lambda b:c(b*D('0.1952'))+c(b*D('0.08'))+c(b*D('0.0245'))+c(b*D('0.0167')); w7=c(4806*D('1.04')); w8=c(w7*D('1.04')); p7=c(9420*D('1.04')); p8=c(p7*D('1.04')); b6=c(4806*D('0.3')); b7=c(w7*D('0.3')); b8=c(w8*D('0.3')); bs=c(p8*D('0.6')); h7=c(D('9228.64')*D('1.04')); h8=c(h7*D('1.04')); print('2026-10 base',b6,'zus',s(b6),'| 2027-01 min_wage',w7,'base',b7,'zus',s(b7),'| 2028 pref base',b8,'zus',s(b8),'| 2028-09 proj_wage',p8,'base',bs,'zus',s(bs)+c(bs*D('0.0245')),'| health base 2027/2028',h7,h8,'| health 2026 low/mid',c(D('0.054')*D('9228.64')),c(D('0.09')*D('9228.64')),'2027',c(D('0.054')*h7),c(D('0.09')*h7),'2028',c(D('0.054')*h8),c(D('0.09')*h8))"
2026-10 base 1441.80 zus 456.18 | 2027-01 min_wage 4998.24 base 1499.47 zus 474.44 | 2028 pref base 1559.45 zus 493.41 | 2028-09 proj_wage 10188.67 base 6113.20 zus 2083.99 | health base 2027/2028 9597.79 9981.70 | health 2026 low/mid 498.35 830.58 2027 518.28 863.80 2028 539.01 898.35
```

**Registers**
- **2026-10:** law date and cut only.
- **2027-01:**
  - `assumed` wage_growth and `missing record counted by wage_fc_2026`, from `min_wage_y` and `health_base_y` (rule `projected`);
  - held: `pref_share`, `zus_rate`, `zus_due`, `health_rate`, `health_share`, the `health_tier` band and `fee_net`.
  - `zus_stage[2027-01]` sits in a closed span (2026-09..2028-08), so it isn't held.
- **2028-09:** the same, plus held `zus_stage = standard` (an open span from 2028-09).

### 2.3 pit37 (employment.tally)

- **me:** there is no job, so `pit37_base` = 0. pit_scale(0) = −3600, so the tax is max(0, −3600) = 0 and the advances are 0. **0.00** for 2026, 2027 and 2028.
- **wife:**
  - the base is round(12 × 5790.30) = 69484;
  - the tax is round(0.12 × 69484 − 3600) = 4738;
  - the advances are 12 × 395 = 4740.

  That gives **−2.00** in each year: the inputs don't change, and zus_cap never binds. `child_relief` is 0 (parent = false), and neither the `unused` nor the `abroad` rule applies.
- So `pit37_refund[wife, y]` pays 2 on `return_day[y]` + 45: 2027-06-14 and 2028-06-16.

### 2.4 pit28 and the ryczałt advances (business.tally)

`ryczalt_adv[m]` = round(round(17000 − flows(zus_social_pay, m) − 0.5 × flows(health_pay, m), 1) × 0.12, 1). `flows(p, span)` over a month counts the movements dated in that month (§5.13.3). So it deducts the contributions paid in m, which are those for m − 1.

**2026**

| Advance month | What was paid in that month | Advance |
|---|---|---|
| Mar | nothing | 2040 |
| Apr–Jun | health low 498.35, ZUS relief 0 | 2010 |
| Jul–Sep | health mid 830.58 | 1990 |
| Oct–Dec | ZUS 456.18 + health 830.58 | 1935 |

- The ten advances sum to 19845. All of them are paid by `return_day[2026]` = 2027-04-30.
- `pit28_tax[2026]` = round(round(170000 − 1368.54 − 0.5 × 6478.53) × 0.12) = 19847.
- **pit28_due[2026] = 2.00**, so `pit28_pay[2026]` pays 2 on 2027-04-30.

```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; r=lambda x:D(x).quantize(D(1),H); h=lambda b:r(r(max(0,b))*D('0.12')); a3=h(17000); a4=h(17000-D('498.35')/2); a7=h(17000-D('830.58')/2); a10=h(17000-D('456.18')-D('830.58')/2); adv=a3+3*a4+3*a7+3*a10; tax=r(r(170000-3*D('456.18')-(3*D('498.35')+6*D('830.58'))/2)*D('0.12')); print('adv Mar',a3,'Apr-Jun',a4,'Jul-Sep',a7,'Oct-Dec',a10,'sum 10 months',adv,'paid in 2026 (Mar-Nov)',adv-a10,'tax',tax,'pit28_due 2026',tax-adv)"
adv Mar 2040 Apr-Jun 2010 Jul-Sep 1990 Oct-Dec 1935 sum 10 months 19845 paid in 2026 (Mar-Nov) 17910 tax 19847 pit28_due 2026 2
```

**2027 and 2028**

```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; r=lambda x:D(x).quantize(D(1),H); h=lambda z,g:r(r(17000-z-g/2)*D('0.12')); j=h(D('456.18'),D('830.58')); f=h(D('474.44'),D('518.28')); y=h(D('474.44'),D('863.80')); a27=j+3*f+8*y; t27=r(r(204000-D('456.18')-11*D('474.44')-(D('830.58')+3*D('518.28')+8*D('863.80'))/2)*D('0.12')); p27=j+j+3*f+7*y; print('2027 adv Jan',j,'Feb-Apr',f,'May-Dec',y,'sum',a27,'tax',t27,'pit28_due',t27-a27,'ryczalt paid in 2027',p27,'VAT paid',12*3818,'taxes_paid 2027',p27+12*3818+2-2)"
2027 adv Jan 1935 Feb-Apr 1952 May-Dec 1931 sum 23239 tax 23241 pit28_due 2 ryczalt paid in 2027 23243 VAT paid 45816 taxes_paid 2027 69059

$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; r=lambda x:D(x).quantize(D(1),H); h=lambda z,g:r(r(17000-z-g/2)*D('0.12')); a=h(D('474.44'),D('863.80')); b=h(D('493.41'),D('539.01')); c=h(D('493.41'),D('898.35')); e=h(D('2083.99'),D('898.35')); a28=a+3*b+5*c+3*e; base=204000-D('474.44')-8*D('493.41')-3*D('2083.99')-(D('863.80')+3*D('539.01')+8*D('898.35'))/2; t28=r(r(base)*D('0.12')); p28=a+a+3*b+5*c+2*e; print('2028 adv Jan',a,'Feb-Apr',b,'May-Sep',c,'Oct-Dec',e,'sum',a28,'base',base,'tax',t28,'pit28_due',t28-a28,'ryczalt paid in 2028',p28,'+VAT',p28+12*3818,'+oki_tax 368',p28+12*3818+2-2+368)"
2028 adv Jan 1931 Feb-Apr 1948 May-Sep 1927 Oct-Dec 1736 sum 22618 base 188492.495 tax 22619 pit28_due 1 ryczalt paid in 2028 22813 +VAT 68629 +oki_tax 368 68997
```

- **pit28_due[2027] = 2.00.** It is paid on 2028-05-02.
- **pit28_due[2028] = 1.00.** The 2028 base, 188492.495, rounds half-up to the złoty as 188492.

### 2.5 health_settlement (business.tally)

`health_year[y]` = active_months × round(0.09 × share(tier of `revenue_year`) × `health_base_y`). `health_paid[y]` is the year's `health_pay` instances paid by `health_settle_day[y]`.

| Year | `active_months` | Year tier | `health_year` | `health_paid` | `health_settle` |
|---|---|---|---|---|---|
| 2026 | 10 | mid (170000) | 8305.80 | 3 × 498.35 + 7 × 830.58 = 7309.11 | **996.69** |
| 2027 | 12 | mid | 10365.60 | 3 × 518.28 + 9 × 863.80 = 9329.04 | **1036.56** |
| 2028 | 12 | mid | 10780.20 | 3 × 539.01 + 9 × 898.35 = 9702.18 | **1078.02** |

The figures come from the command in §1. `health_settle_pay[2026]` pays 996.69 from konto on `health_settle_day[2026]` = 2027-05-20, a Thursday.

### 2.6 taxes_paid, 2026 and 2027

`flows(urzad_skarbowy, y)` = everything into the tax office minus everything out (§5.13.3).

**2026: 52272.00**
- ryczałt paid in 2026, the advances for Mar–Nov: 17910;
- VAT for Mar–Nov: 9 × 3818 = 34362.

The command is in §1.

**2027: 69059.00**
- ryczałt paid: 23243 (Dec 2026's 1935, plus Jan–Nov 2027);
- VAT: 45816;
- `pit28_pay[2026]`: +2;
- `pit37_refund[wife, 2026]`: −2.

The command is in §2.4. 2028 needs `oki_tax[2027]` (§8).

---

## 3. The walk, Jan–Aug 2026 (no statement covers these months, so the plan is booked)

### Rules applied

**Order and rule picking**
- Records and flow pays of a day are booked first (§5.13.2; ENGINE §4.2 steps 1–2).
- State pays come next, by phase, then pay id, then keys in index order:
  - `buy_units` (phase 4) and `buy_cash` (phase 5) on the 16th;
  - `invest` (phase 6) on the 15th, in the order ike, oki, mak.
- `invest_amt` reads `room`, and `room` reads `plan_low(konto, 15th, 14th of next month) − keep_floor` (1000).
- `room` is walk-bound, so it is re-read at each invest's point (§5.13.1; ENGINE §3.2). After ike books, oki and mak see the lower balance.

**The look-ahead**
- `plan_low` = bal − P + min over u in [t, b] of (F(u) − F(t) − h(u)) (ENGINE §4.2; LANGUAGE §5.13.1). In it:
  - F is the planned flow movements at cut min(t, K);
  - h is the hold `vat_owed` (`konto ... holds vat_owed`, §5.9);
  - P is the promised movements booked so far that no record confirms (§5.13 `promised`).
- Day t is in the window ("on days a..b", with a = `day_of(m, 15)`). Its value is −h(t).
- F only falls between inflows, so the minimum is on the day before an inflow, or on b. Each command lists those candidates by date.
- State pays are left out of F, so the invests take exactly room. After them the balance is A = keep_floor + P − min, whenever room > 0.

**Which wrapper gets room**
- ike's cap reads `flows(invest[ike], start_of(year)..day_of(m, 15))`. That counts what is booked up to this point of the day ("Same-day totals", §5.13.1), and not ike's own instance.
- mak takes room while `oki_from` is `-` or m < `oki_from`:
  - at cuts before 2026-07-15, `oki_from` is forecast_2026's assumed `-`;
  - from 2026-07-15 it is 2027-01 (oki_start).

**Cuts that matter**
- **Feb 15.** dg_2026 (known Feb 20) is not yet visible, so:
  - `cost_net` is still the assumed 0, `costs_pay`'s `when` is false, and the pay is dropped;
  - `fee_net` is the assumed 0, so there are no client pays;
  - `zus_stage` for 2026-03 on is a hole. Every instance whose `when` reads that hole is dated after Mar 14, so it is dropped by its date "whatever its `when`" (§5.13.1), and `plan_low` is no hole.
- `marek_lend[lend1]` isn't in the Feb window: its key is added on 2026-03-12 (open index, §5.6).
- `marek_back[back1]` isn't in the Jun window either: it is known Jun 30.

### Results

| Invest on | B before | Window minimum (day) | `plan_low` | room | ike | mak | Month-end balance |
|---|---|---|---|---|---|---|---|
| 2026-01-15 | 18574.15 | −7052.85 (Feb 9) | 11521.30 | 10521.30 | 10521.30 | 0 | Jan 31: 5678.61 |
| 2026-02-15 | 5115.93 | −6671.28 (Mar 9) | −1555.35 | 0 | 0 | 0 | Feb 28: 2980.16 |
| 2026-03-15 | 163.98 | −7446.21 (Apr 9) | −7282.23 | 0 | 0 | 0 | Mar 31: −2210.26 |
| 2026-04-15 | 14119.46 | −10142.20 (May 10) | 3977.26 | 2977.26 | 2977.26 | 0 | Apr 30: 6175.90 |
| 2026-05-15 | 22579.72 | −10382.56 (Jun 9) | 12197.16 | 11197.16 | 11197.16 | 0 | May 31: 2681.97 |
| 2026-06-15 | 22401.69 | −10263.81 (Jul 9) | 12137.88 | 11137.88 | 3564.28 (cap) | 7573.60 | Jun 30: 7027.51 |
| 2026-07-15 | 23131.33 | −10670.33 (Aug 9) | 12461.00 | 11461.00 | 0 | 11461.00 | Jul 31: 6327.51 |
| 2026-08-15 | 22431.33 | −10694.79 (Sep 9) | 11736.54 | 10736.54 | 0 | 10736.54 | Aug 31: 6371.97 |

**Notes on the table**
- **WARN-NEGATIVE** prints on 2026-03-01 (−368.23) and again on 2026-03-17 (−132.80): one per stretch (ENGINE §4.2 step 5).
- The June cap: 28260 − 10521.30 − 2977.26 − 11197.16 = 3564.28. The June balance before the invest includes client_a[May] +17220, booked on Jun 15 before the invest (flow pays first). The June 30 balance includes Marek's +700.

```
$ python3 -c "from decimal import Decimal as D; a=D('148.39'); f=D('164.29'); B1=24000-3200-15*a; m1=-16*a-3200-9*f; r1=B1+m1-1000; A1=B1-r1; B2=A1-16*a-3200-15*f+D('5101.67'); m2=-13*f-3200-9*a; print('Jan15 B',B1,'min Feb9',m1,'Feb14',m1+D('5101.67')-5*f,'room=ike',r1,'A',A1,'Jan31',A1-16*a,'| Feb15 B',B2,'min Mar9',m2,'plan_low',B2+m2,'room 0','Feb28',B2-13*f)"
Jan15 B 18574.15 min Feb9 -7052.85 Feb14 -2772.63 room=ike 10521.30 A 8052.85 Jan31 5678.61 | Feb15 B 5115.93 min Mar9 -6671.28 plan_low -1555.35 room 0 Feb28 2980.16

$ python3 -c "from decimal import Decimal as D; a=D('148.39'); n=D('153.33'); w=D('5101.67'); B3=D('2980.16')-3200-15*a-492+w-2000; m3=-16*a-3692-9*n; M31=B3-16*a; B4=M31-3200-15*n-492+w+17220; g4=D('498.35')+2040; T4=-15*n-g4-3818+3690; c4=[-3220,-11*n-g4-3220,-14*n-g4-3818,T4,T4-3692-10*a]; m4=min(c4); r4=B4+m4-1000; A4=B4-r4; print('Mar1',D('2980.16')-3200-a,'Mar17',B3-2*a,'Mar15 B',B3,'min Apr9',m3,'plan_low',B3+m3,'room 0','Mar31',M31,'| Apr15 B',B4,'cands',c4,'min',m4,'room=ike',r4,'A',A4,'Apr30',A4+T4)"
Mar1 -368.23 Mar17 -132.80 Mar15 B 163.98 min Apr9 -7446.21 plan_low -7282.23 room 0 Mar31 -2210.26 | Apr15 B 14119.46 cands [-3220, Decimal('-7444.98'), Decimal('-8502.97'), Decimal('-4966.30'), Decimal('-10142.20')] min -10142.20 room=ike 2977.26 A 11142.20 Apr30 6175.90

$ python3 -c "from decimal import Decimal as D; a=D('148.39'); n=D('153.33'); w=D('5101.67'); B5=D('6175.90')-3200-15*a-492+w+17220; g5=D('498.35')+2010; T5=-16*a-g5-3818; c5=[-3220,-9*a-g5-3220,T5,T5+490-n,T5+490-792-9*n,T5+490-792-14*n+w]; m5=min(c5); r5=B5+m5-1000; A5=B5-r5; B6=A5+T5+490-15*n-792+w+17220; g6=D('498.35')+2010; T6=-15*n-g6-3818+3690; c6=[-3220,-9*n-g6-3220,-14*n-g6-3818,T6,T6-3992-9*a]; m6=min(c6); r6=B6+m6-1000; ike=28260-D('10521.30')-D('2977.26')-r5; print('May15 B',B5,'cands',c5,'room=ike',r5,'A',A5,'May31',A5+T5,'| Jun15 B',B6,'cands',c6,'min',m6,'room',r6,'ike',ike,'mak',r6-ike,'Jun30',B6-r6+T6+700)"
May15 B 22579.72 cands [-3220, Decimal('-7063.86'), Decimal('-8700.59'), Decimal('-8363.92'), Decimal('-10382.56'), Decimal('-6047.54')] room=ike 11197.16 A 11382.56 May31 2681.97 | Jun15 B 22401.69 cands [-3220, Decimal('-7108.32'), Decimal('-8472.97'), Decimal('-4936.30'), Decimal('-10263.81')] min -10263.81 room 11137.88 ike 3564.28 mak 7573.60 Jun30 7027.51

$ python3 -c "from decimal import Decimal as D; a=D('148.39'); n=D('153.33'); w=D('5101.67'); B7=D('7027.51')-3200-15*a-792+w+17220; g7=D('830.58')+2010; T7=-16*a-g7-3818+3690; c7=[-3220,-11*a-g7-3220,-14*a-g7-3818,T7,T7-3992-9*a]; m7=min(c7); r7=B7+m7-1000; A7=B7-r7; B8=A7+T7-3200-15*a-792+w+17220; g8=D('830.58')+1990; T8=-16*a-g8-3818+3690; c8=[-3220,-9*a-g8-3220,-15*a-g8-3818,T8,T8-3992-9*n]; m8=min(c8); r8=B8+m8-1000; A8=B8-r8; print('Jul15 B',B7,'cands',c7,'room=mak',r7,'Jul31',A7+T7,'| Aug15 B',B8,'cands',c8,'min',m8,'room=mak',r8,'Aug31',A8+T8)"
Jul15 B 23131.33 cands [-3220, Decimal('-7692.87'), Decimal('-8736.04'), Decimal('-5342.82'), Decimal('-10670.33')] room=mak 11461.00 Jul31 6327.51 | Aug15 B 22431.33 cands [-3220, Decimal('-7376.09'), Decimal('-8864.43'), Decimal('-5322.82'), Decimal('-10694.79')] min -10694.79 room=mak 10736.54 Aug31 6371.97
```

**How to read the candidate lists, in order**
1. Day t, with its hold (−3220 when client_a has paid and the VAT isn't due yet).
2. The last day of the hold, before the VAT payment.
3. The day before client_b pays.
4. The end of the month.
5. The day before the wage arrives.

"3692" is rent + costs; "3992" adds the laptop instalment; "490" is client_b's 3690 less the rent, both on the 1st.

---

## 4. September 2026, covered by `mbank_2026_09` (LANGUAGE §5.18; ENGINE §4.2, §6.1, §6.2)

**Coverage**
- The version is known 2026-10-02, which is ≤ K, and `covers konto` over 2026-09.
- On 2026-09-01..30 konto's records are booked.
- Every model pay that touches konto is **modelled past**: computed for the ledger and never booked (N-1202).

**How the records are booked**
- The eight records explained by flow pays are booked as those instances, between the pay's places: rent, costs, `wage[Aug]`, `client_a[Aug]`, `marek_back[back2]` (so marek's claim falls by 500), `health_pay[Aug]`, `ryczalt_pay[Aug]` and `vat_pay[Aug]`.
- The ten unexplained records (−3574.02 in all) are booked between konto and the outside place.
- The record explaining `invest[ike, 2026-09]` (−6000) is booked at that instance's point, after the instance is computed (ENGINE §4.2 steps 1 and 3).

**The Sep 15 state pays (modelled past)**
- They are computed against the booked balances. konto stands at 24408.12 at ike's point.
- The look-ahead at cut Sep 15 counts the plan's instances, "whether booked or modelled past", and never later records (ENGINE §4.2). So it uses budget spend at 153.33 and 148.39, and client_b[Aug] +3690 on Sep 30.
- The minimum is −10872.82 on Oct 11, so `plan_low` is 13535.30 and room is 12535.30.
- **ike** is min(12535.30, max(0, 28260 − 28260)) = **0**. Then the −6000 record is booked.
- **oki** is 0.
- **mak** is room re-read: 24408.12 − 6000 − 10872.82 − 1000 = **6535.30**. It is modelled, not booked.

**The close of Sep 30 (anchor)**
- The booked balance is 9289.04. The stated balance is 41386.27.
- **The balance line is +32097.23.** From the close of Sep 30, konto is 41386.27.

**Loose ends**
- `loan_instalment[laptop, 4]` on Sep 5 is modelled past, so it is booked on neither konto nor `loan_debt` (ambiguity B).
- `zus_social_pay[2026-08]` doesn't exist: in relief `zus_base` is 0, so its `when` is false.

```
$ python3 -c "from decimal import Decimal as D; a=D('148.39'); n=D('153.33'); B=D('6371.97')-3200-D('412.37')-250-492-D('286.15')+D('5101.67')-145+17220+500; g=D('830.58')+1990; T=-15*n-g-3818+3690; c=[-3220,-9*n-g-3220,-14*n-g-3818,T,T-3992-11*a,T-3992-12*a+D('5101.67')]; m=min(c); r=B+m-1000; mak=B-6000+m-1000; late=-D('530.80')-180-D('899.99')-D('830.58')-1990-D('348.62')-3818-120-D('401.09'); bk=B-6000+late; print('Sep15 B at ike point',B,'cands',c,'min',m,'plan_low',B+m,'room',r,'ike 0; mak modelled',mak,'records Sep16-30',late,'booked Sep30',bk,'balance line',D('41386.27')-bk)"
Sep15 B at ike point 24408.12 cands [-3220, Decimal('-7420.55'), Decimal('-8785.20'), Decimal('-5248.53'), Decimal('-10872.82'), Decimal('-5919.54')] min -10872.82 plan_low 13535.30 room 12535.30 ike 0; mak modelled 6535.30 records Sep16-30 -9119.08 booked Sep30 9289.04 balance line 32097.23
```

---

## 5. replay_september: `residuals konto 2026-09` (LANGUAGE §5.16; ENGINE §6.3)

**How it is printed**
- K = `on`, so this is the replay.
- Amounts are signed as flows into konto:
  - a matched line shows records − model;
  - an expected-but-missing line shows −model;
  - an unexplained line shows the record itself;
  - the balance line shows stated − booked.
- Every line falls in month 2026-09.
- Groups are sorted by |difference|, and lines within a group the same way. Ties go by month, then group name.

| Group | Model | Records | Difference | Lines |
|---|---|---|---|---|
| balance | 9289.04 (booked) | 41386.27 (stated) | **+32097.23** | `stated[konto]` on 2026-09-30 |
| spend | −4599.90 | 0.00 | **+4599.90** | 30 expected-missing lines: `spend[2026-09-01..30]`, +153.33 each |
| client_pay | +20910.00 | +17220.00 | **−3690.00** | `client_pay[client_b, 2026-08]` expected-missing, 2026-09-30, −3690.00; `client_pay[client_a, 2026-08]` matched, 0.00, timing 0 |
| unexplained | 0.00 | −3574.02 | **−3574.02** | −899.99 MEDIA EXPERT, −530.80 AUCHAN, −412.37 BIEDRONKA, −401.09 BIEDRONKA, −348.62 BIEDRONKA, −286.15 LIDL, −250.00 ORLEN, −180.00 PKP, −145.00 APTEKA, −120.00 RESTAURACJA |
| invest | −6535.30 | −6000.00 | **+535.30** | `invest[mak, 2026-09]` expected-missing, +6535.30; `invest[ike, 2026-09]` matched, model 0.00, record −6000.00, difference −6000.00, timing 0 |
| loan_instalment | −300.00 | 0.00 | **+300.00** | `loan_instalment[laptop, 4]`, 2026-09-05, expected-missing |
| costs_pay | −492.00 | −492.00 | 0.00 | matched, timing 0 (Sep 7) |
| health_pay | −830.58 | −830.58 | 0.00 | matched, timing 0 (Sep 21) |
| marek_back | +500.00 | +500.00 | 0.00 | matched, timing 0 |
| rent_pay | −3200.00 | −3200.00 | 0.00 | matched, timing 0 |
| ryczalt_pay | −1990.00 | −1990.00 | 0.00 | matched, timing 0 (Sep 21) |
| vat_pay | −3818.00 | −3818.00 | 0.00 | matched, timing 0 |
| wage | +5101.67 | +5101.67 | 0.00 | matched, timing 0 |

These instances print nothing: `loan_interest[laptop, 4]` (amount 0, no record) and `invest[oki, 2026-09]` (0). "An instance of amount 0 with no record is no movement and prints nothing."

**Check.** The differences other than the balance line sum to −1828.82, which is records net (+2917.07) minus model net (+4745.89).

```
$ python3 -c "from decimal import Decimal as D; rec=-3200-D('412.37')-250-492-D('286.15')+D('5101.67')-145+17220+500-6000-D('530.80')-180-D('899.99')-D('830.58')-1990-D('348.62')-3818-120-D('401.09'); mod=-3200-492+D('5101.67')+17220+500+0-D('6535.30')-D('830.58')-1990-3818+3690-300-30*D('153.33'); un=-D('412.37')-250-D('286.15')-145-D('530.80')-180-D('899.99')-D('348.62')-120-D('401.09'); print('records net',rec,'model net',mod,'records-model',rec-mod,'| groups: spend',30*D('153.33'),'client_pay',-3690,'unexplained',un,'invest',-6000+D('6535.30'),'loan_instalment',300,'sum',30*D('153.33')-3690+un+D('535.30')+300,'| balance line',D('41386.27')-D('6371.97')-rec)"
records net 2917.07 model net 4745.89 records-model -1828.82 | groups: spend 4599.90 client_pay -3690 unexplained -3574.02 invest 535.30 loan_instalment 300 sum -1828.82 | balance line 32097.23
```

**What the balance line means.** It is mostly the Jan–Aug modelled past: the model invested 61079.52 between January and August (rooms 10521.30 + 2977.26 + 11197.16 + 11137.88 + 11461.00 + 10736.54). The real account kept most of that money, and no statement covers those months.

---

## 6. Oct–Dec 2026, konto_balance for 2026, explain_room

### Booking and the walk

**Spend in October**
- `spend[2026-10-01]` is booked at its cut, min(Oct 1, K) = Oct 1, which is before spend_2026_09 is known. So it is the budget's 148.39.
- From Oct 2, spend is the learned 115.29 or 119.13.
- The Oct 2 versions change every spend instance booked before then by δ = −33.10. `arrears none for spend` books nothing for it (§5.13.3).

**Oct 15**
- B is 57953.49, built from the anchor.
- The minimum is −10313.57 on Nov 9, so room is 46639.92.
- All of it goes to mak, because ike is capped: `flows(invest[ike], 2026)` = 28260 + 6000 = 34260 > 28260.

**konto_balance 2026-10-31 = 6064.17.** `vat_owed` is 0 that day: client_a[Sep]'s VAT was paid on Oct 26, and client_a[Oct] arrives on Nov 16.

**Nov 15**
- B is 5386.89. The window's minimum is **0, on day t itself**: client_a[Oct] arrives Nov 16, and the end-of-day value of every later day is above 0.
- So `plan_low` is 5386.89, and room is 4386.89, all to mak.

**Dec 15**
- B is 29683.61. The minimum is −10339.30 on 2027-01-10, so room is 18344.31, all to mak.
- `marek_last` (promised, Dec 31) is left out of the window (§5.13).

**konto_balance 2026-12-31 = 6944.90.**
- It includes `marek_last` +800: `bal` counts a promised movement on its day (§5.13).
- `vat_owed[2026-12-31]` is 0, because client_a[Nov]'s VAT was paid on Dec 28.

```
$ python3 -c "from decimal import Decimal as D; a=D('148.39'); L=D('115.29'); N=D('119.13'); w=D('5101.67'); B=D('41386.27')-3200-a-14*L-792+w+17220; g=D('456.18')+D('830.58')+1990; T=-16*L-g-3818+3690; c=[-3220,-10*L-g-3220,-14*L-g-3818,T,T-3992-9*N,T-3992-14*N+w]; m=min(c); r=B+m-1000; A=B-r; print('Oct15 B',B,'cands',c,'min',m,'room=mak',r,'A',A,'konto 2026-10-31',A+T)"
Oct15 B 57953.49 cands [-3220, Decimal('-7649.66'), Decimal('-8708.82'), Decimal('-5249.40'), Decimal('-10313.57'), Decimal('-5807.55')] min -10313.57 room=mak 46639.92 A 11313.57 konto 2026-10-31 6064.17

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); w=D('5101.67'); B=D('6064.17')-3200-492-300+w-15*N; g=D('456.18')+D('830.58')+1935; T=17220-15*N-g-3818+3690; c={'t':D(0),'Nov16':17220-N-3220,'Nov24':17220-9*N-g-3220,'Nov29':17220-14*N-g-3818,'Nov30':T,'Dec9':T-3992-9*L,'Dec13':T-3992-13*L+w}; m=min(c.values()); print('Nov15 B',B,c,'min',m,'plan_low',B+m,'room',B+m-1000)"
Nov15 B 5386.89 {'t': Decimal('0'), 'Nov16': Decimal('13880.87'), 'Nov24': Decimal('9706.07'), 'Nov29': Decimal('8512.42'), 'Nov30': Decimal('12083.29'), 'Dec9': Decimal('7053.68'), 'Dec13': Decimal('11694.19')} min 0 plan_low 5386.89 room 4386.89

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); w=D('5101.67'); g=D('456.18')+D('830.58')+1935; B=1000+17220-15*N-g-3818+3690-3200-300-492+w-15*L+17220; T=-16*L-g-3818+3690; c={'t':D(-3220),'Dec27':-12*L-g-3220,'Dec29':-14*L-g-3818,'Dec31':T,'Jan10':T-3992-10*L,'Jan13':T-3992-13*L+w}; m=min(c.values()); r=B+m-1000; A=B-r; print('Dec15 B',B,c,'min',m,'plan_low',B+m,'room=mak',r,'A',A,'konto 2026-12-31 incl. promised 800',A+T+800)"
Dec15 B 29683.61 {'t': Decimal('-3220'), 'Dec27': Decimal('-7825.24'), 'Dec29': Decimal('-8653.82'), 'Dec31': Decimal('-5194.40'), 'Jan10': Decimal('-10339.30'), 'Jan13': Decimal('-5583.50')} min -10339.30 plan_low 19344.31 room=mak 18344.31 A 11339.30 konto 2026-12-31 incl. promised 800 6944.90
```

### explain_room: `explain room[2026-11]` (LANGUAGE §5.16; ENGINE §10)

`room` is walk-bound, so `explain` prints one cell for each walk point where a state pay read it, in walk order:

| # | Point (day, phase, pay) | Value |
|---|---|---|
| 1 | 2026-11-15, phase 6, `invest[ike, 2026-11]` | **4386.89** |
| 2 | 2026-11-15, phase 6, `invest[mak, 2026-11]` | **4386.89** |

**Which invests read `room`**
- ike reads `room` because `min(room[m], …)` evaluates both operands.
- ike books min(4386.89, max(0, 28260 − 34260)) = 0, so mak's re-read sees the same balance.
- `invest[oki, 2026-11]` doesn't read `room`: its `if` takes the `else 0` branch (m < oki_from = 2027-01), and an `if` reads only the branch it takes (§8.2).

**Each cell**
- **Rule:** `base` won. It is the def's only rule.
- **Value:** max(0, plan_low(konto, 2026-11-15, 2026-12-14) − keep_floor) = max(0, 5386.89 − 1000) = 4386.89.
- **Rows:** `keep_floor` = 1000 (home_2026; source household, "reguła 15.: zostawiamy 1 000 zł"; known 2025-12-31; no span).
- **Law date:** 2026-11-01, the start of key 2026-11 (§4.3).
- **Cut:** 2026-10-19. The look-ahead reads each instance at min(2026-11-15, K) = K.
- **What the look-ahead counted:** the lowest end-of-day free balance is on day a, 5386.89. The window holds:
  - +17220 `client_pay[client_a, 2026-10]` on Nov 16;
  - the group on Nov 20: 456.18 + 830.58 + 1935;
  - VAT 3818 on Nov 25;
  - +3690 `client_pay[client_b, 2026-10]` on Nov 30;
  - rent on Dec 1;
  - `loan_instalment[laptop, 7]` 300 on Dec 5;
  - costs on Dec 7;
  - the wage on Dec 10;
  - `client_pay[client_a, 2026-11]` on Dec 14;
  - spend each day;
  - the hold of 3220, on Nov 16–24 and from Dec 14.
- **Register:**
  - `hold: konto holds vat_owed`.
  - Held, from the instances in the window read for periods after K:
    - `rent` = 3200 (home_2026), read for 2026-12;
    - `cost_net` = 400 and `cost_vat` = 92 (dg_2026), read for 2026-12;
    - `salary[wife_acme]` = 7000 (home_2026), read for 2026-11;
    - the `ee_rate` rows (sus_2026), `kup_month`, `pit_reduction_month`, `pit_scale` (pit_2022) and `health_rate` (health_2022), all for 2026-11;
    - `fee_net[client_a]` = 14000 (dg_2026), read for 2026-11;
    - `vat_rate` = 0.23 (vat_2026), for 2026-11.
  - Through the hold def's `where`, which reads `pay_day[c, m]` for every month up to 2059-12 (ambiguity D):
    - `assumed: kalendarz świąt po 2030 nie jest wpisany; terminy przesuwają tylko soboty i niedziele` (holidays_later);
    - held `fee_net` rows, read for 2026-11..2059-12.
  - Only under reading C(a), the `when`s of every instance in the run:
    - `assumed: indeks płac +4% rocznie, założenie prototypu` and `missing record counted by wage_fc_2026`, through the `when`s of `zus_social_pay` and `health_settle_pay` in 2027+;
    - held `zus_stage`, `tax_form`, the ZUS rates and the ryczałt rates.
  - No givens, pins, draws or holes.

---

## 7. Units, net_worth[2026-12-31], ike_left[2026] (§5.9; wrappers.tally; pays.tally)

**Prices and buys**
- `etf_price[m]` is 121 for 2026-01..06 (broker_2026_01, closed span) and 126 from 2026-07 (broker_2026_07). `etf_growth[2026]` = 1.
- `units_to_buy` = round(bal(wcash)/price, 1, down), re-read by `buy_units` (phase 4) and `buy_cash` (phase 5) on the 16th. A buy leaves worth + cash unchanged.

**The two wrappers**
- **ike:**
  - buys of 86 (Jan), 25 (Apr), 93 (May), 29 (Jun) and 48 (Sep, from the 6000 record, which is booked into `wcash[ike]` as the instance);
  - 431 units and 19.00 cash.
- **mak:**
  - buys of 62 (Jun), 91 (Jul), 85 (Aug), 370 (Oct), 35 (Nov) and 146 (Dec);
  - none in Sep: the 6535.30 is modelled past, so 93.14 < 126;
  - 869 units and 38.26 cash.

**net_worth[2026-12-31] = 170202.16.** The parts:
- konto 6944.90;
- gotowka 600;
- marek 0, which is 2000 − 700 − 500 − 800;
- `loan_debt[laptop]` −1200: −3000 plus six booked instalments (Jun, Jul, Aug, Oct, Nov, Dec);
- `wcash` 19.00 + 38.26 + 0;
- worth (431 + 869) × 126.

```
$ python3 -c "from decimal import Decimal as D; i1=D('10521.30')-86*121; i2=i1+D('2977.26')-25*121; i3=i2+D('11197.16')-93*121; i4=i3+D('3564.28')-29*121; i5=i4+6000-48*126; m1=D('7573.60')-62*121; m2=m1+11461-91*126; m3=m2+D('10736.54')-85*126; m4=m3+D('46639.92')-370*126; m5=m4+D('4386.89')-35*126; m6=m5+D('18344.31')-146*126; u=150+86+25+93+29+48; v=80+62+91+85+370+35+146; nw=D('6944.90')+600+0-1200+i5+m6+(u+v)*126; print('ike cash steps',i1,i2,i3,i4,i5,'units',u,'| mak cash steps',m1,m2,m3,m4,m5,m6,'units',v,'| net_worth 2026-12-31',nw,'alt (Sep instalment on loan_debt)',nw+300)"
ike cash steps 115.30 67.56 11.72 67.00 19.00 units 431 | mak cash steps 71.60 66.60 93.14 113.06 89.95 38.26 units 869 | net_worth 2026-12-31 170202.16 alt (Sep instalment on loan_debt) 170502.16
```

**ike_left[2026] = 28260 − 34260 = −6000.00.**
- `flows(invest[ike], 2026)` counts the model's booked 28260 for Jan–Jun and the September record of 6000, which is booked as that instance.
- In the past, the model fills IKE in the uncovered months, and the household's real 6000 comes on top. That mixes the modelled past with the real one (see the balance line in §5).

---

## 8. 2027, and Jan–Mar 2028

**What changes in 2027**
- From 2027-01, `oki_from` = 2027-01 ≤ m. mak's rule gives 0, and oki's gives room re-read after ike.
- `ike_limit[2027]` = 29390.40 and `ike_limit[2028]` = 30566.01.
- **P = 800 from 2027-01-01.** `marek_last` was booked on 2026-12-31, and no record confirms it, so `plan_low` takes it out of the balance it starts from (§5.13; ENGINE §4.2 P(t)). So A = 1800 − min (ambiguity A).
- `etf_price` in 2027 is 126 × 1.06 = 133.56.

**New instances**
- `pit28_pay[2026]` −2 on 2027-04-30.
- `health_settle_pay[2026]` −996.69 on 2027-05-20.
- `pit37_refund[wife, 2026]` +2 on 2027-06-14.
- Laptop k = 8, 9, 10 on Jan 5, Feb 5 and Mar 5.

**The group**

| Paid | Amount |
|---|---|
| 2027-01-20 | 3221.76 |
| 2027-02-22 | 2927.72 |
| 2027-03-22 and 2027-04-20 | 2944.72 |
| 2027-05-20 | 3290.24 (Apr health is mid) |
| 2027-06-21 to 2028-01-20 | 3269.24 |
| 2028-02-21 | 2963.42 |
| 2028-03-20 and 2028-04-20 | 2980.42 |

**2028**
- rent is 3296.00, spend is 118.75, 126.94 or 122.71, and the ZUS and health figures are those of §2.2.

| Invest on | B before | Window minimum (day) | `plan_low` = B − 800 + min | room | ike | oki | A after |
|---|---|---|---|---|---|---|---|
| 2027-01-15 | 23545.22 | −10335.16 (Feb 9) | 12410.06 | 11410.06 | 11410.06 | 0 | 12135.16 |
| 2027-02-15 | 23355.83 | −11720.33 (Mar 1) | 10835.50 | 9835.50 | 9835.50 | 0 | 13520.33 |
| 2027-03-15 | 25405.61 | −9919.79 (Apr 11) | 14685.82 | 13685.82 | 8144.84 | 5540.98 | 11719.79 |
| 2027-04-15 | 23645.15 | −9591.28 (May 9) | 13253.87 | 12253.87 | 0 | 12253.87 | 11391.28 |
| 2027-05-15 | 23429.93 | −11023.74 (Jun 9) | 11606.19 | 10606.19 | 0 | 10606.19 | 12823.74 |
| 2027-06-15 | 23408.89 | −10144.38 (Jul 11) | 12464.51 | 11464.51 | 0 | 11464.51 | 11944.38 |
| 2027-07-15 | 23660.51 | −9971.49 (Aug 9) | 12889.02 | 11889.02 | 0 | 11889.02 | 11771.49 |
| 2027-08-15 | 6209.93 | 0 (Aug 15 itself) | 5409.93 | 4409.93 | 0 | 4409.93 | 1800.00 |
| 2027-09-15 | 30620.84 | −10029.09 (Oct 10) | 19791.75 | 18791.75 | 0 | 18791.75 | 11829.09 |
| 2027-10-15 | 23545.22 | −12251.01 (Nov 1) | 10494.21 | 9494.21 | 0 | 9494.21 | 14051.01 |
| 2027-11-15 | 25651.85 | −9913.80 (Dec 9) | 14938.05 | 13938.05 | 0 | 13938.05 | 11713.80 |
| 2027-12-15 | 23429.93 | −10098.63 (2028-01-09) | 12531.30 | 11531.30 | 0 | 11531.30 | 11898.63 |
| 2028-01-15 | 23409.17 | −10227.70 (Feb 9) | 12381.47 | 11381.47 | 11381.47 | 0 | 12027.70 |
| 2028-02-15 | 23360.03 | −9725.33 (Mar 9) | 12834.70 | 11834.70 | 11834.70 | 0 | 11525.33 |
| 2028-03-15 | 23409.17 | −9900.81 (Apr 9) | 12708.36 | 11708.36 | 7349.84 | 4358.52 | — |

**Two months stand out**
- **Aug 2027.** client_a[Jul] comes on Aug 16, after the invest, because Aug 14 is a Saturday. So B is low, the minimum is day t itself, and A = 1800.
- **Oct 2027.** client_b[Sep] slips to Nov 2, so the minimum falls on Nov 1.

```
$ python3 -c "from decimal import Decimal as D; L=D('115.29'); F=D('127.64'); w=D('5101.67'); P=800; B=D('11339.30')-D('5194.40')+800-3200-15*L-792+w+17220; T=-16*L-D('3221.76')-3818; c={'t':D(-3220),'Jan24':-9*L-D('3221.76')-3220,'Jan30':-15*L-D('3221.76')-3818,'Jan31':T,'Feb1':T+490-F,'Feb9':T+490-792-9*F,'Feb14':T+490-792-14*F+w}; m=min(c.values()); r=B-P+m-1000; print('Jan15 2027 B',B,c,'min',m,'plan_low',B-P+m,'room=ike',r,'A',B-r)"
Jan15 2027 B 23545.22 {'t': Decimal('-3220'), 'Jan24': Decimal('-7479.37'), 'Jan30': Decimal('-8769.11'), 'Jan31': Decimal('-8884.40'), 'Feb1': Decimal('-8522.04'), 'Feb9': Decimal('-10335.16'), 'Feb14': Decimal('-5871.69')} min -10335.16 plan_low 12410.06 room=ike 11410.06 A 12135.16

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); F=D('127.64'); w=D('5101.67'); P=800; g=D('2927.72'); B=D('12135.16')-16*L-D('3221.76')-3818+490-792+w-15*F+17220; T=-13*F-g-3818; c={'t':D(-3220),'Feb24':-9*F-g-3220,'Feb28':T,'Mar1':T-3200-L,'Mar9':T+490-792-9*L,'Mar14':T+490-792-14*L+w}; m=min(c.values()); r=B-P+m-1000; print('Feb15 2027 B',B,c,'min',m,'plan_low',B-P+m,'room=ike',r,'A',B-r)"
Feb15 2027 B 23355.83 {'t': Decimal('-3220'), 'Feb24': Decimal('-7296.48'), 'Feb28': Decimal('-8405.04'), 'Mar1': Decimal('-11720.33'), 'Mar9': Decimal('-9744.65'), 'Mar14': Decimal('-5219.43')} min -11720.33 plan_low 10835.50 room=ike 9835.50 A 13520.33

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); F=D('127.64'); w=D('5101.67'); P=800; g=D('2944.72'); B=D('13520.33')-13*F-D('2927.72')-3818+490-792+w-15*L+17220; T=-16*L-g-3818+3690; c={'t':D(-3220),'Mar24':-9*L-g-3220,'Mar29':-14*L-g-3818,'Mar31':T,'Apr1':T-3200-N,'Apr11':T-3692-11*N,'Apr14':T-3692-14*N+w+17220-3220}; m=min(c.values()); r=B-P+m-1000; ike=D('29390.40')-D('11410.06')-D('9835.50'); print('Mar15 2027 B',B,c,'min',m,'plan_low',B-P+m,'room',r,'ike',ike,'oki',r-ike,'A',B-r)"
Mar15 2027 B 25405.61 {'t': Decimal('-3220'), 'Mar24': Decimal('-7202.33'), 'Mar29': Decimal('-8376.78'), 'Mar31': Decimal('-4917.36'), 'Apr1': Decimal('-8236.49'), 'Apr11': Decimal('-9919.79'), 'Apr14': Decimal('8824.49')} min -9919.79 plan_low 14685.82 room 13685.82 ike 8144.84 oki 5540.98 A 11719.79

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('11719.79'); d=-16*L-D('2944.72')-3818+3690-3200-15*N-492+D('5101.67')+17220; B=A+d; T=-15*N-D('2944.72')-3818+3690-2; c={'t':D(-3220),'Apr25':-10*N-D('2944.72')-3220,'Apr29':-14*N-D('2944.72')-3818,'Apr30':T,'May9':T-3692-9*L,'May13':T-3692-13*L+D('5101.67'),'May14':T-3692-14*L+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; print('B_Apr',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Apr',B-r)"
B_Apr 23645.15 {'t': Decimal('-3220'), 'Apr25': Decimal('-7356.02'), 'Apr29': Decimal('-8430.54'), 'Apr30': Decimal('-4861.67'), 'May9': Decimal('-9591.28'), 'May13': Decimal('-4950.77'), 'May14': Decimal('8933.94')} min -9591.28 plan_low 13253.87 room=oki 12253.87 A_Apr 11391.28

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('11391.28'); d=-15*N-D('2944.72')-3818+3690-2-3200-492-15*L+D('5101.67')+17220; B=A+d; g=D('3290.24')+D('996.69'); T=-16*L-g-3818+3690; c={'t':D(-3220),'May24':-9*L-g-3220,'May30':-15*L-g-3818,'May31':T,'Jun9':T-3692-9*N,'Jun13':T-3692-13*N+D('5101.67'),'Jun14':T-3692-14*N+D('5101.67')+17220+2-3220}; m=min(c.values()); r=B-P+m-1000; print('Delta_Apr',d,'B_May',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_May',B-r)"
Delta_Apr 12038.65 B_May 23429.93 {'t': Decimal('-3220'), 'May24': Decimal('-8544.54'), 'May30': Decimal('-9834.28'), 'May31': Decimal('-6259.57'), 'Jun9': Decimal('-11023.74'), 'Jun13': Decimal('-6398.59'), 'Jun14': Decimal('7484.28')} min -11023.74 plan_low 11606.19 room=oki 10606.19 A_May 12823.74

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('12823.74'); d=-16*L-D('3290.24')-D('996.69')-3818+3690-3200-492-15*N+D('5101.67')+17220+2; B=A+d; g=D('3269.24'); T=-15*N-g-3818+3690; c={'t':D(-3220),'Jun24':-9*N-g-3220,'Jun29':-14*N-g-3818,'Jun30':T,'Jul11':T-3692-11*L,'Jul13':T-3692-13*L+D('5101.67'),'Jul14':T-3692-14*L+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; A2=B-r; print('Delta_May',d,'B_Jun',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Jun',A2,'konto 2027-06-30',A2+T)"
Delta_May 10585.15 B_Jun 23408.89 {'t': Decimal('-3220'), 'Jun24': Decimal('-7561.41'), 'Jun29': Decimal('-8755.06'), 'Jun30': Decimal('-5184.19'), 'Jul11': Decimal('-10144.38'), 'Jul13': Decimal('-5273.29'), 'Jul14': Decimal('8611.42')} min -10144.38 plan_low 12464.51 room=oki 11464.51 A_Jun 11944.38 konto 2027-06-30 6760.19

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('11944.38'); g=D('3269.24'); d=-15*N-g-3818+3690-3200-492-15*L+D('5101.67')+17220; B=A+d; T=-16*L-g-3818+3690; c={'t':D(-3220),'Jul25':-10*L-g-3220,'Jul29':-14*L-g-3818,'Jul31':T,'Aug9':T-3692-9*L,'Aug14':T-3692-14*L+D('5101.67')}; m=min(c.values()); r=B-P+m-1000; print('Delta_Jun',d,'B_Jul',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Jul',B-r)"
Delta_Jun 11716.13 B_Jul 23660.51 {'t': Decimal('-3220'), 'Jul25': Decimal('-7642.14'), 'Jul29': Decimal('-8701.30'), 'Jul31': Decimal('-5241.88'), 'Aug9': Decimal('-9971.49'), 'Aug14': Decimal('-5446.27')} min -9971.49 plan_low 12889.02 room=oki 11889.02 A_Jul 11771.49

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('11771.49'); g=D('3269.24'); d=-16*L-g-3818+3690-3200-492-15*L+D('5101.67'); B=A+d; T=17220-16*L-g-3818+3690; c={'t':D(0),'Aug16':17220-L-3220,'Aug24':17220-9*L-g-3220,'Aug29':17220-14*L-g-3818,'Aug31':T,'Sep9':T-3692-9*N,'Sep13':T-3692-13*N+D('5101.67'),'Sep14':T-3692-14*N+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; print('Delta_Jul',d,'B_Aug',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Aug',B-r)"
Delta_Jul -5561.56 B_Aug 6209.93 {'t': Decimal('0'), 'Aug16': Decimal('13884.71'), 'Aug24': Decimal('9693.15'), 'Aug29': Decimal('8518.70'), 'Aug31': Decimal('11978.12'), 'Sep9': Decimal('7213.95'), 'Sep13': Decimal('11839.10'), 'Sep14': Decimal('25719.97')} min 0 plan_low 5409.93 room=oki 4409.93 A_Aug 1800.00

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('1800'); g=D('3269.24'); d=17220-16*L-g-3818+3690-3200-492-15*N+D('5101.67')+17220; B=A+d; T=-15*N-g-3818+3690; c={'t':D(-3220),'Sep26':-11*N-g-3220,'Sep29':-14*N-g-3818,'Sep30':T,'Oct10':T-3692-10*L,'Oct13':T-3692-13*L+D('5101.67'),'Oct14':T-3692-14*L+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; print('Delta_Aug',d,'B_Sep',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Sep',B-r)"
Delta_Aug 28820.84 B_Sep 30620.84 {'t': Decimal('-3220'), 'Sep26': Decimal('-7799.67'), 'Sep29': Decimal('-8755.06'), 'Sep30': Decimal('-5184.19'), 'Oct10': Decimal('-10029.09'), 'Oct13': Decimal('-5273.29'), 'Oct14': Decimal('8611.42')} min -10029.09 plan_low 19791.75 room=oki 18791.75 A_Sep 11829.09

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('11829.09'); g=D('3269.24'); d=-15*N-g-3818+3690-3200-492-15*L+D('5101.67')+17220; B=A+d; T=-16*L-g-3818; c={'t':D(-3220),'Oct24':-9*L-g-3220,'Oct31':T,'Nov1':T-3200-N,'Nov9':T-3200+3690-492-9*N,'Nov14':T-3200+3690-492-14*N+D('5101.67')}; m=min(c.values()); r=B-P+m-1000; print('Delta_Sep',d,'B_Oct',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Oct',B-r)"
Delta_Sep 11716.13 B_Oct 23545.22 {'t': Decimal('-3220'), 'Oct24': Decimal('-7526.85'), 'Oct31': Decimal('-8931.88'), 'Nov1': Decimal('-12251.01'), 'Nov9': Decimal('-10006.05'), 'Nov14': Decimal('-5500.03')} min -12251.01 plan_low 10494.21 room=oki 9494.21 A_Oct 14051.01

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); N=D('119.13'); P=800; A=D('14051.01'); g=D('3269.24'); d=-16*L-g-3818-3200+3690-492+D('5101.67')-15*N+17220; B=A+d; T=-15*N-g-3818+3690; c={'t':D(-3220),'Nov24':-9*N-g-3220,'Nov29':-14*N-g-3818,'Nov30':T,'Dec9':T-3692-9*L,'Dec13':T-3692-13*L+D('5101.67'),'Dec14':T-3692-14*L+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; print('Delta_Oct',d,'B_Nov',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Nov',B-r)"
Delta_Oct 11600.84 B_Nov 25651.85 {'t': Decimal('-3220'), 'Nov24': Decimal('-7561.41'), 'Nov29': Decimal('-8755.06'), 'Nov30': Decimal('-5184.19'), 'Dec9': Decimal('-9913.80'), 'Dec13': Decimal('-5273.29'), 'Dec14': Decimal('8611.42')} min -9913.80 plan_low 14938.05 room=oki 13938.05 A_Nov 11713.80

$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; L=D('115.29'); N=D('119.13'); P=800; s=D('3574.02')*D('1.03'); L8=(s/31).quantize(D('0.01'),H); F8=(s/29).quantize(D('0.01'),H); N8=(s/30).quantize(D('0.01'),H); R8=(3200*D('1.03')).quantize(D('0.01'),H); A=D('11713.80'); g=D('3269.24'); d=-15*N-g-3818+3690-3200-492-15*L+D('5101.67')+17220; B=A+d; T=-16*L-g-3818+3690; c={'t':D(-3220),'Dec26':-11*L-g-3220,'Dec29':-14*L-g-3818,'Dec31':T,'Jan9':T-R8-492-9*L8,'Jan13':T-R8-492-13*L8+D('5101.67'),'Jan14':T-R8-492-14*L8+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; A2=B-r; print('spend 2028 31/29/30',L8,F8,N8,'rent 2028',R8,'Delta_Nov',d,'B_Dec',B,c,'min',m,'plan_low',B-P+m,'room=oki',r,'A_Dec',A2,'konto 2027-12-31',A2+T)"
spend 2028 31/29/30 118.75 126.94 122.71 rent 2028 3296.00 Delta_Nov 11716.13 B_Dec 23429.93 {'t': Decimal('-3220'), 'Dec26': Decimal('-7757.43'), 'Dec29': Decimal('-8701.30'), 'Dec31': Decimal('-5241.88'), 'Jan9': Decimal('-10098.63'), 'Jan13': Decimal('-5471.96'), 'Jan14': Decimal('8409.29')} min -10098.63 plan_low 12531.30 room=oki 11531.30 A_Dec 11898.63 konto 2027-12-31 6656.75

$ python3 -c "from decimal import Decimal as D; L=D('115.29'); L8=D('118.75'); F8=D('126.94'); P=800; A=D('11898.63'); g=D('3269.24'); d=-16*L-g-3818+3690-3296-492-15*L8+D('5101.67')+17220; B=A+d; T=-16*L8-g-3818+3690; c={'t':D(-3220),'Jan24':-9*L8-g-3220,'Jan30':-15*L8-g-3818,'Jan31':T,'Feb9':T-3788-9*F8,'Feb13':T-3788-13*F8+D('5101.67'),'Feb14':T-3788-14*F8+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; print('Delta_Dec27',d,'B_Jan28',B,c,'min',m,'plan_low',B-P+m,'room=ike',r,'ike rem after',D('30566.01')-r,'A_Jan28',B-r)"
Delta_Dec27 11510.54 B_Jan28 23409.17 {'t': Decimal('-3220'), 'Jan24': Decimal('-7557.99'), 'Jan30': Decimal('-8868.49'), 'Jan31': Decimal('-5297.24'), 'Feb9': Decimal('-10227.70'), 'Feb13': Decimal('-5633.79'), 'Feb14': Decimal('8239.27')} min -10227.70 plan_low 12381.47 room=ike 11381.47 ike rem after 19184.54 A_Jan28 12027.70

$ python3 -c "from decimal import Decimal as D; L8=D('118.75'); F8=D('126.94'); P=800; A=D('12027.70'); g=D('3269.24'); g2=D('2963.42'); d=-16*L8-g-3818+3690-3296-492-15*F8+D('5101.67')+17220; B=A+d; T=-14*F8-g2-3818; c={'t':D(-3220),'Feb24':-9*F8-g2-3220,'Feb29':T,'Mar1':T-3296+3690-L8,'Mar9':T-3296+3690-492-9*L8,'Mar13':T-3296+3690-492-13*L8+D('5101.67'),'Mar14':T-3296+3690-492-14*L8+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; rem=D('19184.54'); print('Delta_Jan28',d,'B_Feb28',B,c,'min',m,'plan_low',B-P+m,'room',r,'ike',min(r,rem),'ike rem after',rem-min(r,rem),'A_Feb28',B-r)"
Delta_Jan28 11332.33 B_Feb28 23360.03 {'t': Decimal('-3220'), 'Feb24': Decimal('-7325.88'), 'Feb29': Decimal('-8558.58'), 'Mar1': Decimal('-8283.33'), 'Mar9': Decimal('-9725.33'), 'Mar13': Decimal('-5098.66'), 'Mar14': Decimal('8782.59')} min -9725.33 plan_low 12834.70 room 11834.70 ike 11834.70 ike rem after 7349.84 A_Feb28 11525.33

$ python3 -c "from decimal import Decimal as D; L8=D('118.75'); F8=D('126.94'); N8=D('122.71'); P=800; A=D('11525.33'); g2=D('2963.42'); g3=D('2980.42'); d=-14*F8-g2-3818-3296+3690-492+D('5101.67')-15*L8+17220; B=A+d; T=-16*L8-g3-3818+3690; c={'t':D(-3220),'Mar26':-11*L8-g3-3220,'Mar29':-14*L8-g3-3818,'Mar31':T,'Apr9':T-3788-9*N8,'Apr13':T-3788-13*N8+D('5101.67'),'Apr14':T-3788-14*N8+D('5101.67')+17220-3220}; m=min(c.values()); r=B-P+m-1000; rem=D('7349.84'); print('Delta_Feb28',d,'B_Mar28',B,c,'min',m,'plan_low',B-P+m,'room',r,'ike',min(r,rem),'oki',r-min(r,rem),'ike 2028 total',D('11381.47')+D('11834.70')+min(r,rem),'ike_left 2028',D('30566.01')-D('11381.47')-D('11834.70')-min(r,rem))"
Delta_Feb28 11883.84 B_Mar28 23409.17 {'t': Decimal('-3220'), 'Mar26': Decimal('-7506.67'), 'Mar29': Decimal('-8460.92'), 'Mar31': Decimal('-5008.42'), 'Apr9': Decimal('-9900.81'), 'Apr13': Decimal('-5289.98'), 'Apr14': Decimal('8587.31')} min -9900.81 plan_low 12708.36 room 11708.36 ike 7349.84 oki 4358.52 ike 2028 total 30566.01 ike_left 2028 0.00
```

`Delta_x` is the net flow from the 16th of month x through the 15th of the next month, so B(next) = A(x) + Delta_x. "3788" is the 2028 rent plus costs.

### Results that follow

**konto_balance**
- 2027-06-30 = **6760.19** and 2027-12-31 = **6656.75**.
- `vat_owed` is 0 on both: client_a[May]'s VAT was paid Jun 25, and client_a[Nov]'s Dec 27. client_a[Jun] arrives Jul 14, and client_a[Dec] on 2028-01-14.

**ike_left**
- 2027: 29390.40 − (11410.06 + 9835.50 + 8144.84) = **0.00**.
- 2028: 30566.01 − (11381.47 + 11834.70 + 7349.84) = **0.00**.

**oki_tax[2027]** (wrappers.tally; state pay `oki_tax_pay`, dated forms only)
- A buy moves cash into units at exactly 133.56, so `worth(wunits[oki], d) + bal(wcash[oki], d)` equals the cumulative oki invest.
- So `oki_avg[2027]` = Σ invest_k × days_k / 365, where days_k runs from the 15th of month k to Dec 31 inclusive. That uses end-of-day values, as `bal(x, d)` is end of day (§5.13.1; ambiguity F).
- The sum is 790722481/18250 = 43327.259…, so `oki_tax[2027]` = round(43327.26 × 0.0085, 1, half-up) = **368**. The rate is oki_2027's `oki_rate_first`, and the exempt amount for `foreign` is 0.
- It is paid on 2028-05-31, a Wednesday.

```
$ python3 -c "from fractions import Fraction as Q; from decimal import Decimal as D; S=D('5540.98')*292+D('12253.87')*261+D('10606.19')*231+D('11464.51')*200+D('11889.02')*170+D('4409.93')*139+D('18791.75')*108+D('9494.21')*78+D('13938.05')*47+D('11531.30')*17; I=D('5540.98')+D('12253.87')+D('10606.19')+D('11464.51')+D('11889.02')+D('4409.93')+D('18791.75')+D('9494.21')+D('13938.05')+D('11531.30'); avg=Q(S)/365; t=avg*Q('0.0085'); print('oki invest 2027',I,'day-sum',S,'oki_avg',avg,float(avg),'x 0.0085',float(t),'oki_tax',int(t+Q(1,2)),'taxes_paid 2028',68629+int(t+Q(1,2)))"
oki invest 2027 109919.81 day-sum 15814449.62 oki_avg 790722481/18250 43327.25923287671 x 0.0085 368.28170347945206 oki_tax 368 taxes_paid 2028 68997
```

**taxes_paid[2028] = 68997.00.** It is the sum of:
- ryczałt 22813;
- VAT 45816;
- `pit28_pay[2027]` +2;
- `pit37_refund[wife, 2027]` −2;
- `oki_tax_pay[2027]` 368.

---

## 9. Where I stopped, and corrections made on the way

**Where I stopped.** I stopped at 2028-03-15, the invest that fills IKE for 2028.

`net_worth` at 2031-12-31, 2041-12-31 and 2059-12-31 needs the walk through 2059. That is about 380 more monthly invests, each with its own window minimum, plus:
- the OKI tax each year. From 2028 the rate is max(0.001, floor(0.19 × 0.0375, 0.0001)) = 0.0071, with `nbp_ref` held;
- standard ZUS from 2028-09;
- 3% a year on rent and spend from 2028;
- the health tier by year-to-date revenue;
- the empty holiday calendar from 2031.

As far as I read the rows, nothing ahead is a hole:
- the open-ended rows are held, and the wage figures are projected;
- `tax_form` is ryczałt from 2026 on;
- `overpay_choice` is shorten;
- there are no children.

So the stop is the hand's, under "no loops", not the engine's.

**Corrections, all redone here**
1. **pit28[2026].** An earlier pass counted the 1990 advance four times: advances 21835, due −1988, and a refund of 1988 on 2027-06-14. The recount gives 19845 and a due of +2, paid on 2027-04-30. I redid:
   - the April–June 2027 walk;
   - `oki_avg`;
   - taxes_paid for 2027 and 2028.

   konto_balance 2027-06-30 is unchanged: the invests absorb the difference.
2. **January 2027.** An earlier pass of January 2027 left out P = 800 (`marek_last`, booked and unconfirmed). I redid it: ike in Jan falls from 12210.06 to 11410.06.
3. **Minor.** An earlier net-worth alternative had a sign slip; it is now 170502.16. One weekday label and one non-minimum candidate were miscounted, with no effect on any result.

---

## 10. Outputs

Cut 2026-10-19. PLN.

| Ask | Key | Value |
|---|---|---|
| **wife_net** | 2026-10 | 5101.67 |
| | 2027-01 | 5101.67 |
| **social_and_health** | 2026-10 | zus_social 456.18, health_month 830.58 |
| | 2027-01 | 474.44, 518.28 |
| | 2028-09 | 2083.99, 898.35 |
| **replay_september** | konto 2026-09 | the ledger of §5. Balance line **+32097.23** (stated 41386.27, booked 9289.04). Then spend +4599.90 (30 missing days), client_pay −3690.00 (client_b[Aug] missing), unexplained −3574.02 (10 records), invest +535.30 (mak +6535.30 missing; ike matched, model 0, −6000.00), loan_instalment +300.00. Seven groups at 0.00 (matched, timing 0) |
| **explain_room** | room[2026-11] | 2 cells: (2026-11-15, phase 6, invest[ike, 2026-11]) = 4386.89 and (2026-11-15, phase 6, invest[mak, 2026-11]) = 4386.89. Rule `base` won; `plan_low` 5386.89, its minimum on day a; keep_floor 1000; law date 2026-11-01, cut 2026-10-19; register as in §6 |
| **konto_balance** | 2026-10-31 | 6064.17, 6064.17 |
| | 2026-12-31 | 6944.90, 6944.90 |
| | 2027-06-30 | 6760.19, 6760.19 (5960.19 under A(b)) |
| | 2027-12-31 | 6656.75, 6656.75 (5856.75 under A(b)) |
| **ike_left** | 2026 | −6000.00 |
| | 2027 | 0.00 |
| | 2028 | 0.00 |
| **pit28** | 2026 | 2.00 |
| | 2027 | 2.00 |
| | 2028 | 1.00 |
| **pit37** | me 2026, 2027, 2028 | 0.00, 0.00, 0.00 |
| | wife 2026, 2027, 2028 | −2.00, −2.00, −2.00 |
| **health_settlement** | 2026, 2027, 2028 | 996.69, 1036.56, 1078.02 |
| **taxes_paid** | 2026 | 52272.00 |
| | 2027 | 69059.00 |
| | 2028 | 68997.00 (69003.00 under A(b); 68995.00 under F(b)) |
| **net_worth** | 2026-12-31 | 170202.16 (177037.46 under B(b); 170502.16 with only the debt leg) |
| | 2031-12-31, 2041-12-31, 2059-12-31 | **not computed**: the walk stops at 2028-03-15 (§9) |

---

## 11. Ambiguities

Each one gives what the definition leaves open, the choices, and every choice computed. The first choice is the one used in §10.

### A. A promised movement after its date, in a forecast (§5.13 `promised`; ENGINE §4.2 P(t))

`marek_last` (+800, due 2026-12-31) is booked on its day. `plan_low` then takes it back out of its starting balance "until a record confirms it". After K no record can ever confirm a planned movement, so P = 800 stays for the rest of the run, and konto carries 800 that is never invested.
- **(a) As written: P = 800 from 2027-01-01 to 2059.**
  - Jan 2027 ike is 11410.06, Mar 2027 ike is 8144.84 and oki 5540.98.
  - konto 2027-06-30 is 6760.19 and 2027-12-31 is 6656.75.
  - oki_tax[2027] is 368, and taxes_paid 2028 is 68997.00.
- **(b) A planned promise counts as kept on its date, so P = 0 after it.**
  - Jan 2027 ike is 12210.06, Mar 2027 ike is 7344.84 and oki 6340.98.
  - konto is 5960.19 and 5856.75.
  - oki_avg is 43967.26, oki_tax is 374 and taxes_paid 2028 is 69003.00.
  - ike_left 2027 and 2028 stay 0.00.

```
$ python3 -c "from fractions import Fraction as Q; from decimal import Decimal as D; S=D('6340.98')*292+D('12253.87')*261+D('10606.19')*231+D('11464.51')*200+D('11889.02')*170+D('4409.93')*139+D('18791.75')*108+D('9494.21')*78+D('13938.05')*47+D('11531.30')*17; avg=Q(S)/365; t=avg*Q('0.0085'); print('alt (promised 800 not held back): Jan ike',D('23545.22')-D('10335.16')-1000,'Mar ike',D('29390.40')-D('12210.06')-D('9835.50'),'Mar oki',D('13685.82')-D('7344.84'),'oki_avg',float(avg),'tax',float(t),'oki_tax',int(t+Q(1,2)),'taxes_paid 2028',68629+int(t+Q(1,2)),'konto 2027-06-30',D('6760.19')-800,'2027-12-31',D('6656.75')-800)"
alt (promised 800 not held back): Jan ike 12210.06 Mar ike 7344.84 Mar oki 6340.98 oki_avg 43967.25923287671 tax 373.72170347945206 oki_tax 374 taxes_paid 2028 69003 konto 2027-06-30 5960.19 2027-12-31 5856.75
```

### B. Modelled past when only one end is covered (§5.18 Coverage; ENGINE §6.2)

On a covered day, a pay that touches konto is "never booked", on either end. The text doesn't say what the uncovered place should then show. Two pays are affected:
- `loan_instalment[laptop, 4]` (Sep 5). `loan_debt[laptop]` never receives that 300, so the loan ends at −300 after k = 10 and keeps it to 2059, while `debt_before` reaches 0.
- `invest[mak, 2026-09]` (6535.30). `wcash[mak]` never receives it.

The choices:
- **(a) Neither end is booked.** net_worth[2026-12-31] = **170202.16**: loan_debt −1200 and mak 869 units.
- **(b) The uncovered end is booked.** The money would appear from nowhere, because konto's side is its records. loan_debt is −900, and mak buys 52 more units on Sep 16 (921 units, cash 21.56). net_worth = **177037.46**.
- Booking only the debt leg gives 170502.16.

```
$ python3 -c "from decimal import Decimal as D; c1=D('93.14')+D('6535.30')-52*126; c2=c1+D('46639.92')-370*126; c3=c2+D('4386.89')-35*126; c4=c3+D('18344.31')-146*126; nw=D('6944.90')+600-900+19+c4+(431+869+52)*126; print('Sep16 units',(D('93.14')+D('6535.30'))/126,'cash steps',c1,c2,c3,c4,'net_worth if uncovered ends were booked',nw,'only the debt leg',D('170202.16')+300)"
Sep16 units 52.60666666666666666666666667 cash steps 76.44 96.36 73.25 21.56 net_worth if uncovered ends were booked 177037.46 only the debt leg 170502.16
```

### C. What `plan_low`'s register holds (§8.2 "transitive"; §5.13.1; ENGINE §4.2)

The look-ahead takes every flow instance of the run "as known on day t": first its keys, then its `when`, then its date and its amount. It drops an instance on a false `when`, "its date unread", or on a date outside (t, b], "its amount unread, whatever its `when`". The register lists "whatever its evaluation read". The text doesn't say whether reads made only to drop an instance count.
- **(a) They count.** The register of `room` (and so of every balance after the first invest) carries:
  - the `assumed` wage_growth row and `missing record counted by wage_fc_2026`, from the `when`s of `zus_social_pay`, `health_settle_pay` and `health_settle_back` in 2027+;
  - held `zus_stage`, `tax_form`, the ZUS rates, the ryczałt rates and the `cost_net` rows;
  - `assumed` holidays_later, from the dates of 2031+ instances.
- **(b) Only the instances left in the window count, plus the hold.** The register is §6's list without the C(a) part.
- Values are the same under both. The register of explain_room and of every later balance differs.

The order also matters for holes. Early in 2026, `when`s that read `zus_stage` for 2026-03 on are holes (Jan 15 and Feb 15 cuts). They are dropped by date, so `plan_low` has a value under both readings. But reading (a) evaluates those `when`s first, and only "whatever its `when`" saves them.

### D. The hold def scans every month to 2059 (business.tally `vat_owed`; §8.2)

`vat_owed[d]` sums over every (c, m) `where fee_net[c, m] > 0 and pay_day[c, m] <= d and vat_day[m] > d`. For each m from 2026-03 to 2059-12, `fee_net > 0` is true, so `pay_day` is evaluated, which reads `holidays_pl` at m's law date.
- **(a) As written.** Every `plan_low` read on konto, and every balance after it, carries `assumed: kalendarz świąt po 2030 nie jest wpisany…` and held `fee_net` rows through 2059.
- **(b) A pruned evaluation.** m > month_of(d) can't pass `pay_day ≤ d`, since `pay_day` ≥ `issue_day` = the end of m. It would read only months up to d and leave those entries out.
- Values are the same. Registers differ.

### E. The totals of a `balance` group (ENGINE §6.3 "Totals")

"Each group gives the model's total, the records' total and the difference". For a balance group the text doesn't say what the two totals are.
- (a) model = the booked balance, 9289.04; records = the stated balance, 41386.27; difference +32097.23. I used this.
- (b) Both totals are blank or 0; difference +32097.23.

This is presentation only.

### F. `worth(x, d)` within a day (§5.13.1; wrappers.tally `oki_avg`)

`bal(x, d)` is "x's balance at the end of day d". `worth(x, d)` is "a holding's value at d". `oki_avg` adds the two.
- (a) End of day, the same as `bal`: an invest on the 15th counts from the 15th. oki_avg is 43327.26 and oki_tax[2027] is 368, so taxes_paid 2028 is 68997.00.
- (b) Start of day: an invest counts from the next day. oki_avg is 43026.11 and oki_tax is 366, so taxes_paid 2028 is 68995.00.

```
$ python3 -c "from fractions import Fraction as Q; S=Q('15814449.62')-Q('109919.81'); a=S/365; t=a*Q('0.0085'); print('start-of-day oki_avg',float(a),'tax',float(t),'rounded',int(t+Q(1,2)))"
start-of-day oki_avg 43026.10906849315 tax 365.72192708219177 rounded 366
```

### Points I checked that the text settles

These are listed so a reviewer doesn't re-open them.
- **Day t is in the window**, `plan_low` "on days a..b". Its value is −h(t). In Nov 2026 and Aug 2027 the minimum is day t itself, 0.
- **ike's cap counts what is booked before ike on the same day**, and not ike itself ("Same-day totals").
- **`flows(health_pay, m)` in `ryczalt_adv` means payments dated in month m.** `, for k` would select by period instead.
- **`spend[2026-10-01]` is booked at the budget** (cut Oct 1), and `arrears none for spend` drops the δ of −33.10. konto's balances don't depend on this: the Oct 15 invest absorbs it.
- **client_b's VAT is never held**, since its `pay_day` > `vat_day`. konto pays that VAT before client_b pays.
- **pit_scale(0) = −3600** comes from the band's first row. It makes January's advance 395, matching art. 32.
- **The September look-ahead counts the plan's instances** (budget spend, client_b on Sep 30), not later records (ENGINE §4.2).
- **A matched line prints whatever its difference**, so `invest[ike, 2026-09]` prints with model 0. Amount-0 instances with no record print nothing.
- **WARN-NEGATIVE prints once per stretch**: 2026-03-01 and 2026-03-17.
- **`flows(invest[ike], 2026)` counts the September record**, which is booked as that instance's movement. So ike_left[2026] = −6000.00. Not an ambiguity, but a result of mixing the modelled Jan–Aug with the real September (the +32097.23 balance line says the same).
