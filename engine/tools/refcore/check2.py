"""Writer 2's load-time checks, installed into check.Checker.

`to u` and the transcendentals (L §7.3), the rate built-ins (L §3.3), distributions,
random facts and their queries (L §5.16), scores (L §5.15), `prorata` (L §5.13),
`fill` (L §5.12) and `solve` (L §5.18).  Each `m_<name>` function becomes the method
`<name>` of the Checker.
"""
from fractions import Fraction

from . import units as U
from . import ty as T
from .ty import Ty
from .errors import LoadError, Unspecified, NotYet
from .node import walk, Node
from .values import CURRENCIES

CONT = ('normal', 'lognormal', 'exponential', 'gamma', 'uniform')
DISTS = ('normal', 'lognormal', 'exponential', 'gamma', 'uniform', 'poisson', 'binomial', 'negbin')
TRANS_CALLS = ('exp', 'ln', 'sqrt', 'pow', 'periodic', 'factor', 'accrue')
PERIOD_WORDS = ('day', 'month', 'quarter', 'year')


def install(C):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(C, name[2:], fn)


def _c():
    from . import check
    return check


def uerr(msg, loc):
    return LoadError('LOAD-UNIT', msg, loc)


def strip(n):
    while n.k == 'paren':
        n = n.e
    return n


# -- scopes -----------------------------------------------------------------------------------

def m_tc_dist(self, node, sc, expect=None):
    """Type a node whose dist-typed read is a distribution value, not its sample (L §5.16)."""
    sc2 = sc.child()
    sc2.dist_role = True
    return self.tc(node, sc2, expect)


def m_dist_read_type(self, n, kind, info, ty, sc):
    """A read of a random fact: its distribution where a distribution is taken (dist role), else its
    sample on the path (L §5.16 Samples); the reader is random by lifting."""
    role = 'dist' if getattr(sc, 'dist_role', False) else 'sample'
    if getattr(n, '_affx', False):
        # X of an affine form in cdf, pdf or pmf: its distribution is read, typed as its values
        self.rand_edges.setdefault(sc.item, {}).setdefault((kind, info.id), set()).add('dist')
        n._distrole = True
        n._sample = False
        return self.sample_type(kind, info, ty)
    if getattr(sc, 'in_query', False):
        role = 'query'
    self.rand_edges.setdefault(sc.item, {}).setdefault((kind, info.id), set()).add(role)
    if role == 'dist':
        n._distrole = True
        return ty
    n._sample = True
    return self.sample_type(kind, info, ty)


def m_note_read(self, sc, kind, info):
    """Record a plain read for the random cone (L §5.16 lifting)."""
    role = 'query' if getattr(sc, 'in_query', False) else 'value'
    self.rand_edges.setdefault(sc.item, {}).setdefault((kind, info.id), set()).add(role)


def m_sample_type(self, kind, info, ty):
    """The type of a sample: num U, or a key of the index a `discrete` distribution ranges over."""
    ix = self.discrete_index(kind, info)
    if ix is not None:
        return T.key(ix)
    return T.num(ty.u or ())


def m_discrete_index(self, kind, info):
    nodes = []
    if kind == 'def':
        for lab, vers in info.rules.items():
            for rv in vers:
                if rv.node.value is not None:
                    nodes.append(rv.node.value)
    else:
        for r in info.rows:
            v = r.val
            if isinstance(v, tuple) and v and v[0] == 'node':
                nodes.append(v[1])
    ixs = set()
    other = False
    for nd in nodes:
        nd = strip(nd)
        if nd.k == 'to':
            nd = strip(nd.e)
        if nd.k == 'dist' and nd.fn == 'discrete':
            d = nd.bind.dom
            ixs.add(getattr(d, 'id', None))
        else:
            other = True
    if not ixs:
        return None
    if len(ixs) > 1 or other or None in ixs:
        # L §5.16 (round 12): "Its rows and rules give samples of one type, quantities of unit U or keys of one index
        # (each a discrete over it); a mix is refused (LOAD-UNIT)"
        raise LoadError('LOAD-UNIT', 'random fact %s mixes sample types: discrete over %s%s' % (
            info.id, ', '.join(sorted(str(x) for x in ixs)), ' and other distributions' if other else ''),
            getattr(getattr(info, 'decl', None), 'loc', None))
    return list(ixs)[0]


# -- to u ---------------------------------------------------------------------------------------

def m_t_to(self, n, sc, expect):
    inner = strip(n.e)
    inner._under_to = True
    k = inner.k
    if k == 'dist':
        t = self.tc(inner, sc, expect if expect is not None and expect.k == 'dist' else None)
        vu = t.u or ()
    elif k == 'call' and inner.name in TRANS_CALLS:
        t = self.tc(inner, sc)
        vu = t.u if t.k in ('num', 'money') else ()
    elif k == 'distfn' or (k == 'query' and inner.binds is None) or (k == 'crps' and inner.bind is None):
        t = self.tc(inner, sc)
        vu = t.u if t.k in ('num', 'money') and t.u else ()
    else:
        raise uerr('to u on %s: `to` states the precision of a transcendental value (L §7.3)'
                   % _c()._txt(inner), n.loc)
    self.step_check(n.unit, vu, sc, n)
    n._form = k
    if t.k == 'money':
        t = T.num(t.u)
    if k == 'dist' and getattr(n, '_affx', False):
        return T.num(t.u or ())
    return t


def m_step_check(self, u, vu, sc, n):
    """The step of `to u`: positive, of the value's unit; a bare step takes it (L §7.1 precision steps)."""
    sc2 = sc.child()
    sc2.allow_lit = True
    if u.k == 'num':
        if u.unit:
            su, f = U.from_syntax(self.unit_terms(u.unit, u.loc))
            if U.conv(su, vu) is None:
                raise uerr('to %s: the step and the value (%s) have different units' % (u.text, U.fmt(vu) or 'num'),
                           n.loc)
            u._val = u.v * f * U.conv(su, vu)
        else:
            u._val = u.v
        u._u = vu
        u._t = T.num(vu)
        if u._val <= 0:
            raise LoadError('LOAD-DOMAIN', 'to %s: a step that is not positive' % u.text, n.loc)
        return
    t = self.tc(u, sc2)
    tu = t.u if t.k in ('num', 'money') else (() if t.k in ('int', 'flex') else None)
    if tu is None or U.conv(tu, vu) is None:
        raise uerr('to %s: the step (%s) and the value have different units' % (_c()._txt(u), T.show(t)), n.loc)


def m_under_to(self, n, what):
    if not getattr(n, '_under_to', False):
        raise LoadError('LOAD-TO', '%s needs to u' % what, n.loc)


# -- transcendental calls (L §7.3) --------------------------------------------------------------

def m_c_exp(self, n, sc, expect):
    self.args(n, 1)
    t = self.tc(n.args[0], sc)
    if self.numcls(t) not in ('num', 'int', 'flex') or (t.k == 'num' and t.u):
        raise uerr('%s of %s: its argument is a plain number' % (n.name, T.show(t)), n.loc)
    self.under_to(n, n.name)
    return T.num(())


m_c_ln = m_c_exp


def m_c_sqrt(self, n, sc, expect):
    self.args(n, 1)
    t = self.tc(n.args[0], sc)
    c = self.numcls(t)
    if c not in ('num', 'int', 'flex', 'money'):
        raise uerr('sqrt of %s' % T.show(t), n.loc)
    u = t.u if c in ('num', 'money') else ()
    if any(e % 2 for b, e in u):
        raise uerr('sqrt of %s: its unit has an odd power' % T.show(t), n.loc)
    self.under_to(n, 'sqrt')
    return T.num(U.make([(b, e // 2) for b, e in u]))


def m_c_pow(self, n, sc, expect):
    self.args(n, 2)
    xt = self.tc(n.args[0], sc)
    nt = self.tc(n.args[1], sc)
    c = self.numcls(xt)
    if c is None or c == 'rate':
        raise uerr('pow of %s' % T.show(xt), n.loc)
    nc = self.numcls(nt)
    if nc not in ('int', 'flex'):
        if nc == 'num' and not nt.u:
            # a power that may not be an integer: a real (L §7.3); needs to u unless it is an integer at run
            if xt.k in ('num', 'money') and xt.u:
                raise uerr('pow of %s to a power that may not be an integer: its unit would have a fractional '
                           'power' % T.show(xt), n.loc)
            n._realpow = True
            return T.num(())
        raise uerr('pow: the power is %s' % T.show(nt), n.loc)
    if xt.k in ('num', 'money') and xt.u:
        a = strip(n.args[1])
        if not (a.k == 'num' and a.isint) and not (a.k == 'neg' and a.e.k == 'num' and a.e.isint):
            # L §7.1 (round 12): "where x has a unit, n is a literal, so the result's unit is known at load (LOAD-UNIT
            # otherwise)"
            raise uerr('pow of %s to a power that is not a literal' % T.show(xt), n.loc)
        p = int(a.v) if a.k == 'num' else -int(a.e.v)
        return T.num(U.power(xt.u, p))
    return T.num(xt.u if xt.k in ('num', 'money') else ())


# -- rates (L §3.3) ------------------------------------------------------------------------------

def m_period_arg(self, a, n):
    a = strip(a)
    if a.k == 'name' and a.id in PERIOD_WORDS:
        a._period = a.id
        a._t = Ty('periodword')
        return a.id
    raise uerr('%s: the period argument is a period word (day, month, quarter or year), not %s'
               % (n.name, _c()._txt(a)), n.loc)


def m_code_arg(self, a, code, sc, n):
    t = self.tc(a, sc, expect=T.code(code))
    if not (t.k == 'code' and t.code == code):
        raise uerr('%s: %s is not a %s code' % (n.name, _c()._txt(a), code), n.loc)


def m_rate_arg(self, a, sc, n):
    t = self.tc(a, sc)
    if t.k != 'rate':
        raise uerr('%s: %s is %s, not a rate' % (n.name, _c()._txt(a), T.show(t)), n.loc)


def m_rate_lit_period(self, a):
    """The period of a rate argument that load sees: a literal with a period unit (`1 %/month`)."""
    a = strip(a)
    if a.k == 'num' and a.unit:
        u = getattr(a, '_u', None)
        if u is not None:
            return U.period_of(u)
    return None


def m_day_pairing(self, per, P, n):
    """L §3.3: "Any pairing with day other than a daily r, or a daily r over a longer period, is refused (LOAD-UNIT
    where load sees the rate's period, RUN-DOMAIN otherwise)"."""
    if per is None:
        return
    if (P == 'day' or per == 'day') and not (P == 'day' and per == 'day'):
        raise uerr('%s: a %s rate over a %s: a day\'s share of a longer period is a day count (dcf), never a default'
                   % (n.name, per, P), n.loc)


def m_c_periodic(self, n, sc, expect):
    self.args(n, 3)
    self.rate_arg(n.args[0], sc, n)
    P = self.period_arg(n.args[1], n)
    self.code_arg(n.args[2], 'compounding', sc, n)
    self.day_pairing(self.rate_lit_period(n.args[0]), P, n)
    return T.num(())


def m_c_factor(self, n, sc, expect):
    self.args(n, 4)
    self.rate_arg(n.args[0], sc, n)
    t = self.tc(n.args[1], sc)
    if self.numcls(t) not in ('int', 'flex', 'num') or (t.k == 'num' and t.u):
        raise uerr('factor: n is %s, not a plain number' % T.show(t), n.loc)
    P = self.period_arg(n.args[2], n)
    self.code_arg(n.args[3], 'compounding', sc, n)
    per = self.rate_lit_period(n.args[0])
    self.day_pairing(per, P, n)
    if not getattr(n, '_under_to', False) and not self.factor_power_int(n, t, per, P):
        # "exact when its whole power is an integer ...; otherwise it needs to u (§7.3), LOAD-TO where load can't
        # show the power is an integer" (L §3.3)
        raise LoadError('LOAD-TO', 'factor needs to u: load can\'t show its power is an integer', n.loc)
    return T.num(())


def m_factor_power_int(self, n, nt, per, P):
    """Whether load shows factor's whole power is an integer: t·n (effective, discount), m·t·n (nominal m), n
    (simple), with t P's length in r's periods (L §3.3)."""
    nint = nt.k in ('int',) or (nt.k == 'flex' and Fraction(nt.n).denominator == 1) or (
        nt.k == 'key' and self.key_value_kind(nt) == 'int')
    if not nint:
        return False
    cn = strip(n.args[3])
    r = getattr(cn, '_r', None)
    if not (r is not None and r[0] == 'code'):
        return False
    code = r[1]
    if code.word == 'simple':
        return True
    months = {'year': 12, 'quarter': 3, 'month': 1}
    if P == 'day':
        ts = [Fraction(1)]
    elif per is not None:
        ts = [Fraction(months[P], months[per])]
    else:
        ts = [Fraction(months[P], months[x]) for x in ('year', 'quarter', 'month')]
    mult = Fraction(code.n) if code.word == 'nominal' else Fraction(1)
    return all((mult * t).denominator == 1 for t in ts)


def m_c_dcf(self, n, sc, expect):
    self.args(n, 3)
    self.tdate(n.args[0], sc, n, 'a')
    self.tdate(n.args[1], sc, n, 'b')
    self.code_arg(n.args[2], 'daycount', sc, n)
    return T.num(())


def m_c_accrue(self, n, sc, expect):
    self.args(n, 6)
    pt = self.tc(n.args[0], sc)
    c = self.numcls(pt)
    if c not in ('num', 'money', 'int', 'flex'):
        raise uerr('accrue: the principal is %s' % T.show(pt), n.loc)
    self.rate_arg(n.args[1], sc, n)
    self.tdate(n.args[2], sc, n, 'a')
    self.tdate(n.args[3], sc, n, 'b')
    self.code_arg(n.args[4], 'daycount', sc, n)
    self.code_arg(n.args[5], 'compounding', sc, n)
    return T.num(pt.u if c in ('num', 'money') else ())


# -- distributions (L §5.16) --------------------------------------------------------------------

def m_t_dist(self, n, sc, expect):
    fn = n.fn
    want = expect.u if expect is not None and expect.k == 'dist' else None
    if fn == 'discrete':
        sc2 = sc.child()
        sc2.agg_depth += 1
        v = self.bind(n.bind, sc2)
        dk = n.bind._dom[0]
        if dk in ('kind', 'event', 'obs') or (dk == 'index' and self.m.indexes[n.bind._dom[1]].form == 'open'):
            # L §5.16: "its values are the keys of IDX, in index order" (the quantile walks that order); things,
            # movements and an open index's keys have no order a rule can read (L §5.7, §5.6): refused where the
            # types show it (L §8.4, LOAD-UNIT)
            raise uerr('discrete over %s: its quantile walks the keys in index order, and they have none' % (
                'the keys of the open index %s' % n.bind._dom[1] if dk == 'index' else 'the members of %s' %
                n.bind._dom[1]), n.loc)
        tt = self.tc(n.term, sc2)
        if self.numcls(tt) not in ('num', 'int', 'flex') or (tt.k == 'num' and tt.u):
            raise uerr('discrete: the weight is %s, not a plain number' % T.show(tt), n.loc)
        if dk != 'index':
            # L §5.16's table (round 19, GD-24, C4-11): "over a key list or a stepped range it is refused
            # (LOAD-DOMAIN)"; L §8.3 `discrete over a key list or a stepped range`
            raise LoadError('LOAD-DOMAIN', 'discrete over a key list or a stepped range', n.loc)
        n._unit = ()
        return Ty('dist', u=(), ix=n.bind._dom[1])
    arity = {'normal': 2, 'lognormal': 2, 'exponential': 1, 'gamma': 2, 'uniform': 2, 'poisson': 1,
             'binomial': 2, 'negbin': 2}[fn]
    if len(n.args) != arity:
        raise LoadError('LOAD-PARSE', 'cannot parse: %s takes %d parameters' % (fn, arity), n.loc)
    ts = [self.tc(a, sc.child()) for a in n.args]

    def unit_of(t, what):
        c = self.numcls(t)
        if c in ('flex', 'int'):
            return None
        if c in ('num', 'money'):
            return t.u
        raise uerr('%s: %s is %s, not a number' % (fn, what, T.show(t)), n.loc)

    def plain(t, what):
        u = unit_of(t, what)
        if u:
            raise uerr('%s: %s is %s, not a plain number' % (fn, what, T.show(t)), n.loc)
    if fn in ('normal', 'uniform'):
        a, b = unit_of(ts[0], 'the first parameter'), unit_of(ts[1], 'the second parameter')
        us = [x for x in (a, b) if x is not None]
        if us and len(us) == 2 and U.conv(us[0], us[1]) is None:
            raise uerr('%s(%s, %s): the parameters have different units' % (fn, T.show(ts[0]), T.show(ts[1])), n.loc)
        u = us[0] if us else (want or ())
    elif fn == 'lognormal':
        plain(ts[0], 'm')
        plain(ts[1], 's')
        u = want or ()
    elif fn == 'exponential':
        r = unit_of(ts[0], 'the rate')
        u = U.inv(r) if r else (want or ())
    elif fn == 'gamma':
        plain(ts[0], 'the shape')
        s = unit_of(ts[1], 'the scale')
        u = s if s is not None else (want or ())
    else:
        for i, t in enumerate(ts):
            plain(t, 'a parameter')
        u = ()
        if want:
            raise uerr('%s gives counts, not a dist %s' % (fn, U.fmt(want)), n.loc)
    if want is not None and U.conv(u, want) is None:
        raise uerr('%s of %s for a dist %s' % (fn, U.fmt(u) or 'num', U.fmt(want) or 'num'), n.loc)
    n._unit = want if want is not None else u
    if getattr(n, '_affx', False):
        return T.num(n._unit)
    return Ty('dist', u=n._unit)


def m_dist_whole(self, node, loc, where):
    """A continuous distribution without `to`, in a row, a rule or a pin, is refused (LOAD-TO, L §5.16)."""
    x = strip(node)
    if x.k == 'dist' and x.fn in CONT:
        raise LoadError('LOAD-TO', 'continuous distribution %s needs to u (%s)' % (x.fn, where), loc)


def m_dist_affine_syntax(self, node, sc):
    """The argument of cdf, pdf or pmf: a distribution value, a random fact read directly, or an affine
    form a * X + b of one (L §5.16).  Returns the X node, or None.  The coefficients are found at run by
    evaluating the form at X = 0 and X = 1 (it is affine, so that is exact)."""
    m = self.m

    def is_x(e):
        e = strip(e)
        if e.k == 'dist':
            return True
        if e.k == 'to' and strip(e.e).k == 'dist':
            return True
        if e.k in ('name', 'index'):
            obj = e if e.k == 'name' else e.obj
            if obj.k == 'name' and obj.id not in sc.vars:
                hit = m.lookup(obj.id, sc.regime)
                if hit is not None and hit[0] in ('input', 'def') and hit[1].type is not None and \
                        hit[1].type.k == 'dist':
                    return True
        return False

    def contains_x(e):
        return any(is_x(x) for x in walk(e))

    def go(e):
        """The X node of an affine form in exactly one X, or None."""
        e = strip(e)
        if is_x(e):
            return e
        if e.k == 'neg':
            return go(e.e)
        if e.k == 'bin' and e.op in '+-':
            lx, rx = contains_x(e.l), contains_x(e.r)
            if lx and not rx:
                return go(e.l)
            if rx and not lx:
                return go(e.r)
            return None
        if e.k == 'bin' and e.op == '*':
            lx, rx = contains_x(e.l), contains_x(e.r)
            if lx and not rx:
                return go(e.l)
            if rx and not lx:
                return go(e.r)
            return None
        if e.k == 'bin' and e.op == '/':
            if contains_x(e.l) and not contains_x(e.r):
                return go(e.l)
            return None
        return None
    return go(node)


def m_t_distfn(self, n, sc, expect):
    """cdf, pdf, pmf (L §5.16): the transformed function of D for an affine form a * X + b."""
    fn = n.fn
    dn, y = n.args
    x = self.dist_affine_syntax(dn, sc)
    if x is None:
        raise LoadError('LOAD-RANDOM', '%s of a non-affine random value: it takes a distribution, a random fact '
                        'read directly or an affine form of one; the question is a share over paths' % fn, n.loc)
    xt = self.tc_dist(x, sc)
    if xt.k != 'dist':
        raise uerr('%s of %s' % (fn, T.show(xt)), n.loc)
    r = getattr(strip(x), '_r', None)
    if not getattr(xt, 'ix', None) and r is not None and r[0] == 'read':
        hit = self.m.inputs.get(r[2]) if r[1] == 'input' else self.m.defs.get(r[2])
        ix = self.discrete_index(r[1], hit) if hit is not None else None
        if ix is not None:
            xt = Ty('dist', u=(), ix=ix)
    if getattr(xt, 'ix', None):
        if strip(dn) is not strip(x):
            raise uerr('%s of an affine form of a distribution over keys' % fn, n.loc)
        if fn == 'pdf':
            raise uerr('pdf of a discrete distribution', n.loc)
        yt = self.tc(y, sc, expect=T.key(xt.ix))
        self.assign(T.key(xt.ix), y, n.loc, 'the point of %s' % fn)
        n._x = x
        n._keys = xt.ix
        return T.NUM1
    if strip(dn) is strip(x):
        t = xt
        u = xt.u or ()
    else:
        x._affx = True
        t = self.tc(dn, sc)
        x._t = T.num(xt.u or ())
        c = self.numcls(t)
        if c not in ('num', 'money', 'int', 'flex'):
            raise uerr('%s of %s' % (fn, T.show(t)), n.loc)
        u = t.u if c in ('num', 'money') else ()
    yt = self.tc(y, sc, expect=T.num(u))
    if yt.k == 'flex':
        self.flex_to(y, T.num(u))
        y._flexto = T.num(u)
    else:
        yu = self.unit_or(yt, '%s: the point' % fn, n)
        if U.conv(yu, u) is None:
            raise uerr('%s at %s for a value of unit %s' % (fn, T.show(yt), U.fmt(u) or 'num'), n.loc)
    n._x = x
    n._keys = None
    xs = strip(x)
    if xs.k == 'to':
        xs = strip(xs.e)
    if xs.k == 'dist':
        need = {'cdf': ('normal', 'lognormal', 'exponential', 'gamma', 'poisson'),
                'pdf': ('normal', 'lognormal', 'exponential', 'gamma'), 'pmf': ('poisson',)}[fn]
        if xs.fn in need:
            self.under_to(n, '%s of %s' % (fn, xs.fn))
        if fn == 'pdf' and xs.fn in ('poisson', 'binomial', 'negbin', 'discrete'):
            raise uerr('pdf of the discrete %s: a sample has no density' % xs.fn, n.loc)
        if fn == 'pmf' and xs.fn in CONT:
            raise uerr('pmf of the continuous %s' % xs.fn, n.loc)
    if fn == 'pdf':
        self.orient_items.add(sc.item)
        return T.num(U.inv(u))
    return T.NUM1


def m_unit_or(self, t, what, n):
    c = self.numcls(t)
    if c in ('flex', 'int'):
        return ()
    if c in ('num', 'money'):
        return t.u
    raise uerr('%s is %s, not a number' % (what, T.show(t)), n.loc)


# -- queries over random values (L §5.16) ----------------------------------------------------------

def m_t_query_random(self, n, sc, expect):
    a0 = strip(n.arg)
    if a0.k == 'to':
        a0 = strip(a0.e)
    if a0.k == 'dist':
        # quantile(d, q) of a distribution value is its function, in a def as in an ask (L §5.16 The closed list)
        return self.dist_value_query(n, sc)
    sc2 = sc.child()
    sc2.in_query = True
    tt = self.tc(n.arg, sc2)
    if n.where is not None:
        wt = self.tc(n.where, sc2)
        if not self.is_bool(wt):
            raise uerr('where of %s is %s, not bool' % (n.fn, T.show(wt)), n.loc)
    self.queries.append((n, sc.item, sc.where))
    n._qitem = sc.item
    if n.fn == 'share':
        if not self.is_bool(tt):
            raise uerr('share of %s' % T.show(tt), n.loc)
        return T.NUM1
    c = self.numcls(tt)
    if n.fn == 'mean':
        if c is None:
            raise uerr('mean of %s' % T.show(tt), n.loc)
        if c in ('int', 'flex'):
            return T.NUM1
        if c == 'money':
            return T.num(tt.u)
        return tt
    scq = sc.child()
    if sc.where == 'ask':
        scq.allow_lit = True   # the level of a quantile (L §7.1 An ask)
    qt = self.tc(n.q, scq)
    if self.numcls(qt) not in ('num', 'flex', 'int') or (qt.k == 'num' and qt.u):
        raise uerr('the level of quantile is %s' % T.show(qt), n.loc)
    if c is None and tt.k not in ('key',):
        raise uerr('quantile of %s' % T.show(tt), n.loc)
    if c == 'money':
        return T.num(tt.u)
    if c in ('int', 'flex'):
        return T.NUM1
    return tt


def m_dist_value_query(self, n, sc):
    """quantile(d, q) on a distribution value d: "called on a distribution value, cdf(d, x), pdf(d, x), pmf(d, k)
    and quantile(d, q) are these functions, in a def as in an ask, exact or written with to u where
    transcendental" (L §5.16)."""
    if n.fn == 'mean':
        # round 18 (C1-47): "mean(d) of a distribution value is refused (LOAD-RANDOM, A-42), and the table's mean is the
        # one a closed form and random at mean read" (L §5.16 The closed list); a mean is asked of a random fact
        raise LoadError('LOAD-RANDOM', 'mean of a distribution value; ask it of a random fact', n.loc)
    if n.fn != 'quantile':
        # a share of a distribution value is a question over paths (L §5.16 Closed forms: share(a * X + b op y) of a
        # random fact read directly)
        raise LoadError('LOAD-RANDOM', '%s of a distribution value: share takes a random bool' % n.fn, n.loc)
    if n.where is not None:
        raise LoadError('LOAD-RANDOM', 'quantile of a distribution value with where: where restricts a query over '
                        'paths', n.loc)
    dt = self.tc_dist(n.arg, sc)
    if dt.k != 'dist':
        raise uerr('quantile of %s' % T.show(dt), n.loc)
    scq = sc.child()
    if sc.where == 'ask':
        scq.allow_lit = True
    qt = self.tc(n.q, scq)
    if self.numcls(qt) not in ('num', 'flex', 'int') or (qt.k == 'num' and qt.u):
        raise uerr('the level of quantile is %s' % T.show(qt), n.loc)
    n._dfn = True
    d0 = strip(n.arg)
    if d0.k == 'to':
        d0 = strip(d0.e)
    if getattr(dt, 'ix', None):
        return T.key(dt.ix)
    if d0.k == 'dist' and d0.fn in ('normal', 'lognormal', 'exponential', 'gamma'):
        self.under_to(n, 'quantile of %s' % d0.fn)
    return T.num(dt.u or ())


# -- crps (L §5.15) ----------------------------------------------------------------------------------

def m_t_crps(self, n, sc, expect):
    self.orient_items.add(sc.item)
    if n.bind is None:
        e = strip(n.e)
        if e.k == 'to' and strip(e.e).k == 'dist':
            e = strip(e.e)
        if e.k == 'dist':
            if e.fn != 'normal':
                raise uerr('crps of %s: the closed form is for a normal forecast (L §5.15)' % e.fn, n.loc)
            dt = self.tc_dist(strip(n.e), sc)
            self.under_to(n, 'crps of a normal forecast')
            u = dt.u or ()
            n._form = 'normal'
        else:
            sc2 = sc.child()
            sc2.in_query = True
            et = self.tc(n.e, sc2)
            c = self.numcls(et)
            if c not in ('num', 'int', 'flex', 'money'):
                raise uerr('crps of %s' % T.show(et), n.loc)
            u = et.u if c in ('num', 'money') else ()
            n._form = 'paths'
            self.queries.append((n, sc.item, sc.where))
    else:
        sc2 = sc.child()
        sc2.agg_depth += 1
        self.bind(n.bind, sc2)
        et = self.tc(n.e, sc2)
        c = self.numcls(et)
        if c not in ('num', 'int', 'flex', 'money'):
            raise uerr('crps of %s' % T.show(et), n.loc)
        u = et.u if c in ('num', 'money') else ()
        if n.weight is not None:
            wt = self.tc(n.weight, sc2)
            if self.numcls(wt) not in ('num', 'int', 'flex') or (wt.k == 'num' and wt.u):
                raise uerr('crps: the weight is %s, not a plain number' % T.show(wt), n.loc)
            n._form = 'weighted'
        else:
            n._form = 'sample'
    yt = self.tc(n.y, sc, expect=T.num(u))
    if yt.k == 'flex':
        self.flex_to(n.y, T.num(u))
    else:
        yu = self.unit_or(yt, 'crps: the outcome', n)
        if U.conv(yu, u) is None:
            raise uerr('crps: the outcome %s and the forecast (%s) have different units' % (T.show(yt), U.fmt(u)),
                       n.loc)
    return T.num(u)


# -- random cone and closed forms (E §3.1 step 10) ----------------------------------------------------

def m_random_cone(self, with_proj=True):
    """Items that reach a sample of a random fact (L §5.16 lifting).  A dist fact whose distribution reads a
    sample is random too: its distribution changes with the path.

    E §3.1 step 10: "Every item that reaches a `dist` fact in a world is random"; E §7 step 2: on each path "the
    random-dependent plan instances are recomputed; the day walk runs if any balance or decision depends on a sample",
    and "law defs in the random cone are cells of the path's layer". So a move whose instances, day or amounts reach a
    sample is random, the balances of the places its legs book are, and so is every item that reads one of them (a
    balance or low at a day, `flows`, a decision's legs); a decision that reads a random balance is random in turn.
    A place is told by its literal name; a place read or booked through a binding or an expression matches any.
    L §8.3 item 10 (round 19, GD-4): "every item that reaches a dist fact through the item graph, in each world, and
    through every place a random movement books into: a balance or a projection of such a place is in the cone". A
    projection with a random clause of its own reads the amounts after t as the question of L §5.16 ("which reads no
    sample and is the same on every path"), so it is left out; with_proj=False leaves out every projection, the cone
    for an ask line whose own random clause gives each projection it reaches that clause (L §5.18)."""
    m = self.m
    ITEMS = ('input', 'def', 'fill', 'move', 'moveon')
    rand = set()
    rplaces = set()
    rkinds = set()

    def place_hit(tgt):
        if not rplaces:
            return False
        if tgt[0] == 'thing' and 'any' not in rplaces:
            return tgt in rplaces
        return True

    def mark_hit(item):
        if with_proj:
            for tgt in self.proj_nc.get(item, ()):
                if place_hit(tgt):
                    return True
        for what, tgt in self.marks.get(item, ()):
            if what not in ('bal', 'balnow', 'flows', 'legs'):
                continue
            if tgt[0] == 'move':
                if ('move', tgt[1]) in rand:
                    return True
            elif tgt[0] == 'kind':
                if tgt[1] in m.kinds and rkinds & set(m.kinds[tgt[1]].subtree(m)):
                    return True
            elif place_hit(tgt):
                return True
        return False

    def book(qid):
        mi = m.moves.get(qid)
        if mi is None:
            return
        rkinds.add(mi.kind)
        for ln, d in getattr(mi, 'legs', ()):
            for node in (getattr(ln, 'frm', None), getattr(ln, 'to', None)):
                if node is None:
                    continue
                tgt = self.place_target(node)
                rplaces.add(tgt if tgt[0] == 'thing' else 'any')

    def add(r):
        rand.add(r)
        if r[0] in ('move', 'moveon'):
            book(r[1])

    changed = True
    while changed:
        changed = False
        for r, ts in self.rand_edges.items():
            if r in rand or r[0] not in ITEMS:
                continue
            for t, roles in ts.items():
                roles = roles - {'query'}
                if not roles:
                    continue
                if 'sample' in roles or t in rand:
                    add(r)
                    changed = True
                    break
        # plain edges: anything that reads a random item
        for r, ts in self.edges.items():
            if r in rand or r[0] not in ITEMS:
                continue
            for t, rels in ts.items():
                if t in rand and (rels - {'pin', 'query'}):
                    if any(roles == {'query'} for tt, roles in self.rand_edges.get(r, {}).items() if tt == t):
                        continue
                    add(r)
                    changed = True
                    break
        # running reads of the places random moves book, and of random moves
        if rplaces or rkinds:
            for r in list(self.marks):
                if r in rand or r[0] not in ITEMS:
                    continue
                if mark_hit(r):
                    add(r)
                    changed = True
    if not with_proj:
        return rand, rplaces, rkinds
    self.random_items = rand
    self.random_places = (set(rplaces), set(rkinds))
    m.random_items = set(x[1] for x in rand if x[0] == 'def') | set(x[1] for x in rand if x[0] == 'input')
    m.random_fills = set(x[1] for x in rand if x[0] == 'fill')
    m.random_moves = set(x[1] for x in rand if x[0] in ('move', 'moveon'))
    return rand


def m_line_random(self, e, a):
    """E §3.1 step 10: an ask line's value is random when it reaches the cone outside a query; L §8.3 item 10 (round 19,
    GD-4): its own balance or projection of a place a random movement books into is in the cone (a projection with a
    random clause, its own or the line's, reads the amounts after t by it, L §5.16, §5.18). snap14 found these at the
    read (contradiction 36)."""
    saved = self.random_items
    if a.random is not None:
        if getattr(self, '_cone_base', None) is None:
            self._cone_base = self.random_cone(with_proj=False)
        rand, rplaces, rkinds = self._cone_base
        self.random_items = rand
    else:
        rplaces, rkinds = self.random_places
    try:
        if self.is_random(e):
            return True
    finally:
        self.random_items = saved
    m = self.m
    rand = self.random_items if a.random is None else self._cone_base[0]

    def hit(tgt):
        if tgt[0] == 'move':
            return ('move', tgt[1]) in rand or ('moveon', tgt[1]) in rand
        if tgt[0] == 'kind':
            return tgt[1] in m.kinds and bool(rkinds & set(m.kinds[tgt[1]].subtree(m)))
        if not rplaces:
            return False
        if tgt[0] == 'thing' and 'any' not in rplaces:
            return tgt in rplaces
        return True
    stack = [e]
    while stack:
        x = stack.pop()
        if isinstance(x, (list, tuple)):
            stack.extend(x)
            continue
        if not isinstance(x, Node):
            continue
        if x.k == 'query' and x.binds is None:
            continue
        if x.k == 'crps' and getattr(x, '_form', None) == 'paths':
            continue
        ct = getattr(x, '_ctgt', None)
        if ct is not None:
            what, tgt = ct
            if what in ('bal', 'balnow', 'flows') or (what == 'proj' and a.random is None):
                if hit(tgt):
                    return True
        for k2, b in x.__dict__.items():
            if k2 in ('loc',) or k2.startswith('_'):
                continue
            if isinstance(b, (Node, list, tuple)):
                stack.append(b)
    return False


def m_check_draw_slots(self):
    """L §5.16 A random number of items: "a bounded index of slots, read by a range that ends at the draw ... A draw
    past the last slot reads a key outside the slots' index and is refused on that path (RUN-KEY, §4.2), and at load
    where load sees the draw's support exceed the index (LOAD-RANDOM)"; L §8.3 `the draw <x> can exceed the last slot
    of <index>`. Load sees the support of a distribution written with literal bounds, or of one with none."""
    m = self.m
    done = set()
    for kn, hi, ix in self.draw_keys:
        if not self.is_random(hi):
            continue
        txt = _c()._txt(hi)
        kn._draw = (hi, txt, ix)
        if (id(hi), ix) in done:
            continue
        done.add((id(hi), ix))
        info = m.indexes.get(ix)
        if info is None or not info.isint or not info.keys:
            continue
        last = max(int(k) for k in info.keys)
        r = getattr(hi, '_r', None)
        vals = []
        if r is not None and r[1] == 'input' and r[2] in m.inputs:
            vals = [row.value for row in m.inputs[r[2]].rows]
        elif r is not None and r[1] == 'def' and r[2] in m.defs:
            vals = [rv.node.value for vers in m.defs[r[2]].rules.values() for rv in vers]
        for v in vals:
            sup = dist_support_max(v)
            if sup is not None and (sup == 'inf' or sup > last):
                raise LoadError('LOAD-RANDOM', 'the draw %s can exceed the last slot of %s' % (txt, ix), kn.loc)


def dist_support_max(v):
    """The largest value a distribution written in a row or rule can draw, where the text shows it: 'inf' for one
    unbounded above, a number for literal bounds, None where it rests on facts."""
    if v is None or getattr(v, 'k', None) not in ('dist', 'call'):
        return None
    name = getattr(v, 'fn', None) or getattr(v, 'name', None)
    args = getattr(v, 'args', None) or []
    if name in ('poisson', 'negbin', 'normal', 'lognormal', 'gamma', 'exponential'):
        return 'inf'
    if name == 'binomial' and args and args[0].k == 'num':
        return Fraction(args[0].v)
    if name == 'uniform' and len(args) > 1 and args[1].k == 'num':
        return Fraction(args[1].v)
    return None


def m_is_random(self, e):
    """Whether an expression's value changes with the path: a sample read, a read of a random item, outside
    the queries and path scores that take it over the paths."""
    stack = [e]
    while stack:
        x = stack.pop()
        if x is None:
            continue
        if isinstance(x, (list, tuple)):
            stack.extend(x)
            continue
        if not isinstance(x, Node):
            continue
        if x.k == 'query' and x.binds is None:
            continue
        if x.k == 'crps' and getattr(x, '_form', None) == 'paths':
            continue
        if getattr(x, '_sample', False):
            return True
        r = getattr(x, '_r', None)
        if r is not None and isinstance(r, tuple) and r and r[0] == 'read' and (r[1], r[2]) in self.random_items:
            return True
        if r is not None and isinstance(r, tuple) and r and r[0] == 'fillread' and ('fill', r[1]) in \
                self.random_items:
            return True
        for a, b in x.__dict__.items():
            if a in ('loc',) or a.startswith('_'):
                continue
            if isinstance(b, Node) or isinstance(b, (list, tuple)):
                stack.append(b)
    return False


def m_affine(self, e):
    """The random facts read directly in an affine form (sum of known multiples of them plus a known
    constant), as a list of their read nodes; None when e isn't one (L §5.16 Closed forms)."""
    def is_fact(x):
        x = strip(x)
        r = getattr(x, '_r', None)
        return getattr(x, '_sample', False) and r is not None and r[0] == 'read'

    def go(x):
        x = strip(x)
        if is_fact(x):
            return [x]
        if not self.is_random(x):
            return []
        if x.k == 'neg':
            return go(x.e)
        if x.k == 'bin' and x.op in '+-':
            l = go(x.l)
            r = go(x.r)
            if l is None or r is None:
                return None
            return l + r
        if x.k == 'bin' and x.op == '*':
            if not self.is_random(x.l):
                return go(x.r)
            if not self.is_random(x.r):
                return go(x.l)
            return None
        if x.k == 'bin' and x.op == '/':
            if self.is_random(x.r):
                return None
            return go(x.l)
        return None
    return go(e)


def m_closed_form(self, n):
    """Decide a query's closed form at load (L §5.16 table); LOAD-RANDOM when there is none."""
    fn = n.fn if n.k == 'query' else 'crps'
    q = _c()._txt(n)
    if n.k == 'crps':
        raise LoadError('LOAD-RANDOM', '%s has no closed form; state paths N seed S' % q, n.loc)
    if n.where is not None:
        raise LoadError('LOAD-RANDOM', '%s has no closed form (a query with where); state paths N seed S' % q,
                        n.loc)
    arg = strip(n.arg)
    if fn == 'share':
        if arg.k != 'cmp' or arg.op not in ('<', '<=', '>', '>=', '==', '!='):
            raise LoadError('LOAD-RANDOM', '%s has no closed form; state paths N seed S' % q, n.loc)
        lr, rr = self.is_random(arg.l), self.is_random(arg.r)
        if lr and rr:
            raise LoadError('LOAD-RANDOM', '%s has no closed form (random values on both sides); state paths N '
                            'seed S' % q, n.loc)
        side = arg.l if lr else arg.r
        xs = self.affine(side)
        if xs is None or len(xs) != 1:
            raise LoadError('LOAD-RANDOM', '%s has no closed form (one random fact, read directly, in an affine '
                            'form, is needed); state paths N seed S' % q, n.loc)
        n._closed = ('share', xs, 'l' if lr else 'r')
    else:
        xs = self.affine(arg)
        if xs is None or not xs:
            raise LoadError('LOAD-RANDOM', '%s has no closed form; state paths N seed S' % q, n.loc)
        if fn == 'quantile' and len(xs) != 1:
            raise LoadError('LOAD-RANDOM', '%s has no closed form (a quantile of more than one random fact); '
                            'state paths N seed S' % q, n.loc)
        n._closed = (fn, xs)
    # the facts' distributions must be known: no random parameter
    for x in n._closed[1]:
        r = x._r
        if (r[1], r[2]) in self.random_items:
            raise LoadError('LOAD-RANDOM', '%s has no closed form (%s has a random parameter); state paths N '
                            'seed S' % (q, r[2]), n.loc)
        x._xdist = True


def m_finish_random(self):
    """E §3.1 step 10: mark the random cone, check every random ask line and query (LOAD-RANDOM)."""
    m = self.m
    self.random_cone()
    self.check_draw_slots()
    asks = {('ask', a.id): a for a in m.asks}
    for n, item, where in self.queries:
        a = asks.get(item)
        if a is None and n.k == 'query':
            # L §5.16: "A query that has no closed form is a statistic over paths: it stands only in an ask line that
            # states paths N seed S (LOAD-ASKONLY elsewhere)"; one the table gives in closed form is computed so in a
            # rule too ("a query around the read, mean(…) or quantile(…, q), computed in closed form"); L §8.4
            # LOAD-ASKONLY `<query> has no closed form outside an ask`
            n._qmode = 'closed'
            if not self.is_random(n.arg) and (n.where is None or not self.is_random(n.where)) and \
                    not reads_balances(n.arg):
                raise LoadError('LOAD-RANDOM', '%s of a value that is not random: a query is asked of a random '
                                'value (L §5.16)' % n.fn, n.loc)
            try:
                self.closed_form(n)
            except LoadError as e:
                if e.code != 'LOAD-RANDOM':
                    raise
                raise LoadError('LOAD-ASKONLY', '%s has no closed form outside an ask' % _c()._txt(n), n.loc)
            continue
        if a is not None and a.paths is not None:
            n._qmode = 'paths'
            continue
        if item[0] != 'ask' and n.k == 'crps':
            raise LoadError('LOAD-RANDOM', 'crps of a random value outside a paths ask', n.loc)
        n._qmode = 'closed'
        if n.k == 'query' and not self.is_random(n.arg) and (n.where is None or not self.is_random(n.where)) and \
                not reads_balances(n.arg):
            raise LoadError('LOAD-RANDOM', '%s of a value that is not random: a query is asked of a random value '
                            '(L §5.16)' % n.fn, n.loc)
        self.closed_form(n)
    for a in m.asks:
        exprs = []
        if a.form == 'exprs':
            exprs = list(a.exprs)
        elif a.form == 'check':
            exprs = list(a.check)
        elif a.form in ('explain', 'inputs-of'):
            exprs = [a.ref]
        elif a.form == 'solve':
            s = a.solve
            exprs = [s.f, s.g]
        for e in exprs:
            if a.path is None and self.line_random(e, a):
                raise LoadError('LOAD-RANDOM', 'ask %s: %s is random; apply a query' % (a.id, _c()._txt(e)), a.loc)
        if a.path is not None and a.paths is None:
            # L §2.3 (round 12): ["paths" INT "seed" INT ["path" INT]]: path p is written after paths N seed S
            raise LoadError('LOAD-PARSE', 'cannot parse: path %d without paths N seed S' % a.path, a.loc)
        if a.paths is not None:
            if a.paths < 1:
                raise LoadError('LOAD-DOMAIN', 'paths %d: N is at least 1' % a.paths, a.loc)
            if a.path is not None and not 1 <= a.path <= a.paths:
                raise LoadError('LOAD-DOMAIN', 'path %d is outside 1..%d' % (a.path, a.paths), a.loc)
    # orient (L §5.15): an ask that reaches crps or pdf states it
    for a in m.asks:
        if a.orient is not None:
            continue
        if self.reaches_any(('ask', a.id), self.orient_items):
            raise LoadError('LOAD-ORIENT', 'ask %s reaches crps or pdf without orient' % a.id, a.loc)


def reads_balances(e):
    """Whether an expression reads a balance, a low or flows: refcore's load marks the random cone through moves into
    the defs, fills and decisions that read the balances they book (random_cone), not into an ask line's own reads
    (E §3.1 step 10), so it can't see whether such a read reaches a random amount (an ask line's own projection reads
    it on a path by that path's sample, L §5.16), and a query around one is let through."""
    stack = [e]
    while stack:
        x = stack.pop()
        if isinstance(x, (list, tuple)):
            stack.extend(x)
            continue
        if not isinstance(x, Node):
            continue
        if x.k in ('bal', 'low', 'flows', 'value'):
            return True
        for a, b in x.__dict__.items():
            if a in ('loc',) or a.startswith('_'):
                continue
            if isinstance(b, Node) or isinstance(b, (list, tuple)):
                stack.append(b)
    return False


def m_reaches_any(self, start, targets):
    seen = set()
    stack = [start]
    while stack:
        x = stack.pop()
        if x in seen:
            continue
        seen.add(x)
        if x in targets:
            return True
        for t in self.edges.get(x, {}):
            stack.append(t)
        for t in self.rand_edges.get(x, {}):
            stack.append(t)
    return False


# -- dist rows: the distributions a version states (L §5.16, §5.3) -----------------------------------

def m_check_dist_rows(self):
    m = self.m
    for qid, inf in m.inputs.items():
        if inf.type is None or inf.type.k != 'dist':
            continue
        for r in inf.rows:
            v = r.val
            if not (isinstance(v, tuple) and v and v[0] == 'node'):
                continue
            node = v[1]
            check = _c()
            sc = check.Scope(self, ('input', qid), inf.regime, 'row')
            sc.allow_lit = True
            sc.dist_role = True
            t = self.tc(node, sc, expect=inf.type)
            if t.k != 'dist' or U.conv(t.u or (), inf.type.u or ()) is None:
                raise uerr('a row of %s gives %s, not %s' % (qid, T.show(t), T.show(inf.type)), r.loc)
            self.dist_whole(node, r.loc, 'a row of %s' % qid)


# -- scenario expression rows (L §6.4) -----------------------------------------------------------------

def m_check_scen_rows(self):
    """A scenario row may be an expression over the base's facts: the base's items named with the base's id as a
    prefix (read in the base), the scenario's own items bare (read in the scenario).  It has the target's type; its
    reads hang off the target, through an `srow` item of the scenario (L §6.4)."""
    m = self.m
    infos = [('input', q, inf, inf.rows) for q, inf in m.inputs.items()] + \
            [('def', q, d, d.givens) for q, d in m.defs.items()]
    for kind, qid, inf, rows in infos:
        for r in rows:
            v = r.val
            if not (isinstance(v, tuple) and v and v[0] == 'sexpr'):
                continue
            node = v[1]
            scen = r.version.scenario
            base_w = m.scenario_over.get(scen)
            base_id = base_w if base_w is not None else m.header.id
            chain = set()
            w = base_w
            while w is not None:
                chain.add(w)
                w = m.scenario_over.get(w)
            for x in list(walk(node)):
                if x.k == 'call' and getattr(x, 'prefix', None) == [base_id]:
                    # household.b(x): the base's band (L §2.3 Calls, round 14)
                    x.prefix = None
                    continue
                if x.k == 'field' and x.obj.k == 'name' and x.obj.id == base_id:
                    f = x.name
                    del x.obj
                    del x.name
                    x.k = 'name'
                    x.id = f
                    x._world = ('w', base_w, base_id, chain)
            item = ('srow', kind, qid, scen, r.sid)
            if not hasattr(self, 'srow_loc'):
                self.srow_loc = {}
            self.srow_loc[item] = r.loc
            check = _c()
            sc = check.Scope(self, item, inf.regime, 'row')
            sc.allow_lit = True
            self.edge(check.Scope(self, (kind, qid), inf.regime, 'row'), item)
            self.tc(node, sc, expect=inf.type)
            self.assign(inf.type, node, r.loc, 'a row of %s' % qid)


def m_check_srow_reads(self):
    """L §6.4: a scenario's expression row "reads inputs, bands and defs, never a place or a movement (a balance,
    `flows`, a move's instance), directly or through a def (LOAD-SCENARIO)"."""
    m = self.m
    for item in list(self.edges):
        if not (isinstance(item, tuple) and item and item[0] == 'srow'):
            continue
        seen = set()
        stack = [(item, ())]
        while stack:
            r, path = stack.pop()
            for t in self.edges.get(r, {}):
                if not isinstance(t, tuple) or not t:
                    continue
                bad = t[0] in ('place', 'balances', 'move', 'moveon') or (
                    t[0] == 'input' and t[1] in m.inputs and m.inputs[t[1]].observed)
                if bad:
                    what = {'place': 'the place %s', 'balances': 'a balance', 'move': 'the move %s',
                            'moveon': 'the move %s', 'input': 'the movements of %s'}[t[0]]
                    what = what % t[1] if '%s' in what else what
                    via = (' through %s' % ', '.join(path)) if path else ''
                    raise LoadError('LOAD-SCENARIO', 'the expression row of %s in %s reads %s%s: it reads inputs, '
                                    'bands and defs, never a place or a movement' % (item[2], item[3], what, via),
                                    getattr(self, 'srow_loc', {}).get(item))
                if t[0] in ('def', 'fill', 'band') and t not in seen:
                    seen.add(t)
                    stack.append((t, path + (t[1],)))


# -- sample pins (L §5.16 Pins) ---------------------------------------------------------------------

def m_sample_pin(self, p, sc):
    m = self.m
    tgt = p.target
    hit = m.qualified(tgt.qual, tgt.id) if tgt.qual is not None else m.lookup(tgt.id, sc.regime)
    if hit is None:
        raise LoadError('LOAD-ASIF', 'pin on unknown id %s' % tgt.id, p.loc)
    kind, info = hit
    if kind not in ('input', 'def') or info.type is None or info.type.k != 'dist':
        raise uerr('sample %s: a sample pin names a random fact (an input or def of type dist)' % tgt.id, p.loc)
    fields = list(tgt.fields)
    if fields and fields != ['u']:
        raise LoadError('LOAD-FIELD', '%s is not a field of a sample (u is)' % '.'.join(fields), p.loc)
    binds = info.binds
    keys = tgt.keys
    if keys is not None and len(keys) != len(binds):
        raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, p.loc)
    if keys is None and binds:
        raise LoadError('LOAD-KEY', 'wrong number of keys for %s: write [*] to pin every address' % info.id, p.loc)
    p.keyconv = []
    for kn, b in zip(keys or [], binds):
        if kn.k == 'star':
            p.keyconv.append(None)
        else:
            p.keyconv.append(self.key_pos(kn, b.dom, sc))
    p.pkind = 'sample'
    p.info = info
    p.kindname = kind
    p.ufield = bool(fields)
    sc2 = sc.child()
    if sc.where == 'ask':
        sc2.allow_lit = True
    if p.ufield:
        t = self.tc(p.value, sc2)
        if self.numcls(t) not in ('num', 'flex', 'int') or (t.k == 'num' and t.u):
            raise uerr('sample %s.u is %s, not a plain number' % (tgt.id, T.show(t)), p.loc)
        p.ty = T.NUM1
    else:
        st = self.sample_type(kind, info, info.type)
        p.ty = st
        if not self.pin_literal(p.value, st, sc2):
            self.tc(p.value, sc2, expect=st)
            self.assign(st, p.value, p.loc, 'the sample pin of %s' % info.id, pin=True)
    self.edge(sc, (kind, info.id), 'pin')


# -- solve (L §5.18) ----------------------------------------------------------------------------------

def m_check_solve(self, s, sc, deftype, loc):
    """solve f == g for v in [lo, hi] to u: v an input or def, pinned at each step; the bracket and u take
    v's type (L §5.18)."""
    m = self.m
    ref = s.ref
    keys = []
    if ref.k == 'index' and ref.obj.k == 'name':
        name = ref.obj.id
        keys = ref.keys
    elif ref.k == 'name':
        name = ref.id
    else:
        raise LoadError('LOAD-PARSE', 'cannot parse: solve names an input or a def, not %s' % _c()._txt(ref), loc)
    hit = m.lookup(name, sc.regime)
    if hit is None:
        raise LoadError('LOAD-NAME', 'unknown id %s' % name, loc)
    kind, info = hit
    if kind not in ('input', 'def'):
        raise LoadError('LOAD-ASIF', 'solve for %s, which is not an input or a def' % name, loc)
    if len(keys) != len(info.binds):
        raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, loc)
    keyconv = []
    for kn, b in zip(keys, info.binds):
        if kn.k == 'star':
            raise LoadError('LOAD-PARSE', 'cannot parse: * stands only in a pin', kn.loc)
        keyconv.append(self.key_pos(kn, b.dom, sc))
    vt = info.type
    if vt is None or vt.k not in ('num', 'money', 'int', 'rate'):
        raise uerr('solve for %s of type %s: a root is searched over numbers' % (info.id, T.show(vt)), loc)
    s._target = (kind, info, keyconv)
    s._vt = vt
    self.edge(sc, (kind, info.id), 'pin')
    ft = self.tc(s.f, sc)
    gt = self.tc(s.g, sc, expect=ft)
    if ft.k in ('flex',):
        self.tc(s.f, sc, expect=gt)
    self.compare('==', s.f, s.g, loc)
    if not (self.numcls(s.f._t) or s.f._t.k == 'hole') or not (self.numcls(s.g._t) or s.g._t.k == 'hole'):
        raise uerr('solve %s == %s: F = f - g is a number' % (T.show(s.f._t), T.show(s.g._t)), loc)
    sc2 = sc.child()
    sc2.allow_lit = True     # the bracket of a solve (L §7.1)
    vals = []
    for b in (s.lo, s.hi):
        if not self.pin_literal(b, vt, sc2):
            t = self.tc(b, sc2, expect=vt)
            self.assign(vt, b, loc, 'the bracket of solve')
        vals.append(_lit_value(b, vt))
    u = s.unit
    if vt.k == 'rate':
        if u.k != 'num' or not u.unit:
            raise uerr('solve for a rate: the step states its period (to 0.0001 %%/year)', loc)
        su, f = U.from_syntax(self.unit_terms(u.unit, u.loc))
        if U.period_of(su) is None:
            raise uerr('solve for a rate: the step %s is not a rate' % u.text, loc)
        u._val = u.v * f
        u._u = su
        u._t = T.RATE
    else:
        vu = vt.u if vt.k in ('num', 'money') else ()
        self.step_check(u, vu, sc2, s)
    # the grid lo + i u holds lo and hi (L §5.18)
    lo, hi = vals
    step = getattr(u, '_val', None)
    if lo is not None and hi is not None and step is not None:
        lq, hq = lo[0], hi[0]
        if lo[1] != hi[1] or (vt.k == 'rate' and lo[1] != u._u):
            if vt.k == 'rate':
                raise uerr('solve: the bracket and the step are rates of different periods', loc)
        if hq < lq:
            raise LoadError('LOAD-DOMAIN', 'solve: the bracket [lo, hi] has hi below lo', loc)
        if ((hq - lq) / step).denominator != 1:
            raise LoadError('LOAD-DOMAIN', 'solve: hi is not on the grid lo + i u', loc)
    s._t = vt if vt.k != 'int' else T.INT
    return s._t


def _lit_value(b, vt):
    """A bracket literal's value and unit, when it is a literal (else None)."""
    x = b
    sign = 1
    while x.k in ('paren', 'neg'):
        if x.k == 'neg':
            sign = -sign
        x = x.e
    if x.k != 'num':
        return None
    v = getattr(x, '_val', None)
    if v is None:
        v = x.v
    u = getattr(x, '_u', ())
    if vt.k == 'rate' and not u:
        return None
    return (Fraction(v) * sign, u)


# -- prorata (L §5.13) -------------------------------------------------------------------------------

def m_t_prorata(self, n, sc, expect):
    tt = self.tc(n.total, sc)
    c = self.numcls(tt)
    if c not in ('money', 'num') or not U.is_currency(tt.u or ()):
        raise uerr('prorata of %s: the total is money or a num CUR (L §5.13)' % T.show(tt), n.loc)
    cur = tt.u[0][0]
    if len(n.binds) != 1:
        # L §2.3: prorata = "prorata" "(" expr "," expr "for" bind "," ...: one binding
        raise LoadError('LOAD-PARSE', 'cannot parse: prorata takes one binding', n.loc)
    sc2 = sc.child()
    sc2.agg_depth += 1
    b = n.binds[0]
    self.bind(b, sc2)
    wt = self.tc(n.w, sc2)
    if self.numcls(wt) not in ('num', 'int', 'flex', 'money'):
        raise uerr('prorata: the weight is %s, not a number' % T.show(wt), n.loc)
    if b._dom[0] in ('index', 'kind'):
        kt = self.tc(n.key, sc, expect=T.key(b._dom[1]))
        self.assign(T.key(b._dom[1]), n.key, n.loc, 'the key of prorata')
    else:
        # a key list or a stepped range: k is one of its values (L §5.13 "key k's share")
        n._kconv = self.key_pos(n.key, b._dom, sc)
    # the step: as round's (L §7.1, §7.2)
    u = n.u
    sc3 = sc.child()
    if sc.where == 'ask':
        sc3.allow_lit = True
    ut = self.tc(u, sc3, expect=tt)
    if u.k == 'num' and not u.unit and not u.get('frac'):
        ut = u._t = T.flex(u.v)
    uc = self.numcls(ut)
    if uc is None:
        raise uerr('prorata: the step is %s' % T.show(ut), n.loc)
    if sc.where in ('def', 'at'):
        ok = (u.k == 'num' and u.v == 1) or (u.k == 'call' and u.name == 'minor') or (
            getattr(u, '_r', None) and u._r[0] == 'read' and u._r[1] == 'input')
        if not ok:
            raise LoadError('LOAD-LITERAL', 'the step of prorata is 1 with a unit, minor(CUR) or a row', n.loc)
    if uc != 'flex' and U.conv(ut.u if uc in ('num', 'money') else (), tt.u) is None:
        raise uerr('prorata: the step %s and the total %s have different units' % (T.show(ut), T.show(tt)), n.loc)
    if uc == 'flex':
        u._stepflex = True
    if n.rest in ('none', 'last'):
        n.name = 'prorata'
        self.mode_arg(n.mode, sc, n)
    if n.tie:
        for e in n.tie:
            self.tc(e, sc2)
    whole = uc == 'money' or (uc == 'flex' and (Fraction(ut.n) / CURRENCIES[cur]).denominator == 1)
    money = whole and (n.rest != 'last' or c == 'money')
    n._money = cur if money else None
    return T.money(cur) if money else T.num(tt.u)


# -- fill (L §5.12) ------------------------------------------------------------------------------------

def m_fill_scope(self, qid):
    check = _c()
    s = self.m.fills[qid]
    return check.Scope(self, ('fill', qid), None, 'def', [b.var for b in s.binds])


def m_fill_sides(self, qid):
    """A fill's bindings: its keys, its supply keys and its demand keys (domains), and taken's type."""
    fi = self.fill_infos.get(qid)
    if fi is not None:
        return fi
    s = self.m.fills[qid]
    fi = {'s': s, 'taken': None}
    self.fill_infos[qid] = fi
    sc = self.fill_scope(qid)
    for b in s.binds:
        # the key bind of a fill is defined for every key: no on or over (L §5.6, §5.7, round 18 C1-38)
        self.bind(b, sc, key_bind=True)
        if b._dom[0] not in ('index', 'kind', 'list', 'range'):
            raise NotYet('a keyed fill over %s' % b._dom[0], b.loc)
    fi['fdoms'] = [b._dom for b in s.binds]
    for side, binds in (('sdoms', s.sbinds), ('ddoms', s.dbinds)):
        sc2 = sc.child()
        for b in binds:
            self.bind(b, sc2)
            if b._dom[0] not in ('index', 'kind', 'event', 'obs', 'list', 'range'):
                raise NotYet('a fill side over %s' % b._dom[0], b.loc)
        fi[side] = [b._dom for b in binds]
    fi['sc'] = sc
    return fi


def m_check_fill(self, qid):
    """Type a fill, once: in the order of the check (moves, fills, defs), or first where a move reads its output (a
    plan's leg amount reading F[y].taken is no read inside the fill's own supply, L §5.12, §5.14.1; MC-X-F7)."""
    done = self.__dict__.setdefault('_fills_checked', set())
    if qid in done:
        return
    done.add(qid)
    busy = self.__dict__.setdefault('_fills_checking', set())
    busy.add(qid)
    try:
        self.check_fill_body(qid)
    finally:
        busy.discard(qid)


def m_check_fill_body(self, qid):
    fi = self.fill_sides(qid)
    s = fi['s']
    sc = fi['sc']
    scs = sc.child()
    for b in s.sbinds:
        scs.vars[b.var] = b._var
    st = self.tc(s.supply, scs)
    cs = self.numcls(st)
    if cs is None or cs == 'rate':
        raise uerr('fill %s: the supply is %s' % (qid, T.show(st)), s.loc)
    fi['taken'] = st if cs in ('money', 'num') else T.num(())
    scd = sc.child()
    for b in s.dbinds:
        scd.vars[b.var] = b._var
    dt = self.tc(s.demand, scd)
    cd = self.numcls(dt)
    su = st.u if cs in ('money', 'num') else ()
    du = dt.u if cd in ('money', 'num') else ()
    # a side whose unit only the run knows (a leg of unit any, an adjustment's change, "of the leg's type", L §5.14.3)
    # is checked per value at the take (L §3.2: "The run checks the keys load couldn't see, as load checks two
    # currencies"; L §8.4: refused at load where the types show it, otherwise at the key) (MC-X-F10)
    dyn = 'dyn' in (cs, cd)
    if cd is None or cd == 'rate' or (not dyn and U.conv(su, du) is None):
        raise uerr('fill %s: supply (%s) and demand (%s) must share a unit (L §5.12)' % (qid, T.show(st), T.show(dt)),
                   s.loc)
    sca = sc.child()
    for b in list(s.sbinds) + list(s.dbinds):
        if b.var in sca.vars:
            raise LoadError('LOAD-SHADOW', 'bound variable %s is bound on both sides of fill %s' % (b.var, qid),
                            b.loc)
        sca.vars[b.var] = b._var
    at = self.tc(s.arc, sca)
    if not self.is_bool(at):
        raise uerr('fill %s: arc is %s, not bool' % (qid, T.show(at)), s.loc)
    for o in s.order:
        self.tc(o, sca)
    for e in (s.tie or []):
        self.tc(e, sca)


def m_fill_ref(self, obj, sc):
    """obj is F.taken or F[y].taken: the fill's qualified id and its key nodes, or None."""
    if obj.k != 'field' or obj.name != 'taken':
        return None
    f = obj.obj
    keys = []
    if f.k == 'index':
        keys = f.keys
        f = f.obj
    if f.k != 'name' or f.id in sc.vars:
        return None
    hit = self.m.lookup(f.id, sc.regime)
    if hit is None or hit[0] != 'fill':
        return None
    qid = f.id if sc.regime is None or ('%s.%s' % (sc.regime, f.id)) not in self.m.fills else \
        '%s.%s' % (sc.regime, f.id)
    if qid not in self.m.fills:
        qid = [q for q in self.m.fills if q == f.id or q.endswith('.' + f.id)][0]
    return qid, keys


def m_fill_read_type(self, n, sc, obj, keys):
    qid, fkeys = self.fill_ref(obj, sc)
    fi = self.fill_sides(qid)
    if len(fkeys) != len(fi['fdoms']):
        raise LoadError('LOAD-KEY', 'wrong number of keys for fill %s' % qid, n.loc)
    tdoms = fi['sdoms'] + fi['ddoms']
    if len(keys) != len(tdoms):
        raise LoadError('LOAD-KEY', 'wrong number of keys for %s.taken: %d, not %d (supply keys, then demand keys)'
                        % (qid, len(keys), len(tdoms)), n.loc)
    n._fconvs = [self.key_pos(kn, d, sc) for kn, d in zip(fkeys, fi['fdoms'])]
    n._convs = [self.key_pos(kn, d, sc) for kn, d in zip(keys, tdoms)]
    n._fkeys = fkeys
    n._tkeys = keys
    n._r = ('fillread', qid)
    rel = self.relation(sc, fkeys) if fkeys else ('same' if sc.item[0] in ('def', 'fill') else 'any')
    if sc.readmode is not None:
        rel = sc.readmode
    self.edge(sc, ('fill', qid), rel)
    if rel.startswith('shift'):
        var, sign = rel.split(':')[1], rel.split(':')[2]
        self.shift_reads.append((sc.item, ('fill', qid), var, sign, var in sc.guards, n.loc))
    self.note_read(sc, 'fill', fi['s'] if False else _Id(qid))
    if fi['taken'] is None and qid not in self.__dict__.get('_fills_checking', ()):
        # read before the fill's own check (a move's leg is checked before the fills): check it now
        self.check_fill(qid)
    if fi['taken'] is None:
        raise NotYet('fill %s read inside its own supply (its taken type is the supply\'s)' % qid, n.loc)
    return fi['taken']


class _Id:
    def __init__(self, i):
        self.id = i


def m_inline_fill_edges(self):
    """A fill's terms are read at the law date of its reader (L §5.12 Law dates): its reads count as its
    reader's."""
    def fill_reads(q, seen):
        out = {}
        for t, rels in self.edges.get(('fill', q), {}).items():
            if t[0] == 'fill':
                if t[1] in seen:
                    continue
                for t2, r2 in fill_reads(t[1], seen | {t[1]}).items():
                    out.setdefault(t2, set()).update(r2)
                continue
            out.setdefault(t, set()).update(set('computed' if r == 'same' or r.startswith('shift') else r
                                                for r in rels))
        return out
    for r in list(self.edges):
        if r[0] == 'fill':
            continue
        for t in list(self.edges[r]):
            if t[0] == 'fill':
                for t2, rels in fill_reads(t[1], {t[1]}).items():
                    self.edges[r].setdefault(t2, set()).update(rels)
