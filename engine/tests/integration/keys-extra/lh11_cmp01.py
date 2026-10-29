#!/usr/bin/env python3
"""Independent keys for the 5 old-model rows the tally 2.0 freeze re-keyed from interpreter prints (session 4.5c,
gap item 6; `tmp/design2/freeze/KEYS.md` rows 1-5). Standard library only; reads the published case files
read-only; never runs the interpreter. Every figure is computed here with Decimal, then asserted against the row's
expected text in `engine/spec/expected/`.

    python3 -I -B tmp/tests/keys-extra/lh11_cmp01.py LH-11     # rows 1-3: a60_a144, a89_n, a133_x
    python3 -I -B tmp/tests/keys-extra/lh11_cmp01.py CMP-01    # rows 4-5: q_c6_k1..k3, q_c6_tax (the lifted rows)

LH-11: the case's own inputs (`expected/LH-11.md`, Calculations 1, 3, 4). Under `lh11_step = printed` every printed
intermediate is carried at its printed places and rounded by `lh11_mode` (half-up or down, i.e. truncation).
CMP-01: the rows list facts, not figures. MIGRATION.md's rule (row 20) lifts every inline figure but 0, 1 and -1
into an input row named from its use; so the four inline figures of the defs the asks reach in the 1.0.1 text
must be the four `lifted_figures` rows of the 2.0 text, with the same values, and the expected rows must name them.
"""
import re
import sys
from decimal import Decimal as D, ROUND_DOWN, ROUND_HALF_UP, getcontext

getcontext().prec = 40
ROOT = '/Users/jswob/projects/trop/'
MODES = {'half-up': ROUND_HALF_UP, 'down': ROUND_DOWN}
fails = []


def check(cond, what):
    print(('ok   ' if cond else 'FAIL ') + what)
    if not cond:
        fails.append(what)


def q(x, places, mode):
    return x.quantize(D(1).scaleb(-places), rounding=MODES[mode])


def printed(x):
    """A figure as the expected rows print it: trailing zeros dropped."""
    s = format(x.normalize(), 'f')
    return s


def ann(n, i):
    return (1 - 1 / (1 + i) ** n) / i


def expected(name, row):
    t = open(ROOT + 'engine/spec/expected/%s.md' % name, encoding='utf-8').read()
    for l in t.splitlines():
        if l.startswith('| `%s`' % row):
            return l
    raise SystemExit('no row %s in %s' % (row, name))


def lh11():
    # row 1, a60_a144: a_{144|0.0075} printed at 4 places
    a144 = ann(144, D('0.0075'))
    got = {m: printed(q(a144, 4, m)) for m in MODES}
    row = expected('LH-11', 'a60_a144')
    check(got == {'half-up': '87.8711', 'down': '87.871'} and '"87.8711" under `half-up`; "87.871" under `down`' in row,
          'LH-11 a60_a144: exact %s; half-up %s, down %s; the expected row says the same' % (
              str(a144)[:12], got['half-up'], got['down']))

    # row 2, a89_n: the months n under `printed`, each mode
    i = D('0.005')
    pay = 200000 * i / (1 - 1 / (1 + i) ** 360)            # 1199.1010...
    res = {}
    for m in MODES:
        R = q(pay, 2, m)                                    # a89_pay, printed 1199.10 in both modes
        j = q((1 + i) ** 12 - 1, 4, m)                      # a89_j, the carried annual rate: 0.0617 / 0.0616
        pv = q(sum(10000 / (1 + j) ** k for k in range(1, 6)), 2, m)   # a89_pv
        L = 200000 - pv                                     # a89_reduced
        n = -(1 - L * i / R).ln() / (1 + i).ln()
        res[m] = (str(j), printed(q(n, 2, m)))
    row = expected('LH-11', 'a89_n')
    check(res['half-up'][1] == '215.78' and res['down'] == ('0.0616', '215.75') and
          '"215.78"; "215.75" under `printed` with `down`' in row,
          'LH-11 a89_n (printed): half-up j %s n %s; down j %s n %s; the expected row says the same' % (
              res['half-up'] + res['down']))

    # row 3, a133_x: 1 - 1.0045^-n = B * 0.0045 / 500, under `printed`, each mode; and `exact`
    i = D('0.005')
    res = {}
    for m in MODES:
        R = q(85000 / ann(240, i), 2, m)                    # a133_pay: 608.97 / 608.96
        a192 = q(ann(192, i), 4, m)                         # a133_a192: 123.2380 in both modes
        B = q(R * a192, 2, m)                               # a133_bal
        res[m] = (str(R), printed(q(B * D('0.0045') / 500, 5, m)))
    exact = (85000 / ann(240, i)) * ann(192, i) * D('0.0045') / 500
    ex = {m: printed(q(exact, 5, m)) for m in MODES}
    row = expected('LH-11', 'a133_x')
    check(res['half-up'] == ('608.97', '0.67543') and res['down'] == ('608.96', '0.67542') and
          ex == {'half-up': '0.67543', 'down': '0.67543'} and '"0.67543"; "0.67542" under `printed` with `down`' in row,
          'LH-11 a133_x: printed half-up (pay %s) %s, printed down (pay %s) %s; exact %s, both modes %s; '
          'the expected row says the same' % (res['half-up'] + res['down'] + (str(exact)[:11], ex['down'])))


def cmp01():
    old = open(ROOT + 'tmp/design/models/CMP-01.tally', encoding='utf-8').read().splitlines()
    new = open(ROOT + 'engine/spec/models/CMP-01.tally', encoding='utf-8').read()
    mig = open(ROOT + 'engine/spec/MIGRATION.md', encoding='utf-8').read()
    check(old[1].strip() == 'tally 1', 'CMP-01 1.0.1 text is tally 1 (tmp/design/models/CMP-01.tally)')
    check(re.search(r'any inline figure but 0, 1, −1 .*lifts each distinct figure into one input row', mig) is not None,
          'MIGRATION.md: every inline figure but 0, 1, -1 is lifted into an input row')
    # (1.0.1 line, the def, the inline figures there, the lifted row(s) its figure becomes)
    want = [(172, 'qf2026', ['2'], ['extra_halves_2']),
            (176, 'bgb_2026', ['18'], ['first_rank_18']),
            (177, 'bgb_2026', ['18', '21'], ['first_rank_18', 'first_rank_21']),
            (263, 'dt2026', ['2'], ['zahlbetrag_2'])]
    for ln, ver, figs, lifted in want:
        l = old[ln - 1]
        nums = [x for x in re.findall(r'(?<![\w.])\d+(?:\.\d+)?(?![\w.])', l.split('=', 1)[1]) if x not in ('0', '1')]
        nums = [x for x in nums if x != '0.01']             # the rounding step, a unit not a figure (minor(EUR))
        check(ver in l and nums == figs, 'CMP-01 1.0.1 line %d (%s): inline figures %s' % (ln, ver, nums))
        for name, fig in zip(lifted, figs if len(figs) == len(lifted) else figs[-len(lifted):]):
            m = re.search(r'(?m)^\s*%s = (\S+) "inline in tally 1\.0\.1: [^"]*line %d' % (name, ln), new)
            check(m is not None and m.group(1) == fig and ('input %s: int' % name) in new,
                  'CMP-01 2.0: input %s = %s, lifted from 1.0.1 line %d' % (name, m.group(1) if m else '?', ln))
    lifted_rows = set(re.findall(r'(?m)^\s*(\w+) = \S+ "inline in tally 1\.0\.1', new))
    k = expected('CMP-01', 'q_c6_k1')
    t = expected('CMP-01', 'q_c6_tax')
    ks = set(re.findall(r'lifted_figures:(\w+)', k))
    ts = set(re.findall(r'lifted_figures:(\w+)', t))
    check(ks == {'first_rank_18', 'first_rank_21', 'zahlbetrag_2'} and ks <= lifted_rows,
          'CMP-01 q_c6_k1..k3 names the lifted rows of bgb_2026 and dt2026: %s' % sorted(ks))
    check(ts == {'extra_halves_2'} and ts <= lifted_rows,
          'CMP-01 q_c6_tax names the lifted row of qf2026: %s' % sorted(ts))


if __name__ == '__main__':
    which = sys.argv[1:] or ['LH-11', 'CMP-01']
    if 'LH-11' in which:
        lh11()
    if 'CMP-01' in which:
        cmp01()
    print('%s: %d failed' % (' '.join(which), len(fails)))
    sys.exit(1 if fails else 0)
