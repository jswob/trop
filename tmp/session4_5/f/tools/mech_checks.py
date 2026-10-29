"""checks.tsv at HEAD -> its mechanical rewrite: a cwd-relative file is first joined to its cwd, then paths.rw()."""
import os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import paths
old = subprocess.run(['git', 'show', sys.argv[1]], capture_output=True, text=True, check=True).stdout
out = []
for i, l in enumerate(old.split('\n')):
    if i == 0 or not l:
        out.append(l)
        continue
    c = l.split('\t')
    cwd = c[2]
    c[3] = ' '.join(f if f.startswith('/') else os.path.normpath(os.path.join(cwd, f)) for f in c[3].split())
    out.append(paths.rw('\t'.join(c)))
open(sys.argv[2], 'w', encoding='utf-8').write('\n'.join(out))
