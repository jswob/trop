"""The pin oracle (tally 2.3, the plan's §9 as corrected; issue #247). Seeded and deterministic, with no stored answer.

It writes small texts in each of which one cell `w` holds a window, runs each text once with every line asked unpinned
and again under `with w = D` for each day D of the window, and holds the unpinned line to one of:
  (1) what every pinned day prints, among the days no balance check refuses;
  (2) a hole that names the day (`w` among its labels);
  (3) the refusal a refused day prints (a pinned day a balance check refuses is counted apart: rulings, C-3);
  (4) for a line whose value is a date, the stretch `window(a, b)` holding every pinned day's print.
It reports the lines that printed a value (the lines with teeth) and the refused days; a text whose every unpinned
line is a hole or a refusal is reported empty and fails, as does a text refused at load.

G1 holds the places (§9.2): #244's ten meetings (the run's start, a covered span's end, a version's known day inside
and after the stretch, a scenario's `from`, a law-date class boundary, a reading's instant, a lifespan's end, two
movements stated `same`, the register, the ledger), the sign rule and its twin with a hole amount on a plain day, a
division by a plain zero beside the hole day, a count and a sum over `adjustments` (C-4) and, C13 being taken, a take
at a place that keeps parts (`parts`: on the engine it waits for M5b and is skipped). Each holds a plain movement at
another place and counts beside it. G2 adds the readers (`rd_*`: each listed reader in each ranged position, readers
off the list, compositions, two windows and one twice, the refusals at the ends of time), each line held harder by
what it must print (`judge`). A place whose every line is the hole by a rule kept (`APART`, N2) is counted apart.

A ledger's or an explanation's lines are its body and are held line by line; the register under an answer (`pinned:`,
`unknown day:`, `unfulfilled:`, `assumed`, an adjustment, a pad, an anchor, `open`, `left out`, `cut`) is left out:
it carries the day by design, and §9.4 keeps its words in hand-stored vectors. Refused days, counted apart, are those a
balance check refuses (C-3: `RUN-PLACE`, `RUN-LIFESPAN`, a take's) and those where a movement doesn't exist
(`RUN-NONE`, ruling 15); any other refusal is a value every day must agree on.

    python3 -I -B engine/tests/properties/oracle.py [--interp refcore|fast|engine] [--seed N] [--variants N]
        [--sample K] [--only NAME[,NAME]] [--out DIR] [--keep] [--check]
`--check` is run.py --self-test's form (step 8t): the defaults, nothing written but out/oracle/. `--sample 10` runs a
fixed tenth by seed. Exit 0 when every line holds and no text is empty or refused at load; 1 otherwise. A line that
`HELD` names (a fault found, mended at a later pin) is reported held, not failed. Last line: `oracle: <t> texts, <l>
lines, <v> value lines, <r> refused days, <a> texts apart, <f> failed, <h> held by name`.
"""
import datetime
import os
import random
import re
import shutil
import subprocess
import sys

TESTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.dirname(os.path.dirname(TESTS))
PYPY = os.path.expanduser('~/.local/share/uv/python/pypy-3.11.13-macos-aarch64-none/bin/pypy3.11')
D = datetime.date
DAY = datetime.timedelta(days=1)
# where the stretch's meeting point X falls: a quarter's and month's end, a year's end, 29 February, an ordinary day
CROSSINGS = [D(2026, 3, 31), D(2026, 12, 31), D(2028, 2, 29), D(2026, 5, 14), D(2026, 6, 30)]


def iso(d):
    return d.isoformat()


def mon(d):
    return d.strftime('%Y-%m')


def last_of_month(d):
    n = D(d.year + (d.month == 12), d.month % 12 + 1, 1)
    return n - DAY


class Text:
    """One generated text: its files, its lines (expr, scenario, value pins, tail), and the window."""

    def __init__(self, name, rng, x=None):
        self.name, self.rng = name, rng
        self.x = x or rng.choice(CROSSINGS)
        k1, k2 = rng.randint(1, 3), rng.randint(1, 3)
        self.a, self.b = self.x - k1 * DAY, self.x + k2 * DAY
        self.start, self.end = D(self.x.year, 1, 1), D(self.x.year + 1, 12, 31)
        self.places = ['place konto: accounts PLN lives %s..' % iso(self.start - 366 * DAY),
                       'place other: accounts PLN lives %s..' % iso(self.start - 366 * DAY),
                       'place world: outside lives 1900-01-01..']
        self.inputs = ['input amt: money PLN', 'input w: date', 'input dx: date']
        self.moves = ['move pw: transfers\n  on w\n  leg cash from world to konto amount amt',
                      'move px: transfers\n  on dx\n  leg cash from world to other amount amt']
        self.rows = ['amt = 100.00 PLN "s"', 'w = window(%s, %s) "s"' % (iso(self.a), iso(self.b)),
                     'dx = %s "s"' % iso(self.x)]
        self.versions, self.extra_files, self.head_extra, self.lines, self.anchor = [], {}, [], [], {}
        self.m, self.m1 = mon(self.x), mon(last_of_month(self.x) + DAY)
        self.after = self.b + 3 * DAY
        self.v2 = None  # a second window's days (a2, b2): the pins run over every pair (G2's two windows)
        self.expect = {}  # line index -> list, off or false (the readers'; see `judge`)

    def window(self, a, b):
        """Set the window w to a .. b (the readers' refusals at the ends of time)."""
        self.a, self.b = a, b
        self.rows[1] = 'w = window(%s, %s) "s"' % (iso(a), iso(b))

    def read(self, expr, expect, pins='', tail=''):
        self.expect[len(self.lines)] = expect
        self.line(expr, pins=pins, tail=tail)

    def common(self, before=True, konto=True):
        if before and konto:
            self.line('bal(konto, %s)' % iso(self.a - 2 * DAY))
        if konto:
            self.line('bal(konto, %s)' % iso(self.x))
            self.line('bal(konto, %s)' % iso(self.after))
            self.line('flows(konto, %s)' % self.m)
        self.line('pw.cash.amount')  # the instance's amount: a value beside its hole day
        self.line('bal(other, %s)' % iso(self.after))
        self.line('flows(other, %s)' % self.m)
        self.line('count(m in transfers over %s where m.cash.to == other)' % self.m)
        self.line('count(m in transfers over %s)' % self.m)

    def line(self, expr, scen='', pins='', tail=''):
        self.lines.append((expr, scen, pins, tail))

    def opening(self):
        return ['opening[konto] = 500.00 PLN at %s start "s"' % iso(self.start),
                'opening[other] = 500.00 PLN at %s start "s"' % iso(self.start)]

    def ask(self, i, pin):
        expr, scen, pins, tail = self.lines[i]
        ps = [p for p in (pins, pin[1] if pin else '') if p]
        return 'ask l%02d%s: %s%s%s%s' % (i, '_p' + pin[0] if pin else '', expr,
                                         ' in ' + scen if scen else '', ' with ' + ', '.join(ps) if ps else '',
                                         ' ' + tail if tail else '')

    def days(self):
        return [self.a + i * DAY for i in range((self.b - self.a).days + 1)]

    def pinsets(self):
        """Each pinned run's key (`YYYYMMDD`, and the second window's day after it) and its `with` pins."""
        ws = [(d.strftime('%Y%m%d'), 'w = %s' % iso(d)) for d in self.days()]
        if not self.v2:
            return ws
        vs = [(d.strftime('%Y%m%d'), 'v2 = %s' % iso(d))
              for d in (self.v2[0] + i * DAY for i in range((self.v2[1] - self.v2[0]).days + 1))]
        return [(k1 + k2, p1 + ', ' + p2) for k1, p1 in ws for k2, p2 in vs]

    def files(self):
        asks = [self.ask(i, d) for i in range(len(self.lines)) if not self.lines[i][1] for d in [None] + self.pinsets()]
        main = '\n'.join(['model %s' % self.name, '  tally 2', '  on %s' % iso(self.after + 60 * DAY),
                          '  start %s' % iso(self.start), '  end %s' % iso(self.end), 'source s "invented"',
                          'unit PLN: currency minor 0.01'] + self.head_extra + self.places + self.inputs + self.moves +
                         ['version v\n  cite s\n  governs always\n  known %s\n' % iso(self.start - 366 * DAY) +
                          '\n'.join('  ' + r for r in self.rows + self.opening())] + self.versions + asks) + '\n'
        out = {self.name + '.tally': main}
        for fname, body in self.extra_files.items():
            sc = [self.ask(i, d) for i in range(len(self.lines)) if self.lines[i][1] for d in [None] + self.pinsets()]
            out[fname] = body + '\n' + '\n'.join(sc) + '\n'
        return out


def version(name, known, rows, governs='always', head=()):
    return '\n'.join(['version %s' % name, '  cite s', '  governs %s' % governs, '  known %s' % iso(known)] +
                     ['  ' + h for h in head] + ['  ' + r for r in rows])


# -- the places ---------------------------------------------------------------------------------------------------

def t_start(t):
    t.start = t.x  # the stretch crosses the run's start S
    t.common(before=False)


def t_cover(t):
    k1, k2 = t.x - t.a, t.b - t.x
    t.x = last_of_month(t.x)  # the covered span ends inside the stretch
    t.a, t.b, t.after = t.x - k1, t.x + k2, t.x + k2 + 3 * DAY
    t.m, t.m1 = mon(t.x), mon(t.x + DAY)
    t.rows[1:] = ['w = window(%s, %s) "s"' % (iso(t.a), iso(t.b)), 'dx = %s "s"' % iso(t.x)]
    t.places.append('place unexplained: outside lives 1900-01-01..')
    t.versions.append(version('st', t.x + 10 * DAY, ['stated[konto] = 600.00 PLN at %s end "s"' % iso(t.x)],
                              governs=t.m, head=['covers konto', 'pad konto from unexplained']))
    t.common()


def known_replaced(t, known):
    t.inputs.append('input d_at: date')
    t.moves[0] = 'move pw: transfers\n  on w\n  at d_at\n  leg cash from world to konto amount amt'
    t.rows.append('d_at = %s "s"' % iso(t.start))
    t.versions.append(version('a1', known, ['replace amt = 150.00 PLN "s"']))
    t.common()
    t.line('count(a in adjustments over %s .. %s)' % (t.m, t.m1))  # C-4: a count and a sum
    t.line('sum(a.change for a in adjustments over %s .. %s)' % (t.m, t.m1))
    t.line('bal(konto, %s)' % iso(t.after), tail='as known %s' % iso(t.b + 5 * DAY))


def t_known_in(t):
    known_replaced(t, t.x)


def t_known_after(t):
    known_replaced(t, t.b + 2 * DAY)


def t_scen(t):
    t.extra_files['sc.tally'] = ('scenario sc over %s\n  tally 2\n  from %s\n\nmove extra: transfers\n  on w\n'
                                 '  leg cash from world to konto amount amt\n' % (t.name, iso(t.x)))
    t.common()
    for e in ('bal(konto, %s)' % iso(t.after), 'flows(konto, %s)' % t.m, 'bal(other, %s)' % iso(t.after),
              'count(m in transfers over %s where m.cash.to == other)' % t.m):
        t.line(e, scen='sc')


def t_class(t):
    t.inputs.append('input fee: money PLN')
    t.moves[0] = 'move pw: transfers\n  on w\n  at w\n  leg cash from world to konto amount fee'
    t.versions.append(version('old', t.start, ['fee = 5.00 PLN "s"'], governs='%s..%s' % (iso(t.start), iso(t.x))))
    t.versions.append(version('new', t.start, ['fee = 7.00 PLN "s"'], governs='%s..' % iso(t.x + DAY)))
    t.common()


def t_reading(t):
    t.versions.append(version('st', t.x + 10 * DAY, ['stated[konto] = 600.00 PLN at %s end "s"' % iso(t.x)],
                              governs=t.m, head=['anchor konto']))
    t.common()


def t_life(t):
    t.places[0] = 'place konto: accounts PLN lives %s..%s' % (iso(t.start - 366 * DAY), iso(t.x))
    t.common(konto=False)
    t.line('bal(konto, %s)' % iso(t.a - 2 * DAY))


def t_same(t):
    t.inputs.append('input dd: date')
    t.moves.append('move pd: transfers\n  on dd\n  leg cash from world to konto amount amt')
    t.rows.append('dd = %s "s"' % iso(t.x))
    t.versions.append(version('later', t.b + 20 * DAY, ['same pw = pd'], head=()))
    t.common()


def t_register(t):
    t.rows[0] = 'amt = 100.00 PLN assumed "a guess" "s"'
    t.common()
    t.line('bal(konto, %s)' % iso(t.after), pins='amt = 120.00 PLN')
    t.line('bal(other, %s)' % iso(t.after), pins='amt = 120.00 PLN')


def t_ledger(t):
    t.places.append('place unexplained: outside lives 1900-01-01..')
    t.inputs.append('input bank: observed')
    t.versions.append(version('st', last_of_month(t.x) + 10 * DAY, [
        'stated[konto] = 600.00 PLN at %s end "s"' % iso(last_of_month(t.x)),
        'bank[r1] = (on: %s, kind: transfers, leg cash from world to konto amount 100.00 PLN, fulfils: pw) "r1"'
        % iso(t.x)], governs=t.m, head=['covers konto', 'pad konto from unexplained']))
    t.common()
    t.line('residuals konto %s .. %s' % (t.m, t.m1))
    t.anchor[len(t.lines) - 1] = t.x  # the observation answers the day: held to the pin on the observation's day


def t_sign(t):
    t.places[0] = 'place konto: claims PLN lives %s..' % iso(t.start - 366 * DAY)  # can't go below zero
    t.inputs.append('input out: money PLN')
    t.moves.append('move pay_out: transfers\n  on dx\n  leg cash from konto to other amount out')
    t.rows.append('out = 80.00 PLN "s"')
    t.common()


def t_sign_hole(t):
    """The sign rule's twin (§9.2): a hole amount on a plain day into the `claims` place, the window at another."""
    t.places[0] = 'place konto: claims PLN lives %s..' % iso(t.start - 366 * DAY)
    t.inputs += ['input out: money PLN', 'input amt_h: money PLN']
    t.moves = ['move pw: transfers\n  on dx\n  leg cash from world to konto amount amt_h',
               'move px: transfers\n  on w\n  leg cash from world to other amount amt',
               'move pay_out: transfers\n  on dx\n  leg cash from konto to other amount out']
    t.rows += ['out = 80.00 PLN "s"', 'amt_h = ?"how much comes in?" "s"']
    t.common()


def t_div(t):
    """A division by a plain zero beside a hole day (§9.2's last row; the plan's P31, `hd5.tally`)."""
    t.inputs += ['input nz: int', 'input dz: date']
    t.moves.append('move pz: transfers\n  on dz\n  leg cash from world to konto amount round(amt / nz, minor(PLN), half-even)')
    t.rows += ['nz = 0 "s"', 'dz = %s "s"' % iso(t.b + DAY)]
    t.common()


def t_decision(t):
    """A decision (a move with `reads`) whose day is the window: a hole day as any move (the plan's §8.3, row 159)."""
    t.moves[0] = ('move pw: transfers\n  on w\n  when bal(other) >= 0.00 PLN\n  reads other\n'
                  '  leg cash from world to konto amount amt')
    t.common()
    t.line('sum(m.cash.amount for m in transfers over %s)' % t.m)


def t_known_at(t):
    """C-4 beside a hole `at`: the instance on a plain day, its law date the window, its amount replaced after it."""
    t.moves[0] = 'move pw: transfers\n  on dx\n  at w\n  leg cash from world to konto amount amt'
    t.versions.append(version('a1', t.b + 2 * DAY, ['replace amt = 150.00 PLN "s"']))
    t.common()
    t.line('count(a in adjustments over %s .. %s)' % (t.m, t.m1))
    t.line('sum(a.change for a in adjustments over %s .. %s)' % (t.m, t.m1))


def t_when_gone(t):
    """A later version, known inside the window, makes the hole-day instance's `when` false."""
    t.inputs.append('input live: bool')
    t.moves[0] = 'move pw: transfers\n  on w\n  when live\n  leg cash from world to konto amount amt'
    t.rows.append('live = true "s"')
    t.versions.append(version('gone', t.x, ['replace live = false "s"']))
    t.common()
    t.line('count(a in adjustments over %s .. %s)' % (t.m, t.m1))


def t_parts(t):
    t.head_extra = ['kind persons: parties', 'kind cash_accounts: place one min 0\n  owned by arrive\n'
                    '  step minor(PLN)\n  takes k for w2 order (if k.owner == ania then 0 else 1)']
    t.places = ['place acc: cash_accounts PLN lives %s..' % iso(t.start), 'place other: accounts PLN lives %s..'
                % iso(t.start), 'place world: outside lives 1900-01-01..']
    t.inputs.append('input a_out: money PLN')
    t.moves = ['def arrive[m in events, p in parties]: num at m.day\n  rule hers = (if p == ania then 1 else 0)',
               'move pw: transfers\n  on w\n  at dx\n  leg cash from world to acc amount amt',
               'move out: transfers\n  on dx\n  at dx\n  leg cash from acc to world amount a_out',
               'move px: transfers\n  on dx\n  leg cash from world to other amount amt']
    t.rows.append('a_out = 50.00 PLN "s"')
    t.versions.append('version cast\n  cite s\n  known %s\n  persons + [ania] lives 1991-03-14..\n'
                      '  persons + [tomek] lives 1989-07-02..' % iso(t.start - 366 * DAY))
    t.opening = lambda: ['opening[acc] of tomek = 100.00 PLN at %s start "s"' % iso(t.start),
                         'opening[acc] = 100.00 PLN at %s start "s"' % iso(t.start),
                         'opening[other] = 500.00 PLN at %s start "s"' % iso(t.start)]
    for e in ('out.cash.amount of ania', 'out.cash.amount of tomek', 'bal(acc, %s) of tomek' % iso(t.after),
              'bal(other, %s)' % iso(t.after), 'count(m in transfers over %s where m.cash.to == other)' % t.m):
        t.line(e)


# -- G2: the readers (the plan's §9.2 rows 1 to 4, and the refusals of row 6) ---------------------------------------
# A readers' text has no movement: `w` is read by asks alone, its dates and counts are rows (an ask takes no figure),
# and each line says what it must print (`Text.expect`, held by `judge`): `list`, a reader on §4.4's list or a
# composition of them; `off`, a reader off it; `false`, `covered` over a ranged end (ruling 12). A line with none (a
# two-date reader of one window twice, each argument by its own ends, C11) is held to the places' rule.

def shift_years(d, n):
    try:
        return d.replace(year=d.year + n)
    except ValueError:  # 29 February
        return d.replace(year=d.year + n, day=28)


def readers(t, rows=None):
    """The readers' rows: dates before (`db`), inside (`di`) and after (`da`) the window, births whose anniversary
    falls inside it (`bi`, `bm` by the month) or not (`bo`), days of the window's tenth anniversary (`ei`) or not
    (`eo`), counts of days, months and years either way, and a calendar."""
    r, x, a, b = t.rng, t.x, t.a, t.b
    t.moves = ['def wc: date\n  rule base = w', 'def wc2: date\n  rule base = w']
    rows = rows or {
        'db': a - r.randint(2, 40) * DAY, 'di': x, 'da': b + r.randint(2, 40) * DAY,
        'bi': shift_years(x, -30), 'bm': shift_years(x, -2), 'bo': shift_years(x + 100 * DAY, -30),
        'ei': shift_years(x, 10), 'eo': shift_years(x + 100 * DAY, 10),
        'np': r.randint(1, 40), 'nn': -r.randint(1, 40), 'mp': r.randint(1, 14), 'mn': -r.randint(1, 14),
        'yp': r.randint(1, 3), 'yn': -r.randint(1, 3), 'k': r.randint(2, 9)}
    for n, v in sorted(rows.items()):
        t.inputs.append('input %s: %s' % (n, 'int' if isinstance(v, int) else 'date'))
        t.rows.append('%s = %s "s"' % (n, v if isinstance(v, int) else iso(v)))
    t.inputs += ['input y: date', 'input len: calendar']
    t.rows += ['y = 2000-01-01 "s"', 'len = calendar(weekends [sat, sun], yearly [01-01, 12-25]) "s"']


def t_rd_dates(t):
    """The date-to-date readers, each ranged position, n positive and negative: a stretch, or one day."""
    readers(t)
    for e in ('add_days(w, np)', 'add_days(w, nn)', 'add_months(w, mp)', 'add_months(w, mn)', 'add_years(w, yp)',
              'add_years(w, yn)', 'w + 1 day', 'w - 1 day'):
        t.read(e, 'list')
    for d in ('db', 'di', 'da'):
        for e in ('min(w, %s)', 'min(%s, w)', 'max(w, %s)', 'max(%s, w)'):
            t.read(e % d, 'list')


def t_rd_cmp(t):
    """A comparison of dates, each side, against a day before, inside and after the window."""
    readers(t)
    for d in ('db', 'di', 'da'):
        for op in ('<', '<=', '==', '!=', '>=', '>'):
            t.read('w %s %s' % (op, d), 'list')
    t.read('di < w', 'list')
    t.read('da >= w', 'list')


def t_rd_values(t):
    """The readers that give a value where every combination of ends agrees; a period before, at and after."""
    readers(t)
    for e in ('year_of(w)', 'quarter_of(w)', 'month_of(w)', 'age(bi, w)', 'age(bo, w)', 'age_months(bm, w)',
              'age_months(bo, w)', 'age(w, ei)', 'age(w, eo)', 'age_months(w, ei)'):
        t.read(e, 'list')
    for p in (mon(t.a - 45 * DAY), t.m, mon(t.b + 45 * DAY)):
        t.read('overlap_days(w, da, %s)' % p, 'list')
        t.read('overlap_days(db, w, %s)' % p, 'list')


def t_rd_off(t):
    """Readers off the list, each the hole naming the day; `covered` over a ranged end, false (ruling 12)."""
    readers(t)
    for e in ('days_between(w, di)', 'days_between(db, w)', 'dcf(w, da, act365)', 'weekday(w)', 'start_of(w)',
              'end_of(w)', 'is_business(w, len)', 'add_business(w, 1, len)', 'abs(days_between(w, di))',
              'days_between(w, da) * k', 'round(days_between(w, di), 1, down)', 'pow(k, days_between(db, w))',
              'day_of(month_of(di), days_between(db, w))', 'days_between(db, w) / k', 'k / days_between(db, w)'):
        t.read(e, 'off')
    t.read('amt', 'off', tail='law w')  # an ask's `law`
    t.read('covered(konto, %s .. w)' % iso(t.start), 'false')


def t_rd_comp(t):
    """A listed reader of a computed stretch, a copy through a def, an `if` branch, a pin whose value is a copy, a
    field copied by two defs."""
    readers(t)
    for e in ('year_of(add_years(w, yp))', 'month_of(add_months(w, mn))', 'min(add_days(w, np), di) < da',
              'max(add_days(w, nn), db) >= di', 'quarter_of(min(w, di))', 'wc == w', 'wc < da', 'year_of(wc)',
              'add_days(wc, np)', '(if amt > 0.00 PLN then w else db) == w', '(if amt > 0.00 PLN then w else db) <= di',
              'wc == wc2', 'wc2 <= di', 'abs(age(bo, w))', 'age(bo, w) + k',
              # a count from a listed reader, read by a plain function: the value where the ends agree, else the hole
              'round(age(bi, w), 1, down)', 'round(age(bo, w), 1, down)', 'pow(age(bo, w), k)', 'pow(k, age(bi, w))',
              'day_of(month_of(di), age_months(bm, w))', 'age(bi, w) / k', 'count(n in 1 .. age(bi, w))',
              'count(n in 1 .. age(bo, w))', 'add_months(di, age(bi, w))', 'add_days(di, age(bo, w))'):
        t.read(e, 'list')
    t.read('year_of(y)', 'list', pins='y = w')  # a pin whose value is a copy (its value is read in the base, so a
    # pinned day doesn't reach it: `y == w with y = w, w = D` is the hole, and no line here compares it)


def t_rd_pairs(t):
    """Two windows, before, overlapping or after each other: the pins run over every pair of days."""
    readers(t)
    lo, hi = t.rng.choice([(t.a - 4 * DAY, t.a - 2 * DAY), (t.x, t.b + 2 * DAY), (t.b + 2 * DAY, t.b + 4 * DAY)])
    t.v2 = (lo, hi)
    t.inputs.append('input v2: date')
    t.rows.append('v2 = window(%s, %s) "s"' % (iso(lo), iso(hi)))
    for e in ('min(w, v2)', 'max(w, v2)', 'w < v2', 'w <= v2', 'w == v2', 'w > v2', 'overlap_days(w, v2, %s)' % t.m,
              'age(w, v2)'):
        t.read(e, 'list')
    t.read('days_between(w, v2)', 'off')


def t_rd_twice(t):
    """One window in both arguments: a comparison against its own name is one day; any other reader takes each
    argument by its own ends (C11), so its hole where every pin agrees is right (no expectation)."""
    readers(t)
    for e in ('min(w, w)', 'max(w, w)', 'w == w', 'w < w', 'w <= w', 'w != w', 'w >= w'):
        t.read(e, 'list')
    for e in ('age(w, w)', 'age_months(w, w)', 'overlap_days(w, w, %s)' % t.m):
        t.read(e, '')
    t.read('days_between(w, w)', 'off')


def t_rd_refuse(t):
    """A listed reader past the ends of time: at one end the hole, at every end the first's refusal (§4.4)."""
    high = t.rng.random() < 0.5
    k = t.rng.randint(2, 4)
    end, s = (D(9999, 12, 31), 1) if high else (D(1, 1, 1), -1)
    if high:
        t.window(end - k * DAY, end)
    else:
        t.window(end, end + k * DAY)
    readers(t, {'np': s * t.rng.randint(1, k), 'nb': s * (k + t.rng.randint(1, 9)), 'mp': s * t.rng.randint(1, 3),
                'di': end})
    for e in ('add_days(w, np)', 'add_days(w, nb)', 'add_months(w, mp)', 'add_years(w, mp)',
              'w %s 1 day' % ('+' if high else '-'), 'year_of(add_days(w, np))', 'max(w, di)', 'w == di'):
        t.read(e, 'list')


PLACES = [('start', t_start), ('cover', t_cover), ('known_in', t_known_in), ('known_after', t_known_after),
          ('scen', t_scen), ('class', t_class), ('reading', t_reading), ('life', t_life), ('same', t_same),
          ('register', t_register), ('ledger', t_ledger), ('sign', t_sign), ('sign_hole', t_sign_hole), ('div', t_div),
          ('parts', t_parts), ('decision', t_decision), ('known_at', t_known_at), ('when_gone', t_when_gone),
          ('rd_dates', t_rd_dates), ('rd_cmp', t_rd_cmp), ('rd_values', t_rd_values), ('rd_off', t_rd_off),
          ('rd_comp', t_rd_comp), ('rd_pairs', t_rd_pairs), ('rd_twice', t_rd_twice), ('rd_refuse', t_rd_refuse)]
WAITS = {'engine': {'parts': 'M5b: the engine keeps no parts before it'}}
# Faults a review or this oracle found, on refcore and the engine alike, each mended at a later pin: a line of the
# named place whose expression starts so is held by name (reported, not failed) while it differs; any other line of
# the place is held to the rule as usual. Delete a row when its pin lands (B1, B2 and Q3 went with the second pin).
HELD = {
    ('known_in', 'pw.cash.amount'): 'N1: a hole-day instance a later version changes prints the new amount',
    ('known_after', 'pw.cash.amount'): 'N1',
    ('rd_off', 'covered('): 'R12: `covered` over a ranged end prints the hole, where ruling 12 makes it false',
}
# A place whose every line is the hole by a rule kept (N2, ruled: a window `at` holes every place, so a plain
# movement's balance beside it is the hole too): counted apart, neither a pass nor a failure. One hand-stored vector
# keeps its words (`m3/fixes/t23/at/n2_at.tally`).
APART = {
    'class': 'N2: a window `at` holes every place',
    'known_at': 'N2',
}


def held(place, expr):
    return next((why for (p, pre), why in HELD.items() if p == place and expr.startswith(pre)), '')


# -- running and judging ------------------------------------------------------------------------------------------

def command(interp):
    py = PYPY if os.path.exists(PYPY) else sys.executable
    if interp == 'engine':
        return [os.path.join(ROOT, 'engine', 'target', 'release', 'tally'), 'run']
    tool = 'refcore_fast' if interp == 'fast' else 'refcore'
    return [py, '-I', '-B', os.path.join(ROOT, 'engine', 'tools', tool), 'run']


HEAD = re.compile(r'^(l\d\d)(?:_p(\d{8}(?:\d{8})?))?: (.*)$')
def names_w(u):
    """A label list `(…)` with no quote or parenthesis inside, followed by a space or the line's end, one of whose
    comma-separated names is `w` or `v2` (a window's cell; the runner's `m3_oracle` reads labels the same way)."""
    i = u.find('(')
    while i >= 0:
        j = u.find(')', i)
        if j < 0:
            return False
        inner = u[i + 1:j]
        if '"' not in inner and '(' not in inner and {'w', 'v2'} & {x.strip() for x in inner.split(',')} \
                and (j + 1 == len(u) or u[j + 1] == ' '):
            return True
        i = u.find('(', i + 1)
    return False



DATE = re.compile(r'^\d{4}-\d{2}-\d{2}$')
STRETCH = re.compile(r'^window\((\d{4}-\d{2}-\d{2}), (\d{4}-\d{2}-\d{2})\)$')


# The register under an answer: a pinned ask prints `pinned:`, an unpinned one `unknown day:`, and the rest carry a
# day by design. §9.4 keeps their words hand-stored (the hole's label, `unknown day:`, C-4's register); the oracle
# holds the answer and its body (a ledger's lines, an explanation's).
REGISTER = re.compile(r'^  (cut|unfulfilled|pinned|assumed|adjustment|pad|unknown day|anchor|open|left out|world)\b')
# The days counted apart: a balance check's refusal (C-3) and a movement that doesn't exist (`RUN-NONE`, ruling 15: a
# field read of a movement whose `when` is a hole prints its amount, though a day that answers the `when` false
# refuses it).
BALANCE_CHECK = re.compile(r'^refused: (RUN-PLACE|RUN-LIFESPAN|RUN-TAKE|RUN-PART|RUN-NONE)\b')


def judge_block(u, pinned, expect=''):
    """A printed answer of several lines (a ledger, an explanation): the same verdict line by line where every block
    has the same shape, its refused days those whose first line a balance check refuses; `block` where shapes differ
    and the unpinned block isn't what every unrefused day prints."""
    refused = {d for d, v in pinned.items() if BALANCE_CHECK.match(v[0])}
    rest = [v for d, v in pinned.items() if d not in refused]
    if len(u) == 1 and all(len(v) == 1 for v in pinned.values()):
        return judge(u[0], {d: v[0] for d, v in pinned.items()}, expect=expect)
    if rest and all(v == u for v in rest):
        return '', ('hole' if u[0].startswith('?') else 'refusal' if u[0].startswith('refused:') else 'value'), ''
    if any(len(v) != len(u) for v in pinned.values()):
        return 'a block of %d lines where the pinned days print %s lines' % (
            len(u), '/'.join(sorted({str(len(v)) for v in pinned.values()}))), 'value', 'block'
    kind = None
    for i in range(len(u)):
        why, k, fault = judge(u[i].strip(), {d: v[i].strip() for d, v in pinned.items()}, refused)
        kind = kind or k
        if why:
            return 'line %d of the block: %s' % (i + 1, why), kind, fault
    return '', kind, ''


CITE = ['']  # the text being judged: its window as printed, `window(a, b)`


def judge(u, pinned, refused_days=None, expect=''):
    """(verdict, kind, fault): kind is value, hole, refusal or stretch; verdict '' when the line holds; fault, when it
    doesn't, is `label` (a hole naming no day: on 2.2.1 the movement's name stands where 2.3 names `w`), `refusal`,
    `stretch` or `value` (a printed value no unrefused pinned day agrees with: #244's kind), and for a reader's line
    `list` (the hole where every pinned day prints one value, or a stretch wider than the pinned prints), `off` (a
    value from a reader off the list) or `known` (`covered` not false).

    A reader's line is held harder than a place's: on the list (`list`), every reader rises or falls with each
    argument, so the ends agree exactly where every pinned day agrees, and the line prints that value, never the
    hole; a date prints the stretch from the first pinned print to the last. Off the list (`off`) it prints the hole
    naming the day whatever the days print, or the refusal every day prints. `covered` (`false`) prints false."""
    if refused_days is None:
        refused_days = {d for d, v in pinned.items() if BALANCE_CHECK.match(v)}
    refused = {d: v for d, v in pinned.items() if d in refused_days}
    rest = {d: v for d, v in pinned.items() if d not in refused}
    vals = set(rest.values())
    kind = 'hole' if u.startswith('?') else 'refusal' if u.startswith('refused:') else 'value'
    if expect == 'false':
        return ('', kind, '') if u == 'false' else ('printed %s where a test of what is known is false' % u[:90],
                                                     kind, 'known')
    if expect == 'off' and not (kind == 'refusal' and vals == {u}):
        ok = kind == 'hole' and names_w(u)
        return ('', kind, '') if ok else ('a reader off the list printed %s, not the hole' % u[:90], kind, 'off')
    if expect == 'list' and len(vals) == 1 and u not in vals and kind == 'hole':
        return ('the hole where every pinned day prints %s: %s' % (next(iter(vals))[:40], u[:90]), kind, 'list')
    if expect == 'list' and STRETCH.match(u) and rest and all(DATE.match(v) for v in vals) and \
            STRETCH.match(u).groups() != (min(vals), max(vals)):
        return ('the stretch %s where the pinned days print %s to %s' % (u, min(vals), max(vals)), 'stretch', 'list')
    if vals == {u}:  # (1): every unrefused day prints it, a hole or refusal the window doesn't cause included
        return '', kind, ''
    if kind == 'hole':  # names the day: `w` among its labels, or its words cite this window (C13's HOLE-PART)
        ok = names_w(u) or (CITE[0] and 'date within %s' % CITE[0] in u)
        return ('', kind, '') if ok else ('a hole that names no day: ' + u[:90], kind, 'label')
    if kind == 'refusal':
        return ('', kind, '') if u in refused.values() else ('a refusal no pinned day prints: ' + u[:90], kind, 'refusal')
    m = STRETCH.match(u)
    if m:
        ok = rest and all(DATE.match(v) and m.group(1) <= v <= m.group(2) for v in rest.values())
        return ('', 'stretch', '') if ok else ('a stretch that doesn\'t hold every pinned day: ' + u, 'stretch', 'stretch')
    return ('printed %s where the pinned days print %s' % (u[:60], ' | '.join(sorted(v[:40] for v in vals))[:200]),
            'value', 'value')


def teeth(t):
    """Whether a text has a line that may print a value: one whose every line is off the list or a test of what is
    known (`rd_off`) is held line by line, never as empty."""
    return any(t.expect.get(i, '') not in ('off', 'false') for i in range(len(t.lines)))


def run_text(t, interp, out, place=''):
    d = os.path.join(out, t.name)
    os.makedirs(d, exist_ok=True)
    files = t.files()
    for f, body in files.items():
        open(os.path.join(d, f), 'w', encoding='utf-8').write(body)
    p = subprocess.run(command(interp) + sorted(files, key=lambda f: f != t.name + '.tally'), cwd=d,
                       capture_output=True, text=True, timeout=600)
    o = p.stdout + p.stderr
    open(os.path.join(d, 'run.out'), 'w', encoding='utf-8').write(o)
    got, cur = {}, None  # each ask's block: its head line's answer and the indented lines under it
    for l in o.splitlines():
        m = HEAD.match(l)
        if m:
            cur = got[(m.group(1), m.group(2))] = [m.group(3)]
        elif cur is not None and REGISTER.match(l):
            continue
        elif cur is not None and l.startswith(' '):
            cur.append(l)
        else:
            cur = None
    res = {'lines': 0, 'values': 0, 'refused': 0, 'fails': [], 'held': [], 'apart': [], 'refused_at': set(),
           'faults': {}}
    if not got:
        res['fails'].append('refused at load or printed nothing: ' + (o.splitlines() or [''])[0][:150])
        res['faults']['load'] = 1
        return res
    for i in range(len(t.lines)):
        k = 'l%02d' % i
        u = got.get((k, None))
        pinned = {key: got.get((k, key)) for key, _ in t.pinsets()}
        if u is None or None in pinned.values():
            res['fails'].append('%s: no line printed' % k)
            continue
        res['lines'] += 1
        nref = sum(bool(BALANCE_CHECK.match(v[0])) for v in pinned.values())
        res['refused'] += nref
        if nref:
            res['refused_at'].add(t.lines[i][0].split('(')[0])
        CITE[0] = 'window(%s, %s)' % (iso(t.a), iso(t.b))
        if i in t.anchor:  # an observation answers the day: the pin on its day is the reality it records
            day = t.anchor[i].strftime('%Y%m%d')
            pinned = {day: pinned[day]}
        why, kind, fault = judge_block(u, pinned, t.expect.get(i, ''))
        res['values'] += kind in ('value', 'stretch') and not why
        hold = why and held(place, t.lines[i][0])
        if hold:
            res['held'].append('%s `%s`: held (%s): %s' % (k, t.lines[i][0], hold.split(':')[0], why))
        elif why:
            res['fails'].append('%s `%s`: %s' % (k, t.lines[i][0], why))
            res['faults'][fault] = res['faults'].get(fault, 0) + 1
    if res['lines'] and not res['values'] and not res['fails'] and place in APART:
        res['apart'].append('every line the hole, counted apart (%s)' % APART[place])
    elif res['lines'] and not res['values'] and not res['fails'] and teeth(t):
        res['fails'].append('empty: every unpinned line is a hole or a refusal')
        res['faults']['empty'] = 1
    return res


CORPUS = os.path.join(TESTS, 'properties', 'oracle.corpus')


def ns(b):
    return b'%d:%s,' % (len(b), b)


def corpus_bytes(texts):
    """The texts the engine target `m3_oracle` runs (all but `WAITS['engine']`'s), nine netstrings each: id, place,
    the command line's words (newline-joined), the file table (INTERFACE.md §4), the pinned runs' keys (YYYYMMDD, and
    the second window's day after it in a text of two, space-joined, w's first and last day first and last), the
    number of lines, the held lines (`<index> <code>`, and `apart <code>` for `APART`'s place, newline-joined), the
    anchored lines (`<index> <YYYYMMDD>`, newline-joined) and the readers' expectations (`<index> list|off|false`,
    newline-joined)."""
    out = b''
    for name, t in texts:
        if name in WAITS['engine']:
            continue
        files = t.files()
        order = sorted(files, key=lambda f: f != t.name + '.tally')
        table = b''.join(ns(f.encode()) + ns(files[f].encode()) for f in order)
        hl = ['%d %s' % (i, held(name, t.lines[i][0]).split(':')[0]) for i in range(len(t.lines))
              if held(name, t.lines[i][0])] + (['apart ' + APART[name].split(':')[0]] if name in APART else [])
        an = ['%d %s' % (i, d.strftime('%Y%m%d')) for i, d in sorted(t.anchor.items())]
        ex = ['%d %s' % (i, e) for i, e in sorted(t.expect.items()) if e]
        for part in (t.name, name, '\n'.join(['run'] + order), None, ' '.join(k for k, _ in t.pinsets()),
                     str(len(t.lines)), '\n'.join(hl), '\n'.join(an), '\n'.join(ex)):
            out += ns(table) if part is None else ns(part.encode())
    return out


def main(argv):
    opt = {'--interp': 'fast', '--seed': '20261121', '--variants': '3', '--sample': '1', '--only': '',
           '--out': os.path.join(TESTS, 'out', 'oracle')}
    i = 0
    while i < len(argv):
        a = argv[i].split('=', 1)
        if a[0] in ('--keep', '--check', '--corpus'):  # --check: run.py --self-test's form, and the corpus matches
            opt[a[0]] = '1'
        elif a[0] in opt:
            opt[a[0]] = a[1] if len(a) > 1 else argv[i + 1]
            i += len(a) == 1
        else:
            sys.exit('unknown option ' + argv[i])
        i += 1
    rng = random.Random(int(opt['--seed']))
    only = set(filter(None, opt['--only'].split(',')))
    texts = []
    for name, f in PLACES:
        for v in range(int(opt['--variants'])):
            t = Text('o_%s_%d' % (name, v), random.Random(rng.random()))
            f(t)
            texts.append((name, t))
    if '--corpus' in opt:
        open(CORPUS, 'wb').write(corpus_bytes(texts))
        print('wrote %s: %d texts' % (os.path.relpath(CORPUS, ROOT), sum(n not in WAITS['engine'] for n, _ in texts)))
        return 0
    stale = '--check' in opt and (not os.path.exists(CORPUS) or open(CORPUS, 'rb').read() != corpus_bytes(texts))
    k = int(opt['--sample'])
    texts = [x for j, x in enumerate(texts) if j % k == int(opt['--seed']) % k and (not only or x[0] in only)]
    if os.path.isdir(opt['--out']) and '--keep' not in opt:
        shutil.rmtree(opt['--out'])
    tot = {'texts': 0, 'lines': 0, 'values': 0, 'refused': 0, 'failed': 0, 'held': 0, 'apart': 0}
    faults, by_place = {}, {}
    for name, t in texts:
        wait = WAITS.get(opt['--interp'], {}).get(name)
        if wait:
            print('%-16s skipped: %s' % (t.name, wait))
            continue
        r = run_text(t, opt['--interp'], opt['--out'], name)
        tot['texts'] += 1
        for key in ('lines', 'values', 'refused'):
            tot[key] += r[key]
        tot['failed'] += len(r['fails'])
        tot['held'] += len(r['held'])
        tot['apart'] += len(r['apart'])
        print('%-16s window(%s, %s): %d lines, %d value lines, %d refused days%s, %s%s' % (
            t.name, iso(t.a), iso(t.b), r['lines'], r['values'], r['refused'],
            ' (at %s)' % ', '.join(sorted(r['refused_at'])) if r['refused_at'] else '',
            'holds' if not r['fails'] else '%d FAILED' % len(r['fails']),
            ', %d held' % len(r['held']) if r['held'] else ''))
        for f in r['fails'] + r['held'] + r['apart']:
            print('    ' + f)
        for fk, n in r['faults'].items():
            faults[fk] = faults.get(fk, 0) + n
            by_place.setdefault(fk, set()).add(name)
    if faults:
        print('faults: ' + '; '.join('%s %d (%s)' % (fk, faults[fk], ', '.join(sorted(by_place[fk])))
                                     for fk in sorted(faults)))
    if stale:
        print('oracle.corpus is not what --corpus writes')
    print('oracle: %d texts, %d lines, %d value lines, %d refused days, %d texts apart, %d failed, %d held by name' % (
        tot['texts'], tot['lines'], tot['values'], tot['refused'], tot['apart'], tot['failed'], tot['held']))
    return 1 if tot['failed'] or not tot['texts'] or stale else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
