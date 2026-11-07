"""Milestone M0's vectors (issue #56; MILESTONE-M0.md): refcore's runs over every text set checks.tsv names, and over
M0's own fixtures, for the format (row 4) and the load (rows 1 and 2). Standard library only.

A text set is a (cwd, files) pair of a check of checks.tsv that runs a reality (its `not-run:` load rows included: their
texts load and reprint, only their asks are never run): 315 sets, each named by the first of its checks' ids. To them
come the format fixtures of m0/format/ (fixtures.tsv: one invented text per rule of L §2.4, each written off its
canonical form in the one way the rule fixes; a row may name its run folder under m0/format/ and its arguments, the
columns `cwd` and `args`, empty for m0/format/ and `<name>.tally`). A set's file table holds the files its command
line names, as named (a directory given: each of its files as `<dir>/<path>`, as the binary names them), and the
members a joint model's `use` lines name, as INTERFACE.md §2 resolves them.

The format half: `reprint <files>` on the frozen refcore (PyPy, through run.py's own run_one: its 90 s budget, its
cache, the watchdog). Its output is a section per file (`--- <path>` and the file's canonical text, L §2.4), then the
reality's `sha256` line (E §3.1 step 1). Stored per set in format.refcore, eight netstrings:

    id, group, cwd, the argument words joined by single spaces,
    the file table (one line per file: the path as the run names it, a tab, the path under engine/tests/),
    refcore's exit code,
    its sections (one line per section: the path, a tab, the canonical text's length in bytes, a tab, its digest,
    two polynomial residues modulo 61-bit primes in 2 × 16 lower-case hex digits; the canonical text is the
    section's lines, each ending in a newline),
    and the rest of its output, verbatim (the sha256 line; a refusal's whole output)

so every expected byte is refcore's, held by its digest (the texts would be 6 MB more). Before it writes anything, the
script holds refcore to the round trip (DONE gate 6): each canonical text, written back in place of its file
(engine/tests/out/m0/canon/, a mirror of the repository's paths), reprints to itself, byte for byte, and the whole
output is the same. A fixture of m0/format/ must differ from its canonical text (else it tests nothing). Since tally
2.1.6 a text that has no canonical form is refused as load refuses it (L §2.4, E §3.1 step 1): the sets
REPRINT_REFUSED names, each with its ruling, store their one refusal line and exit 1, and no other set may refuse.

The load half (rows 1 and 2): M0's engine evaluates nothing, and its tests must stay green after M1, so a set's load
is seen through the load probe, `run <files> --ask zz_m0_probe`, an ask no text declares: load runs first, a load
refusal prints as in any run, and after a clean load the run prints `refused: REQ-ASK no ask zz_m0_probe` alone (E §11,
tally 2.1.3, #58). A scenario's own load refusal refuses only that scenario, so the probe can't show it; for each
scenario a stored output shows refused, the set gets a second vector, `<set>#<scenario>`, that asks the asks that
scenario's file declares (refused with it, which needs no evaluation), when it declares any. Row 1's 10 checks are vectors of their own: the
check's command line, and its stored baseline as the expected output (refcore's run of it must print the baseline).
Stored in load.refcore, seven netstrings each: id, group, cwd, the argument words, the file table, refcore's exit
code, its whole output. Before it writes, the script ties every probe to every stored output of checks.tsv that names
its set: a stored output whose first line is a refusal (`refused: …`) is the probe's whole output; any other gives
the REQ-ASK line; each `scenario <s>: refused: …` line it holds is printed by that set's vector for s (or listed, if
the scenario declares no ask: only a full run shows it).

The runner embeds format.refcore, load.refcore and m0_texts.rs (runner/src/m0.rs), generated here: one include_str!
per text a table names, so the tests read no file (INTERFACE.md §4) and nothing is copied. milestone/m0_format.rs
holds the engine's reprint to refcore's, and its reprint of its own canonical text to that text; milestone/m0_load.rs
holds the engine's exit code and whole output to each load vector's, `warning:` lines left out on both sides (the
warnings are M1's, BUILD-ORDER § M1).

    python3 -I -B engine/tests/m0/vectors.py              -> writes format.refcore, load.refcore, runner/src/m0_texts.rs
    python3 -I -B engine/tests/m0/vectors.py --check      -> exit 1 if any isn't what refcore's runs give
    python3 -I -B engine/tests/m0/vectors.py --show GLOB  -> refcore's reprint of the sets whose id matches
    python3 -I -B engine/tests/m0/vectors.py --only GLOB  -> runs and checks only those sets' format; writes nothing
    python3 -I -B engine/tests/m0/vectors.py --probe GLOB ASKS -> refcore's `run <files> --ask ASKS` of those sets
"""
import csv
import fnmatch
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import run as R  # noqa: E402

FORMAT = os.path.join(HERE, 'format.refcore')
LOAD = os.path.join(HERE, 'load.refcore')
PROBE = 'zz_m0_probe'
PROBED = 'refused: REQ-ASK no ask %s\n' % PROBE
# BUILD-ORDER § M0 Green, its first line: the 10 load refusals
REFUSALS = ['U:law/LAW-13/rate25', 'U:rand/RAND-04/plan_b', 'U:rule/RULE-09/inpath', 'U:scen/SCEN-01/amend',
            'U:scen/SCEN-02/kept', 'U:scen/SCEN-03/infl8', 'U:scen/SCEN-04/crash', 'U:surf/SURF-04/redefine',
            'U:rule/RULE-04', 'U:own/OWN-11/p21-signed']
SCENARIO_REFUSED = re.compile(r'^scenario ([a-z][a-z0-9_]*): refused: ', re.M)
# the sets whose reprint refuses, as load refuses them, before it prints anything (tally 2.1.6): a joint model's
# member absent or changed (E §3.1 step 1, #74, P15); a duplicated id or an unknown prefix, which has no canonical
# form (L §2.4, #91, P22). Each stores its one refusal line (no section) and its exit code; the round trip of a
# refusal is the same refusal. Any other set must reprint, and each of these must refuse.
REPRINT_REFUSED = {
    'U:set/surf/SURF-04/SURF-04': '#91: its scenario declares the base\'s contrib again (LOAD-DUP)',
    'format/member_absent': '#74: a joint model whose member file is absent (LOAD-PIN)',
    'format/member_changed': '#74: a joint model whose member\'s text no longer matches its pin (LOAD-PIN)',
    'format/dup_id': '#91: a source declared twice, with two texts (LOAD-DUP; the fuzz input b943 minimized)',
    'format/dup_kind': '#91: a kind declared twice under two parents (LOAD-DUP; the fuzz input e523 minimized)',
    'format/unknown_prefix': '#91: an input keyed by an index under a prefix no use line names (LOAD-NAME; the '
                             'fuzz input c5a8 minimized)',
    'format/same_key_signed': '#88, #90: a same side keyed by a signed literal, no sref (LOAD-PARSE at parse, before '
                              'the canonical order of same sides)',
    'format/member_outside_parent': '#105 (held): refcore looks the member up from the printed name, LOAD-PIN (no such '
                                    'file), where run finds it',
    'format/member_nested_absent': '#105 (held): refcore refuses a nested member\'s absent member LOAD-NAME, the engine '
                                   'LOAD-PIN as its run does',
}
REFUSED_LINE = re.compile(r'refused: (?:member [a-z][a-z0-9_]*: )*[A-Z]+(?:-[A-Z]+)+ [^\n]*\n')  # a member's: INTERFACE §2
TEXTS_RS = R.T + 'runner/src/m0_texts.rs'
FIXTURES = os.path.join(HERE, 'format', 'fixtures.tsv')
FIXTURE_CWD = 'engine/tests/m0/format'
CANON = 'engine/tests/out/m0/canon/'   # run output: git ignores out/, check-frozen skips it
PREFIX = 'engine/tests/'
JOBS = 2
# the digest: two polynomial residues of the bytes, modulo the primes 2^61 - 1 and 2^61 - 31 (the runner's
# m0::digest computes them with checked arithmetic, R-23)
LANES = ((2 ** 61 - 1, 1099511628211), (2 ** 61 - 31, 1540483477))
USE = re.compile(r'^  use [a-z][a-z0-9_]* = "([^"]*)" sha256 ', re.M)


def digest(b):
    """Two residues, h = (h * base + byte) mod p from h = 0, each as 16 hex digits (the runner's m0::digest)."""
    out = ''
    for p, base in LANES:
        h = 0
        for x in b:
            h = (h * base + x) % p
        out += '%016x' % h
    return out


def ns(b):
    b = b if isinstance(b, bytes) else b.encode('utf-8')
    return b'%d:' % len(b) + b + b','


def group(i):
    """The corpus group a set's tests run it in (milestone/m0_format.rs, m0_load.rs)."""
    if i.startswith('U:set/'):
        return 'probe-sets'
    if i.startswith('U:'):
        return 'probes'
    if i.startswith('I:M:'):
        return 'models'
    if i.startswith('I:light/'):
        return 'light'
    if i.startswith('I:hardset/') or re.match(r'I:S\d:', i):
        return 'blind'
    m = re.match(r'I:(A|B|C|D|E1|E2|X):', i)
    if m:
        return 'household-' + m.group(1)
    sys.exit('%s: no corpus group' % i)


def collect(p):
    """refcore's loader.collect for one path: a file, or every .tally file under a directory (walk order)."""
    if not os.path.isdir(p):
        return [p]
    out = []
    for root, dirs, fs in os.walk(p):
        dirs.sort()
        out.extend(os.path.join(root, f) for f in sorted(fs) if f.endswith('.tally'))
    return out


def members(cwd, named):
    """The files a joint model's use lines name, as INTERFACE.md §2 resolves them: [(path as the run reads it, path
    from the repository root)], each member's own use lines followed too."""
    out, todo, seen = [], list(named), set(named)
    while todo:
        f = todo.pop(0)
        with open(R.ROOT + os.path.normpath(os.path.join(cwd, f)), encoding='utf-8') as fh:
            src = fh.read()
        for use in USE.findall(src.split('\n\n', 1)[0]):
            p = os.path.normpath(os.path.join(os.path.dirname(f), use))
            if not os.path.exists(os.path.join(R.ROOT, cwd, p)):
                continue   # a member that isn't there: refcore leaves it out (its pin refuses it at load)
            for m in collect(os.path.join(R.ROOT, cwd, p)):
                key = os.path.join(p, os.path.relpath(m, os.path.join(R.ROOT, cwd, p))) if os.path.isdir(
                    os.path.join(R.ROOT, cwd, p)) else p
                key = key.replace(os.sep, '/')
                if key not in seen:
                    seen.add(key)
                    out.append((key, os.path.normpath(os.path.join(cwd, key))))
                    todo.append(key)
    return out


def text_sets():
    """Every (cwd, files) pair of a check that runs a reality, named by the first of its ids, then the fixtures."""
    sets = {}
    for c in R.load_checks():
        if not c['files'].strip() or c['layer'] == 'interpreter' or R.script(c):
            continue
        sets.setdefault((c['cwd'], c['files']), []).append(c)
    out = []
    for (cwd, files), checks in sets.items():
        sid = min(c['id'] for c in checks)
        args = [R.here({'cwd': cwd}, f) for f in files.split()]
        out.append({'id': sid, 'group': group(sid), 'cwd': cwd, 'args': args, 'rule': '', 'checks': checks})
    with open(FIXTURES, encoding='utf-8', newline='') as f:
        rows = list(csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE))
    for r in rows:
        cwd = FIXTURE_CWD + ('/' + r['cwd'] if r.get('cwd') else '')
        args = r['args'].split() if r.get('args') else [r['name'] + '.tally']
        out.append({'id': 'format/' + r['name'], 'group': 'format/' + r['name'], 'cwd': cwd,
                    'args': args, 'rule': r['rule'], 'checks': []})
    ids = [s['id'] for s in out]
    if len(set(ids)) != len(ids):
        sys.exit('two sets share an id')
    for s in out:
        cwd = s['cwd'].rstrip('/') or '.'
        table = []
        for a in s['args']:
            p = os.path.normpath(os.path.join(cwd, a))
            if os.path.isdir(R.ROOT + p):   # a directory given: its files, named <dir>/<path> as the binary names them
                table += [((a if a.endswith('/') else a + '/') + os.path.relpath(f, R.ROOT + p).replace(os.sep, '/'),
                           os.path.relpath(f, R.ROOT).replace(os.sep, '/')) for f in collect(R.ROOT + p)]
            else:
                table.append((a, p))
        table += members(cwd, [k for k, _ in table])
        for key, repo in table:
            if not repo.startswith(PREFIX) or not os.path.isfile(R.ROOT + repo):
                sys.exit('%s: %s is no file under engine/tests/' % (s['id'], key))
        s['table'] = table
    return sorted(out, key=lambda s: s['id'])


def check_of(s, cwd=None, root=''):
    """A run.py check that runs `reprint` on the set (its files under `root`, a mirror of the repository)."""
    cwd = s['cwd'] if cwd is None else cwd
    return {'id': s['id'], 'layer': 'interpreter', 'cwd': root + cwd, 'asks': '', 'baseline': '', 'block': '',
            'note': '', 'baseline_s': '', 'files': ' '.join(root + repo for _, repo in s['table']),
            'options': ' '.join(['reprint'] + s['args'])}


def sections(out):
    """refcore's reprint as [(path, canonical text)] and the rest (the sha256 line, or a refusal)."""
    lines = out.split('\n')
    if lines and lines[-1] == '':
        lines.pop()
    secs, i = [], 0
    while i < len(lines) and lines[i].startswith('--- '):
        name, j = lines[i][4:], i + 1
        while j < len(lines) and not lines[j].startswith('--- ') and not (
                j == len(lines) - 1 and lines[j].startswith('sha256 ')):
            j += 1
        secs.append((name, ''.join(l + '\n' for l in lines[i + 1:j])))
        i = j
    return secs, ''.join(l + '\n' for l in lines[i:])


def runs(sets):
    rc_hash = R.refcore_hash()

    def one(c):
        e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed on refcore' % c['id'])
        if 'REFCORE-NOT-YET' in e['out'] or 'Traceback' in e['out']:
            sys.exit('%s: refcore prints REFCORE-NOT-YET or crashes: a gap in refcore, not a target' % c['id'])
        return e
    with ThreadPoolExecutor(JOBS) as ex:
        first = list(ex.map(one, [check_of(s) for s in sets]))
    # the round trip on refcore: each canonical text in place of its file reprints to itself
    canons = []
    for s, e in zip(sets, first):
        secs, _ = sections(e['out'])
        canon = dict((os.path.normpath(os.path.join(s['cwd'].rstrip('/') or '.', n)), t) for n, t in secs)
        canons.append(canon)
        for _, repo in s['table']:
            dst = R.ROOT + CANON + repo
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            data = canon[repo].encode('utf-8') if repo in canon else open(R.ROOT + repo, 'rb').read()
            if not os.path.exists(dst) or open(dst, 'rb').read() != data:
                with open(dst, 'wb') as f:
                    f.write(data)
    with ThreadPoolExecutor(JOBS) as ex:
        again = list(ex.map(one, [check_of(s, root=CANON) for s in sets]))
    bad = 0
    for s, e, g, canon in zip(sets, first, again, canons):
        secs, tail = sections(e['out'])
        if s['id'] in REPRINT_REFUSED:
            if e['rc'] != 1 or secs or not REFUSED_LINE.fullmatch(e['out']) or g['out'] != e['out']:
                bad += 1
                print('FAIL %s: refcore\'s reprint should refuse (%s), in one line, again on its round trip: exit '
                      '%s: %s' % (s['id'], REPRINT_REFUSED[s['id']], e['rc'], e['out'][:200]))
        elif e['rc'] != 0 or not secs or not tail.startswith('sha256 '):
            bad += 1
            print('FAIL %s: refcore\'s reprint exits %s: %s' % (s['id'], e['rc'], e['out'][:200]))
        elif g['out'] != e['out'] or g['rc'] != e['rc']:
            bad += 1
            print('FAIL %s: refcore\'s reprint of its own canonical text differs: %s' % (
                s['id'], R.first_diff(e['out'], g['out'])))
        elif s['rule'] and all(open(R.ROOT + repo, encoding='utf-8').read() == t for repo, t in canon.items()):
            bad += 1
            print('FAIL %s: the fixture is already canonical, so it tests nothing' % s['id'])
    if bad:
        sys.exit('%d of %d sets fail on refcore: a STOP-REPORT, not a vector' % (bad, len(sets)))
    return first


def stored(sets, res):
    out = []
    for s, e in zip(sets, res):
        secs, tail = sections(e['out'])
        out.append(ns(s['id']) + ns(s['group']) + ns(s['cwd']) + ns(' '.join(['reprint'] + s['args'])) +
                   ns(''.join('%s\t%s\n' % (k, repo[len(PREFIX):]) for k, repo in s['table'])) + ns(str(e['rc'])) +
                   ns(''.join('%s\t%d\t%s\n' % (n, len(t.encode('utf-8')), digest(t.encode('utf-8')))
                              for n, t in secs)) + ns(tail))
    return b''.join(out)


def texts_rs(sets):
    """runner/src/m0_texts.rs: every text a table names, by its path under engine/tests/, sorted (rustfmt's layout)."""
    paths = sorted({repo[len(PREFIX):] for s in sets for _, repo in s['table']})
    out = ['//! The texts M0\'s vectors name (`m0/format.refcore`, `m0/load.refcore`), by their path under',
           '//! `engine/tests/`, sorted: generated by `m0/vectors.py` (its `--check` holds this file to it),',
           '//! so the tests read no file (`INTERFACE.md` §4) and no text is copied. Never edited by hand.',
           '',
           '/// Each text: its path under `engine/tests/` and its content.',
           'pub static TEXTS: [(&str, &str); %d] = [' % len(paths)]
    for p in paths:   # every pair is wider than rustfmt's 60-column call width, so it stands on four lines
        out += ['    (', '        "%s",' % p, '        include_str!("../../%s"),' % p, '    ),']
    out.append('];')
    return '\n'.join(out) + '\n'


def probe_check(s, asks):
    """A run.py check that runs `run <files> --ask <asks>` on the set (every table file hashed for the cache)."""
    return dict(check_of(s), options=' '.join(['run'] + s['args'] + ['--ask'] + asks.split()))


def baseline(c):
    """The stored output of check `c`, or None (a load row's run, or a check with no baseline)."""
    if c['note'].startswith('not-run:') or not c['baseline'] or not os.path.isfile(R.ROOT + c['baseline']):
        return None
    with open(R.ROOT + c['baseline'], 'rb') as f:
        return R.text(f.read())


def scenario_lines(out):
    """[(scenario, its whole line)] of each `scenario <s>: refused: …` line of a stored output."""
    return [(m.group(1), out[m.start():out.find('\n', m.start())]) for m in SCENARIO_REFUSED.finditer(out)]


def scenario_asks(s, scen):
    """The asks the file of scenario `scen` in set `s` declares, in its order ([] when it declares none)."""
    for _, repo in s['table']:
        with open(R.ROOT + repo, encoding='utf-8') as f:
            src = f.read()
        if src.startswith('scenario %s over ' % scen):
            return re.findall(r'^ask ([a-z][a-z0-9_]*):', src, re.M)
    sys.exit('%s: no file of the set is scenario %s' % (s['id'], scen))


def load_vectors(sets):
    """[(vector, the run.py check that runs it)]: a probe per corpus set, a vector per scenario a stored output shows
    refused that declares asks, and row 1's checks; and the refused scenarios no vector shows."""
    out, unseen = [], []
    of = {c['id']: s for s in sets for c in s['checks']}
    for s in sets:
        if not s['checks']:
            continue   # a format fixture
        out.append(({'id': s['id'], 'group': s['group'], 'set': s}, probe_check(s, PROBE)))
        scens = sorted({n for c in s['checks'] for n, _ in scenario_lines(baseline(c) or '')})
        for scen in scens:
            asks = scenario_asks(s, scen)
            if asks:
                out.append(({'id': s['id'] + '#' + scen, 'group': s['group'], 'set': s, 'scenario': scen},
                            probe_check(s, ' '.join(asks))))
            else:
                unseen.append('%s: scenario %s declares no ask, so only a full run shows its refusal' % (s['id'], scen))
    for rid in REFUSALS:
        c = next(c for c in of[rid]['checks'] if c['id'] == rid)
        out.append(({'id': 'refusal/' + rid, 'group': 'refusal/' + rid, 'set': of[rid], 'baseline': baseline(c)}, c))
    return out, unseen


def load_runs(vecs):
    rc_hash = R.refcore_hash()

    def one(vc):
        e = R.run_one(vc[1], 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
        if e['over'] or e['rc'] is None or e['rc'] < 0:
            sys.exit('%s: over the budget or killed on refcore' % vc[0]['id'])
        if 'REFCORE-NOT-YET' in e['out'] or 'Traceback' in e['out']:
            sys.exit('%s: refcore prints REFCORE-NOT-YET or crashes: a gap in refcore, not a target' % vc[0]['id'])
        return e
    with ThreadPoolExecutor(JOBS) as ex:
        return list(ex.map(one, vecs))


def tie(vecs, res):
    """Each load vector against every stored output of checks.tsv that names its set; the failures."""
    bad = []
    got = {v['id']: e for (v, _), e in zip(vecs, res)}
    for (v, _), e in zip(vecs, res):
        if v['id'].startswith('refusal/'):
            if e['out'] != v['baseline'] or e['rc'] != 1:
                bad.append('%s: refcore prints %r, its stored baseline %r' % (v['id'], e['out'][:200], v['baseline']))
            continue
        if 'scenario' in v:
            continue
        for c in v['set']['checks']:
            b = baseline(c)
            if b is None:
                continue
            first = b.split('\n', 1)[0]
            want = first + '\n' if first.startswith('refused: ') else PROBED
            if e['out'] != want or e['rc'] != 1:
                bad.append('%s: the probe prints %r; %s\'s stored output wants %r' % (v['id'], e['out'][:200],
                                                                                      c['id'], want))
            for scen, line in scenario_lines(b):
                g = got.get(v['id'] + '#' + scen)
                if g is not None and line not in g['out'].split('\n'):
                    bad.append('%s#%s: refcore doesn\'t print %r, which %s\'s stored output holds' % (
                        v['id'], scen, line, c['id']))
    return bad


def load_bytes(vecs, res):
    return b''.join(ns(v['id']) + ns(v['group']) + ns(c['cwd']) + ns(' '.join(R.argv_of(c))) +
                    ns(''.join('%s\t%s\n' % (k, repo[len(PREFIX):]) for k, repo in v['set']['table'])) +
                    ns(str(e['rc'])) + ns(e['out']) for (v, c), e in zip(vecs, res))


def main(argv):
    R.need_watchdog()
    sets = text_sets()
    if '--probe' in argv:
        glob = argv[argv.index('--probe') + 1]
        rc_hash = R.refcore_hash()
        for s in [s for s in sets if fnmatch.fnmatchcase(s['id'], glob)]:
            e = R.run_one(probe_check(s, argv[argv.index('--probe') + 2]), 'pypy', R.BUDGET, False, rc_hash,
                          R.T + 'cache')
            print('=== %s (exit %s)\n%s' % (s['id'], e['rc'], e['out']), end='')
        return 0
    if '--show' in argv or '--only' in argv:
        flag = '--show' if '--show' in argv else '--only'
        glob = argv[argv.index(flag) + 1] if argv.index(flag) + 1 < len(argv) else '*'
        sets = [s for s in sets if fnmatch.fnmatchcase(s['id'], glob)]
        res = runs(sets)
        for s, e in zip(sets, res):
            if flag == '--show':
                print('=== %s (exit %s)\n%s' % (s['id'], e['rc'], e['out']), end='')
        print('%d sets: refcore reprints each, and its reprint of its own canonical text is that text' % len(sets))
        return 0
    if '--load' in argv:   # the load half only, written nowhere: its vectors and the tie
        vecs, unseen = load_vectors(sets)
        res = load_runs(vecs)
        for (v, _), e in zip(vecs, res):
            if e['out'] != PROBED:
                print('=== %s (exit %s)\n%s' % (v['id'], e['rc'], e['out']), end='')
        bad = tie(vecs, res)
        print('\n'.join(['FAIL ' + b for b in bad] + ['unseen: ' + u for u in unseen]))
        print('%d load vectors, %d print the probe\'s REQ-ASK line, %d fail the tie' % (
            len(vecs), sum(e['out'] == PROBED for e in res), len(bad)))
        return 1 if bad else 0
    missing = sorted(set(REPRINT_REFUSED) - {s['id'] for s in sets})
    if missing:
        sys.exit('REPRINT_REFUSED names sets that don\'t exist: ' + ', '.join(missing))
    res = runs(sets)
    data, rs = stored(sets, res), texts_rs(sets)
    vecs, unseen = load_vectors(sets)
    lres = load_runs(vecs)
    bad = tie(vecs, lres)
    if bad:
        print('\n'.join('FAIL ' + b for b in bad))
        sys.exit('%d load vectors fail their tie to checks.tsv\'s stored outputs: a STOP-REPORT, not a vector' % len(bad))
    load = load_bytes(vecs, lres)
    if '--check' in argv:
        old = open(FORMAT, 'rb').read() if os.path.exists(FORMAT) else b''
        olds = open(TEXTS_RS, encoding='utf-8').read() if os.path.exists(TEXTS_RS) else ''
        oldl = open(LOAD, 'rb').read() if os.path.exists(LOAD) else b''
        if old != data or olds != rs or oldl != load:
            print('m0/format.refcore, m0/load.refcore or runner/src/m0_texts.rs is not what refcore\'s runs give: '
                  'run m0/vectors.py')
            return 1
        print('m0/format.refcore: %d sets, refcore\'s bytes; each round-trips on refcore; m0/load.refcore: %d '
              'vectors, each tied to the stored outputs' % (len(sets), len(vecs)))
        return 0
    with open(FORMAT, 'wb') as f:
        f.write(data)
    with open(TEXTS_RS, 'w', encoding='utf-8', newline='') as f:
        f.write(rs)
    with open(LOAD, 'wb') as f:
        f.write(load)
    print('m0/format.refcore: %d sets written (%d texts in m0_texts.rs); m0/load.refcore: %d vectors' % (
        len(sets), rs.count('include_str!'), len(vecs)))
    for u in unseen:
        print('not a vector: ' + u)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
