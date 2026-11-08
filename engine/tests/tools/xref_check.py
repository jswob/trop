"""xref_check.py: the codes LANGUAGE §8 and ENGINE §11 define, against the codes and messages T's stored outputs
print (DONE gate 5, "counted at the freeze by tools/xref_check.py"; BUILD-ORDER § What T still writes, K8-13; issue
#56, milestone M0). Standard library only; it runs nothing.

The catalogue: every row of the tables of L §8.1 (holes), §8.3 (load), §8.4 (per value), §8.5 (warnings) and E §11,
each code once (a code defined twice fails). A code's message forms are the backticked spans of its message column
(§8.1's "Question printed", E §11's "What happens") that stand outside parentheses and hold a space or a `<…>`: the
parenthesised spans and the bare words are the prose's examples, not messages. A form matches a printed message when
its words appear in it in order, each `<…>` and each bare one-letter name (`x`, `k`, `d`; not the article `a`)
standing for any text; a form that joins two messages in one sentence (ALTERNATIVES: LOAD-LAWDATE's ask form) matches
either.

What is printed, and where: every stored output of checks.tsv (its `baseline`; a `not-run:` row has none), at the
milestone map/placement.tsv places the check in; and T's own vectors: kernel/vectors.refcore (S), codes/fixtures.tsv
and m0/load.refcore and codes/m0.refcore (M0), m1/vectors.refcore, codes/m1.refcore and codes/m1_invariant.refcore
(M1). In an output a refusal prints `refused: <CODE> <message>` (at
the start of a line, after an ask's or a scenario's or a member's name, or in a list), a warning
`warning: <CODE> <message>` (in a register, indented, with no `warning:`: L §8.2's dropped labels), and a hole its
questions, `?"<question>"`, with no code: a question is put to the
HOLE-* code whose fixed form it matches, and one no form matches is HOLE-STATED's or an input's own `question` (a
writer's text; counted apart). A code printed that the catalogue lacks fails (refcore's REFCORE-NOT-YET included:
INTERFACE.md §3).

Writes map/codes.tsv, one row per code (form `*`), one per message form, and one (form `?`) for a code some output
prints with a message no form of its matches, in the catalogue's order:

    code, section, built (the milestone that builds it: M0 for every LOAD-* code, else the first milestone whose
    BUILD-ORDER section names the code, else BUILT's, T's plan for M1 (issue #108), `-` if none), form, outputs (how
    many print it), first (the earliest milestone of one that does), example (the id of the first such output, by
    milestone then id)

    python3 -I -B engine/tests/tools/xref_check.py              -> writes map/codes.tsv, prints the counts
    python3 -I -B engine/tests/tools/xref_check.py --check      -> exit 1 if map/codes.tsv isn't what the sources give
    python3 -I -B engine/tests/tools/xref_check.py --gate M0    -> exit 1 while a LOAD-* code has no output at S or M0
    python3 -I -B engine/tests/tools/xref_check.py --gate M1    -> exit 1 while a code built by M1 has no output by M1

`--check` runs in run.py --self-test. `--gate M0` is M0's gate (MILESTONE-M0.md row 5): red until G4's fixtures land.
`--gate M1` is M1's (MILESTONE-M1.md row 3): every code built at S, M0 or M1 printed by an output at S, M0 or M1.
"""
import csv
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
T = os.path.dirname(HERE) + os.sep
ROOT = os.path.dirname(os.path.dirname(T.rstrip(os.sep))) + os.sep
sys.path.insert(0, T)
import run as R  # noqa: E402

SPEC = ROOT + 'engine/spec/'
OUT = T + 'map/codes.tsv'
COLS = ['code', 'section', 'built', 'form', 'outputs', 'first', 'example']
ORDER = ['S', 'M0', 'M1', 'M2', 'M3', 'M4', 'M5a', 'M5b', 'M5c', 'M6', 'M7', 'M8', 'M9']   # as map/placement.py
# The milestone that builds a code no BUILD-ORDER section names (T's plan for M1, section 2.2; issue #108): S for the
# codes S's outputs already print, M1 for those of M1's constructs, the later ones where their constructs are built.
BUILT = dict([(c, 'S') for c in ['HOLE-UNGOVERNED', 'HOLE-ANCHOR', 'HOLE-CALENDAR', 'HOLE-CHANGED', 'HOLE-PRERUN',
                                  'RUN-NONE', 'RUN-NOT-INT', 'RUN-KEY', 'RUN-NO-RULE', 'RUN-LIMIT', 'RUN-DIV',
                                  'RUN-DOMAIN', 'RUN-NEG', 'RUN-PLACE', 'RUN-LIFESPAN', 'RUN-FUTURE', 'RUN-RECORD',
                                  'WARN-NEGATIVE', 'WARN-UNREAD', 'WARN-LEGS', 'REQ-ASK']] +
             [(c, 'M1') for c in ['HOLE-STATED', 'HOLE-NOTYET', 'HOLE-PRICE', 'HOLE-IMPRECISE', 'RUN-CONFLICT',
                                   'RUN-EMPTY', 'RUN-ZERO', 'RUN-WEIGHTS', 'RUN-PRORATA', 'RUN-ORDER', 'RUN-SOLVE',
                                   'REQ-CLOCK', 'ENGINE-INVARIANT']] +
             [('HOLE-NOREADING', 'M2'), ('HOLE-SAME', 'M2'), ('WARN-STALE', 'M3'), ('HOLE-LOTS', 'M5a'),
              ('RUN-UNDECIDED', 'M6'), ('RUN-PARAM', 'M6'), ('REQ-PATHS', 'M7')])
CODE = r'[A-Z]+(?:-[A-Z]+)+'
REFUSED = re.compile(r'refused: (%s) (.*?)(?=, refused: |$)' % CODE, re.M)
WARNING = re.compile(r'(?:warning: |^ +)(WARN-[A-Z]+) (.*)$', re.M)
QUESTION = re.compile(r'\?"((?:[^"\\]|\\.)*)"')
# the tables (heading, then the column of a row that holds its message forms)
TABLES = [('LANGUAGE.md', '### 8.1 ', 'L §8.1', 2), ('LANGUAGE.md', '### 8.3 ', 'L §8.3', 1),
          ('LANGUAGE.md', '### 8.4 ', 'L §8.4', 1), ('LANGUAGE.md', '### 8.5 ', 'L §8.5', 1),
          ('ENGINE.md', '## 11. ', 'E §11', 2)]


def section_text(doc, heading):
    """The lines of `doc` from `heading` to the next heading of its level or above."""
    lines = open(SPEC + doc, encoding='utf-8').read().split('\n')
    start = next(i for i, l in enumerate(lines) if l.startswith(heading))
    level = heading.split(' ', 1)[0]
    end = next((i for i in range(start + 1, len(lines)) if re.match(r'#{1,%d} ' % len(level), lines[i])), len(lines))
    return lines[start:end]


def forms_of(cell):
    """The backticked spans of `cell` outside parentheses that hold a space or a <…>."""
    out, depth, i = [], 0, 0
    while i < len(cell):
        c = cell[i]
        if c == '`':
            j = cell.index('`', i + 1)
            span = cell[i + 1:j]
            if depth == 0 and (' ' in span or '<' in span):
                out.append(span)
            i = j + 1
            continue
        depth += {'(': 1, ')': -1}.get(c, 0)
        i += 1
    return out


def catalogue():
    """{code: (section, [forms])}, in the order the tables define them; exits on a code defined twice."""
    out = {}
    for doc, heading, name, col in TABLES:
        for line in section_text(doc, heading):
            cells = [c.strip() for c in line.strip().strip('|').split('|')]
            m = re.fullmatch(r'`(%s)`' % CODE, cells[0]) if line.startswith('| `') else None
            if not m:
                continue
            if m.group(1) in out:
                sys.exit('%s is defined twice: %s and %s' % (m.group(1), out[m.group(1)][0], name))
            out[m.group(1)] = (name, forms_of(cells[col]) if col < len(cells) else [])
    return out


# A catalogue form that joins two messages in one sentence, each printed alone: its alternatives, either of which
# matches (L §8.3 LOAD-LAWDATE: an ask that reaches an `at reader` def, or that reads a law-dated input directly)
ALTERNATIVES = {
    'ask <q> reaches an at reader def, or reads law-dated <x> directly, without law':
        ['ask <q> reaches an at reader def without law', 'ask <q> reads law-dated <x> directly, without law'],
}


def matches(form, message):
    """Whether a printed message is of the form: the form's pattern, or one of its alternatives'."""
    return any(pattern(f).search(message) for f in ALTERNATIVES.get(form, [form]))


def pattern(form):
    """A form as a regular expression: each <…>, each `…` (tally 2.1.6's LOAD-ORDER: `… read each other …`) and
    bare one-letter name (not `a`) any text, a bracket of names (`[k]`, `[keys]`, `[lo, hi]`) any bracket or none
    (an item with no keys prints none), a bracket holding a <…> (`[through <path>]`, tally 2.1.6's LOAD-SCENARIO)
    any text or none, and ± either sign."""
    parts = re.split(r'(\[[a-z, ]+\]|\[[^\]<]*<[^>]*>\]|<[^>]*>|…|\b[b-zA-Z]\b)', form)
    return re.compile(''.join(re.escape(p).replace('±', '[+-]') if k % 2 == 0 else
                              '.*?' if p.startswith('[') and '<' in p else
                              r'(?:\[.+?\])?' if p.startswith('[') else '.+?' for k, p in enumerate(parts)))


def built_at(cat):
    """{code: milestone}: M0 for LOAD-*, else the first BUILD-ORDER milestone section that names the code, else
    BUILT's."""
    sections, current = {}, None
    for line in open(SPEC + 'BUILD-ORDER.md', encoding='utf-8').read().split('\n'):
        m = re.match(r'#{2,3} (S|M\d[abc]?)\. ', line)
        if m:
            current = m.group(1)
        elif re.match(r'## ', line):
            current = None
        if current:
            sections[current] = sections.get(current, '') + line + '\n'
    out = {}
    for code in cat:
        named = [ms for ms in ORDER if ms in sections and re.search(r'`%s`' % re.escape(code), sections[ms])]
        out[code] = 'M0' if code.startswith('LOAD-') else (named[0] if named else BUILT.get(code, '-'))
    return out


def netstrings(b):
    out, at = [], 0
    while at < len(b):
        colon = b.index(b':', at)
        n = int(b[at:colon])
        out.append(b[colon + 1:colon + 1 + n].decode('utf-8'))
        at = colon + n + 2
    return out


def outputs():
    """[(id, milestone, output)]: every stored output and T's own vectors."""
    out = []
    placed = {r['id']: r['milestone'] for r in csv.DictReader(open(T + 'map/placement.tsv', encoding='utf-8'),
                                                                delimiter='\t', quoting=csv.QUOTE_NONE)}
    for c in R.load_checks():
        if c['note'].startswith('not-run:') or not c['baseline'] or not os.path.isfile(ROOT + c['baseline']):
            continue
        with open(ROOT + c['baseline'], 'rb') as f:
            out.append((c['id'], placed[c['id']], R.text(f.read())))
    for path, width, at, ms, prefix in [('kernel/vectors.refcore', 6, 5, 'S', 'kernel/'),
                                        ('m0/load.refcore', 7, 6, 'M0', 'm0/load/'),
                                        ('codes/m0.refcore', 7, 6, 'M0', 'codes/m0/'),
                                        ('m1/vectors.refcore', 6, 5, 'M1', 'm1/'),
                                        ('codes/m1.refcore', 7, 6, 'M1', 'codes/m1/'),
                                        ('codes/m1_invariant.refcore', 7, 6, 'M1', 'codes/m1/')]:
        ns = netstrings(open(T + path, 'rb').read())
        out += [(prefix + ns[k], ms, ns[k + at]) for k in range(0, len(ns), width)]
    for r in csv.DictReader(open(T + 'codes/fixtures.tsv', encoding='utf-8'), delimiter='\t', quoting=csv.QUOTE_NONE):
        out.append(('codes/' + r['id'], 'M0', r['out'].replace('\\n', '\n')))
    return out


def stated_ids():
    """The ids of codes/m1.refcore's HOLE-STATED fixtures: a question there that no HOLE-* form matches is the
    fixture's stated hole, HOLE-STATED's, not a writer's text counted apart."""
    ns = netstrings(open(T + 'codes/m1.refcore', 'rb').read())
    return {'codes/m1/' + ns[k] for k in range(0, len(ns), 7) if ns[k + 1] == 'HOLE-STATED'}


def printed(cat, outs, stated=frozenset()):
    """{(code, form or '*'): [(milestone, id)]}, the writers' questions counted apart, and the unknown codes."""
    hits, writers, unknown = {}, set(), set()
    holes = [(code, pattern(f)) for code, (_, forms) in cat.items() if code.startswith('HOLE-') for f in forms]
    for oid, ms, text in outs:
        seen = set()
        found = [(m.group(1), m.group(2)) for m in REFUSED.finditer(text)]
        found += [(m.group(1), m.group(2)) for m in WARNING.finditer(text)]
        for q in QUESTION.findall(text):
            codes = [code for code, p in holes if p.search(q)]
            found += [(code, q) for code in codes]
            if not codes and oid in stated:
                found.append(('HOLE-STATED', q))
            elif not codes:
                writers.add(oid)
        for code, message in found:
            if code not in cat:
                unknown.add('%s (%s)' % (code, oid))
                continue
            matched = {(code, f) for f in cat[code][1] if matches(f, message)}
            seen.update(matched or ({(code, '?')} if cat[code][1] else set()))
            seen.add((code, '*'))
        for key in seen:
            hits.setdefault(key, []).append((ORDER.index(ms), oid))
    return hits, writers, unknown


def table(cat, hits):
    built, rows = built_at(cat), []
    for code, (section, forms) in cat.items():
        for f in ['*'] + forms + (['?'] if (code, '?') in hits else []):
            h = sorted(hits.get((code, f), []))
            rows.append({'code': code, 'section': section, 'built': built[code], 'form': f, 'outputs': str(len(h)),
                         'first': ORDER[h[0][0]] if h else '-', 'example': h[0][1] if h else '-'})
    return rows


def tsv(rows):
    return '\t'.join(COLS) + '\n' + ''.join('\t'.join(r[c] for c in COLS) + '\n' for r in rows)


def unshown_load(rows):
    """The LOAD-* codes no output at S or M0 prints."""
    return [r['code'] for r in rows if r['form'] == '*' and r['code'].startswith('LOAD-') and
            (r['first'] == '-' or ORDER.index(r['first']) > ORDER.index('M0'))]


def unshown_load_forms(rows):
    """The LOAD-* message forms no output at S or M0 prints: [(code, form)]."""
    return [(r['code'], r['form']) for r in rows if r['code'].startswith('LOAD-') and r['form'] not in ('*', '?') and
            (r['first'] == '-' or ORDER.index(r['first']) > ORDER.index('M0'))]


def unshown_by(rows, ms):
    """The codes built at or before ms that no output at or before ms prints."""
    k = ORDER.index(ms)
    return [r['code'] for r in rows if r['form'] == '*' and r['built'] in ORDER and ORDER.index(r['built']) <= k and
            (r['first'] == '-' or ORDER.index(r['first']) > k)]


def unshown_forms_by(rows, ms):
    """The message forms of the codes built at or before ms that no output at or before ms prints: [(code, form)]."""
    k = ORDER.index(ms)
    return [(r['code'], r['form']) for r in rows if r['form'] not in ('*', '?') and r['built'] in ORDER and
            ORDER.index(r['built']) <= k and (r['first'] == '-' or ORDER.index(r['first']) > k)]


def selftest():
    """The rules above on small inputs, each beside a case it must refuse; the failures."""
    cat = {'LOAD-X': ('L §8.3', ['band <b> at <x>', 'a key entered twice']), 'HOLE-Y': ('L §8.1', ['x at k: no value'])}
    out = 'q: (refused: LOAD-X band t at 5 (f:3), refused: LOAD-X a key entered twice)\nz: ?"inc at 2026-01: no value"\n'
    two = 'refused: LOAD-ZZ no\n?"why?"\nq: refused: LOAD-X other words\n'
    hits, writers, unknown = printed(cat, [('one', 'M0', out), ('two', 'M1', two)])
    want = {('LOAD-X', '*'): [(1, 'one'), (2, 'two')], ('LOAD-X', 'band <b> at <x>'): [(1, 'one')],
            ('LOAD-X', 'a key entered twice'): [(1, 'one')], ('LOAD-X', '?'): [(2, 'two')],
            ('HOLE-Y', '*'): [(1, 'one')], ('HOLE-Y', 'x at k: no value'): [(1, 'one')]}
    return [why for ok, why in [
        (forms_of('`a <x> b` (`not <this>`), `word`; `c d`') == ['a <x> b', 'c d'], 'forms: outside parentheses'),
        (not pattern('a key entered twice').search('b key entered twice'), 'the article a is a word'),
        (all(matches(f, m) for f, m in [(list(ALTERNATIVES)[0], 'ask q reaches an at reader def without law'),
                                         (list(ALTERNATIVES)[0], 'ask q reads law-dated x directly, without law')])
         and not matches(list(ALTERNATIVES)[0], 'ask q reaches x without law'), 'a form\'s alternatives'),
        (pattern('<x>[k] reads itself').search('cyc reads itself') and pattern('on [lo, hi] now').search(
            'on [0, 2] now') and not pattern('<x>[k] reads itself').search('cyc[2] reads its'),
         'a bracket of names: any bracket or none'),
        (hits == want, 'two refusals in a list, a form each, a question by its form: %r' % hits),
        (writers == {'two'} and unknown == {'LOAD-ZZ (two)'}, 'a writer\'s question; a code not in the catalogue')]
        if not ok]


def main(argv):
    bad = selftest()
    if bad:
        sys.exit('xref_check.py fails its own rules: ' + '; '.join(bad))
    cat = catalogue()
    hits, writers, unknown = printed(cat, outputs(), stated_ids())
    if unknown:
        sys.exit('codes printed that the catalogue lacks: ' + ', '.join(sorted(unknown)))
    rows = table(cat, hits)
    data = tsv(rows)
    codes = [r for r in rows if r['form'] == '*']
    forms = [r for r in rows if r['form'] not in ('*', '?')]
    other = [r for r in rows if r['form'] == '?']
    by = {}
    for r in codes:
        by[r['section']] = by.get(r['section'], 0) + 1
    print('%d codes: %s; %d message forms, %d printed by some output' % (
        len(codes), ', '.join('%s %d' % kv for kv in by.items()), len(forms), sum(r['outputs'] != '0' for r in forms)))
    print('%d codes printed by no output; %d outputs print a question no HOLE-* form matches (a writer\'s text)' % (
        sum(r['outputs'] == '0' for r in codes), len(writers)))
    print('%d codes print a message no form of theirs matches: %s' % (
        len(other), ' '.join('%s (%s)' % (r['code'], r['example']) for r in other)))
    load = unshown_load(rows)
    print('M0: %d of %d LOAD-* codes have no output at S or M0: %s' % (
        len(load), sum(r['code'].startswith('LOAD-') for r in codes), ' '.join(load)))
    forms_left = unshown_load_forms(rows)
    print('M0: %d of %d LOAD-* message forms have no output at S or M0 (tier 2; codes/m0/held.tsv holds the fixtures held and why)' % (
        len(forms_left), sum(r['code'].startswith('LOAD-') for r in forms)))
    m1, m1_forms = unshown_by(rows, 'M1'), unshown_forms_by(rows, 'M1')
    print('M1: %d of %d codes built by M1 have no output by M1: %s' % (
        len(m1), sum(r['built'] in ORDER[:3] for r in codes), ' '.join(m1)))
    print('M1: %d message forms of codes built by M1 have no output by M1 (tier 2; codes/m1/held.tsv holds the '
          'fixtures held and why)' % len(m1_forms))
    if '--gate' in argv:
        ms = argv[argv.index('--gate') + 1]
        if ms not in ('M0', 'M1'):
            sys.exit('--gate knows M0 and M1 only')
        return 1 if (load if ms == 'M0' else m1) else 0
    if '--check' in argv:
        old = open(OUT, encoding='utf-8').read() if os.path.exists(OUT) else ''
        if old != data:
            print('map/codes.tsv is not what the catalogue and the stored outputs give: run tools/xref_check.py')
            return 1
        print('map/codes.tsv: %d codes, %d message forms, what the catalogue and the stored outputs give' % (
            len(codes), len(forms)))
        return 0
    with open(OUT, 'w', encoding='utf-8', newline='') as f:
        f.write(data)
    print('wrote map/codes.tsv: %d rows' % len(rows))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
