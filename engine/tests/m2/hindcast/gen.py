"""The hindcast fixtures H0-H5, H8, H9 in tally 2 (DONE gate 4; issue #167; engine/spec/verification/hindcast.md;
T's plan tmp/m2/t-plan.md §2.2). Standard library only. Synthetic: no bank record was ever used.

The generator holds the true model M* (the constants below) and writes, from them alone:

- `model.tally`: the modeller's M, one model for every fixture: 2 adults (A, B), a joint account `konto`, a
  brokerage account `broker`, rent, a job each, 6 spending categories, a subscription, and one law both sides share:
  a flat payroll tax rounded to whole PLN, half-up. Pay reads the payslip's gross where one is known (E §6.2: the
  law for the past reads observations) and the contract's otherwise. Every row the bank shows is linked to the
  instance it fulfils by one written rule: a row fulfils the move of its counterparty and month (`fulfils:`), and a
  row no move of M explains fulfils nothing.
- `ledger/<yyyy-mm>.tally`: the base realisation, 60 parts (S = 2021-01, month 0; K = 2025-01, month 48; E =
  2026-01, month 60): each month's bank statement (`covers konto`, `stated[konto]` at the month's end, the sum of the
  opening and every row so far) and its payslips.
- `<h>/<yyyy-mm>.tally` (`h1`, `h2`, `h4`, `h5`, `h8`, `h9`): a fixture's parts for the months its planted cause
  changes, each replacing the base part of that month in the fixture's run; `h3/law.tally`, H3's change to M (a
  version only, the ledger is the base's).
- `key.tsv`: the answer key, one row per planted line: fixture, ask, line kind, move, months from and to, field,
  value. `judge.py` holds refcore's stored bytes to it.

Planted (hindcast.md §3; a month is its index from S):
  H1  A's gross +5 % from month 30 (payslips and bank)       H2  a car repair in month 41, fulfilling nothing
  H3  M's tax rounds down (truncates), M* rounds half-up     H4  rent paid on the 3rd instead of the 1st from month 20
  H5  the subscription cancelled after month 45               H8  B's gross +4 % from month 52 (after K)
  H9  H1 and H4 at once                                       H0  nothing (the base)
H6 and H7 (frequency and severity of spending) are random and banded: M7's.

    python3 -I -B engine/tests/m2/hindcast/gen.py           -> writes the texts and key.tsv
    python3 -I -B engine/tests/m2/hindcast/gen.py --check   -> exit 1 if any file isn't what it would write
"""
import os
import sys
from decimal import ROUND_DOWN, ROUND_HALF_UP, Decimal as D

HERE = os.path.dirname(os.path.abspath(__file__))
MONTHS = [(2021 + i // 12, 1 + i % 12) for i in range(60)]
K_CUT = '2025-01-02'          # month 48: the statements to 2024-12 are known, no payslip of 2025-01
K29_CUT = '2023-07-02'        # month 30's eve, for H1's backtest
OPENING = D('5000.00')
GROSS = {'a': D('8770.00'), 'b': D('6450.00')}
TAX_RATE = D('0.17')
PAY_DOM = {'a': 10, 'b': 25}
RENT, RENT_DOM = D('2900.00'), 1
INVEST, INVEST_DOM = D('1500.00'), 11
SUB, SUB_DOM = D('49.99'), 15
SPEND = [('food', D('1240.00'), 6), ('transport', D('310.00'), 8), ('utilities', D('455.20'), 12),
         ('health', D('120.00'), 20), ('clothes', D('260.00'), 22), ('leisure', D('380.00'), 26)]
H1_FROM, H1_RAISE = 30, D('1.05')
H2_MONTH, H2_DAY, H2_AMOUNT = 41, 17, D('2340.00')
H4_FROM, H4_DOM = 20, 3
H5_LAST = 45
H8_FROM, H8_RAISE = 52, D('1.04')
FIXTURES = {'h0': set(), 'h1': {'h1'}, 'h2': {'h2'}, 'h3': set(), 'h4': {'h4'}, 'h5': {'h5'}, 'h8': {'h8'},
            'h9': {'h1', 'h4'}}


def ym(i):
    y, m = MONTHS[i]
    return '%04d-%02d' % (y, m)


def day(i, dom):
    return '%s-%02d' % (ym(i), dom)


def money(x):
    """The engine's money form: groups of three by a space, two decimals."""
    s = '%.2f' % abs(x)
    whole, frac = s.split('.')
    groups = []
    while len(whole) > 3:
        groups.insert(0, whole[-3:])
        whole = whole[:-3]
    groups.insert(0, whole)
    return ('-' if x < 0 else '') + ' '.join(groups) + '.' + frac + ' PLN'


def plain(x):
    return '%.2f PLN' % x


def tax(gross, mode):
    return (gross * TAX_RATE).quantize(D('1'), rounding=mode)


def gross(who, i, causes):
    g = GROSS[who]
    if who == 'a' and 'h1' in causes and i >= H1_FROM:
        g = (g * H1_RAISE).quantize(D('0.01'))
    if who == 'b' and 'h8' in causes and i >= H8_FROM:
        g = (g * H8_RAISE).quantize(D('0.01'))
    return g


def net(who, i, causes, mode=ROUND_HALF_UP):
    g = gross(who, i, causes)
    return g - tax(g, mode)


def rows(i, causes):
    """The bank's rows of month i under M*: (day, id letter, from, to, amount, fulfils or None, note)."""
    out = [(H4_DOM if ('h4' in causes and i >= H4_FROM) else RENT_DOM, 'konto', 'landlord', RENT, 'rent[%s]' % ym(i),
            'czynsz'),
           (PAY_DOM['a'], 'employer_a', 'konto', net('a', i, causes), 'pay_a[%s]' % ym(i), 'wynagrodzenie A'),
           (PAY_DOM['b'], 'employer_b', 'konto', net('b', i, causes), 'pay_b[%s]' % ym(i), 'wynagrodzenie B'),
           (INVEST_DOM, 'konto', 'broker', INVEST, 'invest[%s]' % ym(i), 'przelew do biura maklerskiego')]
    if not ('h5' in causes and i > H5_LAST):
        out.append((SUB_DOM, 'konto', 'streaming', SUB, 'sub[%s]' % ym(i), 'abonament'))
    for (c, amount, dom) in SPEND:
        out.append((dom, 'konto', 'shops', amount, 'spend[%s, %s]' % (c, ym(i)), c))
    if 'h2' in causes and i == H2_MONTH:
        out.append((H2_DAY, 'konto', 'garage', H2_AMOUNT, None, 'naprawa samochodu'))
    return sorted(out, key=lambda r: (r[0], r[4] or '~'))


def flow(r):
    return r[3] if r[2] == 'konto' else -r[3]


def month_text(i, causes, label):
    """Month i's part: the statement and the payslips, as M* realises them under `causes`."""
    balance = OPENING + sum(flow(r) for k in range(i) for r in rows(k, causes))
    t = ['part hc_%s' % ym(i).replace('-', '_'), '  tally 2',
         '  note "%s: month %d (%s) of the hindcast\'s 60, written by gen.py from M*; synthetic."' % (label, i, ym(i)),
         '', 'version st_%s' % ym(i).replace('-', '_'), '  cite statements', '  governs %s' % ym(i),
         '  known %s' % known_after(i), '  covers konto', '  pad konto from unexplained']
    seen = {}
    for r in rows(i, causes):
        letter = seen.setdefault(r[0], 0)
        seen[r[0]] += 1
        rid = 'r%s%s' % (day(i, r[0]).replace('-', ''), 'abcdefghij'[letter])
        link = ', fulfils: %s' % r[4] if r[4] else ''
        kind = 'wages' if (r[4] or '').startswith('pay_') else 'transfers'
        t.append('  bank[%s] = (on: %s, kind: %s, leg cash from %s to %s amount %s%s) "%s"' % (
            rid, day(i, r[0]), kind, r[1], r[2], plain(r[3]), link, r[5]))
        balance += flow(r)
    y, m = MONTHS[i]
    t.append('  stated[konto] = %s at %s end "saldo"' % (plain(balance), last_day(y, m)))
    t += ['', 'version slips_%s' % ym(i).replace('-', '_'), '  cite slips', '  governs %s' % ym(i),
          '  known %s' % day(i, PAY_DOM['a']),
          '  slip_a[%s] = %s "pasek A"' % (ym(i), plain(gross('a', i, causes))),
          '  slip_b[%s] = %s "pasek B"' % (ym(i), plain(gross('b', i, causes)))]
    return '\n'.join(t) + '\n'


def last_day(y, m):
    nxt = (y + (m == 12), 1 if m == 12 else m + 1)
    import datetime
    return (datetime.date(nxt[0], nxt[1], 1) - datetime.timedelta(days=1)).isoformat()


def known_after(i):
    y, m = MONTHS[i]
    nxt = (y + (m == 12), 1 if m == 12 else m + 1)
    return '%04d-%02d-02' % nxt


MODEL = '''model hindcast
  tally 2
  on 2026-01-05
  start 2021-01-01
  end 2025-12-31
  note "T, M2 (issue #167), DONE gate 4: the hindcast fixtures' model M, the modeller's, one for every fixture (engine/spec/verification/hindcast.md, restated in tally 2's residual ledger, E §6.3). Written by gen.py with the ledger it is run over: synthetic, no bank record was ever used. S = 2021-01 (month 0), K = 2025-01 (month 48), E = 2026-01 (month 60); on is past the last statement's known day, so every row is known."

index month = month 2021-01..2025-12
index category = [food, transport, utilities, health, clothes, leisure]

source contracts "umowy"
source statements "wyciągi bankowe"
source slips "paski płacowe"
source statute "ustawa (syntetyczna)"
unit PLN: currency minor 0.01

kind wages: event certain
  leg cash: PLN

input bank: observed
input gross_plan_a: money PLN
input gross_plan_b: money PLN
input slip_a[m in month]: money PLN
input slip_b[m in month]: money PLN
input tax_rate: num
input law_truncates: bool
input pay_dom_a: int
input pay_dom_b: int
input rent_amt: money PLN
input rent_dom: int
input invest_amt: money PLN
input invest_dom: int
input sub_amt: money PLN
input sub_dom: int
input spend_amt[c in category]: money PLN
input spend_dom[c in category]: int

place konto: accounts PLN lives 2020-01-01..
place broker: accounts PLN lives 2020-01-01..
place employer_a: outside lives 1900-01-01..
place employer_b: outside lives 1900-01-01..
place landlord: outside lives 1900-01-01..
place streaming: outside lives 1900-01-01..
place shops: outside lives 1900-01-01..
place garage: outside lives 1900-01-01..
place unexplained: outside lives 1900-01-01..

def gross_a[m in month]: money PLN at start_of(m)
  rule contract = gross_plan_a
  rule slip beats contract v plan = slip_a[m] when not missing(slip_a[m])
def gross_b[m in month]: money PLN at start_of(m)
  rule contract = gross_plan_b
  rule slip beats contract v plan = slip_b[m] when not missing(slip_b[m])
def tax_a[m in month]: money PLN at start_of(m)
  rule half_up = round(gross_a[m] * tax_rate, 1, half-up)
  rule truncated beats half_up = round(gross_a[m] * tax_rate, 1, down) when law_truncates
def tax_b[m in month]: money PLN at start_of(m)
  rule half_up = round(gross_b[m] * tax_rate, 1, half-up)
  rule truncated beats half_up = round(gross_b[m] * tax_rate, 1, down) when law_truncates
def net_a[m in month]: money PLN = gross_a[m] - tax_a[m] at start_of(m)
def net_b[m in month]: money PLN = gross_b[m] - tax_b[m] at start_of(m)

move pay_a[m in month]: wages
  on day_of(m, pay_dom_a)
  at start_of(m)
  leg cash from employer_a to konto amount net_a[m]
move pay_b[m in month]: wages
  on day_of(m, pay_dom_b)
  at start_of(m)
  leg cash from employer_b to konto amount net_b[m]
move rent[m in month]: transfers
  on day_of(m, rent_dom)
  at start_of(m)
  leg cash from konto to landlord amount rent_amt
move invest[m in month]: transfers
  on day_of(m, invest_dom)
  at start_of(m)
  leg cash from konto to broker amount invest_amt
move sub[m in month]: transfers
  on day_of(m, sub_dom)
  at start_of(m)
  leg cash from konto to streaming amount sub_amt
move spend[c in category, m in month]: transfers
  on day_of(m, spend_dom[c])
  at start_of(m)
  leg cash from konto to shops amount spend_amt[c]

version plan
  cite contracts
  governs always
  known 2021-01-01
  gross_plan_a = %(ga)s "umowa A"
  gross_plan_b = %(gb)s "umowa B"
  pay_dom_a = %(pda)d "umowa A"
  pay_dom_b = %(pdb)d "umowa B"
  rent_amt = %(rent)s "umowa najmu"
  rent_dom = %(rentd)d "umowa najmu"
  invest_amt = %(inv)s "zlecenie stałe"
  invest_dom = %(invd)d "zlecenie stałe"
  sub_amt = %(sub)s "abonament"
  sub_dom = %(subd)d "abonament"
%(spend)s
  opening[konto] = %(open)s at 2021-01-01 start "wyciąg"
  opening[broker] = 0.00 PLN at 2021-01-01 start "wyciąg"

version tax_law
  cite statute
  governs always
  known 2021-01-01
  tax_rate = %(rate)s "art. 1: podatek płacowy"
  law_truncates = false "art. 2: zaokrąglenie do pełnych złotych, połowa w górę"

ask replay: residuals konto 2021-01..2025-12
ask back: residuals konto 2021-01..2025-12 as known %(k)s
ask back29: residuals konto 2021-01..2025-12 as known %(k29)s
ask mask_bal: bal(konto, 2025-12-31) as known %(k)s
ask mask_net: net_b[2025-09] as known %(k)s
''' % {'ga': plain(GROSS['a']), 'gb': plain(GROSS['b']), 'pda': PAY_DOM['a'], 'pdb': PAY_DOM['b'],
       'rent': plain(RENT), 'rentd': RENT_DOM, 'inv': plain(INVEST), 'invd': INVEST_DOM, 'sub': plain(SUB),
       'subd': SUB_DOM, 'open': plain(OPENING), 'rate': str(TAX_RATE), 'k': K_CUT, 'k29': K29_CUT,
       'spend': '\n'.join('  spend_amt[%s] = %s "budżet"\n  spend_dom[%s] = %d "budżet"' % (c, plain(a), c, d)
                          for (c, a, d) in SPEND)}

H3_LAW = '''part hc_h3_law
  tally 2
  note "H3 (hindcast.md §3), written by gen.py: M's law rounds the tax down (truncates) where M* rounds half-up, a law bug in M only; the ledger is the base's."

version tax_law_h3
  cite statute
  governs always
  known 2021-01-02
  replace law_truncates = true "the modeller's reading of art. 2"
'''


def changed(fixture):
    """The months whose part the fixture changes."""
    causes = FIXTURES[fixture]
    return [i for i in range(60) if month_text(i, causes, '') != month_text(i, set(), '')]


def key_rows():
    """The answer key (fixture, ask, kind, move, from, to, field, value), from M* and M's difference alone."""
    k = []

    def add(f, ask, kind, move, a, b, field, value):
        k.append((f, ask, kind, move, ym(a), ym(b), field, value))

    def raise_a(i):
        return net('a', i, {'h1'}) - net('a', i, set())
    for f in ('h1', 'h9'):
        add(f, 'back', 'matched', 'pay_a', 48, 59, 'difference', money(raise_a(59)))
        add(f, 'back29', 'matched', 'pay_a', H1_FROM, 59, 'difference', money(raise_a(59)))
    add('h2', 'replay', 'unexplained', '-', H2_MONTH, H2_MONTH, 'observed', money(-H2_AMOUNT))
    add('h2', 'back', 'unexplained', '-', H2_MONTH, H2_MONTH, 'observed', money(-H2_AMOUNT))
    for ask in ('replay', 'back'):
        for who in ('a', 'b'):
            d = net(who, 0, set()) - net(who, 0, set(), ROUND_DOWN)
            add('h3', ask, 'matched', 'pay_' + who, 0, 59, 'difference', money(d))
    for f in ('h4', 'h9'):
        for ask in ('replay', 'back') + (('back29',) if f == 'h9' else ()):
            add(f, ask, 'matched', 'rent', H4_FROM, 59, 'timing', '+%d days' % (H4_DOM - RENT_DOM))
    for ask in ('replay', 'back'):
        add('h5', ask, 'missing', 'sub', H5_LAST + 1, 59, 'model', money(-SUB))
    add('h8', 'back', 'matched', 'pay_b', H8_FROM, 59, 'difference',
        money(net('b', 59, {'h8'}) - net('b', 59, set())))
    return k


def outputs():
    out = {'model.tally': MODEL, 'h3/law.tally': H3_LAW}
    for i in range(60):
        out['ledger/%s.tally' % ym(i)] = month_text(i, set(), 'the base (H0)')
    for f in ('h1', 'h2', 'h4', 'h5', 'h8', 'h9'):
        for i in changed(f):
            out['%s/%s.tally' % (f, ym(i))] = month_text(i, FIXTURES[f], f.upper())
    lines = ['fixture\task\tkind\tmove\tfrom\tto\tfield\tvalue'] + ['\t'.join(r) for r in key_rows()]
    out['key.tsv'] = '\n'.join(lines) + '\n'
    return out


def files_of(fixture):
    """A fixture's run: the model, H3's law, and each month's part, the fixture's where it changes it."""
    own = set(changed(fixture)) if fixture in ('h1', 'h2', 'h4', 'h5', 'h8', 'h9') else set()
    parts = ['%s/%s.tally' % (fixture if i in own else 'ledger', ym(i)) for i in range(60)]
    return ['model.tally'] + (['h3/law.tally'] if fixture == 'h3' else []) + parts


def main(argv):
    out = outputs()
    if '--check' in argv:
        bad = [p for p, text in out.items() if not os.path.exists(os.path.join(HERE, p)) or
               open(os.path.join(HERE, p), encoding='utf-8').read() != text]
        for p in bad:
            print('gen.py: %s is not what gen.py writes' % p)
        print('gen.py: %d files%s' % (len(out), ', all as written' if not bad else ''))
        return 1 if bad else 0
    for p, text in out.items():
        os.makedirs(os.path.dirname(os.path.join(HERE, p)), exist_ok=True)
        with open(os.path.join(HERE, p), 'w', encoding='utf-8') as f:
            f.write(text)
    print('gen.py: wrote %d files' % len(out))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
