import csv, random
ck = {r['id']: r for r in csv.DictReader(open('engine/tests/checks.tsv'), delimiter='\t')}
pl = list(csv.DictReader(open('tmp/session8b/drafts/BUILD-ORDER-placement.tsv'), delimiter='\t'))
print(list(next(iter(ck.values())).keys()))
random.seed(8)
# prefer those with non-trivial why: pick spread across milestones
import collections
by = collections.defaultdict(list)
for r in pl: by[r['milestone']].append(r)
picks=[]
for m in ['S','M0','M1','M2','M4','M5a','M5b','M5c','M6','M7','M9']:
    picks += random.sample(by[m], 1)
for m in ['M1','M4','M5a','M5b']:
    picks += random.sample(by[m], 1)
for r in picks:
    c = ck[r['id']]
    print('==', r['id'], '->', r['milestone'])
    print('  why:', r['why'][:300])
    for k in ('question','note','files'):
        print('  %s: %s' % (k, (c.get(k) or '')[:300]))
