"""codes/m1.py: M1's fixtures for the codes and message forms of M1's constructs that no output at S, M0 or M1 prints
(DONE gate 5; BUILD-ORDER § M1; tools/xref_check.py --gate M1; issue #108, MILESTONE-M1.md row 3). Standard library
only.

codes/m0.py with the store m1 (its configure): the families are the folders of codes/m1/, the fixtures the rows of
codes/m1/edits.tsv in m0's columns, the store codes/m1.refcore in m0's seven netstrings, run on the frozen refcore
through run.py's run_one. What M1 adds:

- **A family's answers.** Its codes are refused per value, so a base that loads clean isn't twin enough: beside its
  load probe, each family of codes/m1/ stores `<family>/answers/<ask>` for every ask its texts declare (in the order
  they are declared), a run of that ask alone, which must answer with exit 0 and print no refusal, no hole and no
  warning. A fixture is the run of its asks with one edit, refused or a hole where its code says, and its stated
  twins are its family's base and the answers of its asks, so a code's test needs only the constructs its asks read.
- **A hole's code.** A hole prints its question with no code (L §8.1), so a HOLE-* fixture must print a question its
  code's form matches (tools/xref_check.py's matcher) in place of a `refused: <code>` line.
- **#90's two fixtures** (the coordinator's pick P20: a bound name's field, ruled for M1): a family named `m0:<name>`
  is codes/m0/<name>, so `adjust/bound-field` and `gifts/lifespan-zz` run on M0's own families, held to refcore's
  bytes here until M1 gates, then joining m0_codes.

codes/m1/held.tsv holds, in m0's held columns, the fixtures refcore can't print (no catalogue form, or no refusal where
the spec has one): each a STOP-REPORT in GAPS.md, never a vector of other words. This script doesn't read it.

    python3 -I -B engine/tests/codes/m1.py              -> writes codes/m1.refcore
    python3 -I -B engine/tests/codes/m1.py --check      -> exit 1 if codes/m1.refcore isn't what refcore's runs give
    python3 -I -B engine/tests/codes/m1.py --show GLOB  -> refcore's run of the vectors whose id matches; writes nothing
"""
import importlib.util
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location('codes_m0', os.path.join(HERE, 'm0.py'))
M = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(M)
M.configure('m1')

M0_REL = 'engine/tests/codes/m0/'
ASK = re.compile(r'^ask ([a-z_][a-z0-9_]*)\b', re.M)
_family, _vectors = M.family, M.vectors


def family(name):
    """m0's family, or for `m0:<name>` M0's own family codes/m0/<name> (#90's fixtures)."""
    if not name.startswith('m0:'):
        return _family(name)
    rel = M.REL
    try:
        M.REL = M0_REL
        return _family(name[3:])
    finally:
        M.REL = rel


def is_twin(vid):
    return vid.endswith('/base') or '/answers/' in vid


def vectors():
    """m0's vectors with each M1 family's answers after the bases: each ask its texts declare, alone."""
    out = _vectors()
    bases = [v for v in out if v[0].endswith('/base')]
    answers = []
    for v in bases:
        if v[2].startswith('m0:'):
            continue
        asks = [a for _, text in v[4] for a in ASK.findall(text)]
        if not asks:
            sys.exit('%s: the family declares no ask' % v[2])
        answers += [(v[2] + '/answers/' + a, '-', v[2], v[3], v[4], a, '') for a in asks]
    return bases + answers + [v for v in out if not v[0].endswith('/base')]


def hole_form(v, out, cat):
    """Why a HOLE-* fixture prints no question its code's form matches (the one it names, else any of the code's).
    HOLE-STATED's question is the writer's own text (L §8.1), so its fixture's must match no HOLE-* form."""
    code, form = v[1], v[6]
    questions = M.X.QUESTION.findall(out)
    if code == 'HOLE-STATED':
        fixed = [f for c, (_, fs) in cat.items() if c.startswith('HOLE-') for f in fs]
        if any(not any(M.X.matches(f, q) for f in fixed) for q in questions):
            return None
        return '%s: no question is a writer\'s own text: %r' % (v[0], out[:300])
    forms = [form] if form else cat.get(code, ('', []))[1]
    if any(M.X.matches(f, q) for f in forms for q in questions):
        return None
    return '%s: no question prints a form of %s: %r' % (v[0], code, out[:300])


def failures(vs, res):
    """m0's checks, with M1's: a family's answers are clean, a HOLE-* fixture prints its question, and a per-value
    refusal may print inside an ask line's tuple (`q: (refused: <code> …, 490.00 PLN)`)."""
    cat = M.X.catalogue()
    bad = []
    for v, e in zip(vs, res):
        vid, code, out = v[0], v[1], e['out']
        if 'REFCORE-NOT-YET' in out or 'Traceback' in out:
            bad.append('%s: refcore prints REFCORE-NOT-YET or crashes' % vid)
        elif re.search(r'^warning: ', out, re.M):
            bad.append('%s: prints a warning: %r' % (vid, out[:300]))
        elif vid.endswith('/base') and (out != M.V.PROBED or e['rc'] != 1):
            bad.append('%s: the base doesn\'t load clean: exit %s, %r' % (vid, e['rc'], out[:300]))
        elif '/answers/' in vid:
            if e['rc'] != 0 or re.search(r'refused: |\?"', out):
                bad.append('%s: the family doesn\'t answer clean: exit %s, %r' % (vid, e['rc'], out[:300]))
        elif code.startswith('HOLE-'):
            if e['rc'] != 0 or 'refused: ' in out:
                bad.append('%s: a hole fixture is refused: exit %s, %r' % (vid, e['rc'], out[:300]))
            elif hole_form(v, out, cat):
                bad.append(hole_form(v, out, cat))
        elif not re.search(r'(^|: |\(|, )refused: %s ' % re.escape(code), out, re.M):
            bad.append('%s: prints no %s: exit %s, %r' % (vid, code, e['rc'], out[:300]))
        elif M.wrong_form(v, out, cat):
            bad.append(M.wrong_form(v, out, cat))
    return bad


M.family, M.is_twin, M.vectors, M.failures = family, is_twin, vectors, failures

FAULT = os.path.join(HERE, 'm1_fault.py')
INVARIANT = os.path.join(HERE, 'm1_invariant.refcore')
FAULTED = [('decide/conservation', 'decide/answers/q_decide')]   # (id, answers): an answers vector, planted


def run_faulted(v):
    """refcore's run of v with ENGINE.md §9 invariant 1's planted fault (codes/m1_fault.py), as run.py runs refcore."""
    c = M.check_of(v)
    p = subprocess.run([M.R.PY['pypy'], '-I', '-B', FAULT] + c['options'].split(), cwd=M.R.ROOT + c['cwd'],
                       capture_output=True, timeout=M.R.BUDGET, start_new_session=True)
    return {'rc': p.returncode, 'out': M.R.text(p.stdout) + M.R.text(p.stderr)}


def invariant(argv):
    """codes/m1_invariant.refcore: each FAULTED answers vector, refused ENGINE-INVARIANT under the plant."""
    answers = {v[0]: v for v in vectors() if '/answers/' in v[0]}
    vs = [(vid, 'ENGINE-INVARIANT', answers[a][2]) + answers[a][3:] for vid, a in FAULTED]
    res = [run_faulted(v) for v in vs]
    bad = ['%s: exit %s, %r' % (v[0], e['rc'], e['out'][:300]) for v, e in zip(vs, res)
           if not re.search(r'(^|: |\(|, )refused: ENGINE-INVARIANT conservation: ', e['out'], re.M)]
    for b in bad:
        print('FAIL %s: the planted fault isn\'t refused ENGINE-INVARIANT' % b)
    if bad:
        return 1
    data = M.stored(vs, res)
    if '--check' in argv:
        old = open(INVARIANT, 'rb').read() if os.path.exists(INVARIANT) else b''
        if old != data:
            print('codes/m1_invariant.refcore is not what refcore\'s planted runs give: run codes/m1.py')
            return 1
        print('codes/m1_invariant.refcore: %d vectors, refcore\'s bytes under the plant' % len(vs))
        return 0
    with open(INVARIANT, 'wb') as f:
        f.write(data)
    print('codes/m1_invariant.refcore: %d vectors written' % len(vs))
    return 0


def main(argv):
    rc = M.main(argv)
    if rc or '--show' in argv:
        return rc
    return invariant(argv)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
