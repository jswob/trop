"""Milestone M2's vectors where refcore crashes or answers otherwise: the spec's answer, written by T, its section named (issue #167;
MILESTONE-M2.md rows 6 and 8). Standard library only.

A vector of m2/vectors.tsv holds refcore's bytes, and refcore can't hold an input it crashes on (`REFCORE-NOT-YET`,
INTERFACE.md §3). Where the spec gives the answer, spec.tsv holds it, one vector per row. It also holds the spec's
answer where refcore answers otherwise and the vector of m2/vectors.tsv of the same id keeps refcore's bytes, held
"for #n" in held.tsv (#169, refcore 2.1.7's misses: `spec/…` beside the held `fixes/…`); `--refcore` then lists
those too:

    id  group  check  cwd  files  asks  options  section  code  out

The first seven columns are vectors.tsv's (no `check`). `section` names the spec's sections (and the pick or issue)
that give the answer; `code` is the exit code and `out` the whole output, `\\n` for a new line and `\\\\` for a
backslash. T writes `out` from the engine's run (`--fill`; `--engine PATH` names a build other than the release one)
only after reading it against `section`; the
answer is the spec's, never refcore's. Written to spec.vectors in vectors.refcore's six netstrings, which
runner/src/m2.rs embeds (milestone/m2_fixes.rs, one test per group).

    python3 -I -B engine/tests/m2/spec.py              -> writes spec.vectors
    python3 -I -B engine/tests/m2/spec.py --check      -> exit 1 if spec.vectors isn't spec.tsv's
    python3 -I -B engine/tests/m2/spec.py --fill       -> the engine's run into each row whose out is empty
    python3 -I -B engine/tests/m2/spec.py --refcore    -> exit 1 if refcore answers a row (it moves to vectors.tsv)
"""
import csv
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

LIST = os.path.join(HERE, 'spec.tsv')
STORED = os.path.join(HERE, 'spec.vectors')
COLUMNS = KV.COLUMNS + ['section', 'code', 'out']


def unescape(s):
    out, i = [], 0
    while i < len(s):
        if s[i] == '\\' and i + 1 < len(s):
            out.append({'n': '\n', '\\': '\\'}.get(s[i + 1], '\\' + s[i + 1]))
            i += 2
        else:
            out.append(s[i])
            i += 1
    return ''.join(out)


def escape(s):
    return s.replace('\\', '\\\\').replace('\n', '\\n')


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLUMNS:
            sys.exit('spec.tsv: the columns are %s, not %s' % (r.fieldnames, COLUMNS))
        out = list(r)
    for v in out:
        if v['check'] or not v['section']:
            sys.exit('spec.tsv: %s names a check or no section' % v['id'])
    return out


def main(argv):
    vs = rows()
    if '--engine' in argv:
        R.ENGINE[0] = os.path.abspath(argv[argv.index('--engine') + 1])
    if '--fill' in argv:
        pairs = KV.checks_of(vs)
        for (v, c) in pairs:
            if not v['out']:
                e = R.execute(c, 'engine', R.BUDGET)
                v['code'], v['out'] = str(e['rc']), escape(e['out'])
        with open(LIST, 'w', encoding='utf-8', newline='') as f:
            f.write('\t'.join(COLUMNS) + '\n')
            for v in vs:
                f.write('\t'.join(v[k] for k in COLUMNS) + '\n')
        print('spec.tsv: %d rows' % len(vs))
        return 0
    for v in vs:
        if not v['out'] or not v['code'].isdigit():
            sys.exit('spec.tsv: %s has no answer yet (--fill, then read it against its section)' % v['id'])
    pairs = KV.checks_of(vs)
    if '--refcore' in argv:
        R.need_watchdog()
        rc_hash = R.refcore_hash()
        bad = 0
        for (v, c) in pairs:
            e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, R.T + 'cache')
            if 'REFCORE-NOT-YET' not in e['out']:
                bad += 1
                print('%s: refcore answers; move it to vectors.tsv' % v['id'])
        print('%d of %d rows: refcore still crashes' % (len(pairs) - bad, len(pairs)))
        return 0 if bad == 0 else 1
    data = KV.stored_bytes(pairs, [{'rc': int(v['code']), 'out': unescape(v['out'])} for v in vs])
    if '--check' in argv:
        old = open(STORED, 'rb').read() if os.path.exists(STORED) else b''
        if old != data:
            print('m2/spec.vectors is not spec.tsv\'s: run m2/spec.py')
            return 1
        print('m2/spec.vectors: %d vectors, the spec\'s answers' % len(pairs))
        return 0
    with open(STORED, 'wb') as f:
        f.write(data)
    print('m2/spec.vectors: %d vectors written' % len(pairs))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
