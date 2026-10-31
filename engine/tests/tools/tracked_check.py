"""The tracked-files guard (session 4.5d). Standard library only.

    python3 -I -B engine/tests/tools/tracked_check.py

Lists every file checks.tsv names that git doesn't track: the `files` column (resolved against `cwd`), the files a
listed .tally input reaches by `use a = "path"` (relative to that file, transitively), and the `baseline` column. A
`not-run:` row marked `no-baseline` names a baseline that was never made; it is skipped while that file doesn't exist.
Exits 1 if there is one. Paths in checks.tsv are relative to the repository root (4.5f); a few inputs still live
under tmp/, which .gitignore excludes, so a new file there needs `git add -f`.
"""
import csv
import os
import re
import subprocess
import sys

ROOT = subprocess.run(['git', '-C', os.path.dirname(os.path.abspath(__file__)), 'rev-parse', '--show-toplevel'],
                      capture_output=True, text=True, check=True).stdout.strip() + '/'
USE = re.compile(r'^\s*use\s+\w+\s*=\s*"([^"]+)"', re.M)


def rel(p):
    return os.path.relpath(os.path.normpath(p), ROOT)


def uses(p, seen):
    """Files the .tally file p (repository-relative) reaches through `use` lines; a member may be a directory tree."""
    if p in seen or not p.endswith('.tally') or not os.path.isfile(ROOT + p):
        return
    seen.add(p)
    for target in USE.findall(open(ROOT + p, encoding='utf-8', errors='replace').read()):
        t = os.path.normpath(os.path.join(os.path.dirname(ROOT + p), target))
        found = [os.path.join(d, f) for d, _, fs in os.walk(t) for f in fs if '__pycache__' not in d] \
            if os.path.isdir(t) else [t]
        for f in found:
            seen_files.add(rel(f))
            uses(rel(f), seen)


rows = list(csv.DictReader(open(ROOT + 'engine/tests/checks.tsv', encoding='utf-8', newline=''), delimiter='\t'))
named = set()
seen_files = set()
for c in rows:
    for f in c['files'].split():
        named.add(rel(ROOT + f))
for p in sorted(named):
    uses(p, set())
named |= seen_files
for c in rows:
    b = c['baseline']
    if b.startswith(('tmp/', 'engine/', '/')):
        if c['note'].startswith('not-run:') and 'no-baseline' in c['note'].split() and not os.path.exists(ROOT + b):
            continue
        named.add(rel(b if b.startswith('/') else ROOT + b))
tracked = set(subprocess.run(['git', '-C', ROOT, 'ls-files', '-z'], capture_output=True, check=True)
              .stdout.decode('utf-8').split('\0'))
untracked = sorted(p for p in named if p not in tracked)
for p in untracked:
    print('not tracked: %s%s' % (p, '' if os.path.exists(ROOT + p) else ' (missing)'))
print('%d files named by checks.tsv, %d not tracked by git%s' % (
    len(named), len(untracked), ': add them with git add -f' if untracked else ''))
sys.exit(1 if untracked else 0)
