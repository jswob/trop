"""hh4.py: household X's hand-checked vectors (spec/smoke/HH4-a.md, HH4-b.md) restated in tally 2.1.1's output form
for milestone S (issue #16; DONE gate 3; MILESTONE-S.md row 4). Standard library only.

A vector is a line refcore prints for X's base in one of milestone S's three checks (`I:X:konto_balance@b_r1`,
`I:X:replay_september@b_r1`, `I:X:replay_september@b_r1s`, on `integration/amendments/cut11/`), with the figures on
it that HH4's two hand runs wrote, each with the HH4 line it is written on, and the rule it checks. Nothing in a
vector is worked by hand: SPEC names each line by how it starts and each figure as HH4 writes it; this script takes
the whole line from refcore's output, reads the figures off it and checks each against HH4's text at its lines.

    python3 -I -B engine/tests/hh4/hh4.py            check vectors.tsv: as SPEC and refcore's outputs give it, and
                                                     every figure on refcore's line and on its HH4 lines
    python3 -I -B engine/tests/hh4/hh4.py --write    write vectors.tsv (refused while any figure disagrees)
    python3 -I -B engine/tests/hh4/hh4.py --self-test  each kind of disagreement, planted, is caught
    python3 -I -B engine/tests/hh4/hh4.py --store    copy refcore's runs of the three checks from run.py's cache
                                                     into refcore/ (plain refcore when cached, else refcore_fast,
                                                     which TESTING.md section 5 holds byte-equal to it)

vectors.tsv's columns: `id`; `checks` (the checks whose output prints it); `block` (the ask whose block holds it, or
`warning`); `kind` (`line`: the line is printed in the block, the block's lines in the table's order; `absent`: no
line of the block holds the text); `text` (the whole line, or the text that is absent); `figures` (`field = HH4's
text @ HH4 lines`, `;`-separated); `rule` (the spec sections it checks). The engine is held to it by tally-runner's
milestone S target `s_household` (in process, INTERFACE.md section 4); tally-runner's own tests hold it on refcore's
outputs in refcore/. The last line ends `: 0 failed` when every check passes (the key scripts' convention).

Fields read off a line: `value1`, `value2` (an ask's two values); `model`, `observed`, `difference`, `booked`,
`stated`, `gap` (an amount after the word); `-model` (HH4's figure is minus the model's amount: an expected but
missing line's difference, ENGINE section 6.3 Signs); `timing` (days); `day` (the date after ` on `, or the instant's
date); `cut`, `from` (the date after the word); `names` (HH4's text is on the line as written). A figure's HH4 text
must stand on each of its HH4 lines as written (not inside a longer number). HH4 writes U+2212 for minus.
"""
import importlib.util
import json
import os
import re
import sys
from decimal import Decimal, InvalidOperation

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
SMOKE = os.path.join(ROOT, 'engine/spec/smoke')
TSV = os.path.join(HERE, 'vectors.tsv')
REFCORE = os.path.join(HERE, 'refcore')
COLS = ['id', 'checks', 'block', 'kind', 'text', 'figures', 'rule']

K = 'I:X:konto_balance@b_r1'
R1 = 'I:X:replay_september@b_r1'
R1S = 'I:X:replay_september@b_r1s'
CHECKS = [K, R1, R1S]
REPLAY = [R1, R1S]

# HH4's lines, cited as a:<n> (HH4-a.md) and b:<n> (HH4-b.md)
L_LEDGER = 'E §6.3 (lines, signs, totals, order); L §5.18 residuals'
L_MATCH = 'E §6.3 matched; L §5.19 fulfils; E §6.1 booking observations'
L_MISS = 'E §6.3 expected but missing, signs; E §6.2 the modelled past'
L_UNEX = 'E §6.3 unexplained; L §5.19'


def group(gid, name, model, observed, diff, cites, extra=()):
    """A group line of the ledger: `  2026-09 move <name>: model …, observed …, difference …`."""
    start = '  2026-09 %s: ' % name
    return (gid, REPLAY, 'replay_september', 'line', start,
            [('model', model, cites), ('observed', observed, cites), ('difference', diff, cites)] + list(extra),
            L_LEDGER)


def matched(vid, inst, names_cite, day=None, day_cite=None):
    """A matched line HH4 gives as 'a difference of 0.00 with timing 0' (a:233)."""
    figs = [('names', inst, names_cite), ('difference', '0.00', 'a:233'), ('timing', 'timing 0', 'a:233')]
    if day:
        figs.append(('day', day, day_cite))
    return (vid, REPLAY, 'replay_september', 'line', '    matched %s on ' % inst, figs, L_MATCH)


SPEC = [
    # konto_balance at 2026-10-31 (b_r1's ask, cut to its first date): its values and the register entries HH4 states
    ('K-1', [K], 'konto_balance', 'line', 'konto_balance (d = 2026-10-31): ',
     [('value1', '6064.17', 'a:638 b:716'), ('value2', '6064.17', 'a:638 b:716')],
     'L §5.10 bal; E §4.2 the day walk (the 10-15 invest: room by the projection, L §5.14.1); E §6.1 the anchor'),
    ('K-2', [K], 'konto_balance', 'line', '  anchor: konto 2026-09-30 end, ',
     [('stated', '41386.27', 'a:70 b:363'), ('difference', '+32097.23', 'a:253 b:364'), ('day', '09-30', 'a:70')],
     'L §8.2 anchor; E §6.1 readings (anchor)'),
    ('K-3', [K], 'konto_balance', 'line', '  law date 2026-10-19, cut ',
     [('cut', '2026-10-19', 'a:19 b:15 b:705')], 'L §8.2 law date and cut; L §4.5 the cut'),
    # replay_september: the residual ledger of konto, 2026-09, on b_r1 (with konto_balance's run) and b_r1s (alone)
    ('R-0', REPLAY, 'replay_september', 'line', 'replay_september: residuals konto 2026-09',
     [('names', 'residuals konto 2026-09', 'b:377')], 'L §5.18 residuals; E §6.3 (K = on: the replay)'),
    group('R-1', 'balance', '9289.04', '41386.27', '+32097.23', 'b:391'),
    ('R-1.1', REPLAY, 'replay_september', 'line', '    balance konto 2026-09-30 end (anchored): ',
     [('booked', '9289.04', 'a:612'), ('stated', '41386.27', 'a:612'), ('gap', '+32097.23', 'a:253 a:612'),
      ('day', '2026-09-30', 'b:391')], 'E §6.3 balance line (stated − booked); E §6.1 readings (anchor)'),
    group('R-2', 'move spend', '−4599.90', '0.00', '+4599.90', 'b:392'),
] + [
    ('R-2.%d' % d, REPLAY, 'replay_september', 'line',
     '    expected but missing spend[2026-09-%02d] on 2026-09-%02d: ' % (d, d),
     [('-model', '+153.33', 'a:247 b:392')], L_MISS)
    for d in range(1, 31)
] + [
    group('R-3', 'move client_pay', '+20910.00', '+17220.00', '−3690.00', 'b:393'),
    ('R-3.1', REPLAY, 'replay_september', 'line', '    expected but missing client_pay[client_b, 2026-08] on ',
     [('names', 'client_pay[client_b, 2026-08]', 'a:244 b:393'), ('day', '2026-09-30', 'b:393'),
      ('-model', '−3690.00', 'a:244 b:393')], L_MISS),
    ('R-3.2', REPLAY, 'replay_september', 'line', '    matched client_pay[client_a, 2026-08] on ',
     [('names', 'client_pay[client_a, 2026-08]', 'a:235 b:393'), ('difference', '0.00', 'a:233 b:393'),
      ('timing', 'timing 0', 'a:233 b:393')], L_MATCH),
    group('R-4', 'unexplained', '0.00', '−3574.02', '−3574.02', 'b:394'),
] + [
    ('R-4.%d' % n, REPLAY, 'replay_september', 'line', '    unexplained bank[r%s] on ' % row,
     [('observed', amount, 'b:394')], L_UNEX)
    for n, (row, amount) in enumerate([('20260919a', '−899.99'), ('20260916a', '−530.80'), ('20260902a', '−412.37'),
                                       ('20260929a', '−401.09'), ('20260923a', '−348.62'), ('20260909a', '−286.15'),
                                       ('20260903a', '−250.00'), ('20260918a', '−180.00'), ('20260911a', '−145.00'),
                                       ('20260926a', '−120.00')], 1)
] + [
    group('R-5', 'move invest', '−6535.30', '−6000.00', '+535.30', 'b:395'),
    ('R-5.1', REPLAY, 'replay_september', 'line', '    expected but missing invest[mak, 2026-09] on ',
     [('names', 'invest[mak, 2026-09]', 'a:246 b:395'), ('day', '09-15', 'a:246'),
      ('-model', '+6535.30', 'a:246 b:395')], L_MISS),
    ('R-5.2', REPLAY, 'replay_september', 'line', '    matched invest[ike, 2026-09] on ',
     [('names', 'invest[ike, 2026-09]', 'a:238 b:395'), ('model', '0.00', 'b:395'),
      ('observed', '−6000.00', 'a:238 b:395'), ('difference', '−6000.00', 'a:238 b:395'),
      ('timing', 'timing 0', 'b:395')],
     L_MATCH + '; E §4.2 step 3 (an observation booked at its instance\'s point)'),
    group('R-6', 'move loan_instalment', '−300.00', '0.00', '+300.00', 'b:396'),
    ('R-6.1', REPLAY, 'replay_september', 'line', '    expected but missing loan_instalment[laptop, 4] on ',
     [('names', 'loan_instalment[laptop, 4]', 'a:245 b:396'), ('day', '2026-09-05', 'b:396'),
      ('-model', '+300.00', 'a:245')], L_MISS),
    group('R-7', 'move costs_pay', '−492.00', '−492.00', '0.00', 'b:397'),
    matched('R-7.1', 'costs_pay[2026-09]', 'a:234', 'Sep 7', 'b:397'),
    group('R-8', 'move health_pay', '−830.58', '−830.58', '0.00', 'b:398'),
    matched('R-8.1', 'health_pay[2026-08]', 'a:236', 'Sep 21', 'b:398'),
    group('R-9', 'move marek_back', '+500.00', '+500.00', '0.00', 'b:399'),
    matched('R-9.1', 'marek_back[back2]', 'a:235'),
    group('R-10', 'move rent_pay', '−3200.00', '−3200.00', '0.00', 'b:400'),
    matched('R-10.1', 'rent_pay[2026-09]', 'a:234'),
    group('R-11', 'move ryczalt_pay', '−1990.00', '−1990.00', '0.00', 'b:401'),
    matched('R-11.1', 'ryczalt_pay[2026-08]', 'a:236', 'Sep 21', 'b:401'),
    group('R-12', 'move vat_pay', '−3818.00', '−3818.00', '0.00', 'b:402'),
    matched('R-12.1', 'vat_pay[2026-08]', 'a:236'),
    group('R-13', 'move wage', '+5101.67', '+5101.67', '0.00', 'b:403'),
    matched('R-13.1', 'wage[wife_acme, 2026-08]', 'a:234'),
    ('R-14', REPLAY, 'replay_september', 'line', '  law date 2026-10-19, cut ',
     [('cut', '2026-10-19', 'a:19 b:15 b:705')], 'L §8.2 law date and cut; L §4.5 the cut'),
    ('R-15', REPLAY, 'replay_september', 'absent', 'loan_interest',
     [('names', 'loan_interest', 'a:249 a:626 b:405')], 'E §6.3 zero lines (amount 0, no observation: no line)'),
    ('R-16', REPLAY, 'replay_september', 'absent', 'invest[oki',
     [('names', 'invest[oki, 2026-09]', 'a:249 b:405')], 'E §6.3 zero lines (amount 0, no observation: no line)'),
    # the run's warnings: WARN-NEGATIVE once per stretch, in every one of the three runs
    ('W-1', CHECKS, 'warning', 'line', 'warning: WARN-NEGATIVE konto is below zero from 2026-03-01',
     [('names', 'WARN-NEGATIVE', 'a:188 b:314'), ('from', '2026-03-01', 'a:188 b:314')],
     'L §8.5 WARN-NEGATIVE (the first day of each stretch), sorted by code then text; E §4.2'),
    ('W-2', CHECKS, 'warning', 'line', 'warning: WARN-NEGATIVE konto is below zero from 2026-03-17',
     [('names', 'WARN-NEGATIVE', 'a:188 b:314'), ('from', '2026-03-17', 'a:188 b:314')],
     'L §8.5 WARN-NEGATIVE (the first day of each stretch), sorted by code then text; E §4.2'),
]

MONTHS = {m: '%02d' % i for i, m in enumerate(['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct',
                                               'Nov', 'Dec'], 1)}


# -- reading -----------------------------------------------------------------------------------------------------

def out_name(check):
    return check.replace('/', '__').replace(':', '_') + '.out'


def refcore_output(check):
    with open(os.path.join(REFCORE, out_name(check)), encoding='utf-8', newline='') as f:
        return f.read()


def hh4_lines():
    lines = {}
    for tag, name in (('a', 'HH4-a.md'), ('b', 'HH4-b.md')):
        with open(os.path.join(SMOKE, name), encoding='utf-8') as f:
            lines[tag] = f.read().split('\n')
    return lines


def owner(line):
    m = re.match(r'[A-Za-z_][A-Za-z0-9_]*', line)
    return m.group(0) if m else ''


def block(out, name):
    """The lines of the block `name` in a run's output: an ask's (its unindented line and the indented ones after
    it), or every `warning:` line."""
    lines = [l for l in out.split('\n') if l]
    if name == 'warning':
        return [l for l in lines if l.startswith('warning: ')]
    res, inside = [], False
    for l in lines:
        if not l.startswith(' '):
            inside = not l.startswith('warning: ') and owner(l) == name
        if inside:
            res.append(l)
    return res


def amount(text):
    """HH4's or refcore's money figure as a Decimal: U+2212 is minus, `+`, spaces, `**` and ` PLN` dropped."""
    t = text.replace('−', '-').replace('+', '').replace('*', '').replace(' PLN', '').replace(' ', '')
    try:
        return Decimal(t)
    except InvalidOperation:
        return None


def field(line, name):
    """The figure `name` read off a refcore line, as text."""
    money = r'(-?[0-9][0-9 ]*\.[0-9]{2}) PLN'
    if name in ('value1', 'value2'):
        m = re.match(r'[^:]*: \((.*)\)$', line)
        parts = m.group(1).split(', ') if m else []
        k = 0 if name == 'value1' else 1
        return parts[k] if len(parts) == 2 else None
    if name in ('model', 'observed', 'difference', 'booked', 'stated', 'gap', '-model'):
        m = re.search(r'\b%s %s' % (name.lstrip('-'), money), line)
        return m.group(1) if m else None
    if name == 'timing':
        m = re.search(r'timing ([+-]?[0-9]+) days?', line)
        return m.group(1) if m else None
    if name == 'day':
        m = re.search(r' on ([0-9]{4}-[0-9]{2}-[0-9]{2})', line) or re.search(r' ([0-9]{4}-[0-9]{2}-[0-9]{2}) end',
                                                                                line)
        return m.group(1) if m else None
    if name in ('cut', 'from'):
        m = re.search(r'\b%s ([0-9]{4}-[0-9]{2}-[0-9]{2})' % name, line)
        return m.group(1) if m else None
    return None


def agrees(name, hh4, got, line):
    """Whether HH4's text `hh4` for figure `name` is what the refcore line says."""
    if name == 'names':
        return hh4 in line
    if got is None:
        return False
    if name == 'timing':
        m = re.search(r'[+-]?[0-9]+$', hh4)
        return bool(m) and int(m.group(0)) == int(got)
    if name == 'day':
        m = re.fullmatch(r'([A-Z][a-z]{2}) ([0-9]{1,2})', hh4)
        want = '%s-%02d' % (MONTHS[m.group(1)], int(m.group(2))) if m else hh4
        return got.endswith(want)
    if name in ('cut', 'from'):
        return got == hh4
    a, b = amount(hh4), amount(got)
    if a is None or b is None:
        return False
    return a == (-b if name == '-model' else b)


def on_line(text, line):
    """HH4's text stands on its line as written, not inside a longer number."""
    return re.search(r'(?<![0-9.])%s(?![0-9])' % re.escape(text), line) is not None


def cites(spec):
    out = []
    for c in spec.split():
        tag, n = c.split(':')
        out.append((tag, int(n)))
    return out


# -- the vectors -------------------------------------------------------------------------------------------------

def build(outs, hh4, spec=None):
    """(rows, problems): vectors.tsv's rows from SPEC (or `spec`) and refcore's outputs, and every disagreement."""
    rows, problems = [], []
    for vid, checks, name, kind, start, figs, rule in (SPEC if spec is None else spec):
        texts = set()
        for c in checks:
            lines = block(outs[c], name)
            if kind == 'absent':
                hit = [l for l in lines if start in l]
                if hit:
                    problems.append('%s %s: refcore prints %r, HH4 says it prints nothing' % (vid, c, hit[0]))
                texts.add(start)
                continue
            hit = [l for l in lines if l.startswith(start)]
            if len(hit) != 1:
                problems.append('%s %s: %d lines of %s start %r, not 1' % (vid, c, len(hit), name, start))
                continue
            texts.add(hit[0])
        if len(texts) != 1:
            problems.append('%s: the checks print it differently: %r' % (vid, sorted(texts)))
            continue
        text = texts.pop()
        for f, value, where in figs:
            for tag, n in cites(where):
                lines = hh4[tag]
                if not (1 <= n <= len(lines)) or not on_line(value, lines[n - 1]):
                    problems.append('%s: %s %r is not on HH4-%s.md:%d' % (vid, f, value, tag, n))
            if kind == 'line' and not agrees(f, value, field(text, f), text):
                problems.append('%s: %s: HH4 %s (%s), refcore %r in %r' % (vid, f, value, where, field(text, f), text))
        rows.append({'id': vid, 'checks': ' '.join(checks), 'block': name, 'kind': kind, 'text': text,
                     'figures': '; '.join('%s = %s @ %s' % (f, v, w) for f, v, w in figs), 'rule': rule})
    # the order: each (check, block)'s lines appear in the table's order
    for c in CHECKS:
        for name in sorted({r['block'] for r in rows}):
            want = [r['text'] for r in rows if r['kind'] == 'line' and r['block'] == name and c in r['checks'].split()]
            got = block(outs[c], name)
            pos = -1
            for t in want:
                nxt = got.index(t, pos + 1) if t in got[pos + 1:] else -1
                if nxt < 0:
                    problems.append('%s: %s: %r is not after the line before it' % (c, name, t))
                    break
                pos = nxt
    return rows, problems


def tsv(rows):
    out = ['\t'.join(COLS)]
    for r in rows:
        if any('\t' in r[k] or '\n' in r[k] for k in COLS):
            raise SystemExit('a tab or a newline in vector %s' % r['id'])
        out.append('\t'.join(r[k] for k in COLS))
    return '\n'.join(out) + '\n'


def store():
    """Copy refcore's runs of the three checks from run.py's cache into refcore/."""
    spec = importlib.util.spec_from_file_location('run', os.path.join(ROOT, 'engine/tests/run.py'))
    run = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(run)
    checks = {c['id']: c for c in run.load_checks()}
    h = run.refcore_hash()
    os.makedirs(REFCORE, exist_ok=True)
    for check in CHECKS:
        for v in ('pypy', 'pypy+fast'):
            k = run.cache_key(checks[check], v, h)
            p = os.path.join(ROOT, 'engine/tests/cache', k[:2], k + '.json')
            if os.path.exists(p):
                e = json.load(open(p, encoding='utf-8'))
                if not e['over'] and e['rc'] == 0:
                    break
        else:
            raise SystemExit('refcore\'s run of %s is not cached: run.py %s first' % (check, check))
        with open(os.path.join(REFCORE, out_name(check)), 'w', encoding='utf-8', newline='') as f:
            f.write(e['out'])
        print('%s: %s, rc 0, %.1f s' % (check, 'refcore' if v == 'pypy' else 'refcore_fast', e['seconds']))
    return 0


def self_test():
    """Each kind of disagreement, planted in a copy of SPEC, is caught; SPEC itself passes."""
    outs = {c: refcore_output(c) for c in CHECKS}
    hh4 = hh4_lines()
    by = {v[0]: v for v in SPEC}

    def with_(vid, **kw):
        v = list(by[vid])
        for k, x in kw.items():
            v[{'kind': 3, 'start': 4, 'figs': 5}[k]] = x
        return [tuple(v) if s[0] == vid else s for s in SPEC]

    plants = [
        ('a figure refcore prints otherwise', with_('K-1', figs=[('value1', '6064.18', 'a:638')]), 'refcore'),
        ('a figure not on its HH4 line', with_('R-3', figs=[('model', '+20910.00', 'b:394')]), 'is not on'),
        ('a missing line read as model, not minus model', with_('R-6.1', figs=[('-model', '−300.00', 'b:396')]),
         'refcore'),
        ('a line HH4 says is absent, printed', with_('R-16', start='invest[ike'), 'prints nothing'),
        ('a line refcore doesn\'t print', with_('R-0', start='replay_september: residuals konto 2026-10'), 'not 1'),
        ('two lines out of order', [s for s in SPEC if s[0] != 'R-1'] + [by['R-1']], 'not after'),
    ]
    bad = 0
    _, problems = build(outs, hh4)
    if problems:
        print('FAIL SPEC itself: %s' % problems[0])
        bad += 1
    for what, spec, word in plants:
        _, problems = build(outs, hh4, spec)
        caught = any(word in p for p in problems)
        print('%s  %s%s' % ('pass' if caught else 'FAIL', what, '' if caught else ': %r' % problems))
        bad += not caught
    print('hh4 self-test: %d plants: %d failed' % (len(plants), bad))
    return 1 if bad else 0


def main(argv):
    if argv == ['--store']:
        return store()
    if argv == ['--self-test']:
        return self_test()
    if argv not in ([], ['--write']):
        print(__doc__)
        return 64
    outs = {c: refcore_output(c) for c in CHECKS}
    rows, problems = build(outs, hh4_lines())
    text = tsv(rows)
    if not problems and argv == ['--write']:
        with open(TSV, 'w', encoding='utf-8', newline='') as f:
            f.write(text)
    elif not problems:
        with open(TSV, encoding='utf-8', newline='') as f:
            if f.read() != text:
                problems.append('vectors.tsv differs from what SPEC and refcore\'s outputs give (--write)')
    for p in problems:
        print('FAIL ' + p)
    n_fig = sum(len(v[5]) for v in SPEC)
    print('hh4: %d vectors, %d figures, %d lines of refcore over %d checks: %d failed'
          % (len(rows), n_fig, sum(len(r['checks'].split()) for r in rows), len(CHECKS), len(problems)))
    return 1 if problems else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
