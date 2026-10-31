"""The move table for session 4.5f (old path -> new path) and its checks. Standard library only.

    python3 -I -B tmp/session4_5/f/tools/layout.py [--md OUT.md] [--tsv OUT.tsv]

Every tracked file under the suite's old homes gets a new path by the longest matching prefix of MOVES; files under
STAY are kept where they are. Checks: no two files land on one path; every `use "..."` line still resolves to the
moved target; every file a check reads (tracked_check.py's set) and every file build_checks.py reads is moved.
"""
import collections
import os
import re
import subprocess
import sys

ROOT = subprocess.run(['git', 'rev-parse', '--show-toplevel'], capture_output=True, text=True, check=True,
                      cwd=os.path.dirname(os.path.abspath(__file__))).stdout.strip() + '/'
N = 'engine/tests/'
S = 'tmp/design2/scratch/'
# (old prefix, new prefix); a prefix ending in '/' is a directory, else one file
MOVES = [
    # the runner, the lists, the docs
    ('tmp/tests/run.py', N + 'run.py'),
    ('tmp/tests/checks.tsv', N + 'checks.tsv'),
    ('tmp/tests/README.md', N + 'README.md'),
    ('tmp/tests/TESTING.md', N + 'TESTING.md'),
    ('tmp/tests/findings.md', N + 'findings.md'),
    ('tmp/tests/known-faults.md', N + 'known-faults.md'),
    ('tmp/tests/build_checks.py', N + 'tools/build_checks.py'),
    ('tmp/tests/tracked_check.py', N + 'tools/tracked_check.py'),
    ('tmp/tests/knobdiff.py', N + 'tools/knobdiff.py'),
    ('tmp/session5/tools/memcap.py', N + 'tools/memcap.py'),
    ('tmp/session5/tools/answers.py', N + 'tools/answers.py'),
    ('tmp/tests/map/', N + 'map/'),
    ('tmp/tests/after.md', N + 'reports/after.md'),
    ('tmp/tests/after-fast.md', N + 'reports/after-fast.md'),
    ('tmp/tests/before-after.md', N + 'reports/before-after.md'),
    ('tmp/tests/before-after.4.5a.md', N + 'reports/before-after.4.5a.md'),
    ('tmp/tests/over-budget.md', N + 'reports/over-budget.md'),
    ('tmp/tests/over-budget.4.5a.md', N + 'reports/over-budget.4.5a.md'),
    ('tmp/tests/adhoc.log', N + 'reports/adhoc.log'),
    # the case set (the keys, the cards, the stories, compare.py and the key scripts)
    ('tmp/cases/', N + 'cases/'),
    # unit: the probe texts
    ('tmp/design2/cases/probes/', N + 'unit/probes/'),
    # integration
    ('tmp/design2/cases/households/', N + 'integration/households/'),
    ('tmp/design2/cases/law/', N + 'integration/law/'),
    ('tmp/design2/models/', N + 'integration/models/'),
    ('tmp/design/models/', N + 'integration/models-1.0/'),
    ('tmp/design2/regress/light/', N + 'integration/light/'),
    ('tmp/design2/hardset/', N + 'integration/blind/'),
    (S + 'lead/check/b2fix/', N + 'integration/blind/b2fix/'),
    (S + 'mc-a/', N + 'integration/cuts/A/'),
    (S + 'mc-b/', N + 'integration/cuts/B/'),
    (S + 'mc-d/', N + 'integration/cuts/D/'),
    (S + 'w-e1/', N + 'integration/cuts/E1/'),
    (S + 'tx2/', N + 'integration/cuts/E1-02-B/'),
    (S + 'w-b/', N + 'integration/cuts/B-law/'),
    (S + 'mc-x/', N + 'integration/amendments/'),
    ('tmp/tests/texts/new/', N + 'integration/new/'),
    ('tmp/tests/texts/split/', N + 'integration/split/'),
    ('tmp/tests/texts/recipes/', N + 'integration/recipes/'),
    ('tmp/tests/keys-extra/', N + 'integration/keys-extra/'),
    # load
    ('tmp/tests/load.tsv', N + 'load/load.tsv'),
    ('tmp/design2/benchmarks.md', N + 'load/benchmarks.md'),
    # baselines and the run definitions build_checks.py reads
    (S + 'lead/check/e1-02-B16.out', N + 'baselines/e1-02-B16.out'),
    (S + 'lead/check/snap16-', N + 'baselines/snap16-'),
    (S + 'lead/check/', N + 'defs/'),
    ('tmp/tests/baselines/', N + 'baselines/'),
    ('tmp/session5/runs/visible/', N + 'baselines/session5-visible/'),
]
STAY = [('tmp/cases/tally-1.0.1/', 'session 3\'s case set 1.0.1: history, read by no check; tracked as it is'),
        ('engine/', 'the engine: spec, refcore, refcore_fast stay'),
        ('tmp/session4_5/', 'session records'),
        ('tmp/active/', 'briefs and the research note, cited by path')]


def new(p):
    best = None
    for o, n in MOVES:
        if (p == o or (o.endswith(('/', '-')) and p.startswith(o))) and (best is None or len(o) > len(best[0])):
            best = (o, n)
    if best is None:
        return None
    o, n = best
    return n + p[len(o):] if o.endswith(('/', '-')) else n


def main(argv):
    tracked = [p for p in subprocess.run(['git', '-C', ROOT, 'ls-files', 'tmp', 'engine'], capture_output=True,
                                         text=True, check=True).stdout.split('\n') if p]
    table, stay, unknown = [], collections.Counter(), []
    for p in tracked:
        if any(p.startswith(s) for s, _ in STAY):
            stay[next(s for s, _ in STAY if p.startswith(s))] += 1
            continue
        n = new(p)
        if n is None:
            unknown.append(p)
        else:
            table.append((p, n))
    bad = []
    targets = collections.Counter(n for _, n in table)
    bad += ['two files land on %s' % n for n, k in targets.items() if k > 1]
    bad += ['%s already exists' % n for _, n in table if os.path.exists(ROOT + n)]
    bad += ['no rule for %s' % p for p in unknown]
    # every use line still resolves after the move
    USE = re.compile(r'^\s*use\s+\w+\s*=\s*"([^"]+)"', re.M)
    newof = dict(table)
    nuse = 0
    for p, n in table:
        if not p.endswith('.tally'):
            continue
        for t in USE.findall(open(ROOT + p, encoding='utf-8', errors='replace').read()):
            nuse += 1
            old_t = os.path.normpath(os.path.join(os.path.dirname(p), t))
            new_t = os.path.normpath(os.path.join(os.path.dirname(n), t))
            if os.path.isdir(ROOT + old_t):
                ok = all(new(os.path.relpath(os.path.join(d, f), ROOT)) ==
                         os.path.normpath(os.path.join(new_t, os.path.relpath(os.path.join(d, f), ROOT + old_t)))
                         for d, _, fs in os.walk(ROOT + old_t) for f in fs if '__pycache__' not in d and f != '.DS_Store')
            else:
                ok = newof.get(old_t) == new_t
            if not ok:
                bad.append('use "%s" in %s: %s no longer at %s' % (t, p, old_t, new_t))
    print('%d tracked files under tmp/ and engine/: %d move, %d stay (%s); %d use lines checked'
          % (len(tracked), len(table), sum(stay.values()), ', '.join('%s %d' % kv for kv in stay.items()), nuse))
    for b in bad[:40]:
        print('BAD', b)
    if '--tsv' in argv:
        with open(argv[argv.index('--tsv') + 1], 'w', encoding='utf-8') as f:
            f.write('old\tnew\n' + ''.join('%s\t%s\n' % r for r in table))
    if '--md' in argv:
        by = collections.Counter()
        for p, n in table:
            o = max((o for o, _ in MOVES if new(p) and (p == o or (o.endswith(('/', '-')) and p.startswith(o)))), key=len)
            by[(o, dict(MOVES)[o])] += 1
        L = ['| old | new | files |', '|---|---|---|']
        L += ['| `%s` | `%s` | %d |' % (o, n, k) for (o, n), k in sorted(by.items(), key=lambda x: x[0][1])]
        open(argv[argv.index('--md') + 1], 'w', encoding='utf-8').write('\n'.join(L) + '\n')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
