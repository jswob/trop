"""Session 8 step 1: count what one refcore run does, in units that carry over to any engine.

    cd <check's cwd>; COUNT8_OUT=out.txt [COUNT8_LIMIT=88] [COUNT8_FAST=1] \
        /usr/bin/time -l pypy -I -B /…/tmp/session8/tools/m8/refcore run <files> [--ask …]

The directory is named `refcore` so the 8 GB watchdog (engine/tests/tools/memcap.py) matches the command line
(`/refcore run `). refcore itself is imported unchanged from engine/tools; this file only wraps methods with counters
(adapted from tmp/session4_5/e/tools/prof.py, count mode). At COUNT8_LIMIT seconds the counts so far are written and
the run exits (marked partial). The run's own stdout is discarded."""
import collections
import io
import os
import signal
import sys
import time

sys.path.insert(0, '/Users/jswob/projects/trop/engine/tools')
sys.setrecursionlimit(100000)
OUT = os.environ['COUNT8_OUT']
LIMIT = float(os.environ.get('COUNT8_LIMIT', '88'))
if os.environ.get('COUNT8_FAST'):
    from refcore_fast.layer import install
    install()
from refcore import cli, evaluate, values, walk  # noqa: E402

T0 = time.time()
C = collections.Counter()
ENGINES = []
DEF_MISS_BY_ITEM = collections.Counter()
READ_KEYS = set()
WALK_SPANS = []
STRATA = collections.Counter()      # distinct (phase, stratum) pairs of decision acts per non-empty day
BITS = collections.Counter()        # Num results by the larger bit length of numerator and denominator
MAXBITS = [0]
E = evaluate.Engine
W = walk.Walk
N = values.Num


def wrap(cls, name, before):
    f = getattr(cls, name)

    def g(self, *a, **kw):
        before(self, *a, **kw)
        return f(self, *a, **kw)
    setattr(cls, name, g)


_einit = E.__init__


def e_init_keep(self, *a, **kw):
    C['engines'] += 1
    _einit(self, *a, **kw)
    ENGINES.append(self)


E.__init__ = e_init_keep


def ev(self, n, env, ctx):
    C['ev'] += 1


def read_def(self, info, keys, ctx):
    C['read_def'] += 1


def def_cell(self, info, keys, env, ctx):
    C['def_cell'] += 1
    ck = ('def', info.id, tuple(evaluate._kv(k) for k in keys), ctx.key())
    if ck in self.memo:
        C['def_hit'] += 1
    else:
        C['def_miss'] += 1
        DEF_MISS_BY_ITEM[info.id] += 1


def read_input(self, info, keys, ctx, calendar=False, probe=False):
    C['read_input'] += 1
    try:
        READ_KEYS.add((id(info), tuple(evaluate._kv(k) for k in keys), ctx.key(), calendar, probe))
    except TypeError:
        C['read_input_unhashable'] += 1


for name, f in (('ev', ev), ('read_def', read_def), ('def_cell', def_cell), ('read_input', read_input)):
    wrap(E, name, f)


def w_init(self, eng, tctx, start, end, *a, **kw):
    C['walks'] += 1
    if kw.get('proj') or (len(a) > 2 and a[2]):
        C['walks_proj'] += 1
    try:
        WALK_SPANS.append((end - start).days)
    except Exception:  # noqa: BLE001 (a span without dates)
        C['walk_span_unknown'] += 1


def walk_day(self, d):
    C['walk_days'] += 1
    acts = self.sched.get(d.toordinal())
    if acts:
        C['walk_days_nonempty'] += 1
        C['acts'] += len(acts)
        st = set()
        for key, kind, obj in acts:
            C['act:' + kind] += 1
            if kind == 'decday':
                st.add((key[1], key[2]))
        if st:
            STRATA[len(st)] += 1


def run_one(self, i, k):
    C['decision_instances'] += 1


def book_leg(self, mv, lg, key, *a, **kw):
    C['legs_booked'] += 1


def add(self, place, unit, e):
    C['log_entries'] += 1


def parts_flush(self, *a, **kw):
    C['parts_flush'] += 1


def pgroup_run(self, *a, **kw):
    C['parts_groups_run'] += 1


def padd(self, *a, **kw):
    C['parts_entries'] += 1


for name, f in (('__init__', w_init), ('walk_day', walk_day), ('run_one', run_one), ('book_leg', book_leg),
                ('add', add), ('parts_flush', parts_flush), ('pgroup_run', pgroup_run), ('padd', padd)):
    if hasattr(W, name):
        wrap(W, name, f)
    else:
        C['missing_hook:' + name] += 1

_ninit = N.__init__


def n_init(self, q, *a, **kw):
    _ninit(self, q, *a, **kw)
    C['num_made'] += 1
    nb = self.q.numerator.bit_length()
    db = self.q.denominator.bit_length()
    b = nb if nb > db else db
    if b > MAXBITS[0]:
        MAXBITS[0] = b
    BITS['<=63' if b <= 63 else '<=127' if b <= 127 else '<=1024' if b <= 1024 else '<=4096' if b <= 4096 else
         '>4096'] += 1
    if self.q.denominator == 1:
        C['num_den1'] += 1
    elif 10 ** 6 % self.q.denominator == 0:
        C['num_den_decimal6'] += 1
    if self.money:
        C['money_made'] += 1
        if 100 % self.q.denominator != 0:
            C['money_below_cent'] += 1


N.__init__ = n_init

_arith = evaluate.arith


def arith(op, *a, **kw):
    C['arith:' + op] += 1
    return _arith(op, *a, **kw)


evaluate.arith = arith


def report(partial):
    el = time.time() - T0
    memo_kinds = collections.Counter()
    memo_total = 0
    for eng in ENGINES:
        memo = getattr(eng, 'memo', {}) or {}
        for _ in range(5):
            try:
                ks = list(memo)   # the run's own thread may still be writing at the limit
                break
            except RuntimeError:
                ks = []
        memo_total += len(ks)
        for k in ks:
            memo_kinds[k[0] if isinstance(k, tuple) and k and isinstance(k[0], str) else '?'] += 1
    with open(OUT, 'w') as fh:
        fh.write('argv: %s\ncwd: %s\nseconds: %.1f%s\nfast layer: %s\n\n' % (
            ' '.join(sys.argv[1:])[:400], os.getcwd(), el, ' PARTIAL (limit hit)' if partial else '',
            bool(os.environ.get('COUNT8_FAST'))))
        for k in sorted(C):
            fh.write('%-28s %12d\n' % (k, C[k]))
        fh.write('%-28s %12d\n' % ('read_input_distinct', len(READ_KEYS)))
        fh.write('%-28s %12d\n' % ('def_items_evaluated', len(DEF_MISS_BY_ITEM)))
        fh.write('\ndef misses by item (top 8): %s\n' % DEF_MISS_BY_ITEM.most_common(8))
        fh.write('memo entries at end, all engines: %d by kind %s\n' % (memo_total, dict(memo_kinds.most_common(10))))
        if WALK_SPANS:
            s = sorted(WALK_SPANS)
            fh.write('walk spans (days): n %d, min %d, median %d, max %d, sum %d\n' % (
                len(s), s[0], s[len(s) // 2], s[-1], sum(s)))
        fh.write('decision strata per day with decisions: %s\n' % dict(sorted(STRATA.items())))
        fh.write('Num results by max bit length: %s; max %d bits\n' % (dict(BITS), MAXBITS[0]))
    os._exit(0)


signal.signal(signal.SIGALRM, lambda *_: report(True))
signal.setitimer(signal.ITIMER_REAL, LIMIT)
sys.stdout = io.StringIO()
try:
    cli.main(sys.argv[1:])
except SystemExit:
    pass
finally:
    report(False)
