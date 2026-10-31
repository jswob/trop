"""Rule 3 of the 4.5f brief: a stored output that changes only because it prints a source path is re-recorded on
refcore, and this shows the old and new differ in the path prefix only.

    python3 -I -B tmp/session4_5/f/tools/prefix_diff.py OUT_DIR ID...        check
    python3 -I -B tmp/session4_5/f/tools/prefix_diff.py OUT_DIR ID... --write   and then replace the baselines

For each check: the old baseline, with every absolute path under the old root replaced by its moved path as the
check's cwd sees it (layout.new, then relative to the cwd), must equal the new refcore output OUT_DIR/<id>.out
byte for byte, and the two must differ (else nothing to re-record)."""
import csv, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import layout
ROOT = layout.ROOT
ABS = re.compile(r'/Users/jswob/projects/trop/([^\s()\'",:]+)')
out_dir, ids = sys.argv[1], [a for a in sys.argv[2:] if not a.startswith('--')]
rows = {r['id']: r for r in csv.DictReader(open(ROOT + 'engine/tests/checks.tsv', encoding='utf-8', newline=''),
                                           delimiter='\t', quoting=csv.QUOTE_NONE)}
ok = True
for i in ids:
    c = rows[i]
    old = open(ROOT + c['baseline'], encoding='utf-8', errors='surrogateescape', newline='').read()
    new = open(os.path.join(out_dir, i.replace('/', '__').replace(':', '_') + '.out'), encoding='utf-8',
               errors='surrogateescape', newline='').read()
    n = [0]

    def sub(m):
        n[0] += 1
        return os.path.relpath(ROOT + layout.new(m.group(1)), ROOT + c['cwd'])
    mapped = ABS.sub(sub, old)
    good = mapped == new and old != new
    ok &= good
    print('%s %s: %d path(s) rewritten; old -> new %s' % ('ok  ' if good else 'FAIL', i, n[0],
          'differs in the path prefix only' if good else 'differs elsewhere too' if old != new else 'no change'))
    if good and '--write' in sys.argv:
        open(ROOT + c['baseline'], 'w', encoding='utf-8', errors='surrogateescape', newline='').write(new)
sys.exit(0 if ok else 1)
