"""Profile one refcore run with a time limit (4.5e step 2).

    pypy -I -B prof.py cprofile|count SECONDS OUT -- <refcore argv>      (run from the check's cwd)

cprofile: cProfile, stats dumped to OUT when the limit hits.  count: counts calls and distinct keys of the hot Engine
methods (read_input, candidates, row_matches, read_def), and the callers of read_input; written to OUT as text."""
import collections
import os
import signal
import sys

sys.path.insert(0, '/Users/jswob/projects/trop/engine/tools')
sys.setrecursionlimit(100000)
mode, limit, out = sys.argv[1], float(sys.argv[2]), sys.argv[3]
if mode.endswith('+fast'):   # the same profile with refcore_fast's layer installed
    mode = mode[:-5]
    from refcore_fast.layer import install
    install()
argv = sys.argv[sys.argv.index('--') + 1:]
from refcore import cli, evaluate, model  # noqa: E402

E = evaluate.Engine
calls = collections.Counter()
keys = collections.defaultdict(set)
callers = collections.Counter()


def ckey(ctx):
    return (ctx.law, ctx.cut, getattr(ctx.cut, 'windows', None), ctx.pins, ctx.axes, ctx.windows, ctx.wording,
            ctx.partial, ctx.path, ctx.world, ctx.unc, ctx.rnd, id(ctx.walk), ctx.wp, model._WSET[0])


def kv(ks):
    return tuple(model._kv(k) for k in ks)


def wrap(name, keyf):
    f = getattr(E, name)

    def g(self, *a, **kw):
        calls[name] += 1
        try:
            keys[name].add(keyf(self, *a, **kw))
        except TypeError:
            keys[name].add(('unhashable', calls[name]))
        if name == 'read_input':
            fr = sys._getframe(1)
            callers['%s:%d' % (fr.f_code.co_name, fr.f_lineno)] += 1
        return f(self, *a, **kw)
    setattr(E, name, g)


def report(*_):
    with open(out, 'w') as fh:
        if mode == 'sample':
            fh.write('%d samples\n\nself:\n' % nsamp[0])
            for f, n in selfc.most_common(30):
                fh.write('  %5.1f%% %s\n' % (100.0 * n / max(1, nsamp[0]), f))
            fh.write('\ninclusive:\n')
            for f, n in incl.most_common(60):
                fh.write('  %5.1f%% %s\n' % (100.0 * n / max(1, nsamp[0]), f))
        elif mode == 'cprofile':
            pr.disable()
            import pstats
            pstats.Stats(pr, stream=fh).sort_stats('cumulative').print_stats(60)
            fh.write('\n\n')
            pstats.Stats(pr, stream=fh).sort_stats('tottime').print_stats(40)
        else:
            for n in sorted(calls):
                fh.write('%-12s calls %9d distinct %8d\n' % (n, calls[n], len(keys[n])))
            fh.write('\nread_input callers:\n')
            for c, n in callers.most_common(25):
                fh.write('  %8d %s\n' % (n, c))
    os._exit(0)


if mode == 'count':
    wrap('read_input', lambda s, info, ks, ctx, calendar=False, probe=False: (id(info), kv(ks), ckey(ctx), calendar,
                                                                              probe, s.tracing))
    wrap('candidates', lambda s, rows, ks, d, ctx, info, instant=None, last_before=None:
         (id(rows), kv(ks), d, ckey(ctx), id(info), instant, last_before))
    wrap('row_matches', lambda s, r, ks, info: (id(r), kv(ks), id(info)))
    wrap('read_def', lambda s, info, ks, ctx, *a, **kw: (id(info), kv(ks), ckey(ctx)))
selfc = collections.Counter()
incl = collections.Counter()
nsamp = [0]
if mode == 'sample':
    import threading
    import time
    me = threading.get_ident()

    def sampler():
        while True:
            time.sleep(0.005)
            for tid, fr in sys._current_frames().items():
                if tid == me or tid == threading.get_ident():
                    continue
                nsamp[0] += 1
                seen = set()
                first = True
                while fr is not None:
                    c = fr.f_code
                    name = '%s:%s' % (os.path.basename(c.co_filename), c.co_name)
                    if first:
                        selfc[name] += 1
                        first = False
                    if name not in seen:
                        incl[name] += 1
                        seen.add(name)
                    fr = fr.f_back
    threading.Thread(target=sampler, daemon=True).start()
if mode == 'count2':
    def k2(s, info, ks, ctx, calendar=False, probe=False):
        d, per = s.time_point(info, ks, ctx)
        return (id(info), kv(ks), d, ctx.cut, getattr(ctx.cut, 'windows', None), ctx.pins, ctx.axes, ctx.windows,
                ctx.wording, ctx.path, ctx.world, ctx.unc, ctx.rnd, model._WSET[0], s.ask_K, calendar, probe)
    wrap('read_input', k2)
    wrap('candidates', lambda s, rows, ks, d, ctx, info, instant=None, last_before=None:
         (id(rows), kv(ks), id(info), s.ask_K, model._WSET[0]))
    elig = collections.Counter()
    f0 = E.read_input

    def ri(self, info, ks, ctx, calendar=False, probe=False):
        why = ('tracing' if self.tracing else 'each_met' if self.each_met is not None else 'pins' if ctx.pins else
               'prices' if info.prices is not None else 'reading' if info.reading else
               'same_rows' if getattr(self.m, 'same_rows', None) else 'joint' if getattr(self.m, 'joint', None) else
               'expr-row' if any(isinstance(r.val, tuple) and r.val and r.val[0] in ('node', 'sexpr') for r in info.rows)
               else 'ok')
        elig[why] += 1
        return f0(self, info, ks, ctx, calendar, probe)
    E.read_input = ri
    callers = elig
signal.signal(signal.SIGALRM, report)
signal.setitimer(signal.ITIMER_REAL, limit)
if mode == 'cprofile':
    import cProfile
    pr = cProfile.Profile()
    _bs = cli.big_stack
    cli.big_stack = lambda fn: _bs(lambda: (pr.enable(), fn())[1])   # the work runs in big_stack's thread
try:
    cli.main(argv)
finally:
    report()
