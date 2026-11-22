"""P8-L2's and P8-L10's run lists for the rules step's `load:` recipe (issue #252; pick P5; MILESTONE-M4.md row 4).
Standard library only.

Writes runs.tsv, one cold run of the engine per row, a row of PERFORMANCE §8 judged by its slowest run:

    row  run  cwd  files  asks  sha256

`cwd` is repository-relative; `files` are the run's paths as its command line names them from `cwd` (a directory
reads its .tally files); `asks` its `--ask` ids. `sha256` is the digest of the expected output (stdout then stderr),
from refcore alone, or `-` where no refcore run judges the output (timed only):

- **P8-L2** (load.tsv L:2): D-11's six asks M4 runs on household D's full text (`integration/households/D`, its law
  parts in full, `end 2064-12-31`). Its cut `integration/cuts/D/cut11` differs only in `reality.tally`'s `end`, and
  no D-11 block reads past the cut, so the expected output is refcore's: each ask's blocks from cut11's stored
  baseline (refcore's run of the cut) in the asks' order, then the warnings refcore's `check` prints for the full
  text (a run-end cut's judged twin, TESTING.md §4). `d11_not_follows_fund` is held to M7 (held.tsv).
- **P8-L10** (load.tsv L:10): change 1's walk run `cut11/c1_r2` (the six asks of its checks), then each stage of
  m3/changes/stages.tsv with a run (b1 .. c7, 17), asked the walk asks of L:10 its files hold. None judged: refcore
  stops at 8 GB on c1_r2 (snap15), and benchmarks.md row 10 leaves the long-horizon cells unjudged until the engine.
  `v5_r2` isn't listed: change 5's own lines all start 2029-05-01, after c1_r2's horizon (2027-01-14), so a run of
  change 5 at that horizon is change 1's; the gate reports it not run (P5).

    python3 -I -B engine/tests/m4/load/runs.py           -> writes runs.tsv (runs refcore's `check` of D's full text)
    python3 -I -B engine/tests/m4/load/runs.py --check   -> exit 1 if runs.tsv isn't what it writes
"""
import csv
import hashlib
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, TESTS)
import run as R  # noqa: E402

OUT = os.path.join(HERE, 'runs.tsv')
D_FULL = 'engine/tests/integration/households/D'
D11 = ['d11_follows', 'd11_follows_bonds', 'd11_isb', 'd11_pairs', 'd11_register', 'd11_not_follows_life']
C1_R2 = ['I:X:explain_room', 'I:X:ike_left', 'I:X:konto_balance@c1_r2', 'I:X:net_worth', 'I:X:replay_september@c1_r2',
         'I:X:taxes_paid']
REAL11 = 'engine/tests/integration/amendments/real11'
WALK = ['konto_balance', 'net_worth', 'ike_left', 'taxes_paid', 'pit28', 'health_settlement', 'explain_room',
        'replay_september', 'ola_800_paid']


def l2(checks):
    """P8-L2's run: cut11's file list, each file named where the full text holds it, and its expected output."""
    by = {c['block']: c for c in checks if c['id'].startswith('I:D:d11_')}
    files = []
    for f in by[D11[0]]['files'].split():
        name = os.path.basename(f)
        files.append(name if os.path.isfile(R.ROOT + D_FULL + '/' + name) else 'law/' + name)
    lines = []
    for a in D11:
        for k, v in R.blocks(open(R.ROOT + by[a]['baseline'], encoding='utf-8').read()).items():
            if R.owner(k) == a:
                lines += v
    p = subprocess.run([sys.executable, '-I', '-B', R.FAST, 'check'] + files, cwd=R.ROOT + D_FULL,
                       capture_output=True, text=True)
    out = p.stdout.splitlines()
    if p.returncode != 0 or not out or out[0] != 'ok' or p.stderr:
        sys.exit('refcore check of D\'s full text: exit %d, %r' % (p.returncode, (p.stdout + p.stderr)[:200]))
    digest = hashlib.sha256(('\n'.join(lines + out[1:]) + '\n').encode('utf-8')).hexdigest()
    return ['P8-L2', 'd11_full', D_FULL, ' '.join(files), ' '.join(D11), digest]


def l10(checks):
    """P8-L10's runs: c1_r2 as its checks run it, then each stage with a run, with the walk asks its files hold."""
    by = {c['id']: c for c in checks}
    c = by[C1_R2[0]]
    out = [['P8-L10', 'c1_r2', c['cwd'], ' '.join(R.here(c, f) for f in c['files'].split()),
            ' '.join(by[i]['asks'] for i in C1_R2), '-']]
    with open(os.path.join(TESTS, 'm3', 'changes', 'stages.tsv'), encoding='utf-8', newline='') as f:
        stages = [s for s in csv.DictReader(f, delimiter='\t') if s['run'] != '-']
    for s in stages:
        held = set()
        for w in s['files'].split():
            p = os.path.normpath(os.path.join(R.ROOT + REAL11, w))
            paths = [os.path.join(d, x) for d, _, xs in os.walk(p) for x in xs if x.endswith('.tally')] \
                if os.path.isdir(p) else [p]
            for q in paths:
                held |= set(re.findall(r'^ask ([a-z_0-9]+):', open(q, encoding='utf-8').read(), re.M))
        out.append(['P8-L10', s['stage'], REAL11, s['files'], ' '.join(a for a in WALK if a in held), '-'])
    return out


def main(argv):
    checks = R.load_checks()
    rows = [l2(checks)] + l10(checks)
    data = '\t'.join(['row', 'run', 'cwd', 'files', 'asks', 'sha256']) + '\n' + ''.join(
        '\t'.join(r) + '\n' for r in rows)
    if '--check' in argv:
        old = open(OUT, encoding='utf-8').read() if os.path.exists(OUT) else ''
        if old != data:
            print('m4/load/runs.tsv is not what runs.py writes: run m4/load/runs.py')
            return 1
        print('m4/load/runs.tsv: %d runs, P8-L2 judged by refcore' % len(rows))
        return 0
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write(data)
    print('m4/load/runs.tsv: %d runs written' % len(rows))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
