"""Milestone M0's vectors (issue #56; MILESTONE-M0.md): refcore's runs over every text set checks.tsv names, and over
M0's own fixtures, for the format (row 4). Standard library only.

A text set is a (cwd, files) pair of a check of checks.tsv that runs a reality (its `not-run:` load rows included: their
texts load and reprint, only their asks are never run): 315 sets, each named by the first of its checks' ids. To them
come the format fixtures of m0/format/ (fixtures.tsv: one invented text per rule of L §2.4, each written off its
canonical form in the one way the rule fixes). A set's file table holds the files its command line names, as named,
and the members a joint model's `use` lines name, as INTERFACE.md §2 resolves them.

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
output is the same. A fixture of m0/format/ must differ from its canonical text (else it tests nothing).

The runner embeds format.refcore and m0_texts.rs (runner/src/m0.rs), generated here: one include_str! per text a
table names, so the tests read no file (INTERFACE.md §4) and nothing is copied. milestone/m0_format.rs holds the
engine's reprint to refcore's, and its reprint of its own canonical text to that text.

    python3 -I -B engine/tests/m0/vectors.py              -> writes format.refcore and runner/src/m0_texts.rs
    python3 -I -B engine/tests/m0/vectors.py --check      -> exit 1 if either isn't what refcore's runs give
    python3 -I -B engine/tests/m0/vectors.py --show GLOB  -> refcore's reprint of the sets whose id matches
    python3 -I -B engine/tests/m0/vectors.py --only GLOB  -> runs and checks only those sets; writes nothing
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
        sets.setdefault((c['cwd'], c['files']), []).append(c['id'])
    out = []
    for (cwd, files), ids in sets.items():
        sid = min(ids)
        args = [R.here({'cwd': cwd}, f) for f in files.split()]
        out.append({'id': sid, 'group': group(sid), 'cwd': cwd, 'args': args, 'rule': ''})
    with open(FIXTURES, encoding='utf-8', newline='') as f:
        rows = list(csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE))
    for r in rows:
        out.append({'id': 'format/' + r['name'], 'group': 'format/' + r['name'], 'cwd': FIXTURE_CWD,
                    'args': [r['name'] + '.tally'], 'rule': r['rule']})
    ids = [s['id'] for s in out]
    if len(set(ids)) != len(ids):
        sys.exit('two sets share an id')
    for s in out:
        cwd = s['cwd'].rstrip('/') or '.'
        table = [(a, os.path.normpath(os.path.join(cwd, a))) for a in s['args']]
        table += members(cwd, s['args'])
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
        if e['rc'] != 0 or not secs or not tail.startswith('sha256 '):
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
    out = ['//! The texts M0\'s vectors name (`m0/format.refcore`), by their path under `engine/tests/`, sorted:',
           '//! generated by `m0/vectors.py` (its `--check` holds this file to it), so the tests read no',
           '//! file (`INTERFACE.md` §4) and no text is copied. Never edited by hand.',
           '',
           '/// Each text: its path under `engine/tests/` and its content.',
           'pub static TEXTS: [(&str, &str); %d] = [' % len(paths)]
    for p in paths:   # every pair is wider than rustfmt's 60-column call width, so it stands on four lines
        out += ['    (', '        "%s",' % p, '        include_str!("../../%s"),' % p, '    ),']
    out.append('];')
    return '\n'.join(out) + '\n'


def main(argv):
    R.need_watchdog()
    sets = text_sets()
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
    res = runs(sets)
    data, rs = stored(sets, res), texts_rs(sets)
    if '--check' in argv:
        old = open(FORMAT, 'rb').read() if os.path.exists(FORMAT) else b''
        olds = open(TEXTS_RS, encoding='utf-8').read() if os.path.exists(TEXTS_RS) else ''
        if old != data or olds != rs:
            print('m0/format.refcore or runner/src/m0_texts.rs is not what refcore\'s runs give: run m0/vectors.py')
            return 1
        print('m0/format.refcore: %d sets, refcore\'s bytes; each round-trips on refcore' % len(sets))
        return 0
    with open(FORMAT, 'wb') as f:
        f.write(data)
    with open(TEXTS_RS, 'w', encoding='utf-8', newline='') as f:
        f.write(rs)
    print('m0/format.refcore: %d sets written (%d texts in m0_texts.rs)' % (len(sets), rs.count('include_str!')))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
