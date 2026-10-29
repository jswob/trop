"""The family's joins, stated once (E1: households A, B and D as one family).

Each household states its family events as facts it got from outside (A: the gift it received;
A and B: the payouts from Zbigniew Sowa's estate), and D derives them. Here each event is stated
once. The households' facts modules take these values instead of their own copies (a small hook:
their output is byte-identical), and `tools/union_e1.py` shows that the union reproduces every
household's keys and that D's derivation gives what is stated here.

Of Marta, the union knows only what household D shows: her first name, her 3/16 share, and her
payout's amount and day, paid "to her account".
"""
from datetime import date
from decimal import Decimal as D
from fractions import Fraction

# --------------------------------------------------------------------------- Halina's gifts to Anna
# A-D19: cash, handed over, never paid into an account
CASH_GIFT = {"day": date(2021, 4, 30), "amount": D("15000.00"), "giver": "Halina Sowa", "to": "Anna Lis"}
# A-D18 (the gift contract) and D's DM Kopia statement (records/kopia-30-118-117-halina.tsv)
GIFT = {"day": date(2023, 9, 12), "giver": "Halina Sowa", "to": "Anna Lis", "fund": "BRZ-GLB-A",
        "lots": [("G1", "2018-11", D("90")), ("G2", "2019-03", D("60")), ("G3", "2019-08", D("50"))],
        "from": "30-118-117", "into": "30-118-552"}

# --------------------------------------------------------------------------- Zbigniew Sowa's estate
# D-D04 (the heirs), D-D07 (the division and the payouts), A-D36, B-D27 (the payouts as A and B were told)
SHARES = {"Halina": Fraction(1, 4), "Piotr": Fraction(3, 16), "Anna": Fraction(3, 16), "Marta": Fraction(3, 16),
          "Jakub": Fraction(3, 16)}
DIVISION_DAY = date(2025, 6, 12)
PAYOUT_DAY = date(2025, 6, 16)
PAYOUTS = {"Piotr": {"amount": D("38750.00"), "to": "LN45 2090 0017 5521 3380 0002"},
           "Anna": {"amount": D("38750.00"), "to": "LN61 1140 2004 0000 3402 9921 5503"},
           "Marta": {"amount": D("38750.00"), "to": None},          # D-D07: "to her account"
           "Jakub": {"amount": D("7812.00"), "to": "LN52 2090 0017 4471 0098 0001"}}
