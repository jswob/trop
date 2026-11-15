"""R7's long-horizon pins, checked against the prototype's own figures (issue #114; the coordinator's pick P45;
GOLDENS.md § P2). Standard library only.

refcore can't run R7 past 2031 within the 8 GB cap (P2: each month's 15th-rule projection keeps a walk of its own),
so these pins have no refcore bytes. The prototype is an independent oracle: `pins_long.tsv` holds its figure for
each (`engine/spec/verification/blocks/pins-1-7.json`, the key in `prototype`), written in the engine's money form,
on the joint-return twin (`r7/joint.tally`), where L-26 doesn't move it. Nothing is hand-worked.

    id  header  ask  item  expected  prototype  hold  why

A row's run is the bank half and R7's forward half under its horizon's header, as `goldens/r7_joint_*` run
(m2/vectors.tsv): `<header> bank.tally r7/forward.tally r7/business.tally r7/joint.tally records/*.tally --ask <ask>`,
in this folder. The figure is the ask's value on its first line, the `item`-th of a tuple (from 0) or the whole value
(`-`). A row held "for #n" (`hold`) is printed and not counted.

These runs are engine benchmarks, not checks of `engine/tools/check`: on 2026-11-15 the quick profile took 4 s and
2.1 GB to 2036, 80 s and 12.1 GB to 2050, 103 s and 12.5 GB to 2056, past the 8 GB a check may take. The gate's
timing run uses them (GOLDENS.md names the three horizons). P54: nobody runs this on the Mac until the engine's
runs of the three horizons fit in 8 GB (G6's word); memcap.py now stops an engine run past the cap in any profile.

    python3 -I -B engine/tests/m2/bank/pins_long.py --engine PATH   -> one line per pin; exit 1 if one unheld differs
    python3 -I -B engine/tests/m2/bank/pins_long.py --check         -> the table's shape only (no run)
"""
import csv
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
COLUMNS = ['id', 'header', 'ask', 'item', 'expected', 'prototype', 'hold', 'why']
TAIL = ['bank.tally', 'r7/forward.tally', 'r7/business.tally', 'r7/joint.tally']


def rows():
    with open(os.path.join(HERE, 'pins_long.tsv'), encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        if r.fieldnames != COLUMNS:
            sys.exit('pins_long.tsv: the header is not ' + ' '.join(COLUMNS))
        out = list(r)
    for v in out:
        if not all(v[k] for k in COLUMNS) or not (v['item'] == '-' or v['item'].isdigit()):
            sys.exit('pins_long.tsv: an incomplete row: ' + v['id'])
        if v['hold'] != '-' and not v['hold'].startswith('#'):
            sys.exit('pins_long.tsv: hold is - or #n: ' + v['id'])
        if not os.path.exists(os.path.join(HERE, v['header'])):
            sys.exit('pins_long.tsv: no header ' + v['header'])
    return out


def value_of(out, ask, item):
    """The ask's value on its first line: the whole, or the item-th of a tuple `(a, b, …)`."""
    for line in out.split('\n'):
        if line.startswith(ask + ': '):
            value = line[len(ask) + 2:]
            if item == '-':
                return value
            if not (value.startswith('(') and value.endswith(')')):
                return None
            parts = value[1:-1].split(', ')
            k = int(item)
            return parts[k] if k < len(parts) else None
    return None


def main(argv):
    vs = rows()
    if '--check' in argv:
        print('pins_long.tsv: %d pins' % len(vs))
        return 0
    if '--engine' not in argv or argv.index('--engine') + 1 >= len(argv):
        sys.exit(__doc__)
    engine = os.path.abspath(argv[argv.index('--engine') + 1])
    records = sorted('records/' + f for f in os.listdir(os.path.join(HERE, 'records')) if f.endswith('.tally'))
    runs, bad = {}, 0
    for v in vs:
        key = (v['header'], v['ask'])
        if key not in runs:
            p = subprocess.run([engine, 'run', v['header']] + TAIL + records + ['--ask', v['ask']], cwd=HERE,
                               capture_output=True, text=True)
            runs[key] = p.stdout + p.stderr
        got = value_of(runs[key], v['ask'], v['item'])
        same = got == v['expected']
        state = 'holds' if same else ('held for %s' % v['hold'] if v['hold'] != '-' else 'DIFFERS')
        bad += 0 if same or v['hold'] != '-' else 1
        print('%-28s %-10s expected %s, engine %s' % (v['id'], state, v['expected'], got))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
