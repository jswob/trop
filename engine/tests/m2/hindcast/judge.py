"""The hindcasts' judge (DONE gate 4; issue #167; hindcast.md §1 "Judge"): holds refcore's stored residual ledgers of
each fixture to the answer key `key.tsv` that gen.py wrote from M*. Standard library only.

A residual ledger's **planted lines** are its detail lines that say something moved: a `matched` line whose amount
difference isn't 0.00 (its `difference`) or whose timing isn't +0 days (its `timing`), every `expected but missing`
line (its `model`), every `unexplained` line (its `observed`); balance lines echo these and aren't judged. For each
fixture and each of its residual asks, the set of planted lines (kind, move, month, field, value) must equal the
key's rows for that fixture and ask, expanded month by month: each planted cause in its own line, and no line of
another kind it would also explain. H0, the base, has none. Masking at K (H8, hindcast.md §4): every ask `as known
K` that isn't a residual ledger (`mask_bal`, `mask_net`) prints the same bytes on H0 and H8, the raise being known
only after K.

It reads m2/vectors.refcore (the vectors `hindcasts/<fixture>/…`), so it judges refcore's bytes, never its own run.

    python3 -I -B engine/tests/m2/hindcast/judge.py           -> one line per fixture and ask; exit 1 on a miss
"""
import csv
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(os.path.dirname(HERE))
STORE = os.path.join(TESTS, 'm2', 'vectors.refcore')
RESIDUAL_ASKS = ('replay', 'back', 'back29')
MASK_ASKS = ('mask_bal', 'mask_net')

MATCHED = re.compile(r'^    matched ([a-z_]+)\[(?:[a-z]+, )?(\d{4}-\d{2})\] on [0-9-]+: model (.+?), observed (.+?), '
                     r'difference (.+?), timing (\S+) days$')
MISSING = re.compile(r'^    expected but missing ([a-z_]+)\[(?:[a-z]+, )?(\d{4}-\d{2})\] on [0-9-]+: model (.+)$')
UNEXPLAINED = re.compile(r'^    unexplained .+ on (\d{4}-\d{2})-\d{2}: observed (.+)$')


def read_stored(data):
    """kernel/vectors.py's store: six netstrings a vector (id, group, words, file table, exit code, output)."""
    fields, i = [], 0
    while i < len(data):
        colon = data.index(b':', i)
        n = int(data[i:colon])
        fields.append(data[colon + 1:colon + 1 + n])
        i = colon + 2 + n
    return [{'id': fields[k].decode('utf-8'), 'out': fields[k + 5].decode('utf-8')} for k in range(0, len(fields), 6)]


def blocks(out):
    """An output's asks, each its lines (a block opens at a line that starts with `<name>: `)."""
    by, cur = {}, None
    for line in out.split('\n'):
        m = re.match(r'^([a-z0-9_]+): ', line)
        if m and not line.startswith(' '):
            cur = m.group(1)
            by[cur] = []
        if cur is not None and line:
            by[cur].append(line)
    return by


def planted(lines):
    found = set()
    for line in lines:
        m = MATCHED.match(line)
        if m:
            move, month, _, _, diff, timing = m.groups()
            if diff != '0.00 PLN':
                found.add(('matched', move, month, 'difference', diff))
            if timing != '+0':
                found.add(('matched', move, month, 'timing', timing + ' days'))
            continue
        m = MISSING.match(line)
        if m:
            found.add(('missing', m.group(1), m.group(2), 'model', m.group(3)))
            continue
        m = UNEXPLAINED.match(line)
        if m:
            found.add(('unexplained', '-', m.group(1), 'observed', m.group(2)))
            continue
        if line.startswith('    ') and not line.startswith('    balance '):
            found.add(('unread', '-', '-', '-', line.strip()))
    return found


def months(a, b):
    y, m = int(a[:4]), int(a[5:])
    out = []
    while '%04d-%02d' % (y, m) <= b:
        out.append('%04d-%02d' % (y, m))
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)
    return out


def key():
    want = {}
    with open(os.path.join(HERE, 'key.tsv'), encoding='utf-8', newline='') as f:
        for r in csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE):
            s = want.setdefault((r['fixture'], r['ask']), set())
            for mo in months(r['from'], r['to']):
                s.add((r['kind'], r['move'], mo, r['field'], r['value']))
    return want


def main():
    stored = {}
    for v in read_stored(open(STORE, "rb").read()):
        if v['id'].startswith('hindcasts/'):
            fixture = v['id'].split('/')[1]
            stored.setdefault(fixture, {}).update(blocks(v['out']))
    if not stored:
        sys.exit('judge.py: no hindcast vector in m2/vectors.refcore')
    want, bad = key(), 0
    for fixture in sorted(stored):
        for ask in RESIDUAL_ASKS:
            if ask not in stored[fixture]:
                continue
            got = planted(stored[fixture][ask])
            exp = want.get((fixture, ask), set())
            ok = got == exp
            bad += not ok
            print('%s %-6s %s: %d planted lines%s' % ('ok  ' if ok else 'MISS', ask, fixture, len(got), '' if ok else
                  '; missing %s; extra %s' % (sorted(exp - got)[:3], sorted(got - exp)[:3])))
    for ask in MASK_ASKS:
        a, b = stored.get('h0', {}).get(ask), stored.get('h8', {}).get(ask)
        ok = a is not None and a == b
        bad += not ok
        print('%s %-8s h0 = h8 (masking at K)' % ('ok  ' if ok else 'MISS', ask))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
