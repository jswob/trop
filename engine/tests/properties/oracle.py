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
movements stated `same`, the register, the ledger), the sign rule, a count over `adjustments` (C-4) and, C13 being
taken, a take at a place that keeps parts (`parts`: on the engine it waits for M5b and is skipped). Each holds a plain
movement at another place and counts beside it. Its readers join in G2.

    python3 -I -B engine/tests/properties/oracle.py [--interp refcore|fast|engine] [--seed N] [--variants N]
        [--sample K] [--only NAME[,NAME]] [--out DIR] [--keep]
Exit 0 when every line holds and no text is empty or refused at load; 1 otherwise. Last line:
`oracle: <t> texts, <l> lines, <v> value lines, <r> refused days, <f> failed`.
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
        self.versions, self.extra_files, self.head_extra, self.lines = [], {}, [], []
        self.m, self.m1 = mon(self.x), mon(last_of_month(self.x) + DAY)
        self.after = self.b + 3 * DAY

    def common(self, before=True, konto=True):
        if before and konto:
            self.line('bal(konto, %s)' % iso(self.a - 2 * DAY))
        if konto:
            self.line('bal(konto, %s)' % iso(self.x))
            self.line('bal(konto, %s)' % iso(self.after))
            self.line('flows(konto, %s)' % self.m)
        self.line('bal(other, %s)' % iso(self.after))
        self.line('flows(other, %s)' % self.m)
        self.line('count(m in transfers over %s where m.cash.to == other)' % self.m)
        self.line('count(m in transfers over %s)' % self.m)

    def line(self, expr, scen='', pins='', tail=''):
        self.lines.append((expr, scen, pins, tail))

    def opening(self):
        return ['opening[konto] = 500.00 PLN at %s start "s"' % iso(self.start),
                'opening[other] = 500.00 PLN at %s start "s"' % iso(self.start)]

    def ask(self, i, day):
        expr, scen, pins, tail = self.lines[i]
        ps = [p for p in (pins, 'w = %s' % iso(day) if day else '') if p]
        return 'ask l%02d%s: %s%s%s%s' % (i, '_p' + day.strftime('%Y%m%d') if day else '', expr,
                                         ' in ' + scen if scen else '', ' with ' + ', '.join(ps) if ps else '',
                                         ' ' + tail if tail else '')

    def days(self):
        return [self.a + i * DAY for i in range((self.b - self.a).days + 1)]

    def files(self):
        asks = [self.ask(i, d) for i in range(len(self.lines)) if not self.lines[i][1] for d in [None] + self.days()]
        main = '\n'.join(['model %s' % self.name, '  tally 2', '  on %s' % iso(self.after + 60 * DAY),
                          '  start %s' % iso(self.start), '  end %s' % iso(self.end), 'source s "invented"',
                          'unit PLN: currency minor 0.01'] + self.head_extra + self.places + self.inputs + self.moves +
                         ['version v\n  cite s\n  governs always\n  known %s\n' % iso(self.start - 366 * DAY) +
                          '\n'.join('  ' + r for r in self.rows + self.opening())] + self.versions + asks) + '\n'
        out = {self.name + '.tally': main}
        for fname, body in self.extra_files.items():
            sc = [self.ask(i, d) for i in range(len(self.lines)) if self.lines[i][1] for d in [None] + self.days()]
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
    t.line('count(a in adjustments over %s .. %s)' % (t.m, t.m1))  # C-4
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


def t_sign(t):
    t.places[0] = 'place konto: claims PLN lives %s..' % iso(t.start - 366 * DAY)  # can't go below zero
    t.inputs.append('input out: money PLN')
    t.moves.append('move pay_out: transfers\n  on dx\n  leg cash from konto to other amount out')
    t.rows.append('out = 80.00 PLN "s"')
    t.common()


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


PLACES = [('start', t_start), ('cover', t_cover), ('known_in', t_known_in), ('known_after', t_known_after),
          ('scen', t_scen), ('class', t_class), ('reading', t_reading), ('life', t_life), ('same', t_same),
          ('register', t_register), ('ledger', t_ledger), ('sign', t_sign), ('parts', t_parts)]
WAITS = {'engine': {'parts': 'M5b: the engine keeps no parts before it'}}


# -- running and judging ------------------------------------------------------------------------------------------

def command(interp):
    py = PYPY if os.path.exists(PYPY) else sys.executable
    if interp == 'engine':
        return [os.path.join(ROOT, 'engine', 'target', 'release', 'tally'), 'run']
    tool = 'refcore_fast' if interp == 'fast' else 'refcore'
    return [py, '-I', '-B', os.path.join(ROOT, 'engine', 'tools', tool), 'run']


HEAD = re.compile(r'^(l\d\d)(?:_p(\d{8}))?: (.*)$')
NAMES_W = re.compile(r'\((?:[^()"]*,\s*)?w(?:,[^()"]*)?\)\s*$|\((?:[^()"]*,\s*)?w(?:,[^()"]*)?\) ')
DATE = re.compile(r'^\d{4}-\d{2}-\d{2}$')
STRETCH = re.compile(r'^window\((\d{4}-\d{2}-\d{2}), (\d{4}-\d{2}-\d{2})\)$')


def judge(u, pinned):
    """(verdict, kind, fault): kind is value, hole, refusal or stretch; verdict '' when the line holds; fault, when it
    doesn't, is `label` (a hole naming no day: on 2.2.1 the movement's name stands where 2.3 names `w`), `refusal`,
    `stretch` or `value` (a printed value no unrefused pinned day agrees with: #244's kind)."""
    refused = {d: v for d, v in pinned.items() if v.startswith('refused:')}
    rest = {d: v for d, v in pinned.items() if d not in refused}
    vals = set(rest.values())
    kind = 'hole' if u.startswith('?') else 'refusal' if u.startswith('refused:') else 'value'
    if vals == {u}:  # (1): every unrefused day prints it, a hole or refusal the window doesn't cause included
        return '', kind, ''
    if kind == 'hole':
        return ('', kind, '') if NAMES_W.search(u) else ('a hole that names no day: ' + u[:90], kind, 'label')
    if kind == 'refusal':
        return ('', kind, '') if u in refused.values() else ('a refusal no pinned day prints: ' + u[:90], kind, 'refusal')
    m = STRETCH.match(u)
    if m:
        ok = rest and all(DATE.match(v) and m.group(1) <= v <= m.group(2) for v in rest.values())
        return ('', 'stretch', '') if ok else ('a stretch that doesn\'t hold every pinned day: ' + u, 'stretch', 'stretch')
    return ('printed %s where the pinned days print %s' % (u[:60], ' | '.join(sorted(v[:40] for v in vals))[:200]),
            'value', 'value')


def run_text(t, interp, out):
    d = os.path.join(out, t.name)
    os.makedirs(d, exist_ok=True)
    files = t.files()
    for f, body in files.items():
        open(os.path.join(d, f), 'w', encoding='utf-8').write(body)
    p = subprocess.run(command(interp) + sorted(files, key=lambda f: f != t.name + '.tally'), cwd=d,
                       capture_output=True, text=True, timeout=600)
    o = p.stdout + p.stderr
    open(os.path.join(d, 'run.out'), 'w', encoding='utf-8').write(o)
    got = {}
    for l in o.splitlines():
        m = HEAD.match(l)
        if m:
            got[(m.group(1), m.group(2))] = m.group(3)
    res = {'lines': 0, 'values': 0, 'refused': 0, 'fails': [], 'refused_at': set(), 'faults': {}}
    if not got:
        res['fails'].append('refused at load or printed nothing: ' + (o.splitlines() or [''])[0][:150])
        res['faults']['load'] = 1
        return res
    for i in range(len(t.lines)):
        k = 'l%02d' % i
        u = got.get((k, None))
        pinned = {dd: got.get((k, dd.strftime('%Y%m%d'))) for dd in t.days()}
        if u is None or None in pinned.values():
            res['fails'].append('%s: no line printed' % k)
            continue
        res['lines'] += 1
        nref = sum(v.startswith('refused:') for v in pinned.values())
        res['refused'] += nref
        if nref:
            res['refused_at'].add(t.lines[i][0].split('(')[0])
        why, kind, fault = judge(u, pinned)
        res['values'] += kind in ('value', 'stretch') and not why
        if why:
            res['fails'].append('%s `%s`: %s' % (k, t.lines[i][0], why))
            res['faults'][fault] = res['faults'].get(fault, 0) + 1
    if res['lines'] and not res['values'] and not res['fails']:
        res['fails'].append('empty: every unpinned line is a hole or a refusal')
        res['faults']['empty'] = 1
    return res


def main(argv):
    opt = {'--interp': 'fast', '--seed': '20261121', '--variants': '3', '--sample': '1', '--only': '',
           '--out': os.path.join(TESTS, 'out', 'oracle')}
    i = 0
    while i < len(argv):
        a = argv[i].split('=', 1)
        if a[0] == '--keep':
            opt['--keep'] = '1'
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
    k = int(opt['--sample'])
    texts = [x for j, x in enumerate(texts) if j % k == int(opt['--seed']) % k and (not only or x[0] in only)]
    if os.path.isdir(opt['--out']) and '--keep' not in opt:
        shutil.rmtree(opt['--out'])
    tot = {'texts': 0, 'lines': 0, 'values': 0, 'refused': 0, 'failed': 0}
    faults, by_place = {}, {}
    for name, t in texts:
        wait = WAITS.get(opt['--interp'], {}).get(name)
        if wait:
            print('%-16s skipped: %s' % (t.name, wait))
            continue
        r = run_text(t, opt['--interp'], opt['--out'])
        tot['texts'] += 1
        for key in ('lines', 'values', 'refused'):
            tot[key] += r[key]
        tot['failed'] += len(r['fails'])
        print('%-16s window(%s, %s): %d lines, %d value lines, %d refused days%s, %s' % (
            t.name, iso(t.a), iso(t.b), r['lines'], r['values'], r['refused'],
            ' (at %s)' % ', '.join(sorted(r['refused_at'])) if r['refused_at'] else '',
            'holds' if not r['fails'] else '%d FAILED' % len(r['fails'])))
        for f in r['fails']:
            print('    ' + f)
        for fk, n in r['faults'].items():
            faults[fk] = faults.get(fk, 0) + n
            by_place.setdefault(fk, set()).add(name)
    if faults:
        print('faults: ' + '; '.join('%s %d (%s)' % (fk, faults[fk], ', '.join(sorted(by_place[fk])))
                                     for fk in sorted(faults)))
    print('oracle: %d texts, %d lines, %d value lines, %d refused days, %d failed' % (
        tot['texts'], tot['lines'], tot['values'], tot['refused'], tot['failed']))
    return 1 if tot['failed'] else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
