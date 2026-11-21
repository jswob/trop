"""Answering asks (L §5.18, §5.5 fan-out, §8.2 register; E §10).

The printed form.  The spec fixes how values, holes, refusals and register
entries print (L §5.18 Output, §8.1, §8.2) but not the layout of a line; this
interpreter prints, for each line:

    <ask id> (<var> = <key>, ...) [<axis> = <options>, ...]: <result>
      <register entry>
      ...

the binding part only when the ask has a `for`, the axis part only when the
ask reaches an axis.  The register entries follow the order of the table in
L §8.2, each kind sorted by code point, `law date <d>, cut <K>` last.
"""
import datetime
import re

from .errors import TallyError, Unspecified, NotYet, Refusal
from .evaluate import (Engine, Ctx, Coll, TraceNode, HoleSignal, OutKey, ListKey, merge, lin_of, after_on, new_ask,
                       WinDay, PinVal, imprecise_hole)
from .model import key_text, set_world
from .values import (CURRENCIES, fmt_typed, Num, Hole, Iv, Special, Thing, Key, Lst, Text, NONE, vkey, fmt, fmt_hole, fmt_q)
from .check import _txt
from .eval2 import QOut


REG_ORDER = ['assumed', 'given', 'missing', 'pinned', 'pinned sample', 'sample', 'world', 'reading', 'anchor', 'pad',
             'adjustment', 'unfulfilled', 'open', 'left out', 'uncertain', 'held']
MAX_PATHS = 100000   # E §7: N <= 100 000 (REQ-PATHS)


class LoadAtRun(Exception):
    """A load refusal (L §8.3) that this interpreter finds only while evaluating (a value's unit or
    period known only from the rows).  L §8.3: the model does not run, so it stops every ask and prints
    as a load refusal, never per value."""

    def __init__(self, err):
        Exception.__init__(self, err)
        self.err = err


_PIN_DAY = re.compile(r'^(pinned: (?:known|law) .+) = (\d{4}-\d{2}-\d{2})$')


def merge_pin_days(pins):
    """L §8.2 `pinned:`: "The pins one def forms under many as if (a known d at each of its keys, as cpi_yy[m] as if
    {known d}) print once, their days merged into maximal spans". An as if pin given by an expression prints
    `pinned: known <expr> = <day>`; those of one written pin print once, `= <a> .. <b>, <c>` (refcore's layout)."""
    groups = {}
    rest = set()
    for t in pins:
        mt = _PIN_DAY.match(t)
        if mt is None:
            rest.add(t)
            continue
        groups.setdefault(mt.group(1), set()).add(datetime.date.fromisoformat(mt.group(2)))
    for head, days in groups.items():
        if len(days) == 1:
            rest.add('%s = %s' % (head, next(iter(days))))
            continue
        spans = []
        for d in sorted(days):
            if spans and spans[-1][1] + datetime.timedelta(days=1) == d:
                spans[-1][1] = d
            else:
                spans.append([d, d])
        rest.add('%s = %s' % (head, ', '.join(('%s .. %s' % (a, b)) if a != b else str(a) for a, b in spans)))
    return rest


def is_load(err):
    return isinstance(err, TallyError) and err.code.startswith('LOAD-')


def req_ask(qid):
    """E §11 (tally 2.1.3, issue #58): `REQ-ASK` | the request names an ask that doesn't exist | the request is refused,
    after load (a load refusal outranks it), in every shape of reality, printing `no ask <q>`, q the first such ask in
    the request's order, as the run's one result; a member's ask named as asked, a.q (L §6.1)."""
    return TallyError('REQ-ASK', 'no ask %s' % qid)


def req_clock(m, a):
    """E §11 (tally 2.1.7 R5, #112): a cut an asked line's literal states after `on` (`as known D`, `with known D`)
    refuses the request, after load, as REQ-ASK does: the run's one result. A computed one is refused at its value."""
    def lit(n):
        # a literal in ( ) is that literal (E §11 REQ-CLOCK, tally 2.2.1 S-233.8)
        while n is not None and n.k == 'paren':
            n = n.e
        return n if n is not None and n.k == 'date' else None
    lits = []
    if a.known is not None and lit(a.known) is not None:
        lits.append(('as known', lit(a.known)))
    for p in a.pins:
        if getattr(p, 'kind', None) == 'known' and lit(p.value) is not None:
            lits.append(('with known', lit(p.value)))
    for word, n in lits:
        d = datetime.date.fromisoformat(n.v) if isinstance(getattr(n, 'v', None), str) else getattr(n, 'v', None)
        if isinstance(d, datetime.date) and d > m.on:
            raise TallyError('REQ-CLOCK', "%s %s is after the model's on %s" % (word, d, m.on))


def knows_ask(m, qid):
    """Whether the request's id qid names an ask of the loaded reality m (E §11 `REQ-ASK`): one of its asks, refused
    alone or not; or a member's ask a.q (L §6.1) as Asker.member_ask finds it: the member's own, one of its members'
    a.b.q when the member is a joint model, or an ask in one of the member's scenario files."""
    members = getattr(m, 'joint_members', None) or {}
    if '.' in qid and qid.split('.', 1)[0] in members:
        alias, aid = qid.split('.', 1)
        mm = members[alias]
        if any(b.id == aid for b in list(mm.asks) + list(getattr(mm, 'asks_alone', ()))):
            # a member's ask refused alone exists too: its own line (OPERATIONS §2.1; tally 2.2.1, 233.2)
            return True
        if '.' in aid and aid.split('.', 1)[0] in (getattr(mm, 'joint_members', None) or {}):
            return knows_ask(mm, aid)
        return aid in getattr(mm, 'scenario_asks', ())
    return any(a.id == qid for a in list(m.asks) + list(getattr(m, 'asks_alone', ())))


def member_names_world(mm, a):
    """Whether a member's base ask names one of the member's scenarios: its `in`, a diff's worlds, a scenario pin, an
    `as if {scenario s}` in its line or a def it reaches."""
    ws = getattr(mm, 'member_worlds', None)
    if not ws:
        return False
    need = set()
    if a.world is not None:
        need.add(a.world)
    if a.form == 'diff':
        need |= set(w for w in a.diff[2:] if w is not None)
    for p in a.pins:
        if p.kind == 'scenario':
            need.add(p.id)
    from .cli import asif_worlds
    need |= set(asif_worlds(mm, a))
    return bool(need & set(ws))


class Result:
    """One evaluation of an ask line under one axis combination."""
    __slots__ = ('vals', 'coll', 'trace')

    def __init__(self, vals, coll, trace=None):
        self.vals = vals
        self.coll = coll
        self.trace = trace

    def key(self):
        return tuple(rkey(v) for v in self.vals)

    def carry_vals(self):
        return list(self.vals) + ([self.trace] if self.trace is not None else [])


def rkey(v):
    if isinstance(v, TallyError):
        return ('refused', v.code, str(v))
    if isinstance(v, CheckOut):
        return ('check', v.key())
    if isinstance(v, RowSet):
        return ('rows', tuple(sorted(v.rows)))
    if isinstance(v, (Ledgers, DiffOut)):
        return v.key()
    if isinstance(v, QOut):
        return v.key()
    if isinstance(v, Hole):
        return ('hole', v.qs, lin_key(v.lin))
    return vkey(v)


def lin_key(l):
    if not l:
        return None
    k, ts = l
    return (vkey(k) if k is not None else None, tuple((a, vkey(c)) for a, c in ts))


class CheckOut:
    def __init__(self, text, kind, vals=()):
        self.text = text
        self.kind = kind
        self.vals = vals

    def carry_vals(self):
        return self.vals

    def key(self):
        return (self.kind, self.text)


class DiffOut:
    """A diff cell's answer: its head's counts and its lines (L §5.18)."""

    def __init__(self, counted, body, vals=()):
        self.counted = counted
        self.body = body
        self.vals = vals

    def carry_vals(self):
        return self.vals

    def key(self):
        return ('diff', self.counted, tuple(self.body))


class Ledgers:
    """residuals' answer: its head (the place, the span, the grouping) and its lines, each ledger's groups with
    their totals and lines (E §6.3)."""

    def __init__(self, head, lines, vals=()):
        self.head = head
        self.lines = lines
        self.vals = vals

    def carry_vals(self):
        return self.vals

    def key(self):
        return ('ledgers', self.head, tuple(self.lines))


class RowSet:
    def __init__(self, rows, texts, read=None):
        self.rows = rows
        self.texts = texts
        self.read = read          # the value whose rows they are: what the set rests on (tally 2.1.8 R13)

    def carry_vals(self):
        return [self.read]


def day_text(d, inst=None):
    """A day as L §2.4 prints values (tally 2.1.7, R9; F17): a date in ISO; a day that is a hole as the hole, `?`
    with its question; a booking cut an instance never reached (its day a hole or refused) as that day."""
    if d is None and inst is not None:
        d = inst.day if inst.err is None or isinstance(inst.day, Hole) else inst.err
    if isinstance(d, datetime.date):
        return d.isoformat()
    return fmt_value(d)


def fmt_value(v, partial=False, t=None):
    if isinstance(v, TallyError):
        return 'refused: %s %s' % (v.code, v.msg)
    if isinstance(v, Hole):
        if partial and v.lin:
            return fmt_partial(v)
        return fmt_hole(v)
    if isinstance(v, Iv):
        if isinstance(v.lo, datetime.date):
            return fmt(v)
        from .evaluate import imprecise_hole
        return fmt_hole(imprecise_hole(v, 'a print'))
    if isinstance(v, (OutKey,)):
        return 'refused: RUN-KEY %s is outside %s' % (v.text, v.index)
    if isinstance(v, ListKey):
        return v.text
    if isinstance(v, (CheckOut, QOut)):
        return v.text
    if isinstance(v, tuple) and v and v[0] == 'mmdd':
        return '%02d-%02d' % v[1]
    if isinstance(v, Lst) and any(computed_iv(x) for x in v.items):
        # an entry computed from a window is where one value is needed: the hole (L §4.4 l.669; #221, C2)
        return '[%s]' % ', '.join(fmt_value(x) for x in v.items)
    return fmt_typed(v, t)


def computed_iv(x):
    """A value computed from a window that is no window of days (fmt_value prints it as the hole)."""
    return isinstance(x, Iv) and not isinstance(x.lo, datetime.date) or \
        isinstance(x, Lst) and any(computed_iv(y) for y in x.items)


def fmt_bind(b, v):
    """A binding's value in a line's head: a key of a key list prints as written (L §2.5), an integer key too."""
    dom = getattr(b, '_dom', None)
    if dom is not None and dom[0] == 'list' and isinstance(v, Num) and v.isint:
        return str(int(v.q))
    return fmt_value(v)


def fmt_partial(h):
    """412 300.00 PLN + 1 × ?"value of the firm share" (L §5.18 Partial answers)."""
    k, ts = h.lin
    parts = []
    if k is not None:
        parts.append(fmt(k))
    for (text, origin), c in ts:
        q = '?"%s"' % text.replace('\\', '\\\\').replace('"', '\\"')
        if origin:
            q += ' (%s)' % origin
        parts.append('%s × %s' % (fmt(c), q))
    return ' + '.join(parts)


def lin_day(d):
    from .moves import day_lo
    return day_lo(d)


class ExplainLines:
    """An explanation given as text lines (a movement instance, E §10)."""

    def __init__(self, lines):
        self.lines = lines


class Line:
    def __init__(self, env_text, axes_text, vals, coll, reading, trace):
        self.env_text = env_text
        self.axes_text = axes_text
        self.vals = vals
        self.coll = coll
        self.reading = reading
        self.trace = trace
        self.each_vals = None   # the values of every combination the line stands for (group), for its names


def alone_line(a):
    """An ask refused alone (L §6.4, §8.3, GD-18; OPERATIONS §2.1): `<q>: refused: LOAD-SCENARIO …`, the one line
    run and check (§2.3) print for it."""
    return '%s: refused: %s' % (a.id, a.refused_alone.text())


# a member run alone gives its warnings here, as data, not as lines among its results (Asker.member_alone; R20)
WARN_SINK = []


def emit_warnings(out, warnings):
    """A run's lines and then its warnings (L §8.5), or, for a member run alone, the warnings handed to its caller."""
    if WARN_SINK:
        WARN_SINK[-1].extend(warnings)
        return out
    return out + warning_lines(warnings)


def member_ask_node(m, qid):
    """(the member, its ask) a request's a.q names (L §6.1), a nested member's a.b.q too, or None."""
    members = getattr(m, 'joint_members', None) or {}
    alias, aid = qid.split('.', 1)
    mm = members.get(alias)
    if mm is None:
        return None
    for b in list(mm.asks) + list(getattr(mm, 'asks_alone', ())):
        if b.id == aid:
            return mm, b
    if '.' in aid and aid.split('.', 1)[0] in (getattr(mm, 'joint_members', None) or {}):
        return member_ask_node(mm, aid)
    return None


# the walk's warnings (L §8.5), the ones a member's ask brings to the run's level (L §6.1, R20 clause 3)
WALK_WARNINGS = ('WARN-NEGATIVE', 'WARN-LEGS', 'WARN-LABELS')


def joint_warning(alias, code, msg):
    """A member's walk warning by the joint model's names (L §6.1: `a.konto`, R20 clause 3)."""
    import re
    from .joint import SEP
    msg = msg.replace(SEP, '.')
    if code == 'WARN-NEGATIVE':
        return '%s.%s' % (alias, msg)
    if code == 'WARN-LEGS':
        m = re.match(r'^the two observations of (\S+) disagree: (\S+?)\[(.*)\) and (\S+?)\[(.*)$', msg)
        if m:
            return 'the two observations of %s.%s disagree: %s.%s[%s) and %s.%s[%s' % (
                alias, m.group(1), alias, m.group(2), m.group(3), alias, m.group(4), m.group(5))
    if code == 'WARN-LABELS':
        m = re.match(r'^labels dropped at (\S+) from (\S+): (.*)$', msg)
        if m:
            return 'labels dropped at %s.%s from %s: %s' % (alias, m.group(1), m.group(2), ', '.join(
                '%s.%s' % (alias, x) for x in m.group(3).split(', ')))
    return msg


def warning_lines(warnings):
    """L §8.5 (tally 2.1.1): a run's warnings [(code, text)] print sorted by code, then by their text (code point),
    whatever the order of the reality's files.  A joint model's member prefix prints as '.' (cli.joint), so the text
    is compared as it prints."""
    from .joint import SEP
    return ['warning: %s %s' % w for w in sorted(warnings, key=lambda w: (w[0], w[1].replace(SEP, '.')))]


class Asker:
    def __init__(self, m):
        self.m = m
        self.walk_warnings = []
        # the lines a member's ask brings to the run's own level (L §6.1, R20 clause 3): its refused scenarios'
        # lines and its walk's warnings, by the joint model's names, each once per member
        self.level = []
        self.member_warn = []

    def lift(self, alias, scen=(), warns=()):
        """A member's run-level lines, renamed by the joint model's names, each once."""
        for l in scen:
            l = 'scenario %s.%s' % (alias, l[len('scenario '):])
            if l not in self.level:
                self.level.append(l)
        for code, msg in warns:
            if code not in WALK_WARNINGS:
                continue
            w = (code, joint_warning(alias, code, msg))
            if w not in self.member_warn:
                self.member_warn.append(w)

    def lifted_warnings(self):
        """Every member's walk warnings this asker's asks met: its sub-askers' and the members run alone."""
        for alias, sub in sorted(getattr(self, '_member_askers', {}).items()):
            self.lift(alias, (), sub.walk_warnings)
        return list(self.member_warn)

    def run(self, ids=None):
        m = self.m
        asks = {a.id: a for a in list(m.asks) + list(getattr(m, 'asks_alone', ()))}
        members = getattr(m, 'joint_members', {})
        # L §2.5, §8.4 (tally 2.1.7 R5, #106): a literal key at a literal stepped range is load's to refuse, whatever
        # the asks, before REQ-ASK (E §11), not at the first ask answered
        Engine(m).check_range_lits()
        if ids:
            for i in ids:
                if not knows_ask(m, i):
                    # E §11 (tally 2.1.3, issue #58): after load and before any ask is answered, the first id of the
                    # request that names no ask refuses the whole request (was LOAD-NAME, and a member's a.q only
                    # once the asks before it were answered)
                    raise req_ask(i)
            order = []
            for i in ids:
                if '.' in i and i.split('.', 1)[0] in members:
                    # a member's ask, asked inside the joint model as a.q (L §6.1): answered by the member alone
                    order.append(i)
                    continue
                order.append(asks[i])
        else:
            order = [asks[i] for i in sorted(asks)]
        for a in order:
            if not isinstance(a, str):
                req_clock(m, a)
            else:
                # a member's ask is checked against its member's on, as the member alone answers it (L §6.1;
                # tally 2.2.1, 233.3, 228.Q10), unless load left it refused alone
                hit = member_ask_node(m, a)
                if hit is not None and getattr(hit[1], 'refused_alone', None) is None:
                    req_clock(hit[0], hit[1])
        out = []
        try:
            for a in order:
                new_ask()
                if isinstance(a, str):
                    out.extend(self.member_ask(a))
                    continue
                out.extend(self.answer(a))
        except LoadAtRun as x:
            return ['refused: %s' % x.err.text()]
        # the lines a member's ask brings print at the run's own level: its scenarios' before every result, its
        # walk's warnings with the run's (OPERATIONS §2.1, L §6.1, §8.5; R20)
        ws = list(m.warnings) + list(self.walk_warnings)
        ws += [w for w in self.lifted_warnings() if w not in ws]
        return emit_warnings(self.level + out, ws)

    def load_check(self):
        """`check` (OPERATIONS §2.3) on a loaded reality with no scenario file: what run() does before its first ask is
        answered, the load's last check (a literal key at a literal stepped range, L §2.5, §8.4) included, and no ask
        evaluated. Each ask refused alone prints its line as run prints it (alone_line), by id; `ok` alone when there
        is none; then the warnings load decides (WARN-UNREAD; L §8.5's order), never a walk's."""
        m = self.m
        Engine(m).check_range_lits()
        out = [alone_line(a) for a in sorted(getattr(m, 'asks_alone', ()), key=lambda a: a.id)]
        return emit_warnings(out or ['ok'], list(m.warnings))

    def member_alone(self, qid, alias, aid, mm):
        """The member alone, all its files loaded (its scenarios included), answers its ask aid (L §6.1)."""
        from .cli import run_texts
        out = []
        WARN_SINK.append([])
        try:
            lines = run_texts(mm.member_all, [aid], mm.member_options)
        finally:
            warns = WARN_SINK.pop()
        scen = []
        for l in lines:
            if l.startswith('refused: '):
                out.append('%s: %s' % (qid, l))
            elif l.startswith('scenario ') and not out:
                # its scenario's load refused (L §8: that scenario only): a line at the run's own level, by the
                # joint model's name (R20)
                scen.append(l)
            elif not l.startswith(' '):
                out.append(alias + '.' + l)
            else:
                out.append(l)
        self.lift(alias, scen, warns)
        return out

    def member_ask(self, qid):
        alias, aid = qid.split('.', 1)
        mm = self.m.joint_members[alias]
        hit = [b for b in list(mm.asks) + list(getattr(mm, 'asks_alone', ())) if b.id == aid]
        sub = self.__dict__.setdefault('_member_askers', {})
        if not hit and '.' in aid and aid.split('.', 1)[0] in (getattr(mm, 'joint_members', None) or {}):
            # a member that is a joint model asks its members' asks as b.q, so they are asked here as a.b.q (L §6.1)
            if alias not in sub:
                sub[alias] = Asker(mm)
            inner = sub[alias]
            lines = [alias + '.' + l if not l.startswith(' ') else l for l in inner.member_ask(aid)]
            # the nested member's run-level lines, named from here (`scenario a.b.s`, `a.b.konto`; 228.Q9)
            self.lift(alias, inner.level, inner.lifted_warnings())
            return lines
        if not hit:
            if aid in getattr(mm, 'scenario_asks', ()):
                # an ask in one of the member's scenario files: the member is all its files (E §3.1 step 1), so
                # the member alone, its scenarios loaded, answers it in its world (L §6.1 "A member's asks are asked
                # inside the joint model as a.q"; L §6.4 Asking)
                return self.member_alone(qid, alias, aid, mm)
            # the request's ids are checked before any is answered (knows_ask), so this is a guard
            raise req_ask(qid)
        if member_names_world(mm, hit[0]):
            # a base ask of the member that names one of its scenarios (in, diff, a scenario pin, as if): L §6.1 "A
            # member's asks are asked by name in a run of the joint model, as a.q, and answered by the member
            # alone", and L §5.18 checks and answers a line only in the worlds it is evaluated in, so the member
            # alone, with all its files, answers it (MC-E1-F1)
            return self.member_alone(qid, alias, aid, mm)
        if alias not in sub:
            sub[alias] = Asker(mm)
        out = []
        for l in sub[alias].answer(hit[0]):
            # the ask's own lines: its result's head named a.q, the lines under it the member's (L §6.1; 233.1)
            out.append(alias + '.' + l if not l.startswith(' ') else l)
        return out

    # ------------------------------------------------------------------------------------------
    def answer(self, a):
        self.eng_warn = []
        cur = getattr(self.m, 'currencies', None)
        if cur is not None:
            # the global currency table is this reality's (a scenario whose load was refused may have reset it;
            # L §8, §8.3: a load refusal in a scenario file refuses only that scenario)
            CURRENCIES.clear()
            CURRENCIES.update(cur)
        if getattr(a, 'refused_alone', None) is not None:
            # GD-18: this ask's LOAD-SCENARIO refuses it alone (L §6.4, §8)
            return [alone_line(a)]
        try:
            return self._answer(a)
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            return ['%s: refused: %s %s' % (a.id, e.code, e.msg)]
        except (LoadAtRun, KeyboardInterrupt):
            raise
        except RecursionError:
            return ['%s: refused: REFCORE-NOT-YET nesting deeper than this interpreter\'s stack' % a.id]
        except Exception as e:
            # nothing may crash: an input this interpreter can't handle is a refusal
            import os
            if os.environ.get('REFCORE_DEBUG'):
                raise
            return ['%s: refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)'
                    % (a.id, type(e).__name__, e)]
        finally:
            for x in self.eng_warn:
                if x not in self.walk_warnings:
                    self.walk_warnings.append(x)

    def _answer(self, a):
        m = self.m
        if a.paths is not None and a.paths > MAX_PATHS:
            return ['%s: refused: REQ-PATHS %d paths for one ask; the cap is 100 000' % (a.id, a.paths)]
        eng = Engine(m)
        eng.explainer = (self, a)
        self.pin_cells = eng.ask_pin_cells
        self.eng_warn = eng.walk_warnings
        eng.check_range_lits()
        eng.cross = getattr(self, 'cross', None)
        eng.cross_ask = a
        if a.paths is not None:
            eng.ask_paths = (a.paths, a.seed)
        base = Ctx(None, m.on)
        # as known D sets the cut K (L §4.5); law D gives the law date (L §4.3)
        K = m.on
        if a.known is not None:
            v = eng.ev(a.known, {}, base)
            if isinstance(v, Hole):
                # L §5.11: "an ask's law or as known that is a hole make ... the ask line, a hole carrying their
                # questions, and nothing inside is evaluated"
                return ['%s: %s' % (a.id, fmt_value(v))]
            K = eng.date_of(v, 'as known')
            after_on(m, K)
        eng.ask_K = K
        law = None
        # a law that reads its ask-level for is read per tuple (L §5.18: every clause stands on every form; item 17 of
        # the fourth turn; on every form, not only diff: tally 2.2.1, 208.R2), in diff_binds and the tuples' loop
        self._tuple_law = a.law if (a.law is not None and law_reads_binds(a)) else None
        if a.law is not None and self._tuple_law is None:
            v = eng.ev(a.law, {}, Ctx(None, K))
            if isinstance(v, Hole):
                return ['%s: %s' % (a.id, fmt_value(v))]
            law = eng.date_of(v, 'the law of an ask')
        # in S names the world (L §5.18, §6.4); the model's own id is the base
        w0 = a.world if (a.world is not None and a.world != m.header.id) else None
        ctx = Ctx(law, K, partial=a.partial, unc=bool(a.uncertain), world=w0)
        if a.random is not None:
            # the line's random clause, for every projection it reaches (L §5.18)
            ctx = ctx.but(rnd=eng.rnd_clause(a.random, ctx))
        pins_coll = Coll()
        eng.stack.append(pins_coll)
        self._outside_ctx = ctx
        try:
            if a.world is not None:
                eng.reg(('world', a.world))
            try:
                ctx = eng.apply_pins(a.pins, {}, ctx, ask=True)
            except HoleSignal as h:
                # a pin whose key or date is a hole makes the ask line that hole (L §5.11)
                return ['%s: %s' % (a.id, fmt_value(h.hole))]
        finally:
            eng.stack.pop()
        eng.ask_K = ctx.cut
        old_ws = set_world(eng.world_chain(ctx.world))
        try:
            out = self._answer_in(eng, a, ctx, pins_coll)
        finally:
            set_world(old_ws)
        # E §9 invariant 4: asks move no money; the timelines the ask walked are as they were when walked
        for w in list(eng.walks.values()):
            sig = getattr(w, 'sig', None)
            if sig is not None and getattr(w, 'failed', None) is None and w.signature() != sig:
                return ['%s: refused: ENGINE-INVARIANT asks move no money: the timeline at cut %s changed while the '
                        'ask was answered (E §9 invariant 4)' % (a.id, w.K)]
        return out

    def _answer_in(self, eng, a, ctx, pins_coll):
        m = self.m
        if a.form == 'diff':
            return self.answer_diff(eng, a, ctx, pins_coll)
        if a.path is not None:
            # path p replays path p of the run (L §5.16)
            ctx = ctx.but(path=(a.seed, a.path))
        pinned_axes = dict(ctx.axes)
        axes = [ax for ax in getattr(a, '_axes', []) if ax not in pinned_axes]
        combos = [()]
        for ax in axes:
            opts = m.axes[ax].options
            combos = [c + ((ax, o),) for c in combos for o in opts]
        if a.binds and axes and a.form != 'readers-of':
            return self.binds_over_axes(eng, a, ctx, pins_coll, pinned_axes, axes, combos)
        # binding tuples (L §5.18: one line per binding tuple)
        tuples = []
        tregs = {}
        if a.binds:
            # each tuple keeps what its members brought (an unfulfilled instance it binds, L §8.2), for its own line
            mem = list(eng.members(a.binds, {}, ctx, regs=[]))
            for env, st, rg in mem:
                if isinstance(st, tuple) and st[0] == 'domainhole':
                    return ['%s: %s' % (a.id, fmt_value(st[1]))]
                # L §5.18: "A tuple whose where is a hole prints its line as that hole, its value unevaluated"
                tuples.append((env, st if isinstance(st, Hole) else None))
                tregs[id(env)] = rg
        else:
            tuples.append(({}, None))
        out = []
        if a.form == 'readers-of':
            return self.readers_of(a)
        for env, wh in tuples:
            env_text = ''
            if a.binds:
                env_text = ' (%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in a.binds)
            if wh is not None:
                out.append('%s%s: %s' % (a.id, env_text, fmt_value(wh)))
                continue
            tctx = ctx
            if getattr(self, '_tuple_law', None) is not None:
                # the tuple's law date (208.R2, as diff_binds): a hole makes its line that hole
                v = eng.ev(self._tuple_law, env, Ctx(None, ctx.cut))
                if isinstance(v, Hole):
                    out.append('%s%s: %s' % (a.id, env_text, fmt_value(v)))
                    continue
                tctx = ctx.but(law=eng.date_of(v, 'the law of an ask'))
            results = {}
            for c in combos:
                cctx = tctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
                results[c] = self.evaluate(eng, a, env, cctx)
            lines = self.group(a, axes, combos, results)
            for ln in lines:
                for rc in tregs.get(id(env), ()):
                    ln.coll.merge(rc)
                out.extend(self.print_line(a, env_text, ln, pins_coll, tctx))
        return out

    def binds_over_axes(self, eng, a, ctx, pins_coll, pinned_axes, axes, combos):
        """An ask whose evaluation, its `for … where` included, reaches axes: every combination of them is evaluated
        (L §5.5), the binding tuples with it; one line per binding tuple and per distinct result across the
        combinations where that tuple is bound (L §5.18 Lines)."""
        order = []
        per = {}
        wholes = {}
        for c in combos:
            cctx = ctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
            mem = list(eng.members(a.binds, {}, cctx, regs=[]))
            prev = -1
            for env, st, rg in mem:
                if isinstance(st, tuple) and st[0] == 'domainhole':
                    return ['%s: %s' % (a.id, fmt_value(st[1]))]
                key = tuple(vkey(env[b.var]) for b in a.binds)
                if isinstance(st, Hole):
                    # L §5.18: "A tuple whose where is a hole prints its line as that hole, its value unevaluated"
                    wholes.setdefault(key, []).append(st)
                if key not in per:
                    per[key] = (env, [], {})
                    order.insert(prev + 1, key)
                prev = order.index(key)
                per[key][1].append(c)
                # each combination binds its own tuple: a movement bound is that combination's (L §5.5, §5.18;
                # tally 2.1.8 R13, #169-k)
                per[key][2][c] = (env, rg)
        out = []
        for key in order:
            env, cs, envs = per[key]
            env_text = ' (%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in a.binds)
            if key in wholes:
                out.append('%s%s: %s' % (a.id, env_text, fmt_value(merge(*wholes[key]))))
                continue
            tctx = ctx
            if getattr(self, '_tuple_law', None) is not None:
                # the tuple's law date (208.R2, as diff_binds): a hole makes its line that hole
                v = eng.ev(self._tuple_law, env, Ctx(None, ctx.cut))
                if isinstance(v, Hole):
                    out.append('%s%s: %s' % (a.id, env_text, fmt_value(v)))
                    continue
                tctx = ctx.but(law=eng.date_of(v, 'the law of an ask'))
            results = {}
            for c in cs:
                cctx = tctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
                cenv, rg = envs[c]
                results[c] = self.evaluate(eng, a, cenv, cctx)
                # what the tuple's members brought in that combination (an unfulfilled instance it binds, L §8.2)
                for rc in rg:
                    results[c].coll.merge(rc)
            for ln in self.group(a, axes, cs, results):
                out.extend(self.print_line(a, env_text, ln, pins_coll, tctx))
        return out

    # -- diff (L §5.18, §6.4) -------------------------------------------------------------------------
    def diff_axes(self, a, ctx):
        """The axes a diff fans out over (L §5.5): those its cone reaches, less its pins."""
        return [ax for ax in getattr(a, '_axes', []) if ax not in dict(ctx.axes)]

    def diff_combos(self, axes):
        combos = [()]
        for ax in axes:
            combos = [c + ((ax, o),) for c in combos for o in self.m.axes[ax].options]
        return combos

    def answer_diff(self, eng, a, ctx, pins_coll):
        """diff(e [for binds]) in S against B, evaluated per combination of the axes it reads (L §5.5, tally 2.1.7
        R8): each combination a cell, `diff_worlds`; the cells' results printed by `diff_print`. On a stack that holds
        one of the two worlds (cli.cross_stack_diff) the cells of that side are kept for the caller instead."""
        if a.binds and getattr(self, '_diff_env', None) is None:
            return self.diff_binds(eng, a, ctx, pins_coll)
        axes = self.diff_axes(a, ctx)
        pinned = list(ctx.axes)
        cells = {}
        for c in self.diff_combos(axes):
            cctx = ctx.but(axes=tuple(sorted(pinned + list(c)))) if c else ctx
            cells[c] = self.diff_worlds(eng, a, cctx, pins_coll)
        if getattr(self, '_diff_only', None) is not None:
            self._diff_side = (axes, cells, ctx)
            return []
        return self.diff_print(a, axes, cells, ctx)

    def diff_binds(self, eng, a, ctx, pins_coll):
        """A diff with an ask-level `for`: the body per binding tuple (L §5.18; tally 2.1.8 R13.1, item 3 of the
        follow-up), each tuple's block under the head naming it, `h (k = b): diff in …`; a tuple whose where is a
        hole prints its line as that hole."""
        out = []
        only = getattr(self, '_diff_only', None) is not None
        tsides = []
        for env, st in list(eng.members(a.binds, {}, ctx)):
            if isinstance(st, tuple) and st[0] == 'domainhole':
                out.append('%s: %s' % (a.id, fmt_value(st[1])))
                tsides.append(('', ('line', out[-1:])))
                continue
            txt = ' (%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in a.binds)
            if isinstance(st, Hole):
                out.append('%s%s: %s' % (a.id, txt, fmt_value(st)))
                tsides.append((txt, ('line', out[-1:])))
                continue
            tctx = ctx
            if getattr(self, '_tuple_law', None) is not None:
                # the tuple's law date (item 17 of the fourth turn): a hole makes its line that hole
                v = eng.ev(self._tuple_law, env, Ctx(None, ctx.cut))
                if isinstance(v, Hole):
                    out.append('%s%s: %s' % (a.id, txt, fmt_value(v)))
                    tsides.append((txt, ('line', out[-1:])))
                    continue
                tctx = ctx.but(law=eng.date_of(v, 'the law of an ask'))
            self._diff_env = env
            self._diff_side = None
            try:
                lines = self.answer_diff(eng, a, tctx, pins_coll)
            finally:
                self._diff_env = None
            if only:
                # one side of a diff over two stacks: each tuple's cells, for cli.cross_stack_diff
                tsides.append((txt, self._diff_side if self._diff_side is not None else ('line', lines)))
                continue
            out.extend(a.id + txt + ln[len(a.id):] if ln.startswith(a.id) else ln for ln in lines)
        if only:
            self._diff_side = ('binds', tsides)
            return []
        return out

    def diff_worlds(self, eng, a, ctx, pins_coll):
        """One cell of a diff: e in world S and in world B (or the one this stack holds), line by line keyed by the
        binding tuple (the expression's text without binds). Returns ({world: [(key, value)]}, coll, ctx, type)."""
        m = self.m
        e, bs, sw, bw = a.diff
        coll = Coll()
        coll.merge(pins_coll)
        res = {}
        only = getattr(self, '_diff_only', None)
        base = getattr(self, '_diff_env', None) or {}
        for w in (sw, bw):
            if only is not None and w != only:
                continue
            wv = None if w == m.header.id else w
            wctx = ctx.but(world=wv)
            old = set_world(eng.world_chain(wv))
            eng.stack.append(coll)
            try:
                eng.reg(('world', w))
                lines = []
                if bs:
                    mem = list(eng.members(bs, dict(base), wctx))
                    # a thing only another world declares is no member here: what only S declares is `added` (L §5.18's
                    # diff row, §6.4; tally 2.2.1, 231.R2)
                    from .moves import item_in_world
                    mem = [(env, st) for env, st in mem
                           if not any(isinstance(env.get(b.var), Thing) and env[b.var].name in m.places and
                                      not item_in_world(m.places[env[b.var].name]) for b in bs)]
                    if not self.declared_in_world(e):
                        # a line whose expression names a thing this world doesn't declare: none of its tuples has a
                        # line here, so each is `added` (L §5.18's diff row; tally 2.2.1 P37, Q6)
                        mem = []
                    for env, st in mem:
                        if isinstance(st, tuple) and st[0] == 'domainhole':
                            lines.append((_txt(e), st[1]))
                            continue
                        key = '(%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in bs)
                        if isinstance(st, Hole):
                            # "and so does a diff line in the world where its where is a hole" (L §5.18)
                            lines.append((key, st))
                            continue
                        lines.append((key, self.one(eng, e, env, wctx)))
                elif self.declared_in_world(e):
                    # what only the scenario declares is `added` (L §5.18's diff row; tally 2.2.1, #231): a world
                    # that doesn't declare an item the expression reads has no line for it
                    lines.append((_txt(e), self.one(eng, e, dict(base), wctx)))
            finally:
                eng.stack.pop()
                set_world(old)
            res[w] = lines
        return res, coll, ctx, getattr(e, '_t', None)

    def declared_in_world(self, e):
        """Whether every item e names is declared in the world being evaluated (L §6.4: an item declared in a
        scenario file exists only in that scenario and those over it)."""
        from .node import walk as nodes
        from .moves import item_in_world
        from .model import in_world
        for n in nodes(e):
            if n.k == 'name':
                hit = self.m.lookup(n.id)
                if hit is not None and not item_in_world(hit[1]):
                    return False
            t = self.m.things.get(getattr(n, 'id', None)) if isinstance(getattr(n, 'id', None), str) else None
            if t is not None and getattr(t, 'entries', None) and \
                    not any(x.version is None or in_world(x.version) for x in t.entries):
                # a kind's entry only another world states: what only S states is `added` (tally 2.2.1 P43 S7)
                return False
        return True

    def diff_lines(self, a, res, ctx, t):
        """A cell's result, the set of its lines (L §5.18): `added` (only in S), `removed` (only in B), `changed` with
        both values, the unchanged ones counted. Returns (the head's counts, the lines)."""
        e, bs, sw, bw = a.diff
        sl = dict(res[sw])
        bl = dict(res[bw])
        out_lines = []
        counts = {'changed': 0, 'added': 0, 'removed': 0, 'unchanged': 0}
        for key, v in res[sw]:
            if key in bl:
                if rkey(v) == rkey(bl[key]):
                    counts['unchanged'] += 1
                else:
                    counts['changed'] += 1
                    out_lines.append('  changed %s: %s against %s' % (key, fmt_value(v, ctx.partial, t),
                                                                    fmt_value(bl[key], ctx.partial, t)))
            else:
                counts['added'] += 1
                out_lines.append('  added %s: %s' % (key, fmt_value(v, ctx.partial, t)))
        for key, v in res[bw]:
            if key not in sl:
                counts['removed'] += 1
                out_lines.append('  removed %s: %s' % (key, fmt_value(v, ctx.partial, t)))
        counted = 'diff in %s against %s: %d changed, %d added, %d removed, %d unchanged' % (
            sw, bw, counts['changed'], counts['added'], counts['removed'], counts['unchanged'])
        return counted, tuple(out_lines)

    def diff_print(self, a, axes, cells, ctx):
        """Print a diff's cells (L §5.5): one line per distinct result, the combinations giving it split into products
        and named as a value's are (`group`); no head when it reads no axis. Layout (this interpreter's): a head line
        with the counts, one line per difference, then the register of the evaluations it stands for."""
        m = self.m
        results = {c: self.diff_lines(a, res, cctx, t) for c, (res, coll, cctx, t) in cells.items()}
        if not axes:
            counted, body = results[()]
            return (['%s: %s' % (a.id, counted)] + list(body) +
                    ['  ' + r for r in self.register(cells[()][1], [], ctx, unknown=diff_days(cells[()][0]))])
        by, order = {}, []
        for c in self.diff_combos(axes):
            if results[c] not in by:
                by[results[c]] = []
                order.append(results[c])
            by[results[c]].append(c)
        out = []
        for k in order:
            cs = by[k]
            for prod in split(axes, [dict(c) for c in cs], m):
                coll = Coll()
                days = set()
                for c in cs:
                    d = dict(c)
                    if all(d[ax] in opts for ax, opts in prod):
                        coll.merge(cells[c][1])
                        days |= diff_days(cells[c][0])
                lab, reading, sk = [], [], []
                for ax, opts in prod:
                    allo = m.axes[ax].options
                    ol = [o for o in allo if o in opts]
                    whole = len(ol) == len(allo)
                    lab.append('%s = %s' % (ax, '*' if whole else ' | '.join(ol)))
                    if not whole:
                        reading.append('%s = %s' % (ax, ' | '.join(ol)))
                    sk.append(-1 if whole else allo.index(ol[0]))
                out.append((sk, ['%s [%s]: %s' % (a.id, ', '.join(lab), k[0])] + list(k[1]) +
                            ['  ' + r for r in self.register(coll, reading, ctx, unknown=days)]))
        # order: by the axes' options in declaration order, axes in id order; * first (as `group`)
        out.sort(key=lambda x: x[0])
        return [l for _, ls in out for l in ls]

    # -- residuals (E §6.3) ----------------------------------------------------------------------------
    def ledgers(self, eng, a, env, ctx):
        """residuals x span [by f]: the ledger's groups, each with its totals and its lines, as one answer of the
        line, so a binding and an axis fan it out as any answer (L §5.5, §5.18; tally 2.1.8 R13, #185,
        #186).  Layout (this interpreter's): per group `<month> <group>: model M, observed O, difference D`
        and its lines indented under it; a place of unit any's ledgers each under `ledger <u>:`."""
        r, sp, by = a.resid
        p = eng.place_arg(r, env, ctx)
        if isinstance(p, Hole):
            return p
        span = eng.span_value(sp, env, ctx)
        if isinstance(span, Hole):
            return span
        try:
            pu = eng.m.place_unit(eng.m.things[p.name])
            if pu == 'any':
                # E §6.3 (round 12): "A place of unit any gives one ledger per unit, in the order of the units'
                # ids" (writer 5's T-101)
                ledgers = []
                for u in eng.residual_units(p.name, span, ctx):
                    groups, unit = eng.residuals(p.name, span, by, ctx, unit=u)
                    ledgers.append((u, groups, unit))
            else:
                groups, unit = eng.residuals(p.name, span, by, ctx)
                ledgers = [(None, groups, unit)]
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return err
        out = []
        vals = []
        for u, groups, unit in ledgers:
            ind = '  '
            if u is not None:
                out.append('  ledger %s:' % u)
                ind = '    '
            for g, mt, ot, dt, ls in groups:
                vals += [mt, ot, dt] + [x for ln in ls for x in (ln.model, ln.observed, ln.diff, ln.day,
                                                           getattr(ln, 'carry_day', None))]
                out.append('%s%s: model %s, observed %s, difference %s' % (
                    ind, ('%s %s' % (g[0].label(), g[1])) if g[0] is not None else g[1], eng.rtext(mt, unit),
                    eng.rtext(ot, unit), eng.rtext(dt, unit)))
                for ln in ls:
                    out.append(ind + '  ' + ln.text)
        # the head's span with each bound variable printed as its value (P24; P48), never the variable
        from .evaluate import env_txt
        return Ledgers('%s %s%s' % (p.name, env_txt(sp, env) if env else _txt(sp), (' by %s' % by) if by else ''),
                       out, vals)

    def evaluate(self, eng, a, env, ctx):
        coll = Coll()
        eng.stack.append(coll)
        try:
            f = a.form
            if f == 'exprs':
                vals = tuple(self.one(eng, e, env, ctx, line=True) for e in a.exprs)
                trace = None
            elif f == 'solve':
                vals = (self.one_solve(eng, a, env, ctx),)
                trace = None
            elif f == 'residuals':
                vals = (self.ledgers(eng, a, env, ctx),)
                trace = None
            elif f == 'check':
                vals = (self.check(eng, a, env, ctx),)
                trace = None
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'moveref':
                # a movement instance (E §10)
                v = self.one(eng, a.ref, env, ctx)
                vals = (v,)
                trace = ExplainLines(self.explain_instance(eng, v, ctx))
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'fillread':
                # a fill take (E §10)
                v, lines = self.explain_fill(eng, a, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'read' and \
                    a.ref._r[1] == 'def' and a.ref._r[2] in getattr(self.m, 'walk_bound', ()):
                # a walk-bound def (E §10, L §5.18): its cell at each walk point that read it
                v, lines = self.explain_walk_bound(eng, a, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain' and a.ref.k == 'bal':
                # a balance or a projection (L §5.18 explain row, E §10)
                v, lines = self.explain_balance(eng, a, a.ref, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain':
                eng.tracing = True
                eng.trace_cells = {}
                eng.memo_saved = eng.memo
                eng.memo = {}
                root = TraceNode('root', _txt(a.ref))
                eng.trace_stack = [root]
                try:
                    v = self.one(eng, a.ref, env, ctx)
                finally:
                    eng.tracing = False
                    eng.trace_stack = []
                    eng.memo = eng.memo_saved
                root.value = v
                if a.pins and getattr(self, '_outside_ctx', None) is not None:
                    root = self.explain_pins(eng, a, env, ctx, root, v)
                vals = (v,)
                trace = root
            elif f == 'inputs-of':
                sub = Coll()
                eng.stack.append(sub)
                try:
                    v = self.one(eng, a.ref, env, ctx)
                finally:
                    eng.stack.pop()
                coll.merge(sub)
                texts = {}
                for sid, r in sub.rows.items():
                    texts[sid] = self.row_line(r)
                vals = (RowSet(set(texts), texts, v),)
                trace = None
            else:
                raise NotYet('ask form %s' % f)
        finally:
            eng.stack.pop()
        return Result(vals, coll, trace)

    def one(self, eng, e, env, ctx, line=False):
        try:
            if line and is_query_line(e):
                return eng.query_line(e, env, ctx)
            return eng.ev(e, env, ctx)
        except HoleSignal as h:
            return h.hole
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return err
        except WinDay as wd:
            return self.window_days(eng, e, env, ctx, line, wd)

    def window_days(self, eng, e, env, ctx, line, wd):
        """A line that meets a decision dated by a window, read once per day of the window, the cell its day is read
        from pinned to that day (E §4.1 step 2: it counts in flows when the window lies inside the span; L §4.4 l.668:
        the window's days in turn; tally 2.2.1 P30, N3): one value when they agree, the refusal when every day refuses,
        else the window's hole. A window no cell names is its hole."""
        import datetime
        iv = wd.iv
        nm = iv.name
        if nm is not None and nm[0] == 'row' and '[' not in nm[2]:
            item, keys = nm[2], None
        elif nm is not None and nm[0] == 'pin':
            item, keys = nm[1], (nm[2] or None)
        else:
            return imprecise_hole(iv, wd.use)
        days = sorted(iv.days) if iv.days is not None else [
            iv.lo + datetime.timedelta(days=k) for k in range((iv.hi - iv.lo).days + 1)]
        layer = 1 + max((p.layer for p in ctx.pins), default=0)
        outs = []
        for d in days:
            pv = PinVal('solve', item, keys, (), d, '', layer)
            c2 = ctx.but(pins=tuple(sorted(list(ctx.pins) + [pv], key=lambda p: repr(p.key()))))
            outs.append(self.one(eng, e, env, c2, line))
        errs = [r for r in outs if isinstance(r, TallyError)]
        if errs:
            return errs[0] if len(errs) == len(outs) else imprecise_hole(iv, wd.use)
        holes = [r for r in outs if isinstance(r, Hole)]
        if holes:
            return merge(*holes)
        if len(set(repr(vkey(r)) for r in outs)) == 1:
            return outs[0]
        return imprecise_hole(iv, wd.use)

    def one_solve(self, eng, a, env, ctx):
        try:
            return eng.solve_value(a.solve, env, ctx)
        except HoleSignal as h:
            return h.hole
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return err

    def check(self, eng, a, env, ctx):
        """check e == f: ok, or differs with both values and e − f (L §5.18)."""
        l, r = a.check
        out = self._check(eng, a, env, ctx)
        # "Sides read at different law dates list each" (L §5.18): a side's own `as if {law D}`
        laws = (self.side_law(eng, l, env, ctx), self.side_law(eng, r, env, ctx))
        if laws[0] != laws[1]:
            out.laws = laws
        return out

    def side_law(self, eng, e, env, ctx):
        while e.k == 'paren':
            e = e.e
        if e.k == 'asif':
            for p in e.pins:
                if p.kind == 'law':
                    try:
                        v = eng.ev(p.value, env, ctx)
                        return eng.date_of(v, 'the law pin')
                    except (TallyError, HoleSignal):
                        return None
        return ctx.law

    def _check(self, eng, a, env, ctx):
        l, r = a.check
        lv = self.one(eng, l, env, ctx)
        rv = self.one(eng, r, env, ctx)
        lt, rt = _txt(l), _txt(r)
        if isinstance(lv, (TallyError, Hole)) or isinstance(rv, (TallyError, Hole)):
            return CheckOut('%s = %s, %s = %s' % (lt, fmt_value(lv, ctx.partial), rt, fmt_value(rv, ctx.partial)),
                            'open', (lv, rv))
        try:
            same = eng.compare('==', lv, rv, None, ctx)
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return CheckOut('refused: %s %s' % (err.code, err.msg), 'refused')
        if isinstance(same, Hole):
            return CheckOut('%s = %s, %s = %s' % (lt, fmt_value(lv), rt, fmt_value(rv)), 'open', (lv, rv, same))
        if same:
            return CheckOut('ok', 'ok')
        diff = ''
        if isinstance(lv, Num) and isinstance(rv, Num):
            from .evaluate import arith
            try:
                d = arith('-', lv, rv, l._t, l._t, r._t)
                diff = ', difference %s' % fmt(d)
            except TallyError as err:
                if is_load(err):
                    raise LoadAtRun(err)
                diff = ', difference refused: %s %s' % (err.code, err.msg)
        else:
            # L §5.18 (round 12): "and, where e - f is defined (§3.2, §4.2), the difference e − f": of two keys of one
            # ordered index an int (L §4.2); of anything else (dates, periods off an index, text, codes) none
            ltt, rtt = getattr(l, '_t', None), getattr(r, '_t', None)
            if ltt is not None and rtt is not None and ltt.k == 'key' and rtt.k == 'key' and ltt.ix == rtt.ix \
                    and ltt.ix in eng.m.indexes:
                fake = type('N', (), {})()
                fake.l = l
                try:
                    diff = ', difference %s' % fmt(eng.key_diff(fake, lv, rv))
                except TallyError as err:
                    if is_load(err):
                        raise LoadAtRun(err)
                    diff = ', difference refused: %s %s' % (err.code, err.msg)
        return CheckOut('differs: %s = %s, %s = %s%s' % (lt, fmt(lv), rt, fmt(rv), diff), 'differs')

    # -- axes (L §5.5) --------------------------------------------------------------------------
    def group(self, a, axes, combos, results):
        """One line per distinct result; each set of combinations split into products by
        the axes in id order (L §5.5)."""
        by = {}
        order = []
        for c in combos:
            k = results[c].key()
            if k not in by:
                by[k] = []
                order.append(k)
            by[k].append(c)
        lines = []
        for k in order:
            cs = by[k]
            for prod in split(axes, [dict(c) for c in cs], self.m):
                coll = Coll()
                first = None
                each_vals = []
                for c in cs:
                    d = dict(c)
                    if all(d[ax] in opts for ax, opts in prod):
                        coll.merge(results[c].coll)
                        each_vals.append(results[c].vals)
                        if first is None:
                            first = results[c]
                axes_parts = []
                reading = []
                for ax, opts in prod:
                    allo = self.m.axes[ax].options
                    if len(opts) == len(allo):
                        axes_parts.append((ax, None))
                    else:
                        ol = [o for o in allo if o in opts]
                        axes_parts.append((ax, ol))
                        reading.append('%s = %s' % (ax, ' | '.join(ol)))
                ln = Line(None, axes_parts, first.vals, coll, reading, first.trace)
                # its unknown days are the union of the combinations' it stands for, as its register is
                # (L §8.2, §5.5; tally 2.2, pick P26)
                ln.each_vals = each_vals
                lines.append(ln)
        # order: by the axes' options in declaration order, axes in id order; * first
        def sk(ln):
            out = []
            for ax, ol in ln.axes_text:
                if ol is None:
                    out.append(-1)
                else:
                    out.append(self.m.axes[ax].options.index(ol[0]))
            return out
        lines.sort(key=sk)
        return lines

    # -- printing ---------------------------------------------------------------------------------
    def print_line(self, a, env_text, ln, pins_coll, ctx):
        ax = ''
        if ln.axes_text:
            ax = ' [%s]' % ', '.join('%s = %s' % (x, '*' if ol is None else ' | '.join(ol)) for x, ol in ln.axes_text)
        head = '%s%s%s' % (a.id, env_text, ax)
        vals = ln.vals
        out = []
        if a.form == 'residuals':
            if not isinstance(vals[0], Ledgers):
                # a ledger whose place or span is a hole, or that is refused, is that line alone
                return ['%s: %s' % (head, fmt_value(vals[0]))]
            out.append('%s: residuals %s' % (head, vals[0].head))
            out.extend(vals[0].lines)
        elif a.form == 'inputs-of':
            rs = vals[0]
            out.append('%s:' % head)
            for sid in sorted(rs.rows):
                out.append('  %s' % rs.texts[sid])
        elif a.form == 'explain':
            out.append('%s: %s' % (head, fmt_value(vals[0], ctx.partial)))
            if a.path is not None:
                # a path value: "the seed and path number, the samples it reached in canonical order" (E §10; the
                # samples are the register's sample: lines)
                out.append('  path %d of seed %d' % (a.path, a.seed))
            out.extend(self.explain_lines(ln.trace, ctx))
        elif len(vals) == 1:
            out.append('%s: %s' % (head, fmt_value(vals[0], ctx.partial, self.etype(a, 0))))
        else:
            out.append('%s: (%s)' % (head, ', '.join(fmt_value(v, ctx.partial, self.etype(a, i))
                                                      for i, v in enumerate(vals))))
        coll = Coll()
        coll.merge(pins_coll)
        coll.merge(ln.coll)
        # L §8.2 One read set (tally 2.1.7 R7; T's SR-6, the review's finding 4): an ask's with pin that the ask's
        # reads of its cell met only under an inner or narrower claim (an as if's pin, a solve's variable; R4: the
        # innermost claim wins) is in no read of the line, so it isn't listed. The line's own read set says both:
        # a read that took the pin registered it, one that met it under another claim `pin met` (eng.pin_value; P45:
        # per result, not per ask). tally 2.1.8 §4 (#161): an entry is listed exactly when a read in its set gives it,
        # so a with pin on a cell that no read of the line took, met or not, isn't listed either
        cells = getattr(self, 'pin_cells', {})
        refused = bool(vals) and all(isinstance(v, TallyError) for v in vals)
        for e in [e for e in pins_coll.reg if e[0] == 'pinned' and e[1] in cells and e not in ln.coll.reg]:
            # a refusal carries no read set of its own: what it rests on took the pin unless an inner claim met it
            # (item 9 of the follow-up; the stored m1 vectors)
            coll.reg.discard(e)
        laws = getattr(vals[0], 'laws', None) if vals else None
        out.extend('  ' + r for r in self.register(coll, ln.reading, ctx, a.path is not None, laws,
                                                          hole_days(ln.each_vals if ln.each_vals is not None else vals)))
        if a.orient is not None:
            # "Any other ask may state it, and it is printed" (L §5.15): this interpreter's layout
            out.append('  orient %s' % a.orient)
        return out

    def etype(self, a, i):
        if a.form == 'exprs' and i < len(a.exprs):
            return getattr(a.exprs[i], '_t', None)
        if a.form == 'solve':
            return getattr(a.solve, '_t', None)
        return None

    def register(self, coll, reading, ctx, samples=False, laws=None, unknown=()):
        ent = {}
        assumed = {}
        smp = []
        lj = {}
        for e in coll.reg:
            kind = e[0]
            if kind == 'hz':
                continue
            if kind == 'sample':
                smp.append(e)
                continue
            if kind == 'left out joint':
                # L §8.2 `left out: <move>, <n> instances, first <day>`: the joint model's own movements into a
                # member's place that nothing links (§6.1)
                lj.setdefault(e[1], set()).add(e[2])
                continue
            if kind == 'labels':
                # tally 2.1 (L §8.2 Dropped labels, §8.5 WARN-LABELS `labels dropped at <place> from <d>: <movements>`)
                lb = ent.setdefault('_labels', {})
                cur = lb.setdefault(e[1], [e[2], set()])
                cur[0] = min(cur[0], e[2])
                cur[1].add(e[3])
                continue
            if kind == 'assumed':
                assumed.setdefault(e[1], set()).add(e[2])
            elif kind == 'missing':
                ent.setdefault('missing', set()).add('missing record counted by %s' % e[1])
            else:
                ent.setdefault(kind, set()).add('%s: %s' % (kind, e[1]))
        for mid, days in lj.items():
            from .joint import shown as _jshown
            ent.setdefault('left out', set()).add('left out: %s, %d instances, first %s' % (
                _jshown(mid), len(days), min(days)))
        out = []
        for reason in sorted(assumed):
            ks = sorted(x for x in assumed[reason] if x)
            out.append('assumed: %s%s' % (reason, (' (%s)' % ', '.join(ks)) if ks else ''))
        if ent.get('pinned'):
            ent['pinned'] = merge_pin_days(ent['pinned'])
        for kind in ('given', 'missing', 'pinned', 'pinned sample'):
            out.extend(sorted(ent.get(kind, ())))
        # each unknown day a hole of the result carries, by the name its read gives it, sorted as each kind's lines
        # are (L §8.2 `unknown day:`, §4.4; tally 2.2, 2.2-11)
        out.extend(sorted('unknown day: %s' % u for u in unknown))
        if samples:
            # each sample the path p line reached, in canonical order (L §5.16 Order, §8.2)
            first = {}
            for e in smp:
                if e[2] not in first or e[1] < first[e[2]]:
                    first[e[2]] = e[1]
            out.extend('sample: %s' % t for t in sorted(first, key=lambda t: (first[t], t)))
        out.extend(sorted(ent.get('world', ())))
        out.extend('reading: %s' % r for r in reading)
        for kind in ('before from', 'anchor', 'pad', 'adjustment', 'unfulfilled', 'open', 'left out', 'uncertain'):
            out.extend(sorted(ent.get(kind, ())))
        for place, (d0, mids) in sorted(ent.get('_labels', {}).items()):
            out.append('WARN-LABELS labels dropped at %s from %s: %s' % (place, d0, ', '.join(sorted(mids))))
        out.extend(sorted(ent.get('held', ())))
        if laws is not None:
            # a check whose sides are read at different law dates lists each (L §8.2, §5.18)
            out.append('law date %s (left), %s (right), cut %s' % (laws[0] or 'none', laws[1] or 'none', ctx.cut))
        elif ctx.law is not None:
            out.append('law date %s, cut %s' % (ctx.law, ctx.cut))
        else:
            out.append('cut %s' % ctx.cut)
        return out

    def row_line(self, r):
        """A row read, with its source (L §5.18 inputs-of)."""
        m = self.m
        v = r.version
        src = m.sources.get(v.cite) if v.cite else None
        s = '%s (source %s' % (r.sid, v.cite)
        if src is not None and getattr(src, 'title', None):
            s += ' "%s"' % src.title
        loc = getattr(r, 'locator', None)
        if loc:
            s += ', locator "%s"' % loc
        return s + ')'

    def readers_of(self, a):
        r = a.ref._r
        kind, iid = r[1], r[2]
        item = (kind, iid)
        hits = self.m.checker.readers_of(item)
        m = self.m
        if iid in m.kinds:
            # a binding over a kind reads it and its subkinds, so a binding over a kind above reads it too (E §10,
            # L §8.2's one read set; tally 2.1.7 R7)
            sub = m.kinds[iid].subtree(m)
            for k in sorted(m.kinds):
                if k != iid and (k in sub or iid in m.kinds[k].subtree(m)):
                    for h in m.checker.readers_of(('kind', k)):
                        if h not in hits:
                            hits.append(h)
        names = ['%s' % h[1] for h in hits]
        return ['%s: %s' % (a.id, ', '.join(names) if names else 'nothing reads %s' % iid)]

    # -- explain (L §5.18 Explanation order; E §10) --------------------------------------------------
    def explain_instance(self, eng, v, ctx):
        """A movement instance: its class, day, booking cut and leg derivations; for a decision, its stratum and
        walk point; each later version with its δ, the adjustment it made and what settles or waives it (E §10).
        Layout: this interpreter's."""
        from .values import Mv
        from .moves import Inst
        if not isinstance(v, Mv) or not isinstance(v.src, Inst):
            return ['(%s: no instance at the cut)' % (v.id if isinstance(v, Mv) else fmt_value(v))]
        inst = v.src
        out = []
        tctx = eng.tctx(ctx)
        run = None
        if inst.decision:
            w = eng.main_walk(ctx)
            run = w.decs.get(inst.id)
        src = run if run is not None else inst
        st = getattr(src, 'status', None) or eng.status(inst, ctx)
        if inst.decision:
            out.append('decision, day %s, status %s' % (fmt_value(inst.day), st))
            out.append('stratum %d%s' % (inst.move.stratum, (', walk point %s' % self.point_text(run.point))
                                         if run is not None and run.point is not None else ', not run'))
            if run is not None and getattr(run, 'read_places', None):
                out.append('read at its walk point: %s' % ', '.join(sorted(run.read_places)))
        else:
            out.append('plan, day %s, booking cut %s, status %s' % (fmt_value(inst.day), day_text(inst.cut, inst), st))
        if inst.exists is not True:
            out.append('exists: %s' % fmt_value(inst.exists))
        if src.err is not None:
            out.append('refused: %s %s' % (src.err.code, src.err.msg))
        for lg in src.legs:
            amt = lg.amount
            at = fmt_value(amt) if not isinstance(amt, TallyError) else 'refused: %s %s' % (amt.code, amt.msg)
            on = (' on %s' % fmt_value(lg.day)) if lg.day is not None and lg.day != inst.day else ''
            out.append('leg %s from %s to %s amount %s%s' % (lg.name, fmt_value(lg.frm), fmt_value(lg.to), at, on))
        for k, val in sorted(src.attrs.items()):
            out.append('attr %s = %s' % (k, fmt_value(val)))
        if not inst.decision and inst.cut is not None and inst.env is not None:
            # the derivation at the booking cut, traced (E §10: re-evaluates the cell's cone with tracing on)
            sub = self.trace_inst(eng, inst, tctx, ctx, inst.cut)
            if sub:
                out.append('derivation at %s:' % inst.cut)
                out.extend(sub)
            ans = getattr(inst, 'answered', None)
            if ans is not None:
                # E §10 (tally 2.1.8 S4, #179): "a version that answers a hole booking prints `answered at <cut>
                # (version <V>):` before its rows", the derivation under that version's known
                out.append('answered at %s (version %s):' % (ans[0], ', '.join(ans[1])))
                out.extend(self.trace_inst(eng, inst, tctx, ctx, ans[0]))
        if not inst.decision:
            for adj in eng.adjustments(ctx):
                if adj.inst.id != inst.id:
                    continue
                if adj.err is not None:
                    # tally 2.1.2 (issue #34): a refused re-measurement's adjustment
                    out.append('adjustment %s: change refused: %s on %s (version %s)' % (
                        adj.id, adj.err.text(), adj.day, ', '.join(adj.vids)))
                    continue
                rem = adj.remainder()
                by = ', '.join(sorted(s.id for s, sign, amt in adj.settled if amt is not None)) or 'nothing'
                out.append('adjustment %s: change %s on %s (version %s), %s; remaining %s' % (
                    adj.id, fmt_value(adj.delta), adj.day, ', '.join(adj.vids),
                    ('waived' if adj.waived else 'settled by %s' % by), 'none' if rem is None else fmt_value(rem)))
        return out

    def trace_inst(self, eng, inst, tctx, ctx, cut):
        """A plan instance's derivation at a cut, traced (E §10: re-evaluates the cell's cone with tracing on)."""
        eng.tracing = True
        eng.trace_cells = {}
        saved = eng.memo
        eng.memo = {}
        root = TraceNode('root', inst.id)
        eng.trace_stack = [root]
        try:
            eng.eval_inst(inst.move, inst.env, inst.keys, tctx, at_cut=cut)
        finally:
            eng.tracing = False
            eng.trace_stack = []
            eng.memo = saved
        return self.explain_lines(root, ctx)

    def explain_pins(self, eng, a, env, ctx, root, v):
        """explain under an ask's `with` pins, an as if over the line (E §10): the pins with their values, the
        derivation inside them, and the one outside them.  Layout: this interpreter's."""
        from .evaluate import _tfmt
        pinned = set(p.axis for p in a.pins if getattr(p, 'pkind', None) == 'axis')
        octx = self._outside_ctx.but(axes=tuple((ax, o) for ax, o in ctx.axes if ax not in pinned))

        def outside_of(oc, fixed):
            # outside the pins, an axis the ask pinned is read as every other line reads an axis: the derivation fans
            # out over the options of each one it reads, in declaration order (L §5.5 Fan-out; tally 2.1.7 R8, T's
            # SR-10); 2.1.7's first refcore stopped at its guard
            eng.tracing = True
            eng.trace_cells = {}
            saved = eng.memo
            eng.memo = {}
            oroot = TraceNode('root', _txt(a.ref))
            eng.trace_stack = [oroot]
            old = set_world(eng.world_chain(oc.world))
            eng.stack.append(Coll())
            # the derivation outside the pins is no read of the result: neither its register nor the ask pins it
            # takes are listed (L §8.2 One read set; T's SR-C1, P42)
            record, eng.pin_record = eng.pin_record, False
            try:
                vo = self.one(eng, a.ref, env, oc)
            finally:
                eng.pin_record = record
                eng.stack.pop()
                set_world(old)
                eng.tracing = False
                eng.trace_stack = []
                eng.memo = saved
            ax = getattr(vo, 'axis', None)
            if ax in pinned and ax not in dict(fixed):
                nodes = []
                for o in self.m.axes[ax].options:
                    nodes += outside_of(oc.but(axes=tuple(sorted(oc.axes + ((ax, o),)))), fixed + ((ax, o),))
                return nodes
            # the axes the value reads (L §5.18 fan-out), in the order of their ids (tally 2.2.1, 231.R1; P48 F-E)
            fixed = tuple(sorted(fixed))
            tag = (' [%s]' % ', '.join('%s = %s' % f for f in fixed)) if fixed else ''
            node = TraceNode('group', 'outside the pins%s: %s' % (tag, _tfmt(vo)))
            node.carry_v = vo
            node.children = oroot.children
            node.fixed = tuple((ax, self.m.axes[ax].options.index(o)) for ax, o in fixed)
            return [node]
        outs = sorted(outside_of(octx, ()), key=lambda n: getattr(n, 'fixed', ()))
        out = TraceNode('root', _txt(a.ref))
        pn = TraceNode('group', 'the pins, evaluated outside them')
        for line in eng.pin_lines(octx, ctx):
            pn.children.append(('node', TraceNode('group', 'pin ' + line)))
        inside = TraceNode('group', 'inside the pins: %s' % _tfmt(v))
        inside.children = root.children
        out.children = [('node', pn), ('node', inside)] + [('node', o) for o in outs]
        out.value = v
        return out

    def explain_walk_bound(self, eng, a, env, ctx):
        """`explain x[k]` of a def evaluated at walk points: the cell at each walk point that read it, in walk order,
        each with its point (day, stratum, movement, keys); with none, that nothing read it at those keys (E §10,
        L §5.18).  Layout: this interpreter's."""
        from .model import _kv
        n = a.ref
        qid = n._r[2]
        info = self.m.defs[qid]
        keynodes = n.keys if n.k == 'index' else []
        try:
            keys, hole = eng.key_values(info, keynodes, n._convs, env, ctx)
            if hole is not None:
                return hole, []
            w = eng.main_walk(ctx)
        except HoleSignal as h:
            return h.hole, []
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            return e, []
        org = eng.origin(info, keys)
        cells = eng.walk_cells.get((id(w), qid, tuple(_kv(k) for k in keys)), {})
        if not cells:
            return CheckOut('walk-bound, read at no walk point', 'walk'), [
                'nothing read %s at a walk point' % org]
        # E §10 (round 14): "the cell at each walk point where a decision read it, in walk order, each with its point
        # (day, stratum, movement, keys) and explained as a def cell (above), its running reads as balances up to
        # that point"
        out = []
        byp = {r.point: r.id for r in w.decs.values() if getattr(r, 'point', None) is not None}
        from .walk import dec_head
        for p in sorted(cells):
            import datetime as _dt
            d = _dt.date.fromordinal(p[0])
            out.append('%s at (%s, stratum %s, %s): %s' % (org, d, p[2], byp.get(p, dec_head(p[3])),
                                                           fmt_value(cells[p])))
            # the state just before the reader's own booking at p: "its running reads as balances up to that point"
            ctx2 = w.tctx.but(cut=min(d, w.K), walk=w, wp=p[:4] + (p[4] - 0.5,))
            # inside the reader's run: its own earlier observations left out, as its value was (E §10 l.472, §4.2
            # step 3; tally 2.1.8 #169-g)
            old_less = getattr(w, 'less_obs', None)
            w.less_obs = w.own_obs(byp[p], d) if p in byp else None
            eng._explaining_wb = True
            eng.tracing = True
            eng.trace_cells = {}
            saved = eng.memo
            eng.memo = {}
            root = TraceNode('root', org)
            eng.trace_stack = [root]
            sub = Coll()
            eng.stack.append(sub)
            try:
                try:
                    v2 = eng.read_def(info, keys, ctx2)
                except HoleSignal as h:
                    v2 = h.hole
                except TallyError as e:
                    if is_load(e):
                        raise LoadAtRun(e)
                    v2 = e
            finally:
                eng.stack.pop()
                eng.tracing = False
                eng.trace_stack = []
                eng.memo = saved
                eng._explaining_wb = False
                w.less_obs = old_less
            root.value = v2
            out.extend('  ' + l for l in self.explain_lines(root, ctx2))
        return CheckOut('walk-bound, read at %d walk point%s' % (len(cells), '' if len(cells) == 1 else 's'),
                        'walk'), out

    def explain_fill(self, eng, a, env, ctx):
        """`explain F.taken[s, d]`: the fill's v version if it has one, the arcs in order, the remaining supply and
        demand before and after each take (E §10).  Layout: this interpreter's."""
        from .model import _kv
        n = a.ref
        qid = n._r[1]
        eng.fill_trace = []
        saved = eng.memo
        eng.memo = {}
        try:
            v = self.one(eng, n, env, ctx)
            trace = eng.fill_trace
            fi = eng.m.checker.fill_infos[qid]
            try:
                tkeys, h = eng.fill_keys(n._tkeys, n._convs, fi['sdoms'] + fi['ddoms'], env, ctx)
                fkeys, h2 = eng.fill_keys(n._fkeys, n._fconvs, fi['fdoms'], env, ctx)
            except TallyError:
                tkeys = fkeys = None
                h = h2 = None
        finally:
            eng.fill_trace = None
            eng.memo = saved
        if isinstance(v, TallyError) and not any(t[0] == qid for t in trace):
            return v, []
        ent = None
        want = tuple(_kv(k) for k in fkeys) if fkeys is not None and h2 is None else None
        for t in trace:
            if t[0] == qid and (want is None or t[1] == want):
                ent = t
        if ent is None:
            return v, []
        _, fk, sec, log = ent
        out = []
        head = 'fill %s%s' % (qid, ('[%s]' % ', '.join(key_text(k) for k in fkeys)) if fkeys else '')
        if sec.v is not None:
            ver = self.m.versions.get(sec.v)
            head += ', v %s%s' % (sec.v, (' (source %s)' % ver.cite) if ver is not None and ver.cite else '')
        else:
            head += ', no v'
        out.append(head)
        asked = tuple(_kv(k) for k in tkeys) if tkeys is not None and h is None else None
        found = False
        out.append('arcs in order:')
        for i, (x, r, q, t, r2, q2) in enumerate(log):
            sk = ', '.join(fmt(x['se'][b.var]) for b in sec.sbinds)
            dk = ', '.join(fmt(x['de'][b.var]) for b in sec.dbinds)
            mark = ''
            if asked is not None and x['sk'] + x['dk'] == asked:
                mark = '  <- this take'
                found = True
            out.append('  %d. (%s; %s): supply %s, demand %s; takes %s; then supply %s, demand %s%s' % (
                i + 1, sk, dk, fmt_value(r), fmt_value(q), fmt_value(t), fmt_value(r2), fmt_value(q2), mark))
        if asked is not None and not found:
            out.append('this pair is not an arc: it takes 0')
        return v, out

    def explain_balance(self, eng, a, n, env, ctx):
        """`explain bal(x, d)` (a balance) and `explain bal(x, d) from t` (a projection), E §10.  Layout: this
        interpreter's."""
        form = getattr(n, '_form', None)
        v = self.one(eng, n, env, ctx)
        if isinstance(v, TallyError) or form == 'now':
            return v, []
        sub = Coll()
        eng.stack.append(sub)
        try:
            lines = self._explain_bal_lines(eng, a, n, env, ctx)
        except HoleSignal:
            lines = []
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            lines = ['refused: %s %s' % (e.code, e.msg)]
        finally:
            eng.stack.pop()
        return v, lines

    def balance_cause(self, eng, a, n, env, ctx):
        """The lines of a balance read met as a cause (E §10's balance form), its register kept in the cause's."""
        try:
            return self._explain_bal_lines(eng, a, n, env, ctx)
        except HoleSignal:
            return []
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            return ['refused: %s %s' % (e.code, e.msg)]

    def _explain_bal_lines(self, eng, a, n, env, ctx):
        from .walk import K_end, K_start, kday, gtext, St
        p = eng.place_arg(n.ref, env, ctx)
        if isinstance(p, Hole):
            return ['the place is a hole: %s' % fmt_hole(p)]
        place = p.name
        unit = eng.bal_unit(n, place, env, ctx)
        if isinstance(unit, Hole):
            return ['the unit is a hole: %s' % fmt_hole(unit)]
        d = eng.ev(n.args[-1], env, ctx)
        if isinstance(d, Hole):
            return []
        if isinstance(d, Iv) and isinstance(d.lo, datetime.date):
            return []
        end = eng.date_of(d, 'the day of bal')
        if getattr(n, '_form', None) == 'proj':
            return self._explain_projection(eng, a, n, env, ctx, place, unit, end)
        w = eng.main_walk(ctx)
        key = K_start(end) if getattr(n, 'start', False) else K_end(end)
        modelled = getattr(n, '_form', None) == 'modelled'
        out = ['balance of %s (%s) at the %s of %s%s' % (place, unit, 'start' if getattr(n, 'start', False) else 'end', end,
                                                          ', modelled' if modelled else '')]
        st, lg, i = w.state(place, unit, key)
        if modelled:
            # the model's own account (L §5.19; tally 2.1.8 R16): its start, entries and modelled legs
            lg, r0, m_ents, m_legs = eng.modelled_account(w, place, unit, key, end)
            st = St(st.val, st.hole, st.perm, st.imp, st.bad, r0, st.pbad)
        since = None
        lo = -1
        if lg is None or st.rst < 0:
            st0 = w.default(place, unit)
            if st0.val is None:
                out.append('from: no reading or opening before it')
            else:
                th = self.m.things[place]
                from .model import SameDiffers
                try:
                    starts = [self.m.lifespan(th, w.K).start]
                except SameDiffers:
                    # a place stated same whose sides' lifespans differ, each starting at or after S (L §6.1)
                    starts = sorted(set(x.start for x in w.side_spans(th)))
                out.append('from: 0 at the start of its lifespan, %s' % ' and '.join(str(x) for x in starts))
        else:
            e = lg.ents[st.rst]
            lo = st.rst
            since = e.key
            out.append('from: %s' % self._reading_text(w, place, e, gtext))
        ents = []
        if modelled:
            ents.extend(m_ents)
        elif lg is not None:
            ents.extend(lg.ents[lo + 1:i + 1])
        wl = w.logs.get((place, '*'))
        if wl is not None:
            ents.extend(x for x in wl.ents if (since is None or x.key > since) and x.key <= key)
        rows = []
        for x in sorted(ents, key=lambda x: self._listing_key(x.key)):
            t = self._entry_text(x, gtext)
            if t is not None:
                rows.append((kday(x.key), t))
        extra = []
        if modelled:
            for x, g, sg, bk in m_legs:
                if g is not None:
                    rows.append((kday(bk), '%s.%s (modelled): %s' % (x.id, g.name, self._amount_text(g.amount, sg))))
        if n.proj and n.proj.get('uncertain'):
            extra.extend((x, 'uncertain') for x in w.uncertain)
        for x, stt in extra:
            for g in x.legs:
                gd = g.day if isinstance(g.day, datetime.date) else lin_day(x.day)
                if gd is None or gd > end or (since is not None and K_end(gd) <= since):
                    continue
                for pp, sg in ((g.frm, -1), (g.to, 1)):
                    if isinstance(pp, Thing) and pp.name == place:
                        rows.append((gd, '%s.%s (%s): %s' % (x.id, g.name, stt, self._amount_text(g.amount, sg))))
        out.extend(self._by_day(rows))
        return out

    def _explain_projection(self, eng, a, n, env, ctx, place, unit, end):
        from .walk import K_end, kday, gtext
        t = eng.ev(n.proj['frm'], env, ctx)
        if isinstance(t, Hole):
            return []
        t = eng.date_of(t, 'the day of a projection')
        clauses = eng.proj_clauses(n.proj, ctx)
        w = eng.proj_walk(ctx, t, n._leftout, clauses)
        w.run_to(end)
        rnd, unc = clauses
        out = ['projection of %s (%s) from %s to the end of %s' % (place, unit, t, end)]
        out.append('holder: ask %s (an ask line leaves out nothing, L §5.14.1)' % a.id)
        rtext = 'none'
        if rnd:
            rtext = 'at mean' if rnd[0] == 'mean' else 'at quantile(%s)' % fmt_q(rnd[1])
        out.append('clauses: random %s, %s' % (rtext, 'with uncertain' if unc else 'without uncertain'))
        st0 = w.default(place, unit)
        out.append('start: %s at the start of %s' % (gtext(w.st_value(st0, place, unit)), t))
        lg = w.logs.get((place, unit))
        ents = []
        if lg is not None:
            ents.extend(x for x in lg.ents if x.key <= K_end(end))
        wl = w.logs.get((place, '*'))
        if wl is not None:
            ents.extend(x for x in wl.ents if x.key <= K_end(end))
        rows = []
        for x in sorted(ents, key=lambda x: self._listing_key(x.key)):
            tx = self._entry_text(x, gtext)
            if tx is not None:
                rows.append((kday(x.key), tx))
        if unc:
            for x in w.uncertain:
                for g in x.legs:
                    gd = g.day if isinstance(g.day, datetime.date) else lin_day(x.day)
                    if gd is None or gd > end or gd < t:
                        continue
                    for pp, sg in ((g.frm, -1), (g.to, 1)):
                        if isinstance(pp, Thing) and pp.name == place:
                            rows.append((gd, '%s.%s (uncertain): %s' % (x.id, g.name, self._amount_text(g.amount, sg))))
        out.append('counted:')
        out.extend('  ' + l for l in self._by_day(rows))
        lo = sorted((dd, i) for dd, i in w.leftout_insts if dd is not None and t <= dd <= end)
        if lo:
            out.append('left out:')
            out.extend('  %s %s' % (dd, i) for dd, i in lo)
        else:
            out.append('left out: none')
        return out

    def _reading_text(self, w, place, e, gtext):
        from .moves import instant_text
        rows = list(e.coll.rows.values()) if e.coll is not None else []
        val = fmt_hole(Hole(e.qs)) if e.qs else fmt(e.val)
        if not rows:
            return '%s = %s' % (e.note or 'reading', val)
        r = rows[0]
        v = r.version
        s = '%s %s = %s, %s (version %s, source %s' % (e.note, r.sid, val, instant_text(r), v.id, v.cite)
        loc = getattr(r, 'locator', None)
        if loc:
            s += ', locator "%s"' % loc
        s += ')'
        if e.note in ('anchor', 'pad'):
            from .walk import kday
            for key, gap, r2, role, u, _bc in w.gaps.get((place, kday(e.key)), ()):
                if r2 is r or getattr(r2, 'sid', None) == r.sid:
                    if role[0] == 'anchor':
                        s += ', gap role anchor, gap %s' % gtext(gap)
                    else:
                        s += ', gap role pad from %s, gap %s (%s)' % (role[1].name, gtext(gap),
                                                                    e.mv.id if e.mv is not None else 'pad')
                    break
        return s

    def _amount_text(self, amt, sign):
        if isinstance(amt, Hole):
            return 'hole: %s' % fmt_hole(amt)
        if isinstance(amt, TallyError):
            return 'refused: %s %s' % (amt.code, amt.msg)
        q = sign * amt.q
        u = amt.u[0][0] if amt.u else None
        from .moves import qty
        return ('+' if q > 0 else '') + fmt(qty(q, u))

    def _entry_text(self, x, gtext):
        """One booked entry of a place's log: its movement's id, status and what it moved (E §10)."""
        from .moves import PadMv
        mv = x.mv
        mid = mv.id if mv is not None else 'the walk'
        if x.note and x.kind in ('delta', 'hole') and mv is not None and not isinstance(mv, PadMv):
            mid += '.%s' % x.note
        st = getattr(mv, 'status', None) if mv is not None else None
        head = '%s (%s)' % (mid, st) if st else mid
        if x.kind == 'delta':
            q = x.flow.q
            return '%s: %s%s' % (head, '+' if q > 0 else '', fmt(x.flow))
        if x.kind in ('hole', 'perm'):
            return '%s: hole: %s' % (head, fmt_hole(Hole(x.qs)))
        if x.kind == 'imp_on':
            return '%s: its window %s opens: %s' % (head, x.win.window_text() if x.win is not None else '',
                                                    fmt_hole(Hole(x.qs)))
        if x.kind in ('bad', 'pbad'):
            return '%s: refused: %s %s' % (head, x.err.code, x.err.msg)
        if x.kind == 'reset':
            return '%s' % self._reading_text_plain(x)
        return None

    def _reading_text_plain(self, x):
        rows = list(x.coll.rows.values()) if x.coll is not None else []
        val = fmt_hole(Hole(x.qs)) if x.qs else fmt(x.val)
        return '%s %s = %s' % (x.note, rows[0].sid if rows else '', val)

    def _by_day(self, rows):
        if not rows:
            return ['no movement']
        out = []
        cur = None
        for dd, t in sorted(rows, key=lambda r: r[0]):
            if dd != cur:
                out.append('%s:' % dd)
                cur = dd
            out.append('  %s' % t)
        return out

    def point_text(self, p):
        import datetime as _dt
        from .walk import DSEP
        d = _dt.date.fromordinal(p[0])
        # a decision's entry text prints its terms joined by `|`, as written before SR-B4 (walk.DSEP)
        return '(%s, stratum %s, %s)' % (d, p[2], p[3].replace(DSEP, '|') if isinstance(p[3], str) else p[3])

    def _listing_key(self, k):
        """A walk entry's key for an explain's listing: an id at the key's fourth place in id order, its keys in index
        order, never as text (E §4.1; tally 2.1.8 P50: save[k9] before save[k10])."""
        if isinstance(k, tuple) and len(k) > 3 and isinstance(k[3], str):
            from .moves import id_order
            return k[:3] + (id_order(self.m, k[3]),) + k[4:]
        return k

    def explain_lines(self, root, ctx):
        if isinstance(root, ExplainLines):
            return ['  ' + l for l in root.lines]
        out = []
        self._no = 0
        self._seen = {}
        under = set()
        for kind, ch in root.children:
            self.explain_node(ch, kind, 1, out, under)
        return out

    def def_causes(self, tn):
        """A def cell's causes in the order explain_node prints them: each rule's when, the winner's value, the rest."""
        states = getattr(tn, 'rule_states', {})
        whens = dict(getattr(tn, 'when_children', []))
        for lab in sorted(states):
            w = whens.get(lab)
            if w is not None:
                yield from (ch for _k, ch in w.children)
            vc = getattr(tn, 'value_children', None)
            if states[lab] == 'won' and vc is not None:
                yield from (ch for _k, ch in vc.children)
        yield from (ch for _k, ch in tn.children)

    def chain_next(self, tn, taken):
        """The first cell of tn's own def that tn reads and nothing has printed: the next step of a recurrence."""
        for ch in self.def_causes(tn):
            if ch.kind == 'def' and getattr(ch, 'def_id', None) == tn.def_id and id(ch) not in self._seen and \
                    id(ch) not in taken:
                return ch
        return None

    def explain_step(self, tn, ch, kind, depth, out, under):
        """A def cell's cause. A chain of more than 3 cells of one def, each read by the one before it, folds (E §10
        Long derivations fold; tally 2.1.7 S4, #131): tn, then `… <n> more steps of <x>, <k2> .. <kn-1> …` where its
        second step would print, then its last step there; a cause only the folded steps read prints where it is next
        met, as they never print."""
        if ch.kind != 'def' or getattr(ch, 'def_id', None) != getattr(tn, 'def_id', None) or id(ch) in self._seen:
            self.explain_node(ch, kind, depth, out, under)
            return
        rest = [ch]
        taken = {id(tn), id(ch)}
        while True:
            nx = self.chain_next(rest[-1], taken)
            if nx is None:
                break
            rest.append(nx)
            taken.add(id(nx))
        if 1 + len(rest) <= 3:
            self.explain_node(ch, kind, depth, out, under)
            return
        keys = lambda c: ', '.join(key_text(k) for k in c.keys)
        out.append('%s\u2026 %d more steps of %s, %s .. %s \u2026' % (
            '  ' * depth, len(rest) - 1, tn.def_id, keys(rest[0]), keys(rest[-2])))
        self.explain_node(rest[-1], kind, depth, out, under)

    def explain_node(self, tn, kind, depth, out, under):
        """tn under a line whose causes printed so far are under (a set of their idents)."""
        ind = '  ' * depth
        ident = ('row', tn.label) if tn.kind in ('row', 'bandrow') else id(tn)
        # a cause prints in full where it is first printed and as a reference after (L §5.18 "Each cause prints
        # once; a cause met again is referenced by its line"): the print order (rules in canonical order) need not
        # be the evaluation order (beaters first), so a cell traced first may print second.  E §10 (tally 2.1.8 S8,
        # #181): "A cause met again is referenced by its line, once under each line that reads it", whatever order
        # the cells were computed in: a line's cause met again under that line prints nothing more
        if ident in under:
            return
        under.add(ident)
        if ident in self._seen:
            out.append('%s(%s: see [%d])' % (ind, tn.label, self._seen[ident]))
            return
        self._no += 1
        self._seen[ident] = self._no
        no = self._no
        if tn.kind == 'row':
            out.append('%s[%d] row %s = %s' % (ind, no, tn.label, fmt_value(tn.value)))
            for l in tn.lines:
                out.append('%s    %s' % (ind, l))
            return
        if tn.kind == 'bandrow':
            out.append('%s[%d] band row %s' % (ind, no, tn.label))
            return
        if tn.kind == 'balread':
            # a balance cause in its own form (E §10; tally 2.1.7 R7, #132b)
            e = getattr(tn, 'refused', None)
            out.append('%s[%d] %s = %s' % (ind, no, tn.label, fmt_value(tn.value) if e is None else 'refused: %s %s' % (
                e.code, e.msg)))
            for l in tn.lines:
                out.append('%s    %s' % (ind, l))
            return
        if tn.kind == 'def':
            law = getattr(tn, 'law', None)
            out.append('%s[%d] %s = %s%s' % (ind, no, tn.label, fmt_value(tn.value),
                                            (', law date %s' % law) if law is not None else ''))
            for l in tn.lines:
                out.append('%s    %s' % (ind, l))
            states = getattr(tn, 'rule_states', {})
            vers = getattr(tn, 'rule_vers', {})
            whens = dict(getattr(tn, 'when_children', []))
            for lab in sorted(states):
                st = states[lab]
                rv = vers.get(lab)
                extra = ''
                if st == 'won' and rv is not None:
                    vv = rv.v or rv.version
                    if vv is not None:
                        extra = ' (version %s, source %s)' % (vv.id, vv.cite)
                out.append('%s    rule %s: %s%s' % (ind, lab, st, extra))
                w = whens.get(lab)
                below = set()       # the rule's line reads its when and, the winner, its value
                if w is not None:
                    for k2, ch in w.children:
                        self.explain_step(tn, ch, k2, depth + 3, out, below)
                if st == 'won':
                    vc = getattr(tn, 'value_children', None)
                    if vc is not None:
                        for k2, ch in vc.children:
                            self.explain_step(tn, ch, k2, depth + 3, out, below)
            below = set()
            for k2, ch in tn.children:
                self.explain_step(tn, ch, k2, depth + 2, out, below)
            return
        out.append('%s[%d] %s' % (ind, no, tn.label))
        below = set()
        for k2, ch in tn.children:
            self.explain_node(ch, k2, depth + 1, out, below)


def law_reads_binds(a):
    """Whether an ask's law expression reads one of its ask-level for's variables."""
    from .node import walk
    vs = set(b.var for b in (a.binds or ()))
    return bool(vs) and any(getattr(x, '_r', None) is not None and x._r[0] == 'var' and x._r[1] in vs
                            for x in walk(a.law))


def hole_days(vals):
    """The names of the unknown days the holes among vals carry (L §8.2 `unknown day:`; tally 2.2, 2.2-11): each
    question of a hole to the windows it was made of (evaluate.CARRY), by the label its read gives it (window_id). A
    window value that is no hole gives none."""
    from .evaluate import CARRY, iv_carry
    out = set()

    def walk(x, depth=0):
        if depth > 50:
            return
        if isinstance(x, Hole):
            for q in x.qs:
                out.update(str(w[1]) for w in CARRY.get(q, ()))
        elif isinstance(x, Iv) and not isinstance(x.lo, datetime.date):
            # a value computed from a window prints as the hole `not precise enough for a print` (fmt_value): a hole
            # made of it carries its unknown days (L §4.4; tally 2.2, pick P25)
            out.update(str(w[1]) for w in iv_carry(x))
        elif isinstance(x, (tuple, list, frozenset, set)):
            for y in x:
                walk(y, depth + 1)
        elif isinstance(x, Lst):
            for y in x.items:
                walk(y, depth + 1)
        elif hasattr(x, 'carry_vals'):
            for y in x.carry_vals():
                walk(y, depth + 1)
    walk(vals)
    return out


def diff_days(res):
    """hole_days of a diff cell's values in its two worlds ({world: [(key, value)]})."""
    return hole_days([v for lines in res.values() for _k, v in lines])


def is_query_line(e):
    """An ask line's expression that is a query over random values (L §5.16), or a crps over paths."""
    from .eval2 import strip
    x = strip(e)
    if x.k == 'to' and getattr(x, '_form', None) in ('query', 'crps'):
        x = strip(x.e)
    if x.k == 'query' and x.binds is None and not getattr(x, '_dfn', False):
        return True
    if x.k == 'crps' and getattr(x, '_form', None) == 'paths':
        return True
    return False


def split(axes, combos, m):
    """Split a set of combinations (dicts axis -> option) into products, by the axes in id
    order (L §5.5 A set that isn't a product).  Returns lists of (axis, set of options)."""
    if not axes:
        return [[]]
    ax = axes[0]
    rest = axes[1:]
    groups = {}
    order = []
    for o in m.axes[ax].options:
        tails = frozenset(tuple(sorted((k, v) for k, v in c.items() if k != ax)) for c in combos if c[ax] == o)
        if not tails:
            continue
        if tails not in groups:
            groups[tails] = []
            order.append(tails)
        groups[tails].append(o)
    out = []
    for tails in order:
        opts = set(groups[tails])
        for sub in split(rest, [dict(t) for t in tails], m):
            out.append([(ax, opts)] + sub)
    return out
