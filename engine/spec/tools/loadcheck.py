#!/usr/bin/env python3
"""loadcheck.py: the static load check of engine design D8 (brief W3).

A line scan of ids and keywords over Tally files (LANGUAGE.md §2.1, §2.2, §2.5,
§8.3). It parses no expression and computes no figure: it reads each line's
tokens, the bracket groups that hold binds, row keys and key lists, and the
words at the head of a line. It checks:

  LOAD-RESERVED  a keyword (§2.2) used as an id, a key, a record field or a
                 bound variable; rule labels may be keywords (§2.2, §2.5)
  LOAD-DUP       one namespace across all of a reality's files (§2.5); a
                 built-in id declared again (§2.2); a band with a built-in's
                 name; a key added twice; two rules with one label and version
  LOAD-SHADOW    a bound variable (a bind, an aggregate's or a `for`'s
                 variable, a band's argument) with the name of a top-level id,
                 built-in ids included
  LOAD-KEY       a declared key (index list, axis option, index addition, row
                 key) that is also a top-level id, as an ask named for a key
                 (UL-05); the keys of `place` are the place ids, so they are not
  LOAD-EXCEPTS   an `excepts` naming no rule of its def, or an excepts cycle
  LOAD-PARSE     what the scan itself needs: tabs, odd indentation, an
                 unterminated string, an id outside [a-z][a-z0-9_]* or longer
                 than 48, a key outside the key pattern, an unknown section

What it doesn't check: anything that needs an expression parsed or a name
resolved (LOAD-NAME, which index a key literal in an expression belongs to,
`ambiguous key`, `not a key of`, units, cycles over defs, bands' intervals).

Usage (from anywhere; paths are taken from tmp/design/):
  python3 -I tools/loadcheck.py                 the D8 run: every model, the
                                                household, and each change
  python3 -I tools/loadcheck.py PATH ...        a .tally file is a reality of
                                                its own, a directory is one
                                                reality of all its .tally
                                                files, a .md under
                                                amendments/changes/ is that
                                                change on the household
  -v                                            one line per reality
  --selftest                                    a planted reality: every
                                                refusal found, no look-alike
Exit status: 0 with no refusal, 1 otherwise.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOUSEHOLD = 'amendments/household'

KEYWORDS = set("""
model part scenario source version unit axis index place input band def fill draw pay bound ask
tally note if then else and or not in for where with as when excepts true false inf by at law
known governs assumed path check solve explain inputs-of residuals
""".split())

BUILTIN_NAMES = set("""
sum count min max any all first last argmin argmax share mean quantile prorata flows bal free
plan_low next_on worth covered round pow exp ln sqrt abs periodic factor dcf days add_days
add_months add_years start_of end_of year_of month_of day_of overlap_days pos age age_months
weekday next_business prev_business add_business band_lo band_hi marginal kind currency
missing real normal lognormal exponential gamma poisson binomial negbin uniform discrete
cdf pdf pmf crps u
""".split())

AGGREGATES = set('sum count min max any all first last argmin argmax share mean quantile'.split())

# §2.2 built-in ids; the outside places outside_<cur> and outside_<unit> are added per reality.
BUILTIN_IDS = {
    'opening': 'input', 'stated': 'input', 'short': 'input', 'account_place': 'input',
    'party_place': 'input', 'match_days': 'input', 'match_amount': 'input',
    'place': 'index', 'bank_account': 'index', 'bank_party': 'index', 'placekind': 'axis',
}
# The index of each key position of the built-in inputs (§3.6, §5.9, §5.18).
BUILTIN_KEYED = {
    'opening': ['place'], 'stated': ['place'], 'short': ['place'], 'match_days': ['place'],
    'match_amount': ['place'], 'account_place': ['bank_account'], 'party_place': ['bank_party'],
}
# The options of the built-in axis placekind (§5.9: kind(x)).
BUILTIN_KEYS = {'placekind': 'account cash holding asset debt claim outside'.split()}

KINDS = ('source', 'version', 'unit', 'axis', 'index', 'place', 'input', 'band', 'def',
         'fill', 'draw', 'pay', 'bound', 'ask')
HEADS = ('model', 'part', 'scenario')
KEYED_KINDS = ('input', 'def', 'band', 'place', 'fill', 'draw', 'pay')
VERSION_CLAUSES = ('cite', 'governs', 'known', 'enters', 'arrears', 'correction', 'assumed',
                   'moves', 'covers', 'note')

ID_RE = re.compile(r'[a-z][a-z0-9_]*\Z')
KEY_RE = re.compile(r'(?:[a-z0-9][a-z0-9_-]*|-?[0-9]+(?:\.[0-9]+)?)\Z')
TOKEN_RE = re.compile(r'[A-Za-z0-9_][A-Za-z0-9_.-]*|\S')
CUR_RE = re.compile(r'(?<![A-Za-z0-9_])[A-Z]{3}(?![A-Za-z0-9_])')


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


class Reality:
    def __init__(self, name):
        self.name = name
        self.files = {}          # display path -> [Line]
        self.refusals = []
        self.counts = {'ids': 0, 'keys': 0, 'row keys': 0, 'bound variables': 0, 'rule labels': 0}

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
    pairs = {'[': ']', '(': ')'}
    stack = [pairs[code[start]]]
    i = start + 1
    while i < len(code) and stack:
        c = code[i]
        if c in pairs:
            stack.append(pairs[c])
        elif c in ')]':
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
    """Split every file into its header and sections; the per-line checks of the scan itself."""
    sections = []
    heads = []
    for path, lines in reality.files.items():
        current = None
        first = True
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
            if indent == 0:
                toks = tokens(line.code)
                kind = toks[0]
                if first and kind in HEADS:
                    current = Section(kind, toks[1] if len(toks) > 1 else '', line)
                    heads.append(current)
                elif kind in KINDS:
                    ident = toks[1] if len(toks) > 1 else ''
                    current = Section(kind, ident, line)
                    sections.append(current)
                else:
                    reality.refuse('LOAD-PARSE', line, path, 'not a section: %s' % kind)
                    current = None
                first = False
            elif current is not None:
                current.body.append(line)
            else:
                reality.refuse('LOAD-PARSE', line, path, 'an indented line outside a section')
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
    """The index of each key position of a declaration: the binds right after its id."""
    m = re.match(r'[a-z]+ ' + re.escape(ident) + r'\[', code)
    if not m:
        return []
    inside, _ = bracket_group(code, m.end() - 1)
    if inside is None:
        return []
    out = []
    for bind in split_top(inside):
        t = tokens(bind)
        if len(t) >= 3 and t[1] == 'in' and ID_RE.match(t[2]):
            out.append(t[2])
        else:
            out.append(None)
    return out


def bound_variables(code):
    """(name, how) for each bound variable on a line: `ID in` after `[`, `,`, `for` or an aggregate's `(`."""
    t = tokens(code)
    out = []
    for i in range(1, len(t)):
        if t[i] != 'in' or not ID_RE.match(t[i - 1]):
            continue
        prev = t[i - 2] if i >= 2 else None
        if prev in ('[', ',', 'for'):
            out.append(t[i - 1])
        elif prev == '(' and i >= 3 and t[i - 3] in AGGREGATES:
            out.append(t[i - 1])
    return out


def record_fields(code, kind, ident):
    """The field names of a record type, `: (f type, ...)` or `: movement CUR (f type, ...)`, after the binds."""
    pos = len(kind) + 1 + len(ident)
    if code[pos:pos + 1] == '[':
        inside, pos = bracket_group(code, pos)
        if inside is None:
            return []
    if code[pos:pos + 2] != ': ':
        return []
    pos += 2
    m = re.match(r'movement [A-Z]{3} ', code[pos:])
    if m:
        pos += m.end()
    if code[pos:pos + 1] != '(':
        return []
    inside, _ = bracket_group(code, pos)
    return [tokens(f)[0] for f in split_top(inside or '') if tokens(f)]


def key_list(code, start):
    inside, _ = bracket_group(code, start)
    if inside is None or inside.strip() == '':
        return []
    return [k.strip() for k in split_top(inside)]


def check(reality):
    heads, sections = read_sections(reality)

    for h in heads:
        check_id(reality, h.line, h.kind, h.id, h.kind + ' id')

    # Top-level ids, one namespace (§2.5).
    namespace = {}
    for s in sections:
        item = '%s %s' % (s.kind, s.id)
        if not check_id(reality, s.line, item, s.id, s.kind + ' id'):
            continue
        reality.counts['ids'] += 1
        if s.id in namespace:
            first = namespace[s.id]
            reality.refuse('LOAD-DUP', s.line, item, '%s is declared twice (first: %s %s, %s)'
                           % (s.id, first.kind, s.id, first.line.loc))
        else:
            namespace[s.id] = s

    # Built-in ids: the fixed ones, and outside_<cur> for every currency used, outside_<unit> for every unit (§2.2, §5.9).
    currencies = set()
    for lines in reality.files.values():
        for line in lines:
            if line.code:
                currencies.update(CUR_RE.findall(line.code))
    builtin = dict(BUILTIN_IDS)
    for c in currencies:
        builtin['outside_' + c.lower()] = 'place'
    for s in sections:
        if s.kind == 'unit':
            builtin['outside_' + s.id] = 'place'
    for s in sections:
        if s.id in builtin:
            reality.refuse('LOAD-DUP', s.line, '%s %s' % (s.kind, s.id),
                           '%s is declared twice (a built-in %s, §2.2)' % (s.id, builtin[s.id]))
        if s.kind == 'band' and s.id in BUILTIN_NAMES:
            reality.refuse('LOAD-DUP', s.line, 'band ' + s.id, "band %s has a built-in's name" % s.id)

    top = set(namespace) | set(builtin)
    places = {s.id for s in sections if s.kind == 'place'}

    keyed = dict(BUILTIN_KEYED)
    for s in sections:
        if s.kind in KEYED_KINDS:
            keyed[s.id] = bind_indexes(s.line.code, s.id)

    # Bound variables, record fields and band arguments.
    for s in sections:
        item = '%s %s' % (s.kind, s.id)
        for line in [s.line] + s.body:
            for name in bound_variables(line.code):
                reality.counts['bound variables'] += 1
                if name in KEYWORDS:
                    reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (bound variable)' % name)
                elif name in top:
                    reality.refuse('LOAD-SHADOW', line, item,
                                   'bound variable %s has the name of a top-level id' % name)
        if s.kind == 'band':
            m = re.match(r'band ' + re.escape(s.id) + r'(\[[^\]]*\])?\(\s*([a-z][a-z0-9_]*)\s*:', s.line.code)
            if m:
                name = m.group(2)
                reality.counts['bound variables'] += 1
                if name in KEYWORDS:
                    reality.refuse('LOAD-RESERVED', s.line, item, '%s is a keyword (band argument)' % name)
                elif name in top:
                    reality.refuse('LOAD-SHADOW', s.line, item,
                                   'band argument %s has the name of a top-level id' % name)
        if s.kind in ('input', 'def'):
            for field in record_fields(s.line.code, s.kind, s.id):
                check_id(reality, s.line, item, field, 'record field')

    # Keys: index lists, axis options, index additions, row keys (§2.5, §5.3, §5.5, §5.6).
    declared = {}   # index or axis -> [(key, line)]

    def declare_key(owner, key, line, item, what):
        reality.counts['keys'] += 1
        if not KEY_RE.match(key):
            reality.refuse('LOAD-PARSE', line, item, '%s %s is not a key' % (what, key))
            return
        if key in KEYWORDS:
            reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (%s of %s)' % (key, what, owner))
            return
        if key in top:
            reality.refuse('LOAD-KEY', line, item, 'a key is also an id: %s (%s of %s)' % (key, what, owner))
        seen = declared.setdefault(owner, [])
        for k, l in seen:
            if k == key:
                reality.refuse('LOAD-DUP', line, item, 'a key added twice: %s of %s (first %s)' % (key, owner, l.loc))
                break
        seen.append((key, line))

    builtin_key_ids = {k: ax for ax, ks in BUILTIN_KEYS.items() for k in ks}
    for ident, s in namespace.items():
        if ident in builtin_key_ids:
            reality.refuse('LOAD-KEY', s.line, '%s %s' % (s.kind, ident),
                           'a key is also an id: %s (option of the built-in axis %s)' % (ident, builtin_key_ids[ident]))

    for s in sections:
        item = '%s %s' % (s.kind, s.id)
        if s.kind == 'index':
            code = s.line.code
            m = re.match(r'index ' + re.escape(s.id) + r'\s*=\s*\[', code)
            if m:
                for key in key_list(code, m.end() - 1):
                    declare_key(s.id, key, s.line, item, 'key')
        elif s.kind == 'axis':
            for line in s.body:
                t = tokens(line.code)
                if t:
                    declare_key(s.id, t[0], line, item, 'option')
        elif s.kind == 'version':
            for line in s.body:
                code = line.code
                t = tokens(code)
                if not t:
                    continue
                nxt = t[1] if len(t) > 1 else ''
                if t[0] in VERSION_CLAUSES and nxt not in ('[', '=', '+', ':'):
                    continue
                target = t[0]
                after = code[len(target):].lstrip()
                if after.startswith('+'):
                    k = after.find('[')
                    if k >= 0:
                        for key in key_list(after, k):
                            declare_key(target, key, line, item, 'added key')
                    continue
                if after.startswith('['):
                    positions = keyed.get(target)
                    for n, rk in enumerate(key_list(after, 0)):
                        index = positions[n] if positions and n < len(positions) else None
                        check_row_key(reality, line, item, target, index, rk, top, places)
        elif s.kind == 'def':
            check_rules(reality, s)
    return reality


def check_row_key(reality, line, item, target, index, rk, top, places):
    if rk == '*' or '[' in rk:
        return    # every key, or a keyed place as a key of place (§5.6)
    for key in (rk.split('..') if '..' in rk else [rk]):
        key = key.strip()
        if key == '':
            continue
        reality.counts['row keys'] += 1
        if not KEY_RE.match(key):
            reality.refuse('LOAD-PARSE', line, item, 'row key %s of %s is not a key' % (key, target))
        elif key in KEYWORDS:
            reality.refuse('LOAD-RESERVED', line, item, '%s is a keyword (row key of %s)' % (key, target))
        elif key in top and not (key in places and index in ('place', None)):
            reality.refuse('LOAD-KEY', line, item, 'a key is also an id: %s (row key of %s, index %s)'
                           % (key, target, index or 'unknown'))


def check_rules(reality, s):
    item = 'def ' + s.id
    rules = []    # (label, v, excepts, line)
    for line in s.body:
        t = tokens(line.code)
        if not t or t[0] != 'rule':
            continue
        if len(t) < 2 or not ID_RE.match(t[1]) or len(t[1]) > 48:
            reality.refuse('LOAD-PARSE', line, item, 'rule label %s is not a word of the id pattern' % (t[1] if len(t) > 1 else ''))
            continue
        label, i, excepts, v = t[1], 2, [], None
        if i < len(t) and t[i] == 'excepts':
            i += 1
            if i < len(t):
                excepts.append(t[i])
                i += 1
            while i + 1 < len(t) and t[i] == ',':
                excepts.append(t[i + 1])
                i += 2
        if i + 1 < len(t) and t[i] == 'v':
            v = t[i + 1]
        rules.append((label, v, excepts, line))
        reality.counts['rule labels'] += 1
    labels = {}
    for label, v, excepts, line in rules:
        if (label, v) in labels:
            reality.refuse('LOAD-DUP', line, item, 'two rules %s with version %s (first %s)'
                           % (label, v or '(none)', labels[(label, v)].loc))
        else:
            labels[(label, v)] = line
    names = {label for label, _, _, _ in rules}
    graph = {}
    for label, v, excepts, line in rules:
        for e in excepts:
            if e not in names:
                reality.refuse('LOAD-EXCEPTS', line, item, 'excepts names no rule %s' % e)
            graph.setdefault(label, set()).add(e)
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
            reality.refuse('LOAD-EXCEPTS', s.line, item, 'an excepts cycle through rule %s' % n)
            break


# ---------------------------------------------------------------- inputs

def rel(path):
    r = os.path.relpath(path, ROOT)
    return os.path.abspath(path) if r.startswith('..') else r


def file_lines(path, display=None):
    display = display or rel(path)
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
    r = Reality(rel(path))
    r.files[rel(path)] = file_lines(path)
    return r


def reality_of_dir(directory, name=None):
    r = Reality(name or rel(directory))
    for path in tally_files(directory):
        r.files[os.path.relpath(path, directory)] = file_lines(path)
    return r


def change_blocks(md_path):
    """The diff blocks of a change, and any other fenced block that is a whole section (an optional addition)."""
    with open(md_path, encoding='utf-8') as f:
        lines = f.read().split('\n')
    diffs, extras, i = [], [], 0
    while i < len(lines):
        m = re.match(r'( *)```(\w*)\s*$', lines[i])
        if not m:
            i += 1
            continue
        pad, lang = len(m.group(1)), m.group(2)
        j = i + 1
        block = []
        while j < len(lines) and not re.match(r' *```\s*$', lines[j]):
            block.append((j + 1, lines[j][pad:] if lines[j][:pad].strip() == '' else lines[j]))
            j += 1
        if lang == 'diff':
            diffs.append(block)
        elif block and all(t.split(' ', 1)[0] in KINDS for _, t in block if t and not t.startswith(' ')):
            extras.append(block)
        i = j + 1
    return diffs, extras


def reality_with_change(md_path, extras_too=False):
    """The household with one change applied: `-` lines removed where they stand, `+` lines that replace them put
    there, other `+` runs appended to their file (each must start a section, so where it falls doesn't matter)."""
    name = os.path.splitext(os.path.basename(md_path))[0]
    r = reality_of_dir(os.path.join(ROOT, HOUSEHOLD), 'change ' + name + (' + its optional addition' if extras_too else ''))
    md = rel(md_path)
    diffs, extras = change_blocks(md_path)
    stats = {'new': [], 'edited': [], 'added': 0, 'removed': 0, 'unplaced': []}
    for block in diffs:
        target, lines, pos = None, None, None
        run_start = True
        k = 0
        while k < len(block):
            n, t = block[k]
            if t.startswith('--- '):
                path = t[4:].split('   ')[0].strip()
                is_new = not (k + 1 < len(block) and block[k + 1][1].startswith('+++ '))
                if not is_new:
                    k += 1
                if path not in r.files:
                    if not is_new:
                        stats['unplaced'].append('%s:%d edits %s, which the household lacks' % (md, n, path))
                    r.files[path] = []
                    stats['new'].append(path)
                elif path not in stats['new'] and path not in stats['edited']:
                    stats['edited'].append(path)
                target, lines, pos, run_start = path, r.files[path], None, True
            elif target is None:
                pass
            elif t.startswith('-'):
                body = t[1:]
                idx = next((x for x, line in enumerate(lines) if line.text == body), None)
                if idx is None:
                    stats['unplaced'].append('%s:%d removes a line %s lacks' % (md, n, target))
                else:
                    del lines[idx]
                    pos = idx
                    stats['removed'] += 1
            elif t.startswith('+'):
                body = t[1:]
                line = Line('%s:%d (+%s)' % (md, n, target), body)
                stats['added'] += 1
                if pos is not None:
                    lines.insert(pos, line)
                    pos += 1
                else:
                    if run_start and body.strip() != '':
                        if body.startswith(' ') and target not in stats['new']:
                            stats['unplaced'].append('%s:%d adds an indented line after context' % (md, n))
                        run_start = False
                    lines.append(line)
            else:
                pos, run_start = None, True     # a context line or a blank between files
            k += 1
    if extras_too:
        for block in extras:
            lines = r.files.setdefault('asks/core.tally', [])
            for n, t in block:
                lines.append(Line('%s:%d (optional, +asks/core.tally)' % (md, n), t))
                stats['added'] += 1
    r.change = stats
    r.change_extras = len(extras)
    return r


def change_ids(r):
    """The top-level ids the change's own lines declare."""
    out = set()
    for lines in r.files.values():
        for line in lines:
            if '(+' in line.loc and line.text and not line.text.startswith(' '):
                t = tokens(strip_strings(line.text) or '')
                if len(t) > 1 and t[0] in KINDS:
                    out.add(t[1])
    return out


# ---------------------------------------------------------------- self-test

# A planted reality: every refusal the scan gives, and look-alikes it must let pass (rule labels that are keywords,
# a built-in name as an input's id, `where k in`, `solve x in`, a place as a key of place, a keyed place, `*`,
# a band row, a mode name as an axis option, words inside strings).
SELFTEST = {
    'a.tally': '''model t1
  tally 1
  on 2026-10-18

source src1 "a source with in, for and [k in x] inside a string"

axis mode_ax
  down "rounds down"
  key5 "an option named like an ask"

index person = [ana, ben, ana]

index lot = [if, src1]

index month = month 2026-01..2026-12

place konto: account PLN

place wallet[p in person]: cash PLN

input salary[p in person, m in month]: money PLN

input rec: (amount money PLN, then int)

input bal_in[x in place]: money PLN

input count[p in person]: int

band sum(n: int): money PLN

band zr(salary: int): money PLN

def total[p in person]: money PLN = sum(m in month: salary[p, m])

def shadow[month in person]: int = 1

def agg2: int = count(salary in person: 1) + sum(k in person where k in [ana], j in month: 1)

def lab[p in person]: money PLN
  rule first = 1
  rule none excepts first = 2
  rule first = 3
  rule x excepts nope = 4
  rule cyc1 excepts cyc2 = 5
  rule cyc2 excepts cyc1 = 6

def outside_pln: int = 1

def opening: int = 1

def debt: int = 1

ask key5: total[ana] for k in person, konto in [1]

ask q2: solve rec in [0, 1] to 0.01: lab[ana] = 0

ask q3: x for for in person

ask src1: total[ana]

version v1
  cite src1
  known 2026-01-01
  person + [cid]
  person + [ben]
  salary[ana, 2026-01..2026-03] = 1 "x"
  salary[q2, 2026-01] = 1 "id as row key"
  bal_in[konto] = 1 "a place as a key of place"
  bal_in[wallet[ana]] = 1 "a keyed place"
  opening[konto] = 0
  salary[*, 2026-04] = 2 "every key"
  zr: [0, 5) = 1 "band row"
''',
    'b.tally': '''part second
  tally 1

source src1 "declared again in another file"

version cid
  cite src1
  known 2026-01-02
''',
}
SELFTEST_PARSE = ('model t2\n  tally 1\n  on 2026-10-18\n\nsource s "unterminated\n\n'
                  'def a_very_long_identifier_that_goes_past_the_limit_of_48: int = 1\n\n'
                  'def b: int = 1\n   note "odd"\n\ndef c: int =\t1\n\nwidget w\n\nindex k = [Ab, 2025-03, t4e-171]\n')
SELFTEST_EXPECTED = sorted([
    ('LOAD-DUP', 'a.tally:59'), ('LOAD-DUP', 'b.tally:4'), ('LOAD-DUP', 'a.tally:29'), ('LOAD-DUP', 'a.tally:47'),
    ('LOAD-DUP', 'a.tally:49'), ('LOAD-RESERVED', 'a.tally:23'), ('LOAD-SHADOW', 'a.tally:31'),
    ('LOAD-SHADOW', 'a.tally:35'), ('LOAD-SHADOW', 'a.tally:37'), ('LOAD-SHADOW', 'a.tally:53'),
    ('LOAD-RESERVED', 'a.tally:57'), ('LOAD-KEY', 'a.tally:51'), ('LOAD-KEY', 'a.tally:9'), ('LOAD-DUP', 'a.tally:11'),
    ('LOAD-RESERVED', 'a.tally:13'), ('LOAD-KEY', 'a.tally:13'), ('LOAD-DUP', 'a.tally:42'),
    ('LOAD-EXCEPTS', 'a.tally:43'), ('LOAD-EXCEPTS', 'a.tally:39'), ('LOAD-KEY', 'a.tally:64'),
    ('LOAD-DUP', 'a.tally:65'), ('LOAD-KEY', 'a.tally:67'),
    ('LOAD-PARSE', 'c.tally:5'), ('LOAD-PARSE', 'c.tally:7'), ('LOAD-PARSE', 'c.tally:10'),
    ('LOAD-PARSE', 'c.tally:12'), ('LOAD-PARSE', 'c.tally:14'), ('LOAD-PARSE', 'c.tally:16'),
])


def selftest():
    found = []
    for files in (SELFTEST, {'c.tally': SELFTEST_PARSE}):
        r = Reality('selftest')
        for name, text in files.items():
            r.files[name] = [Line('%s:%d' % (name, n), t) for n, t in enumerate(text.split('\n'), 1)]
        found += [(code, loc) for code, loc, _, _ in check(r).refusals]
    found.sort()
    missed = [x for x in SELFTEST_EXPECTED if x not in found]
    extra = [x for x in found if x not in SELFTEST_EXPECTED]
    print('selftest: %d planted refusals, %d found, %d missed, %d extra%s' % (
        len(SELFTEST_EXPECTED), len(found) - len(extra), len(missed), len(extra),
        ''.join('\n  missed %s %s' % x for x in missed) + ''.join('\n  extra %s %s' % x for x in extra)))
    return 1 if missed or extra else 0


# ---------------------------------------------------------------- output

def fmt_counts(c):
    return ', '.join('%s %d' % (k, v) for k, v in c.items())


def run(groups, verbose):
    total = 0
    out = []
    for title, realities, note in groups:
        refusals = [x for r in realities for x in r.refusals]
        total += len(refusals)
        sums = {}
        for r in realities:
            for k, v in r.counts.items():
                sums[k] = sums.get(k, 0) + v
        files = sum(len(r.files) for r in realities)
        out.append('%s: %d %s, %d files: %d refusals (%s)%s' % (
            title, len(realities), 'reality' if len(realities) == 1 else 'realities', files,
            len(refusals), fmt_counts(sums), note))
        for r in realities:
            if verbose:
                out.append('  %s: %d files, %d refusals (%s)' % (r.name, len(r.files), len(r.refusals), fmt_counts(r.counts)))
            for code, loc, item, msg in r.refusals:
                out.append('  %s %s %s: %s' % (code, loc, item, msg))
    return out, total


def main(argv):
    if '--selftest' in argv:
        return selftest()
    verbose = '-v' in argv
    args = [a for a in argv if a != '-v']
    out = ['loadcheck (D8 W3): a line scan of ids and keywords; no expression is parsed and no figure computed.']
    groups = []
    if not args:
        models = [check(reality_of_file(p)) for p in sorted(
            os.path.join(ROOT, 'models', f) for f in os.listdir(os.path.join(ROOT, 'models')) if f.endswith('.tally'))]
        groups.append(('models/*.tally', models, ''))
        groups.append((HOUSEHOLD, [check(reality_of_dir(os.path.join(ROOT, HOUSEHOLD), HOUSEHOLD))], ''))
        cdir = os.path.join(ROOT, 'amendments', 'changes')
        added = {}
        for f in sorted(os.listdir(cdir)):
            if not f.endswith('.md'):
                continue
            r = check(reality_with_change(os.path.join(cdir, f)))
            st = r.change
            note = '; +%d -%d lines, new %s, edited %s; %s' % (
                st['added'], st['removed'], ', '.join(st['new']) or 'none', ', '.join(st['edited']) or 'none',
                'every line placed' if not st['unplaced'] else 'not placed: ' + '; '.join(st['unplaced']))
            groups.append(('change ' + f[:-3] + ' on the household', [r], note))
            for ident in change_ids(r):
                added.setdefault(ident, []).append(f[:-3])
            if r.change_extras:
                rx = check(reality_with_change(os.path.join(cdir, f), extras_too=True))
                groups.append(('change ' + f[:-3] + ' with its optional addition', [rx],
                               '; +%d lines (the fenced section outside the diff, put in asks/core.tally)' % rx.change['added']))
        lines, total = run(groups, verbose)
        out += lines
        both = sorted((i, c) for i, c in added.items() if len(c) > 1)
        out.append('ids declared by two changes: %s' % (', '.join('%s (%s)' % (i, ', '.join(c)) for i, c in both) or 'none'))
    else:
        realities = []
        for a in args:
            p = a if os.path.isabs(a) else os.path.join(ROOT, a)
            if not os.path.exists(p):
                p = os.path.abspath(a)
            if os.path.isdir(p):
                realities.append(check(reality_of_dir(p)))
            elif p.endswith('.md'):
                realities.append(check(reality_with_change(p)))
            else:
                realities.append(check(reality_of_file(p)))
        lines, total = run([('arguments', realities, '')], verbose)
        out += lines
    out.append('total: %d refusals' % total)
    print('\n'.join(out))
    return 1 if total else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
