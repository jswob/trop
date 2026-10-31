# W2 scratch: writes tmp/session8b/drafts/BUILD-ORDER-placement.tsv from the placement rules of the draft's
# section "How a check is placed". Reads engine/tests/checks.tsv, map/tags.tsv, the stored baselines and the
# checks' own files (a text scan for a few constructs). Prints every raise the scan makes, for review.
import csv, os, re, sys
root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../..'))
T = os.path.join(root, 'engine/tests')
ORDER = ['S', 'M0', 'M1', 'M2', 'M3', 'M4', 'M5a', 'M5b', 'M5c', 'M6', 'M7', 'M8', 'M9']
rank = {m: i for i, m in enumerate(ORDER)}
def mx(*ms): return max(ms, key=lambda m: rank[m])

EDGE = {}
for e in 'NUM-01 NUM-02 NUM-03 NUM-04 NUM-05 NUM-06 NUM-07 TIME-01 TIME-02 TIME-03 TIME-04 TIME-05 TIME-06 TIME-07 TIME-08 PTY-01 OWN-01 OWN-02 OWN-03 OWN-08 FLOW-01 FLOW-02 VAL-01 VAL-02 VAL-03 VAL-04 VAL-06 VAL-07 LAW-01 LAW-03 LAW-05 LAW-06 LAW-08 LAW-13 LIFE-01 LIFE-02 LIFE-03 LIFE-04 LIFE-05 RULE-01 RULE-02 RULE-03 RULE-04 RULE-05 SCEN-01 SCEN-03 SCEN-04 ASK-01 ASK-03 ASK-04 ASK-05 SURF-01 SURF-02 SURF-04 SURF-05 SURF-07'.split(): EDGE[e] = 'M1'
for e in 'VAL-05 LAW-04 RULE-06 RULE-07 RULE-08 REC-01 REC-02 REC-03 REC-04 REC-05 REC-06 REC-07 SCEN-02 SURF-06'.split(): EDGE[e] = 'M2'
for e in 'CUR-01 CUR-02 CUR-03 LAW-02 LAW-07 LAW-10 LAW-11 LAW-12'.split(): EDGE[e] = 'M4'
for e in 'PTY-02 PTY-03 PTY-04 OWN-04 OWN-05 OWN-06 OWN-07 OWN-09 FLOW-03 LAW-09 ASK-02'.split(): EDGE[e] = 'M5a'
for e in 'OWN-10 OWN-11 OWN-12'.split(): EDGE[e] = 'M5b'
EDGE['PTY-05'] = 'M5c'
for e in 'LAW-20 RAND-02'.split(): EDGE[e] = 'M6'
for e in 'RAND-01 RAND-03 RAND-04 RULE-09 SURF-03'.split(): EDGE[e] = 'M7'
# REC-06 is M1: knowledge dates on versions are read by every X change (map/tags.tsv I:X:1..I:X:6)
EDGE['REC-06'] = 'M1'

SCAN = [('fill', r'^fill\b', 'M5a'), ('parts', r'^\s+(owned by|batched)\b', 'M5b'), ('use', r'^\s+use \w+ =', 'M5c'),
        ('fx', r'\bat fx\b|\bfx\b', 'M4'),
        ('transcendental', r'\b(ln|exp|sqrt|pdf|cdf|pmf|crps)\s*\(|\bto u\b', 'M6'),
        ('paths', r'\bpaths\s+\d', 'M7')]
def scan(files):
    hit = {}
    for f in files.split():
        p = os.path.join(root, f)
        if not (os.path.isfile(p) and p.endswith('.tally')): continue
        for line in open(p, errors='replace'):
            line = re.sub(r'"[^"]*"', '""', line).split('#')[0]
            for n, rx, m in SCAN:
                if re.search(rx, line): hit[n] = m
    return hit

M101 = {'M1': 'UL-01 UL-02 UL-05', 'M2': 'LH-05 LH-14 RC-08 UL-06 UL-09 UL-15 CH-04',
        'M4': 'CB-01 CB-02 CB-03 CB-04 CB-06 CB-07 CB-08 CB-11 CB-12 LH-07 LH-08 LH-12 LH-13 MP-05 MP-06 MP-07 MP-08 MP-10 MP-11 MP-13 MP-16 RC-02 RC-04 RC-05 RC-07 UL-03 UL-04 UL-10 UL-11 UL-12 UL-14 CH-05 MP-14 CMP-02',
        'M5a': 'CB-05 CB-09 CB-10 CB-13 CB-14 LH-01 LH-02 LH-03 LH-04 LH-06 LH-09 MP-01 MP-02 MP-03 MP-04 MP-09 MP-12 MP-15 RC-01 RC-03 RC-06 UL-13 CMP-01',
        'M6': 'UL-07 UL-08 LH-10 LH-11 LH-15 LH-16 CH-01 CH-02 CH-03 LH-17 LH-18 CH-06'}
CASE = {c: m for m, cs in M101.items() for c in cs.split()}

tags = {r['id']: r['edges'].split() for r in csv.DictReader(open(os.path.join(T, 'map/tags.tsv')), delimiter='\t')}
rows = list(csv.DictReader(open(os.path.join(T, 'checks.tsv')), delimiter='\t'))

def load_refusal(r):
    b = r['baseline']
    p = os.path.join(root, b) if b else ''
    if not (b and os.path.isfile(p)): return None
    t = [l for l in open(p, errors='replace').read().splitlines() if l.strip()]
    if len(t) == 1 and re.match(r'refused: LOAD-[A-Z-]+', t[0]): return t[0].split()[1]
    return None

FLOOR = {'A': ('M5b', "A's places keep parts"), 'B': ('M5a', "B's tax reads a fill"), 'C': ('M4', "C's law parts"),
         'D': ('M4', "D's law parts"), 'E1': ('M5c', 'a joint model of A, B, D'), 'E2': ('M5c', 'a joint model of A and D')}
XRUN = {'b_r1': ('S', 'X base replay: the tracer'), 'b_r1s': ('S', 'X base replay: the tracer'),
        'c1_fast': ('M1', 'X change 1'), 'c2_fast': ('M1', 'X change 2'), 'c4_fast': ('M1', 'X change 4'),
        'c6_fast': ('M1', 'X change 6'), 'c7_fast': ('M1', 'X change 7, a scenario'), 'v4_fast': ('M1', 'X change 4, variant'),
        'c3_fast': ('M2', 'X change 3, retroactive law'), 'v3_fast': ('M2', 'X change 3, variant'),
        'c5_fast': ('M4', 'X change 5, FX'), 'v5_fast': ('M4', 'X change 5, variant')}
out, raises = [], []
for r in rows:
    i, note, q = r['id'], r['note'], r['question']
    if note.startswith('not-run:'):
        m = re.search(r'load (L:\d+)', note) or re.search(r'benchmarks\.md rows? (\d+(?:, \d+)*)', note)
        if m: rowid = m.group(1) if m.group(1).startswith('L:') else ', '.join('L:' + x.strip() for x in m.group(1).split(','))
        elif 'K3 cut' in note: rowid = 'L:3'
        elif 'load 4.5c' in note: rowid = 'L:16'
        elif 'K13' in note: rowid = 'L:12'
        elif 'A-13' in i: rowid = 'L:17'
        else: raise SystemExit('no load row for ' + i)
        out.append((i, 'M9', 'not run: load row ' + rowid + ', judged at full size')); continue
    if i == 'R:all': out.append((i, 'S', "refcore's own tests: the oracle, kept green")); continue
    if i.startswith('U:'):
        lr = load_refusal(r)
        if lr: out.append((i, 'M0', 'a load refusal, ' + lr)); continue
        es = re.findall(r'[A-Z]+-\d+', i.split('~')[0])
        es = [e for e in es if e in EDGE] or []
        m = mx(*[EDGE[e] for e in es]) if es else 'M1'
        why = 'edge ' + '+'.join(dict.fromkeys(es))
        s = scan(r['files'])
        for n, sm in s.items():
            if rank[sm] > rank[m]: raises.append((i, m, sm, n)); m = sm; why += ', ' + n
        out.append((i, m, why)); continue
    if i.startswith('I:M:'):
        c = i[4:].split('~')[0]
        m = CASE[c]; why = 'case ' + c + (' (key script, no engine run)' if '~key' in i else '')
        out.append((i, m, why)); continue
    if i.startswith('I:light/'):
        if 'LH-17' in i: out.append((i, 'M7', 'LH-17-mc cut-down: paths'))
        else: out.append((i, 'M6', 'case ' + ('LH-15' if 'LH-15' in i else 'LH-16') + ' cut-down'))
        continue
    if i.startswith('I:X:'):
        run = i.split('@')[1] if '@' in i else ''
        if run in XRUN: m, why = XRUN[run]
        elif run.startswith('min_'):
            m, why = 'M1', 'X minimal text ' + run[4:]
            for n, sm in scan(r['files']).items():
                if rank[sm] > rank[m]: raises.append((i, m, sm, n)); m = sm; why += ', ' + n
        elif i == 'I:X:q1': m, why = 'M5a', 'card I-10: shares that change, a shared ledger (OWN-04, OWN-05, FLOW-03)'
        else: raise SystemExit('unplaced X ' + i)
        out.append((i, m, why)); continue
    mh = re.match(r'I:(A|B|C|D|E1|E2|S\d):', i)
    if mh:
        h = mh.group(1)
        files = r['files']
        if h.startswith('S'):
            m, why = 'M1', '2.1 blind card ' + h
        else:
            m, why = FLOOR[h]
            if h == 'E1' and 'cuts/E1-02-B/' in files: m, why = FLOOR['B'][0], "B alone (E1-02's B side): " + FLOOR['B'][1]
            if h == 'E2' and 'households/C/' in files: m, why = FLOOR['C'][0], "C alone (E2's C side): " + FLOOR['C'][1]
        qq = q.split(',')[0].strip()
        if qq == 'O:B-3': qq = 'B-12'
        qq = re.sub(r'x$', '', qq)
        if qq.endswith('P'): qq = qq[:-1]
        es = [e for e in tags.get('I:%s:%s' % (h, qq), []) if e != 'PTY-05']
        for e in es:
            if rank[EDGE[e]] > rank[m]: m = EDGE[e]; why += ', edge ' + e
        for n, sm in scan(files).items():
            if n in ('parts', 'use') and rank[sm] > rank[m]: raises.append((i, m, sm, n)); m = sm; why += ', ' + n
        if 'xfail:' in note or 'known-fault' in note: why += ' (known fault KF-01)'
        out.append((i, m, why)); continue
    if i.startswith('I:hardset/'):
        nm = 'I-12' if 'b2fix' in i else re.search(r'(I-\d+)', i).group(1)
        key = [k for k in tags if k.startswith('I:hardset/') and nm + '/' in k]
        es = [e for k in key for e in tags[k] if e != 'PTY-05']
        m = mx('M1', *[EDGE[e] for e in es]); why = 'blind text ' + nm + ' by its edges'
        for n, sm in scan(r['files']).items():
            if rank[sm] > rank[m]: raises.append((i, m, sm, n)); m = sm; why += ', ' + n
        out.append((i, m, why)); continue
    raise SystemExit('unplaced ' + i)

with open(os.path.join(root, 'tmp/session8b/drafts/BUILD-ORDER-placement.tsv'), 'w') as f:
    f.write('id\tmilestone\twhy\n')
    for i, m, w in out: f.write('%s\t%s\t%s\n' % (i, m, w))
for x in raises: print('raise', *x)
from collections import Counter
c = Counter(m for _, m, _ in out)
print(' '.join('%s=%d' % (m, c[m]) for m in ORDER), 'total', sum(c.values()))
