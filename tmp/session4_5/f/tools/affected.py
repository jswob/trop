"""Checks whose inputs (files, and what their .tally files reach by `use`, transitively) include any given path or
path prefix: python3 -I -B affected.py PATH... -> ids, by layer, with not-run ones apart."""
import csv, os, re, sys
ROOT = '/Users/jswob/projects/trop/'
USE = re.compile(r'^\s*use\s+\w+\s*=\s*"([^"]+)"', re.M)
rows = list(csv.DictReader(open(ROOT + 'engine/tests/checks.tsv', encoding='utf-8', newline=''), delimiter='\t', quoting=csv.QUOTE_NONE))
def closure(f, seen):
    if f in seen: return
    seen.add(f)
    if f.endswith('.tally') and os.path.isfile(ROOT + f):
        for t in USE.findall(open(ROOT + f, encoding='utf-8', errors='replace').read()):
            n = os.path.normpath(os.path.join(os.path.dirname(f), t))
            if os.path.isdir(ROOT + n):
                for d, _, fs in os.walk(ROOT + n):
                    for x in fs:
                        closure(os.path.relpath(os.path.join(d, x), ROOT), seen)
            else:
                closure(n, seen)
want = sys.argv[1:]
hit = {}
for c in rows:
    s = set()
    for f in c['files'].split():
        closure(f, s)
    if any(any(f == w or f.startswith(w.rstrip('/') + '/') for w in want) for f in s):
        k = (c['layer'], 'not-run' if c['note'].startswith('not-run') else 'run')
        hit.setdefault(k, []).append(c['id'])
for k, v in sorted(hit.items()):
    print('%s %s %d: %s' % (k[0], k[1], len(v), ' '.join(v[:12]) + (' …' if len(v) > 12 else '')))
