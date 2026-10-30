# W2 scratch: for each check, the codes its stored baseline prints (LOAD-, RUN-, HOLE-, WARN-) and its first line.
import csv, os, re, sys
root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
rows = list(csv.DictReader(open(os.path.join(root, 'engine/tests/checks.tsv')), delimiter='\t'))
pat = re.compile(r'\b(LOAD|RUN|HOLE|WARN)-[A-Z-]+')
sel = sys.argv[1] if len(sys.argv) > 1 else ''
for r in rows:
    if not r['id'].startswith(sel): continue
    b = r['baseline']
    p = os.path.join(root, b) if b else ''
    if not b or not os.path.isfile(p):
        print(f"{r['id']}\t-\t(no file: {b[:40]})"); continue
    t = open(p, errors='replace').read()
    codes = sorted(set(m.group(0) for m in pat.finditer(t)))
    first = t.strip().splitlines()[0][:70] if t.strip() else ''
    print(f"{r['id']}\t{len(t)}\t{' '.join(codes)[:120]}\t| {first}")
