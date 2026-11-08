"""Milestone M1's vectors: refcore's whole output for each (issue #108; MILESTONE-M1.md). Standard library only.

Reads vectors.tsv, one vector per row:

    id  group  check  cwd  files  asks  options  for

The first seven columns are kernel/vectors.py's, read and run by its own functions (imported, not copied): a vector
names a check of checks.tsv (`check`; the other columns empty) or is a fixture of its own (`cwd`, `files`, `asks`,
`options`; a `.gen` source expanded as there). `group` is the runner test that holds it (`checks-g1`, `carried-5`,
`fixes-g2`, ...: milestone/m1_checks.rs, m1_carried.rs, m1_goldens.rs and m1_fixes.rs, one test per group). `for` is
the builder's group that turns it green (G1 ... G14 of tmp/m1/build-plan.md §3), or `-` for a vector the engine
already answers; a `fixes-` vector's is the group whose review fix it holds, green from that fix; it is the builder's
work list and is not stored.

Each vector is run on the frozen refcore (PyPy, plain refcore, never the fast layer: run.py's run_one, its 90 s
budget, its cache, the watchdog) and written to vectors.refcore in kernel/vectors.refcore's six netstrings (id, group,
the argument words, the file table, the exit code, the output), which runner/src/m1.rs embeds. Before anything is
written, a vector that is a check of checks.tsv with a stored baseline is tied to it by run.py's own compare (a
household check by its ask's blocks, any other whole), so no expected byte is new; a check with no baseline
(U:time/TIME-08-before) has refcore's run as its expected output. Every expected byte is refcore's.

    python3 -I -B engine/tests/m1/vectors.py              -> writes vectors.refcore
    python3 -I -B engine/tests/m1/vectors.py --check      -> exit 1 if vectors.refcore isn't what refcore prints
    python3 -I -B engine/tests/m1/vectors.py --engine     -> the release binary's whole output against refcore's
    python3 -I -B engine/tests/m1/vectors.py --times      -> refcore's seconds per vector, slowest first
    python3 -I -B engine/tests/m1/vectors.py --show GLOB  -> refcore's output for the vectors whose id matches
"""
import csv
import fnmatch
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(HERE)
sys.path.insert(0, TESTS)
import run as R  # noqa: E402

_spec = importlib.util.spec_from_file_location('kernel_vectors', os.path.join(TESTS, 'kernel', 'vectors.py'))
KV = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(KV)
KV.JOBS = 2  # at most two heavy jobs on the machine (the run's rule)

LIST = os.path.join(HERE, 'vectors.tsv')
STORED = os.path.join(HERE, 'vectors.refcore')
COLUMNS = KV.COLUMNS + ['for']
FOR = {'G%d' % n for n in range(1, 15)} | {'-'}


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLUMNS:
            sys.exit('vectors.tsv: the columns are %s, not %s' % (r.fieldnames, COLUMNS))
        out = list(r)
    for v in out:
        if v['for'] not in FOR:
            sys.exit('vectors.tsv: %s is for %r, not a builder group G1..G14 or -' % (v['id'], v['for']))
    return out


def tie(pairs, res):
    """Each check of checks.tsv with a stored baseline: refcore's run of it compares `same` to it (run.py)."""
    checks = R.load_checks()
    for (v, c), e in zip(pairs, res):
        if v['check'] and c['baseline']:
            verdict, detail = R.compare(c, e['out'], checks)
            if verdict != 'same':
                sys.exit('%s: refcore no longer prints its stored baseline (%s)' % (v['id'], detail))


def main(argv):
    R.need_watchdog()
    pairs = KV.checks_of(rows())
    if '--show' in argv:
        i = argv.index('--show') + 1
        glob = argv[i] if i < len(argv) else '*'
        pairs = [p for p in pairs if fnmatch.fnmatchcase(p[0]['id'], glob)]
        for (v, _), e in zip(pairs, KV.refcore_runs(pairs)):
            print('=== %s (exit %s)\n%s' % (v['id'], e['rc'], e['out']), end='')
        return 0
    res = KV.refcore_runs(pairs)
    tie(pairs, res)
    if '--times' in argv:
        for (v, _), e in sorted(zip(pairs, res), key=lambda p: -p[1]['seconds']):
            print('%8.2f s  %s' % (e['seconds'], v['id']))
        return 0
    if '--engine' in argv:
        return KV.engine_look(pairs, res)
    data = KV.stored_bytes(pairs, res)
    if '--check' in argv:
        old = open(STORED, 'rb').read() if os.path.exists(STORED) else b''
        if old != data:
            print('m1/vectors.refcore is not what refcore prints for m1/vectors.tsv: run m1/vectors.py')
            return 1
        print('m1/vectors.refcore: %d vectors, refcore\'s bytes' % len(pairs))
        return 0
    with open(STORED, 'wb') as f:
        f.write(data)
    print('m1/vectors.refcore: %d vectors written' % len(pairs))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
