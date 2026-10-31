"""F-11 (4.5f): household A's tax.tally to PIT-6 .4/.5's reading, in every copy holding the lines. Line replacements
only; each old line must be found exactly once in each copy."""
R = '/Users/jswob/projects/trop/engine/tests/integration/'
COPIES = ['households/A/tax.tally', 'cuts/A/cut05/A/tax.tally', 'cuts/E1/cdB/households/A/tax.tally']
REP = [
("""  note "PIT-6 .4, PIT-5 .4: the spouse's part less the half of the child relief, never below 0, less the spouse's advances; reading E of pit6_health also takes the health deductible off under PIT-5 v1, rounded as PIT-5 v1 .3 rounds a tax"
""", """  note "PIT-6 .5, PIT-5 .4: the spouse's part less the half of the child relief, less the spouse's advances, with no floor (the floor is the joint tax's, PIT-6 .4 and PIT-7 .5: together_joint); reading E of pit6_health also takes the health deductible off under PIT-5 v1, rounded as PIT-5 v1 .3 rounds a tax"
"""),
("""  rule health beats reliefs v pit5_v1 = round(max(0, jpart[s, y] - relief_half[s, y] - hded_p[s, y]), 1, half-up) - adv_p[s, y] when joint_may[y] and pit6_health == health_off
""", """  rule health beats reliefs v pit5_v1 = round(jpart[s, y] - relief_half[s, y] - hded_p[s, y], 1, half-up) - adv_p[s, y] when joint_may[y] and pit6_health == health_off
"""),
("""  rule reliefs = max(0, jpart[s, y] - relief_half[s, y]) - adv_p[s, y] when joint_may[y]
""", """  rule reliefs = jpart[s, y] - relief_half[s, y] - adv_p[s, y] when joint_may[y]
"""),
("""  note "PIT-6: both spouses' results of the joint return; none where PIT-6 .1 doesn't hold"
""", """  note "PIT-6: both spouses' results of the joint return; none where PIT-6 .1 doesn't hold. PIT-6 .4: both reliefs come off the joint tax (the spouses' parts together), never below 0, then both advances; with no floor reached it is the sum of the spouses' results"
"""),
("""  rule base = sum(res_joint[s, y] for s in spouse) when joint_may[y]
""", """  rule base = max(0, sum(res_joint[s, y] + adv_p[s, y] for s in spouse)) - sum(adv_p[s, y] for s in spouse) when joint_may[y]
"""),
]
for c in COPIES:
    s = open(R + c, encoding='utf-8').read()
    for a, b in REP:
        assert s.count(a) == 1, (c, a[:60])
        s = s.replace(a, b)
    open(R + c, 'w', encoding='utf-8').write(s)
print('F-11: %d lines in each of %d copies' % (len(REP), len(COPIES)))
