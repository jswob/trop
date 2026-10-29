#!/usr/bin/env python3
"""The map's coverage from tags.tsv (session 4.5c; 4.5b counted the same by hand-run scripts).

    python3 -I -B tmp/tests/map/coverage.py            # the table
    python3 -I -B tmp/tests/map/coverage.py --unmet    # also the unmet axis pairs

The catalogue is tmp/cases/boundaries.md (`### <EDGE> · <name>`, ★ for an axis's hardest edge). Integration means
the household questions (`I:<H>:<qid>`, H in A, B, D, E1) plus the old models (`I:M:*`, not the `~key` scripts);
the amendments household (`I:X:*`) and the blind texts (`I:hardset/*`) are counted in their own rows only.
A pair is two distinct edges tagged on one check; an axis pair counts same-axis pairs too (15 axes: 120).
"""
import itertools
import re
import sys

ROOT = '/Users/jswob/projects/trop/'
edges, star = [], set()
for l in open(ROOT + 'tmp/cases/boundaries.md', encoding='utf-8'):
    m = re.match(r'^### ([A-Z]+-\d+) · .*?(★)?\s*$', l)
    if m:
        edges.append(m.group(1))
        if m.group(2):
            star.add(m.group(1))
axis = lambda e: e.split('-')[0]
axes = sorted({axis(e) for e in edges})
tags = {}
for l in list(open(ROOT + 'tmp/tests/map/tags.tsv', encoding='utf-8'))[1:]:
    p = l.rstrip('\n').split('\t')
    tags[p[0]] = [e for e in p[2].split() if e in edges or re.match(r'^[A-Z]+-\d+$', e)]


def kind(cid):
    if cid.startswith('U:'):
        return 'unit'
    if re.match(r'^I:(A|B|D|E1):', cid):
        return 'house'
    if cid.startswith('I:M:') and not cid.endswith('~key'):
        return 'model'
    if cid.startswith('I:X:'):
        return 'X'
    if cid.startswith('I:hardset/'):
        return 'blind'
    return 'other'


def measure(kinds):
    met, pairs, apairs, count = set(), set(), set(), {}
    for cid, es in tags.items():
        if kind(cid) not in kinds:
            continue
        es = sorted(set(es))
        met |= set(es)
        for e in es:
            count[e] = count.get(e, 0) + 1
        for a, b in itertools.combinations(es, 2):
            pairs.add((a, b))
            apairs.add(tuple(sorted((axis(a), axis(b)))))
    return met, pairs, apairs, count


unit = measure({'unit'})[0]
rows = []
for name, ks in (('households', {'house'}), ('+ old models', {'house', 'model'}),
                 ('+ amendments household', {'house', 'model', 'X'}), ('+ blind texts', {'house', 'model', 'X', 'blind'})):
    met, pairs, apairs, count = measure(ks)
    cross = sum(1 for a, b in apairs if a != b)
    st = sum(1 for e in star if count.get(e, 0) >= 2)
    rows.append((name, len(met), len(apairs), cross, len(apairs) - cross, len(pairs), st))
print('edges in the catalogue: %d on %d axes, %d ★; with a unit check: %d' % (len(edges), len(axes), len(star),
                                                                            len(unit & set(edges))))
print('| checks | edges met | axis pairs (of 120) | across | within | edge pairs (of %d) | ★ met ≥ 2 (of %d) |' % (
    len(edges) * (len(edges) - 1) // 2, len(star)))
print('|---|---|---|---|---|---|---|')
for r in rows:
    print('| %s | %d | %d | %d | %d | %d | %d |' % r)
if '--unmet' in sys.argv:
    met, pairs, apairs, count = measure({'house', 'model'})
    allp = {tuple(sorted(p)) for p in itertools.combinations_with_replacement(axes, 2)}
    print('unmet axis pairs (households + old models):', ' '.join('%s–%s' % p for p in sorted(allp - apairs)))
    print('★ edges by integration checks:', ' '.join('%s %d' % (e, count.get(e, 0)) for e in sorted(star)))
