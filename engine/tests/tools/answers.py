"""The bridge from refcore runs to compare.py: copy printed values into an answers file, by a slot map.

It computes nothing. It finds the printed line a map names, takes the value at the named place, and turns the
printed form into compare.py's form:
- `1 234.56 DUK` -> the number 1234.56 (spaces between digit groups dropped, the unit dropped);
- `2047-08-05` -> the date string; any other word or text -> the text, verbatim;
- `X DUK (standard error S; N paths; ...)` -> {"value": X, "se": S}, compare.py's random form;
- a value with holes and one known term (`142 786.33 DUK + -1 x ?"..." (place) + ...`) ->
  {"kind": "partial", "value": the known term, "missing": [...]};
- a value of holes only (`?"..." (place) ?"..." (place)`) -> {"kind": "refusal", "missing": [...]};
- `refused: CODE text` -> {"kind": "refusal", "missing": ["refused: CODE text"]}.
"missing" holds one line per distinct hole question, in print order: `"the question" (place, place, ...)`, the places
of every printed term asking that question. A hole is named by its question, so nine monthly terms asking one
question name one missing fact (ERRATA ERR-33's open edge); the read lines show each, for a judge to agree or not.
The one change of sign it makes is a map's `"neg": true`: a printed signed figure turned into the key's direction
word (a printed -5 under the key's `refund` is a refund of 5). It applies to a number, a partial's known term and a
random value (not its se). Nothing else is ever changed.

A slot map is Markdown with one fenced yaml block per question, one field per line, each value in JSON:

    ```yaml
    question: A-01
    answer: {"2021": {"separately": {"Ania": {"refund": {"from": "a01_2021 [pit6_health = health_off]", "at": 0, "neg": true}}}}}
    without: {...}                                  # optional, as compare.py takes it
    note: free text for the reader                  # optional, never copied
    ```

The answer template mirrors the key's shape. Its dict keys are labels (the key's labels; a by-case's case labels as
the run prints them). Its leaves are refs, never literal values: {"from": "<the ask line's head, exactly as printed
before ': '>", "at": <index into a printed tuple, or a list of indexes for a nested one; omit for a whole value>,
"neg": <true to flip the sign>, "run": "<output file name, when two outputs print the same head differently>"}.
The one literal leaf allowed is a "kind" value ("by-case", "partial", "refusal"). A list in the template is a list of
refs. A ref may give "take": "value" or "take": "missing" to take one half of a printed partial, where the key splits
them across labels.

It refuses (exit 2, nothing written) when a named head isn't printed, a place is out of range, two outputs print a
head differently with no "run" given, a template holds a literal number, or a printed value can't be read.

Usage: python3 -I -B answers.py --map MAP.md --runs OUT ... --out ANSWERS.md
       python3 -I -B answers.py --self-test
"""
import json
import pathlib
import re
import sys
from decimal import Decimal

NUM = re.compile(r'^(-?(?:\d{1,3}(?: \d{3})+|\d+)(?:\.\d+)?)(?: ([^\s(][^\s]*))?$')
DATE = re.compile(r'^\d{4}-\d{2}-\d{2}$')
RANDOM = re.compile(r'^(.*?) \(standard error (-?[\d .]+?); ([^)]*)\)$')
HOLE = re.compile(r'\?(?:"((?:[^"\\]|\\.)*)"|([A-Z][A-Z0-9_]*))(?:\s*\(((?:[^()]|\([^()]*\))*)\))?')
KINDS = ('by-case', 'partial', 'refusal', 'random')


class Refuse(Exception):
    pass


def split_top(s, sep):
    """Split s on sep at depth 0, outside double quotes."""
    out, depth, cur, i, q = [], 0, [], 0, False
    while i < len(s):
        c = s[i]
        if q:
            cur.append(c)
            if c == '\\' and i + 1 < len(s):
                cur.append(s[i + 1])
                i += 1
            elif c == '"':
                q = False
        elif c == '"':
            q = True
            cur.append(c)
        elif c in '([{':
            depth += 1
            cur.append(c)
        elif c in ')]}':
            depth -= 1
            cur.append(c)
        elif depth == 0 and s.startswith(sep, i):
            out.append(''.join(cur))
            cur = []
            i += len(sep)
            continue
        else:
            cur.append(c)
        i += 1
    out.append(''.join(cur))
    return out


def outer_tuple(s):
    """The items of s if s is one parenthesised tuple, else None."""
    if not s.startswith('('):
        return None
    depth, q = 0, False
    for i, c in enumerate(s):
        if q:
            if c == '"':
                q = False
            continue
        if c == '"':
            q = True
        elif c == '(':
            depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0:
                return split_top(s[1:i], ', ') if i == len(s) - 1 else None
    return None


def number(s):
    m = NUM.match(s.strip())
    if not m:
        return None
    return Decimal(m.group(1).replace(' ', ''))


def holes(s):
    """Distinct hole questions in print order, each with the places of its terms."""
    order, places = [], {}
    for m in HOLE.finditer(s):
        q = '"%s"' % m.group(1) if m.group(1) is not None else m.group(2)
        if q not in places:
            order.append(q)
            places[q] = []
        if m.group(3) and m.group(3) not in places[q]:
            places[q].append(m.group(3))
    return ['%s (%s)' % (q, ', '.join(places[q])) if places[q] else q for q in order]


def value(s, neg=False):
    """A printed value in compare.py's form."""
    s = s.strip()
    sign = Decimal(-1) if neg else Decimal(1)
    if s.startswith('refused: '):
        return {'kind': 'refusal', 'missing': [s]}
    t = outer_tuple(s)
    if t is not None:
        return [value(x, neg) for x in t]
    m = RANDOM.match(s)
    if m and number(m.group(1)) is not None:
        se = number(m.group(2))
        if se is None:
            raise Refuse('unreadable standard error: %r' % s[:120])
        return {'value': sign * number(m.group(1)), 'se': se}
    if '?' in s and HOLE.search(s):
        terms = [x.strip() for x in split_top(s, ' + ')]
        known = [x for x in terms if not HOLE.search(x)]
        if not known:
            return {'kind': 'refusal', 'missing': holes(s)}
        if len(known) == 1 and number(known[0]) is not None:
            return {'kind': 'partial', 'value': sign * number(known[0]), 'missing': holes(s)}
        raise Refuse('a partial with %d known terms: %r' % (len(known), s[:160]))
    n = number(s)
    if n is not None:
        return sign * n
    if neg:
        raise Refuse('"neg" on a value that is not a number: %r' % s[:120])
    return s


def read_runs(paths):
    """{head: {file: printed value}} for every unindented answer line 'head: value'."""
    heads = {}
    for p in paths:
        name = pathlib.Path(p).name
        for ln in pathlib.Path(p).read_text(encoding='utf-8', errors='replace').splitlines():
            if not ln or ln[0] in ' \t#' or ln.startswith(('warning', 'rc=', 'time ', 'real', 'user', 'sys', 'exit')):
                continue
            head, sep, val = ln.partition(': ')
            if not sep:
                continue
            heads.setdefault(head, {})[name] = val
    return heads


def resolve(ref, heads):
    if set(ref) - {'from', 'at', 'neg', 'run', 'take'}:
        raise Refuse('unknown ref fields: %s' % sorted(set(ref) - {'from', 'at', 'neg', 'run', 'take'}))
    h = ref['from']
    if h not in heads:
        raise Refuse('not printed: %r' % h)
    got = heads[h]
    if 'run' in ref:
        if ref['run'] not in got:
            raise Refuse('%r not printed in %s' % (h, ref['run']))
        txt = got[ref['run']]
    else:
        if len(set(got.values())) > 1:
            raise Refuse('%r printed differently in %s; give "run"' % (h, sorted(got)))
        txt = next(iter(got.values()))
    at = ref.get('at')
    if at is not None:
        for i in ([at] if isinstance(at, int) else at):
            items = outer_tuple(txt.strip())
            if items is None or not 0 <= i < len(items):
                raise Refuse('%r has no place %r' % (h, at))
            txt = items[i].strip()
    v = value(txt, bool(ref.get('neg')))
    take = ref.get('take')
    if take:
        if not (isinstance(v, dict) and v.get('kind') == 'partial'):
            return v if take == 'value' else []
        v = v[take]
    return v


def fill(tpl, heads):
    if isinstance(tpl, dict):
        if 'from' in tpl:
            return resolve(tpl, heads)
        out = {}
        for k, v in tpl.items():
            if k == 'kind':
                if v not in KINDS:
                    raise Refuse('kind %r' % v)
                out[k] = v
            else:
                out[k] = fill(v, heads)
        return out
    if isinstance(tpl, list):
        return [fill(v, heads) for v in tpl]
    raise Refuse('a literal leaf in the template: %r' % (tpl,))


def dump(x):
    """JSON with Decimals written exactly."""
    if isinstance(x, dict):
        return '{' + ', '.join('%s: %s' % (json.dumps(k, ensure_ascii=False), dump(v)) for k, v in x.items()) + '}'
    if isinstance(x, list):
        return '[' + ', '.join(dump(v) for v in x) + ']'
    if isinstance(x, Decimal):
        return str(x)
    return json.dumps(x, ensure_ascii=False)


def maps(text):
    out = []
    for b in re.findall(r'```yaml\n(.*?)\n```', text, re.S):
        f = {}
        for ln in b.splitlines():
            k, _, v = ln.partition(': ')
            k, v = k.strip(), v.strip()
            if k in ('answer', 'without'):
                f[k] = json.loads(v)
            else:
                f[k] = v
        out.append(f)
    return out


def build(map_text, run_paths):
    heads = read_runs(run_paths)
    blocks = []
    for m in maps(map_text):
        q = m['question']
        try:
            lines = ['question: %s' % q, 'answer: %s' % dump(fill(m['answer'], heads))]
            if 'without' in m:
                lines.append('without: %s' % dump(fill(m['without'], heads)))
        except Refuse as e:
            raise Refuse('%s: %s' % (q, e))
        blocks.append('```yaml\n' + '\n'.join(lines) + '\n```')
    return '# Answers\n\nWritten by `tmp/session5/tools/answers.py` from the runs %s; don\'t edit.\n\n' % (
        ', '.join('`%s`' % pathlib.Path(p).name for p in run_paths)) + '\n\n'.join(blocks) + '\n'


def self_test():
    import tempfile
    run = '\n'.join([
        'a01_2021 [pit6_health = health_off]: (-5.00 DUK, -3.00 DUK, 187 DUK, -195 DUK)',
        '  reading: pit6_health = health_off',
        'a01_2022_causes (c = rate_cut) [pit6_health = *]: (rate_cut, 7 860 DUK)',
        'b05_last: (2047-08-05, 2047-03-05, -5)',
        'b08_cards: 35 368.79 DUK + 1 × ?"What did Piotr pay, on days a, b?" (gap_cards)',
        'b11: (4.00 DUK + -1 × ?"Q one?" (x[2025-04]) + -1 × ?"Q one?" (x[2025-05]) + -1 × ?NAMED (y), 1 DUK)',
        'mca: (?"date not precise, here" (cards[2026-01]) ?"another" (cards[2026-01]), refused: RUN-ORDER a and b read each other)',
        'd10: (18.455 (standard error 0.583646; 200 paths; horizon none), 37 of 200)',
        'nested: ((1 DUK, 2 DUK), 3 DUK)',
    ]) + '\n'
    mp = '\n'.join([
        '```yaml', 'question: T-01',
        'answer: {"sep": {"Ania": {"refund": {"from": "a01_2021 [pit6_health = health_off]", "at": 0, "neg": true}}}, '
        '"case": {"kind": "by-case", "cases": {"health_off": {"Ania": {"payable": {"from": "a01_2021 [pit6_health = health_off]", "at": 2}}}}}, '
        '"cause": [{"from": "a01_2022_causes (c = rate_cut) [pit6_health = *]", "at": 0}, {"from": "a01_2022_causes (c = rate_cut) [pit6_health = *]", "at": 1}], '
        '"last": {"from": "b05_last", "at": 0}, "cards": {"from": "b08_cards"}, "p": {"from": "b11", "at": 0}, '
        '"pv": {"from": "b11", "at": 0, "take": "value"}, "h": {"from": "mca", "at": 0}, "r": {"from": "mca", "at": 1}, '
        '"rnd": {"from": "d10", "at": 0}, "cnt": {"from": "d10", "at": 1}, "n": {"from": "nested", "at": [0, 1]}}',
        '```'])
    with tempfile.TemporaryDirectory() as d:
        rp = pathlib.Path(d) / 'r.out'
        rp.write_text(run, encoding='utf-8')
        out = build(mp, [str(rp)])
        a = json.loads(re.search(r'answer: (.*)', out).group(1), parse_float=Decimal, parse_int=Decimal)
        assert a['sep']['Ania']['refund'] == Decimal('5.00'), a['sep']
        assert a['case']['cases']['health_off']['Ania']['payable'] == 187
        assert a['cause'] == ['rate_cut', 7860]
        assert a['last'] == '2047-08-05'
        assert a['cards'] == {'kind': 'partial', 'value': Decimal('35368.79'), 'missing': ['"What did Piotr pay, on days a, b?" (gap_cards)']}, a['cards']
        assert a['p'] == {'kind': 'partial', 'value': 4, 'missing': ['"Q one?" (x[2025-04], x[2025-05])', 'NAMED (y)']}, a['p']
        assert a['pv'] == 4
        assert a['h'] == {'kind': 'refusal', 'missing': ['"date not precise, here" (cards[2026-01])', '"another" (cards[2026-01])']}, a['h']
        assert a['r'] == {'kind': 'refusal', 'missing': ['refused: RUN-ORDER a and b read each other']}
        assert a['rnd'] == {'value': Decimal('18.455'), 'se': Decimal('0.583646')}
        assert a['cnt'] == '37 of 200'
        assert a['n'] == 2
        for bad in ('{"x": 5}', '{"x": {"from": "nope"}}', '{"x": {"from": "b05_last", "at": 9}}',
                    '{"x": {"from": "a01_2022_causes (c = rate_cut) [pit6_health = *]", "at": 0, "neg": true}}'):
            try:
                build('```yaml\nquestion: T-02\nanswer: %s\n```' % bad, [str(rp)])
            except Refuse:
                continue
            raise AssertionError('not refused: ' + bad)
        rp2 = pathlib.Path(d) / 'r2.out'
        rp2.write_text('b05_last: (2047-08-06, 2047-03-05, -5)\n', encoding='utf-8')
        try:
            build('```yaml\nquestion: T-03\nanswer: {"x": {"from": "b05_last", "at": 0}}\n```', [str(rp), str(rp2)])
            raise AssertionError('two runs differing not refused')
        except Refuse:
            pass
        assert '2047-08-06' in build('```yaml\nquestion: T-03\nanswer: {"x": {"from": "b05_last", "at": 0, "run": "r2.out"}}\n```', [str(rp), str(rp2)])
    print('answers.py self-test: pass (12 forms copied, 5 refusals)')


def main(argv):
    if '--self-test' in argv:
        self_test()
        return 0
    mp = argv[argv.index('--map') + 1]
    out = argv[argv.index('--out') + 1]
    i = argv.index('--runs') + 1
    runs = []
    while i < len(argv) and not argv[i].startswith('--'):
        runs.append(argv[i])
        i += 1
    try:
        text = build(pathlib.Path(mp).read_text(encoding='utf-8'), runs)
    except Refuse as e:
        print('answers.py refuses: %s' % e)
        return 2
    pathlib.Path(out).write_text(text, encoding='utf-8')
    print('answers.py: %d answers from %d runs -> %s' % (text.count('```yaml'), len(runs), out))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
