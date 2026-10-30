"""The close-out's checks 2-5 (check 1 is merge_gaps.py --check).

Usage, from the repository root: python3 -I -B tmp/cases/tally-1.0.1/tools/final_checks.py
  2. parsecheck over probes/, households/, law/ (not scratch/)
  3. loadcheck on each probe card (a .tally file, or a directory for a multi-file
     card) and on each household with the law (households/<H> + law/, copied
     without PROGRESS.md into scratch/CO/<H>/); each refusal is listed with the
     settled rows that name its code for that card or household
  4. tmp/cases/tools/check.py (its last lines)
  5. no __pycache__ under tmp/cases or engine
Standard library only; parsecheck runs in its own venv.
"""
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # tally-1.0.1/
CASES = os.path.dirname(HERE)
REPO = os.path.dirname(os.path.dirname(CASES))
PC = os.path.join(REPO, 'engine/tools/parsecheck/.venv/bin/python')
HOUSES = ['A', 'B', 'D', 'E1']


def run(cmd):
    p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr


def settled():
    rows = []
    for f in sorted(os.listdir(os.path.join(HERE, 'rows'))):
        if f.endswith('.settled.md'):
            for line in open(os.path.join(HERE, 'rows', f), encoding='utf-8'):
                m = re.match(r'^\|\s*`?([PQOS]:[^`| ]+)`?\s*\|', line)
                if m:
                    rows.append((m.group(1), f.split('.')[0], line))
    return rows


def naming(rows, code, ids=None, chunk=None):
    hits = []
    for rid, ch, line in rows:
        if code not in line:
            continue
        if ids is not None and rid not in ids:
            continue
        if chunk is not None and ch != chunk:
            continue
        hits.append(rid)
    return hits


def main():
    ok = True
    print('== check 2: parsecheck')
    rc, out = run([PC, '-I', '-B', 'engine/tools/parsecheck/parsecheck.py', '-v',
                   'tmp/cases/tally-1.0.1/probes', 'tmp/cases/tally-1.0.1/households',
                   'tmp/cases/tally-1.0.1/law'])
    lines = [l for l in out.splitlines() if not l.endswith(': ok')]
    print('\n'.join(lines[-20:]))
    print('exit', rc)
    ok = ok and rc == 0

    print('== check 3: loadcheck')
    rows = settled()
    cards = []
    pdir = os.path.join(HERE, 'probes')
    for ax in sorted(os.listdir(pdir)):
        for e in sorted(os.listdir(os.path.join(pdir, ax))):
            p = os.path.join(pdir, ax, e)
            if e.endswith('.tally') or os.path.isdir(p):
                cards.append((e.replace('.tally', ''), p))
    realities = [(c, p, 'probe') for c, p in cards]
    sc = os.path.join(HERE, 'scratch', 'CO')
    os.makedirs(sc, exist_ok=True)
    for h in HOUSES:
        dst = os.path.join(sc, h)
        if os.path.exists(dst):
            shutil.rmtree(dst)
        ign = shutil.ignore_patterns('PROGRESS.md', '__pycache__')
        shutil.copytree(os.path.join(HERE, 'households', h), os.path.join(dst, h), ignore=ign)
        shutil.copytree(os.path.join(HERE, 'law'), os.path.join(dst, 'law'), ignore=ign)
        realities.append((h, dst, 'household'))
    print('probe cards: %d; households: %d' % (len(cards), len(HOUSES)))
    fails = 0
    for name, path, kind in realities:
        rc, out = run(['python3', '-I', '-B', 'engine/spec/tools/loadcheck.py', path])
        refusals = [l for l in out.splitlines() if re.search(r'LOAD-[A-Z]+', l)]
        if rc == 0 and not refusals:
            continue
        fails += 1
        print('-- %s %s: exit %d' % (kind, name, rc))
        for l in refusals:
            code = re.search(r'LOAD-[A-Z]+', l).group(0)
            if kind == 'probe':
                ids = {'P:' + name} | {r for r, _, _ in rows if r.startswith('O:' + name + '-')}
                hits = naming(rows, code, ids=ids)
            else:
                hits = naming(rows, code, chunk=name)
            print('   %s\n     named by: %s' % (l.strip().replace(REPO + '/', ''),
                                              ', '.join(hits) if hits else 'NO ROW NAMES IT'))
        if not refusals:
            print(out)
    print('realities with refusals: %d of %d' % (fails, len(realities)))

    print('== check 4: tmp/cases/tools/check.py')
    rc, out = run(['python3', '-I', '-B', 'tmp/cases/tools/check.py'])
    print('\n'.join(out.splitlines()[-8:]))
    print('exit', rc)
    ok = ok and rc == 0

    print('== check 5: __pycache__')
    rc, out = run(['find', 'tmp/cases', 'engine', '-name', '__pycache__'])
    print(out.strip() or 'none')
    ok = ok and not out.strip()
    sys.exit(0 if ok else 1)


if __name__ == '__main__':
    main()
