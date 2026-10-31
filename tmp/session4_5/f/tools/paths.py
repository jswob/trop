"""The mechanical path rewrite of session 4.5f: every absolute path under the repository becomes repository-relative,
and every path into a moved file or directory gets its new place (layout.py's MOVES). Standard library only.

    python3 -I -B tmp/session4_5/f/tools/paths.py FILE...        rewrite in place, print the count per file
    python3 -I -B tmp/session4_5/f/tools/paths.py --to OUT FILE  write the rewrite of FILE to OUT
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import layout  # noqa: E402

ABS = '/Users/jswob/projects/trop'
DIRS = {'tmp/design2': 'engine/tests', 'tmp/tests': 'engine/tests', 'tmp/tests/texts': 'engine/tests/integration',
        'tmp/design2/scratch/lead/check': 'engine/tests/defs', 'tmp/cases': 'engine/tests/cases'}
TOK = re.compile(r'%s(?:/[^\s\'"`|,;)\]]*)?|(?<![\w/.-])tmp/[A-Za-z0-9_./-]+' % re.escape(ABS))
unmapped = set()


def move(p):
    """A repository-relative path (file or directory, maybe with a trailing '/') -> its new path, or None."""
    slash = p.endswith('/') and len(p) > 1
    q = p.rstrip('/')
    if q.startswith('tmp/cases/tally-1.0.1'):
        return None
    n = layout.new(q) or (layout.new(q + '/') or '').rstrip('/') or DIRS.get(q)
    if n is None:
        for d in sorted(DIRS, key=len, reverse=True):  # a path inside a directory none of whose files moved
            if q.startswith(d + '/') and not any(q.startswith(o.rstrip('/-')) for o, _ in layout.MOVES):
                break
        return None
    return n + ('/' if slash else '')


def one(m):
    t = m.group(0)
    tail = ''
    while t and t[-1] in '.:' and not t.endswith('..'):
        tail, t = t[-1] + tail, t[:-1]
    if t.startswith(ABS):
        r = t[len(ABS) + 1:]
        if r == '':
            return '.' + tail
        n = move(r)
        if n is None:
            unmapped.add(r)
        return (n or r) + tail
    n = move(t)
    if n is None:
        unmapped.add(t)
    return (n or t) + tail


def rw(s):
    return TOK.sub(one, s)


if __name__ == '__main__':
    a = sys.argv[1:]
    if a[:1] == ['--to']:
        open(a[1], 'w', encoding='utf-8', newline='').write(rw(open(a[2], encoding='utf-8', newline='').read()))
    else:
        for f in a:
            s = open(f, encoding='utf-8', errors='surrogateescape', newline='').read()
            r = rw(s)
            if r != s:
                open(f, 'w', encoding='utf-8', errors='surrogateescape', newline='').write(r)
            print('%4d %s' % (len(TOK.findall(s)), f))
    for u in sorted(unmapped):
        print('kept (not moved):', u, file=sys.stderr)
