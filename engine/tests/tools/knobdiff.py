#!/usr/bin/env python3
"""Confirm a capability copy differs from its original only in the knob lines knobs.md names (session 4.5c).

    python3 -I -B tmp/tests/knobdiff.py <original dir> <copy dir> <KNOBS file>
    python3 -I -B tmp/tests/knobdiff.py --self-test

The KNOBS file (tab-separated, '#' lines are comments) lists the original lines the copy may change or drop:

    <path relative to the dirs>	<line number in the original>	<the knob it is, from knobs.md>

Every file under the two dirs is compared (any extension). A file only in one dir fails, unless the KNOBS file
lists it with line number 'new' (only in the copy) or 'gone' (only in the original). In a listed file, every
replaced or deleted original line must be listed; inserted lines fail unless their path is listed with
line 'insert'. Prints each knob change as `path:line  - old` / `+ new` and exits 0 when the copy passes.
Standard library only.
"""
import difflib
import os
import sys
import tempfile


def files(d):
    out = set()
    for root, _, fs in os.walk(d):
        for f in fs:
            if f == '.DS_Store' or '__pycache__' in root:
                continue
            out.add(os.path.relpath(os.path.join(root, f), d))
    return out


def read_knobs(p):
    allow = {}
    for l in open(p, encoding='utf-8'):
        if not l.strip() or l.startswith('#'):
            continue
        parts = l.rstrip('\n').split('\t')
        path, line = parts[0], parts[1]
        allow.setdefault(path, set()).add(line if line in ('new', 'gone', 'insert') else int(line))
    return allow


def check(orig, copy, allow):
    errors, shown = [], []
    a, b = files(orig), files(copy)
    for f in sorted(a | b):
        ok = allow.get(f, set())
        if f not in b:
            (shown if 'gone' in ok else errors).append('%s: only in the original' % f)
            continue
        if f not in a:
            (shown if 'new' in ok else errors).append('%s: only in the copy' % f)
            continue
        x = open(os.path.join(orig, f), 'rb').read().decode('utf-8', 'surrogateescape').splitlines(True)
        y = open(os.path.join(copy, f), 'rb').read().decode('utf-8', 'surrogateescape').splitlines(True)
        if x == y:
            continue
        for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, x, y, autojunk=False).get_opcodes():
            if tag == 'equal':
                continue
            bad = [n + 1 for n in range(i1, i2) if n + 1 not in ok]
            if tag == 'insert' and 'insert' not in ok:
                bad = ['insert after %d' % i1]
            if tag == 'replace' and j2 - j1 > i2 - i1 and 'insert' not in ok:
                bad.append('%d more lines than it replaces' % (j2 - j1 - (i2 - i1)))
            for n in range(i1, i2):
                shown.append('%s:%d  - %s' % (f, n + 1, x[n].rstrip('\n')[:160]))
            for n in range(j1, j2):
                shown.append('%s:%s  + %s' % (f, '', y[n].rstrip('\n')[:160]))
            if bad:
                errors.append('%s: lines not named as knobs changed: %s' % (f, bad))
    return errors, shown


def self_test():
    with tempfile.TemporaryDirectory() as t:
        o, c = os.path.join(t, 'o'), os.path.join(t, 'c')
        os.makedirs(o), os.makedirs(c)
        open(os.path.join(o, 'r.tally'), 'w').write('model m\n  end 2028-12-31\nrule x = 1\n')
        open(os.path.join(c, 'r.tally'), 'w').write('model m\n  end 2026-01-31\nrule x = 1\n')
        e, _ = check(o, c, {'r.tally': {2}})
        assert not e, e
        e, _ = check(o, c, {'r.tally': {3}})
        assert e, 'an unnamed line change must fail'
        open(os.path.join(c, 'r.tally'), 'w').write('model m\n  end 2026-01-31\nrule x = 2\n')
        e, _ = check(o, c, {'r.tally': {2}})
        assert e, 'a second changed line must fail'
        open(os.path.join(c, 'r.tally'), 'w').write('model m\nrule x = 1\n')
        e, _ = check(o, c, {'r.tally': {2}})
        assert not e, 'a dropped knob line passes'
        open(os.path.join(c, 'extra.tally'), 'w').write('x\n')
        e, _ = check(o, c, {'r.tally': {2}})
        assert e, 'an extra file must fail'
    print('knobdiff self-test PASSED')


def main(argv):
    if argv == ['--self-test']:
        return self_test()
    orig, copy, knobs = argv
    errors, shown = check(orig, copy, read_knobs(knobs))
    for s in shown:
        print(s)
    for e in errors:
        print('FAIL ' + e)
    print('knobdiff %s: %s vs %s (%d knob lines shown)' % ('FAIL' if errors else 'ok', copy, orig, len(shown)))
    return 1 if errors else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
