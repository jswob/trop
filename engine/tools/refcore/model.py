"""The reality: what the loader builds (E §3.1 steps 1, 2, 4, 9).

Kinds and things with lifespans and memberships, indexes and axes, versions
and their rows (inputs' rows, givens, band tables, rule versions), the
namespace (top-level ids; GD-1: no regimes).
"""
import datetime
from fractions import Fraction

from . import units as U
from . import ty as T
from .dates import (Period, day_period, month_period, year_period, quarter_period, add_days, add_months,
                    ONE_DAY)
from .errors import LoadError, Unspecified, NotYet
from .node import Node
from .values import (Num, Int, Money, Text, Key, Thing, Code, Rec, Lst, Iv, Cal, Hole, NONE, NEVER, CURRENCIES)


# ---------------------------------------------------------------------------
# spans

class Span:
    __slots__ = ('start', 'end', 'always', 'end_q')

    def __init__(self, start, end, always=False, end_q=None):
        self.start = start   # first day, or None (no start)
        self.end = end       # first day after, or None (open end)
        self.always = always
        self.end_q = end_q   # a lifespan whose end is a stated hole (L §5.7, round 18): its question; end is None

    def contains(self, d):
        if self.always:
            return True
        return (self.start is None or self.start <= d) and (self.end is None or d < self.end)

    def overlaps(self, o):
        if self.always or o.always:
            return True
        a = max(self.start or datetime.date.min, o.start or datetime.date.min)
        b = min(self.end or datetime.date.max, o.end or datetime.date.max)
        return a < b

    def text(self):
        if self.always:
            return 'always'
        a = self.start.isoformat() if self.start else ''
        b = (self.end - ONE_DAY).isoformat() if self.end else ''
        if self.end_q is not None:
            b = '?"%s"' % self.end_q
        return '%s..%s' % (a, b)

    def __repr__(self):
        return 'Span(%s)' % self.text()


ALWAYS = Span(None, None, True)
EVERY = Span(None, None)


def lit_period(node, loc):
    """A literal date or period (in a span, a lifespan, a version's dates)."""
    if node is None:
        return None
    if node.k == 'date':
        return day_period(node.v)
    if node.k == 'month':
        return month_period(*node.v)
    if node.k == 'quarter':
        return quarter_period(*node.v)
    if node.k == 'num' and node.isint and node.unit is None and len(node.text.lstrip('-')) == 4:
        return year_period(int(node.v))
    raise LoadError('LOAD-PARSE', 'cannot parse: expected a date or a period, got %s' % _lit_text(node), loc)


def _lit_text(node):
    return node.get('text') or str(node.get('v') or node.get('id') or node.k)


def lspan_to_span(ls, loc):
    if ls == 'always':
        return ALWAYS
    a = lit_period(ls.a, loc) if ls.a is not None else None
    b = lit_period(ls.b, loc) if ls.b is not None else None
    if not ls.dots:
        return Span(a.start, a.end)
    if ls.get('bhole') is not None:
        # a lifespan's end not yet known: a stated hole (L §5.7, HOLE-STATED; round 18 C1-8)
        return Span(a.start if a else None, None, end_q=ls.bhole)
    return Span(a.start if a else None, b.end if b else None)


# The world being evaluated (L §6.4): the scenarios whose versions are seen, the chain from the ask's world down to
# the base; empty in the base.  One evaluation runs at a time; the asker sets it per ask and `as if {scenario s}`
# around its expression.
_WSET = [frozenset()]


def set_world(ws):
    old = _WSET[0]
    _WSET[0] = frozenset(ws)
    return old


def in_world(version):
    """Whether a version is seen in the world being evaluated: the base's always, a scenario's in that scenario and
    those over it."""
    sc = getattr(version, 'scenario', None) if version is not None else None
    return sc is None or sc in _WSET[0]


class WCut(datetime.date):
    """A cut that carries its ask line's `windows each` assignment (L §4.4: "That holds for every window, wherever the
    line meets it: ... a version's known (§4.5)"), so every visibility test at it takes the day the run assigns."""

    def __new__(cls, d, windows):
        o = datetime.date.__new__(cls, d.year, d.month, d.day)
        o.windows = windows
        return o

    def __reduce__(self):
        return (WCut, (datetime.date(self.year, self.month, self.day), self.windows))

    # arithmetic gives a plain day: a day computed from the cut is no cut (W-B-F4: date arithmetic builds its result
    # through the class, which takes (day, windows))
    def _plain(self):
        return datetime.date(self.year, self.month, self.day)

    def __add__(self, other):
        return self._plain() + other

    __radd__ = __add__

    def __sub__(self, other):
        return self._plain() - other

    def replace(self, *args, **kw):
        return self._plain().replace(*args, **kw)


_EACH = [None]   # the windows met by the line windows each is running, {window id: (label, days)} (ask.each_rows)


def set_each(met):
    old = _EACH[0]
    _EACH[0] = met
    return old


def wcut(d, windows):
    """The cut d carrying a windows each assignment, or the plain day."""
    if not isinstance(d, datetime.date) or isinstance(d, datetime.datetime):
        return d
    if isinstance(windows, tuple) and windows and windows[0] == 'each':
        if isinstance(d, WCut) and d.windows == windows:
            return d
        return WCut(d, windows)
    if isinstance(d, WCut):
        return datetime.date(d.year, d.month, d.day)
    return d


def known_window_id(sid):
    """The identity of a known window under windows each (as evaluate.window_id gives it): one per version, or per
    row for a row with its own known."""
    return ('row', 'known %s' % sid, 'known of %s' % sid), 'known of %s' % sid


class WindowUndecided(NotYet):
    """A read of something known in a window that holds the cut, with no windows clause to choose (L §4.5): a read
    turns it into HOLE-IMPRECISE (Engine.ev), a binding over a kind into an unknown member; elsewhere it stays
    NOT-YET."""

    def __init__(self, known, K, what, origin):
        NotYet.__init__(self, '%s known in window(%s, %s), which holds the cut %s, read where no hole is kept' % (
            what, known[0], known[1], K))
        self.known = known
        self.K = K
        self.what = what
        self.origin = origin

    def hole(self):
        lo, hi = self.known
        return Hole([('date within window(%s, %s) not precise enough for the cut %s' % (lo, hi, self.K),
                      self.origin)])


class LifeHole(WindowUndecided):
    """A read of whether a thing is alive on a day after the start of a lifespan whose end is a stated hole (L §5.7,
    round 18 C1-8): a read turns it into the hole (HOLE-STATED, its question), a binding into an unknown member, as
    WindowUndecided."""

    def __init__(self, name, span):
        NotYet.__init__(self, 'the lifespan of %s ends on a day not yet known' % name)
        self.name = name
        self.span = span

    def hole(self):
        return Hole([(self.span.end_q, self.name)])


def visible(known, K, sid=None):
    """Whether something known on `known` (a day, or a window (lo, hi)) is seen at cut K: True, False, or None when K
    lies inside the window (L §4.5) and the line states no windows each; under windows each, the day the run assigns
    to the window (a version's, or a row's own, named by sid) decides (L §4.4)."""
    if isinstance(known, tuple):
        lo, hi = known
        if hi <= K:
            return True
        if lo > K:
            return False
        w = getattr(K, 'windows', None)
        if isinstance(w, tuple) and w and w[0] == 'each' and sid is not None:
            wid, label = known_window_id(sid)
            for x, d in w[1]:
                if x == wid:
                    return d <= K
            days = [lo + datetime.timedelta(days=i) for i in range((hi - lo).days + 1)]
            if _EACH[0] is not None:
                _EACH[0].setdefault(wid, (label, days))
            return days[0] <= K
        return None
    return known <= K


def row_known_sid(r):
    """Whose known a row's is: its own (the row), else its version's."""
    return (r.sid or '') if getattr(r, 'own_known', None) is not None else r.version.id


def known_first(known):
    return known[0] if isinstance(known, tuple) else known


# ---------------------------------------------------------------------------
# infos

class VersionInfo:
    def __init__(self, node, model):
        self.node = node
        self.id = node.id
        self.loc = node.loc
        self.cite = node.cite
        self.governs = None if node.governs is None else lspan_to_span(node.governs, node.loc)
        if node.known.k == 'window':
            self.known = (node.known.a, node.known.b)
        else:
            self.known = node.known.v
        self.enters_stated = node.enters
        self.assumed = node.assumed
        self.moved_by = []   # (VersionInfo mover, Span)
        self.scenario = None

    def span_at(self, K):
        """The governs span in force under cut K (L §4.6: the latest moves known by K)."""
        best = None
        for mover, span in self.moved_by:
            if not in_world(mover):
                continue
            vis = visible(mover.known, K, mover.id)
            if vis is None:
                # L §4.5 (round 12): "This holds for everything the version or row brings: ... its moves lines (the
                # span in force is undecided)": HOLE-IMPRECISE unless windows chooses (writer 5's T-39)
                raise WindowUndecided(mover.known, K, 'the moves line of version %s' % mover.id, mover.id)
            if vis and (best is None or known_first(mover.known) > known_first(best[0].known)):
                best = (mover, span)
        if best is not None:
            return best[1]
        return self.governs

    def enters_at(self, K, own=None):
        """Entry into force (L §5.3): for each row, the later of the version's `known` and the start of the row's
        own time (its own span, lifespan or instant: `own`, a Span or a date), else of the version's governs; `enters
        D` states another day for every row."""
        if self.enters_stated is not None:
            return self.enters_stated
        k = known_first(self.known)
        if own is not None:
            start = own if isinstance(own, datetime.date) else (None if own.always else own.start)
            return k if start is None else max(k, start)
        sp = self.span_at(K)
        if sp is None or sp.always or sp.start is None:
            return k
        return max(k, sp.start)


class KindInfo:
    def __init__(self, kid, spec, parent=None, decl=None, regime=None):
        self.id = kid
        self.spec = spec          # plain place unit event (a subkind takes its root's)
        self.parent = parent
        self.children = []
        self.decl = decl
        self.regime = regime
        self.certainty = None
        self.unit = None
        self.sign = None
        self.priced = None
        self.outside = False
        self.legs = []
        self.attrs = []
        self.prelude = decl is None

    def ancestors(self, model):
        out = []
        k = self
        while k.parent is not None:
            k = model.kinds[k.parent]
            out.append(k.id)
        return out

    def subtree(self, model):
        out = {self.id}
        stack = list(self.children)
        while stack:
            c = stack.pop()
            out.add(c)
            stack.extend(model.kinds[c].children)
        return out


class Entry:
    def __init__(self, kind, span, version, replace, unit, priced, loc):
        self.kind = kind
        self.span = span
        self.version = version
        self.replace = replace
        self.unit = unit
        self.priced = priced
        self.loc = loc


class ThingInfo:
    def __init__(self, name):
        self.name = name
        self.base = None
        self.entries = []      # all entries, any kind

    def entries_of(self, kind):
        return [e for e in self.entries if e.kind == kind]

    def span_in(self, kind, K, owner=None):
        """The lifespan (or membership span) in `kind` under cut K, or None if unseen; owner: from that side's entries
        only (a thing stated same in a joint model, L §6.1)."""
        oth = getattr(self, 'same_others', None)
        if owner is not None and oth and owner in oth:
            # a place stated same whose other side is kept whole (loader.same_other)
            return oth[owner].span_in(kind, K)
        es = self.entries_of(kind)
        if owner is not None:
            es = [e for e in es if entry_owner(e) == owner]
        best = None
        for e in es:
            if not in_world(e.version):
                continue
            vis = visible(e.version.known, K, e.version.id) if e.version is not None else True
            if vis is None:
                # L §4.5 (round 12): "its entries, whose things are unknown members of every binding over their kinds,
                # as keys whose where is a hole (§5.11)" (writer 5's T-32)
                raise WindowUndecided(e.version.known, K, 'the entry of %s in %s' % (self.name, kind), e.version.id)
            if not vis:
                continue
            # a replace line wins, a scenario's over the base's whatever their known dates (L §6.4, §4.6 step 2:
            # "a base row known on the scenario's own known day is replaced too"), then the later known
            if best is None or (e.replace and (best.version is None or not best.replace or
                                               (getattr(e.version, '_layer', 0), known_first(e.version.known)) >=
                                               (getattr(best.version, '_layer', 0),
                                                known_first(best.version.known)))):
                best = e
        return best.span if best is not None else None

    def first_known(self, kind):
        ks = [known_first(e.version.known) if e.version else datetime.date.min for e in self.entries_of(kind)
              if not e.replace]
        return min(ks) if ks else None


def entry_owner(e):
    """The side an entry comes from in a joint model: the owner of its version ('' for the joint model's own), or of
    the place section that states it (loader.same_other)."""
    own = getattr(e, 'own', None)
    if own is not None:
        return own
    v = e.version
    if v is None or getattr(v, 'scenario', None) is not None:
        return ''
    from .joint import owner_deep
    return owner_deep(v.id)


class SameDiffers(WindowUndecided):
    """L §6.1 What same does: "on a day on which the two sides' entries into one kind disagree, whatever depends on
    the thing being alive there is a HOLE-SAME hole (in a binding over that kind, an unknown member, §5.11)"; L §8.1
    HOLE-SAME `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`.  A read turns it into the
    hole (Engine.ev), a binding into an unknown member, as WindowUndecided."""

    def __init__(self, names, kind, when, sides):
        NotYet.__init__(self, 'the entries of %s into %s differ on %s' % (' and '.join(names), kind, when))
        self.names = names
        self.kind = kind
        self.when = when
        self.sides = sides

    def hole(self):
        from .joint import shown
        parts = ' and '.join('%s (%s)' % (sp, shown(row)) for sp, row in self.sides)
        q = '%s are one, but the entry into %s differs on %s: %s' % (' and '.join(shown(n) for n in self.names),
                                                                   shown(self.kind), self.when, parts)
        return Hole([(q, ', '.join(shown(n) for n in self.names))])


class PlaceThing(ThingInfo):
    """A place a `place` section declares per binding tuple (L §5.10): `konto[ana]`.  It lives on the overlap of
    the section's span and its key things' lifespans, under each cut."""

    def __init__(self, name, place, keys, keythings, keyidx):
        ThingInfo.__init__(self, name)
        self.place = place
        self.keys = keys            # key values
        self.keythings = keythings  # ThingInfo of the keys over kinds
        self.keyidx = keyidx        # (IndexInfo, key) of the keys over indexes
        self.base = place.kind
        self.entries.append(Entry(place.kind, place.lives, None, False, place.unit, place.priced,
                                  place.decl.loc if place.decl is not None else None))

    def span_in(self, kind, K, owner=None):
        oth = getattr(self, 'same_others', None)
        if owner is not None and oth and owner in oth:
            return oth[owner].span_in(kind, K)
        if kind != self.place.kind:
            return None
        sp = self.place.lives
        start, end = sp.start, sp.end
        for t in self.keythings:
            ls = t.span_in(t.base, K)
            if ls is None:
                return None
            if ls.always:
                continue
            if ls.start is not None and (start is None or ls.start > start):
                start = ls.start
            if ls.end is not None and (end is None or ls.end < end):
                end = ls.end
        if start is not None and end is not None and end <= start:
            return Span(start, start)
        return Span(start, end)

    def first_known(self, kind):
        ks = [t.first_known(t.base) for t in self.keythings]
        ks = [k for k in ks if k is not None]
        return max(ks) if ks else datetime.date.min


class PlaceInfo:
    """A `place` section (L §5.10): one place, or one per binding tuple."""

    def __init__(self, qid, decl, regime):
        self.id = qid
        self.decl = decl
        self.regime = regime
        self.kind = None
        self.binds = []
        self.unit = decl.unit if decl is not None else None
        self.priced = decl.priced if decl is not None else None
        self.lives = None
        self.things = {}            # tuple of _kv(key) -> PlaceThing (keyed); () -> ThingInfo (unkeyed)
        self.scenario = None


class LegDecl:
    def __init__(self, name, unit, kind):
        self.name = name
        self.unit = unit            # a currency, a unit, a unit kind id, or 'any'
        self.kind = kind            # the event kind that declares it


class MoveInfo:
    """A `move` section (L §5.14)."""

    def __init__(self, qid, decl, regime):
        self.id = qid
        self.decl = decl
        self.regime = regime
        self.kind = None            # event kind id
        self.binds = []
        self.legs = []              # leg nodes in the order written
        self.attrs = []             # (name, expr)
        self.decision = bool(decl.reads)
        self.scenario = None
        self.stratum = 0


class ObsRow:
    """A row of an observed input (L §5.19): a real movement."""

    def __init__(self, inf, key, node, version, own_known, locator, replace, loc):
        self.inf = inf
        self.key = key              # the row's id (text)
        self.node = node            # the value node (a record), or a none node
        self.version = version
        self.own_known = own_known
        self.locator = locator
        self.replace = replace
        self.loc = loc
        self.removed = node.k == 'none'
        self.day = None
        self.kind = None
        self.legs = []              # (name, from Thing, to Thing, amount Num, unit, day)
        self.attrs = {}
        self.fulfils = None         # ('inst', move id, keys) or None
        self.sid = '%s[%s]' % (inf.id, key)

    @property
    def known(self):
        return self.own_known if self.own_known is not None else self.version.known


class IndexInfo:
    def __init__(self, iid, form, decl=None):
        self.id = iid
        self.form = form          # list range open builtin
        self.decl = decl
        self.isint = False
        self.pk = None            # 'day' 'month' 'quarter' 'year' for a period index
        self.keys = []            # ordered key values
        self.added = {}           # key -> VersionInfo that added it (list entries, open keys)
        self.assumed = decl.assumed if decl is not None else None
        self.v = decl.v if decl is not None else None
        self._pos = None
        self.texts = {}           # source text -> key value

    def finish(self):
        if self.form == 'open':
            # an open index has no order a rule can read; binding order and every order no rule reads take the text
            # order of its keys' stable ids `index:key`, by code point (L §2.5, §5.6)
            self.keys.sort(key=lambda k: getattr(k, 'name', None) or str(k))
        self._pos = {}
        for i, k in enumerate(self.keys):
            self._pos[_kv(k)] = i

    def pos(self, k):
        return self._pos.get(_kv(k))

    def has(self, k):
        return _kv(k) in self._pos

    def visible(self, k, K):
        v = self.added.get(_kv(k))
        if v is None:
            return True
        return in_world(v) and bool(visible(v.known, K, v.id))

    def keys_at(self, K):
        return [k for k in self.keys if self.visible(k, K)]

    def holding(self, d):
        """The key whose period holds day d (L §4.6 Reading at a date)."""
        for k in self.keys:
            if isinstance(k, datetime.date):
                if k == d:
                    return k
            elif k.contains(d):
                return k
        return None


def _kv(k):
    if isinstance(k, (Key, Thing)):
        return k.key()
    if type(k).__name__ == 'Mv':
        return ('mv', k.id)
    if isinstance(k, Period):
        return ('p',) + k.key()
    if isinstance(k, datetime.date):
        return ('d', k)
    if type(k).__name__ == 'Num' and k.isint and not k.u and not k.money and k.q.denominator == 1:
        # a key of an integer index (or of a key list of integers) is also an int (L §4.2 Keys as values)
        return ('v', int(k.q))
    if type(k).__name__ == 'Num':
        # a key of a stepped range of quantities (L §5.6) compares by its value
        return ('v', k.key())
    return ('v', k)


class AxisInfo:
    def __init__(self, node):
        self.id = node.id
        self.options = [o for o, _ in node.options]
        self.labels = dict(node.options)
        self.decl = node


class BindInfo:
    """A key bind of a declaration: its variable and domain."""

    def __init__(self, var, dom, node):
        self.var = var
        self.dom = dom            # ('index', id) ('kind', id) ('axis', id) ('list', [keys], ix) ('range', node)
        self.node = node


class Row:
    def __init__(self, target, pattern, nstated, value, assumed, version, own_span, own_known, instant, locator,
                 given, replace, loc, node):
        self.target = target
        self.pattern = pattern    # tuple: ('k', keyvalue) | ('*',) | ('r', a, b)
        self.nstated = nstated
        self.value = value        # parsed value node
        self.assumed = assumed
        self.version = version
        self.own_span = own_span
        self.own_known = own_known
        self.instant = instant    # (day, pos) for a reading
        self.locator = locator
        self.given = given
        self.replace = replace
        self.loc = loc
        self.node = node
        self.cache = {}
        self.sid = None

    @property
    def known(self):
        return self.own_known if self.own_known is not None else self.version.known

    def span(self, K):
        if self.own_span is not None:
            return self.own_span
        return self.version.span_at(K)

    def keys_text(self):
        if not self.pattern:
            return ''
        parts = []
        for p in self.pattern:
            if p[0] == '*':
                parts.append('*')
            elif p[0] == 'r':
                parts.append('%s..%s' % (key_text(p[1]), key_text(p[2])))
            else:
                parts.append(key_text(p[1]))
        while parts and parts[-1] == '*':
            parts.pop()
        return '[%s]' % ', '.join(parts) if parts else ''


def key_text(k):
    from .values import fmt
    if isinstance(k, int) and not isinstance(k, bool):
        return str(k)
    return fmt(k)


def row_timeless(r):
    """A row whose span is `always` under every cut (its own, or its version's with no moves)."""
    if r.instant is not None:
        return False
    if r.own_span is not None:
        return r.own_span.always
    v = r.version
    return v.governs is not None and v.governs.always and not v.moved_by


class InputInfo:
    def timeless(self):
        """Every row of the input governs always: its value doesn't change over time."""
        return bool(self.rows) and not self.reading and not self.observed and all(row_timeless(r) for r in self.rows)

    def lawless(self):
        """Not law-dated (L §4.3 rule 1: law-dated is "one with a row or version whose span isn't always"): every row
        governs always, or it has no row at all (a later version that gives it a dated row makes its readers state
        `at`, rule 2). A reading is read at an instant, so it is law-dated."""
        return not self.reading and not self.observed and all(row_timeless(r) for r in self.rows)

    def undated_read(self):
        """Read without a law date: lawless, or (L §4.3 rule 1, round 14) "an input keyed by one period index, which
        is read at its key's period, §4.6 step 1"."""
        if self.lawless():
            return True
        return not self.reading and not self.observed and getattr(self, 'time_pos', None) not in (None, 'multi')

    def __init__(self, qid, decl, regime):
        self.id = qid
        self.decl = decl
        self.regime = regime
        self.binds = []
        self.type = None
        self.reading = decl.reading if decl is not None else False
        self.observed = decl.observed if decl is not None else False
        self.question = decl.question if decl is not None else None
        self.rows = []
        self.time_pos = None
        self.builtin = decl is None
        self.prices = None
        self.prices_by = None


class RuleVer:
    def __init__(self, label, node, version, v, span, replace, loc):
        self.label = label
        self.node = node
        self.version = version    # the version item `replace rule` came from, or None
        self.v = v                # VersionInfo of `v`, or None
        self.span = span          # own governs, or None
        self.replace = replace
        self.loc = loc

    def known(self):
        if self.version is not None:
            return self.version.known
        if self.v is not None:
            return self.v.known
        return None


class DefInfo:
    def __init__(self, qid, decl, regime):
        self.id = qid
        self.decl = decl
        self.regime = regime
        self.binds = []
        self.type = None
        self.at = decl.at
        self.rules = {}           # label -> [RuleVer]
        self.labels = []
        self.givens = []
        self.time_pos = None


class Table:
    def __init__(self, band, version, key, loc):
        self.band = band
        self.version = version
        self.key = key
        self.rows = []            # BandRow
        self.loc = loc

    @property
    def replace(self):
        return bool(self.rows) and all(r.replace for r in self.rows)


class BandRow:
    def __init__(self, lo, lo_closed, hi, hi_closed, value, locator, replace, loc, node):
        self.lo = lo
        self.lo_closed = lo_closed
        self.hi = hi
        self.hi_closed = hi_closed
        self.value = value
        self.locator = locator
        self.replace = replace
        self.loc = loc
        self.node = node

    def holds(self, x):
        if self.lo is not None:
            if x < self.lo or (x == self.lo and not self.lo_closed):
                return False
        if self.hi is not None:
            if x > self.hi or (x == self.hi and not self.hi_closed):
                return False
        return True

    def same_interval(self, o):
        return (self.lo == o.lo and self.hi == o.hi and self.lo_closed == o.lo_closed
                and self.hi_closed == o.hi_closed)


class BandInfo:
    def __init__(self, qid, decl, regime):
        self.id = qid
        self.decl = decl
        self.regime = regime
        self.binds = []
        self.argtype = None
        self.type = None
        self.tables = []


PRELUDE_PLACE = {
    'accounts': ('one', 'signed', False), 'cash': ('one', 'signed', False), 'holdings': ('one', 'min0', False),
    'assets': ('one', 'min0', False), 'debts': ('one', 'max0', False), 'claims': ('one', 'min0', False),
    'outside': ('any', 'signed', True)}
BUILTIN_INPUTS = ('opening', 'stated', 'account_place')
BUILTIN_INDEXES = ('days', 'months', 'years', 'bank_account')
PRELUDE_KINDS = ('places', 'accounts', 'cash', 'holdings', 'assets', 'debts', 'claims', 'outside', 'currencies',
                 'events', 'transfers', 'adjustments', 'pads', 'parties')
BUILTIN_IDS = set(BUILTIN_INPUTS + BUILTIN_INDEXES + PRELUDE_KINDS + ('cut',))


class Reality:
    def __init__(self):
        self.header = None
        self.names = {}           # id -> (kind, info)
        self.sources = {}
        self.versions = {}
        self.units = set()        # declared quantity units
        self.unit_kind_keys = {}  # key -> unit kind
        self.kinds = {}
        self.things = {}
        self.indexes = {}
        self.axes = {}
        self.inputs = {}
        self.bands = {}
        self.defs = {}
        self.places = {}
        self.fills = {}
        self.moves = {}
        self.asks = []
        self.ask_by_id = {}
        self.warnings = []
        self.scenarios = []
        self.sames = []

    # -- names -----------------------------------------------------------
    def lookup(self, name, _scope=None):
        """(kind, info) of a bare name in the one namespace (L §2.5). The second argument is a scope's or an
        info's `regime` field, always None since GD-1 removed `regime` (round 19), and is ignored."""
        hit = self.names.get(name)
        if hit is None:
            return None
        return hit

    def _info(self, kind, qid):
        return {'input': self.inputs, 'def': self.defs, 'band': self.bands, 'kind': self.kinds,
                'place': self.places, 'fill': self.fills, 'move': self.moves}[kind][qid]

    def qualified(self, prefix, item):
        """A qualified name `prefix.item` (L §2.5): only a member's items are read so (§6.1), and a joint model's
        loading reads those itself (`joint_members`), so here it names no item. GD-1 (round 19): regimes, whose
        items were `R.x`, are gone, so `R.x` is an unknown id (LOAD-NAME)."""
        return None

    # -- things ------------------------------------------------------------
    def thing_in(self, name, kind):
        """Is the thing `name` (ever) of kind `kind` or a subkind of it?"""
        t = self.things.get(name)
        if t is None:
            return False
        sub = self.kinds[kind].subtree(self)
        return any(e.kind in sub for e in t.entries)

    def kind_order_key(self, thing, kind, K):
        """A thing's place in binding order and in every order no rule reads: the text order of its stable id
        `kind:key`, compared by code point (L §2.5)."""
        return ('%s:%s' % (thing.base, thing.name),)

    def member_span(self, thing, kind, K):
        """The span over which thing is of `kind` (or a subkind), under cut K (one span:
        the base lifespan, or a membership)."""
        sub = self.kinds[kind].subtree(self)
        spans = []
        for kd in set(e.kind for e in thing.entries if e.kind in sub):
            sp = thing.span_in(kd, K)
            if sp is not None:
                spans.append(sp)
        if not spans:
            return None
        if len(spans) == 1:
            return spans[0]
        # several memberships: their hull is enough for ordering
        st = [s.start for s in spans]
        en = [s.end for s in spans]
        return Span(None if None in st else min(st), None if None in en else max(en))

    def alive_on(self, thing, kind, d, K, owner=None):
        return self.alive_test(thing, kind, K, owner, lambda sp: sp.contains(d), d)

    def alive_over(self, thing, kind, span, K, owner=None):
        return self.alive_test(thing, kind, K, owner, lambda sp: sp.overlaps(span), span.text())

    def alive_test(self, thing, kind, K, owner, test, when):
        """Whether thing is of kind (or a subkind) on a day or over a span.  A thing stated same in a joint model has
        each side's entries (L §6.1): a binding inside a member (owner) reads that member's own; the joint model's own
        read takes them together, and where the sides disagree it is the HOLE-SAME of SameDiffers."""
        sub = self.kinds[kind].subtree(self)
        sides = self.same_sides(thing, sub)
        if sides is None:
            return self._alive(thing, sub, K, None, test)
        if owner is not None:
            from .joint import owns
            return any(self._alive(thing, sub, K, o, test) for o in sides if owns(owner, o))
        # "an entry only one side states stands" (L §6.1): a side whose entries the cut doesn't see yet states none
        seen = [o for o in sides if any(sp is not None for kd, sp in self.side_spans_in(thing, sub, K, o))]
        if len(seen) < 2:
            return self._alive(thing, sub, K, seen[0] if seen else None, test)
        res = [(o, self._alive(thing, sub, K, o, test)) for o in seen]
        if len(set(r for o, r in res)) == 1:
            return res[0][1]
        J = self.joint
        names = dict(J.things.get(thing.name, ())) if J is not None else {}
        if J is not None and not names:
            # a place: its sides as the same line writes them (J.groups)
            names = dict((o, x) for o, x, out in J.groups.get(thing.name, ()) if not out)
        out = []
        for o in seen:
            t_o, es = self.side_entries(thing, sub, o)
            sps = self.side_spans_in(thing, sub, K, o)
            txt = ', '.join(sp.text() for kd, sp in sps if sp is not None) or 'no entry'
            # the version that states the side's entry (its last, by known, when it has several); a place section's
            # lifespan, the place
            row = es[-1].version.id if es and es[-1].version is not None else \
                names.get(o, (o + '.' if o else '') + thing.name.split('\u00b7')[-1])
            out.append((txt, row))
        raise SameDiffers([names.get(o, (o + '.' if o else '') + thing.name.split('\u00b7')[-1]) for o in seen],
                          kind, when, out)

    def side_entries(self, thing, sub, o):
        """(the thing that holds side o's entries, those entries into kinds of sub)."""
        oth = getattr(thing, 'same_others', None)
        if oth and o in oth:
            return oth[o], [e for e in oth[o].entries if e.kind in sub]
        return thing, [e for e in thing.entries if e.kind in sub and entry_owner(e) == o]

    def side_spans_in(self, thing, sub, K, o):
        """[(kind, span or None)] of side o's entries into kinds of sub, under cut K."""
        t_o, es = self.side_entries(thing, sub, o)
        return [(kd, thing.span_in(kd, K, o)) for kd in sorted(set(e.kind for e in es))]

    def _alive(self, thing, sub, K, owner, test):
        oth = getattr(thing, 'same_others', None)
        if owner is not None and oth and owner in oth:
            return self._alive(oth[owner], sub, K, None, test)
        unknown = None
        for kd in set(e.kind for e in thing.entries if e.kind in sub):
            sp = thing.span_in(kd, K, owner)
            if sp is not None and test(sp):
                if sp.end_q is not None:
                    # a lifespan whose end is a stated hole: on the days it may be alive the thing is an unknown
                    # member, by the hole's spread (L §5.7, round 18 C1-8; L §8.1 HOLE-STATED)
                    unknown = LifeHole(thing.name, sp)
                    continue
                return True
        if unknown is not None:
            raise unknown
        return False

    def same_sides(self, thing, sub):
        """The sides whose entries into kinds of sub a thing has, when there are two or more (a thing stated same in a
        joint model, L §6.1); None otherwise."""
        if getattr(self, 'joint', None) is None:
            return None
        oth = getattr(thing, 'same_others', None)
        if not self.joint.things and not oth:
            return None
        owners = set(entry_owner(e) for e in thing.entries if e.kind in sub)
        if oth:
            owners |= set(o for o, t in oth.items() if any(e.kind in sub for e in t.entries))
        owners = sorted(owners)
        return owners if len(owners) > 1 else None

    def thing_seen(self, thing, K):
        if isinstance(thing, PlaceThing):
            return all(self.thing_seen(t, K) for t in thing.keythings) and \
                all(ix.visible(k, K) for ix, k in thing.keyidx)
        for e in thing.entries:
            # an entry known in a window holding the cut makes its thing an unknown member (L §4.5): seen, its
            # lifespan undecided
            if e.version is None or (in_world(e.version) and visible(e.version.known, K, e.version.id) is not False):
                return True
        return False

    def kind_unit(self, kid):
        x = self.kinds[kid]
        while x.unit is None and x.parent is not None:
            x = self.kinds[x.parent]
        return x.unit

    def kind_sign(self, kid):
        x = self.kinds[kid]
        while x.sign is None and x.parent is not None:
            x = self.kinds[x.parent]
        return x.sign

    def kind_outside(self, kid):
        x = self.kinds[kid]
        while x is not None:
            if x.outside:
                return True
            x = self.kinds[x.parent] if x.parent is not None else None
        return False

    def kind_priced(self, kid):
        x = self.kinds[kid]
        while x.priced is None and x.parent is not None:
            x = self.kinds[x.parent]
        return x.priced

    def price_of(self, thing):
        """The price a place is valued by (L §5.10 Value, round 10): `priced p` on its entry or `place` section, else
        on its kind (a parent's included); a `price` node (ref, last, within), or None."""
        if thing.base is None:
            return None
        own = None
        for e in thing.entries:
            if e.priced is not None and e.kind == thing.base:
                own = e.priced
        kp = self.kind_priced(thing.base)
        if own is not None and kp is not None:
            # L §5.10 (round 14): "A place's price input is named once: on its kind, or on the place itself (its place
            # section or its entry), never on both (LOAD-PRICE)"
            x = self.kinds[thing.base]
            while x.priced is None and x.parent is not None:
                x = self.kinds[x.parent]
            raise LoadError('LOAD-PRICE', '%s names a price input on its kind %s and on itself' % (thing.name, x.id),
                            getattr(own, 'loc', None))
        return own if own is not None else kp

    def kind_legs(self, kid):
        """The legs of an event kind: its parents' first, then its own (L §5.7)."""
        chain = []
        x = self.kinds[kid]
        while x is not None:
            chain.append(x)
            x = self.kinds[x.parent] if x.parent is not None else None
        out = []
        for k in reversed(chain):
            for name, u in k.legs:
                out.append(LegDecl(name, u, k.id))
        return out

    def kind_attrs(self, kid):
        chain = []
        x = self.kinds[kid]
        while x is not None:
            chain.append(x)
            x = self.kinds[x.parent] if x.parent is not None else None
        out = []
        for k in reversed(chain):
            for a in k.attrs:
                out.append((a[0], getattr(k, 'attr_types', {}).get(a[0])))
        return out

    def is_place_thing(self, thing):
        return thing.base is not None and self.kinds[thing.base].spec == 'place'

    def place_unit(self, thing):
        """The unit of a place: its kind's, or for a kind of unit `one` the one it states; 'any' (L §5.7)."""
        u = self.kind_unit(thing.base)
        for e in thing.entries:
            if e.unit is not None and e.kind == thing.base:
                # a kind of unit one states it per thing; a place section on a kind of unit any that states a
                # unit holds that unit (`place world: outside PLN`, L §5.10)
                return e.unit
        if u == 'one':
            return None
        return u

    def lifespan(self, thing, K):
        if thing.base is None:
            return None
        sides = self.same_sides(thing, set([thing.base]))
        if sides is not None:
            # a thing stated same (L §6.1): its lifespan is the sides' where they agree, else HOLE-SAME; a side the cut
            # doesn't see yet states none ("an entry only one side states stands")
            sps = [sp for sp in (thing.span_in(thing.base, K, o) for o in sides) if sp is not None]
            if len(sps) == 1:
                return sps[0]
            if len(set(sp.text() for sp in sps)) > 1:
                days = sorted(set(x for sp in sps if sp is not None for x in (sp.start, sp.end) if x is not None))
                d = None
                for x in days:
                    if len(set(sp is not None and sp.contains(x) for sp in sps)) > 1:
                        d = x
                        break
                self.alive_test(thing, thing.base, K, None, lambda sp: sp.contains(d), d)
                return None
        return thing.span_in(thing.base, K)
