"""The canonical format (L §2.4) and the reality's hash (E §3.1 step 1).

reprint(files, members) -> [(path, canonical text)]; reality_hash(files, mode, members) -> 64 lower-case hex digits
(members: a joint model's members' files by alias, for the order of keys of a member's index or kind).

E §3.1 step 1: "Parse every file, and reprint it in the canonical format (LANGUAGE §2.4). The reality's hash is the
sha256 of the UTF-8 bytes of its canonical text, its files reprinted and taken in the order of their paths in its
directory tree (compared by code point), so spacing never changes a hash".  The reality's canonical text is read
here as its files' reprints, each ending with its newline, joined in that order (HANDOVER, writer 8's second pass).

The reprint changes what L §2.4 states and nothing else:
- the header's lines in their order, `use` lines by name, `same` lines by left side;
- sections in kind order, by id; kinds in tree order (each followed by its subkinds, depth first, siblings by id);
  versions last by `known` (a window by its first day), then id; a regime's items in that order one level in, then
  its `export` line and its notes; one blank line between sections, none inside one;
- inside a section its clause lines and notes first, in the grammar's order (a kind's leg, attr and note lines are
  one repeated choice there, so they keep their order), then rules (by label, the `known` of their `v`, none first, then their own
  `governs`) or version items (entries first, then target id, keys in index order, keyless first, then their own
  span or instant); a band's rows keep their order;
- the short form of a def with one plain `base` rule; a `beats` list sorted with ", ";
- numbers (shortest exact decimal, thousands grouped by a space, a reduced a/b); expressions with one space around
  binary operators and after commas, none inside brackets, parentheses only where the parse needs them, and always
  around a `... to u` operand and a converted `flows` or `value` in a comparison.
What the section leaves open keeps the file's own order and spelling: the order of repeated lines of one clause
(legs, attributes, `before`/`after` lines, axis options, notes, a kind's lines), the spelling of a word the grammar accepts in two
forms, the spacing of tokens §2.4 doesn't name (taken from the spec's examples: `f(x)`, `x[k]`, `a: T`, `?"q"`,
`8 PLN/hour`, `19 %`, a version's span `2026-01..` touching, an expression's `a .. b` spaced).  Where two rules of the
section order one input two ways, or name no order for two items that differ, the reprint is REFCORE-UNSPECIFIED.
"""
import datetime
import hashlib
import heapq
from functools import cmp_to_key

from .errors import LoadError, TallyError, Unspecified, NotYet
from .lexer import split_lines, Tok
from .node import Node, walk
from .parser import parse_file, parse_section, prescan_units, BUILTINS, KEYWORDS

SECTION_ORDER = ('source', 'unitdecl', 'kind', 'axis', 'index', 'place', 'input', 'band', 'def', 'fill', 'move',
                 'ask', 'version')
OPERATORS = ('bin', 'cmp', 'in', 'neg', 'not', 'and', 'or')
LIST_WORDS = ('weekends', 'yearly', 'added')       # calendar(weekends [sat, sun], yearly [...], added [...])

# body lines of a section: "its clause lines and notes first, in the grammar's order (§2.2)" (L §2.4, round 17), and
# "Clause order is the order in the grammar"; lines of one rank keep their order. A kind's lines are one repeated
# choice in the grammar (kline = leg | attr | note), so they keep the file's order
BODY_RANK = {
    'kind': {'note': 0, 'leg': 0, 'attr': 0},
    'index': {'v': 0, 'assumed': 1, 'note': 2},
    'input': {'note': 0},
    'band': {'note': 0},
    'fill': {'v': 0, 'note': 1, 'supply': 2, 'demand': 3, 'arc': 4, 'order': 5},
    'move': {'on': 1, 'at': 2, 'for': 3, 'when': 4, 'reads': 5, 'before': 6, 'after': 6, 'ties': 7,
             'attr': 8, 'leg': 9, 'fulfils': 10, 'note': 11},
}
HEADER_ORDER = ('tally', 'on', 'start', 'end', 'use', 'same', 'law', 'from', 'note')


# -- the hash ----------------------------------------------------------------------------------------------------

def reality_hash(files, mode=None, members=None):
    """files: [(path in the reality's tree, source)].  mode 'bytes': the sha256 of the files' bytes as given, in path
    order (refcore's test convenience, `--pin=bytes`); otherwise of the canonical text (E §3.1 step 1)."""
    files = sorted(files, key=lambda f: f[0])
    if mode == 'bytes':
        data = b''.join(src.encode('utf-8') for p, src in files)
    else:
        data = ''.join(text for p, text in reprint(files, members)).encode('utf-8')
    return hashlib.sha256(data).hexdigest()


def reprint(files, members=None):
    """[(path, source)] -> [(path, canonical text)], each reprinted with the declarations of all of them (a key's order
    is its index's, a kind's parent may sit in another file, a rule's `v` names a version anywhere).  members: {alias:
    [(path, source)]}, a joint model's members, whose indexes and kinds order the keys of its items keyed by them (L
    §2.4 "keys in index order"; L §6.1 a member's items are read as a.item)."""
    lined = [(p, split_lines(src, p)) for p, src in files]
    units = prescan_units([ls for p, ls in lined])
    parsed = [(p, ls, parse_file(ls, units)) for p, ls in lined]
    table = Table([t for p, ls, t in parsed])
    for alias, mfiles in sorted((members or {}).items()):
        mlined = [(p, split_lines(src, p)) for p, src in mfiles]
        try:
            munits = prescan_units([ls for p, ls in mlined])
            mtrees = [parse_file(ls, munits) for p, ls in mlined]
        except TallyError:
            continue                    # a member that doesn't parse is refused at its own pin
        for t in mtrees:
            for sec in t.sections:
                table.add_member(alias, sec)
    return [(p, FilePrinter(ls, tree, units, table).text()) for p, ls, tree in parsed]


# -- what the reality declares -----------------------------------------------------------------------------------

class Table:
    def __init__(self, trees):
        self.indexes = {}
        self.kinds = set()
        self.binds = {}
        self.versions = {}
        for t in trees:
            for sec in t.sections:
                self.add(sec, None)

    def add(self, sec, qual):
        k = sec.k
        if k == 'index':
            self.indexes[sec.id] = sec
        elif k == 'kind':
            self.kinds.add(sec.id if qual is None else qual + '.' + sec.id)
        elif k == 'version':
            self.versions[sec.id] = sec.known
        if k in ('input', 'def', 'band', 'place', 'fill', 'move'):
            self.binds[(qual, sec.id)] = getattr(sec, 'binds', None) or []

    def add_member(self, alias, sec):
        """A member's index or kind, read by the joint model as a.ix, a.kind (L §6.1)."""
        if sec.k == 'index':
            self.indexes[alias + '.' + sec.id] = sec
        elif sec.k == 'kind':
            self.kinds.add(alias + '.' + sec.id)

    def key_order(self, dom):
        """How keys at a position over the domain dom sort (L §2.4 "keys in index order (things and an open index's
        keys in the text order of their stable ids, §2.5)"): 'pos' {text: i}, 'value' or 'text'."""
        if dom is None:
            return ('text', None)
        if dom.k == 'dlist':
            return ('pos', dict((key_text(x), i) for i, x in enumerate(dom.keys)))
        if dom.k == 'drange':
            return ('value', None)
        if dom.k == 'dname':
            if dom.qual is None and dom.id in self.indexes:
                ix = self.indexes[dom.id]
                if ix.form == 'list':
                    return ('pos', dict((key_text(x), i) for i, x in enumerate(ix.keys)))
                if ix.form == 'range':
                    return ('value', None)
                return ('text', None)
            if dom.qual is None and dom.id in ('days', 'months', 'years'):
                return ('value', None)
            if dom.qual is not None and dom.qual + '.' + dom.id in self.indexes:
                ix = self.indexes[dom.qual + '.' + dom.id]
                if ix.form == 'list':
                    return ('pos', dict((key_text(x), i) for i, x in enumerate(ix.keys)))
                if ix.form == 'range':
                    return ('value', None)
                return ('text', None)
            if dom.qual is not None and dom.qual + '.' + dom.id not in self.kinds:
                # a member's index or kind (k in a.ix, L §6.1): its order is the member's, which this reprint doesn't
                # load
                return ('member', dom.qual + '.' + dom.id)
            return ('text', None)
        return ('text', None)


def key_text(k):
    if k is None:
        return ''
    if k.k == 'name':
        return k.id
    if k.k == 'key':
        return k.v
    if k.k == 'num':
        return k.text
    if k.k == 'date':
        return k.v.isoformat()
    if k.k == 'month':
        return '%04d-%02d' % k.v
    if k.k == 'quarter':
        return '%04d-Q%d' % k.v
    if k.k == 'bool':
        return 'true' if k.v else 'false'
    if k.k == 'cur':
        return k.v
    if k.k == 'star':
        return '*'
    if k.k == 'krange':
        return key_text(k.a) + '..' + key_text(k.b)
    if k.k == 'placekey':
        return '%s[%s]' % (k.id, ', '.join(key_text(x) for x in k.keys))
    return repr(k)


def key_value(k):
    if k.k == 'num':
        return (0, k.v)
    if k.k == 'date':
        return (1, k.v)
    if k.k == 'month':
        return (1, datetime.date(k.v[0], k.v[1], 1))
    if k.k == 'quarter':
        return (1, datetime.date(k.v[0], 3 * k.v[1] - 2, 1))
    return (2, key_text(k))


def first_day(lit):
    """The first day of a version's span end (a day, a month, a quarter, a year), for sorting."""
    if lit is None:
        return None
    if lit.k == 'date':
        return lit.v
    if lit.k == 'month':
        return datetime.date(lit.v[0], lit.v[1], 1)
    if lit.k == 'quarter':
        return datetime.date(lit.v[0], 3 * lit.v[1] - 2, 1)
    if lit.k == 'num' and lit.isint and lit.unit is None:
        return datetime.date(int(lit.v), 1, 1)
    if lit.k == 'window':
        return lit.a
    return None


# -- sort components ---------------------------------------------------------------------------------------------

class Open:
    """A sort component L §2.4 gives no order for: two that differ are REFCORE-UNSPECIFIED when the order of their
    items rests on them (why: the Unspecified to raise; or 'member …', a key of a member's index or kind)."""

    def __init__(self, text, why):
        self.text = text
        self.why = why


class Tie:
    """L §2.4 (round 19): a sort component with a rank and a value; two items equal on both (two rows with no start,
    two rows with one start, two `after m` rows) sort by their own canonical line (GD-28, the next component)."""

    def __init__(self, rank, val, text, why):
        self.rank, self.val, self.text, self.why = rank, val, text, why


class Rev:
    """A component sorted in reverse (a range's last key: the wider range first, L §2.4)."""

    def __init__(self, v):
        self.v = v

    def __eq__(self, o):
        return isinstance(o, Rev) and self.v == o.v

    def __lt__(self, o):
        return o.v < self.v

    def __gt__(self, o):
        return o.v > self.v

    def __repr__(self):
        return 'Rev(%r)' % (self.v,)


def cmp_comp(x, y):
    if isinstance(x, Tie) and isinstance(y, Tie):
        a, b = (x.rank, x.val), (y.rank, y.val)
        if a != b:
            return (a > b) - (a < b)
        # GD-28: what every key leaves tied sorts by the item's own canonical line, the last component
        return 0
    if isinstance(x, Open) or isinstance(y, Open):
        tx = x.text if isinstance(x, Open) else ('=', repr(x))
        ty = y.text if isinstance(y, Open) else ('=', repr(y))
        if tx == ty:
            return 0
        o = x if isinstance(x, Open) else y
        if isinstance(o.why, str):
            raise NotYet('the canonical order of keys of %s (L §2.4), an index or kind of a member' % o.why[7:])
        raise o.why
    return (x > y) - (x < y)


def ascii_key(text):
    """L §2.4 (GD-28): a canonical line as a sort component, "compared as ASCII" (byte order of its UTF-8 text)."""
    return (9, text.encode('utf-8'))


def cmp_keys(a, b):
    for x, y in zip(a, b):
        c = cmp_comp(x, y)
        if c:
            return c
    return (len(a) > len(b)) - (len(a) < len(b))


def stable_sort(items, keyf):
    """Sort [(key components, payload)] by components (cmp_comp), equal ones keeping their order."""
    dec = [(keyf(x), i, x) for i, x in enumerate(items)]
    dec.sort(key=cmp_to_key(lambda p, q: cmp_keys(p[0], q[0]) or (p[1] > q[1]) - (p[1] < q[1])))
    return [x for k, i, x in dec]


# -- spacing -----------------------------------------------------------------------------------------------------

def line_text(toks):
    out = []
    prev = None
    for t in toks:
        s = t.canon if t.canon is not None else t.s
        if prev is not None and spaced(prev, t):
            out.append(' ')
        out.append(s)
        prev = t
    return ''.join(out)


def spaced(p, t):
    """Whether one space separates tokens p and t (L §2.4 Expressions: "one space around binary operators and after
    commas, none inside brackets"; L §2.1 units "written without spaces inside them"; the rest as the spec's
    examples write it)."""
    if t.role == 'u':
        return False
    if p.s in ('(', '[', '{'):
        return False
    if t.s in (')', ']', '}', ',', ':'):
        return False
    if t.s == '.' or p.s == '.':
        return False
    if t.s == '..':
        return t.role in ('bin', 'lo')
    if p.s == '..':
        return p.role in ('bin', 'hi')
    if p.s == '?':
        return False
    if p.s == '-' and p.t == 'OP' and p.role != 'bin':
        return False
    if t.s == '(' and t.t == 'OP':
        if p.s == ']':
            return False
        if p.t == 'ID' and (p.s in BUILTINS or (p.s not in KEYWORDS and p.nm)):
            return False
        return True
    if t.s == '[' and t.t == 'OP':
        if p.s in (']', ')'):
            return False
        if p.t == 'ID' and p.s not in KEYWORDS and p.s not in LIST_WORDS:
            return False
        return True
    return True


# -- parentheses -------------------------------------------------------------------------------------------------

def sig(x):
    """A node's structure, parentheses and places aside (two parses are one when their sigs are equal)."""
    if isinstance(x, Node):
        if x.k == 'paren':
            return sig(x.e)
        return (x.k,) + tuple((a, sig(b)) for a, b in sorted(x.__dict__.items())
                              if a not in ('k', 'loc') and not a.startswith('_'))
    if isinstance(x, (list, tuple)):
        return tuple(sig(y) for y in x)
    return x


def parents(node):
    out = {}
    stack = [node]
    while stack:
        n = stack.pop()
        if isinstance(n, Node):
            for a, b in n.__dict__.items():
                if a == 'loc' or a.startswith('_'):
                    continue
                for c in _nodes_in(b):
                    out[id(c)] = n
                    stack.append(c)
    return out


def _nodes_in(b):
    if isinstance(b, Node):
        yield b
    elif isinstance(b, (list, tuple)):
        for x in b:
            for y in _nodes_in(x):
                yield y


def forced(x, par):
    """L §2.4: "a `... to u` call used as an operand and a converted flows or value inside a comparison (parse rule 4)
    are always parenthesised"."""
    if par is None:
        return False
    if x.k == 'to' and par.k in OPERATORS:
        return True
    if x.k == 'flows' and x.conv is not None and par.k in ('cmp', 'in'):
        return True
    if x.k == 'value' and par.k in ('cmp', 'in'):
        return True
    return False


def reparse(block, units):
    for l in block:
        for t in l.toks:
            t.role = None
            t.canon = None
    node, j = parse_section(block, 0, units)
    if j != len(block):
        raise LoadError('LOAD-PARSE', 'cannot parse: a section reprinted', block[0].loc)
    return node


def canon_block(block, units):
    """A section's parentheses: each pair the parse doesn't need goes (its removal leaves one parse), the forced ones
    stay or are added.  Returns the section's node, parsed from its final tokens."""
    for l in block:
        l.toks = list(l.toks)
    node = reparse(block, units)
    want = sig(node)
    par = parents(node)
    for pn in [x for x in walk(node) if x.k == 'paren']:
        if forced(pn.e, par.get(id(pn))):
            continue
        line = None
        for l in block:
            if any(t is pn._po for t in l.toks):
                line = l
                break
        if line is None:
            continue
        saved = line.toks
        line.toks = [t for t in saved if t is not pn._po and t is not pn._pc]
        try:
            ok = sig(reparse(block, units)) == want
        except TallyError:
            ok = False
        if not ok:
            line.toks = saved
    node = reparse(block, units)
    par = parents(node)
    adds = [x for x in walk(node) if forced(x, par.get(id(x))) and getattr(x, '_t0', None) is not None]
    if adds:
        for x in adds:
            for l in block:
                ids = [id(t) for t in l.toks]
                if id(x._t0) in ids and id(x._t1) in ids:
                    i, j = ids.index(id(x._t0)), ids.index(id(x._t1))
                    l.toks = l.toks[:i] + [Tok('OP', '(', '(', True, 0)] + l.toks[i:j + 1] + \
                        [Tok('OP', ')', ')', False, 0)] + l.toks[j + 1:]
                    break
        node = reparse(block, units)
    return node


# -- a file ------------------------------------------------------------------------------------------------------

def blocks(lines):
    out = []
    i = 0
    while i < len(lines):
        j = i + 1
        while j < len(lines) and lines[j].level > lines[i].level:
            j += 1
        out.append((i, j))
        i = j
    return out


def first_word(l):
    return l.toks[0].s if l.toks else ''


class FilePrinter:
    def __init__(self, lines, tree, units, table):
        self.ls = lines
        self.tree = tree
        self.units = units
        self.table = table

    def text(self):
        bl = blocks(self.ls)
        out = ['\n'.join(self.header(self.ls[bl[0][0]:bl[0][1]]))]
        secs = []
        for sec, (b0, b1) in zip(self.tree.sections, bl[1:]):
            block = self.ls[b0:b1]
            secs.append((canon_block(block, self.units), block))
        for node, block in self.order_sections(secs):
            out.append('\n'.join(self.section(node, block, 0)))
        return '\n\n'.join(out) + '\n'

    # -- the header (L §2.4 "Header first: model, then tally, on, start, end, use lines (by name), same lines (by left
    # side), notes")
    def header(self, hb):
        out = [line_text(hb[0].toks)]
        groups = {}
        for l in hb[1:]:
            groups.setdefault(first_word(l), []).append(l)
        if 'use' in groups:
            groups['use'] = stable_sort(groups['use'], lambda l: [l.toks[1].s])
        if 'same' in groups:
            def left(l):
                eq = [i for i, t in enumerate(l.toks) if t.s == '=']
                return [line_text(l.toks[1:eq[0] if eq else len(l.toks)])]
            groups['same'] = stable_sort(groups['same'], left)
        for w in HEADER_ORDER:
            for l in groups.get(w, ()):
                out.append('  ' + line_text(l.toks))
        return out

    # -- section order
    def order_sections(self, secs):
        kinds = [(n, b) for n, b in secs if n.k == 'kind']
        others = [(n, b) for n, b in secs if n.k != 'kind']

        def key(nb):
            n = nb[0]
            r = SECTION_ORDER.index(n.k)
            if n.k == 'version':
                return (r, first_day(n.known), n.id)
            return (r, None, n.id)
        others.sort(key=lambda nb: (key(nb)[0], key(nb)[1] or datetime.date.min, key(nb)[2]))
        ordered_kinds = kahn(kinds)
        r = SECTION_ORDER.index('kind')
        return [x for x in others if SECTION_ORDER.index(x[0].k) < r] + ordered_kinds + \
            [x for x in others if SECTION_ORDER.index(x[0].k) > r]

    # -- a section
    def section(self, n, block, lev):
        ind = '  ' * lev
        if n.k == 'def':
            return self.sec_def(n, block, lev)
        if n.k == 'version':
            return self.sec_version(n, block, lev)
        out = [ind + line_text(block[0].toks)]
        body = block[1:]
        rank = BODY_RANK.get(n.k)
        if rank is not None:
            body = stable_sort(body, lambda l: [rank.get(first_word(l), 99)])
        for l in body:
            out.append(ind + '  ' + line_text(l.toks))
        return out

    def sec_def(self, n, block, lev):
        ind = '  ' * lev
        head = block[0]
        if n.short:
            return [ind + line_text(head.toks)]
        body = block[1:]
        notes = [l for l in body if first_word(l) == 'note']
        rules = [l for l in body if first_word(l) == 'rule']
        if len(n.rules) == 1 and not notes:
            r = n.rules[0]
            if r.label == 'base' and r.v is None and r.when is None and r.governs is None and not r.beats:
                # L §2.4 Short form: "a def with one rule labelled base, no v, no when, no own governs, no notes prints
                # as def x: T = expr at E"
                at = getattr(n, '_at', None)
                ht = head.toks
                k = [i for i, t in enumerate(ht) if t is at]
                pre, post = (ht[:k[0]], ht[k[0]:]) if k else (ht, [])
                rt = rules[0].toks
                eq = [i for i, t in enumerate(rt) if t.s == '='][0]
                return [ind + line_text(pre + [Tok('OP', '=', '=', True, 0)] + rt[eq + 1:] + post)]
        out = [ind + line_text(head.toks)]
        for l in notes:
            out.append(ind + '  ' + line_text(l.toks))
        pairs = list(zip(n.rules, rules))
        for r, l in stable_sort(pairs, lambda rl: self.rule_key(rl[0], n.id, rl[1])):
            out.append(ind + '  ' + self.rule_text(l))
        return out

    def rule_key(self, r, d=None, l=None):
        # L §2.4 (round 19, GD-24): "rules (sorted by label, then by the known date of their v, a rule without v first,
        # then by their own governs as a version's rows are, below; a v that names no version is refused, LOAD-NAME,
        # §5.11)"
        if r.v is None:
            vk = (0,)
        else:
            kn = self.table.versions.get(r.v)
            if kn is None:
                raise LoadError('LOAD-NAME', 'rule %s.%s: v %s names no version' % (d or '?', r.label, r.v), r.loc)
            vk = (1, first_day(kn))
        # L §2.4 (GD-28, 2026-10-29): "Two rules, or two items, that every key above leaves tied ... sort by their own
        # canonical line, compared as ASCII"
        return [r.label, vk, self.span_key(r.governs, None), ascii_key(self.rule_text(l) if l is not None else '')]

    def rule_text(self, l):
        """A rule's line, its beats list sorted (L §2.4 "A beats list is sorted by label, with ', '")."""
        toks = l.toks
        if len(toks) > 2 and toks[2].s == 'beats':
            return line_text(toks[:3]) + ' ' + beats_sorted(toks, 3)
        return line_text(toks)

    def sec_version(self, n, block, lev):
        ind = '  ' * lev
        out = [ind + line_text(block[0].toks)]
        body = block[1:]
        clauses = ('cite', 'governs', 'known', 'enters', 'assumed', 'moves', 'covers', 'pad', 'anchor')
        heads = [l for l in body if first_word(l) in clauses]
        notes = [l for l in body if first_word(l) == 'note']
        items = [l for l in body if first_word(l) not in clauses and first_word(l) != 'note']
        for l in heads + notes:
            out.append(ind + '  ' + line_text(l.toks))
        pairs = list(zip(n.items, items))
        for it, l in stable_sort(pairs, lambda il: self.item_key(il[0], il[1])):
            out.append(ind + '  ' + self.item_text(it, l))
        return out

    def item_text(self, it, l):
        toks = l.toks
        if it.k == 'rulerep':
            k = [i for i, t in enumerate(toks) if t.s == 'beats' and i >= 5]
            if k:
                return line_text(toks[:k[0] + 1]) + ' ' + beats_sorted(toks, k[0] + 1)
        return line_text(toks)

    def item_key(self, it, l=None):
        # L §2.4: "Version items are sorted: entries into kinds and indexes first, then by target id, then keys in index
        # order (...), with a keyless row first, then by the start of their own governs span or their instant (a row
        # with no span of its own first, then a span with no start); a band's rows stay ascending"
        if it.k == 'enter':
            tgt = (it.qual + '.' if it.qual else '') + it.target
            return [0, tgt, (1, tuple(key_text(k) for k in it.keys))]
        if it.k == 'rulerep':
            tgt = (it.qual + '.' if it.qual else '') + it.d
            return [1, tgt, (0,), self.span_key(it.rule.governs, None), self.line_key(it, l)]
        if it.k == 'same':
            # L §2.4 (round 19, GD-24): "a same item by its left side (as an item whose target and keys are its left
            # side's)"; it has no span of its own. A row of the same target and keys with no span of its own ties with
            # it, and so do two same items of one left side: GD-28 sorts them by their own canonical line
            side = it.l
            base = side.obj if side.k == 'index' else side
            tgt = (base.id if base.k == 'name' else key_text(base)) if base is not None else ''
            if base is not None and base.k == 'field' and base.obj.k == 'name':
                tgt = base.obj.id + '.' + base.name
            keys = side.keys if side.k == 'index' else None
            comps = [1, tgt, (0,) if not keys else (1,)]
            if keys:
                binds = self.table.binds.get((None, tgt), [])
                for j, k in enumerate(keys):
                    comps.append(self.key_comp(k, binds[j].dom if j < len(binds) else None))
            comps.append(Tie(0, (), '', None))
            comps.append(self.line_key(it, l))
            return comps
        tgt = (it.qual + '.' if it.qual else '') + it.target
        if it.k == 'bandrow':
            keys = [it.key] if it.key is not None else None
        else:
            keys = it.keys
        kc = (0,) if not keys else (1,)
        comps = [1, tgt, kc]
        if keys:
            binds = self.table.binds.get((it.qual, it.target), [])
            for j, k in enumerate(keys):
                dom = binds[j].dom if j < len(binds) else None
                comps.append(self.key_comp(k, dom))
        if it.k == 'bandrow':
            return comps
        comps.append(self.span_key(it.governs, it.instant))
        comps.append(self.line_key(it, l))
        return comps

    def line_key(self, it, l):
        # L §2.4 (GD-28): the last key, the item's own canonical line compared as ASCII (UTF-8 bytes)
        return ascii_key(self.item_text(it, l) if l is not None else '')

    def key_comp(self, k, dom):
        # L §2.4 (round 19, GD-24): "At one key position the wider key comes first: `*` before every other key, and a
        # range before the keys and the ranges it holds (otherwise a range sorts at its first key); a place keyed by
        # things (konto[ana]) is one key, sorted as things are"; "a key its index doesn't hold is refused, LOAD-KEY"
        how, data = self.table.key_order(dom)
        if k.k == 'star':
            return ((-1,),)
        if how == 'member':
            return Open(('key', key_text(k)), 'member ' + data)
        if k.k == 'krange':
            return (self.key_at(k.a, how, data, dom), 0, Rev(self.key_at(k.b, how, data, dom)))
        if k.k == 'placekey':
            return ((2, key_text(k)), 1)
        return (self.key_at(k, how, data, dom), 1)

    def key_at(self, k, how, data, dom):
        if how == 'pos':
            p = data.get(key_text(k))
            if p is None:
                what = dom.id if dom is not None and dom.k == 'dname' else 'the list'
                raise LoadError('LOAD-KEY', '%s is not a key of %s' % (key_text(k), what), k.loc)
            return (0, p)
        if how == 'value':
            return key_value(k)
        return (2, key_text(k))

    def span_key(self, gov, instant):
        # L §2.4 (round 19, GD-24): "then by the start of their own governs span or their instant (a row with no span
        # of its own first, then a span with no start, a row governing always among them, then by start; a row read
        # after m after the dated ones of its target and keys)". Ties it leaves (two rows with no start, two with one
        # start, two after m rows) sort by their own canonical line (GD-28, the caller's last component)
        if gov is not None:
            txt = 'always' if gov == 'always' else repr((first_day(gov.a), first_day(getattr(gov, 'b', None))))
            if gov == 'always' or gov.a is None:
                return Tie(1, (), txt, None)
            return Tie(2, (first_day(gov.a),), txt, None)
        if instant is not None:
            if instant.after is not None:
                t = key_text(instant.after) if instant.after.k != 'name' else instant.after.id
                return Tie(3, (), t, None)
            # "by ... their instant": in time, a day's start before its end (L §4.6: "at D start (before the day's
            # movements), at D end (after them)"); `at D` alone is a plain reading's, never beside the two
            return Tie(2, (instant.day, {'start': 0, None: 1, 'end': 2}[instant.pos]), '', None)
        return Tie(0, (), '', None)

def beats_sorted(toks, i):
    """The labels of a beats list from toks[i], sorted, and the rest of the line."""
    labs = []
    j = i
    while j < len(toks):
        labs.append(toks[j].s)
        j += 1
        if j < len(toks) and toks[j].s == ',':
            j += 1
            continue
        break
    rest = toks[j:]
    return ', '.join(sorted(labs)) + (' ' + line_text(rest) if rest else '')


def kahn(kinds):
    """L §2.4 (round 19, GD-9): "`kind` sections print in tree order: each kind is followed by its subkinds (and
    theirs, depth first), kinds with the same parent (or no parent) sorted by id (ASCII)". A kind whose parent this
    file doesn't print (a prelude kind's, another file's) heads a tree of the file, among the kinds with no parent."""
    ids = set(n.id for n, b in kinds)
    by = dict((n.id, (n, b)) for n, b in kinds)
    children = {}
    roots = []
    for n, b in kinds:
        p = n.parent if n.spec == 'sub' else None
        if p is not None and p in ids and p != n.id:
            children.setdefault(p, []).append(n.id)
        else:
            roots.append(n.id)
    out = []
    done = set()
    stack = sorted(roots, reverse=True)
    while stack:
        x = stack.pop()
        if x in done:
            continue
        done.add(x)
        out.append(by[x])
        stack.extend(sorted(children.get(x, ()), reverse=True))
    for i in sorted(ids - done):
        out.append(by[i])        # a cycle of parents: refused at load (LOAD-CYCLE); printed by id
    return out
