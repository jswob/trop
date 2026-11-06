"""later/later.py: T's vectors for milestones not yet built, stored with refcore's bytes until their milestone's T
session wires them into its runner target (tally 2.1.6: #36 for M1, #49 for M7). Standard library only.

later.tsv lists them, one row each: id (its path under later/, no .tally), milestone, issue, rule. Each is one text,
run as `run <id>.tally` from later/ on the frozen refcore (PyPy through run.py's run_one: its 90 s budget, its
cache, the watchdog); its output and exit code are stored beside it, <id>.refcore (the first line `exit <n>`, then
the output, verbatim). No gating target reads them: they gate from their milestone, when its T session moves them.

    python3 -I -B engine/tests/later/later.py            -> writes each <id>.refcore
    python3 -I -B engine/tests/later/later.py --check    -> exit 1 if any isn't what refcore prints
"""
import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402

LIST = os.path.join(HERE, 'later.tsv')
COLS = ['id', 'milestone', 'issue', 'rule']
CWD = 'engine/tests/later'


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLS:
            sys.exit('later.tsv: the columns are %s, not %s' % (r.fieldnames, COLS))
        out = list(r)
    if len({v['id'] for v in out}) != len(out):
        sys.exit('later.tsv: an id twice')
    for v in out:
        if v['milestone'] in ('S', 'M0') or not v['milestone'].startswith('M'):
            sys.exit('later.tsv: %s waits for %s, which isn\'t a later milestone' % (v['id'], v['milestone']))
    return out


def stored_bytes(v, rc_hash):
    c = {'id': 'later/' + v['id'], 'layer': 'unit', 'cwd': CWD, 'files': CWD + '/' + v['id'] + '.tally', 'asks': '',
         'options': '', 'baseline': '', 'block': '', 'note': '', 'baseline_s': ''}
    e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
    if e['over'] or e['rc'] is None or e['rc'] < 0 or 'REFCORE-NOT-YET' in e['out'] or 'Traceback' in e['out']:
        sys.exit('%s: over the budget, killed or not handled on refcore' % v['id'])
    return ('exit %d\n' % e['rc'] + e['out']).encode('utf-8')


def main(argv):
    R.need_watchdog()
    rc_hash = R.refcore_hash()
    bad = 0
    vs = rows()
    for v in vs:
        data = stored_bytes(v, rc_hash)
        path = os.path.join(HERE, v['id'] + '.refcore')
        if '--check' in argv:
            old = open(path, 'rb').read() if os.path.exists(path) else b''
            if old != data:
                bad += 1
                print('FAIL %s: %s.refcore isn\'t what refcore prints: run later/later.py' % (v['id'], v['id']))
        else:
            with open(path, 'wb') as f:
                f.write(data)
    print('later: %d vectors%s' % (len(vs), ', %d differ' % bad if bad else (', refcore\'s bytes' if '--check' in argv
                                                                               else ' written')))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
