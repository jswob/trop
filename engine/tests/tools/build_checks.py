"""Build engine/tests/checks.tsv: one row per check of session 4.5a's brief §4, with its run (cwd, files, asks, options)
and its baseline. Read-only on every source; writes only checks.tsv. Run from anywhere:
    python3 -I -B engine/tests/tools/build_checks.py [--counts]

Every path it writes is relative to the repository root, which it finds from its own place (4.5f).
Sources (the run definitions, unchanged but for their paths, 4.5f):
  unit        defs/probes.list, probe-sets.list (+ PTY-05 as rerun.sh runs it)                   -> baselines/snap16-probes/, -probe-sets/
  integration defs/models.list, light.list, b1.list, b2.list (+ b2fix as rerun.sh runs it)         -> baselines/snap16-models/, -light/, -b1/, -b2/, -b2fix/
              households A, B, D, X as house13.py defines them and E1 as e1_rerun.sh does, with run5.py's benchmark
              exclusions and its E1-02-B run; one check per ask, each against that ask's block in snap16-house/,
              snap16-e1/ or e1-02-B16.out. The inputs are the ones snap16 ran on (integration/cuts/, integration/
              households/), not session 5's copies, which carried later edits (cases 1.0.2).
Runs on load/benchmarks.md are listed with a `not-run:` note and never run.
The lists name their texts as session 4 did, relative to tmp/design2/ (LIST below says where each part went).
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))) + '/'
T = 'engine/tests/'  # every path below is relative to ROOT
I = T + 'integration/'
CHK = T + 'defs/'
SNAP = T + 'baselines/'
OUT = ROOT + T + 'checks.tsv'
# session 4's tmp/design2/ parts, as the lists name them -> their place in the suite
LIST = [('cases/probes/', T + 'unit/probes/'), ('models/', I + 'models/'), ('regress/light/', I + 'light/'),
        ('hardset/', I + 'blind/')]
COLS = ['id', 'layer', 'cwd', 'files', 'asks', 'options', 'baseline', 'block', 'baseline_s', 'question', 'note', 'key']
rows = []


def rel(p):
    return p[len(ROOT):] if p.startswith(ROOT) else p


def at(p):
    """A list's path (relative to session 4's tmp/design2/) -> its place in the suite."""
    for old, new in LIST:
        if p.startswith(old):
            return new + p[len(old):]
    raise ValueError(p)


def rglob(pat, **kw):
    """glob, repository-relative."""
    return [rel(p) for p in glob.glob(ROOT + pat, **kw)]


def add(**kw):
    r = {c: '' for c in COLS}
    r.update(kw)
    rows.append(r)


def summary(snap):
    secs = {}
    f = ROOT + SNAP + snap + '/summary.txt'
    if os.path.exists(f):
        for l in open(f):
            m = re.match(r'(\S+) rc=-?\d+ sec=(\d+)', l)
            if m:
                secs[m.group(1)] = m.group(2)
    return secs


# --- the interpreter's own tests
add(id='R:all', layer='interpreter', cwd='.', files='', options='test', baseline='950 passed',
    note='refcore test; same = its last line reports 950 passed and nothing failed')


# --- the run_set.py lists: one check per line, the whole output its baseline
def from_list(lst, snap, layer, idf, note=''):
    secs = summary(snap)
    for l in open(ROOT + CHK + lst):
        if not l.strip() or l.startswith('#'):
            continue
        paths = l.split()
        add(id=idf(paths[0]), layer=layer, cwd=T, files=' '.join(at(p) for p in paths),
            baseline=SNAP + '%s/%s.out' % (snap, paths[0].replace('/', '__')), baseline_s=secs.get(paths[0], ''),
            note=note)


def strip(p, pre):
    assert p.startswith(pre) and p.endswith('.tally'), p
    return p[len(pre):-len('.tally')]


from_list('probes.list', 'snap16-probes', 'unit', lambda p: 'U:' + strip(p, 'cases/probes/'))
from_list('probe-sets.list', 'snap16-probe-sets', 'unit', lambda p: 'U:set/' + strip(p, 'cases/probes/'))
pty = sorted(rglob(T + 'unit/probes/pty/PTY-05/**/*.tally', recursive=True))  # rerun.sh: $(find ... | sort)
add(id='U:set/' + strip(pty[0], T + 'unit/probes/'), layer='unit', cwd=T, files=' '.join(pty),
    baseline=SNAP + 'snap16-probe-sets/PTY-05-pin-bytes.out', note='rerun.sh PTY-05 line (find | sort)')
from_list('models.list', 'snap16-models', 'integration', lambda p: 'I:M:' + strip(p, 'models/'))
from_list('light.list', 'snap16-light', 'integration', lambda p: 'I:light/' + strip(p, 'regress/light/'))
from_list('b1.list', 'snap16-b1', 'integration', lambda p: 'I:hardset/' + strip(p, 'hardset/'))
from_list('b2.list', 'snap16-b2', 'integration', lambda p: 'I:hardset/' + strip(p, 'hardset/'))
add(id='I:hardset/b2fix/division', layer='integration', cwd=T, files=I + 'blind/b2fix/division.tally',
    baseline=SNAP + 'snap16-b2fix/division.out', note='rerun.sh b2fix line: I-12 with its ask renamed')


# --- the households, as run5.py (A, B, D, E1) and house13.py (X) define their runs
def ids_of(path):
    out = []
    for l in open(ROOT + path, encoding='utf-8', errors='replace'):
        m = re.match(r'^([a-z][a-z0-9_]*)[ (:\[]', l)
        if m and m.group(1) not in ('warning', 'refused', 'time', 'exit', 'rc', 'real', 'user', 'sys') and m.group(1) not in out:
            out.append(m.group(1))
    return out


OLD = SNAP + 'snap16-house/'
LAWB = open(ROOT + I + 'cuts/B-law/lawfiles.txt').read().split()
runs = []  # (H, run name, baseline, cwd, files, asks, options, not-run reason)
for l in open(ROOT + I + 'cuts/A/drive11.sh'):
    for m in re.finditer(r'\$R \$D/(\S+) (\S+) ([^&]+)', l):
        lst, name, asks = m.group(1), m.group(2), m.group(3).split(';')[0].split()
        nr = 'benchmarks.md row 9' if name in ('q05-ania', 'q05-why') else ''
        files = open(ROOT + I + 'cuts/A/' + lst).read().split()
        runs.append(('A', 'A-' + name, OLD + 'A-%s.out' % name, I + 'cuts/A', files, asks, [], nr))
for f in sorted(rglob(I + 'cuts/B/out11/*.out')):
    name = os.path.basename(f)[:-4]
    m = re.match(r'# dir (\S+); scenarios: (.*); asks: (.*)$', open(ROOT + f, encoding='utf-8').readline().strip())
    if not m:
        continue
    nr = ('benchmarks.md row 4' if name.startswith('cd7full') else 'benchmarks.md row 5' if name == 'b12_liquid'
          else 'benchmarks.md rows 5, 14' if name in ('b12_det', 'b12_p3') else '')
    d, sc, asks = m.group(1), m.group(2).split(), m.group(3).split()
    files = sorted(rglob(d + '/*.tally')) + sorted(rglob(d + '/records/*.tally')) + sorted(rglob(d + '/asks/*.tally'))
    runs.append(('B', 'B-' + name, OLD + 'B-%s.out' % name, d, files + [d + '/' + s for s in sc] + LAWB, asks, [], nr))
for f in sorted(rglob(I + 'cuts/D/out11/*.txt')):
    name = os.path.basename(f)[:-4]
    d = I + 'cuts/D/' + ('p200_11' if name in ('d10b200', 'd10p200') else 'cut11')
    asks = ids_of(f)
    if asks:
        runs.append(('D', 'D-' + name, OLD + 'D-%s.out' % name, d, sorted(rglob(d + '/*.tally')), asks, [], ''))
E = I + 'cuts/E1/cdB/households/E1'
for f in ('s10-cdB-0103', 's10-cdB-02'):
    asks = [a for a in ids_of(I + 'cuts/E1/runs/%s.out' % f) if a not in ('exit',)]
    runs.append(('E1', 'E1-' + f, SNAP + 'snap16-e1/%s.out' % f, E, sorted(rglob(E + '/**/*.tally', recursive=True)),
                 asks, [], ''))
txf = open(ROOT + I + 'cuts/E1-02-B/B/files.txt').read().split()
# run5.py asks three of the six asks the baseline printed; every printed ask is a check
runs.append(('E1', 'E1-02-B', SNAP + 'e1-02-B16.out', I + 'households/B', txf, ids_of(SNAP + 'e1-02-B16.out'), [], ''))
seen = set()
for q in sorted(rglob(I + 'amendments/out11/queue*.log')):
    for l in open(ROOT + q):
        m = re.match(r'^\S+ started (\S+) (engine/\S+)(.*)$', l.strip())
        if not m:
            continue
        name, d, rest = m.group(1), m.group(2), m.group(3).split()
        if name in ('sh', 'python3'):
            if 'job11.sh' not in d:
                continue
            name, d, rest = rest[0], rest[1], rest[2:]
        if name in seen or not os.path.isdir(ROOT + d):
            continue
        seen.add(name)
        files = sorted(rglob(d + '/**/*.tally', recursive=True))
        asks, opts = [], []
        if '--ask' in rest:
            i = rest.index('--ask')
            opts, asks = rest[:i], rest[i + 1:]
        else:
            opts = rest
        assert not any(a.startswith('-') for a in asks), (name, rest)
        nr = 'benchmarks.md row 14' if name in ('mc', 'c1_r2') else ''
        old = OLD + 'X-%s.out' % name
        if not asks:  # a run of all its asks: the asks are those its baseline printed (snap11's when snap16 printed none)
            asks = ids_of(old) if os.path.exists(ROOT + old) else []
            if not asks:
                asks = ids_of(I + 'amendments/out11/%s.out' % name)
        runs.append(('X', 'X-' + name, old, I + 'amendments', files, asks, opts, nr))


def question(files, cwd, ask):
    for f in files:
        p = ROOT + f
        stem = os.path.basename(p)[:-len('.tally')] if p.endswith('.tally') else ''
        m = re.match(r'^(?:asks-)?([A-Z][A-Z0-9]*-\d+[a-z]?)$', stem)
        if not m:
            continue
        if re.search(r'(?m)^\s*ask %s\b' % re.escape(ask), open(p, encoding='utf-8', errors='replace').read()):
            return m.group(1)
    return ''


# asks no file named <question>.tally declares (4.5c, the join gap): their question by the ask's name prefix, from
# session 4's rows (rows/A.check.md, B.check.md, E1.check.md) and the files' notes
FILL = [('A', 'mca05_', 'A-05'), ('A', 'mca_a10_', 'A-10'), ('B', 'o3_', 'O:B-3'), ('B', 'b12_', 'B-12'),
        ('B', 'b10_', 'B-10'), ('E1', 'e1_02_', 'E1-02'), ('E1', 'tx2_', 'E1-02')]


def fill(H, ask):
    for h, p, q in FILL:
        if h == H and ask.startswith(p):
            return q
    return ''


# known faults (tmp/tests/known-faults.md): the ask is checked in its run, with the run's asks, so the stored answer
# still guards it; the ask alone is a second check, '<id>~alone', expected to fail until the fault is fixed
KNOWN = {('D', 'd05_quarters'): 'KF-01'}

# one check per ask; an ask in several runs of one household gets the run's name after '@'
count = {}
for H, name, old, cwd, files, asks, opts, nr in runs:
    for a in asks:
        count[(H, a)] = count.get((H, a), 0) + 1
vis_s = {}  # a run's seconds on CPython 3.14 this morning (session 5, its copies of the inputs), where stored
for f in glob.glob(ROOT + SNAP + 'session5-visible/*.out'):
    t = [l for l in open(f, encoding='utf-8') if l.startswith('# rc ')]
    m = re.search(r', (\d+) s,', t[-1]) if t else None
    if m:
        vis_s[os.path.basename(f)[:-4]] = m.group(1)
for H, name, old, cwd, files, asks, opts, nr in runs:
    for a in asks:
        cid = 'I:%s:%s' % (H, a) + ('@' + name.split('-', 1)[1] if count[(H, a)] > 1 else '')
        note = ['run=%s' % name, 'run_asks=%d' % len(asks)]
        if name in vis_s:
            note.append('run_cpython_s=%s' % vis_s[name])
        if nr:
            note.insert(0, 'not-run: ' + nr)
        if not os.path.exists(ROOT + old) or os.path.getsize(ROOT + old) < 2:
            note.append('no-baseline')
        q = question(files, cwd, a) or fill(H, a)
        kf = KNOWN.get((H, a))
        if not kf:
            add(id=cid, layer='integration', cwd=cwd, files=' '.join(files), asks=a, options=' '.join(opts),
                baseline=old, block=a, question=q, note=' '.join(note))
            continue
        add(id=cid, layer='integration', cwd=cwd, files=' '.join(files), asks=' '.join(asks), options=' '.join(opts),
            baseline=old, block=a, question=q,
            note=' '.join(['known-fault: %s, run with its run\'s asks; alone it is %s~alone' % (kf, cid)] + note))
        add(id=cid + '~alone', layer='integration', cwd=cwd, files=' '.join(files), asks=a, options=' '.join(opts),
            baseline=old, block=a, question=q,
            note=' '.join(['xfail: %s, the ask alone; `fixed` when it matches the baseline' % kf] + note))

# --- the split (4.5c): capability copies under integration/split/, each differing from its original only in the
# knob lines knobs.md names (knobdiff.py). split.tsv: K, the original checks (an id glob), the original dir, the copy,
# 'twin' (the original's baseline is kept: a byte-equal judged twin) or 'J' (no twin: run.py records `ran`, J marks)
import fnmatch
SPLIT = ROOT + I + 'split/split.tsv'
if os.path.exists(SPLIT):
    by_id = {r['id']: r for r in rows}
    for l in open(SPLIT, encoding='utf-8'):
        if not l.strip() or l.startswith('#'):
            continue
        k, pat, old, new, judge = l.rstrip('\n').split('\t')[:5]
        if judge == 'per-ask':  # a whole-output check split into one check per ask (the asks-per-run knob), each
            # against its own blocks of the same baseline; only the asks in column 6 run, the rest are listed as load
            r = by_id[pat]
            runs = l.rstrip('\n').split('\t')[5].split()
            every = ids_of(r['baseline'])
            assert set(runs) <= set(every), (k, set(runs) - set(every))
            for a in every:
                n = dict(r, id='%s~%s@%s' % (pat, a, k.lower()), asks=a, block=a, baseline_s='')
                n['note'] = ('split: %s of %s, one ask (twin = its baseline)' % (k, pat) if a in runs else
                             'not-run: %s, an ask of %s left to its load twin (scale)' % (k, pat))
                assert n['id'] not in by_id, n['id']
                by_id[n['id']] = n
                rows.append(n)
            continue
        if old == '-' and judge.startswith('twin='):  # the same texts, one ask per check, against a judged run that is
            # not the original's baseline (session 4's snap11 output its judges marked), e.g. a benchmark run's asks
            twin = judge[len('twin='):]
            for r in [r for r in list(by_id.values()) if not r['note'].startswith(('split:', 'xfail: split')) and
                      (pat in r['note'].split() if pat.startswith('run=') else fnmatch.fnmatchcase(r['id'], pat))]:
                n = dict(r, id=r['id'].split('@')[0] + '@' + k.lower(), baseline=twin, baseline_s='')
                n['note'] = 'split: %s of %s, the ask alone (twin = %s, judged by session 4) %s' % (
                    k, r['id'], twin, ' '.join(w for w in r['note'].split() if w.startswith('run=')))
                assert n['id'] not in by_id, n['id']
                by_id[n['id']] = n
                rows.append(n)
            continue
        old, new = old.rstrip('/') + '/', new.rstrip('/') + '/'
        for r in [r for r in list(by_id.values()) if not r['note'].startswith(('split:', 'xfail: split')) and
                  (pat in r['note'].split() if pat.startswith('run=') else fnmatch.fnmatchcase(r['id'], pat)) and
                  ((r['cwd'].rstrip('/') + '/').startswith(old) or old in r['files'])]:
            if r['note'].startswith('not-run') and judge.startswith('twin'):
                continue
            n = dict(r)
            n['id'] = r['id'].split('@')[0].replace('~alone', '') + '@' + k.lower()
            if r['id'].endswith('~alone'):
                n['id'] += '~alone'
            n['cwd'] = (r['cwd'].rstrip('/') + '/').replace(old, new).rstrip('/')
            n['files'] = ' '.join(f.replace(old, new) for f in r['files'].split())
            assert n['files'] != r['files'] or n['cwd'] != r['cwd'], (k, r['id'])
            n['baseline'] = r['baseline'] if judge.startswith('twin') else ''
            jb = SNAP + '4.5c/' + n['id'].replace('/', '__').replace(':', '_') + '.out'
            if judge == 'J' and os.path.exists(ROOT + jb):  # J passed it (tmp/session4_5/c/judge/verdicts.md): its output guards it
                n['baseline'] = jb
            n['baseline_s'] = ''
            keep = ' '.join(w for w in r['note'].split() if w.startswith(('run=', 'xfail:', 'known-fault:')))
            n['note'] = ('xfail: ' if r['note'].startswith('xfail:') else '') + \
                'split: %s of %s (%s) %s' % (k, r['id'], 'twin = its baseline' if judge.startswith('twin') else 'no twin: J marks', keep) + \
                (' cmp=no-left-out' if judge == 'twin-cut' else '')
            assert n['id'] not in by_id, n['id']
            by_id[n['id']] = n
            rows.append(n)

# --- the fill (4.5c): C2's blind texts for cases 1.1, one check per run of integration/new/ASKS.md; no baseline
# until J passes a run (then its judged output under baselines/4.5c/ is the baseline)
NEW = I + 'new/'
for l in open(ROOT + NEW + 'runs.tsv', encoding='utf-8') if os.path.exists(ROOT + NEW + 'runs.tsv') else []:
    if not l.strip() or l.startswith('#'):
        continue
    cid, layer, recipe, cwd, extra, asks, note = l.rstrip('\n').split('\t')
    files = []
    if recipe != '-':
        rl = [x.strip() for x in open(ROOT + I + 'recipes/%s.txt' % recipe, encoding='utf-8') if x.strip()]
        cwd = [x[4:] for x in rl if x.startswith('cwd ')][0]
        files = [x if x.startswith(('engine/', 'tmp/')) else cwd + '/' + x  # a recipe line relative to its cwd
                 for x in rl if not x.startswith(('#', 'cwd ', 'options '))]
    files += [NEW + f + '.tally' for f in extra.split()]
    base = SNAP + '4.5c/' + cid.replace('/', '__').replace(':', '_') + '.out'
    add(id=cid, layer=layer, cwd=cwd, files=' '.join(files), asks=asks, baseline=base if os.path.exists(ROOT + base) else '',
        question=cid.split(':')[-1].split('~')[0], note=(note + ' | ' if note else '') + 'fill: C2\'s blind text (4.5c), cases 1.1')

# --- independent keys (4.5c, gap item 6): key scripts under integration/keys-extra/, run by run.py as `script` checks
KX = I + 'keys-extra/lh11_cmp01.py'
for mid, ins in (('LH-11', ['engine/spec/expected/LH-11.md']),
                 ('CMP-01', ['engine/spec/expected/CMP-01.md', 'engine/spec/models/CMP-01.tally',
                             'engine/spec/MIGRATION.md', I + 'models-1.0/CMP-01.tally'])):
    add(id='I:M:%s~key' % mid, layer='integration', cwd='.', files=' '.join([KX] + ins),
        asks=mid, options='script', baseline=KX, question=mid,
        note='key-script: independent Decimal key of the rows the freeze re-keyed from prints (freeze/KEYS.md rows %s)'
             % ('1-3' if mid == 'LH-11' else '4-5'))

# --- retired from the quick set (4.5c, the user's calls of 2026-10-29): the rows stay, ids are for life; a `not-run:`
# note keeps run.py from running them, and load/load.tsv lists each with its load twin
RETIRE = [
    ('I:B:*@k3', 'retired 4.5c: K3 cut proved the run end is scale (24 of 27 same bar `left out:` lines, B-05\'s 3 over) '
                 'but buys no time (44 s against 45 s); the record is tmp/session4_5/c/progress.md'),
    ('I:X:*@k10', 'retired 4.5c: over 60 s with the ask alone; load L:10 (the user: load rows only)'),
    ('I:X:*@k11', 'retired 4.5c: over 60 s with the ask alone; load L:11 (the user: load rows only)'),
    ('I:B:b12_share*@k5', 'retired 4.5c: over 60 s at 3 paths to 2027-01-31; load L:5 (the user: load rows only)'),
    ('I:B:b12_shortfall*@k5', 'retired 4.5c: over 60 s at 3 paths to 2027-01-31; load L:5 (the user: load rows only)'),
    ('I:B:b12_mortgage*@k5', 'retired 4.5c: over 60 s at 3 paths to 2027-01-31; load L:5 (the user: load rows only)'),
    ('I:B:b12_liquid@k5', 'retired 4.5c: over 60 s at 1 path to 2027-01-31; load L:5 (the user: load rows only)'),
    ('I:light/LH-16-light', 'retired 4.5c: split by K13 into its 22 asks (I:light/LH-16-light~*@k13); the whole run is load L:12'),
    ('I:B:b05_as_was', 'load 4.5c (the user: measure once, list as load): over 60 s at every knob; L:16'),
    ('I:B:b05_balance', 'load 4.5c (the user: measure once, list as load): over 60 s at every knob; L:16'),
    ('I:B:b05_instalment', 'load 4.5c (the user: measure once, list as load): over 60 s at every knob; L:16'),
]
for r in rows:
    for pat, why in RETIRE:
        if fnmatch.fnmatchcase(r['id'], pat) and not r['note'].startswith('not-run'):
            r['note'] = 'not-run: ' + why + ' | ' + r['note']

# --- the key column (4.5c, the weaker-key labels): empty = keyed (the case set's key scripts, the old models'
# expected files, the probe cards' keys_probes.py); otherwise what stands behind the baseline
def key_label(r):
    i = r['id']
    if i.startswith('I:X:pit37@v4_fast'):
        return 'interpreter-keyed: X change 4 cells 42-44 (freeze/KEYS.md rows 6-8); the rest hand arithmetic, weaker key'
    if i.startswith('I:X:'):
        return 'hand arithmetic, weaker key (amendments/README.md); X change 5 cells 9-16, 20, 24 are interpreter-keyed, in the walk run only (load L:10)'
    if i.startswith(('U:surf/', 'U:set/surf')):
        return 'no key by design (SURF: a writing rule; the baseline catches changes, not errors)'
    if i.startswith('I:hardset/'):
        return 'no key: a blind text; the baseline is snap16\'s print (catches changes only)'
    if i.endswith('~key'):
        return 'independent key script (gap item 6)'
    if i in ('I:M:LH-11', 'I:M:CMP-01'):
        return 'keyed; the rows the freeze re-keyed from prints are also keyed independently by %s~key' % i
    if r['baseline'].startswith(SNAP + '4.5c/'):
        return ('no key (O:B-3, a gap-log observation): J read it against the papers; the output is the baseline'
                if '@k5' in i and ':o3_' in i else
                'marked by J against the key (4.5c, judge/verdicts.md); the passed output is the baseline')
    if not r['baseline'] and not r['note'].startswith('not-run'):
        return 'no byte-equal twin: J marks the run against the key'
    return ''


for r in rows:
    r['key'] = key_label(r)

ids = [r['id'] for r in rows]
dup = sorted({i for i in ids if ids.count(i) > 1})
assert not dup, dup
for r in rows:
    for c in COLS:
        assert '\t' not in r[c] and '\n' not in r[c], (r['id'], c)
with open(OUT, 'w', encoding='utf-8') as f:
    f.write('\t'.join(COLS) + '\n')
    for r in rows:
        f.write('\t'.join(r[c] for c in COLS) + '\n')
if '--counts' in sys.argv or True:
    from collections import Counter
    c = Counter()
    for r in rows:
        k = r['layer'] + ' ' + (r['id'].split(':')[1] if r['id'].startswith('I:') and r['id'].count(':') == 2 else
                                r['id'].split('/')[0])
        c[(k, 'not-run' in r['note'])] += 1
    for (k, nr), n in sorted(c.items()):
        print('%-28s %s %d' % (k, 'not-run' if nr else 'run    ', n))
    print('%d rows -> %s' % (len(rows), OUT))
