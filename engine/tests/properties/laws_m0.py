"""M0's laws, checked on refcore (issue #56; PROPERTIES.md § M0). Standard library only.

Runs every record of m0.corpus on the frozen refcore through run.py's run_one (its budget, its cache, the watchdog it
needs) twice, as the load probe (the record's argument words) and as `reprint <files>`, then reprints each canonical
text refcore printed, and checks each law on the outputs, as the runner checks it on the engine's
(runner/src/laws_m0.rs; runner/milestone/m0_properties.rs). A law that holds on the oracle, with its control showing
it can fail, is the proof that the generator and the law are sound.

    python3 -I -B engine/tests/properties/laws_m0.py [--jobs N<=4] [--verbose]   -> also writes m0.refcore
    python3 -I -B engine/tests/properties/laws_m0.py --check   -> exit 1 if m0.corpus or m0.refcore isn't what the
                                                                 seeds and refcore give, or a law fails (run.py's
                                                                 self-test, 8i)

m0.refcore holds four netstrings per run (case, variant, exit code, output), the variant `<variant>:probe` or
`<variant>:reprint`. An `again` run (the reprint of a reprint's sections) isn't stored: idempotence, checked here, makes
it its reprint's bytes. The files are unpacked under engine/tests/out/properties-m0/ (run output, ignored by git and by
TESTS.sha256).
"""
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402
import properties.gen_m0 as G  # noqa: E402

OUT = 'engine/tests/out/properties-m0/'
REFCORE = os.path.join(HERE, 'm0.refcore')
REFUSED = re.compile(r'refused: ([A-Z]+-[A-Z0-9-]+) ')


def records(data):
    n = G.netstrings(data)
    out = []
    for k in range(0, len(n), 5):
        files = G.netstrings(n[k + 4])
        out.append((n[k].decode(), n[k + 1].decode(), n[k + 2].decode(), n[k + 3].decode(),
                    [(files[j].decode(), files[j + 1]) for j in range(0, len(files), 2)]))
    return out


def sections(out):
    """A reprint's output: [(path, canonical text)] and the rest (m0/vectors.py's sections, the runner's printed)."""
    lines = out.split('\n')
    if lines and lines[-1] == '':
        lines.pop()
    secs, tail, in_tail = [], [], False
    for at, line in enumerate(lines):
        last = at + 1 == len(lines) and line.startswith('sha256 ')
        if line.startswith('--- ') and not in_tail:
            secs.append([line[4:], ''])
        elif secs and not in_tail and not last:
            secs[-1][1] += line + '\n'
        else:
            in_tail = True
            tail.append(line + '\n')
    return [tuple(s) for s in secs], ''.join(tail)


# -- the laws: each takes {variant: (out, rc)} (variants `<v>:probe`, `<v>:reprint`, `<v>:again`) -> (ok, detail) ----

def code(out):
    """The code of an output's last line, `refused: <code> …`; None if the last line isn't one."""
    last = out.rstrip('\n').split('\n')[-1]
    m = REFUSED.match(last + ' ')
    return m.group(1) if m else None


def clean(v, name):
    out, rc = v[name + ':probe']
    return out == G.PROBED and rc == 1


def law_spelling(v):
    base, brc = v['base:reprint']
    if brc != 0 or not clean(v, 'base'):
        return False, 'the base does not load clean: %s' % base[:120]
    bad = [k for k in v if k.endswith(':reprint') and not k.startswith(('base:', 'control:'))
           and (v[k] != v['base:reprint'] or not clean(v, k[:-8]))]
    if 'control:reprint' not in v:
        return False, 'no control'
    ctrl = v['control:reprint'] != v['base:reprint'] and clean(v, 'control')
    n = sum(1 for k in v if k.endswith(':reprint')) - 2
    return not bad and ctrl and n > 0,'%d spellings print as the base%s; control %s' % (
        n - len(bad), ' (DIFFER: %s)' % ', '.join(bad) if bad else '', 'differs' if ctrl else 'EQUAL (vacuous)')


def law_order(v):
    loads = all(clean(v, k) for k in ('whole', 'split', 'reversed'))
    (s, src), (r, rrc) = v['split:reprint'], v['reversed:reprint']
    same = src == 0 and (s, src) == (r, rrc)
    ctrl = 'control:reprint' in v and v['control:reprint'] != (s, src) and clean(v, 'control')
    return loads and same and ctrl, 'whole and split load clean: %s; split and reversed reprint %s; control %s' % (
        loads, 'equal' if same else 'DIFFER', 'differs' if ctrl else 'EQUAL (vacuous)')


def total_one(out, rc, probe):
    """None if the output is total's (one refusal line, after warnings, or a reprint), else what's wrong."""
    lines = out.split('\n')
    if lines[-1] != '' or rc not in (0, 1):
        return 'exit %s, or no newline at the end' % rc
    lines.pop()
    if rc == 1:
        rest = [l for l in lines[:-1] if not l.startswith('warning: ')]
        c = code(out) if lines else None
        if rest or c is None or not (c.startswith('LOAD-') or (probe and out.endswith(G.PROBED))):
            return 'exit 1 but not one LOAD-* refusal line%s: %r' % (' (or the probe\'s)' if probe else '', out[:160])
        return None
    if probe:
        return 'the probe exits 0: %r' % out[:160]
    secs, tail = sections(out)
    if not secs or not tail.startswith('sha256 ') or tail.count('\n') != 1:
        return 'exit 0 but not sections and a sha256 line: %r' % out[:160]
    return None


def law_total(v):
    bad = []
    for k, (out, rc) in sorted(v.items()):
        if k.endswith(':again'):
            continue
        e = total_one(out, rc, k.endswith(':probe'))
        if e:
            bad.append('%s: %s' % (k, e))
    probe, reprint = v['mutant:probe'], v['mutant:reprint']
    # reprint is E §3.1's step 1, the parse, which every later step needs: a refused reprint is the probe's refusal
    if reprint[1] != 0 and code(reprint[0]) != code(probe[0]):
        bad.append('reprint is refused %s, the probe %s' % (code(reprint[0]), code(probe[0])))
    return not bad, '; '.join(bad) or 'probe %s, reprint exit %d' % (code(probe[0]), reprint[1])


def law_depth(v, case):
    """Each variant `dNN` (the nesting) or `nNN` (a chain's terms): refused past the limit, never short of it."""
    form = case.split('/')[1]
    bad, seen = [], set()
    for k, (out, rc) in sorted(v.items()):
        if not k.endswith(':probe'):
            continue
        name = k[:-6]
        n = int(name[1:])
        c = code(out)
        if form == 'chain':
            want = 'REQ-ASK'
        elif form == 'asif':
            want = 'LOAD-ASIF' if n > 4 else 'REQ-ASK'
        else:
            want = 'LOAD-DEPTH' if n > 8 else 'REQ-ASK'
        seen.add(want)
        if c != want or (want == 'REQ-ASK' and out != G.PROBED):
            bad.append('%s: %s, wants %s' % (name, out.strip()[:120], want))
    both = form == 'chain' or len(seen) == 2
    return not bad and both, '; '.join(bad) or '%d variants, refused exactly past the limit' % len(seen)


def law_again(v):
    """Idempotence: a canonical text reprints as itself."""
    bad = [k for k in v if k.endswith(':again') and v[k] != v[k[:-6] + ':reprint']]
    return not bad, 'again differs: %s' % ', '.join(bad) if bad else 'every reprint reprints as itself'


LAWS = {'spelling': law_spelling, 'order': law_order, 'total': law_total, 'depth': None}


def unpack(case, variant, files, run):
    d = OUT + '%s/%s/' % (case, variant)
    os.makedirs(R.ROOT + d, exist_ok=True)
    for p, t in files:
        with open(R.ROOT + d + p, 'wb') as f:
            f.write(t if isinstance(t, bytes) else t.encode('utf-8'))
    return {'id': 'P:%s/%s' % (case, variant), 'layer': 'interpreter', 'cwd': d,
            'files': ' '.join(d + p for p, _ in files), 'asks': '', 'options': run, 'baseline': '', 'block': '',
            'note': '', 'baseline_s': ''}


def runs(checks, jobs, rc_hash):
    def one(c):
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed; refcore must answer every record' % c['id'])
        if 'Traceback' in e['out'] or 'REFCORE-NOT-YET' in e['out']:
            sys.exit('%s: refcore crashes or prints REFCORE-NOT-YET: %s' % (c['id'], e['out'][-300:]))
        try:
            e['out'].encode('utf-8')
        except UnicodeEncodeError:
            sys.exit('%s: refcore prints bytes that are not UTF-8' % c['id'])
        return e
    with ThreadPoolExecutor(min(jobs, R.MAX_JOBS)) as ex:
        return list(ex.map(one, checks))


def main(argv):
    jobs = int(argv[argv.index('--jobs') + 1]) if '--jobs' in argv else 2
    R.need_watchdog()
    rc_hash = R.refcore_hash()
    data, _, _ = G.corpus()
    if open(G.CORPUS, 'rb').read() != data:
        sys.exit('m0.corpus is not what gen_m0.py\'s seeds give: run gen_m0.py first')
    recs = records(data)
    first = []
    for law, case, variant, args, files in recs:
        first.append((law, case, variant + ':probe', unpack(case, variant + '/probe', files, args)))
        first.append((law, case, variant + ':reprint',
                      unpack(case, variant + '/reprint', files, ' '.join(['reprint'] + [p for p, _ in files]))))
    res = runs([c for _, _, _, c in first], jobs, rc_hash)
    second = []
    for (law, case, variant, c), e in zip(first, res):
        if variant.endswith(':reprint') and e['rc'] == 0:
            secs, _ = sections(e['out'])
            v = variant[:-8]
            second.append((law, case, v + ':again', unpack(case, v + '/again', secs,
                                                           ' '.join(['reprint'] + [p for p, _ in secs]))))
    res2 = runs([c for _, _, _, c in second], jobs, rc_hash)
    cases, stored = {}, []
    for (law, case, variant, c), e in zip(first + second, res + res2):
        cases.setdefault((law, case), {})[variant] = (e['out'], e['rc'])
        if not variant.endswith(':again'):  # an again run is its reprint's bytes when the law holds: not stored
            stored.append(G.ns(case) + G.ns(variant) + G.ns(str(e['rc'])) + G.ns(e['out']))
    if '--check' in argv:
        if open(REFCORE, 'rb').read() != b''.join(stored):
            print('m0.refcore is not refcore\'s runs of m0.corpus: run properties/laws_m0.py')
            return 1
    else:
        with open(REFCORE, 'wb') as f:
            f.write(b''.join(stored))
    tally, codes = {}, {}
    for (law, case), v in sorted(cases.items()):
        checks = [('again', law_again(v)), (law, law_depth(v, case) if law == 'depth' else LAWS[law](v))]
        for name, (ok, detail) in checks:
            tally.setdefault(name, [0, 0])[0 if ok else 1] += 1
            if not ok or '--verbose' in argv:
                print('%s %s %s: %s' % ('ok  ' if ok else 'FAIL', name, case, detail))
        if law == 'total':
            for k, (out, _) in v.items():
                if k.endswith(':probe'):
                    codes[code(out)] = codes.get(code(out), 0) + 1
    for law in list(LAWS) + ['again']:
        print('%-10s %d hold, %d fail on refcore' % (law, *tally.get(law, [0, 0])))
    print('total\'s probe codes: %s' % ', '.join('%s %d' % kv for kv in sorted(codes.items(), key=str)))
    print('m0.refcore: %d runs (%d bytes)%s' % (len(stored), sum(len(s) for s in stored),
                                                  ', what refcore gives' if '--check' in argv else ''))
    return 0 if all(t[1] == 0 for t in tally.values()) else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
