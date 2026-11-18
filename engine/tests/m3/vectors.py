"""Milestone M3's `check` vectors (issue #226; MILESTONE-M3.md rows 2 and 3): `tally check <path>…` (OPERATIONS.md
§2.3) on M0's 315 text sets and on the fixtures of m3/check/ (fixtures.tsv: one row per line kind the sets don't reach).
Standard library only.

Refcore has no `check` until the spec step writes it (the M3 brief, §2), so each vector's expected output is first
**predicted from refcore's `run`** (pick P8, the coordinator's, 2026-11-18), every line from a run refcore prints:

1. the load probe, `run <files> --ask zz_m0_probe` (M0's, stored in m0/load.refcore for the 315 sets; run here for a
   fixture): a load refusal there is `check`'s whole output, exit 1 (§2.3, a base that doesn't load);
2. else the warnings probe, `run <files> zz_m3_probe.tally --ask zz_m3_probe`: m3/zz_m3_probe.tally is one part whose one
   ask names a world no file declares, so it is refused alone (L §6.4, LOAD-SCENARIO) and nothing is evaluated; the run
   prints every refused scenario's `scenario <s>: refused: …` line, the probe's own line, and load's warnings (no walk,
   so no walk warning);
3. the asks a scan flags as naming a world no file declares (`in`, `against`, `with scenario`, `as if {scenario …}`), run
   by id: each line `<q>: refused: LOAD-SCENARIO ask <q> names …` is an ask refused alone; a flagged ask that answers
   isn't one (the scan only saves running every ask);
4. the assembly, §2.3's: `ok` alone on the first line when no scenario and no ask is refused; else each refused
   scenario's line in the full-path order of its file (refcore's `run` prints them by scenario id: pick P4), then the
   asks refused alone by id, then the file's full path, then line (pick P5); then the warnings as `run` prints them;
   exit 0 only after `ok`.

A fixture whose argument words `check` doesn't take (P3: `--ask`) is a usage error: exit 2, its words the engine's
(INTERFACE.md §1), so its stored output is empty and the runner compares the exit code alone.

Each prediction is tied, as m0/vectors.py ties the probe: for a set of checks.tsv, every stored output that compares
whole (not a household's, whose compare drops warning lines) and isn't refused whole must hold exactly the predicted
`WARN-UNREAD` lines (load decides them, E §3.1 step 6), and every `scenario <s>: refused: …` line a stored output holds
is a predicted line. A tie that fails is a STOP-REPORT, not a vector.

Stored in m3/check.vectors, eight netstrings each: id, group, source, cwd, the argument words joined by single spaces,
the file table (one line per file: the path as the run names it, a tab, the path under engine/tests/), the exit code,
the whole output. `source` is `run` (predicted from refcore's `run`, above), `rule` (a usage error: P3) or `check`
(refcore's own `check`, once it exists: `--confirm`, chunk C4, which runs refcore's `check` on every vector, holds it to
the prediction byte for byte and stores its bytes with `check`). runner/src/m3_texts.rs embeds the fixtures' texts
(M0's sets read m0_texts.rs and m0_spec_texts.rs), so the tests read no file (INTERFACE.md §4).

    python3 -I -B engine/tests/m3/vectors.py              -> writes m3/check.vectors, runner/src/m3_texts.rs
    python3 -I -B engine/tests/m3/vectors.py --check      -> exit 1 if either isn't what refcore's runs give
    python3 -I -B engine/tests/m3/vectors.py --show GLOB  -> the predicted output of the vectors whose id matches
    python3 -I -B engine/tests/m3/vectors.py --confirm    -> refcore's `check` on every vector, tied, stored (C4)
"""
import csv
import fnmatch
import importlib.util
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402

_spec = importlib.util.spec_from_file_location('m0vectors', os.path.join(os.path.dirname(HERE), 'm0', 'vectors.py'))
M0 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(M0)

STORE = os.path.join(HERE, 'check.vectors')
FIXTURES = os.path.join(HERE, 'check', 'fixtures.tsv')
TEXTS_RS = R.T + 'runner/src/m3_texts.rs'
PROBE_FILE = 'engine/tests/m3/zz_m3_probe.tally'
PROBE = 'zz_m3_probe'
PREFIX = 'engine/tests/'
SPEC = 'engine/spec/'
JOBS = 3   # run.py's own job count (--jobs 3)
ALONE = re.compile(r'^([a-z][a-z0-9_.]*): refused: LOAD-SCENARIO ask \1 names [a-z][a-z0-9_]*, neither the model nor a '
                   r'loaded scenario \((.*):(\d+)\)$')
SCENARIO_HEAD = re.compile(r'^scenario ([a-z][a-z0-9_]*) over ', re.M)
ASK_LINE = re.compile(r'^ask ([a-z][a-z0-9_]*):(.*)$', re.M)
WORLD_WORD = re.compile(r'(?:\bin|\bagainst|\bscenario) ([a-z][a-z0-9_]*)\b')
DECLARED = re.compile(r'^(?:[a-z]+) ([a-z][a-z0-9_]*)\b', re.M)
# the prelude's kinds and the calendar's indexes (L §3.5, §3.6, §4.1): never a world
PRELUDE = {'places', 'accounts', 'cash', 'claims', 'debts', 'assets', 'outside', 'parties', 'persons', 'events',
           'adjustments', 'pads', 'days', 'weeks', 'months', 'quarters', 'years', 'holidays', 'things'}


def netstrings(b):
    out, i = [], 0
    while i < len(b):
        j = b.index(b':', i)
        n = int(b[i:j])
        out.append(b[j + 1:j + 1 + n])
        if b[j + 1 + n:j + 2 + n] != b',':
            sys.exit('a netstring without its comma')
        i = j + 2 + n
    return out


def stored_probes():
    """m0/load.refcore's probe of each of the 315 sets: {id: (exit, output)}."""
    parts = netstrings(open(M0.LOAD, 'rb').read())
    out = {}
    for k in range(0, len(parts), 7):
        vid, _, _, argv, _, rc, whole = (p.decode('utf-8') for p in parts[k:k + 7])
        if argv.endswith('--ask ' + M0.PROBE):
            out[vid] = (int(rc), whole)
    return out


def path_words(args):
    """The words of a command line that are paths: those before `--ask` that aren't options."""
    out = []
    for a in args:
        if a == '--ask':
            break
        if not a.startswith('--'):
            out.append(a)
    return out


def table_of(vid, cwd, args):
    """[(the path as the run names it, the path from the repository root)]: the files the paths name (a directory: its
    .tally files as <dir>/<path>), then a joint model's members (M0's members); a path that names nothing is left out
    (the run refuses it LOAD-PATH)."""
    cwd = cwd.rstrip('/') or '.'
    table = []
    for a in path_words(args):
        p = os.path.normpath(os.path.join(cwd, a))
        if os.path.isdir(R.ROOT + p):
            table += [((a if a.endswith('/') else a + '/') + os.path.relpath(f, R.ROOT + p).replace(os.sep, '/'),
                       os.path.relpath(f, R.ROOT).replace(os.sep, '/')) for f in M0.collect(R.ROOT + p)]
        elif os.path.isfile(R.ROOT + p):
            table.append((a, p))
    table += M0.members(cwd, [k for k, _ in table])
    for key, repo in table:
        if not (repo.startswith(PREFIX) or repo.startswith(SPEC)):
            sys.exit('%s: %s is no file under engine/tests/ or engine/spec/' % (vid, key))
    return table


def vectors():
    """M0's 315 sets, then the fixtures of fixtures.tsv, each {id, group, cwd, args, table, checks}."""
    out = []
    for s in M0.text_sets():
        if s['checks']:
            out.append({'id': s['id'], 'group': s['group'], 'cwd': s['cwd'], 'args': s['args'],
                        'table': s['table'], 'checks': s['checks']})
    with open(FIXTURES, encoding='utf-8', newline='') as f:
        for r in csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE):
            cwd = os.path.normpath(PREFIX + r['cwd'])
            args = r['args'].split()
            out.append({'id': 'fixture/' + r['id'], 'group': r['id'].split('/')[0], 'cwd': cwd, 'args': args,
                        'table': table_of(r['id'], cwd, args), 'checks': [], 'rule': r['rule']})
    ids = [v['id'] for v in out]
    if len(set(ids)) != len(ids):
        sys.exit('two vectors share an id')
    return out


def under_tests(repo):
    """A table file's path as the store names it: under engine/tests/ (a spec text as ../spec/…)."""
    return repo[len(PREFIX):] if repo.startswith(PREFIX) else '../' + repo[len('engine/'):]


def run_check(v, words, table=None):
    """A run.py check that runs refcore with `words` (the argv after the program name) on v's files."""
    table = v['table'] if table is None else table
    return {'id': v['id'], 'layer': 'interpreter', 'cwd': v['cwd'], 'asks': '', 'baseline': '', 'block': '',
            'note': '', 'baseline_s': '', 'files': ' '.join(repo for _, repo in table), 'options': ' '.join(words)}


def refcore(checks):
    rc_hash = R.refcore_hash()

    def one(c):
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed on refcore (%s)' % (c['id'], c['options'][:120]))
        if 'REFCORE-NOT-YET' in e['out'] or 'Traceback' in e['out']:
            sys.exit('%s: refcore prints REFCORE-NOT-YET or crashes: a gap in refcore, not a target' % c['id'])
        return e
    with ThreadPoolExecutor(JOBS) as ex:
        return list(ex.map(one, checks))


def sources(v):
    """{the path as the run names it: its text} of v's table."""
    return {k: open(R.ROOT + repo, encoding='utf-8').read() for k, repo in v['table']}


def flagged(v):
    """The ids of the asks a scan finds naming a world no file of v declares (a superset of those refused alone)."""
    texts = sources(v)
    declared = set(PRELUDE)
    for t in texts.values():
        declared |= set(DECLARED.findall(t))
        declared |= set(SCENARIO_HEAD.findall(t))
    out = []
    for t in texts.values():
        for aid, line in ASK_LINE.findall(t):
            if any(w not in declared for w in WORLD_WORD.findall(line)) and aid not in out:
                out.append(aid)
    return sorted(out)


def scenario_files(v):
    """{scenario id: the first full path (as the run names it, by code point) of a file declaring it}."""
    out = {}
    for k, t in sorted(sources(v).items()):
        m = re.match(r'scenario ([a-z][a-z0-9_]*) over ', t)
        if m and m.group(1) not in out:
            out[m.group(1)] = k
    return out


def predict(vs):
    """Each vector's predicted (exit, output, source), from refcore's runs."""
    probes = stored_probes()
    usage = [v for v in vs if '--ask' in v['args']]
    todo = [v for v in vs if v not in usage and v['id'] not in probes]
    for v, e in zip(todo, refcore([run_check(v, ['run'] + v['args'] + ['--ask', M0.PROBE]) for v in todo])):
        probes[v['id']] = (e['rc'], e['out'])
    loads = [v for v in vs if v not in usage and probes[v['id']][1] == M0.PROBED]
    warned = {}
    probe_tables = []
    for v in loads:
        key = os.path.relpath(R.ROOT + PROBE_FILE, R.ROOT + v['cwd']).replace(os.sep, '/')
        t = v['table'] + [(key, PROBE_FILE)]
        probe_tables.append(run_check(v, ['run'] + path_words(v['args']) + [key] + [
            a for a in v['args'] if a.startswith('--pin=')] + ['--ask', PROBE], t))
    for v, e in zip(loads, refcore(probe_tables)):
        warned[v['id']] = e
    flags = {v['id']: flagged(v) for v in loads}
    asked = [v for v in loads if flags[v['id']]]
    alone = {}
    for v, e in zip(asked, refcore([run_check(v, ['run'] + v['args'] + ['--ask'] + flags[v['id']]) for v in asked])):
        alone[v['id']] = e['out']
    out = {}
    for v in vs:
        if v in usage:
            out[v['id']] = (2, '', 'rule')
            continue
        rc, whole = probes[v['id']]
        if whole != M0.PROBED:
            if rc != 1 or not M0.REFUSED_LINE.fullmatch(whole):
                sys.exit('%s: the load probe prints neither the REQ-ASK line nor one refusal: %r' % (v['id'], whole[:200]))
            out[v['id']] = (1, whole, 'run')
            continue
        e = warned[v['id']]
        scen, warns, own = [], [], 0
        for line in e['out'].split('\n')[:-1]:
            if M0.SCENARIO_REFUSED.match(line):
                scen.append(line)
            elif line.startswith('warning: '):
                if not line.startswith('warning: WARN-UNREAD '):
                    sys.exit('%s: the warnings probe prints a warning load doesn\'t decide: %s' % (v['id'], line))
                warns.append(line)
            elif line.startswith(PROBE + ': refused: LOAD-SCENARIO ask %s names ' % PROBE):
                own += 1
            else:
                sys.exit('%s: the warnings probe prints a line it shouldn\'t: %r' % (v['id'], line))
        if own != 1 or e['rc'] != 0:
            sys.exit('%s: the warnings probe isn\'t refused alone once, exit 0 (exit %s)' % (v['id'], e['rc']))
        where = scenario_files(v)
        scen.sort(key=lambda line: (where[M0.SCENARIO_REFUSED.match(line).group(1)], line))
        refused = []
        for line in alone.get(v['id'], '').split('\n'):
            m = ALONE.match(line)
            if m:
                refused.append((m.group(1), m.group(2), int(m.group(3)), line))
        lines = scen + [r[3] for r in sorted(refused)]
        rc = 1 if lines else 0
        out[v['id']] = (rc, ''.join(line + '\n' for line in (lines or ['ok']) + warns), 'run')
    return out


def tie(vs, pred):
    """Each prediction against the stored outputs of checks.tsv that run its set; the failures, and the notes."""
    bad, notes = [], []
    for v in vs:
        rc, whole, _ = pred[v['id']]
        if whole.startswith('refused: '):
            continue
        warns = [line for line in whole.split('\n') if line.startswith('warning: WARN-UNREAD ')]
        mine = set(whole.split('\n'))
        for c in v['checks']:
            b = M0.baseline(c)
            if b is None or b.startswith('refused: '):
                continue
            theirs = [line for line in b.split('\n') if line.startswith('warning: WARN-UNREAD ')]
            household = re.match(r'I:(A|B|C|D|E1|E2|X):', c['id'])
            if theirs != warns:
                (notes if household else bad).append('%s: %s\'s stored output warns %d WARN-UNREAD lines, the '
                                                     'prediction %d' % (v['id'], c['id'], len(theirs), len(warns)))
            for _, line in M0.scenario_lines(b):
                if line not in mine:
                    bad.append('%s: %s\'s stored output holds %r, the prediction doesn\'t' % (v['id'], c['id'], line))
    return bad, notes


def stored_sources():
    """The `source` of each vector of the store as it stands: `run`, `rule` or `check`."""
    parts = netstrings(open(STORE, 'rb').read())
    return {parts[k + 2].decode('utf-8') for k in range(0, len(parts), 8)}


def stored(vs, pred):
    def ns(x):
        return M0.ns(x)
    return b''.join(ns(v['id']) + ns(v['group']) + ns(pred[v['id']][2]) + ns(v['cwd']) +
                    ns(' '.join(['check'] + v['args'])) +
                    ns(''.join('%s\t%s\n' % (k, under_tests(repo)) for k, repo in v['table'])) +
                    ns(str(pred[v['id']][0])) + ns(pred[v['id']][1]) for v in vs)


def texts_rs(vs):
    """runner/src/m3_texts.rs: every fixture text a table names that M0's embedded texts don't, by its path under
    engine/tests/, sorted (rustfmt's layout, as m0_texts.rs)."""
    m0 = open(M0.TEXTS_RS, encoding='utf-8').read() + open(R.T + 'runner/src/m0_spec_texts.rs', encoding='utf-8').read()
    paths = sorted({under_tests(repo) for v in vs for _, repo in v['table']} - set(re.findall(r'"([^"]+)",\n', m0)))
    out = ['//! The texts M3\'s `check` vectors name (`m3/check.vectors`) that `m0_texts.rs` and',
           '//! `m0_spec_texts.rs` don\'t, by their path under `engine/tests/`, sorted: generated by',
           '//! `m3/vectors.py` (its `--check` holds this file to it), so the tests read no file',
           '//! (`INTERFACE.md` §4). Never edited by hand.',
           '',
           '/// Each text: its path under `engine/tests/` and its content.',
           'pub static TEXTS: [(&str, &str); %d] = [' % len(paths)]
    for p in paths:
        out += ['    (', '        "%s",' % p, '        include_str!("../../%s"),' % p, '    ),']
    out.append('];')
    return '\n'.join(out) + '\n'


def confirm(vs, pred):
    """C4: refcore's own `check` on every vector, held to the prediction byte for byte; the store with its bytes."""
    res = refcore([run_check(v, ['check'] + v['args']) for v in vs])
    bad = []
    for v, e in zip(vs, res):
        rc, whole, src = pred[v['id']]
        if src == 'rule':
            if e['rc'] != 2:
                bad.append('%s: refcore\'s check exits %s, a usage error wants 2' % (v['id'], e['rc']))
            continue
        if (e['rc'], e['out']) != (rc, whole):
            bad.append('%s: refcore\'s check exits %s, prints %r; the prediction exits %s: %s' % (
                v['id'], e['rc'], e['out'][:300], rc, R.first_diff(whole, e['out'])))
        pred[v['id']] = (e['rc'], e['out'], 'check')
    return bad


def main(argv):
    R.need_watchdog()
    vs = vectors()
    if '--show' in argv:
        glob = argv[argv.index('--show') + 1] if argv.index('--show') + 1 < len(argv) else '*'
        vs = [v for v in vs if fnmatch.fnmatchcase(v['id'], glob)]
        pred = predict(vs)
        for v in vs:
            rc, whole, src = pred[v['id']]
            print('=== %s (exit %s, %s)\n%s' % (v['id'], rc, src, whole), end='')
        return 0
    pred = predict(vs)
    bad, notes = tie(vs, pred)
    for n in notes:
        print('note: ' + n)
    if bad:
        print('\n'.join('FAIL ' + b for b in bad))
        sys.exit('%d predictions fail their tie to checks.tsv\'s stored outputs: a STOP-REPORT, not a vector' % len(bad))
    confirmed = os.path.exists(STORE) and 'check' in stored_sources()
    if '--confirm' in argv or ('--check' in argv and confirmed):
        bad = confirm(vs, pred)
        if bad:
            print('\n'.join('FAIL ' + b for b in bad))
            sys.exit('%d vectors: refcore\'s check differs from what its run predicts: a STOP-REPORT, not a vector'
                     % len(bad))
    data, rs = stored(vs, pred), texts_rs(vs)
    counts = {}
    for v in vs:
        first = pred[v['id']][1].split('\n', 1)[0]
        kind = 'usage' if pred[v['id']][2] == 'rule' else 'ok' if first == 'ok' else \
            'base' if first.startswith('refused: ') else 'lines'
        counts[kind] = counts.get(kind, 0) + 1
    summary = ', '.join('%d %s' % (n, k) for k, n in sorted(counts.items()))
    if '--check' in argv:
        old = open(STORE, 'rb').read() if os.path.exists(STORE) else b''
        olds = open(TEXTS_RS, encoding='utf-8').read() if os.path.exists(TEXTS_RS) else ''
        if old != data or olds != rs:
            print('m3/check.vectors or runner/src/m3_texts.rs is not what refcore\'s runs give: run m3/vectors.py')
            return 1
        print('m3/check.vectors: %d vectors (%s), each predicted from refcore\'s run and tied' % (len(vs), summary))
        return 0
    with open(STORE, 'wb') as f:
        f.write(data)
    with open(TEXTS_RS, 'w', encoding='utf-8', newline='') as f:
        f.write(rs)
    print('m3/check.vectors: %d vectors written (%s); runner/src/m3_texts.rs: %d texts' % (
        len(vs), summary, rs.count('include_str!')))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
