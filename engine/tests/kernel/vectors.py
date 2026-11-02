"""Milestone S's kernel vectors: refcore's whole output for each (issue #41; MILESTONE-S.md row 5). Standard library only.

Reads vectors.tsv, one vector per row:

    id  group  check  cwd  files  asks  options

A vector is either a check of checks.tsv (`check` names its id, the other columns empty: its cwd, files, asks and
options are the check's) or an invented fixture of its own (`check` empty; `cwd` and `files` are repository-relative,
`files` and `asks` space-separated). Each is run on the frozen refcore (PyPy, through run.py's own run_one: its 90 s
budget, its cache, the watchdog it needs), and its command line, its file table (INTERFACE.md §4) and refcore's
output and exit code are written to vectors.refcore, six netstrings per vector:

    id, group, the argument words joined by single spaces, the file table, the exit code, the output

The runner embeds vectors.refcore (engine/tests/runner/src/kernel.rs, include_bytes!) and milestone/s_kernel.rs
holds the engine's whole output to it, byte for byte. Every expected byte is refcore's; nothing is written by hand.

    python3 -I -B engine/tests/kernel/vectors.py              -> writes vectors.refcore
    python3 -I -B engine/tests/kernel/vectors.py --check      -> exit 1 if vectors.refcore isn't what refcore prints
    python3 -I -B engine/tests/kernel/vectors.py --engine     -> the release binary's whole output against refcore's,
                                                                 one line per vector that differs (a look before the
                                                                 Rust test; never a baseline)
"""
import csv
import os
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402

LIST = os.path.join(HERE, 'vectors.tsv')
STORED = os.path.join(HERE, 'vectors.refcore')
COLUMNS = ['id', 'group', 'check', 'cwd', 'files', 'asks', 'options']
JOBS = 3


def ns(b):
    b = b if isinstance(b, bytes) else b.encode('utf-8')
    return b'%d:' % len(b) + b + b','


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLUMNS:
            sys.exit('vectors.tsv: the columns are %s, not %s' % (r.fieldnames, COLUMNS))
        return list(r)


def checks_of(vs):
    """Each vector as a check run.py can run (its own row, or checks.tsv's)."""
    by = {c['id']: c for c in R.load_checks()}
    out, seen = [], set()
    for v in vs:
        if v['id'] in seen:
            sys.exit('vectors.tsv: %s twice' % v['id'])
        seen.add(v['id'])
        if not v['group']:
            sys.exit('vectors.tsv: %s has no group' % v['id'])
        if v['check']:
            if v['cwd'] or v['files'] or v['asks'] or v['options']:
                sys.exit('vectors.tsv: %s names a check and its own files' % v['id'])
            c = by.get(v['check'])
            if c is None:
                sys.exit('vectors.tsv: %s names %s, which checks.tsv lacks' % (v['id'], v['check']))
            if c['layer'] == 'interpreter' or R.script(c) or c['note'].startswith('not-run:'):
                sys.exit('vectors.tsv: %s names %s, which runs no reality' % (v['id'], v['check']))
            c = dict(c, id=v['id'])
        else:
            if not v['cwd'] or not v['files']:
                sys.exit('vectors.tsv: %s has no cwd or no files' % v['id'])
            c = {'id': v['id'], 'layer': 'unit', 'cwd': v['cwd'], 'files': v['files'], 'asks': v['asks'],
                 'options': v['options'], 'baseline': '', 'block': '', 'note': '', 'baseline_s': ''}
        out.append((v, c))
    return out


def table(c):
    """The file table of check `c`: each file as its command line names it (relative to its cwd), then its text."""
    parts = []
    for f in c['files'].split():
        with open(R.ROOT + f, encoding='utf-8', newline='') as fh:
            parts.append(ns(R.here(c, f)) + ns(fh.read()))
    return b''.join(parts)


def refcore_runs(pairs):
    rc_hash = R.refcore_hash()
    with ThreadPoolExecutor(JOBS) as ex:
        res = list(ex.map(lambda p: R.run_one(p[1], 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache'), pairs))
    for (v, _), e in zip(pairs, res):
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed on refcore; a vector must be answered' % v['id'])
        if 'REFCORE-NOT-YET' in e['out']:
            sys.exit('%s: refcore prints REFCORE-NOT-YET, a gap in refcore, not a target (INTERFACE.md §3)' % v['id'])
    return res


def stored_bytes(pairs, res):
    return b''.join(ns(v['id']) + ns(v['group']) + ns(' '.join(R.argv_of(c))) + ns(table(c)) + ns(str(e['rc'])) +
                    ns(e['out']) for (v, c), e in zip(pairs, res))


def engine_look(pairs, res):
    """The release binary on each vector, its whole output against refcore's (never stored)."""
    with ThreadPoolExecutor(JOBS) as ex:
        eng = list(ex.map(lambda p: R.execute(p[1], 'engine', R.BUDGET), pairs))
    bad = 0
    for (v, _), e, g in zip(pairs, res, eng):
        if g['out'] != e['out'] or g['rc'] != e['rc']:
            bad += 1
            print('DIFFER %s: exit %s, refcore %s; %s' % (v['id'], g['rc'], e['rc'], R.first_diff(e['out'], g['out'])))
    print('%d of %d vectors differ on the engine' % (bad, len(pairs)))
    return 0 if bad == 0 else 1


def main(argv):
    R.need_watchdog()
    pairs = checks_of(rows())
    res = refcore_runs(pairs)
    if '--engine' in argv:
        return engine_look(pairs, res)
    data = stored_bytes(pairs, res)
    if '--check' in argv:
        old = open(STORED, 'rb').read() if os.path.exists(STORED) else b''
        if old != data:
            print('vectors.refcore is not what refcore prints for vectors.tsv: run vectors.py')
            return 1
        print('vectors.refcore: %d vectors, refcore\'s bytes' % len(pairs))
        return 0
    with open(STORED, 'wb') as f:
        f.write(data)
    print('vectors.refcore: %d vectors written' % len(pairs))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
