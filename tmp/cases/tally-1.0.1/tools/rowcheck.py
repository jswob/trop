"""Check a rows file mechanically (brief §6.3).

Usage: python3 -I -B rowcheck.py <rows.md> --expect ID ... [--expect-file F]
Reads every Markdown table row whose first cell is a row id in backticks or bare
(P:, Q:, O:, S:). Checks: ids valid and unique; each expected id present; status
one of the scale; a reason on every row that isn't "expressed"; every `L §x.y`
names a section of LANGUAGE.md and every `L:n` a line of it; every Text path
exists under tally-1.0.1/. Prints counts by status and each problem.
"""
import os
import re
import sys
from collections import Counter

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.normpath(os.path.join(HERE, '..', '..', '..'))
LANG = os.path.join(ROOT, 'engine', 'spec', 'LANGUAGE.md')
STATUSES = ['breaks', "can't express", 'awkward', 'expressed', 'case suspect']
ID = re.compile(r'^(P:[A-Z]+-\d\d|Q:(A|B|D|E1)-\d\d|O:[A-Za-z0-9]+(-[A-Za-z0-9]+)*-\d+|S:\d+)$')


def sections():
    secs, n = set(), 0
    with open(LANG, encoding='utf-8') as f:
        for n, line in enumerate(f, 1):
            m = re.match(r'^#{2,4} (\d+(\.\d+)*)[. ]', line)
            if m:
                secs.add(m.group(1))
    return secs, n


def cells(line):
    parts = [c.strip() for c in line.strip().strip('|').split('|')]
    return parts


def main():
    args = sys.argv[1:]
    path = args.pop(0)
    expect = []
    while args:
        a = args.pop(0)
        if a == '--expect':
            while args and not args[0].startswith('--'):
                expect.append(args.pop(0))
        elif a == '--expect-file':
            expect += open(args.pop(0)).read().split()
    secs, nlines = sections()
    rows, problems = {}, []
    with open(path, encoding='utf-8') as f:
        text = f.read()
    for line in text.splitlines():
        if not line.lstrip().startswith('|'):
            continue
        c = cells(line)
        rid = c[0].strip('`* ')
        if not re.match(r'^[PQOS]:', rid):
            continue
        if len(c) < 6:
            problems.append('%s: %d cells, want 6' % (rid, len(c)))
            continue
        status = c[1].strip('*` ').lower()
        reason, spec, gap, textp = c[2], c[3], c[4], c[5].strip('` ')
        if not ID.match(rid):
            problems.append('%s: bad id' % rid)
        if rid in rows:
            problems.append('%s: duplicate' % rid)
        rows[rid] = status
        if status not in STATUSES:
            problems.append('%s: bad status %r' % (rid, c[1]))
        if status != 'expressed' and not reason.strip('—- '):
            problems.append('%s: no reason' % rid)
        for m in re.finditer(r'L ?§ ?(\d+(?:\.\d+)*)', spec + ' ' + reason):
            if m.group(1) not in secs:
                problems.append('%s: no section L §%s' % (rid, m.group(1)))
        for m in re.finditer(r'L:(\d+)', spec + ' ' + reason):
            if int(m.group(1)) > nlines:
                problems.append('%s: no line L:%s' % (rid, m.group(1)))
        for p in re.split(r'[,;]\s*|\s+', textp):
            p = p.strip('` ')
            if not p or p in ('—', '-') or rid.startswith('S:'):
                continue
            full = p if p.startswith('engine/') else os.path.join(HERE, p)
            if not os.path.exists(os.path.join(ROOT, full) if p.startswith('engine/') else full):
                problems.append('%s: no text %s' % (rid, p))
    for e in expect:
        if e not in rows:
            problems.append('%s: missing row' % e)
    count = Counter(rows.values())
    print('%d rows: %s' % (len(rows), ', '.join('%s %d' % (s, count[s]) for s in STATUSES if count[s])))
    for p in problems:
        print('problem:', p)
    sys.exit(1 if problems else 0)


if __name__ == '__main__':
    main()
