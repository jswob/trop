"""codes.py: T's fixtures for the catalogue's codes that no check of checks.tsv prints (DONE gate 5; BUILD-ORDER
§ M0 "Each LOAD-* code has its fixture", § What T still writes). Standard library only. First: LOAD-PATH (tally 2.1.1,
L §8.3: `<path> doesn't exist`), which no check can print, since every file checks.tsv names exists (run.py hashes
each one for its cache key, and tools/tracked_check.py wants each tracked). Then two rules of tally 2.1.2 that only
M0's load catalogue reaches (S's engine stops them as not built): the order of load's checks past parsing (E §3.1,
issue #23: `order/`), and LOAD-DEPTH's `ties by` term (L §2.3, issue #29: `LOAD-DEPTH/`; S's kernel vectors hold its
`from` term, `kernel/g13/`). Each refusal has its control beside it, a refusal of the other check.

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
    # the order of load's checks past parsing (tally 2.1.2, issue #23; E §3.1 The order of the checks)
    ('order/passes', 'LOAD-DUP', ['order/passes.tally'],
     'E §3.1 The order of the checks: the passes run in order, so the declarations pass (PLN declared twice, line 15) '
     'refuses before the items\' own checks reach the def at line 11 (rate is no id), whatever the lines'),
    ('order/passes-control', 'LOAD-KEY', ['order/passes_control.tally'],
     'E §3.1: the control of order/passes: with PLN declared once, the def at line 11 is refused'),
    ('order/items', 'LOAD-LITERAL', ['order/items.tally'],
     'E §3.1: the items\' own checks take one sort at a time, each by full path and line, so of two moves that each '
     'write a date in a rule, zz at line 20 prints, though aa comes first by id (tally 2.1.1 took moves by id: line 24)'),
    ('order/items-control', 'LOAD-LITERAL', ['order/items_control.tally'],
     'E §3.1: the control of order/items: with zz on an input day, aa at line 24 is refused'),
    # LOAD-DEPTH counts a `ties by` term (tally 2.1.2, issue #29; L §2.3); S's vectors hold the `from` term
    ('LOAD-DEPTH/ties-by', 'LOAD-DEPTH', ['LOAD-DEPTH/ties.tally'],
     'L §2.3 LOAD-DEPTH: each ties by term counts one level, so inside the last w[k] of 9 argmins chained by 8 ties '
     'by terms the depth is 8 + 2 brackets = 10, past 8; parsing comes before the declarations pass, so the second '
     'PLN at line 19 isn\'t reached (tally 2.1.1 counted no term and printed LOAD-DUP)'),
    ('LOAD-DEPTH/ties-by-control', 'LOAD-DUP', ['LOAD-DEPTH/ties_control.tally'],
     'L §2.3: the control of LOAD-DEPTH/ties-by: 7 argmins chained by 6 terms are 8 deep, the limit, so the text '
     'parses and the declarations pass refuses the second PLN'),
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
