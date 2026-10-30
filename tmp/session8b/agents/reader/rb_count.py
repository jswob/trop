import re, collections, sys
p = sys.argv[1] if len(sys.argv) > 1 else 'tmp/session8b/drafts/RULEBOOK.md'
lines = open(p).read().split('\n')
grp = None
rows = []
for n, l in enumerate(lines, 1):
    m = re.match(r'^## (\d+)\. (.+)', l)
    if m: grp = (int(m.group(1)), m.group(2)); continue
    m = re.match(r'^\| (R-\d+) \|', l)
    if m:
        cells = [c.strip() for c in l.strip().strip('|').split('|')]
        rows.append((n, m.group(1), grp[0] if grp else None, len(cells), cells))
print('rows', len(rows))
ids = [r[1] for r in rows]
nums = sorted(int(i[2:]) for i in ids)
dups = [i for i, c in collections.Counter(ids).items() if c > 1]
print('dups', dups)
missing = [i for i in range(1, max(nums) + 1) if i not in nums]
print('missing', missing)
cellcounts = collections.Counter(r[3] for r in rows)
print('cell counts', cellcounts)
tab = collections.defaultdict(lambda: collections.Counter())
for n, i, g, k, c in rows:
    rec = c[-2]
    key = 'drop' if rec.startswith('drop') else ('report' if rec.startswith('report') and '→ keep' not in rec else ('keep' if rec.startswith('keep') or rec.endswith('→ keep') or '→ keep' in rec else rec))
    tab[g][key] += 1
tot = collections.Counter()
for g in sorted(tab):
    print(g, dict(tab[g]), sum(tab[g].values())); tot.update(tab[g])
print('total', dict(tot), sum(tot.values()))
for n, i, g, k, c in rows:
    rec = c[-2]
    if rec not in ('keep', 'drop', 'report'):
        print(n, i, 'g%s' % g, 'REC=', rec[:60], '| ANS=', c[-1][:90])
