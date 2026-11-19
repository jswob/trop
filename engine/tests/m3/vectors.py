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

**The seven changes** (OPERATIONS §6.1, MILESTONE-M3.md row 6): m3/changes/stages.tsv names each stage's files (from
engine/tests/integration/amendments/real11, the household's own files by path, T's few texts under m3/changes/): a
`b` stage is the household with only its `on` line moved (§6.1 step 1), a `c` stage the household with the change, a
`k` stage the change without its advance (load refuses it LOAD-KNOWN). A stage with a `check` kind is a `check`
vector (`changes/check/<stage>`, group `changes`, predicted as above, its kind asserted); a stage with asks is a `run`
vector (`changes/run/<stage>`, in m3/runs.vectors). Before anything is stored, every real11 file a stage names is
asserted to be the spec household's (`engine/spec/amendments/household`) but the change's own lines (`equivalence`).

**The run function** (E §1, MILESTONE-M3.md row 5): m3/run/runs.tsv, one row per vector (`run/<name>`, group `run`):
its command line, and `extra`, files put in the table that no argument names (never read).

m3/runs.vectors holds each run vector in check.vectors' eight netstrings, the argument words as given (`run` first, or
a usage error's): refcore's run (refcore_fast on PyPy, byte-equal to refcore by construction, run.py's `pypy+fast`: X's
law asks take 1-2 s there), `source` `run`, or `rule` for a usage error (refcore must exit 2; the bytes not stored).

    python3 -I -B engine/tests/m3/vectors.py              -> writes m3/check.vectors, m3/runs.vectors, m3_texts.rs
    python3 -I -B engine/tests/m3/vectors.py --check      -> exit 1 if any isn't what refcore's runs give
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
RUNS = os.path.join(HERE, 'runs.vectors')
FIXTURES = os.path.join(HERE, 'check', 'fixtures.tsv')
STAGES = os.path.join(HERE, 'changes', 'stages.tsv')
RUN_ROWS = os.path.join(HERE, 'run', 'runs.tsv')
REAL11 = 'engine/tests/integration/amendments/real11'
HOUSEHOLD = 'engine/spec/amendments/household/'
# the files a change edits (its diff's lines are its own); every other real11 file is the household's but `on`
CHANGED = {'c2/law/pl/common.tally', 'c3/law/pl/business.tally', 'c7/asks/core.tally'}
FAST = 'pypy+fast'
# predicted from refcore's `run`, which makes this load check only while answering; refcore's `check` prints the
# spec's line (L §6.4, §8.3): the coordinator's ruling, #229 (2026-11-18)
RULED_229 = {'fixture/alone/scenario-file', 'fixture/alone/same-id'}
# predicted in P5's and P12's full-path order, which the engine follows; refcore's `check` orders same-id asks
# refused alone by scenario id and places a scenario declared twice after the others: held "for #228" (held.tsv)
RULED_228 = {'fixture/reviews/same-id-order', 'fixture/reviews/twin-placed'}
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
    for s in stage_rows():
        if s['check'] != '-':
            args = s['files'].split()
            out.append({'id': 'changes/check/' + s['stage'], 'group': 'changes', 'cwd': REAL11, 'args': args,
                        'table': table_of(s['stage'], REAL11, args), 'checks': [], 'kind': s['check']})
    ids = [v['id'] for v in out]
    if len(set(ids)) != len(ids):
        sys.exit('two vectors share an id')
    return out


def tsv(path, columns):
    with open(path, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != columns:
            sys.exit('%s: the columns are %s, not %s' % (path, r.fieldnames, columns))
        return list(r)


def stage_rows():
    """m3/changes/stages.tsv: stage, the asks its run asks (`-`: no run), its check's kind (`ok`, `known`, `-`), files."""
    rows = tsv(STAGES, ['stage', 'run', 'check', 'files'])
    for s in rows:
        if s['check'] not in ('ok', 'known', '-') or (s['run'] == '-' and s['check'] == '-'):
            sys.exit('stages.tsv: %s: a check kind ok, known or -, and a run or a check' % s['stage'])
    return rows


def equivalence():
    """Every real11 file a stage names is the spec household's, but the change's own: a file of its change (absent from
    the household, or in CHANGED), or reality.tally with its `on` line moved; T's stage-a texts are the first lines
    of their stage-b files, and T's headers the household's with `on` moved."""
    def lines(repo):
        return open(R.ROOT + repo, encoding='utf-8').read().split('\n')

    def on_moved(a, b):
        return len(a) == len(b) and [i for i, (x, y) in enumerate(zip(a, b)) if x != y] in ([], [2]) and \
            b[2].startswith('  on ')
    bad = []
    for s in stage_rows():
        for _, repo in table_of(s['stage'], REAL11, s['files'].split()):
            rel = repo[len(REAL11) + 1:] if repo.startswith(REAL11 + '/') else None
            if rel is None:
                continue
            house = HOUSEHOLD + rel.split('/', 1)[1]
            if rel in CHANGED or not os.path.exists(R.ROOT + house):
                continue
            if rel.endswith('/reality.tally') and rel.count('/') == 1:
                ok = on_moved(lines(house), lines(repo))
            else:
                ok = lines(house) == lines(repo)
            if not ok:
                bad.append('%s: %s is not the household\'s %s' % (s['stage'], rel, house))
    for mine, theirs in (('c4a/facts/2028.tally', 'c4/facts/2028.tally'), ('c6a/facts/2027.tally', 'c6/facts/2027.tally')):
        a, b = lines(PREFIX + 'm3/changes/' + mine), lines(REAL11 + '/' + theirs)
        if a[:-1] != b[:len(a) - 1]:
            bad.append('m3/changes/%s is not the first lines of %s' % (mine, theirs))
    for d in ('on-2028-05-14', 'on-2027-04-02'):
        if not on_moved(lines(HOUSEHOLD + 'reality.tally'), lines(PREFIX + 'm3/changes/%s/reality.tally' % d)) or \
                lines(PREFIX + 'm3/changes/%s/reality.tally' % d)[2] != '  on ' + d[3:]:
            bad.append('m3/changes/%s/reality.tally is not the household\'s with on %s' % (d, d[3:]))
    if bad:
        sys.exit('\n'.join(bad))


def run_vectors():
    """The seven changes' runs (stages with asks), then m3/run/runs.tsv's, each {id, group, cwd, args, table, rule,
    runtime}."""
    out = []
    for s in stage_rows():
        if s['run'] != '-':
            files = s['files'].split()
            out.append({'id': 'changes/run/' + s['stage'], 'group': 'changes', 'cwd': REAL11,
                        'args': ['run'] + files + ['--ask'] + s['run'].split(),
                        'table': table_of(s['stage'], REAL11, files), 'rule': ''})
    for r in tsv(RUN_ROWS, ['id', 'cwd', 'args', 'extra', 'rule']):
        cwd = os.path.normpath(PREFIX + r['cwd'])
        args = r['args'].split()
        table = table_of(r['id'], cwd, args[1:]) + [
            (k, os.path.normpath(os.path.join(cwd, k))) for k in r['extra'].split() if k != '-']
        out.append({'id': r['id'], 'group': r['id'].split('/')[0], 'cwd': cwd, 'args': args, 'table': table,
                    'rule': r['rule'] if r['rule'] == 'usage' else ''})
    return out


def runs_of(rvs):
    """Refcore's run of each run vector: {id: (exit, output, source)}; a usage error must exit 2, its words not kept."""
    res = refcore([run_check(v, v['args']) for v in rvs], FAST)
    out = {}
    for v, e in zip(rvs, res):
        if v['rule']:
            if e['rc'] != 2:
                sys.exit('%s: refcore exits %s on a usage error, not 2' % (v['id'], e['rc']))
            out[v['id']] = (2, '', 'rule')
        else:
            out[v['id']] = (e['rc'], e['out'], 'run')
    return out


def under_tests(repo):
    """A table file's path as the store names it: under engine/tests/ (a spec text as ../spec/…)."""
    return repo[len(PREFIX):] if repo.startswith(PREFIX) else '../' + repo[len('engine/'):]


def run_check(v, words, table=None):
    """A run.py check that runs refcore with `words` (the argv after the program name) on v's files."""
    table = v['table'] if table is None else table
    return {'id': v['id'], 'layer': 'interpreter', 'cwd': v['cwd'], 'asks': '', 'baseline': '', 'block': '',
            'note': '', 'baseline_s': '', 'files': ' '.join(repo for _, repo in table), 'options': ' '.join(words)}


def refcore(checks, runtime='pypy'):
    rc_hash = R.refcore_hash()

    def one(c):
        e = R.run_one(c, runtime, R.BUDGET, False, rc_hash, R.T + 'cache')
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


def stored(vs, pred, command=('check',)):
    def ns(x):
        return M0.ns(x)
    return b''.join(ns(v['id']) + ns(v['group']) + ns(pred[v['id']][2]) + ns(v['cwd']) +
                    ns(' '.join(list(command) + v['args'])) +
                    ns(''.join('%s\t%s\n' % (k, under_tests(repo)) for k, repo in v['table'])) +
                    ns(str(pred[v['id']][0])) + ns(pred[v['id']][1]) for v in vs)


def kinds(vs, pred):
    """A stage's check kind holds: `ok` prints `ok` first, `known` load's one LOAD-KNOWN refusal."""
    for v in vs:
        rc, whole, _ = pred[v['id']]
        if v.get('kind') == 'ok' and not whole.startswith('ok\n'):
            sys.exit('%s: the stage should load (`ok`), the prediction is %r' % (v['id'], whole[:300]))
        if v.get('kind') == 'known' and not (rc == 1 and whole.startswith('refused: LOAD-KNOWN ')):
            sys.exit('%s: the stage without its advance should be refused LOAD-KNOWN, not %r' % (v['id'], whole[:300]))


def texts_rs(vs):
    """runner/src/m3_texts.rs: every fixture text a table names that M0's embedded texts don't, by its path under
    engine/tests/, sorted (rustfmt's layout, as m0_texts.rs)."""
    m0 = open(M0.TEXTS_RS, encoding='utf-8').read() + open(R.T + 'runner/src/m0_spec_texts.rs', encoding='utf-8').read()
    paths = sorted({under_tests(repo) for v in vs for _, repo in v['table']} - set(re.findall(r'"([^"]+)",\n', m0)))
    out = ['//! The texts M3\'s vectors name (`m3/check.vectors`, `m3/runs.vectors`) that `m0_texts.rs` and',
           '//! `m0_spec_texts.rs` don\'t, by their path under `engine/tests/`, sorted: generated by',
           '//! `m3/vectors.py` (its `--check` holds this file to it), so the tests read no file',
           '//! (`INTERFACE.md` §4). Never edited by hand.',
           '',
           '/// Each text: its path under `engine/tests/` and its content.',
           'pub static TEXTS: [(&str, &str); %d] = [' % len(paths)]
    for p in paths:
        inner = '"%s", include_str!("../../%s")' % (p, p)
        if len(inner) <= 60:  # rustfmt's width for a tuple on one line (fn_call_width)
            out.append('    (%s),' % inner)
        else:
            out += ['    (', '        "%s",' % p, '        include_str!("../../%s"),' % p, '    ),']
    out.append('];')
    return '\n'.join(out) + '\n'


def confirm(vs, pred):
    """C4: refcore's own `check` on every vector, held to the prediction byte for byte; the store with its bytes.
    Returns the differences and the notes: a vector of RULED_229 may differ (refcore's `run`, which predicted it, makes
    that load check only while answering; its `check` prints the spec's line: #229), its difference a note."""
    res = refcore([run_check(v, ['check'] + v['args']) for v in vs])
    bad, notes = [], []
    for v, e in zip(vs, res):
        rc, whole, src = pred[v['id']]
        if src == 'rule':
            if e['rc'] != 2:
                bad.append('%s: refcore\'s check exits %s, a usage error wants 2' % (v['id'], e['rc']))
            continue
        if (e['rc'], e['out']) != (rc, whole):
            (notes if v['id'] in RULED_229 | RULED_228 else bad).append(
                '%s: refcore\'s check exits %s, prints %r; the prediction exits %s: %s' % (
                    v['id'], e['rc'], e['out'][:300], rc, R.first_diff(whole, e['out'])))
        pred[v['id']] = (e['rc'], e['out'], 'check')
    return bad, notes


def load_at_run(vs, pred):
    """The sets `check` answers `ok` whose stored run (a baseline of checks.tsv) prints a LOAD- refusal: refcore finds
    those codes only while evaluating (`ask.LoadAtRun`, #229), so `check`'s `ok` there holds only beside that run's
    vector (the coordinator's word, 2026-11-18). One line per (vector, check)."""
    out = []
    for v in vs:
        if not pred[v['id']][1].startswith('ok\n'):
            continue
        for c in v['checks']:
            b = M0.baseline(c)
            m = re.search(r'^(?:[a-z][a-z0-9_.]*: )?refused: LOAD-[A-Z]+.*$', b or '', re.M)
            if m:
                out.append('%s: %s prints %s' % (v['id'], c['id'], m.group(0)[:160]))
    return out


def main(argv):
    R.need_watchdog()
    equivalence()
    vs, rvs = vectors(), run_vectors()
    if '--show' in argv:
        glob = argv[argv.index('--show') + 1] if argv.index('--show') + 1 < len(argv) else '*'
        vs = [v for v in vs if fnmatch.fnmatchcase(v['id'], glob)]
        rvs = [v for v in rvs if fnmatch.fnmatchcase(v['id'], glob)]
        pred = dict(predict(vs), **runs_of(rvs))
        for v in vs + rvs:
            rc, whole, src = pred[v['id']]
            print('=== %s (exit %s, %s)\n%s' % (v['id'], rc, src, whole), end='')
        return 0
    pred = predict(vs)
    kinds(vs, pred)
    runs = runs_of(rvs)
    bad, notes = tie(vs, pred)
    for n in notes:
        print('note: ' + n)
    if bad:
        print('\n'.join('FAIL ' + b for b in bad))
        sys.exit('%d predictions fail their tie to checks.tsv\'s stored outputs: a STOP-REPORT, not a vector' % len(bad))
    confirmed = os.path.exists(STORE) and 'check' in stored_sources()
    if '--confirm' in argv or confirmed:  # once confirmed, every write and --check runs refcore's check again
        bad, ruled = confirm(vs, pred)
        for n in ruled:
            print('ruled (#228, #229): ' + n)
        for n in load_at_run(vs, pred):
            print('load-at-run (#229): ' + n)
        if bad:
            print('\n'.join('FAIL ' + b for b in bad))
            sys.exit('%d vectors: refcore\'s check differs from what its run predicts: a STOP-REPORT, not a vector'
                     % len(bad))
    data, rdata, rs = stored(vs, pred), stored(rvs, runs, ()), texts_rs(vs + rvs)
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
        oldr = open(RUNS, 'rb').read() if os.path.exists(RUNS) else b''
        if old != data or olds != rs or oldr != rdata:
            print('m3/check.vectors, m3/runs.vectors or runner/src/m3_texts.rs is not what refcore\'s runs give: run '
                  'm3/vectors.py')
            return 1
        print('m3/check.vectors: %d vectors (%s), each %s; m3/runs.vectors: %d vectors, refcore\'s runs' % (
            len(vs), summary, 'refcore\'s check, tied to its run\'s prediction' if confirmed else
            'predicted from refcore\'s run and tied', len(rvs)))
        return 0
    with open(STORE, 'wb') as f:
        f.write(data)
    with open(RUNS, 'wb') as f:
        f.write(rdata)
    with open(TEXTS_RS, 'w', encoding='utf-8', newline='') as f:
        f.write(rs)
    print('m3/check.vectors: %d vectors written (%s); m3/runs.vectors: %d; runner/src/m3_texts.rs: %d texts' % (
        len(vs), summary, len(rvs), rs.count('include_str!')))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
