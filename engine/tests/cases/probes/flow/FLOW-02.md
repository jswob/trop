# FLOW-02 · One event, several legs

**Story.** Ania and Tomek sell their car on 15 September 2024 for 42 000.00 DUK. The buyer pays through a dealer, who sends the credit's payoff, 18 350.00, to the lender and keeps a fee of 600.00.

**Rules.**
1. The sale is one event with legs: the lender's payoff, the dealer's fee and the rest to the owners.
2. The legs settle together and add up to the price.

**Question.** Each leg.

**Worked by hand.** Lender 18 350.00; fee 600.00; owners 42 000 − 18 350 − 600 = **23 050.00**; the legs add to 42 000.00.

```yaml
edge: "FLOW-02"
inputs: {"price": 42000.00, "payoff": 18350.00, "fee": 600.00}
expect: {"kind": "relation", "value": {"lender": 18350.00, "dealer_fee": 600.00, "owners": 23050.00, "legs_sum": 42000.00}}
```
