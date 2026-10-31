# PTY-05 × OWN-12 · One party's parts across a nested joint model

**Story.** Households A (Ania and Tomek), D (Halina) and B (Piotr) each keep one account as parts by owner. A and D are first joined as one family model, `fam`; the whole family is then a joint model of `fam` and B, so one of its members is itself a joint model (tally 2.1, LANGUAGE §6.1; ENGINE §3.1 step 1: a member that is a joint model pins its own members through its `use` lines). A's `sav` receives Ania's inheritance of 2 000.00 (5 Jan) and Tomek's of 1 500.00 (6 Jan); D's `deposit` receives Halina's of 10 000.00 (5 Jan) and Ania's of 4 000.00 (7 Jan); B's `pot` receives Piotr's of 3 000.00 (5 Jan) and Ania's of 700.00 (9 Jan). Amounts are DUK, step 0.01.

**Rules.**
1. In each household an inheritance arriving at its account is the heir's.
2. Each household enters its people as parties of the prelude's `parties`; Ania appears in all three.
3. `fam` states that A's Ania, D's Ania and its own `ania` are one party, and reads her parts across its two members; the family model states that `fam`'s `ania` and B's Ania are one, and reads her parts at B.

**Question.** Each household's parts at the end of 30 June 2026; Ania's parts across `fam`; her part of B's account read through the family model; her total across the family.

**The answers** are the key function's (`PTY_05__OWN_12` in `tools/probefns/pairs.py`), copied into the block below by script; the derivation is `keys/probes/PTY-05+OWN-12.md`.

```yaml
edge: ["PTY-05", "OWN-12"]
inputs: {"asked": "2026-06-30", "members": {"a": {"place": "sav", "parties": ["ania", "tomek"], "rows": [{"id": "r1", "on": "2026-01-05", "kind": "inheritances", "from": "world", "to": "sav", "amount": 2000.00, "heir": "ania"}, {"id": "r2", "on": "2026-01-06", "kind": "inheritances", "from": "world", "to": "sav", "amount": 1500.00, "heir": "tomek"}]}, "d": {"place": "deposit", "parties": ["ania", "halina"], "rows": [{"id": "r1", "on": "2026-01-05", "kind": "inheritances", "from": "world", "to": "deposit", "amount": 10000.00, "heir": "halina"}, {"id": "r2", "on": "2026-01-07", "kind": "inheritances", "from": "world", "to": "deposit", "amount": 4000.00, "heir": "ania"}]}, "b": {"place": "pot", "parties": ["ania", "piotr"], "rows": [{"id": "r1", "on": "2026-01-05", "kind": "inheritances", "from": "world", "to": "pot", "amount": 3000.00, "heir": "piotr"}, {"id": "r2", "on": "2026-01-09", "kind": "inheritances", "from": "world", "to": "pot", "amount": 700.00, "heir": "ania"}]}}, "inner": {"name": "fam", "party": "ania", "same": {"a": "ania", "d": "ania"}}, "outer": {"party": "fam.ania", "member": "b", "same": "ania"}}
expect: {"kind": "value", "value": {"members": {"a": {"ania": 2000.00, "tomek": 1500.00}, "d": {"ania": 4000.00, "halina": 10000.00}, "b": {"ania": 700.00, "piotr": 3000.00}}, "inner": 6000.00, "outer_member": 700.00, "total": 6700.00}}
```
