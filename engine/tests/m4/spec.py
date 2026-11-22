"""Milestone M4's vectors where refcore crashes or answers otherwise: the spec's answer, its section named (issue
#252; MILESTONE-M4.md row 5). Standard library only.

A vector of m4/vectors.tsv holds refcore's bytes. Where the spec gives another answer, that vector stays, held "for
#n" in held.tsv, and spec.tsv holds the spec's answer, one vector per row (M2's `spec/…` form, m2/spec.py); where
refcore crashes (`REFCORE-NOT-YET`), only the spec's answer is stored:

    id  group  check  cwd  files  asks  options  section  twin  code  out

The first seven columns are vectors.tsv's (no `check`). `section` names the spec's sections (and the issue) that give
the answer. The answer comes one of two ways:

- **A twin** (M3's form, m3/spec.tsv): `twin` names a file beside the row's last file, the same text with a hole
  replaced by a known value, where the spec's answer can't depend on that value (a refusal that outranks the hole,
  L §8). The answer is refcore's run of the twin in the file's place, the twin's name read as the file's; `code` and
  `out` are `-`.
- **Written:** `twin` is `-`, `code` the exit code and `out` the whole output, `\\n` a new line and `\\\\` a
  backslash. T writes `out` from an engine's run (`--fill`; `--engine PATH` names the build) only after checking
  it by hand against `section`; the answer is the spec's, never refcore's.

Stored in spec.vectors in vectors.refcore's six netstrings, the row's own command line and file table, which
runner/src/m4.rs embeds (milestone/m4_fixes.rs, one test per group).

    python3 -I -B engine/tests/m4/spec.py                    -> writes spec.vectors (runs refcore on the twins)
    python3 -I -B engine/tests/m4/spec.py --check            -> exit 1 if spec.vectors isn't spec.tsv's
    python3 -I -B engine/tests/m4/spec.py --fill --engine P  -> P's run into each written row whose out is empty
"""
import csv
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(HERE)
sys.path.insert(0, TESTS)
import run as R  # noqa: E402

_spec = importlib.util.spec_from_file_location('m4_vectors', os.path.join(HERE, 'vectors.py'))
M4 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(M4)   # kernel/vectors.py's functions, with M4's file table (a joint model's members)
KV = M4.KV

LIST = os.path.join(HERE, 'spec.tsv')
STORED = os.path.join(HERE, 'spec.vectors')
COLUMNS = KV.COLUMNS + ['section', 'twin', 'code', 'out']


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
        if v['check'] or not v['section'] or not v['id'].startswith('spec/') or not v['group'].startswith('spec-'):
            sys.exit('spec.tsv: %s names a check, no section, or no spec/ id and spec- group' % v['id'])
        if (v['twin'] == '-') == (v['out'] == '-'):
            sys.exit('spec.tsv: %s names a twin or writes its out, not both' % v['id'])
    return out


def twin_runs(vs):
    """refcore's run of each twin row with the twin in its file's place, the twin's name read as the file's."""
    twins = [v for v in vs if v['twin'] != '-']
    swapped = [dict(v, files=' '.join(v['files'].split()[:-1] + [v['twin']])) for v in twins]
    res = KV.refcore_runs(KV.checks_of(swapped))
    out = {}
    for v, e in zip(twins, res):
        name, twin = os.path.basename(v['files'].split()[-1]), os.path.basename(v['twin'])
        out[v['id']] = {'rc': e['rc'], 'out': e['out'].replace(twin, name)}
    return out


def main(argv):
    vs = rows()
    if '--fill' in argv:
        if '--engine' not in argv:
            sys.exit('--fill needs --engine PATH, the build whose answers T checks by hand')
        R.ENGINE[:] = [os.path.abspath(argv[argv.index('--engine') + 1])]
        for (v, c) in KV.checks_of([v for v in vs if v['twin'] == '-' and not v['out']]):
            e = R.execute(c, 'engine', R.BUDGET)
            v['code'], v['out'] = str(e['rc']), escape(e['out'])
        with open(LIST, 'w', encoding='utf-8', newline='') as f:
            f.write('\t'.join(COLUMNS) + '\n')
            for v in vs:
                f.write('\t'.join(v[k] for k in COLUMNS) + '\n')
        print('spec.tsv: %d rows' % len(vs))
        return 0
    for v in vs:
        if v['twin'] == '-' and (not v['out'] or not v['code'].isdigit()):
            sys.exit('spec.tsv: %s has no answer yet (--fill, then check it against its section)' % v['id'])
    R.need_watchdog()
    by_twin = twin_runs(vs)
    res = [by_twin[v['id']] if v['twin'] != '-' else {'rc': int(v['code']), 'out': unescape(v['out'])} for v in vs]
    data = KV.stored_bytes(KV.checks_of(vs), res)
    if '--check' in argv:
        old = open(STORED, 'rb').read() if os.path.exists(STORED) else b''
        if old != data:
            print('m4/spec.vectors is not spec.tsv\'s: run m4/spec.py')
            return 1
        print('m4/spec.vectors: %d vectors, the spec\'s answers' % len(vs))
        return 0
    with open(STORED, 'wb') as f:
        f.write(data)
    print('m4/spec.vectors: %d vectors written' % len(vs))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
