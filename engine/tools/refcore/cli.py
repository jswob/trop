"""python -m refcore run <path ...> [--pin=bytes] [--ask id ...]   |   python -m refcore test [name ...]
   python -m refcore reprint <path ...>   (each file's canonical text, L §2.4, and the reality's sha256, E §3.1)"""
import os
import sys
import threading

from .errors import TallyError, NotYet


def run_texts(texts, ids=None, options=None):
    """Load a reality from [(file name, source)] and answer its asks; returns the printed lines.  options: {'pin':
    'bytes'} takes a use line's sha256 over the member's file bytes instead of its canonical text (joint.py);
    {'mj': 'check'} (writer 10, not language) computes a path quantile's Maritz-Jarrett weights both ways (eval2)."""
    from . import eval2
    eval2.MJ_CHECK[0] = (options or {}).get('mj') == 'check'
    try:
        return _run_texts(texts, ids, options)
    finally:
        eval2.MJ_CHECK[0] = False


def by_path(texts):
    """L §8.3 Which refusal prints (tally 2.1.1): a reality's files are taken in the order of their full paths, compared
    by code point (as E §3.1 step 1 takes them), whatever order they are given in."""
    return sorted(texts, key=lambda t: t[0])


def _run_texts(texts, ids=None, options=None):
    from .loader import scenario_header
    if (options or {}).get('reprint') is not None:
        return reprint_lines(texts)
    texts = by_path(texts)
    try:
        heads = [(f, src, scenario_header(f, src)) for f, src in texts]
    except TallyError as e:
        return ['refused: %s' % e.text()]
    from .joint import split_reality, header_uses, header_has_sames
    root, own, roots = split_reality(texts)
    srcs = dict(texts)
    uses = header_uses(srcs[root]) if root is not None else []
    if uses or (root is not None and header_has_sames(srcs[root])):
        return joint(texts, uses, ids, options)
    if any(h is not None for f, src, h in heads):
        return run_worlds(heads, ids)
    return run_one(texts, ids)


def reprint_lines(texts):
    """Each file's canonical text (L §2.4) in the order of their paths, then the reality's hash (E §3.1 step 1)."""
    from .canon import reprint, reality_hash
    from .joint import use_member_files, header_uses
    files = sorted(texts, key=lambda f: f[0])
    try:
        # a joint model's members, for the order of keys of their indexes (canon.reprint)
        members = {}
        on_disk = any(os.path.isfile(p) for p, src in texts)
        for p, src in files:
            if header_uses(src):
                members.update(use_member_files([(p, p, src)], texts, on_disk))
        out = []
        for p, t in reprint(files, members):
            out.append('--- ' + p)
            out.extend(t.rstrip('\n').split('\n'))
        out.append('sha256 ' + reality_hash(files, None, members))
    except TallyError as e:
        return ['refused: %s' % e.text()]
    return out


def joint(texts, uses, ids=None, options=None):
    """A joint model (L §6.1): each member (a file or a directory tree of files) found relative to the joint model's
    file and checked against its pin (LOAD-PIN; the sha256 of its canonical text, E §3.1 step 1, canon.py, or of its
    bytes with options pin=bytes), loaded alone (its asks, a.q) and merged in under its prefix (joint.py), so the
    joint model reads its things, places and movements, and a binding over a prelude kind ranges over every
    member's."""
    from .joint import joint_texts, MemberRefused, SEP
    from .loader import load_lines
    from .ask import Asker
    try:
        merged, alone, info = joint_texts(texts, uses, options or {})
    except MemberRefused as e:
        return ['refused: member %s: %s' % (e.alias, e.err.text())]
    except TallyError as e:
        return ['refused: %s' % e.text().replace(SEP, '.')]
    except RecursionError:
        return ['refused: REFCORE-NOT-YET nesting deeper than this interpreter\'s stack']
    except Exception as e:
        if os.environ.get('REFCORE_DEBUG'):
            raise
        return ['refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e)]

    def shown(lines):
        out = []
        for l in lines:
            if l.startswith('warning: WARN-UNREAD') and SEP in l:
                # a member's items unread in the joint model: each member warns alone (L §8.5)
                continue
            out.append(l.replace(SEP, '.'))
        return out
    from .loader import scenario_header
    srcs = dict(texts)
    heads = [(f, ls, scenario_header(f, srcs[f]) if f in srcs else None) for f, ls in merged]
    if any(h is not None for f, ls, h in heads):
        # L §6.1 Writes into members: "A scenario over a joint model (§6.4) may replace the members' rows too, named
        # with their prefixes": its files are the joint model's own, merged as they are, and run as worlds (§6.4)
        return shown(run_worlds(heads, ids, loader=lambda files, worlds: load_lines(files, worlds, members=alone,
                                                                                  joint=info), srcs=srcs))
    try:
        m = load_lines(merged, members=alone, joint=info)
    except TallyError as e:
        return shown(['refused: %s' % e.text()])
    except RecursionError:
        return ['refused: REFCORE-NOT-YET nesting deeper than this interpreter\'s stack']
    except Exception as e:
        if os.environ.get('REFCORE_DEBUG'):
            raise
        return ['refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e)]
    try:
        return shown(Asker(m).run(ids))
    except TallyError as e:
        return shown(['refused: %s' % e.text()])
    except Exception as e:
        if os.environ.get('REFCORE_DEBUG'):
            raise
        return ['refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e)]


def guarded(fn):
    """fn() or the refusal it ends in (nothing may crash)."""
    try:
        return fn(), None
    except TallyError as e:
        return None, e
    except RecursionError:
        from .errors import NotYet
        return None, NotYet('nesting deeper than this interpreter\'s stack')
    except Exception as e:
        if os.environ.get('REFCORE_DEBUG'):
            raise
        from .errors import NotYet
        return None, NotYet('an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e))


def cross_stack_diff(aid, a, realities, chains, askers, mid):
    """A diff whose two worlds sit on two stacks of scenarios: each side answered by the reality holding its world,
    then combined (ask.Asker.diff_combine)."""
    from .ask import Asker
    from .errors import Unspecified
    sw, bw = a.diff[2], a.diff[3]
    res = {}
    coll = None
    ctx = t = None
    asker0 = None
    for w in (sw, bw):
        if w == mid:
            x = None
        else:
            hit = sorted(s for s in chains if s is not None and w in chains[s])
            x = w if w in chains else (hit[0] if hit else None)
        r = realities[x]
        ra = [b for b in r.asks if b.id == aid]
        if not ra:
            # L §6.1: the reality is all its files; refcore loads one reality per stack, and the ask's text isn't
            # in the one that holds this side's stack
            return ['%s: refused: %s' % (aid, NotYet('a diff whose worlds (%s, %s) sit on different stacks, asked '
                                                     'where one of them is not loaded' % (sw, bw)).text())]
        asker = askers.get(x)
        if asker is None:
            asker = askers[x] = Asker(r)
        asker._diff_only = w
        asker._diff_side = None
        try:
            lines = asker.answer(ra[0])
        finally:
            asker._diff_only = None
        side = asker._diff_side
        if side is None:
            return lines
        res[w] = side[0]
        if coll is None:
            coll, ctx, t, asker0 = side[1], side[2], side[3], asker
        else:
            coll.merge(side[1])
    return asker0.diff_combine(a, res, coll, ctx, t)


def run_worlds(heads, ids=None, loader=None, srcs=None):
    """A reality with scenarios (L §6.4): the base loads alone, and each scenario with the base and the scenarios it
    stacks on, so a load refusal in a scenario file refuses only that scenario and those over it.  Each ask is
    answered in the deepest world it names (its file's scenario, its `in`, diff's worlds, its scenario pins); the
    worlds below it are evaluated in the same reality with the higher layers hidden."""
    from .loader import load_texts
    from .ask import Asker, LoadAtRun, warning_lines
    from .errors import LoadError, Unspecified
    base = [(f, src) for f, src, h in heads if h is None]
    scen = {}
    dup = set()
    for f, src, h in heads:
        if h is not None:
            if h[0] in scen:
                dup.add(h[0])
            scen[h[0]] = (f, src, h[1], h[2])
    worlds = set(scen)
    if loader is None:
        loader = load_texts
    m0, err = guarded(lambda: loader(base, worlds))
    if err is not None:
        # a base item or ask that names an item declared only in a scenario file (L §6.4, LOAD-SCENARIO)
        name = None
        if err.code == 'LOAD-NAME' and err.msg.startswith('unknown id '):
            name = err.msg[len('unknown id '):].split(' ')[0]
        elif err.code == 'LOAD-KEY' and err.msg.startswith('ambiguous key '):
            # an unknown bare name reads as a key literal
            name = err.msg[len('ambiguous key '):].split(' ')[0].rstrip(':')
        if name is not None:
            for sid in sorted(scen):
                if name in declared_ids(scen[sid][1] if srcs is None else srcs[scen[sid][0]]):
                    err = LoadError('LOAD-SCENARIO', '%s of scenario %s is read outside it' % (name, sid), err.loc)
                    break
        return ['refused: %s' % err.text()]
    mid = m0.header.id
    realities = {None: m0}
    errs = {}
    chains = {None: []}
    for sid in sorted(scen):
        chain = []
        w = sid
        e = None
        while w != mid:
            if w in chain:
                e = LoadError('LOAD-SCENARIO', 'over cycle or deeper than 4 (scenario %s)' % sid, scen[sid][3])
                break
            if w not in scen:
                e = LoadError('LOAD-SCENARIO', 'scenario %s is over %s, neither the model nor a scenario' % (
                    chain[-1] if chain else sid, w), scen[sid][3])
                break
            chain.append(w)
            w = scen[w][2]
        if e is None and len(chain) > 4:
            e = LoadError('LOAD-SCENARIO', 'over cycle or deeper than 4 (scenario %s)' % sid, scen[sid][3])
        if e is None and any(c in dup for c in chain):
            e = LoadError('LOAD-SCENARIO', 'scenario %s declared twice' % [c for c in chain if c in dup][0],
                          scen[sid][3])
        if e is None:
            files = base + [(scen[c][0], scen[c][1]) for c in reversed(chain)]
            r, e = guarded(lambda: loader(files, worlds))
            if e is None:
                realities[sid] = r
                chains[sid] = chain
        if e is not None:
            if getattr(e, 'base_file', False):
                return ['refused: %s' % e.text()]
            errs[sid] = e
    # the asks: the base's from the base, a scenario's from its own reality
    # L §6.4 Its items: "Two scenarios over one base never meet, so each may declare an id the other declares": an
    # ask is known by its id and the file's world
    asks = {}
    for a in m0.asks:
        asks[(a.id, '')] = (a, None)
    for sid, r in realities.items():
        if sid is None:
            continue
        for a in r.asks:
            if getattr(a, '_scen', None) == sid:
                asks[(a.id, sid)] = (a, sid)
    out = []
    for sid in sorted(errs):
        out.append('scenario %s: refused: %s' % (sid, errs[sid].text()))
    askers = {}
    members = getattr(m0, 'joint_members', None) or {}
    if ids:
        order = []
        for i in ids:
            if '.' in i and i.split('.', 1)[0] in members:
                order.append(i)      # a member's ask a.q (L §6.1), answered by the member alone
            else:
                order.extend(k for k in sorted(asks) if k[0] == i)
    else:
        order = sorted(asks)
    for akey in order:
        if isinstance(akey, str):
            asker = askers.get(None)
            if asker is None:
                asker = askers[None] = Asker(m0)
            out.extend(asker.member_ask(akey))
            continue
        aid = akey[0]
        a, sf = asks[akey]
        need = set()
        if sf is not None:
            need.add(sf)
        if a.world is not None and a.world != mid:
            need.add(a.world)
        if a.form == 'diff':
            need |= set(w for w in a.diff[2:] if w != mid)
        for p in a.pins:
            if p.kind == 'scenario' and p.id != mid:
                need.add(p.id)
        # `x as if {scenario s}` is x in s (L §6.4 Asking): in the ask's line or a def it reaches
        need |= set(w for w in asif_worlds(m0, a) if w != mid and w in scen)
        if any(w in errs for w in need):
            w = sorted(w for w in need if w in errs)[0]
            out.append('%s: refused: %s' % (aid, errs[w].text()))
            continue
        x = None
        ok = True
        cross = None
        if need:
            cands = [w for w in need if need <= set(chains.get(w, ()))]
            if not cands and a.form == 'diff' and sf is None:
                # L §5.18 "diff(e) in S against B: e evaluated in world S and in world B"; L §6.1 the reality is all
                # its files: each side in the reality that holds its stack (writer 5's T-96, stated)
                out.extend(cross_stack_diff(aid, a, realities, chains, askers, mid))
                continue
            if not cands:
                # L §5.18: "an ask is checked and answered only in the worlds its line is evaluated in (its in, both
                # worlds of a diff, each as if {scenario s} in it)": the line in the reality of its own world, each as
                # if {scenario s} of another stack in the reality that holds s (Engine.e_asif)
                own = sf if sf is not None else (a.world if a.world is not None and a.world != mid else None)
                for p in a.pins:
                    if p.kind == 'scenario':
                        own = None if p.id == mid else p.id
                cands = [own]
                if own not in realities:
                    out.append('%s: refused: %s' % (aid, NotYet('an ask whose worlds (%s) are on different stacks '
                                                                'of scenarios' % ', '.join(sorted(need))).text()))
                    continue
                cross = make_cross(aid, sf, realities, chains)
            x = cands[0]
        r = realities[x]
        ra = [b for b in r.asks if b.id == aid and getattr(b, '_scen', None) == sf] or \
            [b for b in r.asks if b.id == aid]
        ra = ra[0]
        asker = askers.get(x)
        if asker is None:
            asker = askers[x] = Asker(r)
        asker.cross = cross
        try:
            out.extend(asker.answer(ra))
        except LoadAtRun as lx:
            if x is None:
                return ['refused: %s' % lx.err.text()]
            out.append('%s: refused: %s' % (aid, lx.err.text()))
    seen = set()

    def unread_everywhere(msg):
        # WARN-UNREAD is decided on the item graph of the reality, scenario files included: a source that only a
        # scenario's version cites is cited (L §5.2 "a source that no version cites"), an item only a scenario reads
        # is read (L §8.5): kept only where every reality that declares the item warns it
        ws = msg.split(' ')
        if len(ws) < 2:
            return True
        kind, iid = ws[0], ws[1]
        for r in realities.values():
            table = {'source': getattr(r, 'sources', {}), 'input': r.inputs, 'def': r.defs}.get(kind)
            if table is None:
                return True
            if iid in table and ('WARN-UNREAD', msg) not in r.warnings:
                return False
        return True
    warned = []
    for x in [None] + sorted(k for k in realities if k is not None):
        for code, msg in realities[x].warnings:
            if code == 'WARN-UNREAD' and not unread_everywhere(msg):
                continue
            if (code, msg) not in seen:
                seen.add((code, msg))
                warned.append((code, msg))
    for x in [None] + sorted(k for k in askers if k is not None):
        if x not in askers:
            continue
        for code, msg in askers[x].walk_warnings:
            if (code, msg) not in seen:
                seen.add((code, msg))
                warned.append((code, msg))
    return out + warning_lines(warned)


def make_cross(aid, sf, realities, chains):
    """An ask's evaluator of an `as if {scenario s}` whose s sits on another stack of scenarios (L §6.4, §5.18): the
    same node of the same ask line, in the reality that holds s."""
    from .evaluate import Engine
    engines = {}

    def holder(w):
        if w in chains:
            return w
        hit = sorted(s for s in chains if s is not None and w in chains[s])
        return hit[0] if hit else None

    def cross(eng, n, env, ctx):
        w = [p.id for p in n.pins if p.kind == 'scenario'][-1]
        x = holder(w)
        r = realities.get(x)
        ra = [b for b in r.asks if b.id == aid and getattr(b, '_scen', None) == sf] if r is not None else []
        if not ra:
            raise NotYet('an as if {scenario %s} of an ask whose line the reality holding %s doesn\'t load (an ask in '
                         'a scenario file of another stack)' % (w, w))
        path = node_path(eng.cross_ask, n)
        if path is None:
            raise NotYet('an as if {scenario %s} reached through a def from another stack' % w)
        n2 = follow_path(ra[0], path)
        e2 = engines.get(x)
        if e2 is None:
            e2 = engines[x] = Engine(r)
            e2.check_range_lits()
            e2.cross = cross
            e2.cross_ask = ra[0]
        e2.ask_K = getattr(eng, 'ask_K', None)
        e2.ask_paths = getattr(eng, 'ask_paths', None)
        return e2.ev(n2, env, ctx)
    return cross


def node_path(root, target):
    """The steps (field, index) from root to the node target, or None."""
    from .node import Node
    stack = [(root, [])]
    while stack:
        n, path = stack.pop()
        if n is target:
            return path
        if isinstance(n, Node):
            for a in sorted(n.__dict__):
                if a == 'loc' or a.startswith('_'):
                    continue
                stack.append((n.__dict__[a], path + [a]))
        elif isinstance(n, (list, tuple)):
            for i, x in enumerate(n):
                stack.append((x, path + [i]))
    return None


def follow_path(root, path):
    n = root
    for step in path:
        n = n.__dict__[step] if isinstance(step, str) else n[step]
    return n


def asif_worlds(m, a):
    """The scenarios an ask names in an `as if {scenario s}`, in its line or in a def it reaches (the item graph)."""
    from .node import walk
    out = set()

    def scan(node):
        if node is None:
            return
        for x in walk(node):
            if getattr(x, 'k', None) == 'asif':
                for p in x.pins or ():
                    if getattr(p, 'kind', None) == 'scenario':
                        out.add(p.id)
    scan(a)
    ch = getattr(m, 'checker', None)
    if ch is None:
        return out
    seen = set()
    stack = [('ask', a.id)]
    while stack:
        x = stack.pop()
        if x in seen:
            continue
        seen.add(x)
        if x[0] == 'def' and x[1] in m.defs:
            d = m.defs[x[1]]
            scan(getattr(d, 'decl', None))
        stack.extend(ch.edges.get(x, {}).keys())
    return out


def declared_ids(src):
    """The ids a file's sections declare (the word after the section keyword), for naming a refusal."""
    out = set()
    for line in src.split('\n'):
        ws = line.split()
        if len(ws) >= 2 and not line.startswith((' ', '\t')) and ws[0] in (
                'def', 'input', 'band', 'fill', 'move', 'place', 'kind', 'index', 'unit', 'axis', 'source',
                'version'):
            out.add(ws[1].split('[')[0].split(':')[0].split('(')[0])
    return out


def run_one(texts, ids=None, members=None):
    from .loader import load_texts
    from .ask import Asker
    try:
        m = load_texts(texts, members=members)
    except TallyError as e:
        return ['refused: %s' % e.text()]
    except RecursionError:
        return ['refused: REFCORE-NOT-YET nesting deeper than this interpreter\'s stack']
    except Exception as e:
        # nothing may crash: an input this interpreter can't handle is a refusal
        if os.environ.get('REFCORE_DEBUG'):
            raise
        return ['refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e)]
    try:
        return Asker(m).run(ids)
    except TallyError as e:
        return ['refused: %s' % e.text()]
    except Exception as e:
        if os.environ.get('REFCORE_DEBUG'):
            raise
        return ['refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)' % (type(e).__name__, e)]


def read_files(pairs):
    """[(path, name)] -> ([(name, source)], None), or (None, the refusal's line).  L §8.3 Which refusal prints (tally
    2.1.1): load reads every file before it splits any into tokens, the files in the order of their full paths
    (code point); the first path that doesn't exist is refused (LOAD-PATH), as the first file that isn't UTF-8 is
    (LOAD-PARSE), each named as the run is given it."""
    texts = []
    for f, name in sorted(pairs):
        if not os.path.exists(f):
            return None, 'refused: LOAD-PATH %s doesn\'t exist' % f
        with open(f, 'rb') as fh:
            raw = fh.read()
        try:
            texts.append((name, raw.decode('utf-8')))
        except UnicodeDecodeError:
            return None, 'refused: LOAD-PARSE cannot parse: file is not UTF-8 (%s)' % f
    return texts, None


def run_paths(paths, ids=None, options=None):
    from .loader import collect
    texts, refused = read_files([(f, f) for f in collect(paths)])
    if refused is not None:
        return [refused]
    return run_texts(texts, ids, options)


def reprint_paths(paths):
    """`reprint`: each file named relative to the directory given, else by its base name (INTERFACE gap I-3)."""
    from .loader import collect
    pairs = []
    for a in paths:
        for f in collect([a]):
            pairs.append((f, os.path.relpath(f, a).replace(os.sep, '/') if os.path.isdir(a) else os.path.basename(f)))
    texts, refused = read_files(pairs)
    if refused is not None:
        return [refused]
    return reprint_lines(texts)


def command_lines(argv):
    """The lines `run` or `reprint` prints (argv as main takes it; testrun's `=== options disk` calls it too)."""
    if argv[0] == 'reprint':
        return reprint_paths(argv[1:])
    paths = []
    ids = []
    mode = 'paths'
    options = {}
    for a in argv[1:]:
        if a == '--ask':
            mode = 'ask'
        elif a.startswith('--pin='):
            # --pin=bytes: a use line's sha256 over the member's bytes, not its canonical text (E §3.1 step 1)
            options['pin'] = a[len('--pin='):]
        elif mode == 'ask':
            ids.append(a)
        else:
            paths.append(a)
    return run_paths(paths, ids or None, options)


def big_stack(fn):
    """Run fn in a thread with a large stack (deep recurrences)."""
    sys.setrecursionlimit(200000)
    threading.stack_size(512 * 1024 * 1024)
    box = {}

    def target():
        try:
            box['v'] = fn()
        except BaseException as e:  # re-raised in the caller
            box['e'] = e
    t = threading.Thread(target=target)
    t.start()
    t.join()
    if 'e' in box:
        raise box['e']
    return box['v']


def main(argv):
    if not argv or argv[0] not in ('run', 'test', 'reprint'):
        print(__doc__)
        return 2
    if argv[0] == 'test':
        from .testrun import run_tests
        return big_stack(lambda: run_tests(argv[1:]))
    lines = big_stack(lambda: command_lines(argv)) if argv[0] == 'run' else command_lines(argv)
    for l in lines:
        print(l)
    return 1 if lines and lines[0].startswith('refused: ') else 0
