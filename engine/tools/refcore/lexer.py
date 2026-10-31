"""Lexical rules (L §2.1).

A file is split into lines; each line has an indent level (two spaces per
level, tabs refused) and a list of tokens.  Tokens remember whether white
space came before them, since a touching `*` or `/` may belong to a unit
(L §2.1 Units).
"""
import re
import datetime
from fractions import Fraction

from .errors import LoadError


class Tok:
    __slots__ = ('t', 's', 'v', 'ws', 'col', 'nm', 'role', 'canon')

    def __init__(self, t, s, v=None, ws=False, col=0):
        self.t = t      # ID KEY INT DEC DAY MONTH MMDD CUR STR OP
        self.s = s      # source text
        self.v = v      # value
        self.ws = ws    # white space before it
        self.col = col
        self.nm = False  # the parser read it as an id (a name, a declaration's id), not as a word of the language
        self.role = None  # what the parser read it as, for the canonical reprint (canon.py): 'bin' a spaced
        #                   operator, 'u' a token inside a unit or a fraction (touching the one before)
        self.canon = None  # its canonical text, where it differs (a number, L §2.4)

    def __repr__(self):
        return '%s(%s)' % (self.t, self.s)


class Line:
    __slots__ = ('level', 'toks', 'lineno', 'file', 'text')

    def __init__(self, level, toks, lineno, file, text):
        self.level = level
        self.toks = toks
        self.lineno = lineno
        self.file = file
        self.text = text

    @property
    def loc(self):
        return '%s:%d' % (self.file, self.lineno)


GROUPED = re.compile(r'\d{1,3}(?: \d{3})+(?:\.\d+)?(?![0-9a-z_\-])')
WORDCH = set('abcdefghijklmnopqrstuvwxyz0123456789_-')
DIGITS = set('0123456789')
TWO = ('==', '!=', '<=', '>=', '..')
ONE = '<>+-*/()[]{},:=.?^%'


def _number(word, loc):
    """Classify a word that starts with a digit."""
    if re.fullmatch(r'\d+', word):
        return 'INT', int(word)
    m = re.fullmatch(r'(\d{4})-(\d{2})-(\d{2})', word)
    if m:
        try:
            return 'DAY', datetime.date(int(m.group(1)), int(m.group(2)), int(m.group(3)))
        except ValueError:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a day' % word, loc)
    m = re.fullmatch(r'(\d{4})-(\d{2})', word)
    if m:
        mo = int(m.group(2))
        if not 1 <= mo <= 12:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a month' % word, loc)
        return 'MONTH', (int(m.group(1)), mo)
    m = re.fullmatch(r'(\d{2})-(\d{2})', word)
    if m:
        return 'MMDD', (int(m.group(1)), int(m.group(2)))
    return 'KEY', word


def lex_line(text, loc):
    toks = []
    i = 0
    n = len(text)
    ws = False
    while i < n:
        c = text[i]
        if c == ' ':
            ws = True
            i += 1
            continue
        if c == '\t':
            raise LoadError('LOAD-PARSE', 'cannot parse: tab', loc)
        col = i
        if c == '"':
            j = i + 1
            buf = []
            while True:
                if j >= n:
                    raise LoadError('LOAD-PARSE', 'cannot parse: unterminated string', loc)
                d = text[j]
                if d == '\\':
                    if j + 1 < n and text[j + 1] in '"\\':
                        buf.append(text[j + 1])
                        j += 2
                        continue
                    raise LoadError('LOAD-PARSE', 'cannot parse: bad escape in string', loc)
                if d == '"':
                    break
                buf.append(d)
                j += 1
            toks.append(Tok('STR', text[i:j + 1], ''.join(buf), ws, col))
            i = j + 1
        elif "0" <= c <= "9":
            m = GROUPED.match(text, i)
            if m:
                s = m.group(0)
                raw = s.replace(' ', '')
                if '.' in raw:
                    toks.append(Tok('DEC', s, Fraction(raw), ws, col))
                else:
                    toks.append(Tok('INT', s, int(raw), ws, col))
                i = m.end()
            else:
                j = i
                while j < n and text[j] in WORDCH:
                    j += 1
                word = text[i:j]
                if re.fullmatch(r'\d{4}-', word) and j + 1 < n and text[j] == 'Q' and text[j + 1] in '1234' \
                        and not (j + 2 < n and text[j + 2] in WORDCH):
                    # L §2.1: "A quarter ... is 2026-Q1 to 2026-Q4, SDMX's form of a reporting quarter"
                    toks.append(Tok('QUARTER', text[i:j + 2], (int(word[:4]), int(text[j + 1])), ws, col))
                    i = j + 2
                    ws = False
                    continue
                if word.isdigit() and j + 1 < n and text[j] == '.' and text[j + 1] in DIGITS:
                    k = j + 1
                    while k < n and text[k] in DIGITS:
                        k += 1
                    if k < n and text[k] in WORDCH:
                        raise LoadError('LOAD-PARSE', 'cannot parse: %s' % text[i:k + 1], loc)
                    toks.append(Tok('DEC', text[i:k], Fraction(text[i:k]), ws, col))
                    i = k
                else:
                    t, v = _number(word, loc)
                    toks.append(Tok(t, word, v, ws, col))
                    i = j
        elif 'a' <= c <= 'z':
            j = i
            while j < n and text[j] in WORDCH:
                j += 1
            word = text[i:j]
            if '-' in word:
                toks.append(Tok('KEY', word, word, ws, col))
            else:
                if len(word) > 48:
                    raise LoadError('LOAD-PARSE', 'cannot parse: identifier %s is longer than 48 characters' % word, loc)
                toks.append(Tok('ID', word, word, ws, col))
            i = j
        elif 'A' <= c <= 'Z':
            j = i
            while j < n and 'A' <= text[j] <= 'Z':
                j += 1
            word = text[i:j]
            if len(word) != 3 or (j < n and text[j] in WORDCH):
                raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a currency code' % text[i:j + 1], loc)
            toks.append(Tok('CUR', word, word, ws, col))
            i = j
        else:
            two = text[i:i + 2]
            if two in TWO:
                toks.append(Tok('OP', two, two, ws, col))
                i += 2
            elif c in ONE:
                toks.append(Tok('OP', c, c, ws, col))
                i += 1
            else:
                raise LoadError('LOAD-PARSE', 'cannot parse: character %r' % c, loc)
        ws = False
    return toks


def split_lines(src, fname):
    """Split a file into Lines with indent levels (L §2.1)."""
    out = []
    if '\r' in src:
        raise LoadError('LOAD-PARSE', 'cannot parse: lines end with \\n only', fname)
    for no, raw in enumerate(src.split('\n'), 1):
        loc = '%s:%d' % (fname, no)
        if '\t' in raw:
            raise LoadError('LOAD-PARSE', 'cannot parse: tab', loc)
        if raw.strip() == '':
            continue
        sp = len(raw) - len(raw.lstrip(' '))
        if sp % 2:
            raise LoadError('LOAD-PARSE', 'cannot parse: indentation of %d spaces' % sp, loc)
        if raw.rstrip(' ') != raw:
            pass  # trailing spaces are spacing the grammar allows
        body = raw[sp:]
        # Polish text is allowed in strings only (L §2.1)
        toks = lex_line(body, loc)
        out.append(Line(sp // 2, toks, no, fname, body))
    return out
