"""Build rows/<chunk>.settled.md from the writer's rows, the judge's verdicts and
the supervisor's decisions.

Usage: python3 -I -B settle.py <chunk> [--decide decisions.tsv] [--renumber S:101=S:4 ...]
decisions.tsv: one line per overridden row, tab-separated:
  row  status  reason  cause
(cause: writer misread | judge misread | spec unclear | case suspect | supervisor;
an empty reason keeps the writer's). Rows the judge confirmed get "confirmed by
judge"; rows no judge saw get "not judged (writer's status)".
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def table(path):
    rows = {}
    order = []
    for line in open(path, encoding='utf-8'):
        if not line.lstrip().startswith('|'):
            continue
        c = [x.strip() for x in line.strip().strip('|').split('|')]
        rid = c[0].strip('`* ')
        if re.match(r'^[PQOS]:', rid):
            rows[rid] = c
            order.append(rid)
    return rows, order


def main():
    chunk = sys.argv[1]
    args = sys.argv[2:]
    decide, renum = {}, {}
    while args:
        a = args.pop(0)
        if a == '--decide':
            for line in open(args.pop(0), encoding='utf-8'):
                if line.strip() and not line.startswith('#'):
                    f = line.rstrip('\n').split('\t')
                    decide[f[0]] = f[1:]
        elif a == '--renumber':
            while args and not args[0].startswith('--'):
                old, new = args.pop(0).split('=')
                renum[old] = new
    writer, order = table(os.path.join(HERE, 'rows', chunk + '.writer.md'))
    # rows the writer's message left out, recovered by the supervisor from its map
    rpath = os.path.join(HERE, 'rows', chunk + '.recovered.md')
    if os.path.exists(rpath):
        rec, rorder = table(rpath)
        writer.update(rec)
        order += [r for r in rorder if r not in order]
    jpath = os.path.join(HERE, 'rows', chunk + '.judge.md')
    judge = table(jpath)[0] if os.path.exists(jpath) else {}
    out = ['# %s settled' % chunk, '',
           'Writer rows, judge verdicts and the supervisor\'s settlements (brief §7). The last column is the settlement.', '',
           '| Row | Status | Reason | Spec | Gap | Text | Settlement |', '|---|---|---|---|---|---|---|']
    for rid in order:
        c = list(writer[rid][:6])
        while len(c) < 6:
            c.append('')
        how = 'not judged (writer\'s status)'
        if rid in judge:
            v = judge[rid][1].lower()
            how = 'confirmed by judge' if v.startswith('confirm') else 'judge: ' + v
        if rid in decide:
            st, reason, cause = (decide[rid] + ['', '', ''])[:3]
            c[1] = st
            if reason:
                c[2] = reason
            how = (how + '; ' if how.startswith('judge') else '') + 'settled: ' + cause
        elif how.startswith('judge'):
            sys.exit('%s: judge verdict %r has no decision' % (rid, how))
        for old, new in renum.items():
            c = [re.sub(re.escape(old) + r'(?!\d)', new, x) for x in c]
        rid2 = c[0].strip('`')
        c[0] = '`%s`' % rid2
        out.append('| ' + ' | '.join(c + [how]) + ' |')
    for rid in decide:
        if rid not in writer:
            sys.exit('decision for unknown row ' + rid)
    path = os.path.join(HERE, 'rows', chunk + '.settled.md')
    open(path, 'w', encoding='utf-8').write('\n'.join(out) + '\n')
    print('wrote', path, len(order), 'rows')


if __name__ == '__main__':
    main()
