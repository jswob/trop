"""git mv every row of moves.tsv, grouped by target directory (basenames never change)."""
import collections, os, subprocess, sys
rows = [l.rstrip('\n').split('\t') for l in open(sys.argv[1], encoding='utf-8')][1:]
assert all(os.path.basename(o) == os.path.basename(n) for o, n in rows)
by = collections.defaultdict(list)
for o, n in rows:
    by[os.path.dirname(n)].append(o)
for d, srcs in sorted(by.items()):
    os.makedirs(d, exist_ok=True)
    for i in range(0, len(srcs), 200):
        subprocess.run(['git', 'mv'] + srcs[i:i + 200] + [d + '/'], check=True)
print(len(rows), 'moved into', len(by), 'directories')
