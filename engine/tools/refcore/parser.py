"""The grammar of L §2.3, with the seven parse rules and the nesting limits.

parse_file(lines, units) -> Node('file', header=..., sections=[...])

`units` is the set of unit names a pre-scan found (declared `unit` ids and
keys of unit kinds); with the period words, it decides whether a word after
a number is its unit (L §2.1 Units).
"""
import re
import datetime
from fractions import Fraction

from .errors import LoadError, Unspecified
from .node import Node, walk

KEYWORDS = set('''model part scenario source version unit kind axis index place input band def given
fill move ask tally note if then else and or not in for where with as when true false inf by at
law known governs assumed none never check solve diff explain inputs-of readers-of
residuals'''.split())

# L §2.2 Built-in names (the review removed first_business, latest, earliest, in_force, meta, reasons, real,
# empirical and paths(); each is now an ordinary id, read as a band call before `(`)
BUILTINS = set('''sum count min max any all first last argmin argmax share mean quantile prorata flows
bal value low next covered round pow exp ln sqrt abs periodic factor dcf accrue days_between add_days
add_months add_years start_of end_of year_of quarter_of month_of day_of overlap_days pos age
age_months weekday add_business last_business is_business schedule calendar window one_of since
band_lo band_hi marginal unit minor lifespan gap observed missing normal lognormal exponential
gamma poisson binomial negbin uniform discrete cdf pdf pmf crps'''.split())

AGGS = set('sum count min max any all first last argmin argmax'.split())
QUERIES = set('mean share quantile'.split())
DISTS = set('normal lognormal exponential gamma uniform poisson binomial negbin'.split())
BUILTIN_FIELDS = set('kind version source known governs assumed at'.split())

PERIOD_WORDS = {'day': 'day', 'days': 'day', 'month': 'month', 'months': 'month',
                'quarter': 'quarter', 'quarters': 'quarter', 'year': 'year', 'years': 'year'}

CMPOPS = ('==', '!=', '<', '<=', '>', '>=')
TYPE_CLAUSE_WORDS = ('question', 'reading', 'prices', 'lives', 'priced')
MAX_DEPTH = 8
MAX_ASIF = 4



def canon_number(tok):
    """L §2.4: "Numbers print as the shortest exact decimal, with a leading 0 before the point (0.05) and thousands
    grouped by a space (96 000.00)" (contradiction 39: the example isn't the shortest).  None where the token keeps
    its text: an integer written with a leading zero (a key's spelling, L §2.1)."""
    raw = tok.s.replace(' ', '')
    if '.' in raw:
        ip, fp = raw.split('.', 1)
        fp = fp.rstrip('0')
        ip = ip.lstrip('0') or '0'
    else:
        if len(raw) > 1 and raw[0] == '0':
            return None
        ip, fp = raw, ''
    g = []
    while len(ip) > 3:
        g.insert(0, ip[-3:])
        ip = ip[:-3]
    g.insert(0, ip)
    out = ' '.join(g)
    return out + ('.' + fp if fp else '')


def _name_chain(e):
    """The ids of a name or a chain of fields over a name (a.b.c), or None."""
    out = []
    while e.k == 'field':
        out.append(e.name)
        e = e.obj
    if e.k != 'name':
        return None
    out.append(e.id)
    return list(reversed(out))

class P:
    """A parser over the tokens of one line."""

    def __init__(self, toks, loc, units):
        self.t = toks
        self.i = 0
        self.loc = loc
        self.units = units
        self.depth = 0
        self.askfor = False     # inside an ask line's `for`: a closing `in ID` names the world (parse rule 4)

    # -- token helpers -------------------------------------------------
    def peek(self, k=0):
        j = self.i + k
        return self.t[j] if j < len(self.t) else None

    def done(self):
        return self.i >= len(self.t)

    def src(self, i0):
        """The source text of the tokens from i0 to the current one, as written (for a register line that prints
        a pin as written, L §8.2)."""
        out = []
        for j in range(i0, min(self.i, len(self.t))):
            tok = self.t[j]
            if out and tok.ws:
                out.append(' ')
            out.append(tok.s)
        return ''.join(out)

    def next(self):
        tok = self.peek()
        if tok is None:
            self.err('more text')
        self.i += 1
        return tok

    def at(self, s, k=0):
        tok = self.peek(k)
        return tok is not None and tok.t in ('ID', 'KEY', 'OP') and tok.s == s

    def at_t(self, t, k=0):
        tok = self.peek(k)
        return tok is not None and tok.t == t

    def accept(self, s):
        if self.at(s):
            self.i += 1
            return True
        return False

    def expect(self, s):
        if not self.accept(s):
            self.err('"%s"' % s)

    def err(self, what):
        tok = self.peek()
        got = tok.s if tok is not None else 'end of line'
        raise LoadError('LOAD-PARSE', 'cannot parse: expected %s, got %s' % (what, got), self.loc)

    def end(self):
        if not self.done():
            self.err('end of line')

    def open(self):
        self.depth += 1
        if self.depth > MAX_DEPTH:
            raise LoadError('LOAD-DEPTH', 'expression nested deeper than 8', self.loc)

    def close(self):
        self.depth -= 1

    def ident(self, what='an id', allow_kw=False):
        tok = self.peek()
        if tok is None or tok.t != 'ID' or (not allow_kw and tok.s in KEYWORDS):
            if tok is not None and tok.t == 'ID' and tok.s in KEYWORDS:
                raise LoadError('LOAD-RESERVED', '%s is a keyword' % tok.s, self.loc)
            self.err(what)
        self.i += 1
        tok.nm = True
        return tok.s

    def label(self):
        tok = self.peek()
        if tok is None or tok.t != 'ID':
            self.err('a rule label')
        self.i += 1
        return tok.s

    def string(self):
        tok = self.peek()
        if tok is None or tok.t != 'STR':
            self.err('a string')
        self.i += 1
        return tok.v

    def day(self):
        tok = self.peek()
        if tok is None or tok.t != 'DAY':
            self.err('a day')
        self.i += 1
        return tok.v

    def integer(self):
        tok = self.peek()
        if tok is None or tok.t != 'INT':
            self.err('an integer')
        self.i += 1
        tok.canon = canon_number(tok)
        return tok.v

    def cur(self):
        tok = self.peek()
        if tok is None or tok.t != 'CUR':
            self.err('a currency code')
        self.i += 1
        return tok.s

    def N(self, k, **kw):
        return Node(k, self.loc, **kw)

    # -- units -----------------------------------------------------------
    def is_uterm_start(self, k=0, touching=False, syntactic=False):
        tok = self.peek(k)
        if tok is None:
            return False
        if touching and tok.ws:
            return False
        if tok.t == 'CUR':
            return True
        if tok.t == 'OP' and tok.s == '%':
            return True
        if tok.t == 'ID':
            if syntactic:
                return tok.s not in KEYWORDS
            return tok.s in PERIOD_WORDS or tok.s in self.units
        return False

    def uterm(self, syntactic=False):
        j0 = self.i
        tok = self.next()
        base = tok.s
        if tok.t == 'ID' and base in PERIOD_WORDS:
            base = PERIOD_WORDS[base]
        elif tok.t == 'ID':
            tok.nm = True        # a declared unit: a member's own is renamed under its prefix (joint.py, L §6.1)
        exp = 1
        if self.at('^') and not self.peek().ws:
            self.i += 1
            neg = False
            if self.at('-'):
                self.i += 1
                neg = True
            exp = self.integer() * (-1 if neg else 1)
        for j in range(j0 + 1, self.i):
            self.t[j].role = 'u'
        return (base, exp)

    def unit_after_number(self, syntactic=False):
        """A unit written after a number, or None (L §2.1 Units, §2.3 unit)."""
        terms = []
        if self.is_uterm_start(syntactic=syntactic):
            terms.append(self.uterm(syntactic))
        elif (self.at_t('INT') and self.peek().v == 1 and self.at('/', 1) and not self.peek(1).ws
              and self.is_uterm_start(2, touching=True, syntactic=True)):
            self.i += 2
            self.t[self.i - 1].role = 'u'
            self.t[self.i].role = 'u'
            b, e = self.uterm(True)
            terms.append((b, -e))
        else:
            return None
        while (self.peek() is not None and self.peek().t == 'OP' and self.peek().s in '*/'
               and not self.peek().ws and self.is_uterm_start(1, touching=True, syntactic=True)):
            optok = self.next()
            op = optok.s
            optok.role = 'u'
            self.peek().role = 'u'
            b, e = self.uterm(True)
            terms.append((b, e if op == '*' else -e))
        return tuple(terms)

    def type_unit(self):
        """A unit in a type: parsed by syntax only.  A type runs to its line's clause word (L §2.3 parse
        rule 3): `question`, `reading`, `prices`, `lives` and `priced` end it and are no unit."""
        terms = []
        tok = self.peek()
        if tok is not None and tok.t == 'ID' and tok.s in TYPE_CLAUSE_WORDS:
            return None
        if self.is_uterm_start(syntactic=True):
            terms.append(self.uterm(True))
        elif self.at_t('INT') and self.peek().v == 1 and self.at('/', 1):
            self.i += 2
            self.t[self.i - 1].role = 'u'
            self.t[self.i].role = 'u'
            b, e = self.uterm(True)
            terms.append((b, -e))
        else:
            return None
        while self.peek() is not None and self.peek().t == 'OP' and self.peek().s in '*/' \
                and not self.peek().ws and self.is_uterm_start(1, touching=True, syntactic=True):
            optok = self.next()
            op = optok.s
            optok.role = 'u'
            self.peek().role = 'u'
            b, e = self.uterm(True)
            terms.append((b, e if op == '*' else -e))
        return tuple(terms)

    # -- literals ----------------------------------------------------------
    def number(self, neg=False):
        tok = self.next()
        if tok.t == 'INT':
            v = Fraction(tok.v)
            isint = True
        elif tok.t == 'DEC':
            v = tok.v
            isint = False
        else:
            self.err('a number')
        if neg:
            v = -v
        unit = self.unit_after_number()
        four = tok.t == 'INT' and re.fullmatch(r'\d{4}', tok.s) is not None and unit is None
        if not four:
            tok.canon = canon_number(tok)
        return self.N('num', v=v, isint=isint, unit=unit, text=('-' if neg else '') + tok.s, four=four)

    def lit(self, signed=True):
        """lit / slit of the grammar: a number with an optional unit, a date, a key,
        a duration, true / false."""
        neg = False
        if signed and self.at('-'):
            nt = self.peek(1)
            if nt is not None and nt.t in ('INT', 'DEC'):
                self.i += 1
                neg = True
            elif nt is not None and nt.t == 'ID' and nt.s == 'inf':
                self.i += 2
                return self.N('inf', neg=True)
        tok = self.peek()
        if tok is None:
            self.err('a literal')
        if tok.t in ('INT', 'DEC'):
            # a fraction a/b, allowed only in rows (L §2.1)
            if (tok.t == 'INT' and self.at('/', 1) and self.at_t('INT', 2)
                    and not self.peek(1).ws and not self.peek(2).ws):
                a = tok.v
                self.i += 1
                self.t[self.i].role = 'u'
                self.i += 1
                btok = self.next()
                btok.role = 'u'
                b = btok.v
                if b == 0:
                    raise LoadError('LOAD-PARSE', 'cannot parse: fraction with denominator 0', self.loc)
                v = Fraction(a, b) * (-1 if neg else 1)
                # L §2.4: "a reduced a/b when the value doesn't terminate"
                tok.canon = '%d' % abs(v.numerator)
                btok.canon = '%d' % v.denominator
                unit = self.unit_after_number()
                return self.N('num', v=v, isint=False, unit=unit, text='%s%d/%d' % ('-' if neg else '', a, b),
                              frac=True, four=False)
            return self.number(neg)
        if neg:
            self.err('a number after -')
        if tok.t == 'DAY':
            self.i += 1
            return self.N('date', v=tok.v)
        if tok.t == 'MONTH':
            self.i += 1
            return self.N('month', v=tok.v)
        if tok.t == 'QUARTER':
            self.i += 1
            return self.N('quarter', v=tok.v)
        if tok.t == 'MMDD':
            self.i += 1
            return self.N('mmdd', v=tok.v, text=tok.s)
        if tok.t == 'KEY':
            self.i += 1
            return self.N('key', v=tok.s)
        if tok.t == 'CUR':
            self.i += 1
            return self.N('cur', v=tok.s)
        if tok.t == 'ID':
            if tok.s in ('true', 'false'):
                self.i += 1
                return self.N('bool', v=(tok.s == 'true'))
            if tok.s == 'inf':
                self.i += 1
                return self.N('inf', neg=False)
            if tok.s == 'nominal' and self.at_t('INT', 1):
                self.i += 2
                return self.N('code', v=('nominal', self.t[self.i - 1].v))
            if tok.s in KEYWORDS:
                self.err('a literal')
            self.i += 1
            tok.nm = True
            return self.N('name', id=tok.s)
        self.err('a literal')

    def lspan(self):
        """lspan = lit ".." [lit] | ".." lit | lit"""
        if self.at('..'):
            self.peek().role = 'lo'          # `governs ..2026-05`: spaced from the clause word (canon.py)
            self.i += 1
            b = self.lit()
            return self.N('lspan', a=None, b=b, dots=True)
        a = self.lit()
        if self.at('..'):
            dots = self.peek()
            self.i += 1
            b = None
            if not self.done() and not self._at_clause_after_span():
                b = self.lit()
            else:
                dots.role = 'hi'             # `governs 2026.. "art. 1"`: spaced from what follows
            return self.N('lspan', a=a, b=b, dots=True)
        return self.N('lspan', a=a, b=None, dots=False)

    def life_span(self):
        """An entry's lifespan: lspan | lit ".." hole (L §2.3 enter, round 18 C1-8: "an end not yet known a hole",
        `lives 1950-01-01..?"date of death"`, HOLE-STATED, L §5.7)."""
        j = self.i
        if not self.at('..'):
            a = self.lit()
            if self.at('..') and self.at('?', 1) and self.at_t('STR', 2):
                self.i += 2
                return self.N('lspan', a=a, b=None, dots=True, bhole=self.string())
            self.i = j
        return self.lspan()

    def _at_clause_after_span(self):
        tok = self.peek()
        if tok is None:
            return True
        if tok.t == 'STR':
            return True
        if tok.s in (',', '='):
            # `rule base governs 2028.. = …` (L §2.2 rulebody: ["governs" lspan] "=" …)
            return True
        if tok.t == 'ID' and tok.s in ('known', 'at', 'after', 'governs', 'priced', 'lives'):
            return True
        return False

    def known(self):
        if self.at('window') and self.at('(', 1):
            self.i += 2
            a = self.day()
            self.expect(',')
            b = self.day()
            self.expect(')')
            return self.N('window', a=a, b=b)
        return self.N('date', v=self.day())

    # -- expressions ---------------------------------------------------------
    def expr(self, no_in=False, no_to=False):
        if self.at('if'):
            self.i += 1
            self.open()
            c = self.expr()
            self.expect('then')
            a = self.expr(no_in, no_to)
            self.expect('else')
            b = self.expr(no_in, no_to)
            self.close()
            return self.N('if', c=c, a=a, b=b)
        return self.or_(no_in, no_to)

    def or_(self, no_in, no_to):
        l = self.and_(no_in, no_to)
        while self.accept('or'):
            r = self.and_(no_in, no_to)
            l = self.N('or', l=l, r=r)
        return l

    def and_(self, no_in, no_to):
        l = self.not_(no_in, no_to)
        while self.accept('and'):
            r = self.not_(no_in, no_to)
            l = self.N('and', l=l, r=r)
        return l

    def not_(self, no_in, no_to):
        if self.accept('not'):
            return self.N('not', e=self.cmp(no_in, no_to))
        return self.cmp(no_in, no_to)

    def cmp(self, no_in, no_to):
        l = self.sum(no_in, no_to)
        tok = self.peek()
        if tok is not None and tok.t == 'OP' and tok.s in CMPOPS:
            self.i += 1
            r = self.sum(no_in, no_to)
            self._no_chain()
            return self.N('cmp', op=tok.s, l=l, r=r)
        if self.at('in') and not no_in and not (self.askfor and self.depth == 0 and self.world_in()):
            self.i += 1
            r = self.sum(no_in, no_to)
            on = None
            if self.accept('on'):
                on = self.sum(no_in, no_to)
            self._no_chain()
            return self.N('in', l=l, r=r, on=on)
        return l

    def world_in(self):
        """`in ID` that ends an ask line, or comes before its later clauses: the world (L §2.3 parse rule 4: "On an ask
        line, in ID after the body and its for names the world")."""
        if not (self.at('in') and self.peek(1) is not None and self.peek(1).t == 'ID'):
            return False
        nxt = self.peek(2)
        return nxt is None or (nxt.t == 'ID' and nxt.s in ('with', 'law', 'as', 'windows', 'random', 'paths', 'path',
                                                            'partial', 'orient'))

    def _no_chain(self):
        tok = self.peek()
        if tok is not None and tok.t == 'OP' and tok.s in CMPOPS:
            raise LoadError('LOAD-PARSE', 'cannot parse: comparisons don\'t chain', self.loc)

    def sum(self, no_in, no_to):
        l = self.prod(no_in, no_to)
        while self.peek() is not None and self.peek().t == 'OP' and self.peek().s in ('+', '-'):
            optok = self.next()
            optok.role = 'bin'
            op = optok.s
            r = self.prod(no_in, no_to)
            l = self.N('bin', op=op, l=l, r=r)
        return l

    def prod(self, no_in, no_to):
        l = self.unary(no_in, no_to)
        while self.peek() is not None and self.peek().t == 'OP' and self.peek().s in ('*', '/'):
            optok = self.next()
            optok.role = 'bin'
            op = optok.s
            r = self.unary(no_in, no_to)
            l = self.N('bin', op=op, l=l, r=r)
        return l

    def unary(self, no_in, no_to):
        if self.at('-'):
            self.i += 1
            return self.N('neg', e=self.post(no_in, no_to))
        return self.post(no_in, no_to)

    def post(self, no_in=False, no_to=False):
        i0 = self.i
        e = self.atom()
        while True:
            if self.at('['):
                self.i += 1
                self.open()
                keys = [self.key_expr()]
                while self.accept(','):
                    keys.append(self.key_expr())
                self.expect(']')
                self.close()
                chain = _name_chain(e)
                if self.at('(') and chain is not None:
                    # a keyed band call b[k](x), with its prefixes R.b[k](x) (L §2.3 call, round 14)
                    if len(keys) != 1:
                        self.err('one key of a keyed band')
                    args = self.call_args()
                    e = self.N('call', name=chain[-1], args=args, bkey=keys[0], prefix=chain[:-1] or None)
                    continue
                e = self.N('index', obj=e, keys=keys)
            elif self.at('.') and not self.peek().ws:
                self.i += 1
                tok = self.peek()
                if tok is None or tok.t != 'ID':
                    self.err('a field')
                self.i += 1
                e = self.N('field', obj=e, name=tok.s)
                chain = _name_chain(e)
                if self.at('(') and chain is not None and not self.peek().ws:
                    # L §2.3 (round 14): call = (ID {"." ID} ["[" expr "]"] | BUILTIN) "(": a band named with the
                    # prefixes that name it outside its member or its base, a.b(x), household.b(x) (GD-1: no regime)
                    args = self.call_args()
                    e = self.N('call', name=chain[-1], args=args, bkey=None, prefix=chain[:-1])
            else:
                break
        if not no_to and self.at('to'):
            self.i += 1
            e = self.N('to', e=e, unit=self.step_unit())
            e._t0, e._t1 = self.t[i0], self.t[self.i - 1]
        if self.at('as') and self.at('if', 1):
            self.i += 2
            self.expect('{')
            self.open()
            pins = [self.pin()]
            while self.accept(','):
                pins.append(self.pin())
            self.expect('}')
            self.close()
            e = self.N('asif', e=e, pins=pins)
        return e

    def key_expr(self):
        if self.at('*'):
            self.i += 1
            return self.N('star')
        return self.expr()

    def step_unit(self):
        """UNIT = a positive lit (a step), "1e-" INT, "minor" "(" CUR ")", or a ref (L §7.2)."""
        tok = self.peek()
        if tok is None:
            self.err('a unit step')
        if tok.t == 'KEY' and re.fullmatch(r'1e-\d+', tok.s):
            self.i += 1
            return self.N('num', v=Fraction(1, 10 ** int(tok.s[3:])), isint=False, unit=None,
                          text=tok.s, four=False, step=True)
        if tok.t in ('INT', 'DEC'):
            return self.number()
        if self.at('minor') and self.at('(', 1):
            self.i += 2
            c = self.cur()
            self.expect(')')
            return self.N('call', name='minor', args=[self.N('cur', v=c)], bkey=None)
        return self.ref()

    def ref(self):
        """ref = ID ["[" expr {"," expr} "]"] {"." FIELD}"""
        name = self.ident('a reference')
        e = self.N('name', id=name)
        if self.at('['):
            self.i += 1
            self.open()
            keys = [self.key_expr()]
            while self.accept(','):
                keys.append(self.key_expr())
            self.expect(']')
            self.close()
            e = self.N('index', obj=e, keys=keys)
        while self.at('.') and not self.peek().ws:
            self.i += 1
            tok = self.peek()
            if tok is None or tok.t != 'ID':
                self.err('a field')
            self.i += 1
            e = self.N('field', obj=e, name=tok.s)
            if self.at('['):
                # a.x[k]: a member's item read with keys (L §2.3 ref)
                self.i += 1
                self.open()
                keys = [self.key_expr()]
                while self.accept(','):
                    keys.append(self.key_expr())
                self.expect(']')
                self.close()
                e = self.N('index', obj=e, keys=keys)
        return e

    def call_args(self, span_at=()):
        self.expect('(')
        self.open()
        args = []
        if not self.at(')'):
            pos = 0
            while True:
                if pos in span_at:
                    args.append(self.span())
                else:
                    args.append(self.expr())
                pos += 1
                if not self.accept(','):
                    break
        self.expect(')')
        self.close()
        return args

    def span(self):
        """span = expr ".." [expr] | ".." [expr] | expr"""
        if self.at('..'):
            self.peek().role = 'bin'
            self.i += 1
            b = None
            if not self._span_end():
                b = self.expr()
            return self.N('span', a=None, b=b, dots=True)
        a = self.expr()
        if self.at('..'):
            self.peek().role = 'bin'
        if self.accept('..'):
            b = None
            if not self._span_end():
                b = self.expr()
            return self.N('span', a=a, b=b, dots=True)
        return self.N('span', a=a, b=None, dots=False)

    def _span_end(self):
        tok = self.peek()
        if tok is None:
            return True
        if tok.t == 'OP' and tok.s in (',', ')', ']', '}'):
            return True
        if tok.t == 'ID' and tok.s in ('where', 'by', 'for', 'law', 'with', 'as', 'windows', 'random',
                                       'paths', 'path', 'partial', 'orient', 'in', 'from'):
            return True
        return False

    def is_bind_start(self):
        tok = self.peek()
        return (tok is not None and tok.t == 'ID' and tok.s not in KEYWORDS and self.at('in', 1))

    def bind(self):
        i0 = self.i
        n = self.bind_()
        n._src = self.src(i0)
        return n

    def bind_(self):
        var = self.ident('a bound variable')
        self.expect('in')
        if self.at('['):
            # a key list, unless it is the start of a range
            save = self.i
            self.i += 1
            self.open()
            keys = []
            if not self.at(']'):
                keys.append(self.lit())
                while self.accept(','):
                    keys.append(self.lit())
            self.expect(']')
            self.close()
            dom = self.N('dlist', keys=keys)
        else:
            a = self.expr(no_in=True)
            if self.at('..'):
                self.peek().role = 'bin'
            if self.accept('..'):
                b = self.expr(no_in=True)
                step = None
                if self.accept('step'):
                    step = self.expr(no_in=True)
                dom = self.N('drange', a=a, b=b, step=step)
            else:
                if a.k == 'name':
                    dom = self.N('dname', id=a.id, qual=None)
                elif a.k == 'field' and a.obj.k == 'name':
                    dom = self.N('dname', id=a.name, qual=a.obj.id)
                else:
                    self.err('an index, a kind, a key list or a range')
        on = over = None
        if self.accept('on'):
            on = self.expr()
        elif self.accept('over'):
            over = self.span()
        where = None
        if self.accept('where'):
            where = self.expr()
        return self.N('bind', var=var, dom=dom, on=on, over=over, where=where)

    def binds(self):
        out = [self.bind()]
        while self.at(',') and self.peek(1) is not None and self.peek(1).t == 'ID' and self.at('in', 2):
            self.i += 1
            out.append(self.bind())
        return out

    def tieby(self):
        self.expect('ties')
        self.expect('by')
        if self.at('('):
            save = self.i
            self.i += 1
            self.open()
            first = self.expr()
            if self.accept(','):
                items = [first, self.expr()]
                while self.accept(','):
                    items.append(self.expr())
                self.expect(')')
                self.close()
                return items
            self.close()
            self.i = save
        return [self.post()]

    def proj(self):
        out = {}
        if self.accept('from'):
            out['frm'] = self.post()
        if self.at('windows'):
            self.i += 1
            out['windows'] = self.wchoice()
        if self.at('random') and self.at('at', 1):
            self.i += 2
            out['random'] = self.rchoice()
        if self.at('with') and self.at('uncertain', 1):
            self.i += 2
            out['uncertain'] = True
        return out

    def wchoice(self):
        # L §2.3 wchoice = "each" (the review removed latest and earliest)
        tok = self.next()
        if tok.s != 'each':
            self.i -= 1
            self.err('each')
        return tok.s

    def rchoice(self):
        if self.accept('mean'):
            return ('mean', None)
        if self.at('quantile'):
            self.i += 1
            self.expect('(')
            self.open()
            q = self.expr()
            self.expect(')')
            self.close()
            return ('quantile', q)
        self.err('mean or quantile(q)')

    def mode(self):
        tok = self.peek()
        if tok is not None and tok.t == 'KEY':
            self.i += 1
            return self.N('key', v=tok.s)
        return self.ref()

    def atom(self):
        tok = self.peek()
        if tok is None:
            self.err('an expression')
        t, s = tok.t, tok.s
        if t in ('INT', 'DEC', 'DAY', 'MONTH', 'QUARTER', 'MMDD', 'KEY', 'CUR'):
            return self.lit(signed=False)
        if t == 'STR':
            self.i += 1
            return self.N('str', v=tok.v)
        if t == 'OP':
            if s == '(':
                self.i += 1
                self.open()
                if self.peek() is not None and self.peek().t == 'ID' and self.at(':', 1):
                    fields = [self.field()]
                    while self.accept(','):
                        fields.append(self.field())
                    self.expect(')')
                    self.close()
                    return self.N('record', fields=fields)
                po = self.t[self.i - 1]
                e = self.expr()
                self.expect(')')
                self.close()
                n = self.N('paren', e=e)
                n._po, n._pc = po, self.t[self.i - 1]
                return n
            if s == '[':
                self.i += 1
                self.open()
                items = []
                if not self.at(']'):
                    items.append(self.lit())
                    while self.accept(','):
                        items.append(self.lit())
                self.expect(']')
                self.close()
                return self.N('list', items=items)
            if s == '?':
                raise LoadError('LOAD-PARSE', 'cannot parse: a hole stands only as a whole value', self.loc)
            self.err('an expression')
        # an ID
        if s == 'none':
            self.i += 1
            return self.N('none')
        if s == 'never':
            self.i += 1
            return self.N('never')
        if s == 'cut' and not self.at('(', 1):
            self.i += 1
            return self.N('cut')
        if s in ('true', 'false', 'inf'):
            return self.lit(signed=False)
        if s == 'nominal' and self.at_t('INT', 1):
            return self.lit(signed=False)
        if self.at('(', 1) and (s in BUILTINS):
            return self.builtin(s)
        if s in KEYWORDS:
            if s in ('if',):
                self.err('a parenthesised if')
            raise LoadError('LOAD-RESERVED', '%s is a keyword' % s, self.loc) if s not in (
                'and', 'or', 'then', 'else', 'for', 'where', 'when', 'at', 'with', 'as', 'by', 'in', 'not',
                'law', 'known', 'governs', 'assumed') else self.err('an expression')
        self.i += 1
        self.t[self.i - 1].nm = True
        if self.at('('):
            # a band call
            args = self.call_args()
            return self.N('call', name=s, args=args, bkey=None)
        return self.N('name', id=s)

    def field(self):
        tok = self.peek()
        if tok is None or tok.t != 'ID' or not (self.at(':', 1) or tok.s == 'leg'):
            # L §3.1, §8.3: a positional record is refused (LOAD-FIELD)
            raise LoadError('LOAD-FIELD', 'positional record; name its fields', self.loc)
        if tok.s == 'leg' and not self.at(':', 1):
            return ('leg', self.leg_body())
        self.i += 1
        name = tok.s
        self.expect(':')
        return (name, self.field_value())

    def field_value(self):
        if self.at('?') and self.at_t('STR', 1):
            self.i += 1
            return self.N('hole', text=self.string())
        if self.at('none'):
            self.i += 1
            return self.N('none')
        return self.expr()

    def leg_body(self):
        self.expect('leg')
        name = self.ident('a leg name')
        self.expect('from')
        a = self.expr(no_to=True)
        self.expect('to')
        b = self.expr()
        self.expect('amount')
        q = self.expr()
        on = None
        if self.accept('on'):
            on = self.expr()
        return self.N('leg', name=name, frm=a, to=b, amount=q, on=on)

    def builtin(self, s):
        i0 = self.i
        if s in AGGS:
            return self.agg(s)
        if s in QUERIES:
            return self.query(s)
        if s == 'bal':
            self.i += 1
            self.expect('(')
            self.open()
            r = self.ref()
            args = []
            start = False
            if self.accept(','):
                args.append(self.expr())
                if self.accept(','):
                    args.append(self.expr())
                if self.accept('start'):
                    start = True
            self.expect(')')
            self.close()
            modelled = False
            if self.at('modelled'):
                self.i += 1
                modelled = True
                proj = {}
            elif self.at('from'):
                proj = self.proj()
            else:
                # L §2.3 parse rule 7: a windows or random clause belongs to a projection read (bal(...) from t,
                # low(...)); after a plain bal(x, d) it is the ask line's. A plain balance read may state with
                # uncertain, the read's own clause (L §5.14: "a plain balance read that states it, bal(x, d) with
                # uncertain")
                proj = {}
                if self.at('with') and self.at('uncertain', 1):
                    self.i += 2
                    proj['uncertain'] = True
            return self.N('bal', ref=r, args=args, start=start, modelled=modelled, proj=proj)
        if s == 'low':
            self.i += 1
            self.expect('(')
            self.open()
            r = self.ref()
            self.expect(',')
            sp = self.span()
            self.expect(')')
            self.close()
            return self.N('low', ref=r, span=sp, proj=self.proj())
        if s == 'value':
            self.i += 1
            self.expect('(')
            self.open()
            r = self.ref()
            self.expect(',')
            d = self.expr()
            self.expect(')')
            self.close()
            self.expect('in')
            c = self.cur()
            by = None
            bylast = False
            if self.accept('by'):
                if self.at('last') and self.at('(', 1):
                    self.i += 2
                    by = self.ref()
                    self.expect(')')
                    bylast = True
                else:
                    by = self.ref()
            n = self.N('value', ref=r, d=d, cur=c, by=by, bylast=bylast)
            n._t0, n._t1 = self.t[i0], self.t[self.i - 1]
            return n
        if s == 'flows':
            self.i += 1
            self.expect('(')
            self.open()
            r = self.ref()
            self.expect(',')
            sp = self.span()
            fk = None
            if self.accept(','):
                self.expect('for')
                fk = self.expr()
            self.expect(')')
            self.close()
            conv = None
            if self.at('in') and self.at_t('CUR', 1):
                self.i += 1
                c = self.cur()
                self.expect('at')
                fx = self.ref()
                m = self.mode()
                conv = (c, fx, m)
            n = self.N('flows', ref=r, span=sp, forkey=fk, conv=conv)
            n._t0, n._t1 = self.t[i0], self.t[self.i - 1]
            return n
        if s == 'crps':
            self.i += 1
            self.expect('(')
            self.open()
            e = self.expr()
            weight = None
            bind = None
            if self.accept('weight'):
                weight = self.expr()
                self.expect('for')
                bind = self.bind()
            elif self.accept('for'):
                bind = self.bind()
            self.expect(',')
            y = self.expr()
            self.expect(')')
            self.close()
            return self.N('crps', e=e, weight=weight, bind=bind, y=y)
        if s in ('cdf', 'pdf', 'pmf'):
            self.i += 1
            args = self.call_args()
            if len(args) != 2:
                self.err('two arguments')
            return self.N('distfn', fn=s, args=args)
        if s in DISTS:
            self.i += 1
            args = self.call_args()
            return self.N('dist', fn=s, args=args, term=None, bind=None)
        if s == 'discrete':
            self.i += 1
            self.expect('(')
            self.open()
            e = self.expr()
            self.expect('for')
            b = self.bind()
            self.expect(')')
            self.close()
            return self.N('dist', fn=s, args=[], term=e, bind=b)
        if s == 'prorata':
            return self.prorata()
        if s == 'covered':
            self.i += 1
            args = self.call_args(span_at=(1,))
            return self.N('call', name='covered', args=args, bkey=None)
        if s == 'calendar':
            return self.calval()
        # a plain built-in call
        self.i += 1
        i0 = self.i
        args = self.call_args()
        if s in ('periodic', 'factor'):
            self.period_word_arg(i0, 1 if s == 'periodic' else 2)
        return self.N('call', name=s, args=args, bkey=None)

    def period_word_arg(self, i0, pos):
        """L §2.2: period words are "the period argument of periodic and factor (day, month, quarter, year, §3.3)",
        and contextual words are reserved only in their own position: the argument at pos, a lone word there, is a
        word of the language, not an id (so a member's index named `month` doesn't take it, L §6.1)."""
        depth, k, start = 0, 0, None
        for j in range(i0, self.i):
            t = self.t[j]
            if t.t == 'OP' and t.s in ('(', '['):
                depth += 1
                if depth == 1 and t.s == '(' and j == i0:
                    start = j + 1
                continue
            if t.t == 'OP' and t.s in (')', ']'):
                if depth == 1 and k == pos and start is not None and j - start == 1:
                    break
                depth -= 1
                continue
            if depth == 1 and t.t == 'OP' and t.s == ',':
                if k == pos:
                    break
                k += 1
                start = j + 1
        else:
            return
        if k == pos and start is not None and j - start == 1:
            t = self.t[start]
            if t.t == 'ID' and t.s in ('day', 'month', 'quarter', 'year'):
                t.nm = False

    def price(self):
        """price = ref | "last" "(" ref ")"  (L §2.3: a reading's carry, §4.6, §5.10)"""
        if self.at('last') and self.at('(', 1):
            self.i += 2
            r = self.ref()
            self.expect(')')
            return self.N('price', ref=r, last=True)
        return self.N('price', ref=self.ref(), last=False)

    def agg(self, fn):
        self.i += 1
        self.expect('(')
        self.open()
        if self.is_bind_start():
            term = None
            binds = self.binds()
        else:
            first = self.expr()
            if self.accept('for'):
                term = first
                binds = self.binds()
            elif fn in ('min', 'max') and self.at(','):
                args = [first]
                while self.accept(','):
                    args.append(self.expr())
                self.expect(')')
                self.close()
                return self.N('call', name=fn, args=args, bkey=None)
            elif fn == 'last' and self.at(','):
                self.i += 1
                d = self.expr()
                self.expect(')')
                self.close()
                return self.N('lastx', ref=first, d=d)
            else:
                self.err('"for" or a binding')
        self.expect(')')
        self.close()
        tie = None
        if self.at('ties') and self.at('by', 1):
            tie = self.tieby()
        return self.N('agg', fn=fn, term=term, binds=binds, tie=tie)

    def query(self, fn):
        self.i += 1
        self.expect('(')
        self.open()
        e = self.expr()
        binds = where = None
        if self.accept('for'):
            binds = self.binds()
        elif self.accept('where'):
            where = self.expr()
        q = None
        if fn == 'quantile':
            self.expect(',')
            q = self.expr()
        self.expect(')')
        self.close()
        return self.N('query', fn=fn, arg=e, binds=binds, where=where, q=q)

    def prorata(self):
        self.i += 1
        self.expect('(')
        self.open()
        total = self.expr()
        self.expect(',')
        w = self.expr()
        self.expect('for')
        binds = self.binds()
        self.expect(',')
        k = self.expr()
        self.expect(',')
        u = self.step_unit()
        self.expect(',')
        mode = rest = tie = None
        if self.at('rest'):
            self.i += 1
            self.expect('largest')
            rest = 'largest'
            if self.at('ties'):
                tie = self.tieby()
        else:
            mode = self.mode()
            self.expect(',')
            self.expect('rest')
            if self.accept('none'):
                rest = 'none'
            else:
                self.expect('last')
                rest = 'last'
        self.expect(')')
        self.close()
        return self.N('prorata', total=total, w=w, binds=binds, key=k, u=u, mode=mode, rest=rest, tie=tie)

    def calval(self):
        """calval = "calendar" "(" "weekends" "[" ID {"," ID} "]" ["," "yearly" "[" MMDD ... "]" ["from" INT]]
        ["," "added" "[" cday ... "]"] ")"  (L §2.3, §4.2: the review removed `removed`)"""
        self.expect('calendar')
        self.expect('(')
        self.open()
        self.expect('weekends')
        self.expect('[')
        wk = [self.ident('a weekday')]
        while self.accept(','):
            wk.append(self.ident('a weekday'))
        self.expect(']')
        yearly = []
        yfrom = None
        added = []
        if self.at(',') and self.at('yearly', 1):
            self.i += 2
            self.expect('[')
            if not self.at(']'):
                yearly.append(self._mmdd())
                while self.accept(','):
                    yearly.append(self._mmdd())
            self.expect(']')
            if self.accept('from'):
                yfrom = self.integer()
        if self.at(',') and self.at('added', 1):
            self.i += 2
            self.expect('[')
            if not self.at(']'):
                added.append(self.cday())
                while self.accept(','):
                    added.append(self.cday())
            self.expect(']')
        self.expect(')')
        self.close()
        return self.N('calval', weekends=wk, yearly=yearly, yfrom=yfrom, added=added)

    def _mmdd(self):
        tok = self.peek()
        if tok is None or tok.t != 'MMDD':
            self.err('a month-day (MM-DD)')
        from .values import monthday_ok
        if not monthday_ok(*tok.v):
            # L §3.1 monthday (GD-2): "02-29 is no month-day, since not every year has it (refused, LOAD-PARSE)"; a
            # calendar's yearly entries are month-days (L §4.2)
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a month-day every year has' % tok.s, self.loc)
        self.i += 1
        return tok.v

    def cday(self):
        tok = self.peek()
        if tok is not None and tok.t == 'DAY':
            self.i += 1
            return self.N('date', v=tok.v)
        if tok is not None and tok.t == 'MMDD':
            # L §2.3 cday = DAY | hole | dayset; L §4.2 "a month-day ... stands only in a calendar's yearly list and an
            # anchored index's from"
            raise LoadError('LOAD-PARSE', 'cannot parse: a month-day %s in added; added takes days' % tok.s, self.loc)
        if self.at('?') and self.at_t('STR', 1):
            self.i += 1
            return self.N('hole', text=self.string())
        return self.dayset()

    def dayset(self):
        if self.at('window') and self.at('(', 1):
            self.i += 2
            a = self.day()
            self.expect(',')
            b = self.day()
            self.expect(')')
            return self.N('window', a=a, b=b)
        if self.at('one_of') and self.at('(', 1):
            self.i += 2
            self.expect('[')
            ds = [self.day()]
            while self.accept(','):
                ds.append(self.day())
            self.expect(']')
            self.expect(')')
            return self.N('oneof', days=ds)
        self.err('a window or one_of')

    # -- pins ----------------------------------------------------------------
    def pin(self):
        i0 = self.i
        n = self.pin_()
        n._src = self.src(i0)
        return n

    def pin_(self):
        if self.at('law'):
            self.i += 1
            return self.N('pin', kind='law', value=self.expr())
        if self.at('known'):
            self.i += 1
            return self.N('pin', kind='known', value=self.expr())
        if self.at('wording'):
            self.i += 1
            d = self.expr()
            self.expect('for')
            self.expect('[')
            ids = [self.qid_text()]
            while self.accept(','):
                ids.append(self.qid_text())
            self.expect(']')
            return self.N('pin', kind='wording', value=d, ids=ids)
        if self.at('scenario') and self.peek(1) is not None and self.peek(1).t == 'ID':
            self.i += 1
            return self.N('pin', kind='scenario', id=self.ident())
        sample = False
        if self.at('sample') and self.peek(1) is not None and self.peek(1).t == 'ID' and not self.at('=', 1):
            self.i += 1
            sample = True
        target = self.pref()
        self.expect('=')
        if not sample and target.keys is None and not target.fields:
            # ID = key: an axis option, decided at resolution (L §2.3 pin)
            tok = self.peek()
            val = self.expr()
            return self.N('pin', kind='set', target=target, value=val, sample=False)
        return self.N('pin', kind='set', target=target, value=self.expr(), sample=sample)

    def qid_text(self):
        a = self.ident()
        if self.at('.') and not self.peek().ws:
            self.i += 1
            return a + '.' + self.ident()
        return a

    def pref(self):
        """pref = ID ["[" pk {"," pk} "]"] {"." FIELD}   pk = expr | "*" """
        name = self.ident('a pinned id')
        qual = None
        keys = None
        fields = []
        if self.at('.') and not self.peek().ws and self.peek(1) is not None and self.peek(1).t == 'ID' \
                and self.at('[', 2):
            # R.x[...]
            self.i += 1
            qual = name
            name = self.ident()
        if self.at('['):
            self.i += 1
            self.open()
            keys = [self.key_expr()]
            while self.accept(','):
                keys.append(self.key_expr())
            self.expect(']')
            self.close()
        while self.at('.') and not self.peek().ws:
            self.i += 1
            tok = self.next()
            if tok.t != 'ID':
                self.err('a field')
            fields.append(tok.s)
        return self.N('pref', id=name, qual=qual, keys=keys, fields=fields)

    # -- types -----------------------------------------------------------------
    def type(self):
        tok = self.peek()
        if tok is None:
            self.err('a type')
        if self.at('('):
            self.i += 1
            fields = []
            while True:
                f = self.next()
                if f.t != 'ID':
                    self.err('a field name')
                fields.append((f.s, self.type()))
                if not self.accept(','):
                    break
            self.expect(')')
            return ('record', tuple(fields))
        if tok.t != 'ID':
            self.err('a type')
        s = tok.s
        self.i += 1
        if s == 'money':
            return ('money', self.cur())
        if s == 'num':
            return ('num', self.type_unit())
        if s == 'dist':
            return ('dist', self.type_unit())
        if s == 'key':
            a = self.ident('an index or kind')
            if self.at('.') and not self.peek().ws:
                self.i += 1
                a = a + '.' + self.ident()
            return ('key', a)
        if s in ('int', 'bool', 'text', 'date', 'dates', 'month', 'monthday', 'quarter', 'year', 'rate', 'calendar',
                 'rounding', 'daycount', 'roll', 'compounding'):
            return (s,)
        self.i -= 1
        self.err('a type')

    def bindlist(self):
        """binds = "[" bind {"," bind} "]" """
        self.expect('[')
        self.open()
        out = [self.bind()]
        while self.accept(','):
            out.append(self.bind())
        self.expect(']')
        self.close()
        return out

    # -- version items -----------------------------------------------------------
    def qid(self):
        a = self.ident('an id')
        if self.at('.') and not self.peek().ws and self.peek(1) is not None and self.peek(1).t == 'ID':
            self.i += 1
            return (a, self.ident())
        return (None, a)

    def rowkey(self):
        if self.accept('*'):
            return self.N('star')
        tok = self.peek()
        if tok is not None and tok.t == 'ID' and self.at('[', 1):
            name = self.ident()
            self.i += 1
            ks = [self.lit()]
            while self.accept(','):
                ks.append(self.lit())
            self.expect(']')
            return self.N('placekey', id=name, keys=ks)
        a = self.lit()
        if self.accept('..'):
            b = self.lit()
            return self.N('krange', a=a, b=b)
        return a

    def value(self):
        """value of a row (L §2.3 value)."""
        if self.at('?') and self.at_t('STR', 1):
            self.i += 1
            return self.N('hole', text=self.string()), None
        if self.at('never'):
            self.i += 1
            return self.N('never'), None
        if self.at('none'):
            self.i += 1
            assumed = None
            if self.accept('assumed'):
                assumed = self.string()
            return self.N('none'), assumed
        tok = self.peek()
        if tok is None:
            self.err('a value')
        if tok.t == 'STR':
            self.i += 1
            v = self.N('str', v=tok.v)
        elif self.at('('):
            self.i += 1
            fields = [self.field()]
            while self.accept(','):
                fields.append(self.field())
            self.expect(')')
            v = self.N('record', fields=fields)
        elif self.at('['):
            self.i += 1
            items = []
            if not self.at(']'):
                items.append(self.lit())
                while self.accept(','):
                    items.append(self.lit())
            self.expect(']')
            v = self.N('list', items=items)
        elif self.at('calendar') and self.at('(', 1):
            v = self.calval()
        elif (self.at('window') or self.at('one_of')) and self.at('(', 1):
            v = self.dayset()
        elif tok.t == 'ID' and (tok.s in DISTS or tok.s == 'discrete') and self.at('(', 1):
            v = self.builtin(tok.s)
            if self.accept('to'):
                v = self.N('to', e=v, unit=self.step_unit())
        else:
            save = self.i
            if tok.t == 'ID' and self.at('.', 1):
                v = None
            else:
                v = self.lit()
            nxt = self.peek()
            if v is None or (nxt is not None and nxt.s in ('.', '*', '/', '+', '-', '(', '[')):
                # in a scenario's version, an expression over the base's facts (L §6.4); the loader refuses it
                # elsewhere
                self.i = save
                v = self.N('rowexpr', e=self.expr())
        assumed = None
        if self.accept('assumed'):
            assumed = self.string()
        return v, assumed

    def rowmeta(self):
        gov = known = instant = loc = None
        if self.accept('governs'):
            if self.accept('always'):
                gov = 'always'
            else:
                gov = self.lspan()
        if self.accept('known'):
            known = self.known()
        if self.at('at'):
            self.i += 1
            d = self.day()
            pos = None
            if self.accept('start'):
                pos = 'start'
            elif self.accept('end'):
                pos = 'end'
            instant = self.N('instant', day=d, pos=pos, after=None)
        elif self.accept('after'):
            instant = self.N('instant', day=None, pos=None, after=self.ref())
        if self.at_t('STR'):
            loc = self.string()
        return gov, known, instant, loc

    def interval(self):
        if self.accept('['):
            lo_closed = True
        elif self.accept('('):
            lo_closed = False
        else:
            self.err('an interval')
        if self.at('-') and self.at('inf', 1):
            self.i += 2
            lo = None
        else:
            lo = self.lit()
        self.expect(',')
        if self.accept('inf'):
            hi = None
        else:
            hi = self.lit()
        if self.accept(']'):
            hi_closed = True
        elif self.accept(')'):
            hi_closed = False
        else:
            self.err('"]" or ")"')
        return self.N('interval', lo=lo, hi=hi, lo_closed=lo_closed, hi_closed=hi_closed)

    def vitem(self):
        replace = False
        if self.accept('replace'):
            replace = True
            if self.accept('rule'):
                d = self.ident('a def')
                self.expect('.')
                lab = self.label()
                q = None
                if self.at('.') and not self.peek().ws:
                    # replace rule R.d.L
                    self.i += 1
                    q, d, lab = d, lab, self.label()
                return self.N('rulerep', qual=q, d=d, label=lab, rule=self.rulebody(lab))
        given = False
        if self.accept('given'):
            if replace:
                self.err('a row')
            given = True
        if self.accept('same'):
            if replace or given:
                self.err('a row')
            l = self.ref()
            self.expect('=')
            r = self.ref()
            self.end()
            return self.N('same', l=l, r=r)
        q, x = self.qid()
        if self.at('+'):
            if given:
                self.err('a row')
            self.i += 1
            self.expect('[')
            keys = [self.lit()]
            while self.accept(','):
                keys.append(self.lit())
            self.expect(']')
            unit = None
            if self.at_t('CUR'):
                unit = self.cur()
            elif self.peek() is not None and self.peek().t == 'ID' and self.peek().s not in ('priced', 'lives'):
                unit = self.ident()
            priced = None
            if self.accept('priced'):
                priced = self.price()
            lives = None
            if self.accept('lives'):
                lives = self.life_span()
            self.end()
            return self.N('enter', qual=q, target=x, keys=keys, unit=unit, priced=priced, lives=lives,
                          replace=replace)
        key = None
        keys = None
        if self.at('['):
            self.i += 1
            ks = [self.rowkey()]
            while self.accept(','):
                ks.append(self.rowkey())
            self.expect(']')
            keys = ks
        if self.at(':'):
            if given:
                self.err('a row')
            self.i += 1
            if keys is not None and len(keys) != 1:
                self.err('one key of a keyed band')
            iv = self.interval()
            self.expect('=')
            if self.at('?') and self.at_t('STR', 1):
                self.i += 1
                val = self.N('hole', text=self.string())
            else:
                val = self.expr()
            loc = None
            if self.at_t('STR'):
                loc = self.string()
            self.end()
            return self.N('bandrow', qual=q, target=x, key=(keys[0] if keys else None), interval=iv, value=val,
                          locator=loc, replace=replace)
        self.expect('=')
        val, assumed = self.value()
        gov, known, instant, loc = self.rowmeta()
        self.end()
        return self.N('row', qual=q, target=x, keys=keys, value=val, assumed=assumed, governs=gov, known=known,
                      instant=instant, locator=loc, given=given, replace=replace)

    def rulebody(self, label):
        beats = []
        if self.accept('beats'):
            beats.append(self.label())
            while self.accept(','):
                beats.append(self.label())
        v = None
        if self.at('v') and self.peek(1) is not None and self.peek(1).t == 'ID':
            self.i += 1
            v = self.ident()
        gov = None
        if self.accept('governs'):
            gov = self.lspan()
        # L §2.3 rulebody: "=" (expr | hole | solvex) is required (the review removed the defeater)
        self.expect('=')
        if self.at('?') and self.at_t('STR', 1):
            self.i += 1
            val = self.N('hole', text=self.string())
        elif self.at('solve'):
            val = self.solvex()
        else:
            val = self.expr()
        when = None
        if self.accept('when'):
            when = self.expr()
        self.end()
        return self.N('rule', label=label, beats=beats, v=v, governs=gov, value=val, when=when)

    def solvex(self):
        self.expect('solve')
        f = self.sum(False, False)
        self.expect('==')
        g = self.sum(False, False)
        self._no_chain()
        self.expect('for')
        r = self.ref()
        self.expect('in')
        self.expect('[')
        lo = self.lit()
        self.expect(',')
        hi = self.lit()
        self.expect(']')
        self.expect('to')
        u = self.step_unit()
        return self.N('solve', f=f, g=g, ref=r, lo=lo, hi=hi, unit=u)


# ---------------------------------------------------------------------------
# sections

def _children(lines, i, level):
    """The lines after lines[i] that are indented deeper than `level`."""
    j = i + 1
    while j < len(lines) and lines[j].level > level:
        j += 1
    return lines[i + 1:j], j


def _p(line, units):
    return P(line.toks, line.loc, units)


def _first(line):
    return line.toks[0].s if line.toks else ''


def parse_header(lines, units):
    """Check the header before the grammar (L §5.1), then parse it."""
    first = lines[0]
    kind = _first(first)
    if first.level != 0 or kind not in ('model', 'part', 'scenario'):
        raise LoadError('LOAD-HEADER', 'file has no model, part or scenario header', first.loc)
    body, j = _children(lines, 0, 0)
    words = [_first(l) for l in body if l.level == 1]
    if 'tally' not in words:
        raise LoadError('LOAD-HEADER', 'missing tally version', first.loc)
    if kind == 'model':
        if 'on' not in words:
            raise LoadError('LOAD-HEADER', 'missing on date', first.loc)
        if 'start' not in words or 'end' not in words:
            raise LoadError('LOAD-HEADER', 'missing start or end', first.loc)
    if kind == 'scenario' and 'from' not in words:
        raise LoadError('LOAD-HEADER', 'missing from date', first.loc)
    p = _p(first, units)
    p.i = 1
    hid = p.ident('an id')
    over = None
    if kind == 'scenario':
        p.expect('over')
        over = p.ident('an id')
    p.end()
    h = Node('header', first.loc, kind=kind, id=hid, over=over, tally=None, on=None, start=None, end=None,
             uses=[], sames=[], notes=[], law=False, frm=None)
    order = {'model': ['tally', 'on', 'start', 'end', 'use', 'same', 'note'],
             'part': ['tally', 'law', 'note'],
             'scenario': ['tally', 'from', 'note']}[kind]
    pos = 0
    for l in body:
        if l.level != 1:
            raise LoadError('LOAD-PARSE', 'cannot parse: indentation in the header', l.loc)
        w = _first(l)
        if w not in order:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in a header' % w, l.loc)
        k = order.index(w)
        if k < pos:
            raise LoadError('LOAD-PARSE', 'cannot parse: header line %s out of order' % w, l.loc)
        if w in ('tally', 'on', 'start', 'end', 'law', 'from') and k == pos and getattr(h, '_seen_' + w, False):
            raise LoadError('LOAD-PARSE', 'cannot parse: header line %s twice' % w, l.loc)
        setattr(h, '_seen_' + w, True)
        pos = k
        q = _p(l, units)
        q.i = 1
        if w == 'tally':
            h.tally = q.integer()
        elif w == 'on':
            h.on = q.day()
        elif w == 'start':
            h.start = q.day()
        elif w == 'end':
            h.end = q.day()
        elif w == 'law':
            h.law = True
        elif w == 'from':
            h.frm = q.day()
        elif w == 'use':
            a = q.ident()
            q.expect('=')
            f = q.string()
            q.expect('sha256')
            hsh = q.string()
            h.uses.append((a, f, hsh))
        elif w == 'same':
            a = q.ref()
            q.expect('=')
            b = q.ref()
            h.sames.append((a, b))
        elif w == 'note':
            h.notes.append(q.string())
        q.end()
    if kind == 'model' and h.start > h.end:
        raise LoadError('LOAD-HEADER', 'start after end', first.loc)
    return h, j


def parse_section(lines, i, units):
    line = lines[i]
    body, j = _children(lines, i, line.level)
    w = _first(line)
    p = _p(line, units)
    # GD-1 (L §2.3 section, round 19): `regime` is no section; a `regime` line is refused as any unknown section is
    fn = SECTION.get(w)
    if fn is None:
        raise LoadError('LOAD-PARSE', 'cannot parse: expected a section, got %s' % w, line.loc)
    p.i = 1
    node = fn(p, body, units, line)
    node.loc = line.loc
    return node, j


def _notes(body, units, allowed_from=0):
    notes = []
    rest = []
    for l in body:
        if _first(l) == 'note':
            q = _p(l, units)
            q.i = 1
            notes.append(q.string())
            q.end()
        else:
            rest.append(l)
    return notes, rest


def _flat(body, line):
    for l in body:
        if l.level != line.level + 1:
            raise LoadError('LOAD-PARSE', 'cannot parse: indentation', l.loc)


def sec_source(p, body, units, line):
    sid = p.ident()
    title = p.string()
    at = None
    if p.accept('at'):
        at = p.string()
    p.end()
    if body:
        raise LoadError('LOAD-PARSE', 'cannot parse: a source has no body', body[0].loc)
    return Node('source', id=sid, title=title, at=at)


def sec_version(p, body, units, line):
    vid = p.ident()
    p.end()
    _flat(body, line)
    v = Node('version', id=vid, cite=None, governs=None, known=None, enters=None, assumed=None, moves=[],
             covers=[], gaps=[], notes=[], items=[])
    order = ['cite', 'governs', 'known', 'enters', 'assumed', 'moves', 'covers', 'gap', 'note', 'item']
    pos = -1
    for l in body:
        w = _first(l)
        q = _p(l, units)
        q.i = 1
        tag = w if w in ('cite', 'governs', 'known', 'enters', 'assumed', 'moves', 'covers', 'note') else (
            'gap' if w in ('pad', 'anchor') else 'item')
        # `known` and the like are keywords; an item never starts with one, except `replace`, `given`, `same`
        k = order.index(tag)
        if k < pos or (k == pos and tag in ('cite', 'governs', 'known', 'enters', 'assumed', 'covers')):
            raise LoadError('LOAD-PARSE', 'cannot parse: version line %s out of order or repeated' % w, l.loc)
        pos = k
        if tag == 'cite':
            v.cite = q.ident()
        elif tag == 'governs':
            if q.accept('always'):
                v.governs = 'always'
            else:
                v.governs = q.lspan()
        elif tag == 'known':
            v.known = q.known()
        elif tag == 'enters':
            v.enters = q.day()
        elif tag == 'assumed':
            v.assumed = q.string()
        elif tag == 'moves':
            r = q.ref()
            q.expect('to')
            v.moves.append((r, q.lspan()))
        elif tag == 'covers':
            items = []
            while True:
                r = q.ref()
                sp = None
                if not q.done() and not q.at(','):
                    sp = q.lspan()
                items.append((r, sp))
                if not q.accept(','):
                    break
            v.covers = items
        elif tag == 'gap':
            if w == 'pad':
                r = q.ref()
                q.expect('from')
                v.gaps.append(('pad', r, q.ref()))
            else:
                v.gaps.append(('anchor', q.ref(), None))
        elif tag == 'note':
            v.notes.append(q.string())
        else:
            q.i = 0
            it = q.vitem()
            it.loc = l.loc
            v.items.append(it)
            continue
        q.end()
    if v.cite is None:
        raise LoadError('LOAD-PARSE', 'cannot parse: version %s has no cite' % vid, line.loc)
    if v.known is None:
        raise LoadError('LOAD-KNOWN', 'version %s has no known date' % vid, line.loc)
    return v


def sec_unit(p, body, units, line):
    if p.at_t('CUR'):
        c = p.cur()
        p.expect(':')
        p.expect('currency')
        p.expect('minor')
        m = p.lit(signed=False)
        p.end()
        if body:
            raise LoadError('LOAD-PARSE', 'cannot parse: a unit has no body', body[0].loc)
        return Node('unitdecl', id=c, cur=True, minor=m, label=None)
    uid = p.ident()
    label = p.string()
    p.end()
    if body:
        raise LoadError('LOAD-PARSE', 'cannot parse: a unit has no body', body[0].loc)
    return Node('unitdecl', id=uid, cur=False, minor=None, label=label)


def sec_kind(p, body, units, line):
    kid = p.ident()
    k = Node('kind', id=kid, spec='plain', parent=None, certainty=None, unit=None, sign=None, priced=None,
             outside=False, legs=[], attrs=[], notes=[])
    if p.accept(':'):
        if p.accept('place'):
            k.spec = 'place'
            if p.at_t('CUR'):
                k.unit = p.cur()
            else:
                k.unit = p.ident('a unit, any or one')
            if p.accept('signed'):
                k.sign = 'signed'
            elif p.accept('min'):
                if not (p.at_t('INT') and p.peek().v == 0):
                    p.err('0')
                p.i += 1
                k.sign = 'min0'
            elif p.accept('max'):
                if not (p.at_t('INT') and p.peek().v == 0):
                    p.err('0')
                p.i += 1
                k.sign = 'max0'
            else:
                raise LoadError('LOAD-PLACEKIND', 'place kind %s has no unit or sign rule' % kid, line.loc)
            if p.accept('priced'):
                k.priced = p.price()
            if p.accept('outside'):
                k.outside = True
        elif p.accept('unit'):
            k.spec = 'unit'
        elif p.at('event') and (p.at('certain', 1) or p.at('uncertain', 1) or p.peek(1) is None):
            p.i += 1
            k.spec = 'event'
            if p.accept('certain'):
                k.certainty = 'certain'
            elif p.accept('uncertain'):
                k.certainty = 'uncertain'
            else:
                p.err('certain or uncertain')
        else:
            k.spec = 'sub'
            k.parent = p.ident('a kind')
            if p.accept('certain'):
                k.certainty = 'certain'
            elif p.accept('uncertain'):
                k.certainty = 'uncertain'
    p.end()
    _flat(body, line)
    for l in body:
        q = _p(l, units)
        w = _first(l)
        q.i = 1
        if w == 'leg':
            name = q.ident()
            q.expect(':')
            if q.at_t('CUR'):
                u = q.cur()
            else:
                u = q.ident()
            k.legs.append((name, u))
        elif w == 'attr':
            name = q.ident()
            q.expect(':')
            k.attrs.append((name, q.type()))
        elif w == 'note':
            k.notes.append(q.string())
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in a kind' % w, l.loc)
        q.end()
    return k


def sec_axis(p, body, units, line):
    aid = p.ident()
    p.end()
    _flat(body, line)
    opts = []
    for l in body:
        q = _p(l, units)
        tok = q.next()
        if tok.t not in ('ID', 'KEY', 'INT'):
            q.i -= 1
            q.err('an option key')
        lab = q.string()
        q.end()
        opts.append((tok.s, lab))
    if len(opts) < 2:
        raise LoadError('LOAD-PARSE', 'cannot parse: axis %s needs two options' % aid, line.loc)
    return Node('axis', id=aid, options=opts)


def sec_index(p, body, units, line):
    iid = p.ident()
    n = Node('index', id=iid, form=None, keys=None, pk=None, lo=None, hi=None, by=None, frm=None, v=None,
             assumed=None, notes=[])
    if p.accept(':'):
        p.expect('open')
        n.form = 'open'
    else:
        p.expect('=')
        if p.at('['):
            p.i += 1
            ks = [p.lit()]
            while p.accept(','):
                ks.append(p.lit())
            p.expect(']')
            n.form = 'list'
            n.keys = ks
        else:
            pk = None
            if p.peek() is not None and p.peek().t == 'ID' and p.peek().s in ('year', 'month', 'day', 'int'):
                pk = p.next().s
            n.form = 'range'
            n.pk = pk
            n.lo = p.lit()
            p.expect('..')
            n.hi = p.lit()
            if p.accept('by'):
                n.by = p.integer()
            if p.accept('from'):
                tok = p.next()
                if tok.t == 'MMDD':
                    n.frm = ('mmdd', tok.v)
                elif tok.t == 'INT':
                    n.frm = ('day', tok.v)
                else:
                    p.i -= 1
                    p.err('a month-day or a day of the month')
    p.end()
    _flat(body, line)
    for l in body:
        q = _p(l, units)
        w = _first(l)
        q.i = 1
        if w == 'v':
            n.v = q.ident()
        elif w == 'assumed':
            n.assumed = q.string()
        elif w == 'note':
            n.notes.append(q.string())
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in an index' % w, l.loc)
        q.end()
    return n


def sec_place(p, body, units, line):
    pid = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.expect(':')
    kind = p.ident('a place kind')
    unit = None
    if p.at_t('CUR'):
        unit = p.cur()
    elif p.peek() is not None and p.peek().t == 'ID' and p.peek().s not in ('priced', 'lives'):
        unit = p.ident()
    priced = None
    if p.accept('priced'):
        priced = p.price()
    p.expect('lives')
    lives = p.lspan()
    p.end()
    if body:
        raise LoadError('LOAD-PARSE', 'cannot parse: a place has no body', body[0].loc)
    return Node('place', id=pid, binds=binds, kind=kind, unit=unit, priced=priced, lives=lives)


def sec_input(p, body, units, line):
    iid = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.expect(':')
    n = Node('input', id=iid, binds=binds, type=None, reading=False, prices=None, prices_by=None,
             question=None, notes=[], observed=False)
    if p.at('observed') and p.peek(1) is None:
        p.i += 1
        if binds:
            p.err('no keys on an observed input')
        n.observed = True
    else:
        n.type = p.type()
        if p.accept('reading'):
            n.reading = True
        if p.accept('prices'):
            n.prices = p.lit()
            if p.accept('by'):
                n.prices_by = p.ref()
        if p.accept('question'):
            n.question = p.string()
    p.end()
    notes, rest = _notes(body, units)
    if rest:
        raise LoadError('LOAD-PARSE', 'cannot parse: %s in an input' % _first(rest[0]), rest[0].loc)
    n.notes = notes
    return n


def sec_band(p, body, units, line):
    bid = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.expect('(')
    arg = p.ident()
    p.expect(':')
    at = p.type()
    p.expect(')')
    p.expect(':')
    rt = p.type()
    p.end()
    notes, rest = _notes(body, units)
    if rest:
        raise LoadError('LOAD-PARSE', 'cannot parse: %s in a band' % _first(rest[0]), rest[0].loc)
    return Node('band', id=bid, binds=binds, arg=arg, argtype=at, type=rt, notes=notes)


def _lawdate(p):
    """lawdate = expr | "reader" (L §2.3; the review removed `each day of m split by days`)"""
    if p.accept('reader'):
        return Node('law_reader', p.loc)
    return Node('law_expr', p.loc, e=p.expr())


def sec_def(p, body, units, line):
    did = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.expect(':')
    t = p.type()
    n = Node('def', id=did, binds=binds, type=t, at=None, rules=[], notes=[], short=False)
    if p.accept('='):
        n.short = True
        if p.at('?') and p.at_t('STR', 1):
            p.i += 1
            val = Node('hole', p.loc, text=p.string())
        else:
            val = p.expr()
        if p.accept('at'):
            n.at = _lawdate(p)
        p.end()
        n.rules = [Node('rule', line.loc, label='base', beats=[], v=None, governs=None, value=val, when=None)]
        if body:
            raise LoadError('LOAD-PARSE', 'cannot parse: a short def has no body', body[0].loc)
        return n
    if p.at('at'):
        n._at = p.peek()
    if p.accept('at'):
        n.at = _lawdate(p)
    p.end()
    _flat(body, line)
    seen_rule = False
    for l in body:
        q = _p(l, units)
        w = _first(l)
        q.i = 1
        if w == 'note':
            if seen_rule:
                raise LoadError('LOAD-PARSE', 'cannot parse: notes come before rules', l.loc)
            n.notes.append(q.string())
            q.end()
        elif w == 'rule':
            seen_rule = True
            lab = q.label()
            r = q.rulebody(lab)
            r.loc = l.loc
            n.rules.append(r)
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in a def' % w, l.loc)
    if not n.rules:
        raise LoadError('LOAD-PARSE', 'cannot parse: def %s has no rule' % did, line.loc)
    return n


def sec_fill(p, body, units, line):
    fid = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.end()
    _flat(body, line)
    n = Node('fill', id=fid, binds=binds, v=None, notes=[], supply=None, sbinds=[], demand=None, dbinds=[],
             arc=None, order=None, tie=None)
    for l in body:
        q = _p(l, units)
        w = _first(l)
        q.i = 1
        if w == 'v':
            n.v = q.ident()
        elif w == 'note':
            n.notes.append(q.string())
        elif w in ('supply', 'demand'):
            e = q.expr()
            bs = []
            if q.accept('for'):
                bs = q.binds()
            if w == 'supply':
                n.supply, n.sbinds = e, bs
            else:
                n.demand, n.dbinds = e, bs
        elif w == 'arc':
            n.arc = q.expr()
        elif w == 'order':
            os_ = [q.expr()]
            while q.accept(','):
                os_.append(q.expr())
            if q.at('ties'):
                n.tie = q.tieby()
            n.order = os_
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in a fill' % w, l.loc)
        q.end()
    if n.supply is None or n.demand is None or n.arc is None or n.order is None:
        raise LoadError('LOAD-PARSE', 'cannot parse: fill %s needs supply, demand, arc and order' % fid, line.loc)
    return n


def sec_move(p, body, units, line):
    mid = p.ident()
    binds = p.bindlist() if p.at('[') else []
    p.expect(':')
    kind = p.ident('an event kind')
    if p.at('.') and not p.peek().ws:
        p.i += 1
        kind = kind + '.' + p.ident()
    p.end()
    _flat(body, line)
    n = Node('move', id=mid, binds=binds, kind=kind, on=None, at=None, forp=None, when=None, reads=[],
             order=[], tie=None, attrs=[], legs=[], fulfils=None, notes=[])
    for l in body:
        q = _p(l, units)
        w = _first(l)
        q.i = 1
        if w == 'on':
            n.on = q.expr()
        elif w == 'at':
            n.at = q.expr()
        elif w == 'for':
            n.forp = q.expr()
        elif w == 'when':
            n.when = q.expr()
        elif w == 'reads':
            rs = [q.ref()]
            while q.accept(','):
                rs.append(q.ref())
            n.reads = rs
        elif w in ('before', 'after'):
            n.order.append((w, q.ident()))
        elif w == 'ties':
            q.i = 0
            n.tie = q.tieby()
        elif w == 'attr':
            a = q.ident()
            q.expect('=')
            n.attrs.append((a, q.expr()))
        elif w == 'leg':
            q.i = 0
            n.legs.append(q.leg_body())
        elif w == 'fulfils':
            n.fulfils = q.ref()
        elif w == 'note':
            n.notes.append(q.string())
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s in a move' % w, l.loc)
        q.end()
    if n.on is None:
        raise LoadError('LOAD-PARSE', 'cannot parse: move %s has no on' % mid, line.loc)
    return n


ASK_CLAUSES = ('for', 'in', 'with', 'law', 'as', 'windows', 'random', 'paths', 'path', 'partial', 'orient')


def sec_ask(p, body, units, line):
    aid = p.ident()
    p.expect(':')
    a = Node('ask', id=aid, form=None, exprs=None, ref=None, check=None, solve=None, diff=None, resid=None,
             binds=[], world=None, pins=[], law=None, known=None, windows=None, uncertain=False, random=None,
             paths=None, seed=None, path=None, partial=False, orient=None)
    if p.accept('explain'):
        a.form = 'explain'
        if p.at('bal') or p.at('low'):
            # explain bal(x, d) [from t] / explain low(...) (L §2.3 askbody, §5.18)
            a.ref = p.builtin(p.peek().s)
        else:
            a.ref = p.ref()
    elif p.accept('inputs-of'):
        a.form = 'inputs-of'
        a.ref = p.ref()
    elif p.accept('readers-of'):
        a.form = 'readers-of'
        a.ref = p.ref()
    elif p.accept('check'):
        a.form = 'check'
        l = p.sum(True, False)
        p.expect('==')
        r = p.sum(True, False)
        p._no_chain()
        a.check = (l, r)
    elif p.accept('residuals'):
        a.form = 'residuals'
        r = p.ref()
        sp = p.span()
        by = None
        if p.accept('by'):
            by = p.ident()
        a.resid = (r, sp, by)
    elif p.at('solve'):
        a.form = 'solve'
        a.solve = p.solvex()
    elif p.at('diff') and p.at('(', 1):
        p.i += 2
        p.open()
        e = p.expr()
        bs = []
        if p.accept('for'):
            bs = p.binds()
        p.expect(')')
        p.close()
        p.expect('in')
        s = p.ident()
        p.expect('against')
        b = p.ident()
        a.form = 'diff'
        a.diff = (e, bs, s, b)
    else:
        a.form = 'exprs'

        def body_expr():
            # L §2.3 parse rule 4: `in ID` after the body names the world; a membership test followed by a
            # comma, or by `on`, can't be the world, so it is read as the test
            save = p.i
            e = p.expr(no_in=True)
            if p.at('in') and p.peek(1) is not None and p.peek(1).t == 'ID' and (p.at(',', 2) or p.at('on', 2)):
                p.i = save
                e = p.expr()
            return e
        es = [body_expr()]
        while p.accept(','):
            es.append(body_expr())
        a.exprs = es
    if p.accept('for'):
        p.askfor = True
        try:
            a.binds = p.binds()
        finally:
            p.askfor = False
    if p.at('in') and p.peek(1) is not None and p.peek(1).t == 'ID':
        p.i += 1
        a.world = p.ident()
    if p.at('with') and not p.at('uncertain', 1):
        p.i += 1
        a.pins = [p.pin()]
        while p.accept(','):
            a.pins.append(p.pin())
    if p.accept('law'):
        a.law = p.expr()
    if p.at('as') and p.at('known', 1):
        p.i += 2
        a.known = p.expr()
    if p.accept('windows'):
        a.windows = p.wchoice()
    if p.at('with') and p.at('uncertain', 1):
        p.i += 2
        a.uncertain = True
    if p.at('random') and p.at('at', 1):
        p.i += 2
        a.random = p.rchoice()
    if p.accept('paths'):
        a.paths = p.integer()
        p.expect('seed')
        a.seed = p.integer()
    if p.accept('path'):
        a.path = p.integer()
    if p.accept('partial'):
        a.partial = True
    if p.accept('orient'):
        if p.accept('lower'):
            a.orient = 'lower'
        elif p.accept('higher'):
            a.orient = 'higher'
        else:
            p.err('lower or higher')
    p.end()
    if body:
        raise LoadError('LOAD-PARSE', 'cannot parse: an ask is one line', body[0].loc)
    return a


SECTION = {'source': sec_source, 'version': sec_version, 'unit': sec_unit, 'kind': sec_kind, 'axis': sec_axis,
           'index': sec_index, 'place': sec_place, 'input': sec_input, 'band': sec_band, 'def': sec_def,
           'fill': sec_fill, 'move': sec_move, 'ask': sec_ask}


def asif_depth(node):
    """The nesting of `as if` (L §2.3: at most 4)."""
    from .node import Node as _N
    best = 0

    def d(n):
        if isinstance(n, list) or isinstance(n, tuple):
            return max([d(x) for x in n] or [0])
        if not isinstance(n, _N):
            return 0
        inner = 0
        for a, b in n.__dict__.items():
            if a in ('k', 'loc') or a.startswith('_'):
                continue
            if isinstance(b, (_N, list, tuple)):
                inner = max(inner, d(b))
        return inner + (1 if n.k == 'asif' else 0)
    return d(node)


def parse_file(lines, units):
    if not lines:
        raise LoadError('LOAD-HEADER', 'empty file')
    header, i = parse_header(lines, units)
    sections = []
    while i < len(lines):
        l = lines[i]
        if l.level != 0:
            raise LoadError('LOAD-PARSE', 'cannot parse: indentation', l.loc)
        node, i = parse_section(lines, i, units)
        sections.append(node)
    for s in sections:
        if asif_depth(s) > MAX_ASIF:
            raise LoadError('LOAD-ASIF', 'as if deeper than 4', s.loc)
    return Node('file', header=header, sections=sections)


def prescan_units(all_lines):
    """Unit names a number may carry: declared `unit` ids and keys of unit kinds (L §3.2)."""
    units = set()
    unit_kinds = set(['currencies'])
    for lines in all_lines:
        for l in lines:
            if l.level != 0 and not (l.level == 1 and l.toks and l.toks[0].s == 'kind'):
                continue
            t = l.toks
            if len(t) >= 2 and t[0].s == 'unit' and t[1].t == 'ID':
                units.add(t[1].s)
            if len(t) >= 4 and t[0].s == 'kind' and t[2].s == ':' and t[3].s == 'unit' and len(t) == 4:
                unit_kinds.add(t[1].s)
    for lines in all_lines:
        for l in lines:
            t = l.toks
            # K + [k1, k2] ... inside a version, or replace K + [...]
            j = 1 if (t and t[0].s == 'replace') else 0
            if len(t) > j + 2 and t[j].t == 'ID' and t[j].s in unit_kinds and t[j + 1].s == '+' and t[j + 2].s == '[':
                k = j + 3
                while k < len(t) and t[k].s != ']':
                    if t[k].t in ('ID', 'KEY'):
                        units.add(t[k].s)
                    k += 1
    return units
