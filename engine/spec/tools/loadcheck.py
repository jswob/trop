#!/usr/bin/env python3
"""loadcheck.py: the static load check, a line scan of ids and keywords, for Tally 2.

The tally 1 scan (engine/spec/tools/loadcheck.py) re-based on Tally 2's word
classes (tmp/design2/spec/LANGUAGE.md §2.2: keywords, built-in names,
contextual words, built-in ids) and §2.3's section kinds. It parses no
expression and computes no figure: it reads each line's tokens, the bracket
groups that hold binds, row keys and key lists, and the words at the head of a
line. It checks:

  LOAD-RESERVED  a keyword (§2.2) used as an id, a key, a record field (a field
                 of a record type or a record value other than the built-in
                 fields kind version source known governs assumed at, §2.3
                 FIELD; a leg's or an attr's name) or a bound variable; rule
                 labels may be keywords (§2.2)
  LOAD-DUP       one namespace of top-level ids across a reality's files; a
                 built-in id declared again
                 (§2.2); a band with a built-in name (§2.2); a key added twice
                 to one index or kind (a `replace` entry adds none); two rules
                 with one label and version
  LOAD-SHADOW    a bound variable (a bind, an aggregate's, a `for`'s or a
                 crps's or prorata's variable, a band's argument) with the name
                 of a top-level id, built-in ids included
  LOAD-KEY       a declared key (index list, axis option, `+ [..]` entry, row
                 key) that is also a top-level id; place ids may be row keys
  LOAD-LABEL     a `beats` naming no rule of its def
  LOAD-CYCLE     a `beats` cycle among a def's rules
  LOAD-PARSE     what the scan itself needs: tabs, odd indentation, an
                 unterminated string, an id outside [a-z][a-z0-9_]* or longer
                 than 48, a key outside the key pattern, an unknown section, an
                 indented line outside a section

What it doesn't check: anything that needs an expression parsed or a name
resolved (LOAD-NAME, which index or kind a key literal belongs to, units,
cycles over defs), and the syntax (tmp/design2/tools/parsecheck/ does that).

Usage:
  python3 -I loadcheck.py PATH ...    a .tally file is a reality of its own, a
                                      directory is one reality of all its .tally
                                      files; a file whose lines are all indented
                                      is a fragment of a section body, scanned as
                                      body lines
  -v                                  one line per reality
  --selftest                          a planted reality: every refusal found, no
                                      look-alike
Exit status: 0 with no refusal, 1 otherwise.
"""
import os
import re
import sys

# §2.2 keywords
KEYWORDS = set("""
model part scenario source version unit kind axis index place input band def given fill move
ask tally note if then else and or not in for where with as when true false inf by at
law known governs assumed none pending never check solve diff explain inputs-of readers-of
residuals
""".split())

# §2.2 built-in names
BUILTIN_NAMES = set("""
sum count min max any all first last argmin argmax share mean quantile prorata flows bal value
low next covered round pow exp ln sqrt abs periodic factor dcf accrue days_between add_days
add_months add_years start_of end_of year_of quarter_of month_of day_of overlap_days pos age
age_months weekday add_business first_business last_business is_business schedule calendar
window one_of latest earliest since band_lo band_hi marginal unit minor lifespan in_force meta
reasons gap observed missing real normal lognormal exponential gamma poisson binomial negbin
uniform discrete empirical cdf pdf pmf crps paths
""".split())

# §2.2 / §2.3 FIELD: the keywords a record's field may be (and a built-in field after ".")
FIELD_KEYWORDS = set('kind version source known governs assumed at'.split())

# §2.2 contextual words: free as ids; the scan uses the clause words at the head of a line
VERSION_CLAUSES = ('cite', 'governs', 'known', 'enters', 'assumed', 'moves', 'covers', 'pad', 'anchor', 'note')

# §2.2 built-in ids
BUILTIN_IDS = {}
for _w in 'opening stated account_place'.split():  # review W-4 removed party_place, match_days, match_amount
    BUILTIN_IDS[_w] = 'input'
for _w in 'days months years bank_account'.split():  # review W-4 removed quarters, bank_party
    BUILTIN_IDS[_w] = 'index'
BUILTIN_IDS['cut'] = 'value'
for _w in 'places accounts cash holdings assets debts claims outside currencies events transfers adjustments pads'.split():
    BUILTIN_IDS[_w] = 'kind'
# The index or kind of each key position of the built-in inputs.
BUILTIN_KEYED = {'opening': ['places'], 'stated': ['places'], 'account_place': ['bank_account']}

AGGREGATES = set('sum count min max any all first last argmin argmax'.split())

# §2.3 section kinds
KINDS = ('source', 'version', 'unit', 'kind', 'axis', 'index', 'place', 'input', 'band', 'def',
         'fill', 'move', 'ask')
HEADS = ('model', 'part', 'scenario')
KEYED_KINDS = ('input', 'def', 'band', 'place', 'fill', 'move')

ID_RE = re.compile(r'[a-z][a-z0-9_]*\Z')
KEY_RE = re.compile(r'(?:[a-z0-9][a-z0-9_-]*|-?[0-9]+(?:\.[0-9]+)?)\Z')
TOKEN_RE = re.compile(r'[A-Za-z0-9_][A-Za-z0-9_.-]*|\S')
CUR_RE = re.compile(r'[A-Z]{3}\Z')


class Line:
    __slots__ = ('loc', 'text', 'code', 'indent')

    def __init__(self, loc, text):
        self.loc = loc
        self.text = text
        self.code = None
        self.indent = 0


class Section:
    def __init__(self, kind, ident, line):
        self.kind = kind
        self.id = ident
        self.line = line
        self.body = []

    @property
    def item(self):
        return '%s %s' % (self.kind, self.id)


class Reality:
    def __init__(self, name):
        self.name = name
        self.files = {}          # display path -> [Line]
        self.refusals = []
        self.counts = {'ids': 0, 'keys': 0, 'row keys': 0, 'bound variables': 0, 'fields': 0, 'rule labels': 0}

    def refuse(self, code, line, item, message):
        self.refusals.append((code, line.loc, item, message))


def strip_strings(text):
    """The line with each "..." replaced by "" (escapes \\" and \\\\), or None if unterminated."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == '\\' else 1
            if j >= n:
                return None
            out.append('""')
            i = j + 1
        else:
            out.append(c)
            i += 1
    return ''.join(out)


def tokens(code):
    return TOKEN_RE.findall(code)


def bracket_group(code, start):
    """The text inside the bracket that opens at code[start], and the index after its close."""
    pairs = {'[': ']', '(': ')', '{': '}'}
    stack = [pairs[code[start]]]
    i = start + 1
    while i < len(code) and stack:
        c = code[i]
        if c in pairs:
            stack.append(pairs[c])
        elif c in ')]}':
            if c != stack[-1]:
                return None, i
            stack.pop()
        i += 1
    if stack:
        return None, i
    return code[start + 1:i - 1], i


def split_top(text):
    """Split at the commas outside brackets."""
    parts, depth, cur = [], 0, []
    for c in text:
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        if c == ',' and depth == 0:
            parts.append(''.join(cur).strip())
            cur = []
        else:
            cur.append(c)
    parts.append(''.join(cur).strip())
    return parts


def read_sections(reality):
    """Split every file into its header and sections; the scan's own line checks."""
    sections, heads = [], []
    for path, lines in reality.files.items():
        live = [l for l in lines if l.text.strip() != '']
        fragment = bool(live) and all(l.text[0] in ' \t' for l in live)
        current, item, first = None, None, True
        if fragment:
            current = Section('fragment', '', live[0])
            sections.append(current)
        for line in lines:
            text = line.text
            if text.strip() == '':
                continue
            if '\t' in text:
                reality.refuse('LOAD-PARSE', line, path, 'a tab')
                continue
            code = strip_strings(text)
            if code is None:
                reality.refuse('LOAD-PARSE', line, path, 'unterminated string')
                continue
            indent = len(code) - len(code.lstrip(' '))
            if indent % 2:
                reality.refuse('LOAD-PARSE', line, path, 'indentation is not a multiple of two spaces')
                continue
            line.code = code.strip()
            line.indent = indent // 2
            toks = tokens(line.code)
            if indent == 0:
                kind = toks[0]
                ident = toks[1] if len(toks) > 1 else ''
                item = None
                if first and kind in HEADS:
                    current = Section(kind, ident, line)
                    heads.append(current)
                elif kind in KINDS:
                    current = Section(kind, ident, line)
                    sections.append(current)
                else:
                    reality.refuse('LOAD-PARSE', line, path, 'not a section: %s' % kind)
                    current = None
                first = False
            elif current is None:
                reality.refuse('LOAD-PARSE', line, path, 'an indented line outside a section')
            else:
                current.body.append(line)
    return heads, sections


def check_id(reality, line, item, ident, what):
    if not ID_RE.match(ident) or len(ident) > 48:
        reality.refuse('LOAD-PARSE', line, item, '%s %s is not an id ([a-z][a-z0-9_]*, at most 48)' % (what, ident))
        return False
    if ident in KEYWORDS:
        reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (%s)' % (ident, what))
        return False
    return True


def bind_indexes(code, ident):
    """The index or kind of each key position of a declaration: the binds right after its id."""
    m = re.match(r'[a-z]+ ' + re.escape(ident) + r'\[', code)
    if not m:
        return []
    inside, _ = bracket_group(code, m.end() - 1)
    if inside is None:
        return []
    out = []
    for bind in split_top(inside):
        t = tokens(bind)
        out.append(t[2] if len(t) >= 3 and t[1] == 'in' and ID_RE.match(t[2]) else None)
    return out


def bound_variables(code):
    """Each bound variable on a line: `ID in` after `[`, `,`, `for` or an aggregate's `(`; not the ref of `solve … for`."""
    t = tokens(code)
    skip = None
    if 'solve' in t:   # `solve l == r for ref in [a, b]`: ref is what is solved for, not bound
        depth, sdepth = 0, None
        for i, w in enumerate(t):
            if w == 'solve' and sdepth is None:
                sdepth = depth
            elif w in '([{':
                depth += 1
            elif w in ')]}':
                depth -= 1
            elif w == 'for' and sdepth is not None and depth == sdepth:
                skip = i + 1
                break
    out = []
    for i in range(1, len(t)):
        if t[i] != 'in' or i - 1 == skip or not ID_RE.match(t[i - 1]):
            continue
        prev = t[i - 2] if i >= 2 else None
        if prev in ('[', ',', 'for'):
            out.append(t[i - 1])
        elif prev == '(' and i >= 3 and t[i - 3] in AGGREGATES:
            out.append(t[i - 1])
    return out


def type_fields(code, kind, ident):
    """The field names of a record type, `: (f type, ...)`, after the binds."""
    pos = len(kind) + 1 + len(ident)
    if code[pos:pos + 1] == '[':
        inside, pos = bracket_group(code, pos)
        if inside is None:
            return []
    if code[pos:pos + 2] != ': ':
        return []
    pos += 2
    if code[pos:pos + 1] != '(':
        return []
    inside, _ = bracket_group(code, pos)
    return [tokens(f)[0] for f in split_top(inside or '') if tokens(f)]


def value_fields(code):
    """(name, is a record value's field) for a record value's fields (`(f: …, …)`) and legs (`leg f …`)."""
    t = tokens(code)
    out = []
    stack = []
    for i, w in enumerate(t):
        if w in '([{':
            stack.append(w)
        elif w in ')]}':
            if stack:
                stack.pop()
        if i + 2 < len(t) and stack and stack[-1] == '(' and w in ('(', ',') and t[i + 2] == ':' \
                and re.match(r'[a-z]', t[i + 1] or ''):
            out.append((t[i + 1], True))
        if w == 'leg' and i + 1 < len(t):
            out.append((t[i + 1], False))
    return out


def key_list(code, start):
    inside, _ = bracket_group(code, start)
    if inside is None or inside.strip() == '':
        return []
    return [k.strip() for k in split_top(inside)]


def check(reality):
    heads, sections = read_sections(reality)

    for h in heads:
        check_id(reality, h.line, h.kind, h.id, h.kind + ' id')

    def declared(s):
        """Is this section's id a name it declares? (`unit CUR:` names a currency; a fragment has none.)"""
        return s.kind != 'fragment' and not (s.kind == 'unit' and CUR_RE.match(s.id))

    # Top-level ids, one namespace.
    namespace = {}
    for s in sections:
        if not declared(s):
            continue
        if not check_id(reality, s.line, s.item, s.id, s.kind + ' id'):
            continue
        reality.counts['ids'] += 1
        if s.id in namespace:
            first = namespace[s.id]
            reality.refuse('LOAD-DUP', s.line, s.item, '%s is declared twice (first: %s %s, %s)'
                           % (s.id, first.kind, s.id, first.line.loc))
        else:
            namespace[s.id] = s

    everything = list(sections)

    # Built-in ids declared again; bands with a built-in's name (§2.2).
    for s in everything:
        if s.id in BUILTIN_IDS and declared(s):
            reality.refuse('LOAD-DUP', s.line, s.item, '%s is declared twice (a built-in %s, §2.2)' % (s.id, BUILTIN_IDS[s.id]))
        if s.kind == 'band' and s.id in BUILTIN_NAMES:
            reality.refuse('LOAD-DUP', s.line, s.item, "band %s has a built-in's name" % s.id)

    top = set(namespace) | set(BUILTIN_IDS)
    places = {s.id for s in everything if s.kind == 'place'}

    keyed = dict(BUILTIN_KEYED)
    for s in everything:
        if s.kind in KEYED_KINDS:
            keyed[s.id] = bind_indexes(s.line.code, s.id)

    # Bound variables, record fields and band arguments.
    for s in everything:
        names = top
        for line in [s.line] + s.body:
            for name in bound_variables(line.code):
                reality.counts['bound variables'] += 1
                if name in KEYWORDS:
                    reality.refuse('LOAD-RESERVED', line, s.item, '%s is a keyword (bound variable)' % name)
                elif name in names:
                    reality.refuse('LOAD-SHADOW', line, s.item, 'bound variable %s has the name of a top-level id' % name)
            if not (line is s.line and s.kind == 'band'):
                for field, in_value in value_fields(line.code):
                    reality.counts['fields'] += 1
                    if not (in_value and field in FIELD_KEYWORDS):
                        check_id(reality, line, s.item, field, 'record field' if in_value else 'leg name')
            t = tokens(line.code)
            if line is not s.line and s.kind in ('kind', 'move') and len(t) > 1 and t[0] == 'attr':
                reality.counts['fields'] += 1
                check_id(reality, line, s.item, t[1], 'attr name')
        if s.kind == 'band':
            m = re.match(r'band ' + re.escape(s.id) + r'(\[[^\]]*\])?\(\s*([a-z][a-z0-9_-]*)\s*:', s.line.code)
            if m:
                name = m.group(2)
                reality.counts['bound variables'] += 1
                if name in KEYWORDS:
                    reality.refuse('LOAD-RESERVED', s.line, s.item, '%s is a keyword (band argument)' % name)
                elif name in names:
                    reality.refuse('LOAD-SHADOW', s.line, s.item, 'band argument %s has the name of a top-level id' % name)
        if s.kind in ('input', 'def', 'band'):
            for field in type_fields(s.line.code, s.kind, s.id):
                reality.counts['fields'] += 1
                if field not in FIELD_KEYWORDS:
                    check_id(reality, s.line, s.item, field, 'record field')

    # Keys: index lists, axis options, `+ [..]` entries, row keys.
    added = {}   # index or kind -> [(key, line)]

    def declare_key(owner, key, line, item, what, replace=False):
        reality.counts['keys'] += 1
        if not KEY_RE.match(key):
            reality.refuse('LOAD-PARSE', line, item, '%s %s is not a key' % (what, key))
            return
        if key in KEYWORDS:
            reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (%s of %s)' % (key, what, owner))
            return
        if key in top:
            reality.refuse('LOAD-KEY', line, item, 'a key is also an id: %s (%s of %s)' % (key, what, owner))
        if replace:
            return
        seen = added.setdefault(owner, [])
        for k, l in seen:
            if k == key:
                reality.refuse('LOAD-DUP', line, item, 'a key added twice: %s of %s (first %s)' % (key, owner, l.loc))
                break
        seen.append((key, line))

    for s in everything:
        if s.kind == 'index':
            m = re.match(r'index ' + re.escape(s.id) + r'\s*=\s*\[', s.line.code)
            if m:
                for key in key_list(s.line.code, m.end() - 1):
                    declare_key(s.id, key, s.line, s.item, 'key')
        elif s.kind == 'axis':
            for line in s.body:
                t = tokens(line.code)
                if t:
                    declare_key(s.id, t[0], line, s.item, 'option')
        elif s.kind in ('version', 'fragment'):
            for line in s.body if s.kind == 'version' else [s.line] + s.body:
                scan_vitem(reality, line, s.item, declare_key, keyed, top, places)
            if s.kind == 'fragment':
                check_rules(reality, s, [s.line] + s.body)
        elif s.kind == 'def':
            check_rules(reality, s, s.body)
    return reality


def scan_vitem(reality, line, item, declare_key, keyed, top, places):
    code = line.code
    t = tokens(code)
    if not t:
        return
    replace = False
    if t[0] in ('replace', 'given'):
        replace = t[0] == 'replace'
        code = code[len(t[0]):].lstrip()
        t = t[1:]
        if not t:
            return
    nxt = t[1] if len(t) > 1 else ''
    if t[0] in VERSION_CLAUSES + ('same',) and nxt not in ('[', '=', '+', ':'):
        return
    target = t[0]
    after = code[len(target):].lstrip()
    if after.startswith('+'):
        k = after.find('[')
        if k >= 0:
            for key in key_list(after, k):
                declare_key(target, key, line, item, 'added key', replace)
        return
    if after.startswith('['):
        positions = keyed.get(target)
        for n, rk in enumerate(key_list(after, 0)):
            index = positions[n] if positions and n < len(positions) else None
            check_row_key(reality, line, item, target, index, rk, top, places)


def check_row_key(reality, line, item, target, index, rk, top, places):
    if rk == '*' or '[' in rk:
        return    # every key, or a keyed place (parsecheck refuses that: §2.3 rk has no keyed place)
    for key in (rk.split('..') if '..' in rk else [rk]):
        key = key.strip()
        if key == '':
            continue
        reality.counts['row keys'] += 1
        if not KEY_RE.match(key):
            reality.refuse('LOAD-PARSE', line, item, 'row key %s of %s is not a key' % (key, target))
        elif key in KEYWORDS:
            reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (row key of %s)' % (key, target))
        elif key in top and key not in places:
            reality.refuse('LOAD-KEY', line, item, 'a key is also an id: %s (row key of %s, position %s)'
                           % (key, target, index or 'unknown'))


def check_rules(reality, s, body):
    item = s.item
    rules = []    # (label, v, beats, line)
    for line in body:
        t = tokens(line.code)
        if not t or t[0] != 'rule':
            continue
        if len(t) < 2 or not ID_RE.match(t[1]) or len(t[1]) > 48:
            reality.refuse('LOAD-PARSE', line, item, 'rule label %s is not a word of the id pattern' % (t[1] if len(t) > 1 else ''))
            continue
        label, i, beats, v = t[1], 2, [], None
        if i < len(t) and t[i] == 'beats':
            i += 1
            if i < len(t):
                beats.append(t[i])
                i += 1
            while i + 1 < len(t) and t[i] == ',':
                beats.append(t[i + 1])
                i += 2
        if i + 1 < len(t) and t[i] == 'v':
            v = t[i + 1]
        rules.append((label, v, beats, line))
        reality.counts['rule labels'] += 1
    labels = {}
    for label, v, beats, line in rules:
        if (label, v) in labels:
            reality.refuse('LOAD-DUP', line, item, 'two rules %s with version %s (first %s)'
                           % (label, v or '(none)', labels[(label, v)].loc))
        else:
            labels[(label, v)] = line
    names = {label for label, _, _, _ in rules}
    graph = {}
    for label, v, beats, line in rules:
        for b in beats:
            if b not in names:
                reality.refuse('LOAD-LABEL', line, item, 'beats names no rule %s of its def' % b)
            graph.setdefault(label, set()).add(b)
    state = {}

    def visit(n):
        state[n] = 1
        for m in graph.get(n, ()):
            if state.get(m) == 1 or (m not in state and visit(m)):
                return True
        state[n] = 2
        return False
    for n in sorted(graph):
        if n not in state and visit(n):
            reality.refuse('LOAD-CYCLE', s.line, item, 'a beats cycle through rule %s' % n)
            break


# ---------------------------------------------------------------- inputs

def file_lines(path, display):
    with open(path, encoding='utf-8') as f:
        text = f.read()
    return [Line('%s:%d' % (display, n), t) for n, t in enumerate(text.split('\n'), 1)]


def tally_files(directory):
    out = []
    for dirpath, dirnames, filenames in os.walk(directory):
        dirnames.sort()
        for name in sorted(filenames):
            if name.endswith('.tally'):
                out.append(os.path.join(dirpath, name))
    return sorted(out)


def reality_of_file(path):
    r = Reality(path)
    r.files[path] = file_lines(path, path)
    return r


def reality_of_dir(directory):
    r = Reality(directory)
    for path in tally_files(directory):
        rel = os.path.relpath(path, directory)
        r.files[rel] = file_lines(path, os.path.join(directory, rel))
    return r


# ---------------------------------------------------------------- self-test

# A planted reality: every refusal the scan gives, and look-alikes it must let pass (rule labels that are
# keywords, a built-in name as an input's id, contextual words as ids, `where k in`, `solve … for r in`, a
# place as a row key, a keyed place, `*`, a band row, a `replace` entry, words inside strings).
SELFTEST = {
    'a.tally': '''model t1
  tally 2
  on 2026-10-18
  start 2026-01-01
  end 2026-12-31

source src1 "a source with in, for and [k in x] inside a string"

axis mode_ax
  down "rounds down"
  key5 "an option named like an ask"

index person = [ana, ben, ana]

index lot = [if, src1]

kind wrapped

kind transfers: event certain

place konto: accounts PLN lives 2019-05-01..

input salary[p in person, m in months]: money PLN

input rec: (amount money PLN, then int, kind int)

input count[p in person]: int

band sum(n: int): money PLN

band zr(salary: int): money PLN

def total[p in person]: money PLN = sum(salary[p, m] for m in months)

def shadow[months in person]: int = 1

def agg2: int = count(salary in person) + sum(1 for k in person where k in [ana], j in years)

def lab[p in person]: money PLN
  rule first = 1
  rule none beats first = 2
  rule first = 3
  rule x beats nope = 4
  rule cyc1 beats cyc2 = 5
  rule cyc2 beats cyc1 = 6

def opening: int = 1

def start: int = 1

ask key5: total[ana] for k in person

ask q2: solve lab[ana] == 0 for rec in [0, 1] to 0.01

ask q3: x for for in person

ask src1: total[ana]

input rate: num

input rate: num
input owner[p in person]: num
def tax[s in person]: num = sum(rate for rate in person)
move levy[i in person]: transfers
  on d
  leg in from konto to konto amount 1
  note "the regime block of the tally 2 draft, unboxed at G2 (G2-1)"

version v1
  cite src1
  known 2026-01-01
  person + [cid]
  person + [ben]
  replace person + [ana] lives 2020-01-01..
  salary[ana, 2026-01..2026-03] = 1 "x"
  salary[q2, 2026-01] = 1 "id as row key"
  opening[konto] = 1 "a place as a row key"
  opening[konto[ana]] = 1 "a keyed place"
  salary[*, 2026-04] = 2 "every key"
  zr: [0, 5) = 1 "band row"
  bank[r1] = (on: 2026-01-01, kind: transfers, leg cash from konto to world amount 1 PLN) "line"
  bank[r2] = (if: 1) "a keyword no FIELD admits"
  owner[q2] = x "an input keyed by an ask's id, unboxed from a regime"
''',
    'b.tally': '''part second
  tally 2

source src1 "declared again in another file"

version cid
  cite src1
  known 2026-01-02
''',
}
SELFTEST_PARSE = ('model t2\n  tally 2\n  on 2026-10-18\n  start 2026-01-01\n  end 2026-12-31\n\nsource s "unterminated\n\n'
                  'def a_very_long_identifier_that_goes_past_the_limit_of_48: int = 1\n\n'
                  'def b: int = 1\n   note "odd"\n\ndef c: int =\t1\n\nwidget w\n\nindex k = [Ab, 2025-03, t4e-171]\n')
SELFTEST_EXPECTED = sorted([
    ('LOAD-KEY', 'a.tally:11'),        # axis option key5 is an ask's id
    ('LOAD-DUP', 'a.tally:13'),        # ana added twice to person
    ('LOAD-RESERVED', 'a.tally:15'),   # if as a key
    ('LOAD-KEY', 'a.tally:15'),        # src1 as a key
    ('LOAD-DUP', 'a.tally:19'),        # kind transfers: a built-in kind
    ('LOAD-RESERVED', 'a.tally:25'),   # record type field then
    ('LOAD-DUP', 'a.tally:29'),        # band sum: a built-in's name
    ('LOAD-SHADOW', 'a.tally:31'),     # band argument salary
    ('LOAD-SHADOW', 'a.tally:35'),     # bound variable months (a built-in index)
    ('LOAD-SHADOW', 'a.tally:37'),     # bound variable salary
    ('LOAD-DUP', 'a.tally:42'),        # two rules first
    ('LOAD-LABEL', 'a.tally:43'),      # beats nope
    ('LOAD-CYCLE', 'a.tally:39'),      # cyc1 / cyc2
    ('LOAD-DUP', 'a.tally:47'),        # def opening: a built-in input
    ('LOAD-RESERVED', 'a.tally:55'),   # bound variable for
    ('LOAD-DUP', 'a.tally:57'),        # ask src1: src1 is a source
    ('LOAD-DUP', 'a.tally:61'),        # input rate twice
    ('LOAD-SHADOW', 'a.tally:63'),     # bound variable rate: a top-level id
    ('LOAD-RESERVED', 'a.tally:66'),   # leg in
    ('LOAD-KEY', 'a.tally:72'),        # added key cid is a version's id (b.tally)
    ('LOAD-DUP', 'a.tally:73'),        # ben added twice to person
    ('LOAD-KEY', 'a.tally:76'),        # row key q2 is an ask's id
    ('LOAD-KEY', 'a.tally:83'),        # row key q2 of input owner
    ('LOAD-RESERVED', 'a.tally:82'),   # record field if (kind, a FIELD keyword, is admitted)
    ('LOAD-DUP', 'b.tally:4'),         # source src1 in another file
    ('LOAD-PARSE', 'c.tally:7'), ('LOAD-PARSE', 'c.tally:9'), ('LOAD-PARSE', 'c.tally:12'),
    ('LOAD-PARSE', 'c.tally:14'), ('LOAD-PARSE', 'c.tally:16'), ('LOAD-PARSE', 'c.tally:18'),
])


def selftest():
    found = []
    for files in (SELFTEST, {'c.tally': SELFTEST_PARSE}):
        r = Reality('selftest')
        for name, text in files.items():
            r.files[name] = [Line('%s:%d' % (name, n), t) for n, t in enumerate(text.split('\n'), 1)]
        found += [(code, loc) for code, loc, _, _ in check(r).refusals]
    found.sort()
    exp = list(SELFTEST_EXPECTED)
    missed, extra = [], list(found)
    for x in exp:
        if x in extra:
            extra.remove(x)
        else:
            missed.append(x)
    print('selftest: %d planted refusals, %d found, %d missed, %d extra%s' % (
        len(exp), len(exp) - len(missed), len(missed), len(extra),
        ''.join('\n  missed %s %s' % x for x in missed) + ''.join('\n  extra %s %s' % x for x in extra)))
    return 1 if missed or extra else 0


# ---------------------------------------------------------------- output

def fmt_counts(c):
    return ', '.join('%s %d' % (k, v) for k, v in c.items())


def main(argv):
    if '--selftest' in argv:
        return selftest()
    verbose = '-v' in argv
    args = [a for a in argv if not a.startswith('-')]
    if not args:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    out = ['loadcheck (Tally 2): a line scan of ids and keywords; no expression is parsed and no figure computed.']
    total = 0
    for a in args:
        r = check(reality_of_dir(a) if os.path.isdir(a) else reality_of_file(a))
        total += len(r.refusals)
        if verbose or r.refusals:
            out.append('%s: %d files, %d refusals (%s)' % (r.name, len(r.files), len(r.refusals), fmt_counts(r.counts)))
        for code, loc, item, msg in r.refusals:
            out.append('  %s %s %s: %s' % (code, loc, item, msg))
    out.append('total: %d refusals in %d realities' % (total, len(args)))
    print('\n'.join(out))
    return 1 if total else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
