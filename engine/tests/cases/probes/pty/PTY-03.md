# PTY-03 · Succession down the family tree

**Story.** Zbigniew dies, leaving his wife Halina and children Ania, Piotr and Marta. His son Adam died before him, leaving a son, Kuba. There is no will.

**Rules.**
1. The spouse and the children inherit in equal shares; the spouse's share is never below 1/4.
2. A child who died first is represented by his own children, in equal parts.

**Question.** Each heir's share, as an exact fraction.

**Worked by hand.** Five heads give 1/5 each, below 1/4, so Halina takes **1/4** and the four children's branches share 3/4: **3/16** each. Adam's 3/16 goes to Kuba. Shares: Halina 1/4, Ania 3/16, Piotr 3/16, Marta 3/16, Kuba 3/16; they sum to 1.

```yaml
edge: "PTY-03"
inputs: {"spouse": "Halina", "children": [{"name": "Ania", "alive": true}, {"name": "Piotr", "alive": true}, {"name": "Marta", "alive": true}, {"name": "Adam", "alive": false, "issue": ["Kuba"]}]}
expect: {"kind": "value", "value": {"Halina": "1/4", "Ania": "3/16", "Piotr": "3/16", "Marta": "3/16", "Kuba": "3/16"}}
```
