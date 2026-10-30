# W2 scratch: prints the appendix rows `| M | `glob` | n |` for the globs on stdin; refuses a mixed glob.
import csv, fnmatch, os, sys
from collections import Counter
root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
w = {r['id']: r['milestone'] for r in csv.DictReader(open(os.path.join(root, 'tmp/session8b/drafts/BUILD-ORDER-placement.tsv')), delimiter='\t')}
seen = set()
for g in sys.stdin.read().split():
    hit = [i for i in w if fnmatch.fnmatchcase(i, g)]
    c = Counter(w[i] for i in hit)
    if len(c) != 1: sys.exit('mixed or empty: %s %s' % (g, dict(c)))
    dup = seen & set(hit)
    if dup: sys.exit('overlap: %s %s' % (g, sorted(dup)[:3]))
    seen |= set(hit)
    print('| %s | `%s` | %d |' % (list(c)[0], g, len(hit)))
print('named:', len(seen), file=sys.stderr)
