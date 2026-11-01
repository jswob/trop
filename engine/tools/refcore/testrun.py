"""The test runner: every tests/*.test file is a reality and its expected output.

A test file:

    # comments (the spec section the test comes from, how the expected
    # value follows from the spec's text)
    === file_a.tally
    <tally text>
    === file_b.tally
    <tally text>
    === expect
    <expected output lines>

The expected lines must equal the printed lines.  An expected line ending in
` ...` matches any printed line it begins (used for refusals, whose messages
are this interpreter's own).  An expected line that is `...` alone, unindented,
matches any number of printed lines (a long register whose other lines the test
doesn't assert).  Warning lines are compared only when the expect block holds
one.  `=== expect ask1 ask2` answers only those asks.  `=== options pin=bytes` (before the files) runs the reality
with those options (cli.run_texts): a joint model's pins taken over the member files' bytes.  `=== options reprint`
prints each file's canonical text (L §2.4) and the files' sha256 (E §3.1 step 1) instead of running them.
`=== options disk` writes the files into a fresh folder and runs them as the command line does (cli.command_lines,
`run`, or `reprint` with `reprint`), given by their names in the test's order and then the names in `missing=a,b`,
which are never written; a file named in `latin1=a,b` is written in Latin-1, so a letter past ASCII isn't UTF-8.
"""
import os
import tempfile
os.environ.setdefault("REFCORE_DEBUG", "1")

from .cli import run_texts, command_lines

HERE = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'tests')


def parse_test(path, with_options=False):
    files = []
    expect = None
    ids = None
    cur = None
    options = {}
    with open(path, encoding='utf-8') as fh:
        text = fh.read()
    for line in text.split('\n'):
        if line.startswith('=== '):
            name = line[4:].strip()
            if name.startswith('options ') and cur is None:
                for kv in name[len('options '):].split():
                    k, _, v = kv.partition('=')
                    options[k] = v
                continue
            if name == 'expect' or name.startswith('expect '):
                expect = []
                cur = expect
                rest = name[6:].split()
                ids = rest or None
            else:
                buf = []
                files.append((name, buf))
                cur = buf
            continue
        if cur is None:
            continue
        cur.append(line)
    texts = [(n, '\n'.join(b).rstrip('\n') + '\n') for n, b in files]
    while expect and expect[-1] == '':
        expect.pop()
    if with_options:
        return texts, expect or [], ids, options
    return texts, expect or [], ids


def match(exp, got):
    if exp.endswith(' ...'):
        return got.startswith(exp[:-4])
    return exp == got


def compare(expect, got):
    if not any(l.startswith('warning: ') for l in expect):
        got = [l for l in got if not l.startswith('warning: ')]
    if '...' not in expect:
        if len(expect) != len(got):
            return False
        return all(match(e, g) for e, g in zip(expect, got))
    return _cmp(expect, 0, got, 0)


def _cmp(ex, i, got, j):
    while i < len(ex):
        if ex[i] == '...':
            return any(_cmp(ex, i + 1, got, k) for k in range(j, len(got) + 1))
        if j >= len(got) or not match(ex[i], got[j]):
            return False
        i += 1
        j += 1
    return j == len(got)


def run_on_disk(texts, ids, options):
    """`=== options disk`: the files in a fresh folder, run from it as the command line runs paths."""
    latin = set(n for n in options.get('latin1', '').split(',') if n)
    missing = [n for n in options.get('missing', '').split(',') if n]
    here = os.getcwd()
    with tempfile.TemporaryDirectory() as d:
        for n, t in texts:
            p = os.path.join(d, n)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, 'wb') as fh:
                fh.write(t.encode('latin-1' if n in latin else 'utf-8'))
        argv = ['reprint' if 'reprint' in options else 'run'] + [n for n, t in texts] + missing
        if ids:
            argv += ['--ask'] + ids
        os.chdir(d)
        try:
            return command_lines(argv)
        finally:
            os.chdir(here)


def run_tests(names):
    files = sorted(f for f in os.listdir(HERE) if f.endswith('.test'))
    if names:
        files = [f for f in files if any(n in f for n in names)]
    fails = 0
    for f in files:
        texts, expect, ids, options = parse_test(os.path.join(HERE, f), with_options=True)
        try:
            got = run_on_disk(texts, ids, options) if 'disk' in options else run_texts(texts, ids, options)
        except Exception as e:  # an interpreter bug
            import traceback
            got = ['CRASH %s: %s' % (type(e).__name__, e)] + traceback.format_exc().split('\n')
        if compare(expect, got):
            continue
        fails += 1
        print('FAIL %s' % f)
        print('  expected:')
        for l in expect:
            print('    ' + l)
        print('  got:')
        for l in got:
            print('    ' + l)
    print('%d tests, %d passed, %d failed' % (len(files), len(files) - fails, fails))
    return 1 if fails else 0
