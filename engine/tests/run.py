"""The test runner for the interpreter, unit and integration layers (session 4.5a). Standard library only.

Runs the checks of engine/tests/checks.tsv on the frozen engine/tools/refcore, unchanged, and compares each output
with its stored baseline. Form only: it runs and compares; it never explains a difference.

    python3 -I -B engine/tests/run.py [ID_GLOB ...] [--layer L[,L]] [--runtime pypy|cpython|both] [--jobs N<=4]
                                      [--budget S] [--force] [--list] [--interp refcore|fast|both|engine] [--verify N]
    python3 -I -B engine/tests/run.py --self-test
    python3 -I -B engine/tests/run.py --tables     -> reports/before-after.md, over-budget.md (from results/pypy.tsv)
    python3 -I -B engine/tests/run.py --adhoc [--cwd DIR] -- run <files> [--ask a ...]   one 90 s PyPy run, logged

- Paths (4.5f): every path in checks.tsv is relative to the repository root, which run.py finds from its own place,
  so the suite runs from any checkout. A check runs in its `cwd`, and its files are passed relative to that cwd, so
  a diagnostic that prints a source path prints the same bytes in every checkout.

- Selection: id globs (fnmatch) and/or layers; none selects every check. Rows whose note starts `not-run:` (runs on
  load/benchmarks.md) are never run; they are counted as not run with their reason.
- Budget (the user's rule, 4.5f): 90 s per run, hard. Past it the run is killed and recorded as `over`; an `over` is
  cached like any result, so it is not rerun (unless --force or a larger --budget). 60 s is the soft line: a check
  that passes it is counted `slow`, and one over 60 s in 3 uncached runs in a row is listed after the table as a
  candidate to move to load or to cut down (results/<variant>.tsv, column slow_runs).
- The cache (cache/<key>.json: output, seconds, rc) is keyed by sha256 over: the runtime and its version; the refcore
  tree (its lines of engine/tools/SHA256SUMS, checked against the files at start); each input file's path (repository-
  relative) and bytes in run order; the asks; the options, the mode and the cwd (repository-relative). A hit is reported `cached` and not rerun.
- Compare: a unit, model, light or hardset check's whole output (stdout then stderr, as run_set.py stored it) equals
  its baseline file; a household check (I:<H>:<ask>) is compared in run5.py's form: the ask's blocks (WARN-UNREAD
  and warning lines, '#' lines and blank lines dropped, lines right-stripped) plus the baseline's blocks no ask owns.
  R:all is `same` when the last line reads '950 tests, 950 passed, 0 failed'.
- Known faults (known-faults.md): a check whose note starts `xfail:` is expected to differ; it reports `known` while
  it does and `fixed` once it matches its baseline. Its guarded twin (note `known-fault:`) runs as usual.
- `both` runs each check on PyPy and on CPython and also requires the two outputs to be equal.
- --interp (4.5e): `fast` (engine/tools/refcore_fast, the memory layer in front of the same refcore; the default
  since 4.5f), `refcore`, or `both`, which runs each check on the two and requires equal outputs, as --runtime both
  does. Under `fast`, a check with no stored baseline (verdict `ran`) runs on refcore: a new output comes from refcore
  only (TESTING.md §5).
- The watchdog (4.5f): a run, --self-test and --adhoc refuse to start unless engine/tests/tools/memcap.py is running
  (it caps every refcore and refcore_fast run at 8 GB). A run is a
  variant `<runtime>[+fast[+v<N>]]`: --verify N runs the fast side with `--verify N`. The fast variants' cache keys add
  the variant and the hash of refcore_fast's files; the plain runtimes' keys are unchanged.
- --interp engine (T1, issue #5; INTERFACE.md): the engine's release binary engine/target/release/tally, with the same
  argv (argv_of), from the check's cwd, under the same budget and watchdog. Its cache key is the binary's sha256 instead
  of the runtime's and refcore's. An engine output never becomes a baseline: a check with no stored baseline is
  compared with refcore's own run of it from the cache (an error until refcore has run it), never `ran`. An exit code
  other than INTERFACE §3's (1 when the first line is a refusal, else 0) is an `error`, whatever the output. R:all
  (refcore's own tests) and the key scripts run no engine: they are listed as not run, with the load rows.
- Results: results/<runtime>.tsv (the latest result per id); a differing output is kept in out/<runtime>/.
"""
import csv
import fnmatch
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))) + '/'
T = ROOT + 'engine/tests/'
RC = ROOT + 'engine/tools/refcore'
FAST = ROOT + 'engine/tools/refcore_fast'
ENGINE = [ROOT + 'engine/target/release/tally']  # a list, so the self-test can put a stand-in in its place
SUMS = ROOT + 'engine/tools/SHA256SUMS'
PY = {'pypy': os.path.expanduser('~/.local/share/uv/python/pypy-3.11.13-macos-aarch64-none/bin/pypy3.11'),
      'cpython': ROOT + 'engine/tools/parsecheck/.venv/bin/python'}  # the python the snap16 baselines ran on
MAX_JOBS = 4
BUDGET = 90.0  # the hard budget per run (the user's rule, 4.5f; 60 s before)
SLOW = 60.0  # the soft line: a check past it is slow
SLOW_RUNS = 3  # slow in this many uncached runs in a row: a candidate for load or reduction
TESTS_LINE = '1115 tests, 1115 passed, 0 failed'  # refcore for tally 2.1 after session 7c's fixes; 1105 at tally-2.1 (7b), 950 on 2.0
LAYERS = ('interpreter', 'unit', 'integration')
LEFT_OUT = re.compile(r'^\s+left out: ')
VERDICTS = ('same', 'differ', 'over', 'killed', 'error', 'known', 'fixed', 'ran')


# -- the checks ------------------------------------------------------------------------------------------------------

def load_checks(path=T + 'checks.tsv'):
    with open(path, encoding='utf-8', newline='') as f:
        return list(csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE))


def household(c):
    return c['block'] != ''


def script(c):
    """A key script check (4.5c, keys-extra/): `files` is the script then the inputs it reads (hashed for the cache),
    `asks` its arguments; it passes when it exits 0 and its last line ends ': 0 failed'."""
    return c['options'] == 'script'


def variant(v):
    """'pypy', 'cpython+fast', 'pypy+fast+v50', 'pypy+fast+v1+plant' (the self-test's) -> (runtime, interp argv)."""
    parts = v.split('+')
    if 'fast' not in parts:
        return parts[0], [RC]
    a = [FAST]
    for p in parts[2:]:
        a += ['--verify', p[1:]] if p.startswith('v') else ['--plant']
    return parts[0], a


def here(c, f):
    """A repository-relative path as the check's cwd sees it."""
    return os.path.relpath(ROOT + f, ROOT + c['cwd'])


def argv_of(c):
    if script(c):
        return [here(c, f) for f in c['files'].split()[:1]] + c['asks'].split()
    if c['layer'] == 'interpreter':
        return c['options'].split()
    a = ['run'] + [here(c, f) for f in c['files'].split()] + c['options'].split()
    return a + (['--ask'] + c['asks'].split() if c['asks'] else [])


# -- the cache key ---------------------------------------------------------------------------------------------------

_VERSION = {}


def version(rt):
    if rt not in _VERSION:
        _VERSION[rt] = subprocess.run([PY[rt], '-I', '-B', '-c', 'import sys; print(sys.version)'], capture_output=True,
                                      text=True, check=True).stdout.strip()
    return _VERSION[rt]


def refcore_hash():
    """sha256 over refcore's lines of SHA256SUMS, after checking that every listed file matches and none is unlisted."""
    lines = [l for l in open(SUMS, encoding='utf-8') if l.split(None, 1)[1].startswith('refcore/')]
    base = os.path.dirname(SUMS)
    listed = set()
    for l in lines:
        h, p = l.rstrip('\n').split(None, 1)
        listed.add(p)
        if hashlib.sha256(open(os.path.join(base, p), 'rb').read()).hexdigest() != h:
            sys.exit('refcore/%s differs from engine/tools/SHA256SUMS: stop' % p)
    for d, dirs, fs in os.walk(RC):
        dirs[:] = [x for x in dirs if x != '__pycache__']
        for f in fs:
            p = os.path.relpath(os.path.join(d, f), base)
            if p not in listed and not f.endswith('.pyc') and f != '.DS_Store':
                sys.exit('%s is not in engine/tools/SHA256SUMS: stop' % p)
    return hashlib.sha256(''.join(lines).encode()).hexdigest()


def fast_hash(d=FAST):
    """sha256 over refcore_fast's files, path and bytes."""
    h = hashlib.sha256()
    for dd, dirs, fs in sorted(os.walk(d)):
        dirs[:] = sorted(x for x in dirs if x != '__pycache__')
        for f in sorted(fs):
            if not f.endswith('.pyc') and f != '.DS_Store':
                p = os.path.join(dd, f)
                h.update(os.path.relpath(p, d).encode() + b'\0' + open(p, 'rb').read() + b'\0')
    return h.hexdigest()


_ENGINE_HASH = {}


def engine_hash():
    """sha256 of the engine's binary, read once per path and size and mtime (the cache key's interpreter part)."""
    p = ENGINE[0]
    if not os.path.isfile(p):
        sys.exit('%s is not built: the build role builds it (engine/tools/check, or cargo build --release '
                 '-p tally-cli), then run again' % os.path.relpath(p, ROOT))
    st = os.stat(p)
    k = (p, st.st_size, st.st_mtime_ns)
    if k not in _ENGINE_HASH:
        _ENGINE_HASH[k] = hashlib.sha256(open(p, 'rb').read()).hexdigest()
    return _ENGINE_HASH[k]


def cache_key(c, v, rc_hash):
    h = hashlib.sha256()

    def part(b):
        b = b if isinstance(b, bytes) else b.encode('utf-8')
        h.update(b'%d:' % len(b))
        h.update(b)
    if v == 'engine':
        part('engine')
        part(engine_hash())
    else:
        rt, interp = variant(v)
        part(rt)
        part(version(rt))
        part(rc_hash)
    if v != 'engine' and interp != [RC]:
        part(v)
        part(fast_hash())
        part(' '.join(interp))
    for f in c['files'].split():
        part(f)
        part(open(ROOT + f, 'rb').read())
    part(c['asks'])
    part(c['options'])
    part(' '.join(argv_of(c)))
    part(c['cwd'])
    return h.hexdigest()


# -- one run ---------------------------------------------------------------------------------------------------------

def text(b):
    """Bytes as subprocess's text mode gives them (UTF-8, universal newlines)."""
    return b.decode('utf-8', 'surrogateescape').replace('\r\n', '\n').replace('\r', '\n')


def execute(c, v, budget):
    env = {k: x for k, x in os.environ.items() if k != 'REFCORE_DEBUG'}
    if v == 'engine':
        cmd = ENGINE + argv_of(c)
    else:
        rt, interp = variant(v)
        cmd = [PY[rt], '-I', '-B'] + ([] if script(c) else interp) + argv_of(c)
    t0 = time.time()
    p = subprocess.Popen(cmd, cwd=ROOT + c['cwd'], stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, env=env, start_new_session=True)
    try:
        out, err = p.communicate(timeout=budget)
        return {'out': text(out) + text(err), 'seconds': round(time.time() - t0, 2), 'rc': p.returncode, 'over': False}
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):  # the group already gone (macOS answers EPERM)
            pass
        p.communicate()
        return {'out': '', 'seconds': round(time.time() - t0, 2), 'rc': None, 'over': True}


def run_one(c, rt, budget, force, rc_hash, cache_dir):
    key = cache_key(c, rt, rc_hash)
    path = os.path.join(cache_dir, key[:2], key + '.json')
    if not force and os.path.exists(path):
        e = json.load(open(path, encoding='utf-8'))
        if not e['over'] or e['budget'] >= budget:
            e['cached'] = True
            return e
    e = execute(c, rt, budget)
    e.update(id=c['id'], runtime=rt, version=engine_hash() if rt == 'engine' else version(variant(rt)[0]), key=key,
             budget=budget, when=time.strftime('%Y-%m-%d %H:%M:%S'))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + '.tmp%d' % os.getpid()
    with open(tmp, 'w', encoding='utf-8') as f:
        json.dump(e, f)
    os.replace(tmp, path)
    e['cached'] = False
    return e


# -- compare ---------------------------------------------------------------------------------------------------------

def blocks(txt):
    """run5.py's blocks(), on a text: each unindented line opens a block named by what precedes its first ':'."""
    out, cur = {}, None
    for l in txt.splitlines(True):
        if l.startswith(('#', 'rc=', 'time ', 'exit', 'real', 'user', 'sys')) or 'WARN-UNREAD' in l or not l.strip():
            continue
        if not l.startswith(' ') and not l.startswith('warning'):
            cur = l.split(':')[0]
            out.setdefault(cur, [])
        if cur is not None and not l.startswith('warning'):
            out[cur].append(l.rstrip())
    return out


def owner(k):
    m = re.match(r'[A-Za-z_][A-Za-z0-9_]*', k)
    return m.group(0) if m else ''


_RUN_ASKS = {}


def run_asks(c, checks):
    """The asks of the run a household check was cut from: those of every check on its baseline."""
    if c['baseline'] not in _RUN_ASKS:
        _RUN_ASKS[c['baseline']] = {x['block'] for x in checks if x['baseline'] == c['baseline']}
    return _RUN_ASKS[c['baseline']]


def expected_blocks(c, checks):
    """The baseline's blocks this ask owns, plus those no ask of its run owns."""
    base = blocks(open(ROOT + c['baseline'], encoding='utf-8', errors='replace').read())
    asks = run_asks(c, checks)
    return {k: v for k, v in base.items() if owner(k) == c['block'] or owner(k) not in asks}


def stored_s(c):
    """The CPython seconds stored for a check, or for its whole run; 0 when none."""
    m = re.search(r'run_cpython_s=(\d+)', c['note'])
    return float(c['baseline_s'] or (m.group(1) if m else 0))


def first_diff(a, b):
    al, bl = a.split('\n'), b.split('\n')
    for i, (x, y) in enumerate(zip(al, bl)):
        if x != y:
            return 'line %d: %r != %r' % (i + 1, x[:70], y[:70])
    return 'length %d != %d lines' % (len(al), len(bl))


def compare(c, out, checks):
    """('same' | 'differ', detail)."""
    if c['layer'] == 'interpreter':
        last = out.rstrip('\n').split('\n')[-1]
        return ('same' if last == TESTS_LINE else 'differ'), last
    if script(c):
        last = out.rstrip('\n').split('\n')[-1]
        return ('same' if last.endswith(': 0 failed') else 'differ'), last
    if household(c):
        asks = run_asks(c, checks)
        exp = expected_blocks(c, checks)
        got = {k: v for k, v in blocks(out).items() if owner(k) == c['block'] or owner(k) not in asks}
        if 'cmp=no-left-out' in c['note']:  # a run-end cut (4.5c): the `left out:` lines are the run end's own register
            exp, got = ({k: [l for l in v if not LEFT_OUT.match(l)] for k, v in d.items()} for d in (exp, got))
        if exp == got:
            return 'same', '%d blocks' % len(exp)
        ks = sorted(k for k in set(exp) | set(got) if exp.get(k) != got.get(k))
        return 'differ', 'blocks differ: ' + '; '.join(k[:50] for k in ks[:5])
    base = open(ROOT + c['baseline'], encoding='utf-8', errors='surrogateescape').read()
    return ('same', '') if out == base else ('differ', first_diff(base, out))


def input_age(c):
    """Rule 5: does any input postdate the baseline?"""
    if not c['files'] or not os.path.exists(ROOT + c['baseline']):
        return ''
    b = os.path.getmtime(ROOT + c['baseline'])
    newer = [f for f in c['files'].split() if os.path.getmtime(ROOT + f) > b]
    return ('INPUT NEWER THAN BASELINE: ' + ' '.join(newer[:3])) if newer else 'inputs older than the baseline'


# -- a session -------------------------------------------------------------------------------------------------------

RES_COLS = ['id', 'layer', 'runtime', 'verdict', 'seconds', 'rc', 'cached', 'budget', 'detail', 'when', 'slow_runs']


def slow_runs(prev, row):
    """The uncached runs in a row past SLOW, this one included: a cached result keeps the count, a run under resets it."""
    n = int(prev.get('slow_runs') or 0) if prev else 0
    if row['cached']:
        return n
    return n + 1 if row['verdict'] not in ('over', 'killed') and float(row['seconds']) > SLOW else 0


WATCHDOG = re.compile(r'(^|[\s/])engine/tests/tools/memcap\.py(\s|$)')


def watchdog(ps=None):
    """The pid of a running engine/tests/tools/memcap.py, or None."""
    if ps is None:
        ps = subprocess.run(['ps', '-axo', 'pid=,command='], capture_output=True, text=True).stdout
    for l in ps.splitlines():
        pid, _, cmd = l.strip().partition(' ')
        if WATCHDOG.search(cmd):
            return int(pid)
    return None


def need_watchdog():
    if watchdog() is None:
        sys.exit('the memory watchdog is not running: start it with\n'
                 '  nohup python3 -I -B engine/tests/tools/memcap.py 8 5 > /dev/null 2>&1 &\nand run again')


def own_variant(c, v):
    """U2 (4.5f): a check with no stored baseline runs on refcore, whatever variant was asked."""
    return v.split('+')[0] if '+fast' in v and not c['baseline'] and not script(c) else v


def save_results(rt, rows):
    path = T + 'results/%s.tsv' % rt
    old = {}
    if os.path.exists(path):
        with open(path, encoding='utf-8', newline='') as f:
            old = {r['id']: r for r in csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)}
    for r in rows:
        r['slow_runs'] = slow_runs(old.get(r['id']), r)
    old.update({r['id']: r for r in rows})
    os.makedirs(T + 'results', exist_ok=True)
    with open(path + '.tmp', 'w', encoding='utf-8') as f:
        f.write('\t'.join(RES_COLS) + '\n')
        for r in sorted(old.values(), key=lambda r: r['id']):
            f.write('\t'.join(str(r.get(k, '')).replace('\t', ' ').replace('\n', ' ') for k in RES_COLS) + '\n')
    os.replace(path + '.tmp', path)
    return old


def engine_exit(e):
    """INTERFACE §3: exit 1 when the first line starts `refused: `, else 0. Anything else (the stub's 69, a crash's 101
    or 134) is an error, '' when the exit is right."""
    want = 1 if e['out'].startswith('refused: ') else 0
    if e['rc'] == want:
        return ''
    first = e['out'].split('\n', 1)[0][:70]
    return 'exit %s, not %d: %r' % (e['rc'], want, first)


def refcore_output(c, cache_dir):
    """refcore's own run of a check from the cache (the comparator for an engine run of a check with no baseline):
    the plain refcore variant first, as own_variant runs such checks; None when neither is cached."""
    rc_hash = refcore_hash()
    for v in ('pypy', 'pypy+fast'):
        k = cache_key(c, v, rc_hash)
        p = os.path.join(cache_dir, k[:2], k + '.json')
        if os.path.exists(p):
            e = json.load(open(p, encoding='utf-8'))
            if not e['over'] and (e['rc'] is None or e['rc'] >= 0):
                return e['out']
    return None


def not_run(c, v):
    """Why a check isn't run on this variant, or ''."""
    if c['note'].startswith('not-run'):
        return 'a load row (not-run:)'
    if v == 'engine' and c['layer'] == 'interpreter':
        return "refcore's own tests: the engine runs nothing in them"
    if v == 'engine' and script(c):
        return 'a key script: it runs no engine'
    return ''


def verdict_of(c, e, checks, cache_dir=None):
    if e['over']:
        return 'over', 'past %g s' % e['budget']
    if e['rc'] is not None and e['rc'] < 0:
        return 'killed', 'signal %d (the memory watchdog?)' % -e['rc']
    if e.get('runtime') == 'engine':
        bad = engine_exit(e)
        if bad:
            return 'error', bad
        if not c['baseline']:  # never `ran`: an engine output never becomes a baseline; refcore's own run judges it
            ref = refcore_output(c, cache_dir or T + 'cache')
            if ref is None:
                return 'error', 'no stored baseline, and refcore\'s run of it is not in the cache: run it on refcore first'
            return ('same', 'equal to refcore\'s run (no stored baseline)') if e['out'] == ref else (
                'differ', 'refcore\'s run (no stored baseline): ' + first_diff(ref, e['out']))
    if not c['baseline']:  # a capability check with no byte-equal judged twin (4.5c): J marks its kept output
        return 'ran', 'no twin: marked by J against the key'
    v, d = compare(c, e['out'], checks)
    if c['note'].startswith('xfail:'):
        return ('known', d) if v == 'differ' else ('fixed', 'now matches its baseline: ' + c['note'].split(',')[0])
    if v == 'differ':
        d += ' | ' + input_age(c)
    return v, d


def session(sel, checks, runtimes, jobs, budget, force, cache_dir=T + 'cache', out_dir=T + 'out', quiet=False,
            save=True):
    rc_hash = refcore_hash()
    rows_by_rt = {rt: [] for rt in runtimes}
    table = []
    t_all = time.time()
    why_not = {}
    for layer in LAYERS:
        cs = [c for c in sel if c['layer'] == layer]
        if not cs:
            continue
        todo = [c for c in cs if not not_run(c, runtimes[0])]
        skipped = [c for c in cs if not_run(c, runtimes[0])]
        for c in skipped:
            why_not.setdefault((layer, not_run(c, runtimes[0])), []).append(c['id'])
        # the slow ones first, so the tail of the pool is short
        todo.sort(key=lambda c: -stored_s(c))
        t0 = time.time()
        work = [(c, rt) for c in todo for rt in runtimes]
        with ThreadPoolExecutor(jobs) as ex:
            res = list(ex.map(lambda w: (w[0], w[1], run_one(w[0], own_variant(w[0], w[1]), budget, force, rc_hash,
                                                               cache_dir)), work))
        wall = time.time() - t0
        counts = {rt: dict.fromkeys(VERDICTS + ('cached', 'both-differ', 'slow'), 0) for rt in runtimes}
        outs = {}
        for c, rt, e in res:
            v, d = verdict_of(c, e, checks, cache_dir)
            counts[rt][v] += 1
            counts[rt]['cached'] += e['cached']
            counts[rt]['slow'] += v not in ('over', 'killed') and e['seconds'] > SLOW
            outs[(c['id'], rt)] = e
            if v in ('differ', 'error', 'killed', 'fixed', 'ran'):
                od = os.path.join(out_dir, rt)
                os.makedirs(od, exist_ok=True)
                open(os.path.join(od, c['id'].replace('/', '__').replace(':', '_') + '.out'), 'w',
                     encoding='utf-8', errors='surrogateescape').write(e['out'])
                if not quiet and v != 'ran':
                    print('%s %s %s: %s' % (v.upper(), rt, c['id'], d))
            rows_by_rt[rt].append({'id': c['id'], 'layer': layer, 'runtime': rt, 'verdict': v, 'seconds': e['seconds'],
                                   'rc': e['rc'], 'cached': 'cached' if e['cached'] else '', 'budget': e['budget'],
                                   'detail': d, 'when': e['when']})
        if len(runtimes) == 2:
            for c in todo:
                a, b = outs[(c['id'], runtimes[0])], outs[(c['id'], runtimes[1])]
                if a['over'] or b['over']:
                    continue
                if a['out'] != b['out'] or a['rc'] != b['rc']:
                    counts[runtimes[0]]['both-differ'] += 1
                    if not quiet:
                        print('BOTH-DIFFER %s: %s' % (c['id'], first_diff(a['out'], b['out'])))
        for rt in runtimes:
            k = counts[rt]
            table.append((layer, rt, len(cs), k['same'], k['differ'], k['error'], k['over'], k['slow'], k['killed'], k['known'], k['fixed'],
                          k['ran'], k['cached'], len(skipped),
                          wall, k['both-differ'] if len(runtimes) == 2 and rt == runtimes[0] else ''))
    candidates = []
    if save:
        for rt in runtimes:
            allr = save_results(rt, rows_by_rt[rt])
            candidates += ['%s (%s)' % (i, rt) for i, r in sorted(allr.items()) if int(r.get('slow_runs') or 0) >= SLOW_RUNS]
    lines = ['| layer | runtime | checks | same | differ | error | over | slow (> %g s) | killed | known fault | fixed | ran (J marks) '
             '| cached | not run | wall s | %s |' % (SLOW, '≠'.join(runtimes if len(runtimes) == 2 else ['pypy', 'cpython'])),
             '|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|']
    for r in table:
        lines.append('| %s | %s | %d | %d | %d | %d | %d | %d | %d | %d | %d | %d | %d | %d | %.0f | %s |' % r)
    lines.append('')
    lines.append('total wall %.0f s, %s, budget %g s, jobs %d' % (time.time() - t_all, time.strftime('%Y-%m-%d %H:%M'),
                                                                   budget, jobs))
    if candidates:
        lines.append('candidates for load or reduction (over %g s in %d uncached runs in a row): %s' % (
            SLOW, SLOW_RUNS, ' '.join(candidates)))
    for (layer, why), ids in sorted(why_not.items()):
        lines.append('not run (%s, %d): %s' % (layer, len(ids), why) + (': ' + ' '.join(ids) if len(ids) <= 3 else ''))
    if not quiet:
        print('\n'.join(lines))
    return table, rows_by_rt, lines


def select(checks, globs, layers):
    out = []
    for c in checks:
        if layers and c['layer'] not in layers:
            continue
        if globs and not any(fnmatch.fnmatchcase(c['id'], g) for g in globs):
            continue
        out.append(c)
    return out


# -- the self-test ---------------------------------------------------------------------------------------------------

def self_test(checks):
    os.makedirs(T + 'out', exist_ok=True)
    tmp = tempfile.mkdtemp(prefix='run-selftest-', dir=T + 'out')
    ok = True

    def say(cond, what):
        nonlocal ok
        ok = ok and cond
        print('%s  %s' % ('pass' if cond else 'FAIL', what))
    try:
        by = {c['id']: c for c in checks}
        fast = dict(by['U:num/NUM-02'])
        cache = os.path.join(tmp, 'cache')
        kw = dict(runtimes=['pypy'], jobs=1, budget=BUDGET, force=False, cache_dir=cache, out_dir=os.path.join(tmp, 'out'),
                  quiet=True, save=False)
        # the check itself is same against its true baseline
        _, r, _ = session([fast], checks, **kw)
        say(r['pypy'][0]['verdict'] == 'same' and not r['pypy'][0]['cached'], 'U:num/NUM-02 runs and is same')
        # 1. a planted one-byte difference is caught
        base = open(ROOT + fast['baseline'], 'rb').read()
        i = len(base) // 2
        planted = base[:i] + bytes([base[i] ^ 0x01]) + base[i + 1:]
        pb = os.path.join(tmp, 'planted.out')
        open(pb, 'wb').write(planted)
        c1 = dict(fast, baseline=os.path.relpath(pb, ROOT))
        _, r, _ = session([c1], checks, **kw)
        say(r['pypy'][0]['verdict'] == 'differ', 'a planted one-byte difference in the baseline is caught')
        hh = dict(by['I:D:d01_flat'])
        _, r, _ = session([hh], checks, **kw)
        say(r['pypy'][0]['verdict'] == 'same', 'I:D:d01_flat is same against its true baseline')
        hb = open(ROOT + hh['baseline'], encoding='utf-8').read()
        j = hb.index('\n', hb.index('d01_flat')) - 1  # last char of d01_flat's first line
        ph = os.path.join(tmp, 'planted-house.out')
        open(ph, 'w', encoding='utf-8').write(hb[:j] + chr(ord(hb[j]) ^ 1) + hb[j + 1:])
        _, r, _ = session([dict(hh, baseline=os.path.relpath(ph, ROOT))], checks, **kw)
        say(r['pypy'][0]['verdict'] == 'differ', 'a planted one-byte difference in a household ask\'s block is caught')
        # a known fault: alone it reports `known`; with its run's asks it is `same`
        _, r, _ = session([by['I:D:d05_quarters~alone'], by['I:D:d05_quarters']], checks, **kw)
        v = {x['id']: x['verdict'] for x in r['pypy']}
        say(v == {'I:D:d05_quarters~alone': 'known', 'I:D:d05_quarters': 'same'},
            'known fault KF-01: alone `known`, in its run `same` (%s)' % v)
        # 2. a second run is a cache hit
        _, r, _ = session([fast], checks, **kw)
        say(r['pypy'][0]['cached'] == 'cached' and r['pypy'][0]['verdict'] == 'same', 'a second run of a check is a cache hit')
        # 3. a changed input byte misses the cache (same path, one byte changed)
        src = fast['files'].split()[0]
        cp = os.path.join(tmp, os.path.basename(src))
        shutil.copyfile(ROOT + src, cp)
        c3 = dict(fast, files=os.path.relpath(cp, ROOT))
        _, r, _ = session([c3], checks, **kw)
        _, r2, _ = session([c3], checks, **kw)
        say(r2['pypy'][0]['cached'] == 'cached', 'the copy at a new path runs once, then hits')
        b = open(cp, 'rb').read()
        open(cp, 'wb').write(b[:-1] + (b' ' if b[-1:] != b' ' else b'\n'))  # the last byte changed
        _, r3, _ = session([c3], checks, **kw)
        say(r3['pypy'][0]['cached'] == '', 'a changed input byte misses the cache')
        # 4. a run past a 2 s test budget is marked over, and the over is cached
        slow = max((c for c in checks if c['layer'] == 'unit' and c['baseline_s']), key=lambda c: float(c['baseline_s']))
        _, r, _ = session([slow], checks, **dict(kw, budget=2))
        say(r['pypy'][0]['verdict'] == 'over' and r['pypy'][0]['seconds'] < 10,
            'a run past a 2 s test budget is marked over (%s, %.1f s)' % (slow['id'], r['pypy'][0]['seconds']))
        _, r, _ = session([slow], checks, **dict(kw, budget=2))
        say(r['pypy'][0]['verdict'] == 'over' and r['pypy'][0]['cached'] == 'cached', 'an over is cached, not rerun')
        # 5. --interp both (4.5e): the fast side runs (no hit on refcore's entry), is same, and equals refcore's output
        tb, r, _ = session([fast, hh], checks, **dict(kw, runtimes=['pypy', 'pypy+fast']))
        f = r['pypy+fast']
        say(all(x['verdict'] == 'same' and not x['cached'] for x in f) and tb[0][-1] == 0 and tb[2][-1] == 0,
            '--interp both: NUM-02 and d01_flat run on refcore_fast, same, and equal to refcore')
        # 6. a changed refcore_fast byte changes its hash (so the fast variants' cache keys)
        cf = os.path.join(tmp, 'refcore_fast')
        shutil.copytree(FAST, cf, ignore=shutil.ignore_patterns('__pycache__'))
        lp = os.path.join(cf, 'layer.py')
        open(lp, 'ab').write(b'\n')
        say(fast_hash(cf) != fast_hash(), 'a changed refcore_fast byte changes its hash')
        # 7. the planted cache-key omission (the read's day left out) is caught by --verify 1, and only with the plant
        law = by['U:law/LAW-03']
        _, r, _ = session([law], checks, **dict(kw, runtimes=['pypy+fast+v1+plant']))
        e = r['pypy+fast+v1+plant'][0]
        _, r2, _ = session([law], checks, **dict(kw, runtimes=['pypy+fast+v1']))
        say(e['verdict'] == 'differ' and e['rc'] == 70 and r2['pypy+fast+v1'][0]['verdict'] == 'same',
            'a planted key omission is caught by --verify 1 on LAW-03 (exit 70), and LAW-03 passes without it')
        # 7b. the soft line (4.5f): uncached runs past SLOW count up, a cached result keeps the count, a run under resets
        rows, n = [], None
        for cached, sec, v in (('', 70, 'same'), ('', 61, 'differ'), ('cached', 10, 'same'), ('', 65, 'same'),
                               ('', 95, 'over'), ('', 20, 'same')):
            n = slow_runs({'slow_runs': n} if n is not None else None, {'cached': cached, 'seconds': sec, 'verdict': v})
            rows.append(n)
        say(rows == [1, 2, 2, 3, 0, 0] and BUDGET == 90 and SLOW == 60,
            'the budget is 90 s; a check over 60 s in 3 uncached runs in a row is a candidate (%s)' % rows)
        # 8. the examples' lint (4.5f, TESTING.md §9)
        p = subprocess.run([sys.executable, '-I', '-B', T + 'map/examples.py'], capture_output=True, text=True)
        say(p.returncode == 0, 'examples.tsv: ' + (p.stdout.strip().split('\n')[-1] if p.stdout.strip() else p.stderr[-200:]))
        # 8b. the placement (T1): every check in one milestone, BUILD-ORDER's globs and counts agree
        p = subprocess.run([sys.executable, '-I', '-B', T + 'map/placement.py'], capture_output=True, text=True)
        say(p.returncode == 0, 'map/placement.tsv against BUILD-ORDER: ' + ' '.join(p.stdout.split('\n')[1:2]).strip())
        # 8c. the kernel's property corpus (T1, PROPERTIES.md) is what gen.py's seeds give
        p = subprocess.run([sys.executable, '-I', '-B', T + 'properties/gen.py', '--check'], capture_output=True, text=True)
        say(p.returncode == 0, p.stdout.strip() or p.stderr[-200:])
        # 9. U2 (4.5f): under fast, a check with no baseline runs on refcore (its cache entry is refcore's)
        nb = dict(fast, baseline='')
        _, r, _ = session([nb], checks, **dict(kw, runtimes=['pypy+fast']))
        say(r['pypy+fast'][0]['verdict'] == 'ran' and os.path.exists(os.path.join(
            cache, cache_key(nb, 'pypy', refcore_hash())[:2], cache_key(nb, 'pypy', refcore_hash()) + '.json')),
            'under --interp fast a check with no baseline runs on refcore (U2)')
        say(main_variants([]) == ['pypy+fast'] and main_variants(['--interp', 'refcore']) == ['pypy'],
            'the default interpreter is fast; --interp refcore still selects refcore')
        # 10. the watchdog: found when its command line names engine/tests/tools/memcap.py, not an old copy
        say(watchdog('  12 python3 -I -B engine/tests/tools/memcap.py 8 5\n') == 12 and
            watchdog('  13 python3 -I -B tmp/session5/tools/memcap.py 8 5\n') is None and watchdog() is not None,
            'the watchdog is recognised (engine/tests/tools/memcap.py), an old copy is not, and one is running')
        p = subprocess.run([sys.executable, '-I', '-B', T + 'tools/memcap.py', '--self-test'], capture_output=True, text=True)
        say(p.returncode == 0, 'memcap.py matches refcore, refcore_fast and engine runs: ' + p.stdout.strip())
        # 11. --interp engine (T1), on a stand-in for the binary: a shell script that prints given bytes and exits
        engine_self_test(say, checks, by, fast, tmp, cache, kw)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print('self-test %s' % ('PASSED' if ok else 'FAILED'))
    return ok


def engine_self_test(say, checks, by, fast, tmp, cache, kw):
    """--interp engine's wiring, proven on a stand-in binary (no engine needed): same, error on a wrong exit, the
    binary's hash in the cache key, never `ran`, and R:all and the key scripts not run."""
    real = ENGINE[0]
    stand = os.path.join(tmp, 'tally')
    base = open(ROOT + fast['baseline'], 'rb').read()

    def stand_in(out, rc):
        with open(stand, 'wb') as f:
            f.write(b"#!/bin/sh\ncat <<'TALLY_EOF'\n" + out + b"TALLY_EOF\nexit %d\n" % rc)
        os.chmod(stand, 0o755)
    ekw = dict(kw, runtimes=['engine'])
    try:
        ENGINE[0] = stand
        rc = 1 if base.startswith(b'refused: ') else 0
        stand_in(base, rc)
        _, r, _ = session([fast], checks, **ekw)
        say(r['engine'][0]['verdict'] == 'same' and not r['engine'][0]['cached'],
            '--interp engine: a stand-in printing NUM-02\'s baseline, exit %d, is same' % rc)
        _, r, _ = session([fast], checks, **ekw)
        say(r['engine'][0]['cached'] == 'cached', '--interp engine: a second run is a cache hit')
        stand_in(base, 69)
        _, r, _ = session([fast], checks, **ekw)
        say(r['engine'][0]['verdict'] == 'error' and not r['engine'][0]['cached'],
            '--interp engine: a changed binary misses the cache, and the right bytes with exit 69 are an error (%s)'
            % r['engine'][0]['detail'])
        stand_in(b'refused: LOAD-PARSE planted\n', 0)
        _, r, _ = session([fast], checks, **ekw)
        say(r['engine'][0]['verdict'] == 'error', '--interp engine: a refusal with exit 0 is an error')
        stand_in(b'tally run: not implemented\n', 69)
        _, r, _ = session([fast, by['I:D:d01_flat']], checks, **ekw)
        say([x['verdict'] for x in r['engine']] == ['error', 'error'],
            '--interp engine: the stub\'s answer (exit 69) is an error on a unit and a household check')
        # never `ran`: a check with no baseline is judged against refcore's own run from the cache (test 9 made it)
        nb = dict(fast, baseline='')
        stand_in(base, rc)
        _, r, _ = session([nb], checks, **ekw)
        _, r2, _ = session([nb], checks, **dict(ekw, cache_dir=os.path.join(tmp, 'empty-cache')))
        say(r['engine'][0]['verdict'] == 'same' and r2['engine'][0]['verdict'] == 'error',
            '--interp engine: no baseline is never `ran`: same against refcore\'s cached run, an error without one '
            '(%s; %s)' % (r['engine'][0]['detail'], r2['engine'][0]['detail'][:40]))
        say(all(x['verdict'] != 'ran' for x in r['engine'] + r2['engine']) and 'ran' not in str(
            [verdict_of(nb, dict(out=base.decode(), rc=rc, over=False, runtime='engine', budget=BUDGET), checks, cache)]),
            '--interp engine: no verdict of an engine run is `ran`')
        say(not_run(by['R:all'], 'engine') and not_run(by['I:M:LH-11~key'], 'engine') and not not_run(by['R:all'], 'pypy+fast')
            and not_run(next(c for c in checks if c['note'].startswith('not-run')), 'engine'),
            '--interp engine: R:all, the key scripts and the load rows are not run; R:all still runs on refcore')
    finally:
        ENGINE[0] = real


# -- the tables ------------------------------------------------------------------------------------------------------

def tables(checks):
    res = {}
    for rt in ('pypy', 'cpython'):
        p = T + 'results/%s.tsv' % rt
        if os.path.exists(p):
            with open(p, encoding='utf-8', newline='') as f:
                res[rt] = {r['id']: r for r in csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)}
    py = res.get('pypy', {})
    cp = res.get('cpython', {})

    def cpy_s(c):
        """(stored CPython seconds, what they time)."""
        if c['baseline_s']:
            return float(c['baseline_s']), 'check'
        m = re.search(r'run_cpython_s=(\d+)', c['note'])
        return (float(m.group(1)), 'run') if m else (None, '')

    def group(c):
        if c['layer'] == 'interpreter':
            return 'interpreter'
        if c['id'].startswith('U:set/'):
            return 'unit: probe sets'
        if c['id'].startswith('U:'):
            return 'unit: probes'
        if c['id'].startswith('I:M:'):
            return 'integration: old models'
        if c['id'].startswith(('I:light/', 'I:hardset/')):
            return 'integration: light and hardset'
        return 'integration: household ' + c['id'].split(':')[1]
    groups = {}
    for c in checks:
        groups.setdefault(group(c), []).append(c)
    L = ['# Before and after: the checks on CPython (stored) and on PyPy (session 4.5a)', '',
         'Made by `python3 -I -B engine/tests/run.py --tables` from `results/pypy.tsv` (and `results/cpython.tsv` where the '
         '`both` mode ran) and `checks.tsv`. **Before:** the CPython seconds stored with the baselines (snap16\'s '
         '`summary.txt`, whole seconds; for household asks only the whole run\'s seconds are stored, from session 5\'s '
         'runs this morning, `run_cpython_s` in the note). **After:** PyPy seconds per check, budget 60 s, 4 jobs. '
         '**Equality:** `same` = byte for byte (whole output), or the ask\'s blocks in run5.py\'s form (households).', '',
         '## Per layer', '',
         '| group | checks | same | differ | over | known fault | not run | CPython s (stored, finished checks) '
         '| PyPy s (same checks) | PyPy s (all run) |',
         '|---|---|---|---|---|---|---|---|---|---|']
    detail = []
    for g in sorted(groups, key=lambda g: (not g.startswith('interp'), not g.startswith('unit'), g)):
        cs = groups[g]
        n = {v: 0 for v in VERDICTS + ('not run', 'missing')}
        cs_s = py_s = py_all = 0.0
        for c in cs:
            r = py.get(c['id'])
            if c['note'].startswith('not-run'):
                n['not run'] += 1
                continue
            if r is None:
                n['missing'] += 1
                continue
            n[r['verdict']] += 1
            py_all += float(r['seconds'])
            s, what = cpy_s(c)
            if s is not None and what == 'check' and r['verdict'] == 'same':
                cs_s += s
                py_s += float(r['seconds'])
        L.append('| %s | %d | %d | %d | %d | %d | %d | %s | %s | %.0f |' % (
            g, len(cs), n['same'], n['differ'] + n['fixed'], n['over'], n['known'], n['not run'], '%.0f' % cs_s if cs_s else '–',
            '%.0f' % py_s if cs_s else '–', py_all) + (' missing %d' % n['missing'] if n['missing'] else ''))
        detail += ['', '## ' + g, '', '| id | CPython s | PyPy s | verdict | note |', '|---|---|---|---|---|']
        for c in cs:
            r = py.get(c['id'])
            s, what = cpy_s(c)
            cs_txt = '' if s is None else ('%.0f' % s if what == 'check' else '(run %.0f)' % s)
            if c['note'].startswith('not-run'):
                detail.append('| `%s` | %s | – | not run | %s |' % (c['id'], cs_txt, c['note'].split(' run=')[0][9:]))
                continue
            if r is None:
                detail.append('| `%s` | %s | – | missing | |' % (c['id'], cs_txt))
                continue
            both = cp.get(c['id'])
            extra = ('cpython %s s, %s' % (both['seconds'], both['verdict'])) if both else ''
            v = r['verdict'] + (' (cached)' if r['cached'] else '')
            pys = '> %s' % r['budget'] if r['verdict'] == 'over' else '%.1f' % float(r['seconds'])
            detail.append('| `%s` | %s | %s | %s | %s |' % (c['id'], cs_txt, pys, v,
                                                           r['detail'] if r['verdict'] != 'same' else extra))
    # household runs whose whole-run CPython seconds are stored: that run against its asks run one by one on PyPy
    runs = {}
    for c in checks:
        m = re.search(r'run=(\S+) run_asks=\d+ run_cpython_s=(\d+)', c['note'])
        if m and not c['note'].startswith('not-run') and not c['id'].endswith('~alone'):
            runs.setdefault((m.group(1), float(m.group(2))), []).append(py.get(c['id']))
    if runs:
        L += ['', '## Household runs with stored CPython seconds', '',
              'The run\'s CPython seconds (session 5\'s run this morning, on its copies of the inputs) against its asks '
              'run one by one on PyPy: the slowest ask is what the 60 s budget sees.', '',
              '| run | asks | CPython s (whole run) | PyPy s (sum of asks) | PyPy s (slowest ask) | verdicts |',
              '|---|---|---|---|---|---|']
        for (name, s), rs in sorted(runs.items(), key=lambda x: -x[0][1]):
            done = [r for r in rs if r]
            sec = [float(r['seconds']) for r in done if r['verdict'] != 'over']
            vs = {}
            for r in done:
                vs[r['verdict']] = vs.get(r['verdict'], 0) + 1
            L.append('| %s | %d | %.0f | %.0f%s | %s | %s |' % (
                name, len(rs), s, sum(sec), ' + over' if any(r['verdict'] == 'over' for r in done) else '',
                '> 60' if any(r['verdict'] == 'over' for r in done) else '%.0f' % max(sec),
                ', '.join('%d %s' % (n, v) for v, n in sorted(vs.items()))))
        L += ['', 'E1-02-B\'s stored run asked 3 of the 6 asks its baseline printed and this table checks '
                  '(`tx2_owes`, `tx2_worth`, `tx2_mtg_day`), so its CPython seconds cover only those.']
    open(T + 'reports/before-after.md', 'w', encoding='utf-8').write('\n'.join(L + detail) + '\n')

    # over-budget.md: every over (or killed) check, grouped by household
    O = ['# Checks over the 60 s budget on PyPy (session 4.5a)', '',
         'Made by `run.py --tables` from `results/pypy.tsv`. Every check whose PyPy run passed 60 s (`over`) or was '
         'stopped by the memory watchdog (`killed`), with its files, asks and the stored CPython seconds. Input for 4.5b '
         '(the halving test) and 4.5c (the split). The runs on `load/benchmarks.md` are not here: they are never run.',
         '']
    og = {}
    for c in checks:
        r = py.get(c['id'])
        if r and r['verdict'] in ('over', 'killed') and not c['note'].startswith('not-run'):  # 4.5c: retired rows stay out
            og.setdefault(group(c), []).append((c, r))
    O.append('| group | over |')
    O.append('|---|---|')
    for g in sorted(og):
        O.append('| %s | %d |' % (g, len(og[g])))
    for g in sorted(og):
        O += ['', '## %s (%d)' % (g, len(og[g])), '']
        for c, r in og[g]:
            s, what = cpy_s(c)
            O.append('- `%s` — %s; asks: %s; CPython: %s; question: %s; %s' % (
                c['id'], r['verdict'], c['asks'] or 'all',
                '–' if s is None else ('%.0f s' % s if what == 'check' else '%.0f s for the whole run' % s),
                c['question'] or '–', c['note'] or ''))
            O.append('  - cwd `%s`; options `%s`; files (%d, in run order): %s' % (
                c['cwd'].replace(ROOT, ''), c['options'], len(c['files'].split()),
                ' '.join('`%s`' % f.replace(ROOT, '') for f in c['files'].split())))
    open(T + 'reports/over-budget.md', 'w', encoding='utf-8').write('\n'.join(O) + '\n')
    print('wrote before-after.md and over-budget.md')


# -- main ------------------------------------------------------------------------------------------------------------

def adhoc(argv):
    """--adhoc [--cwd DIR] -- <refcore args>: one run of the frozen refcore on PyPy, 90 s, for a writer or the judge
    (4.5c). Prints the output and a '# ' line with rc and seconds (or 'over 60 s'); appends a line to out/adhoc.log.
    Nothing is cached or recorded in results/."""
    cwd = os.getcwd()
    if '--cwd' in argv:
        cwd = os.path.abspath(argv[argv.index('--cwd') + 1])
    if '--' not in argv:
        sys.exit('usage: run.py --adhoc [--cwd DIR] -- run <files> [--ask ...]')
    args = argv[argv.index('--') + 1:]
    if not args or args[0] not in ('run', 'check'):
        sys.exit('--adhoc runs `refcore run ...` or `refcore check ...` only')
    refcore_hash()
    c = {'layer': 'adhoc', 'cwd': cwd, 'files': '', 'asks': '', 'options': ''}
    env = {k: v for k, v in os.environ.items() if k != 'REFCORE_DEBUG'}
    t0 = time.time()
    p = subprocess.Popen([PY['pypy'], '-I', '-B', RC] + args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                         env=env, start_new_session=True)
    try:
        out, err = p.communicate(timeout=BUDGET)
        res = 'rc %s, %.1f s' % (p.returncode, time.time() - t0)
        sys.stdout.write(text(out) + text(err))
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):  # the group already gone (macOS answers EPERM)
            pass
        p.communicate()
        res = 'over %g s (killed, nothing printed)' % BUDGET
    print('# ' + res)
    os.makedirs(T + 'out', exist_ok=True)
    with open(T + 'out/adhoc.log', 'a', encoding='utf-8') as f:  # run output (T1): reports/adhoc.log keeps the log to 2026-10-31
        f.write('%s\t%s\t%s\t%s\n' % (time.strftime('%Y-%m-%d %H:%M:%S'), cwd, ' '.join(args), res))
    return 0


def main_variants(argv):
    """The variants a command line asks for (the interpreter's default is fast, 4.5f U2)."""
    rt = argv[argv.index('--runtime') + 1] if '--runtime' in argv else 'pypy'
    interp = argv[argv.index('--interp') + 1] if '--interp' in argv else 'fast'
    verify = int(argv[argv.index('--verify') + 1]) if '--verify' in argv else 0
    if interp == 'engine':
        return ['engine']
    rts = ['pypy', 'cpython'] if rt == 'both' else [rt]
    fast = lambda r: r + '+fast' + ('+v%d' % verify if verify else '')
    return rts if interp == 'refcore' else [fast(r) for r in rts] if interp == 'fast' else [rts[0], fast(rts[0])]


def main(argv):
    if '--adhoc' in argv:
        need_watchdog()
        return adhoc(argv)
    # `--opt=value` is `--opt value`: at the repository root the guard reads a bare `engine` as the folder (T1)
    argv = [x for a in argv for x in (a.split('=', 1) if a.startswith('--') and '=' in a else [a])]
    checks = load_checks()
    if '--self-test' in argv:
        need_watchdog()
        return 0 if self_test(checks) else 1
    if '--tables' in argv:
        tables(checks)
        return 0
    globs, layers, rt, jobs, budget, interp, verify = [], [], 'pypy', MAX_JOBS, BUDGET, 'fast', 0
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == '--layer':
            layers += argv[i + 1].split(',')
            i += 1
        elif a == '--runtime':
            rt = argv[i + 1]
            i += 1
        elif a == '--interp':
            interp = argv[i + 1]
            i += 1
        elif a == '--verify':
            verify = int(argv[i + 1])
            i += 1
        elif a == '--jobs':
            jobs = int(argv[i + 1])
            i += 1
        elif a == '--budget':
            budget = float(argv[i + 1])
            i += 1
        elif a in ('--force', '--list'):
            pass
        elif a.startswith('-'):
            sys.exit('unknown option %s' % a)
        else:
            globs.append(a)
        i += 1
    if jobs < 1 or jobs > MAX_JOBS:
        sys.exit('--jobs is 1..%d' % MAX_JOBS)
    if rt not in ('pypy', 'cpython', 'both'):
        sys.exit('--runtime is pypy, cpython or both')
    if interp not in ('refcore', 'fast', 'both', 'engine') or (rt == 'both' and interp == 'both'):
        sys.exit('--interp is refcore, fast, both (not with --runtime both) or engine')
    if verify and interp in ('refcore', 'engine'):
        sys.exit('--verify N is for --interp fast or both')
    if interp == 'engine' and '--runtime' in argv:
        sys.exit('--runtime is for refcore; the engine is its own binary')
    sel = select(checks, globs, layers)
    if '--list' in argv:
        for c in sel:
            print(c['id'], c['layer'], c['note'])
        print('%d checks' % len(sel))
        return 0
    if not sel:
        sys.exit('no check selected')
    need_watchdog()
    if interp == 'engine':
        engine_hash()  # stops here, with how to build it, when the binary is missing
    session(sel, checks, main_variants(argv), jobs, budget, '--force' in argv)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
