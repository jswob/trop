# W2 scratch: per I:M model, the constructs a text scan finds that 1.0.1's milestone may not have.
import os, re, csv
root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
rows = list(csv.DictReader(open(os.path.join(root, 'engine/tests/checks.tsv')), delimiter='\t'))
P = [('fill', r'^fill\b'), ('use', r'^\s+use \w+ ='), ('parts', r'^\s+(owned by|batched)\b'),
     ('dist', r'\b(normal|lognormal|uniform|poisson|binomial|gamma|exponential|negbin|discrete)\s*\('),
     ('paths', r'\bpaths\b'), ('transc', r'\b(ln|exp|sqrt|pow|pdf|cdf|pmf|quantile|crps)\s*\(|\bto u\b'),
     ('fx', r'\bat fx\b|\bfx\b'), ('origin', r'as if \{\s*known'), ('solve', r'\bsolve\b'), ('orient', r'\borient\b')]
def scan(files):
    hit = set()
    for f in files.split():
        p = os.path.join(root, f)
        if not os.path.isfile(p): continue
        for line in open(p, errors='replace'):
            line = re.sub(r'"[^"]*"', '""', line).split('#')[0]
            for n, rx in P:
                if re.search(rx, line): hit.add(n)
    return sorted(hit)
import sys
sel = sys.argv[1]
for r in rows:
    if r['id'].startswith(sel):
        print(r['id'], ' '.join(scan(r['files'])))
