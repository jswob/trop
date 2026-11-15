# The prototype's bank history and its reality R7 (issue #114)

**Synthetic.** No bank record was ever used. `ledger.json` (1 943 rows, 2024-01-01 .. 2026-10-16) was made by
`make-ledger.ts` with seed 20261017 from the prototype's own modelled past: pay, rent, the business's taxes and
invoices from one run of its model (pay dates spread over the 3rd to the 7th, and one bonus the model doesn't
know about, +3 000 on `my_tax/net` for 2025-11, paid 2025-12-04: that is G4), seeded everyday spending at
`sklepy` (1 764 rows, about 140 zł a day), and a "keep about 12 000, IKE first" habit on the 15th. Three rows
are openings (`otwarcie`). `make-ledger.ts` is kept as provenance: it needs the prototype's engine and is never
run here; `ledger.json` is copied byte for byte from the prototype's `model/` folder.

- `to_tally.py` writes the ledger as tally 2 records, one part a month (`records/2024-01.tally` ..
  `records/2026-10.tally`, 1 940 `bank` rows), each covering konto with an anchor at the month's end; the links
  are stated by one written rule (P1, in its docstring and each part's note). `--check` runs in
  `run.py --self-test` (8p).
- `bank.tally` is R7's bank half, a part: the places, the pay law (`employment_pl@2022` as household X and
  `m1/goldens/y22/` write it, with ZUS's yearly cap), my pay, my wife's and the rent (the gross and the rent
  follow `index.wage` and `index.prices`, both 1 until 2027), the learned daily spending, the Polish holidays
  of the prototype's calendar.
- `reality.tally` is the bank run's header (to 2027-01-31) and the asks of G2, G4 and G5. Its vectors are
  `goldens/bank_*` in `m2/vectors.tsv` (`m2_goldens`).
- `marek.tally` and `marek_out.tally` are G1's twins over the bank run: block 14's rows on konto, Marek a
  claim or outside (`goldens/bank_g1_*`).
- `r7/` is R7's forward half, from today (2026-10-17): `forward.tally` (the daily spending, the 15th rule with
  IKE first and OKI, the ETF units, OKI's tax, net worth, FIRE), `business.tally` (block 05's side business),
  `returns.tally` (the yearly PIT-37 returns, each spouse alone by L-26) and `joint.tally`, its twin with the
  prototype's joint return. A header per horizon: `to2028.tally` and `to2031.tally` are stored
  (`goldens/r7_*`); `to2036.tally`, `to2050.tally` (with `raise.tally`, G6) and `to2056.tally` (with
  `infl5.tally`, G10) pass refcore's 8 GB cap and wait for the user's ruling (P2, `GOLDENS.md`).
