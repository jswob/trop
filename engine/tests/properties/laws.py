"""The kernel's laws, checked on refcore (T1, issue #5; PROPERTIES.md). Standard library only.

Runs every record of kernel.corpus on the frozen refcore through run.py's own run_one (its 90 s budget, its cache, the
watchdog it needs), then checks each law on refcore's outputs, as the runner's Rust tests check it on the engine's
(engine/tests/runner/milestone/s_properties.rs). A law that holds on the oracle, with its control showing it can fail, is the
proof that the generator and the law are sound; the engine's outputs are then held to the same law.

    python3 -I -B engine/tests/properties/laws.py [--jobs N<=4] [--verbose]   -> also writes kernel.refcore

The files are unpacked under engine/tests/out/properties/ (run output, ignored by git and by TESTS.sha256).
"""
import os
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402
import properties.gen as G  # noqa: E402

OUT = 'engine/tests/out/properties/'
REFCORE = os.path.join(HERE, 'kernel.refcore')


def blocks(out):
    """{ask name: [its lines]}: each unindented line opens a block named by what precedes its first ':'."""
    res, cur = {}, None
    for line in out.split('\n'):
        if not line:
            continue
        if not line.startswith(' '):
            cur = line.split(':', 1)[0]
            res[cur] = []
        res.setdefault(cur, []).append(line)
    return res


def values(line):
    """The values of an ask line `name: v` or `name: (v1, v2, …)`, split at top-level ', ' (a hole's question is
    quoted and may hold commas or parentheses)."""
    v = line.split(': ', 1)[1]
    if not v.startswith('('):
        return [v]
    v = v[1:-1]
    out, depth, q, cur = [], 0, False, ''
    i = 0
    while i < len(v):
        ch = v[i]
        if ch == '"':
            q = not q
        elif not q and ch == '(':
            depth += 1
        elif not q and ch == ')':
            depth -= 1
        if not q and depth == 0 and v[i:i + 2] == ', ':
            out.append(cur)
            cur = ''
            i += 2
            continue
        cur += ch
        i += 1
    return out + [cur]


def cents(v):
    """'-1 234.56 DUK' -> -123456; None for a hole or anything else."""
    if not v.endswith(' DUK'):
        return None
    n = v[:-4].replace(' ', '')
    neg = n.startswith('-')
    n = n.lstrip('-')
    if '.' not in n or not n.replace('.', '').isdigit():
        return None
    a, b = n.split('.')
    if len(b) != 2:
        return None
    return (-1 if neg else 1) * (int(a) * 100 + int(b))


def is_hole(v):
    return v.startswith('?')


# -- the laws: each takes {variant: (out, rc)} and returns (ok, detail) ---------------------------------------------

def law_determinism(v):
    (a, ra), (b, rb) = v['one'], v['split']
    if ra != 0:
        return False, 'not answered (rc %s): %s' % (ra, a.split('\n')[0][:80])
    return (a == b and ra == rb), 'one file and %s equal' % 'split' if a == b else R.first_diff(a, b)


def law_conservation(v):
    out, rc = v['one']
    if rc != 0:
        return False, 'not answered: ' + out.split('\n')[0][:80]
    vs = values(blocks(out)['flows_all'][0])
    cs = [cents(x) for x in vs]
    if any(c is None for c in cs):
        return False, 'a value is not a sum in DUK: %s' % vs
    return sum(cs) == 0 and any(cs), 'sum of %d places\' flows %d (nonzero terms %d)' % (len(cs), sum(cs), sum(1 for c in cs if c))


def law_foresight_known(v):
    (a, ra), (b, rb) = v['base'], v['later']
    if ra or rb:
        return False, 'not answered'
    ba, bb = blocks(a), blocks(b)
    same = ba['early'] == bb['early']
    control = ba['at_on'] != bb['at_on']
    return same and control, 'as known D: %s; at on (control): %s' % (
        'equal' if same else 'DIFFERS', 'differs' if control else 'EQUAL (vacuous)')


def law_foresight_decision(v):
    (a, ra), (b, rb), (c, rc) = v['base'], v['later'], v['control']
    if ra or rb or rc:
        return False, 'not answered'
    sa, sb, sc = blocks(a)['swept'][0], blocks(b)['swept'][0], blocks(c)['swept'][0]
    return sa == sb and sa != sc and not is_hole(values(sa)[0]), 'after: %s; before (control): %s (%s)' % (
        'equal' if sa == sb else 'DIFFERS', 'differs' if sa != sc else 'EQUAL (vacuous)', values(sa)[0])


def law_hole(v):
    (a, ra), (b, rb) = v['base'], v['dropped']
    if ra or rb:
        return False, 'not answered'
    ba, bb = blocks(a), blocks(b)
    bad, changed = [], 0
    for k in ('flows_all', 'bals', 'zero'):  # the asks; a `warning:` line isn't one
        va, vb = values(ba[k][0]), values(bb[k][0])
        if any(is_hole(x) for x in va):
            bad.append('%s: a hole with the row present' % k)
        for x, y in zip(va, vb):
            if x != y:
                changed += 1
                if not is_hole(y):
                    bad.append('%s: %s became %s, not a hole' % (k, x, y))
    zero = values(bb['zero'][0])
    if not all(is_hole(z) for z in zero):
        bad.append('zero: %s' % zero)
    return not bad and changed > 0, '; '.join(bad) or '%d values became holes, none a number' % changed


LAWS = {'determinism': law_determinism, 'conservation': law_conservation, 'foresight-known': law_foresight_known,
        'foresight-decision': law_foresight_decision, 'foresight-projection': law_foresight_decision, 'hole': law_hole}


def unpack(case, variant, files):
    d = OUT + '%s/%s/' % (case, variant)
    os.makedirs(R.ROOT + d, exist_ok=True)
    for p, t in files:
        with open(R.ROOT + d + p, 'w', encoding='utf-8') as f:
            f.write(t)
    return {'id': 'P:%s/%s' % (case, variant), 'layer': 'unit', 'cwd': d, 'files': ' '.join(d + p for p, _ in files),
            'asks': '', 'options': '', 'baseline': '', 'block': '', 'note': '', 'baseline_s': ''}


def main(argv):
    jobs = int(argv[argv.index('--jobs') + 1]) if '--jobs' in argv else 4
    R.need_watchdog()
    rc_hash = R.refcore_hash()
    if open(G.CORPUS, 'rb').read() != G.corpus():
        sys.exit('kernel.corpus is not what gen.py\'s seeds give: run gen.py first')
    recs = G.records(open(G.CORPUS, 'rb').read())
    checks = [(law, case, variant, unpack(case, variant, files)) for law, case, variant, args, files in recs]
    with ThreadPoolExecutor(min(jobs, R.MAX_JOBS)) as ex:
        res = list(ex.map(lambda x: R.run_one(x[3], 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache'), checks))
    cases = {}
    stored = []
    for (law, case, variant, c), e in zip(checks, res):
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s/%s: over the budget or killed; refcore must answer every record' % (case, variant))
        cases.setdefault((law, case), {})[variant] = (e['out'], e['rc'])
        stored.append(G.ns(case) + G.ns(variant) + G.ns(str(e['rc'])) + G.ns(e['out']))
    # refcore's outputs, for the runner's own test of the Rust laws (laws.rs, every_law_holds_on_refcore)
    with open(REFCORE, 'wb') as f:
        f.write(b''.join(stored))
    tally = {}
    for (law, case), v in sorted(cases.items()):
        ok, detail = LAWS[law](v)
        tally.setdefault(law, [0, 0])[0 if ok else 1] += 1
        if not ok or '--verbose' in argv:
            print('%s %s: %s' % ('ok  ' if ok else 'FAIL', case, detail))
    for law in LAWS:
        print('%-20s %d hold, %d fail on refcore' % (law, *tally.get(law, [0, 0])))
    return 0 if all(t[1] == 0 for t in tally.values()) else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
