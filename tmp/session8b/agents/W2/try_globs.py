# W2 scratch: for each glob given on stdin (one per line), the milestones its checks are placed in.
import csv, fnmatch, os, sys
from collections import Counter
root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
w = {r['id']: r['milestone'] for r in csv.DictReader(open(os.path.join(root, 'tmp/session8b/drafts/BUILD-ORDER-placement.tsv')), delimiter='\t')}
for g in sys.stdin.read().split():
    c = Counter(m for i, m in w.items() if fnmatch.fnmatchcase(i, g))
    print(g, dict(c), 'PURE' if len(c) == 1 else 'MIXED')
