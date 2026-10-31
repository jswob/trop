"""Static types (L §3.1).

Ty is immutable.  Kinds:
  num(u) money(cur) int bool text date dates period(pk) key(ix) rate dist(u)
  calendar code(kind) record(fields) list(elem) tuple(items)
and the kinds of literals whose type the context decides:
  flex(n)   a bare 0, 1 or -1 (L §3.2 Inline figures)
  year4     a four-digit integer: a year or an int (L §2.1)
  keylit    a key literal not yet resolved by position (L §2.5)
  none      `none`; hole: a rule value `?"..."`; never (a date)
"""


class Ty:
    __slots__ = ('k', 'u', 'cur', 'pk', 'ix', 'code', 'fields', 'elem', 'items', 'n')

    def __init__(self, k, u=None, cur=None, pk=None, ix=None, code=None, fields=None, elem=None, items=None,
                 n=None):
        self.k = k
        self.u = u
        self.cur = cur
        self.pk = pk
        self.ix = ix
        self.code = code
        self.fields = fields
        self.elem = elem
        self.items = items
        self.n = n

    def key(self):
        return (self.k, self.u, self.cur, self.pk, self.ix, self.code,
                tuple((a, b.key()) for a, b in self.fields) if self.fields else None,
                self.elem.key() if self.elem else None,
                tuple(b.key() for b in self.items) if self.items else None)

    def __eq__(self, o):
        return isinstance(o, Ty) and self.key() == o.key()

    def __hash__(self):
        return hash(self.key())

    def __repr__(self):
        return show(self)

    def field(self, name):
        for a, b in self.fields or ():
            if a == name:
                return b
        return None


INT = Ty('int')
BOOL = Ty('bool')
TEXT = Ty('text')
DATE = Ty('date')
DATES = Ty('dates')
RATE = Ty('rate')
CAL = Ty('calendar')
MD = Ty('mmdd')           # L §3.1 monthday (GD-2)
NONE = Ty('none')
HOLE = Ty('hole')
NEVER = Ty('never')
YEAR4 = Ty('year4')
NUM1 = Ty('num', u=())


def num(u):
    return Ty('num', u=tuple(u) if u else ())


def pct():
    """`num %`: the num whose unit is the scale % (L §3.2); equal to `num` as a type, printed with `%`."""
    return Ty('num', u=(), n='pct')


def is_pct(t):
    return t is not None and t.k == 'num' and not t.u and t.n == 'pct'


def money(cur):
    return Ty('money', u=((cur, 1),), cur=cur)


def period(pk):
    return Ty('period', pk=pk)


def key(ix):
    return Ty('key', ix=ix)


def code(kind):
    return Ty('code', code=kind)


def record(fields):
    return Ty('record', fields=tuple(fields))


def flex(n):
    return Ty('flex', n=n)


def show(t):
    from . import units as U
    if t is None:
        return '?'
    k = t.k
    if k == 'num':
        if t.n == 'pct':
            return 'num %'
        return 'num' + ((' ' + U.fmt(t.u)) if t.u else '')
    if k == 'money':
        return 'money ' + t.cur
    if k == 'dist':
        return 'dist' + ((' ' + U.fmt(t.u)) if t.u else '')
    if k == 'period':
        return t.pk
    if k == 'mmdd':
        return 'monthday'
    if k == 'key':
        return 'key ' + t.ix
    if k == 'code':
        return t.code
    if k == 'record':
        return '(' + ', '.join('%s %s' % (a, show(b)) for a, b in t.fields) + ')'
    if k == 'list':
        return 'list of ' + show(t.elem)
    if k == 'tuple':
        return '(' + ', '.join(show(b) for b in t.items) + ')'
    if k == 'flex':
        return 'the figure %d' % t.n
    return k


CODE_WORDS = {
    'rounding': {'down', 'up', 'floor', 'ceil', 'half-up', 'half-down', 'half-even'},
    'daycount': {'act365'},     # L §2.2, §3.3: the review removed the other day counts
    'roll': {'following', 'preceding', 'modified_following', 'unadjusted'},
    'compounding': {'simple', 'effective', 'discount', 'nominal'},
}


def code_kind_of(word):
    for k, ws in CODE_WORDS.items():
        if word in ws:
            return k
    return None
