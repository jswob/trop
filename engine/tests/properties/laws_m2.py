"""M2's laws, checked on refcore (T, issue #167; PROPERTIES.md § M2). Standard library only.

Runs every record of m2.corpus on the frozen refcore through run.py's run_one (its 90 s budget, its cache, the
watchdog it needs) and checks each law on refcore's outputs, as runner/src/laws_m2.rs checks it on the engine's
(runner/milestone/m2_properties.rs). A case refcore breaks inside a filed issue is held "for #n" in held.tsv (target
m2_properties, its case id); any other failure fails the run: a STOP-REPORT, never a changed law.

    python3 -I -B engine/tests/properties/laws_m2.py [--jobs N<=2] [--verbose]   -> also writes m2.refcore
    python3 -I -B engine/tests/properties/laws_m2.py --check   -> exit 1 if m2.corpus or m2.refcore isn't what the seeds
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
import properties.gen_m2 as G  # noqa: E402
from properties.laws import blocks  # noqa: E402

OUT = 'engine/tests/out/properties/'
REFCORE = os.path.join(HERE, 'm2.refcore')
HELD = os.path.join(os.path.dirname(HERE), 'held.tsv')
PAY = re.compile(r'^    matched pay\[\d{4}-\d{2}\] on [0-9-]+: .*, difference (.+?), timing ')


def ok_runs(v, *names):
    for n in names:
        out, rc = v[n][0], v[n][1]
        if rc != 0:
            return '%s: not answered (rc %s): %s' % (n, rc, out.split('\n')[0][:100])
    return None


def law_masking(v):
    bad = ok_runs(v, 'base', 'late')
    if bad:
        return False, bad
    a, b = blocks(v['base'][0]), blocks(v['late'][0])
    k = sorted(n for n in a if n.startswith('k_'))
    o = sorted(n for n in a if n.startswith('o_'))
    if not k or not o:
        return False, 'no as-known or control asks'
    differ = [n for n in k if a.get(n) != b.get(n)]
    if differ:
        return False, 'as known K the late row moved %s' % differ
    seen = [n for n in o if a.get(n) != b.get(n)]
    if not seen:
        return False, 'control: at on the late row moved nothing'
    return True, '%d reads as known K unchanged; at on %s moved' % (len(k), seen)


def pay_differences(out):
    rep = blocks(out).get('rep') or []
    return [m.group(1) for m in (PAY.match(line) for line in rep) if m]


def law_split(v):
    bad = ok_runs(v, 'model', 'bug')
    if bad:
        return False, bad
    model, bug = pay_differences(v['model'][0]), pay_differences(v['bug'][0])
    if len(model) != 6 or len(bug) != 6:
        return False, 'not six matched pay lines: %d, %d' % (len(model), len(bug))
    if any(d != '0.00 DUK' for d in model):
        return False, 'the replay shows a law difference: %s' % model
    if all(d == '0.00 DUK' for d in bug):
        return False, 'control: the rounding bug shows nothing'
    return True, 'six pay lines 0.00; the bug shows %d' % sum(d != '0.00 DUK' for d in bug)


def law_fixed_past(v):
    bad = ok_runs(v, 'base', 'late', 'early')
    if bad:
        return False, bad
    a, b, c = blocks(v['base'][0]), blocks(v['late'][0]), blocks(v['early'][0])
    names = sorted(a)
    if not names:
        return False, 'no asks'
    differ = [n for n in names if a.get(n) != b.get(n)]
    if differ:
        return False, 'the version known later moved %s before its known day' % differ
    seen = [n for n in names if a.get(n) != c.get(n)]
    if not seen:
        return False, 'control: the version known before the bookings moved nothing'
    return True, '%d reads before D unchanged; known early, %s moved' % (len(names), seen)


LAWS = {'masking': law_masking, 'split': law_split, 'fixed-past': law_fixed_past}


def held():
    out = {}
    for line in open(HELD, encoding='utf-8').read().split('\n'):
        f = line.split('\t')
        if len(f) >= 3 and f[1] == 'm2_properties':
            out[f[0]] = f[2]
    return out


def unpack(case, variant, files):
    d = OUT + 'm2/%s/%s/' % (case, variant)
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
        sys.exit('m2.corpus is not what gen_m2.py\'s seeds give: run gen_m2.py first')
    recs = G.G.records(data)
    checks = [(law, case, variant, unpack(case, variant, files)) for law, case, variant, args, files in recs]

    def one(c):
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed; refcore must answer every record' % c['id'])
        if 'Traceback' in e['out']:
            sys.exit('%s: refcore crashes: %s' % (c['id'], e['out'][-300:]))
        return e
    with ThreadPoolExecutor(min(jobs, R.MAX_JOBS, 2)) as ex:
        res = list(ex.map(lambda x: one(x[3]), checks))
    cases, stored = {}, []
    for (law, case, variant, c), e in zip(checks, res):
        cases.setdefault((law, case), {})[variant] = (e['out'], e['rc'])
        stored.append(G.G.ns(case) + G.G.ns(variant) + G.G.ns(str(e['rc'])) + G.G.ns(e['out']))
    if '--check' in argv:
        if open(REFCORE, 'rb').read() != b''.join(stored):
            print('m2.refcore is not refcore\'s runs of m2.corpus: run properties/laws_m2.py')
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
        print('%-12s %d hold, %d fail, %d held on refcore' % (law, *tally.get(law, [0, 0, 0])))
    print('m2.refcore: %d runs of %d cases%s' % (len(stored), len(cases), ', what refcore gives' if '--check' in argv
                                                 else ''))
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
