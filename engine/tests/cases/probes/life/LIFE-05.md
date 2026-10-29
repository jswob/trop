# LIFE-05 · A sole trader's month and year

**Story.** Piotr started his business in August 2023. In March 2025 he invoices, all net, due 31 March: #7 4 000.00 (paid 28 March), #8 3 000.00 (paid 2 April), #9 2 500.00 (paid 30 March). His revenue was 8 000.00 in January and 9 000.00 in February.

**Rules.**
1. VAT is 23 % of the month's invoiced revenue, due by the 25th of the next month; clients pay gross.
2. The social contribution is 400.00 a month in the business's first 24 months (August 2023 is month 1), 1 400.00 after.
3. The health contribution is 350.00 a month while the year's revenue to date is not more than 60 000.00; 600.00 up to 300 000.00; 1 050.00 above.

**Question.** March's VAT and its due date; the social and health contributions; the cash received in March; the late invoices.

**Worked by hand.** VAT 23 % × 9 500 = **2 185.00**, due **25 April 2025**. March is month 20: **400.00**. Revenue to date 26 500: **350.00**. Cash: 4 920.00 + 3 075.00 = **7 995.00**. Late: **#8**.

```yaml
edge: "LIFE-05"
inputs: {"month": "2025-03", "started": "2023-08", "earlier": [8000.00, 9000.00], "vat": 0.23,
  "invoices": [["#7", 4000.00, "2025-03-28"], ["#8", 3000.00, "2025-04-02"], ["#9", 2500.00, "2025-03-30"]], "due": "2025-03-31",
  "social": {"reduced": 400.00, "full": 1400.00, "months": 24},
  "health": [{"lo": null, "lo_incl": false, "hi": 60000.00, "hi_incl": true, "value": 350.00}, {"lo": 60000.00, "lo_incl": false, "hi": 300000.00, "hi_incl": true, "value": 600.00}, {"lo": 300000.00, "lo_incl": false, "hi": null, "hi_incl": false, "value": 1050.00}]}
expect: {"kind": "value", "value": {"vat": 2185.00, "vat_due": "2025-04-25", "social": 400.00, "health": 350.00, "cash_march": 7995.00, "late": ["#8"]}}
```
