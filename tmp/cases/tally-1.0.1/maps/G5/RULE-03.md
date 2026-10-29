# RULE-03 · map
Text: `probes/rule/RULE-03.tally`. Key: a value {R-A 3000.00, R-B 2000.00, left 0.00}; b refusal (priority between R-A, R-B missing).

step 1 · (a) in turn R-A, R-B = {R-A 3000.00, R-B 2000.00}, left 0 · lines 8–23, 25, 32–35, 37 · L §5.11 (procedure steps 1–4, L:850–853; outputs L:855) · same: arcs (surplus, r_a) and (surplus, r_b) sorted by order 1, 2; take r_a = min(5000, 3000) = 3000, R = 2000; take r_b = min(2000, 4000) = 2000; left = 5000 − 5000 = 0.00
step 2 · (b) no priority = refused · lines 18–23, 27, 36 · L §5.11 (L:851: an order that can't be decided, a hole, is refused, RUN-ORDER), L §8.4 (L:1642), L:799 (a refusal outranks a hole) · same: prio[b, *] is a stated hole, so the order of split[b]'s arcs is a hole and the fill is refused; the parts are two keys of one keyed fill, so a's result stands

ask returns: q_a value; q_b refusal · `refused: RUN-ORDER fill split: the order of <arc> is a hole` on each term; the message names an arc, and the hole's own question ("Which of R-A and R-B takes first?") isn't printed, since the refusal outranks it (L:799); whether every undecided arc is named is not said
checks: parsecheck ok · loadcheck ok
