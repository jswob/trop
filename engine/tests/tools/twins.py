"""The `windows each` twin check (tally 2.1.8, plan §2 "Where the twin check lives"; picks P34, P36: `windows each` is
cut by the answer, a block's register is the read set of its days, so only answer lines are compared). Standard
library only.

For every stored vector (kernel, m1 and m2 `vectors.refcore`) whose ask states `windows each`, its stored output's
head lines for that ask are read: `<ask> [(tuple)] [<sub-window>] [<axis>]: <answer>`. Two twins are asked of refcore
on a copy of the vector's text (under out/twins/, through run.py's run_one and its cache, so a twin re-runs only when
refcore or the text changes), each the same line without the clause:

  - the plain twin, per tuple and axis combination: the hole (`?"…"`) or a window value where two of its
    sub-windows' answers differ, else their one answer (L §4.4: a read is a hole exactly where its answer depends on
    the day);
  - the pinned twins: for each sub-window whose every named window is an input a `with` pin can write (`wa`,
    `dk[a]`; a calendar's `added` entry can't be pinned), the line pinned to each of its days (up to the first, the
    middle and the last of a longer run) answers what the sub-window's line answers.

map/twins.tsv lists each (vector, ask) with its sub-window lines, its plain twin's kind (`hole`, `window`, `one`) and
its pinned days; --check fails where a twin fails or the table differs from what refcore gives.

    python3 -I -B engine/tests/tools/twins.py              -> writes map/twins.tsv; exit 1 if a twin fails
    python3 -I -B engine/tests/tools/twins.py --check      -> exit 1 if a twin fails or map/twins.tsv differs
    python3 -I -B engine/tests/tools/twins.py --self-test  -> the parsers and the rule on planted lines
"""
import csv
import datetime
import importlib.util
import os
import re
import shutil
import sys

TESTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + '/'
ROOT = os.path.dirname(os.path.dirname(TESTS.rstrip('/'))) + '/'
TABLE = TESTS + 'map/twins.tsv'
WORK = TESTS + 'out/twins/'
STORES = ['kernel/vectors', 'm1/vectors', 'm2/vectors']
DATE = re.compile(r'\d{4}-\d{2}-\d{2}')
HEAD = re.compile(r'^(?P<ask>[A-Za-z_][A-Za-z0-9_]*)(?P<tuple> \([^()]*(?:\([^()]*\)[^()]*)*\))?(?P<br>(?: \[[^\]]*\])*): (?P<ans>.*)$')
STOP = (' law ', ' as known ', ' windows ', ' with uncertain', ' partial')
csv.field_size_limit(1 << 30)


def load_run():
    spec = importlib.util.spec_from_file_location('runpy_twins', TESTS + 'run.py')
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def netstrings(b):
    out, i = [], 0
    while i < len(b):
        j = b.index(b':', i)
        n = int(b[i:j])
        out.append(b[j + 1:j + 1 + n].decode('utf-8', 'surrogateescape'))
        i = j + 2 + n
    return out


def stored(store):
    """id -> its stored output (field 6 of the six-netstring records)."""
    f = netstrings(open(TESTS + store + '.refcore', 'rb').read())
    return {f[k]: f[k + 5] for k in range(0, len(f), 6)}


def rows(store):
    with open(TESTS + store + '.tsv', encoding='utf-8', newline='') as fh:
        return [r for r in csv.DictReader(fh, delimiter='\t', quoting=csv.QUOTE_NONE) if r.get('files')]


def ask_lines(paths):
    """ask name -> (path, line) of the asks the files state."""
    out = {}
    for p in paths:
        for line in open(ROOT + p, encoding='utf-8').read().split('\n'):
            m = re.match(r'^ask (\w+)\s*:', line)
            if m:
                out[m.group(1)] = (p, line)
    return out


def brackets(s):
    return re.findall(r'\[([^\]]*)\]', s)


def is_window(part):
    """A bracket of a head is a sub-window when a value names a day or a window."""
    return any(DATE.search(x.split(' = ', 1)[1]) or 'window(' in x for x in split_parts(part) if ' = ' in x)


def split_parts(b):
    out, depth, cur = [], 0, ''
    for ch in b:
        depth += ch in '(['
        depth -= ch in ')]'
        if ch == ',' and depth == 0:
            out.append(cur.strip())
            cur = ''
        else:
            cur += ch
    return out + ([cur.strip()] if cur.strip() else [])


def heads(out, ask):
    """[(tuple, sub-window or '', axis, answer)] of `ask`'s head lines."""
    res = []
    for line in out.split('\n'):
        m = HEAD.match(line)
        if not m or m.group('ask') != ask:
            continue
        bs = brackets(m.group('br') or '')
        sub = next((b for b in bs if is_window(b)), '')
        axis = ' '.join(b for b in bs if b != sub)
        res.append(((m.group('tuple') or '').strip(), sub, axis, m.group('ans')))
    return res


def days(v):
    """The days of a sub-window part's value (`d`, `d1 .. d2`): up to the first, middle and last."""
    ds = DATE.findall(v)
    if len(ds) == 1 and '..' not in v:
        return ds
    if len(ds) == 2 and ' .. ' in v:
        a, b = (datetime.date.fromisoformat(x) for x in ds)
        n = (b - a).days
        picks = sorted({0, n // 2, n})
        return [(a + datetime.timedelta(days=k)).isoformat() for k in picks]
    return []


def pinnable(name, inputs):
    return re.sub(r'\[.*$', '', name) in inputs


def strip_each(line):
    return re.sub(r' windows each\b', '', line)


def with_pin(line, pins):
    """The ask line with `with` pins added in the grammar's place (after `for`/`in`, before `law` and the rest)."""
    head, body = line.split(':', 1)
    cut = min([body.find(s) for s in STOP if body.find(s) >= 0] or [len(body)])
    pre, post = body[:cut], body[cut:]
    m = re.search(r' with (?!uncertain)', pre)
    if m:  # a pin the ask writes of the same name takes the day: two pins of one name refuse the text
        mine = dict(pins)
        have = [p.split(' = ', 1) for p in split_parts(pre[m.end():])]
        kept = ['%s = %s' % (n, mine.pop(n) if n in mine else v) for n, v in have]
        pre = pre[:m.end()] + ', '.join(kept + ['%s = %s' % (n, d) for n, d in pins if n in mine])
        return head + ':' + pre + post
    return head + ':' + pre + ' with ' + ', '.join('%s = %s' % p for p in pins) + post


def element_each(line):
    """`windows each` stated on one element of a tuple (R13), not on the ask: a `,` at the top level after it."""
    rest = line.split(' windows each', 1)[1]
    return len(split_parts(rest)) > 1


def components(ans):
    """A tuple answer's components, or None."""
    if not (ans.startswith('(') and ans.endswith(')')):
        return None
    parts = split_parts(ans[1:-1])
    return parts if len(parts) > 1 else None


def varies(answers, p):
    """The plain line's kind where the sub-windows' answers differ: hole, window, or None (a wrong plain line);
    a tuple is judged component by component."""
    def kind(x):
        return 'hole' if x.startswith('?"') else 'window' if x.startswith(('window(', 'one_of(')) else None
    ps, cs = components(p), [components(a) for a in answers]
    if ps is None or any(c is None or len(c) != len(ps) for c in cs):
        return kind(p)
    for k, pk in enumerate(ps):
        if len({c[k] for c in cs}) > 1:
            if kind(pk) is None:
                return None
        elif pk != cs[0][k]:
            return None
    return 'hole'


def plan(store, row, out):
    """The vector's `windows each` asks and their twins: [(ask, line, heads, [(variant name, line, pins)])], and
    the asks left out: [(ask, why)]."""
    files = row['files'].split()
    asks = ask_lines(files)
    want = row['asks'].split() or list(asks)
    inputs, replaced = set(), set()
    for p in files:
        text = open(ROOT + p, encoding='utf-8').read()
        inputs |= set(re.findall(r'^input (\w+)', text, re.M))
        replaced |= set(re.findall(r'^\s+replace (\w+)', text, re.M))
    res, out_ = [], []
    for a in want:
        if a not in asks or ' windows each' not in asks[a][1]:
            continue
        hs = heads(out, a)
        if len([h for h in hs if h[1]]) < 2:
            continue
        line = asks[a][1]
        if element_each(line):
            out_.append((a, 'element'))  # its plain and pinned twins aren't the ask's line less one clause
            continue
        stacked = re.search(r' in \w+', line.split(':', 1)[1]) is not None
        plain = strip_each(line)
        variants = [('zz_plain_' + a, plain.replace('ask %s' % a, 'ask zz_plain_%s' % a, 1), None)]
        for i, (tup, sub, axis, ans) in enumerate(hs):
            parts = [p.split(' = ', 1) for p in split_parts(sub) if ' = ' in p]
            if not parts or not all(pinnable(n, inputs) and days(v) for n, v in parts):
                continue
            if stacked and any(re.sub(r'\[.*$', '', n) in replaced for n, _ in parts):
                continue  # an asked stack states the input itself: a pin would overrule it, the sub-window doesn't
            combos = [[]]
            for n, v in parts:
                combos = [c + [(n, d)] for c in combos for d in days(v)]
            for j, pins in enumerate(combos[:9]):
                name = 'zz_pin_%s_%d_%d' % (a, i, j)
                variants.append((name, with_pin(plain.replace('ask %s' % a, 'ask %s' % name, 1), pins), (i, pins)))
        res.append((a, asks[a][0], hs, variants))
    return res, out_


def judge(hs, got):
    """Problems of one ask: its heads, and the variants' outputs {variant: [(tuple, axis, answer)]}. A `diff`'s head
    counts its rows, not a value (`diff in …: n changed, …`): its plain line isn't judged, its pinned lines are."""
    probs, kinds = [], set()
    groups = {}
    for tup, sub, axis, ans in hs:
        groups.setdefault((tup, axis), []).append(ans)
    plain = {(t, ax): an for t, ax, an in got.get('plain', [])}
    for (tup, axis), answers in groups.items():
        if all(a.startswith('diff ') for a in answers):
            kinds.add('diff')
            continue
        p = plain.get((tup, axis))
        if p is None:
            p = next((an for (t, ax), an in plain.items() if t == tup), None)
        if p is None:
            probs.append('no plain line for %s %s' % (tup, axis))
            continue
        if len(set(answers)) > 1:
            kind = varies(answers, p)
            if kind is None:
                probs.append('%s %s: the sub-windows answer %d ways, the plain line %r' % (tup, axis, len(set(answers)),
                                                                                         p[:80]))
            kinds.add(kind or 'FAIL')
        else:
            if p != answers[0]:
                probs.append('%s %s: one answer %r, the plain line %r' % (tup, axis, answers[0][:60], p[:60]))
            kinds.add('one')
    for (i, pins), lines in got.get('pins', []):
        tup, sub, axis, ans = hs[i]
        mine = [an for t, ax, an in lines if t == tup and ax == axis] or [an for t, ax, an in lines if t == tup]
        if mine and mine[0] != ans:
            probs.append('%s [%s] pinned %s: %r, the line %r' % (tup, sub, pins, mine[0][:60], ans[:60]))
        elif not mine:
            probs.append('%s [%s] pinned %s: no line' % (tup, sub, pins))
    return probs, kinds


def run_vector(R, rc_hash, store, row, items):
    """One refcore run of a copy of the vector's text with every variant ask appended."""
    vid = row['id']
    d = WORK + re.sub(r'[^A-Za-z0-9_.-]', '_', store + '__' + vid) + '/'
    if os.path.isdir(d):
        shutil.rmtree(d)
    files = []
    for p in row['files'].split():
        q = d + os.path.relpath(ROOT + p, ROOT + row['cwd'])
        os.makedirs(os.path.dirname(q), exist_ok=True)
        shutil.copyfile(ROOT + p, q)
        files.append((p, q))
    names = []
    for a, path, hs, variants in items:
        q = dict(files)[path]
        with open(q, 'a', encoding='utf-8') as fh:
            for name, line, _ in variants:
                fh.write('\n' + line + '\n')
                names.append(name)
    c = {'id': 'twins:' + vid, 'layer': 'unit', 'cwd': os.path.relpath(d, ROOT), 'note': '',
         'files': ' '.join(os.path.relpath(q, ROOT) for _, q in files), 'asks': ' '.join(names),
         'options': row.get('options', '')}
    e = R.run_one(c, 'pypy', R.BUDGET, False, rc_hash, TESTS + 'cache')
    return e['out']


def check_all():
    R = load_run()
    rc_hash = R.refcore_hash()
    table, fails = [], []
    for store in STORES:
        outs = stored(store)
        for row in rows(store):
            out = outs.get(row['id'])
            if out is None or 'windows each' not in ' '.join(open(ROOT + p, encoding='utf-8').read()
                                                              for p in row['files'].split()):
                continue
            items, left = plan(store, row, out)
            table += ['\t'.join([store, row['id'], a, '', why, '0', 'left out']) for a, why in left]
            if not items:
                continue
            text = run_vector(R, rc_hash, store, row, items)
            for a, path, hs, variants in items:
                got = {'plain': [(t, ax, an) for t, _, ax, an in heads(text, 'zz_plain_' + a)], 'pins': []}
                for name, _, meta in variants[1:]:
                    got['pins'].append((meta, [(t, ax, an) for t, _, ax, an in heads(text, name)]))
                probs, kinds = judge(hs, got)
                table.append('\t'.join([store, row['id'], a, str(len([h for h in hs if h[1]])),
                                        '+'.join(sorted(kinds)), str(len(variants) - 1), 'ok' if not probs else 'FAIL']))
                fails += ['%s %s %s: %s' % (store, row['id'], a, p) for p in probs]
    return ['store\tvector\task\tsub-window lines\tplain twin\tpinned twins\tverdict'] + table, fails


def self_test():
    ok = True

    def say(c, what):
        nonlocal ok
        ok &= bool(c)
        print(('ok    ' if c else 'FAIL  ') + what)
    hs = heads('q [wa = 2026-04-05 .. 2026-04-06]: refused: X\n  cut 2026-04-30\nq [wa = 2026-04-07 .. 2026-04-10]: '
               '100.00 PLN\nr: 1', 'q')
    say(len(hs) == 2 and hs[0][1] == 'wa = 2026-04-05 .. 2026-04-06' and hs[1][3] == '100.00 PLN', 'heads reads a '
        'sub-window and its answer')
    say(heads('d05 (m = 2026-03) [hh_round = *]: 1', 'd05')[0][:3] == ('(m = 2026-03)', '', 'hh_round = *'),
        'a tuple and an axis bracket, no sub-window')
    say(days('2026-04-05 .. 2026-04-10') == ['2026-04-05', '2026-04-07', '2026-04-10'], 'a run of days: first, '
        'middle, last')
    say(with_pin('ask q: bal(jar, 2026-04-30) law 2026-04-30 windows each', [('wa', '2026-04-05')]) ==
        'ask q: bal(jar, 2026-04-30) with wa = 2026-04-05 law 2026-04-30 windows each', 'a pin before law')
    say(with_pin('ask q: x for k in ks with b = 1', [('wa', '2026-04-05')]) == 'ask q: x for k in ks with b = 1, '
        'wa = 2026-04-05', 'a pin after the pins written')
    good = judge(hs, {'plain': [('', '', '?"date within window"')], 'pins': [((0, [('wa', '2026-04-05')]),
                                                                              [('', '', 'refused: X')])]})
    say(not good[0] and good[1] == {'hole'}, 'two answers and a plain hole: no problem')
    bad = judge(hs, {'plain': [('', '', '100.00 PLN')], 'pins': [((1, [('wa', '2026-04-07')]),
                                                                  [('', '', '90.00 PLN')])]})
    say(len(bad[0]) == 2, 'a plain answer where the days differ, and a pinned twin that differs, are both caught')
    one = judge([('', 'wa = 2026-04-05', '', '1'), ('', 'wa = 2026-04-06', '', '1')], {'plain': [('', '', '2')]})
    say(len(one[0]) == 1, 'one answer every day and a plain line that differs is caught')
    say(with_pin('ask q: a < b with a = window(2026-01-01, 2026-01-02), b = 1 windows each', [('a', '2026-01-02')])
        == 'ask q: a < b with a = 2026-01-02, b = 1 windows each', 'a pin of a name the ask pins takes its place')
    say(element_each('ask w: bal(k, 2026-03-30) windows each, bal(k, 2026-04-30) with ax = lo') and
        not element_each('ask w: bal(k, 2026-03-31), bal(k, 2026-04-30) with ax = lo windows each'),
        "an element's own windows each is told from the ask's")
    tup = [('', 'wd = 2026-03-28', '', '(10.00 PLN, 20.00 PLN)'), ('', 'wd = 2026-03-31', '', '(0.00 PLN, 20.00 PLN)')]
    say(not judge(tup, {'plain': [('', '', '(?"date within window", 20.00 PLN)')]})[0] and
        judge(tup, {'plain': [('', '', '(?"date within window", ?"date within window")')]})[0],
        'a tuple is judged by component: the hole only where its days differ')
    dif = [('', 'dj = 2026-03-02', '', 'diff in sd against sc: 1 changed'), ('', 'dj = 2026-03-04', '',
                                                                            'diff in sd against sc: 0 changed')]
    say(judge(dif, {'plain': [('', '', 'diff in sd against sc: 0 changed')]}) == ([], {'diff'}),
        "a diff's plain line, a count of rows, isn't judged")
    print('self-test: %s' % ('all hold' if ok else 'FAILED'))
    return 0 if ok else 1


def main(argv):
    if '--self-test' in argv:
        return self_test()
    table, fails = check_all()
    text = '\n'.join(table) + '\n'
    for f in fails:
        print('FAIL ' + f)
    if '--check' in argv:
        have = open(TABLE, encoding='utf-8').read() if os.path.exists(TABLE) else ''
        if have != text:
            print('map/twins.tsv differs from what refcore gives: run engine/tests/tools/twins.py')
            return 1
    else:
        open(TABLE, 'w', encoding='utf-8', newline='').write(text)
        print('wrote map/twins.tsv: %d asks' % (len(table) - 1))
    left = sum(r.endswith('\tleft out') for r in table[1:])
    print('twins: %d asks judged, %d left out, %d failures' % (len(table) - 1 - left, left, len(fails)))
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
