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
`=== options disk` writes the files into a fresh folder and runs them as the command line does (cli.command, `run`,
or `reprint` with `reprint`, or `check` with `check`), given by their names in the test's order and then the names in
`missing=a,b`, which are never written, or by the words of `paths=a,b` in their place (a directory); the words of
`args=a,b` come last; a file named in `latin1=a,b` is written in Latin-1, so a letter past ASCII isn't UTF-8.
`=== options check` prints `check`'s lines (OPERATIONS §2.3) instead of running the asks, its warning lines compared
always.  `exit=N` holds the exit code too, as cli.exit_code (or, on disk, cli.command) gives it.
"""
import os
import tempfile
os.environ.setdefault("REFCORE_DEBUG", "1")

from .cli import run_texts, command, exit_code

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


def compare(expect, got, warnings=False):
    """warnings: compare the warning lines whatever the expect block holds (`check`'s, OPERATIONS §2.3)."""
    if not warnings and not any(l.startswith('warning: ') for l in expect):
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


def words(options, key):
    return [n for n in options.get(key, '').split(',') if n]


def run_on_disk(texts, ids, options):
    """`=== options disk`: the files in a fresh folder, run from it as the command line runs paths; (lines, exit)."""
    latin = set(words(options, 'latin1'))
    missing = words(options, 'missing')
    here = os.getcwd()
    with tempfile.TemporaryDirectory() as d:
        for n, t in texts:
            p = os.path.join(d, n)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, 'wb') as fh:
                fh.write(t.encode('latin-1' if n in latin else 'utf-8'))
        verb = 'reprint' if 'reprint' in options else 'check' if 'check' in options else 'run'
        argv = [verb] + (words(options, 'paths') or [n for n, t in texts] + missing)
        if ids:
            argv += ['--ask'] + ids
        argv += words(options, 'args')
        os.chdir(d)
        try:
            return command(argv)
        finally:
            os.chdir(here)


def run_one_test(texts, ids, options):
    """(the printed lines, the exit code) of one test."""
    if 'disk' in options:
        return run_on_disk(texts, ids, options)
    got = run_texts(texts, ids, options)
    return got, exit_code('check' if 'check' in options else 'run', got)


def run_tests(names):
    files = sorted(f for f in os.listdir(HERE) if f.endswith('.test'))
    if names:
        files = [f for f in files if any(n in f for n in names)]
    fails = 0
    for f in files:
        texts, expect, ids, options = parse_test(os.path.join(HERE, f), with_options=True)
        code = None
        try:
            got, code = run_one_test(texts, ids, options)
        except Exception as e:  # an interpreter bug
            import traceback
            got = ['CRASH %s: %s' % (type(e).__name__, e)] + traceback.format_exc().split('\n')
        if compare(expect, got, 'check' in options) and ('exit' not in options or str(code) == options['exit']):
            continue
        fails += 1
        print('FAIL %s' % f)
        if 'exit' in options:
            print('  exit: expected %s, got %s' % (options['exit'], code))
        print('  expected:')
        for l in expect:
            print('    ' + l)
        print('  got:')
        for l in got:
            print('    ' + l)
    print('%d tests, %d passed, %d failed' % (len(files), len(files) - fails, fails))
    return 1 if fails else 0
