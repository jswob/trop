"""Milestone M4's vectors: refcore's whole output for each (issue #252; MILESTONE-M4.md). Standard library only.

Reads vectors.tsv, one vector per row, in m2/vectors.tsv's columns:

    id  group  check  cwd  files  asks  options  for

The first seven columns are kernel/vectors.py's, read and run by its own functions (imported, not copied): a vector
names a check of checks.tsv (`check`; the other columns empty) or is a fixture of its own (`cwd`, `files`, `asks`,
`options`). `group` is the runner test that holds it: `checks-…` in milestone/m4_checks.rs (73 of BUILD-ORDER § M4's 77
checks), `constructs-…` in milestone/m4_constructs.rs (the corners of M4's constructs, the paper check), `items-…` in
milestone/m4_items.rs (the in-scope backlog), `fixes-…` in milestone/m4_fixes.rs (a group review's findings, the spec's
answers beside them are spec.py's; and the performance groups' guard, `fixes-g6` and fence F7's `fixes-narrow`), one
test per group. `for` is the construct that turns it green
(MILESTONE-M4.md rows 1 to 3 and 5: `pins`, `accrue`, `keydiff`, `ratepin`, `tuple`, `monthcheck`, `explain`,
`runnone`, `agemonths`, `middot`, `pinwalk`, `keypin`, `oneref`; `narrow` for G6c's review B1, `backlog` for P23 and
P24's, G4's), `M7` for the vector held to M7 by name
(held.tsv), or `-` for a vector the engine already answers; it is the builder's work list and is not stored. A joint
model's members go into its file table as m3/vectors.py puts them (M0's `members`).

Each vector is run on the frozen refcore (PyPy, plain refcore, never the fast layer: run.py's run_one, its 90 s
budget, its cache, the watchdog) and written to vectors.refcore in kernel/vectors.refcore's six netstrings (id, group,
the argument words, the file table, the exit code, the output), which runner/src/m4.rs embeds. Before anything is
written, a vector that is a check of checks.tsv with a stored baseline is tied to it by run.py's own compare (a
household check by its ask's blocks, any other whole), so no expected byte is new. Every expected byte is refcore's.

    python3 -I -B engine/tests/m4/vectors.py              -> writes vectors.refcore
    python3 -I -B engine/tests/m4/vectors.py --check      -> exit 1 if vectors.refcore isn't what refcore prints
    python3 -I -B engine/tests/m4/vectors.py --engine     -> the release binary's whole output against refcore's
    python3 -I -B engine/tests/m4/vectors.py --times      -> refcore's seconds per vector, slowest first
    python3 -I -B engine/tests/m4/vectors.py --show GLOB  -> refcore's output for the vectors whose id matches
"""
import csv
import fnmatch
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(HERE)
sys.path.insert(0, TESTS)
import run as R  # noqa: E402

_spec = importlib.util.spec_from_file_location('kernel_vectors', os.path.join(TESTS, 'kernel', 'vectors.py'))
KV = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(KV)
KV.JOBS = 2  # at most two heavy jobs on the machine (the run's rule)

_m0 = importlib.util.spec_from_file_location('m0vectors', os.path.join(TESTS, 'm0', 'vectors.py'))
M0 = importlib.util.module_from_spec(_m0)
_m0.loader.exec_module(M0)
_KV_TABLE = KV.table


def table(c):
    """kernel/vectors.py's file table, then a joint model's members as m3/vectors.py's `table_of` adds them (M0's
    `members`, INTERFACE.md §2), so the engine reads a member from the table where refcore reads it from disk."""
    names = [R.here(c, f) for f in c['files'].split()]
    with_members = []
    for key, repo in M0.members(c['cwd'], names):
        with open(R.ROOT + repo, encoding='utf-8', newline='') as fh:
            with_members.append(KV.ns(key) + KV.ns(fh.read()))
    return _KV_TABLE(c) + b''.join(with_members)


KV.table = table  # kernel/vectors.py's stored_bytes and engine_look write and run M4's tables with the members

LIST = os.path.join(HERE, 'vectors.tsv')
STORED = os.path.join(HERE, 'vectors.refcore')
COLUMNS = KV.COLUMNS + ['for']
# MILESTONE-M4.md rows 1 to 3: the constructs the builder's groups build (an `as if` pin at a two-key def's keys,
# `accrue`, a `check` of two keys or periods, a rate pin's unit, `argmax`'s tuple; a plain `check` of two months,
# `explain` of a decision's `flows` or `count`, `RUN-NONE` in L §8.4's form, `age_months` at 9999-12, a joint run
# that leaves a member's `·` as written; G1's review B1, a per-tuple pin through a walk's read; G2's review B1 and
# B4, a pin's bare key or four-digit figure read as its target's key or period; P12, a built-in's known
# refusal before its holes, F1 a hole compounding code), and M7's hold; tally 2.3.1's (#259): `decides` (group 1, R1
# with S9, S16, S23: G1's build), `existence` (group 2, R6 and R2: G2's build), `g1fix` (G1's review's fixes)
FOR = {'-', 'pins', 'accrue', 'keydiff', 'ratepin', 'tuple', 'monthcheck', 'explain', 'runnone', 'agemonths',
       'middot', 'pinwalk', 'keypin', 'oneref', 'narrow', 'backlog', 'M7', 'decides', 'existence', 'g1fix'}


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLUMNS:
            sys.exit('vectors.tsv: the columns are %s, not %s' % (r.fieldnames, COLUMNS))
        out = list(r)
    for v in out:
        if v['for'] not in FOR:
            sys.exit('vectors.tsv: %s is for %r, not one of %s' % (v['id'], v['for'], ', '.join(sorted(FOR))))
    return out


def tie(pairs, res):
    """Each check of checks.tsv with a stored baseline: refcore's run of it compares `same` to it (run.py)."""
    checks = R.load_checks()
    for (v, c), e in zip(pairs, res):
        if v['check'] and c['baseline']:
            verdict, detail = R.compare(c, e['out'], checks)
            if verdict != 'same':
                sys.exit('%s: refcore no longer prints its stored baseline (%s)' % (v['id'], detail))


def main(argv):
    R.need_watchdog()
    pairs = KV.checks_of(rows())
    if '--show' in argv:
        i = argv.index('--show') + 1
        glob = argv[i] if i < len(argv) else '*'
        pairs = [p for p in pairs if fnmatch.fnmatchcase(p[0]['id'], glob)]
        for (v, _), e in zip(pairs, KV.refcore_runs(pairs)):
            print('=== %s (exit %s)\n%s' % (v['id'], e['rc'], e['out']), end='')
        return 0
    res = KV.refcore_runs(pairs)
    tie(pairs, res)
    if '--times' in argv:
        for (v, _), e in sorted(zip(pairs, res), key=lambda p: -p[1]['seconds']):
            print('%8.2f s  %s' % (e['seconds'], v['id']))
        return 0
    if '--engine' in argv:
        return KV.engine_look(pairs, res)
    data = KV.stored_bytes(pairs, res)
    if '--check' in argv:
        old = open(STORED, 'rb').read() if os.path.exists(STORED) else b''
        if old != data:
            print('m4/vectors.refcore is not what refcore prints for m4/vectors.tsv: run m4/vectors.py')
            return 1
        print('m4/vectors.refcore: %d vectors, refcore\'s bytes' % len(pairs))
        return 0
    with open(STORED, 'wb') as f:
        f.write(data)
    print('m4/vectors.refcore: %d vectors written' % len(pairs))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
