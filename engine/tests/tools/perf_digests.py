"""perf-digests.tsv: T's expected output digest for each P8 row that runs a check (T1, issue #5). Standard library only.

engine/tools/perf times a row's check on the engine and compares the SHA-256 of its output (stdout then stderr, the
raw bytes) with this file's digest, so a fast wrong answer fails (BUILD-ORDER § What T still writes, gate 12). The
digest is of refcore's output for the same command line: refcore's run from run.py's cache, or a run now through
run.py's run_one (plain refcore, its 90 s budget, the watchdog). Never an engine's output.

    python3 -I -B engine/tests/tools/perf_digests.py [--jobs N<=2]       -> writes engine/tests/perf-digests.tsv
    python3 -I -B engine/tests/tools/perf_digests.py --check             -> exit 1 if the file differs
"""
import hashlib
import os
import sys
import tomllib
from concurrent.futures import ThreadPoolExecutor

T = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + '/'
sys.path.insert(0, T)
import run as R  # noqa: E402

TABLE = R.ROOT + 'engine/tools/perf-budgets.toml'
OUT = T + 'perf-digests.tsv'


def rows():
    with open(TABLE, 'rb') as f:
        return [r for r in tomllib.load(f)['row'] if r['recipe'].startswith('check:')]


def text(jobs):
    by = {c['id']: c for c in R.load_checks()}
    rs = rows()
    R.need_watchdog()
    rc_hash = R.refcore_hash()
    cs = [by[r['recipe'][len('check:'):]] for r in rs]
    with ThreadPoolExecutor(jobs) as ex:
        res = list(ex.map(lambda c: R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache'), cs))
    lines = ['# T\'s expected output digests for the P8 rows that run a check (engine/tools/perf; tools/perf_digests.py).',
             '# id<TAB>sha256 of refcore\'s output (stdout then stderr) for the check\'s command line, from refcore\'s run.']
    for r, c, e in sorted(zip(rs, cs, res), key=lambda x: x[1]['id']):
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s (%s): refcore didn\'t finish it in %g s; no digest' % (c['id'], r['id'], R.BUDGET))
        lines.append('%s\t%s' % (c['id'], hashlib.sha256(e['out'].encode('utf-8', 'surrogateescape')).hexdigest()))
    return '\n'.join(lines) + '\n'


def main(argv):
    jobs = min(int(argv[argv.index('--jobs') + 1]) if '--jobs' in argv else 2, 2)
    t = text(jobs)
    if '--check' in argv:
        same = os.path.exists(OUT) and open(OUT, encoding='utf-8').read() == t
        print('perf-digests.tsv %s refcore\'s runs' % ('matches' if same else 'DIFFERS from'))
        return 0 if same else 1
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write(t)
    print('wrote perf-digests.tsv: %d rows' % (t.count('\n') - 2))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
