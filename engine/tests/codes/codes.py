"""codes.py: T's fixtures for the catalogue's codes that no check of checks.tsv prints (DONE gate 5; BUILD-ORDER
§ M0 "Each LOAD-* code has its fixture", § What T still writes). Standard library only. First: LOAD-PATH (tally 2.1.1,
L §8.3: `<path> doesn't exist`), which no check can print, since every file checks.tsv names exists (run.py hashes
each one for its cache key, and tools/tracked_check.py wants each tracked).

A fixture is a command line run from a folder under codes/, and refcore's output and exit code for it, stored in
fixtures.tsv. This script runs each on the frozen refcore through run.py's own execute (PyPy, plain refcore, its 90 s
budget, under the watchdog) and writes or checks the stored output. They are not checks.tsv rows: the build order's
placement counts its 634 checks, and a fixture lands in the milestone that builds its construct, LOAD-* at M0. The
engine is held to them by tally-runner's M0 target `m0_codes` (in process, INTERFACE.md section 4: a path the file
table doesn't hold is a path that doesn't exist); tally-runner's own tests read them.

    python3 -I -B engine/tests/codes/codes.py            run each fixture on refcore; exit 1 if fixtures.tsv differs
    python3 -I -B engine/tests/codes/codes.py --write    run each and write fixtures.tsv

fixtures.tsv's columns: `id`; `code` (the code its output prints); `cwd` (relative to the repository root); `args`
(the words after the program name, space-separated; a path is relative to `cwd`); `rc` (refcore's exit code); `out`
(refcore's output, stdout then stderr, `\\n` written as the two characters backslash and n); `rule`.
"""
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
TSV = os.path.join(HERE, 'fixtures.tsv')
COLS = ['id', 'code', 'cwd', 'args', 'rc', 'out', 'rule']
CWD = 'engine/tests/codes'

# id, code, the paths given (relative to CWD), rule
FIXTURES = [
    ('LOAD-PATH/alone', 'LOAD-PATH', ['LOAD-PATH/nope.tally'],
     'L §8.3 LOAD-PATH: a path the run is given names no file or folder; named as given'),
    ('LOAD-PATH/before-parse', 'LOAD-PATH', ['LOAD-PATH/m.tally', 'LOAD-PATH/a.tally', 'LOAD-PATH/z.tally'],
     'L §8.3 Which refusal prints: every file is read before any is split or parsed, so a missing path is refused '
     'before a.tally, first by full path, fails to parse'),
    ('LOAD-PATH/control', 'LOAD-PARSE', ['LOAD-PATH/m.tally', 'LOAD-PATH/a.tally'],
     'L §8.3 LOAD-PARSE: the control of before-parse: without the missing path, a.tally is refused'),
]


def runpy():
    spec = importlib.util.spec_from_file_location('run', os.path.join(ROOT, 'engine/tests/run.py'))
    run = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(run)
    return run


def rows():
    run = runpy()
    run.need_watchdog()
    run.refcore_hash()  # refcore matches engine/tools/SHA256SUMS, or stop
    out = []
    for fid, code, paths, rule in FIXTURES:
        c = {'id': fid, 'layer': 'unit', 'cwd': CWD, 'files': ' '.join(CWD + '/' + p for p in paths), 'asks': '',
             'options': ''}
        e = run.execute(c, 'pypy', run.BUDGET)
        if e['over'] or e['rc'] is None:
            raise SystemExit('%s: refcore didn\'t finish in %g s' % (fid, run.BUDGET))
        if '\\n' in e['out'] or '\t' in e['out']:
            raise SystemExit('%s: refcore\'s output holds a tab or a backslash-n' % fid)
        out.append({'id': fid, 'code': code, 'cwd': CWD, 'args': ' '.join(run.argv_of(c)), 'rc': str(e['rc']),
                    'out': e['out'].replace('\n', '\\n'), 'rule': rule})
    return out


def text(rs):
    return '\n'.join(['\t'.join(COLS)] + ['\t'.join(r[k] for k in COLS) for r in rs]) + '\n'


def main(argv):
    if argv not in ([], ['--write']):
        print(__doc__)
        return 64
    rs = rows()
    bad = 0
    for r in rs:
        printed = r['out'].split('\\n', 1)[0]
        ok = printed.startswith('refused: %s ' % r['code']) and r['rc'] == '1'
        bad += not ok
        print('%s  %s: rc %s: %s' % ('pass' if ok else 'FAIL', r['id'], r['rc'], printed))
    t = text(rs)
    if argv == ['--write'] and not bad:
        with open(TSV, 'w', encoding='utf-8', newline='') as f:
            f.write(t)
    elif not bad and (not os.path.exists(TSV) or open(TSV, encoding='utf-8', newline='').read() != t):
        print('FAIL fixtures.tsv differs from refcore\'s runs (--write)')
        bad += 1
    print('codes: %d fixtures: %d failed' % (len(rs), bad))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
