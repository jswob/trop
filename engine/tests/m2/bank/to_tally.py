"""The prototype's bank history (`ledger.json`) as tally 2 records: one part per month under `records/`.

Issue #114 (M2's chunk C4, `MILESTONE-M2.md`). Standard library only. `python3 -I -B to_tally.py` writes
`records/2024-01.tally` .. `records/2026-10.tally`; `--check` writes nothing and fails when a tracked part
differs from what the ledger gives (`run.py --self-test` step 8p).

Each part is a version `governs` its month (October 2026 to the ledger's last day, 2026-10-16), `known` two
days after (October's on the prototype's today, 2026-10-17, the model's `on`), `covers konto`, `anchor konto`; a `bank` row per ledger row in household X's `records/mbank`
form, its note the ledger's row number; and `stated[konto]` at the month's end, the opening plus every row
so far (a figure of the fixture, the only one this script adds). The three `otwarcie` rows are the
reality's openings (`reality.tally`), not bank rows.

The link rule (P1, the coordinator's pick on Q5; the prototype's own matcher, P-02, by category and month):
a row of category `wynagrodzenie` from `pracodawca` fulfils `my_pay[m - 1]`, from `pracodawca_zony`
`wife_pay[m - 1]` (the pay a month earns arrives in the next, as the model pays it); a row of category
`czynsz` fulfils `rent_pay[m]`; m is the row's month. No other row is linked: it is an observed movement
the model doesn't plan, with its category (the category `zus` is written `skladki_zus`: a key
can't share the place's name).
"""
import json
import os
import sys
from decimal import Decimal

HERE = os.path.dirname(os.path.abspath(__file__))
LAST = '2026-10-16'
TODAY = '2026-10-17'
CAT = {'zus': 'skladki_zus'}  # a key can't share the place zus's name (LOAD-KEY)


def money(x):
    return Decimal(str(x)).quantize(Decimal('0.01'))


def month_before(m):
    y, n = int(m[:4]), int(m[5:7])
    return '%04d-%02d' % ((y, n - 1) if n > 1 else (y - 1, 12))


def last_day(m):
    y, n = int(m[:4]), int(m[5:7])
    days = [31, 29 if y % 4 == 0 and (y % 100 or y % 400 == 0) else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    return '%s-%02d' % (m, days[n - 1])


def plus_two(d):
    y, n, k = int(d[:4]), int(d[5:7]), int(d[8:])
    k += 2
    end = int(last_day(d[:7])[8:])
    if k > end:
        k -= end
        y, n = (y, n + 1) if n < 12 else (y + 1, 1)
    return '%04d-%02d-%02d' % (y, n, k)


def link(r):
    m = r['date'][:7]
    if r['category'] == 'wynagrodzenie' and r['from'] == 'pracodawca':
        return 'my_pay[%s]' % month_before(m)
    if r['category'] == 'wynagrodzenie' and r['from'] == 'pracodawca_zony':
        return 'wife_pay[%s]' % month_before(m)
    if r['category'] == 'czynsz':
        return 'rent_pay[%s]' % m
    return None


def parts():
    rows = json.load(open(os.path.join(HERE, 'ledger.json'), encoding='utf-8'))
    konto = sum(money(r['amount']) for r in rows if r['category'] == 'otwarcie' and r['to'] == 'konto')
    months = {}
    for n, r in enumerate(rows):
        if r['category'] != 'otwarcie':
            months.setdefault(r['date'][:7], []).append((n, r))
    out = {}
    for m in sorted(months):
        end = min(last_day(m), LAST)
        name = 'bank_' + m.replace('-', '_')
        span = m if end == last_day(m) else '%s-01..%s' % (m, end)
        lines = [
            'part ' + name,
            '  tally 2',
            '  note "The prototype\'s bank history for %s, synthetic (ledger.json, seed 20261017): no bank record was '
            'ever used. Written by to_tally.py; links by P1\'s rule: wynagrodzenie from pracodawca fulfils '
            'my_pay of the month before, from pracodawca_zony wife_pay of the month before, czynsz rent_pay of '
            'the month; no other row is linked."' % m,
            '',
            'version ' + name,
            '  cite ledger',
            '  governs ' + span,
            '  known ' + (plus_two(end) if end == last_day(m) else TODAY),
            '  covers konto',
            '  anchor konto',
        ]
        seq = {}
        for n, r in months[m]:
            d = r['date']
            k = seq.get(d, 0)
            seq[d] = k + 1
            rid = 'r%s%s' % (d.replace('-', ''), chr(ord('a') + k) if k < 26 else 'z%d' % k)
            amt = money(r['amount'])
            if r['to'] == 'konto':
                konto += amt
            if r['from'] == 'konto':
                konto -= amt
            f = link(r)
            tail = ', fulfils: %s' % f if f else ''
            lines.append('  bank[%s] = (on: %s, kind: bank_line, leg cash from %s to %s amount %s PLN, cat: %s%s) '
                         '"ledger.json row %d"' % (rid, d, r['from'], r['to'], amt, CAT.get(r['category'], r['category']), tail, n))
        lines.append('  stated[konto] = %s at %s end "the opening plus every row to the day"' % (konto, end))
        out['%s.tally' % m] = '\n'.join(lines) + '\n'
    return out


def main():
    check = '--check' in sys.argv[1:]
    want = parts()
    folder = os.path.join(HERE, 'records')
    have = sorted(f for f in os.listdir(folder) if f.endswith('.tally')) if os.path.isdir(folder) else []
    bad = [f for f in sorted(set(want) | set(have))
           if f not in want or f not in have or open(os.path.join(folder, f), encoding='utf-8').read() != want[f]]
    if check:
        if bad:
            print('to_tally: %d record parts differ from ledger.json: %s' % (len(bad), ' '.join(bad[:5])))
            sys.exit(1)
        print('to_tally: %d record parts, all ledger.json\'s' % len(want))
        return
    os.makedirs(folder, exist_ok=True)
    for f, s in want.items():
        open(os.path.join(folder, f), 'w', encoding='utf-8').write(s)
    print('to_tally: wrote %d record parts, %d rows' % (len(want), sum(s.count('\n  bank[') for s in want.values())))


if __name__ == '__main__':
    main()
