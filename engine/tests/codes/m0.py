"""codes/m0.py: M0's fixtures for the LOAD-* codes no output at S or M0 prints (DONE gate 5; BUILD-ORDER § M0 "Each
LOAD-* code has its fixture"; tools/xref_check.py's M0 list; issue #56, MILESTONE-M0.md rows 5 and 6). Standard
library only.

A family is a folder of codes/m0/ whose texts load clean: its .tally files, given on the command line in the order of
their names (a joint model's members, in a sub-folder its `use` lines name, are in the file table only). Its base
vector is the load probe, `run <files> --ask zz_m0_probe`, on which refcore prints `refused: REQ-ASK no ask
zz_m0_probe` alone, exit 1 (E §11, tally 2.1.3): the stated twin of every fixture of the family. A fixture is one edit
of one file of a family, a row of codes/m0/edits.tsv:

    id, code, family, file, old, new, asks, rule, form

`old` occurs once in the file and becomes `new` (`\\n` stands for a newline); `asks` is empty for the load probe, or
the asks the run asks where the code refuses an ask, not the reality; `form`, empty for G4's fixtures (one per code),
is one of the code's message forms in L §8.3 (tools/xref_check.py's), which the fixture's refusal must match (G7,
tier 2: a fixture for each form no other output prints). codes/m0/held.tsv holds, in the same columns and a last one
saying why, the tier-2 fixtures held out: refcore prints the code in words no form gives (issue #71's family), or
doesn't refuse where the spec does (a STOP-REPORT); once ruled, a row moves to edits.tsv. This script doesn't read it. A fixture that needs two edits has two rows of
one id, applied in order (its first row's code, asks and rule stand). The script writes each fixture's texts under
engine/tests/out/m0/codes/ (git-ignored), runs every vector on the frozen refcore through run.py's run_one (PyPy, its
90 s budget, its cache, the watchdog), and stops unless each base prints the probe's line alone (and a run of all
its asks, not stored, prints no LOAD-* or REQ-* refusal: a scenario's or an ask's refusal the probe can't see) and each fixture
prints its code (`refused: <code> …` at the start of a line or after a name and a colon) and no warning (the
warnings are M1's). It writes codes/m0.refcore, seven netstrings per vector, the bases first, by family, then the
fixtures in edits.tsv's order:

    id, code (REQ-ASK for a base), family, the argument words, the file table (INTERFACE.md §4, packed: each path as
    the run names it and its text), refcore's exit code, its whole output

The runner embeds it (runner/src/codes.rs) and milestone/m0_codes.rs holds the engine to each vector, byte for byte:
one test per code, running its fixtures and their families' bases.

    python3 -I -B engine/tests/codes/m0.py              -> writes codes/m0.refcore
    python3 -I -B engine/tests/codes/m0.py --check      -> exit 1 if codes/m0.refcore isn't what refcore's runs give
    python3 -I -B engine/tests/codes/m0.py --show GLOB  -> refcore's run of the vectors whose id matches; writes nothing
"""
import csv
import fnmatch
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402

sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'm0'))
import vectors as V  # noqa: E402

sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'tools'))
import xref_check as X  # noqa: E402

COLS = ['id', 'code', 'family', 'file', 'old', 'new', 'asks', 'rule', 'form']
JOBS = 2


def configure(name):
    """Points the script at the store `name` (m0, or m1 for codes/m1.py): its families, edits and store."""
    global NAME, FAMILIES, EDITS, STORE, REL, BUILT
    NAME = name
    FAMILIES = os.path.join(HERE, name)
    EDITS = os.path.join(FAMILIES, 'edits.tsv')
    STORE = os.path.join(HERE, name + '.refcore')
    REL = 'engine/tests/codes/%s/' % name
    BUILT = 'engine/tests/out/%s/codes/' % name   # run output: git ignores out/, check-frozen skips it


configure('m0')


def is_twin(vid):
    """Whether a vector is a family's twin, not a fixture: its base (codes/m1.py adds its answers)."""
    return vid.endswith('/base')


def family(name):
    """(the argument files, [(path as the run names it, its text)]) of a family, its members included."""
    cwd = REL + name
    args = sorted(f for f in os.listdir(R.ROOT + cwd) if f.endswith('.tally'))
    table = [(a, cwd + '/' + a) for a in args] + V.members(cwd, args)
    return args, [(key, open(R.ROOT + repo, encoding='utf-8').read()) for key, repo in table]


def unescape(s):
    return s.replace('\\n', '\n')


def vectors():
    """[(id, code, family, args, [(key, text)], asks, form)]: each family's base, then each fixture."""
    with open(EDITS, encoding='utf-8', newline='') as f:
        rows = list(csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE))
    if not rows or list(rows[0]) != COLS:
        sys.exit('codes/%s/edits.tsv\'s columns are ' % NAME + ', '.join(COLS))
    names = sorted({r['family'] for r in rows})
    out = []
    for name in names:
        args, table = family(name)
        out.append((name + '/base', 'REQ-ASK', name, args, table, V.PROBE, ''))
    fixtures = {}
    for r in rows:   # a fixture's rows share its id, its edits applied in order; the first row names the rest
        fixtures.setdefault(r['id'], []).append(r)
    for fid, edits in fixtures.items():
        first = edits[0]
        args, table = family(first['family'])
        for r in edits:
            if r['family'] != first['family'] or r['code'] != first['code']:
                sys.exit('%s: its rows name two families or codes' % fid)
            text = dict(table).get(r['file'])
            if text is None:
                sys.exit('%s: %s is no file of family %s' % (fid, r['file'], r['family']))
            old, new = unescape(r['old']), unescape(r['new'])
            if text.count(old) != 1:
                sys.exit('%s: %r occurs %d times in %s/%s, not once' % (fid, old, text.count(old), r['family'],
                                                                         r['file']))
            table = [(k, t.replace(old, new) if k == r['file'] else t) for k, t in table]
        out.append((fid, first['code'], first['family'], args, table, first['asks'] or V.PROBE, first['form']))
    ids = [v[0] for v in out]
    if len(set(ids)) != len(ids):
        sys.exit('two vectors share an id')
    return out


def check_of(v):
    """Writes v's texts under BUILT and gives the run.py check that runs them (every file hashed for the cache)."""
    vid, _, _, args, table, asks, _ = v
    cwd = BUILT + vid.replace('/', '~')
    for key, text in table:
        dst = R.ROOT + cwd + '/' + key
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        data = text.encode('utf-8')
        if not os.path.exists(dst) or open(dst, 'rb').read() != data:
            with open(dst, 'wb') as f:
                f.write(data)
    return {'id': vid, 'layer': 'interpreter', 'cwd': cwd, 'asks': '', 'baseline': '', 'block': '', 'note': '',
            'baseline_s': '', 'files': ' '.join(cwd + '/' + k for k, _ in table),
            'options': ' '.join(['run'] + args + ['--ask'] + asks.split())}


def runs(vs):
    rc_hash = R.refcore_hash()

    def one(v):
        e = R.run_one(check_of(v), 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed on refcore' % v[0])
        return e
    with ThreadPoolExecutor(JOBS) as ex:
        return list(ex.map(one, vs))


def unseen_refusals(vs):
    """The bases that a run of every ask shows refused somewhere the probe can't see (a scenario, an ask)."""
    bases = [(v[0] + '~all', v[1], v[2], v[3], v[4], '', '') for v in vs if v[0].endswith('/base')]
    rc_hash = R.refcore_hash()
    bad = []
    for v in bases:
        c = dict(check_of(v), options=' '.join(['run'] + v[3]))
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or re.search(r'refused: (LOAD|REQ)-', e['out']):
            bad.append('%s: a run of every ask is refused: %r' % (v[0], e['out'][:300]))
    return bad


def wrong_form(v, out, cat):
    """Why the fixture's refusal doesn't print its form (None if it does, or if it names none)."""
    code, form = v[1], v[6]
    if not form:
        return None
    if form not in cat.get(code, ('', []))[1]:
        return '%s: %r is no message form of %s in the catalogue' % (v[0], form, code)
    messages = [m.group(2) for m in X.REFUSED.finditer(out) if m.group(1) == code]
    if not any(X.matches(form, msg) for msg in messages):
        return '%s: no %s refusal prints the form %r: %r' % (v[0], code, form, out[:300])
    return None


def failures(vs, res):
    bad = []
    cat = X.catalogue()
    for v, e in zip(vs, res):
        vid, code, out = v[0], v[1], e['out']
        if 'REFCORE-NOT-YET' in out or 'Traceback' in out:
            bad.append('%s: refcore prints REFCORE-NOT-YET or crashes' % vid)
        elif re.search(r'^warning: ', out, re.M):
            bad.append('%s: prints a warning (M0\'s tests leave warnings to M1): %r' % (vid, out[:300]))
        elif vid.endswith('/base') and (out != V.PROBED or e['rc'] != 1):
            bad.append('%s: the base doesn\'t load clean: exit %s, %r' % (vid, e['rc'], out[:300]))
        elif not vid.endswith('/base') and not re.search(r'(^|: )refused: %s ' % re.escape(code), out, re.M):
            bad.append('%s: prints no %s: exit %s, %r' % (vid, code, e['rc'], out[:300]))
        elif wrong_form(v, out, cat):
            bad.append(wrong_form(v, out, cat))
    return bad


def pack(table):
    return ''.join('%d:%s,%d:%s,' % (len(k.encode('utf-8')), k, len(t.encode('utf-8')), t) for k, t in table)


def stored(vs, res):
    return b''.join(V.ns(v[0]) + V.ns(v[1]) + V.ns(v[2]) + V.ns(' '.join(['run'] + v[3] + ['--ask'] + v[5].split())) +
                    V.ns(pack(v[4])) + V.ns(str(e['rc'])) + V.ns(e['out']) for v, e in zip(vs, res))


def main(argv):
    R.need_watchdog()
    vs = vectors()
    if '--show' in argv:
        glob = argv[argv.index('--show') + 1]
        vs = [v for v in vs if fnmatch.fnmatchcase(v[0], glob)]
        cat = X.catalogue()
        for v, e in zip(vs, runs(vs)):
            print('=== %s (%s, exit %s)%s\n%s' % (v[0], v[1], e['rc'], ' FORM? ' + v[6] if wrong_form(v, e['out'], cat)
                                                else ' ok' if v[6] else '', e['out']), end='')
        return 0
    res = runs(vs)
    bad = failures(vs, res) + unseen_refusals(vs)
    for b in bad:
        print('FAIL ' + b)
    if bad:
        print('codes/%s: %d of %d vectors fail on refcore: fix the fixture, or a STOP-REPORT' % (NAME, len(bad), len(vs)))
        return 1
    data = stored(vs, res)
    codes = sorted({v[1] for v in vs if not is_twin(v[0])})
    if '--check' in argv:
        old = open(STORE, 'rb').read() if os.path.exists(STORE) else b''
        if old != data:
            print('codes/%s.refcore is not what refcore\'s runs give: run codes/%s.py' % (NAME, NAME))
            return 1
        print('codes/%s.refcore: %d vectors (%d codes), refcore\'s bytes' % (NAME, len(vs), len(codes)))
        return 0
    with open(STORE, 'wb') as f:
        f.write(data)
    print('codes/%s.refcore: %d vectors written: %d twins, %d fixtures of %d codes' % (
        NAME, len(vs), sum(is_twin(v[0]) for v in vs), sum(not is_twin(v[0]) for v in vs), len(codes)))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
