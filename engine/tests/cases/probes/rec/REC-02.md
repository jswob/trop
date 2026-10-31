# REC-02 · A statement with errors ★

**Story.** Piotr's account opens March 2025 at 1 000.00 DUK. Bank Wrzosowy's export (semicolon-separated, decimal comma, dates dd.mm.yyyy, two header lines, a balance-after column) lists five rows.

**Rules.**
1. Each row's balance-after is the bank's; a row whose balance didn't move was booked twice in the export.
2. The closing balance is the last row's balance-after.

**Question.** The closing balance; the faulty row; the naive sum of all rows; the sum without the faulty row.

**Worked by hand.** Rows: −200 → 800; +500 → 1 300; +500 → 1 300 (didn't move: **row 3 is a duplicate**); −100 → 1 200; −50 → 1 150. Closing **1 150.00**. Naive 1 000 − 200 + 500 + 500 − 100 − 50 = 1 650.00; without row 3, 1 150.00: dropping it moves the sum by exactly 500.00.

```yaml
edge: "REC-02"
inputs: {"opening": 1000.00, "csv": "Bank Wrzosowy;wyciag;PKB-1201;2025-03\nData;Opis;Kwota;Saldo\n03.03.2025;Sklep Rolnik;-200,00;800,00\n05.03.2025;Klient Kowal;500,00;1300,00\n05.03.2025;Klient Kowal;500,00;1300,00\n10.03.2025;Apteka;-100,00;1200,00\n12.03.2025;Kawiarnia;-50,00;1150,00"}
expect: {"kind": "relation", "value": {"closing": 1150.00, "issues": [{"row": 3, "kind": "no-effect", "amount": 500.00}], "naive_sum": 1650.00, "without_faulty": 1150.00}}
```
