"""Re-record a household check on refcore: in its run's shared baseline file, the lines of the blocks its ask owns are
replaced by the same blocks of a refcore output (run.py's block form decides what a block is); every other line keeps
its bytes. python3 -I -B splice.py ID=OUTPUT ... [--write]; checks afterwards that the check compares `same`."""
import importlib.util, sys
R = '/Users/jswob/projects/trop/'
s = importlib.util.spec_from_file_location('run', R + 'engine/tests/run.py'); r = importlib.util.module_from_spec(s)
s.loader.exec_module(r)
checks = r.load_checks(); by = {c['id']: c for c in checks}


def header(l):
    return l.strip() and not l.startswith((' ', '#', 'warning', 'rc=', 'time ', 'exit', 'real', 'user', 'sys')) \
        and 'WARN-UNREAD' not in l


def span(lines, ask):
    """[first, last) of the lines of the blocks ask owns: one contiguous run, or an error."""
    idx = [i for i, l in enumerate(lines) if header(l)]
    own = [i for i in idx if r.owner(l := lines[i]) == ask]
    if not own:
        raise SystemExit('no block of %s' % ask)
    end = own[-1] + 1  # a block is its header and the indented lines under it
    while end < len(lines) and lines[end].startswith(' '):
        end += 1
    if any(r.owner(lines[i]) != ask for i in idx if own[0] <= i < end):
        raise SystemExit('the blocks of %s are not contiguous' % ask)
    return own[0], end


ok = True
for arg in [a for a in sys.argv[1:] if '=' in a]:
    cid, out = arg.split('=', 1)
    c = by[cid]
    base = open(R + c['baseline'], encoding='utf-8').read().splitlines(True)
    new = open(out, encoding='utf-8').read().splitlines(True)
    a, b = span(base, c['block'])
    x, y = span(new, c['block'])
    result = ''.join(base[:a] + new[x:y] + base[b:])
    v, d = r.compare(c, open(out, encoding='utf-8').read(), checks) if False else (None, None)
    if '--write' in sys.argv:
        open(R + c['baseline'], 'w', encoding='utf-8').write(result)
        v, d = r.compare(c, open(out, encoding='utf-8').read(), checks)
        ok &= v == 'same'
        print('%s %s: lines %d-%d of %s replaced by %d lines; now %s' % ('ok  ' if v == 'same' else 'FAIL', cid, a + 1, b,
              c['baseline'].split('/')[-1], y - x, v))
    else:
        print('%s: would replace lines %d-%d (%d) by %d lines' % (cid, a + 1, b, b - a, y - x))
sys.exit(0 if ok else 1)
