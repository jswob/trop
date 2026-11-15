# The prototype's bank history (issue #114)

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
- `reality.tally` is R7's bank half: the places, the pay law (`employment_pl@2022` as household X and
  `m1/goldens/y22/` write it), my pay, my wife's and the rent, the learned daily spending, and the asks of G2,
  G4 and G5. Its vectors are `goldens/bank_*` in `m2/vectors.tsv` (`m2_goldens`); `GOLDENS.md` records each
  against the prototype's figure.
- Not here yet: R7's forward half (the 15th rule, IKE first, the side business, OKI, Marek's claim) and the
  figures that read it (G1, G3, G6's and G10's net worth, the 20 pins, the long horizons).
