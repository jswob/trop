"""M0's property generator (issue #56; PROPERTIES.md § M0). Standard library only.

Writes m0.corpus: records in kernel.corpus's form (five netstrings: the law, the case id, the variant, the argument
words, the file table), each run twice by laws_m0.py and by the runner, as the load probe (its argument words,
`run <files> --ask zz_m0_probe`) and as `reprint <files>`. Seeded, so the corpus is the same bytes on every run;
`--check` fails when the file on disk differs from what the seeds give.

The laws (PROPERTIES.md § M0):

  spelling  a corpus text and its rewrites that the grammar allows and L §2.4 makes equal (sections shuffled, two
            blank lines between sections, a version's items shuffled, a def's rules shuffled, `beats` reversed, a
            one-line def written long with `rule base`) reprint to the same bytes; the control changes a figure
  order     a corpus text split over two files (a model file and a `part`), given in either order, loads as the
            whole text and reprints the same in either order; the control changes a figure
  total     a corpus text mutated by one operator (12: a line deleted, duplicated or swapped with the next; a
            token deleted or duplicated; an id made a keyword; a tab; an indent off by one; the text cut short; a
            byte that isn't UTF-8; a bracket left open; CRLF line ends) loads clean or is refused by one LOAD-* line,
            and reprints or is refused by one line
  depth     an expression nested d deep (brackets, calls, `if`s; `as if` pins) is refused LOAD-DEPTH past 8 and
            LOAD-ASIF past 4 (L §2.3), and a chain of operators at one level never is

Sources: the texts of checks.tsv's one-file sets (unit probes and models) that load clean on refcore (their
m0/load.refcore probe prints the REQ-ASK line alone), short ones only (the corpus is embedded in the runner), no joint
model (its members aren't in a one-file record); none
that uses `schedule(`, `fulfils:` or `settles` (their probe's bytes hang on issue #72, load checks refcore makes only
where an ask reaches the item); no record asks a scenario's ask, so none hangs on #73.

    python3 -I -B engine/tests/properties/gen_m0.py            -> writes m0.corpus
    python3 -I -B engine/tests/properties/gen_m0.py --check    -> exit 1 if m0.corpus isn't what the seeds give
"""
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
T = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(T)) + os.sep
CORPUS = os.path.join(HERE, 'm0.corpus')
LOAD = os.path.join(T, 'm0', 'load.refcore')
CHECKS = os.path.join(T, 'checks.tsv')
SEED = 20261104
PROBE = 'zz_m0_probe'
PROBED = 'refused: REQ-ASK no ask %s\n' % PROBE
SPELLINGS, ORDERS, MUTANTS = 16, 10, 10   # sources per law (the corpus is embedded: kept near 400 KB)
EXCLUDE = ('schedule(', 'fulfils:', 'settles')   # issue #72
CLAUSES = ('cite', 'governs', 'known', 'covers', 'pad', 'anchor', 'moves', 'note', 'enters', 'assumed', 'same')


def ns(b):
    b = b if isinstance(b, bytes) else b.encode('utf-8')
    return b'%d:' % len(b) + b + b','


def netstrings(b):
    out, at = [], 0
    while at < len(b):
        colon = b.index(b':', at)
        n = int(b[at:colon])
        out.append(b[colon + 1:colon + 1 + n])
        at = colon + n + 2
    return out


def record(law, case, variant, files):
    """One record: its files (name, text or bytes), given in this order to the load probe."""
    args = ' '.join(['run'] + [n for n, _ in files] + ['--ask', PROBE])
    return ns(law) + ns(case) + ns(variant) + ns(args) + ns(b''.join(ns(n) + ns(t) for n, t in files))


def sources():
    """[(set id, text)]: the clean one-file texts of checks.tsv, sorted by id; and the ones #72 leaves out."""
    ns_ = netstrings(open(LOAD, 'rb').read())
    clean = {ns_[k].decode() for k in range(0, len(ns_), 7) if ns_[k + 6].decode() == PROBED}
    rows = [l.split('\t') for l in open(CHECKS, encoding='utf-8').read().split('\n')[1:] if l]
    sets = {}
    for r in rows:
        cid, layer, cwd, files = r[0], r[1], r[2], r[3]
        if layer == 'interpreter' or len(files.split()) != 1 or r[5] == 'script':
            continue
        sets.setdefault(files, []).append(cid)
    out, left = [], []
    for files, ids in sorted(sets.items()):
        sid = min(ids)
        if sid not in clean or not (sid.startswith('U:') or sid.startswith('I:M:')):
            continue
        text = open(ROOT + files, encoding='utf-8').read()
        if any(x in text for x in EXCLUDE):
            left.append(sid)
        elif re.search(r'^\s+use ', text, re.M):
            continue  # a joint model: it loads only with its members, which a one-file record lacks
        elif '\t' not in text and '\r' not in text and text.count('\n') <= 70:
            out.append((sid, text))
    return sorted(out), left


def blocks(text):
    """The text's sections: blocks split at blank lines, a block that starts indented joined to the one before."""
    out = []
    for b in text.strip('\n').split('\n\n'):
        if out and (b.startswith(' ') or b.startswith('#')):
            out[-1] += '\n\n' + b
        else:
            out.append(b)
    return out


def join(bs, sep='\n\n'):
    return sep.join(bs) + '\n'


def shuffled_sections(text, rng):
    bs = blocks(text)
    rest = bs[1:]
    rng.shuffle(rest)
    return join([bs[0]] + rest)


def blank_lines(text, rng):
    return join(blocks(text), '\n\n\n')


def shuffled_lines(text, rng, head, pick):
    """In each block whose first line starts with `head`, the single-line items `pick` chooses shuffled in place."""
    out = []
    for b in blocks(text):
        lines = b.split('\n')
        if lines[0].startswith(head) and all(not l.startswith('   ') for l in lines):
            at = [i for i, l in enumerate(lines) if i and pick(l)]
            items = [lines[i] for i in at]
            rng.shuffle(items)
            for i, l in zip(at, items):
                lines[i] = l
        out.append('\n'.join(lines))
    return join(out)


def version_items(text, rng):
    return shuffled_lines(text, rng, 'version ', lambda l: l.startswith('  ') and l.split()[0] not in CLAUSES)


def def_rules(text, rng):
    return shuffled_lines(text, rng, 'def ', lambda l: l.startswith('  rule '))


def beats_reversed(text, rng):
    def rev(m):
        return 'beats ' + ', '.join(reversed(m.group(1).split(', ')))
    return re.sub(r'beats ([a-z0-9_]+(?:, [a-z0-9_]+)+)', rev, text)


def short_written_long(text, rng):
    """One one-line def with no `at`, `note` or rule after it, written as its long form with a plain `rule base`."""
    lines = text.split('\n')
    for i, l in enumerate(lines):
        m = re.match(r'(def [a-z][a-z0-9_]*(?:\[[^\]]*\])?: [^=]+?) = (.+)$', l)
        nxt = lines[i + 1] if i + 1 < len(lines) else ''
        if m and ' at ' not in m.group(2) and not nxt.startswith(' ') and '"' not in l:
            lines[i:i + 1] = [m.group(1), '  rule base = ' + m.group(2)]
            return '\n'.join(lines)
    return text


def figure_changed(text, rng):
    """The control: the last digit of one version row's figure, plus one."""
    lines = text.split('\n')
    at = [i for i, l in enumerate(lines) if re.match(r'  [a-z][a-z0-9_]*(\[[^\]]*\])? = -?[0-9][0-9 ]*(\.[0-9]+)?\b', l)]
    if not at:
        return text
    i = rng.choice(at)
    m = re.match(r'(  [^=]+ = -?[0-9 ]*?)([0-9])((?:\.[0-9]*[0-9])?)\b', lines[i])
    if not m:
        return text
    head, digit, tail = m.group(1), m.group(2), m.group(3)
    if tail:
        tail = tail[:-1] + str((int(tail[-1]) + 1) % 10)
    else:
        digit = str((int(digit) + 1) % 10)
    lines[i] = head + digit + tail + lines[i][m.end():]
    return '\n'.join(lines)


REWRITES = [('sections', shuffled_sections), ('blank', blank_lines), ('items', version_items), ('rules', def_rules),
            ('beats', beats_reversed), ('long', short_written_long)]


def spelling(srcs, rng):
    out = []
    figures = [s for s in srcs if figure_changed(s[1], random.Random(0)) != s[1]]  # those the control can change
    for sid, text in rng.sample(figures, SPELLINGS):
        case = 'spelling/' + sid
        out.append(record('spelling', case, 'base', [('m.tally', text)]))
        for name, f in REWRITES:
            t = f(text, rng)
            if t != text:
                out.append(record('spelling', case, name, [('m.tally', t)]))
        c = figure_changed(text, rng)
        if c != text:
            out.append(record('spelling', case, 'control', [('m.tally', c)]))
    return out


def split(text, rng):
    """The text as two files: the model file (its header and the first sections) and a part with the rest."""
    bs = blocks(text)
    k = rng.randint(1, max(1, len(bs) - 1))
    return join(bs[:k]), join(['part rest\n  tally 2'] + bs[k:])


def order(srcs, rng):
    out = []
    for sid, text in rng.sample([s for s in srcs if len(blocks(s[1])) >= 4], ORDERS):
        case = 'order/' + sid
        a, b = split(text, rng)
        out.append(record('order', case, 'whole', [('m.tally', text)]))
        out.append(record('order', case, 'split', [('a.tally', a), ('b.tally', b)]))
        out.append(record('order', case, 'reversed', [('b.tally', b), ('a.tally', a)]))
        c = figure_changed(b, rng)
        if c != b:
            out.append(record('order', case, 'control', [('a.tally', a), ('b.tally', c)]))
    return out


def tokens(line):
    return [m for m in re.finditer(r'[A-Za-z_][A-Za-z0-9_]*|[0-9]+|\S', line)]


def mutate(op, text, rng):
    lines = text.split('\n')
    i = rng.randrange(1, max(2, len(lines) - 1))
    if op == 'delete-line':
        del lines[i]
    elif op == 'duplicate-line':
        lines.insert(i, lines[i])
    elif op == 'swap-lines':
        j = min(i + 1, len(lines) - 1)
        lines[i], lines[j] = lines[j], lines[i]
    elif op in ('delete-token', 'duplicate-token', 'keyword'):
        cand = [k for k, l in enumerate(lines) if tokens(l)]
        k = rng.choice(cand)
        m = rng.choice(tokens(lines[k]))
        new = {'delete-token': '', 'duplicate-token': m.group() + ' ' + m.group(), 'keyword': 'rule'}[op]
        lines[k] = lines[k][:m.start()] + new + lines[k][m.end():]
    elif op == 'tab':
        lines[i] = '\t' + lines[i]
    elif op == 'indent':
        lines[i] = ' ' + lines[i]
    elif op == 'truncate':
        return text[:rng.randrange(1, len(text))]
    elif op == 'non-utf8':
        cut = rng.randrange(1, len(text))
        return text[:cut].encode('utf-8') + b'\xff' + text[cut:].encode('utf-8')
    elif op == 'unclosed':
        at = [k for k, ch in enumerate(text) if ch in ')]']
        if not at:
            return None
        k = rng.choice(at)
        return text[:k] + text[k + 1:]
    elif op == 'crlf':
        return text.replace('\n', '\r\n')
    return '\n'.join(lines)


OPERATORS = ['delete-line', 'duplicate-line', 'swap-lines', 'delete-token', 'duplicate-token', 'keyword', 'tab',
             'indent', 'truncate', 'non-utf8', 'unclosed', 'crlf']


def total(srcs, rng):
    out = []
    for sid, text in rng.sample([s for s in srcs if s[1].count('\n') <= 40], MUTANTS):
        for op in OPERATORS:
            t = mutate(op, text, rng)
            if t is not None and t != text:
                out.append(record('total', 'total/%s/%s' % (sid, op), 'mutant', [('m.tally', t)]))
    return out


DEPTH_BASE = '''model depth
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31
  note "T's generated reality for L §2.3's depth (invented)"

source card "T's card (invented)"

input a: num

input b: num

input c: bool

def v: num = %s

ask q: %s

version card_2026
  cite card
  governs always
  known 2026-01-01
  a = 1 "the card"
  b = 2 "the card"
  c = true "the card"
'''


def nested(form, d):
    e = 'a'
    for _ in range(d):
        e = {'parens': '(%s)', 'call': 'max(%s, b)', 'if': 'if c then %s else b', 'asif': '(%s as if {a = b})',
             'chain': None}[form] % e
    return e


def depth():
    out = []
    for form, top in [('parens', 12), ('call', 12), ('if', 12), ('asif', 6)]:
        for d in range(1, top + 1):
            text = DEPTH_BASE % (('a + b', nested(form, d)) if form == 'asif' else (nested(form, d), 'v'))
            out.append(record('depth', 'depth/' + form, 'd%02d' % d, [('m.tally', text)]))
    for n in (2, 10, 30, 60):
        text = DEPTH_BASE % (' + '.join(['a', 'b'] * (n // 2)), 'v')
        out.append(record('depth', 'depth/chain', 'n%02d' % n, [('m.tally', text)]))
    return out


def corpus():
    rng = random.Random(SEED)
    srcs, left = sources()
    data = b''.join(spelling(srcs, rng) + order(srcs, rng) + total(srcs, rng) + depth())
    return data, len(srcs), left


def main(argv):
    data, n, left = corpus()
    count = len(netstrings(data)) // 5
    if '--check' in argv:
        old = open(CORPUS, 'rb').read() if os.path.exists(CORPUS) else b''
        if old != data:
            print('m0.corpus is not what gen_m0.py\'s seeds give: run properties/gen_m0.py')
            return 1
        print('m0.corpus: %d records, what the seeds give' % count)
        return 0
    with open(CORPUS, 'wb') as f:
        f.write(data)
    print('m0.corpus: %d records from %d clean one-file texts (%d bytes); left out for #72: %s' % (
        count, n, len(data), ' '.join(left) or 'none'))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
