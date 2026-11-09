"""M1's laws, checked on refcore (T, issue #108; PROPERTIES.md § M1). Standard library only.

Runs every record of m1.corpus on the frozen refcore through run.py's run_one (its 90 s budget, its cache, the
watchdog it needs) and checks each law on refcore's outputs, as runner/src/laws_m1.rs checks it on the engine's
(runner/milestone/m1_properties.rs). A case refcore breaks inside a filed issue is held "for #n" in held.tsv (target
m1_properties, its case id): it is reported, and its bytes are never an expected value. Any other failure fails the
run: it is a STOP-REPORT, never a changed law.

    python3 -I -B engine/tests/properties/laws_m1.py [--jobs N<=2] [--verbose]   -> also writes m1.refcore
    python3 -I -B engine/tests/properties/laws_m1.py --check   -> exit 1 if m1.corpus or m1.refcore isn't what the seeds
                                                                  and refcore give, or a law fails outside held.tsv

The files are unpacked under engine/tests/out/properties/ (run output, ignored by git and by TESTS.sha256).
"""
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402
import properties.gen_m1 as G  # noqa: E402
from properties.laws import blocks, values, cents, is_hole  # noqa: E402

OUT = 'engine/tests/out/properties/'
REFCORE = os.path.join(HERE, 'm1.refcore')
HELD = os.path.join(os.path.dirname(HERE), 'held.tsv')


def ok_runs(v, *names):
    for n in names:
        out, rc = v[n][0], v[n][1]
        if rc != 0:
            return '%s: not answered (rc %s): %s' % (n, rc, out.split('\n')[0][:100])
    return None


def first(out, ask):
    b = blocks(out).get(ask)
    return b[0] if b else None


def law_determinism(v):
    bad = ok_runs(v, 'one', 'split')
    if bad:
        return False, bad
    same = v['one'][:2] == v['split'][:2]
    return same, 'one file and split %s' % ('equal' if same else 'DIFFER')


def law_conservation(v):
    bad = ok_runs(v, 'one')
    if bad:
        return False, bad
    cs = [cents(x) for x in values(first(v['one'][0], 'flows_all'))]
    if any(c is None for c in cs):
        return False, 'a flow is not a sum in DUK'
    return sum(cs) == 0 and any(cs), 'flows sum to %d (%d nonzero)' % (sum(cs), sum(1 for c in cs if c))


def nw(out):
    b = blocks(out)
    return [cents(values(b[k][0])[0]) for k in sorted((k for k in b if k.startswith('nw_')), key=lambda k: int(k[3:]))]


def law_networth(v):
    bad = ok_runs(v, 'base', 'inside', 'boundary')
    if bad:
        return False, bad
    a, i, b = nw(v['base'][0]), nw(v['inside'][0]), nw(v['boundary'][0])
    if None in a + i + b or not a:
        return False, 'a worth is not a sum in DUK'
    if a != i:
        return False, 'an inside transfer moved the worth: %s vs %s' % (a, i)
    d = [y - x for x, y in zip(a, b)]
    j = next((k for k, x in enumerate(d) if x), None)
    if j is None or j == 0 or d[j] <= 0 or any(x != d[j] for x in d[j:]) or any(d[:j]):
        return False, 'the boundary move changed the worth by %s, not by one amount from its day on' % d
    return True, 'inside: unchanged; boundary: +%d from day %d of %d' % (d[j], j, len(d))


def law_scenario_before(v):
    bad = ok_runs(v, 'one')
    if bad:
        return False, bad
    b = blocks(v['one'][0])
    listed = [x for x in b['pre_s'] if x.startswith('  before from: ')]
    pre_m, pre_s = values(b['pre_m'][0]), values(b['pre_s'][0])
    if b['post_m'][0].split(': ', 1)[1] == b['post_s'][0].split(': ', 1)[1]:
        return False, 'the scenario changes nothing after its from: vacuous'
    if any(is_hole(x) or x.startswith('refused') for x in pre_m + pre_s):
        return False, 'a value before from is a hole or a refusal'
    if any(x.startswith('  before from: mv_l ') for x in listed):
        return True, 'listed: %s' % '; '.join(x.strip() for x in listed)
    if 'amt_mv_l' in v['one'][2]:
        return False, 'mv_l reads the scenario before from, and no before from: line lists it'
    if listed:
        return False, 'listed with nothing to list: %s' % listed
    return pre_m == pre_s, 'before from: %s' % ('equal' if pre_m == pre_s else 'DIFFERS: %s vs %s' % (pre_m, pre_s))


def proj(out):
    return {k: x for k, x in blocks(out).items() if k.startswith('p_')}


def law_asks_no_money(v):
    bad = ok_runs(v, 'alone', 'beside', 'control')
    if bad:
        return False, bad
    a, b, c = proj(v['alone'][0]), proj(v['beside'][0]), proj(v['control'][0])
    if not a:
        return False, 'no projection asks'
    if a != b:
        return False, 'the projection differs beside other asks: %s' % sorted(k for k in a if a[k] != b.get(k))
    return a != c, 'equal beside other asks; the planted move %s' % ('changes it' if a != c else 'DOESN\'T (vacuous)')


def law_asks_no_money_explain(v):
    bad = ok_runs(v, 'alone', 'explained')
    if bad:
        return False, bad
    a, b = proj(v['alone'][0]), proj(v['explained'][0])
    if not first(v['explained'][0], 'k_explain'):
        return False, 'no explain block'
    return a == b and bool(a), 'equal beside explain' if a == b else 'the projection differs beside explain'


def life(out):
    return values(first(out, 'life'))


def law_lifespans(v):
    bad = ok_runs(v, 'inside', 'early', 'late')
    if bad:
        return False, bad
    i = life(v['inside'][0])
    if [cents(x) for x in i][0::2] != [0, 0] or not cents(i[1]):
        return False, 'inside: before, during, after are %s, not 0, the amount, 0' % i
    for k in ('early', 'late'):
        out = v[k][0]
        if 'RUN-LIFESPAN' not in out:
            return False, '%s: a leg outside the lifespan isn\'t refused' % k
        for x in life(out)[0::2]:  # before the lifespan and after it; the middle day is inside it
            if not (x.startswith('refused: RUN-LIFESPAN') or cents(x) == 0):
                return False, '%s: tmp holds %s outside its lifespan' % (k, x)
    return True, 'inside: %s; outside: RUN-LIFESPAN' % ', '.join(i)


def law_foresight(v):
    bad = ok_runs(v, 'base', 'later', 'control')
    if bad:
        return False, bad
    a, b, c = (first(v[k][0], 'swept') for k in ('base', 'later', 'control'))
    if any(is_hole(x) or x.startswith('refused') for x in values(a)):
        return False, 'the decision is a hole or refused: %s' % a
    return a == b and a != c, 'after: %s; before (control): %s' % (
        'equal' if a == b else 'DIFFERS', 'differs' if a != c else 'EQUAL (vacuous)')


HOLE_ASKS = ('flows_all', 'bals', 'v_share', 'v_fee', 'v_agg', 'v_cmp', 'v_if', 'zero')


def law_hole(v):
    bad = ok_runs(v, 'base', 'dropped')
    if bad:
        return False, bad
    ba, bb = blocks(v['base'][0]), blocks(v['dropped'][0])
    out, changed = [], 0
    for k in HOLE_ASKS:
        va, vb = values(ba[k][0]), values(bb[k][0])
        if any(is_hole(x) or x.startswith('refused') for x in va):
            out.append('%s: a hole or refusal with the row present' % k)
        if len(va) != len(vb):
            out.append('%s: %d values, then %d' % (k, len(va), len(vb)))
        for x, y in zip(va, vb):
            if x != y:
                changed += 1
                if not is_hole(y):
                    out.append('%s: %s became %s, not a hole' % (k, x, y))
    if not all(is_hole(z) for z in values(bb['zero'][0])):
        out.append('zero: %s' % bb['zero'][0])
    return not out and changed > 0, '; '.join(out) or '%d values became holes, none a number' % changed


def law_hole_explain(v):
    bad = ok_runs(v, 'base', 'dropped')
    if bad:
        return False, bad
    a, b = values(first(v['base'][0], 'ex'))[0], values(first(v['dropped'][0], 'ex'))[0]
    return not is_hole(a) and is_hole(b), 'explained: %s, then %s' % (a, b)


def oracle(text):
    m = re.search(r'note "steps oracle: ([^"]*)"', text)
    return dict(x.split('=', 1) for x in m.group(1).split('; ')) if m else {}


def law_steps(v):
    bad = ok_runs(v, 'one')
    if bad:
        return False, bad
    out, text = v['one'][0], v['one'][2]
    want = oracle(text)
    if not want:
        return False, 'no oracle in the text'
    wrong = []
    for ask, w in sorted(want.items()):
        got = values(first(out, ask))[0]
        good = got.startswith('refused: ' + w + ' ') if w in ('RUN-KEY', 'RUN-DOMAIN') else got == w
        if not good:
            wrong.append('%s: %s, the oracle %s' % (ask, got, w))
    return not wrong, '; '.join(wrong) or 'all %d asks as the oracle' % len(want)


def law_low_min(v):
    bad = ok_runs(v, 'one')
    if bad:
        return False, bad
    lo, mn = values(first(v['one'][0], 'lo'))[0], values(first(v['one'][0], 'mn'))[0]
    return lo == mn and cents(lo) is not None, 'low %s, min over days %s' % (lo, mn)


LAWS = {'determinism': law_determinism, 'conservation': law_conservation, 'networth': law_networth,
        'scenario-before': law_scenario_before, 'asks-no-money': law_asks_no_money,
        'asks-no-money-explain': law_asks_no_money_explain, 'lifespans': law_lifespans,
        'foresight-keyed': law_foresight, 'foresight-low': law_foresight, 'hole': law_hole,
        'hole-explain': law_hole_explain, 'steps': law_steps, 'low-min': law_low_min}


def held():
    out = {}
    for line in open(HELD, encoding='utf-8').read().split('\n'):
        f = line.split('\t')
        if len(f) >= 3 and f[1] == 'm1_properties':
            out[f[0]] = f[2]
    return out


def unpack(case, variant, files):
    d = OUT + 'm1/%s/%s/' % (case, variant)
    os.makedirs(R.ROOT + d, exist_ok=True)
    for p, t in files:
        with open(R.ROOT + d + p, 'w', encoding='utf-8') as f:
            f.write(t)
    return {'id': 'P:%s/%s' % (case, variant), 'layer': 'unit', 'cwd': d, 'files': ' '.join(d + p for p, _ in files),
            'asks': '', 'options': '', 'baseline': '', 'block': '', 'note': '', 'baseline_s': ''}


def main(argv):
    jobs = int(argv[argv.index('--jobs') + 1]) if '--jobs' in argv else 2
    R.need_watchdog()
    rc_hash = R.refcore_hash()
    data = G.corpus()
    if open(G.CORPUS, 'rb').read() != data:
        sys.exit('m1.corpus is not what gen_m1.py\'s seeds give: run gen_m1.py first')
    recs = G.G.records(data)
    checks = [(law, case, variant, '\n'.join(t for _, t in files), unpack(case, variant, files))
              for law, case, variant, args, files in recs]

    def one(c):
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed; refcore must answer every record' % c['id'])
        if 'Traceback' in e['out']:
            sys.exit('%s: refcore crashes: %s' % (c['id'], e['out'][-300:]))
        return e
    with ThreadPoolExecutor(min(jobs, R.MAX_JOBS, 2)) as ex:
        res = list(ex.map(lambda x: one(x[4]), checks))
    cases, stored = {}, []
    for (law, case, variant, text, c), e in zip(checks, res):
        cases.setdefault((law, case), {})[variant] = (e['out'], e['rc'], text)
        stored.append(G.G.ns(case) + G.G.ns(variant) + G.G.ns(str(e['rc'])) + G.G.ns(e['out']))
    if '--check' in argv:
        if open(REFCORE, 'rb').read() != b''.join(stored):
            print('m1.refcore is not refcore\'s runs of m1.corpus: run properties/laws_m1.py')
            return 1
    else:
        with open(REFCORE, 'wb') as f:
            f.write(b''.join(stored))
    hold = held()
    tally, failed = {}, 0
    for (law, case), v in sorted(cases.items()):
        ok, detail = LAWS[law](v)
        if not ok and case in hold:
            tally.setdefault(law, [0, 0, 0])[2] += 1
            if '--verbose' in argv:
                print('held %s for %s: %s' % (case, hold[case], detail))
            continue
        if ok and case in hold:
            print('UNHELD %s holds on refcore: drop its held.tsv row (%s)' % (case, hold[case]))
            failed += 1
        tally.setdefault(law, [0, 0, 0])[0 if ok else 1] += 1
        failed += 0 if ok else 1
        if not ok or '--verbose' in argv:
            print('%s %s: %s' % ('ok  ' if ok else 'FAIL', case, detail))
    for law in LAWS:
        print('%-22s %d hold, %d fail, %d held on refcore' % (law, *tally.get(law, [0, 0, 0])))
    print('m1.refcore: %d runs of %d cases%s' % (len(stored), len(cases), ', what refcore gives' if '--check' in argv
                                                 else ''))
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
