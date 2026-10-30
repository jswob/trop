# Key · OWN-10

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-10"
key: {"p01": {"kind": "value", "value": {"parts": {"ania": 3000.00, "ania_tomek": 1112.34, "tomek": 0}, "total": 4112.34}}, "p02": {"kind": "value", "value": {"sav_a": {"ania": 1200.00, "ania_tomek": 0, "tomek": 0}, "sav_b": {"ania": 800.00, "ania_tomek": 1000.00, "tomek": 0}}}, "p03": {"kind": "value", "value": {"fund": {"ania": 33, "ania_tomek": 0, "tomek": 67}, "total": 100}}, "p07": {"kind": "value", "value": {"parts": {"ania, r1": 20, "ania, r2": 10, "ania, r3": 15, "tomek, r1": 0, "tomek, r2": 15, "tomek, r3": 0}, "total": 60, "gift_tax": {"ania": 0.00, "tomek": 40.04}}}, "p08": {"kind": "value", "value": {"2026-03-31": {"sav": {"ania": 1500.00, "kuba": 0, "tomek": 0, "tomek_estate": 7000.00}, "fund": {"ania": 10, "kuba": 0, "tomek": 0, "tomek_estate": 45}}, "2026-06-30": {"sav": {"ania": 3833.33, "kuba": 4666.67, "tomek": 0, "tomek_estate": 0}, "fund": {"ania": 10, "kuba": 45, "tomek": 0, "tomek_estate": 0}}}}, "p09": {"kind": "value", "value": {"parts": {"ania": 2800.01, "ania_tomek": 0, "tomek": 2500.00}, "total": 5300.01}}, "p11": {"kind": "value", "value": {"2026-03-31": {"units": {"ania": 40, "tomek": 60}, "price": 25.00, "value": {"ania": 1000.00, "tomek": 1500.00}}, "2026-04-30": {"units": {"ania": 40, "tomek": 60}, "price": 27.50, "value": {"ania": 1100.00, "tomek": 1650.00}}, "gain": {"ania": 100.00, "tomek": 150.00}}}, "p13": {"kind": "value", "value": {"booked": {"ania": 3000.00, "ania_tomek": 5300.00, "tomek": 0}, "gap": 0.00, "own_gaps": {}, "parts": {"ania": 3000.00, "ania_tomek": 5300.00, "tomek": 0}}}, "p14": {"kind": "value", "value": {"booked": {"ania": 3000.00, "ania_tomek": 5300.00, "tomek": 0}, "gap": 100.00, "own_gaps": {"ania": 100.00}, "parts": {"ania": 3100.00, "ania_tomek": 5300.00, "tomek": 0}}}, "p15": {"kind": "value", "value": {"as_known_on": {"sav": {"ania": 0, "ania_tomek": 1000.00, "tomek": 0}, "acc_ania": {"ania": 2500.00, "ania_tomek": 0, "tomek": 0}}, "as_known_before": {"sav": {"ania": 0, "ania_tomek": 3000.00, "tomek": 0}, "acc_ania": {"ania": 500.00, "ania_tomek": 0, "tomek": 0}}}}, "p16": {"kind": "value", "value": {"fund_at_start": {"ania": 30, "ania_tomek": 0, "tomek": 10}, "sav": {"ania": 75.01, "ania_tomek": 0, "tomek": 25.00}, "fund": {"ania": 30, "ania_tomek": 0, "tomek": 30}}}, "p18": {"2026-01-10": {"kind": "value", "value": {"ania": 3000.00, "ania_tomek": 0, "tomek": 0}}, "2026-01-31": {"kind": "refusal", "code": "RUN-NO-RULE", "missing": ["a rule of arrive for r2 (transfers)"], "reason": "RUN-NO-RULE: no rule applies to arrive[r2, p] on 2026-01-20"}}, "p24": {"2026-03-09": {"sav": {"kind": "value", "value": {"ania": 1500.00, "kuba": 0, "tomek": 7000.00, "tomek_estate": 0}}, "fund": {"kind": "value", "value": {"ania": 10, "kuba": 0, "tomek": 45, "tomek_estate": 0}}}, "2026-03-31": {"sav": {"kind": "refusal", "code": "RUN-LIFESPAN", "missing": ["tomek's part labelled after his death's posting on 2026-03-10"], "reason": "RUN-LIFESPAN: final_pay labels a part of sav to tomek after his death's posting on 2026-03-10"}, "fund": {"kind": "value", "value": {"ania": 10, "kuba": 0, "tomek": 0, "tomek_estate": 45}}}}}
```

## Derivation

1. (P-01) parts of sav at the end of 2026-01-31: an inheritance the heir's, interest and the rest the couple's; the withdrawal takes the couple's part first (rules 1-2) = {"ania": 3000.00, "ania_tomek": 1112.34, "tomek": 0}
2. (P-02) parts of sav_a at the end of 2026-01-31: the transfer took the couple's part first (rule 2) = {"ania": 1200.00, "ania_tomek": 0, "tomek": 0}
3. (P-02) parts of sav_b at the end of 2026-01-31: the transfer's arrival keeps its labels: each party's share is what the leg took of it over what it took (rule 1) = {"ania": 800.00, "ania_tomek": 1000.00, "tomek": 0}
4. (P-03) units of the fund by owner at the end of 2026-01-31: each owner's share is what the purchase's cash legs took of their parts over what they paid; split rest largest to the step of 1 unit (rules 1-2) = {"ania": 33, "ania_tomek": 0, "tomek": 67}
5. (P-07) parts of the fund by (owner, batch) at the end of 2026-03-31: the gift takes the giver's parts first, oldest batch first, and returns them labelled the receiver's, batches kept (rules 1-3) = {"ania, r1": 20, "ania, r2": 10, "ania, r3": 15, "tomek, r1": 0, "tomek, r2": 15, "tomek, r3": 0}
6. (P-07) gift tax on r4 for giver ania: round(max(0, 0 units given × 52.40 − 1000.00) × 0.07, 0.01, half up) (rule 4) = 0.00
7. (P-07) gift tax on r4 for giver tomek: round(max(0, 30 units given × 52.40 − 1000.00) × 0.07, 0.01, half up) (rule 4) = 40.04
8. (P-08) parts of sav at the end of 2026-03-31: the death posting relabels all the deceased's parts to the estate (rules 2-3); the division relabels the estate's parts to each heir by the agreement's share for that place, rest largest (rules 4-5) = {"ania": 1500.00, "kuba": 0, "tomek": 0, "tomek_estate": 7000.00}
9. (P-08) parts of fund at the end of 2026-03-31: the death posting relabels all the deceased's parts to the estate (rules 2-3); the division relabels the estate's parts to each heir by the agreement's share for that place, rest largest (rules 4-5) = {"ania": 10, "kuba": 0, "tomek": 0, "tomek_estate": 45}
10. (P-08) parts of sav at the end of 2026-06-30: the death posting relabels all the deceased's parts to the estate (rules 2-3); the division relabels the estate's parts to each heir by the agreement's share for that place, rest largest (rules 4-5) = {"ania": 3833.33, "kuba": 4666.67, "tomek": 0, "tomek_estate": 0}
11. (P-08) parts of fund at the end of 2026-06-30: the death posting relabels all the deceased's parts to the estate (rules 2-3); the division relabels the estate's parts to each heir by the agreement's share for that place, rest largest (rules 4-5) = {"ania": 10, "kuba": 45, "tomek": 0, "tomek_estate": 0}
12. (P-09) parts of sav at the end of 2026-05-31: the divorce posting relabels the couple's parts to each spouse by stake; the odd step to the lower odd_step_rank (rules 2-4) = {"ania": 2800.01, "ania_tomek": 0, "tomek": 2500.00}
13. (P-11) value by owner at the end of 2026-03-31: units × the last price on or before 2026-03-31, 25.00 (rules 2-3) = {"ania": 1000.00, "tomek": 1500.00}
14. (P-11) value by owner at the end of 2026-04-30: units × the last price on or before 2026-04-30, 27.50 (rules 2-3) = {"ania": 1100.00, "tomek": 1650.00}
15. (P-11) gain by owner: value at 2026-04-30 − value at 2026-03-31: units × the price's rise; nothing booked (rule 3) = {"ania": 100.00, "tomek": 150.00}
16. (P-13) opening parts of sav at 2026-01-01 start: stated beside the opening total 8000.00 (rule 1) = {"ania": 3000.00, "ania_tomek": 5000.00}
17. (P-13) booked parts of sav at the end of 2026-01-31: the opening parts, the deposit the couple's, the withdrawal the couple's part first (rules 1-3) = {"ania": 3000.00, "ania_tomek": 5300.00}
18. (P-13) the total's gap: stated 8300.00 − booked (rule 4) = 0.00
19. (P-13) each stated part's own gap: stated part − booked part (rule 5) = {}
20. (P-13) the unstated rest's gap: (stated total − stated parts) − the rest's booked parts (rule 5) = 0.00
21. (P-13) parts of sav after the reading: an anchor whose gaps are explained keeps the parts (rules 4-5) = {"ania": 3000.00, "ania_tomek": 5300.00, "tomek": 0}
22. (P-14) opening parts of sav at 2026-01-01 start: stated beside the opening total 8000.00 (rule 1) = {"ania": 3000.00, "ania_tomek": 5000.00}
23. (P-14) booked parts of sav at the end of 2026-01-31: the opening parts, the deposit the couple's, the withdrawal the couple's part first (rules 1-3) = {"ania": 3000.00, "ania_tomek": 5300.00}
24. (P-14) the total's gap: stated 8400.00 − booked (rule 4) = 100.00
25. (P-14) each stated part's own gap: stated part − booked part (rule 5) = {"ania": 100.00}
26. (P-14) the unstated rest's gap: (stated total − stated parts) − the rest's booked parts (rule 5) = 0.00
27. (P-14) parts of sav after the reading: an anchor whose gaps are explained keeps the parts (rules 4-5) = {"ania": 3100.00, "ania_tomek": 5300.00, "tomek": 0}
28. (P-15) cut 2026-10-30: the deed rows seen: the ruling, known later, replaces the row (rule 3) = {"d1": true}
29. (P-15) cut 2026-10-30: the sweep took: the past decision reads Ania's part at the walk's cut, so it follows the ruling (rule 4) = {"ania": 2500.00}
30. (P-15) cut 2026-10-30: parts of sav at the end of 2026-03-31: labels are the walk's at the cut (rules 1-3) = {"ania": 0, "ania_tomek": 1000.00, "tomek": 0}
31. (P-15) cut 2026-10-30: parts of acc_ania at the end of 2026-03-31: the sweep's arrival keeps its labels (rule 2) = {"ania": 2500.00, "ania_tomek": 0, "tomek": 0}
32. (P-15) cut 2026-04-01: the deed rows seen: the ruling, known later, replaces the row (rule 3) = {"d1": false}
33. (P-15) cut 2026-04-01: the sweep took: the past decision reads Ania's part at the walk's cut, so it follows the ruling (rule 4) = {"ania": 500.00}
34. (P-15) cut 2026-04-01: parts of sav at the end of 2026-03-31: labels are the walk's at the cut (rules 1-3) = {"ania": 0, "ania_tomek": 3000.00, "tomek": 0}
35. (P-15) cut 2026-04-01: parts of acc_ania at the end of 2026-03-31: the sweep's arrival keeps its labels (rule 2) = {"ania": 500.00, "ania_tomek": 0, "tomek": 0}
36. (P-16) units of the fund at the start of 2026-03-31: before that day's movements (rule 2) = {"ania": 30, "ania_tomek": 0, "tomek": 10}
37. (P-16) for contrast: units at the end of 2026-03-31: the same day's purchase is in them; the rule doesn't read them = {"ania": 30, "ania_tomek": 0, "tomek": 30}
38. (P-16) parts of sav at the end of 2026-03-31: the income of 100.01 split by the start-of-day units, rest largest to 0.01 (rules 1-2) = {"ania": 75.01, "ania_tomek": 0, "tomek": 25.00}
39. (P-18) parts of sav at the end of 2026-01-10: rules 1-2 reach every arrival so far = {"ania": 3000.00, "ania_tomek": 0, "tomek": 0}
40. (P-18) parts of sav at the end of 2026-01-31: refused: RUN-NO-RULE: no rule applies to arrive[r2, p] on 2026-01-20 = RUN-NO-RULE
41. (P-24) parts of sav at the end of 2026-03-09: as P-08 = {"ania": 1500.00, "kuba": 0, "tomek": 7000.00, "tomek_estate": 0}
42. (P-24) parts of fund at the end of 2026-03-09: as P-08 = {"ania": 10, "kuba": 0, "tomek": 45, "tomek_estate": 0}
43. (P-24) sav at the end of 2026-03-31: refused: RUN-LIFESPAN: final_pay labels a part of sav to tomek after his death's posting on 2026-03-10 = RUN-LIFESPAN
44. (P-24) parts of fund at the end of 2026-03-31: as P-08 = {"ania": 10, "kuba": 0, "tomek": 0, "tomek_estate": 45}
