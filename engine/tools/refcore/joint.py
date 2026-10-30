"""Joint models (L §6.1, L §5.1's joint model bullet, E §3.1 step 1).

A joint model imports each member, pinned by its hash, "loaded as closed members under their prefixes ... the
prelude's kinds are shared by all of them, a member's own kinds stay under its prefix" (E §3.1 step 1).  This
interpreter loads the joint model and its members as one reality: every name a member declares is read under its
prefix (`a.konto`; internally `a` + SEP + `konto`, printed with a dot), the prelude and the built-in ids are one,
so a binding over a prelude kind ranges over every member's things and movements (L §6.1 Kinds across members).

`same` lines are taken in two steps:
- before parsing, identities of things, places and kinds (a thing stated `same` as another is one thing, bound
  once; a member's outside place stated `same` as another member's place contributes only its legs to it, so its
  name is that place's; two places of which neither is outside must be of one kind, LOAD-SAME);
- at load and in the walk, identities of rows and movements (loader.joint_sames, walk consolidation): two rows read
  together (HOLE-SAME where they disagree), two movements counted once, an internal leg with no written link a hole
  naming it (HOLE-SAME).

A member is also loaded alone, so its asks are asked as `a.q` by the member alone (L §6.1: "joining changes no
member's answer").
"""
import hashlib
import os
import re

from .errors import TallyError, LoadError, NotYet, Unspecified
from .lexer import split_lines, Tok
from .model import PRELUDE_PLACE, BUILTIN_IDS

SEP = '·'

SECTION_CATS = ('source', 'version', 'kind', 'axis', 'index', 'place', 'input', 'band', 'def', 'fill', 'move')


def owner_of(name):
    """The member a name belongs to ('' for the joint model's own)."""
    return name.split(SEP, 1)[0] if SEP in name else ''


def owner_deep(name):
    """The innermost member a name belongs to ('a' for a·x, 'a·b' for a·b·x of a member's member; '' for the joint
    model's own): a member that is a joint model holds its members' things, each member's are its own (L §6.1)."""
    return name.rsplit(SEP, 1)[0] if SEP in name else ''


def owns(owner, o):
    """Whether the member `owner` holds what member `o` holds: itself, or a member of it (nested, E §3.1 step 1)."""
    return o == owner or o.startswith(owner + SEP)


def shown(name):
    return name.replace(SEP, '.')


class JointInfo:
    """What the loader and the walk need to know of a joint model."""

    def __init__(self):
        self.members = []          # aliases
        self.groups = {}           # representative place -> [(owner, name as written, outside)]
        self.owner_place = {}      # (owner, representative) -> True when that owner holds a non-outside place there
        self.things = {}           # representative thing -> [(owner, name as written)]: things stated same, whose
        #                            entries are all kept and read together (L §6.1)
        self.own_alias = {}        # the joint model's own names its same lines joined to a member's: name -> the
        #                            representative, so a model that has it as a member still reads them (f.ania)
        self.place_others = {}     # representative place -> [(owner, kind, [(lives, version id or None, replace,
        #                            loc)])]: the other sides of places stated same whose lifespans may differ, read
        #                            together with the representative's (L §6.1; loader.joint_place_sides)


# -- the pin (E §3.1 step 1, L §6.1, L §8.3 LOAD-PIN) -------------------------------------------------------------

def use_member_files(files, pool=(), on_disk=False):
    """The members a joint model among `files` (path, path in its tree, source) names by its use lines: {alias:
    [(path in the member's tree, source)]}, found relative to its model file; a member not found is left out (its own
    pin refuses it)."""
    mods = [(fn, src) for fn, rel, src in files if is_model(src)]
    if len(mods) != 1:
        return {}
    fn, src = mods[0]
    out = {}
    for a, f, h in header_uses(src):
        try:
            got = member_texts(a, f, os.path.normpath(os.path.join(os.path.dirname(fn), f)), pool, on_disk)
        except TallyError:
            continue
        out[a] = [(rel, s) for f2, rel, s in got]
    return out


def member_hash(files, mode=None, pool=(), on_disk=False):
    """A member's hash (E §3.1 step 1): its files' canonical text, a member that is a joint model reprinted with its
    own members' declarations (the order of keys of their indexes, canon.reprint); or its bytes (mode 'bytes')."""
    from .canon import reality_hash
    if mode == 'bytes':
        return reality_hash([(rel, src) for fn, rel, src in files], 'bytes')
    return reality_hash([(rel, src) for fn, rel, src in files], None, use_member_files(files, pool, on_disk))


def pin_check(alias, fname, files, hsh, options, pool=(), on_disk=False):
    """`use a = "path" sha256 "…"`: a member whose hash isn't the one written is refused (LOAD-PIN).  E §3.1 step 1:
    "The reality's hash is the sha256 of the UTF-8 bytes of its canonical text, its files reprinted and taken in the
    order of their paths in its directory tree (compared by code point) ... A joint model's use line pins a member by
    this hash, written as 64 lower-case hexadecimal digits" (canon.py); a hash of any other form matches no member.
    `pin=bytes` (a run option of refcore's, not the language) takes the files' bytes as given instead.
    files: [(path, path in the member's tree, source)]."""
    msg = '%s doesn\'t match its sha256 in use %s' % (fname, alias)
    if not re.fullmatch(r'[0-9a-f]{64}', hsh or ''):
        raise LoadError('LOAD-PIN', msg)
    mode = (options or {}).get('pin')
    got = member_hash(files, 'bytes' if mode == 'bytes' else None, pool, on_disk)
    if got != hsh:
        raise LoadError('LOAD-PIN', msg)


def header_uses(src):
    """The `use a = "path" sha256 "…"` lines of a model file's header (L §6.1 Joint models)."""
    out = []
    lines = src.split('\n')
    if not lines or not lines[0].startswith('model '):
        return out
    for l in lines[1:]:
        if not l.startswith((' ', '\t')):
            break
        mm = re.match(r'\s+use\s+(\w+)\s*=\s*"([^"]*)"\s+sha256\s+"([^"]*)"\s*$', l)
        if mm:
            out.append((mm.group(1), mm.group(2), mm.group(3)))
    return out


def header_has_sames(src):
    lines = src.split('\n')
    if not lines or not lines[0].startswith('model '):
        return False
    for l in lines[1:]:
        if not l.startswith((' ', '\t')):
            break
        if re.match(r'\s+same\s', l):
            return True
    return False


def is_model(src):
    return src.split('\n', 1)[0].startswith('model ')


def is_scenario(src):
    return src.split('\n', 1)[0].startswith('scenario ')


def split_reality(texts):
    """(the file with the reality's model header, the reality's own files, the model files that are no member's).
    L §6.1: "A model is one or more .tally files in one directory tree ... Exactly one file has [the model header]";
    a member's files given beside the joint model's are the member's (the files of any path a use line names)."""
    models = [(f, src) for f, src in texts if is_model(src)]
    targets = set()
    for f, src in models:
        for a, path, h in header_uses(src):
            targets.add(os.path.normpath(os.path.join(os.path.dirname(f), path)))

    def inside(f):
        n = os.path.normpath(f)
        return any(n == t or n.startswith(t + os.sep) for t in targets)
    own = [(f, src) for f, src in texts if not inside(f)]
    roots = [f for f, src in own if is_model(src)]
    return (roots[0] if len(roots) == 1 else None), own, roots


def member_texts(alias, fname, p, pool, on_disk):
    """A member's files: the file a use line names, or every .tally file in the directory tree it names (L §6.1 "A
    model is one or more .tally files in one directory tree"), from the files given, else from the disk.
    [(path, path in the member's tree, source)], in the order of their paths (E §3.1 step 1)."""
    exact = [(f, src) for f, src in pool if os.path.normpath(f) == p]
    if exact:
        f, src = exact[0]
        return [(f, os.path.basename(f), src)]
    got = {}
    for f, src in pool:
        n = os.path.normpath(f)
        if n.startswith(p + os.sep):
            got[n] = (f, src)
    if on_disk and os.path.isdir(p):
        from .loader import collect
        for f in collect([p]):
            n = os.path.normpath(f)
            if n not in got:
                got[n] = (f, _read(f))
    elif on_disk and not got and os.path.isfile(p):
        return [(p, os.path.basename(p), _read(p))]
    if not got:
        raise LoadError('LOAD-PIN', '%s doesn\'t match its sha256 in use %s (no such file)' % (fname, alias))
    out = []
    for n in got:
        f, src = got[n]
        out.append((f, os.path.relpath(n, p).replace(os.sep, '/'), src))
    out.sort(key=lambda x: x[1])
    return out


def _read(f):
    with open(f, 'rb') as fh:
        raw = fh.read()
    try:
        return raw.decode('utf-8')
    except UnicodeDecodeError:
        raise LoadError('LOAD-PARSE', 'cannot parse: file is not UTF-8 (%s)' % f)


# -- token helpers ---------------------------------------------------------------------------------------------------

def blocks(lines):
    """[(start, end)] of the level-0 blocks of a file (its header first, then its sections)."""
    out = []
    i = 0
    while i < len(lines):
        j = i + 1
        while j < len(lines) and lines[j].level > 0:
            j += 1
        out.append((i, j))
        i = j
    return out


def _rename_ok(toks, j):
    """Whether the ID token toks[j] names an item (renamed under a member's prefix), not a field, a rule label, a leg
    or an attribute name, a record field or a band's argument."""
    t = toks[j]
    if t.t != 'ID':
        return False
    prev = toks[j - 1] if j > 0 else None
    nxt = toks[j + 1] if j + 1 < len(toks) else None
    if prev is not None and prev.s == '.' and not t.ws:
        return False
    if prev is not None and prev.s in ('rule', 'leg', 'attr'):
        return False
    if nxt is not None and nxt.s == ':' and prev is not None and prev.s in ('(', ','):
        return False
    # a beats list: rule labels
    k = j - 1
    while k >= 0 and (toks[k].s == ',' or (toks[k].t == 'ID' and toks[k].s != 'beats')):
        if toks[k].t == 'ID' and k > 0 and toks[k - 1].s not in (',', 'beats'):
            break
        k -= 1
    if k >= 0 and toks[k].s == 'beats':
        return False
    return True


def retok(t, s):
    x = Tok('ID', s, s, t.ws, t.col)
    x.nm = t.nm
    return x


# -- a member's declarations ----------------------------------------------------------------------------------------

class Decls:
    """The names a file set declares (before renaming), with what each is."""

    def __init__(self, owner):
        self.owner = owner
        self.cat = {}              # name -> category
        self.kinds = {}            # kind name -> kind node
        self.places = {}           # place name -> (kind name, unit, how: ('sec', file, block) | ('enter', line))
        self.things = {}           # thing name -> [(kind name, line, enter node)]
        self.unitkinds = set()
        self.units = set()         # the units it declares, currencies aside (L §6.1: "Any other unit a member declares
        #                            is its own (a.hour)")
        self.indexes = set()
        self.asks = set()
        self.entry_version = {}    # id(entry item) -> the id of the version that holds it

    def kind_chain(self, k):
        seen = []
        while k is not None and k not in seen:
            seen.append(k)
            kn = self.kinds.get(k)
            if kn is None:
                break
            k = kn.parent if kn.spec == 'sub' else None
        return seen

    def is_place_kind(self, k):
        for x in self.kind_chain(k):
            if x in PRELUDE_PLACE or x == 'places':
                return True
            kn = self.kinds.get(x)
            if kn is not None and kn.spec == 'place':
                return True
        return False

    def is_outside_kind(self, k):
        for x in self.kind_chain(k):
            if x == 'outside':
                return True
            kn = self.kinds.get(x)
            if kn is not None and kn.spec == 'place' and kn.outside:
                return True
        return False

    def is_unit_kind(self, k):
        for x in self.kind_chain(k):
            if x == 'currencies':
                return True
            kn = self.kinds.get(x)
            if kn is not None and kn.spec == 'unit':
                return True
        return False


def collect_decls(owner, files, units):
    """files: [(fname, lines)].  Parses each and records its declarations."""
    from .parser import parse_file
    d = Decls(owner)
    for fname, lines in files:
        tree = parse_file(lines, units)
        line_of = dict((l.loc, l) for l in lines)
        bl = blocks(lines)[1:]
        for sec, (b0, b1) in zip(tree.sections, bl):
            k = sec.k
            if k == 'ask':
                d.asks.add(sec.id)
                continue
            if k == 'unitdecl':
                if not sec.cur:
                    d.units.add(sec.id)
                continue
            if k in SECTION_CATS:
                cat = k
                if k == 'input' and getattr(sec, 'observed', False):
                    cat = 'obs'
                d.cat[sec.id] = cat
                if k == 'kind':
                    d.kinds[sec.id] = sec
                if k == 'index':
                    d.indexes.add(sec.id)
                if k == 'place':
                    d.places[sec.id] = (sec.kind, sec.unit, ('sec', fname, b0, b1, sec, id(lines[b0])),
                                       bool(sec.binds))
        for sec, (b0, b1) in zip(tree.sections, bl):
            if sec.k != 'version':
                continue
            for it in sec.items:
                if it.k != 'enter' or it.qual is not None:
                    continue
                kn = it.target
                if kn in d.indexes or kn in ('days', 'months', 'years', 'bank_account'):
                    continue
                d.entry_version[id(it)] = sec.id
                for key in it.keys:
                    name = key.get('id') if key.k == 'name' else None
                    if name is None:
                        continue
                    d.things.setdefault(name, []).append((kn, it, line_of.get(it.loc)))
        for kn in d.kinds:
            if d.is_unit_kind(kn):
                d.unitkinds.add(kn)
    for name, ents in list(d.things.items()):
        kn = ents[0][0]
        if d.is_unit_kind(kn):
            # a key of a unit kind is a unit (L §3.2); a member's is its own, a currency code is one in every member
            del d.things[name]
            d.units.add(name)
            continue
        if d.is_place_kind(kn):
            if name not in d.places:
                d.places[name] = (kn, ents[0][1].unit, ('enter', ents[0][1], ents[0][2]), False)
            d.cat.setdefault(name, 'place')
        else:
            d.cat.setdefault(name, 'thing')
    return d


# -- the joint model -------------------------------------------------------------------------------------------------

def joint_texts(texts, uses, options, pool=None, jfile=None, own=None):
    """Resolve, pin and merge.  Returns (merged lines [(fname, lines)], members loaded alone {alias: Reality},
    JointInfo).  texts: the files given; for a member that is itself a joint model (E §3.1 step 1: "a member that is
    itself a joint model pins its own members through its use lines") the call recurses with its model file (jfile),
    its own files (own) and the files given (pool), where its members are looked up first."""
    from .loader import load_texts, load_lines
    from .parser import prescan_units
    from .lexer import Line
    pool = texts if pool is None else pool
    if jfile is None:
        jfile, own, roots = split_reality(texts)
        if jfile is None:
            raise LoadError('LOAD-HEADER', 'two files have a model header' if roots else 'no file has a model header')
    base = os.path.dirname(jfile)
    on_disk = os.path.isfile(jfile)
    member_files = {}
    for a, f, h in uses:
        if a in member_files:
            raise LoadError('LOAD-DUP', 'use %s twice' % a)
        member_files[a] = member_texts(a, f, os.path.normpath(os.path.join(base, f)), pool, on_disk)
    for a, f, h in uses:
        try:
            pin_check(a, f, member_files[a], h, options, pool, on_disk)
        except LoadError as e:
            if e.code == 'LOAD-PIN':
                raise
            raise MemberRefused(a, e)
        except TallyError as e:
            raise MemberRefused(a, e)
    # each member alone (its asks, a.q; L §6.1 "joining changes no member's answer"): its base, without its
    # scenario files (L §6.4 "The base is whole without its scenarios")
    alone = {}
    sub = {}
    mlines = {}
    mfile = {}
    for a, files in member_files.items():
        base_files = [(fn, src) for fn, rel, src in files if not is_scenario(src)]
        mods = [(fn, src) for fn, src in base_files if is_model(src)]
        # the member's scenarios: its base asks may name them (L §6.4; §5.18 "an ask is checked and answered only in
        # the worlds its line is evaluated in"), and the member alone answers those (ask.Asker.member_ask)
        mworlds = set()
        for fn, rel, src in files:
            if is_scenario(src):
                hm = re.match(r'scenario ([a-z][a-z0-9_]*)', src)
                if hm:
                    mworlds.add(hm.group(1))
        try:
            if len(mods) != 1:
                raise LoadError('LOAD-HEADER', 'two files have a model header' if mods else 'no file has a model '
                                                                                         'header')
            mfile[a] = mods[0][0]
            if header_uses(mods[0][1]) or header_has_sames(mods[0][1]):
                s_merged, s_alone, s_info = joint_texts(base_files, header_uses(mods[0][1]), options, pool=pool,
                                                        jfile=mods[0][0], own=base_files)
                alone[a] = load_lines(s_merged, mworlds or None, members=s_alone, joint=s_info)
                sub[a] = s_info
                mlines[a] = [(fn, [Line(l.level, list(l.toks), l.lineno, l.file, l.text) for l in ls])
                             for fn, ls in s_merged]
            else:
                alone[a] = load_texts(base_files, mworlds or None)
                mlines[a] = [(fn, split_lines(src, fn)) for fn, src in base_files]
        except MemberRefused as e:
            raise MemberRefused(a + '.' + e.alias, e.err)
        except TallyError as e:
            raise MemberRefused(a, e)
        alone[a].scenario_asks = set()
        alone[a].member_worlds = mworlds
        # the member's every file, its scenario files included (its reality, E §3.1 step 1), for an ask in one of
        # its scenario files, asked as a.q by the member alone (ask.Asker.member_ask)
        alone[a].member_all = [(fn, src) for fn, rel, src in files]
        alone[a].member_options = options
        for fn, rel, src in files:
            if is_scenario(src):
                alone[a].scenario_asks |= set(re.findall(r'^ask ([a-z][a-z0-9_]*)\s*:', src, re.M))
    jlines = [(f, split_lines(src, f)) for f, src in own]
    units = prescan_units([ls for f, ls in jlines] + [ls for a in mlines for fn, ls in mlines[a]])
    decls = {a: collect_decls(a, mlines[a], units) for a in mlines}
    tally = None
    for f, ls in jlines:
        if ls and ls[0].toks and ls[0].toks[0].s == 'model':
            tally = header_tally(ls)
    for a in sorted(mlines):
        for fn, ls in mlines[a]:
            if header_tally(ls) != tally:
                raise LoadError('LOAD-VERSION', 'member %s declares tally %s, the joint model %s' % (
                    a, header_tally(ls), tally))
    jf_lines = dict(jlines)[jfile]
    shared_law(mlines, sub, jf_lines)
    # renaming sets: what a member declares, its own units included (L §6.1 "Any other unit a member declares is its
    # own (a.hour)"; currencies are one by their code), less the built-in ids (one in every member, L §6.1)
    rename = {}
    jls = [ls for f, ls in jlines if ls and ls[0].toks and ls[0].toks[0].s == 'model']
    jspan = (header_clause(jls[0], 'start'), header_clause(jls[0], 'end')) if jls else (None, None)
    own_cal = {}
    for a, d in decls.items():
        names = set(n for n in d.cat if n not in BUILTIN_IDS) | d.units
        mls = dict(mlines[a])[mfile[a]]
        mspan = (header_clause(mls, 'start'), header_clause(mls, 'end'))
        if None not in mspan and mspan != jspan:
            # the member's days, months and years are its own (W-E1-F3)
            names |= set(CALENDAR_IDS)
            own_cal[a] = mspan
        rename[a] = names
    # the joint model's reads of a member, a.x (a.b.x of a member's member), as one name, and the keys written as
    # literals after them as the member's things
    keynames = dict((a, set(n for n, c in d.cat.items() if c in ('thing', 'place'))) for a, d in decls.items())
    # a member that is a joint model declares its own names, those its same lines joined to one of its members' too:
    # f.ania is f's ania, the one thing f made of it (L §6.1; E §3.1 step 1; writer 11, the probe writer's family)
    rename_j = dict(rename)
    alias = {}
    for a in sorted(sub):
        own = getattr(sub[a], 'own_alias', {}) or {}
        if own:
            rename_j[a] = set(rename[a]) | set(own)
            for x, r in own.items():
                alias[a + SEP + x] = a + SEP + r
    for f, ls in jlines:
        for l in ls:
            l.toks = rewrite_member_refs(l.toks, rename_j, keynames)
            if alias:
                l.toks = [retok(t, alias[t.s]) if t.t == 'ID' and t.s in alias else t for t in l.toks]
    jdecls = collect_decls('', jlines, units)
    # same lines of things, places and kinds (the rest stay for the loader)
    joins, kept = same_joins(jf_lines, decls, jdecls)
    info = JointInfo()
    info.members = sorted(mlines)
    jmap, drop_secs, drop_keys = plan_joins(joins, decls, jdecls, info)
    info.own_alias = dict((x, r) for x, r in jmap.items() if owner_of(x) == '')
    # a member that is a joint model: its place groups and its same lines of rows and movements, under its prefix
    hoisted = []
    for a in sorted(sub):
        for rep, mems in sub[a].groups.items():
            info.groups[jmap.get(a + SEP + rep, a + SEP + rep)] = [
                ((a + SEP + o) if o else a, a + '.' + nm, out) for o, nm, out in mems]
        for rep, mems in sub[a].things.items():
            info.things[jmap.get(a + SEP + rep, a + SEP + rep)] = [((a + SEP + o) if o else a, a + '.' + nm)
                                                                   for o, nm in mems]
        for (o, rep), v in sub[a].owner_place.items():
            info.owner_place[((a + SEP + o) if o else a, jmap.get(a + SEP + rep, a + SEP + rep))] = v
        mls = dict(mlines[a])[mfile[a]]
        for l in mls[1:]:
            if l.level == 0:
                break
            if l.toks and l.toks[0].s == 'same':
                hoisted.append(Line(l.level, rename_tokens(a, l.toks, rename[a], jmap), l.lineno, l.file, l.text))
    # rename the members' tokens, then apply the joins everywhere
    out = []
    for a in sorted(mlines):
        for fn, ls in mlines[a]:
            ls = member_lines(a, ls, rename[a], jmap, drop_secs.get(a, set()), drop_keys.get(a, {}))
            if a in own_cal and fn == mfile[a]:
                # after the member's part header (its first lines up to the first section)
                k = 1
                while k < len(ls) and ls[k].level > 0:
                    k += 1
                ls = ls[:k] + own_calendar_lines(a, fn, *own_cal[a]) + ls[k:]
            out.append((fn, ls))
    jdrop = drop_secs.get('', set())
    jkeys = drop_keys.get('', {})
    jout = []
    for f, ls in jlines:
        ls = [l for l in ls if id(l) not in kept]
        ls = drop_blocks(ls, f, jdrop, jkeys)
        for l in ls:
            l.toks = [retok(t, jmap[t.s]) if t.t == 'ID' and t.s in jmap else t for t in l.toks]
        if f == jfile and hoisted:
            k = 1
            while k < len(ls) and ls[k].level > 0 and ls[k].toks and ls[k].toks[0].s != 'note':
                k += 1
            ls = ls[:k] + hoisted + ls[k:]
        if f == jfile:
            jout.insert(0, (f, ls))
        else:
            jout.append((f, ls))
    out = jout[:1] + out + jout[1:]
    owners = {}
    for a in mlines:
        for fn, ls in mlines[a]:
            owners[fn] = a
    out = dedupe_units(out, jfile, owners)
    return out, alone, info


def shared_law(mlines, sub, jf_lines):
    """L §6.1 Shared law: "Two members that copy different versions of one law part (identified by the part's id and
    the versions it holds, never by equal names) are refused (LOAD-SAME) unless the joint model states, by a same line,
    which one both follow."  A member's own law parts (a member that is a joint model: its own files)."""
    laws = {}
    for a in sorted(mlines):
        for fn, ls in mlines[a]:
            if not ls or not ls[0].toks or ls[0].toks[0].s != 'part' or len(ls[0].toks) < 2:
                continue
            pid = ls[0].toks[1].s
            if SEP in pid:
                continue
            law = any(l.toks and l.toks[0].s == 'law' for l in ls[1:] if l.level == 1 and not _past_header(ls, l))
            if not law:
                continue
            vs = set(l.toks[1].s for l in ls if l.level == 0 and len(l.toks) > 1 and l.toks[0].s == 'version')
            laws.setdefault(pid, {}).setdefault(a, set()).update(vs)
    for pid in sorted(laws):
        by = laws[pid]
        if len(by) < 2 or len(set(frozenset(v) for v in by.values())) < 2:
            continue
        # L §6.1 Shared law (round 19, GD-24): "Two members that copy different versions of one law part ... are refused
        # (LOAD-SAME)", with no same-line exception
        a, b = sorted(by)[:2]
        raise LoadError('LOAD-SAME', 'members %s and %s copy different versions of law part %s' % (a, b, pid))


def _past_header(ls, l):
    for x in ls[1:]:
        if x.level == 0:
            return True
        if x is l:
            return False
    return True


def rename_tokens(a, toks, names, jmap):
    new = []
    for j, t in enumerate(toks):
        if t.t == 'ID' and t.s in names and t.nm and _rename_ok(toks, j):
            s = a + SEP + t.s
            new.append(retok(t, jmap.get(s, s)))
        else:
            new.append(t)
    return new


class MemberRefused(Exception):
    def __init__(self, alias, err):
        Exception.__init__(self, alias)
        self.alias = alias
        self.err = err


def joint_uses_of(src):
    lines = src.split('\n')
    if not lines or not lines[0].startswith('model '):
        return False
    for l in lines[1:]:
        if not l.startswith((' ', '\t')):
            break
        if re.match(r'\s+(use|same)\s', l):
            return True
    return False


def header_tally(ls):
    for l in ls[1:]:
        if l.level == 0:
            break
        if l.toks and l.toks[0].s == 'tally' and len(l.toks) > 1:
            return l.toks[1].v
    return None


def header_clause(ls, word):
    """The value of a header clause (`start`, `end`) of a file's header lines."""
    for l in ls[1:]:
        if l.level == 0:
            break
        if l.toks and l.toks[0].s == word and len(l.toks) > 1:
            return l.toks[1].v
    return None


CALENDAR_IDS = ('days', 'months', 'years')


def own_calendar_lines(a, fn, S, E):
    """A member's built-in period indexes, its own (L §2.2: built-in ids are every reality's own; §4.1 they are the
    calendar periods that meet the member's [S, E]): declared under its prefix where its run differs from the joint
    model's, so the member reads them as it does alone (L §6.1 A member is closed)."""
    text = '\n'.join([
        'index zzcaldays = day %s..%s' % (S.isoformat(), E.isoformat()),
        'index zzcalmonths = month %04d-%02d..%04d-%02d' % (S.year, S.month, E.year, E.month),
        'index zzcalyears = year %04d..%04d' % (S.year, E.year)]) + '\n'
    ls = split_lines(text, fn)
    for l, name in zip(ls, CALENDAR_IDS):
        l.toks = [retok(t, a + SEP + name) if t.t == 'ID' and t.s.startswith('zzcal') else t for t in l.toks]
    return ls


def rewrite_member_refs(toks, rename, keynames=None):
    """`a . x` (no space) with a a member and x a name a declares -> the one name a·x; `a . b . x`, x a name of a's
    member b, -> a·b·x (a member that is a joint model reads its members' items as b.x, L §6.1).  A key written as a
    literal right after a member's item names that member's thing (L §6.1: "its items are read as a.item (a.wallet,
    a.due[ola, 2025])", "its keys written as literals (... same a.a_stmt[a_g1] = d.g_in)"): keynames {a: the things and
    places a declares}."""
    out = rewrite_member_names(toks, rename)
    if not keynames:
        return out
    res = []
    for j, t in enumerate(out):
        res.append(t)
    j = 0
    while j < len(res):
        t = res[j]
        if t.t == 'ID' and SEP in t.s and j + 1 < len(res) and res[j + 1].s == '[' and not res[j + 1].ws:
            a = t.s.split(SEP, 1)[0]
            pre = t.s.rsplit(SEP, 1)[0]
            rel = pre.split(SEP, 1)[1] + SEP if SEP in pre else ''
            names = keynames.get(a, ())
            depth = 0
            k = j + 1
            while k < len(res):
                s = res[k].s
                if s in ('[', '('):
                    depth += 1
                elif s in (']', ')'):
                    depth -= 1
                    if depth == 0:
                        break
                elif depth == 1 and res[k].t == 'ID' and res[k - 1].s in ('[', ',') and \
                        k + 1 < len(res) and res[k + 1].s in (',', ']') and rel + res[k].s in names:
                    x = retok(res[k], a + SEP + rel + res[k].s)
                    x.nm = True
                    res[k] = x
                k += 1
        j += 1
    return res


def rewrite_member_names(toks, rename):
    out = []
    j = 0
    while j < len(toks):
        t = toks[j]
        if t.t == 'ID' and t.s in rename and not (j > 0 and toks[j - 1].s == '.' and not t.ws):
            chain = []
            k = j + 1
            while k + 1 < len(toks) and toks[k].s == '.' and not toks[k].ws and toks[k + 1].t == 'ID' and \
                    not toks[k + 1].ws:
                chain.append(toks[k + 1].s)
                k += 2
            hit = None
            for m in range(len(chain), 0, -1):
                if SEP.join(chain[:m]) in rename[t.s]:
                    hit = m
                    break
            if hit is not None:
                x = retok(t, t.s + SEP + SEP.join(chain[:hit]))
                x.nm = True
                out.append(x)
                j += 1 + 2 * hit
                continue
        out.append(t)
        j += 1
    return out


def sref_tokens(toks):
    """The two sides of a `same` line's tokens: lists of tokens."""
    body = toks[1:]
    eq = [i for i, t in enumerate(body) if t.s == '=']
    if len(eq) != 1:
        return None, None
    return body[:eq[0]], body[eq[0] + 1:]


def category(name, decls, jdecls):
    o = owner_of(name)
    bare = name.split(SEP, 1)[1] if o else name
    d = decls.get(o) if o else jdecls
    if d is None:
        return None, None
    return d.cat.get(bare), d


def same_joins(jf_lines, decls, jdecls):
    """The header `same` lines naming things, places or kinds (one name on each side, no keys): [(l, r, line)].
    Returns them and the ids of the header lines taken (the others are left to the loader)."""
    joins = []
    kept = set()
    if not jf_lines:
        return joins, kept
    for l in jf_lines[1:]:
        if l.level == 0:
            break
        if not l.toks or l.toks[0].s != 'same':
            continue
        a, b = sref_tokens(l.toks)
        if a is None or len(a) != 1 or len(b) != 1 or a[0].t != 'ID' or b[0].t != 'ID':
            continue
        ca, da = category(a[0].s, decls, jdecls)
        cb, db = category(b[0].s, decls, jdecls)
        if ca in ('kind', 'place', 'thing') or cb in ('kind', 'place', 'thing'):
            if ca != cb:
                # L §6.1 (round 14): "The two sides are of one sort: two kinds declared alike ..., two things of one
                # base kind ..., two rows of one type, or two movements; any other pair is refused (LOAD-SAME)"
                if ca in ('place', 'thing') and cb in ('place', 'thing'):
                    ka, kb = thing_kind(a[0].s, ca, da), thing_kind(b[0].s, cb, db)
                    raise LoadError('LOAD-SAME', 'same %s = %s: things of kinds %s and %s' % (
                        shown(a[0].s), shown(b[0].s), shown(ka), shown(kb)), l.loc)
                raise LoadError('LOAD-SAME', 'same %s = %s: %s and %s' % (shown(a[0].s), shown(b[0].s), sort_name(ca),
                                                                         sort_name(cb)), l.loc)
            joins.append((a[0].s, b[0].s, l))
            kept.add(id(l))
    return joins, kept


SORT_NAMES = {'kind': 'a kind', 'place': 'a place', 'thing': 'a thing', 'input': 'a row', 'obs': 'a movement',
              'move': 'a movement', 'def': 'a def', 'index': 'an index'}


def sort_name(cat):
    return SORT_NAMES.get(cat, 'another item (%s)' % cat if cat else 'another item')


def thing_kind(name, cat, d):
    bare = name.split(SEP, 1)[1] if owner_of(name) else name
    o = owner_of(name)
    if cat == 'place' and d is not None and bare in d.places:
        k = d.places[bare][0]
    elif d is not None and d.things.get(bare):
        k = d.things[bare][0][0]
    else:
        return '?'
    if k is None or k in BUILTIN_IDS or d is None or k not in d.kinds:
        return k
    return (o + SEP + k) if o else k


def decl_keeps_parts(d, bare):
    """Whether a member's (or the joint model's) declared place keeps parts: its kind or a parent states the parts
    clauses (L §3.6, §5.7)."""
    kn = d.places[bare][0]
    seen = set()
    while kn is not None and kn not in seen and kn in d.kinds:
        seen.add(kn)
        kd = d.kinds[kn]
        if getattr(kd, 'pclauses', None):
            return True
        kn = kd.parent
    return False


def plan_joins(joins, decls, jdecls, info):
    """Union the joined names; pick each group's representative; check kinds (LOAD-SAME).  Returns the name map
    (name -> representative), the blocks to drop per owner ({(fname, start)}) and the entry keys to drop per owner
    ({line id: set(names)})."""
    parent = {}

    def find(x):
        while parent.get(x, x) != x:
            x = parent[x]
        return x
    order = []
    for a, b, l in joins:
        for x in (a, b):
            if x not in order:
                order.append(x)
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[rb] = ra
    groups = {}
    for x in order:
        groups.setdefault(find(x), []).append(x)
    lines_of = {}
    for a, b, l in joins:
        lines_of.setdefault(find(a), []).append((a, b, l))
    jmap = {}
    drop_secs = {}
    drop_keys = {}
    # kinds first: a joined kind is one name everywhere, so places and things are compared under it
    kind_groups = [g for g in groups.values() if category(g[0], decls, jdecls)[0] == 'kind']
    for g in kind_groups:
        rep = g[0]
        for x in g[1:]:
            jmap[x] = rep
    for g in kind_groups:
        rep = g[0]
        cr, dr = category(rep, decls, jdecls)
        nr = dr.kinds[rep.split(SEP, 1)[1] if owner_of(rep) else rep]
        for x in g[1:]:
            cx, dx = category(x, decls, jdecls)
            nx = dx.kinds[x.split(SEP, 1)[1] if owner_of(x) else x]
            if kind_sig(nr, owner_of(rep), jmap) != kind_sig(nx, owner_of(x), jmap):
                # L §6.1 (round 14): two kinds are of one sort when "declared alike (the same clauses, legs and
                # attributes, and the same parent ...; their notes aside)"; any other pair is LOAD-SAME
                raise LoadError('LOAD-SAME', 'same %s = %s: kinds declared differently' % (shown(rep), shown(x)))
            drop_secs.setdefault(owner_of(x), set()).add(('kind', x.split(SEP, 1)[1] if owner_of(x) else x))

    def full_kind(owner, k):
        if k is None or k in BUILTIN_IDS:
            return k
        d = decls.get(owner) if owner else jdecls
        if d is not None and k in d.kinds:
            n = (owner + SEP + k) if owner else k
            return jmap.get(n, n)
        return k
    for root, g in groups.items():
        cat, _ = category(g[0], decls, jdecls)
        if cat == 'kind':
            continue
        members = []
        for x in g:
            c, d = category(x, decls, jdecls)
            o = owner_of(x)
            bare = x.split(SEP, 1)[1] if o else x
            if cat == 'place':
                kn, unit, how, keyed = d.places[bare]
                if keyed:
                    # a keyed place is a thing only at its keys (L §6.1 "each side names a kind, a thing (a keyed place
                    # at its keys included) ...; any other pair is refused (LOAD-SAME)"); at its keys it is joined at
                    # load (loader.joint_place_sames)
                    raise LoadError('LOAD-SAME', 'same names %s, a keyed place, without its keys' % shown(x),
                                    lines_of[root][0][2].loc)
                members.append((x, o, bare, full_kind(o, kn), unit, d.is_outside_kind(kn), how, d))
            else:
                # L §6.1 What same does: "A thing's entries are such facts: an entry only one side states stands, and
                # on a day on which the two sides' entries into one kind disagree, whatever depends on the thing being
                # alive there is a HOLE-SAME hole": every entry of every side is kept, under the one name, and read
                # together (model.Model.alive_on); its base kind is its first entry's (L §5.7)
                ents = d.things.get(bare, [])
                firsts = [e for e in ents if not e[1].replace] or ents
                kn, ent, eline = firsts[0]
                members.append((x, o, bare, full_kind(o, kn), None, False, ('enter', ent, eline), d))
        a0, b0, l0 = lines_of[root][0]
        if cat == 'place':
            # L §6.1 (tally 2.1, R-2, C-9): "a place whose kind keeps parts (§3.6) can't be stated same at all
            # (LOAD-PARTS), since two members' arrival rules would book its parts"
            for mm in members:
                if decl_keeps_parts(mm[7], mm[2]):
                    raise LoadError('LOAD-PARTS', 'same %s = %s: %s keeps parts' % (shown(a0), shown(b0), shown(mm[0])),
                                    l0.loc)
            inside = [mm for mm in members if not mm[5]]
            kinds = sorted(set(mm[3] for mm in inside))
            if len(kinds) > 1:
                p = [mm for mm in inside if mm[3] == kinds[0]][0]
                q = [mm for mm in inside if mm[3] == kinds[1]][0]
                raise LoadError('LOAD-SAME', 'same %s = %s: places of kinds %s and %s' % (
                    shown(p[0]), shown(q[0]), shown(p[3]), shown(q[3])), l0.loc)
            units = sorted(set(str(mm[4]) for mm in inside))
            if len(units) > 1:
                # L §6.1 (round 14): "Two places of which neither is outside must also be of one unit and name one
                # price input, if any (LOAD-SAME)"
                raise LoadError('LOAD-SAME', 'same %s = %s: places of two units' % (
                    shown(inside[0][0]), shown(inside[1][0])), l0.loc)
            prices = sorted(set(repr(price_of_decl(mm)) for mm in inside))
            if len(prices) > 1:
                raise LoadError('LOAD-SAME', 'same %s = %s: places of two price inputs' % (
                    shown(inside[0][0]), shown(inside[1][0])), l0.loc)
            rep = inside[0] if inside else members[0]
            info.groups[rep[0]] = [(mm[1], shown(mm[0]), mm[5]) for mm in members]
            for mm in members:
                if not mm[5]:
                    info.owner_place[(mm[1], rep[0])] = True
        else:
            kinds = sorted(set(mm[3] for mm in members))
            if len(kinds) > 1:
                # L §6.1 (round 14): "two things of one base kind"; any other pair is refused (LOAD-SAME)
                raise LoadError('LOAD-SAME', 'same %s = %s: things of kinds %s and %s' % (
                    shown(members[0][0]), shown(members[1][0]), shown(members[0][3]), shown(members[1][3])), l0.loc)
            rep = members[0]
            info.things[rep[0]] = [(mm[1], shown(mm[0])) for mm in members]
            for mm in members:
                if mm is not rep:
                    jmap[mm[0]] = rep[0]
            continue
        for mm in members:
            if mm is rep:
                continue
            jmap[mm[0]] = rep[0]
            how = mm[6]
            if how[0] == 'sec':
                drop_secs.setdefault(mm[1], set()).add(('blockid', how[5]))
            else:
                drop_keys.setdefault(mm[1], {}).setdefault(id(how[2]), set()).add(mm[2])
        # two places: the facts both declarations state must agree (L §6.1: "a fact on which they disagree ... is a
        # hole"); a unit or a price input that differ were refused above (LOAD-SAME)
        for mm in members:
            if mm is rep or mm[5]:
                continue
            if decl_sig(mm, jmap) != decl_sig(rep, jmap) or n_entries(rep) > 1 or n_entries(mm) > 1:
                # L §6.1 What same does: "A thing's entries are such facts: an entry only one side states stands, and
                # on a day on which the two sides' entries into one kind disagree, whatever depends on the thing being
                # alive there is a HOLE-SAME hole": the other side's lifespan is kept, read with the representative's
                # (loader.joint_place_sides; in a binding, model.Model.alive_test; in the walk, its legs, its start and
                # its end)
                x, o, bare, kind, unit, outside, how, d = mm
                if how[0] == 'sec':
                    ents = [(how[4].lives, None, False, how[4].loc)]
                else:
                    ents = [(it.lives, o + SEP + d.entry_version[id(it)], it.replace, it.loc)
                            for kn, it, line in d.things.get(bare, [])]
                info.place_others.setdefault(rep[0], []).append((o, kind, ents))
    return jmap, drop_secs, drop_keys


def price_of_decl(mm):
    """The price input a place names, under its owner (each member's inputs are its own: a.val, b.val), or None."""
    x, o, bare, kind, unit, outside, how, d = mm
    p = how[4].priced if how[0] == 'sec' else how[1].priced
    if p is None:
        return None
    return (o, repr(p))


def kind_sig(n, owner, jmap):
    def f(k):
        if k is None or k in BUILTIN_IDS:
            return k
        full = (owner + SEP + k) if owner else k
        return jmap.get(full, full)
    return (n.spec, f(n.parent), n.certainty, n.unit, n.sign, n.outside, repr(n.priced),
            tuple((x if isinstance(x, str) else repr(x)) for x in n.legs),
            tuple((x if isinstance(x, str) else repr(x)) for x in n.attrs))


def n_entries(mm):
    """How many entries state a place's lifespan: one for a place section, each entry line for an entered place."""
    x, o, bare, kind, unit, outside, how, d = mm
    return 1 if how[0] == 'sec' else len(d.things.get(bare, []))


def decl_sig(mm, jmap):
    """What a place's or thing's declaration states besides its name: its kind, unit, price and lifespan."""
    x, o, bare, kind, unit, outside, how, d = mm
    if how[0] == 'sec':
        sec = how[4]
        return (kind, str(unit), repr(sec.priced), repr(sec.lives))
    ent = how[1]
    return (kind, str(ent.unit), repr(ent.priced), repr(ent.lives))


def member_lines(a, ls, names, jmap, drop, keys):
    """A member's lines under its prefix: its header and its asks dropped, its names renamed, the joins applied."""
    bl = blocks(ls)
    keep = []
    for n, (b0, b1) in enumerate(bl):
        first = ls[b0]
        w = first.toks[0].s if first.toks else ''
        if n == 0:
            # the member's header: its file becomes a part of the joint model (its `on`, `start` and `end` are the
            # joint model's; L §6.1 reads a member at the joint model's cut); a part file of the member stays a part,
            # its id under the member's prefix
            from .lexer import Line
            h0 = ls[b0]
            tl = [l for l in ls[b0 + 1:b1] if l.toks and l.toks[0].s in ('tally', 'law')]
            pid = 'member'
            if w == 'part' and len(first.toks) > 1:
                pid = first.toks[1].s
            keep.append(Line(0, [Tok('ID', 'part', 'part', False, 0), Tok('ID', a + SEP + pid,
                                                                         a + SEP + pid, True, 5)],
                             h0.lineno, h0.file, 'part'))
            keep.extend(tl)
            continue
        if w == 'ask':
            continue                      # its asks are asked as a.q, by the member alone
        if ('blockid', id(first)) in drop:
            continue
        if w == 'kind' and len(first.toks) > 1 and ('kind', first.toks[1].s) in drop:
            continue
        keep.extend(ls[b0:b1])
    out = []
    for l in keep:
        toks = l.toks
        if id(l) in keys:
            # a thing joined to another member's: its entry is that thing's (the declarations agree, checked above)
            toks = drop_entry_keys(toks, keys[id(l)])
            if toks is None:
                continue
        new = []
        for j, t in enumerate(toks):
            if t.t == 'ID' and t.s in names and t.nm and _rename_ok(toks, j):
                # only a token the member's own parse read as an id (W-E1-F1: `index month = month ...`, a def
                # named like a built-in, ids named like record fields stay words where they are words, L §2.2)
                s = a + SEP + t.s
                new.append(retok(t, jmap.get(s, s)))
            else:
                new.append(t)
        l.toks = new
        out.append(l)
    return out


def drop_entry_keys(toks, names):
    """Remove the keys named from an entry line `K + [k1, k2] ...`; None when no key is left."""
    ss = [t.s for t in toks]
    if '[' not in ss or ']' not in ss:
        return toks
    i, j = ss.index('['), ss.index(']')
    keys = [t for t in toks[i + 1:j] if t.s != ',']
    left = [t for t in keys if t.s not in names]
    if not left:
        return None
    mid = []
    for k, t in enumerate(left):
        if k:
            mid.append(Tok('OP', ',', ',', False, t.col))
        mid.append(t)
    return toks[:i + 1] + mid + toks[j:]


def drop_blocks(ls, fname, drop, keys):
    if not drop and not keys:
        return ls
    bl = blocks(ls)
    out = []
    for b0, b1 in bl:
        first = ls[b0]
        w = first.toks[0].s if first.toks else ''
        if ('blockid', id(first)) in drop:
            continue
        if w == 'kind' and len(first.toks) > 1 and ('kind', first.toks[1].s) in drop:
            continue
        for l in ls[b0:b1]:
            if id(l) in keys:
                nt = drop_entry_keys(l.toks, keys[id(l)])
                if nt is None:
                    continue
                l.toks = nt
            out.append(l)
    return out


def dedupe_units(files, jfile, owners=None):
    """A currency declared by several files is one currency when the declarations agree.  L §6.1
    (round 14): "Currencies are one by their code in the joint model and in every member ...; two declarations of one
    code with different minor units are refused (LOAD-SAME). Any other unit a member declares is its own (a.hour)." """
    owners = owners or {}
    seen = {}
    who = {}
    out = []
    for f, ls in files:
        bl = blocks(ls)
        keep = []
        for n, (b0, b1) in enumerate(bl):
            first = ls[b0]
            if n > 0 and first.toks and first.toks[0].s == 'unit' and len(first.toks) > 1 and \
                    first.toks[1].t == 'CUR':
                # a member's other units are renamed under its prefix (a.hour); two of the joint model's own
                # declarations of one id stay for the loader (LOAD-DUP, L §2.5)
                sig = tuple(t.s for t in first.toks)
                uid = first.toks[1].s
                if uid in seen:
                    if seen[uid] != sig:
                        raise LoadError('LOAD-SAME', 'currency %s is declared with different minor units by %s '
                                                     'and %s' % (uid, who[uid], owners.get(f, 'the joint model')))
                    continue
                seen[uid] = sig
                who[uid] = owners.get(f, 'the joint model')
            keep.extend(ls[b0:b1])
        out.append((f, keep))
    return out
