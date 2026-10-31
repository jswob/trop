"""placement_check.py: the build order's placement check (session 8b, writer W2). A document check, not engine code.

Reads engine/tests/checks.tsv, tmp/session8b/drafts/BUILD-ORDER-placement.tsv and the draft
tmp/session8b/drafts/BUILD-ORDER.md. Prints every check placed zero times or more than once, every placement
id not in checks.tsv, every unknown milestone, and the count per milestone. Then it reads the draft's
appendix (rows `| M | `glob` | n |`), expands each glob against checks.tsv, and checks that it names n checks,
all placed in M; and the draft's `Counts:` line against the TSV. Exits 1 on any fault.
Run: python3 -I -B tmp/session8b/agents/W2/placement_check.py [--checks P] [--placement P] [--doc P]
"""
import csv, fnmatch, os, re, sys
from collections import Counter

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
ORDER = ['S', 'M0', 'M1', 'M2', 'M3', 'M4', 'M5a', 'M5b', 'M5c', 'M6', 'M7', 'M8', 'M9']

def arg(name, default):
    a = sys.argv
    return a[a.index(name) + 1] if name in a else os.path.join(ROOT, default)

def main():
    checks_p = arg('--checks', 'engine/tests/checks.tsv')
    place_p = arg('--placement', 'tmp/session8b/drafts/BUILD-ORDER-placement.tsv')
    doc_p = arg('--doc', 'tmp/session8b/drafts/BUILD-ORDER.md')
    faults = 0
    ids = [r['id'] for r in csv.DictReader(open(checks_p), delimiter='\t')]
    if len(ids) != len(set(ids)):
        print('FAULT checks.tsv repeats an id'); faults += 1
    rows = list(csv.DictReader(open(place_p), delimiter='\t'))
    if not rows or list(rows[0].keys()) != ['id', 'milestone', 'why']:
        print('FAULT placement columns are not id, milestone, why'); return 1
    n = Counter(r['id'] for r in rows)
    known = set(ids)
    for i in ids:
        if n[i] == 0: print('FAULT placed 0 times:', i); faults += 1
        elif n[i] > 1: print('FAULT placed %d times: %s' % (n[i], i)); faults += 1
    for i in n:
        if i not in known: print('FAULT not in checks.tsv:', i); faults += 1
    where = {}
    for r in rows:
        if r['milestone'] not in ORDER: print('FAULT unknown milestone %r for %s' % (r['milestone'], r['id'])); faults += 1
        if not r['why'].strip(): print('FAULT no why for', r['id']); faults += 1
        where[r['id']] = r['milestone']
    count = Counter(where.values())
    line = ' '.join('%s=%d' % (m, count[m]) for m in ORDER)
    print('checks.tsv: %d checks; placement: %d rows' % (len(ids), len(rows)))
    print('per milestone:', line, '| total', sum(count.values()))
    # the draft: its globs and its Counts line
    if not os.path.isfile(doc_p):
        print('FAULT no draft at', doc_p); return 1
    doc = open(doc_p).read()
    globs = re.findall(r'^\|\s*(S|M\d[abc]?)\s*\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|', doc, re.M)
    named = set()
    for m, g, k in globs:
        hit = [i for i in ids if fnmatch.fnmatchcase(i, g)]
        named.update(hit)
        bad = [i for i in hit if where.get(i) != m]
        if len(hit) != int(k): print('FAULT glob %s (%s) names %d checks, the draft says %s' % (g, m, len(hit), k)); faults += 1
        if not hit: print('FAULT glob %s names nothing' % g); faults += 1
        for i in bad: print('FAULT glob %s says %s, the placement says %s: %s' % (g, m, where.get(i), i)); faults += 1
    print('draft globs: %d, naming %d of %d checks' % (len(globs), len(named), len(ids)))
    c = re.search(r'^Counts: (.+)$', doc, re.M)
    if not c: print('FAULT the draft has no Counts: line'); faults += 1
    elif c.group(1).strip() != line: print('FAULT the draft counts %r, the placement %r' % (c.group(1).strip(), line)); faults += 1
    print('faults:', faults)
    return 1 if faults else 0

if __name__ == '__main__':
    sys.exit(main())
