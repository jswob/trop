"""The test runner for the interpreter, unit and integration layers (session 4.5a). Standard library only.

Runs the checks of tmp/tests/checks.tsv on the frozen engine/tools/refcore, unchanged, and compares each output with
its stored baseline. Form only: it runs and compares; it never explains a difference.

    python3 -I -B tmp/tests/run.py [ID_GLOB ...] [--layer L[,L]] [--runtime pypy|cpython|both] [--jobs N<=4]
                                    [--budget S] [--force] [--list]
    python3 -I -B tmp/tests/run.py --self-test
    python3 -I -B tmp/tests/run.py --tables        -> before-after.md, over-budget.md (from results/pypy.tsv)
    python3 -I -B tmp/tests/run.py --adhoc [--cwd DIR] -- run <files> [--ask a ...]   one 60 s PyPy run, logged

- Selection: id globs (fnmatch) and/or layers; none selects every check. Rows whose note starts `not-run:` (runs on
  tmp/design2/benchmarks.md) are never run; they are counted as not run with their reason.
- Budget: 60 s per run (the user's rule). Past it the run is killed and recorded as `over`; an `over` is cached like
  any result, so it is not rerun (unless --force or a larger --budget).
- The cache (cache/<key>.json: output, seconds, rc) is keyed by sha256 over: the runtime and its version; the refcore
  tree (its lines of engine/tools/SHA256SUMS, checked against the files at start); each input file's path and bytes in
  run order; the asks; the options, the mode and the cwd. A hit is reported `cached` and not rerun.
- Compare: a unit, model, light or hardset check's whole output (stdout then stderr, as run_set.py stored it) equals
  its baseline file; a household check (I:<H>:<ask>) is compared in run5.py's form: the ask's blocks (WARN-UNREAD
  and warning lines, '#' lines and blank lines dropped, lines right-stripped) plus the baseline's blocks no ask owns.
  R:all is `same` when the last line reads '950 tests, 950 passed, 0 failed'.
- Known faults (known-faults.md): a check whose note starts `xfail:` is expected to differ; it reports `known` while
  it does and `fixed` once it matches its baseline. Its guarded twin (note `known-fault:`) runs as usual.
- `both` runs each check on PyPy and on CPython and also requires the two outputs to be equal.
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

ROOT = '/Users/jswob/projects/trop/'
T = ROOT + 'tmp/tests/'
RC = ROOT + 'engine/tools/refcore'
SUMS = ROOT + 'engine/tools/SHA256SUMS'
PY = {'pypy': os.path.expanduser('~/.local/share/uv/python/pypy-3.11.13-macos-aarch64-none/bin/pypy3.11'),
      'cpython': ROOT + 'engine/tools/parsecheck/.venv/bin/python'}  # the python the snap16 baselines ran on
MAX_JOBS = 4
TESTS_LINE = '950 tests, 950 passed, 0 failed'
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


def argv_of(c):
    if script(c):
        return c['files'].split()[:1] + c['asks'].split()
    if c['layer'] == 'interpreter':
        return c['options'].split()
    a = ['run'] + c['files'].split() + c['options'].split()
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


def cache_key(c, rt, rc_hash):
    h = hashlib.sha256()

    def part(b):
        b = b if isinstance(b, bytes) else b.encode('utf-8')
        h.update(b'%d:' % len(b))
        h.update(b)
    part(rt)
    part(version(rt))
    part(rc_hash)
    for f in c['files'].split():
        part(f)
        part(open(f if f.startswith('/') else os.path.join(c['cwd'], f), 'rb').read())
    part(c['asks'])
    part(c['options'])
    part(' '.join(argv_of(c)))
    part(c['cwd'])
    return h.hexdigest()


# -- one run ---------------------------------------------------------------------------------------------------------

def text(b):
    """Bytes as subprocess's text mode gives them (UTF-8, universal newlines)."""
    return b.decode('utf-8', 'surrogateescape').replace('\r\n', '\n').replace('\r', '\n')


def execute(c, rt, budget):
    env = {k: v for k, v in os.environ.items() if k != 'REFCORE_DEBUG'}
    t0 = time.time()
    p = subprocess.Popen([PY[rt], '-I', '-B'] + ([] if script(c) else [RC]) + argv_of(c), cwd=c['cwd'], stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, env=env, start_new_session=True)
    try:
        out, err = p.communicate(timeout=budget)
        return {'out': text(out) + text(err), 'seconds': round(time.time() - t0, 2), 'rc': p.returncode, 'over': False}
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
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
    e.update(id=c['id'], runtime=rt, version=version(rt), key=key, budget=budget, when=time.strftime('%Y-%m-%d %H:%M:%S'))
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
    newer = [f for f in c['files'].split() if os.path.getmtime(f if f.startswith('/') else os.path.join(c['cwd'], f)) > b]
    return ('INPUT NEWER THAN BASELINE: ' + ' '.join(newer[:3])) if newer else 'inputs older than the baseline'


# -- a session -------------------------------------------------------------------------------------------------------

RES_COLS = ['id', 'layer', 'runtime', 'verdict', 'seconds', 'rc', 'cached', 'budget', 'detail', 'when']


def save_results(rt, rows):
    path = T + 'results/%s.tsv' % rt
    old = {}
    if os.path.exists(path):
        with open(path, encoding='utf-8', newline='') as f:
            old = {r['id']: r for r in csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)}
    old.update({r['id']: r for r in rows})
    os.makedirs(T + 'results', exist_ok=True)
    with open(path + '.tmp', 'w', encoding='utf-8') as f:
        f.write('\t'.join(RES_COLS) + '\n')
        for r in sorted(old.values(), key=lambda r: r['id']):
            f.write('\t'.join(str(r.get(k, '')).replace('\t', ' ').replace('\n', ' ') for k in RES_COLS) + '\n')
    os.replace(path + '.tmp', path)


def verdict_of(c, e, checks):
    if e['over']:
        return 'over', 'past %g s' % e['budget']
    if e['rc'] is not None and e['rc'] < 0:
        return 'killed', 'signal %d (the memory watchdog?)' % -e['rc']
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
    for layer in LAYERS:
        cs = [c for c in sel if c['layer'] == layer]
        if not cs:
            continue
        todo = [c for c in cs if not c['note'].startswith('not-run')]
        skipped = [c for c in cs if c['note'].startswith('not-run')]
        # the slow ones first, so the tail of the pool is short
        todo.sort(key=lambda c: -stored_s(c))
        t0 = time.time()
        work = [(c, rt) for c in todo for rt in runtimes]
        with ThreadPoolExecutor(jobs) as ex:
            res = list(ex.map(lambda w: (w[0], w[1], run_one(w[0], w[1], budget, force, rc_hash, cache_dir)), work))
        wall = time.time() - t0
        counts = {rt: dict.fromkeys(VERDICTS + ('cached', 'both-differ'), 0) for rt in runtimes}
        outs = {}
        for c, rt, e in res:
            v, d = verdict_of(c, e, checks)
            counts[rt][v] += 1
            counts[rt]['cached'] += e['cached']
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
            table.append((layer, rt, len(cs), k['same'], k['differ'], k['over'], k['killed'], k['known'], k['fixed'],
                          k['ran'], k['cached'], len(skipped),
                          wall, k['both-differ'] if len(runtimes) == 2 and rt == runtimes[0] else ''))
    if save:
        for rt in runtimes:
            save_results(rt, rows_by_rt[rt])
    lines = ['| layer | runtime | checks | same | differ | over | killed | known fault | fixed | ran (J marks) | cached '
             '| not run | wall s | pypy≠cpython |', '|---|---|---|---|---|---|---|---|---|---|---|---|---|---|']
    for r in table:
        lines.append('| %s | %s | %d | %d | %d | %d | %d | %d | %d | %d | %d | %d | %.0f | %s |' % r)
    lines.append('')
    lines.append('total wall %.0f s, %s, budget %g s, jobs %d' % (time.time() - t_all, time.strftime('%Y-%m-%d %H:%M'),
                                                                   budget, jobs))
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
    tmp = tempfile.mkdtemp(prefix='run-selftest-', dir=ROOT + 'tmp/session4_5/a')
    ok = True

    def say(cond, what):
        nonlocal ok
        ok = ok and cond
        print('%s  %s' % ('pass' if cond else 'FAIL', what))
    try:
        by = {c['id']: c for c in checks}
        fast = dict(by['U:num/NUM-02'])
        cache = os.path.join(tmp, 'cache')
        kw = dict(runtimes=['pypy'], jobs=1, budget=60, force=False, cache_dir=cache, out_dir=os.path.join(tmp, 'out'),
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
        shutil.copyfile(src, cp)
        c3 = dict(fast, files=cp)
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
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print('self-test %s' % ('PASSED' if ok else 'FAILED'))
    return ok


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
         'Made by `python3 -I -B tmp/tests/run.py --tables` from `results/pypy.tsv` (and `results/cpython.tsv` where the '
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
    open(T + 'before-after.md', 'w', encoding='utf-8').write('\n'.join(L + detail) + '\n')

    # over-budget.md: every over (or killed) check, grouped by household
    O = ['# Checks over the 60 s budget on PyPy (session 4.5a)', '',
         'Made by `run.py --tables` from `results/pypy.tsv`. Every check whose PyPy run passed 60 s (`over`) or was '
         'stopped by the memory watchdog (`killed`), with its files, asks and the stored CPython seconds. Input for 4.5b '
         '(the halving test) and 4.5c (the split). The runs on `tmp/design2/benchmarks.md` are not here: they are never run.',
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
    open(T + 'over-budget.md', 'w', encoding='utf-8').write('\n'.join(O) + '\n')
    print('wrote before-after.md and over-budget.md')


# -- main ------------------------------------------------------------------------------------------------------------

def adhoc(argv):
    """--adhoc [--cwd DIR] -- <refcore args>: one run of the frozen refcore on PyPy, 60 s, for a writer or the judge
    (4.5c). Prints the output and a '# ' line with rc and seconds (or 'over 60 s'); appends a line to adhoc.log.
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
        out, err = p.communicate(timeout=60.0)
        res = 'rc %s, %.1f s' % (p.returncode, time.time() - t0)
        sys.stdout.write(text(out) + text(err))
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        p.communicate()
        res = 'over 60 s (killed, nothing printed)'
    print('# ' + res)
    with open(T + 'adhoc.log', 'a', encoding='utf-8') as f:
        f.write('%s\t%s\t%s\t%s\n' % (time.strftime('%Y-%m-%d %H:%M:%S'), cwd, ' '.join(args), res))
    return 0


def main(argv):
    if '--adhoc' in argv:
        return adhoc(argv)
    checks = load_checks()
    if '--self-test' in argv:
        return 0 if self_test(checks) else 1
    if '--tables' in argv:
        tables(checks)
        return 0
    globs, layers, rt, jobs, budget = [], [], 'pypy', MAX_JOBS, 60.0
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == '--layer':
            layers += argv[i + 1].split(',')
            i += 1
        elif a == '--runtime':
            rt = argv[i + 1]
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
    sel = select(checks, globs, layers)
    if '--list' in argv:
        for c in sel:
            print(c['id'], c['layer'], c['note'])
        print('%d checks' % len(sel))
        return 0
    if not sel:
        sys.exit('no check selected')
    session(sel, checks, ['pypy', 'cpython'] if rt == 'both' else [rt], jobs, budget, '--force' in argv)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
