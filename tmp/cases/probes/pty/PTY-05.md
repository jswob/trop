# PTY-05 · Joins across households

**Story.** On 2 May 2024 Halina gives Ania 5 000.00 DUK. Ania's household states it as an outside fact ("received from Halina"); Halina's household derives it from her own records ("given to Ania").

**Rules.**
1. In the union a family event is stated once: the deriving household's statement stands, and the outside fact names it.
2. Joining households changes no answer of either.

**Question.** In the union: how many transfers; each person's change from it; the family's total change; do both households' keys still hold?

**Worked by hand.** One transfer. Ania **+5 000.00**, Halina **−5 000.00**, family total **0.00**. Each household's own key (Ania +5 000, Halina −5 000) is reproduced: holds.

```yaml
edge: "PTY-05"
inputs: {"A": [{"id": "A-G1", "date": "2024-05-02", "from": "Halina", "to": "Ania", "amount": 5000.00, "stated": "outside"}],
  "D": [{"id": "D-G1", "date": "2024-05-02", "from": "Halina", "to": "Ania", "amount": 5000.00, "stated": "derived"}],
  "people": {"A": "Ania", "D": "Halina"}}
expect: {"kind": "relation", "value": {"union_transfers": 1, "Ania": 5000.00, "Halina": -5000.00, "family_total": 0.00, "keys_hold": true}}
```
