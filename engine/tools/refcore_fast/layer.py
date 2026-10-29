"""The memory layer: refcore's lookups remembered, from outside (session 4.5e).

refcore (engine/tools/refcore) is the obviously right interpreter and is never edited. This module wraps four of its
methods, like a cache in front of a database; it knows nothing about Tally's rules. Each memo is kept per Engine (one
per ask) or per model object, and its key holds everything the lookup reads (tmp/session4_5/e/plan.md says why each
part is there):

- Engine.candidates: an index of the rows whose pattern matches the keys (row_matches and unstated_ok, which read only
  the row, the keys and the model); refcore's own candidates then runs on those rows, in order.
- Engine.read_input: a plain read (no pins, no tracing, no expression row, no prices) remembered with its notes: what
  it wrote to the collector (register entries, rows used, ...) is recorded in a child Coll and merged again on reuse,
  and self._K is set again where candidates set it.
- InputInfo.lawless and IndexInfo.holding: pure scans of static lists.

verify(N): one in N remembered answers per memo is recomputed the slow way (the layer bypassed) and compared, answer
and notes; a difference stops the run (exit 70).
"""
import datetime
import os
import sys
from fractions import Fraction

from refcore import evaluate, model
from refcore.evaluate import Coll

E = evaluate.Engine
ORIG = {}
VERIFY = [0]          # N of --verify N, 0 off
PLANT = [False]       # --plant: d left out of read_input's key (the self-test's planted omission)
SLOW = [False]        # inside a verify recomputation: every wrapper calls the original
COUNT = {}
NOK = object()
ATOMS = (type(None), bool, int, float, str, bytes, Fraction, datetime.date)


def check_due(name):
    n = COUNT[name] = COUNT.get(name, 0) + 1
    return VERIFY[0] and n % VERIFY[0] == 0


def stop(what, a, b):
    sys.stderr.write('refcore_fast: VERIFY MISMATCH in %s\n  remembered: %r\n  recomputed: %r\n' % (what, a, b))
    sys.stderr.flush()
    os._exit(70)


def same(a, b, seen=None):
    """Deep structural equality: same types, same atoms, containers in the same order, objects slot by slot."""
    if a is b:
        return True
    if type(a) is not type(b):
        return False
    if isinstance(a, ATOMS):
        return a == b
    if isinstance(a, (set, frozenset)):
        return a == b
    seen = set() if seen is None else seen
    if (id(a), id(b)) in seen:
        return True
    seen.add((id(a), id(b)))
    if isinstance(a, (list, tuple)):
        return len(a) == len(b) and all(same(x, y, seen) for x, y in zip(a, b))
    if isinstance(a, dict):
        return list(a) == list(b) and all(same(a[k], b[k], seen) for k in a)
    names = [s for c in type(a).__mro__ for s in getattr(c, '__slots__', ())]
    if hasattr(a, '__dict__'):
        names += sorted(set(vars(a)) | set(vars(b)))
    if not names:
        return a == b
    return all(same(getattr(a, n, NOK), getattr(b, n, NOK), seen) for n in names)


def state(eng):
    st = eng.__dict__.get('_fast')
    if st is None:
        m = eng.m
        st = eng._fast = {'rows': {}, 'reads': {}, 'ok': {}, 'cands': 0,
                          'model_ok': not getattr(m, 'same_rows', None) and getattr(m, 'joint', None) is None}
    return st


# -- the row index ----------------------------------------------------------------------------------------------------

def candidates(self, rows, keys, d, ctx, info, instant=None, last_before=None):
    orig = ORIG['candidates']
    st = state(self)
    st['cands'] += 1
    if SLOW[0]:
        return orig(self, rows, keys, d, ctx, info, instant, last_before)
    try:
        k = (id(rows), len(rows), id(info), tuple(map(model._kv, keys)), self.ask_K, model._WSET[0])
        e = st['rows'].get(k)
    except TypeError:
        return orig(self, rows, keys, d, ctx, info, instant, last_before)
    if e is None or e[0] is not rows:
        try:
            sub = [r for r in rows if self.row_matches(r, keys, info) and self.unstated_ok(r, keys, info)]
        except Exception:
            # a filter test that raises: the original meets it in its own order
            return orig(self, rows, keys, d, ctx, info, instant, last_before)
        st['rows'][k] = e = (rows, info, sub)
    elif check_due('candidates'):
        a, b = run_both(lambda: orig(self, e[2], keys, d, ctx, info, instant, last_before),
                        lambda: orig(self, rows, keys, d, ctx, info, instant, last_before))
        if not same(a, b):
            stop('candidates of %s' % info.id, a, b)
        if a[0] == 'raise':
            raise a[1]
        return a[1]
    return orig(self, e[2], keys, d, ctx, info, instant, last_before)


def run_both(fa, fb):
    """Each run's ('value', v) or ('raise', exception): the indexed run and the full one."""
    out = []
    for f in (fa, fb):
        SLOW[0], was = True, SLOW[0]
        try:
            out.append(('value', f()))
        except Exception as x:
            out.append(('raise', x))
        finally:
            SLOW[0] = was
    return out


# -- the read memo ----------------------------------------------------------------------------------------------------

def plain(st, info):
    """An input whose read evaluates no expression: no prices, not a reading, no node or sexpr row."""
    e = st['ok'].get(id(info))
    if e is None:
        ok = info.prices is None and not info.reading and not any(
            isinstance(r.val, tuple) and r.val and r.val[0] in ('node', 'sexpr') for r in info.rows)
        e = st['ok'][id(info)] = (info, ok, len(info.rows))
    return e[1] and e[2] == len(info.rows)


def recorded(self, st, fn):
    """fn() with a fresh collector, merged afterwards into the current one as if fn had written there; returns
    (value, coll, whether candidates ran); an exception passes through after the merge."""
    c = Coll()
    n0 = st['cands']
    self.stack.append(c)
    try:
        v = fn()
    finally:
        self.stack.pop()
        merge(self.cur, c)
    return v, c, st['cands'] != n0


def merge(into, c):
    into.merge(c)
    into.missing.extend(c.missing)
    if c.trace is not None:
        into.trace = c.trace


def notes(c):
    return (c.reg, c.rows, c.axes, c.missing, c.trace, c.each)


def read_input(self, info, keys, ctx, calendar=False, probe=False):
    orig = ORIG['read_input']
    st = state(self)
    if SLOW[0] or self.tracing or self.each_met is not None or ctx.pins or not st['model_ok'] or not plain(st, info):
        return orig(self, info, keys, ctx, calendar, probe)
    d, period = self.time_point(info, keys, ctx)
    try:
        k = (id(info), tuple(map(model._kv, keys)), None if PLANT[0] else d, ctx.cut,
             getattr(ctx.cut, 'windows', None), ctx.axes, ctx.windows, ctx.wording, ctx.path, ctx.world, ctx.unc,
             ctx.rnd, model._WSET[0], self.ask_K, calendar, probe)
        e = st['reads'].get(k)
    except TypeError:
        return orig(self, info, keys, ctx, calendar, probe)
    if e is None:
        v, c, ran = recorded(self, st, lambda: orig(self, info, keys, ctx, calendar, probe))
        if not c.missing and c.trace is None:
            st['reads'][k] = (info, v, c, ran)
        return v
    if check_due('read_input'):
        SLOW[0] = True
        try:
            v, c, ran = recorded(self, st, lambda: orig(self, info, keys, ctx, calendar, probe))
        finally:
            SLOW[0] = False
        if not same((e[1], notes(e[2]), e[3]), (v, notes(c), ran)):
            stop('read_input of %s' % self.origin(info, keys), (e[1], notes(e[2]), e[3]), (v, notes(c), ran))
        return v
    merge(self.cur, e[2])
    if e[3]:
        self._K = ctx.cut
    return e[1]


# -- pure scans of static lists ---------------------------------------------------------------------------------------

def pure(cls, name, key):
    """Remember cls.name(self, *a) by key(self, *a) (None: not remembered); the entry holds self, so its id is not
    reused."""
    orig = ORIG[name] = getattr(cls, name)
    memo = {}

    def wrapped(self, *a):
        if SLOW[0]:
            return orig(self, *a)
        try:
            k = key(self, *a)
            e = memo.get(k) if k is not None else None
        except TypeError:
            k = None
        if k is None:
            return orig(self, *a)
        if e is None or e[0] is not self:
            e = memo[k] = (self, orig(self, *a))
        elif check_due(name):
            SLOW[0] = True
            try:
                v = orig(self, *a)
            finally:
                SLOW[0] = False
            if not same(e[1], v):
                stop('%s.%s' % (cls.__name__, name), e[1], v)
        return e[1]
    setattr(cls, name, wrapped)


def install(verify=0, plant=False):
    VERIFY[0], PLANT[0] = verify, plant
    for name, fn in (('candidates', candidates), ('read_input', read_input)):
        ORIG[name] = getattr(E, name)
        setattr(E, name, fn)
    pure(model.InputInfo, 'lawless', lambda s: (id(s), len(s.rows)))
    pure(model.IndexInfo, 'holding', lambda s, d: (id(s), d) if type(d) in (datetime.date, model.WCut) else None)
