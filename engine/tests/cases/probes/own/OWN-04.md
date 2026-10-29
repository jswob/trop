# OWN-04 · Four ownership regimes

**Story.** Four flats, each worth 300 000.00 DUK: one in Ania and Tomek's community property; one that is Ania's separate property; one owned by Piotr and Ola 60/40; one in Zbigniew's estate (Halina 1/4; Ania, Piotr, Marta and Kuba 3/16 each).

**Rules.**
1. Community property counts half to each spouse in per-person net worth; separate property to its owner; co-ownership and an estate by their shares.

**Question.** Each person's part of each flat.

**Worked by hand.** Community: 150 000.00 each. Separate: Ania 300 000.00. 60/40: Piotr 180 000.00, Ola 120 000.00. Estate: Halina 75 000.00, each of the four 56 250.00.

```yaml
edge: "OWN-04"
inputs: {"value": 300000.00, "regimes": {"community": {"Ania": "1/2", "Tomek": "1/2"}, "separate": {"Ania": "1"},
  "co-owned": {"Piotr": "3/5", "Ola": "2/5"}, "estate": {"Halina": "1/4", "Ania": "3/16", "Piotr": "3/16", "Marta": "3/16", "Kuba": "3/16"}}}
expect: {"kind": "value", "value": {"community": {"Ania": 150000.00, "Tomek": 150000.00}, "separate": {"Ania": 300000.00},
  "co-owned": {"Piotr": 180000.00, "Ola": 120000.00}, "estate": {"Halina": 75000.00, "Ania": 56250.00, "Piotr": 56250.00, "Marta": 56250.00, "Kuba": 56250.00}}}
```
